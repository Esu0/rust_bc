#![allow(dead_code)]

mod database;
mod material;
use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{Anchor, Material2dPlugin},
};

#[derive(Component)]
struct Enemy;

struct BattleCatsUnit;

impl Plugin for BattleCatsUnit {
    fn build(&self, app: &mut App) {
        app.add_system(animate_system);
    }
}

#[derive(Resource)]
struct Entities {
    parent: Entity,
    child: Entity,
}

#[derive(Resource)]
struct GrandChild(Entity);
fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    let texture = asset_server.load("org/enemy/000/000_e.png");

    let mut sprite = SpriteBundle {
        texture,
        sprite: Sprite {
            color: Color::rgba(1., 1., 1., 1.),
            rect: Some(Rect::new(30., 30., 60., 70.)),
            custom_size: Some(Vec2::new(100., 100.)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 0.)
            .with_scale(Vec3::new(-2., 1., 1.))
            .with_rotation(Quat::from_rotation_z(f32::to_radians(-90.))),
        ..default()
    };

    // let mut origin = sprite.clone();
    // let mut sprite2 = sprite.clone();
    // origin.transform.translation += Vec3::new(-300., 0., 0.);
    // sprite.transform.translation += Vec3::new(100., 100., 0.);
    // sprite2.transform.translation += Vec3::new(100., 0., 0.);
    // let origin = commands.spawn(origin).id();
    let sprite = commands.spawn(sprite).id();
    // let sprite2 = commands.spawn(sprite2).id();
    // commands.insert_resource(Entities {
    //     parent: origin,
    //     child: sprite,
    // });
    // commands.entity(origin).add_child(sprite);
    // commands.entity(sprite).add_child(sprite2);
    // commands.insert_resource(GrandChild(sprite2));
}

fn animate_system(mut query: Query<&mut TextureAtlasSprite>) {
    for mut sprite in &mut query {
        sprite.index = (sprite.index + 1) % 3;
    }
}

// fn test_system(test: Res<Test>, mut images: ResMut<Assets<Image>>) {
//     let image = images.get_mut(&test.img).unwrap();
//     println!("{:#?}", image.texture_descriptor);
// }

fn toggle_child(
    mut entities: ResMut<Entities>,
    mut commands: Commands,
    mut query: Query<&mut Transform>,
    grandchild: Res<GrandChild>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::A) {
        let Entities { parent, child } = entities.as_mut();
        commands.entity(*child).remove_parent().add_child(*parent);
        std::mem::swap(parent, child);
    }
    if input.just_pressed(KeyCode::Z) {
        let Entities { parent, child } = entities.as_mut();
        // commands.entity(*child).remove_parent();
        commands.entity(*parent).set_parent(*child);
        // let mut transform = query.get_mut(*parent).unwrap();
        // transform.translation *= -1.;
        std::mem::swap(parent, child);
    }
    if input.just_pressed(KeyCode::S) {
        commands.entity(grandchild.0).remove_parent();
    }
}

fn draw_alpha(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        texture: server.load("org/unit/693/f/693_f_123_glow1.png"),
        transform: Transform::from_xyz(0., 0., 100.).with_scale(Vec3::new(3., 3., 1.)),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(BattleCatsUnit)
        // .add_startup_system(startup_system)
        // .add_startup_system(draw_alpha)
        // .add_system(test_system)
        // .add_system(toggle_child)
        .add_plugin(database::animation::PluginTemp)
        .insert_resource(ClearColor(Color::GRAY))
        .add_plugin(Material2dPlugin::<material::Glow1Material>::default())
        // .add_startup_system(material::startup)
        // .add_system(material::system)
        .run();
}
