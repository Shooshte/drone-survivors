use super::*;
use crate::{
    energy::{Energy, EnergyConfig},
    game::GamePhase,
};

#[derive(Component)]
pub(crate) struct ModuleHud(pub usize);

pub(crate) fn present(
    modules: Res<Modules>,
    config: Res<ModuleConfig>,
    energy: Res<Energy>,
    battery_config: Res<EnergyConfig>,
    phase: Res<GamePhase>,
    bomb: Res<crate::combat::bombs::BombState>,
    mut rows: Query<(&ModuleHud, &mut Text, &mut TextColor)>,
) {
    for (row, mut text, mut color) in &mut rows {
        let i = row.0;
        let value = if let Some(kind) = modules.loadout.slots()[i] {
            let enabled = modules.enabled[i] && modules.disabled_for[i] <= 0.;
            let state = if enabled {
                "ON"
            } else if energy.current == 0. {
                "EMPTY"
            } else {
                "OFF"
            };
            let drain = if enabled && *phase == GamePhase::Playing {
                config.drain(kind)
            } else {
                0.
            };
            let detail = if kind == ModuleKind::Shield {
                if modules.shield.blocks > 0 {
                    format!(" READY {}", modules.shield.blocks)
                } else {
                    format!(
                        " {:.1}s{}",
                        modules.shield.remaining,
                        if enabled && *phase == GamePhase::Playing {
                            " recharge"
                        } else {
                            " paused"
                        }
                    )
                }
            } else if kind == ModuleKind::Repulsor {
                format!(" {:.1}s pulse", bomb.pulse_cooldown)
            } else {
                String::new()
            };
            let rejection = if modules.rejected_for[i] > 0. {
                format!(" Need {:.0}", battery_config.activation)
            } else {
                String::new()
            };
            if modules.disabled_for[i] > 0. {
                format!(
                    "{} {} LOCK {:.1}s 0/s",
                    i + 1,
                    kind.name(),
                    modules.disabled_for[i]
                )
            } else {
                format!(
                    "{} {} {state} {drain:.0}/{:.0}/s{detail}{rejection}",
                    i + 1,
                    kind.name(),
                    config.drain(kind)
                )
            }
        } else {
            format!("{} EMPTY SLOT  0/s", i + 1)
        };
        if text.0 != value {
            text.0 = value;
        }
        color.0 = if modules.disabled_for[i] > 0. {
            Color::srgb(0.85, 0.6, 1.)
        } else if modules.rejected_for[i] > 0. {
            Color::srgb(1., 0.65, 0.3)
        } else if modules.enabled[i] {
            Color::srgb(0.4, 1., 0.8)
        } else {
            Color::srgb(0.68, 0.76, 0.8)
        };
    }
}
