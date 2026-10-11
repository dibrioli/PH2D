//! ⭐⭐ **O miolo das formas sem degraus nos pontos tangentes** (F5, smoke do dono 11/10: a cena 7, as
//! setas nos pontos onde a sombra nasce — a fronteira acesa/escura do interior dos discos em degraus, e
//! a fatia do centro à borda torta conforme o zoom).
//!
//! Dois mecanismos, os dois na lei do interior (o texel de dentro lê a luz do exterior mais próximo):
//! (1) o ponto de fronteira da busca (a janela 5×5, a semente do JFA de CÉLULA) só é o mais próximo à
//! resolução da grade — salta ao longo da face com a fase dela; a cura é o PÉ (`Config::pe_na_borda`,
//! o raio pela normal SECANTE até assentar). (2) a lâmpada vista do ponto rente marchava raios
//! RASANTES, ora livres ora tapados; a cura é a face cortar a abertura na tangente e a marcha partir
//! afastada (`Config::vis_na_face`). Rodada `medida_luz2d_w5g` (handoff W5g).
//!
//! A régua: a de `as_formas_redondas` (os polígonos da cena 5 contra o gabarito) na oitava de zoom
//! onde o controlo mais falha (1,5), três fases da grade, os anéis do portão e mais três (0,5 · 0,8 ·
//! 0,97 — a passagem que sai dos pontos tangentes).

use ph2d_light2d_gpu::Config;
use vello::kurbo::Affine;

use crate::as_formas_redondas::{ANEIS, anel, cena_redonda, ler_anel, passo_tx, vista_em};
use crate::comum::{fluencia_lida, gpu};
use crate::medida_luz2d_w5d::solver;

const ZOOM: f64 = 1.5;
const FASES: [u32; 3] = [0, 3, 5];
const MAIS_ANEIS: [(&str, f64, f64); 3] = [
    ("dentro 0,5", 0.5, 0.0),
    ("dentro 0,8", 0.8, 0.0),
    ("dentro 0,97", 0.97, 0.0),
];

/// O teto do desvio da passagem luz/sombra ao gabarito, em texels finos — MEDIDO (rodada
/// `medida_luz2d_w5g`, RTX 5060 Ti, zoom 1,5): o produto 1,4; o controlo (a W5f) 4,2.
const TETO_DESVIO: f64 = 2.0;
/// O teto do excesso de variação — MEDIDO na mesma rodada: o produto +0,16; o controlo +0,85.
const TETO_TV: f64 = 0.30;

fn config(gpu: &ph2d_gpu::GpuContext, a: Affine, controlo: bool) -> Config {
    Config {
        pe_na_borda: !controlo,
        vis_na_face: !controlo,
        ..crate::medida_luz2d_w5d::config(gpu, a, crate::medida_luz2d_w5d::VARIANTES[2])
    }
}

/// ⭐⭐ **GATE — a passagem luz/sombra do miolo sai lisa e no lugar**: sem cruzamento a mais, a menos
/// de [`TETO_DESVIO`] do gabarito e sem excesso de variação acima de [`TETO_TV`]; o controlo (o ponto
/// da busca e a marcha do ponto rente) tem de mostrar os degraus na mesma corrida.
#[test]
fn o_miolo_das_formas_nao_sai_em_degraus() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let aneis_todos: Vec<_> = ANEIS.iter().chain(MAIS_ANEIS.iter()).copied().collect();
    let mut falhas = Vec::new();
    let (mut desvio0, mut tv0, mut cruzes0) = (0.0_f64, 0.0_f64, 0usize);
    for ri in 0..2 {
        for k in FASES {
            let a = vista_em(ZOOM, cena_redonda([0.0; 2]).1[ri].centro);
            let tx = {
                let c = config(&gpu, a, false);
                c.grade.texel_m / f64::from(c.fina)
            };
            let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
            let (c, redondos) = cena_redonda(d);
            let r = redondos[ri];
            let ler = |controlo: bool| {
                let mut s = solver(&gpu, a, config(&gpu, a, controlo));
                s.quadro(&gpu, &c);
                fluencia_lida(&gpu, &s)
            };
            let (produto, controlo) = (ler(false), ler(true));
            let aneis: Vec<_> = aneis_todos
                .iter()
                .map(|(nome, f, fora)| (*nome, anel(&c, &r, *f, fora * tx)))
                .collect();
            let max = aneis
                .iter()
                .flat_map(|(_, (_, v))| v.iter().copied())
                .fold(0.0, f64::max);
            for (nome, (pts, gab)) in &aneis {
                let passo = passo_tx(pts, tx);
                let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
                let l = ler_anel(&produto, pts, gab, max, passo, eixo);
                let l0 = ler_anel(&controlo, pts, gab, max, passo, eixo);
                desvio0 = desvio0.max(l0.desvio);
                tv0 = tv0.max(l0.tv);
                cruzes0 = cruzes0.max(l0.cruzes);
                eprintln!(
                    "miolo · {k}/8 · {} · {nome}: ×{} ↔{:.1} tx TV {:+.2} (W5f: ×{} ↔{:.1} TV {:+.2})",
                    r.nome, l.cruzes, l.desvio, l.tv, l0.cruzes, l0.desvio, l0.tv
                );
                if l.errados > 0 || l.cruzes > 0 || l.desvio > TETO_DESVIO || l.tv > TETO_TV {
                    falhas.push(format!(
                        "{k}/8 · {} · {nome}: {} errados · ×{} · ↔{:.1} · TV {:+.2}",
                        r.nome, l.errados, l.cruzes, l.desvio, l.tv
                    ));
                }
            }
        }
    }
    assert!(
        desvio0 > 1.5 * TETO_DESVIO && tv0 > 2.0 * TETO_TV && cruzes0 > 0,
        "o controlo (a W5f) não mostra os degraus (↔{desvio0:.1} · TV {tv0:+.2} · ×{cruzes0}) — a régua não os vê"
    );
    assert!(
        falhas.is_empty(),
        "o miolo das formas sai em degraus: {}",
        falhas.join(" · ")
    );
}
