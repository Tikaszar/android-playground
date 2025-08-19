# playground-core-math

Mathematical primitives and utilities for the Android Playground engine.

## Overview

The core math crate provides mathematical types and operations using nalgebra as the underlying math library. Currently under development.

## Dependencies

- `nalgebra`: Linear algebra library with serialization support
- `approx`: Floating point comparison utilities  
- `num-traits`: Numeric traits for generic programming
- `bytemuck` (optional): GPU buffer compatibility
- `mint` (optional): Interoperability with other math libraries

## Features

- `default`: Basic math functionality
- `gpu`: Enable bytemuck for GPU buffer compatibility
- `interop`: Enable mint for math library interoperability

## Status

🚧 **Under Construction** - Source files not yet implemented

## See Also

- [core/types](../types/README.md) - Core type definitions
- [nalgebra docs](https://nalgebra.org/) - Underlying math library