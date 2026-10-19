//! Character geometry, authored eye point and pose.
use super::*;
pub const MODEL_SCALE: f32 = 0.78;
/// Maximum vertical visual offset, including the torso's gait, in world units.
pub const MODEL_BOB: f32 = 0.07;
pub(crate) const BODY_BOB: f32 = 0.1;

#[derive(Component)]
pub struct PlayerModel;

/// Authored local rest height: animate the torso without moving the foot/base.
#[derive(Component)]
pub(crate) struct BodyMotion {
    pub(super) rest_y: f32,
}

pub fn limb_rotation(name: &str) -> Quat {
    match name {
        "Dummy_ArmLeft" => Quat::from_rotation_z(-70_f32.to_radians()),
        "Dummy_ArmRight" => Quat::from_rotation_z(70_f32.to_radians()),
        _ => Quat::IDENTITY,
    }
}

pub fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<crate::level::LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut scene = crate::level::scene::Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    let player = scene.model(
        "meshes/characters/Dummy_Base.gltf".into(),
        Transform::from_translation(START).with_scale(Vec3::splat(crate::player::MODEL_SCALE)),
    );
    scene.commands.entity(player).insert(PlayerModel);
}

pub fn pose_player(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<(), With<PlayerModel>>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Transform), Without<PlayerModel>>,
) {
    if !roots.contains(ready.entity) {
        return;
    }
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut transform)) = nodes.get_mut(entity) {
            pose_part(&mut commands, entity, name.as_str(), &mut transform);
        }
    }
}

pub(crate) fn pose_part(
    commands: &mut Commands,
    entity: Entity,
    name: &str,
    transform: &mut Transform,
) {
    transform.rotation *= limb_rotation(name);
    match name {
        "Dummy_Body" => {
            commands.entity(entity).insert(BodyMotion {
                rest_y: transform.translation.y,
            });
        }
        "Dummy_Head" => camera::spawn_eye(commands, entity),
        _ => (),
    }
}

pub fn update_model(
    player: Res<Player>,
    nav: Res<Navigation>,
    view: Res<ViewState>,
    settings: Res<Settings>,
    mut model: Single<(&mut Transform, &mut Visibility), With<PlayerModel>>,
    mut bodies: Query<(&BodyMotion, &mut Transform), Without<PlayerModel>>,
    mut gizmos: Gizmos,
) {
    let (ref mut player_transform, ref mut visibility) = *model;
    player_transform.translation = ground(player.position, player.height + MODEL_BOB - BODY_BOB);
    // Distance drives the grounded gait in both perspectives and for keyboard/click
    // routes. In the air, only the root follows the jump; the torso stays at rest.
    // Convert world amplitude to model units once. Head, arms and eye follow it.
    let bob = if player.is_grounded(&nav) {
        (player.walked * 1.50).sin().abs() * BODY_BOB / MODEL_SCALE
    } else {
        0.0
    };
    for (body, mut transform) in &mut bodies {
        transform.translation.y = body.rest_y + bob;
    }
    // Dummy_Base faces local +Z; movement and the FPV camera use local -Z.
    player_transform.rotation = Quat::from_rotation_y(player.yaw + std::f32::consts::PI);
    **visibility = if view.first_person() {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    if settings.show_path() && !view.first_person() && !player.route.is_empty() {
        gizmos.linestrip(
            std::iter::once(player.feet())
                .chain(player.route.iter().copied())
                .map(|p| p + Vec3::Y * 0.12),
            level::TEAL,
        );
        if let Some(&goal) = player.route.back() {
            gizmos.circle(
                Isometry3d::new(
                    goal + Vec3::Y * 0.12,
                    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ),
                0.4,
                level::TEAL,
            );
        }
    }
    if !view.first_person() {
        gizmos.circle(
            Isometry3d::new(
                ground(player.position, player.height + 0.1),
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            0.65,
            level::TEAL,
        );
    }
}
