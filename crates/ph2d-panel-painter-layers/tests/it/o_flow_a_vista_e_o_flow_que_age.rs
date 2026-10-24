//! ⭐ **O Flow está à vista ⟺ a ferramenta na mão, no meio dela, o lê** (auditoria 24/10). Antes ele só se pintava
//! no Pincel Digital por um `if` à mão, e agia escondido no Impasto (centro do traço 93 em vez de 255 com Flow 0,05)
//! e na Borracha com o Sync (levantava 64/255). Cada par meio × ferramenta, sobre riscas secas (e molhadas no Wet
//! Paint): o painel pinta a linha Flow, e Flow 1 → 0,05 muda o traço. Os dois têm de concordar.

use super::censo_dos_controlos::{Bancada, Gesto, diferenca, risca, traco};
use super::censo_ferramenta_x_linha::{FERRAMENTAS, bancada_com, molha, riscas};
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::ids::{PAINTER_BRUSH_FLOW_SLIDER, PAINTER_BRUSH_SYNC};

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

const SYNC: [Gesto; 1] = [Gesto::Clique(PAINTER_BRUSH_SYNC)];

/// A ferramenta `modo` no `meio` (com o Sync: o Flow do Pincel é o dela), ou `None` se o meio não a tem.
/// `"composite"` é o Pincel com o Composite Brush armado.
fn bancada(modo: &str, meio: PaintMedia, molhado: bool) -> Option<Bancada> {
    let composite = modo == "composite";
    let modo = if composite { "brush" } else { modo };
    let mut b = bancada_com(modo, &SYNC, &Gesto::Nenhum);
    b.tool.set_paint_media(meio);
    b.no_modo(modo);
    if b.tool.paint_media() != meio {
        return None;
    }
    if composite {
        b.tool.toggle_composite();
        b.tool.set_composite_layer_strength(0, 1.0);
        b.no_modo(modo);
        if !b.tool.composite_enabled() {
            return None;
        }
    }
    if molhado {
        molha(&mut b, modo);
    }
    Some(b)
}

fn traco_com(modo: &str, meio: PaintMedia, molhado: bool, flow: f32) -> Vec<u8> {
    let mut b = bancada(modo, meio, molhado).expect("o par existe");
    b.tool.set_brush_flow(flow);
    risca(&mut b.tool)
}

/// O Sculpt mexe no RELEVO, não na pré-visualização: `(Flow à vista, células do relevo que o Flow muda)`, sobre
/// riscas pintadas no Impasto.
fn relevo_do_sculpt() -> (bool, usize) {
    let um = |flow: f32| {
        let mut b = Bancada::nova(PaintMedia::Impasto, &SYNC);
        riscas(&mut b);
        b.no_modo("sculpt");
        assert!(
            b.tool.brush_settings().is_sculpt,
            "o Sculpt não ficou na mão"
        );
        let a_vista = b.ids().contains(&PAINTER_BRUSH_FLOW_SLIDER);
        b.tool.set_brush_flow(flow);
        let mut ultima = None;
        traco(
            &mut b.tool,
            &mut ultima,
            [8.0, 64.0],
            [128.0, 64.0],
            [1.0, 1.0],
            [0.0, 0.0],
        );
        let camada = b.tool.layers().active().expect("uma camada");
        let relevo = b.tool.layer_height_view(camada);
        (a_vista, relevo.expect("as riscas deixaram relevo"))
    };
    let (a_vista, cheio) = um(1.0);
    let (_, fraco) = um(0.05);
    let n = cheio
        .iter()
        .zip(&fraco)
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    (a_vista, n)
}

#[test]
fn o_flow_esta_a_vista_onde_age_e_so_ai() {
    let mut medidas = Vec::new();
    for meio in MEIOS {
        for modo in FERRAMENTAS
            .into_iter()
            .chain(["knife", "inpaint", "composite"])
        {
            let Some(b) = bancada(modo, meio, false) else {
                continue;
            };
            let a_vista = b.ids().contains(&PAINTER_BRUSH_FLOW_SLIDER);
            let estados: &[bool] = if meio == PaintMedia::WetPaint {
                &[false, true]
            } else {
                &[false]
            };
            // Um par sem traço na pré-visualização (o Inpaint no Wet Paint) não é desta pergunta.
            let Ok(texels) = std::panic::catch_unwind(|| {
                estados
                    .iter()
                    .map(|&molhado| {
                        diferenca(
                            &traco_com(modo, meio, molhado, 1.0),
                            &traco_com(modo, meio, molhado, 0.05),
                        )
                        .0
                    })
                    .max()
                    .unwrap_or(0)
            }) else {
                eprintln!("FLOW\t{meio:?}\t{modo}\tsem traço");
                continue;
            };
            medidas.push((format!("{meio:?}/{modo}"), a_vista, texels));
        }
    }
    let (a_vista, relevo) = relevo_do_sculpt();
    medidas.push(("Impasto/sculpt (relevo)".to_owned(), a_vista, relevo));
    let mut falhas = Vec::new();
    for (par, a_vista, n) in &medidas {
        eprintln!("FLOW\t{par}\tà vista={a_vista}\tmuda={n}");
        if *a_vista != (*n > 0) {
            falhas.push(format!("{par}: Flow à vista = {a_vista}, mas muda {n}"));
        }
    }
    assert!(
        medidas.len() >= 15,
        "controlo: só {} pares meio × ferramenta",
        medidas.len()
    );
    // Controlo dos dois lados: há pares onde o Flow age e pares onde não age.
    assert!(medidas.iter().any(|m| m.2 > 0) && medidas.iter().any(|m| m.2 == 0));
    assert!(falhas.is_empty(), "{falhas:#?}");
}
