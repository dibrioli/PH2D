//! B16.3 (BUGS #13): ligar a aguada a meio do traço. O meio do traço decide-se no pen-down; o toque
//! no interruptor com a caneta em baixo não pode deixar estado da aguada para o traço seguinte.

use super::*;

fn pincel(t: &mut PainterTool, aguada: bool) {
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: aguada,
        edge_gain: 3.0,
        edge_spread: 12.0,
        granulation: 0.4,
        fill: 0.35,
        depth: 2.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
}

/// Traço em `y`; `troca_em = Some(i)` vira o interruptor da aguada no Move `i` com a caneta em baixo.
fn traco(t: &mut PainterTool, y: f32, troca_em: Option<usize>) {
    assert!(t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down)));
    for i in 1..=30 {
        if troca_em == Some(i) {
            t.toggle_brush_watercolor();
            t.paint.brush_by_mode.fill(t.paint.brush);
        }
        t.on_canvas_pointer(cp([30.0 + i as f32 * 6.0, y], PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp([210.0, y], PointerPhase::Up));
    frame(t);
}

struct Foto {
    tela: Vec<u8>,
    molhado: bool,
    sessao: bool,
    estilos: usize,
}

fn foto(t: &PainterTool) -> Foto {
    Foto {
        tela: t.canvas_rgba.to_vec(),
        molhado: t.paint.canvas_wet_rect.is_some(),
        sessao: t.paint.wet_session_canvas.is_some(),
        estilos: t.paint.wet_styles.table.len(),
    }
}

/// `molhado_antes`: uma aguada ainda molhada debaixo do traço que troca a meio; `inicial` = o meio
/// no pen-down desse traço.
fn corre(inicial: bool, troca_em: Option<usize>, molhado_antes: bool) -> (Foto, Foto) {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    if molhado_antes {
        pincel(&mut t, true);
        traco(&mut t, 70.0, None);
    }
    pincel(&mut t, inicial);
    traco(&mut t, 80.0, troca_em);
    let um = foto(&t);
    // O 2.º traço, já com a aguada ligada desde o pen-down, cruza o 1.º.
    pincel(&mut t, true);
    traco(&mut t, 90.0, None);
    (um, foto(&t))
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

#[test]
fn ligar_a_aguada_a_meio_do_traco_nao_deixa_estado_para_o_seguinte() {
    let mut falhas = Vec::new();
    for (inicial, troca_em, molhado_antes) in [false, true].into_iter().flat_map(|a| {
        [1usize, 10, 29]
            .into_iter()
            .flat_map(move |i| [(a, i, false), (a, i, true)])
    }) {
        let (ref1, ref2) = corre(inicial, None, molhado_antes);
        assert!(
            difere(&ref1.tela, &vec![255u8; ref1.tela.len()]) > 0,
            "o traço pintou"
        );
        let (um, dois) = corre(inicial, Some(troca_em), molhado_antes);
        let d1 = difere(&um.tela, &ref1.tela);
        let d2 = difere(&dois.tela, &ref2.tela);
        let estado = (um.molhado, um.sessao, um.estilos);
        let ref_estado = (ref1.molhado, ref1.sessao, ref1.estilos);
        if d1 != 0 || d2 != 0 || estado != ref_estado {
            falhas.push(format!(
                "aguada no pen-down {inicial}, troca no Move {troca_em}, molhado antes {molhado_antes}: 1.º traço {d1} px; estado (molhado, sessão, estilos) {estado:?} × \
                 {ref_estado:?}; 2.º traço (aguada desde o pen-down) {d2} px"
            ));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}
