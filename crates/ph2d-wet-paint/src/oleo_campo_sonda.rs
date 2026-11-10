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
    for _ in 0..3 {
        for lado in [LADO / 8, LADO / 4, LADO / 2, LADO] {
            let c0 = (LADO - lado) / 2 + 1;
            let t = std::time::Instant::now();
            let c = CampoAltura::de_com(
                &EmSerie,
                &camadas,
                &o,
                1,
                (c0, c0, c0 + lado - 1, c0 + lado - 1),
            );
            let ms = t.elapsed().as_secs_f64() * 1e3;
            let texels = c.w * c.h;
            eprintln!(
                "janela {lado}² células · texels do campo {texels} · {ms:.1} ms · {:.1} ns/texel",
                ms * 1e6 / texels as f64
            );
        }
    }
}
