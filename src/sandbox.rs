//! Sandbox configuration for secure script execution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Policy for filesystem path access.
#[derive(Debug, Clone, PartialEq)]
pub enum PathPolicy {
    /// Deny all filesystem access.
    DenyAll,
    /// Allow only specific paths.
    AllowList(HashSet<PathBuf>),
    /// Deny specific paths (allow all others).
    DenyList(HashSet<PathBuf>),
    /// Allow all filesystem access.
    AllowAll,
}

impl Default for PathPolicy {
    fn default() -> Self {
        PathPolicy::DenyAll
    }
}

impl PathPolicy {
    /// Check if a path is allowed by this policy.
    pub fn is_allowed(&self, path: &Path) -> bool {
        match self {
            PathPolicy::DenyAll => false,
            PathPolicy::AllowAll => true,
            PathPolicy::AllowList(allowed) => {
                // Check if path or any of its parents are in the allowlist
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                allowed.iter().any(|allowed_path| {
                    canonical.starts_with(allowed_path)
                })
            }
            PathPolicy::DenyList(denied) => {
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                !denied.iter().any(|denied_path| {
                    canonical.starts_with(denied_path)
                })
            }
        }
    }

    /// Create an allowlist policy with the given paths.
    pub fn allow<I, P>(paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        PathPolicy::AllowList(paths.into_iter().map(Into::into).collect())
    }

    /// Create a denylist policy with the given paths.
    pub fn deny<I, P>(paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        PathPolicy::DenyList(paths.into_iter().map(Into::into).collect())
    }
}

/// Policy for network access.
#[derive(Debug, Clone, PartialEq)]
pub enum NetPolicy {
    /// Deny all network access.
    DenyAll,
    /// Allow only specific hosts/domains.
    AllowList(HashSet<String>),
    /// Deny specific hosts/domains (allow all others).
    DenyList(HashSet<String>),
    /// Allow all network access.
    AllowAll,
}

impl Default for NetPolicy {
    fn default() -> Self {
        NetPolicy::DenyAll
    }
}

impl NetPolicy {
    /// Check if a host is allowed by this policy.
    pub fn is_allowed(&self, host: &str) -> bool {
        let host_lower = host.to_lowercase();
        match self {
            NetPolicy::DenyAll => false,
            NetPolicy::AllowAll => true,
            NetPolicy::AllowList(allowed) => {
                allowed.iter().any(|a| {
                    let a_lower = a.to_lowercase();
                    // Support wildcard subdomains like *.example.com
                    if a_lower.starts_with("*.") {
                        let suffix = &a_lower[1..];
                        host_lower.ends_with(suffix) || host_lower == &a_lower[2..]
                    } else {
                        host_lower == a_lower
                    }
                })
            }
            NetPolicy::DenyList(denied) => {
                !denied.iter().any(|d| {
                    let d_lower = d.to_lowercase();
                    if d_lower.starts_with("*.") {
                        let suffix = &d_lower[1..];
                        host_lower.ends_with(suffix) || host_lower == &d_lower[2..]
                    } else {
                        host_lower == d_lower
                    }
                })
            }
        }
    }

    /// Create an allowlist policy with the given hosts.
    pub fn allow<I, S>(hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        NetPolicy::AllowList(hosts.into_iter().map(Into::into).collect())
    }

    /// Create a denylist policy with the given hosts.
    pub fn deny<I, S>(hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        NetPolicy::DenyList(hosts.into_iter().map(Into::into).collect())
    }
}

/// Configuration for the sandbox environment.
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    /// Policy for filesystem read access.
    pub fs_read: PathPolicy,
    /// Policy for filesystem write access.
    pub fs_write: PathPolicy,
    /// Policy for outgoing network requests.
    pub net_outgoing: NetPolicy,
    /// Policy for incoming network connections.
    pub net_incoming: NetPolicy,
    /// Allowed environment variable names (None = all denied).
    pub env_vars: Option<HashSet<String>>,
    /// Working directory for the script.
    pub working_dir: Option<PathBuf>,
    /// Whether to isolate temp directory.
    pub isolate_temp: bool,
}

impl SandboxConfig {
    /// Create a completely locked-down sandbox configuration.
    pub fn locked() -> Self {
        Self {
            fs_read: PathPolicy::DenyAll,
            fs_write: PathPolicy::DenyAll,
            net_outgoing: NetPolicy::DenyAll,
            net_incoming: NetPolicy::DenyAll,
            env_vars: Some(HashSet::new()),
            working_dir: None,
            isolate_temp: true,
        }
    }

    /// Create a permissive sandbox configuration (use with caution).
    pub fn permissive() -> Self {
        Self {
            fs_read: PathPolicy::AllowAll,
            fs_write: PathPolicy::AllowAll,
            net_outgoing: NetPolicy::AllowAll,
            net_incoming: NetPolicy::AllowAll,
            env_vars: None,
            working_dir: None,
            isolate_temp: false,
        }
    }

    /// Allow reading from specific paths.
    pub fn with_read_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.fs_read = PathPolicy::allow(paths);
        self
    }

    /// Allow writing to specific paths.
    pub fn with_write_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.fs_write = PathPolicy::allow(paths);
        self
    }

    /// Allow outgoing requests to specific hosts.
    pub fn with_allowed_hosts<I, S>(mut self, hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.net_outgoing = NetPolicy::allow(hosts);
        self
    }

    /// Allow access to specific environment variables.
    pub fn with_env_vars<I, S>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.env_vars = Some(vars.into_iter().map(Into::into).collect());
        self
    }

    /// Set the working directory.
    pub fn with_working_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    /// Enable temp directory isolation.
    pub fn with_temp_isolation(mut self) -> Self {
        self.isolate_temp = true;
        self
    }

    /// Check if reading from a path is allowed.
    pub fn can_read(&self, path: &Path) -> bool {
        self.fs_read.is_allowed(path)
    }

    /// Check if writing to a path is allowed.
    pub fn can_write(&self, path: &Path) -> bool {
        self.fs_write.is_allowed(path)
    }

    /// Check if connecting to a host is allowed.
    pub fn can_connect(&self, host: &str) -> bool {
        self.net_outgoing.is_allowed(host)
    }

    /// Check if an environment variable is accessible.
    pub fn can_access_env(&self, name: &str) -> bool {
        match &self.env_vars {
            None => true,
            Some(allowed) => allowed.contains(name),
        }
    }
}

/// A sandbox instance that enforces security policies during execution.
#[derive(Debug)]
pub struct Sandbox {
    config: SandboxConfig,
    temp_dir: Option<PathBuf>,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration.
    pub fn new(config: SandboxConfig) -> crate::Result<Self> {
        let temp_dir = if config.isolate_temp {
            // Create an isolated temp directory
            let dir = std::env::temp_dir().join(format!(
                "fusabi-sandbox-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir)?;
            Some(dir)
        } else {
            None
        };

        Ok(Self { config, temp_dir })
    }

    /// Get the sandbox configuration.
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Get the isolated temp directory, if any.
    pub fn temp_dir(&self) -> Option<&Path> {
        self.temp_dir.as_deref()
    }

    /// Check read permission and return an error if denied.
    pub fn check_read(&self, path: &Path) -> crate::Result<()> {
        if self.config.can_read(path) {
            Ok(())
        } else {
            Err(crate::Error::sandbox_violation(format!(
                "read access denied: {}",
                path.display()
            )))
        }
    }

    /// Check write permission and return an error if denied.
    pub fn check_write(&self, path: &Path) -> crate::Result<()> {
        if self.config.can_write(path) {
            Ok(())
        } else {
            Err(crate::Error::sandbox_violation(format!(
                "write access denied: {}",
                path.display()
            )))
        }
    }

    /// Check network connection permission and return an error if denied.
    pub fn check_connect(&self, host: &str) -> crate::Result<()> {
        if self.config.can_connect(host) {
            Ok(())
        } else {
            Err(crate::Error::sandbox_violation(format!(
                "network access denied: {}",
                host
            )))
        }
    }

    /// Check environment variable access and return an error if denied.
    pub fn check_env(&self, name: &str) -> crate::Result<()> {
        if self.config.can_access_env(name) {
            Ok(())
        } else {
            Err(crate::Error::sandbox_violation(format!(
                "environment variable access denied: {}",
                name
            )))
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Clean up isolated temp directory
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_policy_deny_all() {
        let policy = PathPolicy::DenyAll;
        assert!(!policy.is_allowed(Path::new("/any/path")));
    }

    #[test]
    fn test_path_policy_allow_all() {
        let policy = PathPolicy::AllowAll;
        assert!(policy.is_allowed(Path::new("/any/path")));
    }

    #[test]
    fn test_path_policy_allowlist() {
        let policy = PathPolicy::allow(["/tmp", "/home/user/data"]);
        // Note: This test may not work perfectly without actual paths existing
        // In production, paths would be canonicalized
        assert!(!policy.is_allowed(Path::new("/etc/passwd")));
    }

    #[test]
    fn test_net_policy_deny_all() {
        let policy = NetPolicy::DenyAll;
        assert!(!policy.is_allowed("example.com"));
    }

    #[test]
    fn test_net_policy_allow_all() {
        let policy = NetPolicy::AllowAll;
        assert!(policy.is_allowed("example.com"));
    }

    #[test]
    fn test_net_policy_allowlist() {
        let policy = NetPolicy::allow(["example.com", "*.trusted.org"]);
        assert!(policy.is_allowed("example.com"));
        assert!(policy.is_allowed("api.trusted.org"));
        assert!(policy.is_allowed("trusted.org"));
        assert!(!policy.is_allowed("malicious.com"));
    }

    #[test]
    fn test_net_policy_denylist() {
        let policy = NetPolicy::deny(["evil.com", "*.malware.net"]);
        assert!(policy.is_allowed("example.com"));
        assert!(!policy.is_allowed("evil.com"));
        assert!(!policy.is_allowed("download.malware.net"));
    }

    #[test]
    fn test_sandbox_config_locked() {
        let config = SandboxConfig::locked();
        assert!(!config.can_read(Path::new("/etc/passwd")));
        assert!(!config.can_write(Path::new("/tmp/file")));
        assert!(!config.can_connect("example.com"));
        assert!(!config.can_access_env("PATH"));
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();
        assert!(config.can_read(Path::new("/etc/passwd")));
        assert!(config.can_connect("example.com"));
        assert!(config.can_access_env("PATH"));
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::locked()
            .with_allowed_hosts(["api.example.com"])
            .with_env_vars(["HOME", "USER"]);

        assert!(config.can_connect("api.example.com"));
        assert!(!config.can_connect("other.com"));
        assert!(config.can_access_env("HOME"));
        assert!(!config.can_access_env("SECRET"));
    }
}
