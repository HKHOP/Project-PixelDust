mod browser;
mod simple_html;

fn main() -> Result<(), eframe::Error> {
    browser::run()
}
