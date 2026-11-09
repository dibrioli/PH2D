# ph2d-outline-trace — picture alpha → collision outline (plan f2, W1)

Pure leaf (no ECS, no rapier). Plan: `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md`.
Oracle: `docs/GameEngineFuncional/oraculo_contorno/` (Godot 4.7.2, MIT). Tests: `tests/oraculo.rs`.

## Pipeline (all integer, fixed order → same bytes on any machine)

1. `Mask::from_rgba8` (α ≥ cutoff) · `Mask::union` (byte OR: one outline for every frame).
2. Marching squares along pixel edges, row-major scan; solid on the LEFT; at a diagonal pinch the walk
   turns toward the solid (4-connected). Holes come out with negative area.
3. Rings below `min_island_px` (staircase area) drop: islands go, holes fill.
4. **Outward** Douglas–Peucker in Q4 (owner 09/11, «SIM» to «the outline may stand a little OUT of the
   picture to use fewer points»): Detail = the most the outline may stand OUT of the art, never in (into a
   hole counts as out). A chord stands when the strip holding the vertices it skips is ≤ `detail` wide; its
   edge is the chord's line pushed to the empty side until no skipped vertex is out. Kept vertices become
   the meeting point of the two pushed lines (exact `i128`, rounded to nearest Q4); a meeting point
   farther than `detail` from its staircase vertex splits both chords. Edges that meet any other edge
   (except at a shared endpoint) split; then every pixel centre beside the staircase is checked inside
   the polygons, and the chords around a miss split. ⇒ **0 opaque pixels outside, always; deviation ≤
   Detail + 1 Q4** (the Q4 rounding of a meeting point, `tests/fora.rs`).
5. `convex_pieces`: each hole bridged to the nearest mutually visible vertex (in-cone at both ends, no
   edge met), ear clipping with exact predicates, Hertel–Mehlhorn (published algorithm; `swap_remove`
   order so the result equals `parry2d::transformation::hertel_mehlhorn_idx` — in-process oracle), then
   neighbours sharing an edge merge into their hull when every vertex is within `detail` of it and no
   hole vertex is strictly inside.

## Outward law (09/11) — before × after, `tests/fora.rs` + `tests/oraculo.rs`

Vertices at Detail 0.5 · 2 · 4 px (before = the inward-only law of 08/11, table below it). The disc of
`fora.rs` (r = 20 px): 92 points at every Detail before; now 92 · 14 · 8 · 7 at 0 · 2 · 4 · 8 px.

| case | before v | after v | after pieces | after empty covered | Godot v (cuts in) |
|---|---|---|---|---|---|
| f00_parado | 166·57·44 | 166·26·12 | 79·5·2 | 0·4.9·10.3 % | 144·29·24 |
| f11_corre | 202·52·45 | 202·33·13 | 101·8·3 | 0·4.9·11.4 % | 168·35·24 |
| f20_agachado | 136·34·28 | 136·17·12 | 68·3·2 | 0·5.1·8.2 % | 114·26·16 |
| uniao_corrida_8 | 172·49·43 | 172·24·13 | 85·5·3 | 0·4.0·8.2 % | 146·37·20 |
| uniao_folha_32 | 176·51·36 | 176·36·15 | 86·6·1 | 0·2.4·5.7 % | 150·41·17 |
| sint_L | 6·6·6 | 6·6·6 | 2·2·2 | 0·0·0 | 6·6·6 |
| sint_anel | 288·77·59 | 288·38·16 | 159·21·8 | 0·6.7·17.0 % | 164·28·22 (no hole) |
| sint_crescente | 192·52·38 | 192·42·12 | 90·12·5 | 0·6.2·16.5 % | 168·34·24 |
| sint_moeda_ilha | 136·32·32 | 136·16·12 | 65·2·2 | 0·8.0·10.8 % | 108·28·22 |

Godot (fixture of 08/11, same inputs, same epsilons) leaves 1–9 % of the opaque pixels OUT; ours leaves 0
and uses FEWER points than Godot at 4 px in every case (L equal); at 2 px too, except the crescent (42 × 34) and the ring (38 × 28 — ours keeps the hole). Hole covered: 0 · 4.3 · 12.4 % of the
hole pixels. Test bound: empty covered ≤ 17 % (worst: sint_anel at 4 px, 594/3496).
Time (µs, minimum of 15, `--profile smoke`, trace + pieces, 0.5 · 2 · 4 px): f00 319·64·42 · f11 421·72·45 ·
f20 236·47·39 · corrida_8 338·61·45 · folha_32 352·87·53 · L 30·29·29 · anel 831·97·58 · crescente
392·112·41 · moeda_ilha 209·31·25.

| mutation (09/11) | red |
|---|---|
| no outward push (chord stays, cuts in) | coin hash, hero hash, coin piece count, ceiling (the pixel-centre guard heals the cut: 0 out) |
| Detail ignored (every chord stands) | coverage, deviation, tiling, hole, hashes |
| no pixel-centre guard | `no_opaque_pixel…` (crescent tips: 4 opaque pixels out at 2 px) |
| no miter bound | `nenhum_pixel…_desvio…` (deviation), hashes |
| no edge-conflict pass | `a_small_hole_is_never_merged_over` |

## Ours × Godot (α 0.5; detail/epsilon 0.5 · 2 · 4 px; measured 08/11, the inward-only law)

Vertices / convex pieces / % opaque outside / % empty covered. Godot: `cobertura.txt` + fixture.

| case | ours v | ours pieces | ours out | ours empty | Godot v | Godot pieces | Godot out |
|---|---|---|---|---|---|---|---|
| f00_parado | 166·57·44 | 79·6·3 | 0·0·0 | 0·2.9·4.2 | 144·29·24 | 81·12·11 | 0·4.1·5.1 |
| f11_corre | 202·52·45 | 101·6·4 | 0·0·0 | 0·3.2·4.6 | 168·35·24 | 81·13·8 | 0·2.9·8.5 |
| f20_agachado | 136·34·28 | 68·3·2 | 0·0·0 | 0·2.7·4.0 | 114·26·16 | 57·10·6 | 0·2.8·8.8 |
| uniao_corrida_8 | 172·49·43 | 85·6·4 | 0·0·0 | 0·2.5·3.7 | 146·37·20 | 83·15·6 | 0·2.0·6.7 |
| uniao_folha_32 | 176·51·36 | 86·6·2 | 0·0·0 | 0·2.0·3.4 | 150·41·17 | 80·16·6 | 0·1.2·4.2 |
| sint_L | 6·6·6 | 2·2·2 | 0·0·0 | 0·0·0 | 6·6·6 | 2·2·2 | 0·0·0 |
| sint_anel (hole kept) | 288·77·59 | 159·19·12 | 0·0·0 | 0·5.3·7.2 | 164·28·22 (no hole) | 102·12·8 | 0·4.0·6.5 |
| sint_crescente | 192·52·38 | 90·10·4 | 0·0·0 | 0·5.3·10.3 | 168·34·24 | 70·11·9 | 0·3.8·7.7 |
| sint_moeda_ilha | 136·32·32 | 65·2·2 | 0·0·0 | 0·3.0·3.0 | 108·28·22 | 61·9·9 | 0·3.7·9.4 |

Godot's ring covers 100 % of its hole (43.7 % of the covered area is empty); ours covers 0 · 6.0 · 10.3 %
of the hole pixels (the rim moves ≤ detail into it). Coin + island at 2 px: 2 pieces = coin 1 + island 1.
Bound in the test: empty covered ≤ 11 % (worst: crescent at 4 px, 10.3 %); hole covered < 15 %.

## Time (µs, minimum of 15, interleaved, `--profile smoke`, trace + pieces)

| case | 0.5 px | 2 px | 4 px | Godot (3 calls) 0.5 · 2 · 4 |
|---|---|---|---|---|
| f00_parado | 571 | 89 | 63 | 231 · 98 · 96 |
| f11_corre | 626 | 77 | 66 | 284 · 104 · 98 |
| f20_agachado | 303 | 51 | 41 | 179 · 85 · 81 |
| uniao_corrida_8 | 619 | 83 | 69 | 263 · 117 · 107 |
| uniao_folha_32 | 465 | 75 | 59 | 295 · 137 · 125 |
| sint_L | 32 | 31 | 31 | 86 · 86 · 87 |
| sint_anel | 1143 | 128 | 93 | 305 · 111 · 109 |
| sint_crescente | 561 | 70 | 54 | 298 · 90 · 86 |
| sint_moeda_ilha | 215 | 31 | 28 | 131 · 60 · 57 |

0.5 px is the O(S²) chord-conflict pass and the O(n²) ear search over ~170–290 vertices; at the default
2 px every case is ≤ 128 µs. Re-run: `cargo test -p ph2d-outline-trace --profile smoke -- --ignored timing --nocapture`.

## Ceilings (W0, 09/11) — `MAX_PIECES = 256`, `MAX_POINTS = 512`, `trace_fitted`

Measured by `crates/ph2d-physics/tests/it/measure_outline_pieces.rs` (`--profile smoke`, minimum of 7
interleaved rounds, the product's `PhysicsWorld::step`: 4 sub-steps, our solver parameters). Fixture: a
comb with its teeth down (bar 12 px, teeth 3 × 12 px, gap 3 px), 64 px/m, Detail 2 px — every piece
touches the floor, the worst case of contacts per piece. Base: a box of the same size, 3.6 µs (alone)
and 5.0 µs (two stacked).

| teeth | pieces | outline points | trace + pieces | build compound | one, resting | two, stacked |
|---|---|---|---|---|---|---|
| 16 | 17 | 68 | 0.11 ms | 7 µs | 21 µs | 41 µs |
| 64 | 65 | 260 | 1.02 ms | 26 µs | 77 µs | 154 µs |
| 128 | 129 | 516 | 3.56 ms | 53 µs | 151 µs | 318 µs |
| 256 | 257 | 1028 | 13.4 ms | 100 µs | 300 µs | 608 µs |
| 512 | 513 | 2052 | 53.7 ms | 190 µs | 602 µs | 1238 µs |

- **`MAX_PIECES = 256`** — resource: the solver tick, ~1.2 µs per touching piece per tick (0.30 ms at 257).
- **`MAX_POINTS = 512`** — resource: the retrace, synchronous in the frame of the gesture (dragging Detail)
  and of keyed Detail in Play; 3.56 ms at 516 points, under a quarter of a 60 Hz frame. O(points²).
  Outward law (09/11, `fora.rs::tempo_do_pente_do_tecto`, trace + pieces at 2 px, minimum of 7): the comb's
  points are unchanged (68 / 260 / 516 / 1028 at 16 / 64 / 128 / 256 teeth: 3 px gaps do not bridge at 2 px);
  time 0.11 / 1.39 / 4.83 / 18.5 ms — the pixel-centre guard (O(border × points)) costs +36 % at 516 points,
  29 % of a 60 Hz frame. The ceiling stands; ⛔ open: the guard only needs the runs next to a moved vertex.
- Above either, `trace_fitted` raises Detail to the SMALLEST value that fits (doubling, then bisection);
  only when Detail spans the picture does Ignore specks under rise. Never cuts the art (`tests/tecto.rs`).
- ⭐ **The pieces were CUBIC before this measure** (761 ms at 513 teeth, 14.9 ms at 129): Hertel–Mehlhorn
  scanned every later polygon for each edge's twin, and the merge rescanned from `(0, 1)` after every
  merge. An edge→owner index and a resumed scan give the SAME sequence (one owner per directed index edge;
  after a merge only piece `i` is new) — byte-identical over 21 masks × 3 Details × 2 speck settings
  (16 107 pieces, one sha256), and `hero_union_outline_is_byte_stable` unchanged. 14× at 512 teeth.
- ⛔ Not done: the remaining O(points²) (ear search, chord-conflict pass) — a lower order moves `MAX_POINTS`.
- W2a (09/11): the measure now goes through the PRODUCT path (`register_outline` + a `ShapeDesc::Outline`
  body), and «build» is register + spawn in an empty world: 11 / 39 / 76 / 150 / 285 µs at 17 / 65 / 129 /
  257 / 513 pieces (the content hash and the body ride on top of the bare compound). The ticks are unchanged
  (21 / 77 / 151 / 299 / 599 µs resting, 41 / 155 / 319 / 607 / 1239 µs stacked), so both ceilings stand.

| mutation (09/11) | red |
|---|---|
| no Detail search (straight to specks) | `acima_do_tecto_o_detail_sobe_ao_menor_que_cabe` |
| no bisection (first doubling wins) | idem (192 → 256 q4: not the smallest) |
| no specks step | `ilhas_demais_sobem_o_ignore_specks_depois_do_detail` |

## Mutation proofs (08/11; each restored, tree clean)

| mutation | red |
|---|---|
| two-sided DP (violator ignored) | `no_opaque_pixel…`, tiling, counts, golden |
| holes not bridged | `the_ring_keeps_its_hole`, tiling |
| merge without the hole check | `a_small_hole_is_never_merged_over` |
| scan rows reversed | golden, counts |
| direction order per vertex reversed | survives: rings are rotated to the smallest point and sorted, so it cannot change bytes |
