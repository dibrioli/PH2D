//! Gates do acabamento (Add Object estreito, dono 07/11) — pela PORTA da geometria e do pintor.

use super::super::scroll::metrics;
use super::super::{
    PaletteGroup, PaletteItem, PaletteModel, PaletteSelect, PaletteSub, WidgetStore,
    paint as paint_palette,
};
use super::*;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;
use std::cell::RefCell;

/// Um ecrã largo: a janela de sempre chega ao tecto dela (1160), a nova ao dela.
const VP: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1000.0,
};

fn model(select: bool) -> PaletteModel {
    let id = |n: &str| ph2d_tool_registry::hash_node_id_runtime(n);
    let group = |t: &str, items: &[&str]| PaletteGroup {
        title: t.into(),
        color: ColorToken::NodeCatOutput,
        subs: vec![PaletteSub {
            title: None,
            items: items
                .iter()
                .map(|n| PaletteItem {
                    badge: None,
                    label: (*n).into(),
                    id: id(n),
                    keywords: "",
                })
                .collect(),
        }],
    };
    PaletteModel {
        title: "Add Object".into(),
        groups: vec![
            group(
                "Pictures",
                &[
                    "Sprite",
                    "Animated Sprite",
                    "Pixel Sprite",
                    "Background Layer",
                ],
            ),
            group("Drawing", &["Vector Drawing", "Text"]),
            group("Game", &["Character", "Static Body", "Camera"]),
            group("Empty", &["Empty", "Macro"]),
        ],
        toggle: None,
        tips: Vec::new(),
        select: select.then(|| PaletteSelect {
            initial: id("Sprite"),
            teach: Vec::new(),
        }),
    }
}

/// ⭐⭐ **O Add Object é estreito, em 3 colunas; os outros consumidores ficam como eram.**
///
/// (Mutações: o tecto do `compact` de volta a 1160 ⇒ RED; o `look` a devolver sempre o `compact`
/// ⇒ RED na metade de sempre.)
#[test]
fn o_add_object_e_estreito_e_os_outros_nao_mudam() {
    let mut ts = TextSystem::new();
    let novo = metrics(&mut ts, &model(true), VP, false, "");
    assert!(
        novo.card.w <= 820.0,
        "o Add Object passou do tecto da referência: {}",
        novo.card.w
    );
    let mut colunas: Vec<i32> = novo.placed.iter().map(|p| p.1.round() as i32).collect();
    colunas.sort_unstable();
    colunas.dedup();
    assert_eq!(
        colunas.len(),
        3,
        "a referência arruma os grupos em 3 colunas"
    );

    // A janela de sempre: a mesma largura e as mesmas medidas de antes de 07/11.
    let velho = metrics(&mut ts, &model(false), VP, false, "");
    assert_eq!(velho.card.w, 1160.0, "a paleta de sempre mudou de largura");
    let c = look(&model(false));
    assert_eq!(c, classic());
    assert_eq!(
        (c.header_h, c.pill_h, c.pill_gap, c.card_pad_x, c.units),
        (44.0, 26.0, 6.0, 12.0, 4),
        "as medidas da paleta de sempre mudaram"
    );
    assert!(!c.count_right && !c.search_icon);
}

thread_local! {
    static TEXTOS: RefCell<Vec<(String, [f32; 4])>> = const { RefCell::new(Vec::new()) };
}
fn ouve(t: &str, r: [f32; 4], _corpo: f32) {
    TEXTOS.with(|v| v.borrow_mut().push((t.to_owned(), r)));
}

/// Pinta a paleta e devolve os textos pintados com o rect de cada um.
fn textos(select: bool) -> Vec<(String, [f32; 4])> {
    let mut store = WidgetStore::with_capacity(4);
    store.open_command_palette(model(select));
    let m = store.command_palette_model().expect("aberta").clone();
    TEXTOS.with(|v| v.borrow_mut().clear());
    crate::paint::ouve_o_texto(Some(ouve));
    let _ = paint_palette(
        &mut VectorScene::new(),
        &mut TextSystem::new(),
        Theme::default(),
        &mut HitIndex::new(),
        &m,
        "",
        VP,
        &crate::motion::UiMotion::default(),
        &store,
    );
    crate::paint::ouve_o_texto(None);
    TEXTOS.with(|v| v.borrow().clone())
}

/// ⭐⭐ **A contagem do grupo fica à DIREITA do cartão**, separada do título (a referência); a
/// paleta de sempre continua com «Title · N» colado.
///
/// (Mutação: `count_right: false` no `compact` ⇒ RED.)
#[test]
fn a_contagem_do_grupo_fica_a_direita() {
    let mut ts = TextSystem::new();
    let m = metrics(&mut ts, &model(true), VP, false, "");
    let lk = compact();
    let novo = textos(true);
    let achar = |t: &str| novo.iter().find(|(s, _)| s == t).map(|p| p.1);
    let titulo = achar("Game").expect("o título do grupo pinta-se sozinho");
    let conta = achar("3").expect("a contagem pinta-se sozinha");
    let (_, ox, _, w) = m
        .placed
        .iter()
        .find(|p| (p.1 - titulo[0] + lk.card_pad_x).abs() < 0.5)
        .expect("o cartão do título");
    let direita = ox + w - lk.card_pad_x;
    assert!(
        (conta[0] + conta[2] - direita).abs() <= 1.0,
        "a contagem não encosta à direita: acaba em {} e o cartão em {direita}",
        conta[0] + conta[2]
    );
    assert!(
        conta[0] > titulo[0] + titulo[2],
        "a contagem cola no título"
    );
    assert!(!novo.iter().any(|(s, _)| s.contains('\u{00b7}')));

    let velho = textos(false);
    assert!(
        velho
            .iter()
            .any(|(s, _)| s.replace('\u{a0}', " ") == "Game \u{00b7} 3"),
        "a paleta de sempre perdeu o «Title · N»"
    );
}

/// As pílulas de uma paleta, por rótulo: `(rótulo, rect no cartão + origem do cartão)`.
fn pilulas(ts: &mut TextSystem, m: &PaletteModel) -> Vec<(String, [f32; 4])> {
    let mt = metrics(ts, m, VP, false, "");
    let mut v = Vec::new();
    for (cl, ox, oy, _) in &mt.placed {
        for p in &cl.placed {
            if let super::super::Placed::Pill { rect, label, .. } = p {
                v.push((label.clone(), [ox + rect.x, oy + rect.y, rect.w, rect.h]));
            }
        }
    }
    v
}

/// ⭐ **A plaquinha «+N» (Add Component, maquete B `.bri`) cabe DENTRO da pílula, e quem não a tem
/// fica como era.** (a) a pílula com plaquinha é mais larga que a mesma sem ela; (b) num modelo sem
/// plaquinha nenhuma cada pílula mede o de sempre (ponto + vão + texto + recuos), e uma plaquinha
/// num cartão não mexe nos outros cartões. Nos dois acabamentos.
///
/// (Mutação: tirar a reserva de largura da plaquinha no `flow_pills` ⇒ RED em (a).)
#[test]
fn a_plaquinha_cabe_na_pilula_e_nao_muda_quem_nao_a_tem() {
    let mut ts = TextSystem::new();
    for select in [false, true] {
        let lk = look(&model(select));
        let sem = model(select);
        let mut com = model(select);
        com.groups[0].subs[0].items[0].badge = Some("+3".into());
        let (p_sem, p_com) = (pilulas(&mut ts, &sem), pilulas(&mut ts, &com));
        assert_eq!(p_sem.len(), p_com.len());

        let w = |v: &[(String, [f32; 4])], l: &str| {
            v.iter().find(|(s, _)| s == l).map(|p| p.1[2]).expect(l)
        };
        let chip =
            super::super::layout::badge_gap() + super::super::layout::badge_size(&mut ts, "+3").0;
        assert!(
            (w(&p_com, "Sprite") - w(&p_sem, "Sprite") - chip).abs() < 0.01,
            "a pílula não reservou a plaquinha ({select}): {} contra {}",
            w(&p_com, "Sprite"),
            w(&p_sem, "Sprite")
        );

        for (label, r) in &p_sem {
            let texto = ts.prefix_width_weighted(label, lk.pill_font, lk.pill_weight);
            let antes =
                lk.dot_r * 2.0 + ph2d_tokens::Spacing::Xs.px() + texto + lk.pill_pad_x * 2.0;
            assert!(
                (r[2] - antes).abs() < 0.01,
                "{label:?} mudou de largura sem plaquinha ({select}): {} contra {antes}",
                r[2]
            );
        }
        let pictures = [
            "Sprite",
            "Animated Sprite",
            "Pixel Sprite",
            "Background Layer",
        ];
        for (a, b) in p_sem.iter().zip(&p_com) {
            if !pictures.contains(&a.0.as_str()) {
                assert_eq!(a, b, "uma plaquinha noutro cartão mexeu em {:?}", a.0);
            }
        }
    }
}
