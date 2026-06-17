═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · continuar a implementação do Painter
Autor: Implementador Painter anterior (sessão 2026-06-01) · você roda em CONTEXTO SEPARADO
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ O Painter W3 (multi-layer) está FUNCIONAL e passou no smoke do Enio.║
║ Sua missão: LEVAR a implementação adiante (W3 que falta + W4+),     ║
║ COMEÇANDO por uma AUDITORIA COMPLETA, e programando MAIS por ciclo. ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — COMECE PELA AUDITORIA COMPLETA (mandato do Enio)
───────────────────────────────────────────────────────────────────
ANTES de qualquer feature, rode uma auditoria adversarial multi-lente do
módulo Painter inteiro (a baseline anterior foi 5 lentes, zero CRITICAL —
relatório em [[project-painter-w3-ui-plumbing]] + `HANDOFF_painter_w3_audit_coord_items.md`).
Rotacione as lentes (canon [[feedback-audit-lens-diversity]]):
  1. correctness · Rust safety · panics · HR-3 (alloc/hot-path) · data-loss
  2. color-space · premul/straight · sRGB/linear · 22 blend modes · compositor
  3. perf por-frame (composite/upload/paint — o FPS já foi caçado, confirme
     que segura; mede, NÃO chuta — vide §3)
  4. UI canônica (HR-15 tokens/strings) · Widget Gallery · dispatch · hit-test
  5. contratos congelados · persistência · determinismo HR-5 · lifecycle
  6. cobertura de testes vs claims verbais (vire claim em gate executável)
Spawne agentes read-only em paralelo (≤3 cargo-ativos por RAM; auditoria não
compila, então pode mais). Feche os achados in-pasta NA MESMA sessão (padrão-
ouro sem adiamentos, [[feedback-perfection-no-deferrals]]); foundational →
handoff Coord. Entregue um relatório curto (SEVERITY · file:line · fix).
SÓ DEPOIS parta pras features.

───────────────────────────────────────────────────────────────────
§2 — LEIA OS DOCS CANÔNICOS (não reinvente padrão)
───────────────────────────────────────────────────────────────────
Leia ANTES de codar (use o roteador — NÃO leia tudo inteiro):
  - `CLAUDE.md` (§0 os 7 inegociáveis · §1 roteador · §6 contratos congelados).
  - `docs/IntegracaoMultiAgente/DIRETRIZ.md` §0 (sanity) · §1 (papéis) · §2
    (TRIAGEM — seu 1º output) · §3.D (modificar feature existente) · §5 (UI
    canônica + Widget Gallery §5.2/§5.3) · §6 (VELOCIDADE) · §7 (git anti-colisão).
  - `SKILL_Stack_PH2D_Definitiva.md` §HR-1..18 (cite por ID).
  - **O PLANO:** `docs/Painter_projeto/15_plano_de_implementacao.md` (roadmap
    inteiro do Painter — §6 = W3, §7+ = W4..W14). É o seu mapa.
  - ADRs do Painter: `docs/architecture/decisions/0043..0053`.
  - Memória: `MEMORY.md` (índice) + `project_painter_w3_ui_plumbing_2026_06_01.md`
    (estado DETALHADO desta wave: bugs, fixes, lições, commits).
  - Handoffs vivos: `HANDOFF_painter_w3_ui_plumbing_impl.md` (a wave que fechou),
    `HANDOFF_painter_w3_audit_coord_items.md` (itens Coord pendentes).

───────────────────────────────────────────────────────────────────
§3 — RITMO: PROGRAME MAIS POR CICLO (mandato do Enio: "está muito lenta")
───────────────────────────────────────────────────────────────────
A wave anterior foi LENTA — muitos round-trips de smoke por feature minúscula
e DOIS chutes de perf antes de medir. Acelere assim:
  - **Bloco por ciclo, não migalha.** Implemente um BLOCO coerente de várias
    tarefas do plano por sessão (ex.: T3.5 Mask + T3.6 Clipping + T3.7 Alpha-lock
    juntos) ANTES de pedir smoke. Um smoke por bloco, não por botão.
  - **Auto-verifique antes de reportar.** Rode os gates + testes do módulo
    (vide §6) e clippy `--all-targets` ANTES de entregar. Não reporte meio-feito.
  - **Triage e siga.** Enio é relay mecânico ([[feedback-communication-simplicity]]);
    NÃO micro-pergunte. Decida pelo plano + padrão-ouro e prossiga. AskUser
    só em fork genuíno irreversível.
  - **MEÇA, não chute** ([[feedback-visual-bug-debug]], DIRETRIZ §5.3 regra 5).
    Bug de perf/visual: pergunte o repro EXATO (parado vs interagindo) OU
    instrumente (env-gate) ANTES de mexer. (Lição cara: o FPS-9 era o composite
    de traço; perdi 2 commits chutando clips do painel.)
  - **Reuse os padrões já estabelecidos** (§7) — não re-descubra.
  - **Inner loop = `cargo check -p <crate>`** (slot CoW, DIRETRIZ §6). Gate
    pesado batched 1× no fim do bloco, nunca por task.

───────────────────────────────────────────────────────────────────
§4 — ESTADO ATUAL (o que JÁ existe — confirme no git, NÃO refaça)
───────────────────────────────────────────────────────────────────
W0 ratificado (ADR-0043..0053, padrão-ouro). W1 (contratos/stamp pipeline/cor/
tiers/input/durabilidade/Day-7). W2 (sidebar Procreate · color thumb · eyedropper
· undo/redo). **W3 (esta wave, FUNCIONAL + smoke OK):**
  - `LayerStack` runtime (cap 8 níveis/999) + compositor CPU (22 blend W3C, HSL,
    gamma OK) com **dirty-rect** (`composite_region` do bbox dos stamps —
    O(N×bbox), corrigiu o FPS de traço).
  - Savefile v2 layer-stack (Coord, congelado ADR-0046-amд-1) + sprite-suppression
    via `PreviewOverride` (Coord — composite SUBSTITUI a sprite, não overlay).
  - **Painel de camadas interativo** (`ph2d-panel-painter-layers`): eye toggle ·
    opacity (slider puro) · blend (dropdown 22 modos) · reorder ↑↓ (base travada
    no fundo) · row-select · +Layer · **Apply CTA** (ambos painéis) · dock-toggle
    C (brush⇄layers) · scroll (wheel + scrollbar visual). Base = a sprite (sem
    blend). 3 drenos de composite (take_preview_arc/current_preview/run_full)
    todos compõem. Apply baka o composite full.
Commits desta jornada (locais): `71f5f3a`..`1074543` (UI-plumbing → z-order →
smoke fixes → reorder → Apply → scroll → audit-remediação → dirty-rect → cleanup).

───────────────────────────────────────────────────────────────────
§5 — ROADMAP (o seu trabalho; ordem do plano §6+)
───────────────────────────────────────────────────────────────────
W3 que falta: **T3.5 Mask** (R8 grayscale ligado a um Raster; o compositor tem
ponto de extensão + `collect_subtree` TODO) · **T3.6 Clipping mask** · **T3.7
Reference + Alpha-lock + Group** · **T3.8 gestures** (drag-reorder precisa de
scaffold de dispatch do Coord — vide §7; pinch-merge) · **T3.9 smoke+audit W3**.
Pedido do Enio (adiado, faça num bloco): **mover +Layer pra ícone no header +
delete + duplicar como ícones** (precisa `delete_layer` [LayerStack tem `remove`,
falta cleanup buffer/active como `select_layer`] + `duplicate_layer` novo).
Depois: **W4** (adjustment layers) · **W5+** (pipeline GPU) · W6+ (seleção/
transform, brushes) — tudo no plano. Leia o plano e siga em blocos.

───────────────────────────────────────────────────────────────────
§6 — SUA PASTA + GATES (isolamento DIRETRIZ §1.4)
───────────────────────────────────────────────────────────────────
EDITE: `crates/ph2d-tool-painter/` · `crates/ph2d-painter-brush/` ·
`crates/ph2d-panel-painter-layers/` · `crates/ph2d-panel-painter-sidebar/` ·
`crates/ph2d-editor-core/src/ids.rs` (SÓ aditivo — helpers de id, padrão
estabelecido) · `shells/desktop/src/render_loop/painter_bridge.rs` (publish +
preview). NÃO TOQUE (PARE e reporte ao Coord): `ph2d-render`, `ph2d-painter-stroke`
(savefile congelado), `shells/` além do bridge, contratos congelados (§6 CLAUDE.md).
GATES (rode batched no fim do bloco — `cargo check` ESCONDE eles,
[[feedback-full-gate-periodically]]):
  cargo test -p ph2d-editor-core --test arch_color_space_typed --test no_magic_numeric \
    --test no_literal_color --test hr12_widgets_a11y --test architecture_panel_loc_cap
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
  cargo clippy -p <suas crates> --all-targets -- -D warnings
**panel_loc_cap tem 2 caps:** por-fn (200) E por-arquivo (600) → divida em módulos
irmãos (ex.: `blend.rs`). **Bug do parser:** apóstrofo solto em comentário (`row's`)
infla a contagem — evite apóstrofos/aspas em comentários ([[project-panel-loc-gate-parser-masked-debt]]).

───────────────────────────────────────────────────────────────────
§7 — PADRÕES + ARMADILHAS (reuse; já queimaram)
───────────────────────────────────────────────────────────────────
  - **Widget por-row = id hash-derivado** (`ids::painter_layer_widget_id(layer_u64,
    kind)`) registrado no `WidgetStore` DENTRO do `paint` via `register_if_absent`
    (NÃO o allowlist companion-bit da Hierarchy — o painel tem `store_mut`). Decode
    iterando `layers × kinds` no `event.rs` (panel) e no `handle_panel_event` (tool).
  - **Dispatch:** `apply_event` é broadcast a todos os painéis até `Consumed`;
    decode via snapshot `current_layers()`. Per-row routing usa as 4 `PanelEvent`
    existentes (Click/SetValue/Toggle/SelectOption) — NÃO crie variant (contrato
    congelado). Métodos novos do PainterTool são CONCRETOS (não-trait) → não
    mexem no cap `Tool=10`/`RasterEditTool=5`.
  - **TRÊS drenos de preview compõem** (take_preview_arc · current_preview ·
    run_full). Mexeu no composite → cheque os 3.
  - **Composite é dirty-rect-gated** agora: `dirty_rect` (bbox dos stamps) +
    `composite_region` no cache; `invalidate_composite` dropa o cache (edit
    estrutural → recompose full). Edit estrutural mid-stroke = no-op guard.
  - **Clip do Vello é caro POR FRAME** — não faça `push_clip` por-row (10 = FPS
    sink). Trunque texto com `TextSystem::prefix_width` (memoizado).
  - **Color:** straight no canvas; blend em linear; premul byte-space (convenção
    project-wide, preview≡Apply). Não grave linear em buffer sRGB.
  - **base = bottom-of-root = a sprite** (sem blend; reorder trava ela no fundo).
  - UI strings em INGLÊS ([[feedback-app-ui-english-only]]).

───────────────────────────────────────────────────────────────────
§8 — COORD-SCOPE (NÃO faça; coordene) — `HANDOFF_painter_w3_audit_coord_items.md`
───────────────────────────────────────────────────────────────────
  - Persistência: host de save/load do doc Painter NÃO existe → multi-layer perde
    no reload (Coord shell). Quando existir, você adiciona `LayerStack::from_nodes`
    + setter de `next_id` (in-pasta).
  - Upload parcial GPU `replace_individual_pixels`→`replace_pixels_region` (Coord
    fez, `e4cffbc`) — CONSUMA no bridge pra fechar o dirty-rect ponta-a-ponta
    (sobe só o bbox em vez da textura full).
  - Drag-reorder do painel: precisa de scaffold de dispatch + WidgetEvent novo
    (Coord). Os ↑↓ são o interim.
  - Scrollbar thumb-drag: Coord fez o foundational (`d5146b7`,
    `widget::PAINTER_LAYERS_SCROLLBAR_ID`); FALTA a 1-linha SUA no `paint.rs`
    (`hit.register(PAINTER_LAYERS_SCROLLBAR_ID, scrollbar_thumb_rect(...))`,
    espelho do Inspector) — feche isso no 1º bloco.

───────────────────────────────────────────────────────────────────
§9 — GIT
───────────────────────────────────────────────────────────────────
Você NÃO pusha (Coord faz ship 1× no fim). `git status` antes de stage;
`git add -- <só seus paths>`; `git commit --no-verify -m 'msg' -- <paths>`
(aspas SIMPLES). WIP alheio (ph2d-render, docs, .vscode) NÃO comite.
═══════════════════════════════════════════════════════════════════
