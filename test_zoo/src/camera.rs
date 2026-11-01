//! Detail viewports, fixed projections and pointer-controlled orbit.
use bevy::{
    camera::{RenderTarget, ScalingMode, Viewport, visibility::RenderLayers},
    prelude::*,
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Top,
    Front,
    Right,
    Orbit,
}

impl View {
    pub const ALL: [Self; 4] = [Self::Top, Self::Front, Self::Right, Self::Orbit];

    pub fn label(self) -> &'static str {
        match self {
            Self::Top => "Oben",
            Self::Front => "Front",
            Self::Right => "Rechts",
            Self::Orbit => "Orbit",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Top => 0,
            Self::Front => 1,
            Self::Right => 2,
            Self::Orbit => 3,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Separator {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    #[default]
    Four,
    Single(View),
}

#[derive(Resource)]
pub struct Views {
    pub layout: Layout,
    pub yaw: f32,
    pub pitch: f32,
    zoom: [f32; 4],
    dragging: bool,
}

impl Default for Views {
    fn default() -> Self {
        Self {
            layout: Layout::Four,
            yaw: 0.55,
            pitch: 0.30,
            zoom: [1.; 4],
            dragging: false,
        }
    }
}

impl Views {
    pub fn toggle(&mut self, view: View) {
        self.layout = if self.layout == Layout::Single(view) {
            Layout::Four
        } else {
            Layout::Single(view)
        };
        self.dragging = false;
    }

    pub fn show_all(&mut self) {
        self.layout = Layout::Four;
        self.dragging = false;
    }

    pub fn reset(&mut self) {
        *self = Self {
            layout: self.layout,
            ..default()
        };
    }

    pub fn drag(&mut self, hovered: Option<View>, pressed: bool, just_pressed: bool, delta: Vec2) {
        if !pressed {
            self.dragging = false;
        } else if just_pressed {
            self.dragging = hovered == Some(View::Orbit);
        }
        if self.dragging && hovered == Some(View::Orbit) {
            self.yaw = (self.yaw - delta.x * 0.006).rem_euclid(std::f32::consts::TAU);
            self.pitch = (self.pitch + delta.y * 0.006).rem_euclid(std::f32::consts::TAU);
        }
    }

    pub fn zoom(&mut self, hovered: Option<View>, delta: f32) {
        if let Some(view) = hovered {
            let zoom = &mut self.zoom[view.index()];
            *zoom = (*zoom * (-delta * 0.08).exp()).clamp(0.45, 2.);
        }
    }

    pub fn transform(&self, view: View, focus: Vec3, overview: bool) -> Transform {
        let rotation = match view {
            View::Top => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            View::Front => Quat::IDENTITY,
            View::Right => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            View::Orbit => {
                let pitch = if overview { 0.82 } else { self.pitch };
                // Rotate the up direction along with the orbit, including across both poles.
                Quat::from_euler(EulerRot::YXZ, self.yaw, -pitch, 0.)
            }
        };
        Transform::from_translation(focus + rotation * Vec3::Z * 35.).with_rotation(rotation)
    }

    pub fn height(&self, view: View, base: f32) -> f32 {
        base * self.zoom[view.index()]
    }

    pub fn frame(&self, size: UVec2, scale: f32, detail: bool) -> Frame {
        let mut frame = Frame {
            rects: [None; 4],
            scale,
            detail,
        };
        if size.min_element() == 0 {
            return frame;
        }
        if !detail {
            frame.rects[View::Orbit.index()] = Some(URect::from_corners(UVec2::ZERO, size));
            return frame;
        }

        let pixels = |value: f32| (value * scale).round() as u32;
        let margin = pixels(12.);
        let min = UVec2::new(margin, pixels(110.));
        let max = UVec2::new(
            size.x.saturating_sub(margin),
            size.y.saturating_sub(pixels(88.)),
        );
        if min.cmpge(max).any() {
            return frame;
        }
        let content = URect::from_corners(min, max);
        if let Layout::Single(view) = self.layout {
            frame.rects[view.index()] = Some(content);
            return frame;
        }

        let gap = pixels(8.);
        let available = content.size().saturating_sub(UVec2::splat(gap));
        let half = available / 2;
        if half.min_element() == 0 {
            return frame;
        }
        for (index, view) in View::ALL.into_iter().enumerate() {
            let column = index as u32 % 2;
            let row = index as u32 / 2;
            let start = min + UVec2::new(column, row) * (half + UVec2::splat(gap));
            let end = UVec2::new(
                if column == 0 { min.x + half.x } else { max.x },
                if row == 0 { min.y + half.y } else { max.y },
            );
            frame.rects[view.index()] = Some(URect::from_corners(start, end));
        }
        frame
    }
}

pub struct Frame {
    rects: [Option<URect>; 4],
    scale: f32,
    detail: bool,
}

impl Frame {
    pub fn rect(&self, view: View) -> Option<URect> {
        self.rects[view.index()]
    }

    /// Thin dividers centered in the existing gaps, without reducing the camera viewports.
    pub fn separator(&self, separator: Separator) -> Option<URect> {
        let [Some(top), Some(front), Some(right), Some(orbit)] = self.rects else {
            return None;
        };
        let thickness = (2. * self.scale).round().max(1.) as u32;
        match separator {
            Separator::Vertical => {
                let gap = front.min.x - top.max.x;
                let width = thickness.min(gap);
                if width == 0 {
                    return None;
                }
                let x = top.max.x + (gap - width) / 2;
                Some(URect::from_corners(
                    UVec2::new(x, top.min.y),
                    UVec2::new(x + width, orbit.max.y),
                ))
            }
            Separator::Horizontal => {
                let gap = right.min.y - top.max.y;
                let height = thickness.min(gap);
                if height == 0 {
                    return None;
                }
                let y = top.max.y + (gap - height) / 2;
                Some(URect::from_corners(
                    UVec2::new(top.min.x, y),
                    UVec2::new(front.max.x, y + height),
                ))
            }
        }
    }

    /// The cameras share a color attachment; clearing it also erases other viewports.
    pub fn clear_color(&self, view: View) -> ClearColorConfig {
        if self.rect(view).is_some() && self.rects[..view.index()].iter().all(Option::is_none) {
            ClearColorConfig::Default
        } else {
            ClearColorConfig::None
        }
    }

    pub fn viewport(&self, view: View) -> Option<Viewport> {
        self.rect(view).map(|rect| Viewport {
            physical_position: rect.min,
            physical_size: rect.size(),
            ..default()
        })
    }

    /// Logical cursor coordinates share the same scale as the window's UI.
    pub fn hovered(&self, cursor: Option<Vec2>) -> Option<View> {
        let point = cursor? * self.scale;
        if !point.is_finite() {
            return None;
        }
        View::ALL.into_iter().find(|view| {
            self.rect(*view).is_some_and(|rect| {
                let mut min = rect.min.as_vec2();
                if self.detail {
                    // The title button at the top of each pane is not an orbit/zoom surface.
                    min.y += 40. * self.scale;
                }
                point.cmpge(min).all() && point.cmplt(rect.max.as_vec2()).all()
            })
        })
    }
}

pub fn spawn(commands: &mut Commands, target: RenderTarget) {
    for (index, view) in View::ALL.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Zoo-Kamera {}", view.label())),
            view,
            Camera3d::default(),
            Camera {
                order: index as isize,
                is_active: view == View::Orbit,
                clear_color: if view == View::Orbit {
                    ClearColorConfig::Default
                } else {
                    ClearColorConfig::None
                },
                ..default()
            },
            target.clone(),
            bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 20.,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::default(),
            Msaa::Sample4,
        ));
    }
    commands.spawn((
        Name::new("Zoo-Oberfläche"),
        Camera2d,
        IsDefaultUiCamera,
        Camera {
            order: 4,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        target,
        RenderLayers::layer(1),
        Msaa::Sample4,
    ));
}
