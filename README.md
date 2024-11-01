# Swamp-Render Workspace 🐊

Welcome to the **Swamp-Render** workspace! This Rust workspace is designed to streamline development by 
organizing multiple interrelated crates into one repo.

## Crates

The Swamp-Render workspace is divided into high-level and low-level crates.

### High Level

These crates provide higher-level abstractions to simplify application and rendering development.

- **[`swamp-app`](crates/swamp-app/README.md)**: A framework for building applications with integrated 
rendering capabilities. It handles window management, event handling, and integrates seamlessly with rendering crates.

- **[`swamp-render`](crates/swamp-render/README.md)**: The main rendering engine that leverages the low-level 
wgpu-based crates to provide an easy-to-use API for rendering pixel perfect 2D graphics in your applications.

### Low Level

These crates offer low-level functionalities.

- **[`swamp-wgpu`](crates/swamp-wgpu/README.md)**: A low-level wrapper around the wgpu graphics API, 
providing essential rendering functionalities and resource management.

- **[`swamp-wgpu-sprites`](crates/swamp-wgpu-sprites/README.md)**: A specialized crate for handling sprite 
rendering, including sprite batching, animations, and texture management using wgpu.

- **[`swamp-wgpu-window`](crates/swamp-wgpu-window/README.md)**: Manages window creation and event handling using
wgpu, enabling seamless integration with rendering pipelines.
