//! Os palcos do Painter em cena: só a Timeline acorda um palco, a troca vai à sprite-alvo, uma tela
//! parada não sobe outra vez, e as ranhuras de palcos que morrem ou renascem devolvem-se.

use super::{PalcosDoPainter, RanhurasDaCena, TrocaDaSprite};
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, StableId, World};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const QUADRO_US: u64 = 16_666;

/// Um traço Wet Paint gravado quadro a quadro sobre a sprite `alvo`, drenando como o editor.
fn gravada(alvo: u64) -> (Vec<u8>, u64) {
    let n = 48;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(5.0);
    let mut rec = MacroRecorder::new(
        RefRect {
            w: n as f32,
            h: n as f32,
        },
        0,
    );
    let trilha = t.macro_abre(&mut rec, alvo).expect("arma no Wet Paint");
    let mut q = 0u64;
    for i in 0..=16u32 {
        if i <= 12 {
            let f = i as f32 / 12.0;
            let phase = match i {
                0 => PointerPhase::Down,
                12 => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: [n as f32 * (0.2 + 0.6 * f), n as f32 * 0.5],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
        drop(t.take_preview_arc());
        t.on_tick(16.0);
        for c in t.macro_passos() {
            rec.push(trilha, q * QUADRO_US, c).expect("tempo crescente");
        }
        q += 1;
    }
    for c in t.macro_para() {
        rec.push(trilha, q * QUADRO_US, c).expect("tempo crescente");
    }
    (ph2d_macro::codec::write(&rec.finish()), q)
}

fn em(q: u64) -> MacroTempo {
    MacroTempo {
        segundos: (q * QUADRO_US) as f64 / 1e6,
    }
}

#[derive(Default)]
struct Placa {
    proxima: u32,
    adquiridas: Vec<u32>,
    substituidas: usize,
    soltas: Vec<u32>,
}

impl RanhurasDaCena for Placa {
    fn adquire(&mut self, _: u32, _: u32, _: &[u8]) -> Result<u32, String> {
        self.proxima += 1;
        self.adquiridas.push(self.proxima);
        Ok(self.proxima)
    }

    fn substitui(&mut self, _: u32, _: u32, _: u32, _: &[u8]) -> Result<(), String> {
        self.substituidas += 1;
        Ok(())
    }

    fn solta(&mut self, id: u32) {
        self.soltas.push(id);
    }
}

fn quadro(p: &mut PalcosDoPainter, w: &mut World, placa: &mut Placa) -> Vec<TrocaDaSprite> {
    p.segue(w);
    p.sobe(placa, &mut ToastQueue::default())
}

#[test]
fn so_a_timeline_acorda_o_palco_e_a_troca_vai_a_sprite_alvo() {
    let (doc, quadros) = gravada(7);
    let mut w = World::new();
    let sprite = w.spawn(StableId(7)).id();
    let dono = w.spawn(Macro {
        doc,
        modo: ModoDaMacro::Fiel,
    })
    .id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());

    assert!(quadro(&mut p, &mut w, &mut placa).is_empty(), "sem relógio não toca");
    assert!(placa.adquiridas.is_empty());

    w.entity_mut(dono).insert(em(quadros / 2));
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!(
        trocas,
        [TrocaDaSprite {
            entity_bits: sprite.to_bits(),
            texture_id: 1
        }]
    );
    // O mesmo instante: a tela não mudou, não sobe outra vez, e a troca continua.
    assert_eq!(quadro(&mut p, &mut w, &mut placa), trocas);
    assert_eq!((placa.adquiridas.len(), placa.substituidas), (1, 0));
    // Para a frente, a pincelada sobe pela MESMA ranhura.
    w.entity_mut(dono).insert(em(quadros));
    assert_eq!(quadro(&mut p, &mut w, &mut placa), trocas);
    assert_eq!((placa.adquiridas.len(), placa.substituidas), (1, 1));
    // A Timeline deixa de conduzir: a sprite volta a ser a dela, e o palco fica em cache.
    w.entity_mut(dono).remove::<MacroTempo>();
    assert!(quadro(&mut p, &mut w, &mut placa).is_empty());
    assert!(placa.soltas.is_empty());
}

#[test]
fn sem_alvo_a_macro_pinta_a_sprite_que_a_tem() {
    let (doc, quadros) = gravada(0);
    let mut w = World::new();
    let dono = w
        .spawn((
            Macro {
                doc,
                modo: ModoDaMacro::Fiel,
            },
            em(quadros),
        ))
        .id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!(trocas.len(), 1);
    assert_eq!(trocas[0].entity_bits, dono.to_bits());
}

#[test]
fn o_palco_que_renasce_ou_morre_devolve_a_ranhura() {
    let (doc, quadros) = gravada(7);
    let mut w = World::new();
    w.spawn(StableId(7));
    let m = Macro {
        doc,
        modo: ModoDaMacro::Fiel,
    };
    let dono = w.spawn((m.clone(), em(quadros))).id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    quadro(&mut p, &mut w, &mut placa);
    assert_eq!(placa.adquiridas, [1]);

    // O `Macro` muda (outra gravação): o palco renasce numa ranhura nova e a velha volta.
    w.entity_mut(dono).insert(m);
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!((placa.adquiridas.as_slice(), placa.soltas.as_slice()), (&[1, 2][..], &[1][..]));
    assert_eq!(trocas[0].texture_id, 2);

    w.despawn(dono);
    assert!(quadro(&mut p, &mut w, &mut placa).is_empty());
    assert_eq!(placa.soltas, [1, 2]);
}
