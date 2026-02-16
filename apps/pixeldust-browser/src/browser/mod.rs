use eframe::egui;
use encoding_rs::Encoding;
use image::GenericImageView;
use pd_ipc::ProcessRole;
use pd_js::JsExecutionReport;
use pd_js::JsHostElement;
use pd_js::JsHostEnvironment;
use pd_js::JsRuntime;
use pd_js::JsRuntimeConfig;
use pd_js::ScriptSource;
use pd_net::Header;
use pd_net::TrustStoreMode;
use pd_net::client::Http11Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use url::Url;

use crate::simple_html;

include!("constants.rs");
include!("types.rs");

mod navigation;
mod runtime;
mod startup;
mod ui;

pub(crate) use startup::run;
