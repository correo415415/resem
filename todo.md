# Resort Empire — Port a Rust/Bevy — TODO

Objetivo: port jugable 1:1 del juego Flash original (Little Giant World), nativo + WASM,
usando el arte original extraído del SWF. Compilación SIEMPRE en el runner self-hosted
(GitHub Actions), nunca en el sandbox. Mini-PRs por hito.

## Hecho (PRs #1–#4 mergeados)
- [x] Extracción de datos del AS3: gamedata.json (serbi.as), anchors2.json (medidos del arte), sprites, sonidos
- [x] Matemática isométrica (rumus.as) + mapa por defecto (MapContainer.buildMap + zmap)
- [x] Cámara pan/zoom, reloj de juego (velocidades x1/x3/x5, pausa), cartera ($10.000)
- [x] Sistema de construcción: menú, ghost preview verde/rojo, ocupación, costes
- [x] Visitantes: spawn probabilístico por franja horaria (PROB_RANGE), caminan al lobby y se van
- [x] CI: check rápido en cada push + build release (nativo + WASM) → Release `latest` + gh-pages
- [x] Primer test in-browser del WASM compilado (screenshot OK)

## En curso (este PR)
- [ ] Fix parseo BoothDef: `booked_price` puede ser número o lista → precios $0 en menú
- [ ] Fix Lobby/booths: dibujar también el sprite de pared/edificio (walls/N.png) sobre el suelo
- [ ] Fix HUD: quitar glifos no soportados por la fuente por defecto (Día/☀/🌙 → ASCII)
- [ ] Añadir dist_test/ a .gitignore

## Siguiente — hacia el 1:1 visual
- [ ] Animación de visitantes: ciclo de andar con los 4 frames por skin (visitorN/1..4.png) + flip por dirección
- [ ] Extraer el arte de la UI original del SWF (paneles, botones, iconos, barra inferior de construcción,
      panel superior de dinero/día) y reconstruir el HUD/menú con ese arte (el menú actual es PLACEHOLDER)
- [ ] Fuente: usar/extraer la tipografía del juego o una equivalente con acentos (Día, etc.)
- [ ] Paredes alpha (wall_alpha) al pasar el ratón por detrás de edificios
- [ ] Coche de entrada (car/1..6.png) y llegada de visitantes por carretera como el original
- [ ] Pantalla de título / menú principal del original

## Gameplay pendiente
- [ ] Economía: reservas de habitaciones (booked_price), horarios opened/closed, salarios, ingresos por día
- [ ] Interior de habitaciones (frames wall 13/16 estilo interior) y estados ocupado/libre
- [ ] Empleados: janitors + sistema de basura (trash/1..3.png)
- [ ] Humor de visitantes (Smiley), popularidad, drainpop
- [ ] Expansión de terreno (Expand/ExpandPrice)
- [ ] Misiones y logros (Mission/Achievements de gamedata)
- [ ] Upgrades de booths (price[1], price[2], niveles)
- [ ] Guardado/carga (localStorage en WASM, archivo en nativo)
- [ ] Sonidos y música (assets/sounds ya extraídos)

## Infra
- [ ] GitHub Pages: API sigue devolviendo 403 con el PAT → habilitar manualmente en
      Settings → Pages → Deploy from branch → gh-pages / (root), o revisar permiso "Pages: Read and write"
- [x] Cuota de artifacts (500MB) resuelta: Release rodante `latest` + gh-pages
