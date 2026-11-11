//! B10.5 — a imagem aplicada pela caixa marcada é a inteira ao bit depois de cada tipo de mudança.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const L: u32 = 96;

fn traco(t: &mut PainterTool, de: [f32; 2], a: [f32; 2]) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(de, PointerPhase::Down));
    t.on_canvas_pointer(p(
        [(de[0] + a[0]) * 0.5, (de[1] + a[1]) * 0.5],
        PointerPhase::Move,
    ));
    t.on_canvas_pointer(p(a, PointerPhase::Move));
    t.on_canvas_pointer(p(a, PointerPhase::Up));
}

/// A leitura incremental é a inteira ao bit; devolve se foi pela caixa.
fn confere(t: &PainterTool, passo: &str) -> bool {
    let antes = t.aplicada.pistas();
    let inc = t.imagem_aplicada().expect("documento").0;
    let pista = t.aplicada.pistas() > antes;
    let ora = t.imagem_aplicada_inteira().expect("documento").0;
    let dif = inc.iter().zip(&ora).filter(|(a, b)| a != b).count();
    assert_eq!(
        (inc.len(), dif),
        (ora.len(), 0),
        "{passo}: {dif} bytes ≠ da inteira"
    );
    pista
}

#[test]
fn incremental_igual_ao_inteiro_ao_bit() {
    use PaintMedia::{Digital, Impasto, Watercolor, WetPaint};
    for meio in [Impasto, Digital, Watercolor, WetPaint] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
        t.set_paint_media(meio);
        t.set_brush_size_px(9.0);
        t.set_substrate_depth(0.8);
        let mut pistas = Vec::new();
        let mut v = |t: &PainterTool, passo: &'static str| pistas.push((passo, confere(t, passo)));
        v(&t, "1.ª");
        traco(&mut t, [10.0, 10.0], [40.0, 30.0]);
        v(&t, "traço");
        traco(&mut t, [60.0, 70.0], [85.0, 50.0]);
        v(&t, "2.º traço");
        assert!(t.undo_last(), "controlo: há o que desfazer");
        v(&t, "Ctrl+Z");
        let b = t
            .add_raster_layer_with_pixels("B", vec![90u8; (L * L * 4) as usize])
            .expect("B");
        v(&t, "camada nova");
        traco(&mut t, [20.0, 80.0], [70.0, 85.0]);
        v(&t, "traço na B");
        traco(&mut t, [30.0, 20.0], [50.0, 60.0]);
        v(&t, "2.º traço na B");
        t.set_layer_opacity(b, 0.5);
        v(&t, "opacidade");
        t.set_layer_visible(b, false);
        v(&t, "visibilidade");
        t.set_layer_visible(b, true);
        t.move_layer_down(b);
        v(&t, "ordem");
        t.set_impasto_light_angle(1.1);
        v(&t, "luz");
        t.set_substrate_depth(0.2);
        v(&t, "relevo do papel");
        t.set_paper_color_rgb8(230, 200, 150);
        v(&t, "cor do papel");
        traco(&mut t, [5.0, 5.0], [90.0, 90.0]);
        v(&t, "traço sobre o papel");
        t.set_source(vec![200u8; (64 * 48 * 4) as usize], 64, 48);
        v(&t, "set_source (outro tamanho)");
        traco(&mut t, [3.0, 3.0], [60.0, 40.0]);
        v(&t, "traço depois do set_source");
        // O 1.º traço numa camada pode subir a revisão (o Impasto publica a camada esculpida); o resto é fixo.
        let caixa = ["2.º traço", "2.º traço na B", "traço sobre o papel"];
        let livre = [
            "traço",
            "Ctrl+Z",
            "traço na B",
            "cor do papel",
            "traço depois do set_source",
        ];
        let erradas: Vec<_> = pistas
            .iter()
            .filter(|(p, pista)| !livre.contains(p) && *pista != caixa.contains(p))
            .collect();
        assert!(
            erradas.is_empty(),
            "{meio:?}: a pista falhou em {erradas:?} (todas: {pistas:?})"
        );
    }
}

/// A luz difere as alturas dos 4 vizinhos: um texel marcado muda a luz da moldura de 1 px à volta dele.
#[test]
fn a_luz_dos_vizinhos_anda_com_o_texel_marcado() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_size_px(9.0);
    traco(&mut t, [10.0, 10.0], [40.0, 30.0]);
    let antes = t.imagem_aplicada().expect("documento").0;
    let ativa = t.layers.active().expect("activa");
    let (x, y) = (25u32, 20u32); // no meio do traço: há tinta para a luz
    let alt = Arc::make_mut(t.heights.get_mut(&ativa).expect("a camada foi esculpida"));
    alt[(y * L + x) as usize] += 4.0;
    t.mark_dirty(Region { x, y, w: 1, h: 1 });
    assert!(
        confere(&t, "texel marcado"),
        "controlo: a pista é a da caixa"
    );
    let depois = t.imagem_aplicada_inteira().expect("documento").0;
    let vizinho = |p: &[u8]| p[(((y * L) + x + 1) * 4) as usize..][..3].to_vec();
    assert_ne!(
        vizinho(&antes),
        vizinho(&depois),
        "controlo: a luz do vizinho mudou"
    );
}

/// Um escritor que bate o `pixel_clock` sem caixa (o contrato de `content_revision`) faz a leitura inteira.
#[test]
fn a_escrita_sem_caixa_faz_a_leitura_inteira() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    let _ = t.imagem_aplicada();
    let mut c = t.canvas_rgba.to_vec();
    c[..4].copy_from_slice(&[10, 20, 30, 255]);
    t.replace_canvas(Arc::new(c));
    let ativa = t.layers.active();
    t.bump_layer_pixels(ativa);
    assert!(
        !confere(&t, "escrita sem caixa"),
        "a leitura tinha de ser inteira"
    );
}

/// A 2.ª leitura depois de um traço curto recompõe só a moldura dele: o contador do gate é a caixa.
#[test]
fn a_leitura_depois_de_um_traco_e_so_a_caixa() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.set_paint_media(PaintMedia::Impasto);
    let _ = t.imagem_aplicada();
    traco(&mut t, [40.0, 40.0], [44.0, 42.0]);
    let r = t.aplicada.sujo_agora().expect("o traço marcou a caixa");
    assert!(
        r.w * r.h < L * L / 4,
        "a caixa {r:?} não é a de um traço curto"
    );
    confere(&t, "traço curto");
}

/// Sonda: a leitura depois de um traço curto numa tela 4K com papel e crista de Impasto, inteira × pela caixa,
/// intercaladas, o MÍNIMO de 4. `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke --
/// --ignored sonda_aplicada_4k --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_aplicada_4k() {
    let (w, h) = (3840u32, 2160u32);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_size_px(h as f32 * 0.3);
    traco(
        &mut t,
        [w as f32 * 0.2, h as f32 * 0.5],
        [w as f32 * 0.8, h as f32 * 0.5],
    );
    t.set_brush_size_px(24.0);
    let _ = t.imagem_aplicada();
    let mut m = [f64::MAX; 2];
    for k in 0..4 {
        let x = 300.0 + 200.0 * k as f32;
        traco(&mut t, [x, 300.0], [x + 60.0, 330.0]);
        for v in [k % 2, 1 - k % 2] {
            let t0 = std::time::Instant::now();
            let _ = if v == 0 {
                t.imagem_aplicada_inteira()
            } else {
                t.imagem_aplicada()
            };
            m[v] = m[v].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    eprintln!(
        "[aplicada 4K] inteira {:.1} ms → pela caixa {:.1} ms",
        m[0], m[1]
    );
}
