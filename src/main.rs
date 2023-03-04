#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use winsafe::AnyResult;
use winsafe::HKEY;
use winsafe::RegistryValue;
use winsafe::co;
use winsafe::prelude::*;
use winsafe::gui;

#[derive(Clone)]
struct MainWindow {
    window: gui::WindowMain,
    class_checkbox_group: HashMap<String, gui::CheckBox>,
    submit_button: gui::Button
}

impl MainWindow {
    fn new() -> AnyResult<Self> {
        let clsid_vec = vec![
            "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}", // 3D Objects
            "{B4BFCC3A-DB2C-424C-B029-7FE99A87C641}", // Desktop
            "{f42ee2d3-909f-4907-8871-4c22fc0bf756}", // Local Documents
            "{7d83ee9b-2244-4e70-b1f5-5393042af1e4}", // Local Downloads
            "{a0c69a99-21c8-4671-8703-7934162fcf1d}", // Local Music
            "{0ddd015d-b06c-45d5-8c4c-f59713854639}", // Local Pictures
            "{35286a68-3c57-41a1-bbb1-0eae73d76c95}", // Local Videos
        ];

        let window = gui::WindowMain::new(
            gui::WindowMainOpts {
                title: "Hide Default Folders".to_owned(),
                size: (300, 260),
                ..Default::default()
            }
        );

        let mut height = 0;

        let _label = gui::Label::new(
            &window, 
            gui::LabelOpts { 
                text: "Show/Hide folder default shown in explorer:".to_owned(), 
                position: (10, 10),
                ..Default::default()
            }
        );

        height = height + 20 + 10;

        let mut class_checkbox_group = HashMap::new();

        let folder_descriptions = HKEY::LOCAL_MACHINE.RegOpenKeyEx(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FolderDescriptions",
            co::REG_OPTION::default(),
            co::KEY::READ
        )?;

        for clsid in clsid_vec {
            let classname = {
                match folder_descriptions.RegGetValue(Some(clsid), Some("Name")) {
                    Ok(RegistryValue::Sz(classname)) => classname,
                    _ => clsid.to_owned()
                }
            };

            let checkbox = gui::CheckBox::new(
                &window,
                gui::CheckBoxOpts {
                    text: classname.to_owned(),
                    position: (10, height + 10),
                    ..Default::default()
                }
            );

            // let class_checkbox = ClassCheckBox::new(clsid.to_string(), checkbox);
            class_checkbox_group.insert(clsid.to_owned(), checkbox);
            height = height + 10 + 16;
        }

        let submit_button = gui::Button::new(
            &window, 
            gui::ButtonOpts {
                text: "Submit".to_owned(),
                width: 280,
                height: 25,
                position: (10, height + 10),
                ..Default::default()
            }
        );

        let main_window = Self { window, class_checkbox_group, submit_button };
        main_window.events();

        Ok(main_window)
    }

    fn run(&self) -> AnyResult<i32> {
        self.window.run_main(None)
    }

    fn events(&self) {

        let self2 = self.clone();
        self.submit_button.on().bn_clicked(move || {
            let folder_descriptions = HKEY::LOCAL_MACHINE.RegOpenKeyEx(
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FolderDescriptions",
                co::REG_OPTION::default(),
                co::KEY::WRITE
            )?;

            for (clsid, checkbox) in &self2.class_checkbox_group {
                if checkbox.is_checked() {
                    folder_descriptions.RegSetKeyValue(&format!("{clsid}\\PropertyBag"), "ThisPCPolicy", RegistryValue::Sz("Show".to_owned()))?;
                    if clsid == "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}" {
                        HKEY::LOCAL_MACHINE.RegCreateKeyEx(
                            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\NameSpace\\{0DB7E03F-FC29-4DC6-9020-FF41B59E513A}",
                            None, 
                            co::REG_OPTION::default(),
                            co::KEY::READ,
                            None
                        )?;
                    }
                } else {
                    folder_descriptions.RegSetKeyValue(&format!("{clsid}\\PropertyBag"), "ThisPCPolicy", RegistryValue::Sz("Hide".to_owned()))?;
                    if clsid == "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}" {
                        HKEY::LOCAL_MACHINE.RegDeleteKey(
                            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\NameSpace\\{0DB7E03F-FC29-4DC6-9020-FF41B59E513A}"
                        )?;
                    }
                }
            }

            self2.load_show_policy()?;

            Ok(())
        });

        let self2 = self.clone();
        self.window.on().wm_create(move |_| {
            self2.load_show_policy()?;
            Ok(0)
        });
    }

    fn load_show_policy(&self) -> AnyResult<()> {

        let folder_descriptions = HKEY::LOCAL_MACHINE.RegOpenKeyEx(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FolderDescriptions",
            co::REG_OPTION::default(),
            co::KEY::READ
        )?;

        for clsid in self.class_checkbox_group.keys() {
            let should_show = {
                match folder_descriptions.RegGetValue(Some(&format!("{clsid}\\PropertyBag")), Some("ThisPCPolicy")) {
                    Ok(RegistryValue::Sz(policy)) => policy != "Hide",
                    _ => true
                }
            };
            
            if should_show {
                if let Some((_, checkbox)) = self.class_checkbox_group.get_key_value(clsid) {
                    checkbox.set_check_state(gui::CheckState::Checked);
                }
            }
        }

        Ok(())
    }
}

fn main() {
    match MainWindow::new() {
        Ok(main_window) => {
            if let Err(e) = main_window.run() {
                winsafe::HWND::NULL.MessageBox(
                    &e.to_string(), "Uncaught error", co::MB::ICONERROR).unwrap();
            }  
        }
        Err(e) => {
            winsafe::HWND::NULL.MessageBox(
                &e.to_string(), "Uncaught error", co::MB::ICONERROR).unwrap();
        }
    }
}