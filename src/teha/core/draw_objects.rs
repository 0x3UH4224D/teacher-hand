//
// draw_objects.rs
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
use gdk::{EventMotion, EventButton, EventType};

use gettextrs::*;

use ncollide;
use ncollide::transformation::ToPolyline;
use ncollide::bounding_volume::{AABB, BoundingVolume};
use na;
use alga::linear::{Transformation, ProjectiveTransformation,
                   EuclideanSpace, AffineSpace};

use palette::{self};

use common::types::Size;

pub type Point = na::Point2<f64>;
pub type Vector = na::Vector2<f64>;
pub type Translation = na::Translation2<f64>;
pub type Rotation = na::Rotation2<f64>;
pub type Segment = ncollide::shape::Segment2<f64>;
pub type Cone = ncollide::shape::Cone<f64>;

pub type RgbColor = palette::Rgb<f64>;
pub type RgbaColor = palette::Alpha<RgbColor, f64>;
pub type Context = cairo::Context;
pub type Surface = cairo::ImageSurface;

pub trait Draw {
    fn draw(&self, cr: &Context, zoom_level: f64, translate: &Vector);
    fn in_draw(&self, pos: &Point, zoom_level: f64, translate: &Vector) -> bool;
    fn draw_extents(
        &self,
        zoom_level: f64,
        translate: &Vector
    ) -> Option<AABB<Point>>;
}

pub trait Name {
    fn name(&self) -> &str;
    fn set_name(&mut self, name: &str);
}

pub trait Move {
    fn position(&self) -> Point;
    fn move_to(&mut self, &Point);
    fn translate_by(&mut self, &Translation);
    fn rotate_by(&mut self, &Rotation, &Vector);
}

pub trait Lock {
    fn is_locked(&self) -> bool;
    fn lock(&mut self);
    fn unlock(&mut self);
    fn toggle_lock(&mut self) -> bool;
}

pub trait Visible {
    fn is_visible(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn toggle_visible(&mut self) -> bool;
}

pub trait Container {
    fn add(&mut self, child: Box<ShapeTrait>);
    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>>;
}

pub trait Event {
    // like gtk::Widget events it return "TRUE to stop other handlers from
    // being invoked for the event. FALSE to propagate the event further."
    #[allow(unused_variables)]
    fn motion_notify(
        &mut self,
        event: &EventMotion,
        zoom_level: f64,
        translate: &Vector
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_press(
        &mut self,
        event: &EventButton,
        zoom_level: f64,
        translate: &Vector
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_release(
        &mut self,
        event: &EventButton,
        zoom_level: f64,
        translate: &Vector
    ) -> bool { false }
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
}

impl Page {
    pub fn draw(&self, cr: &Context) {
        cr.save();
        cr.translate(
            self.translate.x,
            self.translate.y
        );
        cr.scale(self.zoom_level, self.zoom_level);
        cr.set_line_width(5.0);
        cr.rectangle(
            -self.size.width as f64 / 2.0,
            -self.size.height as f64 / 2.0,
            self.size.width as f64,
            self.size.height as f64
        );
        if let Some(color) = self.border {
            cr.set_source_rgb(
                color.red,
                color.green,
                color.blue
            );
            cr.stroke_preserve();
        }
        if let Some(color) = self.color {
            cr.set_source_rgb(
                color.red,
                color.green,
                color.blue,
            );
            cr.fill();
        }
        cr.new_path();
        cr.identity_matrix();
        cr.restore();
        for layer in self.layers.iter() {
            layer.draw(cr, self.zoom_level, &self.translate);
        }
    }

    pub fn in_draw(&self, pos: &Point) -> bool {
        for layer in self.layers.iter() {
            if layer.in_draw(&pos, self.zoom_level, &self.translate) {
                return true;
            }
        }
        false
    }

    pub fn draw_extents(&self) -> Option<AABB<Point>> {
        let mut page_bound = AABB::new(
            Point::new(0.0, 0.0),
            Point::new(self.size.width as f64,
                       self.size.height as f64)
        );
        let mins = page_bound.mins()
            .scale_by(self.zoom_level)
            .translate_by(&self.translate);
        let maxs = page_bound.maxs()
            .scale_by(self.zoom_level)
            .translate_by(&self.translate);
        page_bound = AABB::new(mins, maxs)
            // the draw line width for page border is 5 2.5 in the page
            // and 2.5 out the page bounds
            .loosened(2.5 * self.zoom_level);
        if self.layers.len() == 0 {
            return Some(page_bound);
        }

        let mut iter =
            self.layers
                .iter()
                .filter_map(
                    |s| s.draw_extents(self.zoom_level, &self.translate)
                );
        let init = match iter.next() {
            None => return Some(page_bound),
            Some(val) => val,
        };
        let mut result = iter.fold(init, |acc, ref x| acc.merged(x));
        result.merge(&page_bound);
        Some(result)
    }

    pub fn motion_notify(&mut self, event: &EventMotion) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.motion_notify(event, self.zoom_level, &self.translate) {
                return true;
            }
        }
        false
    }

    pub fn button_press(&mut self, event: &EventButton) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.button_press(event, self.zoom_level, &self.translate) {
                return true;
            }
        }
        false
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.button_release(event, self.zoom_level, &self.translate) {
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
            translate: Vector::new(0.0, 0.0),
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

pub trait LayerTrait: Draw + Name + Lock + Visible + Container + Event {}

pub struct Layer {
    pub children: Vec<Box<ShapeTrait>>,
    pub name: String,
    pub lock: bool,
    pub visible: bool,
}

impl Layer {
    fn new() -> Self {
        // FIXME: this line for testing/debugging
        // and should be removeed when release

        let line_arrow = LineArrow::new(
            RgbaColor::new(0.5, 0.5, 1.0, 1.0),
            10.0,
            cairo::LineCap::Round,
            cairo::LineJoin::Round,
            vec![],
            0.0,
            false,
            Segment::new(Point::new(-50.0, -50.0),
                         Point::new(50.0, 50.0)),
        );

        let mut children: Vec<Box<ShapeTrait>> = vec![];
        children.push(Box::new(line_arrow));

        Layer {
            children: children,
            name: gettext("Unnamed Layer"),
            lock: false,
            visible: true,
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            children: vec![],
            name: gettext("Unnamed Layer"),
            lock: false,
            visible: true,
        }
    }
}

impl LayerTrait for Layer {}

impl Draw for Layer {
    fn draw(&self, cr: &Context, zoom_level: f64, translate: &Vector) {
        if !self.visible {
            return;
        }

        for child in self.children.iter() {
            child.draw(cr, zoom_level, translate);
        }
    }

    fn in_draw(
        &self,
        pos: &Point,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        if !self.visible {
            return false;
        }

        for child in self.children.iter() {
            if child.in_draw(pos, zoom_level, translate) {
                return true;
            }
        }
        false
    }

    fn draw_extents(
        &self,
        zoom_level: f64,
        translate: &Vector
    ) -> Option<AABB<Point>> {
        if self.children.len() == 0 {
            return None;
        }

        let mut iter =
            self.children
                .iter()
                .filter_map(|s| s.draw_extents(zoom_level, translate));
        let init = match iter.next() {
            None => return None,
            Some(val) => val,
        };
        let result = iter.fold(init, |acc, ref x| acc.merged(x));
        Some(result)
    }
}

impl Name for Layer {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Lock for Layer {
    fn is_locked(&self) -> bool {
        self.lock
    }

    fn lock(&mut self) {
        if !self.lock {
            self.lock = true;
        }
    }

    fn unlock(&mut self) {
        if self.lock {
            self.lock = false;
        }
    }

    fn toggle_lock(&mut self) -> bool {
        self.lock = !self.lock;
        self.lock
    }
}

impl Visible for Layer {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        if !self.visible {
            self.visible = true;
        }
    }

    fn hide(&mut self) {
        if self.visible {
            self.visible = false;
        }
    }

    fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }
}

impl Container for Layer {
    fn add(&mut self, child: Box<ShapeTrait>) {
        self.children.push(child);
    }

    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>> {
        if index > self.children.len() {
            None
        } else {
            Some(self.children.remove(index))
        }
    }
}

impl Event for Layer {
    fn motion_notify(
        &mut self,
        event: &EventMotion,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.motion_notify(event, zoom_level, translate) {
                return true;
            }
        }
        false
    }

    fn button_press(
        &mut self,
        event: &EventButton,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_press(event, zoom_level, translate) {
                return true;
            }
        }
        false
    }

    fn button_release(
        &mut self,
        event: &EventButton,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_release(event, zoom_level, translate) {
                return true;
            }
        }
        false
    }
}

pub trait ShapeTrait: Draw + Name + Move + Lock +
          Visible + Container + Event {}

#[derive(Debug, PartialEq, Eq)]
pub enum LineArrowControllers {
    Body,
    StartPoint,
    EndPoint,
    GoDirection,
    ArriveDirection,
}

pub struct LineArrow {
    pub children: Vec<Box<ShapeTrait>>,
    // ID field
    pub name: String,

    // control fields
    pub lock: bool,
    pub selected: bool,
    // this field for Event trait, it contains the selected controller and click
    // position, it contains None if the user click outside the draw area.
    selected_controller: Option<(LineArrowControllers, Point)>,

    // draw fields
    pub visible: bool,
    pub color: RgbaColor,
    pub width: f64,
    pub cap: cairo::LineCap,
    pub join: cairo::LineJoin,
    pub dashes: Vec<f64>,
    pub offset: f64,

    // Segment field
    pub segment: Segment,

    // Curve fields
    pub curve_like: bool,
    // these vector needed if we want to convert this line to curve
    pub go_dir: Vector, // dir refer to direction
    pub arrive_dir: Vector,
}

impl LineArrow {
    pub fn new(color: RgbaColor, width: f64, cap: cairo::LineCap,
           join: cairo::LineJoin, dashes: Vec<f64>, offset: f64,
           curve_like: bool, segment: Segment) -> Self {
        LineArrow {
            children: vec![],
            name: String::new(),
            lock: false,
            selected: false,
            selected_controller: None,
            visible: true,
            color: color,
            width: width,
            cap: cap,
            join: join,
            dashes: dashes,
            offset: offset,
            segment: segment,
            curve_like: curve_like,
            go_dir: Vector::new(0.0, 0.0),
            arrive_dir: Vector::new(0.0, 0.0),
        }
    }

    pub fn new_from_segment(segment: Segment) -> Self {
        LineArrow {
            children: vec![],
            name: String::new(),
            lock: false,
            selected: false,
            selected_controller: None,
            visible: true,
            color: RgbaColor::new(0.0, 0.0, 0.0, 1.0),
            width: 10.0,
            cap: cairo::LineCap::Round,
            join: cairo::LineJoin::Round,
            dashes: vec![],
            offset: 0.0,
            segment: segment,
            curve_like: false,
            go_dir: Vector::new(0.0, 0.0),
            arrive_dir: Vector::new(0.0, 0.0),
        }
    }

    fn radius(&self) -> f64 {
        if self.width < 10.0  {
            3.0
        } else if self.width > 20.0 {
            9.0
        } else {
            // radius will be in range 4.0-8.0
            self.width * 0.40
        }
    }

    fn fill_color(&self) -> RgbColor {
        RgbColor::new(0.97, 0.97, 1.0) // #F8F8FF
    }

    fn stroke_color(&self) -> RgbColor {
        RgbColor::new(0.47, 0.53, 0.60) // #778899
    }

    fn line_width(&self) -> f64 {
        2.0
    }

    fn select_controller(
        &self,
        pos: &Point,
        zoom_level: f64,
        translate: &Vector
    ) -> Option<LineArrowControllers> {
        // we create zero size image since we don't really need it, but it's
        // required to create cairo::Context
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = &Context::new(&surface);
        let pos = pos.clone() - translate.clone();

        cr.save();
        self.draw_start_point(cr, zoom_level, translate, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::StartPoint);
        }
        cr.restore();

        cr.save();
        self.draw_end_point(cr, zoom_level, translate, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::EndPoint);
        }
        cr.restore();

        cr.save();
        self.draw_go_direction(cr, zoom_level, translate, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::GoDirection);
        }
        cr.restore();

        cr.save();
        self.draw_arrive_direction(cr, zoom_level, translate, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::ArriveDirection);
        }
        cr.restore();

        cr.save();
        self.draw_segment(cr, zoom_level, translate, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        cr.save();
        self.draw_head(cr, zoom_level, translate, false);
        if cr.in_fill(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        cr.save();
        self.draw_tail(cr, zoom_level, translate, false);
        if cr.in_fill(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        None
    }

    fn draw_segment(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.set_source_rgba(
            self.color.color.red,
            self.color.color.green,
            self.color.color.blue,
            self.color.alpha
        );
        cr.set_line_width(self.width * zoom_level);
        cr.set_line_cap(self.cap);
        cr.set_line_join(self.join);
        cr.set_dash(self.dashes.as_slice(), self.offset);
        cr.translate(translate.x, translate.y);

        let start = self.segment
                        .a()
                        .scale_by(zoom_level);
        let go_dir = &self.go_dir
                          .clone() * zoom_level;
        let arrive_dir = &self.arrive_dir
                              .clone() * zoom_level;
        let end = self.segment
                      .b()
                      .scale_by(zoom_level);
        // start point
        cr.move_to(start.x, start.y);
        if self.curve_like {
            cr.curve_to(
                // go direction
                go_dir.x + start.x,
                go_dir.y + start.y,
                // arrive direction
                arrive_dir.x + end.x,
                arrive_dir.y + end.y,
                // end point
                end.x,
                end.y);
        } else {
            // end point
            cr.line_to(end.x, end.y);
        }

        if draw_it {
            cr.stroke();
        }
    }

    // TODO: Not finished.
    fn draw_head(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.set_source_rgba(
            self.color.color.red,
            self.color.color.green,
            self.color.color.blue,
            self.color.alpha
        );
        cr.translate(translate.x, translate.y);

        let start = self.segment
                        .a()
                        .scale_by(zoom_level);
        let arrive_dir = &self.arrive_dir
                              .clone() * zoom_level;
        let end = self.segment
                      .b()
                      .scale_by(zoom_level);
        let line_width = self.width * zoom_level;

        let mut triangle =
            Cone::new(line_width * 1.25, line_width * 1.25).to_polyline(());
        let mut rotate;
        if self.curve_like {
            rotate = Rotation::rotation_between(
                &arrive_dir,
                &Vector::new(0.0, -1.0),
            );
        } else {
            // Convert @start point to Vector with @end point as it's origin.
            let start_vec =
                Translation::new(-end.x, -end.y).transform_point(&start);
            // calcualte the angle between @start_vec and our triangle.
            rotate = Rotation::rotation_between(
                &Vector::new(start_vec.x, start_vec.y),
                // y = -1.0 becuse cairo/gtk invert the y-axis after drawing it.
                &Vector::new(0.0, -1.0),
            );
        }
        // we make sure that the angle is not negative value.
        let angle = ((360_f64).to_radians() - rotate.angle())
                              .to_degrees()
                              .abs()
                              .to_radians();
        // create a Rotation object.
        rotate = Rotation::new(angle);
        // rotate @triangle
        triangle.rotate_by(&rotate);
        // translate @triangle to end point.
        triangle.translate_by(&Translation::new(end.x, end.y));

        cr.move_to(triangle.coords()[0].x, triangle.coords()[0].y);
        cr.line_to(triangle.coords()[1].x, triangle.coords()[1].y);
        cr.line_to(triangle.coords()[2].x, triangle.coords()[2].y);
        cr.close_path();
        if draw_it {
            cr.fill();
        }
    }

    fn draw_tail(
        &self,
        _cr: &Context,
        _zoom_level: f64,
        _translate: &Vector,
        _draw_it: bool
    ) {
        // TODO
    }

    fn draw_body(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.save();
        self.draw_segment(cr, zoom_level, translate, draw_it);
        cr.restore();
        cr.save();
        self.draw_head(cr, zoom_level, translate, draw_it);
        cr.restore();
        cr.save();
        self.draw_tail(cr, zoom_level, translate, draw_it);
        cr.restore();
    }

    fn draw_start_point(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.translate(translate.x, translate.y);
        cr.set_line_width(self.line_width() * zoom_level);

        let start = self.segment
                        .a()
                        .scale_by(zoom_level);
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();

        // draw start and end circle
        cr.arc(start.x, start.y,
               self.radius() * zoom_level,
               0.0, (360_f64).to_radians());

        if draw_it {
            cr.set_source_rgb(
                fill_color.red,
                fill_color.green,
                fill_color.blue,
            );
            cr.fill_preserve();
            cr.set_source_rgb(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
            );
            cr.stroke();
        }
    }

    fn draw_end_point(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.translate(translate.x, translate.y);
        cr.set_line_width(self.line_width() * zoom_level);

        let end = self.segment
                      .b()
                      .scale_by(zoom_level);
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();

        cr.arc(end.x, end.y,
               self.radius() * zoom_level,
               0.0, (360_f64).to_radians());

        if draw_it {
            cr.set_source_rgb(
                fill_color.red,
                fill_color.green,
                fill_color.blue,
            );
            cr.fill_preserve();
            cr.set_source_rgb(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
            );
            cr.stroke();
        }
    }

    fn draw_go_direction(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.translate(translate.x, translate.y);
        cr.set_line_width(self.line_width() * zoom_level);

        let go_dir = &self.go_dir
                          .clone() * zoom_level;
        let start = self.segment
                        .a()
                        .scale_by(zoom_level);
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();

        // draw @go_dir and @arrive_dir circle
        cr.arc(start.x + go_dir.x, start.y + go_dir.y,
               self.radius() * zoom_level,
               0.0, (360_f64).to_radians());

        if draw_it {
            cr.set_source_rgb(
                fill_color.red,
                fill_color.green,
                fill_color.blue,
            );
            cr.fill_preserve();
            cr.set_source_rgb(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
            );
            cr.stroke();
        }
    }

    fn draw_arrive_direction(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector,
        draw_it: bool
    ) {
        cr.identity_matrix();
        cr.new_path();
        cr.translate(translate.x, translate.y);
        cr.set_line_width(self.line_width() * zoom_level);

        let arrive_dir = &self.arrive_dir
                              .clone() * zoom_level;
        let end = self.segment
                      .b()
                      .scale_by(zoom_level);
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();

        cr.arc(end.x + arrive_dir.x, end.y + arrive_dir.y,
               self.radius() * zoom_level,
               0.0, (360_f64).to_radians());

        if draw_it {
            cr.set_source_rgb(
                fill_color.red,
                fill_color.green,
                fill_color.blue,
            );
            cr.fill_preserve();
            cr.set_source_rgb(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
            );
            cr.stroke();
        }
    }

    fn draw_helper_shapes(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector
    ) {
        cr.save();
        cr.identity_matrix();
        cr.new_path();
        cr.translate(translate.x, translate.y);
        cr.set_line_width(self.line_width() * zoom_level);

        let start = self.segment
                        .a()
                        .scale_by(zoom_level);
        let go_dir = &self.go_dir
                          .clone() * zoom_level;
        let arrive_dir = &self.arrive_dir
                              .clone() * zoom_level;
        let end = self.segment
                      .b()
                      .scale_by(zoom_level);
        let stroke_color = self.color.color.clone();

        if self.curve_like {
            cr.set_source_rgba(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
                0.3,
            );
            cr.set_dash(&[10.0 * zoom_level], 0.0);

            cr.move_to(start.x, start.y);
            cr.rel_line_to(go_dir.x, go_dir.y);
            cr.stroke();

            cr.move_to(end.x, end.y);
            cr.rel_line_to(arrive_dir.x, arrive_dir.y);
            cr.stroke();
        }
        cr.restore();
    }

    fn draw_controllers(
        &self,
        cr: &Context,
        zoom_level: f64,
        translate: &Vector
    ) {
        cr.save();
        self.draw_start_point(cr, zoom_level, translate, true);
        cr.restore();
        cr.save();
        self.draw_end_point(cr, zoom_level, translate, true);
        cr.restore();
        cr.save();
        self.draw_go_direction(cr, zoom_level, translate, true);
        cr.restore();
        cr.save();
        self.draw_arrive_direction(cr, zoom_level, translate, true);
        cr.restore();
    }
}

impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context, zoom_level: f64, translate: &Vector) {
        if !self.visible || self.color.alpha == 0.0 {
            return;
        }

        let draw_extents = match self.draw_extents(zoom_level, translate) {
            None => return,
            Some(val) => val,
        };
        let width = (draw_extents.maxs().x - draw_extents.mins().x).abs();
        let height = (draw_extents.maxs().y - draw_extents.mins().y).abs();
        let surface = Surface::create(
            cairo::Format::ARgb32,
            width as i32,
            height as i32
        );
        let context = Context::new(&surface);

        self.draw_body(&context, zoom_level, translate, true);
        if self.selected {
            self.draw_helper_shapes(&context, zoom_level, translate);
            self.draw_controllers(&context, zoom_level, translate);
        }

        // draw children if there are any.
        for child in self.children.iter() {
            child.draw(&context, zoom_level, translate);
        }

        cr.save();
        cr.set_source_surface(&surface, 0.0, 0.0);
        cr.paint();
        cr.restore();
    }

    fn in_draw(
        &self,
        pos: &Point,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        match self.select_controller(pos, zoom_level, translate) {
            None => return false,
            _ => return true,
        };
    }

    fn draw_extents(
        &self,
        zoom_level: f64,
        translate: &Vector
    ) -> Option<AABB<Point>> {
        // we create zero size image since we don't really need it, but it's
        // required to create cairo::Context
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = &Context::new(&surface);
        let mut extents = vec![];

        cr.save();
        self.draw_start_point(cr, zoom_level, translate, false);
        let extent = cr.stroke_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_end_point(cr, zoom_level, translate, false);
        let extent = cr.stroke_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_go_direction(cr, zoom_level, translate, false);
        let extent = cr.stroke_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_arrive_direction(cr, zoom_level, translate, false);
        let extent = cr.stroke_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_segment(cr, zoom_level, translate, false);
        let extent = cr.stroke_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_head(cr, zoom_level, translate, false);
        let extent = cr.fill_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        cr.save();
        self.draw_tail(cr, zoom_level, translate, false);
        let extent = cr.fill_extents();
        let extent = AABB::new(
            Point::new(extent.0, extent.1),
            Point::new(extent.2, extent.3)
        );
        extents.push(extent);
        cr.restore();

        let mut result = extents[0].clone();
        for val in extents.iter() {
            result.merge(&val);
        }
        Some(result)
    }
}

impl Name for LineArrow {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Move for LineArrow {
    // get the position/center of this line.
    fn position(&self) -> Point {
        na::center(self.segment.a(), self.segment.b())
    }

    fn move_to(&mut self, pos: &Point) {
        let center = self.position();
        let mut trans = Translation::new(-center.x, -center.y);
        let mut a = trans.transform_point(self.segment.a());
        let mut b = trans.transform_point(self.segment.b());
        trans = Translation::new(pos.x, pos.y);
        a = trans.transform_point(&a);
        b = trans.transform_point(&b);
        self.segment = Segment::new(a, b);
    }

    fn translate_by(&mut self, trans: &Translation) {
        let a = trans.transform_point(self.segment.a());
        let b = trans.transform_point(self.segment.b());
        self.segment = Segment::new(a, b);
    }

    // TODO: test origin functionality.
    fn rotate_by(&mut self, rotate: &Rotation, origin: &Vector) {
        let center = self.position() + origin;
        let trans = Translation::new(-center.x, -center.y);
        let mut a = trans.transform_point(self.segment.a());
        let mut b = trans.transform_point(self.segment.b());

        a = rotate.transform_point(&a);
        b = rotate.transform_point(&b);

        a = trans.inverse_transform_point(&a);
        b = trans.inverse_transform_point(&b);

        self.segment = Segment::new(a, b);
    }
}

impl Lock for LineArrow {
    fn is_locked(&self) -> bool {
        self.lock
    }

    fn lock(&mut self) {
        if !self.lock {
            self.lock = true;
        }
    }

    fn unlock(&mut self) {
        if self.lock {
            self.lock = false;
        }
    }

    fn toggle_lock(&mut self) -> bool {
        self.lock = !self.lock;
        self.lock
    }
}

impl Visible for LineArrow {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        if !self.visible {
            self.visible = true;
        }
    }

    fn hide(&mut self) {
        if self.visible {
            self.visible = false;
        }
    }

    fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }
}

impl Container for LineArrow {
    fn add(&mut self, child: Box<ShapeTrait>) {
        self.children.push(child);
    }

    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>> {
        if index > self.children.len() {
            None
        } else {
            Some(self.children.remove(index))
        }
    }
}

// TODO: override default methods
impl Event for LineArrow {
    fn button_press(
        &mut self,
        event: &EventButton,
        zoom_level: f64,
        translate: &Vector
    ) -> bool {
        if self.lock || !self.visible {
            return false;
        }

        if event.get_event_type() == EventType::ButtonPress {
            let position = event.get_position();
            let position = Point::new(position.0, position.1);
            let controller =
                self.select_controller(&position, zoom_level, translate);
            match controller {
                None => {
                    self.selected_controller = None;
                    self.selected = false;
                    return false;
                },
                Some(val) => {
                    self.selected_controller = Some((val, position));
                    self.selected = true;
                    return true;
                },
            }
        }
        false
    }
}
