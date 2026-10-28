//! Optional launch settings; ordinary launches retain their defaults.
use bevy::prelude::IVec2;

pub struct Options {
    pub fixed_window: bool,
    pub start_unfocused: bool,
    pub scale_factor: Option<f32>,
    pub window_position: Option<IVec2>,
}

impl Options {
    pub fn from_env() -> Self {
        let scale_factor = value::<f32>("--scale-factor");
        assert!(
            scale_factor.is_none_or(|value| value.is_finite() && value > 0.0),
            "invalid scale factor"
        );
        Self {
            fixed_window: std::env::args().any(|arg| arg == "--fixed-window"),
            start_unfocused: std::env::args().any(|arg| arg == "--start-unfocused"),
            scale_factor,
            window_position: value::<String>("--window-position").map(|position| {
                let (x, y) = position
                    .split_once(',')
                    .expect("--window-position requires x,y in physical pixels");
                IVec2::new(
                    x.parse().expect("invalid --window-position x"),
                    y.parse().expect("invalid --window-position y"),
                )
            }),
        }
    }
}

pub fn value<T: std::str::FromStr>(name: &str) -> Option<T> {
    let mut arguments = std::env::args().skip(1);
    arguments.find(|argument| argument == name)?;
    let value = arguments
        .next()
        .unwrap_or_else(|| panic!("{name} requires a value"));
    Some(
        value
            .parse()
            .unwrap_or_else(|_| panic!("invalid {name}: {value}")),
    )
}
