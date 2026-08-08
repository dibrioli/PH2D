//! Os dois gates que o [ADR-0156] declara **nascendo VERMELHOS** — a lei do Reshape, não a velocidade
//! dele.
//!
//! ## O oráculo é GEOMÉTRICO, e isso é o ponto
//!
//! O primeiro gate não afirma um número nosso: afirma que **uma rotação é uma rotação**. Girar um ponto
//! de raio `r` em torno do centro não pode deslocá-lo mais que **`2r`** — o diâmetro, alcançado a 180°,
//! e nenhuma composição de rotações passa disso porque rotações formam um GRUPO. Um oráculo que
//! comparasse o mapa novo com o mapa antigo seria *razão entre dois doentes*; este não pode ser satisfeito
//! por um bug que ande junto nos dois lados.
//!
//! ## O que o produto faz hoje, reproduzido termo a termo
//!
//! [`super::apply`] avalia `field.at([dx, dy])` no pixel de DESTINO — fixo — e soma: `d += a`. Logo o
//! campo dele é `Σ_k f_k(p)`, e é exatamente isso que [`summed_at`] computa. Não é uma paráfrase: é a
//! mesma aritmética, na mesma ordem, com os mesmos dabs.
//!
//! Somar as cordas `R(θ)v − v` N vezes dá `N·(corda)` — uma **reta tangente**, que cresce sem limite.
//! Compor dá `R(Nθ)`, que é limitado. Somar É composição exata **para translação e para mais nada**, e é
//! por isso que só o **Push** parecia bom.
//!
//! ⚠️ **A mutação que prova cada gate é a mesma:** troque [`compose_at`] por [`summed_at`] — a lei que
//! shipa hoje. Os dois sangram, e as mensagens imprimem o número do produto ao lado do teto geométrico.
//!
//! [ADR-0156]: ../../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::apply::bilinear_clamped;
use super::field::{DabField, DeformMode, compose_at};

const SIDE: u32 = 128;
const CENTRE: [f32; 2] = [64.0, 64.0];
/// Raio do PINCEL. O ponto sondado fica bem dentro dele, onde o falloff é forte — é ali que a divergência
/// aparece, e é ali que a arte do artista está.
const BRUSH_R: f32 = 100.0;
/// O raio SONDADO: a distância do centro em que o teto `2r` é afirmado.
const PROBE_R: f32 = 30.0;

/// A lista de dabs de um Twist mantido no lugar — o gesto do report do Enio (*"Twist nas imagens: veja
/// linhas sumindo"*), que é como um artista de fato usa a ferramenta: ele insiste.
fn twist_dabs(n: usize) -> Vec<DabField> {
    (0..n)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                CENTRE,
                BRUSH_R,
                [0.0, 0.0],
                [0.0, 0.0],
                1.0, // strength no máximo — o regime que o artista alcança com o slider
                0.8, // pressão default
                0.0,
                0.0,
                k as u64 + 1,
            )
        })
        .collect()
}

/// **A lei que SHIPA hoje**, isolada: `Σ_k f_k(p)`, cada dab avaliado no pixel de destino fixo. Não é uma
/// segunda porta — é a rota de ablação que dá sentido às mutações, e ela existe só sob `cfg(test)`.
fn summed_at(dabs: &[DabField], p: [f32; 2]) -> [f32; 2] {
    let mut d = [0.0_f32, 0.0];
    for f in dabs {
        let v = f.at(p);
        d[0] += v[0];
        d[1] += v[1];
    }
    d
}

fn len(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// **Gate 1 — um Twist é uma ROTAÇÃO, não um cisalhamento divergente.**
///
/// Nasce VERMELHO contra a lei de hoje: a mensagem imprime os dois números lado a lado, e o do produto
/// passa do teto por múltiplos.
#[test]
fn a_twist_is_a_rotation_not_a_runaway_shear() {
    let ceiling = 2.0 * PROBE_R;
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    for n in [1usize, 5, 20, 60, 200] {
        let dabs = twist_dabs(n);
        let composed = len(compose_at(&dabs, probe));
        assert!(
            composed <= ceiling,
            "com {n} dabs a composição deslocou {composed:.2} px num raio de {PROBE_R} — uma rotação \
             não passa de {ceiling:.0} px (o diâmetro, a 180°). A soma que shipa hoje dá {:.2} px.",
            len(summed_at(&dabs, probe))
        );
    }
}

/// **E o teto não é alcançado por acidente de escala.** Um gate que só dissesse `<= 2r` ficaria verde com
/// um campo IDENTICAMENTE ZERO — a ferramenta desligada. Este exige que ela ainda DEFORME.
#[test]
fn the_bounded_twist_still_turns_the_picture() {
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    let d = compose_at(&twist_dabs(60), probe);
    assert!(
        len(d) > 1.0,
        "60 dabs de Twist têm de girar algo visível; deslocou {:.3} px",
        len(d)
    );
}

/// **A tabela do [ADR-0156], saindo da MESMA fixture dos gates.**
///
/// ⚠️ Ela existe porque o ADR nasceu citando números de uma sonda exploratória com fixture PRÓPRIA
/// (158,55 px · 3,4%), e um fato medido duas vezes com duas fixtures é um fato que ninguém consegue
/// reproduzir depois. Aqui o gate e a tabela partilham `twist_dabs`, então **concordam por construção**.
///
/// Rodar: `cargo test -p ph2d-tool-painter --lib warp::compose -- --ignored --nocapture`
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_the_divergence_of_the_sum() {
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    let src = line_canvas();
    let before = ink(&src) as f64;
    println!(
        "\n=== o preço da SOMA (pincel r={BRUSH_R}, sonda r={PROBE_R}, teto geométrico {:.0} px) ===",
        2.0 * PROBE_R
    );
    println!(
        "{:>6} {:>14} {:>14} {:>12} {:>12}",
        "dabs", "|D| soma", "|D| composto", "tinta soma", "tinta comp."
    );
    for n in [1usize, 5, 20, 60, 200] {
        let dabs = twist_dabs(n);
        let s = ink(&warp_with(&src, |p| summed_at(&dabs, p))) as f64 / before * 100.0;
        let c = ink(&warp_with(&src, |p| compose_at(&dabs, p))) as f64 / before * 100.0;
        println!(
            "{n:>6} {:>14.2} {:>14.2} {:>11.1}% {:>11.1}%",
            len(summed_at(&dabs, probe)),
            len(compose_at(&dabs, probe)),
            s,
            c
        );
    }
}

/// Uma tela branca com uma linha preta HORIZONTAL de 3 px pelo meio — a figura da foto do Enio.
fn line_canvas() -> Vec<u8> {
    let mut px = vec![255u8; (SIDE * SIDE) as usize * 4];
    for y in 63..66 {
        for x in 0..SIDE {
            let b = ((y * SIDE + x) * 4) as usize;
            px[b] = 0;
            px[b + 1] = 0;
            px[b + 2] = 0;
        }
    }
    px
}

/// Quantos texels ainda carregam tinta escura — a régua de *"as linhas somem"*.
fn ink(px: &[u8]) -> usize {
    px.chunks_exact(4).filter(|c| c[0] < 128).count()
}

/// O gather REAL do produto ([`bilinear_clamped`]), dirigido por um campo dado. Uma reamostragem por
/// texel, como o `apply` faz — reescrevê-lo aqui seria a segunda resposta a *"como um warp lê a fonte?"*.
fn warp_with(src: &[u8], field: impl Fn([f32; 2]) -> [f32; 2]) -> Vec<u8> {
    let mut out = vec![0u8; src.len()];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let p = [x as f32, y as f32];
            let d = field(p);
            let c = bilinear_clamped(src, SIDE, SIDE, p[0] - d[0], p[1] - d[1]);
            let b = ((y * SIDE + x) * 4) as usize;
            out[b..b + 4].copy_from_slice(&c);
        }
    }
    out
}

/// **Gate 2 — a linha fina SOBREVIVE ao swirl.**
///
/// Uma rotação move tinta; ela não a apaga. Sob a soma, cada destino busca a fonte longe demais, a linha
/// é esticada até virar fio translúcido e some no branco — os arcos finos da foto. Nasce VERMELHO.
#[test]
fn the_thin_line_survives_a_twist() {
    let src = line_canvas();
    let before = ink(&src);
    let dabs = twist_dabs(60);

    let kept = ink(&warp_with(&src, |p| compose_at(&dabs, p)));
    let pct = kept as f64 / before as f64 * 100.0;
    let summed_pct = ink(&warp_with(&src, |p| summed_at(&dabs, p))) as f64 / before as f64 * 100.0;
    assert!(
        pct >= 80.0,
        "a linha tem de sobreviver ao Twist: restaram {pct:.1}% da tinta ({kept} de {before} texels). \
         A soma que shipa hoje deixa {summed_pct:.1}%."
    );
}
