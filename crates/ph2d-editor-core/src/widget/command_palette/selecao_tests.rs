//! Gates do «destaca, depois escolhe» (plano 24 §2.1–2.2) — pela PORTA da paleta inteira e do store.

use super::super::{
    HitIndex, PaletteGroup, PaletteItem, PaletteModel, PaletteSub, WidgetStore,
    paint as paint_palette,
};
use super::*;
use crate::widget::illustration::Mark;

const VP: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1187.0,
    h: 953.0,
};
const PICTURE: &[Mark] = &[Mark::Disc {
    c: [0.5, 0.5],
    r: 0.2,
    fill: ColorToken::Accent,
}];

fn id(n: &str) -> NodeId {
    hash_node_id_rt(n)
}
fn hash_node_id_rt(n: &str) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(n)
}

fn model(select: bool) -> PaletteModel {
    let item = |n: &str| PaletteItem {
        label: n.into(),
        id: id(n),
        keywords: if n == "Wall" { "floor, wall" } else { "" },
    };
    let tip = |n: &str, body: &str| PaletteTip {
        id: id(n),
        title: n.into(),
        body: body.into(),
        picture: PICTURE,
    };
    PaletteModel {
        title: "Add Object".into(),
        groups: vec![PaletteGroup {
            title: "Game".into(),
            color: ColorToken::NodeCatOutput,
            subs: vec![PaletteSub {
                title: None,
                items: vec![item("Sprite"), item("Wall"), item("Hero")],
            }],
        }],
        toggle: None,
        tips: vec![
            tip("Sprite", "Short."),
            tip(
                "Wall",
                &"A much longer explanation that wraps over several lines. ".repeat(30),
            ),
        ],
        select: select.then(|| PaletteSelect {
            initial: id("Wall"),
            teach: vec![PaletteTeach {
                words: "coin, coins",
                text: "A coin is built from pieces.".into(),
                start: id("Sprite"),
                start_label: "Start with the Sprite".into(),
            }],
        }),
    }
}

fn open(select: bool) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.open_command_palette(model(select));
    store
}

/// Pinta a paleta aberta no `store`: os caminhos da cena e os alvos.
fn painted(store: &WidgetStore, query: &str) -> (u32, Vec<(NodeId, Rect)>) {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hits = HitIndex::new();
    let motion = crate::motion::UiMotion::default();
    let m = store.command_palette_model().expect("aberta").clone();
    let _ = paint_palette(
        &mut scene,
        &mut ts,
        Theme::default(),
        &mut hits,
        &m,
        query,
        VP,
        &motion,
        store,
    );
    (
        scene.inner().encoding().n_paths,
        hits.iter_registrations().collect(),
    )
}

/// ⭐⭐ **Ao abrir, o inicial vem destacado; sem `select`, nada se destaca** (os outros três consumidores
/// da paleta ficam como eram).
///
/// (Mutação: o `open_command_palette` sem semear o destaque ⇒ RED.)
#[test]
fn ao_abrir_o_inicial_vem_destacado_e_so_no_modo_de_selecao() {
    assert_eq!(open(true).command_palette_highlight(), Some(id("Wall")));
    assert_eq!(open(false).command_palette_highlight(), None);
    assert!(!open(false).command_palette_selects());
}

/// ⭐⭐ **A faixa tem a altura da dica MAIS ALTA**: trocar de destaque não muda a janela.
///
/// (Mutação: medir só a dica destacada ⇒ RED.)
#[test]
fn a_faixa_tem_a_altura_da_dica_mais_alta() {
    let mut ts = TextSystem::new();
    let full = dock_height(&mut ts, &model(true), 900.0);
    let mut only_short = model(true);
    only_short.tips.truncate(1);
    let short = dock_height(&mut ts, &only_short, 900.0);
    assert!(
        full > short,
        "a dica longa não alargou a faixa ({full} contra {short})"
    );
    assert_eq!(
        dock_height(&mut ts, &model(false), 900.0),
        0.0,
        "sem select não há faixa"
    );
    let mut store = open(true);
    let a = painted(&store, "").1;
    store.highlight_command_palette(id("Sprite"));
    let b = painted(&store, "").1;
    assert_eq!(a, b, "trocar de destaque mexeu nos alvos da janela");
}

/// ⭐ **A dica pinta na faixa do destacado, e o rato não pinta dica no modo de seleção.**
///
/// (Mutação: pintar a dica do hover também no modo de seleção ⇒ RED.)
#[test]
fn a_dica_e_a_do_destacado_e_o_rato_nao_a_pinta() {
    let mut store = open(true);
    let (base, _) = painted(&store, "");
    store.set_hot(Some(id("Sprite")));
    let (hovered, _) = painted(&store, "");
    assert_eq!(hovered, base, "o rato pintou uma dica no modo de seleção");
    store.set_hot(None);
    store.highlight_command_palette(id("Hero"));
    let (no_tip, _) = painted(&store, "");
    assert!(
        base > no_tip,
        "a faixa do destacado com dica não pintou o desenho e o texto"
    );
}

/// ⭐⭐ **↑ ↓ andam pelos visíveis, presos às pontas; a busca leva o destaque ao melhor resultado.**
///
/// (Mutação: o passo sem `min(last)` ⇒ RED; o destaque fixo quando a busca o esconde ⇒ RED.)
#[test]
fn as_setas_e_a_busca_movem_o_destaque() {
    let mut store = open(true);
    store.step_command_palette_highlight(1);
    assert_eq!(store.command_palette_highlight(), Some(id("Hero")));
    store.step_command_palette_highlight(5);
    assert_eq!(
        store.command_palette_highlight(),
        Some(id("Hero")),
        "passou do fim"
    );
    store.step_command_palette_highlight(-9);
    assert_eq!(store.command_palette_highlight(), Some(id("Sprite")));
    for c in "floor".chars() {
        store.command_palette_push_char(c);
    }
    assert_eq!(
        store.command_palette_highlight(),
        Some(id("Wall")),
        "a busca não levou o destaque"
    );
    assert_eq!(store.command_palette_enter_target(), Some(id("Wall")));
}

/// ⭐⭐ **A busca que não acha ensina e oferece o começo**; o botão limpa a busca e destaca a peça.
///
/// (Mutação: o `paint_teach` sem registar o botão ⇒ RED.)
#[test]
fn a_busca_que_nao_acha_ensina_e_o_botao_leva_ao_comeco() {
    let mut store = open(true);
    for c in "coins".chars() {
        store.command_palette_push_char(c);
    }
    let (_, hits) = painted(&store, "coins");
    assert!(
        hits.iter().any(|(h, _)| *h == CMD_PALETTE_TEACH_START),
        "o botão do ensino não está"
    );
    let mut ts = TextSystem::new();
    let m = store.command_palette_model().unwrap().clone();
    let filtered = super::super::scoped(&m, "coins", None).model.into_owned();
    let dock = super::super::scroll::metrics(&mut ts, &filtered, VP, true, "coins").dock;
    assert_eq!(dock, None, "sem resultado, a faixa vazia da dica ficou");
    assert_eq!(
        teach_for(store.command_palette_model().unwrap(), " Coin ").map(|t| t.start),
        Some(id("Sprite"))
    );
    store.restart_command_palette_highlight();
    assert_eq!(store.command_palette_query(), "");
    assert_eq!(store.command_palette_highlight(), Some(id("Sprite")));
    let mut plain = open(true);
    plain.command_palette_push_char('z');
    let (_, hits) = painted(&plain, "z");
    assert!(
        !hits.iter().any(|(h, _)| *h == CMD_PALETTE_TEACH_START),
        "sem ensino, sem botão"
    );
}
