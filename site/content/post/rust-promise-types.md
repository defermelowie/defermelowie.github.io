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
That is, the existence of a `CriticalSection` instance is a **promise** to the callee about the current context.

```rust
critical_section::with(|cs| {
    // This code runs within a critical section.
    // `cs` is a token that you can use to "prove" this fact to some API
    MY_VALUE.borrow(cs).set(42);
});
```

The snipped above uses this principle to enforce that `Mutex::borrow()` can never be called when it might get interrupted.
This is important because if other threads try to access the global `MY_VALUE` at _the same time_, race conditions could start to occur.

A `CriticalSection` is a very strong promise.
Sometimes &mdash; on single core systems when the shared state is only accessed in other processes &mdash; it's not necessary to disable interrupts all together.
Instead, a promise from the scheduler that we won't be switched out while mutating the shared state is sufficient as it implies that no one else will be switched in nor will be able to access it in the meantime.
This promise, called a `SchedulerLock`, could therefore be used to build a different kind of mutex (`ProcMutex`) that can only protect data that's exclusively used from processes.
Therefore, it's important to prevent the creation of this promise from outside a process context.
This could be achieved by checking the current context inside the system call, or by using a new promise type that's given to the process entry point and promises to the `SchedulerLock` constructor that it's being called from a process context.

> [!CAUTION]
> Even though a `CriticalSection` also prevents against being switched out, it should never be converted to a `SchedulerLock`!
> It can happen for `CriticalSection`s created from an interrupt service routine (ISR) that there is already a `SchedulerLock` promise granted and giving out a second one would enable the ISR to also mutate `ProcMutex` protected state.
> This is violation of the requirement that `ProcMutex` protected state is exclusively used from a process context an should therefore be prevented.

:construction: Under construction :construction:

# Relation to RAII

This _promise type_ pattern is fairly related to Resource Acquisition Is Initialization ([RAII]).
When a promise is needed, a function is called to acquire an instance of the right _promise type_.
This function then automatically _initializes_ the system such that the promise holds.
Afterwards, when the instance goes out of scope, its `Drop()` implementation can relax the system again, consuming the promise at the same time.

[ZST]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
[critical-section]: https://docs.rs/critical-section/latest/critical_section
[RAII]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
