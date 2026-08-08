//! **A SONDA QUE DECIDE O DESENHO DA W11** — quanto o envelope LAVA um padrão
//! DIRECIONAL, medido em vez de afirmado.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_directional_wash -- --ignored --nocapture`
//!
//! # A pergunta, e por que ela vem antes do código
//!
//! A lei 2 do módulo (`docs/3D/HANDOFF_CONTINUACAO_…` §3) afirma que um
//! mapeamento **projetado** seria lavado pelo `max` sobre dezenas de dabs. Ela foi
//! escrita por MECANISMO, e nunca teve número. O número decide o produto:
//!
//! * se lava, um alpha direcional só é honesto com **um dab por gesto** (o
//!   `DragRect` do ZBrush, o `Lock+Radius` do Nomad) ⇒ o carimbo é um MÉTODO DE
//!   TRAÇO novo, e a wave é grande;
//! * se não lava, ele é mais um item da lista de padrões e a wave é pequena.
//!
//! ⚠️ **Esta sonda mede um HIPOTÉTICO de propósito** — nenhum mapeamento
//! direcional existe no produto hoje, então ela não pode passar pela porta
//! (`Brush::alpha_weight`, que é função só da POSIÇÃO). Ela reimplementa a lei do
//! envelope em vinte linhas e diz isso em voz alta; o dia em que o produto tiver
//! um frame de dab, o oráculo passa a ser a porta.
//!
//! # O que separa os dois candidatos
//!
//! Não é *"projetado × 3D"* — é **RE-ANCORADO × ABSOLUTO**. Um mapeamento cuja
//! coordenada é medida a partir do CENTRO DO DAB muda de fase a cada dab; um cuja
//! coordenada é absoluta (o objeto, a tela do Blender `Tiled`, o arco do traço)
//! não muda. Os dois podem ser direcionais; só o primeiro re-fasa.

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Falloff, min_spacing, recommended_scale};

/// Onda triangular em `[0, 1]`, média 0,5 — a listra mais simples que é
/// **direcional** (varia num eixo e é constante no outro).
fn stripe(u: f32) -> f32 {
    let t = u.rem_euclid(1.0);
    1.0 - 2.0 * (t - 0.5).abs()
}

/// Onde a coordenada do padrão é medida.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Anchor {
    /// Absoluta — o objeto (o que shipa), a tela do `Tiled`, o arco do traço.
    Absolute,
    /// Re-ancorada no centro do dab — o `View Plane` do Blender, o `04.1`.
    PerDab,
}

/// Os centros de dab de um traço reto pela frente da esfera, no espaçamento REAL.
///
/// A câmera olha por `-Z`, então o traço anda no X de tela **e** de objeto: para
/// a pergunta da lavagem os dois são o mesmo eixo, e o que muda entre os
/// candidatos é só a ÂNCORA.
fn dab_centres(radius: f32, half_len: f32) -> Vec<[f32; 3]> {
    let step = min_spacing(radius);
    let n = (2.0 * half_len / step).floor() as usize;
    (0..=n)
        .map(|i| {
            let x = -half_len + step * i as f32;
            [x, 0.0, (1.0 - x * x).max(0.0).sqrt()]
        })
        .collect()
}

/// O campo que o artista de fato vê: `accum[v] = max sobre os dabs de
/// (falloff × alpha)`, e o mesmo sem alpha, para dividir um pelo outro.
///
/// A razão é **o alpha APARENTE** — quanto do padrão sobreviveu ao envelope. Um
/// padrão de média 0,5 que chega em 0,5 sobreviveu; um que chega perto de 1,0 foi
/// lavado até a envoltória superior dele, que é o defeito que a lei 2 prevê.
fn apparent_alpha(
    mesh: &Mesh,
    radius: f32,
    centres: &[[f32; 3]],
    anchor: Anchor,
    scale: f32,
) -> Vec<f32> {
    let inv_r = 1.0 / radius;
    let mut out = Vec::new();
    for &v in mesh.positions() {
        let (mut with, mut without) = (0.0f32, 0.0f32);
        for &c in centres {
            let d = [v[0] - c[0], v[1] - c[1], v[2] - c[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let f = Falloff::Smooth.weight(dist * inv_r);
            if f <= 0.0 {
                continue;
            }
            let u = match anchor {
                Anchor::Absolute => v[0],
                Anchor::PerDab => v[0] - c[0],
            };
            with = with.max(f * stripe(u / scale));
            without = without.max(f);
        }
        // Só os vértices que o traço de fato tocou entram na estatística: fora
        // da pegada a razão é 0/0, e incluí-los mediria o tamanho da esfera.
        if without > 0.05 {
            out.push(with / without);
        }
    }
    out
}

fn mean_std(xs: &[f32]) -> (f32, f32) {
    let n = xs.len() as f32;
    let m = xs.iter().sum::<f32>() / n;
    let var = xs.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / n;
    (m, var.sqrt())
}

/// **A TABELA QUE DECIDE A WAVE.**
///
/// A coluna `média` é o alpha aparente: o padrão tem média 0,5 por construção, e
/// tudo acima disso é padrão que o `max` comeu. A coluna `desvio` é o CONTRASTE
/// que sobrou — zero quer dizer *o pincel ficou uniformemente mais forte*, que é
/// a frase exata da lei 2.
#[test]
#[ignore = "sonda"]
fn measure_how_much_the_envelope_washes_a_directional_pattern() {
    let mesh = shapes::uv_sphere(160, 240, 1.0);
    let scale = recommended_scale(&mesh);
    let radius = 0.25_f32;

    println!(
        "\n== A LAVAGEM (esfera 160x240, {} verts, escala {scale:.4}, raio {radius}) ==",
        mesh.vert_count()
    );
    println!("A listra tem media 0,500 e contraste cheio quando amostrada UMA vez.\n");
    println!(
        "{:<28} {:>6} {:>9} {:>9}",
        "gesto / ancora", "dabs", "media", "desvio"
    );

    for half_len in [0.0_f32, 0.15, 0.5] {
        let centres = dab_centres(radius, half_len);
        // `half_len == 0` degenera num centro só: é o CONTROLE, o carimbo.
        let centres = if half_len == 0.0 {
            vec![[0.0, 0.0, 1.0]]
        } else {
            centres
        };
        for (name, anchor) in [
            ("absoluta (o que shipa)", Anchor::Absolute),
            ("por-dab (View Plane)", Anchor::PerDab),
        ] {
            let (m, s) = mean_std(&apparent_alpha(&mesh, radius, &centres, anchor, scale));
            let gesture = if centres.len() == 1 {
                "CARIMBO (1 dab)".to_string()
            } else {
                format!("traco {:.2}", half_len * 2.0)
            };
            println!(
                "{:<28} {:>6} {m:>9.3} {s:>9.3}",
                format!("{gesture} · {name}"),
                centres.len()
            );
        }
    }
}

/// **O CONTRASTE CONTRA O COMPRIMENTO DO TRAÇO** — a lavagem é progressiva ou
/// tem joelho?
///
/// ⚠️ Ela decide se *"um traço curto ainda mostra o padrão"* é uma saída de
/// produto (um método que carimba a cada N dabs) ou se o único regime honesto é
/// **um** dab.
#[test]
#[ignore = "sonda"]
fn measure_the_wash_against_stroke_length() {
    let mesh = shapes::uv_sphere(160, 240, 1.0);
    let scale = recommended_scale(&mesh);
    let radius = 0.25_f32;

    println!("\n== A LAVAGEM CONTRA O COMPRIMENTO (ancora POR-DAB) ==");
    println!(
        "{:>6} {:>8} {:>9} {:>9}",
        "dabs", "compr.", "media", "desvio"
    );
    for n in [1usize, 2, 3, 5, 8, 13, 21, 34] {
        let step = min_spacing(radius);
        let centres: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                let x = step * i as f32 - step * (n - 1) as f32 * 0.5;
                [x, 0.0, (1.0 - x * x).max(0.0).sqrt()]
            })
            .collect();
        let (m, s) = mean_std(&apparent_alpha(
            &mesh,
            radius,
            &centres,
            Anchor::PerDab,
            scale,
        ));
        println!("{n:>6} {:>8.3} {m:>9.3} {s:>9.3}", step * (n - 1) as f32);
    }
}
