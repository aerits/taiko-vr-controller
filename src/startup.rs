use std::mem::MaybeUninit;

use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::OxrSendActionBindings, add_xr_plugins,
    features::overlay::OxrOverlaySessionEvent, init::OxrInitPlugin, resources::OxrSessionConfig,
    types::OxrExtensions,
};
use bevy_mod_xr::session::{XrSessionCreated, XrTracker};
use bevy_rapier3d::prelude::*;
use bevy_xr_utils::{
    tracking_utils::{
        suggest_action_bindings, TrackingUtilitiesPlugin, XrTrackedLeftGrip, XrTrackedLocalFloor, XrTrackedRightGrip, XrTrackedStage, XrTrackedView
    },
    xr_utils_actions::{XRUtilsAction, XRUtilsActionSet, XRUtilsBinding},
};
use openxr::EnvironmentBlendMode;

use crate::Bachi;

/// set up a simple 3D scene
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
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

pub fn spawn_bachi(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    left: Query<Entity, With<XrTrackedLeftGrip>>,
    right: Query<Entity, With<XrTrackedRightGrip>>,
    mut query: Query<&mut Transform>,
) {
    let left_hand = left.iter().reduce(|acc, cur| cur).unwrap();
    let right_hand = right.iter().reduce(|acc, cur| cur).unwrap();
    let mut spawn = |x: i32, y: f32, z: f32| {
        if x == 0 {
            let rotation = Quat::from_rotation_x((y * 30.0_f32).to_radians())
                * Quat::from_rotation_y((z * 30.0_f32).to_radians())
                * Quat::from_rotation_z(0.0_f32.to_radians());
            cmds.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(rotation),
                
            ))
            .id()
        } else {
            cmds.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.01, 0.01, 0.5))),
                MeshMaterial3d(materials.add(Color::srgb_u8(0, 0, 0))),
                Transform::from_xyz(0.0, 0.0, -0.3),
                // RigidBody::KinematicPositionBased,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::all(),
                ActiveEvents::COLLISION_EVENTS,
                Collider::capsule(Vec3 { x: 0.0, y: 0.0, z: -0.25 }, Vec3 { x: 0.0, y: 0.0, z: 0.25 }, 0.01),
                Bachi {state: crate::BachiState::None},
            ))
            .id()
        }
    };
    let bachi_pivot = spawn(0, -1.0, -1.0);
    let bachi = spawn(1, 1.0, 1.0);
    let bachi_pivot_r = spawn(0, -1.0, 1.0);
    let bachi_r = spawn(1, 1.0, 1.0);
    cmds.entity(bachi_pivot).add_child(bachi);
    cmds.entity(left_hand).add_child(bachi_pivot);
    cmds.entity(bachi_pivot_r).add_child(bachi_r);
    cmds.entity(right_hand).add_child(bachi_pivot_r);
}

pub fn spawn_hands(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let left = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            XrTrackedLeftGrip,
            XrTracker,
        ))
        .id();
    let right = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            XrTrackedRightGrip,
            XrTracker,
        ))
        .id();
    //head
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 144))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        XrTrackedView,
        XrTracker,
    ));
    //local_floor emulated
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.1, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb_u8(144, 255, 144))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        XrTrackedLocalFloor,
        XrTracker,
    ));

    // cmds.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(0.5, 0.1, 0.5))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(144, 255, 255))),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    //     XrTrackedStage,
    //     XrTracker,
    // ));
}