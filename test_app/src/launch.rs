//! Optional launch settings; ordinary launches retain their defaults.
pub struct Options {
    pub smoke_test: bool,
    pub offscreen: bool,
    pub fixed_window: bool,
    pub start_unfocused: bool,
    pub scale_factor: Option<f32>,
}

impl Options {
    pub fn from_env() -> Self {
        let smoke_test = std::env::args().any(|arg| arg == "--smoke-test");
        let scale_factor = value::<f32>("--scale-factor");
        assert!(
            scale_factor.is_none_or(|value| value.is_finite() && value > 0.0),
            "invalid scale factor"
        );
        Self {
            smoke_test,
            offscreen: smoke_test && std::env::args().any(|arg| arg == "--offscreen"),
            fixed_window: std::env::args().any(|arg| arg == "--fixed-window"),
            start_unfocused: std::env::args().any(|arg| arg == "--start-unfocused"),
            scale_factor,
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
