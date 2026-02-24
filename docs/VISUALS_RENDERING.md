# Visuals & Rendering :art:

This covers every visual layer in the editor from the playground backdrop all the way to the spline-smoothed skin.

## 1. The Playground

The playground is the world boundary. It auto-syncs its dimensions to match the window dimensions every frame, so it always fills the screen correctly.

## 2. Node Visuals

Every node that gets added to the world automatically spawns a set of child visual entities. All of these are children of the node entity itself, so they move along with it for free via Bevy's transform hierarchy.

### The Circle

The node body is a **hollow circle** (a ring mesh). Its thickness is constant (`CIRCLE_THICKNESS = 4.0`) and its radius matches the node's physics radius.

The color depends on node type:

| Node Type | Color |
|-----------|-------|
| `Normal` | `srgb(0.8, 0.8, 0.8)` — light grey |
| `Anchor` | `srgb(1.0, 0.4, 0.4)` — reddish |
| `Limb` | `srgb(0.4, 1.0, 0.4)` — greenish |
| `Selected` | `srgb(0.3, 0.7, 1.0)` — blue highlight |
| `Limb's Root` | `srgb(0.6, 0.2, 0.8)` — purple |

### Contact Points

Two tiny filled dots, one on each side of the node perpendicular to `chain_angle`. These are used in Catmull-Rom Spline but I added their visualizing for debugging purposes (and they looked good?).

### Look Vector

A short line extending from the node center along `chain_angle`. This is the node's "forward" direction. It is useful for visually confirming which way the node thinks it's facing. Only visible in debug mode.

### Direction Vector

A longer line (only on Anchor nodes) pointing from the node toward its `target_position`. This gives you a live visual of where the anchor is trying to walk. Only visible in debug mode.

### Target Marker (X)

An X-shaped marker placed at the node's `target_position`. Anchor nodes get one for their movement target; nodes with a `LimbSet` get one per limb, placed at each limb's planted foot position. Extra markers are spawned dynamically and despawned when limbs are removed.

### Eyes

If `is_head` is `true`, two filled white dots appear perpendicular to `chain_angle` at `radius × EYE_DISTANCE_RATIO` from center. They rotate with `chain_angle`, so they always look the same direction as the node. The alpha is set to `0.0` for non-head nodes rather than despawning them, so toggling `is_head` is instant.

### Angle Arc

A filled pie-slice arc centered on the node, spanning from `chain_angle + angle_min` to `chain_angle + angle_max`. It's drawn at `radius × 0.85` so it sits just inside the ring. Uses a very transparent yellow-gold color (`alpha = 0.15`). Rotates with the node and the arc itself is regenerated only when the angles actually change, thanks to change detection via `NodeVisualCache`.

### Change Detection

`NodeVisualCache` stores the last seen `radius`, `arc_start`, and `arc_end`. The sync system compares these before doing any mesh work:

```rust
if (node.radius - cache.radius).abs() > 1e-4 {
    // rebuild circle mesh
}
if (arc_start - cache.arc_start).abs() > 1e-4 || (arc_end - cache.arc_end).abs() > 1e-4 {
    // rebuild arc mesh
}
```

This avoids reallocating meshes every frame for nodes that aren't changing.

## 4. Body Skin

The skin is the most visually complex part. It draws a smooth, organic-looking body shape that stretches over the node chain.

### Skin Chains

Before rendering, the system traces the `ConstraintGraph` to extract ordered chains of non-Limb nodes, starting from Anchor nodes or leaf nodes. These are the "spine" paths that the skin wraps around. If the graph changes, the chains are re-traced and cached in the `SkinChains` resource.

### The Catmull-Rom Spline

The skin isn't drawn as straight lines between nodes. Instead, for each chain, we compute **left** and **right** contact points perpendicular to each node's `chain_angle`, then run a **Catmull-Rom spline** through them.

The left and right offsets are simple:

$$left_i = position_i + \hat{(chain\_angle_i + \frac{\pi}{2})} \times radius_i$$
$$right_i = position_i + \hat{(chain\_angle_i - \frac{\pi}{2})} \times radius_i$$

Catmull-Rom produces a smooth curve through the control points with $C^1$ continuity. For open chains (spine), we use reflected phantom endpoints so the curve has clean tangents at both ends:

$$phantom\_start = 2 \times P_0 - P_1$$

The formula at each $t$:

$$P(t) = \frac{1}{2}\left[(2P_1) + (-P_0 + P_2)t + (2P_0 - 5P_1 + 4P_2 - P_3)t^2 + (-P_0 + 3P_1 - 3P_2 + P_3)t^3\right]$$

### Building the Fill Mesh

The filled body is a **triangle strip** between the left and right spline curves. Each consecutive pair of points from both curves forms two triangles:

```
left[i]  ── left[i+1]
  |      ╲       |
right[i] ── right[i+1]
```

### End Caps

The head and tail of the chain get semicircular **arc caps** to close off the ends cleanly. Each cap is a fan of triangles radiating from the node center. The cap arc endpoints are snapped to the last points of the left and right spline curves to avoid seams.

### The Outline

A separate outline mesh is generated from the closed polygon (left side -> tail cap -> right side -> head cap -> back). This uses **miter joints** at each vertex:

$$miter = normalize(normal_{prev} + normal_{next})$$
$$miter\_length = \frac{1}{miter \cdot normal_{prev}}$$

The miter length is clamped to `MITER_LIMIT = 2.0` to prevent very sharp corners from exploding outward.

### Skin Color

Each connected group of nodes gets a color from `SKIN_PALETTE` (8 colors). The index is derived from the minimum entity ID in the group, making it stable across frames. When `show_nodes` is off, the color is made fully opaque; otherwise it's semi-transparent so the node circles show through.

## 5. Limb Skin

Limbs get their own separate mesh, rendered behind the body skin.

The shape is also spline-driven but uses **variable width**: each joint's effective radius linearly interpolates from `LIMB_BASE_WIDTH` at the root toward `LIMB_TIP_WIDTH` at the tip. The actual node radius is used if it is larger than this fallback:

$$r_{effective} = \max(radius_i,\; lerp(LIMB\_BASE\_WIDTH,\; LIMB\_TIP\_WIDTH,\; t))$$

The joint angle for the limb strip is computed as the average of the incoming and outgoing segment directions:

$$direction = normalize(dir_{in} + dir_{out})$$

The tip gets a semicircular cap. The root has no cap since the body skin overlaps there.

Multiple limbs from the same creature are merged into a single mesh per group to keep draw calls low.

## 6. Visibility Toggles

The `DisplaySettings` resource controls which visual layers are shown. Each layer has a dedicated system that checks `display_settings.is_changed()` before doing anything:

| Setting | What It Shows/Hides |
|---------|---------------------|
| `show_nodes` | Node ring circles |
| `show_skin` | Body skin + limb skin |
| `show_edge` | Constraint lines |
| `show_debug` | Contact points, look vectors, target markers, direction arrows, angle arcs |
