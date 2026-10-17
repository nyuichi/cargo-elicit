pub struct Counter {
    pub value: u32,
}
impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn set(&mut self, value: u32) {
        self.value = value;
    }
    pub fn len(&self, bytes: &[u8]) -> usize {
        bytes.len()
    }
}
pub fn roundtrip(value: u32) -> u32 {
    let c = Counter { value };
    c.value()
}
