pub(crate) trait Markable {
    fn mark(&mut self);
    fn unmark(&mut self);
    fn is_marked(&self);
}
