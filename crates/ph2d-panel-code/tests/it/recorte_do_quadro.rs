//! Numa aba curta a pilha encolhe o quadro do depurador até `least(c)` (cd1b7d7504): o corpo
//! (Live, Sheet, Cards, Strip) é recortado ao seu rect. Sem placa: a gravação da `VectorScene` dá o
//! rect do recorte, a encoding do Vello dá a ordem — todo o desenho depois da barra cai entre o
//! `BEGIN_CLIP` desse recorte e o `END_CLIP` dele, e há texto do corpo fora do rect (o recorte é que
//! o esconde: sem ele, aparecia por cima da barra do código).

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardState, CardStyle, DebugCard, LiveRow, StackEntry, VarRow};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::DebugCardView;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::{DrawOp, Shape, VectorScene};

/// `vello_encoding::DrawTag::{BEGIN_CLIP, END_CLIP}` (a crate não depende do `vello` por nome).
const BEGIN_CLIP: u32 = 0x49;
const END_CLIP: u32 = 0x21;

fn var(i: usize) -> VarRow {
    VarRow {
        key: format!("v{i}"),
        name: format!("variavel_{i}"),
        depth: 0,
        value: format!("{i}.5"),
        kind: Default::default(),
        field_count: 0,
        expandable: false,
        old: None,
    }
}

fn parado(n: usize) -> DebugCard {
    DebugCard {
        state: CardState::Paused,
        line: 3,
        locals: (0..n).map(var).collect(),
        this: (0..n).map(var).collect(),
        stack: (0..n)
            .map(|i| StackEntry {
                function: format!("update_{i}"),
                line: 3 + i as u32,
            })
            .collect(),
        can_step: true,
        ..DebugCard::default()
    }
}

fn ao_vivo(n: usize) -> DebugCard {
    DebugCard {
        state: CardState::Live,
        live: (0..n)
            .map(|i| LiveRow {
                line: 1 + i as u32,
                var: var(i),
                trail: None,
            })
            .collect(),
        ..DebugCard::default()
    }
}

/// O que a pintura de `c` em `r` deixou: o rect do recorte (da gravação), as etiquetas de desenho e
/// a posição de cada corrida de glifos nelas, com a linha de base (y) dos seus glifos.
struct Pintura {
    clips: Vec<ph2d_vector::Rect>,
    tags: Vec<u32>,
    runs: Vec<(usize, Vec<f32>)>,
}

fn pinta(v: &mut DebugCardView, c: &DebugCard, r: Rect) -> Pintura {
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    scene.record();
    v.paint(
        c,
        [NodeId(1), NodeId(2)],
        r,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    let clips = scene
        .take_record()
        .iter()
        .filter_map(|op| match op {
            DrawOp::Layer { clip, .. } => Some(clip.bounding_box()),
            _ => None,
        })
        .collect();
    let e = scene.inner().encoding();
    let res = &e.resources;
    let runs = res
        .glyph_runs
        .iter()
        .map(|run| {
            let ty = run.transform.translation[1];
            let ys = res.glyphs[run.glyphs.clone()]
                .iter()
                .map(|g| ty + g.y)
                .collect();
            (run.stream_offsets.draw_tags, ys)
        })
        .collect();
    Pintura {
        clips,
        tags: e.draw_tags.iter().map(|t| t.0).collect(),
        runs,
    }
}

const ABA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 900.0,
};

/// Às alturas `least(c)` e intermédias, em cada estilo e ao vivo, o corpo pinta-se todo dentro do
/// recorte do seu rect (o de baixo da barra até ao fundo do quadro menos o respiro).
#[test]
fn o_corpo_do_quadro_numa_aba_curta_fica_dentro_do_seu_rect() {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let casos: Vec<(&str, DebugCard, Option<CardStyle>)> = vec![
        ("ao vivo", ao_vivo(30), None),
        ("Sheet", parado(30), Some(CardStyle::Sheet)),
        ("Cards", parado(30), Some(CardStyle::Cards)),
        ("Strip", parado(30), Some(CardStyle::Strip)),
    ];
    let mut fora_total = 0;
    for (nome, c, style) in &casos {
        let least = DebugCardView::least(c);
        for extra in [0.0, rh * 0.5, rh * 1.5, rh * 3.0] {
            let mut v = DebugCardView::default();
            if let Some(s) = style {
                v.style = *s;
            }
            let h = least + extra;
            let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
            let p = pinta(&mut v, c, r);
            // O corpo é o ÚLTIMO desenho: a etiqueta final fecha o recorte dele; o par acha-se para
            // trás (os recortes de dentro, dos cartões, ficam entre os dois).
            let end = p.tags.len() - 1;
            assert_eq!(
                p.tags[end], END_CLIP,
                "{nome} a {h}: o corpo não acaba num recorte"
            );
            let mut depth = 0i32;
            let begin = (0..=end)
                .rev()
                .find(|&i| {
                    depth += i32::from(p.tags[i] == END_CLIP) - i32::from(p.tags[i] == BEGIN_CLIP);
                    depth == 0
                })
                .unwrap_or_else(|| panic!("{nome} a {h}: o recorte não abre"));
            let antes = p.tags[..begin].iter().filter(|&&t| t == BEGIN_CLIP).count();
            assert_eq!(
                p.clips.len(),
                p.tags.iter().filter(|&&t| t == BEGIN_CLIP).count(),
                "{nome} a {h}: um recorte fora da gravação"
            );
            // O rect do recorte: debaixo da barra, até ao fundo do quadro menos o respiro.
            let body = p.clips[antes];
            let top = r.y + pad + rh;
            assert!(
                (body.x0 - f64::from(r.x)).abs() < 1e-3
                    && (body.x1 - f64::from(r.x + r.w)).abs() < 1e-3
                    && (body.y0 - f64::from(top)).abs() < 1e-3
                    && (body.y1 - f64::from((r.y + r.h - pad).max(top))).abs() < 1e-3,
                "{nome} a {h}: o recorte {body:?} não é o corpo"
            );
            let corpo: Vec<_> = p.runs.iter().filter(|(at, _)| *at > begin).collect();
            assert!(!corpo.is_empty(), "{nome} a {h}: o corpo não pintou texto");
            for (at, _) in &corpo {
                assert!(*at < end, "{nome} a {h}: texto do corpo fora do recorte");
            }
            fora_total += corpo
                .iter()
                .flat_map(|(_, ys)| ys)
                .filter(|&&y| f64::from(y) > body.y1 || f64::from(y) < body.y0)
                .count();
        }
    }
    assert!(
        fora_total > 0,
        "controlo: o texto do corpo cabia sempre — o recorte não estaria a ser medido"
    );
}
