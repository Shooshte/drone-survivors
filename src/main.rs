mod arena;

use arena::{ArenaPlugin, ArenaScenePlugin};
use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, window::WindowResizeConstraints,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.045, 0.065)))
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
        .add_plugins((ArenaPlugin, ArenaScenePlugin))
        .add_systems(Update, quit.run_if(input_just_pressed(KeyCode::Escape)))
        .run();
}

fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
