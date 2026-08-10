# Handoff de integração — `line/Vector`: Sketch + Hatch (Live Path Effects)

**Data:** 2026-07-25 · **Branch:** `line/Vector` · **Estado:** fechado, **pendente de smoke do Enio**.

Increment sobre a linha REABERTA (depois de Falloff/Twist/Knot e de Color Harmonies — este é o
3º increment aberto na linha; ver os handoffs irmãos). Um commit. Todo o trabalho é no motor
`ph2d-vec-scene` + o bump do teto de painel + a cena de smoke na shell.

## O que landou

Os dois últimos itens da Faixa B da pesquisa `20_*` que eram genuinamente NOVOS (Roughen e Pucker
já existiam — Roughen é o toggle "Rough" do Zig Zag, Pucker é o Bloat com `amount` negativo).

- **Sketch** (`fx_sketch.rs`) — o traço à mão. Cada contorno vira **N passadas**, cada uma
  deslocada pela NORMAL por um **ruído coerente** de baixa frequência (value-noise do `splitmix64`
  + `smoothstep` entre offsets — treme suave, não serrilha). Passada k tem seed própria
  (`seed ^ k`) ⇒ as linhas quase-coincidem e **DIFEREM** = "à mão" (1 passada lê como erro, ≥2 como
  esboço — Inkscape/Rough.js). Params **Passes/Roughness/Detail/Seed**. **CONSOME o Falloff**
  (escala o tremor por-amostra, como o Zig Zag).
- **Hatch** (`fx_hatch.rs`) — hachura de preenchimento por **scanline clip**: roda a forma para as
  linhas ficarem horizontais, cruza com todas as arestas (contorno + buracos, **even-odd**), pareia
  spans, cada span vira um sub-contorno **aberto** de 2 vértices. **Cross** = 2ª família a 90°.
  Params **Angle/Spacing/Cross**. `takes_falloff = false` (recorta uma região, sem força por-ponto).

## A espinha (portas únicas, sem contrato tocado)

- Ambos são **multi-output** (padrão Repeater/Knot: `out = sketch_path(...)` / `hatch_path(...)`),
  saída = contorno primário + `subpaths`.
- **Sketch** reusa `ArcPath` (a porta única de "onde fica o arco s", do Trim/ZigZag) + o `splitmix64`
  (o gerador do Roughen/jitter — re-declarado, é O algoritmo, não uma pergunta com duas respostas).
- **Hatch MANTÉM o caminho original** (outline + fill + buracos) e **APENDE** as linhas aos
  `subpaths`. Isto só é correto porque o renderer **preenche só os contornos FECHADOS** (gate
  `an_open_contour_never_punches_a_hole_in_the_fill`, `ph2d-vec-render/src/lib.rs`) e traça TODOS ⇒
  resultado = forma preenchida + outline + linhas, tudo na cor do traço. Só-hachura = `fill = None`.
- ⚠️ **Ambos nascem NEUTROS** (`roughness == 0` / `spacing == 0`) — a lei `every_kind_is_born_neutral`
  (ADR-0132): um efeito recém-posto na pilha é no-op byte-idêntico até o artista o configurar, como
  o Zig Zag nasce com `amplitude == 0`. Sem isso o `Cow::Borrowed` do `cooked()` morre. Os defaults
  de forma (2 passadas / 45°) são o que ele toma quando o slider sobe de 0.

## Superfície tocada (tudo aditivo)

- `fx_sketch.rs` + `fx_sketch_tests.rs` · `fx_hatch.rs` + `fx_hatch_tests.rs` (NOVOS) + `pub mod` em `lib.rs`.
- `effect.rs`: 2 variants `Sketch`/`Hatch` (append), `is_neutral`/`takes_falloff`/`KINDS`/`from_kind`/
  `kind_index`/`apply` arms + consts `SKETCH_KIND`/`HATCH_KIND`.
- `effect_accessors.rs`: `as_sketch`/`as_hatch` (+_mut) + as 2 novas linhas em TODO None-arm exaustivo
  + label. `effect_params.rs`: as tabelas SKETCH/HATCH + get/set.
- `effect_tests.rs`: `PANEL_MAX_FX_KINDS` 19→21. `tests/fx_look.rs`: Twist/Knot passam a `len()-4/-3`,
  Sketch/Hatch = `len()-2/-1` + rows novas (probe `#[ignore]`).
- **`MAX_FX_KINDS` 19→21** em `ph2d-editor-core/src/ids/chrome/vector.rs` (o painel — ids do Add-menu
  são hash em runtime, sem array). O painel se auto-popula de `KINDS`/`params` ⇒ as 4 condições de UI
  saem de graça (Add-menu ganha 2 entradas; os cards saem da tabela de params).
- Shell: `sketch_hatch_smoke.rs` + `mod` + dispatch (levels 31/32).

## Contrato/schema

**Nenhum `PROJECT_SCHEMA`** (variants apendados = postcard posicional preservado). **`PathEffect` NÃO é
§6** (é o motor novo, não congelado). Contrato do doc congelado (`ph2d-vector-doc`) intocado. **Sem ADR.**

## Gates (todos verdes, mutação-provados onde marcado)

- **Sketch (5):** neutro byte-idêntico · produz N passadas · **as passadas DIFEREM** (mutação:
  mesma seed por passada → delta 0, RED) · o tremor escala com a Roughness · `takes_falloff` true.
- **Hatch (6):** neutro · enche o interior e mantém o outline (pontas dentro da caixa) · **o BURACO
  parte o span** (mutação: ignorar buracos → 1 span onde deviam ser 2, RED — a fixture even-odd) ·
  cross dobra as linhas · aberto = intacto · `takes_falloff` false.
- Catálogo: `every_effect_kind_is_reachable` (`from_kind(i).label() == KINDS[i]`) ·
  `the_engine_and_panel_agree_on_the_kind_ceiling` (21) · `every_kind_is_born_neutral`.
- LOC (workspace + shell) · clippy `--all-targets` (vec-scene/editor-core/shell) · `check --workspace`
  · `node_id_collisions` (inclui o gate de ids dinâmicos de vetor até MAX_FX_KINDS).

## Smoke (números MEDIDOS pela sonda headless)

- **`PH2D_BUILD_SMOKE=31 cargo run -p ph2d-host-desktop --release`** (Sketch): 3 estrelas — limpa |
  2 passadas 4% (herói, selecionado → card Sketch) | 3 passadas 7%. Medido: 2 passadas = 72 verts,
  3 passadas = 144 verts.
- **`PH2D_BUILD_SMOKE=32 cargo run -p ph2d-host-desktop --release`** (Hatch): 3 discos PREENCHIDOS —
  liso | hachura 45° 8% (herói → card Hatch) | cross. Medido: 13 linhas, cross = 26 (2× exato).

## Aberto / decisões de produto (não construído sem pedido)

- **Sketch:** overshoot nas pontas de traço aberto (o "ends overshoot" do lápis de construção) —
  deferido; o wobble já lê como à mão sem ele.
- **Hatch:** `keep_outline` toggle (hoje o outline fica sempre — o padrão Illustrator); fase/offset
  da grade de linhas (hoje ancorada a `k*spacing`); Falloff a modular densidade/comprimento das
  linhas (recusado por ora — Hatch não tem força por-ponto). Todos são knobs a mais, não bugs.
