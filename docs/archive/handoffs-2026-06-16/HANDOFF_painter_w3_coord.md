═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W3 (layers) — próximos blocos foundational
Autor: Implementador Painter (sessão 2026-05-31) · você roda em CONTEXTO SEPARADO
Plano: docs/Painter_projeto/15_plano_de_implementacao.md §6 · design: 02_layers.md
Regras: docs/IntegracaoMultiAgente/DIRETRIZ.md (§3.B scaffold central, §3.C foundational, §4 ADR, §8 ship)
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ A FUNDAÇÃO + O SCAFFOLD da W3 já estão prontos e verdes (commits     ║
║ abaixo). Faltam 3 peças que SÓ você pode fazer (foundational):       ║
║  1. Integração tool↔LayerStack  (ARQUITETURA — ratifique 1ª)         ║
║  2. Compositor GPU em ph2d-render (paridade c/ meu CPU reference)    ║
║  3. Decisão de layout do dock (sidebar de brush vs painel de layers) ║
║ Depois eu (implementador) preencho os rows do painel + estendo o     ║
║ compositor com mask/clipping. Detalhe de cada uma abaixo.            ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
JÁ PRONTO (commits LOCAIS não-pushados — confirme no git log)
───────────────────────────────────────────────────────────────────
- `33fb4eb` — T3.3 **22 blend modes** + T3.1 **LayerStack**:
  · `ph2d-painter-brush/src/blend.rs`: `BlendMode` (#[repr(u8)] wire-stable +
    from_u8/to_u8 + `MAX_BLEND_MODES` + serde) + `apply(mode, dst, src)` em
    straight **linear-sRGB**, W3C Compositing L1 (separável + 4 HSL não-
    separável + Behind/Clear). **Esta `apply()` é a FONTE DE VERDADE da
    matemática** — o shader GPU tem que bater bit-a-bit. Re-export:
    `ph2d_painter_brush::{BlendMode, MAX_BLEND_MODES, apply_blend}`. 17 testes.
  · `ph2d-tool-painter/src/layers.rs`: `LayerStack` (arena + z-order top→bottom
    + grupos by-id). `LayerKind={Raster,Mask,Group}`; clip/reference/alpha-lock
    são flags em `Layer` (modifiers, §2.1). `MAX_GROUP_DEPTH=8`, `HARD_CAP=999`,
    serde-ready. 9 testes.
- `1bed40e` — T3.2 **compositor CPU** (`ph2d-tool-painter/src/compositor.rs`):
  `composite` + `composite_region` (dirty rect), top-down recursivo, blend +
  opacity + visibility + grupos. `LayerPixelSource` trait + `MapPixelSource`
  (BTreeMap, HR-5). 8 testes (incl. dirty-rect==full + opacity linear-space).
- `efe59b9` — T3.4 **scaffold do painel** (`ph2d-panel-painter-layers`, VOCÊ
  via agente; revisado+commitado por mim): typed Panel<State> + chrome canon +
  body placeholder + `// TODO(impl W3.T3.4)` markers. Wiring: ids editor-core
  (`PAINTER_LAYERS_PANEL`/`_CLOSE`), slot `HeroLayout::painter_layers`, registro
  no `panel-registry-init` (codegen ph2d-panel-sync, testes 4/4), feature shell,
  `set_current_layers()` publish API. **Painel está DORMENTE** (visibility=false
  em painter_bridge.rs) — ver bloco 3 abaixo.

Verde sobre o diff: cargo check (todas + shell completo), nextest brush+tool,
no_literal_color, no_bare_byte_color, arch_color_space_typed, painter contract
surface, registry-init 4/4, clippy --all-targets -D warnings.

═══════════════════════════════════════════════════════════════════
BLOCO 1 (ARQUITETURA — ratifique antes de codar) — tool↔LayerStack
═══════════════════════════════════════════════════════════════════
**Problema:** hoje `PainterTool` tem UM `canvas_rgba: Arc<Vec<u8>>` (canvas
flat da sprite ativa). Strokes pintam nele; o bridge lê `current_preview()` →
blita na sprite. Layers exigem: pintar na **layer ATIVA**, exibir o
**composite**, e undo por-layer.

**Abordagem RECOMENDADA (ratifique / ajuste / ADR-amendment 0043?):**
- `PainterTool` ganha `layers: LayerStack` + `images: BTreeMap<LayerId,
  LayerImage>` (a impl de `LayerPixelSource`; `LayerImage` já existe em
  `compositor.rs`). Substitui `canvas_rgba` como fonte de verdade.
- **Back-compat:** `set_source(sprite)` inicializa o stack com **1 raster
  layer** = os pixels da sprite → comportamento de hoje é o caso N=1.
- Strokes (`begin_stroke`/`queue_pointer`/`apply_stamps`) miram o buffer da
  **layer ativa** (`stack.active()`), não um canvas único.
- `current_preview()` retorna `composite(&stack, &images, w, h)` (CPU por ora;
  GPU = bloco 2). `on_deactivate` commita o composite na sprite (HR-6 blake3,
  igual hoje, mas sobre o composite).
- **Undo (§2.13):** o ring de 250 frames passa a snapshotar o buffer da layer
  ATIVA (+ a op de stack p/ add/remove/reorder). Hoje snapshota `canvas_rgba`.

**Fronteira de execução (quem faz o quê):**
- VOCÊ: ratifica a abordagem (+ ADR-amendment se mudar contrato/cap), e faz a
  parte SHELL — `shells/desktop/src/render_loop/painter_bridge.rs`
  (`current_preview` drain, writeback, e flipar `set_current_layers(None)` →
  `Some(painter.layers().clone())` + visibility) e qualquer contrato de
  preview que mude.
- EU (implementador): faço a parte INTERNA da tool em `ph2d-tool-painter`
  (campos `layers`/`images`, redirecionar stroke→layer ativa, `current_preview`
  = composite, migração `set_source`, undo por-layer) DEPOIS que você ratificar.
  Não toco shell/contrato — paro e reporto se precisar.

**Decisões que preciso de você (ou ADR):**
- (a) `LayerImage` buffers ficam no `PainterTool` (RAM) com eviction §2.13, ou
  numa estrutura shell-side / GPU cache? Recomendo no tool por ora (CPU path),
  GPU cache quando o bloco 2 landar.
- (b) Persistência do stack no `.ph2d-painter` (`ph2d-painter-stroke/
  persistence.rs`) — tipos têm serde; formato cooked + cook-hash re-lock é
  território congelado → seu/ADR.
- (c) Undo de op-de-stack (add/remove/reorder) entra no mesmo ring ou num
  paralelo? Recomendo mesmo ring (transações), espelhando `ImageEditTransaction`.

═══════════════════════════════════════════════════════════════════
BLOCO 2 — Compositor GPU em ph2d-render (perf)
═══════════════════════════════════════════════════════════════════
- Espelhar o `composite()` CPU num shader WGSL (sibling do `stamp.wgsl`),
  **bit-paridade** com `ph2d_painter_brush::apply(mode, dst, src)`. Minha
  blend.rs é a fonte de verdade; replique os 22 modos + a fórmula W3C +
  decode sRGB→linear / encode linear→sRGB (use a MESMA transfer function que
  `ph2d_color::srgb` — igual ao fix de gamma do stamp). Adote um gate de
  paridade tipo `shader_oklab_coefficients_bit_identical_with_rust`.
- Cache `BTreeMap<LayerId, CachedTexture>` (HR-5) + dirty-rect (minha
  `composite_region` é o reference de correção) + eviction LRU.
- Gates a criar (plano §2.12): `layers_composite_50_4k_under_5ms`,
  `layers_dirty_rect_correctness` (já tenho o reference CPU p/ comparar),
  `layers_no_alloc_hot_compose` (HR-3), `layers_max_count_per_budget`.
- **GAP que eu deixei aberto:** `layers_blend_mode_golden` (SSIM ≥ 0.9999 vs
  rasters do Photoshop) — não tenho assets de referência. Por ora cobri com
  testes de correção matemática W3C na blend.rs. Decida: arranjar rasters
  externos, ou aceitar os math-tests como o gate.
- ph2d-render é tua área (KTX2 W2 acabou de commitar `9745772`) — sem colisão
  com minhas crates.

═══════════════════════════════════════════════════════════════════
BLOCO 3 — Layout do dock (decisão pequena, mas bloqueia o painel ficar visível)
═══════════════════════════════════════════════════════════════════
O painel de layers e o sidebar de brush **compartilham o mesmo slot de dock
do Inspector** (`HeroLayout::painter_layers == painter_sidebar == inspector`).
Por isso deixei o painel **DORMENTE** (`painter_bridge.rs`: `panel_visibility
.insert("painter_layers", false)`) — senão ele cobre Color/Size/Opacity/
eyedropper do sidebar. Decida o layout (Procreate: layers = toggle top-right
separado do brush settings):
- (A) Side-by-side: layers à esquerda do sidebar (dois slots de dock à direita).
- (B) Stacked: layers em cima, brush settings embaixo, mesmo slot.
- (C) Toggle: um botão alterna entre brush-settings e layers no mesmo slot.
Quando decidir, adiciono o slot/toggle + flipo a visibility no fill (bloco do
implementador).

═══════════════════════════════════════════════════════════════════
DEPOIS (eu, implementador, in-scope — assim que os blocos acima permitirem)
═══════════════════════════════════════════════════════════════════
- **T3.4 fill:** rows reais do painel (thumb + name + visibility + opacity
  slider + blend dropdown usando `BlendMode`/popover order) em
  `ph2d-panel-painter-layers/src/{paint,event,populate}.rs` (markers
  `// TODO(impl W3.T3.4)`). Posso fazer a LÓGICA já (testável contra um
  LayerStack de teste) mesmo antes do bloco 1, e ela "acende" quando o
  snapshot real (bloco 1) + visibility (bloco 3) landarem.
- **T3.5 mask / T3.6 clipping:** estendo o compositor CPU (pontos de extensão
  comentados; hoje mask layer é skipped, sem clip).

═══════════════════════════════════════════════════════════════════
GIT / SHIP / REDS
═══════════════════════════════════════════════════════════════════
- Meus commits W3 são LOCAIS (não pushei — você faz ship 1× por jornada, §8).
  Cadeia desta jornada (Painter): swatch `c43d5d7` → T2.4/T2.6 `59555b7` →
  eyedropper-ícone `602f32d` → eyedropper funcional `cb976b3` → W3 found.
  `33fb4eb` → compositor `1bed40e` → scaffold painel `efe59b9`.
- **REDS pré-existentes** (verifique se ainda existem pós teu commit KTX2
  `9745772`): shell HR-18 LOC cap (`app_state.rs`=617, `render_loop/
  inspector_commits.rs`=616) — não meus; e clippy `ph2d-asset` doc-lint (era
  tua WIP KTX2, talvez já resolvido no `9745772`). Confere no `ship.sh`.
- Handoffs irmãos vivos: `docs/HANDOFF_painter_eyedropper_coord.md` (bug do
  dismiss do picker — JÁ resolvi end-to-end em `cb976b3`; tem um item de
  unificação opcional + nice-to-have de ícone dropper dedicado se quiser).
═══════════════════════════════════════════════════════════════════
