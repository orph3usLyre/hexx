/// Type conversions
mod convert;
/// Traits implementations
mod impls;
/// Iterator tools module
mod iter;
/// Hex ring utils
mod rings;
#[cfg(test)]
mod tests;

pub use iter::HexIterExt;

use crate::{DiagonalDirection, Direction};
use glam::{IVec2, IVec3, Vec2};
use itertools::Itertools;
use std::cmp::{max, min};

/// Hexagonal [axial] coordinates
///
/// # Why Axial ?
///
/// Axial coordinates allow to compute and use *cubic* coordinates with less storage,
/// and allow:
/// - Vector operations
/// - Rotations
/// - Symmetry
/// - Simple algorithms
///
/// when *offset* and *doubled* coordinates don't. Furthermore, it makes the [`Hex`] behave like
/// classic 2D coordinates ([`IVec2`]) and therefore more user friendly.
///
/// Check out this [comparison] article for more information.
///
/// # Conversions
///
///  * Cubic: use [`Self::z`] to compute the third axis
///  * Offset: use [`Self::from_offset_coordinates`] and [`Self::from_offset_coordinates`]
///  * Doubled: use [`Self::from_doubled_coordinates`] and [`Self::from_doubled_coordinates`]
///
/// [comparison]: https://www.redblobgames.com/grids/hexagons/#coordinates-comparison
/// [axial]: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub struct Hex {
    /// `x` axial coordinate (sometimes called `q` or `i`)
    pub x: i32,
    /// `y` axial coordinate (sometimes called `r` or `j`)
    pub y: i32,
}

impl Hex {
    /// (0, 0)
    pub const ORIGIN: Self = Self::ZERO;
    /// (0, 0)
    pub const ZERO: Self = Self::new(0, 0);
    /// (1, 1)
    pub const ONE: Self = Self::new(1, 1);
    /// X (Q) axis (1, 0)
    pub const X: Self = Self::new(1, 0);
    /// Y (R) axis (0, 1)
    pub const Y: Self = Self::new(0, 1);
    /// Z (S) axis (0, -1)
    pub const Z: Self = Self::new(0, -1);

    /// Hexagon neighbor coordinates array, following [`Direction`] order
    ///
    /// ```txt
    ///            x Axis
    ///            ___
    ///           /   \
    ///       +--+  1  +--+
    ///      / 2  \___/  0 \
    ///      \    /   \    /
    ///       +--+     +--+
    ///      /    \___/    \
    ///      \ 3  /   \  5 /
    ///       +--+  4  +--+   y Axis
    ///           \___/
    /// ```
    pub const NEIGHBORS_COORDS: [Self; 6] = [
        Self::new(1, -1),
        Self::new(0, -1),
        Self::new(-1, 0),
        Self::new(-1, 1),
        Self::new(0, 1),
        Self::new(1, 0),
    ];

    /// ```txt
    ///            x Axis
    ///           \___/
    ///      \ 2  /   \ 1  /
    ///       +--+     +--+
    ///    __/    \___/    \__
    ///      \    /   \    /
    ///    3  +--+     +--+  0
    ///    __/    \___/    \__
    ///      \    /   \    /
    ///       +--+     +--+   y Axis
    ///      / 4  \___/  5 \
    /// ```
    pub const DIAGONAL_COORDS: [Self; 6] = [
        Self::new(2, -1),
        Self::new(1, -2),
        Self::new(-1, -1),
        Self::new(-2, 1),
        Self::new(-1, 2),
        Self::new(1, 1),
    ];

    #[inline]
    #[must_use]
    /// Instantiates a new hexagon from axial coordinates
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(3, 5);
    /// assert_eq!(coord.x, 3);
    /// assert_eq!(coord.y, 5);
    /// assert_eq!(coord.z(), -3-5);
    /// ```
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    /// Instantiates a new hexagon with all coordinates set to `v`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::splat(3);
    /// assert_eq!(coord.x, 3);
    /// assert_eq!(coord.y, 3);
    /// assert_eq!(coord.z(), -3-3);
    /// ```
    pub const fn splat(v: i32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    #[must_use]
    /// Instantiates new hexagonal coordinates in cubic space
    ///
    /// # Panics
    ///
    /// Will panic if the coordinates are invalid, meaning that the sum of coordinates is not equal
    /// to zero
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new_cubic(3, 5, -8);
    /// assert_eq!(coord.x, 3);
    /// assert_eq!(coord.y, 5);
    /// assert_eq!(coord.z(), -8);
    /// ```
    pub const fn new_cubic(x: i32, y: i32, z: i32) -> Self {
        assert!(x + y + z == 0);
        Self { x, y }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "q")]
    /// `x` coordinate (sometimes called `q` or `i`)
    pub const fn x(self) -> i32 {
        self.x
    }

    #[inline]
    #[must_use]
    #[doc(alias = "r")]
    /// `y` coordinate (sometimes called `r` or `j`)
    pub const fn y(self) -> i32 {
        self.y
    }

    #[inline]
    #[must_use]
    #[doc(alias = "s")]
    /// `z` coordinate (sometimes called `s` or `k`).
    ///
    /// This cubic space coordinate is computed as `-x - y`
    pub const fn z(self) -> i32 {
        -self.x - self.y
    }

    #[inline]
    #[must_use]
    /// Converts `self` to an array as `[x, y]`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(3, 5);
    /// let [x, y] = coord.to_array();
    /// assert_eq!(x, 3);
    /// assert_eq!(y, 5);
    /// ```
    pub const fn to_array(self) -> [i32; 2] {
        [self.x, self.y]
    }

    #[inline]
    #[must_use]
    /// Converts `self` to an array as `[x, y, z]`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(3, 5);
    /// let [x, y, z] = coord.to_array3();
    /// assert_eq!(x, 3);
    /// assert_eq!(y, 5);
    /// assert_eq!(z, -3-5);
    /// ```
    pub const fn to_array3(self) -> [i32; 3] {
        [self.x, self.y, self.z()]
    }

    #[must_use]
    #[inline]
    /// Converts `self` to an [`IVec2`].
    /// This operation is a direct mapping of coordinates, no hex to square coordinates are
    /// performed. To convert hex coordinates to world space use [`HexLayout`]
    ///
    /// [`HexLayout`]: crate::HexLayout
    pub const fn as_ivec2(self) -> IVec2 {
        IVec2 {
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    #[inline]
    /// Converts `self` to an [`IVec3`] using cubic coordinates.
    /// This operation is a direct mapping of coordinates.
    /// To convert hex coordinates to world space use [`HexLayout`]
    ///
    /// [`HexLayout`]: crate::HexLayout
    pub const fn as_ivec3(self) -> IVec3 {
        IVec3 {
            x: self.x,
            y: self.y,
            z: self.z(),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    #[inline]
    /// Converts `self` to a [`Vec2`].
    /// This operation is a direct mapping of coordinates.
    /// To convert hex coordinates to world space use [`HexLayout`]
    ///
    /// [`HexLayout`]: crate::HexLayout
    pub const fn as_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    #[inline]
    #[must_use]
    /// Negates the coordinate, giving its reflection (symmetry) around the origin.
    ///
    /// [`Hex`] implements [`Neg`] (`-` operator) but this method is `const`.
    ///
    /// [`Neg`]: std::ops::Neg
    pub const fn const_neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }

    #[inline]
    #[must_use]
    /// adds `self` and `other`.
    ///
    /// [`Hex`] implements [`Add`] (`+` operator) but this method is `const`.
    ///
    /// [`Add`]: std::ops::Add
    pub const fn const_add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    #[inline]
    #[must_use]
    /// substracts `self` and `rhs`.
    ///
    /// [`Hex`] implements [`Sub`] (`-` operator) but this method is `const`.
    ///
    /// [`Sub`]: std::ops::Sub
    pub const fn const_sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    /// Rounds floating point coordinates to [`Hex`].
    /// This method is used for operations like multiplications and divisions with floating point
    /// numbers.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let [x, y] = [0.6, 10.2];
    /// let coord = Hex::round((x, y));
    /// assert_eq!(coord.x, 1);
    /// assert_eq!(coord.y, 10);
    /// ```
    pub fn round((mut x, mut y): (f32, f32)) -> Self {
        let (mut x_r, mut y_r) = (x.round(), y.round());
        x -= x.round(); // remainder
        y -= y.round(); // remainder
        if x * x >= y * y {
            x_r += 0.5_f32.mul_add(y, x).round();
        }
        if x * x < y * y {
            y_r += 0.5_f32.mul_add(x, y).round();
        }
        Self::new(x_r as i32, y_r as i32)
    }

    #[inline]
    #[must_use]
    /// Computes the absolute value of `self`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(-1, -32).abs();
    /// assert_eq!(coord.x, 1);
    /// assert_eq!(coord.y, 32);
    /// ```
    pub const fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    #[inline]
    #[must_use]
    /// Computes coordinates length as a signed integer.
    /// The lenght of a [`Hex`] coordinate is equal to its distance from the origin.
    ///
    /// See [`Self::ulength`] for the unsigned version
    ///
    /// # Example
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(10, 0);
    /// assert_eq!(coord.length(), 10);
    /// ```
    pub const fn length(self) -> i32 {
        let [x, y, z] = [self.x.abs(), self.y.abs(), self.z().abs()];
        if x >= y && x >= z {
            x
        } else if y >= x && y >= z {
            y
        } else {
            z
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "unsigned_length")]
    /// Computes coordinates length as an unsigned integer
    /// The lenght of a [`Hex`] coordinate is equal to its distance from the origin.
    ///
    /// See [`Self::length`] for the signed version
    ///
    /// # Example
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(10, 0);
    /// assert_eq!(coord.ulength(), 10);
    /// ```
    pub const fn ulength(self) -> u32 {
        let [x, y, z] = [
            self.x.unsigned_abs(),
            self.y.unsigned_abs(),
            self.z().unsigned_abs(),
        ];
        if x >= y && x >= z {
            x
        } else if y >= x && y >= z {
            y
        } else {
            z
        }
    }

    #[inline]
    #[must_use]
    /// Computes the distance from `self` to `rhs` in hexagonal space as a signed integer
    ///
    /// See [`Self::unsigned_distance_to`] for the unsigned version
    pub const fn distance_to(self, rhs: Self) -> i32 {
        self.const_sub(rhs).length()
    }

    #[inline]
    #[must_use]
    /// Computes the distance from `self` to `rhs` in hexagonal space as an unsigned integer
    ///
    /// See [`Self::distance_to`] for the signed version
    pub const fn unsigned_distance_to(self, rhs: Self) -> u32 {
        self.const_sub(rhs).ulength()
    }

    #[inline]
    #[must_use]
    /// Retrieves the hexagonal neighbor coordinates matching the given `direction`
    pub const fn neighbor_coord(direction: Direction) -> Self {
        Self::NEIGHBORS_COORDS[direction as usize]
    }

    #[inline]
    #[must_use]
    /// Retrieves the neighbor coordinates matching the given `direction`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(10, 5);
    /// let bottom = coord.neighbor(Direction::Bottom);
    /// assert_eq!(bottom, Hex::new(10, 6));
    /// ```
    pub const fn neighbor(self, direction: Direction) -> Self {
        self.const_add(Self::neighbor_coord(direction))
    }

    #[inline]
    #[must_use]
    /// Retrieves the direction of the given neighbor. Will return `None` if `other` is not a neighbor
    /// of `self`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// let coord = Hex::new(10, 5);
    /// let bottom = coord.neighbor(Direction::Bottom);
    /// let dir = coord.neighbor_direction(bottom).unwrap();
    /// assert_eq!(dir, Direction::Bottom);
    /// ```
    pub fn neighbor_direction(self, other: Self) -> Option<Direction> {
        Direction::iter().find(|&dir| self.neighbor(dir) == other)
    }

    #[inline]
    /// Retrieves all directions in the line between `self` and `other`
    pub fn directions_to(self, other: Self) -> impl Iterator<Item = Direction> {
        self.line_to(other)
            .tuple_windows::<(_, _)>()
            .filter_map(|(a, b)| a.neighbor_direction(b))
    }

    #[must_use]
    /// Find in which [`DiagonalDirection`] wedge `rhs` is relative to `self`
    pub fn diagonal_to(self, rhs: Self) -> DiagonalDirection {
        let [x, y, z] = (rhs - self).to_array3();
        let [xa, ya, za] = [x.abs(), y.abs(), z.abs()];
        let (v, dir) = match xa.max(ya).max(za) {
            v if v == xa => (x, DiagonalDirection::Right),
            v if v == ya => (y, DiagonalDirection::BottomLeft),
            v if v == za => (z, DiagonalDirection::TopLeft),
            _ => unreachable!(),
        };
        if v < 0 {
            -dir
        } else {
            dir
        }
    }

    #[must_use]
    /// Find in which [`Direction`] wedge `rhs` is relative to `self`
    pub fn direction_to(self, rhs: Self) -> Direction {
        let [x, y, z] = (rhs - self).to_array3();
        let [x, y, z] = [y - x, z - y, x - z];
        let [xa, ya, za] = [x.abs(), y.abs(), z.abs()];
        let (v, dir) = match xa.max(ya).max(za) {
            v if v == xa => (x, Direction::BottomLeft),
            v if v == ya => (y, Direction::Top),
            v if v == za => (z, Direction::BottomRight),
            _ => unreachable!(),
        };
        if v < 0 {
            -dir
        } else {
            dir
        }
    }

    #[inline]
    #[must_use]
    /// Retrieves all 6 neighbor coordinates around `self`
    pub fn all_neighbors(self) -> [Self; 6] {
        Self::NEIGHBORS_COORDS.map(|n| self + n)
    }

    #[inline]
    #[must_use]
    /// Retrieves all 6 neighbor diagonal coordinates around `self`
    pub fn all_diagonals(self) -> [Self; 6] {
        Self::DIAGONAL_COORDS.map(|n| self + n)
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around [`Hex::ZERO`] counter clockwise (by -60 degrees)
    pub const fn left(self) -> Self {
        Self::new(-self.z(), -self.x)
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around `center` counter clockwise (by -60 degrees)
    pub const fn left_around(self, center: Self) -> Self {
        self.const_sub(center).left().const_add(center)
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around [`Hex::ZERO`] counter clockwise by `m` (by `-60 * m` degrees)
    pub const fn rotate_left(self, m: u32) -> Self {
        match m % 6 {
            1 => self.left(),
            2 => self.left().left(),
            3 => self.const_neg(),
            4 => self.right().right(),
            5 => self.right(),
            _ => self,
        }
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around `center` counter clockwise by `m` (by `-60 * m` degrees)
    pub const fn rotate_left_around(self, center: Self, m: u32) -> Self {
        self.const_sub(center).rotate_left(m).const_add(center)
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around [`Hex::ZERO`] clockwise (by 60 degrees)
    pub const fn right(self) -> Self {
        Self::new(-self.y, -self.z())
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around `center` clockwise (by 60 degrees)
    pub const fn right_around(self, center: Self) -> Self {
        self.const_sub(center).right().const_add(center)
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around [`Hex::ZERO`] clockwise by `m` (by `60 * m` degrees)
    pub const fn rotate_right(self, m: u32) -> Self {
        match m % 6 {
            1 => self.right(),
            2 => self.right().right(),
            3 => self.const_neg(),
            4 => self.left().left(),
            5 => self.left(),
            _ => self,
        }
    }

    #[inline]
    #[must_use]
    /// Rotates `self` around `center` clockwise by `m` (by `60 * m` degrees)
    pub const fn rotate_right_around(self, center: Self, m: u32) -> Self {
        self.const_sub(center).rotate_right(m).const_add(center)
    }

    #[inline]
    #[must_use]
    #[doc(alias = "reflect_q")]
    /// Computes the reflection of `self` accross[`Hex::X`]
    pub const fn reflect_x(self) -> Self {
        Self::new(self.x, self.z())
    }

    #[inline]
    #[must_use]
    #[doc(alias = "reflect_r")]
    /// Computes the reflection of `self` accross [`Hex::Y`]
    pub const fn reflect_y(self) -> Self {
        Self::new(self.z(), self.y)
    }

    #[inline]
    #[must_use]
    #[doc(alias = "reflect_s")]
    /// Computes the reflection of `self` accross [`Hex::Z`]
    pub const fn reflect_z(self) -> Self {
        Self::new(self.y, self.x)
    }

    #[allow(clippy::cast_precision_loss)]
    /// Computes all coordinates in a line from `self` to `other`.
    ///
    /// # Example
    /// ```rust
    /// # use hexx::*;
    /// let start = Hex::ZERO;
    /// let end = Hex::new(5, 0);
    ///
    /// let line: Vec<Hex> = start.line_to(end).collect();
    /// assert_eq!(line.len(), 6);
    /// ````
    pub fn line_to(self, other: Self) -> impl Iterator<Item = Self> {
        let distance = self.distance_to(other);
        let [a, b]: [Vec2; 2] = [self.as_vec2(), other.as_vec2()];
        (0..=distance).map(move |step| a.lerp(b, step as f32 / distance as f32).into())
    }

    /// Performs a linear interpolation between `self` and `rhs` based on the value `s`.
    ///
    /// When `s` is `0.0`, the result will be equal to `self`.  When `s` is `1.0`, the result
    /// will be equal to `rhs`. When `s` is outside of range `[0, 1]`, the result is linearly
    /// extrapolated.
    #[doc(alias = "mix")]
    #[inline]
    #[must_use]
    pub fn lerp(self, rhs: Self, s: f32) -> Self {
        let [start, end]: [Vec2; 2] = [self.as_vec2(), rhs.as_vec2()];
        start.lerp(end, s).into()
    }

    #[allow(clippy::cast_possible_wrap)]
    /// Retrieves all [`Hex`] around `self` in a given `range`
    pub fn range(self, range: u32) -> impl Iterator<Item = Self> {
        let range = range as i32;
        (-range..=range).flat_map(move |x| {
            (max(-range, -x - range)..=min(range, range - x)).map(move |y| self + Self::new(x, y))
        })
    }

    #[inline]
    #[must_use]
    /// Counts how many coordinates there are in the given `range`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hexx::*;
    /// assert_eq!(Hex::range_count(15), 721);
    /// ```
    pub const fn range_count(range: u32) -> usize {
        (3 * range * (range + 1) + 1) as usize
    }

    #[must_use]
    /// Wraps `self` in an hex range around the origin ([`Hex::ZERO`]).
    /// this allows for seamless *wraparound* hexagonal maps.
    /// See this [article] for more information.
    ///
    /// Use [`HexMap`] for improved wrapping
    ///
    /// [`HexMap`]: crate::HexMap
    /// [article]: https://www.redblobgames.com/grids/hexagons/#wraparound
    pub fn wrap_in_range(self, radius: u32) -> Self {
        self.wrap_with(radius, &Self::wraparound_mirrors(radius))
    }

    #[must_use]
    /// Wraps `self` in an hex range around the origin ([`Hex::ZERO`]) using custom mirrors.
    ///
    /// # Panics
    ///
    /// Will panic with invalid `mirrors`
    /// Prefer using [`Self::wrap_in_range`] or [`HexMap`] for safe wrapping.
    ///
    /// [`HexMap`]: crate::HexMap
    pub fn wrap_with(self, radius: u32, mirrors: &[Self; 6]) -> Self {
        if self.ulength() <= radius {
            return self;
        }
        let mut res = self;
        while res.ulength() > radius {
            let mirror = mirrors
                .iter()
                .copied()
                .sorted_unstable_by_key(|m| res.distance_to(*m))
                .next()
                .unwrap(); // Safe
            res -= mirror;
        }
        res
    }

    /// Computes the 6 mirror centers of the origin for hexagonal *wraparound* maps
    /// of given `radius`.
    ///
    /// # Notes
    /// * See [`Self::wrap_with`] for a usage
    /// * Use [`HexMap`] for improved wrapping
    /// * See this [article] for more information.
    ///
    /// [`HexMap`]: crate::HexMap
    /// [article]: https://www.redblobgames.com/grids/hexagons/#wraparound
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn wraparound_mirrors(radius: u32) -> [Self; 6] {
        let radius = radius as i32;
        let mirror = Self::new(2 * radius + 1, -radius);
        let [center, left, right] = [mirror, mirror.left(), mirror.right()];
        [
            left,
            center,
            right,
            left.const_neg(),   // -left
            center.const_neg(), // -center
            right.const_neg(),  // -right
        ]
    }
}