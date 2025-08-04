#[derive(Debug)]
pub struct Controller {
    core: crate::core::Core,
}

impl Controller {
    pub fn new(core: crate::core::Core) -> Self {
        Self { core }
    }
    
    pub fn run(&self) {
        println!("Controller is running");
        self.core.run();
    }

    pub fn set_zoom(&mut self, zoom: crate::types::Zoom) {
        self.core.set_zoom(zoom);
    }

    pub fn get_zoom(&self) -> &crate::types::Zoom {
        self.core.get_zoom()
    }
}