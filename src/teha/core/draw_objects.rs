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
use gdk::{self, EventMotion, EventButton};

use gettextrs::*;

use ncollide::transformation::ToPolyline;
use ncollide::bounding_volume::BoundingVolume;
use na;
use alga::linear::{Transformation, ProjectiveTransformation,
                   AffineSpace};

use common::Context;
use common::types::*;

pub trait Draw {
    fn draw(&self, cr: &Context);
    fn in_draw(&self, pos: &Point, cr: &Context) -> bool;
    fn draw_extents(&self, cr: &Context) -> Option<Rectangle>;
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
        pos: &Point,
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_release(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
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

        let mut line_arrow = LineArrow::new(
            RgbaColor::new(0.5, 0.5, 1.0, 1.0),
            10.0,
            cairo::LineCap::Round,
            cairo::LineJoin::Round,
            vec![],
            0.0,
            true,
            Segment::new(Point::new(-50.0, -50.0),
                         Point::new(50.0, 50.0)),
        );
        line_arrow.go_dir = Vector::new(10.0, 80.0);
        line_arrow.arrive_dir = Vector::new(-10.0, -80.0);

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
    fn draw(&self, cr: &Context) {
        if !self.visible {
            return;
        }

        for child in self.children.iter() {
            child.draw(cr);
        }
    }

    fn in_draw( &self, pos: &Point, cr: &Context) -> bool {
        if !self.visible {
            return false;
        }

        for child in self.children.iter() {
            if child.in_draw(pos, cr) {
                return true;
            }
        }
        false
    }

    fn draw_extents(&self, cr: &Context) -> Option<Rectangle> {
        if self.children.len() == 0 {
            return None;
        }

        let mut iter =
            self.children
                .iter()
                .filter_map(|s| s.draw_extents(cr));
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
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.motion_notify(event, pos, cr) {
                return true;
            }
        }
        false
    }

    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_press(event, pos, cr) {
                return true;
            }
        }
        false
    }

    fn button_release(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_release(event, pos, cr) {
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
    selected_controller: Option<(LineArrowControllers, Point, Vector, Vector)>,

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
    pub fn new(
        color: RgbaColor, width: f64, cap: cairo::LineCap,
        join: cairo::LineJoin, dashes: Vec<f64>, offset: f64,
        curve_like: bool, segment: Segment
    ) -> Self {
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
        &self, pos: &Point, cr: &Context
    ) -> Option<LineArrowControllers> {
        cr.new_path();
        cr.save();
        self.draw_start_point(cr, false);
        if cr.in_stroke(pos) || cr.in_fill(pos) {
            return Some(LineArrowControllers::StartPoint);
        }
        cr.restore();

        cr.save();
        self.draw_end_point(cr, false);
        if cr.in_stroke(pos) || cr.in_fill(pos) {
            return Some(LineArrowControllers::EndPoint);
        }
        cr.restore();

        cr.save();
        self.draw_go_direction(cr, false);
        if cr.in_stroke(pos) || cr.in_fill(pos) {
            return Some(LineArrowControllers::GoDirection);
        }
        cr.restore();

        cr.save();
        self.draw_arrive_direction(cr, false);
        if cr.in_stroke(pos) || cr.in_fill(pos) {
            return Some(LineArrowControllers::ArriveDirection);
        }
        cr.restore();

        cr.save();
        self.draw_segment(cr, false);
        if cr.in_stroke(pos) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        cr.save();
        self.draw_head(cr, false);
        if cr.in_fill(pos) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        // cr.save();
        // self.draw_tail(cr, false);
        // if cr.in_fill(pos) {
        //     return Some(LineArrowControllers::Body);
        //     println!("draw_tail");
        // }
        // cr.restore();

        None
    }

    fn move_segment(&mut self, pos: &Point) {
        let &(.., vec_a, vec_b) = match self.selected_controller {
            None => return,
            Some(ref val) => val,
        };
        let a = pos.clone() - vec_a.clone();
        let b = pos.clone() - vec_b.clone();
        self.segment = Segment::new(a, b);
    }

    fn move_go_dir(&mut self, pos: &Point) {
        self.go_dir = Vector::new(
            pos.x - self.segment.a().x,
            pos.y - self.segment.a().y
        );
    }

    fn move_arrive_dir(&mut self, pos: &Point) {
        self.arrive_dir = Vector::new(
            pos.x - self.segment.b().x,
            pos.y - self.segment.b().y
        );
    }

    fn move_start_point(&mut self, pos: &Point) {
        let b = self.segment.b().clone();
        let a = pos.clone();
        self.segment = Segment::new(a, b);
    }

    fn move_end_point(&mut self, pos: &Point) {
        let a = self.segment.a().clone();
        let b = pos.clone();
        self.segment = Segment::new(a, b);
    }

    fn draw_segment(&self, cr: &Context, draw_it: bool) {
        cr.new_path();
        cr.set_source_rgba(&self.color);
        cr.set_line_width(self.width);
        cr.set_line_cap(self.cap);
        cr.set_line_join(self.join);
        cr.set_dash(self.dashes.as_slice(), self.offset);

        if self.curve_like {
            cr.curve(&self.segment, &self.go_dir, &self.arrive_dir);
        } else {
            cr.line(&self.segment);
        }

        if draw_it {
            cr.stroke();
        }
    }

    fn draw_head( &self, cr: &Context, draw_it: bool ) {
        cr.new_path();
        cr.set_source_rgba(&self.color);

        let start = self.segment.a();
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let radius = self.width * 1.25;

        let mut triangle = Cone::new(radius, radius).to_polyline(());

        // get the rotation between out triangle and other vectors
        let mut rotate = if self.curve_like {
            Rotation::rotation_between(arrive_dir, &Vector::new(0.0, -1.0))
        } else {
            // Convert @start point to Vector with @end point as it's origin.
            let start_vec =
                Translation::new(-end.x, -end.y).transform_point(start);
            // calcualte the angle between @start_vec and our triangle.
            Rotation::rotation_between(
                &Vector::new(start_vec.x, start_vec.y),
                &Vector::new(0.0, -1.0)
            )
        };

        // we make sure that the angle is not negative value.
        let angle = ((360_f64).to_radians() - rotate.angle())
                              .to_degrees()
                              .abs()
                              .to_radians();
        rotate = Rotation::new(angle);
        // rotate @triangle
        triangle.rotate_by(&rotate);
        // translate @triangle to end point.
        triangle.translate_by(&Translation::new(end.x, end.y));

        cr.polyline(&triangle);
        cr.close_path();
        if draw_it {
            cr.fill();
        }
    }

    fn draw_tail(&self, _cr: &Context, _draw_it: bool) {
        // TODO
    }

    fn draw_body(&self, cr: &Context, draw_it: bool) {
        cr.save();
        self.draw_segment(cr, draw_it);
        cr.restore();
        cr.save();
        self.draw_head(cr, draw_it);
        cr.restore();
        cr.save();
        self.draw_tail(cr, draw_it);
        cr.restore();
    }

    fn draw_start_point(&self, cr: &Context, draw_it: bool) {
        cr.new_path();

        cr.set_line_width(self.line_width());
        cr.circle(self.segment.a(), self.radius());

        if draw_it {
            cr.set_source_rgb(&self.fill_color());
            cr.fill_preserve();
            cr.set_source_rgb(&self.stroke_color());
            cr.stroke();
        }
    }

    fn draw_end_point(&self, cr: &Context, draw_it: bool) {
        cr.new_path();

        cr.set_line_width(self.line_width());
        cr.circle(self.segment.b(), self.radius());

        if draw_it {
            cr.set_source_rgb(&self.fill_color());
            cr.fill_preserve();
            cr.set_source_rgb(&self.stroke_color());
            cr.stroke();
        }
    }

    fn draw_go_direction(&self, cr: &Context, draw_it: bool) {
        cr.new_path();

        cr.set_line_width(self.line_width());
        let pos = self.segment.a() + self.go_dir.clone();
        cr.circle(&pos, self.radius());

        if draw_it {
            cr.set_source_rgb(&self.fill_color());
            cr.fill_preserve();
            cr.set_source_rgb(&self.stroke_color());
            cr.stroke();
        }
    }

    fn draw_arrive_direction(&self, cr: &Context, draw_it: bool) {
        cr.new_path();

        cr.set_line_width(self.line_width());
        let pos = self.segment.b() + self.arrive_dir.clone();
        cr.circle(&pos, self.radius());

        if draw_it {
            cr.set_source_rgb(&self.fill_color());
            cr.fill_preserve();
            cr.set_source_rgb(&self.stroke_color());
            cr.stroke();
        }
    }

    fn draw_helper_shapes(&self, cr: &Context) {
        cr.save();
        cr.new_path();

        if self.curve_like {
            cr.set_line_width(self.line_width());
            cr.set_source_rgba(&RgbaColor::new(
                self.color.color.red,
                self.color.color.green,
                self.color.color.blue,
                0.3
            ));
            cr.set_dash(&[10.0], 0.0);

            cr.move_to(self.segment.a());
            cr.rel_line_to(&self.go_dir);
            cr.stroke();

            cr.move_to(self.segment.b());
            cr.rel_line_to(&self.arrive_dir);
            cr.stroke();
        }
        cr.restore();
    }

    fn draw_controllers(&self, cr: &Context) {
        cr.save();
        self.draw_start_point(cr, true);
        cr.restore();
        cr.save();
        self.draw_end_point(cr, true);
        cr.restore();
        cr.save();
        self.draw_go_direction(cr, true);
        cr.restore();
        cr.save();
        self.draw_arrive_direction(cr, true);
        cr.restore();
    }
}

impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context) {
        if !self.visible || self.color.alpha == 0.0 {
            return;
        }

        cr.save();

        self.draw_body(&cr, true);
        if self.selected {
            self.draw_helper_shapes(&cr);
            self.draw_controllers(&cr);
        }

        // draw children if there are any.
        for child in self.children.iter() {
            child.draw(&cr);
        }

        cr.restore();
    }

    fn in_draw(&self, pos: &Point, cr: &Context) -> bool {
        match self.select_controller(pos, cr) {
            None => return false,
            _ => return true,
        };
    }

    fn draw_extents(&self, cr: &Context) -> Option<Rectangle> {
        cr.new_path();
        let mut extents = vec![];

        if self.curve_like {
            cr.save();
            self.draw_go_direction(cr, false);
            extents.push(cr.stroke_extents());
            cr.restore();

            cr.save();
            self.draw_arrive_direction(cr, false);
            extents.push(cr.stroke_extents());
            cr.restore();
        }

        cr.save();
        self.draw_start_point(cr, false);
        extents.push(cr.stroke_extents());
        cr.restore();

        cr.save();
        self.draw_end_point(cr, false);
        extents.push(cr.stroke_extents());
        cr.restore();

        cr.save();
        self.draw_segment(cr, false);
        extents.push(cr.stroke_extents());
        cr.restore();

        cr.save();
        self.draw_head(cr, false);
        extents.push(cr.fill_extents());
        cr.restore();

        // cr.save();
        // self.draw_tail(cr, false);
        // extents.push(cr.fill_extents_rect());
        // cr.restore();

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
    fn motion_notify(
        &mut self,
        event: &EventMotion,
        pos: &Point,
        _cr: &Context
    ) -> bool {
        if self.lock || !self.visible {
            return false;
        }

        if event.get_state() == gdk::BUTTON1_MASK {
            match self.selected_controller {
                None => return false,
                Some((LineArrowControllers::GoDirection, ..)) => {
                    self.move_go_dir(pos);
                },
                Some((LineArrowControllers::ArriveDirection, ..)) => {
                    self.move_arrive_dir(pos);
                },
                Some((LineArrowControllers::StartPoint, ..)) => {
                    self.move_start_point(pos);
                },
                Some((LineArrowControllers::EndPoint, ..)) => {
                    self.move_end_point(pos);
                },
                Some((LineArrowControllers::Body, ..)) => {
                    self.move_segment(pos);
                },
            };
        }
        false
    }

    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool {
        if self.lock || !self.visible {
            return false;
        }

        if event.get_button() == 1 {
            match self.select_controller(&pos, cr) {
                None => {
                    self.selected_controller = None;
                    self.selected = false;
                    return false;
                },
                Some(val) => {
                    let vec_a = pos.clone() - self.segment.a().clone();
                    let vec_b = pos.clone() - self.segment.b().clone();
                    if !self.selected && (val == LineArrowControllers::Body ||
                        val == LineArrowControllers::StartPoint ||
                        val == LineArrowControllers::EndPoint) {
                        self.selected_controller =
                        Some((
                            LineArrowControllers::Body,
                            pos.clone(),
                            vec_a, vec_b
                        ));
                        self.selected = true;
                        return true;
                    } else if self.selected {
                        self.selected_controller =
                        Some((
                            val,
                            pos.clone(),
                            vec_a, vec_b
                        ));
                        self.selected = true;
                        return true;
                    }
                },
            }
        }
        false
    }
}
