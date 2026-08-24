Wraps a function or method in `anyhow`'s `with_context`, alleviating the need to do it
in the caller or at every return site.

```rust
#[context("failed to say hello")]
fn say_hello() -> anyhow::Result<()> {
    // ...
    # Ok(())
}
```

Arguments passed to the function can be used as format arguments, as can arbitrary
additional arguments to the attribute.

```rust
#[context("failed to frobnify {foo:?} with {}", SOME_CONSTANT)]
fn frobnify<T: Debug>(foo: T) -> anyhow::Result<()> {
    // ...
    # Ok(())
}
```

If anyhow is not located at `::anyhow` but re-exported somewhere else, use the
special `anyhow` keyword argument to specify where.

```rust
#[context("failed to say goodbye", anyhow = othercrate::anyhow)]
fn say_goodbye() -> othercrate::anyhow::Result<()> {
    // ...
    # Ok(())
}
```