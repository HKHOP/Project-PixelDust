use super::navigation::dispatch_dom_events;
use super::navigation::execute_navigation;
use super::navigation::normalize_input_url;
use super::runtime::bootstrap_runtime;
use super::*;

impl Default for BrowserUiApp {
    fn default() -> Self {
        let (runtime, runtime_last_error) = bootstrap_runtime();

        Self {
            address_input: DEFAULT_URL.to_owned(),
            current_url: None,
            page_view: None,
            status_line: "Ready".to_owned(),
            last_error: None,
            trust_store: TrustStoreSelection::WebPkiOnly,
            ocsp_required: true,
            history: Vec::new(),
            history_index: None,
            next_request_id: 1,
            inflight_request_id: None,
            nav_receiver: None,
            show_navigation_details: false,
            image_textures: HashMap::new(),
            form_state: HashMap::new(),
            cache: Arc::new(Mutex::new(HttpCache::default())),
            runtime,
            runtime_workers: Vec::new(),
            runtime_restarts: 0,
            runtime_last_error,
            runtime_last_poll: Instant::now(),
        }
    }
}

impl BrowserUiApp {
    fn navigate(&mut self, raw_url: String, add_to_history: bool) {
        let normalized_url = normalize_input_url(raw_url);
        self.address_input = normalized_url.clone();
        self.status_line = format!("Loading {}...", normalized_url);
        self.last_error = None;

        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.inflight_request_id = Some(request_id);

        let trust_store = self.trust_store;
        let ocsp_required = self.ocsp_required;
        let cache = Arc::clone(&self.cache);
        let (tx, rx) = mpsc::channel();
        self.nav_receiver = Some(rx);

        let nav_job = move || {
            let result = execute_navigation(&normalized_url, trust_store, ocsp_required, cache);
            let _ = tx.send(NavigationResult {
                request_id,
                url: normalized_url,
                add_to_history,
                result,
            });
        };

        if thread::Builder::new()
            .name("pixeldust-nav".to_owned())
            .stack_size(NAVIGATION_THREAD_STACK_SIZE)
            .spawn(nav_job)
            .is_err()
        {
            self.inflight_request_id = None;
            self.nav_receiver = None;
            self.status_line = "Navigation failed".to_owned();
            self.last_error = Some("failed to spawn navigation worker".to_owned());
        }
    }

    fn poll_navigation(&mut self) {
        loop {
            let message = self
                .nav_receiver
                .as_ref()
                .and_then(|receiver| receiver.try_recv().ok());

            let Some(message) = message else {
                break;
            };

            if Some(message.request_id) != self.inflight_request_id {
                continue;
            }

            self.inflight_request_id = None;
            self.nav_receiver = None;

            match message.result {
                Ok(page) => {
                    self.current_url = Some(page.final_url.clone());
                    self.status_line = format!(
                        "Loaded {} (status {}, {} bytes)",
                        page.final_url, page.status_code, page.body_bytes
                    );

                    if message.add_to_history {
                        self.push_history(message.url);
                    }

                    self.image_textures.clear();
                    self.form_state.clear();
                    self.page_view = Some(page);
                    self.last_error = None;
                }
                Err(error) => {
                    self.status_line = "Navigation failed".to_owned();
                    self.last_error = Some(error);
                }
            }
        }
    }

    fn poll_runtime(&mut self) {
        if self.runtime_last_poll.elapsed() < RUNTIME_POLL_INTERVAL {
            return;
        }
        self.runtime_last_poll = Instant::now();

        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };

        match runtime.restart_exited_workers() {
            Ok(restarts) => {
                self.runtime_restarts = self.runtime_restarts.saturating_add(restarts.len());
            }
            Err(error) => {
                self.runtime_last_error = Some(error.to_string());
            }
        }

        match runtime.worker_health() {
            Ok(health) => {
                self.runtime_workers = health
                    .into_iter()
                    .map(|worker| RuntimeWorkerStatus {
                        role: worker.role,
                        pid: worker.pid,
                        running: worker.running,
                        exit_code: worker.exit_code,
                    })
                    .collect();
            }
            Err(error) => {
                self.runtime_last_error = Some(error.to_string());
            }
        }
    }

    fn shutdown_runtime(&mut self) {
        let Some(runtime) = self.runtime.take() else {
            return;
        };

        if let Err(error) = runtime.shutdown() {
            self.runtime_last_error = Some(error.to_string());
        }
    }

    fn push_history(&mut self, url: String) {
        if let Some(index) = self.history_index {
            let keep_to = index.saturating_add(1);
            self.history.truncate(keep_to);
        }

        if self.history.last().is_some_and(|existing| existing == &url) {
            self.history_index = Some(self.history.len().saturating_sub(1));
            return;
        }

        self.history.push(url);
        self.history_index = Some(self.history.len().saturating_sub(1));
    }

    fn can_go_back(&self) -> bool {
        matches!(self.history_index, Some(index) if index > 0)
    }

    fn can_go_forward(&self) -> bool {
        matches!(self.history_index, Some(index) if index + 1 < self.history.len())
    }

    fn navigate_back(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };

        if index == 0 {
            return;
        }

        let next_index = index - 1;
        self.history_index = Some(next_index);
        if let Some(url) = self.history.get(next_index).cloned() {
            self.navigate(url, false);
        }
    }

    fn navigate_forward(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };

        let next_index = index + 1;
        if next_index >= self.history.len() {
            return;
        }

        self.history_index = Some(next_index);
        if let Some(url) = self.history.get(next_index).cloned() {
            self.navigate(url, false);
        }
    }

    fn reload(&mut self) {
        if let Some(current) = self.current_url.clone() {
            self.navigate(current, false);
        } else {
            self.navigate(self.address_input.clone(), true);
        }
    }

    fn is_loading(&self) -> bool {
        self.inflight_request_id.is_some()
    }

    fn render_viewport(&mut self, ui: &mut egui::Ui, navigate_to: &mut Option<String>) {
        let image_textures = &mut self.image_textures;
        let form_state = &mut self.form_state;
        match self.page_view.as_mut() {
            Some(page) => {
                if let Some(title) = &page.title {
                    ui.label(format!("Title: {title}"));
                }
                ui.label(format!("Content-Type: {}", page.content_type));
                ui.label(format!("Body bytes: {}", page.body_bytes));
                ui.label(format!(
                    "Subresources: css ext {}, inline tags {}, css rules {}, images {}, scripts {}, blocked {}",
                    page.subresource_stats.stylesheets_loaded,
                    page.subresource_stats.inline_style_tags,
                    page.subresource_stats.css_rules_total,
                    page.subresource_stats.images_loaded,
                    page.subresource_stats.scripts_loaded,
                    page.subresource_stats.blocked
                ));
                ui.label(format!(
                    "JavaScript: {} (seen {}, ran {}, failed {}, skipped {}, events {}, event-failures {})",
                    if page.js_execution.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    page.js_execution.scripts_seen,
                    page.js_execution.scripts_executed,
                    page.js_execution.scripts_failed,
                    page.js_execution.scripts_skipped,
                    page.js_execution.event_dispatches,
                    page.js_execution.event_failures
                ));
                if let Some(draw_calls) = page.renderer_draw_calls {
                    ui.label(format!("Renderer baseline draw calls: {draw_calls}"));
                }
                ui.separator();

                if let Some(fallback_text) = page.static_text_fallback.as_ref() {
                    ui.colored_label(
                        egui::Color32::from_rgb(209, 153, 29),
                        "This page is JavaScript-driven. Showing static fallback text.",
                    );
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("viewport_static_fallback_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(fallback_text.as_str())
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(226, 226, 226)),
                            );
                        });
                } else if let Some(doc) = page.html_document.as_ref() {
                    let mut action = simple_html::RenderAction::default();
                    egui::ScrollArea::vertical()
                        .id_salt("viewport_html_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut render_images = HashMap::new();
                            for image in &page.decoded_images {
                                if !image_textures.contains_key(&image.url) {
                                    let texture = ui.ctx().load_texture(
                                        format!("img:{}", image.url),
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [image.width, image.height],
                                            &image.rgba,
                                        ),
                                        egui::TextureOptions::LINEAR,
                                    );
                                    image_textures.insert(image.url.clone(), texture);
                                }

                                if let Some(texture) = image_textures.get(&image.url) {
                                    render_images.insert(
                                        image.url.clone(),
                                        simple_html::RenderImage {
                                            texture_id: texture.id(),
                                            size: egui::vec2(
                                                image.width as f32,
                                                image.height as f32,
                                            ),
                                        },
                                    );
                                }
                            }

                            let resources = simple_html::RenderResources {
                                images: &render_images,
                            };
                            simple_html::render_document(
                                ui,
                                doc,
                                &page.final_url,
                                &resources,
                                &mut action,
                                form_state,
                            );
                        });
                    if action.navigate_to.is_some() {
                        *navigate_to = action.navigate_to;
                    }
                    if let Some(js_nav) = dispatch_dom_events(page, &action.dom_events) {
                        *navigate_to = Some(js_nav);
                    }
                } else {
                    ui.label("Non-HTML response, showing raw preview.");
                    egui::ScrollArea::vertical()
                        .id_salt("viewport_preview_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(page.body_preview.as_str())
                                    .monospace()
                                    .size(12.0),
                            );
                        });
                }
            }
            None => {
                ui.label("No page loaded yet.");
            }
        }
    }

    fn render_navigation_details(&self, ui: &mut egui::Ui) {
        ui.heading("Navigation Details");
        ui.separator();

        ui.label(format!("Runtime restarts: {}", self.runtime_restarts));
        if self.runtime_workers.is_empty() {
            ui.label("Workers: not started");
        } else {
            ui.label("Workers");
            for worker in &self.runtime_workers {
                let state = if worker.running {
                    "running".to_owned()
                } else if let Some(code) = worker.exit_code {
                    format!("exited ({code})")
                } else {
                    "stopped".to_owned()
                };
                ui.label(format!(
                    "{} pid={} {state}",
                    worker.role.as_str(),
                    worker.pid
                ));
            }
        }
        if let Some(error) = &self.runtime_last_error {
            ui.colored_label(
                egui::Color32::from_rgb(200, 65, 65),
                format!("Runtime error: {error}"),
            );
        }
        ui.separator();

        if let Some(page) = &self.page_view {
            ui.label(format!("URL: {}", page.final_url));
            ui.label(format!("Status: {}", page.status_code));
            ui.label(format!("HTTP Version: {}", page.http_version));
            ui.label(format!("Body Bytes: {}", page.body_bytes));
            ui.label(format!(
                "JavaScript: seen {}, ran {}, failed {}, skipped {}, events {}, event-failures {}",
                page.js_execution.scripts_seen,
                page.js_execution.scripts_executed,
                page.js_execution.scripts_failed,
                page.js_execution.scripts_skipped,
                page.js_execution.event_dispatches,
                page.js_execution.event_failures
            ));
            if let Some(draw_calls) = page.renderer_draw_calls {
                ui.label(format!("Renderer baseline draw calls: {draw_calls}"));
            }
            if !page.js_execution.errors.is_empty() {
                ui.separator();
                ui.label("JavaScript Errors");
                for error in &page.js_execution.errors {
                    ui.label(error);
                }
            }
            ui.separator();
            ui.label("Response Headers");
            egui::ScrollArea::vertical()
                .id_salt("response_headers_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (name, value) in &page.headers {
                        ui.label(format!("{name}: {value}"));
                    }
                });
        } else {
            ui.label("No navigation details available yet.");
        }
    }
}

impl Drop for BrowserUiApp {
    fn drop(&mut self) {
        self.shutdown_runtime();
    }
}

impl eframe::App for BrowserUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_navigation();
        self.poll_runtime();
        if ctx.input(|input| input.key_pressed(egui::Key::F12)) {
            self.show_navigation_details = !self.show_navigation_details;
        }
        if self.is_loading() {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else if self.runtime.is_some() {
            ctx.request_repaint_after(RUNTIME_POLL_INTERVAL);
        }

        egui::TopBottomPanel::top("toolbar_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.can_go_back(), egui::Button::new("Back"))
                    .clicked()
                {
                    self.navigate_back();
                }
                if ui
                    .add_enabled(self.can_go_forward(), egui::Button::new("Forward"))
                    .clicked()
                {
                    self.navigate_forward();
                }
                if ui.button("Reload").clicked() {
                    self.reload();
                }

                let width = (ui.available_width() - 110.0).max(200.0);
                let response = ui.add_sized(
                    [width, 28.0],
                    egui::TextEdit::singleline(&mut self.address_input).hint_text("Enter URL"),
                );

                let pressed_enter =
                    response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if pressed_enter || ui.button("Go").clicked() {
                    self.navigate(self.address_input.clone(), true);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Trust");
                egui::ComboBox::from_id_salt("trust_store_mode")
                    .selected_text(self.trust_store.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.trust_store,
                            TrustStoreSelection::WebPkiOnly,
                            TrustStoreSelection::WebPkiOnly.label(),
                        );
                        ui.selectable_value(
                            &mut self.trust_store,
                            TrustStoreSelection::WebPkiAndOs,
                            TrustStoreSelection::WebPkiAndOs.label(),
                        );
                    });

                ui.separator();
                ui.label("OCSP");
                ui.selectable_value(&mut self.ocsp_required, true, "Required");
                ui.selectable_value(&mut self.ocsp_required, false, "Optional");

                ui.separator();
                if let Some(url) = &self.current_url {
                    ui.label(format!("Current: {url}"));
                } else {
                    ui.label("Current: -");
                }

                if self.is_loading() {
                    ui.separator();
                    ui.spinner();
                    ui.label("Loading");
                }

                if !self.runtime_workers.is_empty() {
                    let alive = self
                        .runtime_workers
                        .iter()
                        .filter(|worker| worker.running)
                        .count();
                    ui.separator();
                    ui.label(format!(
                        "Workers: {alive}/{} (restarts {})",
                        self.runtime_workers.len(),
                        self.runtime_restarts
                    ));
                }

                if let Some(error) = &self.runtime_last_error {
                    ui.separator();
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 65, 65),
                        format!("Runtime: {error}"),
                    );
                }

                ui.separator();
                ui.label("F12: Navigation Details");
            });
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(&self.status_line);
                if let Some(error) = &self.last_error {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 65, 65),
                        format!("Error: {error}"),
                    );
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut navigate_to: Option<String> = None;
            self.render_viewport(ui, &mut navigate_to);

            if let Some(url) = navigate_to {
                if !self.is_loading() {
                    self.navigate(url, true);
                }
            }
        });

        if self.show_navigation_details {
            egui::Window::new("Navigation Details")
                .id(egui::Id::new("navigation_details_window"))
                .resizable(true)
                .default_size([520.0, 440.0])
                .show(ctx, |ui| {
                    self.render_navigation_details(ui);
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.shutdown_runtime();
    }
}
