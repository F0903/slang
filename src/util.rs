#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";

pub fn min<T: Ord>(left: T, right: T) -> T {
    if left < right {
        left
    } else {
        right
    }
}
