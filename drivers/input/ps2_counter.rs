#![no_std]
#![feature(allocator_api, global_asm, asm)]
#![feature(test)]

use kernel::prelude::*;
use kernel::timer;
use kernel::c_types::c_void;
use kernel::timer::{Timer, TimerCallback};
use kernel::bindings::{
    request_threaded_irq, free_irq,
    irqreturn_t,
    irqreturn_IRQ_HANDLED as IRQ_HANDLED,
    irqreturn_IRQ_NONE as IRQ_NONE,
    IRQF_SHARED,
    HZ
};
use alloc::boxed::Box;
use core::sync::atomic::{
    AtomicUsize,
    AtomicU64,
    Ordering
};
use core::pin::Pin;

module! {
    type: Ps2Counter,
    name: b"ps2_counter",
    author: b"Ilya Konnov",
    description: b"Simple module that counts number of PS/2 keypresses",
    license: b"GPL v2",
    params: {},
}

// I'm not sure is reading directly from bindings::jiffies_64 will be really volatile.
// Also we can't use get_jiffies_u64, since it is inlined.
// So let's hope, that our machine can read u64 atomically.
fn jiffies() -> u64 {
    unsafe {
        core::ptr::read_volatile(&kernel::bindings::jiffies_64 as *const u64)
    }
}

// https://elixir.bootlin.com/linux/v5.11.9/source/arch/x86/boot/boot.h#L43
// https://c9x.me/x86/html/file_module_x86_id_139.html
fn inb(port: u16) -> u8 {
    unsafe {
        let v: u8;
        asm!("in {}, dx", out(reg_byte) v, in("dx") port);
        v
    }
}

// https://elixir.bootlin.com/linux/v5.11.9/source/arch/x86/boot/boot.h#L39
// https://c9x.me/x86/html/file_module_x86_id_222.html
fn outb(v: u8, port: u16) -> u8 {
    unsafe {
        asm!("out dx, {}", in(reg_byte) v, in("dx") port);
        v
    }
}

struct CounterData {
    counter: AtomicUsize,
    last_printed: AtomicU64,
}

static COUNTER_INSTANCE: CounterData = CounterData::new();

const DELAY: u64 = 10 * (HZ as u64);

impl CounterData {
    const fn new() -> Self {
        CounterData {
            counter: AtomicUsize::new(0),
            // FIXME: It should be initial jiffies value
            last_printed: AtomicU64::new(0),
        }
    }

    fn handle_key(&self) -> irqreturn_t {
        // Reading scancodes is fun, but that makes keylogger very obvious,
        // since keypresses are not processed by "real" driver.
        /*let scancode = inb(0x60);
        println!("[{:x}]", scancode);*/

        // Relaxed will work fine: https://doc.rust-lang.org/nomicon/atomics.html#relaxed
        COUNTER_INSTANCE.counter.fetch_add(1, Ordering::Relaxed);

        IRQ_HANDLED
    }

    fn get_ptr(&self) -> *const Self {
        self as *const Self
    }

    unsafe extern "C" fn trampoline(irq: i32, cookie: *mut c_void) -> irqreturn_t {
        // It's important to not trust signature, that cookie is mut.
        // We can't crate mutable reference to it, since there is immutable one exist.
        if !core::ptr::eq(COUNTER_INSTANCE.get_ptr(), cookie as *const _) || irq != 1 {
            println!("Something went wrong. Ignoring.");
            IRQ_NONE
        } else {
            COUNTER_INSTANCE.handle_key()
        }
    }
}

struct Callback;

impl TimerCallback for Callback {
    fn invoke(&mut self, timer: Pin<&mut Timer<Self>>) {
        let now: u64 = jiffies();
        let last = COUNTER_INSTANCE.last_printed.load(Ordering::Relaxed);
        let (mut diff, overflowed) = now.overflowing_sub(last);
        if overflowed {
            diff += u64::MAX;
        }
        if diff < DELAY {
            let (until, _) = now.overflowing_add(diff);
            timer.modify(until);
            return;
        }

        let counter = COUNTER_INSTANCE.counter.swap(0, Ordering::SeqCst);
        // Account that PS/2 sends events for keydown and for keyup.
        let counter = counter / 2;
        println!("{} keys pressed", counter);
        COUNTER_INSTANCE.last_printed.store(now, Ordering::Relaxed);

        let (until, _) = now.overflowing_add(DELAY);
        timer.modify(until);
    }
}

struct Ps2Counter {
    _timer: Pin<Box<Timer<'static, Callback>>>
}

// Ps2Counter does about nothing, so we can share this pointer.
unsafe impl Sync for Ps2Counter {}

impl KernelModule for Ps2Counter {
    fn init() -> KernelResult<Self> {
        // Firstly, setup an interrupt handler.
        unsafe {
            let res = request_threaded_irq(
                /* line */ 1,
                /* handler */ Some(CounterData::trampoline),
                /* thread_fn */ None,
                /* irqflags */ IRQF_SHARED as _,
                /* name */ (b"ps2counter\0") as *const _ as *const _,
                /* cookie */ COUNTER_INSTANCE.get_ptr() as *mut _
            );
            if res < 0 { return Err(kernel::Error::from_kernel_errno(res)) }
        }

        // Then initialize timer.
        let timer = timer!(Callback).boxed();

        Ok(Ps2Counter {
            _timer: timer,
        })
    }
}

impl Drop for Ps2Counter {
    fn drop(&mut self) {
        unsafe {
            free_irq(1, COUNTER_INSTANCE.get_ptr() as *mut _);
        }
    }
}
