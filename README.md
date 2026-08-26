# Resort Empire — Rust/Bevy Port

Port jugable (sin emuladores Flash) del juego **Resort Empire** (Little Giant World, AS3/Flash),
reescrito en **Rust** con el motor **Bevy**. Compila a **nativo** y a **WebAssembly**.

## Estructura

- `tools/extract_data.py` — parsea `pack/serbi.as` (decompilado JPEXS) y genera `assets/data/gamedata.json` con toda la base de datos original: booths, precios, upgrades, visitantes, misiones, logros.
- `tools/compute_anchors.py` — calcula los anclajes isométricos de los sprites midiendo los bounding boxes alfa de los frames exportados.
- `assets/` — sprites PNG y sonidos extraídos del SWF original + datos JSON.
- `src/` — el port en Rust/Bevy.

## Modelo isométrico (del original)

- `SIZE_ = 24`: tile de 48×24 px. Vértice N del tile `(tx,ty)` en pantalla: `((tx-ty)*24, (tx+ty)*12)`.
- Reloj del juego: `counter += speed` por frame; `sec = counter/FR`, `min = counter/9`, `hora = min/60`.
- El juego empieza a las 08:00 del día 0 con $10.000.
- Visitantes: aparición probabilística según hora (`PROB_RANGE = [50,40,30,25,15,5]`) y popularidad.

## Build

```bash
# nativo
cargo run --release
# web
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/resort_empire.wasm
```

> Los assets provienen del juego original y se incluyen solo para uso personal.
