# Rusty Gears

Rusty Gears is a game engine written in Rust that integrates rendering and physics into a cohesive framework. This engine is currently under active development and is designed for flexibility, performance, and modern rendering techniques.

## Features

### Rendering Engine
- Built using OpenGL with the `gl` and `glfw` crates.
- Transitioning to `winit` and `wgpu` with Vulkan-based rendering for better performance and broader compatibility.
- Supports advanced lighting effects, including:
  - Shadows
  - Fog

### Physics Engine
- Focuses on realistic rigid body dynamics.
- Implements collision detection and resolution, including:
  - Rotational dynamics
  - Friction handling
- Plans to support soft body structures in future updates.

### Current Limitations
- Lacks optimization techniques such as spatial partitioning (e.g., Octree, BVH).
- Collision handling methods, such as `handle_collisions` and `detect_collisions`, are not yet optimized for large-scale simulations.

## Roadmap
- Complete migration to `wgpu` for Vulkan-based rendering.
- Add support for soft body physics.
- Implement spatial partitioning for improved collision detection efficiency.
- Introduce animation support.
- Develop terrain and landscape generation tools.

## Installation
Clone the repository and build the project:

```bash
git clone https://github.com/hikamaree/RustyGears.git
```

To try the demo simulation, navigate to the `RustyGears/game` directory and run the following command:

```bash
cd RustyGears/game
cargo run --release
```

The simulation starts when you fire the first bullet, and bullets can be fired by pressing the `F` key.
Movement in the simulation can be controlled using the W, A, S, D keys and mouse. 
