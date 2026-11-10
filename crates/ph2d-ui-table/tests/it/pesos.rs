//! Os pesos da tabela (os da casa por omissão; os finais de quem os passa), a medida do cabeçalho
//! pela porta de pintar, e a coluna justa sem puxador.

use ph2d_editor_core::paint::{
    paint_text, paint_text_in_weight, paint_text_title, set_text_rendering,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, TextRendering, Theme, TypeToken};
use ph2d_ui_table::{
    Cell, Column, Hit, Ink, Look, Metrics, Row, TableState, UiText, layout, natural_widths, paint,
};
use ph2d_vector::{Color, VectorScene};

/// Cada corrida: os ids, o x do 1.º glifo e os eixos.
fn corridas(scene: &VectorScene) -> Vec<(Vec<u32>, f32, Vec<i16>)> {
    let r = &scene.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .map(|g| {
            let gl = &r.glyphs[g.glyphs.clone()];
            let ids = gl.iter().map(|x| x.id).collect();
            let x0 = g.transform.translation[0] + gl.first().map_or(0.0, |x| x.x);
            (
                ids,
                x0,
                r.normalized_coords[g.normalized_coords.clone()].to_vec(),
            )
        })
        .collect()
}

type Pintor = fn(&mut TextSystem, &mut VectorScene, &str, f32);

/// A corrida de `text` pintada sozinha por `p`.
fn sozinho(ts: &mut TextSystem, text: &str, px: f32, p: Pintor) -> (Vec<u32>, Vec<i16>) {
    let mut sc = VectorScene::new();
    p(ts, &mut sc, text, px);
    let c = corridas(&sc).remove(0);
    (c.0, c.2)
}

const BRANCO: Color = Color::from_rgba8(255, 255, 255, 255);

fn cols() -> Vec<Column> {
    vec![
        Column::new("name", "Name").flex(),
        Column::new("sev", "").snug(),
        Column::new("value", "Value").end(),
    ]
}

fn rows() -> Vec<Row> {
    let grupo = Row {
        key: "Locals".into(),
        group: Some("1".into()),
        cells: vec![Cell::new("Locals", Ink::Plain)],
        expandable: true,
        default_open: true,
        ..Row::default()
    };
    let item = Row {
        key: "dt".into(),
        depth: 1,
        cells: vec![
            Cell::new("delta", Ink::Name),
            Cell {
                dot: Some(ColorToken::Danger),
                ..Cell::default()
            },
            Cell::new("0.017", Ink::Number),
        ],
        ..Row::default()
    };
    vec![grupo, item]
}

/// Pinta a tabela com `look` no preset de fábrica; devolve a cena e a geometria.
fn pinta(
    look: Look,
) -> (
    Vec<(Vec<u32>, f32, Vec<i16>)>,
    ph2d_ui_table::Layout,
    Metrics,
) {
    // A casa é a de omissão: o `standard()` sem nada por cima.
    let mut m = Metrics::standard();
    if look != Look::House {
        m.look = look;
    }
    let (c, r) = (cols(), rows());
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    set_text_rendering(TextRendering::CrispHeavyPlus);
    let mut text = UiText {
        theme: Theme::Dark,
        look,
    };
    let nat = natural_widths(&c, &r, &m, &mut ts, &mut text);
    let mut st = TableState::default();
    let area = Rect::new(0.0, 0.0, m.min_col * 8.0, m.row_h * 6.0);
    let l = layout(&c, &r, &nat, &mut st, area, &m);
    let tones = (ColorToken::PanelBg, ColorToken::Bg3);
    paint(
        &l,
        &c,
        &r,
        &st,
        tones,
        &mut scene,
        &mut ts,
        &mut text,
        Theme::Dark,
    );
    let out = corridas(&scene);
    set_text_rendering(TextRendering::Default);
    (out, l, m)
}

fn eixos_de(cena: &[(Vec<u32>, f32, Vec<i16>)], ids: &[u32], o_que: &str) -> Vec<i16> {
    cena.iter()
        .find(|c| c.0 == ids)
        .unwrap_or_else(|| panic!("{o_que}: nenhuma corrida"))
        .2
        .clone()
}

#[test]
fn a_tabela_da_casa_pinta_nos_pesos_da_casa_e_a_exacta_nos_que_recebe() {
    let small = TypeToken::Xs.px();
    let sm = TypeToken::Sm.px();
    let mut ts = TextSystem::without_system_fonts();
    set_text_rendering(TextRendering::CrispHeavyPlus);
    let casa: [(&str, f32, Pintor); 3] = [
        ("Name", small, |ts, sc, t, f| {
            paint_text(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, BRANCO);
        }),
        ("Locals", small, |ts, sc, t, f| {
            paint_text_title(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, BRANCO);
        }),
        ("delta", sm, |ts, sc, t, f| {
            paint_text(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, BRANCO);
        }),
    ];
    let exacta: [(&str, f32, Pintor); 3] = [
        ("Name", small, |ts, sc, t, f| {
            let w = FontWeight::MEDIUM;
            paint_text_in_weight(ts, sc, t, [0.0, 0.0], f, f32::INFINITY, BRANCO, w);
        }),
        ("Locals", small, |ts, sc, t, f| {
            let w = FontWeight::SEMI_BOLD;
            paint_text_in_weight(ts, sc, t, [0.0, 0.0], f, f32::INFINITY, BRANCO, w);
        }),
        ("delta", sm, |ts, sc, t, f| {
            let w = FontWeight::NORMAL;
            paint_text_in_weight(ts, sc, t, [0.0, 0.0], f, f32::INFINITY, BRANCO, w);
        }),
    ];
    let refs: Vec<_> = casa
        .iter()
        .chain(&exacta)
        .map(|(t, f, p)| sozinho(&mut ts, t, *f, *p))
        .collect();
    set_text_rendering(TextRendering::Default);
    let (cena, ..) = pinta(Look::House);
    for (k, (t, ..)) in casa.iter().enumerate() {
        let got = eixos_de(&cena, &refs[k].0, t);
        assert_eq!(got, refs[k].1, "casa «{t}»: não pintou no peso da casa");
        assert_ne!(got, refs[k + 3].1, "casa «{t}»: a sonda não separa");
    }
    let look = Look::Exact {
        body: FontWeight::NORMAL,
        head: FontWeight::MEDIUM,
        group: FontWeight::SEMI_BOLD,
    };
    let (cena, ..) = pinta(look);
    for (k, (t, ..)) in exacta.iter().enumerate() {
        let got = eixos_de(&cena, &refs[k + 3].0, t);
        assert_eq!(got, refs[k + 3].1, "exacta «{t}»: não pintou no peso dado");
    }
}

#[test]
fn o_cabecalho_a_direita_acaba_no_recuo_com_o_espacamento_do_preset() {
    for look in [
        Look::House,
        Look::Exact {
            body: FontWeight::NORMAL,
            head: FontWeight::MEDIUM,
            group: FontWeight::SEMI_BOLD,
        },
    ] {
        let (cena, l, m) = pinta(look);
        let mut ts = TextSystem::without_system_fonts();
        set_text_rendering(TextRendering::CrispHeavyPlus);
        let small = TypeToken::Xs.px();
        let w = look.weight(ph2d_ui_table::Role::Head);
        let lay = |ts: &mut TextSystem| {
            ts.layout_for_rendering(
                "Value",
                small,
                f32::INFINITY,
                w,
                TextRendering::CrispHeavyPlus,
            )
        };
        let painted_w = match look {
            Look::House => lay(&mut ts).width(),
            Look::Exact { .. } => ph2d_text::com_peso_exato(|| lay(&mut ts)).width(),
        };
        let (ids, _) = sozinho(&mut ts, "Value", small, |ts, sc, t, f| {
            paint_text(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, BRANCO);
        });
        set_text_rendering(TextRendering::Default);
        let c = l.col("value").expect("a coluna value");
        let x0 = cena
            .iter()
            .find(|r| r.0 == ids)
            .expect("o título Value pintado")
            .1;
        let right = c.x + c.w - m.pad;
        assert!(
            (x0 + painted_w - right).abs() <= 0.5,
            "{look:?}: o título acaba em {} e o recuo está em {right}",
            x0 + painted_w
        );
    }
}

#[test]
fn a_coluna_justa_nao_tem_puxador_e_nao_salta_para_o_minimo() {
    let (_, l, m) = pinta(Look::House);
    let c = l.col("sev").expect("a coluna justa").clone();
    assert!(c.w < m.min_col, "a justa é mais estreita que o mínimo");
    let r = rows();
    let edge = (c.x + c.w, l.header.y + l.header.h * 0.5);
    assert_ne!(
        l.hit(&r, edge.0, edge.1),
        Some(Hit::Resize("sev".into())),
        "a borda da coluna justa não oferece puxador"
    );
    let mut st = TableState::default();
    assert!(
        !st.press(&Hit::Resize("sev".into()), edge, &l, &r),
        "o arrasto não começa"
    );
    st.drag_to((edge.0 + 1.0, edge.1), &m);
    assert_eq!(st.widths.get("sev"), None, "a largura não mudou");
}
