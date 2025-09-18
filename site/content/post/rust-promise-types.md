+++
title = "Promise types in embedded rust"
date = "2025-09-17"
[taxonomies]
tags = ["rust"]
+++

# What are promise types?

A common pattern in rust code is to use zero sized types ([ZST]s) in order to signal something.
For example, the [critical-section] crate defines a ZST where any instance indicates that a so called critical section is active.
Functions that rely on being executed in such a context can then take a `CriticalSection` instance as input.
This way, the compiler can enforce that these functions are never be called outside of a critical section context.
In other words, the existence of a `CriticalSection` instance is a **promise** to the callee about the current context.

```rust
critical_section::with(|cs| {
    // This code runs within a critical section.
    // `cs` is a token that you can use to "prove" this fact to some API
    MY_VALUE.borrow(cs).set(42);
});
```

The snipped above uses this principle to enforce that `Mutex::borrow()` can never be called when we might get interrupted or switched out.
This is important because if other threads try to access the global `MY_VALUE` at _the same time_, we could end up with race conditions.

# Other usecases

A `CriticalSection` is a very strong promise.
Sometimes, on single core systems when the shared state is only accessed in other threads, we do not need to disable interrupts all together.
A promise from the scheduler that we won't be switched out is sufficient.
Lets call this promise a `SchedulerLock`.

:construction: Under construction :construction:

# Relation to RAII

The pattern that is discussed above is very much related to Resource Acquisition Is Initialization ([RAII]).
When a promise is needed, a function is called in order to acquire an instance of the right _promise type_.
This function then automatically _initializes_ the system such that the promise holds.
Afterwards, when the instance goes out of scope, its `Drop()` implementation can relax the system again, consuming the promise at the same time.

[ZST]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
[critical-section]: https://docs.rs/critical-section/latest/critical_section
[RAII]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
