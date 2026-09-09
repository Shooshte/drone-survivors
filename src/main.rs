mod arena;
mod combat;
mod energy;
mod game;
mod modules;
mod upgrades;

use arena::{ArenaPlugin, ArenaScenePlugin};
use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, window::WindowResizeConstraints,
};

fn main() {
    let validation = combat::validation::ValidationConfig::parse(std::env::args().skip(1))
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        });
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.025, 0.045, 0.065)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Drone Survivors — Test Arena".into(),
                resolution: (1120, 720).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: 640.,
                    min_height: 480.,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            ArenaPlugin,
            ArenaScenePlugin,
            combat::CombatPlugin,
            combat::CombatScenePlugin,
            energy::scene::EnergyScenePlugin,
            upgrades::runtime::UpgradePlugin,
            upgrades::scene::UpgradeScenePlugin,
        ))
        .add_systems(Update, quit.run_if(input_just_pressed(KeyCode::Escape)));
    if let Some(config) = validation {
        combat::validation::install(&mut app, config);
    }
    app.run();
}

fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
