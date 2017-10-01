//
// drawing_area.rs
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

use gtk;
use gtk::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use super::app::Application;
use common::types::*;

pub struct DrawingArea {
    parent: gtk::DrawingArea,
    scrolled_drawing_area: gtk::ScrolledWindow,
}

impl DrawingArea {
    pub fn new(builder: gtk::Builder) -> Self {
        let drawing_area: gtk::DrawingArea =
            builder.get_object("drawing_area").unwrap();
        let scrolled_drawing_area: gtk::ScrolledWindow =
            builder.get_object("scrolled_drawing_area").unwrap();

        DrawingArea {
            parent: drawing_area,
            scrolled_drawing_area: scrolled_drawing_area,
        }
    }

    pub fn get_parent(&mut self) -> gtk::DrawingArea {
        self.parent.clone()
    }

    pub fn get_scrolled_drawing_area(&mut self) -> gtk::ScrolledWindow {
        self.scrolled_drawing_area.clone()
    }

    pub fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.get_main_window();
        let drawing_area = window.borrow().get_drawing_area();
        let drawing_area = drawing_area
            .borrow()
            .parent
            .clone();

        // connect drawing_area::draw to document::draw
        {
            let window = window.clone();
            drawing_area.connect_draw(move |me, cr| {
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                let page_bound = window.borrow()
                                       .get_active_document()
                                       .get_active_page()
                                       .draw_extents();

                if let Some(val) = page_bound {
                    let fixed_margin = 100.0;
                    let width = (val.maxs().x - val.mins().x).abs();
                    let height = (val.maxs().y - val.mins().y).abs();
                    me.set_size_request(
                        (width + fixed_margin) as i32,
                        (height + fixed_margin) as i32
                    );

                    let mut translate =
                        Vector::new(fixed_margin / 2.0, fixed_margin / 2.0);
                    translate.x += val.mins().x.abs();
                    translate.y += val.mins().y.abs();

                    {
                        window.borrow_mut()
                            .get_mut_active_document()
                            .get_mut_active_page()
                            .set_translate(translate);
                    }
                }

                window.borrow().get_active_document().draw(cr);

                // FIXME: I'm pretty sure this is not the right way!
                me.queue_draw();
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_motion_notify_event to document::motion_notify
        {
            let window = window.clone();
            drawing_area.connect_motion_notify_event(move |_me, event| {
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                window.borrow_mut()
                      .get_mut_active_document()
                      .motion_notify(event);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_button_press_event to document::button_press
        {
            let window = window.clone();
            drawing_area.connect_button_press_event(move |me, event| {
                me.grab_focus();
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                let options_widget = window.borrow().get_ed_options();

                window.borrow_mut()
                      .get_mut_active_document()
                      .button_press(event, &options_widget);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_button_release_event to document::button_release
        {
            let window = window.clone();
            drawing_area.connect_button_release_event(move |_me, event| {
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                window.borrow_mut()
                      .get_mut_active_document()
                      .button_release(event);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_key_press_event to document::key_press
        {
            let window = window.clone();
            drawing_area.connect_key_press_event(move |_me, event| {
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                window.borrow_mut().get_mut_active_document().key_press(event);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_key_release_event to document::key_release
        {
            let window = window.clone();
            drawing_area.connect_key_release_event(move |_me, event| {
                if window.borrow().get_documents().len() == 0 {
                    return Inhibit(false);
                }

                let current_document =
                    window.borrow().get_active_document_index();

                window.borrow_mut()
                      .get_mut_documents()[current_document]
                      .key_release(event);
                Inhibit(true)
            });
        }
    }
}
