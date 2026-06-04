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

───────────────────────────────────────────────────────────────────
§7 — ►► MEDIÇÃO REAL (smoke Enio 2026-06-03) + DESCOBERTA CRÍTICA ◄◄
───────────────────────────────────────────────────────────────────
Instrumentei o `painter_bridge` (probe temporário, uncommitted — repro no fim) e
o Enio mediu o drag de slider. Canvas **1024×1024**, por frame:

    [painter-perf] 1024x1024 drain(composite+encode)=~80ms upload(clone+premul+gpu)=~1.1ms

**=> O custo é 100% CPU drain (composite+encode). Upload é ~1ms (irrelevante —
Apple Silicon memória unificada).** 80ms / 1M px = ~80 ns/px = a assinatura de
`powf`: o compositor faz **decode sRGB→linear da base (3 powf/px)** + **encode
linear→sRGB da saída (3 powf/px)** = **6 powf/px sobre o canvas inteiro, todo
frame**. (Por isso até Brightness/Contrast — zero transcendental no compute do
kind — cai: o custo é o decode/encode do compositor, não o kind.)

**DESCOBERTA CRÍTICA — o CompositorCache sozinho NÃO resolve:**
  1. **O cache core (`2b68ab2`) NÃO está wirado no hot-path do tool.** O
     `PainterTool::take_preview_arc` (tool.rs ~2443) ainda chama `composite()`
     puro, não `composite_with_cache`. **Wire:** o `PainterTool` precisa POSSUIR um
     `CompositorCache`, chamar `composite_with_cache` no drain, e
     `invalidate_from`/`invalidate_above` no edit (param de adjustment = só o cut
     dele p/ cima; structural = clear). Hoje `set_adjustment_param`→
     `invalidate_composite` força full.
  2. **MESMO wirado, o cache só remove o DECODE das layers abaixo (~40ms num doc
     base+1adj). O ENCODE (~40ms) PERMANECE** — `composite_with_cache` re-`encode`
     o canvas inteiro todo frame (a saída do adjustment muda). => cache wirado ≈
     **40ms ≈ 25fps, ainda não suave.**

**=> FIX COMPLETO = cache (wirado) + LUT do decode E encode.** As duas peças são
ortogonais e AMBAS necessárias. O LUT é o que tira os 6 powf/px:
  - **decode** (`compositor.rs::decode`, ~77): `srgb_to_linear_byte` recebe `u8` →
    **LUT de 256 entradas é EXATA** (zero erro). ATENÇÃO: mantenha bit-idêntico —
    o gate `layer_compositor.rs:~1026` faz `assert .to_bits() == srgb_to_linear_byte(b)`.
    Uma LUT com os valores exatos do powf é bit-idêntica. (Com o cache warm, o
    decode-LUT só importa no cold/first-frame; mas é trivial e cobre isso.)
  - **encode** (`compositor.rs::encode`, ~198): `linear_to_srgb_byte` recebe `f32`.
    Use tabela de 255 thresholds `t[b]=srgb_to_linear((b+0.5)/255)` +
    `partition_point` (ou LUT 4096 + ajuste ±1) → byte-EXATO, zero powf/px. **Adicione
    um teste de sweep denso: encode_lut(v) == linear_to_srgb_byte(v) p/ ~1e5 v** —
    o encode alimenta o bake do Apply (não achei cook-hash, mas seja exato).
  Esperado pós-fix: 80ms → ~5-10ms (drag suave a 60fps), e o cache derruba o
  decode das below-layers em docs com muitas camadas.

**Probe pra re-medir** (cole no `shells/desktop/src/render_loop/painter_bridge.rs`,
no `dispatch`, em volta de `take_preview_arc` e do bloco de upload):
    let _t0 = std::time::Instant::now();
    let drained = painter.take_preview_arc();
    let drain_ms = _t0.elapsed().as_secs_f32()*1000.0;
    // … gate em painter_dirty_bbox.is_none() (full recompose) …
    eprintln!("[painter-perf] {w}x{h} drain={drain_ms:.2}ms upload={up_ms:.2}ms");

**ESCOPO/COLISÃO:** decode/encode/wiring são tudo `compositor.rs` + `tool.rs`
(foundational, e o `2b68ab2` está VIVO nesse arquivo) → **Coord-only**. Eu (impl)
fiz só o LUT de decode/encode (commit `902a6cb`, funções `decode`/`encode` — NÃO
o arm Adjustment nem o cache). O probe foi revertido (working tree limpo).

───────────────────────────────────────────────────────────────────
§8 — ►► BREAKDOWN EM RELEASE (pós-LUT `902a6cb`) — o powf NÃO era o grande ◄◄
───────────────────────────────────────────────────────────────────
O LUT só deu ~30% (80→56ms): o `powf` era ~24ms, não 78ms (powf ARM ~4ns).
**O Enio está rodando RELEASE** (o número dele bate com meu bench release; debug
seria ~370ms). Decompus o `composite()` 1024² em RELEASE (`opt-level=3`+thin-LTO):

    base only (decode+blend+encode) ............ 14.8 ms
    + adjustment arm (acc.to_vec + blend-back) .. +8.9 ms  (≈ 24 ms p/ kinds baratos)
    + Brightness/Contrast (math barato) ........ +1.0 ms  → B/C total ~25 ms (40 fps)
    + HSB OKLab cbrt round-trip ................ +30  ms  → HSB total ~55 ms (18 fps)

**Achados:**
  1. **O `composite()` da CPU é o caminho REFERÊNCIA** (o doc do módulo diz: "the
     real-time zero-alloc GPU compositor ... is the Coordinator's `ph2d-render`
     sibling"). **Mas o painter live-preview chama o `composite()` da CPU todo
     frame** (`take_preview_arc`→`composite`). Real-time deveria ir pelo
     **GPU LayerCompositor (`ph2d-render`)**. ← raiz arquitetural.
  2. **O cache (`2b68ab2`) ainda não está wirado** → a base (~15ms) recompõe todo
     frame mesmo só mexendo no param. Wirar derruba os ~15ms da base + os ~9ms do
     arm (na verdade o arm roda sempre; o cache tira o decode/blend da base) →
     **kinds baratos ~60fps**; HSB cai p/ ~45ms (cbrt sobra).
  3. **HSB/Vibrance: o OKLab `cbrt` (~30ms@1024²) domina** e é per-pixel todo
     frame — NEM o cache tira isso (é o apply do próprio adjustment). @4K seria
     ~480ms → o gate `≤1ms@4K` é **impossível na CPU**; só fecha no **GPU**.

**PRIORIDADE (Coord):**
  A. **Wire o cache no `take_preview_arc`** (maior alavanca p/ a maioria dos kinds;
     já 80% pronto em `2b68ab2`). → kinds baratos a 60fps.
  B. **Caminho real-time = GPU**: rotear o preview do painter pelo
     `ph2d-render` LayerCompositor (precisa suporte a adjustment no GPU — peça
     grande, mas é o único jeito de HSB@4K caber em 1ms). É a resolução durável.
  C. (CPU interino, secundário) `apply_blend` fast-path Normal/opaco (corta div/px
     do base-blend + blend-back); fast-`cbrt` no OKLab p/ HSB/Vibrance (−~20ms).
  Eu (impl) posso fazer o fast-`cbrt` em `adjustments.rs` (minha pasta) se você
  pedir — mas o ganho real (GPU) e o wiring do cache são teus.
═══════════════════════════════════════════════════════════════════
