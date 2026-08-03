//! Tests for the scheduling core.
//!
//! Because the core takes time and OS state as arguments, every scenario here —
//! including sleep/wake and both suppression rules — runs instantly and
//! deterministically, with no window and no waiting.

use super::*;

/// Short, valid-under-`sanitized()` durations so tests stay fast and readable.
fn test_settings() -> Settings {
    Settings {
        work_secs: 60,
        break_secs: 10,
        snooze_secs: 30,
        style: BreakStyle::Overlay,
        chime: false,
        prewarn: false,
        skip_when_idle: false,
        idle_threshold_secs: 60,
        defer_on_fullscreen: false,
        autostart: false,
    }
}

fn machine() -> Machine {
    Machine::new(test_settings())
}

fn machine_with(f: impl FnOnce(&mut Settings)) -> Machine {
    let mut s = test_settings();
    f(&mut s);
    Machine::new(s)
}

/// Tick one second at a time with nothing going on in the environment.
fn tick_n(m: &mut Machine, n: u32) -> Vec<Effect> {
    let mut all = Vec::new();
    for _ in 0..n {
        all.extend(m.tick(Env::default(), 1));
    }
    all
}

/// Drop the every-tick tray update so assertions read cleanly.
fn notable(fx: Vec<Effect>) -> Vec<Effect> {
    fx.into_iter()
        .filter(|e| !matches!(e, Effect::Tray(_)))
        .collect()
}

fn remaining(m: &Machine) -> u32 {
    m.snapshot().remaining_secs
}

fn kind(m: &Machine) -> PhaseKind {
    m.snapshot().phase
}

// --- the basic cycle ---

#[test]
fn counts_down_without_firing_early() {
    let mut m = machine();
    assert!(notable(tick_n(&mut m, 59)).is_empty());
    assert_eq!(remaining(&m), 1);
    assert_eq!(kind(&m), PhaseKind::Work);
}

#[test]
fn fires_break_when_work_reaches_zero() {
    let mut m = machine();
    tick_n(&mut m, 59);
    assert_eq!(
        notable(tick_n(&mut m, 1)),
        vec![Effect::ShowBreak(BreakStyle::Overlay)]
    );
    assert_eq!(kind(&m), PhaseKind::Break);
    assert_eq!(remaining(&m), 10);
}

#[test]
fn break_ends_and_restarts_the_work_interval() {
    let mut m = machine();
    tick_n(&mut m, 60);
    assert!(notable(tick_n(&mut m, 9)).is_empty());
    assert_eq!(notable(tick_n(&mut m, 1)), vec![Effect::HideBreak]);
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 60);
}

#[test]
fn honours_the_configured_style() {
    let mut m = machine_with(|s| s.style = BreakStyle::Toast);
    tick_n(&mut m, 59);
    assert_eq!(
        notable(tick_n(&mut m, 1)),
        vec![Effect::ShowBreak(BreakStyle::Toast)]
    );
}

#[test]
fn a_missed_tick_still_lands_the_break() {
    // Loaded machine: several seconds pass between ticks. The break must still
    // fire on the tick that crosses zero, not be skipped past.
    let mut m = machine();
    tick_n(&mut m, 55);
    assert_eq!(
        notable(m.tick(Env::default(), 10)),
        vec![Effect::ShowBreak(BreakStyle::Overlay)]
    );
}

#[test]
fn zero_elapsed_changes_nothing() {
    let mut m = machine();
    tick_n(&mut m, 10);
    let before = remaining(&m);
    assert!(m.tick(Env::default(), 0).is_empty());
    assert_eq!(remaining(&m), before);
}

// --- pause / resume ---

#[test]
fn pause_resume_round_trip_preserves_remaining() {
    let mut m = machine();
    tick_n(&mut m, 20);
    assert_eq!(remaining(&m), 40);

    m.pause();
    assert_eq!(kind(&m), PhaseKind::Paused);

    m.resume();
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 40);
}

#[test]
fn paused_does_not_count_down() {
    let mut m = machine();
    tick_n(&mut m, 20);
    m.pause();
    tick_n(&mut m, 300);
    m.resume();
    assert_eq!(remaining(&m), 40);
}

#[test]
fn pausing_during_a_break_hides_it_and_resumes_into_work() {
    let mut m = machine();
    m.break_now();
    assert_eq!(notable(m.pause()), vec![Effect::HideBreak]);
    m.resume();
    // Not three leftover seconds of break — a whole fresh interval.
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 60);
}

#[test]
fn pausing_twice_is_a_no_op_and_does_not_nest() {
    let mut m = machine();
    tick_n(&mut m, 20);
    m.pause();
    assert!(m.pause().is_empty());
    m.resume();
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 40);
}

#[test]
fn toggle_pause_flips_both_ways() {
    let mut m = machine();
    m.toggle_pause();
    assert!(m.is_paused());
    m.toggle_pause();
    assert!(!m.is_paused());
}

// --- manual controls ---

#[test]
fn break_now_starts_a_break_immediately() {
    let mut m = machine();
    assert_eq!(
        notable(m.break_now()),
        vec![Effect::ShowBreak(BreakStyle::Overlay)]
    );
    assert_eq!(kind(&m), PhaseKind::Break);
}

#[test]
fn break_now_during_a_break_is_a_no_op() {
    let mut m = machine();
    m.break_now();
    assert!(m.break_now().is_empty());
}

#[test]
fn break_now_while_paused_un_pauses() {
    let mut m = machine();
    m.pause();
    m.break_now();
    assert_eq!(kind(&m), PhaseKind::Break);
}

#[test]
fn skip_dismisses_the_break_and_restarts_work() {
    let mut m = machine();
    m.break_now();
    assert_eq!(notable(m.skip()), vec![Effect::HideBreak]);
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 60);
}

#[test]
fn snooze_postpones_by_the_snooze_interval() {
    let mut m = machine();
    m.break_now();
    assert_eq!(notable(m.snooze()), vec![Effect::HideBreak]);
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 30);

    // …and it really does fire again afterwards.
    tick_n(&mut m, 29);
    assert_eq!(
        notable(tick_n(&mut m, 1)),
        vec![Effect::ShowBreak(BreakStyle::Overlay)]
    );
}

// --- settings changes ---

#[test]
fn apply_clamps_a_running_interval_into_new_bounds() {
    let mut m = machine();
    tick_n(&mut m, 10);
    assert_eq!(remaining(&m), 50);

    m.apply(Settings {
        work_secs: 40,
        ..test_settings()
    });
    // Shortening the interval takes effect now, not after the old countdown.
    assert_eq!(remaining(&m), 40);
}

#[test]
fn apply_leaves_a_shorter_remaining_alone() {
    let mut m = machine();
    tick_n(&mut m, 50);
    assert_eq!(remaining(&m), 10);

    m.apply(Settings {
        work_secs: 120,
        ..test_settings()
    });
    assert_eq!(remaining(&m), 10);
}

#[test]
fn apply_sanitizes_out_of_range_values() {
    let mut m = machine();
    m.apply(Settings {
        work_secs: 1,
        break_secs: 0,
        snooze_secs: 0,
        idle_threshold_secs: 1,
        ..test_settings()
    });
    let s = m.settings();
    assert_eq!(s.work_secs, MIN_WORK_SECS);
    assert_eq!(s.break_secs, MIN_BREAK_SECS);
    assert_eq!(s.snooze_secs, MIN_SNOOZE_SECS);
    assert_eq!(s.idle_threshold_secs, MIN_IDLE_THRESHOLD_SECS);
}

#[test]
fn apply_clamps_while_paused_too() {
    let mut m = machine();
    tick_n(&mut m, 10);
    m.pause();
    m.apply(Settings {
        work_secs: 40,
        ..test_settings()
    });
    m.resume();
    assert_eq!(remaining(&m), 40);
}

#[test]
fn new_sanitizes_its_settings() {
    let m = Machine::new(Settings {
        work_secs: 0,
        ..test_settings()
    });
    assert_eq!(m.settings().work_secs, MIN_WORK_SECS);
    assert_eq!(m.snapshot().remaining_secs, MIN_WORK_SECS);
}

// --- suppression: idle ---

#[test]
fn idle_longer_than_the_threshold_resets_instead_of_breaking() {
    let mut m = machine_with(|s| s.skip_when_idle = true);
    tick_n(&mut m, 30);
    assert_eq!(remaining(&m), 30);

    let fx = m.tick(
        Env {
            idle_secs: 60,
            fullscreen_active: false,
        },
        1,
    );
    assert!(notable(fx).is_empty());
    // Eyes already rested: interval restarts rather than firing on return.
    assert_eq!(remaining(&m), 60);
    assert_eq!(kind(&m), PhaseKind::Work);
}

#[test]
fn idle_just_under_the_threshold_keeps_counting_down() {
    let mut m = machine_with(|s| s.skip_when_idle = true);
    tick_n(&mut m, 30);
    m.tick(
        Env {
            idle_secs: 59,
            fullscreen_active: false,
        },
        1,
    );
    assert_eq!(remaining(&m), 29);
}

#[test]
fn idle_is_ignored_when_the_setting_is_off() {
    let mut m = machine_with(|s| s.skip_when_idle = false);
    tick_n(&mut m, 59);
    let fx = m.tick(
        Env {
            idle_secs: 9999,
            fullscreen_active: false,
        },
        1,
    );
    assert_eq!(notable(fx), vec![Effect::ShowBreak(BreakStyle::Overlay)]);
}

#[test]
fn idle_does_not_cut_a_break_short() {
    // The break runs its full 30 seconds even if you sit perfectly still —
    // which is exactly what looking into the distance looks like to the OS.
    let mut m = machine_with(|s| s.skip_when_idle = true);
    m.break_now();
    for _ in 0..9 {
        m.tick(
            Env {
                idle_secs: 9999,
                fullscreen_active: false,
            },
            1,
        );
    }
    assert_eq!(kind(&m), PhaseKind::Break);
    assert_eq!(remaining(&m), 1);
}

// --- suppression: fullscreen ---

#[test]
fn fullscreen_defers_the_break_rather_than_skipping_it() {
    let mut m = machine_with(|s| s.defer_on_fullscreen = true);
    tick_n(&mut m, 59);

    let fx = m.tick(
        Env {
            idle_secs: 0,
            fullscreen_active: true,
        },
        1,
    );
    assert!(notable(fx).is_empty());
    assert_eq!(kind(&m), PhaseKind::Work);
    // Still owed — re-checked after the snooze interval.
    assert_eq!(remaining(&m), 30);

    tick_n(&mut m, 29);
    assert_eq!(
        notable(tick_n(&mut m, 1)),
        vec![Effect::ShowBreak(BreakStyle::Overlay)]
    );
}

#[test]
fn fullscreen_is_ignored_when_the_setting_is_off() {
    let mut m = machine_with(|s| s.defer_on_fullscreen = false);
    tick_n(&mut m, 59);
    let fx = m.tick(
        Env {
            idle_secs: 0,
            fullscreen_active: true,
        },
        1,
    );
    assert_eq!(notable(fx), vec![Effect::ShowBreak(BreakStyle::Overlay)]);
}

#[test]
fn idle_takes_precedence_over_fullscreen() {
    // Fullscreen video playing while the user is away: resetting is right,
    // and deferring forever would be wrong.
    let mut m = machine_with(|s| {
        s.skip_when_idle = true;
        s.defer_on_fullscreen = true;
    });
    tick_n(&mut m, 59);
    m.tick(
        Env {
            idle_secs: 120,
            fullscreen_active: true,
        },
        1,
    );
    assert_eq!(remaining(&m), 60);
}

// --- sleep / wake ---

#[test]
fn a_sleep_sized_gap_restarts_the_work_interval() {
    let mut m = machine();
    tick_n(&mut m, 50);
    assert_eq!(remaining(&m), 10);

    let fx = m.tick(Env::default(), SLEEP_GAP_SECS);
    assert!(notable(fx).is_empty());
    // No stale break the instant the lid opens.
    assert_eq!(kind(&m), PhaseKind::Work);
    assert_eq!(remaining(&m), 60);
}

#[test]
fn a_sleep_sized_gap_hides_a_break_that_was_on_screen() {
    let mut m = machine();
    m.break_now();
    let fx = m.tick(Env::default(), 3600);
    assert_eq!(notable(fx), vec![Effect::HideBreak]);
    assert_eq!(kind(&m), PhaseKind::Work);
}

#[test]
fn a_sleep_sized_gap_leaves_a_paused_machine_paused() {
    let mut m = machine();
    tick_n(&mut m, 20);
    m.pause();
    m.tick(Env::default(), 3600);
    assert!(m.is_paused());
    m.resume();
    assert_eq!(remaining(&m), 40);
}

#[test]
fn a_gap_just_under_the_sleep_threshold_is_treated_as_real_work() {
    let mut m = machine_with(|s| s.work_secs = 120);
    let fx = m.tick(Env::default(), SLEEP_GAP_SECS - 1);
    assert!(notable(fx).is_empty());
    assert_eq!(remaining(&m), 120 - (SLEEP_GAP_SECS - 1));
}

// --- heads-up notification ---

#[test]
fn prewarn_fires_exactly_once_per_interval() {
    let mut m = machine_with(|s| s.prewarn = true);
    let fx = notable(tick_n(&mut m, 59));
    assert_eq!(fx, vec![Effect::Prewarn]);
}

#[test]
fn prewarn_fires_again_after_the_next_break() {
    let mut m = machine_with(|s| s.prewarn = true);
    tick_n(&mut m, 60); // work -> break
    tick_n(&mut m, 10); // break -> work
    let fx = notable(tick_n(&mut m, 59));
    assert_eq!(fx, vec![Effect::Prewarn]);
}

#[test]
fn prewarn_stays_silent_when_disabled() {
    let mut m = machine_with(|s| s.prewarn = false);
    assert!(notable(tick_n(&mut m, 59)).is_empty());
}

#[test]
fn a_reset_clears_the_prewarn_latch() {
    let mut m = machine_with(|s| {
        s.prewarn = true;
        s.skip_when_idle = true;
    });
    tick_n(&mut m, 31); // prewarn has fired
    m.tick(
        Env {
            idle_secs: 120,
            fullscreen_active: false,
        },
        1,
    ); // reset
    let fx = notable(tick_n(&mut m, 59));
    assert_eq!(fx, vec![Effect::Prewarn]);
}

// --- snapshot + formatting ---

#[test]
fn snapshot_reports_the_phase_length_for_the_ring() {
    let mut m = machine();
    assert_eq!(m.snapshot().total_secs, 60);
    m.break_now();
    assert_eq!(m.snapshot().total_secs, 10);
}

#[test]
fn snapshot_total_is_never_zero() {
    // The frontend divides by total_secs.
    let m = Machine::new(Settings {
        break_secs: 0,
        ..test_settings()
    });
    assert!(m.snapshot().total_secs >= 1);
}

#[test]
fn snapshot_carries_style_and_chime_to_the_ui() {
    let m = machine_with(|s| {
        s.style = BreakStyle::Toast;
        s.chime = true;
    });
    let snap = m.snapshot();
    assert_eq!(snap.style, BreakStyle::Toast);
    assert!(snap.chime);
}

#[test]
fn every_tick_refreshes_the_tray() {
    let mut m = machine();
    let fx = m.tick(Env::default(), 1);
    assert!(matches!(fx.last(), Some(Effect::Tray(_))));
}

#[test]
fn tray_label_reflects_the_phase() {
    let mut m = machine();
    assert!(m.tray_label().contains("next break in"));
    m.break_now();
    assert!(m.tray_label().contains("distance"));
    m.pause();
    assert_eq!(m.tray_label(), "Irys — paused");
}

#[test]
fn mmss_formatting() {
    assert_eq!(fmt_mmss(0), "00:00");
    assert_eq!(fmt_mmss(9), "00:09");
    assert_eq!(fmt_mmss(754), "12:34");
    assert_eq!(fmt_mmss(1800), "30:00");
}

#[test]
fn settings_round_trip_through_json() {
    // The frontend sends this shape back verbatim; camelCase must survive.
    let s = test_settings();
    let json = serde_json::to_string(&s).expect("serialize");
    assert!(json.contains("workSecs"));
    assert!(json.contains("\"overlay\""));
    let back: Settings = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, s);
}
