═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Painter W4/W5 — CompositorCache (slider-drag FPS)
Autor: Implementador Painter (sessão 2026-06-03) · foundational = teu (Coord-only)
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ PEDIDO: wire o `CompositorCache` (ADR-0045 §2.7) no hot-path do      ║
║ compositor p/ matar a queda de FPS no drag de slider de adjustment.  ║
║ É foundational (compositor.rs arm Adjustment + cache) — fora da pasta║
║ do impl (inegociável #2 + handoff §3/§5). Diagnóstico + cut-point    ║
║ exato + gate abaixo. O lado compute JÁ está pronto e barato.        ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — SINTOMA + DIAGNÓSTICO (confirmado por 2 smokes do Enio)
───────────────────────────────────────────────────────────────────
Arrastar QUALQUER slider de um adjustment layer derruba o FPS. Smoke 1: só os
display-space (Invert/Posterize/Threshold/Exposure) — eu LUT-otimizei o compute
(commit `9e12b31`: 0 transcendentais/pixel). Smoke 2 (pós-LUT): **cai em TODOS,
incl. HSB/Brightness-Contrast** (que sempre foram OKLab/aritmética barata).

**=> O compute NUNCA foi o gargalo dominante.** O custo é estrutural: cada frame
de drag faz **recompose de canvas INTEIRO** (lê todas as layers + reblenda +
re-encode + reupload). Bandwidth-bound (memória do projeto: 50×4K ≈ 1.66 GB lidos
→ ~70 GB/s Mac = ~23 ms/frame → bem acima do budget de 16 ms).

Isto é EXATAMENTE o que o `CompositorCache` resolve, e já está marcado como teu:
  - `compositor.rs:813` — gate `adjustment_layer_recomposition_perf_4k` está
    `#[ignore]` com nota: *"W4 soft perf gate (ADR-0045 §2.11) … W5 wires
    CompositorCache cut-points into composite + un-ignores (hard ≤1ms @4K)"*.
  - O `CompositorCache` (`compositor.rs:354`) é **skeleton**: `invalidate_from`
    = "skeleton: clears"; `cuts` BTreeMap nunca é populado/consultado no hot-path.

───────────────────────────────────────────────────────────────────
§2 — O CUT-POINT EXATO (onde wirar)
───────────────────────────────────────────────────────────────────
Hot-path: `compositor.rs::composite_into` (linha ~177) — UM walk bottom-up que
blenda as layers em `acc`. O arm **`LayerKind::Adjustment`** (linha ~288):

    let mut adjusted = acc.to_vec();          // cópia do composite ABAIXO
    apply_adjustment(&adj.kind, &adj.params, &mut adjusted);
    // … blenda `adjusted` de volta sobre `acc` por opacity×mask no blend mode

`acc` (= composite de tudo abaixo do adjustment) é recomputado do zero todo frame,
mesmo quando só um PARÂM do adjustment mudou (o stack abaixo é idêntico).

**Design do cache (ADR-0045 §2.7, já esboçado no skeleton):** cada adjustment é
um "cut point" — cacheie o `acc` logo ABAIXO dele em `CompositorCache::cuts[adj_id]`.
Numa mudança de parâmetro do adjustment N:
  - tudo ABAIXO de N não mudou → reusa `cuts[N]` (sem recompor as layers de baixo);
  - re-roda só `apply_adjustment` de N + o blend-back + as layers ACIMA de N.
`invalidate_from(layer, stack)` deve dropar os cuts dos adjustments >= `layer` na
ordem de composição (os de baixo seguem válidos). Mudança ESTRUTURAL (add/remove/
reorder/visibility/opacity de layer abaixo) → invalida o cut afetado p/ baixo.

Subtlety de correção: o cut tem que ser keyed de forma que reordenar/editar uma
layer abaixo invalide. O `cuts` é `BTreeMap<LayerId,…>` (HR-5, determinístico) — a
invalidação por "posição na composição" precisa mapear LayerId→profundidade no
walk atual (ou invalidar todos os cuts num structural edit, conservador-correto).

───────────────────────────────────────────────────────────────────
§3 — LADO DO TOOL (drive da recomposição) — contexto, NÃO precisa mudar muito
───────────────────────────────────────────────────────────────────
  - `tool.rs::set_adjustment_param` (~1843) → `invalidate_composite` (~1419) que
    força FULL recompose (`pending_full`/limpa dirty-rect). Drain → `run_full`
    (~2987) → `compositor::composite`.
  - Pro cache valer no drag, o invalidate de um PARÂM de adjustment deveria sinalizar
    "só o cut deste adjustment p/ cima", não full. Isso pode exigir um caminho novo
    no tool (ex.: `invalidate_adjustment(layer)` que NÃO limpa os cuts abaixo) — é
    fronteira tool↔compositor; decide se mora no `CompositorCache` (held onde?) ou
    num campo do tool. Hoje o `CompositorCache` não é instanciado no PainterTool
    (só existe o tipo) — wirar o ownership é parte do trabalho.
  - Upload: pra um adjustment global (sem mask) o bbox sujo é o canvas inteiro →
    reupload de canvas inteiro por frame. Em Apple Silicon (memória unificada) é
    barato vs o composite CPU, mas MEÇA os dois. `take_preview_upload_bbox`
    (W3 B.1) já existe p/ upload parcial — pra adjustment mascarado, o bbox da mask
    corta o upload.

───────────────────────────────────────────────────────────────────
§4 — JÁ FEITO (não refaça)
───────────────────────────────────────────────────────────────────
  - **Compute de todos os kinds implementados está no orçamento.** Display-space
    (Invert/Posterize/Threshold/Exposure) usam LUT 1-D per-call (`build_lut`/
    `sample_lut`, N=1024) → 0 transcendentais/pixel (commit `9e12b31`). OKLab
    (HSB/Vibrance) = 1 cbrt round-trip. Neutral early-return onde aplica.
  - T4.15 (menu "+ Adjustment") + 5 kinds per-pixel landados (commits `5e4c49f`,
    `9e12b31`). Detalhe: `HANDOFF_painter_w4_fanout_impl.md` §SESSION UPDATE.
  - **5 commits painter locais desta sessão entram no teu ship:** `5e4c49f`,
    `9e12b31`, `3891bde`, `72d8989` (+ o que esta doc gerar). Não pushei.

───────────────────────────────────────────────────────────────────
§5 — VERIFICAÇÃO (definição de pronto)
───────────────────────────────────────────────────────────────────
  - Un-ignore + flesh `adjustment_layer_recomposition_perf_4k` (`compositor.rs:813`):
    budget = slider-drag recompose @ 4K, 10 adjustment layers ≤ 1 ms (hard).
  - O gate de correção `dirty_rect_matches_full_recompose` (`compositor.rs:669`) +
    `dirty_rect_drain_matches_full_recompose` (`tool.rs:3510`) DEVEM continuar
    verdes — o caminho cacheado tem que ser bit-idêntico ao full-recompose.
  - Novo gate sugerido: "cache hit não recompõe as layers abaixo do cut" (ex.:
    provider que conta leituras de `layer_rgba`, assert que mudar 1 parâm de
    adjustment não relê as layers abaixo).
  - Smoke do Enio: drag de slider de HSB/Invert/Exposure num doc com várias layers
    deve ficar a 60 fps.

───────────────────────────────────────────────────────────────────
§6 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
  - ADR-0045 §2.7 (cache cut-point) + §2.11 (perf gate soft no W4).
  - `compositor.rs`: `composite_into` (~177, arm Adjustment ~288), `CompositorCache`
    (~354, `cuts`/`invalidate_from`), gate (~813).
  - `tool.rs`: `invalidate_composite` (~1419), `set_adjustment_param` (~1843),
    `run_full` (~2987), drain (~2416-2444).
  - Contrato congelado: `AdjustmentKind≤32` etc. (CLAUDE.md §6) — o cache não toca
    o contrato (é interno do compositor).
═══════════════════════════════════════════════════════════════════
