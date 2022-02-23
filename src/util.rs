pub fn min<T: Ord>(left: T, right: T) -> T {
    if left < right {
        left
    } else {
        right
    }
}
