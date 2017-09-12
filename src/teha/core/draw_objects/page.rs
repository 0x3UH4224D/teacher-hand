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
use gdk::{self, EventMotion, EventButton, EventKey};

use gettextrs::*;

use ncollide::bounding_volume::BoundingVolume;

use core::context::Context;
use common::types::*;
use super::{Name, LayerTrait, Layer};

pub enum Actions {
    TranslateViewport(Vector),
}

pub struct Page {
    pub size: Size<i32>,
    pub layers: Vec<Box<LayerTrait>>,
    pub color: Option<RgbColor>,
    pub border: Option<RgbColor>,
    pub grid: Option<RgbColor>,
    pub name: String,
    pub translate: Vector,
    pub zoom_level: f64,
    pub action: Option<Actions>,
    pub drawarea_size: Size<i32>,
}

impl Page {
    pub fn new() -> Page {
        Page {
            size: Size::new(800, 600),
            layers: vec![Box::new(Layer::new())],
            color: Some(RgbColor::new(1.0, 1.0, 1.0)),
            border: Some(RgbColor::new(0.47, 0.47, 0.47)), // #797979
            grid: None,
            name: gettext("Unnamed Page"),
            translate: Vector::new(0.0, 0.0),
            zoom_level: 1.0,
            action: None,
            drawarea_size: Size::new(0, 0),
        }
    }

    fn page_bound(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0.0, 0.0),
            Point::new(self.size.width as f64, self.size.height as f64)
        )
    }

    fn line_width(&self) -> f64 {
        5.0
    }

    pub fn draw(&self, cr: &cairo::Context) {
        cr.save();
        let matrix = cr.get_matrix();

        let context =
            Context::new(cr, self.zoom_level, &self.translate);

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
        context.new_path();
        context.restore();

        for layer in self.layers.iter() {
            layer.draw(&context);
        }

        cr.set_matrix(matrix);
        cr.restore();
    }

    pub fn in_draw(&self, pos: &Point) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr =
            Context::new(&cr, self.zoom_level, &self.translate);
        for layer in self.layers.iter() {
            if layer.in_draw(&pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn draw_extents(&self) -> Option<Rectangle> {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr =
            Context::new(&cr, self.zoom_level, &Vector::new(0.0, 0.0));

        cr.save();
        cr.set_line_width(self.line_width());
        cr.rectangle(&self.page_bound());

        let fill_extents = cr.user_to_device_rect(&cr.fill_extents());
        let stroke_extents = cr.user_to_device_rect(&cr.stroke_extents());
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

    pub fn motion_notify(&mut self, event: &EventMotion) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr =
            Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        match self.action {
            Some(Actions::TranslateViewport(_origin_vec)) => {
                // FIXME: this is not the best way to translateviewport
                self.translate = Vector::new(pos.x, pos.y);
            },
            _ => {},
        }

        for layer in self.layers.iter_mut() {
            if layer.motion_notify(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn button_press(&mut self, event: &EventButton) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        for layer in self.layers.iter_mut() {
            if layer.button_press(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        for layer in self.layers.iter_mut() {
            if layer.button_release(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn key_press(&mut self, event: &EventKey) -> bool {
        if event.get_keyval() == gdk::enums::key::space {
            if self.action.is_none() {
                self.action = Some(Actions::TranslateViewport(self.translate.clone()));
            }
        }

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        for layer in self.layers.iter_mut() {
            if layer.key_press(event, &cr) {
                return true;
            }
        }
        false
    }

    pub fn key_release(&mut self, event: &EventKey) -> bool {
        if self.action.is_some() {
            self.action = None;
        }

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        for layer in self.layers.iter_mut() {
            if layer.key_release(event, &cr) {
                return true;
            }
        }
        false
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
