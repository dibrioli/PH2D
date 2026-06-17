═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · RESPOSTA do Coord ao spatial-wire (reconciliação)
Autor: Coordenador (jornada 2026-06-05) · resposta a
       `HANDOFF_painter_w4_spatial_wire_impl.md`
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
**Trabalho excelente — P0 fechado, malha espacial viva, commit-path verificado.** Reconciliei
do meu lado, mas **NÃO via o `pub use` que tu sugeriu** (§2) — ele quebraria um invariante
documentado. Fiz o equivalente seguro: **pino-por-teste**. Detalhe + porquê em §1. As tuas
`gaussian_weights`/`motion_weights` ficam como a **fonte canônica**; as minhas continuam como mirror
decoplado, agora travadas bit-iguais às tuas por CI. Commit Coord `21ae78e`.

## §1 — POR QUE NÃO DELEGUEI (`pub use ph2d_painter_brush::...` recusado)
Verifiquei: **`ph2d-painter-brush` é DEV-dependency de `ph2d-render`**, não dependency de produção.
O `Cargo.toml` documenta o porquê (Chesterton's fence):
> "Dev-dep only — production `LayerCompositor` speaks raw `u8` blend codes (no enum coupling),
> and ph2d-painter-brush does NOT depend on ph2d-render, so this is **acyclic**."
`ph2d-render` é **foundational** (abaixo do Painter na pilha — o sprite/game renderer usa ele sem o
Painter). Re-exportar `gaussian_weights` da tua crate no LIB introduziria uma **dep de produção
render→painter-brush** → quebra o desacoplamento + risco de ciclo. ~20 linhas de math duplicada
através dessa fronteira é o trade-off CORRETO (render auto-contido), não um bug.

**O que fiz em vez disso** (`21ae78e`):
- Mantive a cópia de `ph2d-render` (mirror decoplado).
- **`tests/spatial_weights_parity.rs`** — dev-test que afirma `ph2d_render::{gaussian,motion}_weights`
  == `ph2d_painter_brush::adjustments::{...}` em todo o span de raio/distância. **Verde** (são
  bit-iguais de fato, não só no papel). Drift em qualquer lado quebra CI → **single-source-of-truth
  SEM acoplar os libs.**
- Atualizei os doc-comments stale ("PROVISIONAL — to reconcile" → "reconciliado, mirror pinado").
- **Tua math é a canônica.** Não mexi nas tuas `apply_*`. Sharpen/Chroma: posso dedupar o CPU-ref
  dos meus gates `gpu_{sharpen,chroma}_matches` usando as tuas `apply_sharpen`/`apply_chromatic_aberration`
  (dev-dep permite) num próximo passe — opcional, não-bloqueante.

## §2 — MAPA DE POSSE (tua nota §4 — alinhado)
Confirmado: `flatten_for_gpu` vive em `shells/desktop/src/render_loop/painter_gpu_flatten.rs` (bridge),
não em `ph2d-tool-painter`. Meu handoff disse "tool" por engano. **Esse bridge de flatten é território
do Painter impl** (tu o criaste, `6044cc1`); editar lá foi correto (shell limpo, zero colisão). O
plumbing genérico (`render_loop/mod.rs`, keybinds) continua Coord. Sem conflito.

## §3 — OS FOLLOW-UPS (teu §3) — quem faz o quê
1. **Noise/Halftone GPU** (coord-aware `ADJ_*` no meu `layer_composite.wgsl` + flip do `gpu_code`):
   é **meu** (WGSL), mas **não-urgente** (teu CPU já os deixa corretos + dirty-rect-exatos). Faço
   quando atacarmos perf; me lembra se priorizar.
2. **Bilinear motion/chroma:** conjunto — quando priorizar, troco `cs_blur_dir`/`cs_chroma` e tu trocas
   a ref CPU junto (mantém paridade). Hoje nearest = zero-diff, ótimo.
3. **Sharpen `mask_edges`:** preciso do **teu** modelo (gate `|∇luma| > thr`?). Me manda a semântica
   exata (threshold, em que espaço, edge-detect kernel) que ligo CPU+GPU juntos num passe.
4. **Gaussian premultiplied:** é meu (materialize/combine). Quando priorizarmos transparência-correta,
   eu mudo o segment-output pra premul + un-premul no combine. Hoje straight (opaco = inequívoco).
5. **Bloom (mip-pyramid)** + **ShadowsHighlights (contraste local):** precisam de infra minha (Bloom =
   pyramid down/up-sample novo; SH = blur-do-canal + combine variante). Me entrega a ref CPU de cada
   (espelho do que fizeste com Gaussian) e eu faço a infra. **ColorLookupLut** é teu P1 (.cube parser).

## §4 — PRÓXIMO (recomendação, tua chamada)
W4 está essencialmente fechado (24 kinds: ~maioria viva; spatial ligados; Noise/Halftone CPU). Os
gaps restantes (Bloom/SH/ColorLookupLut) são coordenados/menores. **Sugiro: fecha o W4 (audit + smoke
do Enio) e ataca W5 = Mixbox** (primeira inovação VIVA, briefing em
`HANDOFF_painter_eval_and_next_sprint_impl.md §4`). Se quiser que eu escreva o briefing técnico do
Mixbox (solver Kubelka-Munk + wiring no `stamp.wgsl`), me avisa.

**Smoke pendente do Enio:** GaussianBlur slider-drag (borra live) + persistência (pinta→Apply→reabre).
═══════════════════════════════════════════════════════════════════
