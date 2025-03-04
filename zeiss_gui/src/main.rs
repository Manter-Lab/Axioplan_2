use std::{thread, time::Duration};

use axioplan::Scope;

fn main() {
    colog::basic_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut zeiss = Scope::new("/dev/ttyUSB0").unwrap();

    dbg!(zeiss.firmware_version());
}
