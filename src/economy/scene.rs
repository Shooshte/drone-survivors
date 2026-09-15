//! Reusable pickup visuals and the unbanked combat counter.
use super::{
    AttemptResources,
    runtime::{ComponentCache, Pickup},
};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

pub(crate) struct EconomyScenePlugin;
#[derive(Component)]
struct ResourceHud;
#[derive(Resource)]
struct PickupAssets {
    salvage: Handle<Mesh>,
    cache: Handle<Mesh>,
    ring: Handle<Mesh>,
    gold: Handle<StandardMaterial>,
    violet: Handle<StandardMaterial>,
}
impl Plugin for EconomyScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(crate::combat::CombatSceneSetup))
            .add_systems(Update, (visuals, hud).in_set(GameplaySet::Presentation));
    }
}
fn setup(
    mut commands: Commands,
    root: Single<Entity, With<crate::combat::CombatHudRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(PickupAssets {
        salvage: meshes.add(Cuboid::new(10., 10., 10.)),
        cache: meshes.add(Cuboid::new(22., 20., 22.)),
        ring: meshes.add(Torus::new(30., 33.)),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.76, 0.2),
            unlit: true,
            ..default()
        }),
        violet: materials.add(StandardMaterial {
            base_color: Color::srgb(0.82, 0.48, 1.),
            unlit: true,
            ..default()
        }),
    });
    commands.entity(*root).with_children(|parent| {
        parent.spawn((
            ResourceHud,
            Text::default(),
            TextFont::from_font_size(14.),
            crate::arena::FooterFont::new(14., 12.),
            TextColor(Color::srgb(1., 0.81, 0.47)),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            Node {
                width: percent(100),
                ..default()
            },
        ));
    });
}
fn visuals(
    mut commands: Commands,
    assets: Res<PickupAssets>,
    pickups: Query<(Entity, Option<&ComponentCache>), Added<Pickup>>,
) {
    for (entity, cache) in &pickups {
        let (mesh, material) = if cache.is_some() {
            (&assets.cache, &assets.violet)
        } else {
            (&assets.salvage, &assets.gold)
        };
        commands
            .entity(entity)
            .insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));
        if cache.is_some() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(assets.ring.clone()),
                    MeshMaterial3d(assets.violet.clone()),
                    Transform::from_xyz(0., -8., 0.),
                ));
            });
        }
    }
}
fn hud(
    resources: Res<AttemptResources>,
    phase: Res<GamePhase>,
    notice: Option<Res<super::runtime::DiscoveryNotice>>,
    mut hud: Single<(&mut Text, &mut Node), With<ResourceHud>>,
) {
    let (text, node) = &mut *hud;
    node.display = if matches!(*phase, GamePhase::Playing | GamePhase::Choosing) {
        Display::Flex
    } else {
        Display::None
    };
    let mut value = format!(
        "UNBANKED  Salvage {}  |  Components {}",
        resources.collected.salvage, resources.collected.components
    );
    if let Some(notice) = notice.filter(|notice| notice.remaining > 0.) {
        value.push('\n');
        value.push_str(&notice.text);
    }
    if text.0 != value {
        text.0 = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hud_uses_attempt_collection_and_clears_on_reset() {
        let mut app = App::new();
        app.init_resource::<AttemptResources>()
            .insert_resource(GamePhase::Playing)
            .add_systems(Update, hud);
        let id = app
            .world_mut()
            .spawn((ResourceHud, Text::default(), Node::default()))
            .id();
        app.world_mut().resource_mut::<AttemptResources>().collected = crate::economy::Amounts {
            salvage: 17,
            components: 2,
        };
        app.update();
        assert_eq!(
            app.world().get::<Text>(id).unwrap().0,
            "UNBANKED  Salvage 17  |  Components 2"
        );
        *app.world_mut().resource_mut::<AttemptResources>() = default();
        app.update();
        assert!(app.world().get::<Text>(id).unwrap().0.contains("Salvage 0"));
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Hub;
        app.update();
        assert_eq!(app.world().get::<Node>(id).unwrap().display, Display::None);
    }
}
