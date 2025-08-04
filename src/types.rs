#[derive(Debug)]
pub struct Zoom(pub u32);

impl Zoom {
    pub fn is_valid(&self) -> bool {
        self.0 <= 100
    }
}