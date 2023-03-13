/// Point screen basis
#[derive(Debug, Clone, Copy)]
pub struct ScreenBasis;

/// Point game world basis
#[derive(Debug, Clone, Copy)]
pub struct GameBasis;

/// A point in the game world or screen.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point<Basis: Copy> {
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

    pub fn normalize(&self) -> Self {
        let length = self.length();
        if length.abs() > f32::EPSILON {
            Self {
                x: self.x / length,
                y: self.y / length,
                basis: std::marker::PhantomData,
            }
        } else {
            Self {
                x: self.x,
                y: self.y,
                basis: std::marker::PhantomData,
            }
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

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl<Basis: Copy> std::ops::Add for Point<Basis> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            basis: std::marker::PhantomData,
        }
    }
}

impl<Basis: Copy> std::ops::Sub for Point<Basis> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            basis: std::marker::PhantomData,
        }
    }
}

impl<Basis: Copy> std::ops::Mul<f32> for Point<Basis> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            basis: std::marker::PhantomData,
        }
    }
}

impl<Basis: Copy> std::ops::Div<f32> for Point<Basis> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            basis: std::marker::PhantomData,
        }
    }
}

impl<Basis: Copy> std::ops::AddAssign for Point<Basis> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<Basis: Copy> std::ops::SubAssign for Point<Basis> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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

#[derive(Debug, Clone, Copy)]
pub struct Line<Basis: Copy> {
    pub begin: Point<Basis>,
    pub end: Point<Basis>,
}

impl<Basis: Copy> std::fmt::Display for Line<Basis> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line a: {}, b: {}, c: {}, d: {}",
            self.begin.x, self.begin.y, self.end.x, self.end.y
        )
    }
}

impl<Basis: Copy> Line<Basis> {
    pub fn new(begin: Point<Basis>, end: Point<Basis>) -> Self {
        Self { begin, end }
    }

    pub fn normalize(&self) -> Self {
        let direction = self.end - self.begin;
        let length = direction.length();
        if length.abs() > f32::EPSILON {
            let norm_direction = direction / length;
            Self {
                begin: self.begin,
                end: self.begin + norm_direction,
            }
        } else {
            Self {
                begin: self.begin,
                end: self.end,
            }
        }
    }

    pub fn direction(&self) -> Point<Basis> {
        (self.end - self.begin).normalize()
    }

    pub fn length(&self) -> f32 {
        self.begin.distance_to(&self.end)
    }

    pub fn distance_to(&self, point: &Point<Basis>) -> f32 {
        let a = self.end - self.begin;
        let b = *point - self.begin;
        let c = *point - self.end;

        if a.dot(&b) > 0.0 && a.dot(&c) < 0.0 {
            (a.cross(&b) / a.length()).abs()
        } else {
            b.length().min(c.length())
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let a = self.end - self.begin;
        let b = other.end - other.begin;
        let c = other.begin - self.begin;

        let det = a.cross(&b);

        if det.abs() < f32::EPSILON {
            return false;
        }

        let t = c.cross(&b) / det;
        let u = c.cross(&a) / det;

        (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
    }
}

impl<Basis: Copy> std::ops::Add<Point<Basis>> for Line<Basis> {
    type Output = Self;

    fn add(self, rhs: Point<Basis>) -> Self::Output {
        Self {
            begin: self.begin + rhs,
            end: self.end + rhs,
        }
    }
}

impl<Basis: Copy> std::ops::Sub<Point<Basis>> for Line<Basis> {
    type Output = Self;

    fn sub(self, rhs: Point<Basis>) -> Self::Output {
        Self {
            begin: self.begin - rhs,
            end: self.end - rhs,
        }
    }
}
