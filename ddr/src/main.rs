use std::{error::Error, fs, time::SystemTime};

use bevy::prelude::*;
use bevy_mod_openxr::{
    add_xr_plugins, exts::OxrExtensions, init::OxrInitPlugin, resources::OxrSessionConfig,
};
use bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin;
use bevy_mod_xr::session::XrTracker;
use bevy_xr_utils::{
    generic_tracker::{GenericTracker, GenericTrackerGizmoPlugin},
    mndx_xdev_spaces_trackers::MonadoXDevSpacesPlugin,
};
use openxr::EnvironmentBlendMode;
use serde::Deserialize;
fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).set(OxrInitPlugin {
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_hand_tracking();
                exts.other.push("XR_MNDX_xdev_space".to_string());
                exts
            },
            ..Default::default()
        }))
        .insert_resource(OxrSessionConfig {
            blend_mode_preference: vec![
                EnvironmentBlendMode::ALPHA_BLEND,
                EnvironmentBlendMode::ADDITIVE,
                EnvironmentBlendMode::OPAQUE,
            ],
            ..default()
        })
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins((
            HandGizmosPlugin,
            GenericTrackerGizmoPlugin,
            MonadoXDevSpacesPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, tracker_status)
        .add_systems(Update, init_tracker)
        .add_systems(Update, reload_sets)
        .run()
}

#[derive(Component)]
struct Foot {
    last_pos: Vec3,
    last_vel: Vec3,
}

#[derive(Component, Deserialize, Debug)]
struct Settings {
    acc_factor: f32,
    vel_factor: f32,
}

fn init_tracker(q: Query<Entity, (With<GenericTracker>, Without<Foot>)>, mut cmds: Commands) {
    for e in q {
        cmds.entity(e).insert(Foot {
            last_pos: Vec3::ZERO,
            last_vel: Vec3::ZERO,
        });
        println!("inserted foot");
    }
}

fn reload_sets(mut sets: Query<&mut Settings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyR)
        && let Ok(x) = load_data()
        && let Some(mut o) = sets.iter_mut().next()
    {
        o.acc_factor = x.acc_factor;
        o.vel_factor = x.vel_factor;
    }
}

fn load_data() -> Result<Settings, Box<dyn Error>> {
    let data = fs::read_to_string("./settings.json")?;
    let setts: Settings = serde_json::from_str(&data)?;
    return Ok(setts);
}

fn tracker_status(
    mut q: Query<(&GlobalTransform, &mut Foot), With<GenericTracker>>,
    sets: Query<&Settings>,
) {
    let sets = sets.iter().next().unwrap();
    for (t, mut f) in &mut q {
        let curr_pos = t.translation();
        let last_pos = f.last_pos;
        let last_vel = f.last_vel;

        let v = curr_pos - last_pos;
        let a = v - last_vel;
        let y_vel = v.y;
        let y_acc = a.y;
        // if y-acceleration is negative and y-velocity is negative, then
        // the foot hit the ground
        if y_acc < sets.acc_factor && y_vel < sets.vel_factor {
            println!("foot hit floor at {:?}", SystemTime::now())
        }

        f.last_vel = v;
        f.last_pos = curr_pos;
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));
    // // cube
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    // ));
    // light
    let setts: Settings = load_data().unwrap();
    commands.spawn(setts);
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
