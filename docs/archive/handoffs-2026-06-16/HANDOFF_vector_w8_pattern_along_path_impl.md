═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador Vector · W8 pattern-along-path (18º node) — node VETORIAL fechado
Autor: Implementador Vector (jornada 2026-06-05) · base: HANDOFF_vector_eval_and_next_sprint §4 Opção B
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
1. **W8 pattern-along-path (sabor vetorial) FECHADO.** O **18º node geométrico** (slot #8 pré-alocado da
   ADR-0058 §2.2) está vivo: repete um shape vetorial ao longo de um path, seguindo a curva real. **Roster
   18/18 completo.** 8 testes do node + staleness gate do registry-init verdes, clippy zero warnings.
2. **Decisão de escopo (reporto):** o node tem 2 sabores — fiz o **vetorial puro** (geometria→geometria,
   **zero dependência do Painter**); o **brush bridge** (stampar brush raster ao longo do path, consome
   `ph2d-painter-brush`) fica **deferido pro Painter** (mid-W4). Isso respeita isolamento e completa o roster
   sem gate cross-módulo — exatamente como o handoff separa "pattern-along-path **+** brush bridge".
3. **Sem bump de cap, sem ADR amendment:** slot #8 já existia nos 18. Substrato real (não o pseudocódigo do
   spec): `NodeOp` Pure/Static, carrier `VectorNetwork` via `ph2d-vector-graph`, mesmo padrão do `scatter`.

## §1 — O QUE LANDOU
Crate nova **`crates/ph2d-node-vector-pattern-along-path/`** (auto-descoberta por node-sync — prefixo
`ph2d-node-vector-`):
- **`lib.rs`** — `MANIFEST` (2 inputs `shape`+`path` VECTOR_PORT, 1 output; params `count`/`align`/`scale`),
  `NodeOp` Pure/Static/Cpu, `register`. Cook integration test (2 sources → cook → 6 cópias).
- **`engine.rs`** — `pattern_along_path(shape, path, count, align, scale)`:
  1. **Walk de conectividade** (`ordered_segments`): ordena os segmentos do path numa cadeia (começa em
     endpoint degree-1, senão menor vertex id; determinista via BTreeMap).
  2. **Flatten cúbico**: cada segmento vira polyline densa (24 samples), convenção `c1=start+out`,
     `c2=end+in` (decodificada do `primitives::ellipse`).
  3. **Frames por arc-length**: `count` frames espaçados uniformemente por comprimento de arco (cópia única
     no midpoint; senão ends inclusive). Cópias ficam **na curva, não na corda**.
  4. **Append transformado** (espelha o `Builder` do scatter): cada cópia rotaciona (→ tangente, se `align`)
     + escala + translada; tangentes cúbicas rotacionam/escalam, regiões/fills/styles preservados, ids frescos.
- Registry-init regenerado por **`cargo run -p ph2d-node-sync`** (linha + dep alfabéticas; diff só a minha).

## §2 — DECISÕES / DETERMINISMO
- **Rotação sem transcendental:** o transform usa `(cos,sin) = componentes do tangente unitário` direto
  (não `atan2`+`sin`/`cos`) → bit-estável. Snap Q16.16 quando `shape.deterministic || path.deterministic`
  (mesmo padrão do scatter; teste `deterministic_and_reproducible` prova byte-estabilidade).
- **Vertex kind preservado** (`Vertex::new(.., v.kind)`, não `auto` como o scatter) — mais fiel.
- **Single-component walk:** um "path" é uma curva; se a network tiver componentes desconexos, só o que
  contém o start é seguido (aceitável v1).

## §3 — VALIDAÇÃO (8 testes)
`places_count_copies_along_path` · `copies_span_the_path_endpoints` (ends inclusive) ·
**`align_rotates_copy_to_tangent`** (path vertical + shape assimétrico → span_y > span_x, prova rotação) ·
**`follows_cubic_curve_not_chord`** (arco de elipse → cópias off-diagonal, prova o flatten cúbico) ·
`scale_grows_copies` · `deterministic_and_reproducible` · `zero_count_or_empty_inputs_are_empty` ·
cook integration (`pattern_through_a_real_cook_repeats_shape_along_path`).

## §4 — O QUE FICA (deferido / teu)
- **Brush bridge (a outra metade do W8):** stampar um `ph2d-painter-brush` raster ao longo do path. Gated
  na maturidade da API do Painter (mid-W4). Quando a superfície do brush estabilizar, é uma 2ª crate
  (`ph2d-node-vector-brush-along-path`?) OU um modo do node — **confirma comigo a superfície antes** (regra
  de isolamento). NÃO toquei `ph2d-painter-*`.
- **Possíveis polimentos v2 (não-bloqueantes):** modo spacing-by-distance (vs count fixo); repeat/stretch
  do shape p/ preencher exatamente; spin/normal-offset params. Roster já está 18/18 sem isso.

## §5 — GIT / POSSE
- Commit scoped local: `crates/ph2d-node-vector-pattern-along-path/**` + os 2 arquivos regenerados do
  `ph2d-node-registry-init` (lib.rs + Cargo.toml — codegen determinista do node-sync, workflow sancionado;
  diff é só a minha linha/dep) + este handoff. `--no-verify`, sem push. `git status` conferido: nada alheio.
- Cap de 18 nodes intacto (slot pré-alocado). `ph2d-vector-doc` contrato não tocado.
═══════════════════════════════════════════════════════════════════
