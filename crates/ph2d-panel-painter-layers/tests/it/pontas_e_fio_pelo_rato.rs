//! ⭐ **«Fine Thread», «Flat ends», «Ends beyond» e «Ends fade» PELA PORTA DO RATO** (auditoria J6, 28/10). As caixas
//! levam um clique no centro do rectângulo pintado; os números um ARRASTO real (Down, Moves, Up) sobre a caixa. Para
//! cada um: o valor chega à ferramenta, o painel re-pintado mostra-o, e o mesmo traço sai diferente.
//!
//! **Mutação que sangra:** tirar um destes ids da rota de envio do painel (`event.rs` para as caixas,
//! `number_field::is_param_field` para os números).

use super::censo_dos_controlos::{Bancada, Entrega, Gesto, diferenca, tique, traco};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_tool_painter::ids::{
    PAINTER_BRUSH_PONTAS_ALEM, PAINTER_BRUSH_PONTAS_ESBATE, PAINTER_BRUSH_PONTAS_RECTAS,
    PAINTER_BRUSH_SMEAR_FIO,
};
use ph2d_tool_painter::{BrushSettings, PaintMedia};

/// O centro do rectângulo pintado de `id` — e a prova de que o ponteiro ali ATERRA nele.
fn centro(b: &Bancada, id: NodeId, nome: &str) -> (f32, f32) {
    let r = b
        .rects()
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("«{nome}» não foi pintado"));
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        b.host.hit_at(x, y),
        Some(id),
        "o ponteiro no centro de «{nome}» aterra noutro sítio"
    );
    (x, y)
}

/// Um arrasto horizontal de verdade sobre a caixa numérica: passa o limiar e depois corre `dx` px em passos.
fn arrasta(b: &mut Bancada, id: NodeId, nome: &str, dx: f32) -> Vec<WidgetEvent> {
    let (x, y) = centro(b, id, nome);
    let mut out = Vec::new();
    let mut t = 1_000_000_000_000u128;
    let mut ev = |b: &mut Bancada, kind, x: f32| {
        t += 16_000_000;
        b.host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: t,
        })
    };
    out.extend(ev(b, PointerKind::Down, x));
    for k in 1..=10 {
        out.extend(ev(b, PointerKind::Move, x + 6.0 + dx * k as f32 / 10.0));
    }
    out.extend(ev(b, PointerKind::Up, x + 6.0 + dx));
    out
}

/// Um traço com as DUAS pontas dentro da tela (a da fábrica, `risca`, entra e sai pelas bordas: as pontas ficam fora e
/// o «Ends beyond» lia-se morto) que atravessa o bloco de cor (o Smear tem o que arrastar), e meio segundo de quadros.
fn traco_com_pontas(t: &mut ph2d_tool_painter::PainterTool) -> Vec<u8> {
    let mut ultima = None;
    traco(
        t,
        &mut ultima,
        [24.0, 40.0],
        [100.0, 92.0],
        [0.8, 0.8],
        [0.0, 0.0],
    );
    for _ in 0..30 {
        tique(t, &mut ultima);
    }
    ultima.expect("o traço sujou a pré-visualização")
}

struct Caso {
    nome: &'static str,
    meio: PaintMedia,
    armar: Vec<Gesto>,
    id: NodeId,
    numero: bool,
    le: fn(&BrushSettings) -> f64,
}

#[test]
fn as_pontas_e_o_fio_aceitam_o_rato_e_o_traco_muda() {
    let casos = [
        Caso {
            nome: "Fine Thread",
            meio: PaintMedia::Digital,
            armar: vec![Gesto::Modo("smear")],
            id: PAINTER_BRUSH_SMEAR_FIO,
            numero: false,
            le: |s| f64::from(u8::from(s.smear_fio)),
        },
        Caso {
            nome: "Flat ends",
            meio: PaintMedia::WetPaint,
            armar: vec![],
            id: PAINTER_BRUSH_PONTAS_RECTAS,
            numero: false,
            le: |s| f64::from(u8::from(s.pontas_rectas)),
        },
        Caso {
            nome: "Ends beyond",
            meio: PaintMedia::WetPaint,
            armar: vec![Gesto::Clique(PAINTER_BRUSH_PONTAS_RECTAS)],
            id: PAINTER_BRUSH_PONTAS_ALEM,
            numero: true,
            le: |s| f64::from(s.pontas_alem),
        },
        Caso {
            nome: "Ends fade",
            meio: PaintMedia::WetPaint,
            armar: vec![],
            id: PAINTER_BRUSH_PONTAS_ESBATE,
            numero: true,
            le: |s| f64::from(s.pontas_esbate),
        },
    ];
    let mut falhas = Vec::new();
    for c in &casos {
        let mut base = Bancada::nova(c.meio, &c.armar);
        let traco_antes = traco_com_pontas(&mut base.tool);
        // O controlo do ruído: o mesmo traço numa bancada igual, sem gesto, sai igual ao bit.
        let ruido = diferenca(
            &traco_antes,
            &traco_com_pontas(&mut Bancada::nova(c.meio, &c.armar).tool),
        )
        .0;
        assert_eq!(
            ruido, 0,
            "{}: o traço sem gesto já difere (n={ruido}) — a régua é ruído",
            c.nome
        );
        let mut b = Bancada::nova(c.meio, &c.armar);
        let antes = (c.le)(&b.tool.brush_settings());
        let tela_antes = b.geometria();
        let eventos = if c.numero {
            arrasta(&mut b, c.id, c.nome, 120.0)
        } else {
            let (x, y) = centro(&b, c.id, c.nome);
            b.host.click_at(x, y)
        };
        let mut entrega = Entrega::default();
        b.aplica_eventos(eventos, &mut entrega);
        let depois = (c.le)(&b.tool.brush_settings());
        if entrega.ao_tool == 0 || (depois - antes).abs() < 1e-6 {
            falhas.push(format!(
                "{}: o gesto do rato não chegou à ferramenta (ao_tool={}, {antes} → {depois})",
                c.nome, entrega.ao_tool
            ));
            continue;
        }
        if c.numero {
            let visto = b.host.store().number_value(c.id);
            if visto.is_none_or(|v| (v - depois).abs() > 1e-3) {
                falhas.push(format!(
                    "{}: o painel não redesenha o valor ({visto:?} vs {depois})",
                    c.nome
                ));
            }
        } else if b.geometria() == tela_antes {
            falhas.push(format!(
                "{}: o painel re-pintado desenha a caixa como antes",
                c.nome
            ));
        }
        let n = diferenca(&traco_antes, &traco_com_pontas(&mut b.tool)).0;
        eprintln!("PONTAS-RATO\t{}\t{antes} → {depois}\tn={n}", c.nome);
        if n == 0 {
            falhas.push(format!("{}: o traço saiu igual ao de antes", c.nome));
        }
    }
    assert!(
        falhas.is_empty(),
        "{} controlo(s):\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}
