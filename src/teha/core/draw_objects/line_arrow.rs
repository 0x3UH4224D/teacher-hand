//
// line_arrow.rs
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

use std::rc::Rc;
use std::cell::RefCell;

use cairo;
use gdk::{self, EventMotion, EventButton};
use gtk::prelude::*;
use gtk::{self, SwitchExt, ContainerExt, WidgetExt, GridExt, NotebookExtManual,
          EntryExt, ColorButtonExt, ColorChooserExt, Cast, SpinButtonExt,
          BoxExt, MenuButtonExt};

use gettextrs::*;

use ncollide::transformation::ToPolyline;
use ncollide::bounding_volume::BoundingVolume;
use na;
use alga::linear::{Transformation, ProjectiveTransformation, Similarity};

use core::context::Context;
use common::types::*;
use super::*;

#[derive(Clone, PartialEq, Eq)]
pub enum Mode {
    Editing,
    CreatingLineArrow,
    CreatingLine,
    CreatingCurveArrow,
    CreatingCurve,
}

#[derive(Clone, PartialEq)]
pub enum Actions {
    MoveBody(Vector, Vector),
    MoveStartPoint,
    MoveEndPoint,
    MoveGoDirection,
    MoveArriveDirection,
}

pub struct LineArrow {
    children: Vec<Box<ShapeTrait>>,
    // ID field
    name: Rc<RefCell<String>>,
    // control fields
    lock: Rc<RefCell<bool>>,
    selected: bool,
    // this field for Event trait.
    action: Option<Actions>,
    mode: Mode,
    // draw fields
    visible: Rc<RefCell<bool>>,
    color: Rc<RefCell<RgbaColor>>,
    width: Rc<RefCell<f64>>,
    cap: Rc<RefCell<cairo::LineCap>>,
    join: Rc<RefCell<cairo::LineJoin>>,
    dashes: Rc<RefCell<Vec<f64>>>,
    offset: Rc<RefCell<f64>>,
    // Segment field
    segment: Segment,
    // Curve fields
    curve_like: Rc<RefCell<bool>>,
    // head & tail fields
    have_head: Rc<RefCell<bool>>,
    // these vector needed if we want to convert this line to curve
    go_dir: Vector, // dir refer to direction
    arrive_dir: Vector,
}

impl LineArrow {
    pub fn new(segment: Segment) -> Self {
        LineArrow {
            children: vec![],
            name: Rc::new(RefCell::new(String::new())),
            lock: Rc::new(RefCell::new(false)),
            selected: false,
            action: None,
            mode: Mode::CreatingLineArrow,
            visible: Rc::new(RefCell::new(true)),
            color: Rc::new(RefCell::new(RgbaColor::new(0.0, 0.0, 0.0, 1.0))),
            width: Rc::new(RefCell::new(10.0)),
            cap: Rc::new(RefCell::new(cairo::LineCap::Round)),
            join: Rc::new(RefCell::new(cairo::LineJoin::Round)),
            dashes: Rc::new(RefCell::new(vec![])),
            offset: Rc::new(RefCell::new(0.0)),
            segment: segment,
            curve_like: Rc::new(RefCell::new(false)),
            have_head: Rc::new(RefCell::new(true)),
            go_dir: Vector::new(0.0, 0.0),
            arrive_dir: Vector::new(0.0, 0.0),
        }
    }

    // This crazy method will be replaced with Template UI file when GTK-rs
    // support that
    pub fn connect_ui(&mut self, options_widget: &gtk::Notebook) {
        let create_label = |text: &str| {
            let label = gtk::Label::new(text);
            label.set_halign(gtk::Align::End);
            label
        };

        let grid = gtk::Grid::new();
        grid.set_property_margin(10);
        grid.set_row_spacing(6);
        grid.set_column_spacing(10);

        let name_label = create_label(
            gettext("Name:").as_str()
        );
        let name_entry = gtk::Entry::new();
        name_entry.set_hexpand(true);
        name_entry.set_text(self.name().as_str());
        let name = self.name.clone();
        name_entry.connect_property_text_notify(move |me| {
            if let Some(text) = me.get_text() {
                name.borrow_mut().clone_from(&text);
            }
        });
        grid.attach(&name_label, 0, 0, 1, 1);
        grid.attach(&name_entry, 1, 0, 1, 1);

        let color_label = create_label(
            gettext("Color:").as_str()
        );
        let color_init_value = self.get_color();
        let color_init_value = gdk::RGBA {
            red: color_init_value.color.red,
            green: color_init_value.color.green,
            blue: color_init_value.color.blue,
            alpha: color_init_value.alpha
        };
        let color_button = gtk::ColorButton::new_with_rgba(&color_init_value);
        color_button.set_title(
            gettext("Body Color").as_str()
        );
        color_button.set_use_alpha(true);
        let color = self.color.clone();
        let color_chooser = color_button
            .clone()
            .dynamic_cast::<gtk::ColorChooser>()
            .expect("GtkButtonChooser does implmente GtkColorChooser \
                     interface");
        color_chooser.connect_property_rgba_notify(move |me| {
            let new_color = me.get_rgba();
            let new_color = RgbaColor::new(
                new_color.red, new_color.green, new_color.blue, new_color.alpha
            );
            color.borrow_mut().clone_from(&new_color);
        });
        grid.attach(&color_label, 0, 1, 1, 1);
        grid.attach(&color_button, 1, 1, 1, 1);

        let width_label = create_label(
            gettext("Width:").as_str()
        );
        let width_spin = gtk::SpinButton::new_with_range(0.10, 100.0, 1.0);
        width_spin.set_digits(2);
        width_spin.set_value(self.get_width());
        let width = self.width.clone();
        width_spin.connect_property_value_notify(move |me| {
            width.borrow_mut().clone_from(&me.get_value());
        });
        grid.attach(&width_label, 0, 2, 1, 1);
        grid.attach(&width_spin, 1, 2, 1, 1);

        let cap_label = create_label(
            gettext("Cap:").as_str()
        );

        let cap_styles_butt_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        cap_styles_butt_icon.set_pixel_size(16);
        cap_styles_butt_icon.set_no_show_all(true);
        let cap_styles_butt_label = gtk::Label::new(gettext("Butt").as_str());
        let cap_styles_butt_grid = gtk::Grid::new();
        cap_styles_butt_grid.attach(&cap_styles_butt_label, 0, 0, 1, 1);
        cap_styles_butt_grid.attach(&cap_styles_butt_icon, 1, 0, 1, 1);

        let cap_styles_round_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        cap_styles_round_icon.set_pixel_size(16);
        cap_styles_round_icon.set_no_show_all(true);
        let cap_styles_round_label = gtk::Label::new(gettext("Round").as_str());
        let cap_styles_round_grid = gtk::Grid::new();
        cap_styles_round_grid.attach(&cap_styles_round_label, 0, 0, 1, 1);
        cap_styles_round_grid.attach(&cap_styles_round_icon, 1, 0, 1, 1);

        let cap_styles_square_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        cap_styles_square_icon.set_pixel_size(16);
        cap_styles_square_icon.set_no_show_all(true);
        let cap_styles_square_label =
            gtk::Label::new(gettext("Square").as_str());
        let cap_styles_square_grid = gtk::Grid::new();
        cap_styles_square_grid.attach(&cap_styles_square_label, 0, 0, 1, 1);
        cap_styles_square_grid.attach(&cap_styles_square_icon, 1, 0, 1, 1);

        let cap_styles_listbox = gtk::ListBox::new();
        cap_styles_listbox.set_property_margin(8);
        cap_styles_listbox.add(&cap_styles_butt_grid);
        cap_styles_listbox.add(&cap_styles_round_grid);
        cap_styles_listbox.add(&cap_styles_square_grid);
        cap_styles_listbox.show_all();

        let cap_popover = gtk::Popover::new(&cap_styles_listbox);
        cap_popover.add(&cap_styles_listbox);
        let cap_menu_button = gtk::MenuButton::new();
        let cap_menu_button_icon =
            gtk::Image::new_from_icon_name("pan-down-symbolic", 0);
        cap_menu_button_icon.set_pixel_size(16);
        let cap_menu_button_label = gtk::Label::new("");
        let cap_menu_button_box =
            gtk::Box::new(gtk::Orientation::Horizontal, 0);
        cap_menu_button_box.pack_start(&cap_menu_button_label, false, false, 0);
        cap_menu_button_box.pack_end(&cap_menu_button_icon, false, false, 0);
        cap_menu_button.add(&cap_menu_button_box);
        cap_menu_button.set_popover(&cap_popover);

        match self.get_cap() {
            cairo::LineCap::Butt => {
                cap_styles_butt_icon.show();
                if let Some(label) = cap_styles_butt_label.get_label() {
                    cap_menu_button_label.set_label(label.as_str());
                }
                let row = cap_styles_listbox.get_row_at_index(0)
                    .expect("cap_styles_listBox have 3 rows");
                cap_styles_listbox.select_row(&row);
            },
            cairo::LineCap::Round => {
                cap_styles_round_icon.show();
                if let Some(label) = cap_styles_round_label.get_label() {
                    cap_menu_button_label.set_label(label.as_str());
                }
                let row = cap_styles_listbox.get_row_at_index(1)
                    .expect("cap_styles_listBox have 3 rows");
                cap_styles_listbox.select_row(&row);
            },
            cairo::LineCap::Square => {
                cap_styles_square_icon.show();
                if let Some(label) = cap_styles_square_label.get_label() {
                    cap_menu_button_label.set_label(label.as_str());
                }
                let row = cap_styles_listbox.get_row_at_index(2)
                    .expect("cap_styles_listBox have 3 rows");
                cap_styles_listbox.select_row(&row);
            },
        };

        let inner_cap_menu_button_label = cap_menu_button_label.clone();
        let inner_cap_styles_butt_icon = cap_styles_butt_icon.clone();
        let inner_cap_styles_butt_label = cap_styles_butt_label.clone();
        let inner_cap_styles_round_icon = cap_styles_round_icon.clone();
        let inner_cap_styles_round_label = cap_styles_round_label.clone();
        let inner_cap_styles_square_icon = cap_styles_square_icon.clone();
        let inner_cap_styles_square_label = cap_styles_square_label.clone();
        let cap = self.cap.clone();
        cap_styles_listbox.connect_row_activated(move |_me, row| {
            inner_cap_styles_butt_icon.hide();
            inner_cap_styles_round_icon.hide();
            inner_cap_styles_square_icon.hide();
            let index = row.get_index();
            match index {
                0 => {
                    inner_cap_styles_butt_icon.show();
                    cap.borrow_mut().clone_from(&cairo::LineCap::Butt);
                    if let Some(label) =
                           inner_cap_styles_butt_label.get_label() {
                        inner_cap_menu_button_label.set_label(label.as_str());
                    }
                },
                1 => {
                    inner_cap_styles_round_icon.show();
                    cap.borrow_mut().clone_from(&cairo::LineCap::Round);
                    if let Some(label) =
                           inner_cap_styles_round_label.get_label() {
                        inner_cap_menu_button_label.set_label(label.as_str());
                    }
                },
                2 => {
                    inner_cap_styles_square_icon.show();
                    cap.borrow_mut().clone_from(&cairo::LineCap::Square);
                    if let Some(label) =
                           inner_cap_styles_square_label.get_label() {
                        inner_cap_menu_button_label.set_label(label.as_str());
                    }
                },
                _ => unreachable!(),
            };

        });

        grid.attach(&cap_label, 0, 3, 1, 1);
        grid.attach(&cap_menu_button, 1, 3, 1, 1);

        let join_label = create_label(
            gettext("Join:").as_str()
        );

        let join_styles_bevel_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        join_styles_bevel_icon.set_pixel_size(16);
        join_styles_bevel_icon.set_no_show_all(true);
        let join_styles_bevel_label =
            gtk::Label::new(gettext("Bevel").as_str());
        let join_styles_bevel_grid = gtk::Grid::new();
        join_styles_bevel_grid.attach(&join_styles_bevel_label, 0, 0, 1, 1);
        join_styles_bevel_grid.attach(&join_styles_bevel_icon, 1, 0, 1, 1);

        let join_styles_miter_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        join_styles_miter_icon.set_pixel_size(16);
        join_styles_miter_icon.set_no_show_all(true);
        let join_styles_miter_label =
            gtk::Label::new(gettext("Miter").as_str());
        let join_styles_miter_grid = gtk::Grid::new();
        join_styles_miter_grid.attach(&join_styles_miter_label, 0, 0, 1, 1);
        join_styles_miter_grid.attach(&join_styles_miter_icon, 1, 0, 1, 1);

        let join_styles_round_icon =
            gtk::Image::new_from_icon_name("object-select-symbolic", 0);
        join_styles_round_icon.set_pixel_size(16);
        join_styles_round_icon.set_no_show_all(true);
        let join_styles_round_label =
            gtk::Label::new(gettext("Round").as_str());
        let join_styles_round_grid = gtk::Grid::new();
        join_styles_round_grid.attach(&join_styles_round_label, 0, 0, 1, 1);
        join_styles_round_grid.attach(&join_styles_round_icon, 1, 0, 1, 1);

        let join_styles_listbox = gtk::ListBox::new();
        join_styles_listbox.set_property_margin(8);
        join_styles_listbox.add(&join_styles_bevel_grid);
        join_styles_listbox.add(&join_styles_miter_grid);
        join_styles_listbox.add(&join_styles_round_grid);
        join_styles_listbox.show_all();

        let join_popover = gtk::Popover::new(&join_styles_listbox);
        join_popover.add(&join_styles_listbox);
        let join_menu_button = gtk::MenuButton::new();
        let join_menu_button_icon =
            gtk::Image::new_from_icon_name("pan-down-symbolic", 0);
        join_menu_button_icon.set_pixel_size(16);
        let join_menu_button_label = gtk::Label::new("");
        let join_menu_button_box =
            gtk::Box::new(gtk::Orientation::Horizontal, 0);
        join_menu_button_box.pack_start(
            &join_menu_button_label, false, false, 0
        );
        join_menu_button_box.pack_end(&join_menu_button_icon, false, false, 0);
        join_menu_button.add(&join_menu_button_box);
        join_menu_button.set_popover(&join_popover);

        match self.get_join() {
            cairo::LineJoin::Bevel => {
                join_styles_bevel_icon.show();
                if let Some(label) = join_styles_bevel_label.get_label() {
                    join_menu_button_label.set_label(label.as_str());
                }
                let row = join_styles_listbox.get_row_at_index(0)
                    .expect("join_styles_listBox have 3 rows");
                join_styles_listbox.select_row(&row);
            },
            cairo::LineJoin::Miter => {
                join_styles_miter_icon.show();
                if let Some(label) = join_styles_miter_label.get_label() {
                    join_menu_button_label.set_label(label.as_str());
                }
                let row = join_styles_listbox.get_row_at_index(1)
                    .expect("join_styles_listBox have 3 rows");
                join_styles_listbox.select_row(&row);
            },
            cairo::LineJoin::Round => {
                join_styles_round_icon.show();
                if let Some(label) = join_styles_round_label.get_label() {
                    join_menu_button_label.set_label(label.as_str());
                }
                let row = join_styles_listbox.get_row_at_index(2)
                    .expect("join_styles_listBox have 3 rows");
                join_styles_listbox.select_row(&row);
            },
        };

        let inner_join_menu_button_label = join_menu_button_label.clone();
        let inner_join_styles_bevel_icon = join_styles_bevel_icon.clone();
        let inner_join_styles_bevel_label = join_styles_bevel_label.clone();
        let inner_join_styles_miter_icon = join_styles_miter_icon.clone();
        let inner_join_styles_miter_label = join_styles_miter_label.clone();
        let inner_join_styles_round_icon = join_styles_round_icon.clone();
        let inner_join_styles_round_label = join_styles_round_label.clone();
        let join = self.join.clone();
        join_styles_listbox.connect_row_activated(move |_me, row| {
            inner_join_styles_bevel_icon.hide();
            inner_join_styles_miter_icon.hide();
            inner_join_styles_round_icon.hide();
            let index = row.get_index();
            match index {
                0 => {
                    inner_join_styles_bevel_icon.show();
                    join.borrow_mut().clone_from(&cairo::LineJoin::Bevel);
                    if let Some(label) =
                           inner_join_styles_bevel_label.get_label() {
                        inner_join_menu_button_label.set_label(label.as_str());
                    }
                },
                1 => {
                    inner_join_styles_miter_icon.show();
                    join.borrow_mut().clone_from(&cairo::LineJoin::Miter);
                    if let Some(label) =
                           inner_join_styles_miter_label.get_label() {
                        inner_join_menu_button_label.set_label(label.as_str());
                    }
                },
                2 => {
                    inner_join_styles_round_icon.show();
                    join.borrow_mut().clone_from(&cairo::LineJoin::Round);
                    if let Some(label) =
                           inner_join_styles_round_label.get_label() {
                        inner_join_menu_button_label.set_label(label.as_str());
                    }
                },
                _ => unreachable!(),
            };

        });

        grid.attach(&join_label, 0, 4, 1, 1);
        grid.attach(&join_menu_button, 1, 4, 1, 1);

        let dashes_label = create_label(
            gettext("Dashes:").as_str()
        );

        let dashes_on_spin = gtk::SpinButton::new_with_range(0.0, 100.0, 1.0);
        dashes_on_spin.set_tooltip_text(
            gettext("This value of how tall the dashes will apper").as_str()
        );

        let dashes_off_spin = gtk::SpinButton::new_with_range(0.0, 100.0, 1.0);
        dashes_off_spin.set_tooltip_text(
            gettext("This value of how tall the dashes will disappear").as_str()
        );
        dashes_off_spin.set_digits(2);

        dashes_on_spin.set_digits(2);
        if let Some(values) = self.get_dashes().get(0..2) {
            dashes_on_spin.set_value(values[0]);
            dashes_off_spin.set_value(values[1]);
        } else {
            dashes_on_spin.set_value(0.0);
            dashes_off_spin.set_value(0.0);
        }

        let dashes = self.dashes.clone();
        let inner_dashes_on_spin = dashes_on_spin.clone();
        let inner_dashes_off_spin = dashes_off_spin.clone();
        dashes_on_spin.connect_property_value_notify(move |_me| {
            let on_value = inner_dashes_on_spin.get_value();
            let off_value = inner_dashes_off_spin.get_value();
            if on_value == 0.0 && off_value == 0.0 {
                dashes.borrow_mut().clone_from(&vec![]);
                return;
            }
            let new_dashes = vec![on_value, off_value];
            dashes.borrow_mut().clone_from(&new_dashes);
        });

        let dashes = self.dashes.clone();
        let inner_dashes_on_spin = dashes_on_spin.clone();
        let inner_dashes_off_spin = dashes_off_spin.clone();
        dashes_off_spin.connect_property_value_notify(move |_me| {
            let on_value = inner_dashes_on_spin.get_value();
            let off_value = inner_dashes_off_spin.get_value();
            if on_value == 0.0 && off_value == 0.0 {
                dashes.borrow_mut().clone_from(&vec![]);
                return;
            }
            let new_dashes = vec![on_value, off_value];
            dashes.borrow_mut().clone_from(&new_dashes);
        });

        grid.attach(&dashes_label, 0, 5, 1, 2);
        grid.attach(&dashes_on_spin, 1, 5, 1, 1);
        grid.attach(&dashes_off_spin, 1, 6, 1, 1);

        let curve_like_label = create_label(gettext("Curve Like:").as_str());
        let curve_like_switch = gtk::Switch::new();
        curve_like_switch.set_halign(gtk::Align::Start);
        curve_like_switch.set_active(self.get_curve_like());
        let curve_like = self.curve_like.clone();
        curve_like_switch.connect_property_active_notify(move |me| {
            curve_like.borrow_mut().clone_from(&me.get_active());
        });
        grid.attach(&curve_like_label, 0, 7, 1, 1);
        grid.attach(&curve_like_switch, 1, 7, 1, 1);

        let have_head_label = create_label(gettext("Have Head").as_str());
        let have_head_switch = gtk::Switch::new();
        have_head_switch.set_halign(gtk::Align::Start);
        have_head_switch.set_active(self.get_have_head());
        let have_head = self.have_head.clone();
        have_head_switch.connect_property_active_notify(move |me| {
            have_head.borrow_mut().clone_from(&me.get_active());
        });
        grid.attach(&have_head_label, 0, 8, 1, 1);
        grid.attach(&have_head_switch, 1, 8, 1, 1);

        let visible_label = create_label(gettext("Visible:").as_str());
        let visible_switch = gtk::Switch::new();
        visible_switch.set_halign(gtk::Align::Start);
        visible_switch.set_active(self.is_visible());
        let visible = self.visible.clone();
        visible_switch.connect_property_active_notify(move |me| {
            visible.borrow_mut().clone_from(&me.get_active());
        });
        grid.attach(&visible_label, 0, 9, 1, 1);
        grid.attach(&visible_switch, 1, 9, 1, 1);

        let lock_label = create_label(gettext("Lock:").as_str());
        let lock_switch = gtk::Switch::new();
        lock_switch.set_halign(gtk::Align::Start);
        lock_switch.set_active(self.is_locked());
        let lock = self.lock.clone();
        lock_switch.connect_property_active_notify(move |me| {
            lock.borrow_mut().clone_from(&me.get_active());
        });
        grid.attach(&lock_label, 0, 10, 1, 1);
        grid.attach(&lock_switch, 1, 10, 1, 1);

        grid.show_all();
        let tab_label = gtk::Label::new(
            gettext("Options").as_str()
        );
        options_widget.append_page(&grid, Some(&tab_label));
    }

    fn radius(&self) -> f64 {
        if self.get_width() < 10.0  {
            3.0
        } else if self.get_width() > 20.0 {
            9.0
        } else {
            // radius will be in range 4.0-8.0
            self.get_width() * 0.40
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

    pub fn get_mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Editing => {
                self.set_action(None);
                self.unselect();
            },
            Mode::CreatingCurveArrow => {
                self.set_action(Some(Actions::MoveStartPoint));
                self.select();
                self.set_curve_like(true);
                self.set_have_head(true);
                self.hide();
            },
            Mode::CreatingCurve => {
                self.set_action(Some(Actions::MoveStartPoint));
                self.select();
                self.set_curve_like(true);
                self.set_have_head(false);
                self.hide();
            },
            Mode::CreatingLineArrow => {
                self.set_action(Some(Actions::MoveStartPoint));
                self.select();
                self.set_curve_like(false);
                self.set_have_head(true);
                self.hide();
            },
            Mode::CreatingLine => {
                self.set_action(Some(Actions::MoveStartPoint));
                self.select();
                self.set_curve_like(false);
                self.set_have_head(false);
                self.hide();
            },
        };
        self.mode = mode;
    }

    pub fn get_width(&self) -> f64 {
        *self.width.borrow()
    }

    pub fn set_width(&mut self, width: f64) {
        self.width.borrow_mut().clone_from(&width);
    }

    pub fn get_cap(&self) -> cairo::LineCap {
        *self.cap.borrow()
    }

    pub fn set_cap(&mut self, cap: &cairo::LineCap) {
        self.cap.borrow_mut().clone_from(cap);
    }

    pub fn get_join(&self) -> cairo::LineJoin {
        *self.join.borrow()
    }

    pub fn set_join(&mut self, join: &cairo::LineJoin) {
        self.join.borrow_mut().clone_from(join);
    }

    pub fn get_dashes(&self) -> Vec<f64> {
        self.dashes.borrow().clone()
    }

    pub fn set_dashes(&mut self, dashes: &Vec<f64>) {
        self.dashes.borrow_mut().clone_from(dashes);
    }

    pub fn get_offset(&self) -> f64 {
        *self.offset.borrow()
    }

    pub fn set_offset(&mut self, offset: &f64) {
        self.offset.borrow_mut().clone_from(&offset);
    }

    pub fn get_curve_like(&self) -> bool {
        *self.curve_like.borrow()
    }

    pub fn set_curve_like(&mut self, curve_like: bool) {
        self.curve_like.borrow_mut().clone_from(&curve_like);
    }

    pub fn get_have_head(&self) -> bool {
        *self.have_head.borrow()
    }

    pub fn set_have_head(&mut self, have_head: bool) {
        self.have_head.borrow_mut().clone_from(&have_head);
    }

    pub fn set_action(&mut self, action: Option<Actions>) {
        self.action = action;
    }

    pub fn get_action(&self) -> Option<Actions> {
        self.action.clone()
    }

    fn select_controller(
        &self, pos: &Point, cr: &Context
    ) -> Option<Actions> {
        cr.new_path();

        let vec_a = pos.clone() - self.segment.a().clone();
        let vec_b = pos.clone() - self.segment.b().clone();

        if self.is_selected() {
            if self.get_curve_like() {
                cr.save();
                self.draw_go_direction(cr, false);
                if cr.in_stroke(pos) || cr.in_fill(pos) {
                    return Some(Actions::MoveGoDirection);
                }
                cr.restore();

                cr.save();
                self.draw_arrive_direction(cr, false);
                if cr.in_stroke(pos) || cr.in_fill(pos) {
                    return Some(Actions::MoveArriveDirection);
                }
                cr.restore();
            }

            cr.save();
            self.draw_start_point(cr, false);
            if cr.in_stroke(pos) || cr.in_fill(pos) {
                if self.is_selected() {
                    return Some(Actions::MoveStartPoint);
                } else {
                    return Some(Actions::MoveBody(vec_a, vec_b));
                }
            }
            cr.restore();

            cr.save();
            self.draw_end_point(cr, false);
            if cr.in_stroke(pos) || cr.in_fill(pos) {
                if self.is_selected() {
                    return Some(Actions::MoveEndPoint);
                } else {
                    return Some(Actions::MoveBody(vec_a, vec_b));
                }
            }
            cr.restore();
        }

        cr.save();
        self.draw_segment(cr, false);
        if cr.in_stroke(pos) {
            return Some(Actions::MoveBody(vec_a, vec_b));
        }
        cr.restore();

        if self.get_have_head() {
            cr.save();
            self.draw_head(cr, false);
            if cr.in_fill(pos) {
                return Some(Actions::MoveBody(vec_a, vec_b));
            }
            cr.restore();
        }

        // cr.save();
        // self.draw_tail(cr, false);
        // if cr.in_fill(pos) {
        //     return Some(Actions::MoveBody);
        //     println!("draw_tail");
        // }
        // cr.restore();

        None
    }

    fn move_segment(&mut self, pos: &Point) {
        if let Some(Actions::MoveBody(vec_a, vec_b)) = self.action {
            let a = pos.clone() - vec_a.clone();
            let b = pos.clone() - vec_b.clone();
            self.segment = Segment::new(a, b);
        }
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
        cr.set_source_rgba(&self.get_color());
        cr.set_line_width(self.get_width());
        cr.set_line_cap(self.get_cap());
        cr.set_line_join(self.get_join());
        cr.set_dash(self.get_dashes().as_slice(), self.get_offset());

        if self.get_curve_like() {
            cr.curve(&self.segment, &self.go_dir, &self.arrive_dir);
        } else {
            cr.line(&self.segment);
        }

        if draw_it {
            cr.stroke();
        }
    }

    fn draw_head(&self, cr: &Context, draw_it: bool ) {
        cr.new_path();
        cr.set_source_rgba(&self.get_color());

        let start = self.segment.a();
        let go_dir = &self.go_dir;
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let radius = self.get_width() * 1.25;
        let zero_vec = Vector::new(0.0, 0.0);

        let mut triangle =
                Cone::new(radius * 1.2, radius * 1.2).to_polyline(());

        // get the rotation between out triangle and other vectors
        let mut rotate = if self.get_curve_like() {
            if *go_dir == zero_vec && *arrive_dir == zero_vec {
                let start_vec =
                    Translation::new(-end.x, -end.y).transform_point(start);
                Rotation::rotation_between(
                    &Vector::new(start_vec.x, start_vec.y),
                    &Vector::new(0.0, -1.0)
                )
            } else if *go_dir != zero_vec && *arrive_dir == zero_vec {
                let go_dir_vec =
                    Translation::new(-end.x + go_dir.x, -end.y + go_dir.y)
                        .transform_point(start);
                Rotation::rotation_between(
                    &Vector::new(go_dir_vec.x, go_dir_vec.y),
                    &Vector::new(0.0, -1.0)
                )
            } else {
                Rotation::rotation_between(arrive_dir, &Vector::new(0.0, -1.0))
            }
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

    // fn draw_tail(&self, _cr: &Context, _draw_it: bool) {
        // TODO
    // }

    fn draw_body(&self, cr: &Context, draw_it: bool) {
        cr.save();
        self.draw_segment(cr, draw_it);
        cr.restore();
        if self.get_have_head() {
            cr.save();
            self.draw_head(cr, draw_it);
            cr.restore();
        }
        // cr.save();
        // self.draw_tail(cr, draw_it);
        // cr.restore();
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

        if self.get_curve_like() {
            cr.set_line_width(self.line_width());
            let color = self.get_color();
            cr.set_source_rgba(&RgbaColor::new(
                color.color.red,
                color.color.green,
                color.color.blue,
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
        if self.get_curve_like() {
            cr.save();
            self.draw_go_direction(cr, true);
            cr.restore();
            cr.save();
            self.draw_arrive_direction(cr, true);
            cr.restore();
        }
    }
}

impl Order for LineArrow {}
impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context) {
        if !self.is_visible() || self.get_color().alpha == 0.0 {
            return;
        }

        cr.save();

        self.draw_body(&cr, true);
        if self.is_selected() {
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

        if self.get_curve_like() {
            cr.save();
            self.draw_go_direction(cr, false);
            extents.push(cr.user_to_device_rect(&cr.stroke_extents()));
            cr.restore();

            cr.save();
            self.draw_arrive_direction(cr, false);
            extents.push(cr.user_to_device_rect(&cr.stroke_extents()));
            cr.restore();
        }

        cr.save();
        self.draw_start_point(cr, false);
        extents.push(cr.user_to_device_rect(&cr.stroke_extents()));
        cr.restore();

        cr.save();
        self.draw_end_point(cr, false);
        extents.push(cr.user_to_device_rect(&cr.stroke_extents()));
        cr.restore();

        cr.save();
        self.draw_segment(cr, false);
        extents.push(cr.user_to_device_rect(&cr.stroke_extents()));
        cr.restore();

        if self.get_have_head() {
            cr.save();
            self.draw_head(cr, false);
            extents.push(cr.user_to_device_rect(&cr.fill_extents()));
            cr.restore();
        }

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
    fn name(&self) -> String {
        self.name.borrow().clone()
    }

    fn set_name(&mut self, name: &String) {
        self.name.borrow_mut().clone_from(&name);
    }
}

impl Color for LineArrow {
    fn get_color(&self) -> RgbaColor {
        self.color.borrow().clone()
    }

    fn set_color(&mut self, color: &RgbaColor) {
        self.color.borrow_mut().clone_from(color);
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

impl Select for LineArrow {
    fn is_selected(&self) -> bool {
        self.selected
    }

    fn select(&mut self) {
        self.selected = true;
    }

    fn unselect(&mut self) {
        self.selected = false;
    }

    fn toggle_select(&mut self) -> bool {
        self.selected = !self.selected;
        self.selected
    }
}

impl Lock for LineArrow {
    fn is_locked(&self) -> bool {
        *self.lock.borrow()
    }

    fn lock(&mut self) {
        if !self.is_locked() {
            self.lock.borrow_mut().clone_from(&true);
        }
    }

    fn unlock(&mut self) {
        if self.is_locked() {
            self.lock.borrow_mut().clone_from(&false);
        }
    }

    fn toggle_lock(&mut self) -> bool {
        self.lock.borrow_mut().clone_from(&!self.is_locked());
        self.is_locked()
    }
}

impl Visible for LineArrow {
    fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    fn show(&mut self) {
        if !self.is_visible() {
            self.visible.borrow_mut().clone_from(&true);
        }
    }

    fn hide(&mut self) {
        if self.is_visible() {
            self.visible.borrow_mut().clone_from(&false);
        }
    }

    fn toggle_visible(&mut self) -> bool {
        self.visible.borrow_mut().clone_from(&!self.is_visible());
        self.is_visible()
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

    fn get_children(&self) -> &Vec<Box<ShapeTrait>> {
        &self.children
    }

    fn get_mut_children(&mut self) -> &mut Vec<Box<ShapeTrait>> {
        &mut self.children
    }

    fn set_children(&mut self, children: Vec<Box<ShapeTrait>>) {
        self.children = children;
    }
}

impl Flip for LineArrow {
    fn flip_vertical(&mut self) {
        let matrix = Matrix::new(1.0, 0.0, 0.0, -1.0);
        let mut segment = self.segment.clone();
        let center = na::center(segment.a(), segment.b());
        let translate = Translation::new(-center.x, -center.y);

        let mut a = translate.translate_point(segment.a());
        let mut b = translate.translate_point(segment.b());

        a = matrix * a;
        a = translate.inverse_transform_point(&a);

        b = matrix * b;
        b = translate.inverse_transform_point(&b);

        let mut go_dir = matrix * self.go_dir.clone();
        let mut arrive_dir = matrix * self.arrive_dir.clone();

        segment = Segment::new(a, b);

        self.segment = segment;
        self.go_dir = go_dir;
        self.arrive_dir = arrive_dir;
    }

    fn flip_horizontal(&mut self) {
        let matrix = Matrix::new(-1.0, 0.0, 0.0, 1.0);
        let mut segment = self.segment.clone();
        let center = na::center(segment.a(), segment.b());
        let translate = Translation::new(-center.x, -center.y);

        let mut a = translate.translate_point(segment.a());
        let mut b = translate.translate_point(segment.b());

        a = matrix * a;
        a = translate.inverse_transform_point(&a);

        b = matrix * b;
        b = translate.inverse_transform_point(&b);

        let mut go_dir = matrix * self.go_dir.clone();
        let mut arrive_dir = matrix * self.arrive_dir.clone();

        segment = Segment::new(a, b);

        self.segment = segment;
        self.go_dir = go_dir;
        self.arrive_dir = arrive_dir;
    }
}

impl Rotate for LineArrow {
    fn rotate_left(&mut self) {
        let matrix = Matrix::new(0.0, 1.0, -1.0, 0.0);
        let mut segment = self.segment.clone();
        let center = na::center(segment.a(), segment.b());
        let translate = Translation::new(-center.x, -center.y);

        let mut a = translate.translate_point(segment.a());
        let mut b = translate.translate_point(segment.b());

        a = matrix * a;
        a = translate.inverse_transform_point(&a);

        b = matrix * b;
        b = translate.inverse_transform_point(&b);

        let mut go_dir = matrix * self.go_dir.clone();
        let mut arrive_dir = matrix * self.arrive_dir.clone();

        segment = Segment::new(a, b);

        self.segment = segment;
        self.go_dir = go_dir;
        self.arrive_dir = arrive_dir;
    }

    fn rotate_right(&mut self) {
        let matrix = Matrix::new(0.0, -1.0, 1.0, 0.0);
        let mut segment = self.segment.clone();
        let center = na::center(segment.a(), segment.b());
        let translate = Translation::new(-center.x, -center.y);

        let mut a = translate.translate_point(segment.a());
        let mut b = translate.translate_point(segment.b());

        a = matrix * a;
        a = translate.inverse_transform_point(&a);

        b = matrix * b;
        b = translate.inverse_transform_point(&b);

        let mut go_dir = matrix * self.go_dir.clone();
        let mut arrive_dir = matrix * self.arrive_dir.clone();

        segment = Segment::new(a, b);

        self.segment = segment;
        self.go_dir = go_dir;
        self.arrive_dir = arrive_dir;
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
        if self.is_locked() || !self.is_visible() {
            return false;
        }

        match self.get_mode() {
            Mode::Editing => {
                if event.get_state() == gdk::BUTTON1_MASK {
                    match self.action {
                        None => return false,
                        Some(Actions::MoveGoDirection) => {
                            self.move_go_dir(pos);
                        },
                        Some(Actions::MoveArriveDirection) => {
                            self.move_arrive_dir(pos);
                        },
                        Some(Actions::MoveStartPoint) => {
                            self.move_start_point(pos);
                        },
                        Some(Actions::MoveEndPoint) => {
                            self.move_end_point(pos);
                        },
                        Some(Actions::MoveBody(..)) => {
                            self.move_segment(pos);
                        },
                    };
                    return true;
                }
            },
            _ => {
                match self.action {
                    Some(Actions::MoveStartPoint) => {
                        self.move_start_point(&pos);
                    },
                    Some(Actions::MoveEndPoint) => {
                        self.move_end_point(&pos);
                    },
                    Some(Actions::MoveGoDirection) => {
                        self.move_go_dir(&pos);
                    },
                    Some(Actions::MoveArriveDirection) => {
                        self.move_arrive_dir(&pos);
                    },
                    _ => unreachable!(),
                };
                return true;
            }
        }
        false
    }

    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context,
        options_widget: &gtk::Notebook
    ) -> bool {
        if self.is_locked() {
            return false;
        }

        if event.get_button() == 1 {
            match self.get_mode() {
                Mode::CreatingCurveArrow | Mode::CreatingLineArrow |
                Mode::CreatingCurve | Mode::CreatingLine => {
                    match self.get_action() {
                        Some(Actions::MoveStartPoint) => {
                            self.action = Some(Actions::MoveEndPoint);
                            self.move_start_point(pos);
                            self.move_end_point(pos);
                            self.show();
                        },
                        Some(Actions::MoveEndPoint) => {
                            if self.get_mode() == Mode::CreatingCurve ||
                               self.get_mode() == Mode::CreatingCurveArrow {
                                self.action = Some(Actions::MoveGoDirection);
                            } else {
                                self.action = None;
                                self.mode = Mode::Editing;
                            }
                           self.move_end_point(pos);
                        },
                        Some(Actions::MoveGoDirection) => {
                            self.action = Some(Actions::MoveArriveDirection);
                            self.move_go_dir(pos);
                        },
                        Some(Actions::MoveArriveDirection) => {
                            self.action = None;
                            self.mode = Mode::Editing;
                            self.move_arrive_dir(pos);
                        },
                        _ => unreachable!(),
                    };
                    self.connect_ui(options_widget);
                    self.select();
                    return true;
                },
                Mode::Editing => {
                    let action = self.select_controller(pos, cr);
                    self.set_action(action);
                    if let None = self.get_action() {
                        self.unselect();
                        return false;
                    } else {
                        self.select();
                        self.connect_ui(options_widget);
                        return true;
                    }
                },
            };
        }
        false
    }
}

impl super::Mode for LineArrow {
    fn in_creating_mode(&self) -> bool {
        match self.get_mode() {
            Mode::CreatingLineArrow | Mode::CreatingLine |
            Mode::CreatingCurveArrow | Mode::CreatingCurve => {
                true
            }
            _ => false
        }
    }

    fn in_editing_mode(&self) -> bool {
        match self.get_mode() {
            Mode::Editing => {
                true
            }
            _ => false
        }
    }
}
