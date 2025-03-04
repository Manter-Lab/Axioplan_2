use std::{thread, time::Duration};

use axioplan::Scope;

fn main() {
    let mut zeiss = Scope::new("/dev/ttyUSB0").unwrap();

    zeiss.set_light_diaphragm_aperture(0).unwrap();

    loop {
        thread::sleep(Duration::from_millis(100));

        dbg!(zeiss.light_diaphragm_aperture().unwrap());
    }
}
