#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cardinal {
    North,
    South,
    West,
    East,
}

impl Cardinal {
    pub fn turn_cw(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::East => Self::South,
        }
    }

    pub fn turn_ccw(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::South => Self::East,
            Self::West => Self::South,
            Self::East => Self::North,
        }
    }

    pub fn dir(&self) -> glam::Vec3 {
        match self {
            Self::North => glam::Vec3::new(0.0, 1.0, 0.0),
            Self::South => glam::Vec3::new(0.0, -1.0, 0.0),
            Self::West => glam::Vec3::new(-1.0, 0.0, 0.0),
            Self::East => glam::Vec3::new(1.0, 0.0, 0.0),
        }
    }

    pub fn offsets(&self) -> (i32, i32) {
        match self {
            Self::North => (0, 1),
            Self::South => (0, -1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
        }
    }

    pub fn angle(&self) -> f32 {
        match self {
            Self::North => 0.0,
            Self::South => std::f32::consts::PI,
            Self::West => 3.0 * std::f32::consts::PI / 2.0,
            Self::East => std::f32::consts::PI / 2.0,
        }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t.clamp(0.0, 1.0) * (b - a)
}
