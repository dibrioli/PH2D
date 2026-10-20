//! W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md` — **no fim do clipe a ranhura corre a água até à
//! imagem do artista** (decisão do dono 20/10): pela ordem da shell (segue, a marca liga-se, sobe), com a
//! cabeça parada no fim e o editor a correr a água dele ao lado. Filho de `palcos_fim_tests`.

use super::*;
use ph2d_tool_painter::PaintMedia;

const L: u32 = 96;
/// Um quadro de 60 Hz.
const DT: f32 = 1000.0 / 60.0;

/// O maior desvio, em níveis, entre duas ranhuras.
fn desvio(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0)
}

/// ⭐⭐ **No fim do clipe a ranhura anda até ao documento do editor**: sem o artista tocar (a água assenta
/// nos dois) e com uma vertical dele meio segundo depois do Parar (a água da gravação pára nesse toque,
/// W12). O 1.º quadro no fim ainda é a tela do Parar — a água anda, não salta. ⛔ VERMELHO antes da W14:
/// a ranhura ficava na tela do Parar (sonda W14: 93 % da área pintada muda ao sair do clipe).
#[test]
fn no_fim_do_clipe_a_ranhura_corre_a_agua_ate_a_imagem_do_artista() {
    for pinta in [false, true] {
        let mut world = World::new();
        let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
        let (mut t, doc, d) =
            ph2d_tool_painter::macro_track::roteiro::grava_duas_horizontais_ligado_em(
                L,
                e.to_bits(),
                PaintMedia::WetPaint,
            )
            .expect("grava");
        let mut c = Clipe::com(world, e, &doc);
        c.palcos.dt_do_quadro(DT);
        let _ = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
        if pinta {
            (0..30).for_each(|_| tique(&mut t));
            t.set_brush_size_px(8.0);
            #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
            let l = L as f32;
            traco(&mut t, [0.5 * l, 0.1 * l], [0.5 * l, 0.9 * l], 12, true);
        }
        let primeiro = c.quadro_ligado(d, Some(&mut t), &mut nunca);
        let mut r = primeiro.clone();
        let mut parados = 0;
        for _ in 0..40_000 {
            tique(&mut t);
            let agora = c.quadro_ligado(d, Some(&mut t), &mut nunca);
            parados = if agora == r && t.agua_parada() {
                parados + 1
            } else {
                0
            };
            r = agora;
            if parados > 3 {
                break;
            }
        }
        let documento = premul(&t.imagem_aplicada().expect("documento").0);
        assert!(
            desvio(&primeiro, &documento) > 8,
            "controlo (pinta={pinta}): no 1.º quadro do fim a água ainda não correu"
        );
        let dv = desvio(&r, &documento);
        assert!(
            dv <= 1,
            "pinta={pinta}: no fim da cauda a ranhura difere do documento em {dv} níveis"
        );
    }
}
