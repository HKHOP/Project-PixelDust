pub(super) fn bootstrap_runtime() -> (Option<pd_browser::BrowserRuntime>, Option<String>) {
    let browser = match pd_browser::Browser::new() {
        Ok(browser) => browser,
        Err(error) => return (None, Some(error.to_string())),
    };

    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return (
                None,
                Some(format!("failed to determine runtime executable: {error}")),
            );
        }
    };

    let config = pd_browser::RuntimeLaunchConfig::new(executable);
    match browser.boot_with_runtime(&config) {
        Ok(runtime) => (Some(runtime), None),
        Err(error) => (None, Some(error.to_string())),
    }
}
