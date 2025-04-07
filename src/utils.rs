use std::f32::consts::PI;

use serde::{Serialize, Deserialize};
use strum::EnumIter;

pub type Erm<T> = color_eyre::Result<T>;

pub fn erm<E, T>(e: E) -> Erm<T> where E: std::error::Error + std::marker::Send + std::marker::Sync + 'static {
    Err(e.into())
}

pub struct ErrorHandler;
impl color_eyre::eyre::EyreHandler for ErrorHandler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }
        let mut first = true;
        if let Some(s) = error.source() {
            let errors: Vec<_> = std::iter::successors(Some(s), |e| (*e).source()).collect();
            for err in errors.iter().rev() {
                writeln!(f)?; write!(f, "{}{}", if first {""} else {" - "}, err)?;
                first = false;
            }
        }
        writeln!(f)?; write!(f, "{}{}", if first {""} else {" - "}, error)?;
        Ok(())
    }
}

pub fn install_error_handler() {
    let (panic_hook, _) = color_eyre::config::HookBuilder::default().into_hooks();
    panic_hook.install();
    color_eyre::eyre::set_hook(Box::new(move |_| Box::new(ErrorHandler))).expect("failed to install error handler");
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Cardinal {
    North,
    South,
    West,
    East,
}

impl Cardinal {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

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
            Self::South => PI,
            Self::West => PI / 2.0,
            Self::East => 3.0 * PI / 2.0,
        }
    }

    pub fn turn_by(&self, o: &Self) -> Self {
        match o {
            Self::North => self.clone(),
            Self::South => self.turn_cw().turn_cw(),
            Self::West => self.turn_cw(),
            Self::East => self.turn_ccw(),
        }
    }

    pub fn angle_between(&self, o: &Self) -> f32 {
        if o == &self.turn_cw() { -PI / 2.0 }
        else if o == &self.turn_ccw() { PI / 2.0 }
        else if o == &self.turn_cw().turn_cw() { PI }
        else { 0.0 }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t.clamp(0.0, 1.0) * (b - a)
}

pub fn dir_lerp(a: &glam::Vec3, b: glam::Vec3, t: f32) -> glam::Vec3 {
    let dirrotaxis = a.cross(b).normalize();
    let dirrotangle = a.angle_between(b);
    let dirrotfull = glam::Quat::from_axis_angle(dirrotaxis, dirrotangle);
    let dirrot = glam::Quat::IDENTITY.slerp(dirrotfull, t);
    dirrot.mul_vec3(a.clone())
}
