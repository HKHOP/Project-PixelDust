//! Browser process coordinator.

use pd_ipc::ChannelConfig;
use pd_ipc::ProcessRole;
use pd_net::NetStack;
use pd_privacy::PrivacyPolicy;
use pd_renderer::RendererProcess;
use pd_security::SecurityPolicy;
use pd_storage::StorageConfig;
use pd_storage::StorageManager;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

/// Browser process top-level orchestration object.
#[derive(Debug)]
pub struct Browser {
    pub security: SecurityPolicy,
    pub privacy: PrivacyPolicy,
    pub storage: StorageManager,
    pub network: NetStack,
    pub renderer: RendererProcess,
}

/// Startup summary used by the shell/app layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSummary {
    pub process_channels: usize,
    pub privacy_hardened: bool,
    pub security_hardened: bool,
}

/// Runtime launch configuration for external worker processes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLaunchConfig {
    pub executable: PathBuf,
    pub inherit_stdio: bool,
    pub extra_args: Vec<String>,
}

impl RuntimeLaunchConfig {
    pub fn new(executable: PathBuf) -> Self {
        Self {
            executable,
            inherit_stdio: false,
            extra_args: Vec::new(),
        }
    }
}

/// Spawned worker process metadata.
#[derive(Debug)]
pub struct WorkerProcess {
    pub role: ProcessRole,
    pub child: Child,
}

/// Worker process liveness snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerHealth {
    pub role: ProcessRole,
    pub pid: u32,
    pub running: bool,
    pub exit_code: Option<i32>,
}

/// Worker restart metadata emitted by runtime supervision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerRestart {
    pub role: ProcessRole,
    pub old_pid: u32,
    pub new_pid: u32,
}

/// Browser runtime handle for spawned workers and channel policy.
#[derive(Debug)]
pub struct BrowserRuntime {
    workers: Vec<WorkerProcess>,
    channels: Vec<ChannelConfig>,
    launch_config: RuntimeLaunchConfig,
}

impl BrowserRuntime {
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn channels(&self) -> &[ChannelConfig] {
        &self.channels
    }

    pub fn launch_config(&self) -> &RuntimeLaunchConfig {
        &self.launch_config
    }

    pub fn worker_health(&mut self) -> pd_core::BrowserResult<Vec<WorkerHealth>> {
        let mut health = Vec::with_capacity(self.workers.len());

        for worker in &mut self.workers {
            let pid = worker.child.id();
            let status = worker.child.try_wait().map_err(|error| {
                pd_core::BrowserError::new(
                    "browser.runtime.try_wait_failed",
                    format!(
                        "failed to query {} worker (pid {pid}) status: {error}",
                        worker.role.as_str()
                    ),
                )
            })?;

            let (running, exit_code) = match status {
                Some(status) => (false, status.code()),
                None => (true, None),
            };

            health.push(WorkerHealth {
                role: worker.role,
                pid,
                running,
                exit_code,
            });
        }

        Ok(health)
    }

    pub fn restart_exited_workers(&mut self) -> pd_core::BrowserResult<Vec<WorkerRestart>> {
        let mut restarts = Vec::new();

        for worker in &mut self.workers {
            let old_pid = worker.child.id();
            let exited = worker.child.try_wait().map_err(|error| {
                pd_core::BrowserError::new(
                    "browser.runtime.try_wait_failed",
                    format!(
                        "failed to query {} worker (pid {old_pid}) status: {error}",
                        worker.role.as_str()
                    ),
                )
            })?;

            if exited.is_none() {
                continue;
            }

            let role = worker.role;
            let new_child = spawn_worker_process(&self.launch_config, role)?;
            let new_pid = new_child.id();
            worker.child = new_child;

            restarts.push(WorkerRestart {
                role,
                old_pid,
                new_pid,
            });
        }

        Ok(restarts)
    }

    pub fn shutdown(mut self) -> pd_core::BrowserResult<()> {
        for worker in &mut self.workers {
            let _ = worker.child.kill();
            let _ = worker.child.wait();
        }
        Ok(())
    }
}

impl Browser {
    pub fn new() -> pd_core::BrowserResult<Self> {
        let security = SecurityPolicy::default();
        security.validate()?;

        let privacy = PrivacyPolicy::default();
        let storage =
            StorageManager::new(StorageConfig::default(), privacy.clone(), security.clone())
                .with_persistent_root(default_storage_root());
        let network = NetStack::new(privacy.clone(), security.clone(), storage.clone());

        Ok(Self {
            security,
            privacy,
            storage,
            network,
            renderer: RendererProcess::default(),
        })
    }

    pub fn boot(&self) -> pd_core::BrowserResult<BrowserSummary> {
        let channels = hardened_channels()?;

        Ok(BrowserSummary {
            process_channels: channels.len(),
            privacy_hardened: self.privacy.block_third_party_cookies
                && self.privacy.block_known_trackers
                && self.privacy.fingerprinting_resistance,
            security_hardened: self.security.enforce_site_isolation
                && self.security.enforce_strict_tls
                && self.security.sandbox_renderer,
        })
    }

    /// Spawns renderer/network/storage worker processes for a process-isolated runtime.
    pub fn boot_with_runtime(
        &self,
        config: &RuntimeLaunchConfig,
    ) -> pd_core::BrowserResult<BrowserRuntime> {
        let channels = hardened_channels()?;
        let mut workers = Vec::new();

        for role in [
            ProcessRole::Renderer,
            ProcessRole::Network,
            ProcessRole::Storage,
        ] {
            let child = spawn_worker_process(config, role)?;
            workers.push(WorkerProcess { role, child });
        }

        Ok(BrowserRuntime {
            workers,
            channels,
            launch_config: config.clone(),
        })
    }
}

fn default_storage_root() -> PathBuf {
    if let Some(override_root) = std::env::var_os("PIXELDUST_STORAGE_DIR") {
        return PathBuf::from(override_root);
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".pixeldust")
}

fn hardened_channels() -> pd_core::BrowserResult<Vec<ChannelConfig>> {
    Ok(vec![
        ChannelConfig::hardened(ProcessRole::Browser)?,
        ChannelConfig::hardened(ProcessRole::Renderer)?,
        ChannelConfig::hardened(ProcessRole::Network)?,
        ChannelConfig::hardened(ProcessRole::Storage)?,
    ])
}

fn worker_command_args(extra_args: &[String], role: ProcessRole) -> Vec<String> {
    let mut args = Vec::with_capacity(extra_args.len() + 2);
    args.extend(extra_args.iter().cloned());
    args.push("--pd-role".to_owned());
    args.push(role.as_str().to_owned());
    args
}

fn spawn_worker_process(
    config: &RuntimeLaunchConfig,
    role: ProcessRole,
) -> pd_core::BrowserResult<Child> {
    if config.executable.as_os_str().is_empty() {
        return Err(pd_core::BrowserError::new(
            "browser.runtime.executable_missing",
            "runtime executable path is empty",
        ));
    }

    let mut command = Command::new(&config.executable);
    for arg in worker_command_args(&config.extra_args, role) {
        command.arg(arg);
    }

    if config.inherit_stdio {
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    } else {
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    command.spawn().map_err(|error| {
        pd_core::BrowserError::new(
            "browser.runtime.spawn_failed",
            format!(
                "failed to spawn {} worker from `{}`: {error}",
                role.as_str(),
                config.executable.display()
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::Browser;
    use super::worker_command_args;
    use pd_ipc::ProcessRole;

    #[test]
    fn boot_reports_hardened_defaults() {
        let browser = Browser::new();
        assert!(browser.is_ok());
        let summary = browser.and_then(|browser| browser.boot());
        assert!(summary.is_ok());
        let summary = summary.unwrap_or_else(|_| unreachable!());
        assert!(summary.process_channels >= 4);
        assert!(summary.privacy_hardened);
        assert!(summary.security_hardened);
    }

    #[test]
    fn worker_args_include_role() {
        let args = worker_command_args(
            &["--headless".to_owned(), "--log-level=warn".to_owned()],
            ProcessRole::Renderer,
        );
        assert_eq!(
            args,
            vec![
                "--headless".to_owned(),
                "--log-level=warn".to_owned(),
                "--pd-role".to_owned(),
                "renderer".to_owned()
            ]
        );
    }
}
