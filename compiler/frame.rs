use object::Closure;

#[derive(Debug, Clone)]
pub struct Frame {
    pub closure: Closure,
    pub ip: i32,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(closure: Closure, base_pointer: usize) -> Self {
        Frame {
            closure,
            ip: -1,
            base_pointer,
        }
    }

    pub fn instructions(&self) -> &[u8] {
        &self.closure.func.instructions
    }
}
