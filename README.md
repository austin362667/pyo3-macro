## PyO3-macro



A Rust macro crate builds PyO3-compatible Rust `protobuf` and `gRPC` structures.
So you can easily expose your generated protobuf code as Pythin binding through PyO3.

## Features

1. Macro `with_new` that implements `__new__` constructor for Rust Python binding.
2. Macro `with_pyclass` that add `pyclass` attributes macro for your structures.

