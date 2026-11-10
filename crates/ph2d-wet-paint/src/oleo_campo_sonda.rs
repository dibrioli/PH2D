//! Sonda do custo do campo de altura (B14.A1, AU-12-3; filho de [`super`]): ms e texels do [`CampoAltura::de_com`]
//! pela janela pedida — o motor cobra a JANELA, por isso a óptica cresce com a caixa que o chamador lhe passa.
//! `bash scripts/cargo-test-narrow.sh ph2d-wet-paint --profile smoke sonda_o_campo_custa_pela_janela -- --ignored --nocapture`

use super::*;

#[test]
#[ignore = "sonda de custo (smoke): ms/texel do campo por janela"]
fn sonda_o_campo_custa_pela_janela() {
    // 1440² células = 2880² px a ratio 2 (a cena do FPS); a tinta cobre a folha toda (a caixa molhada de 6 traços).
    const LADO: usize = 1440;
    let mut g = Grid::new(LADO, LADO);
    for y in 1..=LADO {
        for x in 1..=LADO {
            g.sett[x + y * g.s] = 2000.0 + 37.0 * ((x * 7 + y * 13) % 11) as f32;
        }
    }
    g.sett_rgb.fill([200.0, 110.0, 50.0]);
    let o = OleoReal::default();
    let camadas = [(&g, true)];
    // B14.A1: o campo fresco (planos novos, o caminho de antes) × o rascunho guardado, em série e nas faixas do
    // rayon do app (`oleo_plano::EmFaixas`), intercalados; o MÍNIMO de cada.
    let (mut ras_s, mut ras_f) = (RascunhoCampo::default(), RascunhoCampo::default());
    for lado in [LADO / 8, LADO / 4, LADO / 2, LADO] {
        let c0 = (LADO - lado) / 2 + 1;
        let janela = (c0, c0, c0 + lado - 1, c0 + lado - 1);
        let (mut min, mut texels) = ([f64::INFINITY; 4], 0);
        for volta in 0..20 {
            let t = std::time::Instant::now();
            let c = match volta % 4 {
                0 => CampoAltura::de_com(&EmSerie, &camadas, &o, 1, janela),
                1 => CampoAltura::de_com_em(&mut ras_s, &EmSerie, &camadas, &o, 1, janela),
                2 => CampoAltura::de_com(&Faixas, &camadas, &o, 1, janela),
                _ => CampoAltura::de_com_em(&mut ras_f, &Faixas, &camadas, &o, 1, janela),
            };
            min[volta % 4] = min[volta % 4].min(t.elapsed().as_secs_f64() * 1e3);
            texels = c.w * c.h;
        }
        let ns = |ms: f64| ms * 1e6 / texels as f64;
        eprintln!(
            "janela {lado}² · texels {texels} · série fresco {:.1} ms ({:.1} ns/texel) · rascunho {:.1} ms ({:.1}) \
             · faixas fresco {:.1} ms ({:.1}) · rascunho {:.1} ms ({:.1})",
            min[0],
            ns(min[0]),
            min[1],
            ns(min[1]),
            min[2],
            ns(min[2]),
            min[3],
            ns(min[3])
        );
    }
}

/// As faixas do composite (`ph2d-tool-painter` `oleo_plano::EmFaixas`): uma linha por tarefa do rayon.
pub(super) struct Faixas;

impl Reparte for Faixas {
    fn por_linhas<A: Send, B: Send>(
        &self,
        w: usize,
        a: &mut [A],
        b: &mut [B],
        f: &(dyn Fn(usize, &mut [A], &mut [B]) + Sync),
    ) {
        use rayon::prelude::*;
        a.par_chunks_mut(w)
            .zip(b.par_chunks_mut(w))
            .enumerate()
            .for_each(|(y, (la, lb))| f(y, la, lb));
    }
}
