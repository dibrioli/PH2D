//! ⭐⭐ **A galeria em CARTÕES no encaixe da direita** (doc `CartoesUI/08` §1) — os gestos reais
//! (pointer → despacho → `apply_event` → pintura seguinte) sobre o vocabulário do cartão.

use ph2d_a11y::NodeId;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelHost, PanelHostInternal, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, slot_tabs};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_widget_gallery::ids as g;
use ph2d_panel_widget_gallery::state::{Choices, WidgetGalleryState};
use ph2d_panel_widget_gallery::{WidgetGalleryPanel, force_cards_on_this_thread};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

const SEC: u128 = 1_000_000_000;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Alta o bastante para os sete cartões do vocabulário caberem sem rolar.
fn alta() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H * 4.0)
}

struct Galeria {
    host: MockPanelHost,
    st: WidgetGalleryState,
    rects: Vec<(NodeId, Rect)>,
    t: u128,
}

impl Galeria {
    fn aberta() -> Self {
        force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<WidgetGalleryPanel>();
        host.set_panel_visible(WidgetGalleryPanel::ID, true);
        let mut gal = Self {
            host,
            st: WidgetGalleryState::default(),
            rects: Vec::new(),
            t: SEC,
        };
        gal.repinta();
        gal
    }

    /// Pinta até a lista ASSENTAR: as alturas são as do quadro anterior e a mola leva a diferença
    /// no relógio real — um cartão que cresce empurra os de baixo ao longo de vários quadros.
    fn repinta(&mut self) {
        let mut iguais = 0;
        for _ in 0..120 {
            let agora = self.host.paint::<WidgetGalleryPanel>(&mut self.st, alta());
            iguais = if agora == self.rects { iguais + 1 } else { 0 };
            self.rects = agora;
            if iguais >= 2 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) {
        self.t += SEC;
        let evs = self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: self.t,
        });
        for ev in evs {
            let _ = self
                .host
                .apply_panel_event::<WidgetGalleryPanel>(&mut self.st, ev);
        }
    }

    fn clica(&mut self, id: NodeId) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.ponteiro(PointerKind::Down, x, y);
        self.ponteiro(PointerKind::Up, x, y);
        self.repinta();
    }

    /// Arrasta o ÍCONE do campo (a célula da esquerda) `dx` px para a direita.
    fn arrasta_o_icone(&mut self, id: NodeId, dx: f32) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        let (x, y) = (r.x + 2.0, r.y + r.h * 0.5);
        self.ponteiro(PointerKind::Down, x, y);
        for i in 1..=4 {
            self.ponteiro(PointerKind::Move, x + dx * i as f32 / 4.0, y);
        }
        self.ponteiro(PointerKind::Up, x + dx, y);
        self.repinta();
    }

    fn valor(&self, id: NodeId) -> f64 {
        self.host.store().number_value(id).expect("campo registado")
    }
}

fn fabrica(id: NodeId) -> f64 {
    g::FIELDS
        .iter()
        .find(|f| f.0 == id)
        .map(|f| f.1)
        .expect("campo da tabela")
}

/// (1) Com a galeria visível no modo cartões, a lista pinta-se DENTRO do encaixe da direita (o
/// `ctx.slot` do dock, como o Inspector) e a roda rola-a. *Mutação: `CAN_FLOAT = true` ⇒ a galeria
/// não ocupa o encaixe; a lista pinta-se num slot que não é o dela.*
#[test]
fn a_lista_de_cartoes_mora_no_encaixe_da_direita_e_rola() {
    force_cards_on_this_thread(Some(true));
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<WidgetGalleryPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(WidgetGalleryPanel::ID, true);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..3 {
        paint_hero_screen(&mut hero, viewport(), &mut scene, &mut text);
    }
    assert!(
        slot_tabs::occupied(&hero).contains(Slot::RightTop),
        "a galeria visível não ocupa o encaixe da direita"
    );
    let layout = hero.last_layout.expect("o quadro publicou o layout");
    let encaixe = layout
        .slot_rects(slot_tabs::occupied(&hero))
        .get(Slot::RightTop);
    let lista = hero
        .store
        .panel_rect(ids::GAL_PANEL)
        .expect("a lista publicou o rect");
    assert!(
        lista.x >= encaixe.x - 0.5
            && lista.x + lista.w <= encaixe.x + encaixe.w + 0.5
            && lista.w > 0.0,
        "a lista {lista:?} saiu do encaixe {encaixe:?}"
    );
    let (cx, cy) = (lista.x + lista.w * 0.5, lista.y + lista.h * 0.5);
    assert_eq!(hero.store.panel_at(cx, cy), Some(ids::GAL_PANEL));
    let arena = bumpalo::Bump::new();
    let antes = hero.store.panel_scroll(ids::GAL_PANEL);
    let _ = ph2d_editor_core::interaction::dispatch_wheel(
        &mut hero.store,
        ph2d_host::WheelEvent {
            x: cx,
            y: cy,
            delta_x: 0.0,
            delta_y: -40.0,
            modifiers: ph2d_host::Modifiers::default(),
            timestamp_ns: 0,
        },
        &arena,
    );
    assert!(
        hero.store.panel_scroll(ids::GAL_PANEL) > antes,
        "a roda sobre a lista de cartões não a rolou"
    );
}

/// (2) Clicar «Mid» acende-o (e apaga o Low de fábrica).
#[test]
fn clicar_mid_acende_o_segmento() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.level, 0, "fixtura: o de fábrica é o Low");
    gal.clica(g::LEVEL_MID);
    assert_eq!(gal.st.choices.level, 1, "o clique em Mid não o acendeu");
    assert!(
        gal.rect(g::LEVEL_MID).is_some(),
        "a pintura seguinte perdeu o Mid"
    );
}

/// (3) Arrastar o ícone de um campo muda o número no store.
#[test]
fn arrastar_o_icone_do_campo_muda_o_numero() {
    let mut gal = Galeria::aberta();
    let antes = gal.valor(g::POS_X);
    gal.arrasta_o_icone(g::POS_X, 60.0);
    assert!(
        gal.valor(g::POS_X) > antes,
        "arrastar o ícone do X não o mudou ({antes} → {})",
        gal.valor(g::POS_X)
    );
}

/// (4) A caixa de marcar alterna.
#[test]
fn a_caixa_de_marcar_alterna() {
    let mut gal = Galeria::aberta();
    assert!(!gal.st.choices.snap);
    gal.clica(g::SNAP);
    assert!(
        gal.st.choices.snap,
        "o 1.º clique não marcou «Snap to grid»"
    );
    gal.clica(g::SNAP);
    assert!(!gal.st.choices.snap, "o 2.º clique não desmarcou");
}

/// (5) O «+» do cartão ausente cria a secção, e ela passa a pintar um corpo.
#[test]
fn o_mais_do_cartao_ausente_o_cria() {
    let mut gal = Galeria::aberta();
    assert!(
        gal.rect(g::MASS).is_none(),
        "fixtura: o Physics nasce ausente"
    );
    gal.clica(g::PHYSICS_ADD);
    gal.repinta();
    assert!(gal.st.choices.physics, "o «+» não criou a secção");
    assert!(
        gal.rect(g::MASS).is_some(),
        "a secção criada não pintou o corpo (Mass)"
    );
}

/// (6) O ↺ repõe os valores e as escolhas de fábrica.
#[test]
fn o_reset_repoe_a_fabrica() {
    let mut gal = Galeria::aberta();
    gal.arrasta_o_icone(g::POS_X, 60.0);
    gal.clica(g::LEVEL_HIGH);
    gal.clica(g::SNAP);
    assert_ne!(gal.valor(g::POS_X), fabrica(g::POS_X), "fixtura: o X mudou");
    gal.clica(g::RESET);
    assert_eq!(gal.valor(g::POS_X), fabrica(g::POS_X), "o ↺ não repôs o X");
    assert_eq!(
        gal.st.choices,
        Choices::default(),
        "o ↺ não repôs as escolhas"
    );
}

/// (7) Todo controlo pintado está registado (sem estado, o clique morre no despacho), e todo ícone
/// sem texto tem dica.
#[test]
fn todo_controlo_pintado_esta_registado_e_todo_icone_tem_dica() {
    let mut gal = Galeria::aberta();
    gal.clica(g::PHYSICS_ADD);
    let nossos: Vec<NodeId> = g::BUTTONS
        .iter()
        .chain(g::PHYSICS_BUTTONS.iter())
        .copied()
        .chain(g::FIELDS.iter().map(|f| f.0))
        .chain([g::NAME])
        .collect();
    let pintados: Vec<NodeId> = nossos
        .iter()
        .copied()
        .filter(|id| gal.rect(*id).is_some())
        .collect();
    assert!(
        pintados.len() >= 20,
        "controlo de vacuidade: só {} controlos da galeria pintados",
        pintados.len()
    );
    let mortos: Vec<NodeId> = pintados
        .iter()
        .copied()
        .filter(|id| !gal.host.store().contains(*id))
        .collect();
    assert!(mortos.is_empty(), "pintados e sem registo: {mortos:?}");
    let icones = dicas::icon_only_ids(ids::GAL_PANEL);
    assert!(!icones.is_empty(), "controlo: a lista tem ícones sem texto");
    let sem_dica: Vec<NodeId> = icones
        .into_iter()
        .filter(|id| gal.host.store().tooltip_for(*id).is_none())
        .collect();
    assert!(sem_dica.is_empty(), "ícones sem dica: {sem_dica:?}");
}

/// (8) O «−» do Fill tira-o (a linha some, o cartão fica vazio com o «+») e o «+» devolve-o. *Mutação:
/// o «−» só consumido ⇒ a amostra continua pintada.*
#[test]
fn o_menos_tira_o_fill_e_o_mais_o_devolve() {
    let mut gal = Galeria::aberta();
    assert!(gal.rect(g::FILL_SWATCH).is_some(), "fixtura: o Fill nasce");
    assert!(gal.rect(g::FILL_ADD).is_none(), "fixtura: sem «+» com Fill");
    gal.clica(g::FILL_REMOVE);
    assert!(!gal.st.choices.fill, "o «−» não tirou o Fill");
    assert!(
        gal.rect(g::FILL_SWATCH).is_none() && gal.rect(g::OPACITY).is_none(),
        "o Fill tirado continua pintado"
    );
    assert!(
        gal.rect(g::FILL_ADD).is_some(),
        "o cartão vazio não mostra o «+»"
    );
    gal.clica(g::FILL_ADD);
    assert!(gal.st.choices.fill, "o «+» não devolveu o Fill");
    assert!(
        gal.rect(g::FILL_SWATCH).is_some(),
        "o Fill devolvido não pinta"
    );
}

/// (9) A amostra abre o seletor de cor do app apontado para ela, semeado com a cor que mostra; a
/// cor que o seletor escreve (o espelho do quadro do hero) passa a ser a do Fill. *Mutação: a
/// amostra só consumida ⇒ o seletor não abre; a leitura do quadro desligada ⇒ a cor não chega.*
#[test]
fn a_amostra_abre_o_seletor_e_a_cor_escolhida_fica() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.host.store().picker_target(), None, "fixtura: fechado");
    gal.clica(g::FILL_SWATCH);
    assert_eq!(
        gal.host.store().picker_target(),
        Some(g::FILL_SWATCH),
        "o clique na amostra não abriu o seletor para ela"
    );
    let semente = gal
        .host
        .store()
        .widget_color(g::FILL_SWATCH)
        .expect("a amostra semeou a cor");
    let escolhida = [semente[0] ^ 0x40, semente[1], semente[2], 255];
    // O que o hero faz a cada quadro: o valor do seletor no `widget_color` do alvo.
    gal.host
        .store_mut()
        .set_widget_color(g::FILL_SWATCH, escolhida);
    gal.repinta();
    assert_eq!(
        gal.st.choices.fill_rgba,
        Some(escolhida),
        "a cor escolhida no seletor não chegou ao Fill"
    );
    gal.clica(g::FILL_REMOVE);
    assert_eq!(
        gal.host.store().picker_target(),
        None,
        "tirar o Fill deixou o seletor aberto para uma amostra que sumiu"
    );
}

/// (9b) O seletor parte da cor que a amostra MOSTRA — a tinta do cartão Color, com o tema próprio
/// dele, não o `Accent` do app. *Mutação: a tinta da lista em `toggle_picker` ⇒ a semente é a do app.*
#[test]
fn o_seletor_parte_da_cor_da_tinta_do_cartao() {
    let mut gal = Galeria::aberta();
    let lista = gal.host.theme();
    let proprio = if lista == Theme::PlumberRed {
        Theme::default()
    } else {
        Theme::PlumberRed
    };
    gal.host
        .store_mut()
        .set_section_theme(g::CARD_COLOR, Some(proprio));
    gal.repinta();
    gal.clica(g::FILL_SWATCH);
    let a_tinta = CardInk::of_card(Some(proprio), lista)
        .color(ColorToken::Accent)
        .to_rgba8()
        .to_u8_array();
    assert_ne!(
        a_tinta,
        CardInk::of_list(lista)
            .color(ColorToken::Accent)
            .to_rgba8()
            .to_u8_array(),
        "fixtura: os dois temas têm de dar acentos diferentes"
    );
    assert_eq!(
        gal.host.store().widget_color(g::FILL_SWATCH),
        Some(a_tinta),
        "o seletor não partiu da cor que a amostra do cartão mostra"
    );
    gal.repinta();
    assert_eq!(
        gal.st.choices.fill_rgba, None,
        "só abrir o seletor não escolhe cor nenhuma"
    );
}

/// (10) Apply grava os valores de PARTIDA — números, escolhas e o texto do Name; Cancel e o ↺
/// voltam a eles (não à fábrica). *Mutação: o Apply só consumido ⇒ o Cancel volta à fábrica; o
/// Cancel só consumido ⇒ nada volta; o Name fora do retrato ⇒ o texto escrito fica.*
#[test]
fn apply_grava_os_valores_e_cancel_volta_a_eles() {
    let mut gal = Galeria::aberta();
    gal.arrasta_o_icone(g::POS_X, 60.0);
    gal.clica(g::LEVEL_HIGH);
    gal.host.set_text(g::NAME, "Hero");
    let aplicado = gal.valor(g::POS_X);
    assert_ne!(aplicado, fabrica(g::POS_X), "fixtura: o X mudou");
    gal.clica(g::PRIMARY);
    assert_eq!(gal.st.applied.choices.level, 2, "o Apply não gravou o High");
    gal.arrasta_o_icone(g::POS_X, 60.0);
    gal.clica(g::SNAP);
    gal.clica(g::FILL_REMOVE);
    gal.host.set_text(g::NAME, "Villain");
    gal.repinta();
    assert_ne!(gal.valor(g::POS_X), aplicado, "fixtura: o X mudou de novo");
    gal.clica(g::SECONDARY);
    assert_eq!(
        gal.host.store().text(g::NAME),
        Some("Hero"),
        "o Cancel não voltou o Name ao aplicado"
    );
    assert_eq!(
        gal.valor(g::POS_X),
        aplicado,
        "o Cancel não voltou o X ao aplicado"
    );
    assert!(
        gal.st.choices.level == 2 && !gal.st.choices.snap && gal.st.choices.fill,
        "o Cancel não voltou as escolhas às aplicadas: {:?}",
        gal.st.choices
    );
    assert!(
        gal.rect(g::FILL_SWATCH).is_some(),
        "o Fill voltou e não pinta"
    );
    gal.clica(g::SNAP);
    gal.host.set_text(g::NAME, "Villain");
    gal.repinta();
    gal.clica(g::RESET);
    assert!(
        gal.st.choices.level == 2 && !gal.st.choices.snap,
        "o ↺ não voltou aos valores aplicados"
    );
    assert_eq!(
        gal.host.store().text(g::NAME),
        Some("Hero"),
        "o ↺ não voltou o Name ao aplicado"
    );
}
