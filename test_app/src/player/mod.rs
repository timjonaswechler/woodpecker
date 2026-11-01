pub(crate) mod actions;
pub(crate) mod blaster;
mod model;
mod movement;
pub(crate) use model::{MODEL_SCALE, pose_player, spawn_player, update_model};
pub(crate) use movement::move_player;
pub(crate) mod camera;

use crate::{
    app::settings::Settings,
    level::definition::START,
    level::navigation::Navigation,
    level::{self, ground},
    player::camera::ViewState,
};
use bevy::prelude::*;
use std::collections::VecDeque;

/// Locomotion state for the demo's single player.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Player {
    position: Vec2,
    yaw: f32,
    height: f32,
    vertical_speed: f32,
    /// Walking/running mode is retained in the air and updated only on the ground.
    sprinting: bool,
    coyote: f32,
    jump_buffer: f32,
    walked: f32,
    route: VecDeque<Vec3>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RouteOutcome {
    Started,
    Airborne,
    NoSurface,
    Unreachable,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: START.xz(),
            yaw: 0.0,
            height: 0.0,
            vertical_speed: 0.0,
            sprinting: false,
            coyote: 0.0,
            jump_buffer: 0.0,
            walked: 0.0,
            route: VecDeque::new(),
        }
    }
}

impl Player {
    pub(crate) fn move_to(&mut self, nav: &Navigation, target: Vec3) -> RouteOutcome {
        if !self.can_start_route(nav) {
            return RouteOutcome::Airborne;
        }
        self.start_route(nav, target)
    }

    fn can_start_route(&self, nav: &Navigation) -> bool {
        (self.height - nav.support(self.position, self.height + 0.02)).abs() <= 0.15
    }

    fn start_route(&mut self, nav: &Navigation, target: Vec3) -> RouteOutcome {
        match nav.path(self.feet(), target) {
            Some(route) => {
                self.follow_route(route);
                RouteOutcome::Started
            }
            None => {
                self.clear_route();
                RouteOutcome::Unreachable
            }
        }
    }
    pub(crate) fn follow_ray(&mut self, nav: &Navigation, ray: Ray3d) -> RouteOutcome {
        if !self.can_start_route(nav) {
            return RouteOutcome::Airborne;
        }
        let Some(target) = nav.pick(ray) else {
            self.clear_route();
            return RouteOutcome::NoSurface;
        };
        self.start_route(nav, target)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn vertical_speed(&self) -> f32 {
        self.vertical_speed
    }
    pub fn sprinting(&self) -> bool {
        self.sprinting
    }
    pub fn walked(&self) -> f32 {
        self.walked
    }
    pub fn route(&self) -> &VecDeque<Vec3> {
        &self.route
    }
    pub fn clear_route(&mut self) {
        self.route.clear();
    }
    pub fn follow_route(&mut self, route: VecDeque<Vec3>) {
        assert!(
            route.iter().all(|p| p.is_finite()),
            "route points must be finite"
        );
        self.route = route;
    }
    pub(super) fn turn(&mut self, delta: f32) {
        if !delta.is_finite() {
            return;
        }
        let next = self.yaw + delta;
        self.yaw = if next.is_finite() {
            next
        } else {
            self.yaw.rem_euclid(std::f32::consts::TAU) + delta.rem_euclid(std::f32::consts::TAU)
        };
    }
    pub fn face_yaw(&mut self, yaw: f32) {
        if yaw.is_finite() {
            self.yaw = yaw;
        }
    }

    pub fn respawn(&mut self, position: Vec2, height: f32) {
        assert!(
            position.is_finite() && height.is_finite(),
            "spawn point must be finite"
        );
        self.position = position;
        self.height = height;
        self.vertical_speed = 0.0;
        self.sprinting = false;
        self.coyote = 0.0;
        self.jump_buffer = 0.0;
        self.route.clear();
    }

    pub fn feet(&self) -> Vec3 {
        ground(self.position, self.height)
    }

    fn is_grounded(&self, nav: &Navigation) -> bool {
        let floor = nav.support(self.position, self.height + 0.02);
        (self.height - floor).abs() < 0.025 && self.vertical_speed <= 0.0
    }

    /// Ground support and ceiling collision are evaluated on the current floor.
    pub fn advance(&mut self, nav: &Navigation, delta: Vec2, dt: f32, jump: bool) {
        assert!(
            delta.is_finite() && dt.is_finite() && dt >= 0.0,
            "invalid movement step"
        );
        let grounded = self.is_grounded(nav);
        self.coyote = if grounded {
            0.10
        } else {
            (self.coyote - dt).max(0.0)
        };
        self.jump_buffer = if jump {
            0.12
        } else {
            (self.jump_buffer - dt).max(0.0)
        };
        if self.coyote > 0.0 && self.jump_buffer > 0.0 {
            self.vertical_speed = 7.5;
            self.coyote = 0.0;
            self.jump_buffer = 0.0;
            self.route.clear();
        }
        let stepping = grounded && self.vertical_speed <= 0.0;
        let old_height = self.height;
        let ceiling = nav.ceiling(
            self.position,
            self.height + crate::level::navigation::BODY_HEIGHT,
        );
        let mut next_height = self.height + self.vertical_speed * dt - 9.0 * dt * dt;
        self.vertical_speed -= 18.0 * dt;
        if next_height + crate::level::navigation::BODY_HEIGHT > ceiling {
            next_height = ceiling - crate::level::navigation::BODY_HEIGHT;
            self.vertical_speed = self.vertical_speed.min(0.0);
        }
        let from = ground(
            self.position,
            if stepping {
                self.height
            } else {
                next_height.max(0.0)
            },
        );
        let moved = nav.move_body(from, delta, stepping);
        self.position = moved.xz();
        self.height = if stepping {
            moved.y + next_height - old_height
        } else {
            next_height
        };
        let landing = nav.support(self.position, old_height.max(moved.y) + 0.025);
        if self.height <= landing && self.vertical_speed <= 0.0 {
            self.height = landing;
            self.vertical_speed = 0.0;
        }
    }
}

pub(crate) fn install(app: &mut App) {
    use crate::app::schedule::Phase;
    app.init_resource::<Player>()
        .init_resource::<camera::ViewState>()
        .init_resource::<blaster::Blaster>()
        .init_resource::<blaster::FiredShot>()
        .add_systems(
            Startup,
            (spawn_player, camera::spawn_camera, blaster::spawn),
        )
        .add_observer(pose_player)
        .add_observer(blaster::tag_scene_parts)
        .add_systems(
            Update,
            actions::handle_actions
                .in_set(Phase::Commands)
                .after(crate::app::flow::handle_actions),
        )
        .add_systems(Update, blaster::tick.in_set(Phase::PrepareWorld))
        .add_systems(
            Update,
            (camera::mouse_look, move_player)
                .chain()
                .in_set(Phase::Movement),
        )
        .add_systems(
            Update,
            (update_model, camera::update_camera)
                .chain()
                .in_set(Phase::View),
        )
        .add_systems(Update, blaster::fire.in_set(Phase::Fire))
        .add_systems(
            Update,
            (camera::sync_cursor, blaster::update_visuals).in_set(Phase::Present),
        );
}
