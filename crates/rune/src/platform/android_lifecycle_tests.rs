//! The activity surface, through the orders winit can report: a create, a destroy while
//! showing, a destroy while hidden, and a second create. A short walk covers the rest.

use super::{Effect, Event, Surface, step};

fn effects(state: &mut Surface, event: Event) -> Vec<Effect> {
    step(state, event).into_iter().collect()
}

#[test]
fn the_first_surface_creates_nothing_until_a_window_was_showing() {
    let mut state = Surface::Absent;
    assert_eq!(effects(&mut state, Event::Created), Vec::<Effect>::new());
    assert_eq!(state, Surface::Live);
    assert_eq!(
        effects(&mut state, Event::Destroyed { showing: true }),
        vec![Effect::DropWindow]
    );
    assert_eq!(state, Surface::Suspended);
    assert_eq!(
        effects(&mut state, Event::Created),
        vec![Effect::RestoreWindow]
    );
    assert_eq!(state, Surface::Live);
}

#[test]
fn a_hidden_window_is_not_restored() {
    let mut state = Surface::Live;
    assert_eq!(
        effects(&mut state, Event::Destroyed { showing: false }),
        vec![Effect::DropWindow]
    );
    assert_eq!(state, Surface::Absent, "a window the app hid stays hidden");
    assert_eq!(effects(&mut state, Event::Created), Vec::<Effect>::new());
    assert_eq!(state, Surface::Live);
}

#[test]
fn a_destroy_with_no_window_and_a_second_create_do_nothing() {
    let mut state = Surface::Absent;
    assert_eq!(
        effects(&mut state, Event::Destroyed { showing: true }),
        Vec::<Effect>::new()
    );
    assert_eq!(state, Surface::Absent);
    let _ = effects(&mut state, Event::Created);
    assert_eq!(effects(&mut state, Event::Created), Vec::<Effect>::new());
    assert_eq!(state, Surface::Live);
}

#[test]
fn every_short_sequence_drops_only_a_live_window_and_restores_only_a_suspended_one() {
    let alphabet = [
        Event::Created,
        Event::Destroyed { showing: true },
        Event::Destroyed { showing: false },
    ];
    let mut seq = Vec::new();
    walk(&alphabet, 4, &mut seq);
}

fn walk(alphabet: &[Event], depth: usize, seq: &mut Vec<Event>) {
    let mut state = Surface::Absent;
    for &event in seq.iter() {
        let before = state;
        let got: Vec<_> = step(&mut state, event).into_iter().collect();
        match event {
            Event::Created => {
                assert!(
                    got.iter().all(|e| *e == Effect::RestoreWindow),
                    "{seq:?} from {before:?}: {got:?}"
                );
                assert_eq!(
                    got.len(),
                    usize::from(before == Surface::Suspended),
                    "{seq:?}"
                );
                assert_eq!(state, Surface::Live, "{seq:?}");
            }
            Event::Destroyed { showing } => {
                assert!(
                    got.iter().all(|e| *e == Effect::DropWindow),
                    "{seq:?}: {got:?}"
                );
                assert_eq!(got.len(), usize::from(before == Surface::Live), "{seq:?}");
                let expect = if before != Surface::Live {
                    before
                } else if showing {
                    Surface::Suspended
                } else {
                    Surface::Absent
                };
                assert_eq!(state, expect, "{seq:?}");
            }
        }
        assert!(
            got.len() <= 1,
            "{seq:?} asked for more than one effect: {got:?}"
        );
    }
    if seq.len() == depth {
        return;
    }
    for &event in alphabet {
        seq.push(event);
        walk(alphabet, depth, seq);
        seq.pop();
    }
}
