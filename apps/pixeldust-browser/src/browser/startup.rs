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
            ("tahoma", r"C:\Windows\Fonts\tahoma.ttf"),
            ("arial", r"C:\Windows\Fonts\arial.ttf"),
            ("segoe_ui_symbol", r"C:\Windows\Fonts\seguisym.ttf"),
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
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                for name in inserted.iter().rev() {
                    proportional.insert(0, name.clone());
                }
            }
            if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                for name in inserted {
                    monospace.push(name);
                }
            }
        }
    }

    ctx.set_fonts(fonts);
}
