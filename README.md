ArcCow is a Rust smart pointer that combines the best features of Cow (clone-on-write) and Arc (atomic reference counting) to provide efficient, flexible data handling.

The ArcCow type represents either:

- A borrowed reference data (zero-allocation)
- A owned value with atomic reference counting


The code is originally copied from zed.
