//! ⭐⭐ **A BARRA DE MENUS** — *File · Edit · View · Window*, a barra global da **D2**.
//!
//! Enio, 2026-08-30: *«Pode tirar também os botões do topo para começarmos a trabalhar a barra
//! superior.»* Os 29 pills saíram na wave anterior; esta é o que fica no lugar deles.
//!
//! # A lei desta barra: ela REALOJA, não constrói
//!
//! ⚠️ **Quase toda linha destes quatro menus leva um id que já existia.** O *Save* é o
//! `CTX_MENU_SAVE` do `chrome::io_menu`; o *Vector* é o `TOPBAR_VECTOR` que o pill levava, e quem
//! o despacha continua a ser o painel do vetor. É a **D2** ao pé da letra
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`): *existe um sítio canónico para cada comando*
//! — e um comando com dois ids tem dois sítios a apodrecer em separado.
//!
//! ⇒ nasceram **quatro** ids de linha, e só onde não havia verbo nenhum a alcançar: a régua, o
//! tema como categoria, as preferências, e a imagem nova (que só a tecla abria).
//!
//! # ⭐ Uma TABELA, três consumidores
//!
//! [`MENUS`] é a fonte, e [`menu_rects`] é a porta:
//!
//! | quem | o que pergunta |
//! |---|---|
//! | [`paint_menu_bar`] | onde desenho cada título |
//! | [`paint_menu_bar`] (hit) | que rectângulo registo para cada título |
//! | `interaction::dispatch::pointer_down_menus` | que menu abre este clique |
//!
//! ⛔ **Foi assim de propósito.** O trilho lateral tinha a mesma aritmética escrita **TRÊS**
//! vezes — o pintor (`widget/tool_rail/paint.rs`), o registo de hit do trilho e o do flyout
//! (`left_rail.rs`) —, e nada no repo ligava as três: um pintor horizontal com um hit vertical
//! compilaria e passaria a suíte inteira. ⭐ A wave seguinte curou-o com `widget::entry_rects`;
//! ⚠️ esta nota dizia *«tem»* e *«duas»*, e ficou errada nas duas metades um commit depois.
//!
//! # ⚠️ A banda é a MESMA do chrome legado, e é uma escolha
//!
//! A barra ocupa `ChromeBands::top_bar_h`, que é a faixa que a barra de pills ocupava — logo a
//! `F9` **troca** as duas em vez de as empilhar. Duas faixas empilhadas custariam altura
//! permanente ao alvo de 1024 pontos por causa de um interruptor de bissecção.
//!
//! ⚠️ **E ela SUBTRAI altura, nunca flutua** (spec §4): a área de desenho começa por baixo dela,
//! como as colunas começam ao lado do trilho. *Uma barra global a flutuar reproduziria, num modelo
//! novo, o defeito da régua tapada que a wave anterior curou.*

use super::HeroLayout;
use super::ids;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A altura da barra — **uma linha de menu**, que é o que um título é.
///
/// ⚠️ Deriva do mesmo token que a linha de um menu aberto (`ROW_H_PX`), e não de um número
/// escolhido: o título e as linhas que ele abre são a mesma família de alvo.
pub const MENU_BAR_H: f32 = ROW_H_PX;

/// ⭐ **A TABELA** — título, id, e o menu que ele abre. A fonte única desta barra.
pub const MENUS: [(NodeId, &str, ContextMenuKind); 5] = [
    (ids::MENUBAR_FILE, "File", ContextMenuKind::MenuBarFile),
    (ids::MENUBAR_EDIT, "Edit", ContextMenuKind::MenuBarEdit),
    (ids::MENUBAR_VIEW, "View", ContextMenuKind::MenuBarView),
    (
        ids::MENUBAR_WINDOW,
        "Window",
        ContextMenuKind::MenuBarWindow,
    ),
    (ids::MENUBAR_RUN, "Run", ContextMenuKind::MenuBarRun),
];

/// Os ids de linha que **esta** barra trouxe — os que nenhum outro botão do app alcançava.
///
/// ⚠️ Existe para o registo não ser uma lista escrita à mão ao lado da tabela de rows: quem
/// acrescentar um verbo novo a um destes menus acrescenta-o aqui, e o gate
/// `every_menu_bar_row_is_registered` reprova se esquecer.
pub const OWN_ROWS: [NodeId; 6] = [
    ids::MENUBAR_FILE_NEW,
    ids::MENUBAR_FILE_SCENES,
    ids::MENUBAR_EDIT_PREFERENCES,
    ids::MENUBAR_VIEW_THEME,
    ids::MENUBAR_VIEW_RULERS,
    ids::MENUBAR_VIEW_RESET_LAYOUT,
];

/// Padding horizontal de cada título dentro do seu alvo.
///
/// ⚠️ `fn` e não `const` porque `Spacing::px` não é `const fn` — a mesma razão por que
/// `widget::tool_rail_width_px` é função. O valor continua a ser o TOKEN, que é o que importa.
fn title_pad_x() -> f32 {
    Spacing::Md.px()
}

/// Recuo da barra à esquerda, para o primeiro título não nascer colado à borda da janela.
fn bar_inset_x() -> f32 {
    Spacing::Sm.px()
}

/// Regista os quatro títulos e as quatro linhas próprias. Chamado por
/// [`super::pre_populate::populate_shared`].
///
/// ⚠️ **Sem `InteractiveState` um item de menu é pintado e nasce morto**: ele não é focável, o
/// Down não arma o `active` e o Up nunca emite `Click`. É o mesmo defeito que matou o pill
/// `[SHEET]` e os quatro pills de vetor.
pub fn populate(store: &mut WidgetStore) {
    for (id, ..) in MENUS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // ⛔ O fundo é `Plain`: ele não é um botão, é a superfície que ENGOLE o clique — ver
    // `ids::MENUBAR_BACKDROP`.
    store.register(ids::MENUBAR_BACKDROP, InteractiveState::Plain);
    for id in OWN_ROWS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⛔⛔ **O ESTADO da linha *Rulers* vai para o STORE, para o menu o poder mostrar.**
///
/// O `context_menu_overlay::id_is_currently_selected` — quem pinta o marcador de *«é este o valor
/// actual»* — recebe o `WidgetStore`, não o [`super::HeroScreen`]. A régua vive em
/// `view.rulers_visible`, que ele não alcança; sem esta publicação a linha ficaria **sem marca
/// para sempre**, e o menu diria a mesma coisa com a régua ligada e desligada.
///
/// ⚠️ **É a lei que o próprio ficheiro do overlay documenta**, paga na unidade de ângulo:
/// *«fiar o clique não é fiar o ESTADO»*. As outras quinze linhas de estado desta barra (os treze
/// módulos e os dois painéis) já têm o `ButtonState` mantido por quem as despacha — só esta não
/// tinha ninguém a publicá-la.
pub fn publish_toggle_state(hero: &mut super::HeroScreen) {
    for (id, truth) in MODULE_TRUTHS {
        let Some(on) = truth.resolve(hero) else {
            continue; // o dono é outro — ver `ModuleTruth::ShellOwned`
        };
        if let Some(InteractiveState::Button { state }) = hero.store.get_mut(id) {
            *state = if on {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

/// ⭐⭐ **ONDE VIVE A VERDADE de cada linha de alternância** — a tabela que substituiu treze
/// respostas espalhadas, das quais **dez não existiam**.
///
/// ⛔⛔ **O defeito que ela cura tinha duas caras, e a segunda é a cara.** Medido em 2026-08-30:
/// ninguém escrevia o `ButtonState` dos pills de módulo — o laço de reconciliação da shell só
/// percorre os clusters `image_tools` e `vector_tools` do registry, e um pill de módulo não está em
/// cluster nenhum (`hash_node_id("topbar_vector")` ≠ `hash_node_id("vector")`).
///
/// 1. **a marca não aparecia** — o menu *Window* dizia o mesmo com o Vector aberto e fechado;
/// 2. ⚠️ **e o `chrome::vector_toggle` LIA esse estado para escolher a direcção** (activar ou
///    cancelar). Preso em `Normal`, o segundo clique voltava a activar. *Um estado que ninguém
///    escreve e alguém lê não é uma marca em falta: é um `if` com um lado morto.*
///
/// ⭐ **Uma tabela, dois consumidores:** a marca do menu ([`publish_toggle_state`]) e a direcção do
/// toggle ([`module_is_on`]). Escrever a verdade duas vezes era como as duas se separaram.
#[derive(Copy, Clone, Debug)]
pub enum ModuleTruth {
    /// A visibilidade de um painel registado.
    Panel(&'static str),
    /// A ferramenta activa, pelo id do manifesto que a shell espelha em
    /// `ImageEditState::active_tool_id`.
    Tool(&'static str),
    /// O modo das ferramentas de imagem (`ImageEditState::mode_on`).
    ImageMode,
    /// O interruptor das réguas (`ViewState::rulers_visible`).
    Rulers,
    /// ⚠️ **Quem publica é a SHELL, e não escrevemos por cima.** O `sculpt3d` não tem flag: a
    /// verdade dele é *«há barro no ecrã»*, que só a shell vê (`sculpt3d_mode::sync`), e o
    /// doc-comment do handler explica porque ler o estado do botão ali daria a resposta errada
    /// entre uma tecla `D` e o sync seguinte.
    ShellOwned,
}

impl ModuleTruth {
    /// `None` quando o dono é outro.
    #[must_use]
    pub fn resolve(self, hero: &super::HeroScreen) -> Option<bool> {
        Some(match self {
            Self::Panel(name) => hero.is_panel_visible(name),
            Self::Tool(id) => hero.image_edit.active_tool_id == Some(id),
            Self::ImageMode => hero.image_edit.mode_on,
            Self::Rulers => hero.view.rulers_visible,
            Self::ShellOwned => return None,
        })
    }
}

/// A tabela. ⚠️ **Toda linha de alternância dos menus tem de estar aqui**, e há censo a exigi-lo
/// (`every_toggle_row_of_the_bar_is_marked_by_its_own_state`).
pub const MODULE_TRUTHS: [(NodeId, ModuleTruth); 18] = [
    (ids::TOPBAR_VECTOR, ModuleTruth::Tool("vector")),
    (ids::TOPBAR_MOTION, ModuleTruth::Tool("motion")),
    (ids::TOPBAR_FLIP, ModuleTruth::Tool("flip")),
    (ids::TOPBAR_PHYSICS, ModuleTruth::Panel("physics")),
    (ids::TOPBAR_SCULPT3D, ModuleTruth::ShellOwned),
    (ids::TOPBAR_MODEL3D, ModuleTruth::Panel("model3d")),
    (ids::TOPBAR_IMAGE_TOOLS, ModuleTruth::ImageMode),
    (ids::TOPBAR_AUDIO_MIXER, ModuleTruth::Panel("audio_mixer")),
    (ids::TOPBAR_AUDIO_EDITOR, ModuleTruth::Panel("audio_editor")),
    (ids::TOPBAR_TOKENS, ModuleTruth::Panel("tokens")),
    (ids::TOPBAR_AUTHORED, ModuleTruth::Panel("authored")),
    (
        ids::TOPBAR_WIDGET_GALLERY,
        ModuleTruth::Panel("widget_gallery"),
    ),
    (ids::TOPBAR_WIDGET_LAB, ModuleTruth::Panel("widget_lab")),
    (ids::TOPBAR_GRID_SETTINGS, ModuleTruth::Panel("grid_snap")),
    // ⭐⭐⭐ **A BIBLIOTECA** (report do Enio, 2026-09-05). ⚠️ O literal é o `PANEL_ID` do
    // `ph2d-panel-asset-browser` — esta camada é chrome e não depende de painel nenhum, que é a
    // mesma cerca das treze linhas acima.
    (
        ids::TOPBAR_RIGHT_ASSETS,
        ModuleTruth::Panel("asset_browser"),
    ),
    (ids::RAIL_SHOW_HIERARCHY, ModuleTruth::Panel("hierarchy")),
    (ids::RAIL_SHOW_INSPECTOR, ModuleTruth::Panel("inspector")),
    (ids::MENUBAR_VIEW_RULERS, ModuleTruth::Rulers),
];

/// ⭐ **Este módulo está LIGADO?** — a pergunta que um toggle faz para escolher a direcção.
///
/// ⛔ Os três activadores de ferramenta (`vector`, `motion`, `flip`) perguntavam ao
/// `store.button_state(…)`, que **ninguém escrevia** ⇒ o ramo *cancelar* nunca corria.
#[must_use]
pub fn module_is_on(hero: &super::HeroScreen, id: NodeId) -> Option<bool> {
    MODULE_TRUTHS
        .iter()
        .find(|(mid, _)| *mid == id)
        .and_then(|(_, truth)| truth.resolve(hero))
}

/// **Esta linha mostra o próprio estado pelo `ButtonState`?**
///
/// ⭐ A lista dos módulos é **derivada** da tabela do menu *Window* — um módulo novo entra sozinho.
/// As três nomeadas são as de fora dela.
#[must_use]
pub fn row_is_marked_by_button_state(id: NodeId) -> bool {
    MODULE_TRUTHS.iter().any(|(mid, _)| *mid == id)
}

/// ⭐ **A PORTA** — onde cada título fica, medido contra o texto real.
///
/// ⚠️ A largura sai do `prefix_width`, não de um número por título: *Window* é mais largo que
/// *File*, e alvos de largura fixa dariam ou um buraco ou um recorte conforme a fonte.
#[must_use]
pub fn menu_rects(
    bar: Rect,
    text_system: &mut TextSystem,
) -> [(NodeId, &'static str, Rect); MENUS.len()] {
    let font = TypeToken::Sm.px();
    let mut x = bar.x + bar_inset_x();
    std::array::from_fn(|i| {
        let (id, title, _) = MENUS[i];
        let w = text_system.prefix_width(title, font) + title_pad_x() * 2.0;
        let r = Rect::new(x, bar.y, w, bar.h);
        x += w;
        (id, title, r)
    })
}

/// Desenha a barra e regista os alvos dos títulos.
///
/// ⚠️ O título cujo menu está **aberto** fica realçado — sem isso, um menu aberto não diz de onde
/// saiu, e o segundo clique (o que o fecha) parece não ter alvo.
#[allow(clippy::too_many_arguments)] // o relógio é o 7º, como nos irmãos do chrome
pub fn paint_menu_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
) {
    let bar = layout.top_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        return;
    }
    scene.fill_rect(rect_to_vello(bar), resolve(ColorToken::Bg1, theme));
    // ⛔⛔ **PRIMEIRO o fundo, e por isso ele PERDE para os títulos** — o `HitIndex` caminha de
    // trás para a frente, então o que se regista antes fica por baixo. Sem esta linha a barra
    // pinta 1366 px de opaco e deixa passar **86,9 %** deles para o desenho.
    hit_index.register(ids::MENUBAR_BACKDROP, bar);
    let open = store.context_menu().map(|r| r.kind);
    for (id, title, r) in menu_rects(bar, text_system) {
        let state = store.button_state(id).unwrap_or(ButtonState::Normal);
        let is_open = open.is_some_and(|k| MENUS.iter().any(|(mid, _, mk)| *mid == id && *mk == k));
        // ⛔⛔ **O TÍTULO LÊ O RELÓGIO DA UI.** A 1.ª versão desta barra resolvia a cor pelo
        // `ButtonState` duro e ficava **imune ao carácter Expressivo/Discreto** — que é
        // literalmente o defeito para o qual o gate `the_chrome_reads_the_ui_clock` foi escrito
        // (Enio: *«não percebi nenhuma diferença com expressive»*). Ele continuava VERDE porque
        // afirma que o **chrome legado** passa o relógio, e o legado só pinta sob `F9`: *um gate
        // pode ser desarmado por um RAMO, sem um corte nem um rename a fazê-lo falhar alto.*
        //
        // ⚠️ **Um título com o menu ABERTO fica FORA do eixo**, pela mesma lei do chip activo do
        // trilho: *aberto* não é uma quantidade — é o estado que diz *«o menu saiu daqui»*, e
        // desvanecê-lo faria a barra piscar a resposta enquanto o menu está à vista.
        let t = crate::widget::chip_axis_t(state, is_open, motion.get(id));
        let bg = match state {
            _ if is_open => Some(ColorToken::AccentSoft),
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed => {
                Some(ColorToken::BgElev)
            }
            _ => None,
        };
        if let Some(bg) = bg {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(bg, theme));
        }
        let fg = if is_open {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        let fg = crate::widget::chip_axis_color(t, ColorToken::Text2, ColorToken::Text1, fg, theme);
        paint_text_centered(text_system, scene, title, r, TypeToken::Sm.px(), fg);
        hit_index.register(id, r);
    }
    // ⭐⭐⭐ **AS ABAS DE LAYOUT ocupam o vazio à direita** (decisões D7 e D3) — ver `layout_tabs`.
    // ⚠️ Depois dos títulos e com o `menus_end_x` deles: as abas recusam-se a pintar se não
    // couberem sem os tocar.
    let menus_end_x = menu_rects(bar, text_system)
        .iter()
        .map(|(_, _, r)| r.x + r.w)
        .fold(bar.x, f32::max);
    super::layout_tabs::paint(
        bar,
        menus_end_x,
        store.active_layout(),
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
}

/// ⛔⛔ **O clique numa linha destes menus FECHA o menu antes de qualquer um agir** — e tem de
/// correr aqui, no topo do [`super::HeroScreen::apply_event`], porque o registo de painéis é
/// caminhado **antes** do `chrome::dispatch_all`: um `Click(TOPBAR_AUDIO_MIXER)` é consumido pelo
/// painel do mixer e o chrome nunca o vê. Um fecho escrito num handler de chrome ficaria morto
/// exactamente nas treze linhas do menu *Window*.
///
/// ⚠️ **As duas linhas que ABREM outro menu ficam de fora**, e não por precaução: elas fecham e
/// reabrem por si (`chrome::menu_bar`), e fechar aqui apagaria o `last_context_menu` de que o
/// `cascade_anchor` se serve.
pub fn close_on_row_click(hero: &mut super::HeroScreen, event: WidgetEvent) {
    let WidgetEvent::Click(id) = event else {
        return;
    };
    let Some(kind) = hero.store.context_menu().map(|r| r.kind) else {
        return;
    };
    if !MENUS.iter().any(|(_, _, k)| *k == kind) {
        return;
    }
    if id == ids::MENUBAR_EDIT_PREFERENCES
        || id == ids::MENUBAR_VIEW_THEME
        || id == ids::MENUBAR_FILE_SCENES
    {
        return;
    }
    if super::menu_rows::menu_rows(kind)
        .iter()
        .any(|(rid, ..)| *rid == id)
    {
        hero.store.close_context_menu();
    }
}
