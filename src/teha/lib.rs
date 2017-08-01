//
// lib.rs
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

extern crate gtk;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gettextrs;
extern crate euclid;

pub mod gui;
pub mod error;
pub mod common;

use gtk::prelude::*;
use gio::ApplicationExt;

use gettextrs::*;

use gui::app::TehaApplication;
use error::Error;

use std::io;
use std::env;

pub fn run() -> Result<i32, error::Error> {
    setlocale(LocaleCategory::LcAll, "en_US.UTF-8");
    bindtextdomain("teacherhand", "/usr/local/share/locale");
    textdomain("teacherhand");

    let app = match gtk::Application::new(Some("org.muhnnad.teacherhand"), gio::APPLICATION_FLAGS_NONE) {
        Ok(val) => val,
        Err(_) => return Err(Error::from("Failed to start GtkApplication")),
    };

    app.connect_activate(move |gtk_app| {
        glib::set_application_name("Teacher hand");
        glib::set_prgname(Some("teacher-hand"));

        let teha_app = TehaApplication::new(gtk_app.clone());
    });

    let args: Vec<String> = env::args().collect();
    let argv: Vec<&str> = args.iter()
        .map(|ref x| x.as_str())
        .collect();
    Ok(app.run(&argv))
}
