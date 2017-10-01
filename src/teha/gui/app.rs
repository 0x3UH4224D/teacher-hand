//
// app.rs
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

use gettextrs::*;

use gtk::{self, WidgetExt, BuilderExt, GtkApplicationExt};
use gio::{self, SimpleAction, SimpleActionExt, ActionMapExt, MenuExt};

use super::*;
use super::main_window::MainWindow;

pub struct Application {
    parent: gtk::Application,
    builder: gtk::Builder,
    settings: gio::Settings,
    main_window: Rc<RefCell<MainWindow>>,
    shortcuts_window: gtk::ShortcutsWindow,
    view_mode: ViewMode,
    last_view_mode: ViewMode,
}

impl Application {
    pub fn new(app: gtk::Application) -> Rc<RefCell<Self>> {
        let settings = gio::Settings::new("org.muhannad.teacher-hand.application");

        let mod_glade = include_str!("app.glade");
        let shortcuts_ui = include_str!("shortcuts.ui");
        let builder = gtk::Builder::new_from_string(mod_glade);
        builder.add_from_string(shortcuts_ui).unwrap();

        let shortcuts_window: gtk::ShortcutsWindow = builder.get_object("shortcuts-window").unwrap();

        shortcuts_window.connect_delete_event(|window, _event| {
            window.hide_on_delete();
            gtk::Inhibit(true)
        });

        let main_window = MainWindow::new(builder.clone(), app.clone());

        let teha_app = Application {
            parent: app,
            builder: builder,
            main_window: Rc::new(RefCell::new(main_window)),
            shortcuts_window: shortcuts_window,
            settings: settings,
            view_mode: ViewMode::StartUp,
            last_view_mode: ViewMode::StartUp,
        };

        teha_app.setup_app_menu();

        let rc_app = Rc::new(RefCell::new(teha_app));

        Application::setup_actions(rc_app.clone());
        Application::connect_ui(rc_app.clone());

        {
            let rc_app = rc_app.borrow();
            let window = rc_app.main_window.borrow();
            window.show();
        }

        rc_app
    }

    pub fn get_parent(&self) -> gtk::Application {
        self.parent.clone()
    }

    pub fn get_builder(&self) -> gtk::Builder {
        self.builder.clone()
    }

    pub fn get_settings(&self) -> gio::Settings {
        self.settings.clone()
    }

    pub fn get_main_window(&self) -> Rc<RefCell<MainWindow>> {
        self.main_window.clone()
    }

    pub fn get_shortcuts_window(&self) -> gtk::ShortcutsWindow {
        self.shortcuts_window.clone()
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn get_last_view_mode(&self) -> ViewMode {
        self.last_view_mode
    }

    pub fn set_last_view_mode(&mut self, mode: ViewMode) {
        self.last_view_mode = mode;
    }

    pub fn update_view(&mut self, mode: ViewMode) {
        match self.view_mode {
            ViewMode::StartUp | ViewMode::Editing => self.last_view_mode = self.view_mode,
            _ => {},
        }
        self.view_mode = mode;

        let window = self.main_window.clone();
        window.borrow_mut().update_view(mode);

        let header_bar = window.borrow().get_header_bar();
        header_bar.borrow_mut().update_view(mode);
    }

    fn setup_app_menu(&self) {
        let app_menu = gio::Menu::new();
        app_menu.append(gettext("New Work").as_str(), "win.new-work");
        app_menu.append(gettext("Open Work").as_str(), "win.open-work");

        let section1 = gio::Menu::new();
        section1.append(gettext("Preferences").as_str(), "win.preferences");

        let section2 = gio::Menu::new();
        section2.append(gettext("Help").as_str(), "app.help");
        section2.append(gettext("About").as_str(), "win.about");
        section2.append(gettext("Shortcuts").as_str(), "app.shortcuts");
        section2.append(gettext("Quit").as_str(), "app.quit");

        app_menu.append_section("", &section1);
        app_menu.append_section("", &section2);

        self.parent.set_app_menu(Some(&app_menu));
    }

    fn setup_actions(app: Rc<RefCell<Self>>) {
        use gio::ApplicationExt;

        // Quit action
        {
            let teha_app = app.borrow_mut();
            let gtk_app = teha_app.parent.clone();
            let action = SimpleAction::new("quit", None);
            action.connect_activate(move |_action, _data| {
                gtk_app.quit();
            });
            teha_app.parent.add_action(&action);
            teha_app.parent.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
        }

        // Help action
        // {
        //     let action = SimpleAction::new("help", None);
        //     let application = app.clone();
        //     action.connect_activate(move |action, data| {

        //     });
        //     app.add_action(&action);
        //     app.set_accels_for_action("app.help", &["F1"]);
        // }

        // Shortcuts action
        {
            let teha_app = app.borrow_mut();
            let shortcuts_window = teha_app.shortcuts_window.clone();
            let action = SimpleAction::new("shortcuts", None);
            action.connect_activate(move |_action, _me| {
                shortcuts_window.show();
            });
            teha_app.parent.add_action(&action);
            teha_app.parent.set_accels_for_action("app.shortcuts", &["<Ctrl>F1"]);
        }
    }

    fn connect_ui(app: Rc<RefCell<Application>>) {
        MainWindow::connect_ui(app.clone());
    }
}
