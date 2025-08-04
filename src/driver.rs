use crate::types::Zoom;

#[derive(Debug)]
pub struct Driver {
    zoom: Zoom,
}

impl Driver {
    pub fn new(zoom: Zoom) -> Self {
        Self { zoom }
    }

    pub fn run(&self) {
        println!("Driver is running");
    }

    pub fn set_zoom(&mut self, zoom: Zoom) {
        self.zoom = zoom;
    }

    pub fn get_zoom(&self) -> &Zoom {
        dbg!(&self.zoom);
        &self.zoom
    }
}
