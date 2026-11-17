//! The hello example as an Android activity: a switch a finger can tap, and a field that asks
//! for the soft keyboard. The size below is only the first layout, until the surface reports
//! its real size and density.
//!
//! Build and packaging: `research/40-android.md`.

#[cfg(target_os = "android")]
use rune::AppSpec;
use rune::tok::{self, type_};
use rune::{Cx, El, TextEdit, col, el, text, text_input};

#[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
struct State {
    on: bool,
    note: TextEdit,
}

impl Default for State {
    fn default() -> Self {
        Self {
            on: false,
            note: TextEdit::new("Type here"),
        }
    }
}

#[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    let label = text(if s.on { "Lights on" } else { "Lights off" })
        .font(type_::TITLE)
        .color(t.text);
    let knob = el()
        .key("knob")
        .size(18.0, 18.0)
        .radius(9.0)
        .fill(t.knob)
        .absolute(if s.on { 18.0 } else { 2.0 }, 2.0);
    let switch = el()
        .key("switch")
        .role("switch")
        .label("Lights")
        .checked(s.on)
        .focusable(true)
        .size(38.0, 22.0)
        .radius(11.0)
        .fill(if s.on {
            tok::accent(0, true)
        } else {
            t.control_off
        })
        .on_press(|s: &mut State| s.on = !s.on)
        .child(knob);
    let field = text_input(&s.note, type_::BODY)
        .key("note")
        .color(t.text)
        .w_full()
        .single_line()
        .on_edit(|s: &mut State| &mut s.note);
    col()
        .w_full()
        .h_full()
        .center()
        .gap(16.0)
        .padding(24.0)
        .fill(t.bg)
        .child(label)
        .child(switch)
        .child(field)
}

/// `NativeActivity` calls this. Bind the activity, then the same `rune::run` a desktop binary uses.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: rune::android::AndroidApp) {
    rune::android::bind(app);
    rune::run(AppSpec::<State> {
        scenes: &[("on", |s| s.on = true)],
        ..AppSpec::new("Hello", (360.0, 720.0), State::default, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn a_tap_flips_the_switch() {
        let env = Env {
            system_dark: true,
            scale: 2.5,
        };
        let mut rt = Runtime::new(State::default(), view, (360.0, 720.0), env);
        rt.frame(0.0);
        let (x, y) = rt.find("switch").unwrap();
        rt.touch(1, rune::TouchPhase::Started, x, y, 0.0);
        rt.touch(1, rune::TouchPhase::Ended, x, y, 0.05);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.on);
        assert!(!rt.touching());
    }
}
