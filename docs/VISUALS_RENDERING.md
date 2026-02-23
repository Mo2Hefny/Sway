# Visuals & Rendering 🎨

Making math look good is half the battle. If we just rendered white circles, it would look like... well, a physics debug view. We want *life*.

Here is how I implemented the procedural skinning in `src/editor/visuals/skin.rs`.

## 1. From Skeleton to Body 🦴 -> 🐍

The core physics gives us a list of nodes (the "bones"). To make a skin, we need to turn that line of points into a shape.

### Step A: Ordering the Chain
First, we have to figure out who connects to whom. `build_ordered_chains` walks through our `ConstraintGraph` starting from the Answer (the head) and traces valid paths to the tail.

### Step B: Generating Control Points
For every node in the chain, I calculate two points "left" and "right" relative to the node's angle.
*   **Head**: Gets a rounded cap.
*   **Body**: Gets width based on the node's radius.
*   **Tail**: Gets a rounded cap.

This gives us a jagged, blocky outline of the creature.

![Skeleton Outline](assets/skeleton_outline.png)

## 2. Smoothing with Catmull-Rom Splines 🖌️

A blocky snake is ugly. To make it organic, I function called `evaluate_catmull_rom_closed`.

Catmull-Rom splines are great because they pass *through* the control points (unlike Bezier curves which just get near them). This ensures the skin actually touches the physics nodes.

```rust
// We take the rough points and inject extra samples between them
// effectively smoothing out the sharp corners.
let smoothed_points = evaluate_catmull_rom_closed(&control_points, SPLINE_SAMPLES);
```

## 3. Triangulation (Ear Clipping) ✂️

Now we have a beautiful smooth loop of points... but graphics cards only understand triangles.

I implemented a robust **Ear Clipping** algorithm in `ear_clip_triangulate`.
1.  It iterates through the polygon.
2.  It finds a "convex" corner (an "ear") that doesn't contain any other points.
3.  It snips it off (makes a triangle) and repeats until the whole shape is filled.

## 4. The Outline (Miter Joints) ✏️

To give it that crisp vector-art look, I added an outline. But simple line rendering looks bad at corners (you get gaps or overlap).

I calculated **Miter Normals** for every point. This pushes the outline vertices out along the *bisector* of the corner angle, ensuring perfectly sharp corners with constant thickness.

![Final Skin Result](assets/final_skin.gif)

---

*This part was honestly the hardest to debug—getting the math for the miter joints right took a few tries!*
