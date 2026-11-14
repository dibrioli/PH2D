//! ⭐ **O traço molhado fica molhado até à orla** (dono 03/11): o byte de umidade do motor é o chão por onde a água corre.
//! Medição das leis e recusas: [MEDICAO_P1_P4](../../../../../docs/DevOps/MEDICAO_P1_P4_LARGURA_2026-11-03.md).

use super::*;
use crate::sonda_gestos::{W, gesto};
use ph2d_editor_core::tool::RasterEditTool;

/// `(miolo, fração)`: o byte ÷ o cheio do papel, na linha central do traço e em média sobre as células molhadas.
fn medidas_de_umidade(p: &Preset, raio: f32) -> (f64, f64) {
    let h = 6 * raio as u32 + 120;
    let cy = (h / 2) as f32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * h * 4) as usize], W, h);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(7);
    assert!(t.preset_aplicar(&direito(p)).unwrap().is_empty());
    t.set_brush_size_px(raio);
    gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let mut r = (0.0, 0.0);
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        let razao = h as f64 / g.h as f64;
        let j = (f64::from(cy) / razao) as usize + 1;
        let (a, b) = (g.w / 4, 3 * g.w / 4);
        let cobre = |k: usize| {
            let cheio = ph2d_wet_paint::grid::wet_byte_from_paper(f64::from(g.paper[k]));
            (f64::from(g.wet[k]) / f64::from(cheio.max(1))).min(1.0)
        };
        let miolo: f64 = (a..b).map(|x| cobre(x + j * g.s)).sum();
        let (mut soma, mut n) = (0.0, 0u32);
        // A faixa de dreno (linhas 1 e `h`, `apply_boundaries`) nasce selada com `wet = 1` (B12.1): não é papel do traço.
        for k in (2..g.h).flat_map(|y| (a..b).map(move |x| x + y * g.s)) {
            if g.wet[k] > 0 {
                soma += cobre(k);
                n += 1;
            }
        }
        r = (miolo / (b - a) as f64, soma / f64::from(n.max(1)));
    });
    r
}

/// Medido 03/11 (Wet Round, antes do tempo passar). O controlo do miolo: sobrescrever deixava a orla do ÚLTIMO dab no
/// traço inteiro (0,27 a raio 7, 0,44 a 26); o da fração: a rampa pelo raio todo mantinha o miolo (0,93) e secava a
/// faixa de fora, e a mancha encolhia 25 % a raio 26.
#[test]
fn o_traco_molhado_fica_molhado_ate_a_orla() {
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.ends_with("/Wet Round.ph2d-brush.png"))
        .unwrap();
    let p = sem_real_oil(ler(&b).unwrap());
    for raio in [7.0, 26.0] {
        let (miolo, fracao) = medidas_de_umidade(&p, raio);
        println!("[umidade] raio {raio}: miolo {miolo:.3} · fração {fracao:.3}");
        assert!(miolo >= 0.9, "raio {raio}: o miolo secou ({miolo:.3})");
        assert!(
            fracao >= 0.8,
            "raio {raio}: a faixa de fora secou ({fracao:.3})"
        );
    }
}
