//! Isometric math, port of `pack/rumus.as` (SIZE_ = 24, tile 48x24 px).
//!
//! Original Flash (y down): screen.x = (ty - tx) * 24; screen.y = -(tx + ty) * 12.
//! Bevy world is y-up, so we keep Flash coordinates and negate y when spawning:
//! world.x = (ty - tx) * 24; world.y = -(tx + ty) * 12  -> already y-up friendly
//! (bigger tx+ty => lower on screen).

pub const SIZE: f32 = 24.0;
pub const HALF: f32 = SIZE * 0.5;

/// N-vertex of tile (tx,ty) in Bevy world coords (y up).
pub fn tile_to_world(tx: i32, ty: i32) -> (f32, f32) {
    (
        (ty - tx) as f32 * SIZE,
        -((tx + ty) as f32 * HALF),
    )
}

/// Center of tile (tx,ty) in world coords.
pub fn tile_center(tx: i32, ty: i32) -> (f32, f32) {
    let (x, y) = tile_to_world(tx, ty);
    (x + SIZE, y - HALF)
}

/// Inverse: world position -> tile coords (port of rumus.findTile).
pub fn world_to_tile(wx: f32, wy: f32) -> (i32, i32) {
    // Flash used y-down input; convert from our y-up world.
    let py = -wy; // flash y
    let fx = (2.0 * py - wx) * 0.5; // = tx * 24
    let fy = py + fx - 2.0 * py; // placeholder, use derivation below
    let _ = fy;
    // Direct inversion: wx = (ty-tx)*24, py = (tx+ty)*12
    // => ty - tx = wx/24 ; tx + ty = py/12
    let s = py / 12.0; // tx+ty
    let d = wx / 24.0; // ty-tx
    let tx = (s - d) * 0.5;
    let ty = (s + d) * 0.5;
    (tx.floor() as i32, ty.floor() as i32)
}

/// Depth (z) ordering: objects further down-screen draw on top.
/// z grows with (tx+ty); scaled small to stay within camera range.
pub fn depth(tx: i32, ty: i32, layer: f32) -> f32 {
    layer + (tx + ty) as f32 * 0.001
}
