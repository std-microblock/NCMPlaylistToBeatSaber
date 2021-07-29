#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::panic::catch_unwind;
    match catch_unwind(|| {
        let app = ncmp2bs::MicroApp::default();
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(Box::new(app), native_options);
    }) {
        Ok(_) => {}
        Err(err) => {
            match err.downcast::<String>() {
                Ok(panic_msg) => {
                    let mut suggestion = "（无建议）";
                    if panic_msg.contains("PermissionDenied") {
                        suggestion = "用管理员权限启动本程序"
                    }
                    msgbox::create(
                    "出错了",
                    format!("出错了！\n或许你可以试试：{}\n-----------------------\n错误信息:\n{:#?}", suggestion,panic_msg).as_str(),
                    msgbox::IconType::Error,
                )
                .unwrap();
                }
                Err(_) => {
                    msgbox::create(
                        "出错了",
                        format!("错误信息:\n{:#?}", "无").as_str(),
                        msgbox::IconType::Error,
                    )
                    .unwrap();
                }
            }
        }
    };
}
