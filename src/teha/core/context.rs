//
// context.rs
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

use cairo;
use std::ops::{Deref};
use common::types::*;

pub struct Context<'a>{
    cr: &'a cairo::Context,
}

impl<'a> Context<'a> {
    pub fn new(cr: &'a cairo::Context, zoom_level: f64, translate: &Vector) -> Context<'a> {
        cr.translate(translate.x, translate.y);
        cr.scale(zoom_level, zoom_level);
        Context {
            cr: cr,
        }
    }

    pub fn get_zoom_level(&self) -> f64 {
        let matrix = self.cr.get_matrix();
        matrix.x0
    }

    pub fn set_source_rgb(&self, color: &RgbColor) {
        self.cr.set_source_rgb(
            color.red,
            color.green,
            color.blue
        );
    }

    pub fn set_source_rgba(&self, color: &RgbaColor) {
        self.cr.set_source_rgba(
            color.color.red,
            color.color.green,
            color.color.blue,
            color.alpha
        );
    }

    pub fn set_line_width(&self, value: f64) {
        self.cr.set_line_width(value);
    }

    pub fn set_dash(&self, dashes: &[f64], offset: f64) {
        self.cr.set_dash(dashes, offset);
    }

    pub fn move_to(&self, pos: &Point) {
        self.cr.move_to(pos.x, pos.y);
    }

    pub fn line_to(&self, pos: &Point) {
        self.cr.line_to(pos.x, pos.y);
    }

    pub fn rel_line_to(&self, vector: &Vector) {
        self.cr.rel_line_to(vector.x, vector.y);
    }

    pub fn polyline(&self, polyline: &Polyline) {
        let mut coords = polyline.coords().iter();
        if let Some(val) = coords.next() {
            self.move_to(val);
        } else {
            return;
        }
        for coord in coords {
            self.line_to(coord);
        }
    }

    pub fn rectangle(&self, rect: &Rectangle) {
        let top_left = rect.mins();
        let bottom_right = rect.maxs();
        let width = (bottom_right.x - top_left.x).abs();
        let height = (bottom_right.y - top_left.y).abs();
        self.cr.rectangle(
            top_left.x, top_left.y,
            width, height
        );
    }

    pub fn circle(&self, pos: &Point, radius: f64) {
        self.cr.arc(
            pos.x, pos.y,
            radius,
            0.0, (360_f64).to_radians()
        );
    }

    pub fn curve(
        &self,
        segment: &Segment,
        go_dir: &Vector,
        arrive_dir: &Vector
    ) {
        let start = segment.a();
        let end = segment.b();
        self.move_to(segment.a());
        self.cr.curve_to(
            // go direction (x, y)
            go_dir.x + start.x, go_dir.y + start.y,
            // arrive direction
            arrive_dir.x + end.x, arrive_dir.y + end.y,
            // end point
            end.x, end.y
        );
    }

    pub fn line(&self, segment: &Segment) {
        self.move_to(segment.a());
        self.line_to(segment.b());
    }

    pub fn fill_extents(&self) -> Rectangle {
        let (x1, y1, x2, y2) = self.cr.fill_extents();
        Rectangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2)
        )
    }

    pub fn stroke_extents(&self) -> Rectangle {
        let (x1, y1, x2, y2) = self.cr.stroke_extents();
        Rectangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2)
        )
    }

    pub fn in_stroke(&self, pos: &Point) -> bool {
        self.cr.in_stroke(pos.x, pos.y)
    }

    pub fn in_fill(&self, pos: &Point) -> bool {
        self.cr.in_fill(pos.x, pos.y)
    }

    pub fn user_to_device(&self, pos: &Point) -> Point {
        let (x, y) = self.cr.user_to_device(pos.x, pos.y);
        Point::new(x, y)
    }

    pub fn user_to_device_rect(&self, rect: &Rectangle) -> Rectangle {
        let mins = self.user_to_device(rect.mins());
        let maxs = self.user_to_device(rect.maxs());
        Rectangle::new(mins, maxs)
    }

    pub fn device_to_user(&self, pos: &Point) -> Point {
        let (x, y) = self.cr.device_to_user(pos.x, pos.y);
        Point::new(x, y)
    }

    pub fn device_to_user_rect(&self, rect: &Rectangle) -> Rectangle {
        let mins = self.device_to_user(rect.mins());
        let maxs = self.device_to_user(rect.maxs());
        Rectangle::new(mins, maxs)
    }
}

impl<'a> Deref for Context<'a> {
    type Target = cairo::Context;

    fn deref(&self) -> &Self::Target {
        &self.cr
    }
}
