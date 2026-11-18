//! A small test app for the Android backend: a counter with two buttons a finger taps, a bar
//! that grows with the count, and a light/dark switch that starts from the phone's setting.

#[cfg(target_os = "android")]
use rune::AppSpec;
use rune::tok::{self, type_};
use rune::{Cx, El, TextStyle, col, el, row, text};

#[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
#[derive(Default)]
struct State {
    count: i32,
    flip_theme: bool,
}

#[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
fn button(key: &'static str, label: &'static str, dark: bool, on: fn(&mut State)) -> El {
    el()
        .key(key)
        .role("button")
        .label(label)
        .focusable(true)
        .size(120.0, 48.0)
        .radius(12.0)
        .fill(tok::accent(0, dark))
        .center()
        .on_press(on)
        .child(text(label).font(TextStyle::new(18.0, 600.0)).color(rune::oklch(1.0, 0.0, 0.0)))
}

#[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
fn view(s: &State, cx: &mut Cx<State>) -> El {
    let dark = cx.system_dark() ^ s.flip_theme;
    let t = tok::theme(dark);
    let fill = (s.count.clamp(0, 10) as f32) * 28.0;
    let knob = el()
        .key("knob")
        .size(18.0, 18.0)
        .radius(9.0)
        .fill(t.knob)
        .absolute(if dark { 18.0 } else { 2.0 }, 2.0);
    let switch = el()
        .key("theme")
        .role("switch")
        .label("Dark mode")
        .checked(dark)
        .focusable(true)
        .size(38.0, 22.0)
        .radius(11.0)
        .fill(if dark { tok::accent(4, dark) } else { t.control_off })
        .on_press(|s: &mut State| s.flip_theme = !s.flip_theme)
        .child(knob);
    let card = col()
        .key("card")
        .w_full()
        .padding(24.0)
        .gap(20.0)
        .radius(20.0)
        .fill(t.card)
        .center()
        .child(text("Rune on Android").font(TextStyle::new(22.0, 700.0)).color(t.text))
        .child(text(format!("{}", s.count)).font(TextStyle::new(64.0, 700.0)).color(t.text))
        .child(
            el().key("track").size(280.0, 10.0).radius(5.0).fill(t.control_track).child(
                el().key("bar").size(fill, 10.0).radius(5.0).fill(tok::accent(1, dark)),
            ),
        )
        .child(
            row()
                .gap(16.0)
                .child(button("minus", "− 1", dark, |s| s.count -= 1))
                .child(button("plus", "+ 1", dark, |s| s.count += 1)),
        )
        .child(
            row()
                .gap(12.0)
                .center()
                .child(text("Dark mode").font(type_::BODY).color(t.text_secondary))
                .child(switch),
        );
    col()
        .w_full()
        .h_full()
        .center()
        .padding(24.0)
        .fill(t.bg)
        .child(card)
}

/// `NativeActivity` calls this: hand rune the activity, then run the same app a desktop would.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: rune::android::AndroidApp) {
    rune::android::bind(app);
    rune::run(AppSpec::new("Demo", (360.0, 720.0), State::default, view));
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn taps_count() {
        let env = Env { system_dark: false, scale: 2.5 };
        let mut rt = Runtime::new(State::default(), view, (360.0, 720.0), env);
        rt.frame(0.0);
        let (x, y) = rt.find("plus").unwrap();
        for i in 0..3 {
            let t = i as f64 * 0.2;
            rt.touch(1, rune::TouchPhase::Started, x, y, t);
            rt.touch(1, rune::TouchPhase::Ended, x, y, t + 0.05);
            rt.frame(t + 0.1);
        }
        assert_eq!(rt.state.count, 3);
    }
}
