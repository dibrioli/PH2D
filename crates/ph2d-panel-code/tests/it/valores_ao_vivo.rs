//! Plano 14 Q3: o «Live values» é a tabela da casa Line · Name · Value · Last 4 s; pairar sobre um
//! traço pede a mão, clicar abre o gráfico grande por baixo da linha (e fecha); as três aparências
//! registam os traços; a amostra é a de 2,5 Hz e o anel guarda 4 s.

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardStyle, DebugCard, LineLive, ValueKind, Var, WhoTag, live_card};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::live::{GRAPH_ROWS, span_text};
use ph2d_panel_code::debug_card::{CardClick, DebugCardView};
use ph2d_panel_code::live_sampler::{LiveSampler, window_samples};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

const ABA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 900.0,
};
const PERIODO: u128 = 400_000_000;

/// `n` amostras do jogo a correr: `self.t` sobe 0,4 por amostra (linha 13), `label` é texto (14).
fn amostrado(n: usize) -> (LiveSampler, DebugCard) {
    let mut s = LiveSampler::default();
    for i in 1..=n {
        let t = Var::leaf("self.t", format!("{}", i as f32 * 0.4), ValueKind::Number);
        let label = Var::leaf("label", "\"shape\"", ValueKind::String);
        let live = BTreeMap::from([
            (
                13,
                LineLive {
                    hits: 1,
                    values: vec![t],
                    prints: 0,
                },
            ),
            (
                14,
                LineLive {
                    hits: 1,
                    values: vec![label],
                    prints: 0,
                },
            ),
        ]);
        s.take(i as u128 * PERIODO, &live);
    }
    let who = WhoTag {
        name: "Circle".into(),
        color: None,
    };
    let c = live_card(who, s.rows.clone());
    (s, c)
}

fn pinta(v: &mut DebugCardView, c: &DebugCard) -> Rect {
    let h = v.height(c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let ids = [NodeId(1), NodeId(2)];
    let mut code = CodeView::new();
    v.paint(c, ids, r, &mut code, &mut scene, &mut ts, Theme::default());
    r
}

/// Como [`pinta`], com a mono medida (uma pintura de aquecimento do código): as larguras do texto
/// são as de verdade.
fn pinta_medido(v: &mut DebugCardView, c: &DebugCard) {
    let h = v.height(c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let mut ts = TextSystem::without_system_fonts();
    let mut code = CodeView::new();
    let buf = ph2d_code_buffer::Buffer::new("a\n");
    let _ = crate::pinta(&mut code, &buf, &mut ts, ABA, &crate::Deco::default());
    let mut scene = VectorScene::new();
    let ids = [NodeId(1), NodeId(2)];
    v.paint(c, ids, r, &mut code, &mut scene, &mut ts, Theme::default());
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

#[test]
fn o_live_values_e_uma_tabela_com_o_traco_dos_ultimos_4_s() {
    assert_eq!(window_samples(), 10, "4 s a 2,5 Hz");
    let (s, c) = amostrado(12);
    let t = s
        .rows
        .iter()
        .find(|r| r.var.key == "13:self.t")
        .expect("self.t");
    let trail = t.trail.clone().expect("um número tem traço");
    assert_eq!(trail.len(), 10, "o anel guarda 4 s");
    assert!(
        (trail[9] - 4.8).abs() < 1e-4 && (trail[0] - 1.2).abs() < 1e-4,
        "{trail:?}"
    );
    assert_eq!(
        t.var.old.as_deref(),
        Some("4.4"),
        "mudou desde a amostra anterior"
    );
    let label = s
        .rows
        .iter()
        .find(|r| r.var.key == "14:label")
        .expect("label");
    assert!(
        label.trail.is_none() && label.var.old.is_none(),
        "texto igual: sem traço nem âmbar"
    );
    assert_eq!(
        span_text(&trail).as_deref(),
        Some("min 1.2 \u{b7} max 4.8 \u{b7} now 4.8")
    );

    let mut v = DebugCardView::default();
    let r = pinta(&mut v, &c);
    let head = DebugCardView::default().height(&DebugCard::default(), ABA);
    assert!(
        r.h > head + 2.0 * ph2d_tokens::row_pitch_px(),
        "o Live tem corpo: {}",
        r.h
    );
    let (lay, rows, cols) = v.painted.table.clone().expect("a Sheet é a tabela");
    let keys: Vec<&str> = cols.iter().map(|c| &*c.key).collect();
    assert_eq!(keys, ["line", "name", "value", "trail"]);
    assert_eq!(rows.len(), 2);
    assert!(
        v.painted.split.is_none(),
        "sem coluna do lado, sem separador"
    );
    assert!(v.painted.grip.is_some(), "o puxador de cima também no Live");
    let menu: Vec<_> = v.column_menu().into_iter().map(|m| m.0).collect();
    assert_eq!(menu, ["trail"], "o traço esconde-se pelo cabeçalho");

    // Só o número tem traço; pairar pede a mão; o Line leva à linha; o Value copia.
    assert_eq!(v.painted.sparks.len(), 1, "{:?}", v.painted.sparks);
    let (key, spark) = v.painted.sparks[0].clone();
    assert_eq!(key, "13:self.t");
    assert_eq!(Some(spark), lay.cell_rect(0, "trail"));
    let (sx, sy) = centro(spark);
    assert_eq!(v.look(sx, sy), Some(PointerLook::Hand));
    let (lx, ly) = centro(lay.cell_rect(0, "line").expect("Line"));
    assert_eq!(v.press(lx, ly), Some(CardClick::Jump(13)));
    v.release();
    let (vx, vy) = centro(lay.cell_rect(0, "value").expect("Value"));
    assert_eq!(v.press(vx, vy), Some(CardClick::Copy("4.8".into())));
    v.release();

    // Clicar no traço abre o gráfico grande por baixo da linha; clicar nele fecha.
    assert_eq!(v.press(sx, sy), Some(CardClick::Taken));
    v.release();
    assert_eq!(v.graph.as_deref(), Some("13:self.t"));
    let antes = r.h;
    let r = pinta(&mut v, &c);
    assert!(
        r.h > antes,
        "o quadro cresce pelo gráfico: {antes} → {}",
        r.h
    );
    let (lay, rows, _) = v.painted.table.clone().unwrap();
    assert_eq!(rows.len(), 2 + GRAPH_ROWS);
    let big = v
        .painted
        .sparks
        .iter()
        .find(|(_, b)| b.h > spark.h * 2.0)
        .expect("o grande")
        .1;
    let row1 = lay.cell_rect(0, "name").unwrap();
    assert!(big.y >= row1.y + row1.h - 0.5, "por baixo da linha dele");
    let label_r = lay
        .rows
        .iter()
        .find(|s| rows[s.index].key == "14:label")
        .unwrap()
        .rect;
    assert!(label_r.y >= big.y + big.h - 0.5, "a linha seguinte desce");
    let (gx, gy) = centro(big);
    assert_eq!(v.look(gx, gy), Some(PointerLook::Hand));
    assert_eq!(v.press(gx, gy), Some(CardClick::Taken));
    v.release();
    assert_eq!(v.graph, None, "clicar no gráfico fecha-o");
}

#[test]
fn os_cards_e_a_strip_ao_vivo_tambem_tem_o_traco_e_o_grafico() {
    let (_, c) = amostrado(5);
    for style in [CardStyle::Cards, CardStyle::Strip] {
        let mut v = DebugCardView::default();
        v.style = style;
        let r = pinta(&mut v, &c);
        assert!(v.painted.table.is_none(), "{style:?}: sem tabela");
        assert_eq!(
            v.painted.values.len(),
            2,
            "{style:?}: os dois valores copiam-se"
        );
        let (key, spark) = v.painted.sparks.first().cloned().expect("o traço");
        assert_eq!(key, "13:self.t", "{style:?}");
        assert!(r.contains(spark.x, spark.y), "{style:?}: dentro do quadro");
        let (sx, sy) = centro(spark);
        assert_eq!(v.look(sx, sy), Some(PointerLook::Hand), "{style:?}");
        assert_eq!(v.press(sx, sy), Some(CardClick::Taken));
        v.release();
        let antes = r.h;
        let r = pinta(&mut v, &c);
        assert!(r.h > antes, "{style:?}: o gráfico grande cresce o quadro");
        let big = v.painted.sparks.last().expect("o grande").1;
        assert!(
            big.h > spark.h * 2.0 && r.contains(big.x, big.y + big.h - 1.0),
            "{style:?}"
        );
    }
}

/// Uma amostra em que o `self.t` (linha 13) vale `t` e o `label` (14) fica igual.
fn amostra(s: &mut LiveSampler, i: u128, t: &str) -> DebugCard {
    let var = |n: &str, v: &str, k| Var::leaf(n, v, k);
    let live = BTreeMap::from([
        (
            13,
            LineLive {
                hits: 1,
                values: vec![var("self.t", t, ValueKind::Number)],
                prints: 0,
            },
        ),
        (
            14,
            LineLive {
                hits: 1,
                values: vec![var("label", "\"shape\"", ValueKind::String)],
                prints: 0,
            },
        ),
    ]);
    s.take(i * PERIODO, &live);
    let who = WhoTag {
        name: "Circle".into(),
        color: None,
    };
    live_card(who, s.rows.clone())
}

/// Smoke do dono 09/11: as células Line (vai à linha) e Value (copia) pedem a mão.
#[test]
fn as_celulas_line_e_value_pedem_a_mao() {
    let (_, c) = amostrado(3);
    let mut v = DebugCardView::default();
    pinta(&mut v, &c);
    let (lay, ..) = v.painted.table.clone().expect("a Sheet");
    for col in ["line", "value"] {
        let (x, y) = centro(lay.cell_rect(0, col).expect(col));
        assert_eq!(v.look(x, y), Some(PointerLook::Hand), "{col}");
    }
    let (x, y) = centro(lay.cell_rect(0, "name").expect("name"));
    assert_eq!(v.look(x, y), None, "o Name não clica");
}

/// Smoke do dono 09/11: o número vivo cresce e volta a encolher — as colunas da Sheet e o traço da
/// Strip não mexem (as larguras só crescem enquanto o Live dura).
#[test]
fn os_numeros_vivos_mudam_de_largura_e_nada_mexe() {
    let valores = ["1.2", "12345.678", "1.2"];
    let mut s = LiveSampler::default();
    let mut v = DebugCardView::default();
    let mut cols = Vec::new();
    for (i, t) in valores.iter().enumerate() {
        let c = amostra(&mut s, i as u128 + 1, t);
        pinta_medido(&mut v, &c);
        cols.push(v.painted.table.clone().expect("a Sheet").0.cols);
        eprintln!(
            "DBG {t} {:?} {:?}",
            v.painted.table.as_ref().unwrap().0.cols,
            v.painted.table.as_ref().unwrap().1[0].cells[2].text
        );
    }
    assert_ne!(
        cols[0], cols[1],
        "a régua mede: o número comprido alarga o Value"
    );
    assert_eq!(
        cols[1], cols[2],
        "a Sheet: nada encolhe quando o número encurta"
    );
    let traco = |c: &[ph2d_ui_table::ColSpan]| c.iter().find(|s| s.key == "trail").map(|s| s.x);
    assert!(
        traco(&cols[2]) >= traco(&cols[1]),
        "o traço não anda para a esquerda"
    );

    let mut s = LiveSampler::default();
    let mut v = DebugCardView::default();
    v.style = CardStyle::Strip;
    let mut sparks = Vec::new();
    for (i, t) in valores.iter().enumerate() {
        let c = amostra(&mut s, i as u128 + 1, t);
        pinta_medido(&mut v, &c);
        sparks.push(v.painted.sparks.first().expect("o traço").1);
    }
    assert_eq!(sparks[1], sparks[2], "a Strip: o traço não se desloca");

    // O texto no fim da linha: a moldura guarda o texto mais comprido já visto.
    let mut s = LiveSampler::default();
    let mut antes = 0;
    for (i, t) in valores.iter().enumerate() {
        amostra(&mut s, i as u128 + 1, t);
        let g = s.ghosts.iter().find(|g| g.line == 12).expect("linha 13");
        assert!(
            g.min_chars >= antes && g.min_chars >= g.text.chars().count(),
            "{g:?}"
        );
        antes = g.min_chars;
    }
    assert_eq!(
        antes,
        "self.t = 12345.678".chars().count(),
        "não desce com o «1.2»"
    );
}

/// Smoke do dono 10/11: a moldura do número ao vivo no código pinta-se em TODOS os temas. A pele plana
/// dos modernos apaga a moldura de um CONTROLO em repouso; o contorno de um VALOR não é isso.
#[test]
fn a_moldura_do_numero_ao_vivo_pinta_nos_temas_modernos_e_nos_classicos() {
    use super::{Deco, frame, marcas, spans_de};
    use ph2d_code_buffer::Buffer;
    use ph2d_panel_code::{DebugMarks, Ghost};
    let buf = Buffer::new("local x = 1\nx = x + 1\n");
    let d = Deco::default();
    let fantasma = [Ghost {
        line: 1,
        text: "x = 905".into(),
        min_chars: 0,
    }];
    let caminhos = |theme: Theme, ghosts: &[Ghost]| {
        let (mut v, mut ts) = (CodeView::new(), super::ts());
        let mut scene = VectorScene::new();
        for _ in 0..2 {
            scene = VectorScene::new();
            let mut f = |r: std::ops::Range<usize>| spans_de(&buf, r);
            let mut m = marcas(&d);
            let mut fr = frame(&buf, &d, &mut f, &mut m);
            fr.debug = DebugMarks {
                ghosts,
                ..DebugMarks::default()
            };
            v.paint(fr, ABA, &mut scene, &mut ts, &theme);
        }
        scene.inner().encoding().n_paths
    };
    let classico = Theme::ALL
        .into_iter()
        .find(|t| !t.is_modern())
        .expect("um clássico");
    let moderno = Theme::ALL
        .into_iter()
        .find(|t| t.is_modern())
        .expect("um moderno");
    let extra = |t| caminhos(t, &fantasma) - caminhos(t, &[]);
    assert!(extra(classico) >= 1, "controlo: o clássico pinta a moldura");
    assert_eq!(
        extra(moderno),
        extra(classico),
        "{moderno:?}: a moldura do valor sumiu"
    );
}

/// O contorno de valor é um traço em todos os temas (nunca o «nada em repouso» da porta do tema).
#[test]
fn o_contorno_de_valor_e_um_traco_em_todos_os_temas() {
    let r = Rect::new(10.0, 10.0, 80.0, 20.0);
    for t in Theme::ALL {
        let mut scene = VectorScene::new();
        let ink = ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Border, t);
        ph2d_panel_code::cards::value_outline(&mut scene, r, 4.0, ink);
        assert_eq!(scene.inner().encoding().n_paths, 1, "{t:?}");
    }
}
