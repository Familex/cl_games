/// Point screen basis
#[derive(Debug, Clone, Copy)]
pub struct ScreenBasis;

/// Point game world basis
#[derive(Debug, Clone, Copy)]
pub struct GameBasis;

/// A point in the game world or screen.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point<Basis> {
    pub x: f32,
    pub y: f32,
    basis: std::marker::PhantomData<Basis>,
}

impl<Basis> Point<Basis>
where
    Basis: Copy,
{
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            basis: std::marker::PhantomData,
        }
    }

    pub fn cross(&self, other: &Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn scale(&self, k: f32) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
            basis: std::marker::PhantomData,
        }
    }

    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        (*other - *self).length()
    }

    pub fn compare(&self, other: &Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() < epsilon && (self.y - other.y).abs() < epsilon
    }
}

impl<Basis> std::ops::Add for Point<Basis> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            basis: std::marker::PhantomData,
        }
    }
}

impl<Basis> std::ops::Sub for Point<Basis> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            basis: std::marker::PhantomData,
        }
    }
}

impl From<Point<ScreenBasis>> for Point<GameBasis> {
    fn from(point: Point<ScreenBasis>) -> Self {
        Point {
            x: point.x / 2.0,
            y: point.y,
            basis: std::marker::PhantomData,
        }
    }
}

impl From<Point<GameBasis>> for Point<ScreenBasis> {
    fn from(point: Point<GameBasis>) -> Self {
        Point {
            x: point.x * 2.0,
            y: point.y,
            basis: std::marker::PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum BoundsCollision {
    Top,
    Bottom,
    Left,
    Right,
}

impl Point<ScreenBasis> {
    pub fn bounds_check(&self, screen_width: u16, screen_height: u16) -> Option<BoundsCollision> {
        if self.y.round() < 0.0 {
            Some(BoundsCollision::Top)
        } else if self.y.round() > screen_height as f32 {
            Some(BoundsCollision::Bottom)
        } else if self.x.round() < 0.0 {
            Some(BoundsCollision::Left)
        } else if self.x.round() > screen_width as f32 - 2.0 {
            Some(BoundsCollision::Right)
        } else {
            None
        }
    }
}

impl Point<GameBasis> {
    pub fn bounds_check(&self, width: u16, height: u16) -> Option<BoundsCollision> {
        Point::<ScreenBasis>::from(*self).bounds_check(width, height)
    }
}
