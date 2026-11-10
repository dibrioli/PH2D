//! **A tinta Wet presa ao seu documento e camada** (missão 4K, B02.1 do
//! [BACKLOG](../../../../../../../docs/Painter/missao_4k/BACKLOG.md)): os ids de camada colidem entre documentos e o
//! undo repõe o `next_id`, então a tinta de um desenho não pode acordar noutro. Régua T1-23: `wet_pigmento() == None`
//! depois de 120 tiques em B (⚠ «B intacto ao byte» passaria na rota da cópia, O2-8).

use super::super::adormecida_tests::traco_de_outro_meio;
use super::super::agua_sobre_oleo_tests::{H, W, com_oleo};
use super::*;
use crate::tool::paint::PaintMedia;

const A: u64 = 1;
const B: u64 = 2;

/// O documento B no cache (multi-camada, a activa é a camada 1, o mesmo id da de A): `copia` = os píxeis de A.
fn b_no_cache(t: &mut PainterTool, copia: bool) {
    let px = if copia {
        t.canvas_rgba.as_ref().clone()
    } else {
        vec![255u8; W * H * 4]
    };
    let mut u = PainterTool::default();
    u.bind_document(B, px, W as u32, H as u32);
    u.add_raster_layer("B2").expect("a camada");
    u.set_active_layer(RtLayerId(1));
    let doc = u.tira_documento(B).expect("o documento B");
    t.install_document(B, doc);
}

/// A (camada 1 com tinta viva) → B: `adormece` = sai do Wet antes de trocar.
fn troca_para_b(copia: bool, adormece: bool) -> PainterTool {
    let mut t = com_oleo();
    t.bound_doc = Some(A);
    assert!(t.wet_pigmento().is_some(), "fixture: a tinta viva em A");
    b_no_cache(&mut t, copia);
    if adormece {
        t.set_paint_media(PaintMedia::Digital);
    }
    t.bind_document(B, vec![255u8; W * H * 4], W as u32, H as u32);
    assert_eq!(
        t.layers.active(),
        Some(RtLayerId(1)),
        "fixture: B na camada 1"
    );
    t.set_paint_media(PaintMedia::WetPaint);
    t.ticks_em_blocos(120);
    t
}

/// ⭐ Rotas 1 e 2 (troca de documento e cópia), com a tinta adormecida e com ela viva.
#[test]
fn a_tinta_de_um_documento_nao_acorda_noutro() {
    for copia in [false, true] {
        for adormece in [true, false] {
            let t = troca_para_b(copia, adormece);
            assert!(
                t.wet_pigmento().is_none(),
                "cópia={copia} adormecida={adormece}: a tinta de A acordou em B"
            );
        }
    }
}

/// A tinta viaja com o documento: de volta a A, ela acorda (decisão do dono 06/11).
#[test]
fn de_volta_ao_documento_a_tinta_acorda() {
    let mut t = troca_para_b(false, true);
    t.bind_document(A, vec![255u8; W * H * 4], W as u32, H as u32);
    t.ticks_em_blocos(120); // quem acorda é o quadro (`wet_acorda_sem_esperar`)
    assert!(t.wet_pigmento().is_some(), "a tinta de A morreu na troca");
}

/// ⭐ Rota 3: Ctrl+Z do Add layer devolve o `next_id`; o Add layer seguinte reusa-o e não herda a tinta.
#[test]
fn a_tinta_de_uma_camada_desfeita_nao_acorda_na_que_reusa_o_id() {
    let mut t = com_oleo();
    let nova = t.add_raster_layer("L2").expect("a camada");
    traco_de_outro_meio(&mut t, (40.0, 40.0), (200.0, 40.0));
    t.set_active_layer(RtLayerId(1));
    while t.layers.get(nova).is_some() {
        assert!(t.undo_last(), "o undo do Add layer");
    }
    t.set_paint_media(PaintMedia::Digital);
    let de_novo = t.add_raster_layer("L2").expect("a camada");
    assert_eq!(de_novo, nova, "fixture: o undo repôs o next_id");
    t.set_paint_media(PaintMedia::WetPaint);
    t.ticks_em_blocos(120);
    assert!(
        t.wet_pigmento().is_none(),
        "a tinta da camada desfeita acordou na nova"
    );
}

/// B01: apagar a camada leva a tinta adormecida dela.
#[test]
fn apagar_a_camada_leva_a_tinta_adormecida() {
    let mut t = com_oleo();
    let l2 = t.add_raster_layer("L2").expect("a camada");
    traco_de_outro_meio(&mut t, (40.0, 40.0), (200.0, 40.0));
    t.add_raster_layer("L3").expect("a camada");
    assert!(
        t.camadas_adormecidas().contains(&l2),
        "fixture: a tinta de L2 adormeceu"
    );
    assert!(t.delete_layer(l2));
    let vivas: std::collections::BTreeSet<RtLayerId> = t.layers.all_ids().collect();
    for c in t.camadas_adormecidas() {
        assert!(
            vivas.contains(&c),
            "a tinta adormecida da camada morta {c:?}"
        );
    }
}
