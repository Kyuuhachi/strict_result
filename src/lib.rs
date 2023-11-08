#![no_std]
#![feature(try_trait_v2)]

/*!

The `?` operator on [`Result`] has an implicit `.into()` on the error type, to make it easier to
upcast errors into broader error types. This however can sometimes cause problems for type
inference in highly generic contexts, since it effectively turns into `.into().into()`, which is
ambiguous. To combat this, this crate defines a separate `.strict()?` operator, which does not
perform this implicit `.into()`.

For an example, let's define a simple generic function:

```
fn passthrough<T>(f: impl FnOnce() -> T) -> T {
    f()
}
```

If we try to use this combined with the `?` operator, we will get an error because the generic
`<T>` cannot be determined.

```compile_fail
# fn passthrough<T>(f: impl FnOnce() -> T) -> T {
#     f()
# }
# fn lenient() -> std::io::Result<()> {
passthrough(|| {
    std::fs::create_dir("example")?;
    Ok(()) // cannot infer type of the type parameter `E` declared on the enum `Result`
})?;
# Ok(())
# }
```

In this case we can use `.strict()?` to require that the error type is equal to the outer one.

```
# fn passthrough<T>(f: impl FnOnce() -> T) -> T {
#     f()
# }
use strict_result::Strict;

# fn strict() -> std::io::Result<()> {
passthrough(|| {
    std::fs::create_dir("example")?;
    Ok(())
}).strict()?;
# Ok(())
# }
```

This crate uses the `try_trait_v2` feature, and thus requires nightly.
*/

mod seal {
	pub trait Sealed {}
	impl<A, B> Sealed for Result<A, B> {}
}

use core::convert::Infallible;
use core::ops::{ControlFlow, Try, FromResidual};

/// A wrapper around `Result` that suppresses the implicit `.into()` when using the `?` operator.
///
/// It can be converted to and from `Result` with the [`strict`](Strict::strict) and
/// [`loose`](StrictResult::loose) functions.
///
/// It is in general not recommended to have this function in function signatures; the recommended
/// usage is to use `.strict()?` after function calls to suppress the casting. Still, this type is
/// exposed in it might be useful.
#[repr(transparent)]
#[must_use = "the contained `Result` may be an `Err` variant, which should be handled"]
pub struct StrictResult<A, B>(Result<A, B>);

/// Provides the `.strict()?` function.
///
/// See the [top-level description](crate) for details.
pub trait Strict<A, B>: seal::Sealed {
	fn strict(self) -> StrictResult<A, B>;
}

impl<A, B> Strict<A, B> for Result<A, B> {
	fn strict(self) -> StrictResult<A, B> {
		StrictResult(self)
	}
}

impl<A, B> StrictResult<A, B> {
	pub fn loose(self) -> Result<A, B> {
		self.0
	}
}

impl<A, B> FromResidual<StrictResult<Infallible, B>> for StrictResult<A, B> {
	fn from_residual(r: StrictResult<Infallible, B>) -> Self {
		match r.loose() {
			Ok(v) => match v {},
			Err(v) => Err(v)
		}.strict()
	}
}

impl<A, B> FromResidual<StrictResult<Infallible, B>> for Result<A, B> {
	fn from_residual(r: StrictResult<Infallible, B>) -> Self {
		match r.loose() {
			Ok(v) => match v {},
			Err(r) => Err(r)
		}
	}
}

impl<A, B, B2: From<B>> FromResidual<Result<Infallible, B>> for StrictResult<A, B2> {
	fn from_residual(r: Result<Infallible, B>) -> Self {
		match r {
			Ok(v) => match v {},
			Err(r) => Err(r.into())
		}.strict()
	}
}

impl<A, B> Try for StrictResult<A, B> {
	type Output = A;
	type Residual = StrictResult<Infallible, B>;

	fn from_output(r: A) -> Self {
		Ok(r).strict()
	}

	fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
		match self.loose() {
			Ok(v) => ControlFlow::Continue(v),
			Err(e) => ControlFlow::Break(Err(e).strict()),
		}
	}
}
