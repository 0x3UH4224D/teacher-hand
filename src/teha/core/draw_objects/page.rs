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
use gdk::{EventMotion, EventButton, EventKey};
use gtk::{self};

use gettextrs::*;

use ncollide::bounding_volume::BoundingVolume;

use core::context::Context;
use common::types::*;
use super::{Name, Layer};
use super::layer::LayerTrait;


pub struct Page {
    size: Size<i32>,
    layers: Vec<Box<LayerTrait>>,
    active_layer_index: usize,
    color: Option<RgbColor>,
    border: Option<RgbColor>,
    grid: Option<RgbColor>,
    name: String,
    translate: Vector,
    zoom_level: f64,
}

impl Page {
    pub fn new() -> Page {
        Page {
            size: Size::new(800, 600),
            layers: vec![Box::new(Layer::new())],
            active_layer_index: 0,
            color: Some(RgbColor::new(1.0, 1.0, 1.0)),
            border: Some(RgbColor::new(0.47, 0.47, 0.47)), // #797979
            grid: None,
            name: gettext("Unnamed Page"),
            translate: Vector::new(0.0, 0.0),
            zoom_level: 1.0,
        }
    }

    pub fn get_size(&self) -> &Size<i32> {
        &self.size
    }

    pub fn get_mut_size(&mut self) -> &mut Size<i32> {
        &mut self.size
    }

    pub fn set_size(&mut self, size: Size<i32>) {
        self.size = size;
    }

    pub fn get_layers(&self) -> &Vec<Box<LayerTrait>> {
        &self.layers
    }

    pub fn get_mut_layers(&mut self) -> &mut Vec<Box<LayerTrait>> {
        &mut self.layers
    }

    pub fn set_layers(&mut self, layers: Vec<Box<LayerTrait>>) {
        self.layers = layers
    }

    pub fn get_active_layer(&self) -> &Box<LayerTrait> {
        &self.layers[self.active_layer_index]
    }

    pub fn get_mut_active_layer(&mut self) -> &mut Box<LayerTrait> {
        &mut self.layers[self.active_layer_index]
    }

    pub fn get_active_layer_index(&self) -> usize {
        self.active_layer_index
    }

    pub fn set_active_layer_index(&mut self, index: usize) {
        self.active_layer_index = index;
    }

    pub fn remove_shapes_in_creating_mode(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.remove_shapes_in_creating_mode();
        }
    }

    pub fn unselect_all_shapes(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.unselect_all_shapes();
        }
    }

    pub fn get_color(&self) -> &Option<RgbColor> {
        &self.color
    }

    pub fn get_mut_color(&mut self) -> &mut Option<RgbColor> {
        &mut self.color
    }

    pub fn set_color(&mut self, color: Option<RgbColor>) {
        self.color = color;
    }

    pub fn get_border(&self) -> &Option<RgbColor> {
        &self.border
    }

    pub fn get_mut_border(&mut self) -> &mut Option<RgbColor> {
        &mut self.border
    }

    pub fn get_grid(&self) -> &Option<RgbColor> {
        &self.grid
    }

    pub fn get_mut_grid(&mut self) -> &mut Option<RgbColor> {
        &mut self.grid
    }

    pub fn set_grid(&mut self, grid: Option<RgbColor>) {
        self.grid = grid;
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_mut_name(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_translate(&self) -> &Vector {
        &self.translate
    }

    pub fn get_mut_translate(&mut self) -> &mut Vector {
        &mut self.translate
    }

    pub fn set_translate(&mut self, translate: Vector) {
        self.translate = translate;
    }

    pub fn get_zoom_level(&self) -> f64 {
        self.zoom_level
    }

    pub fn set_zoom_level(&mut self, zoom_level: f64) {
        self.zoom_level = zoom_level;
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
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
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
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
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
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        for layer in self.layers.iter_mut().rev() {
            if layer.motion_notify(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn button_press(
        &mut self, event: &EventButton, options_widget: &gtk::Notebook
    ) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        for layer in self.layers.iter_mut().rev() {
            if layer.button_press(event, &pos, &cr, options_widget) {
                return true;
            }
        }
        false
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        let (x, y) = event.get_position();
        let pos = cr.device_to_user(&Point::new(x, y));

        for layer in self.layers.iter_mut().rev() {
            if layer.button_release(event, &pos, &cr) {
                return true;
            }
        }
        false
    }

    pub fn key_press(&mut self, event: &EventKey) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        for layer in self.layers.iter_mut().rev() {
            if layer.key_press(event, &cr) {
                return true;
            }
        }
        false
    }

    pub fn key_release(&mut self, event: &EventKey) -> bool {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
            .expect("Cairo: couldn't create surface");
        let cr = cairo::Context::new(&surface);
        let cr = Context::new(&cr, self.zoom_level, &self.translate);
        for layer in self.layers.iter_mut().rev() {
            if layer.key_release(event, &cr) {
                return true;
            }
        }
        false
    }
}

impl Name for Page {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: &String) {
        self.name.clone_from(name);
    }
}
