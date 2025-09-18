+++
title = "\"Promise\" types in Rust"
date = "2025-09-17"
+++

# Promise types?

A common pattern in rust code is to use zero sized types ([ZST]s) in order to signal something to the developer.
For example, the [critical-section] crate defines a ZST where anny instance indicates that a so called critical section is active.
Functions that rely on being executed in such a context can then take a `CriticalSection` instance as input.
This way, the compiler can enforce that these functions are never be called outside of a critical section context.
In other words, the existence of any `CriticalSection` instance is a **promise** to the calee about the current context.

```rust
critical_section::with(|cs| {
    // This code runs within a critical section.
    // `cs` is a token that you can use to "prove" that to some API
    MY_VALUE.borrow(cs).set(42);
});
```

[ZST]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
[critical-section]: https://docs.rs/critical-section/latest/critical_section
