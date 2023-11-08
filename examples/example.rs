#![allow(unused)]

use strict_result::{Strict, StrictResult};

fn returns_result<E>() -> Result<(), E> {
	returns_strict()?;
	Ok(())
}

fn returns_strict<E>() -> StrictResult<(), E> {
	returns_result()?;
	Ok(()).strict()
}

fn passthrough<T>(f: impl FnOnce() -> T) -> T {
	f()
}

fn example() -> std::io::Result<()> {
	passthrough(|| {
		std::fs::create_dir("example")?;
		Ok(()) // cannot infer type of the type parameter `E` declared on the enum `Result`
	}).strict()?;
	Ok(())
}

struct AError;
struct BError;

enum ABError {
	A(AError),
	B(BError),
}

impl From<AError> for ABError {
	fn from(a: AError) -> ABError { ABError::A(a) }
}

impl From<BError> for ABError {
	fn from(b: BError) -> ABError { ABError::B(b) }
}

fn foob() -> Result<(), AError> {
	Ok(())
}

fn foo<E>() -> StrictResult<(), E>
where
	E: From<AError> + From<BError>,
{
	foob()?;
	Ok(()).strict()
}

fn bar() -> Result<(), ABError> {
	foo()?;
	Ok(())
}

fn main() {}
