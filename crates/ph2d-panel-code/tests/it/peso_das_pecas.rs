//! O peso de cada peça do quadro do depurador, a maquete 14 (`docs/CodeEditor/maquetes/
//! 14_quadros_do_depurador.html`): procura-se na cena a corrida de glifos de cada texto e lê-se o eixo
//! variável dela (onde o peso viaja), contra o mesmo texto pintado no peso que o CSS pede.

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardState, DebugCard, StackEntry, ValueKind, VarRow, WhoTag};
use ph2d_editor_core::paint::paint_text_in_weight;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::DebugCardView;
use ph2d_text::{FontWeight, TextSystem};

/// Os pesos do CSS da maquete, escritos À MÃO (nunca as constantes da crate: a régua não é o medido).
const CSS_400: FontWeight = FontWeight::NORMAL;
const CSS_500: FontWeight = FontWeight::MEDIUM;
const CSS_600: FontWeight = FontWeight::SEMI_BOLD;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};

/// Cada corrida de glifos da cena: os ids, o corpo e os eixos variáveis.
fn corridas(scene: &VectorScene) -> Vec<(Vec<u32>, f32, Vec<i16>)> {
    let r = &scene.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .map(|g| {
            let ids = r.glyphs[g.glyphs.clone()].iter().map(|x| x.id).collect();
            let eixos = r.normalized_coords[g.normalized_coords.clone()]
                .iter()
                .copied()
                .collect();
            (ids, g.font_size, eixos)
        })
        .collect()
}

/// `text` pintado sozinho em `weight` no corpo `px`: a corrida de referência.
fn referencia(
    ts: &mut TextSystem,
    text: &str,
    px: f32,
    weight: FontWeight,
) -> (Vec<u32>, Vec<i16>) {
    let mut sc = VectorScene::new();
    let ink = Color::from_rgba8(255, 255, 255, 255);
    paint_text_in_weight(
        ts,
        &mut sc,
        text,
        [0.0, 0.0],
        px,
        f32::INFINITY,
        ink,
        weight,
    );
    let mut c = corridas(&sc);
    assert_eq!(c.len(), 1, "{text}: uma corrida");
    let (ids, _, eixos) = c.remove(0);
    (ids, eixos)
}

fn var(name: &str, value: &str) -> VarRow {
    VarRow {
        key: name.into(),
        name: name.into(),
        depth: 0,
        value: value.into(),
        kind: ValueKind::Number,
        field_count: 0,
        expandable: false,
        old: None,
    }
}

#[test]
fn cada_peca_do_quadro_parado_pinta_no_peso_da_maquete() {
    let c = DebugCard {
        state: CardState::Paused,
        line: 18,
        function: "update".into(),
        locals: vec![var("dt", "0.017")],
        stack: vec![StackEntry {
            function: "tick".into(),
            line: 18,
        }],
        waiting: vec![WhoTag {
            name: "Star".into(),
            color: None,
        }],
        can_step: true,
        ..DebugCard::default()
    };
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let mut v = DebugCardView::default();
    let aba = Rect::new(0.0, 0.0, 1100.0, 900.0);
    let h = v.height(&c, aba);
    let r = Rect::new(0.0, aba.h - h, aba.w, h);
    let mut view = CodeView::new();
    v.paint(
        &c,
        [NodeId(1), NodeId(2)],
        r,
        &mut view,
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    let cena = corridas(&scene);
    let waiting = tr_with("code.debug.waiting_here", &[("count", &1)]);
    // (o papel no CSS, o texto, o peso da maquete)
    let pecas: [(&str, &str, FontWeight); 9] = [
        (".status", tr("code.debug.paused"), CSS_600),
        (".btn", tr("code.debug.continue"), CSS_500),
        (".btn", tr("code.debug.next_line"), CSS_500),
        (".side h4", tr("code.debug.stack"), CSS_500),
        (".side h4", &waiting, CSS_500),
        (".stack (corpo)", "tick", CSS_400),
        (".tbl th", tr("code.debug.col.name"), CSS_500),
        (".tbl tr.grp", tr("code.debug.group.locals"), CSS_600),
        (".tbl td.t", tr("code.debug.kind.number"), CSS_400),
    ];
    for (papel, text, peso) in pecas {
        let (ids, _) = referencia(&mut ts, text, 12.0, CSS_400);
        let achada = cena
            .iter()
            .find(|(i, _, _)| *i == ids)
            .unwrap_or_else(|| panic!("{papel} «{text}»: nenhuma corrida na cena"));
        let (_, esperado) = referencia(&mut ts, text, achada.1, peso);
        let (_, medio) = referencia(&mut ts, text, achada.1, FontWeight::MEDIUM);
        let (_, normal) = referencia(&mut ts, text, achada.1, FontWeight::NORMAL);
        assert_ne!(medio, normal, "a sonda não separa os pesos");
        assert_eq!(
            achada.2, esperado,
            "{papel} «{text}»: não pintou no peso da maquete"
        );
    }
    // `.mono` (td.v, o código): 400, o peso de omissão da JetBrains Mono — o eixo fica a zero.
    let mut cache = ph2d_panel_code::LineCache::default();
    cache.begin_frame();
    let layout = cache.get(&mut ts, "0.017", &[], ph2d_tokens::TypeToken::Base.px());
    let mut ids = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let ph2d_text::PositionedLayoutItem::GlyphRun(run) = item {
                ids.extend(run.glyphs().map(|g| g.id));
            }
        }
    }
    let mono: Vec<_> = cena.iter().filter(|(i, _, _)| *i == ids).collect();
    assert!(!mono.is_empty(), "td.v «0.017»: nenhuma corrida na cena");
    for (_, _, eixos) in mono {
        assert!(
            eixos.iter().all(|e| *e == 0),
            ".mono «0.017»: não pintou em 400 ({eixos:?})"
        );
    }
}
