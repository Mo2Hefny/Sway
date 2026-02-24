<p align="center">
  <img src="docs/assets/sway.png" alt="Sway Logo" width="240"/>
  <h1 align="center">Sway: A 2D Procedural Animation Engine</h1>
</p>

A 2D procedural animation engine built in Rust using [Bevy](https://bevyengine.org/). No physics library — every system is written from math up.

Nodes move via **Verlet integration**, stay connected through an **iterative constraint solver**, walk on **FABRIK-driven limbs**, navigate using **lookahead steering**, and get wrapped in a **Catmull-Rom spline skin** that deforms live with the simulation.

> Play around with the editor, load an example, and watch it go.

## Running
You can access the project from the attached github pages link: [Sway | Procedural Animation Engine](https://mo2hefny.github.io/Sway/)

OR run it locally:

### Desktop
```bash
git clone https://github.com/Mo2Hefny/Sway.git
cd Sway
cargo run --release
```

### Web
```bash
git clone https://github.com/Mo2Hefny/Sway.git
cd Sway
trunk serve
```

## Documentation

The math and architecture behind each system is documented in `/docs`:

| Doc | What's In It |
|-----|-------------|
| [Physics & Movement](docs/PHYSICS_MOVEMENT.md) | Verlet integration, air damping, constant acceleration, anchor handling, distance & angle constraints |
| [Collision System](docs/COLLISION_SYSTEM.md) | Boundary collision, node–node push-apart, spatial hash grid, group awareness, wander steering |
| [Limbs & Inverse Kinematics](docs/LIMBS_INVERSE_KINEMATICS.md) | FABRIK solver, bend control, stepping system, ideal target computation |
| [Visuals & Rendering](docs/VISUALS_RENDERING.md) | Node visuals, spline skin, miter outlines, limb mesh, Z-draw order |

---

## Examples

### 1. Layers & Construction
Evaluating different visual layers — from the underlying skeleton and FABRIK limbs to the spline-smoothed skin.

https://github.com/user-attachments/assets/4f76feef-f7d8-447f-b87b-fbf8a74ef2e9

### 2. The Swarm
Handling dozens of independent procedural creatures using a spatial hash grid for $O(n)$ collision broadphase and lookahead steering for proactive avoidance.

https://github.com/user-attachments/assets/3153a376-9955-4fc7-83be-64fabedb53ee

### 3. Mixed Simulation
Simulating different creature types (Lizards and Snakes) in the same arena, each with different constraint and limb configurations.

https://github.com/user-attachments/assets/7fb47959-7add-4b17-ad68-c08925fc3a88

## Project Layout

```
src/
  core/      — physics engine (nodes, solver, collision, FABRIK)
  editor/    — rendering, mesh generation, visuals
  ui/        — inspector, toolbar, panels
docs/        — system documentation
examples/    — prebuilt scenes
```
