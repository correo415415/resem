# Resort Empire — Port a Rust/Bevy — TODO

Objetivo: port jugable 1:1 del juego Flash original (Little Giant World), nativo + WASM,
usando el arte original extraído del SWF. Compilación SIEMPRE en el runner self-hosted
(GitHub Actions), nunca en el sandbox. Mini-PRs por hito.

Juego jugable en: https://correo415415.github.io/resem/ (gh-pages, se actualiza con cada build de main)

## Hecho (PRs #1–#8 mergeados)
- [x] Extracción de datos del AS3: gamedata.json (serbi.as), anchors2.json (medidos del arte), sprites, sonidos
- [x] Matemática isométrica (rumus.as) + mapa por defecto (MapContainer.buildMap + zmap)
- [x] Cámara pan/zoom, reloj de juego (velocidades x1/x3/x5, pausa), cartera ($10.000)
- [x] Sistema de construcción: menú, ghost preview verde/rojo, ocupación, costes
- [x] Visitantes: spawn probabilístico por franja horaria (PROB_RANGE), caminan al lobby y se van
- [x] CI: check rápido en cada push + build release (nativo + WASM) → Release `latest` + gh-pages
- [x] Primer test in-browser del WASM compilado (screenshot OK)
- [x] Fix precios $0 en menú (parseo booked_price número-o-lista)
- [x] Fix HUD ASCII, dist_test/ en .gitignore
- [x] GitHub Pages habilitado y desplegado
- [x] Alineación pared-suelo de booths por `n_corner` medido (PR #6) — verificado en el juego compilado
- [x] Animación de visitantes (PR #7): frames 1-2 = walk_front, 3-4 = walk_back,
      flip_x para L/R (port de gerakArah() de Visitor.as), ~6 fps escalado por velocidad del reloj
- [x] Orientación del mapa corregida (PR #8): el port anterior no negaba flash-y → mapa espejado
      verticalmente vs el original (entrada arriba en vez de abajo). Ahora world.y = +(tx+ty)*12,
      anclas de booth/ghost al vértice superior del footprint (tx, ty+cols-1), depth invertido.
      VERIFICADO in-game (screenshot del build a711fb1): carreteras de entrada abajo/adelante,
      arena construible arriba, lobby con la alfombra saliendo hacia el frente — igual que el original.

## Siguiente — hacia el 1:1 visual
- [x] Extraer el arte de la UI original del SWF: navigator1 (panel superior: logo, barra de título,
      barra RP, estrella de nivel, contadores), navigator2 (barra de herramientas izquierda),
      navigator3 (barra inferior: reloj, DAY, botones de velocidad, sliders, popularidad, dinero),
      reloj animasiJam y 18 botones con frames normal/hover → assets/sprites/ui/ (PR #11)
- [x] Limpiar el texto dinámico "horneado" en los exports (título, barra RP, DAY, popularidad,
      dinero, etiqueta del regalo, ticker) con rellenos planos del color de banda (PR #11)
- [x] Reconstruir el HUD en src/ui.rs con el arte original: paneles como ImageNode, overlays de
      texto dinámico (dinero con puntos de miles, DAY, POPULARITY, RP) y botones de velocidad
      clicables (play/1X/2X/3X) cableados a GameClock (PR #11) — verificado en screenshots
- [ ] Botones de la barra de herramientas izquierda (nav2) funcionales con frames hover
      (btn_scenery, btn_destroy, btn_tile, etc.) y menú de construcción con el arte original
- [x] Manecillas del reloj animadas sobre la esfera + caras día/noche (animasiJam) (PR #25)
- [x] Icono de regalo (gift) funcional: ahora muestra el bonus de la misión activa
      (MissionState.gift → HudField::Gift) (PR #33)
- [x] PULIR HUD vs original (reporte del usuario): barra superior + estrella — medidos los
      bboxes exactos en nav1_left.png (estrella (189,23)-(220,54), barra título gris
      (50,33)-(192,52), barra RP negra (134,54)-(192,66)); estrella centrada en caja 32×32,
      título en contenedor centrado 136×20, RP realineado (PR #29) — PENDIENTE verificar
      visualmente en el próximo build
- [x] Fuente: extraída la tipografía original del SWF (Starmap Truetype, 89 glifos ASCII)
      → assets/fonts/starmap.ttf, aplicada a los 9 nodos de texto del HUD (PR #29)
- [x] Fix espacio de palabras: el glifo espacio de starmap.ttf tenía advance 100/1024
      ("GREENISLAND", "POPULARITY0%") → parcheado a 460 con fontTools (PR #32)
      — PENDIENTE verificar visualmente en el próximo build
- [ ] Fallback de fuente con acentos: Starmap NO tiene á/é/í/ó/ú/ñ — añadir fuente de
      respaldo o evitar acentos en textos dinámicos
- [ ] Paredes alpha (wall_alpha) al pasar el ratón por detrás de edificios
- [x] Tráfico de carretera (port de Mobil.as): buses/furgonetas en las carreteras de entrada,
      bus de pasajeros con parada en (5,19), aceleración 2→rand(7..10) px/frame (PR #26)
      — VERIFICADO in-game (screenshot build 44cdf66): bus naranja circulando bien asentado
      sobre el asfalto, anchor (94,96) y profundidad correctos
- [x] Llegada de visitantes bajándose del bus (PR #29): evento BusArrival en car.rs —
      el bus de pasajeros (roll<20) calcula jumlahV = 1 + rand·clamp(pop·0.1, 1, 8) y lo
      emite al llegar a la parada (5,19); listener bus_arrivals en visitor.rs hace spawn
      de `count` visitantes en la acera (4,19) caminando al lobby (helper spawn_visitor
      compartido) — PENDIENTE verificar in-game
- [x] Variante Box van (45%, max 4 pasajeros, frame 5) + bajada 1-por-tick (0.25s) del
      original Mobil.activitiesOnTick (PR #31) — PENDIENTE verificar in-game
- [ ] Pantalla de título / menú principal del original

## Experiencia 1:1 desde el arranque (vídeo de referencia del usuario)
Vídeo del emulador (carga → fin del tutorial) guardado en AI Drive:
`/mnt/aidrive/referencias/resort_empire_tutorial_2026-08-28.mp4` (13.9MB, 2026-08-28).
La experiencia del port debe ser IDÉNTICA a esta secuencia. Fases con timestamps (ms):

- [ ] Preloader (0–2500): fondo gris con grid isométrico, barra de progreso central,
      texto "LOADING" azul; al 100% el logo Resort Empire pulsa/brilla y transiciona
- [ ] Splash 1 (2500–6000): fondo azul claro, mascota dragón verde + logo del sponsor
- [ ] Splash 2 (6000–9500): fondo negro, logo pixel-art "Little Giant World" + PRESENTS
- [ ] Menú principal (9500–14000): escena isométrica del resort de fondo, panel verde a
      la derecha con 4 botones dorados biselados PLAY / OPTIONS / CREDIT / MORE GAMES,
      "ver 1.1" abajo a la derecha
- [ ] Submenú PLAY (12000): el panel verde cambia a NEW GAME / LOAD GAME / BACK
- [ ] NEW GAME (13000–14000): modal "Input Your Name" + OK → barra "Please Wait...
      Initializing Objects" sobre el grid
- [ ] Tutorial (24000–64000): panel central pergamino con borde amarillo + avatar del
      manager (gorra roja) + botón "Click here to skip tutorial". Pasos:
      1. Intro (24s) · 2. objetivo: construir en arena dentro de los setos (28s)
      3. construir habitación (29–38s): flecha roja rebotando sobre el icono de
         Accommodation en la toolbar izquierda → colocar Cottage $1000 (X roja en
         tiles inválidos) · 4. tiles (40–48s): flecha al icono Tiles → conectar 2
         tiles walkable del cottage a la carretera · 5. plantas (48–56s): flecha al
         icono Plants → colocar Windbill Palm $40 · 6. upgrades (56s): flecha al
         icono estrella · 7. cámara y tips (58–62s): drag/WASD y botón "?" ·
         8. felicitación y cierre (63s)
- [ ] Durante el tutorial: los botones de la UI no señalados quedan bloqueados/grises
- [ ] Post-tutorial (64s): notificación "NEW MISSION" arriba a la derecha + objetivo
      en el ticker inferior (ya portado el motor de misiones, falta la notificación)
- [ ] Tooltip en ítems bloqueados del menú de construcción: "Need Resort Upgrade: ..."
- [ ] Menús con efecto pop-in de escala (Achievements, Statistics, Settings);
      Statistics = hoja de ingresos/gastos diarios

## Gameplay pendiente
- [ ] Economía: reservas de habitaciones (booked_price), horarios opened/closed, salarios, ingresos por día
- [ ] Interior de habitaciones y estados ocupado/libre
- [ ] Empleados: janitors + sistema de basura (trash/1..3.png)
- [ ] Humor de visitantes (Smiley), popularidad, drainpop
- [ ] Expansión de terreno (Expand/ExpandPrice)
- [x] Misiones (PR #33): motor CheckMissions portado — cola ordenada de serbi.Mission.listing
      (94 misiones), ticker del navigator3 con desc + contadores (n/goal), bonus junto al
      icono de regalo, flash "MISSION CLEAR!" 2s y pago del bonus al completar. Métricas
      soportadas: rooms/facilities/plants/rooms+facilities/money/visitors/tiles pintados.
      Misiones de sistemas no portados (bookings, janitors, upgrades…) se saltan de momento
- [ ] Misiones: métricas pendientes (bookings, janitors, upgrades, CheckRoomWithPath real
      con pathfinding en vez del proxy tiles_painted) + logros (Achievements de gamedata)
- [ ] Upgrades de booths (price[1], price[2], niveles)
- [ ] Guardado/carga (localStorage en WASM, archivo en nativo)
- [ ] Sonidos y música (assets/sounds ya extraídos)

## Infra
- [x] GitHub Pages habilitado (gh-pages / root) → https://correo415415.github.io/resem/
- [x] Cuota de artifacts (500MB) resuelta: Release rodante `latest` + gh-pages

---

# Archivos del original (scripts/) — procesados vs pendientes

Fuente: `juego.zip` → `source/scripts/` (AS3 descompilado con JPEXS).
"Procesado" = su lógica/datos ya están portados o extraídos a assets.

## Procesados (lógica portada o datos extraídos)
| Archivo original | Qué se extrajo / dónde vive ahora |
|---|---|
| `pack/serbi.as` | Base de datos completa → `assets/data/gamedata.json` (Booth/Tile/Scenery/Visitor/Employee/Smiley/Mission/Expand/Achievements) |
| `pack/rumus.as` | Matemática isométrica → `src/iso.rs` (SIZE_=24, findTile, findTileCoord) |
| `pack/zmap.as` | Layout por defecto (carreteras, lobby, sceneries, cangrejos) → `src/map.rs` (consts KEPITING/SCENERIES/zmap roads) |
| `pack/MapContainer.as` | buildMap (asfalto/acera/arena/hierba/grid, bounds, expand) → `src/map.rs::spawn_map` |
| `Application.as` (parcial) | Reloj (counter/speed/minuto), dinero inicial, PROB_RANGE de spawn, DefaultGameVars (unlocks iniciales) → `src/game.rs`, `src/visitor.rs`, `src/build.rs` |
| `pack/Instance/Booths/Booth.as` (parcial) | Footprint ROWS/COLS, colocación pared+suelo, ydepth → `src/map.rs::spawn_booth_at` |
| `pack/Instance/Plant.as` (parcial) | Colocación de scenery → `src/map.rs::spawn_scenery_at` |
| `pack/Instance/Visitor.as` (parcial) | Velocidad por skin, spawn probabilístico, jeda → `src/visitor.rs` |
| Sprites del SWF | → `assets/sprites/` (booths 57, walls 106, tiles, scenery, 29 visitantes ×4 frames, janitor, car 6, trash 3, groundselect) |
| Sonidos del SWF | → `assets/sounds/` (788K) |

## Pendientes de procesar (lógica aún no portada)
| Archivo original | Contenido pendiente |
|---|---|
| `Application.as` (resto, ~9000 líneas) | Economía completa: ingresos por reserva, día nuevo (salarios, informes), XP/level-up, popularidad, misiones, guardado SharedObject |
| `pack/Instance/Booths/Booth.as` (resto, 3343 líneas) | Colas (antri), entrada/salida de visitantes (enterArray), upgrades de nivel, apertura/cierre por hora, wall_alpha hover, demolición |
| `pack/Instance/Visitor.as` (resto) | Pathfinding real por tiles walkables, decisión de qué booth visitar, humor (Smiley), dejar basura, pagar |
| `pack/Instance/Janitor.as` | Empleado limpiador: patrulla, recoger basura, salario |
| `pack/Instance/Sampah.as` | Basura: aparición, efecto en humor/popularidad |
| `pack/Instance/Mobil.as` + `Car2.as` | Coche de llegada por carretera (animación de entrada de visitantes) |
| `pack/Instance/Tile.as` | Compra/colocación de tiles de suelo por el jugador |
| `pack/Instance/TemporaryObject.as` | Ghost/preview original de construcción (ya hay versión propia, comparar) |
| `pack/Instance/MoneyClip.as` | Popup flotante de dinero ganado/gastado |
| `pack/Instance/Notif.as` | Notificaciones en pantalla |
| `pack/Instance/Achievement.as` | Logros |
| `pack/Instance/Outer.as` | Objetos fuera del recinto |
| `GuideDialog.as` / `GuideInGame.as` | Tutorial |
| `Opening.as` / `Preloader.as` | Pantalla de título / intro |
| `Upgrade.as` | UI de mejora de booths |
| `Wall.as` / `BoothTile.as` / `GroundSelect.as` / `TileClip.as` | Clips de render (blitting) — sustituidos por sprites estáticos ya extraídos; revisar si tienen frames de animación útiles |
| `pack/bitmap/BlittingSingle.as` | Sistema de blitting (no se porta, Bevy lo reemplaza; sirve de referencia para offsets de canvas 240×136, dp=25,84) |
| `Seat.as`, `Area.as`, `Adding.as`, `Titik*.as`, `Blub*.as`, `Cengkling.as`, etc. | Auxiliares menores — revisar uno a uno |
| Sonido: `BGMusic1-5.as`, `ClickSound*.as`, `BuildSound.as`, etc. | Mapear cada clase de sonido a su mp3/wav extraído y reproducir en los eventos correctos |
| UI original (SWF `buttons/`, `images/`, `frames/`) | Extraer paneles/botones/iconos de la UI para reconstruir menús 1:1 |
| Booths individuales (`Lobby.as`, `Cottage.as`, `Sauna.as`, ... 21 clases) | Solo definen frames/comportamiento específico por booth; la mayoría es data ya en gamedata.json — revisar excepciones (Pool, Golf con áreas especiales) |
| `com/` y `fl/` (frameworks) | No se portan (librerías estándar de Flash) |
