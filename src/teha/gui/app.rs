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

use gtk::{self, ApplicationExt, WidgetExt};
use gio::{self, SimpleAction, SimpleActionExt, ActionMapExt, MenuExt};

use super::*;
use super::main_window::TehaMainWindow;

pub struct TehaApplication {
    pub parent: gtk::Application,
    pub builder: gtk::Builder,
    pub settings: gio::Settings,
    pub main_window: Option<Rc<RefCell<TehaMainWindow>>>,
    pub shortcuts_window: gtk::ShortcutsWindow,
    pub current_tool: CurrentTool,
    pub view_mode: ViewMode,
    pub last_view_mode: ViewMode,
}

impl TehaApplication {
    pub fn new(app: gtk::Application) -> Rc<RefCell<Self>> {
        let settings = gio::Settings::new("org.muhannad.teacher-hand.application");

        let mod_glade = include_str!("app.glade");
        let shortcuts_ui = include_str!("shortcuts.ui");
        let builder = gtk::Builder::new_from_string(mod_glade);
        builder.add_from_string(shortcuts_ui).unwrap();

        let shortcuts_window: gtk::ShortcutsWindow = builder.get_object("shortcuts-window").unwrap();

        shortcuts_window.connect_delete_event(|window, event| {
            window.hide_on_delete();
            gtk::Inhibit(true)
        });

        let mut teha_app = TehaApplication {
            parent: app,
            builder: builder,
            main_window: None,
            shortcuts_window: shortcuts_window,
            settings: settings,
            current_tool: CurrentTool::Tool(Tool::Controller),
            view_mode: ViewMode::StartUp,
            last_view_mode: ViewMode::StartUp,
        };

        let main_window = TehaMainWindow::new(&teha_app);
        teha_app.main_window = Some(Rc::new(RefCell::new(main_window)));

        teha_app.setup_app_menu();

        let rc_app = Rc::new(RefCell::new(teha_app));

        TehaApplication::setup_actions(rc_app.clone());
        TehaApplication::connect_ui(rc_app.clone());

        {
            let rc_app = rc_app.borrow();
            let window = rc_app.main_window.as_ref().unwrap().borrow();
            window.show();
        }

        rc_app
    }

    pub fn update_view(&mut self, mode: ViewMode) {
        match self.view_mode {
            ViewMode::StartUp | ViewMode::Editing => self.last_view_mode = self.view_mode,
            _ => {},
        }
        self.view_mode = mode;

        let window = self.main_window.as_ref().unwrap().clone();
        window.borrow_mut().update_view(mode);

        let header_bar = window.borrow().header_bar.as_ref().unwrap().clone();
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
            action.connect_activate(move |action, data| {
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
            action.connect_activate(move |action, data| {
                shortcuts_window.show();
            });
            teha_app.parent.add_action(&action);
            teha_app.parent.set_accels_for_action("app.shortcuts", &["<Ctrl>F1"]);
        }
    }

    fn connect_ui(app: Rc<RefCell<TehaApplication>>) {
        TehaMainWindow::connect_ui(app.clone());
    }
}
