//
// types.rs
//
// Copyright (C) 2017 Muhannad Alrusayni <0x3UH4224D@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

use na;
use ncollide;
use palette;
use cairo;

pub type Point = na::Point2<f64>;
pub type Vector = na::Vector2<f64>;

pub type Translation = na::Translation2<f64>;
pub type Rotation = na::Rotation2<f64>;

pub type Segment = ncollide::shape::Segment2<f64>;
pub type Polyline = ncollide::procedural::Polyline<Point>;
pub type Cone = ncollide::shape::Cone<f64>;
pub type Rectangle = ncollide::bounding_volume::AABB<Point>;

pub type RgbColor = palette::Rgb<f64>;
pub type RgbaColor = palette::Alpha<RgbColor, f64>;

pub type Surface = cairo::ImageSurface;


// TODO: change this to na::Vector2<f64> or Vector
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size<T = f64> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T>
    where T: Add<Output = T> + AddAssign +
             Sub<Output = T> + SubAssign +
             Mul<Output = T> + MulAssign +
             Div<Output = T> + DivAssign +
             Eq + Clone {

    pub fn new(width: T, height: T) -> Self {
        Size {
            width: width,
            height: height,
        }
    }

    pub fn resize(&mut self, width: T, height: T) {
        self.width = width;
        self.height = height;
    }
}
