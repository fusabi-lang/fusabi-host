//! Capability flags for script permissions.

use std::collections::HashSet;

/// Individual capability that can be granted to scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Capability {
    /// Read from the filesystem.
    FsRead,
    /// Write to the filesystem.
    FsWrite,
    /// Execute filesystem operations (list, delete, etc.).
    FsExecute,
    /// Make network requests.
    NetRequest,
    /// Listen on network ports.
    NetListen,
    /// Execute system processes.
    ProcessExec,
    /// Access environment variables.
    EnvRead,
    /// Modify environment variables.
    EnvWrite,
    /// Access system time.
    TimeRead,
    /// Access random number generation.
    Random,
    /// Access to standard input.
    StdinRead,
    /// Access to standard output.
    StdoutWrite,
    /// Access to standard error.
    StderrWrite,
    /// Access to metrics/observability APIs.
    Metrics,
    /// Access to logging APIs.
    Logging,
    /// Ability to spawn async tasks.
    AsyncSpawn,
    /// Access to cryptographic operations.
    Crypto,
    /// Access to serialization (JSON, etc.).
    Serialize,
}

impl Capability {
    /// Get the string name of this capability.
    pub fn name(&self) -> &'static str {
        match self {
            Capability::FsRead => "fs:read",
            Capability::FsWrite => "fs:write",
            Capability::FsExecute => "fs:execute",
            Capability::NetRequest => "net:request",
            Capability::NetListen => "net:listen",
            Capability::ProcessExec => "process:exec",
            Capability::EnvRead => "env:read",
            Capability::EnvWrite => "env:write",
            Capability::TimeRead => "time:read",
            Capability::Random => "random",
            Capability::StdinRead => "stdin:read",
            Capability::StdoutWrite => "stdout:write",
            Capability::StderrWrite => "stderr:write",
            Capability::Metrics => "metrics",
            Capability::Logging => "logging",
            Capability::AsyncSpawn => "async:spawn",
            Capability::Crypto => "crypto",
            Capability::Serialize => "serialize",
        }
    }

    /// Parse a capability from a string name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "fs:read" => Some(Capability::FsRead),
            "fs:write" => Some(Capability::FsWrite),
            "fs:execute" => Some(Capability::FsExecute),
            "net:request" => Some(Capability::NetRequest),
            "net:listen" => Some(Capability::NetListen),
            "process:exec" => Some(Capability::ProcessExec),
            "env:read" => Some(Capability::EnvRead),
            "env:write" => Some(Capability::EnvWrite),
            "time:read" => Some(Capability::TimeRead),
            "random" => Some(Capability::Random),
            "stdin:read" => Some(Capability::StdinRead),
            "stdout:write" => Some(Capability::StdoutWrite),
            "stderr:write" => Some(Capability::StderrWrite),
            "metrics" => Some(Capability::Metrics),
            "logging" => Some(Capability::Logging),
            "async:spawn" => Some(Capability::AsyncSpawn),
            "crypto" => Some(Capability::Crypto),
            "serialize" => Some(Capability::Serialize),
            _ => None,
        }
    }

    /// Returns true if this is a dangerous capability.
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Capability::FsWrite
                | Capability::ProcessExec
                | Capability::NetListen
                | Capability::EnvWrite
        )
    }

    /// Get all available capabilities.
    pub fn all() -> &'static [Capability] {
        &[
            Capability::FsRead,
            Capability::FsWrite,
            Capability::FsExecute,
            Capability::NetRequest,
            Capability::NetListen,
            Capability::ProcessExec,
            Capability::EnvRead,
            Capability::EnvWrite,
            Capability::TimeRead,
            Capability::Random,
            Capability::StdinRead,
            Capability::StdoutWrite,
            Capability::StderrWrite,
            Capability::Metrics,
            Capability::Logging,
            Capability::AsyncSpawn,
            Capability::Crypto,
            Capability::Serialize,
        ]
    }
}

/// A set of capabilities granted to a script.
#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    granted: HashSet<Capability>,
}

impl Capabilities {
    /// Create an empty capability set (no permissions).
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a capability set with all permissions.
    pub fn all() -> Self {
        Self {
            granted: Capability::all().iter().copied().collect(),
        }
    }

    /// Create a safe default capability set.
    ///
    /// Includes: TimeRead, Random, StdoutWrite, StderrWrite, Logging, Serialize
    pub fn safe_defaults() -> Self {
        Self::none()
            .with(Capability::TimeRead)
            .with(Capability::Random)
            .with(Capability::StdoutWrite)
            .with(Capability::StderrWrite)
            .with(Capability::Logging)
            .with(Capability::Serialize)
    }

    /// Add a capability.
    pub fn with(mut self, cap: Capability) -> Self {
        self.granted.insert(cap);
        self
    }

    /// Add multiple capabilities.
    pub fn with_all<I: IntoIterator<Item = Capability>>(mut self, caps: I) -> Self {
        self.granted.extend(caps);
        self
    }

    /// Remove a capability.
    pub fn without(mut self, cap: Capability) -> Self {
        self.granted.remove(&cap);
        self
    }

    /// Grant a capability (mutating version).
    pub fn grant(&mut self, cap: Capability) {
        self.granted.insert(cap);
    }

    /// Revoke a capability (mutating version).
    pub fn revoke(&mut self, cap: Capability) {
        self.granted.remove(&cap);
    }

    /// Check if a capability is granted.
    pub fn has(&self, cap: Capability) -> bool {
        self.granted.contains(&cap)
    }

    /// Check if a capability is granted, returning an error if not.
    pub fn require(&self, cap: Capability) -> crate::Result<()> {
        if self.has(cap) {
            Ok(())
        } else {
            Err(crate::Error::capability_denied(cap.name()))
        }
    }

    /// Get all granted capabilities.
    pub fn granted(&self) -> impl Iterator<Item = &Capability> {
        self.granted.iter()
    }

    /// Get the number of granted capabilities.
    pub fn len(&self) -> usize {
        self.granted.len()
    }

    /// Check if no capabilities are granted.
    pub fn is_empty(&self) -> bool {
        self.granted.is_empty()
    }

    /// Check if any dangerous capability is granted.
    pub fn has_dangerous(&self) -> bool {
        self.granted.iter().any(|c| c.is_dangerous())
    }

    /// Parse capabilities from string names.
    pub fn from_names<'a, I: IntoIterator<Item = &'a str>>(names: I) -> Self {
        let granted = names
            .into_iter()
            .filter_map(Capability::from_name)
            .collect();
        Self { granted }
    }

    /// Get capability names as strings.
    pub fn to_names(&self) -> Vec<&'static str> {
        self.granted.iter().map(|c| c.name()).collect()
    }

    /// Merge with another capability set.
    pub fn merge(&self, other: &Capabilities) -> Capabilities {
        let granted = self.granted.union(&other.granted).copied().collect();
        Capabilities { granted }
    }

    /// Intersect with another capability set.
    pub fn intersect(&self, other: &Capabilities) -> Capabilities {
        let granted = self.granted.intersection(&other.granted).copied().collect();
        Capabilities { granted }
    }
}

impl FromIterator<Capability> for Capabilities {
    fn from_iter<I: IntoIterator<Item = Capability>>(iter: I) -> Self {
        Self {
            granted: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_name_roundtrip() {
        for cap in Capability::all() {
            let name = cap.name();
            let parsed = Capability::from_name(name);
            assert_eq!(parsed, Some(*cap), "Failed roundtrip for {:?}", cap);
        }
    }

    #[test]
    fn test_capabilities_none() {
        let caps = Capabilities::none();
        assert!(caps.is_empty());
        assert!(!caps.has(Capability::FsRead));
    }

    #[test]
    fn test_capabilities_all() {
        let caps = Capabilities::all();
        assert_eq!(caps.len(), Capability::all().len());
        assert!(caps.has(Capability::FsRead));
        assert!(caps.has(Capability::NetRequest));
    }

    #[test]
    fn test_capabilities_safe_defaults() {
        let caps = Capabilities::safe_defaults();
        assert!(caps.has(Capability::TimeRead));
        assert!(caps.has(Capability::Logging));
        assert!(!caps.has(Capability::FsWrite));
        assert!(!caps.has(Capability::ProcessExec));
    }

    #[test]
    fn test_capabilities_builder() {
        let caps = Capabilities::none()
            .with(Capability::FsRead)
            .with(Capability::NetRequest)
            .without(Capability::FsRead);

        assert!(!caps.has(Capability::FsRead));
        assert!(caps.has(Capability::NetRequest));
    }

    #[test]
    fn test_capabilities_require() {
        let caps = Capabilities::none().with(Capability::FsRead);

        assert!(caps.require(Capability::FsRead).is_ok());
        assert!(caps.require(Capability::FsWrite).is_err());
    }

    #[test]
    fn test_capabilities_from_names() {
        let caps = Capabilities::from_names(["fs:read", "net:request", "invalid"]);
        assert!(caps.has(Capability::FsRead));
        assert!(caps.has(Capability::NetRequest));
        assert_eq!(caps.len(), 2);
    }

    #[test]
    fn test_dangerous_capabilities() {
        assert!(Capability::FsWrite.is_dangerous());
        assert!(Capability::ProcessExec.is_dangerous());
        assert!(!Capability::FsRead.is_dangerous());
        assert!(!Capability::TimeRead.is_dangerous());

        let caps = Capabilities::none().with(Capability::FsWrite);
        assert!(caps.has_dangerous());

        let safe = Capabilities::safe_defaults();
        assert!(!safe.has_dangerous());
    }

    #[test]
    fn test_capabilities_merge() {
        let a = Capabilities::none().with(Capability::FsRead);
        let b = Capabilities::none().with(Capability::NetRequest);
        let merged = a.merge(&b);

        assert!(merged.has(Capability::FsRead));
        assert!(merged.has(Capability::NetRequest));
    }

    #[test]
    fn test_capabilities_intersect() {
        let a = Capabilities::none()
            .with(Capability::FsRead)
            .with(Capability::NetRequest);
        let b = Capabilities::none()
            .with(Capability::NetRequest)
            .with(Capability::TimeRead);
        let intersected = a.intersect(&b);

        assert!(!intersected.has(Capability::FsRead));
        assert!(intersected.has(Capability::NetRequest));
        assert!(!intersected.has(Capability::TimeRead));
    }
}
