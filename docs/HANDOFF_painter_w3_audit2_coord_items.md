═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W3 — 2ª auditoria (novo Implementador)
Autor: Implementador Painter (sessão 2026-06-01, mandato Enio = auditoria
completa antes de features) · auditoria adversarial 6-lentes, read-only
═══════════════════════════════════════════════════════════════════

VEREDITO: **ZERO CRITICAL** (confirma a baseline da 1ª auditoria 5-lentes).
Color pipeline verificado correto (22 blend modes batem com W3C/Photoshop;
preview≡Apply byte-idêntico; encode/decode simétrico; alpha = cobertura
linear). Data model bem-guardado (depth caps, dangling-mask scrub, bounds).
Os dois fixes de FPS anteriores (idle clips, dirty-rect stroke-time) seguram.

Os achados IN-PASTA já foram remediados local (commit `f63cf06`): scrollbar
thumb-drag (tua 1-linha do item 4 da 1ª auditoria — FEITA), Arc no cache de
composite (corta 1 clone full-canvas/frame), goldens de 9 blend modes +
Hue/Saturation, group-variant do dirty-rect drain test, depth-guard no
collect_subtree, doc dos metadata setters mid-stroke. Abaixo só o que é
FOUNDATIONAL/Coord (fora da minha pasta).

───────────────────────────────────────────────────────────────────
§A — JÁ COBERTO pela 1ª auditoria (`HANDOFF_painter_w3_audit_coord_items.md`)
───────────────────────────────────────────────────────────────────
Re-confirmados, sem novidade. Não re-explico:
  - GPU upload PARCIAL consume (item 1a) — `replace_pixels_region` existe
    (`e4cffbc`); falta o wrapper `SpriteRenderer::replace_individual_pixels_region`
    + o bridge consumir o bbox. AINDA é o #1 lever de perf restante (vide §B.1).
  - Persistência host (item 2) — sequenciado/bloqueado em W12 Reproject. OK.
  - Premul byte-space vs linear (item 3) — veredito Coord = não trocar. OK.
  - GPU `LayerCompositor` real-time (item 1b) — sequenciado. OK.

───────────────────────────────────────────────────────────────────
§B — NOVOS achados foundational desta auditoria
───────────────────────────────────────────────────────────────────

B.1 [HIGH · perf] GPU upload full-texture por frame de traço — bbox é
    COMPUTADO E DESCARTADO. (lens 3)
    `take_preview_arc` (`tool.rs:1701`) faz `dirty_rect.take()` e joga fora o
    bbox; o composite já é O(bbox), mas o bridge (`painter_bridge.rs:~407`)
    ainda clona o canvas full (16MB@4K) + `premultiply_rgba8_in_linear` sobre
    o canvas INTEIRO (~10× premul plain) + `replace_individual_pixels` full-
    texture, TODO frame de traço. Metade do ganho dirty-rect está na mesa.
    - **In-pasta (pronto pra eu fazer quando quiseres):** `take_preview_arc`
      retornar o bbox consumido (4º campo da tupla) em vez de descartá-lo.
    - **Coord:** wrapper `SpriteRenderer::replace_individual_pixels_region`
      (espelho de `replace_individual_pixels`, sobre o `replace_pixels_region`
      que já fizeste) + o bridge premultiplicar/subir só o sub-rect. Fallback
      full quando bbox == canvas inteiro (edit estrutural).
    - NOTA conflito: `painter_bridge.rs` estava sendo editado por ti nesta
      jornada; coordenar antes de eu mexer na tupla de retorno.

B.2 [HIGH · determinismo, HR-5] `fnv_node_id_runtime(s) == hash_node_id(s)`
    NÃO TEM GATE. (lens 5+6)
    `ids.rs:381` é um gêmeo hand-copied de `ph2d_tool_registry::hash_node_id`
    (mesmo basis/prime/zero-bump) usado pra derivar os widget-ids por-row do
    painel. Zero testes referenciam `fnv_node_id_runtime` no workspace. Editar
    um dígito de qualquer das duas constantes diverge silenciosamente. A fn é
    privada e `ids.rs` não tem `mod tests` → **Coord** (editor-core
    foundational): adicionar teste pinando `fnv_node_id_runtime("a") ==
    0xaf63_dc4c_8601_ec8c`, `== hash_node_id` p/ vários `&'static str`, e
    `fnv_node_id_runtime("") == FNV_OFFSET_BASIS_64`.

B.3 [HIGH · correctness latente] disjunção de id-space NÃO gateada + consts
    `PAINTER_*` ausentes do teste de colisão de chrome. (lens 6)
    `tests/node_id_collisions.rs` (editor-core) NÃO inclui os consts fixos
    `PAINTER_LAYERS_PANEL/CLOSE/ADD/TOGGLE_DOCK`, `PAINTER_SIDEBAR_TOGGLE_DOCK`,
    `PAINTER_APPLY`, `PAINTER_COLOR_THUMB` no `CHROME_IDS` da uniqueness, NEM
    checa os ids dinâmicos `painter_layer_widget_id(layer,kind)` /
    `painter_layer_blend_option_id` contra o set de chrome. Uma colisão de slug
    misrota um clique em produção (exatamente o que o teste existe pra prevenir,
    per seu próprio docstring das 6 colisões pré-PR-11.3). **Coord:** estender
    `CHROME_IDS` com os `PAINTER_*` + um teste amostrando os ids dinâmicos.

B.4 [MED · panic latente] `queue_pointer` Q16.16 estoura p/ canvas >32768px
    ou drag ≥32768px off-origin. (lens 1)
    `tool.rs:queue_pointer` → `pointer_to_raw_sample` alimenta posição raw no
    `f32_to_q1616_saturating` (frozen `ph2d-painter-stroke/determinism.rs:44`),
    que tem `debug_assert!(|v| < 32768.0)` → **panic em debug/test**; em release
    satura e o record/WAL guarda posição clampada (data-loss no replay W12, que
    não está ligado ainda). NÃO há cap de dimensão de canvas em lugar nenhum.
    Fix limpo cruza a crate congelada (policy de canvas-size + `f32_to_q1616_
    checked` + drop/flag de sample fora da janela) → **Coord**. NÃO band-aidei
    com clamp porque distorceria silenciosamente o record de replay.

B.5 [MED · perf] churn por-frame no `painter_bridge.rs` (lens 3 F3/F4)
    - `painter.layers().clone()` (`~313`) clona o `LayerStack` inteiro TODO
      frame com a dock aberta (N Strings), e `state::current_layers()` clona
      DE NOVO → 2 deep-clones/frame, não-gateados por dirtiness.
    - `ui_snapshot()` faz `format!("brush_{}")` (heap String) toda call, 1–2×/
      frame; `painter_bridge.rs:~361` constrói um snapshot inteiro só pra ler 1
      cor.
    - **In-pasta meu (quando coordenarmos o bridge):** `current_layers()`
      devolver borrow/Rc; accessor `painter.active_color_srgb8()` direto.
    - **Coord:** gatear o publish do LayerStack num version/dirty bit.

───────────────────────────────────────────────────────────────────
§C — Follow-ups in-pasta menores que EU faço (não-bloqueantes, próximos blocos)
───────────────────────────────────────────────────────────────────
  - `current_preview` também limpar `dirty_rect` após full recompose (evita
    region-work redundante no drain seguinte; perf marginal). lens 1 LOW.
  - `remove`/active-repoint preferir "irmão sobrevivente mais próximo" a
    `root.first()` (só morde quando group-remove UI existir). lens 5 LOW.
  - Quando ligar delete/duplicate layer (pedido Enio): `delete_layer` precisa
    limpar `images`/`canvas_rgba`/undo do layer removido (espelho de
    `select_layer`) — hoje untested/unimplemented. lens 5 MED, é trabalho meu
    no batch de header-icons.

PRÓXIMO (meu): bloco de features T3.5 Mask + T3.6 Clipping + T3.7 Alpha-lock/
Reference/Group (um smoke por bloco). header-icons (delete/duplicate) junto.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
RESPOSTA DO COORDENADOR · 2026-06-01 (commit `e4d67fa`)
═══════════════════════════════════════════════════════════════════

**B.1 — FEITO (Coord, `e4d67fa`).** `SpriteRenderer::replace_individual_pixels_region`
existe agora (wrapper sobre o `replace_pixels_region` da store, sem rebuild de
bind-group). **Falta a TUA metade:** `take_preview_arc` retornar o bbox (4º campo) +
o bridge premultiplicar/subir só o sub-rect via `renderer.replace_individual_pixels_region(id, x, y, w, h, &region_rgba)`, fallback full quando bbox == canvas. **Coordena comigo
antes de mexer no `painter_bridge.rs`** (eu NÃO estou editando ele agora — tá livre;
só me avisa a mudança da tupla de retorno pra eu não pegar o arquivo no meio).

**B.2 — FEITO (Coord, `e4d67fa`).** Gate em `ids.rs` (`mod tests`): `fnv_node_id_runtime`
≡ `hash_node_id` byte-a-byte + pin do offset basis. Editar 1 dígito de qualquer twin
agora falha no `cargo test`.

**B.3 — FEITO (Coord, `e4d67fa`).** Os 16 `PAINTER_*` fixos entraram no `CHROME_IDS`
(uniqueness passou → zero colisão no set fixo) + teste novo amostrando os ids dinâmicos
(`painter_layer_widget_id`/`blend_option_id` × layer_ids × kinds × modes) contra o set
de chrome E entre si.

**B.4 — RECLASSIFICADO → in-pasta TUA (NÃO Coord, NÃO cruza contrato congelado).**
A crate congelada **já expõe `f32_to_q1616_checked(v) -> Option<i32>`** (em
`determinism.rs:86`, exportado no `lib.rs`, retorna `None` p/ `|v|>=32768` ou não-finito —
é o "caminho recomendado pra input não-validado" que o doc do `_saturating` aponta).
Logo NÃO precisa de ADR nem mudar o congelado: o fix é só o caller `pointer_to_raw_sample`
(`tool.rs:~820`) trocar `f32_to_q1616_saturating` → `f32_to_q1616_checked` e **dropar o
sample** quando vier `None` (não gravar posição clampada → sem data-loss no replay W12).
Opcional: um `MAX_CANVAS_DIM` preventivo no `canvas_size`/resize, mas o checked já
resolve o panic + a distorção. **É teu** — minutos.

**B.5 — SEQUENCIADO (misto).** Tua metade (`current_layers()` borrow/Rc +
`active_color_srgb8()` accessor) primeiro; a metade Coord (gatear o publish do LayerStack
num dirty/version bit no bridge) eu faço quando coordenarmos o `painter_bridge.rs` —
mesmo encontro do B.1. MED-perf, não bloqueia.

**§C** — teus, OK. Segue pro T3.5/3.6/3.7 + header-icons.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
RESPOSTA DO IMPLEMENTADOR · 2026-06-01 (commit `c763c4b`)
═══════════════════════════════════════════════════════════════════

**B.1 — FECHADO ponta-a-ponta (`c763c4b`).** Consumi teu wrapper.
- `tool.rs`: `take_preview_arc` rastreia o bbox recomposto; novo
  `take_preview_upload_bbox()` expõe `Some(bbox)` SÓ no fast lane multi-layer,
  `None` no trivial/full-recompose/pós-edit. Assinatura do `take_preview_arc`
  INALTERADA (zero churn de caller).
- `painter_bridge.rs`: bbox via frame-local (mesma fn → sem mexer no
  `PreviewCache` foundational). Upload parcial via
  `replace_individual_pixels_region(tex, x,y,w,h, &region)` quando há textura GPU
  casada (mesmo entity+dims, bbox in-bounds); senão full. **Invariante de sync:**
  todo partial é precedido de um full upload (o cache `composited` só é `Some`
  pós-full-recompose, que sobe `bbox==None`), e qualquer edit
  estrutural/metadata/dims/entity força full → os pixels não-tocados na GPU estão
  sempre corretos. Guard de bounds faz bbox ruim cair pra full (nunca paniqueia o
  render loop). Premul byte-space casa com o full path (preview≡Apply).
  Testes: `preview_upload_bbox_tracks_partial_vs_full`.
  **SMOKE pedido ao Enio:** pintar traço em stack ≥2 camadas — região do traço
  deve atualizar sem corromper o resto + FPS estável.

**B.4 — FECHADO (`c763c4b`).** `pointer_to_raw_sample` → `f32_to_q1616_checked`,
dropa sample fora da janela (sem clamp, sem panic). Teste
`out_of_window_pointer_sample_is_dropped`.

**B.5 — adiado (coordenado).** Minha metade (borrow em `current_layers` +
`active_color_srgb8()`) está entrelaçada com tua metade (gate do publish no
bridge), que precisa do "encontro" que não rolou (fechaste). MED-perf idle, não
bloqueia. Fica pro próximo bloco coordenado do bridge.

**⚠️ SHIP-BLOCKER teu (não mexi — ph2d-render é tua pasta):**
`crates/ph2d-render/src/individual.rs:352` `replace_pixels_region` (de `e4cffbc`/
`e4d67fa`) estoura clippy `too_many_arguments (8/7)`. `cargo check` passa mas o
`clippy --all-targets -D warnings` do CI VAI falhar. Fix: `#[allow(clippy::
too_many_arguments)]` na fn OU empacotar x/y/w/h num `Region`. Achei rodando
clippy no shell pro meu B.1 (meu código = 0 warnings).
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
PEDIDO CONSOLIDADO · Implementador · 2026-06-01 (batched a pedido do Enio)
═══════════════════════════════════════════════════════════════════
Vou abrir o bloco **T3.5 Mask + T3.6 Clipping + T3.7 Alpha-lock/Reference/Group +
header-icons (delete/duplicate)**. **VERIFIQUEI que é ~100% in-pasta:** os IconId
`Add/Copy/Delete/Duplicate/Plus/Trash` JÁ existem (`icons.rs`); nenhum variant novo
de `PainterUiEdit`/`PanelEvent` (toggles/cria/delete roteiam pelos 4 PanelEvent
existentes + kinds aditivos de `PainterLayerWidget` em `ids.rs`); compositor/tool/
panel são meus. Então **nada abaixo BLOQUEIA esse bloco** exceto o clippy no ship.
Batchando tudo que é teu pra você limpar de uma vez. Prioridade ↓:

**P0 — BLOQUEIA O SHIP (antes do próximo push):**
Clippy `too_many_arguments (8/7)` em `ph2d-render/src/individual.rs:352`
(`replace_pixels_region`) + o wrapper `renderer.rs:348`
(`replace_individual_pixels_region`). `#[allow]` ou empacotar x/y/w/h num `Region`.
CI `clippy --all-targets -D warnings` falha até fechar.

**P1 — débito de perf (B.5, tua metade; enquanto estiver no bridge):**
O bridge publica `painter.layers().clone()` todo frame (~L313) + monta
`ui_snapshot()` (allocs de String) todo frame (~L302/361), incondicional enquanto
Painter ativo. Gateie na mudança.
- **Minha metade (eu landeio primeiro se quiser):** exponho
  `PainterTool::layers_revision() -> u64` (bump em toda mutação do LayerStack) +
  `active_color_srgb8()` accessor.
- **Tua metade:** publica só quando `layers_revision` mudou; lê a cor pelo accessor
  em vez de montar snapshot inteiro.

**P2 — scaffold pra T3.8 (gestures, DEPOIS de T3.5-T3.7 — sem pressa):**
Drag-reorder do layers panel: dispatch foundational + WidgetEvent, espelho do
Hierarchy `find_hierarchy_drop` + `HierReparent` (em `interaction/dispatch/pointer.rs`).
Preciso de um `find_painter_layer_drop` (hit-test da row arrastada → slot/grupo
alvo) + um WidgetEvent `PainterLayerReorder { layer, new_parent, new_index }` que o
tool consome. Os ↑↓ são o interim; o drag é o deliverable do T3.8.

**P3 — enhancement opcional (melhora UX do T3.5+, NÃO é DoD):**
Canal de publish de THUMBNAIL-pixels por layer (bridge → panel). Hoje o panel só
recebe a ESTRUTURA do LayerStack, então as rows não mostram thumbnail real de
raster/mask. Um canal publicando um RGBA pequeno downsampled por layer-id deixaria
eu renderizar thumbnails de verdade (+ indicadores visuais de mask/clip). Baixa
prioridade; sigo com rows structure-only enquanto isso.

**Eu PAUSO aqui (decisão do Enio) e retomo T3.5 quando P0 (idealmente +P1) fechar.**
P2/P3 podem landar async. Posso landar minha metade do P1 já, se ajudar — me diz.
═══════════════════════════════════════════════════════════════════
