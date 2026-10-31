//! Height-aware walking, body collision and A* for overlapping floors.
use bevy::prelude::*;
use serde::Deserialize;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    sync::OnceLock,
};

pub const BODY_HALF: f32 = 0.60;
pub const BODY_HEIGHT: f32 = 1.82;
pub const STEP_HEIGHT: f32 = 0.3;
const GRID: f32 = 0.5;
const BIN: f32 = 2.0;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "RawSurface")]
pub struct Surface {
    center: [f32; 2],
    half: [f32; 2],
    height: f32,
    bottom: f32,
}
#[derive(Deserialize)]
struct RawSurface {
    center: [f32; 2],
    half: [f32; 2],
    height: f32,
    bottom: f32,
}
impl TryFrom<RawSurface> for Surface {
    type Error = String;
    fn try_from(raw: RawSurface) -> Result<Self, String> {
        Self::new(raw.center, raw.half, raw.height, raw.bottom)
    }
}
impl Surface {
    pub fn new(center: [f32; 2], half: [f32; 2], height: f32, bottom: f32) -> Result<Self, String> {
        if !center
            .iter()
            .chain(half.iter())
            .chain([&height, &bottom])
            .all(|n| n.is_finite())
            || half.iter().any(|n| *n <= 0.)
            || bottom > height
        {
            return Err("invalid surface extents or height".into());
        }
        Ok(Self {
            center,
            half,
            height,
            bottom,
        })
    }
    pub fn center(&self) -> [f32; 2] {
        self.center
    }
    pub fn half(&self) -> [f32; 2] {
        self.half
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    pub fn contains(&self, p: Vec2) -> bool {
        (p - Vec2::from(self.center))
            .abs()
            .cmple(Vec2::from(self.half) + Vec2::splat(0.0001))
            .all()
    }
    fn overlaps_body(&self, p: Vec2) -> bool {
        (p - Vec2::from(self.center))
            .abs()
            .cmplt(Vec2::from(self.half) + Vec2::splat(BODY_HALF - 0.001))
            .all()
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "RawObstacle")]
pub struct Obstacle {
    name: String,
    center: [f32; 3],
    half: [f32; 3],
}
#[derive(Deserialize)]
struct RawObstacle {
    name: String,
    center: [f32; 3],
    half: [f32; 3],
}
impl TryFrom<RawObstacle> for Obstacle {
    type Error = String;
    fn try_from(raw: RawObstacle) -> Result<Self, String> {
        Self::new(raw.name, raw.center, raw.half)
    }
}
impl Obstacle {
    pub fn new(name: String, center: [f32; 3], half: [f32; 3]) -> Result<Self, String> {
        if name.trim().is_empty()
            || !center.iter().chain(half.iter()).all(|n| n.is_finite())
            || half.iter().any(|n| *n <= 0.)
        {
            return Err("invalid obstacle name or extents".into());
        }
        Ok(Self { name, center, half })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn center(&self) -> [f32; 3] {
        self.center
    }
    pub fn half(&self) -> [f32; 3] {
        self.half
    }

    fn overlaps(&self, p: Vec2) -> bool {
        let c = Vec3::from(self.center).xz();
        let h = Vec3::from(self.half).xz();
        (p - c)
            .abs()
            .cmplt(h + Vec2::splat(BODY_HALF - 0.001))
            .all()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct WalkRegion {
    pub center: Vec3,
    pub half: Vec2,
}
impl WalkRegion {
    pub fn contains(&self, point: Vec3) -> bool {
        (point.y - self.center.y).abs() < 0.35
            && (point.xz() - self.center.xz()).abs().cmple(self.half).all()
    }
}

/// Shared logical ray test: rendering a cutaway never removes this blocker.
pub(crate) fn box_hit(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut entry: f32 = 0.;
    let mut exit = f32::INFINITY;
    for i in 0..3 {
        let relative = origin[i] - center[i];
        if direction[i].abs() < 0.00001 {
            if relative.abs() > half[i] {
                return None;
            }
        } else {
            let a = (-half[i] - relative) / direction[i];
            let b = (half[i] - relative) / direction[i];
            entry = entry.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    (exit >= entry).then_some(entry)
}

#[derive(Default)]
struct Bin {
    surfaces: Vec<usize>,
    obstacles: Vec<usize>,
}
#[derive(Default)]
struct Graph {
    nodes: Vec<Vec3>,
    edges: Vec<Vec<(usize, u32)>>,
    cells: Vec<Vec<usize>>,
    width: usize,
    depth: usize,
    origin: Vec2,
}
#[derive(Resource)]
pub struct Navigation {
    surfaces: Vec<Surface>,
    obstacles: Vec<Obstacle>,
    min: Vec2,
    max: Vec2,
    bins: HashMap<(i32, i32), Bin>,
    graph: OnceLock<Graph>,
    restricted: Vec<WalkRegion>,
    entry_landings: Vec<WalkRegion>,
    cutaway: HashSet<String>,
    cut_floors: Vec<WalkRegion>,
}
impl Default for Navigation {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new(), Vec2::splat(-39.), Vec2::splat(39.))
    }
}
impl Navigation {
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }
    /// A new pose is checked at entry. Cache maintenance belongs to Navigation.
    pub fn set_obstacle_pose(&mut self, name: &str, center: Vec3) -> Result<bool, String> {
        if !center.is_finite() {
            return Err("obstacle pose must be finite".into());
        }
        let obstacle = self
            .obstacles
            .iter_mut()
            .find(|o| o.name == name)
            .ok_or_else(|| format!("unknown obstacle {name}"))?;
        if obstacle.center == center.to_array() {
            return Ok(false);
        }
        obstacle.center = center.to_array();
        self.rebuild();
        Ok(true)
    }

    pub fn new(surfaces: Vec<Surface>, obstacles: Vec<Obstacle>, min: Vec2, max: Vec2) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min.cmplt(max).all(),
            "invalid navigation bounds"
        );
        assert_eq!(
            obstacles
                .iter()
                .map(Obstacle::name)
                .collect::<HashSet<_>>()
                .len(),
            obstacles.len(),
            "obstacle names must be unique"
        );
        let mut nav = Self {
            surfaces,
            obstacles,
            min,
            max,
            bins: HashMap::new(),
            graph: OnceLock::new(),
            restricted: Vec::new(),
            entry_landings: Vec::new(),
            cutaway: HashSet::new(),
            cut_floors: Vec::new(),
        };
        nav.rebuild();
        nav
    }
    fn bin(p: Vec2) -> (i32, i32) {
        ((p.x / BIN).floor() as i32, (p.y / BIN).floor() as i32)
    }
    fn rebuild(&mut self) {
        self.bins.clear();
        self.graph.take();
        for (i, s) in self.surfaces.iter().enumerate() {
            let lo = Self::bin(Vec2::from(s.center) - Vec2::from(s.half) - Vec2::splat(BODY_HALF));
            let hi = Self::bin(Vec2::from(s.center) + Vec2::from(s.half) + Vec2::splat(BODY_HALF));
            for y in lo.1..=hi.1 {
                for x in lo.0..=hi.0 {
                    self.bins.entry((x, y)).or_default().surfaces.push(i);
                }
            }
        }
        for (i, o) in self.obstacles.iter().enumerate() {
            let c = Vec3::from(o.center).xz();
            let h = Vec3::from(o.half).xz() + Vec2::splat(BODY_HALF);
            let lo = Self::bin(c - h);
            let hi = Self::bin(c + h);
            for y in lo.1..=hi.1 {
                for x in lo.0..=hi.0 {
                    self.bins.entry((x, y)).or_default().obstacles.push(i);
                }
            }
        }
    }
    fn surfaces_at(&self, p: Vec2) -> impl Iterator<Item = &Surface> {
        self.bins
            .get(&Self::bin(p))
            .into_iter()
            .flat_map(|b| b.surfaces.iter())
            .map(|&i| &self.surfaces[i])
            .filter(move |s| s.overlaps_body(p))
    }
    fn obstacles_at(&self, p: Vec2) -> impl Iterator<Item = &Obstacle> {
        self.bins
            .get(&Self::bin(p))
            .into_iter()
            .flat_map(|b| b.obstacles.iter())
            .map(|&i| &self.obstacles[i])
            .filter(move |o| o.overlaps(p))
    }
    /// The highest support BELOW the feet, not the highest floor in the room.
    pub fn support(&self, p: Vec2, maximum: f32) -> f32 {
        self.surfaces_at(p)
            .filter(|s| s.height <= maximum + 0.001)
            .map(|s| s.height)
            .chain(
                self.obstacles_at(p)
                    .map(|o| o.center[1] + o.half[1])
                    .filter(|&top| top <= maximum + 0.001),
            )
            .fold(0., f32::max)
    }
    pub fn ceiling(&self, p: Vec2, head: f32) -> f32 {
        self.surfaces_at(p)
            .map(|s| s.bottom)
            .chain(self.obstacles_at(p).map(|o| o.center[1] - o.half[1]))
            .filter(|&bottom| bottom >= head - 0.01)
            .fold(f32::INFINITY, f32::min)
    }
    pub fn can_occupy(&self, p: Vec3) -> bool {
        p.is_finite()
            && p.xz().cmpgt(self.min + Vec2::splat(BODY_HALF)).all()
            && p.xz().cmplt(self.max - Vec2::splat(BODY_HALF)).all()
            && !self.obstacles_at(p.xz()).any(|o| {
                o.center[1] + o.half[1] > p.y + 0.02
                    && o.center[1] - o.half[1] < p.y + BODY_HEIGHT - 0.02
            })
            && !self
                .surfaces_at(p.xz())
                .any(|s| s.height > p.y + 0.02 && s.bottom < p.y + BODY_HEIGHT - 0.02)
    }
    pub fn walkable(&self, p: Vec3) -> bool {
        self.can_occupy(p) && (self.support(p.xz(), p.y + 0.02) - p.y).abs() < 0.025
    }
    pub(super) fn restrict_routes(
        &mut self,
        regions: Vec<WalkRegion>,
        entries: Vec<WalkRegion>,
    ) -> bool {
        if self.restricted == regions && self.entry_landings == entries {
            return false;
        }
        self.restricted = regions;
        self.entry_landings = entries;
        self.graph.take();
        true
    }
    pub(super) fn set_cutaway(&mut self, names: HashSet<String>, floors: Vec<WalkRegion>) {
        self.cutaway = names;
        self.cut_floors = floors;
    }
    fn route_allowed(&self, p: Vec3) -> bool {
        !self.restricted.iter().any(|r| r.contains(p))
            || self.entry_landings.iter().any(|r| r.contains(p))
    }
    fn route_segment(&self, a: Vec3, b: Vec3) -> bool {
        let n = (a.distance(b) / 0.12).ceil().max(1.) as usize;
        self.clear_segment(a, b)
            && (0..=n).all(|i| self.route_allowed(a.lerp(b, i as f32 / n as f32)))
    }
    fn surface_hit(&self, ray: Ray3d, surface: &Surface) -> Option<f32> {
        box_hit(
            ray.origin,
            *ray.direction,
            Vec3::new(
                surface.center[0],
                (surface.height + surface.bottom) / 2.,
                surface.center[1],
            ),
            Vec3::new(
                surface.half[0],
                (surface.height - surface.bottom) / 2.,
                surface.half[1],
            ),
        )
    }
    pub(crate) fn ray_hit(&self, ray: Ray3d) -> Option<f32> {
        self.obstacles
            .iter()
            .filter_map(|o| box_hit(ray.origin, *ray.direction, o.center.into(), o.half.into()))
            .chain(
                self.surfaces
                    .iter()
                    .filter_map(|s| self.surface_hit(ray, s)),
            )
            .min_by(f32::total_cmp)
    }
    pub fn pick(&self, ray: Ray3d) -> Option<Vec3> {
        let blocker = self
            .obstacles
            .iter()
            .filter(|o| !self.cutaway.contains(o.name()))
            .filter_map(|o| box_hit(ray.origin, *ray.direction, o.center.into(), o.half.into()))
            .chain(
                self.surfaces
                    .iter()
                    .filter_map(|s| self.surface_hit(ray, s))
                    .filter(|d| {
                        !self
                            .cut_floors
                            .iter()
                            .any(|r| r.contains(ray.get_point(*d)))
                    }),
            )
            .min_by(f32::total_cmp)
            .unwrap_or(f32::INFINITY);
        self.surfaces
            .iter()
            .map(|s| s.height)
            .chain([0.])
            .filter_map(|height| {
                let d = ray.intersect_plane(Vec3::Y * height, InfinitePlane3d::new(Vec3::Y))?;
                let mut p = ray.get_point(d);
                p.y = height;
                let on_floor = height == 0.
                    || self
                        .surfaces_at(p.xz())
                        .any(|s| s.contains(p.xz()) && (s.height - height).abs() < 0.001);
                (on_floor
                    && d <= blocker + 0.02
                    && self.route_allowed(p)
                    && !self.cut_floors.iter().any(|r| r.contains(p))
                    && p.xz().cmpge(self.min).all()
                    && p.xz().cmple(self.max).all())
                .then_some((d, p))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, p)| p)
    }
    pub(super) fn clear_body_segment(&self, a: Vec3, b: Vec3) -> bool {
        let n = (a.distance(b) / 0.08).ceil().max(1.) as usize;
        (0..=n).all(|i| self.can_occupy(a.lerp(b, i as f32 / n as f32)))
    }
    pub fn clear_segment(&self, a: Vec3, b: Vec3) -> bool {
        if !self.walkable(a) || !self.walkable(b) {
            return false;
        }
        let n = (a.xz().distance(b.xz()) / 0.12).ceil().max(1.) as usize;
        let mut height = a.y;
        for i in 1..=n {
            let p = a.xz().lerp(b.xz(), i as f32 / n as f32);
            let next = self.support(p, height + STEP_HEIGHT);
            if (next - height).abs() > STEP_HEIGHT + 0.001
                || !self.can_occupy(Vec3::new(p.x, next, p.y))
            {
                return false;
            }
            height = next;
        }
        (height - b.y).abs() < 0.025
    }
    /// Small body sweeps prevent sprint tunnelling and slide along walls.
    pub fn move_body(&self, from: Vec3, delta: Vec2, stairs: bool) -> Vec3 {
        let n = (delta.length() / 0.08).ceil().max(1.) as usize;
        let step = delta / n as f32;
        let mut p = from;
        for _ in 0..n {
            let enter = |d: Vec2, p: Vec3| {
                let v = p.xz() + d;
                let support = self.support(v, p.y + if stairs { STEP_HEIGHT } else { 0.001 });
                let h = if stairs && (support - p.y).abs() <= STEP_HEIGHT + 0.001 {
                    support
                } else {
                    p.y
                };
                let next = Vec3::new(v.x, h, v.y);
                self.can_occupy(next).then_some(next)
            };
            if let Some(next) = enter(step, p) {
                p = next;
            } else {
                if let Some(next) = enter(Vec2::new(step.x, 0.), p) {
                    p = next;
                }
                if let Some(next) = enter(Vec2::new(0., step.y), p) {
                    p = next;
                }
            }
        }
        p
    }
    fn build_graph(&self) -> Graph {
        let origin = self.min + Vec2::splat(GRID);
        let width = ((self.max.x - origin.x) / GRID).ceil() as usize;
        let depth = ((self.max.y - origin.y) / GRID).ceil() as usize;
        let mut g = Graph {
            origin,
            width,
            depth,
            cells: vec![Vec::new(); width * depth],
            ..default()
        };
        for z in 0..depth {
            for x in 0..width {
                let p = origin + Vec2::new(x as f32, z as f32) * GRID;
                let mut levels = vec![0.];
                levels.extend(self.surfaces_at(p).map(|s| s.height));
                levels.sort_by(f32::total_cmp);
                levels.dedup_by(|a, b| (*a - *b).abs() < 0.01);
                for y in levels {
                    let node = Vec3::new(p.x, y, p.y);
                    if self.walkable(node) && self.route_allowed(node) {
                        g.cells[z * width + x].push(g.nodes.len());
                        g.nodes.push(node);
                    }
                }
            }
        }
        g.edges = vec![Vec::new(); g.nodes.len()];
        for z in 0..depth {
            for x in 0..width {
                for &a in &g.cells[z * width + x] {
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dz == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let nz = z as i32 + dz;
                            if nx < 0 || nz < 0 || nx >= width as i32 || nz >= depth as i32 {
                                continue;
                            }
                            for &b in &g.cells[nz as usize * width + nx as usize] {
                                if (g.nodes[a].y - g.nodes[b].y).abs() <= STEP_HEIGHT + 0.001
                                    && self.route_segment(g.nodes[a], g.nodes[b])
                                {
                                    g.edges[a].push((b, if dx != 0 && dz != 0 { 14 } else { 10 }));
                                }
                            }
                        }
                    }
                }
            }
        }
        g
    }
    fn anchor(&self, g: &Graph, p: Vec3) -> Option<usize> {
        let cell = ((p.xz() - g.origin) / GRID).round().as_ivec2();
        let mut best = None;
        let mut distance = f32::INFINITY;
        for z in cell.y - 2..=cell.y + 2 {
            for x in cell.x - 2..=cell.x + 2 {
                if x < 0 || z < 0 || x >= g.width as i32 || z >= g.depth as i32 {
                    continue;
                }
                for &i in &g.cells[z as usize * g.width + x as usize] {
                    let d = p.distance_squared(g.nodes[i]);
                    if d < distance && d < 1.5 && self.route_segment(p, g.nodes[i]) {
                        best = Some(i);
                        distance = d;
                    }
                }
            }
        }
        best
    }
    pub fn path(&self, start: Vec3, goal: Vec3) -> Option<VecDeque<Vec3>> {
        if !self.walkable(start)
            || !self.walkable(goal)
            || !self.route_allowed(start)
            || !self.route_allowed(goal)
        {
            return None;
        }
        if self.route_segment(start, goal) {
            return Some(VecDeque::from([goal]));
        }
        let g = self.graph.get_or_init(|| self.build_graph());
        let first = self.anchor(g, start)?;
        let last = self.anchor(g, goal)?;
        let heuristic = |i: usize| {
            let d = (g.nodes[i].xz() - g.nodes[last].xz()).abs() / GRID;
            (14. * d.min_element() + 10. * (d.max_element() - d.min_element())) as u32
        };
        let mut costs = vec![u32::MAX; g.nodes.len()];
        let mut parent = vec![usize::MAX; g.nodes.len()];
        let mut open = BinaryHeap::new();
        costs[first] = 0;
        open.push(Reverse((heuristic(first), 0, first)));
        while let Some(Reverse((_, cost, a))) = open.pop() {
            if costs[a] != cost {
                continue;
            }
            if a == last {
                let mut points = vec![goal];
                let mut at = a;
                loop {
                    points.push(g.nodes[at]);
                    if at == first {
                        break;
                    }
                    at = parent[at];
                }
                points.reverse();
                let mut route = VecDeque::new();
                let mut from = start;
                let mut i = 0;
                while i < points.len() {
                    let mut far = i;
                    for (j, &point) in points.iter().enumerate().skip(i + 1) {
                        if self.route_segment(from, point) {
                            far = j;
                        }
                    }
                    from = points[far];
                    route.push_back(from);
                    i = far + 1;
                }
                return Some(route);
            }
            for &(b, step) in &g.edges[a] {
                let next = cost + step;
                if next < costs[b] {
                    costs[b] = next;
                    parent[b] = a;
                    open.push(Reverse((next + heuristic(b), next, b)));
                }
            }
        }
        None
    }
}
