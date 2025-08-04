use crate::types::Zoom;

#[derive(Debug)]
pub struct Core {
    driver: crate::driver::Driver,
}

impl Core {
    pub fn new(driver: crate::driver::Driver) -> Self {
        Self { driver }
    }

    pub fn run(&self) {
        println!("Core is running");
        self.driver.run();
    }

    pub fn set_zoom(&mut self, zoom: Zoom) {
        if zoom.is_valid() {
            self.driver.set_zoom(zoom);
        } else {
            println!("Invalid zoom value: {:?}", zoom);
            return;
        }
    }

    pub fn get_zoom(&self) -> &Zoom {
        let zoom = self.driver.get_zoom();
        if !zoom.is_valid() {
            println!("Invalid zoom value: {:?}", zoom);
            //TODO: Handle error
        }

        zoom
    }
}
