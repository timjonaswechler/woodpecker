use crate::{
    game::{Game, GameCamera},
    level::{LevelAssets, TEAL, ground},
};
use bevy::{
    camera::visibility::RenderLayers,
    input_focus::InputFocus,
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility},
    prelude::*,
    text::EditableText,
    world_serialization::WorldInstanceReady,
};

pub const ENTRY: Vec2 = Vec2::new(0.0, -17.0);
pub const TARGETS: [Vec3; 3] = [
    Vec3::new(-8.0, 1.65, -22.9),
    Vec3::new(0.0, 1.65, -22.9),
    Vec3::new(8.0, 1.65, -22.9),
];
pub const ROUND_SECONDS: f32 = 30.0;

pub fn in_range(position: Vec2) -> bool {
    position.y < -14.4
}

#[derive(Resource)]
pub struct Practice {
    pub running: bool,
    pub remaining: f32,
    pub shots: u32,
    pub hits: u32,
    pub score: u32,
    pub best: u32,
    pub respawn: [f32; 3],
    pub cooldown: f32,
    pub hit_flash: f32,
    pub recoil: f32,
    pub tracer: Option<(Vec3, Vec3, f32)>,
    was_captured: bool,
}

impl Default for Practice {
    fn default() -> Self {
        Self {
            running: false,
            remaining: ROUND_SECONDS,
            shots: 0,
            hits: 0,
            score: 0,
            best: 0,
            respawn: [0.0; 3],
            cooldown: 0.0,
            hit_flash: 0.0,
            recoil: 0.0,
            tracer: None,
            was_captured: false,
        }
    }
}

impl Practice {
    pub fn reset(&mut self) {
        *self = Self {
            best: self.best,
            ..default()
        };
    }
    pub fn start(&mut self) {
        self.reset();
        self.running = true;
    }
    pub fn accuracy(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            100.0 * self.hits as f32 / self.shots as f32
        }
    }
    pub fn finish(&mut self) {
        self.running = false;
        self.remaining = 0.0;
        self.best = self.best.max(self.score);
    }

    pub fn hit(&mut self, index: usize, radius: f32) -> Option<u32> {
        if !self.running || self.respawn[index] > 0.0 {
            return None;
        }
        let points = if radius < 0.25 {
            50
        } else if radius < 0.55 {
            25
        } else {
            10
        };
        self.score += points;
        self.hits += 1;
        self.respawn[index] = 1.2;
        self.hit_flash = 0.18;
        Some(points)
    }
}

#[derive(Component)]
pub struct Target(pub usize);
#[derive(Component)]
pub struct TargetPart(pub usize);
#[derive(Component)]
pub struct Weapon;
#[derive(Component)]
pub struct WeaponCamera;
#[derive(Component)]
pub struct ShotIgnore;
#[derive(Component)]
pub struct Debris {
    velocity: Vec3,
    life: f32,
}
#[derive(Resource, Default)]
pub struct Fragments(Vec<Handle<bevy::world_serialization::WorldAsset>>);

pub fn spawn_range(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stand_material = materials.add(Color::srgb(0.38, 0.45, 0.48));
    for (index, center) in TARGETS.into_iter().enumerate() {
        // KayKit's complete stand has a baked-in disk. Use a plain support so
        // only the destructible target remains visible above it.
        for (size, position) in [
            (Vec3::new(1.3, 0.2, 1.0), Vec3::new(center.x, 0.1, -23.4)),
            (
                Vec3::new(0.22, 1.45, 0.22),
                Vec3::new(center.x, 0.825, -23.4),
            ),
            (Vec3::new(0.8, 0.16, 0.8), Vec3::new(center.x, 0.85, -23.15)),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(size))),
                MeshMaterial3d(stand_material.clone()),
                Transform::from_translation(position),
            ));
        }
        let scene = assets.load(GltfAssetLabel::Scene(0).from_asset("Assets/gltf/target.gltf"));
        tracked.0.push(scene.clone().untyped());
        commands.spawn((
            Name::new(format!("Zielscheibe {}", index + 1)),
            Target(index),
            WorldAssetRoot(scene),
            Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.8, 0.035, 1.6))),
            MeshMaterial3d(materials.add(Color::srgb(0.06, 0.38, 0.34))),
            Transform::from_xyz(center.x, 0.025, -17.0),
        ));
        // A yellow firing line is visible in both camera modes.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(6.8, 0.045, 0.12))),
            MeshMaterial3d(materials.add(crate::level::GOLD)),
            Transform::from_xyz(center.x, 0.03, -18.0),
        ));
    }
    let mut fragments = Fragments::default();
    for letter in ['A', 'B', 'C', 'D', 'E', 'F'] {
        let scene = assets.load(
            GltfAssetLabel::Scene(0).from_asset(format!("Assets/gltf/target_pieces_{letter}.gltf")),
        );
        tracked.0.push(scene.clone().untyped());
        fragments.0.push(scene);
    }
    commands.insert_resource(fragments);
    let weapon = assets.load(GltfAssetLabel::Scene(0).from_asset("custom/training_blaster.glb"));
    tracked.0.push(weapon.clone().untyped());
    commands.spawn((
        Name::new("Blender Trainingsblaster"),
        Weapon,
        ShotIgnore,
        WorldAssetRoot(weapon),
        Transform::default(),
        Visibility::Hidden,
        RenderLayers::layer(1),
    ));
    commands.spawn((
        Name::new("Blaster-Kamera"),
        WeaponCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            is_active: false,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 65_f32.to_radians(),
            near: 0.01,
            ..default()
        }),
        RenderLayers::layer(1),
        Msaa::Sample4,
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
    ));
}

pub fn tag_scene_parts(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    roots: Query<(Option<&Target>, Has<Weapon>, Has<ShotIgnore>)>,
) {
    let Ok((target, weapon, ignored)) = roots.get(ready.entity) else {
        return;
    };
    for child in children.iter_descendants(ready.entity) {
        if let Some(target) = target {
            commands.entity(child).insert(TargetPart(target.0));
        }
        if weapon {
            commands.entity(child).insert(RenderLayers::layer(1));
        }
        if ignored {
            commands.entity(child).insert(ShotIgnore);
        }
    }
}

pub fn tick_practice(
    time: Res<Time>,
    window: Single<&Window>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    mut practice: ResMut<Practice>,
    mut game: ResMut<Game>,
) {
    let dt = time.delta_secs().min(0.05);
    practice.cooldown = (practice.cooldown - dt).max(0.0);
    practice.hit_flash = (practice.hit_flash - dt).max(0.0);
    practice.recoil = (practice.recoil - dt * 7.0).max(0.0);
    if let Some((_, _, ref mut ttl)) = practice.tracer {
        *ttl -= dt;
        if *ttl <= 0.0 {
            practice.tracer = None;
        }
    }
    if !practice.running
        || game.menu
        || !window.focused
        || !game.first_person
        || !game.captured
        || focus.get().is_some_and(|e| inputs.contains(e))
    {
        return;
    }
    if !in_range(game.position) {
        practice.finish();
        game.status = format!(
            "Übung beendet: Schießstand verlassen. {} Punkte.",
            practice.score
        );
        return;
    }
    practice.remaining = (practice.remaining - dt).max(0.0);
    for timer in &mut practice.respawn {
        *timer = (*timer - dt).max(0.0);
    }
    if practice.remaining <= 0.0 {
        practice.finish();
        game.status = format!(
            "Zeit vorbei! {} Punkte · {}/{} Treffer · {:.0}% Genauigkeit. E für eine neue Übung.",
            practice.score,
            practice.hits,
            practice.shots,
            practice.accuracy()
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shooting reads Bevy input, scene meshes and target metadata."
)]
pub fn shoot(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    mut game: ResMut<Game>,
    mut practice: ResMut<Practice>,
    mut raycast: MeshRayCast,
    ignored: Query<(), With<ShotIgnore>>,
    parts: Query<&TargetPart>,
    fragments: Res<Fragments>,
) {
    // The click that captures the cursor or resumes the menu never fires a shot.
    let was_captured = practice.was_captured;
    practice.was_captured = game.first_person && game.captured && !game.menu;
    if !buttons.just_pressed(MouseButton::Left)
        || !game.ready
        || !game.first_person
        || !game.captured
        || game.menu
        || !window.focused
        || !in_range(game.position)
        || !was_captured
        || focus.get().is_some_and(|e| inputs.contains(e))
    {
        return;
    }
    if !practice.running {
        game.status = "Drücke E, um die 30-Sekunden-Zielübung zu starten.".into();
        return;
    }
    if practice.cooldown > 0.0 {
        return;
    }
    practice.shots += 1;
    practice.cooldown = 0.16;
    practice.recoil = 1.0;
    let rotation = Quat::from_euler(EulerRot::YXZ, game.yaw, game.pitch, 0.0);
    let origin = ground(game.position, 1.5);
    let direction = rotation * Vec3::NEG_Z;
    let filter = |entity| {
        !ignored.contains(entity)
            && parts
                .get(entity)
                .map_or(true, |part| practice.respawn[part.0] <= 0.0)
    };
    let hit = raycast
        .cast_ray(
            Ray3d::new(origin, Dir3::new_unchecked(direction)),
            &MeshRayCastSettings::default()
                .with_visibility(RayCastVisibility::Visible)
                .with_filter(&filter),
        )
        .first()
        .map(|(entity, hit)| (*entity, hit.point, hit.distance));
    let impact = hit
        .filter(|(_, _, distance)| *distance <= 80.0)
        .map_or(origin + direction * 80.0, |(_, point, _)| point);
    practice.tracer = Some((
        origin + rotation * Vec3::new(0.25, -0.19, -0.84),
        impact,
        0.1,
    ));
    let Some((entity, point, distance)) = hit else {
        return;
    };
    if distance > 80.0 {
        return;
    }
    let Ok(part) = parts.get(entity) else {
        return;
    };
    let center = TARGETS[part.0];
    let radius = (point - center).xy().length() / 0.8;
    if let Some(points) = practice.hit(part.0, radius) {
        game.status = format!(
            "{} +{points} Punkte · E startet eine neue Übung.",
            if points == 50 {
                "Volltreffer!"
            } else {
                "Treffer!"
            }
        );
        for (index, fragment) in fragments.0.iter().enumerate() {
            let angle = index as f32 * std::f32::consts::TAU / 6.0;
            commands.spawn((
                WorldAssetRoot(fragment.clone()),
                ShotIgnore,
                Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
                Debris {
                    velocity: Vec3::new(angle.sin() * 1.4, 1.8 + angle.cos(), 1.2),
                    life: 1.0,
                },
            ));
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "The render queries must be disjoint."
)]
pub fn update_visuals(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    practice: Res<Practice>,
    camera: Single<
        &Transform,
        (
            With<GameCamera>,
            Without<Weapon>,
            Without<WeaponCamera>,
            Without<Debris>,
        ),
    >,
    mut gun: Single<
        (&mut Transform, &mut Visibility),
        (With<Weapon>, Without<WeaponCamera>, Without<Debris>),
    >,
    mut overlay: Single<
        (&mut Transform, &mut Camera),
        (With<WeaponCamera>, Without<Weapon>, Without<Debris>),
    >,
    mut targets: Query<(&Target, &mut Visibility), Without<Weapon>>,
    mut debris: Query<(Entity, &mut Transform, &mut Debris)>,
    mut gizmos: Gizmos,
) {
    let visible = game.first_person && game.ready && !game.menu;
    *gun.1 = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay.1.is_active = visible;
    *overlay.0 = **camera;
    let bob = if game.captured {
        (game.walked * 6.0).sin() * 0.008
    } else {
        0.0
    };
    *gun.0 = Transform::from_translation(
        camera.translation
            + camera.rotation * Vec3::new(0.25, -0.25 + bob, -0.42 + practice.recoil * 0.075),
    )
    .with_rotation(camera.rotation * Quat::from_rotation_x(practice.recoil * 0.10))
    .with_scale(Vec3::splat(0.55));
    for (target, mut visibility) in &mut targets {
        *visibility = if practice.respawn[target.0] > 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut piece) in &mut debris {
        if game.menu {
            continue;
        }
        piece.life -= dt;
        piece.velocity.y -= 6.0 * dt;
        transform.translation += piece.velocity * dt;
        transform.rotate_local_x(dt * 2.8);
        if piece.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    if let Some((start, end, _)) = practice.tracer {
        gizmos.line(start, end, TEAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_pauses_for_focus_menu_view_and_text_input_then_expires() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<InputFocus>()
            .init_resource::<Practice>()
            .insert_resource(Game {
                position: ENTRY,
                ready: true,
                first_person: true,
                captured: true,
                ..default()
            })
            .add_systems(Update, tick_practice);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(50));
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        let field = app.world_mut().spawn(EditableText::new("Name")).id();
        app.world_mut().resource_mut::<Practice>().start();

        app.world_mut().run_schedule(Update);
        assert_eq!(app.world().resource::<Practice>().remaining, ROUND_SECONDS);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        for reason in 0..4 {
            match reason {
                0 => app.world_mut().resource_mut::<Game>().menu = true,
                1 => app.world_mut().resource_mut::<Game>().captured = false,
                2 => app.world_mut().resource_mut::<Game>().first_person = false,
                _ => app
                    .world_mut()
                    .resource_mut::<InputFocus>()
                    .set(field, bevy::input_focus::FocusCause::Pressed),
            }
            app.world_mut().run_schedule(Update);
            assert_eq!(app.world().resource::<Practice>().remaining, ROUND_SECONDS);
            let mut game = app.world_mut().resource_mut::<Game>();
            game.menu = false;
            game.captured = true;
            game.first_person = true;
        }
        app.world_mut().resource_mut::<InputFocus>().clear();
        {
            let mut practice = app.world_mut().resource_mut::<Practice>();
            practice.remaining = 0.08;
            practice.hit(1, 0.1);
        }
        app.world_mut().run_schedule(Update);
        assert!(app.world().resource::<Practice>().running);
        app.world_mut().run_schedule(Update);
        let practice = app.world().resource::<Practice>();
        assert!(!practice.running);
        assert_eq!(practice.remaining, 0.0);
        assert_eq!(practice.best, 50);
    }

    #[test]
    fn scoring_requires_a_round_and_prevents_duplicate_hits() {
        let mut practice = Practice::default();
        assert_eq!(practice.hit(0, 0.0), None);
        practice.start();
        assert_eq!(practice.hit(0, 0.0), Some(50));
        assert_eq!(practice.hit(0, 0.0), None);
        assert_eq!(practice.hit(1, 0.4), Some(25));
        assert_eq!(practice.hit(2, 0.8), Some(10));
        assert_eq!(practice.score, 85);
        assert_eq!(practice.hits, 3);
        practice.finish();
        assert_eq!(practice.best, 85);
        assert_eq!(practice.hit(0, 0.0), None);
        practice.start();
        assert_eq!(practice.score, 0);
        assert_eq!(practice.best, 85);
        assert_eq!(practice.respawn, [0.0; 3]);
    }
}
