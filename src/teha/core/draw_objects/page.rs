//
// page.rs
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
use gdk::{EventMotion, EventButton};

use gettextrs::*;

use ncollide::bounding_volume::BoundingVolume;
use alga::linear::{AffineSpace};

use core::context::Context;
use common::types::*;
use super::{Name, LayerTrait, Layer};

pub struct Page {
    pub size: Size<i32>,
    pub layers: Vec<Box<LayerTrait>>,
    pub color: Option<RgbColor>,
    pub border: Option<RgbColor>,
    pub grid: Option<RgbColor>,
    pub name: String,
    pub translate: Vector,
    pub zoom_level: f64,
}

impl Page {
    fn page_bound(&self) -> Rectangle {
        let half_width = self.size.width as f64 / 2.0;
        let half_height = self.size.height as f64 / 2.0;
        let top_left = Point::new(-half_width, -half_height);
        let bottom_right = Point::new(half_width, half_height);
        Rectangle::new(top_left, bottom_right)
    }

    fn create_draw_context(
        &self
    ) -> Option<(cairo::ImageSurface, Context, Rectangle)> {

        let rect = match self.draw_extents() {
            None => return None,
            Some(val) => val,
        };
        let width = (rect.maxs().x - rect.mins().x).abs();
        let height = (rect.maxs().y - rect.mins().y).abs();
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            width as i32,
            height as i32
        );
        let context = Context::new(
            cairo::Context::new(&surface),
            self.zoom_level
        );

        // TODO: this may be done from Context so we can know the translate
        // from other methods in an wasy way.
        context.translate(rect.maxs().x.abs(), rect.maxs().y.abs());

        Some((surface, context, rect))
    }

    fn create_in_draw_context(&self) -> (cairo::ImageSurface, Context) {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let context = Context::new(
            cairo::Context::new(&surface),
            self.zoom_level
        );

        (surface, context)
    }

    fn line_width(&self) -> f64 {
        5.0
    }

    pub fn draw(&self, cr: &cairo::Context) {
        let (surface, context, _) = match self.create_draw_context() {
            None => return,
            Some(val) => val,
        };

        context.save();
        context.set_line_width(self.line_width());
        context.rectangle(&self.page_bound());
        if let Some(color) = self.border {
            context.set_source_rgb(&color);
            context.stroke_preserve();
        }
        if let Some(color) = self.color {
            context.set_source_rgb(&color);
            context.fill();
        }
        context.restore();

        for layer in self.layers.iter() {
            layer.draw(&context);
        }

        cr.save();
        cr.set_source_surface(&surface, self.translate.x, self.translate.y);
        cr.paint();
        cr.restore();
    }

    pub fn in_draw(&self, pos: &Point) -> bool {
        let (_, cr) = self.create_in_draw_context();
        for layer in self.layers.iter() {
            if layer.in_draw(&pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn draw_extents(&self) -> Option<Rectangle> {
        let (_, cr) = self.create_in_draw_context();

        cr.save();
        cr.set_line_width(self.line_width());
        cr.rectangle(&self.page_bound());

        let fill_extents = cr.fill_extents();
        let stroke_extents = cr.stroke_extents();
        let page_bound = fill_extents.merged(&stroke_extents);
        cr.restore();
        if self.layers.len() == 0 {
            return Some(page_bound);
        }

        let mut iter =
            self.layers
                .iter()
                .filter_map(|s| s.draw_extents(&cr));
        let init = match iter.next() {
            None => return Some(page_bound),
            Some(val) => val,
        };
        let mut result = iter.fold(init, |acc, ref x| acc.merged(x));
        result.merge(&page_bound);
        Some(result)
    }

    fn translate_position(&self, pos: (f64, f64)) -> Option<Point> {
        let mut translation = match self.draw_extents() {
            None => return None,
            Some(val) => Vector::new(-val.maxs().x, -val.maxs().y),
        };
        translation -= self.translate.clone();
        Some(Point::new(pos.0, pos.1).translate_by(&translation))
    }

    pub fn motion_notify(&mut self, event: &EventMotion) -> bool {
        let (_, cr) = self.create_in_draw_context();
        let pos = match self.translate_position(event.get_position()) {
            None => return false,
            Some(val) => val,
        };
        for layer in self.layers.iter_mut() {
            if layer.motion_notify(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn button_press(&mut self, event: &EventButton) -> bool {
        let (_, cr) = self.create_in_draw_context();
        let pos = match self.translate_position(event.get_position()) {
            None => return false,
            Some(val) => val,
        };
        for layer in self.layers.iter_mut() {
            if layer.button_press(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        let (_, cr) = self.create_in_draw_context();
        let pos = match self.translate_position(event.get_position()) {
            None => return false,
            Some(val) => val,
        };
        for layer in self.layers.iter_mut() {
            if layer.button_release(event, &pos, &cr) {
                return true;
            }
        }
        false
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            size: Size::new(800, 600),
            layers: vec![Box::new(Layer::new())],
            color: Some(RgbColor::new(1.0, 1.0, 1.0)),
            border: Some(RgbColor::new(0.47, 0.47, 0.47)), // #797979
            grid: None,
            name: gettext("Unnamed Page"),
            translate: Vector::new(800.0, 800.0),
            zoom_level: 1.0,
        }
    }
}

impl Name for Page {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}
