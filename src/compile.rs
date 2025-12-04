//! Compilation APIs for Fusabi source and bytecode.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

/// Options for compilation.
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// Optimization level (0-3).
    pub opt_level: u8,
    /// Whether to include debug information.
    pub debug_info: bool,
    /// Whether to strip symbols.
    pub strip: bool,
    /// Target Fusabi version.
    pub target_version: Option<String>,
    /// Custom compiler flags.
    pub flags: HashMap<String, String>,
    /// Source file name (for error messages).
    pub source_name: Option<String>,
}

impl CompileOptions {
    /// Create new compile options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set optimization level.
    pub fn with_opt_level(mut self, level: u8) -> Self {
        self.opt_level = level.min(3);
        self
    }

    /// Enable debug information.
    pub fn with_debug_info(mut self) -> Self {
        self.debug_info = true;
        self
    }

    /// Enable symbol stripping.
    pub fn with_strip(mut self) -> Self {
        self.strip = true;
        self
    }

    /// Set target version.
    pub fn with_target_version(mut self, version: impl Into<String>) -> Self {
        self.target_version = Some(version.into());
        self
    }

    /// Add a custom flag.
    pub fn with_flag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.flags.insert(key.into(), value.into());
        self
    }

    /// Set source name for error messages.
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// Create options optimized for development.
    pub fn development() -> Self {
        Self {
            opt_level: 0,
            debug_info: true,
            strip: false,
            target_version: None,
            flags: HashMap::new(),
            source_name: None,
        }
    }

    /// Create options optimized for production.
    pub fn production() -> Self {
        Self {
            opt_level: 2,
            debug_info: false,
            strip: true,
            target_version: None,
            flags: HashMap::new(),
            source_name: None,
        }
    }
}

/// Metadata extracted from compiled bytecode.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// Fusabi language version used.
    pub language_version: String,
    /// Compiler version.
    pub compiler_version: String,
    /// Original source name.
    pub source_name: Option<String>,
    /// Compilation timestamp.
    pub compiled_at: Option<u64>,
    /// Required capabilities declared in the script.
    pub required_capabilities: Vec<String>,
    /// Exported functions.
    pub exports: Vec<ExportInfo>,
    /// Imported modules.
    pub imports: Vec<ImportInfo>,
    /// Custom metadata entries.
    pub custom: HashMap<String, String>,
}

/// Information about an exported function.
#[derive(Debug, Clone)]
pub struct ExportInfo {
    /// Function name.
    pub name: String,
    /// Parameter count.
    pub param_count: usize,
    /// Whether the function is async.
    pub is_async: bool,
    /// Documentation comment if available.
    pub doc: Option<String>,
}

/// Information about an imported module.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Module name.
    pub module: String,
    /// Imported items (or "*" for all).
    pub items: Vec<String>,
    /// Version constraint if specified.
    pub version: Option<String>,
}

impl Metadata {
    /// Check if a capability is required.
    pub fn requires_capability(&self, cap: &str) -> bool {
        self.required_capabilities.iter().any(|c| c == cap)
    }

    /// Get an export by name.
    pub fn get_export(&self, name: &str) -> Option<&ExportInfo> {
        self.exports.iter().find(|e| e.name == name)
    }

    /// Check if a module is imported.
    pub fn imports_module(&self, module: &str) -> bool {
        self.imports.iter().any(|i| i.module == module)
    }
}

/// Result of compilation.
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// Compiled bytecode.
    pub bytecode: Vec<u8>,
    /// Extracted metadata.
    pub metadata: Metadata,
    /// Compilation warnings.
    pub warnings: Vec<CompileWarning>,
    /// Compilation statistics.
    pub stats: CompileStats,
}

/// A compilation warning.
#[derive(Debug, Clone)]
pub struct CompileWarning {
    /// Warning message.
    pub message: String,
    /// Source location if available.
    pub location: Option<SourceLocation>,
    /// Warning code.
    pub code: Option<String>,
}

/// Source location for diagnostics.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Source file name.
    pub file: Option<String>,
}

/// Statistics about compilation.
#[derive(Debug, Clone, Default)]
pub struct CompileStats {
    /// Source size in bytes.
    pub source_bytes: usize,
    /// Bytecode size in bytes.
    pub bytecode_bytes: usize,
    /// Number of functions.
    pub function_count: usize,
    /// Compilation time in milliseconds.
    pub compile_time_ms: u64,
}

/// Compile Fusabi source code to bytecode.
///
/// # Arguments
///
/// * `source` - The Fusabi source code
/// * `options` - Compilation options
///
/// # Returns
///
/// A `CompileResult` containing bytecode, metadata, and diagnostics.
pub fn compile_source(source: &str, options: &CompileOptions) -> Result<CompileResult> {
    let start = std::time::Instant::now();

    // Validate source isn't empty
    if source.trim().is_empty() {
        return Err(Error::compilation("empty source"));
    }

    // Simulate compilation - in real implementation would call fusabi_frontend
    let bytecode = generate_bytecode(source, options)?;
    let metadata = extract_metadata(source, options);
    let warnings = check_warnings(source);

    let compile_time = start.elapsed();

    Ok(CompileResult {
        bytecode: bytecode.clone(),
        metadata,
        warnings,
        stats: CompileStats {
            source_bytes: source.len(),
            bytecode_bytes: bytecode.len(),
            function_count: 1,
            compile_time_ms: compile_time.as_millis() as u64,
        },
    })
}

/// Compile a Fusabi source file to bytecode.
///
/// # Arguments
///
/// * `path` - Path to the source file (.fsx)
/// * `options` - Compilation options
///
/// # Returns
///
/// A `CompileResult` containing bytecode, metadata, and diagnostics.
pub fn compile_file(path: &Path, options: &CompileOptions) -> Result<CompileResult> {
    // Check file extension
    let extension = path.extension().and_then(|e| e.to_str());
    if extension != Some("fsx") && extension != Some("fusabi") {
        return Err(Error::compilation(format!(
            "expected .fsx or .fusabi file, got: {}",
            path.display()
        )));
    }

    // Read source
    let source = std::fs::read_to_string(path)?;

    // Compile with source name
    let options = options
        .clone()
        .with_source_name(path.display().to_string());

    compile_source(&source, &options)
}

/// Validate bytecode without executing.
///
/// # Arguments
///
/// * `bytecode` - The bytecode to validate
///
/// # Returns
///
/// Metadata if valid, error if invalid.
pub fn validate_bytecode(bytecode: &[u8]) -> Result<Metadata> {
    // Check minimum size
    if bytecode.len() < 16 {
        return Err(Error::invalid_bytecode("bytecode too short"));
    }

    // Check magic number
    if &bytecode[0..4] != b"FZB\x00" {
        return Err(Error::invalid_bytecode("invalid magic number"));
    }

    // Check version
    let version = bytecode[4];
    if version > 1 {
        return Err(Error::invalid_bytecode(format!(
            "unsupported bytecode version: {}",
            version
        )));
    }

    // Extract metadata from bytecode
    Ok(Metadata {
        language_version: "0.18.0".to_string(),
        compiler_version: "0.18.0".to_string(),
        source_name: None,
        compiled_at: None,
        required_capabilities: Vec::new(),
        exports: Vec::new(),
        imports: Vec::new(),
        custom: HashMap::new(),
    })
}

/// Extract metadata from existing bytecode.
pub fn extract_bytecode_metadata(bytecode: &[u8]) -> Result<Metadata> {
    validate_bytecode(bytecode)
}

// Internal helper functions

fn generate_bytecode(source: &str, options: &CompileOptions) -> Result<Vec<u8>> {
    // Generate fake bytecode for simulation
    // Real implementation would use fusabi_frontend

    let mut bytecode = Vec::new();

    // Magic number: "FZB\0"
    bytecode.extend_from_slice(b"FZB\x00");

    // Version byte
    bytecode.push(1);

    // Flags byte
    let mut flags = 0u8;
    if options.debug_info {
        flags |= 0x01;
    }
    if options.strip {
        flags |= 0x02;
    }
    flags |= (options.opt_level & 0x03) << 4;
    bytecode.push(flags);

    // Reserved bytes
    bytecode.extend_from_slice(&[0u8; 10]);

    // Source hash (simplified)
    let hash = simple_hash(source);
    bytecode.extend_from_slice(&hash.to_le_bytes());

    // Placeholder for actual bytecode
    // In real impl, this would be the compiled instructions
    bytecode.extend_from_slice(source.as_bytes());

    Ok(bytecode)
}

fn extract_metadata(source: &str, options: &CompileOptions) -> Metadata {
    let mut metadata = Metadata {
        language_version: "0.18.0".to_string(),
        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        source_name: options.source_name.clone(),
        compiled_at: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        ),
        required_capabilities: Vec::new(),
        exports: Vec::new(),
        imports: Vec::new(),
        custom: HashMap::new(),
    };

    // Parse source for metadata hints (simplified)
    for line in source.lines() {
        let line = line.trim();

        // Check for capability declarations
        if line.starts_with("@require ") {
            let cap = line.trim_start_matches("@require ").trim();
            metadata.required_capabilities.push(cap.to_string());
        }

        // Check for imports
        if line.starts_with("import ") {
            let module = line.trim_start_matches("import ").trim();
            metadata.imports.push(ImportInfo {
                module: module.to_string(),
                items: vec!["*".to_string()],
                version: None,
            });
        }

        // Check for function exports
        if line.starts_with("export fn ") || line.starts_with("pub fn ") {
            let rest = line
                .trim_start_matches("export fn ")
                .trim_start_matches("pub fn ");
            if let Some(paren) = rest.find('(') {
                let name = rest[..paren].trim();
                metadata.exports.push(ExportInfo {
                    name: name.to_string(),
                    param_count: 0, // Would need proper parsing
                    is_async: rest.contains("async"),
                    doc: None,
                });
            }
        }
    }

    metadata
}

fn check_warnings(source: &str) -> Vec<CompileWarning> {
    let mut warnings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check for TODO comments
        if line.contains("TODO") || line.contains("FIXME") {
            warnings.push(CompileWarning {
                message: "unresolved TODO/FIXME comment".to_string(),
                location: Some(SourceLocation {
                    line: line_num + 1,
                    column: 1,
                    file: None,
                }),
                code: Some("W001".to_string()),
            });
        }

        // Check for unused variable hints
        if line.contains("let _") {
            warnings.push(CompileWarning {
                message: "unused variable".to_string(),
                location: Some(SourceLocation {
                    line: line_num + 1,
                    column: 1,
                    file: None,
                }),
                code: Some("W002".to_string()),
            });
        }
    }

    warnings
}

fn simple_hash(s: &str) -> u64 {
    // Simple FNV-1a hash for simulation
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_source() {
        let result = compile_source("42", &CompileOptions::default()).unwrap();

        assert!(!result.bytecode.is_empty());
        assert!(result.bytecode.starts_with(b"FZB\x00"));
        assert_eq!(result.stats.source_bytes, 2);
    }

    #[test]
    fn test_compile_empty_source() {
        let result = compile_source("", &CompileOptions::default());
        assert!(matches!(result, Err(Error::Compilation(_))));

        let result = compile_source("   ", &CompileOptions::default());
        assert!(matches!(result, Err(Error::Compilation(_))));
    }

    #[test]
    fn test_compile_options_builder() {
        let opts = CompileOptions::new()
            .with_opt_level(2)
            .with_debug_info()
            .with_source_name("test.fsx");

        assert_eq!(opts.opt_level, 2);
        assert!(opts.debug_info);
        assert_eq!(opts.source_name, Some("test.fsx".to_string()));
    }

    #[test]
    fn test_compile_options_presets() {
        let dev = CompileOptions::development();
        assert_eq!(dev.opt_level, 0);
        assert!(dev.debug_info);

        let prod = CompileOptions::production();
        assert_eq!(prod.opt_level, 2);
        assert!(prod.strip);
    }

    #[test]
    fn test_validate_bytecode() {
        let result = compile_source("42", &CompileOptions::default()).unwrap();
        let metadata = validate_bytecode(&result.bytecode).unwrap();

        assert_eq!(metadata.language_version, "0.18.0");
    }

    #[test]
    fn test_validate_invalid_bytecode() {
        assert!(validate_bytecode(b"invalid").is_err());
        assert!(validate_bytecode(b"FZB").is_err()); // Too short
        assert!(validate_bytecode(b"XXX\x00").is_err()); // Wrong magic
    }

    #[test]
    fn test_metadata_extraction() {
        let source = r#"
@require fs:read
import json

export fn main() {
    // TODO: implement
}
"#;

        let result = compile_source(source, &CompileOptions::default()).unwrap();

        assert!(result.metadata.requires_capability("fs:read"));
        assert!(result.metadata.imports_module("json"));
        assert!(result.metadata.get_export("main").is_some());
    }

    #[test]
    fn test_compile_warnings() {
        let source = "// TODO: fix this";
        let result = compile_source(source, &CompileOptions::default()).unwrap();

        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].message.contains("TODO"));
    }

    #[test]
    fn test_compile_file_wrong_extension() {
        let result = compile_file(Path::new("test.txt"), &CompileOptions::default());
        assert!(matches!(result, Err(Error::Compilation(_))));
    }
}
