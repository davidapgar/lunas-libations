// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use lunas_libations::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luna's Libations".to_string(),
                resolution: (800., 600.).into(),
                canvas: Some("#bevy".to_owned()),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(GamePlugin)
        .run();
}
