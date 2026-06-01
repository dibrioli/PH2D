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

**Item 3 — premultiply byte-space vs linear: DECISÃO REVISTA pós blast-radius =
NÃO trocar agora; agendar refactor próprio ou aceitar a convenção.**
Investigação Coord (2026-06-01) achou que o escopo é ~10× o que o handoff sugeriu:
- O premul de bake canônico é `SpriteImage::into_premultiplied()` (usa byte-space
  `premultiply_rgba8`), chamado por **8 sites de produção em TODOS os image tools**:
  color_equalization (×2), sprite_merge, bgremoval (×2), rasterize, equalize_sizes,
  painter, upscale — + os 2 previews que o espelham (`painter_bridge.rs:408`,
  `bgremoval_preview.rs:274`). Trocar p/ linear muda a cor de borda translúcida do
  bake de **todos** eles.
- **Hazard de round-trip:** `into_premultiplied` tem invariante documentada com
  `unpremultiply_rgba8` (±1/canal). O ciclo bake→re-edit faz premul e depois
  unpremul (`old_premultiplied` em `drain_painter`). Trocar só o premul p/ linear,
  deixando o unpremul byte-space, **corrompe cor ao re-editar** uma sprite pintada.
  Switch correto exige `unpremultiply_*_in_linear` casado + auditar todo par
  premul/unpremul.
- **Hazard de determinismo:** se bytes premultiplicados forem serializados em
  qualquer lugar, mudar a matemática do premul faz drift de cook-hash → gate
  replay-hash do CI. Precisa verificar antes.
- byte-space É consistente (preview==Apply, WYSIWYG holds) e round-trip-safe; o halo
  é sutil. `premultiply_rgba8_in_linear` é a matemática correta mas não tem o
  unpremul-inverso casado.

**ATUALIZAÇÃO FINAL — REVERTIDO (`3870733`). EU ERREI; NÃO MEXER.**
Cheguei a trocar pra linear (`008b5bf`) achando que era a "matemática correta".
**Estava errado e reintroduzi um bug que o Enio já tinha corrigido há tempos.**
- O halo NUNCA foi um bug vivo do byte-space: era artefato do **path do Vello**
  (`Rgba8Unorm` raw-byte). Foi corrigido **movendo o preview pro path do sprite
  shader** (`Rgba8UnormSrgb` + premul blend), onde byte-space é correto e bate
  byte-a-byte com o Apply.
- O comentário que eu SOBRESCREVI no `bgremoval_preview.rs` dizia LITERALMENTE:
  *"the gamma-correct variant (Fix C) is intentionally NOT used here — its job was
  to compensate for Vello's Rgba8Unorm raw-byte interpretation, which no longer
  applies once the preview leaves the Vello path."* Ignorei uma decisão documentada.
- `premultiply_rgba8_in_linear` é o helper vestigial do Fix-C (Vello), **sem caller
  de produção, e assim deve ficar**. A convenção byte-space (`premultiply_rgba8` /
  `unpremultiply_rgba8`) é o canônico correto pro path atual.
- **NÃO retrabalhar Item 3.** Não há halo a corrigir; o byte-space É a correção.

**⚠ NÃO-RELACIONADO mas detectado:** o gate `shell_files_respect_hr18_loc_cap` está
VERMELHO por `src/render_loop/inspector_commits.rs — 616 LOC` (cap 600) — arquivo do
**Sprite Inspector** (`ca538e4`/`ad4e918`/`546bf43`), NÃO do Painter nem meu. Owner do
Sprite Inspector deve decompor ou declarar exceção `// ph2d-loc-cap:`. Reportado por
disciplina de escopo (não fixo arquivo alheio).

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
