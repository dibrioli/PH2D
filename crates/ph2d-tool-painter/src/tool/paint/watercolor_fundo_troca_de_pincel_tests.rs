//! **Trocar de pincel para a aguada também aquece o fundo do toque** (n.º 7 item 3f, «Continuação 6»): o preset da
//! aguada sem troca de camada lança o mesmo fio do «+» e o 1.º toque colhe-o, ao bit da rota do clique. E um fio
//! descartado pára entre faixas, não no fim da composição.

use super::watercolor_aquece::{CONTAS, FUNDO_AQUECIDO_EM_FUNDO};
use super::wetpaint_preset::de_fabrica_ou_candidato;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::sync::Arc;

const W: u32 = 96;
const H: u32 = 64;

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32, fotos: &mut Vec<Vec<u8>>) {
    t.on_canvas_pointer(cp(10.0, y, PointerPhase::Down));
    fotos.push(t.paint.wet_backdrop.as_deref().cloned().unwrap_or_default());
    for k in 1..=5 {
        t.on_canvas_pointer(cp(10.0 + 12.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(80.0, y, PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    fotos.push(t.canvas_rgba.to_vec());
    fotos.push(t.chao_preto_congelado());
}

/// O fundo do próximo toque está a caminho: um fio por colher, ou a cache já casa com a pilha de agora.
fn aquecido(t: &PainterTool) -> bool {
    let chave = t.chave_do_fundo();
    t.aquecimento_acabou().is_some()
        || matches!(&t.paint.fundo_em_cache, Some((c, f)) if c.casa(&chave) && t.chao_preto_fresco(f))
}

/// `(fotos, (colhidos, descartados), aquecido antes do preset, aquecido depois)`.
fn corre(em_fundo: bool, pincel: &str, descarta: bool) -> (Vec<Vec<u8>>, (u32, u32), bool, bool) {
    FUNDO_AQUECIDO_EM_FUNDO.set(em_fundo);
    CONTAS.set((0, 0));
    let mut t = PainterTool::default();
    let base: Vec<u8> = (0..W * H * 4)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 9) as u8 | 0x40)
        .collect();
    t.abre_a_sprite(base, W, H);
    let fundo_id = t.layers.active().expect("a base");
    t.preset_aplicar(&de_fabrica_ou_candidato("Hard Round"))
        .expect("meio");
    t.set_brush_size_px(10.0);
    traco(&mut t, 20.0, &mut Vec::new()); // o traço seco não tem fundo de aguada
    let mut fotos = Vec::new();
    t.add_raster_layer("nova").expect("a nova");
    let antes = aquecido(&t);
    t.preset_aplicar(&de_fabrica_ou_candidato(pincel))
        .expect("meio");
    t.set_brush_size_px(10.0);
    let depois = aquecido(&t);
    if descarta {
        // A base muda entre a troca e o toque: o fio compôs (ou compunha) a de antes.
        let img = Arc::make_mut(t.images.get_mut(&fundo_id).expect("a base"));
        Arc::make_mut(&mut img.rgba8)[0] ^= 0xFF;
    } else {
        while t.aquecimento_acabou() == Some(false) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    traco(&mut t, 30.0, &mut fotos);
    traco(&mut t, 40.0, &mut fotos);
    FUNDO_AQUECIDO_EM_FUNDO.set(true);
    (fotos, CONTAS.get(), antes, depois)
}

/// ⭐ O preset da aguada aquece o fundo do 1.º toque sem troca de camada; o toque colhe-o e dá o mesmo bit que a
/// rota do clique. Descartado (a pilha mudou), compõe como antes, ao mesmo bit.
///
/// **Mutação que sangra:** tirar o `aquece_o_fundo_do_toque` do fim de `preset_aplicar` (aquecido depois = `false`,
/// colhidos = 0).
#[test]
fn o_fundo_do_toque_aquece_na_troca_de_pincel_e_e_o_mesmo_bit() {
    for descarta in [false, true] {
        let (com, contas, antes, depois) = corre(true, "Watercolor Basic", descarta);
        let (sem, contas_sem, antes_sem, depois_sem) = corre(false, "Watercolor Basic", descarta);
        assert!(
            !antes && !antes_sem,
            "CONTROLO: com um pincel seco, o «+» não aquece nada"
        );
        assert!(
            depois,
            "descarta={descarta}: a troca para a aguada lança o fio"
        );
        assert!(
            depois_sem,
            "descarta={descarta}: na rota do clique, a troca deixa a cache pronta"
        );
        assert_eq!(
            contas,
            if descarta { (0, 1) } else { (1, 0) },
            "descarta={descarta}: colhidos, descartados"
        );
        assert_eq!(contas_sem, (0, 0), "a rota do clique não lança fio");
        assert_eq!(com.len(), sem.len());
        for (i, (a, b)) in com.iter().zip(&sem).enumerate() {
            assert!(!a.is_empty(), "descarta={descarta}: a foto {i} existe");
            assert!(a == b, "descarta={descarta}: a foto {i} é a mesma ao bit");
        }
    }
    // CONTROLO: um pincel seco depois do «+» não aquece (o portão lê `false` quando nada foi lançado).
    let (_, contas, _, depois) = corre(true, "Hard Round", false);
    assert!(!depois, "um pincel seco não aquece o fundo da aguada");
    assert_eq!(contas, (0, 0));
}
