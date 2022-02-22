pub trait Identifiable {
    fn get_identifier(&self) -> &'static str;
}
