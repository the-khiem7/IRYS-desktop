//! The scheduling core for the 30-30-30 rule.
//!
//! This module is deliberately **pure**: no Tauri types, no I/O, no clock, no
//! logging. Everything it needs from the outside world arrives as arguments
//! ([`Env`] and `elapsed_secs`), and everything it wants done leaves as data
//! ([`Effect`]). That makes the entire schedule - including every suppression
//! rule and edge case - testable with `cargo test` and no window on screen.
//!
//! The one rule to preserve when editing: *never* reach for the clock or the OS
//! in here. Add a field to [`Env`] instead.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Every 30 minutes…
pub const DEFAULT_WORK_SECS: u32 = 30 * 60;
/// …for 30 seconds. (The middle 30 - 30 feet - is guidance, not a timer.)
pub const DEFAULT_BREAK_SECS: u32 = 30;
pub const DEFAULT_SNOOZE_SECS: u32 = 5 * 60;
pub const DEFAULT_IDLE_THRESHOLD_SECS: u32 = 60;

/// How far ahead of a break the optional heads-up notification fires.
pub const PREWARN_LEAD_SECS: u32 = 30;

/// A gap this large between ticks means the machine slept or the process was
/// suspended - the user was not actually working, so the work interval restarts
/// instead of firing a break that is already stale.
pub const SLEEP_GAP_SECS: u32 = 90;

// Guard rails for values arriving from the settings UI or a hand-edited store
// file. Sanitising here, in the core, means no other layer has to trust input.
const MIN_WORK_SECS: u32 = 30;
const MAX_WORK_SECS: u32 = 4 * 60 * 60;
const MIN_BREAK_SECS: u32 = 5;
const MAX_BREAK_SECS: u32 = 10 * 60;
const MIN_SNOOZE_SECS: u32 = 30;
const MAX_SNOOZE_SECS: u32 = 60 * 60;
const MIN_IDLE_THRESHOLD_SECS: u32 = 15;
const MAX_IDLE_THRESHOLD_SECS: u32 = 60 * 60;

/// How the break reminder presents itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BreakStyle {
    /// Frameless window filling the monitor. Hard to ignore, always escapable.
    Overlay,
    /// Small card in the corner. Never steals focus, easy to ignore.
    Toast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub work_secs: u32,
    pub break_secs: u32,
    pub snooze_secs: u32,
    pub style: BreakStyle,
    pub chime: bool,
    /// Fire a native notification [`PREWARN_LEAD_SECS`] before the break.
    pub prewarn: bool,
    pub skip_when_idle: bool,
    pub idle_threshold_secs: u32,
    pub defer_on_fullscreen: bool,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_secs: DEFAULT_WORK_SECS,
            break_secs: DEFAULT_BREAK_SECS,
            snooze_secs: DEFAULT_SNOOZE_SECS,
            style: BreakStyle::Overlay,
            chime: false,
            prewarn: true,
            skip_when_idle: true,
            idle_threshold_secs: DEFAULT_IDLE_THRESHOLD_SECS,
            defer_on_fullscreen: true,
            // Off until the user asks for it - never register autostart silently.
            autostart: false,
        }
    }
}

impl Settings {
    /// Clamp every duration into a sane range. Called on load and on every
    /// update, so the machine can assume its settings are already valid.
    pub fn sanitized(self) -> Self {
        Self {
            work_secs: self.work_secs.clamp(MIN_WORK_SECS, MAX_WORK_SECS),
            break_secs: self.break_secs.clamp(MIN_BREAK_SECS, MAX_BREAK_SECS),
            snooze_secs: self.snooze_secs.clamp(MIN_SNOOZE_SECS, MAX_SNOOZE_SECS),
            idle_threshold_secs: self
                .idle_threshold_secs
                .clamp(MIN_IDLE_THRESHOLD_SECS, MAX_IDLE_THRESHOLD_SECS),
            ..self
        }
    }
}

/// Everything the machine needs to know about the outside world. Injected on
/// every tick and never read directly, which is what keeps this module pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Env {
    pub idle_secs: u32,
    pub fullscreen_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Work { remaining: u32 },
    Break { remaining: u32 },
    Paused { resume_to: Box<Phase> },
}

/// Flattened phase for the frontend - the recursive `Paused` variant would be
/// awkward to consume in TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseKind {
    Work,
    Break,
    Paused,
}

/// What the UI and tray need in order to render. Sent on every tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: PhaseKind,
    pub remaining_secs: u32,
    /// Length of the current phase, so the countdown ring can show a fraction.
    pub total_secs: u32,
    pub style: BreakStyle,
    pub chime: bool,
}

/// Something the outside world should do. The core decides *what*; the effect
/// executor in `effects.rs` decides *how*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    ShowBreak(BreakStyle),
    HideBreak,
    /// Heads-up that a break is coming.
    Prewarn,
    /// New tray tooltip text.
    Tray(String),
}

pub struct Machine {
    phase: Phase,
    settings: Settings,
    /// Ensures the heads-up notification fires at most once per work interval.
    prewarned: bool,
}

impl Machine {
    pub fn new(settings: Settings) -> Self {
        let settings = settings.sanitized();
        Self {
            phase: Phase::Work {
                remaining: settings.work_secs,
            },
            settings,
            prewarned: false,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.phase, Phase::Paused { .. })
    }

    /// Advance by `elapsed_secs` of real time.
    ///
    /// `elapsed_secs` is a measured delta rather than an assumed `1`, so an
    /// overloaded machine that misses ticks still keeps an accurate schedule.
    pub fn tick(&mut self, env: Env, elapsed_secs: u32) -> Vec<Effect> {
        if elapsed_secs == 0 {
            return Vec::new();
        }

        // Paused means paused: no countdown, no suppression, no sleep handling.
        if self.is_paused() {
            return vec![Effect::Tray(self.tray_label())];
        }

        // Slept, hibernated, or the process was starved: the interval is void.
        if elapsed_secs >= SLEEP_GAP_SECS {
            return self.restart_work();
        }

        let mut fx = Vec::new();

        match self.phase {
            // Unreachable: guarded above. Handled explicitly so adding a phase
            // is a compile error rather than a silent fallthrough.
            Phase::Paused { .. } => {}

            Phase::Work { remaining } => {
                if self.settings.skip_when_idle
                    && env.idle_secs >= self.settings.idle_threshold_secs
                {
                    // Away from the keyboard - the eyes are already resting, so
                    // reset rather than queue up an interruption for their return.
                    self.begin_work();
                } else {
                    let next = remaining.saturating_sub(elapsed_secs);
                    if next == 0 {
                        if self.settings.defer_on_fullscreen && env.fullscreen_active {
                            // Mid-call or mid-presentation. Postpone and re-check;
                            // the break is still owed, just not right now.
                            self.phase = Phase::Work {
                                remaining: self.settings.snooze_secs,
                            };
                            self.prewarned = false;
                        } else {
                            fx.extend(self.begin_break());
                        }
                    } else {
                        if self.settings.prewarn && !self.prewarned && next <= PREWARN_LEAD_SECS {
                            self.prewarned = true;
                            fx.push(Effect::Prewarn);
                        }
                        self.phase = Phase::Work { remaining: next };
                    }
                }
            }

            Phase::Break { remaining } => {
                let next = remaining.saturating_sub(elapsed_secs);
                if next == 0 {
                    self.begin_work();
                    fx.push(Effect::HideBreak);
                } else {
                    self.phase = Phase::Break { remaining: next };
                }
            }
        }

        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    /// Freeze the countdown, remembering where to pick up. Pausing during a
    /// break dismisses the overlay and resumes into a fresh work interval -
    /// resuming into three leftover seconds of break would be pointless.
    pub fn pause(&mut self) -> Vec<Effect> {
        if self.is_paused() {
            return Vec::new();
        }

        let was_breaking = matches!(self.phase, Phase::Break { .. });
        let resume_to = if was_breaking {
            Phase::Work {
                remaining: self.settings.work_secs,
            }
        } else {
            self.phase.clone()
        };
        self.phase = Phase::Paused {
            resume_to: Box::new(resume_to),
        };
        self.prewarned = false;

        let mut fx = Vec::new();
        if was_breaking {
            fx.push(Effect::HideBreak);
        }
        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    pub fn resume(&mut self) -> Vec<Effect> {
        // Cloned out first so the borrow of `self.phase` ends before the
        // assignment.
        let resumed = match &self.phase {
            Phase::Paused { resume_to } => Some((**resume_to).clone()),
            _ => None,
        };
        if let Some(phase) = resumed {
            self.phase = phase;
        }
        vec![Effect::Tray(self.tray_label())]
    }

    pub fn toggle_pause(&mut self) -> Vec<Effect> {
        if self.is_paused() {
            self.resume()
        } else {
            self.pause()
        }
    }

    /// Start a break right now. Also un-pauses, since asking for a break while
    /// paused unambiguously means "I want one".
    pub fn break_now(&mut self) -> Vec<Effect> {
        if matches!(self.phase, Phase::Break { .. }) {
            return Vec::new();
        }
        let mut fx = self.begin_break();
        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    /// Dismiss the break and start the next work interval from the top.
    pub fn skip(&mut self) -> Vec<Effect> {
        let was_breaking = matches!(self.phase, Phase::Break { .. });
        self.begin_work();
        let mut fx = Vec::new();
        if was_breaking {
            fx.push(Effect::HideBreak);
        }
        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    /// Put the break off briefly rather than dismissing it outright.
    pub fn snooze(&mut self) -> Vec<Effect> {
        let was_breaking = matches!(self.phase, Phase::Break { .. });
        self.phase = Phase::Work {
            remaining: self.settings.snooze_secs,
        };
        self.prewarned = false;
        let mut fx = Vec::new();
        if was_breaking {
            fx.push(Effect::HideBreak);
        }
        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    /// Replace the settings, clamping any countdown in flight into the new
    /// bounds so shortening the interval takes effect immediately instead of
    /// after the old, longer one finishes.
    pub fn apply(&mut self, settings: Settings) -> Vec<Effect> {
        let settings = settings.sanitized();
        self.settings = settings;

        // Computed into a local first: assigning to `self.phase` while the
        // match still borrows it would not borrow-check.
        let clamped = match &self.phase {
            Phase::Work { remaining } => Phase::Work {
                remaining: (*remaining).min(settings.work_secs),
            },
            Phase::Break { remaining } => Phase::Break {
                remaining: (*remaining).min(settings.break_secs),
            },
            Phase::Paused { resume_to } => Phase::Paused {
                resume_to: Box::new(match resume_to.as_ref() {
                    Phase::Work { remaining } => Phase::Work {
                        remaining: (*remaining).min(settings.work_secs),
                    },
                    Phase::Break { remaining } => Phase::Break {
                        remaining: (*remaining).min(settings.break_secs),
                    },
                    // pause() never nests, so this is unreachable in practice.
                    other @ Phase::Paused { .. } => other.clone(),
                }),
            },
        };
        self.phase = clamped;

        vec![Effect::Tray(self.tray_label())]
    }

    pub fn snapshot(&self) -> Snapshot {
        let (phase, remaining_secs, total_secs) = match &self.phase {
            Phase::Work { remaining } => (PhaseKind::Work, *remaining, self.settings.work_secs),
            Phase::Break { remaining } => (PhaseKind::Break, *remaining, self.settings.break_secs),
            Phase::Paused { resume_to } => match resume_to.as_ref() {
                Phase::Break { remaining } => {
                    (PhaseKind::Paused, *remaining, self.settings.break_secs)
                }
                Phase::Work { remaining } => {
                    (PhaseKind::Paused, *remaining, self.settings.work_secs)
                }
                Phase::Paused { .. } => (PhaseKind::Paused, 0, self.settings.work_secs),
            },
        };

        Snapshot {
            phase,
            remaining_secs,
            // Never zero: the frontend divides by this.
            total_secs: total_secs.max(1),
            style: self.settings.style,
            chime: self.settings.chime,
        }
    }

    // --- internals ---

    fn begin_work(&mut self) {
        self.phase = Phase::Work {
            remaining: self.settings.work_secs,
        };
        self.prewarned = false;
    }

    fn begin_break(&mut self) -> Vec<Effect> {
        self.phase = Phase::Break {
            remaining: self.settings.break_secs,
        };
        self.prewarned = false;
        vec![Effect::ShowBreak(self.settings.style)]
    }

    fn restart_work(&mut self) -> Vec<Effect> {
        let was_breaking = matches!(self.phase, Phase::Break { .. });
        self.begin_work();
        let mut fx = Vec::new();
        if was_breaking {
            fx.push(Effect::HideBreak);
        }
        fx.push(Effect::Tray(self.tray_label()));
        fx
    }

    fn tray_label(&self) -> String {
        match &self.phase {
            Phase::Work { remaining } => {
                format!("IRYS - next break in {}", fmt_mmss(*remaining))
            }
            Phase::Break { remaining } => {
                format!("IRYS - look into the distance · {remaining}s")
            }
            Phase::Paused { .. } => "IRYS - paused".to_owned(),
        }
    }
}

/// `754` -> `"12:34"`. Hours fold into the minutes field; the maximum work
/// interval is 4 hours, so `240:00` is the widest this ever gets.
pub fn fmt_mmss(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
