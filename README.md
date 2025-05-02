# Rusty Gears

**Rusty Gears** is a modular game engine written in Rust, featuring a Vulkan-based renderer and
a custom ECS (Entity-Component-System) architecture. It is built for flexibility, performance,
and extensibility, making it suitable for both experimentation and game development.

## Features

### Modular Architecture
- Pluggable game loop with *Gears* — modular systems that respond to dispatched events.
- Custom ECS implementation tailored for multi-threaded workloads.

### Rendering
- Vulkan-based rendering using [`wgpu`](https://github.com/gfx-rs/wgpu) and [`winit`](https://github.com/rust-windowing/winit).
- Supports physically-based rendering (PBR), shadows, and fog.
- GPU-side frustum culling and efficient instancing.

## Current Limitations

- No spatial partitioning (e.g., Octree or BVH).
- Physics system is not yet complete or integrated.
- No animation or terrain support at this time.

## Roadmap

- Add physics Gear for rigid body simulation.
- Implement spatial partitioning (e.g., BVH).
- Add animation system.
- Add terrain and landscape generation.

## Installation

Clone and build the project:

```bash
git clone https://github.com/hikamaree/RustyGears.git
```

To try the demo simulation, navigate to the `RustyGears/game` directory and run the following command:

```bash
cd RustyGears/game
cargo run --release
```

## License
This project is licensed under the GNU General Public License v3.0 — see the [LICENSE](./LICENSE) file for more information.
