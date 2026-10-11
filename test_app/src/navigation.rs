use bevy::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

pub const ARENA_HALF: f32 = 14.0;
pub const PLAYER_RADIUS: f32 = 0.38;
const STEP: f32 = 0.5;
const WIDTH: usize = 55;
const HEIGHT: usize = 79;
const ORIGIN: Vec2 = Vec2::new(-13.5, -25.5);

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub half: Vec2,
}

#[derive(Resource)]
pub struct Navigation {
    pub obstacles: Vec<Obstacle>,
    pub min: Vec2,
    pub max: Vec2,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            obstacles: Vec::new(),
            min: Vec2::splat(-ARENA_HALF),
            max: Vec2::splat(ARENA_HALF),
        }
    }
}

impl Navigation {
    pub fn add(&mut self, center: Vec2, half: Vec2) {
        self.obstacles.push(Obstacle { center, half });
    }

    pub fn walkable(&self, point: Vec2) -> bool {
        point.is_finite()
            && point
                .cmpgt(self.min + Vec2::splat(0.3 + PLAYER_RADIUS))
                .all()
            && point
                .cmplt(self.max - Vec2::splat(0.3 + PLAYER_RADIUS))
                .all()
            && self.obstacles.iter().all(|o| {
                let nearest = point.clamp(o.center - o.half, o.center + o.half);
                point.distance_squared(nearest) > PLAYER_RADIUS * PLAYER_RADIUS
            })
    }

    pub fn clear_segment(&self, a: Vec2, b: Vec2) -> bool {
        let samples = (a.distance(b) / 0.08).ceil() as usize;
        (0..=samples.max(1)).all(|i| self.walkable(a.lerp(b, i as f32 / samples.max(1) as f32)))
    }

    /// Resolve each small movement step separately, so sprinting cannot tunnel through walls.
    pub fn move_and_slide(&self, position: Vec2, delta: Vec2) -> Vec2 {
        let steps = (delta.length() / 0.1).ceil().max(1.0) as usize;
        let step = delta / steps as f32;
        let mut point = position;
        for _ in 0..steps {
            if self.walkable(point + step) {
                point += step;
            } else {
                if self.walkable(point + Vec2::new(step.x, 0.0)) {
                    point.x += step.x;
                }
                if self.walkable(point + Vec2::new(0.0, step.y)) {
                    point.y += step.y;
                }
            }
        }
        point
    }

    fn point(index: usize) -> Vec2 {
        Vec2::new((index % WIDTH) as f32, (index / WIDTH) as f32) * STEP + ORIGIN
    }

    // Connect off-grid endpoints only to cells that can actually be reached without clipping.
    fn anchor(&self, point: Vec2) -> Option<usize> {
        (0..WIDTH * HEIGHT)
            .filter(|&i| Self::point(i).distance_squared(point) < 2.0)
            .filter(|&i| self.clear_segment(point, Self::point(i)))
            .min_by(|&a, &b| {
                Self::point(a)
                    .distance_squared(point)
                    .total_cmp(&Self::point(b).distance_squared(point))
            })
    }

    /// A* on an eight-connected grid, followed by collision-checked line-of-sight smoothing.
    pub fn path(&self, start: Vec2, goal: Vec2) -> Option<VecDeque<Vec2>> {
        if !self.walkable(start) || !self.walkable(goal) {
            return None;
        }
        if self.clear_segment(start, goal) {
            return Some(VecDeque::from([goal]));
        }
        let start_cell = self.anchor(start)?;
        let goal_cell = self.anchor(goal)?;
        let mut costs = vec![u32::MAX; WIDTH * HEIGHT];
        let mut parents = vec![usize::MAX; WIDTH * HEIGHT];
        let mut open = BinaryHeap::new();
        let heuristic = |index: usize| {
            let dx = (index % WIDTH).abs_diff(goal_cell % WIDTH) as u32;
            let dy = (index / WIDTH).abs_diff(goal_cell / WIDTH) as u32;
            10 * dx.max(dy) + 4 * dx.min(dy)
        };
        costs[start_cell] = 0;
        open.push(Reverse((heuristic(start_cell), 0, start_cell)));
        while let Some(Reverse((_, cost, current))) = open.pop() {
            if cost != costs[current] {
                continue;
            }
            if current == goal_cell {
                let mut reversed = vec![goal];
                let mut cell = current;
                loop {
                    reversed.push(Self::point(cell));
                    if cell == start_cell {
                        break;
                    }
                    cell = parents[cell];
                }
                reversed.reverse();
                let mut route = VecDeque::new();
                let mut from = start;
                let mut index = 0;
                while index < reversed.len() {
                    let mut furthest = index;
                    for (candidate, &point) in reversed.iter().enumerate().skip(index + 1) {
                        if self.clear_segment(from, point) {
                            furthest = candidate;
                        }
                    }
                    from = reversed[furthest];
                    route.push_back(from);
                    index = furthest + 1;
                }
                return Some(route);
            }
            let x = (current % WIDTH) as i32;
            let y = (current / WIDTH) as i32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let (nx, ny) = (x + dx, y + dy);
                    if !(0..WIDTH as i32).contains(&nx) || !(0..HEIGHT as i32).contains(&ny) {
                        continue;
                    }
                    let next = ny as usize * WIDTH + nx as usize;
                    let candidate = cost + if dx != 0 && dy != 0 { 14 } else { 10 };
                    if candidate < costs[next]
                        && self.clear_segment(Self::point(current), Self::point(next))
                    {
                        costs[next] = candidate;
                        parents[next] = current;
                        open.push(Reverse((candidate + heuristic(next), candidate, next)));
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_around_wall_is_clear_including_off_grid_endpoints() {
        let mut nav = Navigation::default();
        nav.add(Vec2::ZERO, Vec2::new(0.3, 3.0));
        let start = Vec2::new(-3.13, 0.17);
        let goal = Vec2::new(3.21, -0.11);
        let route = nav.path(start, goal).expect("route around wall");
        assert!(route.len() > 1);
        assert_eq!(route.back(), Some(&goal));
        let mut from = start;
        for to in route {
            assert!(nav.clear_segment(from, to));
            from = to;
        }
    }

    #[test]
    fn unreachable_and_blocked_destinations_are_rejected() {
        let mut nav = Navigation::default();
        nav.add(Vec2::ZERO, Vec2::new(0.3, 14.0));
        assert!(
            nav.path(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0))
                .is_none()
        );
        assert!(nav.path(Vec2::new(-3.0, 0.0), Vec2::ZERO).is_none());
        assert!(nav.path(Vec2::ZERO, Vec2::splat(20.0)).is_none());
    }

    #[test]
    fn movement_does_not_tunnel_and_slides_along_walls() {
        let mut nav = Navigation::default();
        nav.add(Vec2::ZERO, Vec2::new(0.25, 5.0));
        let position = nav.move_and_slide(Vec2::new(-2.0, 0.0), Vec2::new(8.0, 2.0));
        assert!(position.x < -0.6);
        assert!(position.y > 1.9);
        assert!(nav.walkable(position));
    }

    #[test]
    fn narrow_diagonal_gap_respects_character_radius() {
        let mut nav = Navigation::default();
        nav.add(Vec2::new(-0.6, 0.6), Vec2::splat(0.5));
        nav.add(Vec2::new(0.6, -0.6), Vec2::splat(0.5));
        assert!(!nav.clear_segment(Vec2::splat(-2.0), Vec2::splat(2.0)));
        let mut from = Vec2::splat(-2.0);
        for to in nav.path(from, Vec2::splat(2.0)).unwrap() {
            assert!(nav.clear_segment(from, to));
            from = to;
        }
    }
}
