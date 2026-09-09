use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HazardPhase {
    #[default]
    Inactive,
    Warning,
    Active,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ActiveWindow {
    pub from: f32,
    pub to: f32,
}

#[derive(Resource, Default)]
pub(crate) struct HazardState {
    pub phase: HazardPhase,
    pub elapsed: f32,
    pub cycle: u64,
    pub active_window: Option<ActiveWindow>,
}

pub(crate) const HAZARD_DAMAGE: u32 = 10;
const INACTIVE_SECONDS: f32 = 3.;
const WARNING_SECONDS: f32 = 1.;
const ACTIVE_SECONDS: f32 = 1.;
const PHASE_EPSILON: f32 = 0.00001;

impl HazardState {
    pub(crate) fn remaining(&self) -> f32 {
        let duration = match self.phase {
            HazardPhase::Inactive => INACTIVE_SECONDS,
            HazardPhase::Warning => WARNING_SECONDS,
            HazardPhase::Active => ACTIVE_SECONDS,
        };
        (duration - self.elapsed).max(0.)
    }

    pub(crate) fn advance(&mut self, seconds: f32) {
        self.active_window = None;
        let remaining = self.remaining();
        match self.phase {
            HazardPhase::Inactive => {
                if seconds + PHASE_EPSILON >= remaining {
                    // Never spend an unseen warning inside an inactive hitch.
                    self.phase = HazardPhase::Warning;
                    self.elapsed = 0.;
                } else {
                    self.elapsed += seconds;
                }
            }
            HazardPhase::Warning => {
                if seconds + PHASE_EPSILON >= remaining {
                    self.cycle += 1;
                    let active_seconds = (seconds - remaining).clamp(0., ACTIVE_SECONDS);
                    self.active_window = Some(ActiveWindow {
                        from: if seconds > 0. {
                            (remaining / seconds).min(1.)
                        } else {
                            0.
                        },
                        to: if seconds > 0. {
                            ((remaining + active_seconds) / seconds).min(1.)
                        } else {
                            1.
                        },
                    });
                    if active_seconds + PHASE_EPSILON >= ACTIVE_SECONDS {
                        self.phase = HazardPhase::Inactive;
                        self.elapsed = 0.;
                    } else {
                        self.phase = HazardPhase::Active;
                        self.elapsed = active_seconds;
                    }
                } else {
                    self.elapsed += seconds;
                }
            }
            HazardPhase::Active => {
                self.active_window = Some(ActiveWindow {
                    from: 0.,
                    to: if seconds > 0. {
                        (remaining / seconds).min(1.)
                    } else {
                        1.
                    },
                });
                if seconds + PHASE_EPSILON >= remaining {
                    self.phase = HazardPhase::Inactive;
                    self.elapsed = 0.;
                } else {
                    self.elapsed += seconds;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_is_harmless_and_active_window_respects_frame_time() {
        let mut state = HazardState::default();
        state.advance(3.);
        assert_eq!(state.phase, HazardPhase::Warning);
        assert!(state.active_window.is_none());
        state.advance(0.5);
        assert_eq!(state.phase, HazardPhase::Warning);
        state.advance(0.75);
        assert_eq!(state.phase, HazardPhase::Active);
        let window = state.active_window.unwrap();
        assert!((window.from - 2. / 3.).abs() < 0.0001);
        assert_eq!(window.to, 1.);
        assert_eq!(state.cycle, 1);
        state.advance(1.);
        assert_eq!(state.phase, HazardPhase::Inactive);
        assert!((state.active_window.unwrap().to - 0.75).abs() < 0.0001);
    }

    #[test]
    fn inactive_hitch_cannot_skip_warning_or_accumulate_damage_cycles() {
        let mut state = HazardState::default();
        state.advance(30.);
        assert_eq!(state.phase, HazardPhase::Warning);
        assert_eq!(state.elapsed, 0.);
        assert_eq!(state.cycle, 0);
        assert!(state.active_window.is_none());
        state.advance(30.);
        assert_eq!(state.cycle, 1);
        let active = state.active_window.unwrap();
        assert!((active.from - 1. / 30.).abs() < 0.0001);
        assert!((active.to - 2. / 30.).abs() < 0.0001);
        assert_eq!(state.phase, HazardPhase::Inactive);
    }

    #[test]
    fn phase_timing_matches_normal_frame_rates() {
        for hz in [30, 60, 144] {
            let mut state = HazardState::default();
            for _ in 0..hz * 3 {
                state.advance(1. / hz as f32);
            }
            assert_eq!(state.phase, HazardPhase::Warning, "{hz} Hz");
            for _ in 0..hz {
                state.advance(1. / hz as f32);
            }
            assert_eq!(state.phase, HazardPhase::Active, "{hz} Hz");
            for _ in 0..hz {
                state.advance(1. / hz as f32);
            }
            assert_eq!(state.phase, HazardPhase::Inactive, "{hz} Hz");
        }
    }
}
