use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::mesh::shape::Box as BevyBox;
use bevy::render::render_graph::base::camera::CAMERA_3D;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_rapier3d::na::{Isometry3, Vector3};
use bevy_rapier3d::physics::{RapierConfiguration, RapierPhysicsPlugin};
use bevy_rapier3d::rapier::dynamics::{RigidBodyBuilder, RigidBodySet};
use bevy_rapier3d::rapier::geometry::ColliderBuilder;
use bevy_rapier3d::rapier::na::Vector;

#[allow(unused_imports)]
use bevy_rapier3d::render::RapierRenderPlugin;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(DebugOverlayTimer(Timer::from_seconds(0.2, true)))
        .init_resource::<Player>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        // .add_plugin(RapierRenderPlugin)
        .insert_resource(RapierConfiguration {
            gravity: -Vector::y(),
            ..Default::default()
        })
        .add_startup_system(setup_lighting.system())
        .add_startup_system(setup_cameras.system())
        .add_startup_system(setup_debug_overlay.system())
        .add_startup_system(setup_world.system())
        .add_plugin(FlyCameraPlugin)
        .add_system(toggle_button_system.system())
        .add_system(update_player_camera.system())
        .add_system(debug_overlay.system())
        .run();
}

#[derive(Default)]
struct Player {
    location: Vec3,
    rotation: Quat,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "location = ({}, {}, {})\nrotation = ({}, {}, {}, {})",
            self.location.x,
            self.location.y,
            self.location.z,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
            self.rotation.w,
        )
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}

fn setup_cameras(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(CAMERA_3D.to_string()),
                ..Default::default()
            },
            perspective_projection: Default::default(),
            visible_entities: Default::default(),
            transform: Transform::from_translation(Vec3::new(0., 2.5, 0.)),
            global_transform: Default::default(),
        })
        .insert(FlyCamera::default());

    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup_debug_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "welcome",
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.5, 0.5, 1.0),
            },
            Default::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane will spawn @ (0.0, 1.0, 0.0)
    let plane_transform = Transform::from_translation(Vec3::Y);

    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 128.0 })),
            material: materials.add(Color::RED.into()),
            transform: plane_transform,
            ..Default::default()
        })
        .insert(RigidBodyBuilder::new_static().translation(
            plane_transform.translation.x,
            plane_transform.translation.y,
            plane_transform.translation.z,
        ))
        .insert(ColliderBuilder::cuboid(64., 0., 64.));

    for i in 1..10 {
        let cube_transform =
            Transform::from_translation(Vec3::Y + Vec3::new(0.0, 1.5 + ((i as f32) * 2.), -10.0));
        commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(if i % 2 == 0 {
                    Color::GREEN.into()
                } else {
                    Color::BLUE.into()
                }),
                transform: cube_transform,
                ..Default::default()
            })
            .insert(RigidBodyBuilder::new_dynamic().translation(
                cube_transform.translation.x,
                cube_transform.translation.y,
                cube_transform.translation.z,
            ))
            .insert(ColliderBuilder::cuboid(0.5, 0.5, 0.5));
    }

    // player hitbox
    let hitbox_transform = Transform::from_translation(Vec3::Y + Vec3::new(0., 5., 0.));
    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(BevyBox::new(1., 2., 1.).into()),
            material: materials.add(Color::rgba(1.0, 0.9, 0.9, 0.0).into()), // invisible material
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBodyBuilder::new_kinematic().translation(
            hitbox_transform.translation.x,
            hitbox_transform.translation.y,
            hitbox_transform.translation.z,
        ))
        .insert(ColliderBuilder::cuboid(0.5, 1., 0.5));
}

fn toggle_button_system(
    mut windows: ResMut<Windows>,
    button_event: Res<Input<MouseButton>>,
    keyboard_event: Res<Input<KeyCode>>,
    mut query: Query<&mut FlyCamera>,
) {
    for mut options in query.iter_mut() {
        let window = windows.get_primary_mut().unwrap();

        if button_event.just_pressed(MouseButton::Left) {
            if !options.enabled {
                options.enabled = true
            }

            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
        }

        if keyboard_event.just_pressed(KeyCode::Escape) {
            if options.enabled {
                options.enabled = false
            }

            window.set_cursor_lock_mode(false);
            window.set_cursor_visibility(true);
        }
    }
}

fn update_player_camera(
    mut player: ResMut<Player>,
    query: Query<(&FlyCamera, &GlobalTransform)>,
    mut rigidbodies: ResMut<RigidBodySet>,
) {
    for (_, transform) in query.iter() {
        let location_changed = player.location != transform.translation;
        let rotation_changed = player.rotation != transform.rotation;

        if location_changed {
            player.location = transform.translation;
            for (_, body) in rigidbodies.iter_mut() {
                if body.is_kinematic() {
                    body.set_next_kinematic_position(Isometry3::new(
                        Vector3::new(player.location.x, player.location.y, player.location.z),
                        Vector3::new(0., 0., 0.),
                    ));
                }
            }
        }

        if rotation_changed {
            player.rotation = transform.rotation;
        }
    }
}

struct DebugOverlayTimer(Timer);

fn debug_overlay(
    time: Res<Time>,
    mut timer: ResMut<DebugOverlayTimer>,
    player: Res<Player>,
    mut query: Query<&mut Text>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("{}", *player);
    }
}
