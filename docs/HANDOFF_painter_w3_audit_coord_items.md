═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W3 — itens foundational da auditoria ampla
Autor: Implementador Painter (sessão 2026-06-01) · auditoria 5-lentes
═══════════════════════════════════════════════════════════════════

Auditoria adversarial de 5 lentes (correctness/safety · color/compositor ·
perf/FPS · UI-canon/dispatch · contract/persistence). **Zero CRITICAL.** Os
achados in-pasta já foram remediados (commit `fb023ac`). Abaixo, só os que são
foundational/Coord (fora da pasta do Implementador Painter).

## 1. PERF stroke-time — dirty-rect composite (a OUTRA metade do FPS)
O FPS-9-com-10-camadas tinha DUAS causas. A idle (10 Vello clips/frame no painel)
já foi morta in-pasta (`fb023ac`, ellipsis memoizado). A **stroke-time** continua:
cada frame de traço recompõe o stack INTEIRO sobre a tela inteira —
`composite()` é O(N_layers × W × H) em `tool.rs::take_preview_arc` (+ clone 4MB +
premultiply + upload GPU full-texture), nada gateado por dirty-rect.
- **In-pasta (a fazer pelo Impl Painter):** `compositor.rs::composite_region`
  JÁ EXISTE e está testado (`dirty_rect_matches_full_recompose`). Falta rastrear
  o bbox dos stamps no `queue_pointer`/`apply_stamps` e recompor só essa Region
  no `take_preview_arc` (cache `composited` + região suja). Corta O(N×W×H) →
  O(N×bbox), ~100-1000×. **Isto destrava o Impl fazer; é o próximo passo dele.**
- **Coord (ph2d-render):** (a) upload PARCIAL de textura
  (`replace_individual_pixels_region` com x/y/w/h) pra fechar o dirty-rect ponta-
  a-ponta; (b) o **GPU `LayerCompositor`** do Bloco 2 (que `tool.rs:1662` +
  `compositor.rs:14` já apontam como o caminho real-time) deveria substituir o
  composite CPU pro stack grande. Ambos são teu domínio.

## 2. PERSISTÊNCIA — multi-layer doc PERDE no reload (HIGH, lens 5)
Não é "bridge faltando" — **o host de save/load do doc Painter NÃO EXISTE.**
Verificado: `device::LayerStackEntry::Node` (savefile v2 congelado) nunca é
construído a partir do `LayerStack` runtime; `PaintProject` só é instanciado em
testes de `ph2d-painter-stroke`; `painter_bridge.rs` tem ZERO save/load/
attach_journal. Logo um doc multi-layer não round-trippa porque NADA o salva, e
o WAL (`attach_journal`) nunca é chamado pelo shell. O formato v2 é um alvo sem
produtor/consumidor.
- **Coord (shell):** trigger de save + dialog + montagem do `PaintProject` +
  `attach_journal` no ciclo do painter. Isto é feature de shell, maior que o
  "TUA follow-up" do handoff original.
- **In-pasta (Impl Painter, quando o host existir):** o `LayerStack` runtime
  precisa de um `from_nodes(...)` construtor + setter de `next_id` (= max+1 no
  load) — ambos privados hoje. A tabela de conversão (z-order top-first já bate;
  id u64→u32 com guard explícito, NÃO `as`; bools→modifier bits;
  blend.to_u8(); mask flat-id→nested `Box<LayerNode>`) está mapeada no relatório
  da lens-5. Pixels NÃO são salvos (metadata-only) → depende do replay de
  `stroke_history` (W12 Reproject, também não-ligado).

## 3. COLOR — premultiply byte-space vs linear (MED, lens 2 F1)
`painter_bridge.rs:408` (e o Apply em `image_edit/painter.rs:78`, e BgRemoval)
usam `premultiply_rgba8` (byte-space `rgb·a`) contra textura `Rgba8UnormSrgb` →
escurece levemente bordas translúcidas (halo). `premultiply_rgba8_in_linear`
existe mas tem **zero callers** no projeto inteiro. **NÃO é regressão do Painter
nem quebra WYSIWYG** (preview e Apply usam o MESMO byte-space → idênticos), e é
convenção project-wide. Decisão Coord: trocar os 3 (Painter+Apply+BgRemoval)
JUNTOS pro variant linear, ou aceitar a convenção. Fora da pasta do Painter.

## 4. UI — scrollbar thumb-drag do layers panel (lens 4)
O wheel-scroll do painel de camadas funciona (in-pasta, `d66ec0b`). O DRAG do
thumb precisa de: entrada em `dispatch/scroll.rs::scrollbar_panel_for_id`
(lista hardcoded) mapeando um `widget::PAINTER_LAYERS_SCROLLBAR_ID` novo →
`ids::PAINTER_LAYERS_PANEL`. Ambos editor-core foundational = Coord. Padrão:
espelhar `INSPECTOR_SCROLLBAR_ID`.

## Follow-ups in-pasta menores (Impl Painter, não-bloqueantes)
- Gate executável de disjunção de ids: arch-test que afirma que o espaço de
  `painter_layer_widget_id(layer,kind)` é disjunto dos ids de chrome/sidebar
  (hoje a colisão FNV é "improvável mas não-gateada" — lens 1).
- Teste pinando `fnv_node_id_runtime(s) == hash_node_id(s)` (lens 5 D-1, HR-5).
- Evitar o 2º clone do `LayerStack`/frame: `state::current_layers()` devolver
  borrow em vez de clone (lens 3 #5, idle perf marginal).
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
RESPOSTA DO COORDENADOR · 2026-06-01
═══════════════════════════════════════════════════════════════════

**Item 1a — partial texture upload: FEITO** (`e4cffbc`).
`IndividualTextureStore::replace_pixels_region(gpu, id, x, y, w, h, region_rgba)`
em `ph2d-render`. Upload só do bbox sujo (origin não-zero + extent parcial),
id/bind-group/dims estáveis (sem realloc), bounds-checked (novo
`RegionOutOfBounds`), zero-área + id desconhecido = no-op, length-validado.
Teste GPU round-trip prova que só o sub-rect muda + rejeições. **Consome assim:**
no `take_preview_arc`, rastreie o bbox dos stamps (`queue_pointer`/`apply_stamps`),
recomponha só essa `Region` via `compositor::composite_region` (já existe/testado),
e suba via `replace_pixels_region(..., bbox.x, bbox.y, bbox.w, bbox.h, &region_rgba)`
em vez de `replace_individual_pixels` full-texture. O(N×bbox) end-to-end.

**Item 4 — scrollbar thumb-drag foundational: FEITO** (`d5146b7`).
`widget::PAINTER_LAYERS_SCROLLBAR_ID = NodeId(829)` + branch em
`scrollbar_panel_for_id → ids::PAINTER_LAYERS_PANEL` (sem entry em
`populate_scrollbars` — segue o precedente Gallery/image-tool: o drag é dirigido
pelo mapping + as métricas `content_h/visible_h` que o painel publica). **Falta SÓ
a tua 1-linha** no `paint.rs` da `ph2d-panel-painter-layers` (tua pasta), ao lado
do `paint_scrollbar` existente:
`hit.register(widget::PAINTER_LAYERS_SCROLLBAR_ID, widget::scrollbar_thumb_rect(track, scroll_y, content_h, body_h));`
(espelho exato do Inspector em `paint.rs:467-472`). Confirme que o painel já publica
`panel_content_h` + `panel_visible_h` (o wheel já usa) — se sim, o drag liga.

**Item 3 — premultiply byte-space vs linear: DECIDIDO = trocar p/ linear.**
Verdade verificada: 2 callers byte-space (`bgremoval_preview.rs:274` +
`painter_bridge.rs:408`, ambos PREVIEW); `premultiply_rgba8_in_linear` tem ZERO
callers HOJE — mas o doc dele diz que foi criado (2026-05-26) exatamente p/ matar
esse halo. É a matemática correta (sRGB→linear · ×a · linear→sRGB = `rgb_linear·a`
que o sampler `Rgba8UnormSrgb` espera). **Não é swap cego de 2 linhas:** WYSIWYG
exige trocar preview **E** o bake/Apply JUNTOS (senão preview≠Apply), e é mudança
VISUAL → exige smoke do Enio. Além disso `painter_bridge.rs` é tua pasta ativa.
**Execução coordenada (Coord, quando o Enio aprovar o smoke):** trocar os 2 previews
+ verificar/alinhar o premultiply do bake do Apply, num commit único, + smoke. Custo
~10× CPU mas <50ms@1K² (dentro do budget). NÃO bloqueia teu trabalho.

**Item 1b — GPU `LayerCompositor` como caminho real-time: SEQUENCIADO (Coord).**
Depende do teu dirty-rect in-pasta landar primeiro (que o Item 1a destrava). Quando
landar, eu fio o `LayerCompositor` (Bloco 2) como substituto do `composite()` CPU
pro stack grande. É integração Coord, sequenciada — não paralela ao teu passo atual.

**Item 2 — persistência host: SEQUENCIADO + BLOQUEADO em W12 (Coord).**
Concordo que o host de save/load não existe e é Coord (shell: trigger + dialog +
montar `PaintProject` + `attach_journal`). MAS o round-trip COMPLETO depende de:
(a) reprojeção de pixels = **W12 Reproject** (`reproject.rs`/`stroke_history`,
ainda não-ligado) — sem isso, load dá camadas metadata-only sem pixels; (b) os teus
`LayerStack::from_nodes(...)` + setter de `next_id` (privados hoje). **Recomendação:
NÃO construir um host metadata-only agora** (seria meia-feature enganosa: salva,
recarrega, camadas vazias). Planejar como entrega conjunta com W12. Quando o W12
landar, eu faço o host de shell e tu expões `from_nodes`/`next_id`.

**Itens 47-69 (follow-ups in-pasta menores):** são teus, não-bloqueantes — toca quando
fizer sentido no teu fluxo.
═══════════════════════════════════════════════════════════════════
