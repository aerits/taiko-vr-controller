//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::OxrSendActionBindings, add_xr_plugins, init::OxrInitPlugin,
    resources::OxrSessionConfig, types::OxrExtensions,
};
use bevy_mod_xr::session::XrSessionCreated;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_xr_utils::tracking_utils::{
    suggest_action_bindings, TrackingUtilitiesPlugin, XrTrackedView,
};
use openxr::EnvironmentBlendMode;
use std::fs;

use crate::{
    keyb::{don_fn, ka_fn},
    startup::*,
};

#[derive(Component)]
struct Taiko;

#[derive(Debug, PartialEq, Eq)]
enum BachiState {
    Don,
    Ka,
    None,
}
#[derive(Component, Debug)]
struct Bachi {
    state: BachiState,
    parent: Entity,
}

mod keyb;
mod startup;

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).build().set(OxrInitPlugin {
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_hand_tracking();
                exts.extx_overlay = true;
                exts
            },
            ..OxrInitPlugin::default()
        }))
        .insert_resource(OxrSessionConfig {
            blend_modes: Some({
                vec![
                    EnvironmentBlendMode::ALPHA_BLEND,
                    EnvironmentBlendMode::OPAQUE,
                ]
            }),
            ..OxrSessionConfig::default()
        })
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(ObjPlugin)
        // .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(TrackingUtilitiesPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .add_systems(XrSessionCreated, (spawn_hands, spawn_bachi).chain())
        .add_systems(OxrSendActionBindings, suggest_action_bindings)
        .add_systems(Update, bachi_force)
        .add_systems(FixedUpdate, display_events)
        .run();
}

fn bachi_force(
    bachis: Query<(&Bachi, Entity, &GlobalTransform, &mut ExternalForce)>,
    mut cmds: Commands,
    position: Query<&GlobalTransform>,
) {
    for (bachi, bachi_e, bachi_t, mut extforce) in bachis {
        let parent_t = position.get(bachi.parent).unwrap();
        let strength = 1f32;
        let offset = vec3(0.0, 0.0, 0.3);
        let force = (parent_t.translation() + offset - bachi_t.translation()).normalize_or_zero()
            * strength
            * (parent_t.translation() + offset).distance(bachi_t.translation());
        let torque = offset;
        extforce.force = force;
        extforce.torque = torque;
    }
}

fn is_<T: Component>(query: Query<&T>, entity: &Entity, entity1: &Entity) -> Option<Entity> {
    if query.get(*entity).is_ok() {
        Some(*entity)
    } else if query.get(*entity1).is_ok() {
        Some(*entity1)
    } else {
        None
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    is_don: Query<&Don>,
    is_ka: Query<&Ka>,
    mut bachi: Query<&mut Bachi>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity, entity1, _collision_event_flags) => {
                let mut bachi = if let Ok(b) = bachi.get_mut(*entity) {
                    b
                } else if let Ok(b) = bachi.get_mut(*entity1) {
                    b
                } else {
                    println!("no bachi in collision");
                    return;
                };
                println!("{:?}", bachi);
                if is_(is_don, entity, entity1).is_some() && bachi.state == BachiState::None {
                    bachi.state = BachiState::Don;
                    don_fn();
                } else if is_(is_ka, entity, entity1).is_some() && bachi.state == BachiState::None {
                    bachi.state = BachiState::Ka;
                    ka_fn();
                }
                ()
            }
            CollisionEvent::Stopped(entity, entity1, _collision_event_flags) => {
                let mut bachi = if let Ok(b) = bachi.get_mut(*entity) {
                    b
                } else if let Ok(b) = bachi.get_mut(*entity1) {
                    b
                } else {
                    println!("no bachi in collision");
                    return;
                };
                if is_(is_don, entity, entity1).is_some() {
                    bachi.state = BachiState::None;
                } else if is_(is_ka, entity, entity1).is_some() {
                    bachi.state = BachiState::None;
                }
                ()
            }
        }
    }

    for contact_force_event in contact_force_events.read() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

#[derive(Component, Clone, Copy)]
struct Ka;
#[derive(Component, Clone, Copy)]
struct Don;

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    headset: Query<&Transform, With<XrTrackedView>>,
    old_taiko: Query<Entity, With<Taiko>>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }

    // boilerplate
    for e in old_taiko {
        commands.entity(e).despawn();
    }
    let mut ent = None;
    for e in headset {
        ent = Some(e)
    }
    let e = match ent {
        Some(x) => x,
        None => {
            println!("no headset");
            return;
        }
    };
    let flat_vec = Vec3 {
        x: 1.0,
        y: 0.0,
        z: 1.0,
    };

    // compute position in front of headset to place taiko
    let taiko_position = {
        let forward_direction = (e.local_z() * -1.0) * flat_vec;
        let distance = 0.7;
        let cube_position = e.translation * flat_vec + forward_direction.normalize() * distance;
        cube_position
    };

    let taiko_model = commands
        .spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("taiko.glb")),
        ))
        .id();

    // create the don and ka hitboxes
    let hitboxes = [
        "don_left.obj",
        "don_right.obj",
        "ka_rleft.obj",
        "ka_right.obj",
    ];
    let mut ids: Vec<Entity> = hitboxes
        .iter()
        .map(|s| {
            commands
                .spawn(hitbox_gen(&("./assets/".to_owned() + s)))
                .id()
        })
        .collect();
    for (i, id) in ids.iter_mut().enumerate() {
        if i < 2 {
            commands.entity(*id).insert(Don);
        } else {
            commands.entity(*id).insert(Ka);
        }
    }

    // parent taiko that has the model and the hitboxes
    commands
        .spawn((
            Transform::from_translation(taiko_position)
                .looking_at(e.translation * flat_vec, Vec3::Y),
            Taiko,
        ))
        .add_child(taiko_model)
        .add_children(&ids);
}

/// read a .obj file and turn it into a collider
/// its kinda scuffed though
fn hitbox_gen(
    file: &str,
) -> (
    Transform,
    RigidBody,
    Collider,
    ActiveEvents,
    ActiveCollisionTypes,
) {
    let contents = fs::read_to_string(file).expect("Should have been able to read the file");
    let mut verts = Vec::new();
    let mut tris = Vec::new();
    for line in contents.lines() {
        if line.starts_with("v ") {
            let split: Vec<f32> = line[2..].split(" ").map(|x| x.parse().unwrap()).collect();
            verts.push(Vec3 {
                x: split[0],
                y: split[1],
                z: split[2],
            })
        }
        if line.starts_with("f") {
            let split: Vec<u32> = line[2..]
                .split(" ")
                .map(|x| x.split("/").collect::<Vec<&str>>()[0])
                .map(|x| x.parse::<u32>().unwrap())
                .collect();
            tris.push([split[0] - 1, split[1] - 1, split[2] - 1]);
        }
    }
    println!("{}, {}", verts.len(), tris.len());
    (
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::trimesh(verts, tris).unwrap(),
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::default() | ActiveCollisionTypes::all(),
    )
}
