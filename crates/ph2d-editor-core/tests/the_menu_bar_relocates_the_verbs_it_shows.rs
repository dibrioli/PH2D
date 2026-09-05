//! ⭐⭐ **A BARRA DE MENUS REALOJA VERBOS — e este gate mede o REALOJAMENTO, não o desenho.**
//!
//! A barra nasceu em 2026-08-30, no lugar dos 29 pills (Enio: *«pode tirar também os botões do
//! topo para começarmos a trabalhar a barra superior»*). A promessa dela é a **D2**: cada comando
//! tem um sítio canónico, e a barra **não constrói** verbos — ela dá casa aos que já existiam.
//!
//! ⛔ **Uma barra bonita e muda é exactamente o defeito que este repo já pagou** (o pill `[SHEET]`,
//! os quatro pills de vetor, o botão Redo): pintar é a metade fácil, e nenhum gate de pintura vê a
//! outra. Por isso cada teste aqui **carrega no botão** e mede o efeito.
//!
//! # As quatro formas de esta barra falhar em silêncio
//!
//! | # | falha | quem a apanha |
//! |---|---|---|
//! | 1 | um título pintado que não abre menu nenhum | [`pressing_a_title_opens_its_own_menu`] |
//! | 2 | uma linha do menu *Window* que ninguém despacha | [`every_window_row_reaches_a_consumer`] |
//! | 3 | o menu que **não fecha** ao escolher (o painel consome o clique antes do chrome) | [`choosing_a_row_closes_the_menu`] |
//! | 4 | a barra a **flutuar** sobre o desenho em vez de subtrair altura | [`the_bar_subtracts_height_it_never_floats`] |

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{ContextMenuKind, HitIndex, WidgetEvent, dispatch_pointer};
use ph2d_editor_core::screens::hero::menu_bar::{self, MENUS};
use ph2d_editor_core::screens::hero::{HeroScreen, ids, menu_rows::menu_rows};
use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(ph2d_editor_core::NodeId(1))
}

fn bar() -> Rect {
    Rect::new(0.0, 0.0, 1366.0, menu_bar::MENU_BAR_H)
}

/// **Carregar num título abre o MENU DELE** — pela cadeia real de ponteiro, não chamando o
/// handler à mão.
///
/// ⚠️ A âncora sai do **rectângulo** do título, e o `y` do menu tem de cair por baixo da barra:
/// um menu ancorado no cursor saltaria de sítio conforme onde no título se carregou.
///
/// ⛔ E os quatro juntos de propósito: os cinco blocos gémeos que esta tabela substituiu eram
/// exactamente a forma de um `kind` copiado do vizinho.
#[test]
fn pressing_a_title_opens_its_own_menu() {
    let mut text = TextSystem::without_system_fonts();
    let rects = menu_bar::menu_rects(bar(), &mut text);
    for (id, title, r) in rects {
        let mut h = hero();
        let mut hit = HitIndex::default();
        hit.register(id, r);
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut h.store,
            &hit,
            PointerEvent {
                x: r.x + r.w * 0.5,
                y: r.y + r.h * 0.5,
                pressure: 1.0,
                kind: PointerKind::Down,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: 0,
            },
            &arena,
        );
        let open = h
            .store
            .context_menu()
            .unwrap_or_else(|| panic!("{title}: o título é pintado e não abre menu nenhum"));
        let want = MENUS
            .iter()
            .find(|(mid, ..)| *mid == id)
            .expect("o id veio da tabela")
            .2;
        assert_eq!(open.kind, want, "{title} abriu o menu do vizinho");
        assert!(
            open.y >= r.y + r.h,
            "{title}: o menu abriu POR CIMA do título (y={}, barra acaba em {})",
            open.y,
            r.y + r.h
        );
    }
}

/// **E um id que não é título não abre menu nenhum** — o controlo negativo do `find`, que é o que
/// impede um `_ =>` de ter engolido o mundo.
#[test]
fn an_unrelated_press_opens_nothing() {
    let mut h = hero();
    let mut hit = HitIndex::default();
    let r = Rect::new(400.0, 400.0, 40.0, 20.0);
    hit.register(ids::TOOL_UNDO, r);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut h.store,
        &hit,
        PointerEvent {
            x: r.x + 1.0,
            y: r.y + 1.0,
            pressure: 1.0,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        },
        &arena,
    );
    assert!(h.store.context_menu().is_none());
}

/// ⭐ **As linhas de *File*, *Edit* e *View* chegam a um consumidor.**
///
/// ⛔⛔ **E as treze do *Window* NÃO estão aqui — não por escolha, por ALCANCE.** Elas são
/// despachadas pelos **painéis**, e o `test_support::ensure_panel_registry` desta crate é um
/// `{}`: o registry vive na `ph2d-panel-registry-init`, que depende desta. Escrevê-las aqui daria
/// quatro acusações falsas (mixer, editor de áudio, galeria, grelha) sobre código correcto.
/// ⇒ o gate delas mora em
/// `ph2d-panel-registry-init/tests/the_window_menu_reaches_every_module.rs`, que é a crate mais
/// barata que enxerga as duas cadeias. *Um gate escrito de UMA camada deixa a outra por medir.*
#[test]
fn every_other_menu_bar_row_reaches_a_consumer() {
    let mut dead = Vec::new();
    // ⭐ **A lista sai da TABELA**, menos o *Window* — um menu novo entra aqui sozinho. Aqui
    // estiveram três kinds escritos à mão, e o *Run* (que nasceu depois) não teria sido medido.
    for kind in MENUS
        .iter()
        .map(|(_, _, k)| *k)
        .filter(|k| *k != ContextMenuKind::MenuBarWindow)
    {
        for (id, label, _) in menu_rows(kind) {
            let mut h = hero();
            if !h.apply_event(WidgetEvent::Click(*id)) {
                dead.push(*label);
            }
        }
    }
    assert!(dead.is_empty(), "linhas mudas: {dead:?}");
}

/// ⛔⛔ **Escolher TIRA o menu do ecrã — e é o teste que prova a ordem do `apply_event`.**
///
/// ⚠️ A propriedade é *«este menu deixa de estar aberto»*, e não *«não há menu nenhum»*: a linha
/// *New Image…* **substitui** o menu pelo modal dela, e exigir `None` acusaria a linha correcta.
/// (A primeira redacção deste gate fez exactamente isso.)
///
/// O registo de painéis é caminhado **antes** do `chrome::dispatch_all`, então um
/// `Click(TOPBAR_AUDIO_MIXER)` é consumido pelo painel do mixer e o chrome nunca o vê. Um fecho
/// escrito num handler de chrome ficaria morto exactamente nas treze linhas do menu *Window* — o
/// artista escolheria *Audio Mixer*, o painel abriria, e o menu ficaria pousado por cima dele.
#[test]
fn choosing_a_row_closes_the_menu() {
    for kind in [
        ContextMenuKind::MenuBarWindow,
        ContextMenuKind::MenuBarFile,
        ContextMenuKind::MenuBarView,
    ] {
        let (id, label, _) = menu_rows(kind)[0];
        let mut h = hero();
        h.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind,
            });
        let _ = h.apply_event(WidgetEvent::Click(id));
        assert_ne!(
            h.store.context_menu().map(|r| r.kind),
            Some(kind),
            "{label} ({kind:?}): o menu ficou aberto depois de escolher"
        );
    }
}

/// **As DUAS linhas que abrem outra coisa substituem o menu, não o fecham.**
///
/// ⚠️ Elas estão excluídas do fecho antecipado de propósito — o `cascade_anchor` lê o menu ainda
/// aberto para saber de onde a cascata sai. Se o fecho as apanhasse, o submenu abriria no canto.
#[test]
fn the_cascading_rows_replace_the_menu() {
    for (row, want) in [
        (ids::MENUBAR_EDIT_PREFERENCES, ContextMenuKind::SettingsMenu),
        (ids::MENUBAR_VIEW_THEME, ContextMenuKind::ThemeSelector),
    ] {
        let mut h = hero();
        h.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: ContextMenuKind::MenuBarView,
            });
        assert!(h.apply_event(WidgetEvent::Click(row)));
        assert_eq!(
            h.store.context_menu().map(|r| r.kind),
            Some(want),
            "a linha tinha de SUBSTITUIR o menu pelo dela"
        );
    }
}

/// **A linha *Rulers* mexe no MESMO interruptor que as réguas lêem.**
///
/// ⚠️ A régua tinha um dono só — uma caixa dentro do painel do vetor —, e ele deixou de fazer
/// sentido no dia em que as réguas passaram a valer em todos os modos. Duas portas, **um** valor:
/// um `bool` próprio aqui faria o menu dizer *ligada* sobre uma régua que o painel desligou.
#[test]
fn the_rulers_row_flips_the_switch_the_rulers_read() {
    let mut h = hero();
    let before = h.rulers_live();
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_VIEW_RULERS)));
    assert_eq!(h.rulers_live(), !before, "a linha não mexeu na régua");
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_VIEW_RULERS)));
    assert_eq!(h.rulers_live(), before, "e volta atrás");
}

/// **A linha *New Image…* abre o modal que só a tecla abria.**
#[test]
fn the_new_image_row_opens_the_dialog() {
    let mut h = hero();
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_FILE_NEW)));
    assert_eq!(
        h.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::NewImageDialog)
    );
}

/// ⛔⛔ **A barra SUBTRAI altura — ela nunca flutua sobre o desenho.**
///
/// É a spec §4 ao pé da letra: *«a barra global deixa de flutuar sobre o conteúdo e passa a
/// subtrair altura, como o trilho subtrai largura»*. Uma barra a flutuar reproduziria, num modelo
/// novo, o defeito de 29,4 % que a wave das réguas curou — e desta vez sobre a régua de cima, que
/// nasce exactamente na borda de cima da área de desenho.
#[test]
fn the_bar_subtracts_height_it_never_floats() {
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    let layout = HeroLayout::for_viewport_bands(
        viewport,
        false,
        ChromeBands {
            rail_w: 0.0,
            top_bar_h: menu_bar::MENU_BAR_H,
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    );
    assert!(
        layout.top_bar.h > 0.0,
        "a barra tem de ter faixa — sem ela não há onde pintar os títulos"
    );
    assert!(
        layout.draw_area.y >= layout.top_bar.y + layout.top_bar.h,
        "a área de desenho começa DENTRO da barra: {:?} contra {:?}",
        layout.draw_area,
        layout.top_bar
    );
    for (name, col) in [
        ("hierarchy", layout.hierarchy),
        ("inspector", layout.inspector),
    ] {
        assert!(
            col.y >= layout.top_bar.y + layout.top_bar.h,
            "a coluna {name} começa por baixo da barra"
        );
    }
}

/// **E os títulos não se sobrepõem** — a porta única mede-os contra o texto real, e larguras
/// fixas por título dariam ou um buraco ou um recorte conforme a fonte.
#[test]
fn the_titles_are_laid_side_by_side_without_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let rects = menu_bar::menu_rects(bar(), &mut text);
    for pair in rects.windows(2) {
        let (_, a_title, a) = pair[0];
        let (_, b_title, b) = pair[1];
        assert!(
            a.x + a.w <= b.x + f32::EPSILON,
            "{a_title} e {b_title} sobrepõem-se"
        );
        assert!(a.w > 0.0, "{a_title} tem largura zero");
    }
    let last = rects[rects.len() - 1].2;
    assert!(
        last.x + last.w <= bar().x + bar().w,
        "os títulos transbordam da barra"
    );
}

/// ⛔⛔ **O QUADRO REAL: os quatro títulos ficam agarráveis onde a barra os desenha.**
///
/// ⚠️ Os outros testes deste ficheiro montam o `HitIndex` à mão a partir da porta
/// ([`menu_bar::menu_rects`]) — provam a **lei**, e um pintor que se esquecesse de registar
/// passaria por todos eles. Este pinta o hero de verdade e pergunta ao índice do quadro.
///
/// ⭐ É a mesma lição que o trilho lateral ainda paga: lá o pintor e o registo de hit são **dois
/// laços** sobre a mesma lista, com a aritmética escrita duas vezes e nada a ligá-los.
///
/// *Mutação que sangra:* apagar o `hit_index.register` do `paint_menu_bar`.
#[test]
fn the_painted_bar_registers_every_title_where_it_drew_it() {
    let mut h = hero();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, viewport, &mut scene, &mut text);
    assert!(
        !h.view.legacy_chrome,
        "precondição: sem chrome legado a barra de menus é o inquilino da faixa"
    );
    let layout = h.last_layout.expect("o quadro publicou o layout");
    for (id, title, want) in menu_bar::menu_rects(layout.top_bar, &mut text) {
        let got = h
            .hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("{title}: pintado e SEM alvo no índice do quadro"));
        assert!(
            (got.x - want.x).abs() < 0.5 && (got.w - want.w).abs() < 0.5,
            "{title}: o alvo ({got:?}) não é onde a porta diz ({want:?})"
        );
        assert!(
            got.y + got.h <= layout.draw_area.y + 0.5,
            "{title}: o alvo entra na área de desenho"
        );
    }
}

/// ⛔⛔ **AS DUAS BARRAS ENGOLEM O CLIQUE QUE PINTAM.**
///
/// `chrome_hit::pointer_over_chrome` é `panel_at().is_some() || hit_index.hit().is_some()`. As
/// barras não publicam rect de painel, logo tudo depende do `HitIndex` — e elas pintam faixas
/// **opacas** de ponta a ponta com só os títulos e os chips registados.
///
/// ⚠️ Medido pela auditoria de 2026-08-30, antes da cura: **86,9 %** da barra de menus e
/// **70,6 %** da fila de ferramentas deixavam o ponteiro passar para a arte por baixo — incluindo
/// a banda do RÓTULO que fica por cima de cada chip. Com o Painter em mãos, isso é tinta
/// depositada por baixo do chrome.
///
/// ⛔ **E o gate que devia ter apanhado isto mede a outra metade:**
/// `the_chrome_swallows_the_click_it_was_given` afirma que cada consumidor de canvas PERGUNTA ao
/// `pointer_over_chrome` — todos perguntavam. Ninguém afirmava que o chrome REGISTA um rectângulo
/// que responda que sim.
///
/// *Mutação que sangra:* apagar o `hit_index.register(ids::MENUBAR_BACKDROP, bar)` ou o
/// `hit_index.register(ids::RAIL_BACKDROP, bar)`.
#[test]
fn both_bars_swallow_every_pixel_they_paint() {
    let mut h = hero();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, viewport, &mut scene, &mut text);
    let l = h.last_layout.expect("o quadro publicou o layout");
    for (name, band) in [("barra de menus", l.top_bar), ("fila", l.tool_bar)] {
        assert!(band.w > 0.0 && band.h > 0.0, "{name}: faixa vazia");
        let mut through = 0usize;
        let mut total = 0usize;
        // Uma grelha densa: o buraco não é uma banda contínua, é tudo o que não é chip.
        let mut y = band.y + 1.0;
        while y < band.y + band.h {
            let mut x = band.x + 1.0;
            while x < band.x + band.w {
                total += 1;
                if h.hit_index.hit(x, y).is_none() {
                    through += 1;
                }
                x += 3.0;
            }
            y += 3.0;
        }
        assert_eq!(
            through, 0,
            "{name}: {through} de {total} pontos da faixa PINTADA deixam o clique passar para a \
             arte por baixo"
        );
    }
}

/// ⭐ **As dezasseis linhas de alternância mostram o próprio estado.**
///
/// ⚠️ *«Fiar o clique não é fiar o ESTADO»* — a lei que o `context_menu_overlay` documenta, paga
/// na unidade de ângulo, e que a 1.ª versão desta barra repetiu **dezasseis vezes**: o menu
/// *Window* dizia exactamente a mesma coisa com o Vector aberto e fechado.
#[test]
fn every_toggle_row_of_the_bar_is_marked_by_its_own_state() {
    let mut covered = 0usize;
    for kind in MENUS.iter().map(|(_, _, k)| *k) {
        for (id, label, _) in menu_rows(kind) {
            let is_toggle = kind == ContextMenuKind::MenuBarWindow
                || *id == ids::RAIL_SHOW_HIERARCHY
                || *id == ids::RAIL_SHOW_INSPECTOR
                || *id == ids::MENUBAR_VIEW_RULERS;
            assert_eq!(
                menu_bar::row_is_marked_by_button_state(*id),
                is_toggle,
                "{label}: a marca de estado não bate com o que a linha É"
            );
            covered += usize::from(is_toggle);
        }
    }
    // ⚠️ **O piso é DERIVADO, não escrito à mão.** Ele era `16` e ficou vermelho no dia em que o
    // menu *Window* ganhou a linha do Widget Lab — sem que nada estivesse errado. Um número
    // literal aqui obriga uma linha de UI a editar o gate de outra pessoa a cada painel novo, e o
    // sinal que isso produz (*"o teu painel partiu um gate"*) é ruído com cara de defeito.
    // A grandeza que este gate quer mesmo é *«toda linha do Window, mais as três de fora»*.
    let expected = menu_rows(ContextMenuKind::MenuBarWindow).len() + OUTSIDE_WINDOW_TOGGLES;
    assert_eq!(
        covered, expected,
        "esperadas {expected} linhas de alternância, vi {covered}"
    );
}

/// As três linhas de alternância que **não** vivem no menu *Window*: as duas colunas laterais e a
/// régua. ⚠️ Escritas como contagem porque a lista delas está no `is_toggle` acima — se lá
/// aparecer uma quarta, este número tem de subir no mesmo commit, e o gate diz qual é a diferença.
const OUTSIDE_WINDOW_TOGGLES: usize = 3;

/// **E a régua PUBLICA o estado dela**, porque quem pinta a marca não alcança o `HeroScreen`.
#[test]
fn the_rulers_row_publishes_its_state_to_the_store() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::widget::ButtonState;
    let mut h = hero();
    for on in [true, false, true] {
        h.view.rulers_visible = on;
        menu_bar::publish_toggle_state(&mut h);
        let state = match h.store.get(ids::MENUBAR_VIEW_RULERS) {
            Some(InteractiveState::Button { state }) => *state,
            other => panic!("a linha da régua não é um botão: {other:?}"),
        };
        assert_eq!(
            state == ButtonState::Pressed,
            on,
            "a marca da régua não segue o interruptor"
        );
    }
}

/// ⛔⛔⛔ **CENSO: todo verbo que a barra de pills carregava tem de ter uma porta que não seja a `F9`.**
///
/// A retirada dos pills (2026-08-30) apagou o **único** sítio de onde 29 ids eram alcançáveis, e a
/// auditoria do mesmo dia contou o que ficou sem porta: o **Painter e as dez ferramentas de
/// imagem**, a **lista de cenas** (com ela o campo de busca `CTX_SCENE_SEARCH` saiu do produto) e
/// o **rebobinar** do transporte. Nenhum gate viu — os que existiam mediam *registo* e *despacho*,
/// duas metades certas de uma pergunta que pressupõe que alguém **pinta** o controlo.
///
/// ⚠️ **A fonte é o ficheiro de ids**, não uma lista aqui: um pill novo entra no censo no dia em
/// que é declarado. As excepções vivem em [`NO_DOOR_PENDING`] **com o motivo medido**, e há a
/// metade que recusa uma entrada obsoleta.
///
/// ⛔ As ferramentas de imagem não são medidas aqui: elas não têm const `TOPBAR_*` (a fila é
/// derivada do registry), e o gate delas é
/// `ph2d-tool-registry-init/tests/every_image_tool_is_reachable_without_the_legacy_bar.rs` — a
/// crate mais barata que instala o registry.
const NO_DOOR_PENDING: &[(&str, &str)] = &[
    (
        "TOPBAR_LEFT_BACKDROP",
        "FUNDO de agrupador, nao um verbo: o efeito de o registar e' BLOQUEAR o clique.",
    ),
    ("TOPBAR_RIGHT_BACKDROP", "idem — fundo de agrupador."),
    ("TOPBAR_IMAGE_TOOLS_BACKDROP", "idem — fundo de agrupador."),
    (
        "TOPBAR_PLAY_TOGGLE",
        "id ORFAO, PRE-EXISTENTE: nunca e' pintado nem registado em lado nenhum (a auditoria de \
         2026-08-30 varreu o repo). Nao e' um verbo que perdeu a porta — e' lixo a apagar, e \
         apaga-lo e' de quem lhe mexer.",
    ),
    (
        "TOPBAR_RIGHT_LAYERS",
        "MORTO PRE-EXISTENTE: pintado, registado e com tooltip (\"Layers\"), e SEM consumidor \
         nenhum no repo inteiro — ja' o era antes desta linha existir.",
    ),
    // ⚠️ `TOPBAR_RIGHT_ASSETS` SAIU daqui em 2026-09-05: a nota dizia *«sem consumidor»* e o
    //    `ph2d-panel-asset-browser` é o consumidor desde que existe (`event.rs`) — a nota
    //    envelheceu no dia em que o painel nasceu, e o Enio encontrou o app sem forma de o abrir.
    //    Hoje a linha vive no menu *Window* (ver `LEGACY_PILL_BUTTONS`).
    (
        "TOPBAR_RIGHT_SCRIPT",
        "idem — \"Code · Luau\", sem consumidor.",
    ),
    (
        "TOPBAR_SAVE_AS",
        "o VERBO mudou-se: a linha `Save As…` do menu File leva o `CTX_MENU_SAVE_AS`, que e' o id \
         que o `io_menu` despacha. Este const era so' o pill que abria o menu.",
    ),
    (
        "TOPBAR_THEME",
        "idem — o menu dele e' o `ThemeSelector`, aberto pela linha `Theme…` do menu View.",
    ),
    (
        "TOPBAR_SETTINGS",
        "idem — o `SettingsMenu` e' aberto pela linha `Preferences…` do menu Edit.",
    ),
    // ⛔⛔ **Estas duas isenções CONTAVAM linhas, e uma contagem envelhece sozinha.** Elas diziam
    // *«as duas linhas do `SaveMenu`»* — e em 2026-09-02 o `SaveMenu` passou a ter três (o
    // *Export SVG…* da `line/Vector`), com a terceira sem casa na barra e **sem acordar gate
    // nenhum**: o censo deste ficheiro corre sobre os ids de PILL declarados, e a promessa que a
    // isenção faz é sobre as ROWS que o pill abria — outra população. ⇒ a promessa é medida por
    // `the_bar_relocated_every_row_of_the_menus_it_replaced`, que percorre as rows reais.
    (
        "TOPBAR_SAVE",
        "o verbo mudou-se: as rows do `SaveMenu` estao no menu File — medido, nao contado, por \
         `the_bar_relocated_every_row_of_the_menus_it_replaced`.",
    ),
    (
        "TOPBAR_OPEN",
        "idem para as rows do `OpenMenu`, e o mesmo gate as mede.",
    ),
    (
        "TOPBAR_PROJECT",
        "idem — a `SceneList` e' aberta pela linha `Scenes…` do menu File.",
    ),
];

#[test]
fn every_topbar_verb_has_a_door_that_is_not_the_legacy_key() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ids/chrome/topbar.rs"),
    )
    .expect("o ficheiro de ids da barra");
    // ⚠️ **O SLUG sai da linha, não de `name.to_lowercase()`.** Adivinhar o slug faria um id
    // declarado com outra convenção cair na lista de acusados por engano — e a acusação
    // *«não tem porta»* é a mais cara que este gate pode fazer.
    let declared: Vec<(String, NodeId)> = src
        .lines()
        .filter_map(|l| {
            let l = l.trim().strip_prefix("pub const ")?;
            let (name, rest) = l.split_once(':')?;
            if !name.starts_with("TOPBAR_") {
                return None;
            }
            let slug = rest.split_once("hash_node_id(\"")?.1.split_once('"')?.0;
            // ⚠️ `hash_node_id` é `const fn` sobre `&'static str`; num gate que LÊ o ficheiro o
            // slug é de tempo de execução, e vazá-lo é o preço honesto (um teste, uma corrida).
            let slug: &'static str = Box::leak(slug.to_string().into_boxed_str());
            Some((name.to_string(), ph2d_tool_registry::hash_node_id(slug)))
        })
        .collect();
    assert!(
        declared.len() >= 20,
        "só {} ids lidos — o parser deixou de reconhecer a forma do ficheiro",
        declared.len()
    );
    let rows: Vec<NodeId> = MENUS
        .iter()
        .flat_map(|(_, _, k)| menu_rows(*k))
        .map(|(id, ..)| *id)
        .collect();
    let mut doorless = Vec::new();
    for (name, id) in &declared {
        if NO_DOOR_PENDING.iter().any(|(n, _)| n == name) {
            continue;
        }
        if !rows.contains(id) {
            doorless.push(name.clone());
        }
    }
    assert!(
        doorless.is_empty(),
        "verbos da barra antiga SEM porta fora da `F9` — o artista não tem como lá chegar: \
         {doorless:?}"
    );
    // ⭐ A metade que impede a lista de virar licença: uma entrada que já tem porta sai.
    let stale: Vec<&str> = NO_DOOR_PENDING
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| declared.iter().any(|(d, id)| d == n && rows.contains(id)))
        .collect();
    assert!(
        stale.is_empty(),
        "estes já têm linha de menu e continuam na lista de excepções: {stale:?}"
    );
}

/// ⛔⛔⛔ **CENSO: a marca de uma linha de alternância tem de MEXER quando ela é clicada.**
///
/// `row_is_marked_by_button_state` diz *quais* linhas mostram estado; **este** diz se o estado
/// mudou. A diferença não é académica: uma linha cujo handler flipa um campo e nunca toca no
/// `ButtonState` fica marcada a *«desligado»* para sempre, e o menu mente com a cara de quem
/// funciona.
///
/// ⛔⛔ **E ele nasce com DEZ pendentes, todas PRÉ-EXISTENTES.** Nenhum destes dez botões tinha a
/// marca antes desta barra existir: o laço de reconciliação da shell só percorre os clusters
/// `image_tools` e `vector_tools` do **registry de ferramentas**, e os *pills* de módulo não estão
/// em cluster nenhum — `TOPBAR_VECTOR` é `hash_node_id("topbar_vector")`, o manifesto é
/// `hash_node_id("vector")`. Ninguém escreve o `ButtonState` deles.
///
/// ⚠️⚠️ **E isso é mais do que uma marca em falta:** o `chrome::vector_toggle` **LÊ** esse estado
/// para decidir a direcção (*activar* ou *cancelar*). Com ele preso em `Normal`, o segundo clique
/// volta a activar em vez de desligar. *Um estado que ninguém escreve e alguém lê não é uma marca
/// em falta — é um `if` com um lado morto.* Fica NOMEADO aqui; a cura é uma wave de quem for dono
/// dos toggles de módulo, e a verdade de cada um vive num sítio diferente (visibilidade de painel
/// para uns, ferramenta activa para outros).
/// ⛔⛔⛔ **CENSO: a marca de uma linha de alternância tem de MEXER quando ela é clicada.**
///
/// `row_is_marked_by_button_state` diz *quais* linhas mostram estado; **este** diz que o estado
/// mudou. Uma linha cujo handler flipa um campo e nunca publica a marca fica em *«desligado»* para
/// sempre, e o menu mente com a cara de quem funciona.
///
/// ⛔⛔ **A causa que ele apanhou era maior que uma marca.** Ninguém escrevia o `ButtonState` dos
/// pills de módulo (o laço da shell só percorre os clusters do registry de ferramentas), e o
/// `chrome::vector_toggle` **lia** esse estado para escolher entre *activar* e *cancelar*: preso em
/// `Normal`, o segundo clique reactivava. ⇒ a cura foi a tabela `menu_bar::MODULE_TRUTHS`, que
/// pergunta a verdade **onde ela vive**.
///
/// ⚠️ **O que esta CRATE não consegue conduzir é DERIVADO, não uma lista:**
///
/// | verdade | porque não se mede aqui |
/// |---|---|
/// | `Tool(_)` | o clique empurra `ActivateTool` para o **barramento**, e quem o drena é a shell |
/// | `ShellOwned` | o `sculpt3d` não tem flag — a verdade dele é *«há barro no ecrã»* |
/// | clique **não consumido** | o handler vive numa **crate de painel** (mixer, editor de áudio, galeria, grelha) |
///
/// ⇒ os quatro do último caso têm o gate deles em
/// `ph2d-panel-registry-init/tests/the_window_menu_reaches_every_module.rs`, que é a crate mais
/// barata que os vê. *Um gate escrito de uma camada deixa a outra por medir.*
#[test]
fn clicking_a_toggle_row_moves_its_mark() {
    use ph2d_editor_core::screens::hero::menu_bar::ModuleTruth;
    use ph2d_editor_core::widget::ButtonState;
    let mut stuck = Vec::new();
    let mut moved = Vec::new();
    for (id, label, _) in menu_rows(ContextMenuKind::MenuBarWindow) {
        let truth = menu_bar::MODULE_TRUTHS
            .iter()
            .find(|(mid, _)| mid == id)
            .map(|(_, t)| *t)
            .unwrap_or_else(|| panic!("{label}: linha do menu Window fora da tabela de verdades"));
        if matches!(truth, ModuleTruth::Tool(_) | ModuleTruth::ShellOwned) {
            continue; // conduzido pela shell — ver a tabela do doc
        }
        let mut h = hero();
        menu_bar::publish_toggle_state(&mut h);
        let before = matches!(h.store.button_state(*id), Some(ButtonState::Pressed));
        if !h.apply_event(WidgetEvent::Click(*id)) {
            continue; // despachado por uma crate de painel — o gate dele mora lá
        }
        menu_bar::publish_toggle_state(&mut h);
        let after = matches!(h.store.button_state(*id), Some(ButtonState::Pressed));
        if before == after {
            stuck.push(*label);
        } else {
            moved.push(*label);
        }
    }
    assert!(
        stuck.is_empty(),
        "linhas cuja marca não mexe depois do clique — o menu mente: {stuck:?}"
    );
    assert!(
        moved.len() >= 4,
        "só {} linhas mediram: o controlo positivo caiu e o gate passaria a não medir nada \
         ({moved:?})",
        moved.len()
    );
}

/// ⛔⛔⛔ **O RAMO *cancelar* dos três activadores de ferramenta VOLTOU A EXISTIR.**
///
/// `vector_toggle`/`motion_toggle`/`flip_toggle` escolhem entre `ActivateTool` e `CancelActiveTool`
/// lendo *«a minha ferramenta está activa?»*. Enquanto a pergunta era `store.button_state(id)` —
/// que **ninguém escrevia** — a resposta era sempre *não*: o segundo clique **reactivava**, e o
/// artista não tinha como desligar o módulo pelo menu.
///
/// ⚠️ **A metade da shell é dela**: quem espelha a ferramenta activa para
/// `ImageEditState::active_tool_id` é o `render_loop`. Aqui prova-se a **DECISÃO**, semeando o
/// espelho — que é exactamente a fronteira desta crate.
///
/// *Mutação que sangra:* `module_is_on` a devolver `None`/`false`, ou um braço a voltar ao
/// `button_state`.
#[test]
fn the_tool_toggles_can_cancel_and_not_only_activate() {
    use ph2d_editor_core::action_bus::EditorAction;
    for (id, tool) in [
        (ids::TOPBAR_VECTOR, "vector"),
        (ids::TOPBAR_MOTION, "motion"),
        (ids::TOPBAR_FLIP, "flip"),
    ] {
        // (a) desligada ⇒ o clique ACTIVA.
        let mut h = hero();
        h.image_edit.active_tool_id = None;
        assert!(h.apply_event(WidgetEvent::Click(id)));
        let acts: Vec<_> = h.bus.drain().collect();
        assert!(
            acts.iter()
                .any(|a| matches!(a, EditorAction::ActivateTool { tool_id } if *tool_id == tool)),
            "{tool}: com a ferramenta desligada o clique tinha de ACTIVAR, veio {acts:?}"
        );

        // (b) ligada ⇒ o MESMO clique CANCELA. Era este ramo que estava morto.
        let mut h = hero();
        h.image_edit.active_tool_id = Some(tool);
        assert!(h.apply_event(WidgetEvent::Click(id)));
        let acts: Vec<_> = h.bus.drain().collect();
        assert!(
            acts.iter()
                .any(|a| matches!(a, EditorAction::CancelActiveTool)),
            "{tool}: com a ferramenta ACTIVA o clique tinha de CANCELAR, veio {acts:?} — o ramo \
             está morto outra vez"
        );

        // (c) e a marca segue o espelho, nos dois sentidos.
        let mut h = hero();
        h.image_edit.active_tool_id = Some(tool);
        assert_eq!(menu_bar::module_is_on(&h, id), Some(true));
        h.image_edit.active_tool_id = Some("something_else");
        assert_eq!(
            menu_bar::module_is_on(&h, id),
            Some(false),
            "{tool}: a marca acende com a ferramenta de OUTRO módulo"
        );
    }
}
