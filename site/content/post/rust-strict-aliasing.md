+++
title = "Pointer aliasing in rust"
date = "2025-12-23"
[taxonomies]
tags = ["rust"]
+++

# Pointer aliasing?

Under the C abstract machine, multiple pointers may alias (i.e. point) to the same location.
For example in the snippet below, both function parameters, `*x` and `*y`, may point to the same memory location.
This makes it illegal for the compiler to optimize this function by always returning 42 since `*x` may get overwritten using `y`.
Therefore, the value of `x` must be loaded from memory again after the write using `y`.
In the output assembly, this is performed by the load-word instruction on line 6.

<div style="display: grid; grid-template-columns: 2fr 1fr; gap: 1em">

```c
uint32_t func(uint32_t *x, uint32_t *y) {
    *x = 42;
    *y = 84;
    return *x;
}
```

```riscv,linenos
func:
    li a5, 42
    sw a5, 0(a0)
    li a5, 84
    sw a5, 0(a1)
    lw a0, 0(a0)
    ret
```

</div>

# Pointer aliasing in rust

In Rust, mutable pointers **are not allowed to alias** because that would violate the [borrowing rules].
This makes it possible to optimize the return statement of the following rust snippet.
As shown on line 5, the rust compiler optimizes the extra load away in favour of a load-immediate instruction.

<div style="display: grid; grid-template-columns: 2fr 1fr; gap: 1em">

```rust
fn func(x: &mut u32, y: &mut u32) -> u32 {
    *x = 42;
    *y = 84;
    return *x;
}
```

```riscv,linenos
func:
    li a2, 42
    sw a2, 0(a0)
    li a2, 84
    li a0, 42
    sw a2, 0(a1)
    ret
```

</div>

Note that the C code can be made to behave the same as the rust version by informing the compiler that `y` is not allowed to alias `x` using the `restrict` keyword:

<div style="display: grid; grid-template-columns: 2fr 1fr; gap: 1em">

```c
uint32_t func(uint32_t * restrict x, uint32_t * y) {
    *x = 42;
    *y = 84;
    return *x;
}
```

```riscv,linenos
func:
    li a5, 42
    li a4, 84
    sw a5, 0(a0)
    sw a4, 0(a1)
    mv a0, a5
    ret
```

</div>

[borrowing rules]: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references
