# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- `compile_source`/`compile_file` now produce real Fusabi bytecode by invoking the
  `fusabi-frontend` compiler and serializing the resulting VM chunk (FZB container),
  instead of emitting placeholder bytes.
- `Engine::execute_bytecode` now executes compiled bytecode on the real `fusabi-vm`
  interpreter and returns the program's actual value, instead of always returning
  `Value::Null`. Fixes #12.
- `validate_bytecode` now validates the real FZB bytecode container via the VM
  deserializer.

## [0.1.0] - 2025-12-04

### Added
- Initial release of `fusabi-host`.
- Added `EnginePool` for concurrent script execution.
- Added `Value` and `FromValue`/`IntoValue` traits for Serde-compatible type conversion.
- Added `Sandbox` configuration for resource limits and capabilities.
- Added `compile_source` and `compile_file` helper functions.
