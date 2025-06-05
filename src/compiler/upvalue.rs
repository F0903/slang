#[derive(Debug, Clone)]
pub struct Upvalue {
    pub is_local: bool,
    pub index: u16,
}
