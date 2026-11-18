# 40 — Android backend

An Android app uses the same window shell as the desktop. Vendored winit 0.30.13 already has an Android backend on `android-activity` 0.6; Rune did not turn it on, and its event loop created the window once and never dropped it. This note is the design for the first slice: one example builds for `aarch64-linux-android`, the activity's surface is created and destroyed with the native window, and touches reach the view. It is not a finished phone shell.

## Decided

**Reuse winit and wgpu.** Desktop and web already run on winit's `ApplicationHandler`. wgpu's Vulkan backend is already what every non-Apple, non-Windows target links, and Android's `ANativeWindow` is a raw window handle winit exposes. A new shell, a new renderer, or a GLES backend would duplicate that. PLAN.md puts an owned shell at M7, after iOS text input, because winit's IME cannot see the document. This slice is the winit shell, the same way the desktop shipped before its own text protocols.

**NativeActivity, not GameActivity.** The activity glue is winit's `android-native-activity` feature, which selects `android-activity`'s `NativeActivity` backend. `NativeActivity` is in the OS, so the APK needs no Java and no Maven artifacts. GameActivity is the better long-term base (AppCompat, `GameTextInput`), and it is rejected for this slice: it needs a Gradle build and AndroidX AARs, and winit 0.30.13 does not read `GameTextInput`'s composition buffer anyway. `set_ime_allowed` on both backends only shows or hides the soft keyboard.

**The app owns `android_main`.** A library must not export `android_main`: the glue crate allows one, and an app may want to run before `rune::run`. The app calls `rune::android::bind(app)` and then `rune::run`, the same entry the desktop uses. `bind` keeps the `AndroidApp` winit requires to build an event loop. There is no `fn main` on Android. The shared library the APK loads is named `main`, which is the name `NativeActivity` loads (`android.app.lib_name`).

**One window, created only while the native window exists.** winit maps `APP_CMD_INIT_WINDOW` to `ApplicationHandler::resumed` and `APP_CMD_TERM_WINDOW` to `suspended`. The native window is null outside that pair, and a wgpu surface held past `suspended` points at a destroyed `ANativeWindow`. The host's response is a small state machine (`platform/android_lifecycle.rs`):

| State | Native window | What Rune holds |
| --- | --- | --- |
| `Absent` | no | no winit window was showing |
| `Live` | yes | a winit window may exist |
| `Suspended` | no | a window was showing and must be opened again |

`Created` while `Absent` or `Suspended` enters `Live`. It restores the window only from `Suspended`. `Destroyed` while `Live` drops the winit window and its wgpu surface, keeps the `Gpu` and the `Runtime`, and remembers `Suspended` only when the window was on screen. A window the app had hidden stays hidden. A later `Created` calls the same `open` path as the first show, which builds a new surface on the device it kept.

The first `resumed` still builds the hosts and opens the launch window. Later `resumed` events, which the desktop loop ignores, are delivered to those hosts. `suspended` is implemented only on Android, so desktop loops are unchanged.

A finger that is down when the surface is destroyed is cancelled, so the press does not stay held until a touch that will never arrive.

**Touch is the existing path.** winit delivers `WindowEvent::Touch` in physical pixels. `Host` already divides by `Runtime::input_scale` and calls `Runtime::touch`. On Android that scale is winit's `scale_factor`, which is the display density in dpi divided by 160. A resize uses the same scale. The activity does not lock orientation.

**The soft keyboard is show and hide only.** Focusing a field already calls `Window::set_ime_allowed`. On Android that calls `show_soft_input` / `hide_soft_input`. Key events arrive as `WindowEvent::KeyboardInput`, including a character when the key event has one. `set_ime_cursor_area` and `set_ime_purpose` are empty in this winit. Marked text, suggestions, and the caret rect are not implemented. That is the same limit as winit's iOS `UIKeyInput`, and it is why an owned `InputConnection` stays future work rather than a patch to winit.

**Back closes the activity.** winit reported every key but the volume keys as handled, so Back never reached `NativeActivity` and an app could not be left with it. The patch leaves Back unhandled and does not deliver it to the app, so the activity finishes, `onDestroy` ends the event loop, and `android_main` returns. An app that wants Back for its own navigation needs a way to claim it first; that is not built.

**Packaging.** `examples/android-hello` is a `cdylib` (and an `rlib`, so its tests run on the desktop). `cargo build -p android-hello --target aarch64-linux-android` is the build this slice has to pass. An APK is `NativeActivity` plus that library. The manifest handles orientation, screen size, density, and keyboard changes itself, so Android does not tear down the process and call `android_main` again with a new `AndroidApp`. Minimum SDK 29: that is the first release where Vulkan 1.1 is required, which is the backend this build links. Vulkan is a required feature of the APK.

**Fonts: the NDK's font matcher.** Text uses the fonts the app registers and, with the default `embedded-fonts` feature, the bundled Inter faces. A character those lack goes to the system, as on the desktop (`text/system_fonts.rs`): Android's platform module (`system_fonts/android.rs`) answers with the NDK font API, which API 29 (the minimum SDK) added. `AFontMatcher` names the font Android itself falls back to for a run of text in a locale; one call covers a run, so a line in one script costs one call. `ASystemFontIterator` lists the files the system's font configuration names without opening them; a family named by the app or by Rune (`Noto Color Emoji`, `Droid Sans Mono`) is matched to its files by the family stem of the file name (`NotoSansArabic-Bold.ttf` is `notosansarabic`), and a matched font brings its siblings, so a heading finds the Bold. This is Skia's `SkFontMgr_AndroidNDK`. fontique's own Android backend was not used: it opens and parses every file under `/system/fonts` when it starts, and Rune walks no font folder.

**Light and dark follow the configuration.** winit's Android `Window::theme` returned `None` and `ConfigChanged` never sent `ThemeChanged`, so every app opened dark. `vendor/winit.patch` reads the theme from the configuration's night mode and sends `ThemeChanged` when it changes. The night mode comes from the activity's `Resources` over JNI (`getConfiguration().uiMode`): `AndroidApp::config()` is copied from the `AssetManager` the activity started with, and a night-mode change gives the running activity new resources instead of updating that one, so the copy kept the launch value and a live switch never arrived. The density in that copy may be stale the same way; a Display size change has not been checked. The event loop keeps the night mode and density it last reported, because `AndroidApp::config()` is shared: winit's old handler compared the new scale with a second read of the same configuration, so it sent `ScaleFactorChanged` on every configuration change (keyboard, night mode), always with the current scale. It now sends it only when the density changes. A change made while the app is in the background, when Rune has no window, is read when the window opens again (`window_dark`).

Clipboard, menus, file dialogs, overlays, and AccessKit stay the stubs those platforms already have: there is no Android adapter here.

## Rejected

- **A Rune-owned `ANativeActivity` shell.** It would reimplement the looper, input, and window-handle glue winit already maintains in-tree. Worth it when `InputConnection` needs document context, not before the window draws.
- **GameActivity for this PR.** Better text input only if we also write the `GameTextInput` bridge. winit does not. The packaging cost is paid for nothing this slice can use.
- **`cargo-apk` as the only build.** It is a reasonable packager and the example's manifest matches it, but the thing CI and a clean checkout can do without the Android SDK is `cargo build --target aarch64-linux-android`. `cargo-ndk` is the documented way to point that build at an NDK. A Gradle project is not checked in.
- **GLES as a second backend.** Devices below API 29, and some broken Vulkan drivers, would need it. Rune's non-Apple desktop path is Vulkan. Adding ANGLE or wgpu's GLES feature is a separate backend choice.
- **Safe-area insets in this slice.** winit 0.30 logs `InsetsChanged` and does not report it. Inventing a second JNI query beside winit's would be a new platform API (`AppSpec` and friends are literal-built, so it would be a builder). Left until a device shows the status bar and the IME covering the view.

## Needs validation on a device or emulator

Not run here: there is no emulator, no phone, and no check that a frame was presented.

- The first `inner_size` after `resumed` is non-zero, and a rotation arrives as `Resized` with the new density rather than a new process.
- Vulkan presents on an `ANativeWindow` (`B8G8R8A8` or whatever the device reports) at the density winit reports, including a density change (Settings → Display size), which arrives as `ScaleFactorChanged` and then `Resized`.
- Arabic, Devanagari, Japanese and emoji in the example draw in the system's fonts, and switching the phone between light and dark switches the example, in the foreground and from the background.
- A touch lands on the view in points, and a second finger does not synthesize a second press. winit does not emit mouse events for Android touches; the host's "ignore mouse while touching" guard still applies.
- `show_soft_input` opens the keyboard when a field is focused, and a Latin key inserts a character. CJK composition will not work until an `InputConnection`.
- Suspending (Home, or the system destroying the surface) and returning keeps app state, drops the surface, and draws again. The GPU device is reused.
- Status-bar, navigation-bar, and IME insets. The view is currently laid out in the full native window. With `targetSdkVersion` 35 Android 15 draws every app edge to edge, so the status bar is expected to cover the top of the view until Rune reads the insets.
- `minSdkVersion` 29 and `targetSdkVersion` 35 against a current SDK and a 120 Hz panel. Present mode is the desktop default, `AutoVsync`. Frame pacing against a 120 Hz display is unmeasured.
- Whether `prewarm_gpu` on the activity thread, before the loop, is slow enough to matter. It is the desktop path.

## Build

The NDK's LLVM clang is the linker. `cargo-ndk` puts it on `PATH` and sets the sysroot. With the NDK unpacked and `ANDROID_NDK_HOME` set:

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a build -p android-hello --release
```

Without `cargo-ndk`, the same build is `cargo build -p android-hello --target aarch64-linux-android` once the target's linker is the NDK clang for the minimum SDK, `aarch64-linux-android29-clang` from `$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin`. Linking against a newer API level would let a call that API 29 lacks through to the phone.

The shared library is `libmain.so`. An APK is that library inside `lib/arm64-v8a/`, with `examples/android-hello/AndroidManifest.xml`. `cargo apk build -p android-hello --target aarch64-linux-android` does that when the Android SDK build-tools are installed (`cargo install cargo-apk`). Install with `adb install` and run the `dev.rune.android_hello` activity. stdout from the native code shows up in logcat.

CI (`.github/workflows/android.yml`) does that build on Linux with the runner's NDK at API 29, per push to `dev`, on request, and per pull request touching the Android-only files, and checks that `libmain.so` exports `ANativeActivity_onCreate`. It does not run the app.

What has been checked is the compile, the lifecycle machine's and the font module's tests, and the desktop test suite. What has not been checked is a running activity.

## Next

- **The system's languages.** `rune-i18n` reads the POSIX locale variables, which Android does not set, so an app's catalog is always the fallback language. The activity's configuration has the primary locale; the full list (`LocaleList`) and a per-app language (API 33) need JNI.
- **Insets**, above. Needed as soon as `targetSdkVersion` is 35.
- **Clipboard.** `ClipboardManager` is Java-only, so it needs JNI through the activity's `JavaVM`.
