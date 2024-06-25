use bevy::prelude::*;

use crate::{emulator::Emulator, RESOLUTION, SCALE};

#[derive(Component)]
struct Pixel(usize, usize);

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_display);
        app.add_systems(Update, update_screen_system);
    }
}

fn init_display(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    for y in 0..r8::constants::HEIGHT {
        for x in 0..r8::constants::WIDTH {
            commands.spawn((
                Pixel(x, r8::constants::HEIGHT - y - 1),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(SCALE as _, SCALE as _)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * SCALE as f32 - RESOLUTION.0 * 0.5
                            + SCALE as f32 * 0.5,
                        y as f32 * SCALE as f32 - RESOLUTION.1 * 0.5
                            + SCALE as f32 * 0.5,
                        0.,
                    ),
                    ..default()
                },
            ));
        }
    }
}

fn update_screen_system(r8: Res<Emulator>, mut query: Query<(&mut Sprite, &Pixel)>) {
    if r8.0.display().updated {
        for (mut sprite, pixel) in &mut query {
            sprite.color = if r8.0.display().get(pixel.0, pixel.1) {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
    }
}
