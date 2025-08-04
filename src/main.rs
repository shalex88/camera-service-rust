mod controller;
mod core;
mod driver;
mod types;

use types::Zoom;

fn main() {
    let mut controller = controller::Controller::new(
        core::Core::new(
            driver::Driver::new(
                Zoom(0)
            )
        )
    );

    controller.run();
    controller.get_zoom();
    controller.set_zoom(Zoom(100));
    controller.get_zoom();
}