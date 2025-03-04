#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ScopeTurret {
    Unknown = 0,
    Reflector = 1,
    Objective = 2,
    DensityFilter1 = 3,
    DensityFilter2 = 4,
    Condenser = 5,
}

impl ScopeTurret {
    pub fn positions(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Reflector => 0,
            Self::Objective => 6,
            Self::DensityFilter1 => 4,
            Self::DensityFilter2 => 4,
            Self::Condenser => 0,
        }
    }
}
