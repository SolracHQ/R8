use bevy::prelude::*;
use r8_core::constants;

use crate::{emulator::Emulator, RESOLUTION};

#[derive(Component)]
struct Pixel(usize, usize);

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, init_display);
    app.add_systems(Update, update_screen_system);
  }
}

fn init_display(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // Spawn a camera
  commands.spawn(Camera2d);

  let pixel_width = RESOLUTION.0 as f32 / constants::WIDTH as f32;
  let pixel_height = RESOLUTION.1 as f32 / constants::HEIGHT as f32;

  let rectangle = meshes.add(Rectangle::new(pixel_width, pixel_height));

  // Create a sprite for each pixel in the Chip-8 framebuffer.
  for y in 0..constants::HEIGHT {
    for x in 0..constants::WIDTH {
      // Calculate position: center each pixel in its grid cell
      // Origin is at center of window, so we offset by half resolution
      let pos_x = (x as f32 + 0.5) * pixel_width - (RESOLUTION.0 as f32 / 2.0);
      let pos_y =
        ((constants::HEIGHT - y - 1) as f32 + 0.5) * pixel_height - (RESOLUTION.1 as f32 / 2.0);

      commands.spawn((
        Pixel(x, y),
        Mesh2d(rectangle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::BLACK))),
        Transform::from_xyz(pos_x, pos_y, 0.0),
      ));
    }
  }
}

fn update_screen_system(
  r8: Res<Emulator>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  query: Query<(&MeshMaterial2d<ColorMaterial>, &Pixel)>,
) {
  // Only update when the emulator's display has been flagged as updated to reduce work.
  if r8.0.display().updated {
    for (mesh_material, pixel) in &query {
      let color = if r8.0.display().get(pixel.0, pixel.1) {
        Color::WHITE
      } else {
        Color::BLACK
      };
      if let Some(material) = materials.get_mut(&mesh_material.0) {
        material.color = color;
      }
    }
  }
}
