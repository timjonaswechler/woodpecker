//! Passage regressions use normal mission actions, real body movement and real navigation.
use super::mission::Interaction;
use super::{
    definition as map,
    mission::*,
    navigation::Navigation,
    passages::{self, Passages},
    *,
};
use crate::player::{
    Player,
    camera::{ViewMode, ViewState},
};

struct World {
    nav: Navigation,
    mission: Mission,
    player: Player,
    passages: Passages,
    view: ViewState,
}
impl World {
    fn at(feet: Vec3) -> Self {
        let mut w = Self {
            nav: navigation_world(),
            mission: Mission::default(),
            player: Player::default(),
            passages: Passages::default(),
            view: ViewState::default(),
        };
        w.player.respawn(feet.xz(), feet.y);
        assert!(w.nav.walkable(feet), "initial feet {feet:?}");
        w.passages.discover(&mut w.mission, &w.player, &w.nav);
        w
    }
    fn outside(id: usize) -> Vec3 {
        let p = passages::portal(id);
        Vec3::from(p.center) - Vec3::Y * p.half[1] - Vec3::from(p.inward) * 1.5
    }
    fn inside(id: usize) -> Vec3 {
        let p = passages::portal(id);
        Vec3::from(p.center) - Vec3::Y * p.half[1] + Vec3::from(p.inward)
    }
    fn open(&mut self, id: usize) {
        assert_eq!(
            self.mission
                .interaction(&self.player, &self.nav, &self.view),
            Interaction::Portal(PortalId::ALL[id])
        );
        self.mission
            .try_apply(Interaction::Portal(PortalId::ALL[id]))
            .unwrap();
        for _ in 0..20 {
            self.passages
                .advance(0.05, &self.mission, &mut self.nav, &mut self.player);
        }
        self.passages
            .sync_access(&self.mission, &mut self.player, &self.view, &mut self.nav);
    }
    fn walk(&mut self, goal: Vec3) {
        let route = self
            .nav
            .path(self.player.feet(), goal)
            .unwrap_or_else(|| panic!("no route {:?} -> {goal:?}", self.player.feet()));
        for point in route {
            for _ in 0..5000 {
                if self.player.feet().distance(point) < 0.02 {
                    break;
                }
                self.player.advance(
                    &self.nav,
                    (point.xz() - self.player.position()).clamp_length_max(0.08),
                    1. / 60.,
                    false,
                );
                self.passages
                    .discover(&mut self.mission, &self.player, &self.nav);
                self.passages.sync_access(
                    &self.mission,
                    &mut self.player,
                    &self.view,
                    &mut self.nav,
                );
                self.mission.collect_blueprints(&self.player, &self.nav);
            }
            assert!(
                self.player.feet().distance(point) < 0.03,
                "body {:?} cannot reach {point:?}",
                self.player.feet()
            );
        }
    }
}

#[test]
fn closed_secrets_have_no_hidden_floor_click_or_route_and_reverse_exits_stay_locked() {
    let nav = navigation_world();
    for i in 0..3 {
        let reward = Vec3::from(map::DATA.blueprints()[i]);
        assert!(nav.path(map::START, reward).is_none());
        let ray = Ray3d::new(reward + Vec3::Y * 10., Dir3::NEG_Y);
        assert!(
            nav.pick(ray).is_none_or(|p| (p.y - reward.y).abs() > 0.35),
            "hidden ground selected at {i}"
        );
    }
    for i in 0..6 {
        let w = World::at(World::outside(i));
        let expected = if passages::portal(i).outside {
            Interaction::Portal(PortalId::ALL[i])
        } else {
            Interaction::None
        };
        assert_eq!(
            w.mission.interaction(&w.player, &w.nav, &w.view),
            expected,
            "portal {i}"
        );
        assert!(
            !w.nav.clear_segment(World::outside(i), World::inside(i)),
            "closed leaf {i}"
        );
    }
}

#[test]
fn opening_entering_collecting_returning_and_restart_are_independent_for_all_three_passages() {
    for i in 0..3 {
        let id = i * 2;
        let mut w = World::at(World::outside(id));
        w.open(id);
        assert_eq!(w.mission.secrets(), [false; 3]);
        assert_eq!(w.mission.blueprint_count(), 0);
        assert!(!w.mission.portals()[id + 1]);
        assert!(
            w.nav
                .path(w.player.feet(), Vec3::from(map::DATA.blueprints()[i]))
                .is_none()
        );
        w.walk(World::inside(id));
        assert!(w.mission.secrets()[i]);
        assert_eq!(w.mission.blueprint_count(), 0);
        w.walk(map::DATA.blueprints()[i].into());
        assert!(w.mission.blueprints()[i]);
        assert!(!w.mission.portals()[id + 1]);
        assert!(
            !w.mission.collect_blueprints(&w.player, &w.nav),
            "no duplicate reward"
        );
        let pose = w
            .nav
            .obstacles()
            .iter()
            .find(|o| o.name() == passages::portal(id).name)
            .unwrap()
            .center();
        w.player.respawn(map::START.xz(), map::START.y);
        w.passages
            .sync_access(&w.mission, &mut w.player, &w.view, &mut w.nav);
        assert!(w.mission.secrets()[i] && w.mission.blueprints()[i] && w.mission.portals()[id]);
        assert_eq!(
            w.nav
                .obstacles()
                .iter()
                .find(|o| o.name() == passages::portal(id).name)
                .unwrap()
                .center(),
            pose
        );
        w.mission.reset();
        w.player.reset();
        w.view.reset();
        w.passages
            .reset(&w.mission, &mut w.player, &w.view, &mut w.nav);
        assert_eq!(w.mission.portals(), [false; 6]);
        assert_eq!(w.mission.secrets(), [false; 3]);
        assert_eq!(w.mission.blueprint_count(), 0);
        assert!(w.player.route().is_empty());
        assert_eq!(w.player.feet(), map::START);
        for j in 0..6 {
            assert_eq!(
                w.nav
                    .obstacles()
                    .iter()
                    .find(|o| o.name() == passages::portal(j).name)
                    .unwrap()
                    .center(),
                passages::portal(j).center
            );
        }
    }
}

#[test]
fn jumping_through_an_open_portal_discovers_the_correct_floor_after_crossing() {
    for id in [0, 2, 4] {
        let mut w = World::at(World::outside(id));
        w.open(id);
        let p = passages::portal(id);
        let floor = Vec3::from(p.center) - Vec3::Y * p.half[1];
        let inward = Vec3::from(p.inward);
        w.walk(floor - inward * 0.3);
        for frame in 0..14 {
            w.player
                .advance(&w.nav, inward.xz() * 0.08, 1. / 60., frame == 0);
            w.passages.discover(&mut w.mission, &w.player, &w.nav);
        }
        assert!(
            w.player.height() - floor.y > 0.35,
            "cross the threshold airborne"
        );
        for _ in 0..60 {
            w.player.advance(&w.nav, Vec2::ZERO, 1. / 60., false);
            w.passages.discover(&mut w.mission, &w.player, &w.nav);
        }
        assert!(w.mission.secrets()[id / 2], "missed airborne entry {id}");
    }
}

#[test]
fn g1_can_be_opened_from_both_sides_but_is_not_a_global_shortcut_until_entered() {
    let a = World::outside(0);
    let b = World::outside(1);
    let mut w = World::at(a);
    w.open(0);
    w.walk(b);
    w.open(1);
    assert_eq!(w.mission.secrets(), [false; 3]);
    assert!(
        w.nav.path(a, b).unwrap().len() > 1,
        "unknown passage must not be a shortcut"
    );
    w.walk(World::inside(1));
    assert!(w.mission.secrets()[0]);
    assert_eq!(
        w.mission.blueprint_count(),
        0,
        "exploration does not spawn or award the blueprint"
    );
    assert_eq!(w.nav.path(a, b).unwrap().len(), 1);
}

#[test]
fn every_exit_can_be_unlocked_from_inside_before_taking_its_blueprint() {
    for i in 0..3 {
        let mut w = World::at(World::outside(i * 2));
        w.open(i * 2);
        w.walk(World::inside(i * 2));
        if i == 1 {
            // The main turn bypasses the alcove; possession is not an exit prerequisite.
            for p in [(20., 46.), (20., 48.), (23., 48.)] {
                w.walk(map::point(p.0, p.1, 3.));
            }
        }
        let exit = passages::portal(i * 2 + 1);
        let inside =
            Vec3::from(exit.center) - Vec3::Y * exit.half[1] + Vec3::from(exit.inward) * 1.5;
        let inside = if i == 2 { World::inside(5) } else { inside };
        w.walk(inside);
        assert!(!w.mission.blueprints()[i], "forced reward in passage {i}");
        w.open(i * 2 + 1);
        assert!(w.nav.clear_segment(inside, World::outside(i * 2 + 1)));
        w.walk(World::outside(i * 2 + 1));
        assert!(w.mission.portals()[i * 2] && w.mission.portals()[i * 2 + 1]);
        assert!(!w.mission.blueprints()[i]);
    }
}

#[test]
fn being_below_g2_does_not_discover_the_upper_passage() {
    let mut w = World::at(World::outside(2));
    w.open(2);
    let below = map::point(19.5, 43.5, 0.);
    w.player.respawn(below.xz(), 0.);
    w.passages.discover(&mut w.mission, &w.player, &w.nav);
    for _ in 0..15 {
        w.player
            .advance(&w.nav, Vec2::new(0., -0.1), 1. / 60., false);
        w.passages.discover(&mut w.mission, &w.player, &w.nav);
    }
    assert!(!w.mission.secrets()[1]);
    assert!(
        w.nav
            .clear_segment(map::point(35., 44., 0.), map::point(35., 49., 0.))
    );
    let upper = World::outside(2);
    w.player.respawn(upper.xz(), upper.y);
    w.passages.discover(&mut w.mission, &w.player, &w.nav);
    w.walk(World::inside(2));
    assert!(w.mission.secrets()[1]);
}

#[test]
fn moving_portal_waits_for_the_body_and_invalidates_an_active_route() {
    let mut w = World::at(map::point(50.9, 7.5, 0.));
    assert!(
        w.player.move_to(&w.nav, map::point(51.5, 9., 0.)) == crate::player::RouteOutcome::Started
    );
    w.mission
        .try_apply(Interaction::Portal(PortalId::ALL[1]))
        .unwrap();
    let mut blocked = false;
    for _ in 0..20 {
        blocked |= w
            .passages
            .advance(0.05, &w.mission, &mut w.nav, &mut w.player);
        assert!(
            w.nav.can_occupy(w.player.feet()),
            "moving cabinet pushed into body"
        );
    }
    assert!(blocked);
    assert!(w.player.route().is_empty());
    let p = passages::portal(1);
    let center = w
        .nav
        .obstacles()
        .iter()
        .find(|o| o.name() == p.name)
        .unwrap()
        .center();
    assert_ne!(
        Vec3::from(center),
        Vec3::from(p.center) + Vec3::from(p.slide)
    );
    w.player.respawn(map::point(51.5, 8.5, 0.).xz(), 0.);
    for _ in 0..20 {
        w.passages
            .advance(0.05, &w.mission, &mut w.nav, &mut w.player);
    }
    assert_eq!(
        Vec3::from(
            w.nav
                .obstacles()
                .iter()
                .find(|o| o.name() == p.name)
                .unwrap()
                .center()
        ),
        Vec3::from(p.center) + Vec3::from(p.slide)
    );
}

#[test]
fn overview_cutaway_restores_in_fpv_and_never_removes_body_or_shot_blockers() {
    let mut w = World::at(World::outside(4));
    let ray = Ray3d::new(map::point(8., 42., 10.), Dir3::NEG_Y);
    let physical = w.nav.ray_hit(ray);
    assert_eq!(w.nav.pick(ray).unwrap().y, 3.);
    w.open(4);
    w.walk(World::inside(4));
    assert_eq!(w.nav.pick(ray).unwrap().y, 0.);
    assert_eq!(w.nav.ray_hit(ray), physical);
    assert!(
        !w.nav.can_occupy(map::point(8., 42., 1.4)),
        "ceiling stays solid"
    );
    let cut = map::DATA
        .cutaways
        .iter()
        .position(|c| c.surface.is_some())
        .unwrap();
    let other = map::DATA
        .cutaways
        .iter()
        .position(|c| c.secret == 1 && c.portal < 0)
        .unwrap();
    let mut app = App::new();
    app.insert_resource(w.nav)
        .insert_resource(w.mission)
        .insert_resource(w.player)
        .insert_resource(w.passages)
        .insert_resource(w.view)
        .init_resource::<crate::ui::UiState>()
        .add_systems(Update, (passages::progress, passages::present).chain());
    let cover = app
        .world_mut()
        .spawn((passages::PassageCover(cut), Visibility::Inherited))
        .id();
    let unknown = app
        .world_mut()
        .spawn((passages::PassageCover(other), Visibility::Inherited))
        .id();
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(cover).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(unknown).unwrap(),
        Visibility::Inherited
    );
    for frame in 0..6 {
        app.world_mut()
            .resource_scope(|world, mut player: Mut<Player>| {
                player.advance(
                    world.resource::<Navigation>(),
                    Vec2::ZERO,
                    1. / 60.,
                    frame == 0,
                );
            });
        app.update();
    }
    assert!(app.world().resource::<Player>().height() > 0.35);
    assert_eq!(
        *app.world().get::<Visibility>(cover).unwrap(),
        Visibility::Hidden,
        "a jump must not close the cutaway over the player"
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .set_mode(ViewMode::FirstPerson);
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(cover).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(app.world().resource::<Navigation>().ray_hit(ray), physical);
    app.world_mut()
        .resource_mut::<ViewState>()
        .set_mode(ViewMode::Map);
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(cover).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(unknown).unwrap(),
        Visibility::Inherited
    );
}
