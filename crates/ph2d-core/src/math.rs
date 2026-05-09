//! Math types — newtypes over `glam::Vec2` to prevent mixing
//! coordinate spaces (per §15 anti-pattern "misturar pixel space e
//! world space na mesma função").
//!
//! Convention (per SKILL §11.1):
//! - `WorldPos`: Y-**up**, origin at metro 0, units in meters.
//! - `ScreenPos`: Y-**down**, origin top-left, units in physical pixels.
//!
//! The Y-up → Y-down flip happens **once** in
//! `ph2d-render::projection` (when populated in M5). Inside any single
//! function, only one convention is allowed.

use glam::Vec2;

/// Position in world space — Y-up, meters, `f32`.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WorldPos(pub Vec2);

impl WorldPos {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    #[inline]
    pub const fn x(self) -> f32 {
        self.0.x
    }

    #[inline]
    pub const fn y(self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for WorldPos {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<WorldPos> for Vec2 {
    fn from(p: WorldPos) -> Self {
        p.0
    }
}

/// Position in screen / texture space — Y-down, top-left origin,
/// physical pixels, `f32`.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ScreenPos(pub Vec2);

impl ScreenPos {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    #[inline]
    pub const fn x(self) -> f32 {
        self.0.x
    }

    #[inline]
    pub const fn y(self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for ScreenPos {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<ScreenPos> for Vec2 {
    fn from(p: ScreenPos) -> Self {
        p.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_pos_round_trips_to_vec2() {
        let p = WorldPos::new(1.0, 2.0);
        let v: Vec2 = p.into();
        assert_eq!(v, Vec2::new(1.0, 2.0));
        let p2: WorldPos = v.into();
        assert_eq!(p2, p);
    }

    #[test]
    fn screen_pos_round_trips_to_vec2() {
        let p = ScreenPos::new(100.0, 200.0);
        let v: Vec2 = p.into();
        assert_eq!(v, Vec2::new(100.0, 200.0));
    }

    /// Compile-time guard: `WorldPos` and `ScreenPos` are distinct
    /// types — mixing them is a `cargo build` error, not a runtime bug.
    #[test]
    fn world_and_screen_are_different_types() {
        let w = WorldPos::new(1.0, 1.0);
        let s = ScreenPos::new(1.0, 1.0);
        // The following would not compile (which is the point):
        //     let _ = w == s;
        // Use explicit conversion via Vec2 if you really need to compare.
        assert_eq!(Vec2::from(w), Vec2::from(s));
    }
}
