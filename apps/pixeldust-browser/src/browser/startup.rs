use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessMode {
    BrowserUi,
    Worker(ProcessRole),
}

pub(crate) fn run() -> Result<(), eframe::Error> {
    match process_mode_from_args() {
        Ok(ProcessMode::Worker(role)) => {
            run_worker(role);
            return Ok(());
        }
        Ok(ProcessMode::BrowserUi) => {}
        Err(error) => {
            eprintln!("PixelDust startup error: {error}");
            return Ok(());
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PixelDust Browser")
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PixelDust Browser",
        native_options,
        Box::new(|cc| {
            install_platform_fonts(&cc.egui_ctx);
            Ok(Box::new(BrowserUiApp::default()))
        }),
    )
}

fn process_mode_from_args() -> Result<ProcessMode, String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg != "--pd-role" {
            continue;
        }

        let role_name = args
            .next()
            .ok_or_else(|| "missing role name after --pd-role".to_owned())?;
        let role = ProcessRole::from_role_name(role_name.as_str()).ok_or_else(|| {
            format!(
                "unsupported process role `{role_name}` (expected: renderer|network|storage|browser)"
            )
        })?;
        return Ok(ProcessMode::Worker(role));
    }

    Ok(ProcessMode::BrowserUi)
}

fn run_worker(role: ProcessRole) {
    // Worker entrypoint is intentionally minimal until typed IPC is fully wired over pipes.
    let _ = role;
    loop {
        thread::sleep(WORKER_IDLE_SLEEP);
    }
}

fn install_platform_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            ("segoe_ui", r"C:\Windows\Fonts\segoeui.ttf"),
            ("segoe_ui_italic", r"C:\Windows\Fonts\segoeuii.ttf"),
            ("segoe_ui_bold", r"C:\Windows\Fonts\segoeuib.ttf"),
            ("segoe_ui_bold_italic", r"C:\Windows\Fonts\segoeuiz.ttf"),
            ("segoe_ui_semibold", r"C:\Windows\Fonts\seguisb.ttf"),
            ("tahoma", r"C:\Windows\Fonts\tahoma.ttf"),
            ("arial", r"C:\Windows\Fonts\arial.ttf"),
            ("arial_italic", r"C:\Windows\Fonts\ariali.ttf"),
            ("arial_bold", r"C:\Windows\Fonts\arialbd.ttf"),
            ("arial_bold_italic", r"C:\Windows\Fonts\arialbi.ttf"),
            ("calibri", r"C:\Windows\Fonts\calibri.ttf"),
            ("calibri_italic", r"C:\Windows\Fonts\calibrii.ttf"),
            ("calibri_bold", r"C:\Windows\Fonts\calibrib.ttf"),
            ("calibri_bold_italic", r"C:\Windows\Fonts\calibriz.ttf"),
            ("times_new_roman", r"C:\Windows\Fonts\times.ttf"),
            ("times_new_roman_italic", r"C:\Windows\Fonts\timesi.ttf"),
            ("times_new_roman_bold", r"C:\Windows\Fonts\timesbd.ttf"),
            ("times_new_roman_bold_italic", r"C:\Windows\Fonts\timesbi.ttf"),
            ("consolas", r"C:\Windows\Fonts\consola.ttf"),
            ("segoe_ui_symbol", r"C:\Windows\Fonts\seguisym.ttf"),
            ("segoe_ui_emoji", r"C:\Windows\Fonts\seguiemj.ttf"),
        ];

        let mut inserted = Vec::new();
        for (name, path) in candidates {
            if let Ok(bytes) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert(name.to_owned(), egui::FontData::from_owned(bytes).into());
                inserted.push(name.to_owned());
            }
        }

        if !inserted.is_empty() {
            let has = |name: &str| inserted.iter().any(|item| item == name);
            let mut regular = Vec::new();
            for name in ["segoe_ui", "calibri", "arial", "tahoma", "times_new_roman"] {
                if has(name) {
                    regular.push(name.to_owned());
                }
            }
            for name in ["segoe_ui_symbol", "segoe_ui_emoji"] {
                if has(name) {
                    regular.push(name.to_owned());
                }
            }
            if regular.is_empty() {
                regular.extend(inserted.iter().cloned());
            }

            let mut bold = Vec::new();
            for name in [
                "segoe_ui_bold",
                "segoe_ui_semibold",
                "calibri_bold",
                "arial_bold",
                "times_new_roman_bold",
            ] {
                if has(name) {
                    bold.push(name.to_owned());
                }
            }
            bold.extend(regular.iter().cloned());

            let mut italic = Vec::new();
            for name in [
                "segoe_ui_italic",
                "calibri_italic",
                "arial_italic",
                "times_new_roman_italic",
            ] {
                if has(name) {
                    italic.push(name.to_owned());
                }
            }
            italic.extend(regular.iter().cloned());

            let mut bold_italic = Vec::new();
            for name in [
                "segoe_ui_bold_italic",
                "calibri_bold_italic",
                "arial_bold_italic",
                "times_new_roman_bold_italic",
            ] {
                if has(name) {
                    bold_italic.push(name.to_owned());
                }
            }
            bold_italic.extend(bold.iter().cloned());
            bold_italic.extend(italic.iter().cloned());

            let mut mono = Vec::new();
            if has("consolas") {
                mono.push("consolas".to_owned());
            }
            mono.extend(regular.iter().cloned());

            fonts.families.insert(
                egui::FontFamily::Name("pd-proportional".into()),
                regular.clone(),
            );
            fonts.families.insert(
                egui::FontFamily::Name("pd-proportional-bold".into()),
                bold.clone(),
            );
            fonts.families.insert(
                egui::FontFamily::Name("pd-proportional-italic".into()),
                italic.clone(),
            );
            fonts.families.insert(
                egui::FontFamily::Name("pd-proportional-bold-italic".into()),
                bold_italic,
            );
            fonts
                .families
                .insert(egui::FontFamily::Name("pd-monospace".into()), mono.clone());

            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                for name in regular.iter().rev() {
                    proportional.insert(0, name.clone());
                }
            }
            if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                for name in mono {
                    monospace.push(name);
                }
            }
        }
    }

    ctx.set_fonts(fonts);
}
