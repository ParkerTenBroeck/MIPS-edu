

#[derive(Copy, Clone, PartialEq)]
pub struct BufferIndex {
    pub index: usize,
    pub index_real: usize,
    pub line: usize,
    pub column: usize,
}

impl BufferIndex {
    pub fn new() -> Self {
        BufferIndex {
            index: 0,
            index_real: 0,
            line: 0,
            column: 0,
        }
    }
}