# strict-result

The `?` operator on `Result` has an implicit `.into()` on the error type, to make it easier to
upcast errors into broader error types. This however can sometimes cause problems for type
inference in highly generic contexts, since it effectively turns into `.into().into()`, which is
ambiguous. To combat this, this crate defines a separate `.strict()?` operator, which does not
perform this implicit `.into()`.

For an example, let's define a simple generic function:

```rs
fn passthrough<T>(f: impl FnOnce() -> T) -> T {
    f()
}
```

If we try to use this combined with the `?` operator, we will get an error because the generic
`<T>` cannot be determined.

```rs
passthrough(|| {
    std::fs::create_dir("example")?;
    Ok(()) // cannot infer type of the type parameter `E` declared on the enum `Result`
})?;
```

In this case we can use `.strict()?` to require that the error type is equal to the outer one.

```rs
use strict_result::Strict;

# fn strict() -> std::io::Result<()> {
passthrough(|| {
    std::fs::create_dir("example")?;
    Ok(())
}).strict()?;
```

This crate uses the `try_trait_v2` feature, and thus requires nightly.
