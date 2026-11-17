//! The Android activity winit's event loop has to be built with.
//!
//! An app exports `android_main`, passes the activity here, and calls [`crate::run`]. The
//! loop then follows the activity's native window (`android_lifecycle`): the window and its
//! wgpu surface exist only between `resumed` and `suspended`. See `research/40-android.md`.

use std::sync::Mutex;

pub use winit::platform::android::activity::AndroidApp;

static APP: Mutex<Option<AndroidApp>> = Mutex::new(None);

/// Remember the activity `android_main` was given. The next [`crate::run`] builds its event
/// loop with it. A later call replaces the previous one, so a process that is entered again
/// uses the new activity.
pub fn bind(app: AndroidApp) {
    *APP.lock().unwrap_or_else(|e| e.into_inner()) = Some(app);
}

pub(crate) fn app() -> Option<AndroidApp> {
    APP.lock().unwrap_or_else(|e| e.into_inner()).clone()
}
