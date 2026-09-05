//! HR-12 enforcement: every widget file under `ph2d-editor/src/widget/`
//! must wire AccessKit nodes.
//!
//! Heuristic: file imports something from `ph2d_a11y` (which exposes
//! `Node`/`NodeBuilder`/`Role`/`Action`/`NodeId` — the types every
//! widget builds its a11y node out of). Sufficient because the
//! codebase has no separate `Accessible` trait today; the contract is
//! "you import a11y and emit a `Node` from your `build_a11y` method."
//!
//! A widget that genuinely needs no a11y surface (pure paint helper
//! module, no independent user-facing semantics) must be added to
//! `A11Y_OPT_OUT` below with a 1-line justification.
//!
//! Failure modes this catches:
//! - Adding a new widget without a11y wiring.
//! - Removing a11y wiring from an existing widget.
//!
//! Both are intentional review gates — to fix, either wire a11y or
//! add an explicit opt-out entry.

#[path = "common/cfg_test_modules.rs"]
mod cfg_test_modules;
use cfg_test_modules::is_declared_under_cfg_test;

use std::fs;
use std::path::{Path, PathBuf};

/// Files that legitimately don't need a11y wiring of their own.
// ⚠️ **Um módulo de TESTE não precisa de entrada aqui — a lei partilhada responde.**
//    `common::cfg_test_modules` pergunta ao PAI (e ao AVÔ) quem gateia o ficheiro sob
//    `#[cfg(test)]`, então dez linhas escritas à mão morreram: cinco já estavam mortas
//    desde que a lei nasceu (2026-08-15) e ninguém as tinha removido, e cinco caíram com
//    a recursão que os NETOS obrigaram (`skin/tests.rs` declara `mod axis;` SEM
//    `#[cfg(test)]`, porque já está dentro de um). *A enumeração apodrece; a lei não.*
/// Each entry: (relative path under `src/widget/`, justification).
const A11Y_OPT_OUT: &[(&str, &str)] = &[
    // ⛔⛔ **Os dois passavam por COINCIDÊNCIA, e o corte do tecto de LOC revelou-o** (2026-08-30):
    // o que os fazia casar era `use ph2d_a11y::NodeId` — a importação da ESCADA DE IDS —, não uma
    // linha de a11y. Ao mover a escada para o irmão, o `scrollbar.rs` ficou sem a importação e o
    // gate acordou sobre um ficheiro que nunca teve semântica.
    //
    // ⚠️ **E a ausência é a decisão:** o polegar de uma barra não é anunciado por leitor de ecrã em
    // aplicação nenhuma — o que se anuncia é a LISTA que ele rola, e essa tem a11y própria. Dar-lhe
    // um nó poria um alvo focável entre cada par de linhas.
    (
        "scrollbar.rs",
        "geometria + pintura do polegar: nao ha' semantica a anunciar (o que se anuncia e' a lista que ele rola)",
    ),
    (
        "scrollbar_ids.rs",
        "so' a escada de ids e os dois gates de unicidade -- nao pinta nem regista nada",
    ),
    // ⚠️ **Um FRAGMENTO de linha, não um widget** — ele não recebe `NodeId` nenhum, então não há nó
    // que ele possa construir. Quem tem a semântica é a LINHA, e ela constrói-a:
    // `PropertyBox::a11y_node` (`Role::Slider` + rótulo + `numeric_value`), e o campo em si continua
    // a ser um `NumberInput` registado no store, com o `build_a11y` dele.
    // ⛔ Um nó aqui seria o SEGUNDO a descrever o mesmo número, e um leitor de ecrã anunciá-lo-ia
    // duas vezes. Saiu do `slider_with_chip.rs` em 2026-09-02 por tecto de LOC; a ausência de a11y
    // é a mesma de antes, só que agora tem ficheiro próprio e por isso é vista.
    // ⚠️ **O CORPO CLÁSSICO do interruptor** — uma das duas caras do `toggle`, escolhida pela
    // aparência (`PH2D_UI_NEW`). O nó de acessibilidade é do WIDGET (`Toggle::build_a11y`,
    // `Role::Switch`) e não muda com a cara: *fundir ou separar a tinta de um controlo não muda o
    // que ele É para quem não o vê.* ⛔ Um nó aqui seria o segundo a descrever o mesmo
    // interruptor.
    // ⚠️ **A metade da TINTA da caixa única** — cortada do `property_box.rs` por tecto de LOC em
    // 2026-09-03. O nó de acessibilidade é do MODELO (`PropertyBox::a11y_node`, `Role::Slider` com
    // rótulo e `numeric_value`), que vive no `mod.rs` ao lado. ⛔ Um nó aqui seria o segundo a
    // descrever a mesma linha.
    (
        "property_box/paint.rs",
        "fragmento sem NodeId: a semantica e' do modelo (PropertyBox::a11y_node)",
    ),
    (
        "toggle_classic.rs",
        "fragmento sem NodeId: a semantica e' do widget (Toggle::build_a11y, Role::Switch)",
    ),
    (
        "slider_with_chip/number_chip.rs",
        "fragmento sem NodeId: a semantica e' da LINHA (PropertyBox::a11y_node) e do NumberInput",
    ),
    // ⚠️ **A metade de ROLAGEM da paleta** — geometria (quanto cabe, até onde a roda vai) mais UM
    // traço indicador que **não é um controlo**: ele não se arrasta, e a decisão é explícita (um
    // alvo de arrasto ali competiria com as pílulas do cartão pelo mesmo `x`, e a roda já resolve).
    // Tudo o que é interativo na paleta é pintado e anunciado pelo pai.
    (
        "command_palette/scroll.rs",
        "geometry + a non-interactive scroll hint; every interactive item is painted and announced by the parent",
    ),
    // A tinta de FUNDO de um botão plano, misturada no eixo do hover. Aritmética de COR pura — não
    // pinta, não regista, não conhece um `NodeId`. Quem anuncia é o pintor que a chama, e cada um
    // deles já constrói o próprio nó (é precisamente por pintarem à mão que eles precisam dela).
    (
        "button_surface.rs",
        "colour arithmetic only; paints nothing, registers nothing — the calling painter owns a11y",
    ),
    // A lei de COLOCAÇÃO de um menu: dado um âncora, um tamanho e o viewport, onde é que o rect
    // pousa. Aritmética pura — não pinta, não regista, não conhece um `NodeId`. Quem anuncia é
    // quem desenha as linhas DENTRO do rect que ela devolve (os menus de contexto, e a lista do
    // "+ Track" da timeline), e cada um já constrói os seus nós.
    (
        "panel_chrome/menu.rs",
        "placement arithmetic only; paints nothing, registers nothing — the caller owns a11y",
    ),
    // The command palette's MEASURE half, split out for the widget LOC cap. It computes sizes and
    // placements and returns them — it never touches the scene nor the hit index, então não há o que
    // anunciar; quem pinta e registra é o pai (`command_palette.rs`), e é ele que constrói o nó.
    (
        "command_palette/layout.rs",
        "layout only; paints nothing, registers nothing — parent owns a11y",
    ),
    // Unit tests for `command_palette` (moved into the widget folder so the mod-sync scan stops
    // seeing them as a widget of their own) — no user-facing widget.
    // Gates do `slider` — na pasta pelo MESMO motivo (um `*_tests.rs` solto no topo de
    // `src/widget/` seria varrido como um widget). Eles PINTAM para comparar tinta, mas quem
    // anuncia a trilha é o pai (`slider.rs`), e é ele que constrói o nó.
    // A metade que DESENHA o rail. O corte é por responsabilidade (o teto de 500 LOC dos
    // primitivos forçou-o na wave da UI viva): o PAI carrega o modelo — as entradas, o preset de
    // tamanho e a árvore `Role::Toolbar` — e este filho só põe tinta. Mover o a11y para cá seria
    // dar duas casas à mesma pergunta.
    (
        "tool_rail/paint.rs",
        "paint only; parent owns the Toolbar a11y tree",
    ),
    // Unit tests for `skin` (na pasta pelo MESMO motivo do `command_palette`: um `*_tests.rs`
    // solto vira um "widget" para o gerador de `mod`) — o pai delega ao pintor real do catálogo,
    // que é quem tem a11y.
    // A outra metade dos gates da pele — *quanto da moldura ela ocupa* (BUGS_vector #26), separada
    // quando o pai cruzou o teto de 500 LOC ao catálogo ganhar o `NumberInput` e o `LevelMeter`.
    // Mesma razão do irmão acima: módulo de teste, e quem tem a11y é o pintor real do catálogo.
    // O CANAL de parâmetro por-tipo (`SkinParam` + a lei do índice marcado), separado quando o pai
    // cruzou o teto de 500 LOC. Ele é uma `struct` de dados e duas funções puras: não toca a cena,
    // não regista nada, não anuncia nada — quem pinta (e portanto quem tem a11y) é o pintor real
    // do catálogo, que o pai chama.
    (
        "skin/param.rs",
        "data channel + pure helpers; paints nothing — parent owns a11y",
    ),
    // ⚠️ Este NÃO é módulo de teste: é o CATÁLOGO (o enum `WidgetKind` e o que cada tipo É), o
    // irmão de assunto do pintor. Ele não desenha nada, então não há nó de a11y a construir — o
    // `skin.rs` ao lado é quem pinta, e é ele quem responde pela árvore.
    ("skin/kind.rs", "type catalogue; paints nothing"),
    // ⚠️ Ela responde ONDE a opção `i` caiu, e não desenha uma linha — quem pinta a família de
    // LISTA é o `skin.rs` ao lado, e é ele quem responde pela árvore de a11y das rows.
    ("skin/geometry.rs", "pure layout; paints nothing"),
    // Unit tests for `tool_rail` (split out for the widget LOC cap) — no user-facing widget; the
    // parent `tool_rail.rs` owns the a11y wiring (build_a11y / build_entry_a11y).
    // ⚠️ O QUINTO irmão com a MESMA justificação — `<slug>/tests.rs` é o padrão, e uma lista
    // que o enumera é uma lista que o sexto nasce sem. Flipar o gate para saltar todo
    // `tests.rs` dentro de um directório de widget é a cura, e mexe num gate de que ~20
    // ficheiros dependem: fica NOMEADO em vez de contrabandeado dentro desta wave.
    // Color-Harmonies engine gates — pure `partners()` math, no user-facing widget; the section is
    // painted by `harmony.rs` (which wires a11y) and the picker owns the announcements.

    // The dropdown's OPEN list, split out for the widget LOC cap. Paint only: the parent
    // `dropdown/mod.rs` builds the ComboBox node AND one `ListBoxOption` per row
    // (`build_a11y`/`build_option_a11y`), so the rows painted here are already announced.
    (
        "dropdown/popover.rs",
        "paint only; parent owns a11y for the chip and every option",
    ),
    // BlenderColorPicker sub-components: the parent `mod.rs` owns
    // the a11y tree for the whole picker. These four files are paint
    // helpers and state structs with no standalone user-facing
    // semantics. Re-evaluate if any becomes an independently
    // addressable widget.
    (
        "blender_color_picker/channels.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/hex_field.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/value_slider.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/wheel.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/preview.rs",
        "paint helper (a amostra da cor sobre o xadrez, cortada do paint.rs pelo teto de LOC na wave 3 do redesenho); parent mod owns a11y",
    ),
    // Wave 8 Phase 2.A panel chrome: shared paint helpers + constants
    // (paint_panel_surface, drag/resize hit-zone rects, clamp math,
    // HIGHLIGHTER_RGBA). No standalone user-facing semantics — each
    // panel that uses these owns its own a11y tree via its parent
    // panel manifest.
    (
        "panel_chrome.rs",
        "shared paint helpers; consumer panel owns a11y",
    ),
    // Wave 8 Phase 2.A widget gallery showcase tree: 10 section
    // painters + body orchestrator + state thread-locals. The
    // showcase paints reference widgets which DO emit a11y nodes
    // via the widget primitives they call (paint_button etc.); the
    // sections themselves are paint orchestration with no
    // independent user-facing identity. Owner panel
    // (ph2d-panel-widget-gallery) carries the a11y root.
    (
        "showcase/mod.rs",
        "paint orchestrator; owner panel carries a11y",
    ),
    ("showcase/actions.rs", "section painter; widgets emit a11y"),
    (
        "showcase/body.rs",
        "showcase orchestrator; widgets emit a11y",
    ),
    ("showcase/card.rs", "section painter; widgets emit a11y"),
    ("showcase/color.rs", "section painter; widgets emit a11y"),
    ("showcase/identity.rs", "section painter; widgets emit a11y"),
    ("showcase/inputs.rs", "section painter; widgets emit a11y"),
    (
        "showcase/inspector_w6.rs",
        "section painter; widgets emit a11y",
    ),
    ("showcase/lists.rs", "section painter; widgets emit a11y"),
    ("showcase/notes.rs", "note painter; TextInput emits a11y"),
    ("showcase/slider.rs", "section painter; widgets emit a11y"),
    (
        "showcase/state.rs",
        "thread-locals; no user-facing identity",
    ),
    ("showcase/status.rs", "section painter; widgets emit a11y"),
    ("showcase/switches.rs", "section painter; widgets emit a11y"),
    ("showcase/vector.rs", "section painter; widgets emit a11y"),
    // Canonical icon-button: a pure draw fn. The consumer chrome
    // (TopBar) registers each button's hit-rect AND its AccessKit node,
    // same split as panel_chrome / the showcase section painters.
    (
        "icon_button.rs",
        "paint helper; consumer chrome (TopBar) owns hit + a11y",
    ),
];

#[test]
fn every_widget_file_wires_a11y() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let widget_root = crate_root.join("src/widget");
    let opt_out: Vec<&str> = A11Y_OPT_OUT.iter().map(|(p, _)| *p).collect();
    let mut violations = Vec::new();
    walk(&widget_root, &widget_root, &mut |relpath, abspath| {
        if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
            return;
        }
        let rel = relpath.to_string_lossy().replace('\\', "/");
        // Top-level mod.rs is re-exports only; sub-mod.rs files in
        // composite widgets (e.g. blender_color_picker/mod.rs) ARE
        // checked because that's where the widget's a11y root lives.
        if rel == "mod.rs" {
            return;
        }
        if opt_out.contains(&rel.as_str()) {
            return;
        }
        let content = fs::read_to_string(abspath).expect("read widget file");
        let has_a11y = content.contains("use ph2d_a11y") || content.contains("ph2d_a11y::");
        if !has_a11y {
            violations.push(rel);
        }
    });

    // Wave 10 / Etapa 5.1: extend scope to panel-* crates. Panel files
    // either (a) wire a11y directly (panels with a custom a11y root —
    // e.g. composite chrome), OR (b) DELEGATE all a11y to widget
    // primitives they call (paint_button, paint_toggle, paint_slider…
    // — every widget primitive owns its own AccessKit emission). To
    // make this delegation explicit AND testable, the gate accepts
    // either: `use ph2d_a11y` import, OR a call to a canonical widget
    // primitive (`paint_button`, `paint_toggle`, etc.). Files that
    // satisfy neither (pure-paint helpers with no widget interaction)
    // go on PANEL_DELEGATE_OK below with a one-line justification.
    let crates_root = crate_root.join("..");
    let panel_delegate_ok: &[(&str, &str)] = PANEL_A11Y_DELEGATE_OK;
    let delegate_ok_paths: Vec<&str> = panel_delegate_ok.iter().map(|(p, _)| *p).collect();
    if let Ok(entries) = fs::read_dir(&crates_root) {
        let mut panel_dirs: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if path.is_dir()
                    && name.starts_with("ph2d-panel-")
                    && name != "ph2d-panel-registry-init"
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        panel_dirs.sort();
        for panel_dir in &panel_dirs {
            let src = panel_dir.join("src");
            if !src.is_dir() {
                continue;
            }
            let crate_name = panel_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            walk(&src, &src, &mut |relpath, abspath| {
                if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
                    return;
                }
                let rel = relpath.to_string_lossy().replace('\\', "/");
                // Skip non-paint panel files. State/event/id/populate/sync
                // are panel internals that DON'T paint UI (state machine,
                // input dispatch, NodeId tables, store init, value-sync).
                // The paint-orchestrator files (paint*.rs) are where a11y
                // delegation must surface.
                let base = relpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let is_paint_file = base.starts_with("paint")
                    || base == "sections.rs"  // inspector sections paint
                    || base == "paint_kinds.rs"
                    || base == "paint_rows.rs"
                    || base == "paint_helpers.rs";
                if !is_paint_file {
                    return;
                }
                if rel == "mod.rs" || rel == "lib.rs" {
                    return; // re-export / glue
                }
                let key = format!("{crate_name}/src/{rel}");
                if delegate_ok_paths.contains(&key.as_str()) {
                    return;
                }
                let content = fs::read_to_string(abspath).expect("read panel file");
                let has_direct_a11y =
                    content.contains("use ph2d_a11y") || content.contains("ph2d_a11y::");
                // Canonical widget primitives — calling these wires a11y
                // transitively (each owns its own AccessKit emission).
                let delegates_to_widgets =
                    WIDGET_DELEGATE_MARKERS.iter().any(|m| content.contains(m));
                if !has_direct_a11y && !delegates_to_widgets {
                    violations.push(key);
                }
            });
        }
    }

    assert!(
        violations.is_empty(),
        "HR-12 violation — widget/panel files without AccessKit wiring \
         (must `use ph2d_a11y::...` OR delegate to a canonical widget \
         primitive: {WIDGET_DELEGATE_MARKERS:?}):\n  {}\n\n\
         If a file genuinely has no user-facing semantics, add it to \
         A11Y_OPT_OUT (widget files) or PANEL_A11Y_DELEGATE_OK \
         (panel files) in this test with a 1-line justification.",
        violations.join("\n  "),
    );
}

/// Canonical widget primitives. Calling any of these inside a panel
/// file means a11y is wired transitively (the primitive owns its own
/// AccessKit emission). Keep in sync with `src/widget/` paint helpers.
const WIDGET_DELEGATE_MARKERS: &[&str] = &[
    "paint_button",
    "paint_toggle",
    "paint_slider",
    "paint_number_input",
    "paint_text_input",
    "paint_color_swatch",
    "paint_list_item",
    "paint_chip",
    "paint_icon_button",
    "paint_segmented",
    "paint_dropdown",
    "paint_popover",
    "paint_card",
    "panel_chrome::",
    "paint_panel_title",
    "paint_panel_surface",
];

/// Panel files that owe no a11y of their own. Each entry: (path key, why).
///
/// ⚠️ **Há DUAS categorias aqui, e o doc anterior só nomeava uma.** Ele dizia *"paint via
/// vector/text primitives only (no widget interaction → no a11y to wire)"*, e essa frase é falsa
/// para a primeira entrada da lista: o `paint_sections_chrome.rs` **regista um hit interativo**
/// (um cabeçalho dobrável é clicável). A entrada está **certa** — a a11y é do widget canónico
/// `paint_section_header`, que a emite e tem gates próprios —, mas por **DELEGAÇÃO**, não por
/// ausência de interacção.
///
/// - **(a) sem interacção nenhuma** — desenha vector/texto e não regista hit;
/// - **(b) DELEGA a um widget canónico** — regista hit, e o nó de a11y é emitido pelo primitivo
///   que ele chama.
///
/// A distinção importa para a entrada SEGUINTE: alguém que copie a categoria (a) sobre um arquivo
/// da categoria (b) está a justificar a isenção com um facto que não se verifica, e o gate — que
/// só lê o par `(caminho, razão)` — não sabe a diferença.
const PANEL_A11Y_DELEGATE_OK: &[(&str, &str)] = &[
    (
        "ph2d-panel-motion-graph/src/paint_socket.rs",
        "pure-paint helper (o glifo do socket, o halo de alvo e o dominio da porta), cortado do paint.rs pelo teto de LOC na wave 4 do redesenho; os hit rects e a a11y dos sockets ficam no pai",
    ),
    // ⛔⛔ **Este ficheiro passava por SUBCADEIA, e a remoção de uma fileira revelou-o.**
    // Ele nunca nomeou um primitivo canónico: o que casava era `paint_color_swatch_row` — o
    // helper LOCAL — contendo `paint_color_swatch` como substring. Quando a fileira «Color» do
    // Grid saiu (2026-08-30: o RGB nunca alcançava o canvas), o falso verde caiu junto.
    //
    // ⚠️ **A delegação é REAL, só que TRANSITIVA**: ele chama `paint_show_overlay_row`,
    // `paint_opacity_slider_row`, `paint_labeled_segmented_row` e `paint_kind_button_grid`, que
    // vivem nos irmãos `paint_rows.rs`/`paint_helpers.rs` e nomeiam `paint_toggle(` e
    // `paint_button(`. O modelo deste gate é *«o ficheiro NOMEIA um primitivo»*, e um salto a mais
    // não cabe nele.
    //
    // ⚠️⚠️ **E isso não é um caso isolado — foi MEDIDO: 25 ficheiros de painel passam hoje SÓ por
    // subcadeia** (`paint_number_input_row` a casar `paint_number_input`, `paint_dropdown_chip` a
    // casar `paint_dropdown`, …). ⛔ Apertar a régua para exigir a CHAMADA (`marcador(`) acusaria
    // os 25 injustamente: eles delegam de verdade, um salto mais longe. A cura honesta é o gate
    // seguir **um salto de delegação** dentro da crate — obra própria, com o número já medido.
    // ⚠️ **A seção BOOLEAN saiu do `paint_sections.rs` em 2026-09-05** (o teto de 600 LOC do painel,
    // para dar lugar à seção *Appearance*), e com ela deixou de nomear um primitivo: o ficheiro de
    // onde ela veio nomeia `paint_color_swatch(`/`paint_button(` por causa das OUTRAS seções que lá
    // vivem. ⭐ **A delegação é real e TRANSITIVA** — `self.segmented(…)`, `self.row2(…)` e
    // `self.compound_row(…)` vivem no irmão `paint_rows.rs`, que nomeia `paint_button(`. É o
    // mesmíssimo caso do `paint_body_sections.rs` abaixo, e o mesmo salto que o gate ainda não dá.
    (
        "ph2d-panel-vector/src/paint_boolean.rs",
        "seccao orquestradora: delega em `BodyCtx::segmented`/`row2`/`compound_row` (irmao `paint_rows.rs`, que nomeia `paint_button(`). Cortada do `paint_sections.rs` pelo teto de LOC",
    ),
    (
        "ph2d-panel-grid-snap/src/paint_body_sections.rs",
        "orquestrador de seccoes: delega nos irmaos `paint_rows`/`paint_helpers`, que nomeiam `paint_toggle(`/`paint_button(`. Passava por subcadeia (`paint_color_swatch_row`) ate' a fileira Color sair",
    ),
    // ⭐ **O `paint.rs` do Inspector passou a ser ORQUESTRADOR PURO em 2026-08-27** (ADR-0164 / F5):
    // a moldura do corpo — superfície, alças, cabeçalho, clip, cantos — saiu inteira para o
    // `paint_body.rs` quando a seção COMPONENT o empurrou acima do tecto de LOC. ⚠️ Ele deixou de
    // nomear um primitivo **porque deixou de pintar um pixel**, e não porque alguém tirou a
    // ligação: cada seção que ele chama emite o a11y dela, e o `paint_body` nomeia o
    // `panel_chrome::`. *A ausência aqui é a consequência de o ficheiro ter mudado de trabalho.*
    (
        "ph2d-panel-inspector/src/paint.rs",
        "orquestrador puro desde a F5; a moldura vive no `paint_body.rs` e cada secao emite o a11y dela",
    ),
    // ⭐ **O irmão de 2026-08-31**: os dois CARTOES do topo sairam do `paint.rs` (tecto de 200 LOC
    // por funcao) e levaram com eles o mesmo trabalho — ZERO. Ele nao pinta um pixel: chama
    // `paint_instance_card` e `paint_properties_card`, e o a11y de cada chip e' emitido la' dentro,
    // pelo `paint_button` canonico. *A ausencia aqui e' a consequencia de o ficheiro so' ordenar.*
    (
        "ph2d-panel-inspector/src/paint_cards.rs",
        "orquestrador puro; os dois cartoes emitem o a11y deles pelo `paint_button` canonico",
    ),
    // ⚠️ **A miniatura de um cartão não é um WIDGET — ela é o CORPO de um.** O cartão inteiro é o
    // alvo do gesto (`paint_card` regista o rect e o `InteractiveState::Button`), e esta função só
    // desenha pixels dentro dele: ela **não regista hit-rect nenhum** e não tem estado próprio.
    // ⛔ Dar-lhe semântica de a11y criaria um segundo nó focável por cima do cartão, e o leitor de
    // ecrã anunciaria duas coisas onde o artista vê uma.
    (
        "ph2d-panel-asset-browser/src/paint_thumb.rs",
        "corpo do cartao, nao widget: pinta dentro do rect que o `paint_card` ja' registou, sem hit-rect nem estado proprios",
    ),
    (
        "ph2d-panel-vector/src/paint_brush.rs",
        "a seccao BRUSH (plano 36 W4) pinta SO' por `slider_row`/`checkbox_row`/`action_button` do proprio painel, que delegam nos primitivos canonicos e registam o hit — nao ha' widget nem semantica propria aqui. ⚠️ A irma `paint_texture_pattern.rs` passa este gate por COINCIDENCIA (nomeia `ph2d_a11y::NodeId` numa assinatura de helper), nao por fiar a11y: a delegacao das duas e' a mesma, e esta e' a declaracao honesta dela",
    ),
    (
        "ph2d-panel-audio-editor/src/paint_sections_chrome.rs",
        "chrome de secção extraído do `paint_sections.rs` pelo cap de LOC — pinta pelo `paint_section_header` canónico e regista o hit pelo `ClippedHits` do painel; sem semântica própria",
    ),
    // Motion graph wire drawing — split from `paint.rs` for the 600-LOC cap. It owns no
    // widget and registers nothing: it flattens and strokes the wire splines, while every
    // a11y node for a wire (and for its routing waypoints) is registered by `hits.rs`,
    // which wires AccessKit itself. Keeping the drawing and the hit path in one file is
    // what the cap forbids; keeping them AGREEING is what `wire_path` is for (doc 44).
    // Motion graph port labels — split from `paint.rs` for the 600-LOC cap, same shape as
    // `paint_wire.rs` above: it draws text over a surface it does not own. The interactive
    // thing is the SOCKET, whose a11y node `hits.rs` registers (`push_socket_hits`); this
    // file registers no hit and consumes no gesture.
    // ⚠️ **A metade de LEITOR DE TELA do mesmo report continua ABERTA e é do painel, não deste
    // ficheiro:** o hit de socket carrega `node` + índice de `port` e nunca o nome, então a
    // árvore de AccessKit diz «porta 2». Fechá-la é dar um canal de NOME ao `hits.rs`.
    // ⭐ **A SEMENTE do painel de params — e a ausência aqui é DEMONSTRÁVEL, não prometida.**
    // Cortado do `lib.rs` pelo tecto de LOC de função (219/200) quando a linha de queixa de uma
    // regra malformada entrou. Ele escreve estado no `WidgetStore` (secções dobráveis, dicas de
    // hover, `CurvePoint`/`Button` do editor de curva e de gradiente) e **não pinta um pixel**:
    // ⚠️ `VectorScene` e `TextSystem` não estão sequer no `use` dele, então «não desenha» é uma
    // propriedade do ficheiro e não uma promessa de quem o escreveu. Cada widget que ele regista
    // emite o a11y no sítio onde é PINTADO (`rows_paint` e os irmãos), que é o mesmo modelo do
    // `paint_wire.rs` invertido: ali o desenho sem hit, aqui o hit sem desenho.
    (
        "ph2d-panel-motion-params/src/paint_seed.rs",
        "semente do store — nao pinta (nem `VectorScene` nem `TextSystem` no escopo); o a11y de cada widget e' emitido onde ele e' pintado",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_port_label.rs",
        "pure text drawing — the sockets' AccessKit nodes are registered in hits.rs",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_wire.rs",
        "pure spline drawing — the wires' AccessKit nodes are registered in hits.rs",
    ),
    // ⚠️ **A ESPÉCIE e o PAPEL** (estudo do Mini Cavalry, doc 99 §10) — a cor de um socket e o
    // selo de papel do cabeçalho. Irmão dos dois acima e pela MESMA razão demonstrável: ele
    // resolve um `ColorToken` e enche formas, e não tem `HitIndex` nem `WidgetStore` no escopo.
    // Quem regista o socket e o cartão é o `hits.rs`, onde o a11y deles já é emitido.
    (
        "ph2d-panel-motion-graph/src/paint_role.rs",
        "pure glyph drawing — the sockets' and cards' AccessKit nodes are registered in hits.rs",
    ),
    // The add-node popup's DRAW — split from `paint.rs` for the 600-LOC cap (doc 57).
    //
    // It registers nothing because the MENU registers nothing: its rows are hit-tested
    // panel-side against the full-canvas `Background` shield (`interact::apply_background`
    // → `geom::add_menu_row`), a design that predates this split by a long way (M1.E7).
    // So the a11y gap MOVED here with the split; it did not appear with it — and papering
    // over it with an `use ph2d_a11y` that registers nothing would satisfy this scan while
    // making the menu no more reachable than it is today.
    //
    // **The real debt, named:** the add-menu's 86 rows have no AccessKit nodes at all.
    // Closing it means giving each row an id + hit rect of its own instead of the shield.
    (
        "ph2d-panel-motion-graph/src/paint_menu.rs",
        "add-menu draw — its rows are hit-tested against the Background shield and register \
         no ids anywhere (pre-existing M1.E7 gap, not introduced by the split)",
    ),
    // ⚠️ **A entrada do `paint_wire_tests.rs` SAIU em 2026-08-15**, e o que a substituiu é a lei:
    // um módulo de teste é reconhecido pelo PAI que o declara, não por uma linha escrita à mão.
    // Ela era a prova de que o ponto cego existia — cada `<x>_tests.rs` novo pedia outra igual, ou
    // passava por acidente ao mencionar um pintor canónico.
    (
        "ph2d-panel-motion-graph/src/paint_stamp.rs",
        "the card's postage stamp — pure drawing inside the CARD, whose AccessKit node is hits.rs's",
    ),
    // The ⚠ inert badge's DRAW (ADR-0155) — split from `paint.rs` for the panel caps, the
    // same cut as `paint_stamp.rs` above and with the same a11y story: the pip is CLICKABLE
    // (it asks for the quick-fix), and the node that makes it reachable is registered by
    // `hits::push_inert_badge_hit`, which owns the `ph2d_a11y::NodeId`. Wiring a11y here
    // too would register the badge TWICE under two ids — the drawing must not have an
    // opinion about identity that the hit path already answers.
    (
        "ph2d-panel-motion-graph/src/paint_inert_badge.rs",
        "the inert-warning pip — pure drawing; its AccessKit node is push_inert_badge_hit's",
    ),
    // The transient canvas overlays (wire ghost / probe / knife / marquee / add-menu) — split
    // from `paint.rs` for the 200-LOC/fn + 600-LOC/file caps, the same cut as the four siblings
    // above. It registers nothing because none of the five HAS an identity to register: four
    // are feedback about a gesture the interaction layer already owns, and the menu's rows are
    // hit-tested against the full-canvas `Background` shield that `paint` pushes after this
    // call (the pre-existing M1.E7 gap that `paint_menu.rs` above already names).
    (
        "ph2d-panel-motion-graph/src/paint_overlays.rs",
        "transient gesture feedback + the add-menu draw — registers no ids anywhere",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_grid.rs",
        "the canvas backdrop lattice — decoration with no semantics: nothing hit-tests it",
    ),
    // Painter Brush appearance sections (6–11) — a thin ORCHESTRATOR split from
    // `paint_brush.rs` for the 200-LOC/fn + 600-LOC/file caps. It owns no widget:
    // every row it paints is a call into a section module (`paint_shape`,
    // `paint_texture`, `paint_stroke`, `paint_symmetry`, `paint_watercolor`, …),
    // and each of those wires its own AccessKit nodes via the canonical primitives.
    (
        "ph2d-panel-painter-layers/src/paint_brush_sections.rs",
        "orchestrator only — every section it calls wires its own a11y (paint_shape/_texture/_stroke/…)",
    ),
    // Vector path-reshape subsection — a thin section painter split from
    // `paint_sections.rs` for the LOC cap. Its buttons delegate to
    // `BodyCtx::row2` / `action_button` (in paint_sections), which paint via the
    // a11y-wired `paint_button` primitive; this file has no widget of its own.
    // As QUATRO seções compartilhadas do Inspector (§5 9-Slice, §7 Ordering, §9 Sampling,
    // §10 Material & Blend). ⚠️ Orquestrador PURO: ele decide moldura, separador e slot de nota,
    // e cada corpo é pintado por `sections::paint_*_section`, que são os ficheiros com a fiação
    // de a11y. Um widget próprio aqui seria a duplicação que este corte existe para evitar.
    (
        "ph2d-panel-inspector/src/paint_frame_shared.rs",
        "orquestrador puro: delega os quatro corpos a `sections::paint_*_section` (a11y-wired)",
    ),
    (
        "ph2d-panel-vector/src/paint_arrange.rs",
        "delegates to row2/action_button (paint_button-backed) in paint_sections",
    ),
    // A seção FRAME (plano UI/UX W0) — o contêiner. Um `button_grid` (os presets de dispositivo),
    // de `paint_sections`/`paint_rows`.
    // ⚠️ O `segmented` do *Clip content* MUDOU-SE daqui para a seção irmã em 2026-08-21, quando o
    // recorte deixou de ser fato de moldura — a entrada abaixo é a outra metade desta.
    (
        "ph2d-panel-vector/src/paint_frame.rs",
        "delegates to segmented/button_grid (paint_segmented/paint_button-backed) in paint_sections",
    ),
    // A seção CLIP — o `segmented` Off/On que SAIU da FRAME acima. Mesma delegação, e o gate só o
    // vê agora porque o corte deixou o arquivo com a chamada indireta e mais nada.
    (
        "ph2d-panel-vector/src/paint_clip.rs",
        "delegates to segmented (paint_segmented-backed) in paint_sections — the half that left paint_frame.rs",
    ),
    // A seção LAYOUT (plano UI/UX W2, ADR-0153) — a moldura que EMPILHA. Irmã exacta da FRAME
    // acima: quatro `segmented` e cinco `number_cell`/`number_row`, todos de `paint_rows`.
    (
        "ph2d-panel-vector/src/paint_layout.rs",
        "delegates to segmented/number_row/number_cell (paint_segmented/paint_number_input-backed) \
         in paint_rows",
    ),
    // ⚠️ O `label_line` desta seção pinta TEXTO e regista NADA — de propósito. Ele é o readout de
    // *"main missing"*, um facto a LER; um facto a AGIR é um botão, e esses são os quatro
    // `action_button` (paint_button-backed) logo ao lado. Dar-lhe um nó de AccessKit sem id nem
    // hit-rect seria anunciar um controlo que não existe.
    (
        "ph2d-panel-vector/src/paint_components.rs",
        "delegates to action_button (paint_button-backed) in paint_rows — o único não-botão é o \
         readout de texto, que não é um controlo",
    ),
    (
        "ph2d-panel-vector/src/paint_anchors.rs",
        "delegates to segmented (paint_segmented-backed) in paint_rows — a seção só tem as duas \
         fileiras de chips",
    ),
    // ⚠️ O Transform ENTROU nesta lista em 2026-08-02, e a razão é um MOVE: o `number_cell` que o
    // fazia casar com o marcador `paint_number_input` mudou-se para `paint_rows.rs` quando o AUTO
    // LAYOUT virou o segundo consumidor dele. O arquivo ficou orquestrador puro — a fiação de
    // AccessKit não se perdeu, mudou de casa junto com o primitivo.
    (
        "ph2d-panel-vector/src/paint_transform.rs",
        "orquestrador — o number_cell que ele chama (paint_number_input-backed) vive em paint_rows",
    ),
    // O TRILHO da rampa do Gradient Map — split de `paint_filters.rs` pelo teto de 600 LOC.
    //
    // Os controles dele DELEGAM: o `+`/`−` vão por `BodyCtx::filter_icon` (→ `paint_icon_button`) e
    // a cor do stop por `BodyCtx::filter_color_swatch` (→ `paint_color_swatch`), ambos no irmão.
    //
    // **A dívida real, nomeada:** os PUNHOS (um por stop) não têm nó AccessKit nenhum — eles são
    // `InteractiveState::CurvePoint` desenhados como círculos. Isso é PRÉ-EXISTENTE e compartilhado
    // com o editor de rampa do próprio Painter (`paint_ramp_widget.rs`, cujo único a11y é um `use
    // ph2d_a11y::NodeId` — import de TIPO, que satisfaz este scan sem registrar nada), e não
    // apareceu com este split: fechá-la é dar nó + rect próprios a cada punho, no primitivo
    // compartilhado, para os dois consumidores de uma vez. Um `use ph2d_a11y` decorativo aqui
    // silenciaria o scanner sem tornar um punho mais alcançável do que é hoje.
    (
        "ph2d-panel-vector/src/paint_filters_ramp.rs",
        "trilho: +/− e swatch delegam a filter_icon/filter_color_swatch no irmão; os punhos de \
         arrasto não têm a11y (gap pré-existente, igual ao ramp widget do Painter)",
    ),
    // Flip *tip* selector (Line/Dots/Squares + Spacing) — a thin section painter split from
    // `paint_sections.rs` for the 600-LOC cap. It owns no widget: it delegates to
    // `BodyCtx::segmented` / `slider_row` (in paint_rows/paint_sections), which paint via the
    // a11y-wired `paint_segmented` / `paint_slider` primitives.
    (
        "ph2d-panel-flip/src/paint_tip.rs",
        "delegates to BodyCtx::segmented/slider_row (paint_segmented/paint_slider-backed)",
    ),
    // Flip BRUSH card (Size / Hardness / Opacity / Smoothing / a dinâmica de pressão) — irmão
    // exato do `paint_tip.rs` acima: split do `paint_sections.rs` pelo teto de 600 LOC, sem
    // widget PRÓPRIO. Toda linha dele é `BodyCtx::slider_row` / `slider_row_linked` /
    // `section_label` (em `paint_rows`), que pintam pelos primitivos a11y-wired.
    //
    // ⚠️ **Antes do split isto passava por ACIDENTE**, e vale registrar: o `brush` morava no
    // `paint_sections.rs` junto do `color`, que chama `paint_color_swatch` — um marcador
    // canônico em QUALQUER ponto do arquivo satisfaz o scan para o arquivo INTEIRO. A
    // delegação do brush sempre foi a de uma hop mais funda; o split só a tornou visível.
    (
        "ph2d-panel-flip/src/paint_brush.rs",
        "delegates to BodyCtx::slider_row/slider_row_linked (paint_slider/paint_number_input-backed) in paint_rows",
    ),
    // Vector connector subsection (Route / Jetty / Spread) — mesmo caso do
    // `paint_arrange` acima: a seção não tem widget PRÓPRIO. As três linhas dela são
    // `BodyCtx::labeled_choice_button` / `labeled_number_field` (em `paint_modes`), que
    // pintam pelos primitivos a11y-wired `paint_segmented_button` e
    // `paint_number_input_with_buffer` — os MESMOS que desenham os parâmetros de forma.
    (
        "ph2d-panel-vector/src/paint_connector.rs",
        "delegates to labeled_choice_button/labeled_number_field (paint_segmented/paint_number_input-backed) in paint_modes",
    ),
    // CEQ histogram strip — pure RGB-bar visualization (read-only chart,
    // no widget interaction, no AccessKit semantics). Split out from
    // `paint.rs` to satisfy Wave 10 LOC cap.
    (
        "ph2d-panel-color-equalization/src/paint_histogram.rs",
        "read-only histogram visualization, no a11y semantics",
    ),
    // Falloff curve editor — render half only. Its interactive elements get their
    // a11y from the registered-widget system, not this file: the +/− buttons are
    // registered widgets (populate + event drain), and the draggable curve handles
    // are dispatched in editor-core (the 2D-drag BlenderHit pattern).
    // TODO(a11y follow-up): wire AccessKit nodes for the curve handles themselves.
    (
        "ph2d-panel-painter-layers/src/paint_falloff.rs",
        "falloff-curve render half; handles dispatched in editor-core, buttons are registered widgets",
    ),
    // Stencil card — its number boxes delegate to `number_field` (the a11y-wired NumberInput
    // primitive); the card background + labels are decorative chrome.
    (
        "ph2d-panel-painter-layers/src/paint_stencil.rs",
        "number boxes delegate to number_field (a11y-wired NumberInput); rest is decorative chrome",
    ),
    // Flatten/rotate gizmo — its two handles are `CurvePoint`s dispatched in editor-core (the same
    // pattern as paint_falloff); the rim + ellipse + axes are a decorative template.
    (
        "ph2d-panel-painter-layers/src/paint_shape_dab.rs",
        "gizmo handles are CurvePoints dispatched in editor-core; rest is a decorative template render",
    ),
    // TAPER widget (Procreate Touch Taper) — the two length handles are `CurvePoint`s dispatched in
    // editor-core (the paint_shape_dab pattern), the Link toggle delegates to `paint_checkbox_row` and
    // the Tip / Opacity rows to `number_field` (both a11y-wired); the stroke silhouette is a decorative
    // preview render of the engine's own width law.
    (
        "ph2d-panel-painter-layers/src/paint_taper.rs",
        "taper handles are CurvePoints dispatched in editor-core; toggle + rows delegate to a11y-wired primitives",
    ),
    // Wet Paint TILT dial (doc 22) — the pad is a `CurvePoint` dispatched in editor-core (the
    // paint_shape_dab pattern) and its toggle delegates to `paint_checkbox_row` (the a11y-wired
    // Checkbox); the polar grid + knob are a decorative template render.
    (
        "ph2d-panel-painter-layers/src/paint_wetpaint_tilt.rs",
        "tilt pad is a CurvePoint dispatched in editor-core; toggle delegates to paint_checkbox_row",
    ),
    // Watercolor section — its Wet-edges / Pigment checkboxes delegate to `paint_checkbox_row`
    // (the a11y-wired Checkbox) and the Edge / Spread / Granulation / Mix sliders to `number_field`
    // (the a11y-wired NumberInput); the collapsible header + labels are decorative chrome. Same
    // delegation as `paint_stencil.rs` (its helpers just don't happen to name a canonical primitive).
    (
        "ph2d-panel-painter-layers/src/paint_watercolor.rs",
        "checkboxes delegate to paint_checkbox_row (Checkbox); sliders to number_field (NumberInput); rest is chrome",
    ),
];

fn walk(root: &Path, dir: &Path, cb: &mut dyn FnMut(&Path, &Path)) {
    for entry in fs::read_dir(dir).expect("read widget dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, cb);
        } else if let Ok(rel) = path.strip_prefix(root) {
            // ⚠️ **Um módulo de TESTE não tem a11y a fiar — e passava por ACIDENTE.** Um
            // `<x>_tests.rs` chama os pintores que testa, e um marcador canónico em QUALQUER
            // ponto do ficheiro satisfazia esta varredura: verde pelo motivo errado. A pergunta
            // é feita ao PAI (`tests/common/cfg_test_modules.rs`), a mesma lei que a `hr15`
            // pagou primeiro — e cuja terceira grafia (o irmão PLANO) só apareceu aqui.
            if is_declared_under_cfg_test(&path) {
                continue;
            }
            cb(rel, &path);
        }
    }
}
