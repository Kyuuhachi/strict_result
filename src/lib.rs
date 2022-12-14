#![no_std]
#![feature(try_trait_v2, never_type)]

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

mod inner {
	use core::ops::{ControlFlow, Try, FromResidual};

	#[repr(transparent)]
	#[must_use = ".strict()? is intended as a single operator"]
	pub struct StrictResult<A, B>(Result<A, B>);

	pub trait Sealed {}

	impl<A, B> Sealed for Result<A, B> {}

	/// Provides the `.strict()?` function.
	///
	/// See the [top-level description](crate) for details.
	///
	/// The `StrictResult` type is intentionally not exposed, to discourage use other than a direct
	/// `.strict()?`.
	pub trait Strict<A, B>: Sealed {
		fn strict(self) -> StrictResult<A, B>;
	}

	impl<A, B> Strict<A, B> for Result<A, B> {
		fn strict(self) -> StrictResult<A, B> {
			StrictResult(self)
		}
	}

	impl<A, B> FromResidual<StrictResult<!, B>> for StrictResult<A, B> {
		fn from_residual(r: StrictResult<!, B>) -> Self {
			match r {
				StrictResult(Ok(v)) => match v {},
				StrictResult(Err(v)) => StrictResult(Err(v))
			}
		}
	}

	impl<A, B> FromResidual<StrictResult<!, B>> for Result<A, B> {
		fn from_residual(r: StrictResult<!, B>) -> Self {
			match r {
				StrictResult(Ok(v)) => match v {},
				StrictResult(Err(r)) => Err(r)
			}
		}
	}

	impl<A, B> Try for StrictResult<A, B> {
		type Output = A;
		type Residual = StrictResult<!, B>;

		fn from_output(r: A) -> Self {
			StrictResult(Ok(r))
		}

		fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
			match self {
				StrictResult(Ok(v)) => ControlFlow::Continue(v),
				StrictResult(Err(e)) => ControlFlow::Break(StrictResult(Err(e))),
			}
		}
	}
}

pub use inner::Strict;

#[doc(hidden)]
#[deprecated = "renamed to `Strict`"]
pub use Strict as ResultAsStrict;
