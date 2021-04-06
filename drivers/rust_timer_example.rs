diff --git a/drivers/rust_timer_example.rs b/drivers/rust_timer_example.rs
index d5ca7f54ec21..6ae065549d16 100644
--- a/drivers/rust_timer_example.rs
+++ b/drivers/rust_timer_example.rs
@@ -9,8 +9,8 @@
 use alloc::boxed::Box;
 use core::pin::Pin;
 use kernel::prelude::*;
-use kernel::timer::{Timer, TimerCallback};
 use kernel::time::{jiffies, HZ};
+use kernel::timer::{Timer, TimerCallback};
 
 module! {
     type: RustTimerExample,
@@ -22,11 +22,11 @@ module! {
 }
 
 struct MyHandler {
-    message: String
+    message: String,
 }
 
 impl TimerCallback for MyHandler {
-    fn invoke(&mut self) {
+    fn invoke(&mut self, _timer: Pin<&mut Timer<'_, Self>>) {
         println!("Hello from static timer. {}", self.message)
     }
 }
@@ -37,7 +37,7 @@ fn static_callback() {
 
 struct RustTimerExample {
     boxed_timer: Pin<Box<Timer<'static, MyHandler>>>,
-    static_timer: Pin<&'static mut Timer<'static, fn()>>
+    static_timer: Pin<&'static mut Timer<'static, fn()>>,
 }
 
 impl KernelModule for RustTimerExample {
@@ -58,14 +58,13 @@ impl KernelModule for RustTimerExample {
             builder.in_option(&mut TIMER)
         };
 
-        boxed_timer.as_mut().modify(jiffies().wrapping_add(3*HZ));
-        static_timer.as_mut().modify(jiffies().wrapping_add(5*HZ));
+        boxed_timer.as_mut().modify(jiffies().wrapping_add(3 * HZ));
+        static_timer.as_mut().modify(jiffies().wrapping_add(5 * HZ));
         println!("Timers example initialized!");
 
         Ok(RustTimerExample {
             boxed_timer,
-            static_timer
+            static_timer,
         })
     }
 }
-
