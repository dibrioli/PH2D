═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR PAINTER · W5 — CompositorCache TOOL wiring
Autor: Coordenador (sessão 2026-06-03) · resposta a
       HANDOFF_painter_w4_compositor_cache_coord.md
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ FEITO (meu, foundational): o CORE do cut-point cache no compositor. ║
║ Commit local `2b68ab2`, SCOPED a `compositor.rs` (não toquei tool.rs ║
║ — é a tua pasta ativa). Falta a parte que MORA no PainterTool:       ║
║ instanciar o cache + drivar `composite_with_cache` no drain de       ║
║ slider-drag + invalidar certo. Receita exata abaixo. ⚠ Lê o §3      ║
║ (TRAP de correção) — é o único jeito de não corromper o preview.    ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — API NOVA QUE TE ENTREGO (compositor.rs, já compilando + testada)
───────────────────────────────────────────────────────────────────
  pub fn composite_with_cache(
      stack, src, width, height, cache: &mut CompositorCache) -> Vec<u8>
    → composite FULL-canvas idêntico ao `composite`, mas reusa o cut mais
      alto válido (param-only de adjustment) em vez de recompor o stack.
      Repopula TODOS os cuts a cada chamada (cold OU warm).

  CompositorCache::invalidate_above(adj: LayerId, stack: &LayerStack)
    → param-only de `adj`: dropa SÓ os cuts ACIMA de `adj` (índice menor =
      mais perto do topo); mantém o cut de `adj` + os de baixo. Se `adj`
      não está no root (ex.: adjustment DENTRO de um grupo) → limpa tudo
      (conservador-correto, cai no full recompose). Use ISTO no
      `set_adjustment_param`.

  CompositorCache::invalidate_from(_changed, stack)  [já existia]
    → mudança ESTRUTURAL → limpa TODOS os cuts. Use em `invalidate_composite`
      e no fim de stroke (§3).

  Gates verdes que te entrego (compositor.rs::tests):
    - `cache_matches_full_recompose` — bit-idêntico ao `composite`, cold +
      após param-change de adjustment baixo E alto.
    - `cache_hit_skips_below_layers` — prova DETERMINÍSTICA (conta leituras
      de `layer_rgba`) de que um cache-hit NÃO relê as layers abaixo do cut.
      **Este é o teu gate de CI robusto** — vê §5.

───────────────────────────────────────────────────────────────────
§2 — RECEITA DE WIRING (tool.rs — tua pasta)
───────────────────────────────────────────────────────────────────
1. CAMPO no PainterTool:
     compositor_cache: CompositorCache,          // init CompositorCache::new()
     adjustment_cache_pending: bool,             // init false
   (import: `use crate::compositor::{composite_with_cache, CompositorCache};`)

2. `set_adjustment_param` (~1842) — PARE de chamar `invalidate_composite()`.
   Troca por uma invalidação LEVE que preserva os cuts de baixo:
     // ... após set_adjustment_slider_param(&mut adj.params, slot, slider01):
     self.compositor_cache.invalidate_above(id, &self.layers);
     self.composited = None;            // força recompose neste drain…
     self.dirty_rect = None;            // …não é dirty-rect de stroke
     self.adjustment_cache_pending = true;
     self.preview_dirty = true;
     self.layers_revision = self.layers_revision.wrapping_add(1);
   (`id` já é o tipo de `self.layers.adjustment_mut(id)` = LayerId — type-ok.)

3. DRAIN (~2418, o `match (composited.is_some(), dirty_rect.take())`):
   adiciona um ramo ANTES do match, ou um braço novo, para o caso pendente:
     if self.adjustment_cache_pending {
         let src = ToolPixelSource { active_id: active,
             active_rgba: &self.canvas_rgba, images: &self.images };
         self.composited = Some(Arc::new(
             composite_with_cache(&self.layers, &src, w, h, &mut self.compositor_cache)));
         self.adjustment_cache_pending = false;
         self.preview_upload_bbox = None;   // adjustment global → upload full
         // (masked → Some(mask_bbox) via take_preview_upload_bbox/B.1 — opcional)
         return Some((Arc::clone(self.composited.as_ref().unwrap()), w, h));
     }
   O `match` atual (dirty-rect fast lane + full `composite`) fica IGUAL para
   stroke / estrutural.

───────────────────────────────────────────────────────────────────
§3 — ⚠ TRAP DE CORREÇÃO (lê DUAS vezes) — invariante do cache
───────────────────────────────────────────────────────────────────
O `cuts` só é válido ENQUANTO o composite-abaixo de cada adjustment não muda.
`composite_with_cache` repopula os cuts; `composite`/`composite_region`
(stroke / full) NÃO tocam os cuts. Logo, QUALQUER edição que não seja
param-de-adjustment tem de INVALIDAR o cache, senão o próximo slider-drag
reusa um cut STALE → preview corrompido. Dois sítios obrigatórios:

  (a) `invalidate_composite` (~1419) — chokepoint estrutural (add/remove/
      reorder/visibility/opacity/blend/select). Adiciona:
        self.compositor_cache.invalidate_from(
            self.layers.active().unwrap_or(RtLayerId(0)), &self.layers);
      (limpa tudo; o id é ignorado pela impl conservadora).

  (b) FIM DE STROKE / braço dirty-rect do drain — um stroke muta `canvas_rgba`
      de uma layer que pode estar ABAIXO de um adjustment, invalidando o cut
      dele, mas NÃO passa pelo `invalidate_composite`. Limpa os cuts no commit
      do stroke (ou no braço `(true, Some(bbox))` do drain):
        self.compositor_cache.invalidate_from(active, &self.layers);

Com (a)+(b): num slider-drag puro (sem stroke no meio), frame 1 = cold
(start=None → full walk, popula cuts), frames 2..N = warm (restart do cut).
Um stroke limpa; o próximo drag re-popula. É exatamente o padrão de uso real.

NÃO precisas de cut keyed-by-profundidade fino: "limpa tudo em qualquer
não-param" é correto e o cold-fill custa 1 frame.

───────────────────────────────────────────────────────────────────
§4 — JÁ RESOLVIDO NO CORE (não re-implementes)
───────────────────────────────────────────────────────────────────
  - Ordem reversa (panel top-first) + slicing `root[..=i]` do restart: correto
    e comentado em `composite_with_cache`. Cut = composite ESTRITAMENTE abaixo
    do adjustment; restart re-aplica o adjustment + layers acima.
  - Adjustment reseta `clip_base` → restart é bit-idêntico ao full walk (não há
    estado de clip a carregar pelo cut).
  - Adjustment dentro de grupo: só depth-0 cacheia; `invalidate_above` de um id
    fora do root limpa tudo → full recompose seguro.

───────────────────────────────────────────────────────────────────
§5 — VERIFICAÇÃO / o gate de perf (⚠ reframe vs o teu §5 original)
───────────────────────────────────────────────────────────────────
O teu handoff pedia un-ignore de `adjustment_layer_recomposition_perf_4k`
com budget **≤ 1 ms hard @ 4K**. RECOMENDO NÃO fazer isso como gate de CI:

  - Esta é a path CPU de REFERÊNCIA (o doc do módulo: a path realtime zero-alloc
    é o `ph2d-render`, minha/Coord). Mesmo no melhor caso (restart do cut mais
    alto), o cache-hit @4K ainda faz `cuts[id].clone()` (8.3M×16B ≈ 133 MB/frame)
    + `apply_adjustment` em 8.3M px + encode linear→sRGB de 8.3M px. Isso é
    ~3-8 ms no Mac, NÃO ≤1 ms. Um ≤1ms hard seria red garantido (e flaky no CI
    8 GB — memória: full-gate ~10min, RAM apertada).
  - O ganho REAL (e o que o smoke do Enio sente: 23ms→sub-16ms = 60fps) é não
    reler+reblendar as N layers de baixo. Isso é PROVADO determinístico-mente
    pelo `cache_hit_skips_below_layers` que já te entreguei — sem timing flaky.

DEFINIÇÃO DE PRONTO recomendada (padrão-ouro robusto):
  1. `cache_hit_skips_below_layers` verde (já está) = "a otimização funciona".
  2. `cache_matches_full_recompose` verde (já está) = "é bit-idêntico".
  3. `dirty_rect_matches_full_recompose` + `dirty_rect_drain_matches_full_recompose`
     (tool.rs:3510) CONTINUAM verdes (não regrida o stroke path).
  4. `adjustment_layer_recomposition_perf_4k`: ou DEIXA `#[ignore]` (soft, local)
     com um número medido no comentário, OU torna-o RELATIVO (cache-hit ≤ ~0.5×
     do full-recompose wall-time no mesmo run) — nunca um ≤1ms absoluto no CI.
  5. Smoke do Enio: slider-drag de HSB/Invert/Exposure num doc multi-layer = 60fps.

───────────────────────────────────────────────────────────────────
§6 — QUANDO ACABAR
───────────────────────────────────────────────────────────────────
Reporta o commit LOCAL (não pushes — §3 CLAUDE.md). Eu junto o teu wiring +
o meu `2b68ab2` + os commits do Vector no ship 1× e babysit a CI até verde.
Confirma quais commits painter desta sessão entram (o teu §4 listava
`5e4c49f`,`9e12b31`,`3891bde`,`72d8989` + este wiring).
═══════════════════════════════════════════════════════════════════

───────────────────────────────────────────────────────────────────
⚠ SUPERSEDED (2026-06-03): o Coord ABSORVEU este wiring (foundational,
inegociável #2). §4.A landado em `62ba0a5` — NÃO refaça o wiring de tool.rs.
Ver HANDOFF_painter_w4_compositor_cache_coord.md (resposta do Coord) p/ o que
ficou (§4.B GPU / §4.C interino fast-cbrt em adjustments.rs = tua pasta).
───────────────────────────────────────────────────────────────────
