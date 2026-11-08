//! Gates da dica ilustrada — pela PORTA da paleta inteira (`paint`), com o rato sobre o item.

use super::super::{
    HitIndex, PaletteGroup, PaletteItem, PaletteModel, PaletteSub, PaletteTip, WidgetStore,
    paint as paint_palette,
};
use super::*;
use crate::widget::illustration::Mark;
use ph2d_tool_registry::hash_node_id;

const VP: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1187.0,
    h: 953.0,
};

const PICTURE: &[Mark] = &[
    Mark::Rect {
        at: [0.1, 0.7],
        size: [0.8, 0.2],
        fill: ColorToken::Text3,
        round: 0.05,
    },
    Mark::Disc {
        c: [0.5, 0.4],
        r: 0.15,
        fill: ColorToken::Accent,
    },
];

fn with_tip() -> NodeId {
    hash_node_id("tip.with")
}
fn without_tip() -> NodeId {
    hash_node_id("tip.without")
}

fn model() -> PaletteModel {
    let item = |label: &str, id| PaletteItem {
        badge: None,
        label: label.into(),
        id,
        keywords: "",
    };
    PaletteModel {
        title: "Add Object".into(),
        groups: vec![PaletteGroup {
            title: "Game".into(),
            color: ColorToken::NodeCatOutput,
            subs: vec![PaletteSub {
                title: None,
                items: vec![item("Character", with_tip()), item("Camera", without_tip())],
            }],
        }],
        toggle: None,
        tips: vec![PaletteTip {
            id: with_tip(),
            title: "Character".into(),
            body: "A figure the player controls: it walks, jumps and falls with gravity, and \
                   stands on Static Bodies. Its picture changes with what it is doing."
                .into(),
            picture: PICTURE,
        }],
        select: None,
    }
}

/// Pinta a paleta com o rato sobre `hot`: devolve os caminhos da cena e os alvos registados.
fn painted(hot: Option<NodeId>) -> (u32, Vec<(NodeId, Rect)>) {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hits = HitIndex::new();
    let motion = crate::motion::UiMotion::default();
    let mut store = WidgetStore::with_capacity(4);
    store.open_command_palette(model());
    store.set_hot(hot);
    let _ = paint_palette(
        &mut scene,
        &mut ts,
        Theme::default(),
        &mut hits,
        &model(),
        "",
        VP,
        &motion,
        &store,
    );
    (
        scene.inner().encoding().n_paths,
        hits.iter_registrations().collect(),
    )
}

/// ⭐⭐ **Rato sobre o item com dica ⇒ o cartão pinta; sobre o item sem dica ⇒ nada; e a dica NÃO
/// regista alvo** (o clique continua a ser do item).
///
/// (Mutações: tirar a chamada a `tip::paint_tip` do `paint` ⇒ RED na 1.ª asserção; registar o cartão no
/// `HitIndex` ⇒ RED na 3.ª.)
#[test]
fn hovering_an_item_with_a_tip_paints_the_card_and_registers_nothing() {
    let (rest, rest_hits) = painted(None);
    let (on_tip, on_tip_hits) = painted(Some(with_tip()));
    let (on_plain, on_plain_hits) = painted(Some(without_tip()));
    assert!(
        on_tip > rest,
        "o rato sobre o item com dica não pintou nada ({on_tip} contra {rest} caminhos)"
    );
    assert_eq!(on_plain, rest, "um item sem dica não pode pintar cartão");
    assert_eq!(on_tip_hits, rest_hits, "a dica registou um alvo");
    assert_eq!(on_plain_hits, rest_hits);
}

/// ⭐ **O cartão cai DENTRO do ecrã, ao lado do item (sem o cobrir), e tem lugar para o título, o desenho
/// e o texto** — medido pelo próprio pintor.
#[test]
fn the_card_sits_beside_the_item_inside_the_viewport() {
    let (_, hits) = painted(None);
    let pill = hits
        .iter()
        .find(|(id, _)| *id == with_tip())
        .map(|(_, r)| *r)
        .expect("o item regista-se");
    let mut index = HitIndex::new();
    index.register(with_tip(), pill);
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let card = paint_tip(
        &mut scene,
        &mut ts,
        Theme::default(),
        |id| index.rect_for(id),
        &model(),
        Some(with_tip()),
        VP,
    )
    .expect("o item tem dica");
    assert!(card.x >= VP.x && card.y >= VP.y, "{card:?}");
    assert!(card.x + card.w <= VP.x + VP.w && card.y + card.h <= VP.y + VP.h);
    let overlaps = card.x < pill.x + pill.w
        && pill.x < card.x + card.w
        && card.y < pill.y + pill.h
        && pill.y < card.y + card.h;
    assert!(!overlaps, "o cartão {card:?} cobre o item {pill:?}");
    // O desenho (meia largura útil) mais o título e pelo menos duas linhas de texto.
    let picture_h = (TIP_W - Spacing::Lg.px() * 2.0) * PICTURE_ASPECT;
    assert!(
        card.h > picture_h + TypeToken::Sm.px() + TypeToken::Xs.px() * 2.0,
        "o cartão ({}) não tem altura para o título, o desenho e a explicação",
        card.h
    );
    assert!(
        paint_tip(
            &mut scene,
            &mut ts,
            Theme::default(),
            |id| index.rect_for(id),
            &model(),
            Some(without_tip()),
            VP
        )
        .is_none()
    );
}

/// **Perto da borda direita o cartão vira para a esquerda; perto do fundo sobe.**
#[test]
fn the_card_flips_left_and_rises_at_the_edges() {
    let near_right = Rect::new(VP.w - 120.0, VP.h - 30.0, 100.0, 26.0);
    let c = place(near_right, TIP_W, 400.0, 8.0, VP);
    assert!(
        c.x + c.w <= near_right.x,
        "não virou para a esquerda: {c:?}"
    );
    assert!(c.y + c.h <= VP.h, "saiu pelo fundo: {c:?}");
    let left = Rect::new(40.0, 100.0, 100.0, 26.0);
    let c = place(left, TIP_W, 400.0, 8.0, VP);
    assert!(c.x >= left.x + left.w, "havia lugar à direita: {c:?}");
    assert_eq!(c.y, left.y, "alinha ao topo do item quando cabe");
}

/// Em cartões a moldura do desenho e o texto usam os tons da lista; `PH2D_CARDS=0`, os de antes.
#[test]
fn a_dica_veste_os_tons_da_lista() {
    use crate::widget::card_ink::CardInk;
    use crate::widget::lista_flutuante::force_cromo_on_this_thread;
    let tema = Theme::default();
    let tip = PaletteTip {
        id: with_tip(),
        title: "Character".into(),
        body: "The hero".into(),
        picture: PICTURE,
    };
    let conta = |cartao: bool, cor: ph2d_vector::Color| {
        force_cromo_on_this_thread(Some(cartao));
        let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
        let item = Rect::new(100.0, 100.0, 80.0, 20.0);
        let _ = paint_card(&mut scene, &mut ts, tema, &tip, item, VP);
        force_cromo_on_this_thread(None);
        let mut uma = VectorScene::new();
        fill_rounded_rect(&mut uma, Rect::new(0.0, 0.0, 1.0, 1.0), 0.0, cor);
        let dados = uma.inner().encoding().draw_data.clone();
        let cena = scene.inner().encoding().draw_data.clone();
        cena.windows(dados.len())
            .filter(|w| *w == dados.as_slice())
            .count()
    };
    let moldura = CardInk::of_list(tema).color(ColorToken::CardField);
    let antes = resolve(ColorToken::Bg3, tema);
    assert_eq!(
        conta(true, moldura),
        1,
        "a moldura do desenho na caixa do cartão"
    );
    assert_eq!(conta(false, antes), 1, "PH2D_CARDS=0: a de antes");
}
