//! Optional launch settings; ordinary launches retain their defaults.
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
