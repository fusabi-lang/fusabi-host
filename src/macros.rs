//! Macros for typed host function registration.
//!
//! This module provides macros and helpers for ergonomically registering
//! host functions with automatic value conversion and error handling.

use crate::convert::{FromValue, IntoValue, ValueConversionError};
use crate::engine::ExecutionContext;
use crate::error::{Error, Result};
use crate::value::Value;

/// Trait for types that can be used as host function arguments.
pub trait HostArg: Sized {
    /// Extract this argument from the argument list at the given position.
    fn extract(args: &[Value], position: usize) -> std::result::Result<Self, ArgError>;

    /// Whether this argument is optional (has a default).
    fn is_optional() -> bool {
        false
    }
}

/// Error extracting a host function argument.
#[derive(Debug, Clone)]
pub struct ArgError {
    /// Argument position (0-indexed).
    pub position: usize,
    /// Expected type.
    pub expected: &'static str,
    /// What went wrong.
    pub message: String,
}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "argument {}: expected {}, {}",
            self.position, self.expected, self.message
        )
    }
}

impl std::error::Error for ArgError {}

impl From<ArgError> for Error {
    fn from(err: ArgError) -> Self {
        Error::HostFunction(err.to_string())
    }
}

// Implement HostArg for common types

impl<T: FromValue> HostArg for T {
    fn extract(args: &[Value], position: usize) -> std::result::Result<Self, ArgError> {
        let value = args.get(position).cloned().ok_or_else(|| ArgError {
            position,
            expected: std::any::type_name::<T>(),
            message: "missing argument".to_string(),
        })?;

        T::from_value(value).map_err(|e| ArgError {
            position,
            expected: std::any::type_name::<T>(),
            message: e.to_string(),
        })
    }
}

/// Wrapper for optional arguments.
pub struct Optional<T>(pub Option<T>);

impl<T: FromValue> HostArg for Optional<T> {
    fn extract(args: &[Value], position: usize) -> std::result::Result<Self, ArgError> {
        match args.get(position) {
            None | Some(Value::Null) => Ok(Optional(None)),
            Some(value) => {
                let converted = T::from_value(value.clone()).map_err(|e| ArgError {
                    position,
                    expected: std::any::type_name::<T>(),
                    message: e.to_string(),
                })?;
                Ok(Optional(Some(converted)))
            }
        }
    }

    fn is_optional() -> bool {
        true
    }
}

/// Wrapper for variadic arguments (rest parameters).
pub struct Rest<T>(pub Vec<T>);

impl<T: FromValue> HostArg for Rest<T> {
    fn extract(args: &[Value], position: usize) -> std::result::Result<Self, ArgError> {
        let rest: std::result::Result<Vec<T>, _> = args[position..]
            .iter()
            .enumerate()
            .map(|(i, v)| {
                T::from_value(v.clone()).map_err(|e| ArgError {
                    position: position + i,
                    expected: std::any::type_name::<T>(),
                    message: e.to_string(),
                })
            })
            .collect();
        rest.map(Rest)
    }

    fn is_optional() -> bool {
        true
    }
}

/// Trait for host function return types.
pub trait HostReturn {
    /// Convert this return value to a Result<Value>.
    fn into_result(self) -> Result<Value>;
}

// Value implements IntoValue, so the generic impl below handles it
impl<T: IntoValue> HostReturn for T {
    fn into_result(self) -> Result<Value> {
        Ok(self.into_value())
    }
}

impl<T: IntoValue, E: std::fmt::Display> HostReturn for std::result::Result<T, E> {
    fn into_result(self) -> Result<Value> {
        match self {
            Ok(v) => Ok(v.into_value()),
            Err(e) => Err(Error::HostFunction(e.to_string())),
        }
    }
}

/// Helper to wrap a Rust function as a host function.
///
/// This is used by the `host_fn!` macro but can also be used directly.
pub fn wrap_host_fn<F, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(&[Value], &ExecutionContext) -> R,
    R: HostReturn,
{
    move |args: &[Value], ctx: &ExecutionContext| f(args, ctx).into_result()
}

/// Helper to create a host function with automatic argument extraction.
///
/// # Example
///
/// ```ignore
/// let add = typed_host_fn(|a: i64, b: i64| -> i64 {
///     a + b
/// });
/// ```
pub fn typed_host_fn_0<F, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn() -> R,
    R: HostReturn,
{
    move |_args: &[Value], _ctx: &ExecutionContext| f().into_result()
}

/// Typed host function with 1 argument.
pub fn typed_host_fn_1<F, A, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(A) -> R,
    A: HostArg,
    R: HostReturn,
{
    move |args: &[Value], _ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        f(a).into_result()
    }
}

/// Typed host function with 2 arguments.
pub fn typed_host_fn_2<F, A, B, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(A, B) -> R,
    A: HostArg,
    B: HostArg,
    R: HostReturn,
{
    move |args: &[Value], _ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        let b = B::extract(args, 1)?;
        f(a, b).into_result()
    }
}

/// Typed host function with 3 arguments.
pub fn typed_host_fn_3<F, A, B, C, R>(
    f: F,
) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(A, B, C) -> R,
    A: HostArg,
    B: HostArg,
    C: HostArg,
    R: HostReturn,
{
    move |args: &[Value], _ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        let b = B::extract(args, 1)?;
        let c = C::extract(args, 2)?;
        f(a, b, c).into_result()
    }
}

/// Typed host function with 4 arguments.
pub fn typed_host_fn_4<F, A, B, C, D, R>(
    f: F,
) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(A, B, C, D) -> R,
    A: HostArg,
    B: HostArg,
    C: HostArg,
    D: HostArg,
    R: HostReturn,
{
    move |args: &[Value], _ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        let b = B::extract(args, 1)?;
        let c = C::extract(args, 2)?;
        let d = D::extract(args, 3)?;
        f(a, b, c, d).into_result()
    }
}

/// Typed host function with context access and 0 arguments.
pub fn typed_host_fn_ctx_0<F, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(&ExecutionContext) -> R,
    R: HostReturn,
{
    move |_args: &[Value], ctx: &ExecutionContext| f(ctx).into_result()
}

/// Typed host function with context access and 1 argument.
pub fn typed_host_fn_ctx_1<F, A, R>(f: F) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(&ExecutionContext, A) -> R,
    A: HostArg,
    R: HostReturn,
{
    move |args: &[Value], ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        f(ctx, a).into_result()
    }
}

/// Typed host function with context access and 2 arguments.
pub fn typed_host_fn_ctx_2<F, A, B, R>(
    f: F,
) -> impl Fn(&[Value], &ExecutionContext) -> Result<Value>
where
    F: Fn(&ExecutionContext, A, B) -> R,
    A: HostArg,
    B: HostArg,
    R: HostReturn,
{
    move |args: &[Value], ctx: &ExecutionContext| {
        let a = A::extract(args, 0)?;
        let b = B::extract(args, 1)?;
        f(ctx, a, b).into_result()
    }
}

/// Macro to define a typed host function.
///
/// # Examples
///
/// ```ignore
/// // Simple function without context
/// host_fn!(add(a: i64, b: i64) -> i64 {
///     a + b
/// });
///
/// // Function with context access
/// host_fn!(ctx, log_message(msg: String) -> () {
///     ctx.record_output(msg.len())?;
///     println!("{}", msg);
///     Ok(())
/// });
///
/// // Function with optional argument
/// host_fn!(greet(name: String, greeting: Optional<String>) -> String {
///     let greeting = greeting.0.unwrap_or_else(|| "Hello".to_string());
///     format!("{}, {}!", greeting, name)
/// });
/// ```
#[macro_export]
macro_rules! host_fn {
    // No context, no args
    ($name:ident() -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_0(|| -> $ret $body)
    };

    // No context, 1 arg
    ($name:ident($a:ident: $at:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_1(|$a: $at| -> $ret $body)
    };

    // No context, 2 args
    ($name:ident($a:ident: $at:ty, $b:ident: $bt:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_2(|$a: $at, $b: $bt| -> $ret $body)
    };

    // No context, 3 args
    ($name:ident($a:ident: $at:ty, $b:ident: $bt:ty, $c:ident: $ct:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_3(|$a: $at, $b: $bt, $c: $ct| -> $ret $body)
    };

    // No context, 4 args
    ($name:ident($a:ident: $at:ty, $b:ident: $bt:ty, $c:ident: $ct:ty, $d:ident: $dt:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_4(|$a: $at, $b: $bt, $c: $ct, $d: $dt| -> $ret $body)
    };

    // With context, no args
    (ctx, $name:ident() -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_ctx_0(|ctx: &$crate::ExecutionContext| -> $ret $body)
    };

    // With context, 1 arg
    (ctx, $name:ident($a:ident: $at:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_ctx_1(|ctx: &$crate::ExecutionContext, $a: $at| -> $ret $body)
    };

    // With context, 2 args
    (ctx, $name:ident($a:ident: $at:ty, $b:ident: $bt:ty) -> $ret:ty $body:block) => {
        $crate::macros::typed_host_fn_ctx_2(|ctx: &$crate::ExecutionContext, $a: $at, $b: $bt| -> $ret $body)
    };
}

/// Builder for creating host function registries with typed functions.
pub struct HostFnBuilder {
    registry: crate::engine::HostRegistry,
}

impl HostFnBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            registry: crate::engine::HostRegistry::new(),
        }
    }

    /// Register a global function.
    pub fn register<S, F>(mut self, name: S, f: F) -> Self
    where
        S: Into<String>,
        F: Fn(&[Value], &ExecutionContext) -> Result<Value> + Send + Sync + 'static,
    {
        self.registry.register(name, f);
        self
    }

    /// Register a module function.
    pub fn register_module<M, N, F>(mut self, module: M, name: N, f: F) -> Self
    where
        M: Into<String>,
        N: Into<String>,
        F: Fn(&[Value], &ExecutionContext) -> Result<Value> + Send + Sync + 'static,
    {
        self.registry.register_module(module, name, f);
        self
    }

    /// Build the registry.
    pub fn build(self) -> crate::engine::HostRegistry {
        self.registry
    }
}

impl Default for HostFnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_extraction() {
        let args = vec![Value::Int(42), Value::String("hello".into())];

        let a: i64 = i64::extract(&args, 0).unwrap();
        assert_eq!(a, 42);

        let b: String = String::extract(&args, 1).unwrap();
        assert_eq!(b, "hello");
    }

    #[test]
    fn test_arg_extraction_missing() {
        let args = vec![Value::Int(42)];
        let result: std::result::Result<i64, _> = i64::extract(&args, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_arg() {
        let args = vec![Value::Int(42)];

        let opt: Optional<i64> = Optional::extract(&args, 0).unwrap();
        assert_eq!(opt.0, Some(42));

        let opt: Optional<i64> = Optional::extract(&args, 1).unwrap();
        assert_eq!(opt.0, None);

        let args_with_null = vec![Value::Null];
        let opt: Optional<i64> = Optional::extract(&args_with_null, 0).unwrap();
        assert_eq!(opt.0, None);
    }

    #[test]
    fn test_rest_args() {
        let args = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ];

        let rest: Rest<i64> = Rest::extract(&args, 1).unwrap();
        assert_eq!(rest.0, vec![2, 3, 4]);
    }

    #[test]
    fn test_typed_host_fn() {
        use crate::sandbox::{Sandbox, SandboxConfig};
        use crate::Capabilities;
        use crate::Limits;

        let add = typed_host_fn_2(|a: i64, b: i64| -> i64 { a + b });

        let sandbox = Sandbox::new(SandboxConfig::default()).unwrap();
        let ctx = ExecutionContext::new(1, Capabilities::none(), Limits::default(), sandbox);

        let result = add(&[Value::Int(3), Value::Int(4)], &ctx).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_typed_host_fn_with_result() {
        use crate::sandbox::{Sandbox, SandboxConfig};
        use crate::Capabilities;
        use crate::Limits;

        let div = typed_host_fn_2(
            |a: i64, b: i64| -> std::result::Result<i64, &'static str> {
                if b == 0 {
                    Err("division by zero")
                } else {
                    Ok(a / b)
                }
            },
        );

        let sandbox = Sandbox::new(SandboxConfig::default()).unwrap();
        let ctx = ExecutionContext::new(1, Capabilities::none(), Limits::default(), sandbox);

        let result = div(&[Value::Int(10), Value::Int(2)], &ctx).unwrap();
        assert_eq!(result, Value::Int(5));

        let result = div(&[Value::Int(10), Value::Int(0)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_host_fn_builder() {
        let registry = HostFnBuilder::new()
            .register("test", |_args, _ctx| Ok(Value::Int(42)))
            .register_module("math", "pi", |_args, _ctx| Ok(Value::Float(3.14159)))
            .build();

        assert!(registry.get("test").is_some());
        assert!(registry.get_module("math", "pi").is_some());
    }
}
