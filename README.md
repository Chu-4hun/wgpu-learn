# wgpu-learn

A personal project exploring low-level GPU programming in Rust using [wgpu](https://github.com/gfx-rs/wgpu) — a cross-platform, safe graphics API built on top of WebGPU.

The goal is to build a small 3D engine from scratch, progressively adding rendering features and learning the wgpu pipeline along the way.

---

## Features

- **3D model loading** — `.obj` file parsing and rendering
- **Texture loading** — diffuse texture support for models
- **Depth buffer** — correct occlusion for overlapping geometry
- **FPS camera** — mouse-look with cursor locking, delta-time based movement
- **Debug UI** — real-time FPS and frame time overlay via [egui](https://github.com/emilk/egui)
- **Performance profiler** — integrated GPU/CPU profiling
- **FOV slider & color picker** — runtime render parameter adjustments
- **Modular architecture** — ongoing refactor from monolithic state into clean pipeline modules

---

## Tech Stack

| | |
|---|---|
| **Language** | Rust |
| **Graphics API** | [wgpu](https://github.com/gfx-rs/wgpu) |
| **Shaders** | WGSL |
| **Debug UI** | egui |
| **Build / Dev env** | Nix flake |

---

## Getting Started

### Prerequisites

- Rust (stable toolchain)
- A GPU with Vulkan, Metal, or DX12 support

### Build & Run

```bash
git clone https://github.com/Chu-4hun/wgpu-learn
cd wgpu-learn
cargo run
```

If you're using Nix:

```bash
nix develop
cargo run
```

---

## Project Structure

```
wgpu-learn/
├── app/        # Application entry point and window management
├── engine/     # Core rendering engine (pipeline, camera, buffers)
├── assets/     # Models, textures, and shaders
```

---

## Changelog

All notable changes are documented in [CHANGELOG.md](./CHANGELOG.md).

---

## License

This is a personal learning project and is not licensed for production use.
