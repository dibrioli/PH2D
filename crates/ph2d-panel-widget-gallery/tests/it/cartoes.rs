//! ⭐⭐ **A galeria em CARTÕES no encaixe da direita** (doc `CartoesUI/08` §1) — os gestos reais
//! (pointer → despacho → `apply_event` → pintura seguinte) sobre o vocabulário do cartão.

use ph2d_a11y::NodeId;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelHost, PanelHostInternal, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, slot_tabs};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_widget_gallery::controlos::{Kind, controls, lit};
use ph2d_panel_widget_gallery::ids as g;
use ph2d_panel_widget_gallery::state::{BOOLEAN_NONE, Choices, Flag, Pick, WidgetGalleryState};
use ph2d_panel_widget_gallery::{WidgetGalleryPanel, force_cards_on_this_thread};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

const SEC: u128 = 1_000_000_000;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Alta o bastante para os cartões das ferramentas caberem sem rolar.
fn alta() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H * 10.0)
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

/// (2) Clicar «Subtract» acende-o (e apaga o Union de fábrica); o Flatten deixa nenhum aceso.
#[test]
fn clicar_um_segmento_acende_o_e_o_flatten_apaga() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::Boolean), 0, "fixtura: Union");
    gal.clica(g::BOOL_SUBTRACT);
    assert_eq!(
        gal.st.choices.pick(Pick::Boolean),
        1,
        "o clique em Subtract não o acendeu"
    );
    assert!(
        gal.rect(g::BOOL_SUBTRACT).is_some(),
        "a pintura seguinte perdeu o segmento"
    );
    gal.clica(g::FLATTEN);
    assert_eq!(
        gal.st.choices.pick(Pick::Boolean),
        BOOLEAN_NONE,
        "o Flatten não largou o modo"
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
    assert!(!gal.st.choices.on(Flag::Clip));
    gal.clica(g::CLIP);
    assert!(
        gal.st.choices.on(Flag::Clip),
        "o 1.º clique não marcou «Clip content»"
    );
    gal.clica(g::CLIP);
    assert!(!gal.st.choices.on(Flag::Clip), "o 2.º clique não desmarcou");
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
    assert!(gal.st.choices.on(Flag::Physics), "o «+» não criou a secção");
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
    gal.clica(g::BOOL_EXCLUDE);
    gal.clica(g::CLIP);
    assert_ne!(gal.valor(g::POS_X), fabrica(g::POS_X), "fixtura: o X mudou");
    gal.clica(g::RESET);
    assert_eq!(gal.valor(g::POS_X), fabrica(g::POS_X), "o ↺ não repôs o X");
    assert_eq!(
        gal.st.choices,
        Choices::default(),
        "o ↺ não repôs as escolhas"
    );
}

/// (7) ⭐ TODA a tabela: cada controlo pintado está registado (sem estado, o clique morre no
/// despacho), e todo ícone sem texto tem dica. Com tudo aberto para o controlo de vacuidade ver
/// mais linhas.
#[test]
fn todo_controlo_pintado_esta_registado_e_todo_icone_tem_dica() {
    let mut gal = Galeria::aberta();
    gal.clica(g::PHYSICS_ADD);
    gal.clica(g::CORNER_EACH);
    gal.clica(g::FX_ADD);
    let nossos: Vec<NodeId> = controls()
        .map(|c| c.id)
        .chain(g::FIELDS.iter().map(|f| f.0))
        .chain(g::TEXTS.iter().map(|t| t.0))
        .collect();
    let pintados: Vec<NodeId> = nossos
        .iter()
        .copied()
        .filter(|id| gal.rect(*id).is_some())
        .collect();
    assert!(
        pintados.len() >= 100,
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
    assert!(
        icones.len() >= 40,
        "controlo: a lista tem só {} ícones sem texto",
        icones.len()
    );
    let sem_dica: Vec<NodeId> = icones
        .into_iter()
        .filter(|id| gal.host.store().tooltip_for(*id).is_none())
        .collect();
    assert!(sem_dica.is_empty(), "ícones sem dica: {sem_dica:?}");
    for c in controls() {
        if c.icon.is_some() {
            assert!(c.tip.is_some(), "{:?} é ícone e não tem dica", c.id);
        }
    }
}

/// (7b) ⭐ **Zero no-op silencioso**: cada controlo da tabela, clicado pelo evento, muda alguma
/// coisa ou é a caixa de uma lista (o despacho dos cartões abre-a). *Mutação: um ramo do `match`
/// só consumido ⇒ o id aparece aqui.*
#[test]
fn nenhum_controlo_da_tabela_e_um_no_op() {
    let mut gal = Galeria::aberta();
    let mut mudos = Vec::new();
    for c in controls() {
        if matches!(c.kind, Kind::Box) {
            continue;
        }
        // Parte de um retrato em que o clique tem o que mudar.
        let mut st = WidgetGalleryState::default();
        st.choices.choose(Pick::Boolean, 3);
        match c.kind {
            Kind::Pick(p, i) => st.choices.choose(p, i.wrapping_add(1)),
            Kind::Set(f, v) => st.choices.set(f, !v),
            Kind::Remove(f, _) => st.choices.set(f, true),
            _ => {}
        }
        gal.host.store_mut().set_number_value(g::POS_X, 123.0);
        gal.host.store_mut().set_number_value(g::POS_Y, 123.0);
        gal.host.store_mut().set_number_value(g::ROTATION, 7.0);
        gal.host.store_mut().set_number_value(g::SCALE_X, 2.0);
        gal.host.store_mut().set_number_value(g::SCALE_Y, 2.0);
        gal.host.store_mut().set_picker_target(None);
        let numeros =
            |h: &MockPanelHost| g::FIELDS.map(|f| h.store().number_value(f.0).unwrap_or_default());
        let antes = numeros(&gal.host);
        let (ch, applied, exp) = (st.choices, st.applied.clone(), st.exported);
        let _ = gal
            .host
            .apply_panel_event::<WidgetGalleryPanel>(&mut st, WidgetEvent::Click(c.id));
        let mudou = st.choices != ch
            || st.applied != applied
            || st.exported != exp
            || numeros(&gal.host) != antes
            || gal.host.store().picker_target().is_some();
        if !mudou {
            mudos.push(c.id);
        }
    }
    assert!(mudos.is_empty(), "cliques sem efeito nenhum: {mudos:?}");
}

/// (8) O «−» do Fill tira a linha (o cartão fica vazio com o «+») e o «+» devolve-a. *Mutação: o
/// «−» só consumido ⇒ a amostra continua pintada.*
#[test]
fn o_menos_tira_o_fill_e_o_mais_o_devolve() {
    let mut gal = Galeria::aberta();
    let linha = g::FILLS[0];
    assert!(gal.rect(linha.swatch).is_some(), "fixtura: o Fill nasce");
    gal.clica(linha.remove);
    assert!(!gal.st.choices.on(Flag::Fill0), "o «−» não tirou o Fill");
    assert!(
        gal.rect(linha.swatch).is_none() && gal.rect(linha.opacity).is_none(),
        "o Fill tirado continua pintado"
    );
    assert!(
        gal.rect(g::FILL_ADD).is_some(),
        "o cartão vazio não mostra o «+»"
    );
    gal.clica(g::FILL_ADD);
    assert!(gal.st.choices.on(Flag::Fill0), "o «+» não devolveu o Fill");
    assert!(
        gal.rect(linha.swatch).is_some(),
        "o Fill devolvido não pinta"
    );
}

/// (9) A amostra abre o seletor de cor do app apontado para ela, semeado com a cor que mostra; a
/// cor que o seletor escreve (o espelho do quadro do hero) passa a ser a dela. *Mutação: a amostra
/// só consumida ⇒ o seletor não abre; a leitura do quadro desligada ⇒ a cor não chega.*
#[test]
fn a_amostra_abre_o_seletor_e_a_cor_escolhida_fica() {
    let mut gal = Galeria::aberta();
    let linha = g::FILLS[0];
    assert_eq!(gal.host.store().picker_target(), None, "fixtura: fechado");
    gal.clica(linha.swatch);
    assert_eq!(
        gal.host.store().picker_target(),
        Some(linha.swatch),
        "o clique na amostra não abriu o seletor para ela"
    );
    let semente = gal
        .host
        .store()
        .widget_color(linha.swatch)
        .expect("a amostra semeou a cor");
    let escolhida = [semente[0] ^ 0x40, semente[1], semente[2], 255];
    // O que o hero faz a cada quadro: o valor do seletor no `widget_color` do alvo.
    gal.host
        .store_mut()
        .set_widget_color(linha.swatch, escolhida);
    gal.repinta();
    assert_eq!(
        gal.st.choices.colors[0],
        Some(escolhida),
        "a cor escolhida no seletor não chegou ao Fill"
    );
    gal.clica(linha.remove);
    assert_eq!(
        gal.host.store().picker_target(),
        None,
        "tirar o Fill deixou o seletor aberto para uma amostra que sumiu"
    );
}

/// (9b) O seletor parte da cor que a amostra MOSTRA — a tinta do cartão Fill, com o tema próprio
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
        .set_section_theme(g::CARD_FILL, Some(proprio));
    gal.repinta();
    let amostra = g::FILLS[0].swatch;
    gal.clica(amostra);
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
        gal.host.store().widget_color(amostra),
        Some(a_tinta),
        "o seletor não partiu da cor que a amostra do cartão mostra"
    );
    gal.repinta();
    assert_eq!(
        gal.st.choices.colors[0], None,
        "só abrir o seletor não escolhe cor nenhuma"
    );
}

/// (10) Apply grava os valores de PARTIDA — números, escolhas e os textos; Cancel e o ↺ voltam a
/// eles (não à fábrica). *Mutação: o Apply só consumido ⇒ o Cancel volta à fábrica; o Cancel só
/// consumido ⇒ nada volta; o Name fora do retrato ⇒ o texto escrito fica.*
#[test]
fn apply_grava_os_valores_e_cancel_volta_a_eles() {
    let mut gal = Galeria::aberta();
    gal.arrasta_o_icone(g::POS_X, 60.0);
    gal.clica(g::BOOL_EXCLUDE);
    gal.host.set_text(g::NAME, "Hero");
    let aplicado = gal.valor(g::POS_X);
    assert_ne!(aplicado, fabrica(g::POS_X), "fixtura: o X mudou");
    gal.clica(g::PRIMARY);
    assert_eq!(
        gal.st.applied.choices.pick(Pick::Boolean),
        3,
        "o Apply não gravou o Exclude"
    );
    gal.arrasta_o_icone(g::POS_X, 60.0);
    gal.clica(g::CLIP);
    gal.clica(g::FILLS[0].remove);
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
    let ch = gal.st.choices;
    assert!(
        ch.pick(Pick::Boolean) == 3 && !ch.on(Flag::Clip) && ch.on(Flag::Fill0),
        "o Cancel não voltou as escolhas às aplicadas: {ch:?}"
    );
    assert!(
        gal.rect(g::FILLS[0].swatch).is_some(),
        "o Fill voltou e não pinta"
    );
    gal.clica(g::CLIP);
    gal.host.set_text(g::NAME, "Villain");
    gal.repinta();
    gal.clica(g::RESET);
    let ch = gal.st.choices;
    assert!(
        ch.pick(Pick::Boolean) == 3 && !ch.on(Flag::Clip),
        "o ↺ não voltou aos valores aplicados"
    );
    assert_eq!(
        gal.host.store().text(g::NAME),
        Some("Hero"),
        "o ↺ não voltou o Name ao aplicado"
    );
}

/// (11) Espelhar na horizontal nega o Scale X e o ícone acende — o aceso é o sinal do número, por
/// isso o Scale escrito à mão de volta apaga-o. *Mutação: `Act::FlipH` só consumido.*
#[test]
fn o_flip_h_nega_o_scale_x_e_acende() {
    let mut gal = Galeria::aberta();
    let ligado = |g_: &Galeria| {
        let c = controls().find(|c| c.id == g::FLIP_H).unwrap();
        lit(c, &g_.st.choices, g_.host.store())
    };
    assert!(!ligado(&gal), "fixtura: nasce apagado");
    gal.clica(g::FLIP_H);
    assert_eq!(gal.valor(g::SCALE_X), -1.0, "o Flip H não negou o Scale X");
    assert_eq!(gal.valor(g::SCALE_Y), 1.0, "o Flip H mexeu no Scale Y");
    assert!(ligado(&gal), "o Flip H não acendeu");
    gal.clica(g::FLIP_H);
    assert_eq!(gal.valor(g::SCALE_X), 1.0, "o 2.º Flip H não voltou");
    assert!(!ligado(&gal), "o 2.º Flip H não apagou");
}

/// (12) Girar 90° soma 90 à Rotation, e o quarto giro volta a 0. *Mutação: `Act::Rotate90` só
/// consumido.*
#[test]
fn girar_90_soma_90_a_rotation() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.valor(g::ROTATION), 0.0, "fixtura: de fábrica");
    gal.clica(g::ROTATE_90);
    assert_eq!(gal.valor(g::ROTATION), 90.0, "não somou 90");
    for _ in 0..3 {
        gal.clica(g::ROTATE_90);
    }
    assert_eq!(
        gal.valor(g::ROTATION),
        0.0,
        "a volta inteira não fecha em 0"
    );
}

/// (13) As ferramentas do Align põem a Position na borda da moldura fictícia. *Mutação:
/// `Act::AlignLeft` só consumido.*
#[test]
fn o_align_left_poe_o_x_na_borda() {
    let mut gal = Galeria::aberta();
    assert_ne!(gal.valor(g::POS_X), 0.0, "fixtura: o X não está na borda");
    let y = gal.valor(g::POS_Y);
    gal.clica(g::ALIGN_LEFT);
    assert_eq!(
        gal.valor(g::POS_X),
        0.0,
        "o Align left não pôs o X na borda"
    );
    assert_eq!(gal.valor(g::POS_Y), y, "o Align left mexeu no Y");
    gal.clica(g::ALIGN_RIGHT);
    let direita = gal.valor(g::POS_X);
    gal.clica(g::ALIGN_HCENTER);
    assert_eq!(
        gal.valor(g::POS_X),
        direita * 0.5,
        "o centro não é o meio entre as duas bordas"
    );
}

/// (14) O «+» do Effects acrescenta uma linha e o «−» dela tira-a; com a lista cheia, o «+» some
/// (nunca um clique mudo). *Mutação: `Kind::Add`/`Kind::Remove` só consumidos.*
#[test]
fn o_mais_do_effects_acrescenta_uma_linha_e_o_menos_a_tira() {
    let mut gal = Galeria::aberta();
    let (a, b, c) = (g::EFFECTS[0], g::EFFECTS[1], g::EFFECTS[2]);
    assert!(gal.rect(a.kind).is_some(), "fixtura: uma linha de fábrica");
    assert!(gal.rect(b.kind).is_none(), "fixtura: só uma");
    gal.clica(g::FX_ADD);
    assert!(gal.rect(b.kind).is_some(), "o «+» não acrescentou a linha");
    gal.clica(g::FX_ADD);
    assert!(gal.rect(c.kind).is_some(), "o 2.º «+» não acrescentou");
    assert!(gal.rect(g::FX_ADD).is_none(), "a lista cheia ainda tem «+»");
    gal.clica(b.remove);
    assert!(gal.rect(b.kind).is_none(), "o «−» não tirou a linha dele");
    assert!(
        gal.rect(a.kind).is_some() && gal.rect(c.kind).is_some(),
        "o «−» tirou uma linha alheia"
    );
    assert!(gal.rect(g::FX_ADD).is_some(), "a lista com folga sem «+»");
}

/// (15) Com a corrente fechada, o Scale Y segue o X (nos dois sentidos, na proporção de antes); o
/// Flip não leva o outro eixo; aberta, cada um é livre. *Mutação: `sync_link` desligado.*
#[test]
fn a_corrente_faz_o_y_seguir_o_x() {
    let mut gal = Galeria::aberta();
    gal.arrasta_o_icone(g::SCALE_X, 50.0);
    let solto = gal.valor(g::SCALE_X);
    assert_ne!(solto, 1.0, "fixtura: o X mudou");
    assert_eq!(gal.valor(g::SCALE_Y), 1.0, "sem corrente o Y é livre");
    gal.clica(g::SCALE_LINK);
    let (x0, y0) = (gal.valor(g::SCALE_X), gal.valor(g::SCALE_Y));
    gal.arrasta_o_icone(g::SCALE_X, 50.0);
    let x1 = gal.valor(g::SCALE_X);
    assert_ne!(x1, x0, "fixtura: o X mudou de novo");
    assert!(
        (gal.valor(g::SCALE_Y) - y0 * x1 / x0).abs() < 1e-9,
        "o Y não seguiu o X: {x0} → {x1}, Y = {}",
        gal.valor(g::SCALE_Y)
    );
    let y1 = gal.valor(g::SCALE_Y);
    gal.clica(g::FLIP_H);
    assert_eq!(
        gal.valor(g::SCALE_Y),
        y1,
        "o Flip H levou o Y pela corrente"
    );
}

/// (16) O ↺ do Transform repõe o transform, e só ele. *Mutação: `Act::TransformReset` só consumido.*
#[test]
fn o_reset_do_transform_repoe_so_o_transform() {
    let mut gal = Galeria::aberta();
    gal.clica(g::ROTATE_90);
    gal.clica(g::FLIP_V);
    gal.arrasta_o_icone(g::OPACITY, -30.0);
    let opacidade = gal.valor(g::OPACITY);
    gal.clica(g::TRANSFORM_RESET);
    assert_eq!(
        gal.valor(g::ROTATION),
        fabrica(g::ROTATION),
        "a Rotation ficou"
    );
    assert_eq!(
        gal.valor(g::SCALE_Y),
        fabrica(g::SCALE_Y),
        "o Scale Y ficou"
    );
    assert_eq!(gal.valor(g::OPACITY), opacidade, "o ↺ mexeu na Opacity");
}

/// (17) O Export acende o texto de estado; mudar o formato apaga-o. *Mutação: `Act::Export` só
/// consumido.*
#[test]
fn o_export_acende_o_estado() {
    let mut gal = Galeria::aberta();
    assert!(!gal.st.exported, "fixtura: ainda não exportou");
    gal.clica(g::EXPORT);
    assert!(gal.st.exported, "o Export não acendeu o estado");
    let _ = gal
        .host
        .apply_panel_event::<WidgetGalleryPanel>(&mut gal.st, WidgetEvent::Click(g::FORMAT_SVG));
    assert!(!gal.st.exported, "mudar o formato não apagou o estado");
}

/// (18) A lista suspensa: o clique na opção escolhe-a e a caixa passa a mostrá-la; os títulos de
/// ferramenta cobrem os corpos de sempre. *Mutação: `Kind::Pick` só consumido.*
#[test]
fn a_opcao_da_lista_escolhe() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::Blend), 0, "fixtura: Normal");
    let _ = gal
        .host
        .apply_panel_event::<WidgetGalleryPanel>(&mut gal.st, WidgetEvent::Click(g::BLEND_SCREEN));
    assert_eq!(gal.st.choices.pick(Pick::Blend), 2, "a opção não escolheu");
    assert_eq!(
        gal.st.applied.choices.pick(Pick::Blend),
        0,
        "o clique aplicou"
    );
}
