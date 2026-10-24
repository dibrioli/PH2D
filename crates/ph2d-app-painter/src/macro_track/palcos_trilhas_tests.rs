//! W19 §2.6 — várias imagens no mesmo objecto Macro: cada trilha do Painter toca no seu palco, sobre a
//! imagem que grava; duas trilhas sobre a MESMA imagem mostram a mais recente que já começou.

use super::palcos_tests::Placa;
use super::{Atuais, PalcosDoPainter, TrocaDaSprite};
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, StableId, World};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{MacroDoc, MacroRecorder, MacroTime, Nascimento, RefRect, Track};
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::{PaintMedia, PainterTool};

const QUADRO_US: u64 = 16_666;
const N: u32 = 48;

/// Um traço Wet Paint à altura `y` (fração da tela) sobre `alvo`, a começar no quadro `desde`.
fn trilha(alvo: u64, y: f32, desde: u64) -> (Track, u64) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(5.0);
    let r = RefRect {
        w: N as f32,
        h: N as f32,
    };
    let mut rec = MacroRecorder::new(r, 0);
    let k = t.macro_abre(&mut rec, alvo).expect("arma no Wet Paint");
    let mut q = desde;
    for i in 0..=16u32 {
        if i <= 12 {
            let f = i as f32 / 12.0;
            let phase = match i {
                0 => PointerPhase::Down,
                12 => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: [N as f32 * (0.2 + 0.6 * f), N as f32 * y],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
        drop(t.take_preview_arc());
        t.on_tick(16.0);
        for c in t.macro_passos() {
            rec.push(k, q * QUADRO_US, c).expect("tempo crescente");
        }
        q += 1;
    }
    for c in t.macro_para() {
        rec.push(k, q * QUADRO_US, c).expect("tempo crescente");
    }
    (rec.finish().tracks.remove(0), q)
}

fn nasce(inicio: u64, nome: &str) -> Option<Nascimento> {
    Some(Nascimento {
        inicio: MacroTime(inicio * QUADRO_US),
        nome: nome.to_owned(),
        local: [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
        pai: None,
    })
}

fn doc_de(tracks: Vec<Track>) -> Vec<u8> {
    let mut doc = MacroDoc::new(RefRect {
        w: N as f32,
        h: N as f32,
    });
    doc.tracks = tracks;
    ph2d_macro::codec::write(&doc)
}

/// A tela que a trilha `k` do Painter de `doc` mostra em `q`, tocada à parte, premultiplicada.
fn verdade(doc: &[u8], k: usize, q: u64) -> Vec<u8> {
    let razao = ph2d_tool_painter::macro_track::instantaneos::razao();
    let mut m = MacroEmCena::da_trilha(doc, k, razao)
        .expect("lê-se")
        .expect("trilha");
    m.segue((q * QUADRO_US) as f64 / 1e6).expect("toca");
    let (a, _, _) = m.toma_a_tela().expect("tela");
    let mut tela = a.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut tela);
    tela
}

fn quadro(p: &mut PalcosDoPainter, w: &mut World, placa: &mut Placa, q: u64) -> Vec<TrocaDaSprite> {
    let dono = w
        .query::<(ph2d_ecs::Entity, &Macro)>()
        .iter(w)
        .next()
        .expect("dono")
        .0;
    w.entity_mut(dono).insert(MacroTempo {
        segundos: (q * QUADRO_US) as f64 / 1e6,
    });
    p.segue(w);
    p.sobe(placa, &mut ToastQueue::default(), None, Atuais::default())
}

/// O que a sprite `bits` mostra, das `trocas` (exatamente uma troca para ela).
fn mostra<'a>(trocas: &[TrocaDaSprite], placa: &'a Placa, bits: u64) -> &'a [u8] {
    let delas: Vec<_> = trocas.iter().filter(|t| t.entity_bits == bits).collect();
    assert_eq!(
        delas.len(),
        1,
        "a sprite {bits} tem uma troca só: {trocas:?}"
    );
    &placa.texturas[&delas[0].texture_id].1
}

fn mundo(doc: Vec<u8>, sids: &[u64]) -> (World, Vec<u64>) {
    let mut w = World::new();
    let bits = sids
        .iter()
        .map(|&s| w.spawn(StableId(s)).id().to_bits())
        .collect();
    w.spawn(Macro {
        doc,
        modo: ModoDaMacro::Fiel,
    });
    (w, bits)
}

/// ⛔ VERMELHO antes da W19: só a 1.ª trilha do Painter tocava — a 2.ª imagem não tinha troca.
#[test]
fn duas_imagens_no_mesmo_objeto_macro_tocam_as_duas() {
    let (mut a, fim) = trilha(7, 0.3, 0);
    let (mut b, _) = trilha(8, 0.7, 0);
    (a.nasce, b.nasce) = (nasce(0, "A"), nasce(0, "B"));
    let doc = doc_de(vec![a, b]);
    let (mut w, sprites) = mundo(doc.clone(), &[7, 8]);
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    for q in [fim / 2, fim] {
        let trocas = quadro(&mut p, &mut w, &mut placa, q);
        for (k, &s) in sprites.iter().enumerate() {
            let v = verdade(&doc, k, q);
            assert!(
                mostra(&trocas, &placa, s) == v.as_slice(),
                "quadro {q}: a imagem {k} não mostra a sua pintura"
            );
        }
        assert_ne!(
            verdade(&doc, 0, q),
            verdade(&doc, 1, q),
            "controlo: telas distintas"
        );
    }
}

/// ⛔ VERMELHO antes da W19: a imagem mostrava sempre a 1.ª trilha, mesmo depois de a 2.ª começar.
#[test]
fn a_mesma_imagem_mostra_a_trilha_mais_recente_que_ja_comecou() {
    let (mut a, fim_a) = trilha(7, 0.3, 0);
    let desde = fim_a + 10;
    let (mut b, fim_b) = trilha(7, 0.7, desde);
    a.nasce = nasce(0, "A");
    (b.target, b.continua) = (7, Some(0));
    let doc = doc_de(vec![a, b]);
    let (mut w, sprites) = mundo(doc.clone(), &[7]);
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    // Antes de a 2.ª começar, a 1.ª; depois, a 2.ª; e de volta (scrub) a 1.ª.
    for (q, k) in [
        (fim_a / 2, 0),
        (desde - 1, 0),
        (desde + 6, 1),
        (fim_b, 1),
        (fim_a, 0),
    ] {
        let trocas = quadro(&mut p, &mut w, &mut placa, q);
        assert!(
            mostra(&trocas, &placa, sprites[0]) == verdade(&doc, k, q).as_slice(),
            "quadro {q}: a imagem não mostra a trilha {k}"
        );
        assert_ne!(
            verdade(&doc, 0, q),
            verdade(&doc, 1, q),
            "controlo: quadro {q}"
        );
    }
}
