//! Isometric math, port of `pack/rumus.as` (SIZE_ = 24, tile 48x24 px).
//!
//! Original Flash (y down): screen.x = (ty - tx) * 24; screen.y = -(tx + ty) * 12,
//! i.e. bigger tx+ty moves UP the screen (toward the back of the resort) and the
//! entry roads (small i/j) sit at the bottom. Bevy world is y-up, so the faithful
//! conversion negates flash-y once: world.y = -flash.y = +(tx + ty) * 12.
//! (The previous port kept flash-y unnegated, which mirrored the whole map
//! vertically vs the original game.)

pub const SIZE: f32 = 24.0;
pub const HALF: f32 = SIZE * 0.5;

/// Lattice point of tile (tx,ty) in Bevy world coords (y up).
/// This is the TOP vertex of the tile's screen diamond; the diamond hangs
/// 24 px downward and spans 24 px to each side (Flash blitting registration).
pub fn tile_to_world(tx: i32, ty: i32) -> (f32, f32) {
    ((ty - tx) as f32 * SIZE, (tx + ty) as f32 * HALF)
}

/// Visual center of the diamond of tile (tx,ty).
pub fn tile_center(tx: i32, ty: i32) -> (f32, f32) {
    let (x, y) = tile_to_world(tx, ty);
    (x, y - HALF)
}

/// Inverse: which tile's visual diamond contains the world point.
/// In lattice units s = wy/12 (= tx+ty at the top vertex) and d = wx/24
/// (= ty-tx), the diamond of tile (tx,ty) maps to the open unit square
/// u in (tx-1,tx), v in (ty-1,ty) where u = (s-d)/2 and v = (s+d)/2.
pub fn world_to_tile(wx: f32, wy: f32) -> (i32, i32) {
    let s = wy / HALF;
    let d = wx / SIZE;
    let tx = ((s - d) * 0.5).floor() as i32 + 1;
    let ty = ((s + d) * 0.5).floor() as i32 + 1;
    (tx, ty)
}

/// Painter depth: bigger tx+ty is farther back (higher on screen) and must
/// draw first (smaller z).
pub fn depth(tx: i32, ty: i32, layer: f32) -> f32 {
    layer + (200 - (tx + ty)) as f32 * 0.001
}
