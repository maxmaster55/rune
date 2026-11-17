//! The Android activity's native window, as far as a host needs it.
//!
//! winit sends `ApplicationHandler::resumed` for `APP_CMD_INIT_WINDOW` and `suspended` for
//! `APP_CMD_TERM_WINDOW`. The `ANativeWindow` is null outside that pair, so a wgpu surface
//! must not outlive `suspended`, and a window that was showing has to be opened again on the
//! next `resumed`. The GPU device and the runtime stay; only the window and its surface go.

use crate::machine::Effects;

/// Whether the activity has a native window, and whether a Rune window should come back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Surface {
    /// No native window, and nothing was showing.
    #[default]
    Absent,
    /// A native window exists. The host may create a winit window.
    Live,
    /// The native window was destroyed while a window was showing. The next one restores it.
    Suspended,
}

/// What the activity did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    /// `InitWindow`. winit delivers this as `resumed`.
    Created,
    /// `TermWindow`. winit delivers this as `suspended`. `showing` is whether the window
    /// was on screen, so a window the app had hidden stays hidden.
    Destroyed { showing: bool },
}

/// What the host does. It drops or opens the winit window; the machine never touches one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    /// The native window is gone. Drop the winit window and its wgpu surface.
    DropWindow,
    /// A window was showing when the surface was destroyed. Open it again.
    RestoreWindow,
}

pub(crate) fn step(state: &mut Surface, event: Event) -> Effects<Effect> {
    match (*state, event) {
        (Surface::Absent | Surface::Suspended, Event::Created) => {
            let restore = *state == Surface::Suspended;
            *state = Surface::Live;
            if restore {
                Effects::one(Effect::RestoreWindow)
            } else {
                Effects::none()
            }
        }
        (Surface::Live, Event::Destroyed { showing }) => {
            *state = if showing {
                Surface::Suspended
            } else {
                Surface::Absent
            };
            Effects::one(Effect::DropWindow)
        }
        // A second create, or a destroy while no window can exist: nothing changes.
        (Surface::Live, Event::Created)
        | (Surface::Absent | Surface::Suspended, Event::Destroyed { .. }) => Effects::none(),
    }
}

#[cfg(test)]
#[path = "android_lifecycle_tests.rs"]
mod tests;
