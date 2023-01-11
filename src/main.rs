#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, io::ErrorKind};

use fltk::{prelude::*, *};
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

fn main() {
    let clsids = vec![
        "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}", // 3D Objects
        "{B4BFCC3A-DB2C-424C-B029-7FE99A87C641}", // Desktop
        "{f42ee2d3-909f-4907-8871-4c22fc0bf756}", // Local Documents
        "{7d83ee9b-2244-4e70-b1f5-5393042af1e4}", // Local Downloads
        "{a0c69a99-21c8-4671-8703-7934162fcf1d}", // Local Music
        "{0ddd015d-b06c-45d5-8c4c-f59713854639}", // Local Pictures
        "{35286a68-3c57-41a1-bbb1-0eae73d76c95}", // Local Videos
    ];

    let app = app::App::default();

    let mut win = window::Window::default()
        .with_size(300, 280)
        .with_label("Hide Default Folders");

    let mut vpack = group::Pack::default()
        .with_type(group::PackType::Vertical)
        .with_size(280, 270)
        .center_of_parent();
    
    vpack.set_spacing(5);

    frame::Frame::default()
        .with_size(280, 25)
        .with_label("Show/Hide folder default shown in explorer:");

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let folder_descriptions = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FolderDescriptions").unwrap();

    let mut folder_status = HashMap::new();
    
    for clsid in clsids.to_vec() {

        let description = folder_descriptions.open_subkey(clsid).unwrap();
        let name: String = description.get_value("Name").unwrap();

        let check_button = button::CheckButton::default()
            .with_size(300, 25)
            .with_label(name.as_str());

        check_button.set_checked(true);

        folder_status.insert(clsid, check_button);

        if let Ok(property_bag) = description.open_subkey("PropertyBag") {
            if let Ok(this_pc_policy) = property_bag.get_value::<String, &str>("ThisPCPolicy") {
                if this_pc_policy == "Hide" {
                    if let Some(status) = folder_status.get(clsid) {
                        status.set_checked(false);
                    }
                }
            }
        }
    }

    let mut submit_button = button::Button::default()
        .with_size(280, 25)
        .with_label("Submit Change");

    submit_button.set_callback(move |_| {
        for (clsid, check_buttton) in folder_status.iter() {
            let description = folder_descriptions.open_subkey(clsid).unwrap();            
            let result = description.create_subkey("PropertyBag");
            match result {
                Ok((folder, _)) => {
                    if check_buttton.is_checked() {
                        folder.set_value("ThisPCPolicy", &"Show").unwrap();
                        if clsid.to_string() == "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}" {
                            hklm.create_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\NameSpace\\{0DB7E03F-FC29-4DC6-9020-FF41B59E513A}").unwrap();
                        }
                    } else {
                        folder.set_value("ThisPCPolicy", &"Hide").unwrap();
                        if clsid.to_string() == "{31C0DD25-9439-4F12-BF41-7FF4EDA38722}" {
                            let result = hklm.delete_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\NameSpace\\{0DB7E03F-FC29-4DC6-9020-FF41B59E513A}");
                            match result {
                                _ => {}
                            }
                        }
                    }
                },
                Err(err) => {
                    if err.kind() == ErrorKind::PermissionDenied {
                        dialog::message_title("Permission Denied");
                        dialog::message_default("Permission denied. Please run as administrator.");
                        break;
                    }
                }
            }
        }
    });

    vpack.end();

    win.end();
    win.show();
    
    app.run().unwrap();
   
}
