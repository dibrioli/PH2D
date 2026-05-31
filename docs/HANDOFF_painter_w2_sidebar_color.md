═══════════════════════════════════════════════════════════════════
HANDOFF — Implementador Painter · W2 continuação (color swatch relocation + T2.4/T2.6/T2.7)
Autor: Coordenador (sessão 2026-05-31) · você roda em JANELA DE CONTEXTO SEPARADA
Regras: docs/IntegracaoMultiAgente/DIRETRIZ.md (§1.4 obrigações, §3.D modificar feature, §5 UI canônica, §6.6 velocidade)
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ ⚑ FONTE DE VERDADE DA UI = WIDGET GALLERY (DIRETRIZ §5.2). LEIA.    ║
║ Toda UI sua DEVE usar EXATAMENTE o widget canônico que aparece no   ║
║ Widget Gallery — `ph2d-panel-widget-gallery` + `editor-core/src/    ║
║ widget/showcase/`. SEM "minha variação compacta". Pro color swatch: ║
║ o widget canônico é `ColorSwatch`/`paint_color_swatch`              ║
║ (`crates/ph2d-editor-core/src/widget/color_swatch.rs`), demonstrado ║
║ na seção `widget/showcase/color.rs`. Use ESSE, idêntico ao Gallery. ║
║ (O thumb flutuante já usava o `ColorSwatch` certo — só a POSIÇÃO    ║
║  estava errada. Você reusa o mesmo widget, dentro do painel.)       ║
╚═══════════════════════════════════════════════════════════════════╝

CONTEXTO (1 tela): o color picker do Painter foi wirado (W2.T2.3, commit `b5ba460`),
MAS o swatch de cor foi pintado como elemento FLUTUANTE no canto superior-direito do
canvas (editor-core `painter_color_thumb.rs`) → visualmente aterrissou na barra de
ferramentas do topo, ÓRFÃO, ao lado do pill PNTR — FORA do painel Painter. O Enio
sinalizou: errado. O swatch deve viver DENTRO do painel Painter (a sidebar com
Size/Opacity), como o Procreate. Esta é tua TASK 1. Depois segues a W2 (T2.4/T2.6/T2.7).

───────────────────────────────────────────────────────────────────
§0 — SANITY CHECK (rode primeiro)
───────────────────────────────────────────────────────────────────
  git log --oneline -6        # HEAD contém b5ba460 (picker wire) + f4d24d7 (gate fix)
  git status -sb -- crates/ph2d-tool-painter/ crates/ph2d-panel-painter-sidebar/
    # esperado: limpo nas tuas 2 crates
  source scripts/slot-env.sh impl-painter   # OU bash scripts/slot-seed.sh impl-painter
    # prefixe cada cargo com o CARGO_TARGET_DIR impresso (env não persiste no Bash tool)
  CARGO_TARGET_DIR=<slot> cargo check -p ph2d-tool-painter -p ph2d-panel-painter-sidebar

  ⚠️ Working tree TEM commits/WIP do Coordenador (KTX2 W2 em ph2d-render, docs,
  editor-core/shells). NADA disso é teu. Há push ZERO nesta jornada (modo acumular —
  o Coord faz ship 1× no fim). Você NÃO pusha.

───────────────────────────────────────────────────────────────────
SUA PASTA EXCLUSIVA (edite SÓ aqui — DIRETRIZ §1.4 ISOLAMENTO)
───────────────────────────────────────────────────────────────────
  crates/ph2d-tool-painter/          (tool, params, undo, color helpers)
  crates/ph2d-panel-painter-sidebar/ (o painel docado — onde o swatch DEVE ir)
  (mesmo módulo, só se a task exigir: crates/ph2d-painter-stroke/ · -brush/ · -contracts/)

NÃO TOQUE — foundational do Coord (§3.C) ou de outros. PARE e reporte se precisar:
  - crates/ph2d-editor-core/  (widget/, dispatch/, ids.rs, screens/hero*, blender_color_picker)
  - shells/                   (painter_bridge.rs, keybinds)
  - crates/ph2d-render/ · ph2d-color/ · ph2d-asset*/ · KTX2 (Coord ATIVO em ph2d-render AGORA)
  - Contratos congelados (ADR-0043/0040): caps PainterUiEdit ≤ 24 (hoje 16/24),
    PanelEvent ≤ 4, PainterUiSnapshot. Bump = (C)+ADR via Coord → PARE e reporte.

───────────────────────────────────────────────────────────────────
O QUE O COORDENADOR JÁ FEZ (NÃO refaça — confirme no git)
───────────────────────────────────────────────────────────────────
  1. ✅ **Flutuante REMOVIDO** (`6125409`): `painter_color_thumb.rs` deletado + chamada
     em hero.rs removida. O wire (id + dispatch + bridge) ficou dormente, esperando
     teu swatch registrar o hit. NÃO há swatch nenhum agora — você adiciona o certo no painel.
  2. ✅ **Click-through CORRIGIDO** (`0bcf952`): o painel Painter NÃO deixa mais pintar
     "através" dele (`cursor_over_hero_panel` agora lista `PAINTER_SIDEBAR_PANEL`;
     `painter_pointer_uv` retorna None sobre painel). Consequência pra ti: clicks no teu
     swatch (dentro do painel) NÃO disparam stroke no canvas — o dispatch do painel os recebe.
  3. MANTÉM intactos (placement-agnostic — funcionam onde quer que o hit seja registrado):
     - **Dispatch de ABRIR o picker** em `editor-core/.../dispatch/pointer.rs` (Down handler,
       keyed em `crate::ids::PAINTER_COLOR_THUMB`): no clique faz seed via
       `widget_color(PAINTER_COLOR_THUMB)` + `set_picker_target(Some(thumb))` +
       `set_blender_value(INSP_BLENDER_PICKER, …)`. + a entrada em `is_color_target_id`.
     - **Read-back/publish no bridge** (`shells/.../painter_bridge.rs`): se
       `picker_target()==PAINTER_COLOR_THUMB` lê o picker → `apply_ui_edit(SetColorSrgb)`;
       senão publica `ui_snapshot().active_color_srgb8()` → `set_widget_color(PAINTER_COLOR_THUMB)`.
  → Consequência pra você: **basta registrar o hit `PAINTER_COLOR_THUMB` na posição certa
    (dentro do painel) e pintar o swatch. O resto do wire já existe e dispara sozinho.**

───────────────────────────────────────────────────────────────────
TASK 1 (PRIORIDADE) — mover o color swatch p/ DENTRO do painel Painter
───────────────────────────────────────────────────────────────────
Pinte uma linha "Color" (swatch) no corpo do painel sidebar, junto de Size/Opacity.

Onde: `crates/ph2d-panel-painter-sidebar/src/paint.rs` — o corpo pinta rows top-down com
  o acumulador `y` (Size slider em ~linha 104, depois Opacity). Adicione uma row de cor
  (acima do Size, estilo Procreate "current color" no topo; ou abaixo do Opacity — você
  decide a ergonomia, mas DENTRO do `body_rect` clipado).
Como:
  1. Leia a cor atual de `state::current_snapshot()` → `snapshot.active_color_srgb8()`
     (`[u8;4]`, já existe no PainterUiSnapshot). É a cor a exibir no swatch.
  2. Pinte via o widget canônico de swatch do editor-core (o picker wire usou
     `ColorSwatch`/`paint_color_swatch` — confirme a API com
     `grep -rn "paint_color_swatch\|struct ColorSwatch" crates/ph2d-editor-core/src/widget/`).
     ZERO hex no teu código (gate `no_literal_color`); a user-color entra como dado `[u8;4]`,
     não literal — o widget canônico encapsula o `LITERAL-COLOR-OK`. Borda/chrome via tokens.
  3. Registre o hit: `ctx.host.hit_index_mut().register(ph2d_editor_core::ids::PAINTER_COLOR_THUMB, swatch_rect)`.
     **Reuse o id `PAINTER_COLOR_THUMB` existente** (vem de editor-core ids.rs) — é a chave que o
     dispatch de abrir + o bridge já escutam. NÃO crie id novo (quebraria o wire do Coord).
  4. Se o swatch precisar pré-registro de estado, adicione em `populate.rs` (espelhe
     `register_slider_chip_pair`). Provavelmente NÃO precisa (é só paint + hit + abrir via dispatch).
  5. Sizing/posição via tokens (`Spacing::*`, raios) — sem `f32` literal de UI (gate `no_magic_numeric`;
     `// LITERAL-PX-OK: <motivo>` só p/ geometria de chrome com justificativa).

NÃO precisa: wirar o open do picker nem o read-back — o Coord já tem (placement-agnostic).
NÃO precisa: tocar o `apply_ui_edit`/`SetColorSrgb` do tool — já existe (W2.T2.3 `b5085d9`).

DoD: Painter ativo → swatch de cor VISÍVEL no painel (não na top bar) → clique → popover
  BlenderPicker abre seedado com a cor atual → escolhe → próximo stroke usa a cor. Smoke.

───────────────────────────────────────────────────────────────────
TASKS SEGUINTES — W2 do Painter (você fecha em sequência; redijo nada, segue o plano)
───────────────────────────────────────────────────────────────────
Plano: `docs/Painter_projeto/15_plano_de_implementacao.md` §5 (T2.4/T2.6/T2.7).
  - **T2.4 — Modifier square (eyedropper-while-held)** (§5 T2.4): modifier square no centro
    da sidebar; hold + tap no canvas = eyedropper → cor vai pro primary slot. Id já existe
    (`PAINTER_SIDEBAR_MODIFIER_SQUARE`). A parte de gesto/sample no canvas que toca shell =
    PARE e reporte ao Coord (foundational); você faz o square + a lógica nas tuas crates.
  - **T2.6 — A11y nodes** (§5 T2.6): cada slider `Role::Slider` + value text; color swatch
    `Role::Button` + label; modifier square `Role::Button`. Gate `hr12_widgets_a11y` verde.
  - **T2.7 — Smoke + audit W2** (§5 T2.7): smoke do Enio + auditoria 5-lente sobre o diff
    acumulado da W2 (lentes ROTACIONADAS, NÃO reuse as de T2.1; foco: a11y coverage,
    color round-trip, contract surface, regressões de stroke).
  PERGUNTE ao Coord antes de cada uma se houver dúvida de fronteira foundational.

───────────────────────────────────────────────────────────────────
ARMADILHAS (decoradas — do módulo)
───────────────────────────────────────────────────────────────────
  1. Botão/widget novo em painel typed exige `hit_index.register` no paint E (se clicável
     via panel event) handling em `event.rs` — senão o click é dropado silenciosamente
     ([[feedback-panel-populate-register]]). Aqui o swatch é tratado pelo dispatch de
     editor-core (Coord), então só o register no paint basta — confirme no smoke.
  2. UI strings sempre em INGLÊS ([[feedback-app-ui-english-only]]) mesmo o Enio descrevendo pt-BR.
  3. Gate de workspace escondido: `cargo nextest -p <crate>` NÃO roda os arch-gates de
     workspace (`no_literal_color`, `no_bare_byte_color`, `hr12_widgets_a11y`,
     `architecture_painter_contract_surface`). Rode-os explicitamente no fechamento
     (o picker wire anterior tripou `no_bare_byte_color` e o Coord teve que consertar).
  4. `no_bare_byte_color_in_ui_or_raster_crates`: `&[u8]`/`Vec<u8]` de cor em `pub fn` das
     tuas crates precisa typed wrapper OU `// COLOR-RAW-OK: <motivo>` NA LINHA do `pub fn`
     (rustfmt move trailing-comment pós-`{` pra dentro do corpo → use assinatura multi-linha
     com o comentário na lista de params; vide undo.rs como exemplo).

───────────────────────────────────────────────────────────────────
VELOCIDADE (DIRETRIZ §6.6) + GATES no fechamento (1×, não por task)
───────────────────────────────────────────────────────────────────
  INNER LOOP = SÓ `cargo check -p ph2d-tool-painter -p ph2d-panel-painter-sidebar`.
  Slot warm CoW; 1 cargo por vez (o Coord compila ph2d-render/editor-core em paralelo, RAM 8 GiB).
  FECHAMENTO (sobre o diff acumulado):
    cargo nextest run -p ph2d-panel-painter-sidebar -p ph2d-tool-painter
    cargo test -p ph2d-editor-core --test no_literal_color --test hr12_widgets_a11y \
               --test arch_color_space_typed   # gates de workspace que tocam tuas crates
    cargo test -p ph2d-tool-painter             # architecture_painter_contract_surface
    clippy --all-targets -p ph2d-tool-painter -p ph2d-panel-painter-sidebar -- -D warnings
    ≥2 lentes adversariais ROTACIONADAS sobre o diff.

───────────────────────────────────────────────────────────────────
GIT (DIRETRIZ §7 — índice compartilhado) + REPORT
───────────────────────────────────────────────────────────────────
  `git status` ANTES de stage. WIP alheio (KTX2, editor-core/shells do Coord, .vscode,
  docs untracked, test_strip) — NÃO comite. `git add -- <só teus paths>` (NUNCA -A/-a/.
  /git stash). `git commit --no-verify -m "msg" -- <paths>` (`-m` antes do `--`).
  `git diff --cached --name-only` antes do commit. Você NÃO pusha (Coord faz ship 1× no fim).
  Ao fechar a TASK 1 reporte: "Swatch dentro do painel. Commit local <sha>. Hit
  PAINTER_COLOR_THUMB registrado em paint.rs. check+gates verdes. Smoke: <ok/pendente>."
  Depois reporte cada task seguinte. Bloqueio em fronteira foundational → PARE e reporte ao Coord.
═══════════════════════════════════════════════════════════════════
