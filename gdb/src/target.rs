pub trait Target {
    fn inturrupt(&mut self) -> Result<(), ()>;
    fn step_at(&mut self, addr: Option<u32>);
    fn continue_at(&mut self, addr: Option<u32>);
}
