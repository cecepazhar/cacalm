//! Pomodoro focus timer — pure state machine, no I/O.
//!
//! Cycle (PRD §F5): FOCUS, SHORT, FOCUS, SHORT, FOCUS, SHORT, FOCUS, LONG …
//! i.e. four focus blocks per long break. `config.long_break_every` controls
//! that cadence so it is not hardcoded.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Focus => "FOCUS",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimerConfig {
    pub focus: Duration,
    pub short_break: Duration,
    pub long_break: Duration,
    /// A long break replaces a short break every N focus blocks.
    pub long_break_every: u32,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            focus: Duration::from_secs(25 * 60),
            short_break: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(15 * 60),
            long_break_every: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerEvent {
    Tick { remaining: Duration, phase: Phase },
    PhaseCompleted(Phase),
    PhaseStarted(Phase),
}

/// Deterministic Pomodoro state machine. Time is injected via [`Self::tick`]
/// so this is trivially unit-testable without sleeping.
#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    config: TimerConfig,
    phase: Phase,
    remaining: Duration,
    paused: bool,
    /// Number of focus blocks completed since the last long break reset.
    focus_since_long: u32,
    pub sessions_today: u32,
    pub focus_seconds_today: u64,
}

impl PomodoroTimer {
    pub fn new(config: TimerConfig) -> Self {
        Self {
            phase: Phase::Focus,
            remaining: config.focus,
            paused: false,
            focus_since_long: 0,
            sessions_today: 0,
            focus_seconds_today: 0,
            config,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn config(&self) -> TimerConfig {
        self.config
    }

    pub fn set_config(&mut self, config: TimerConfig) {
        self.config = config;
        self.remaining = self.phase_duration();
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Full reset back to a fresh focus block.
    pub fn reset(&mut self) {
        self.phase = Phase::Focus;
        self.remaining = self.config.focus;
        self.paused = false;
        self.focus_since_long = 0;
    }

    /// Duration of the current phase per config.
    pub fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Focus => self.config.focus,
            Phase::ShortBreak => self.config.short_break,
            Phase::LongBreak => self.config.long_break,
        }
    }

    /// Advance the clock by `elapsed`. Returns the events that occurred, in
    /// order.
    pub fn tick(&mut self, elapsed: Duration) -> Vec<TimerEvent> {
        let mut events = Vec::new();
        if self.paused || elapsed.is_zero() {
            return events;
        }

        if self.phase == Phase::Focus {
            self.focus_seconds_today += elapsed.as_secs();
        }

        if elapsed < self.remaining {
            self.remaining -= elapsed;
            events.push(TimerEvent::Tick {
                remaining: self.remaining,
                phase: self.phase,
            });
            return events;
        }

        let completed = self.phase;
        events.push(TimerEvent::PhaseCompleted(completed));
        self.advance_phase();
        events.push(TimerEvent::PhaseStarted(self.phase));
        events
    }

    /// Force-move to the next phase (Skip button).
    pub fn skip(&mut self) -> TimerEvent {
        self.advance_phase();
        TimerEvent::PhaseStarted(self.phase)
    }

    fn advance_phase(&mut self) {
        match self.phase {
            Phase::Focus => {
                self.sessions_today += 1;
                self.focus_since_long += 1;
                self.phase = if self.focus_since_long >= self.config.long_break_every {
                    self.focus_since_long = 0;
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                };
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Focus;
            }
        }
        self.remaining = self.phase_duration();
    }
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new(TimerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TimerConfig {
        TimerConfig {
            focus: Duration::from_secs(25 * 60),
            short_break: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(15 * 60),
            long_break_every: 4,
        }
    }

    #[test]
    fn starts_in_focus() {
        let t = PomodoroTimer::new(cfg());
        assert_eq!(t.phase(), Phase::Focus);
        assert_eq!(t.remaining(), Duration::from_secs(1500));
    }

    #[test]
    fn pauses_accumulate_nothing() {
        let mut t = PomodoroTimer::new(cfg());
        t.pause();
        let events = t.tick(Duration::from_secs(60));
        assert!(events.is_empty());
        assert_eq!(t.remaining(), Duration::from_secs(1500));
    }

    #[test]
    fn focus_completion_moves_to_short_break() {
        let mut t = PomodoroTimer::new(cfg());
        let events = t.tick(Duration::from_secs(25 * 60));
        assert!(events
            .iter()
            .any(|e| matches!(e, TimerEvent::PhaseCompleted(Phase::Focus))));
        assert_eq!(t.phase(), Phase::ShortBreak);
        assert_eq!(t.sessions_today, 1);
    }

    #[test]
    fn long_break_arrives_every_fourth_focus() {
        let mut t = PomodoroTimer::new(cfg());
        // Drive through three complete focus→break pairs.
        for _ in 0..3 {
            t.tick(Duration::from_secs(25 * 60)); // focus done -> break
            t.tick(t.phase_duration()); // break done -> focus
        }
        // Fourth focus block should yield a long break.
        t.tick(Duration::from_secs(25 * 60));
        assert_eq!(t.phase(), Phase::LongBreak);
    }

    #[test]
    fn skip_advances_phase() {
        let mut t = PomodoroTimer::new(cfg());
        t.skip();
        assert_eq!(t.phase(), Phase::ShortBreak);
        t.skip();
        assert_eq!(t.phase(), Phase::Focus);
    }

    #[test]
    fn custom_config_is_applied() {
        let mut t = PomodoroTimer::new(cfg());
        t.set_config(TimerConfig {
            focus: Duration::from_secs(10 * 60),
            ..cfg()
        });
        assert_eq!(t.remaining(), Duration::from_secs(600));
    }
}