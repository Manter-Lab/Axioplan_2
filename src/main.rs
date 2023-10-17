use zeiss_control::{Scope, ScopeTurret};

fn main() {
    let mut scope = Scope::new(
        "/dev/ttyUSB1",
        "/dev/ttyUSB0"
    );

    //scope.query_scope_print("HPCS4,240\r").unwrap(); // Set substage aperture position
    scope.query_scope_print("HPCs4,1\r").unwrap();   // Get substage aperture position

    scope.set_turret_pos(ScopeTurret::Objective, 1).unwrap();

    loop {
        println!("{}: {}: {}",
            scope.turret_pos(ScopeTurret::Objective).unwrap(),
            scope.ld_pos().unwrap(),
            scope.focus_dist().unwrap()
        );
    }
}
