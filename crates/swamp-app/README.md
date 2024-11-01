# swamp-app 🐊

[![Crates.io](https://img.shields.io/crates/v/swamp-app)](https://crates.io/crates/swamp-app)
[![Documentation](https://docs.rs/swamp-app/badge.svg)](https://docs.rs/swamp-app)

Swamp App is a Rust crate that provides a high-level abstraction for creating and managing game applications 
using winit and wgpu. It leverages the power of swamp-wgpu-window and swamp-render to handle window creation,
rendering surfaces, and the application lifecycle. By implementing the Application trait, developers can focus 
on application logic without worrying about the underlying rendering and window management details.

## ✨ Features

- Simplified Window Management: Easily create and manage application windows using swamp-wgpu-window.
- Rendering Abstraction: Utilize swamp-render for efficient sprite-based rendering with minimal setup.
- Application Lifecycle Management: Implement the Application trait to handle initialization, ticking, and rendering seamlessly.
- Async Support: Leverage asynchronous initialization for smooth and non-blocking application setup.
- Cross-Platform Compatibility: Works across all major operating systems supported by winit and wgpu.

## 📦 Installation

Add swamp-app to your project’s Cargo.toml:

```toml
[dependencies]
swamp-app = "0.0.1"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.