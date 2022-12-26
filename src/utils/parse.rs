use std::str::FromStr;

pub fn parse_number<T: FromStr>(s: &str) -> Option<T> {
    match T::from_str(s) {
        Ok(l) => Some(l),
        _ => None,
    }
}
