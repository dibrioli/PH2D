//! SONDA — **o Emboss do Wet Paint tem um campo para ler; o Digital tem?** (estudo, Enio 2026-08-10)
//!
//! A lei do Wet Paint (`ph2d-wet-paint::render`, os DOIS sítios — o *reference look* e o composite
//! vivo) é uma diferença central do campo de MASSA, impressa dentro da COR:
//!
//! ```text
//! m       = sett[i] + susp[i]                       // gramas de pigmento na célula
//! emb     = ((m_r − m_l)·0,5 + (m_d − m_u)·1,0) · 0,008 · emboss_k
//! emb     = clamp(emb, −40, +40)                    // níveis de cor, 0..255
//! c{r,g,b} += v + emb                               // luminância; o ALFA não é tocado
//! ```
//!
//! Duas propriedades decidem se ela transplanta: o campo tem de **existir** e tem de **variar**. Um
//! gradiente é zero num platô, então onde a tinta satura o emboss morre — e é isso que esta sonda
//! mede no Digital, cujo único campo por-pixel é o RGBA.
//!
//! ## O que ela mediu (2026-08-10, pincel raio 20, `k = 40/255`)
//!
//! | pincel | campo | emboss no MIOLO | na BORDA |
//! |---|---|---|---|
//! | disco DURO | alfa | **0,00** (desvio do alfa: 0,00) | 40,00 (saturado) |
//! | macio | alfa | 1,49 | 1,25 |
//! | **macio + GRAIN** | alfa | **3,84** | 0,16 |
//! | duro + GRAIN | alfa | 1,65 | 18,98 |
//! | disco DURO, tela OPACA | luminância | **0,00** | 33,65 |
//! | macio, tela OPACA | luminância | 3,38 (média 0,53) | 1,14 |
//! | **macio + GRAIN, tela OPACA** | luminância | 3,89 (média **2,03**) | 0,15 |
//!
//! **Três leituras, e a terceira é o achado.**
//!
//! 1. ⚠️ **O Digital NÃO tem campo de massa.** Os três planos que o teriam (`heights`/`covers`/`mats`)
//!    são do IMPASTO e nascem vazios — uma camada digital não tem entrada neles. O que sobra é o
//!    RGBA que o composite já escreveu.
//! 2. ✅ **E ele BASTA, sem plano novo.** Numa camada opaca o composite escreveu
//!    `papel·(1−a) + cor·a`, então a luminância carrega o alfa disfarçado; o kernel de 4 taps custa
//!    **0,75 ns/px** (3,13 ms em 2048² SERIAL, e as linhas são disjuntas ⇒ ADR-0109 se aplica).
//!    ⚠️ O preço é que a luminância **não distingue tinta de DESENHO**: uma fronteira entre dois tons
//!    chapados tem o mesmo gradiente que a borda da tinta. O alfa não tem esse defeito, e só existe
//!    onde a camada é transparente.
//! 3. ⚠️ **O DESENHO muda, e não é detalhe.** No Wet Paint a massa varia continuamente dentro de uma
//!    aguada (edge darkening, granulação, espessura), então o emboss desenha estrutura em toda parte.
//!    Tinta digital OPACA **satura**: desvio 0,00 no miolo, emboss exatamente **0,00**. Sem mais nada,
//!    o emboss no Digital é só um **bisel de silhueta** — o *Bevel & Emboss* do Photoshop, que este
//!    repo já desenha para o VETOR (`ph2d-render::fx_stack_field`, `KIND_BEVEL`), não o papel do
//!    Wet Paint.
//! 4. ✅ **Quem devolve o miolo é o GRAIN, que já existe.** Ligar `TextureKind::Noise` no slot Grain
//!    leva o miolo de **0,00 → 3,84** níveis. É a mesma frase do modelo (*"o dente do papel aparece
//!    ATRAVÉS da tinta"*): lá o dente é o `paper[]` do engine, aqui é o slot que o pincel já tem.
//!
//! Rodar: `cargo test -p ph2d-tool-painter probe_what_digital_could_emboss -- --ignored --nocapture`
use super::*;
use ph2d_painter_brush::{Falloff, TextureKind};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn drag(t: &mut PainterTool, y: f32, x0: f32, x1: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 1.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

/// A lei do Wet Paint aplicada a um campo escalar qualquer, devolvida em NÍVEIS de cor.
///
/// `k` é o `0,008 · emboss_k` do modelo: aqui ele fica explícito porque a escala do campo muda (a
/// massa vai a milhares, o alfa para em 255) e o número do modelo não sobrevive à troca de unidade.
fn emboss_at(f: &dyn Fn(i32, i32) -> f64, x: i32, y: i32, k: f64) -> f64 {
    let e = ((f(x + 1, y) - f(x - 1, y)) * 0.5 + (f(x, y + 1) - f(x, y - 1)) * 1.0) * k;
    e.clamp(-40.0, 40.0)
}

struct Stats {
    min: f64,
    max: f64,
    mean: f64,
    sd: f64,
}

fn stats(v: &[f64]) -> Stats {
    let n = v.len().max(1) as f64;
    let mean = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
    Stats {
        min: v.iter().cloned().fold(f64::INFINITY, f64::min),
        max: v.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        mean,
        sd: var.sqrt(),
    }
}

#[test]
#[ignore = "sonda de estudo: imprime a tabela, nao afirma um bar"]
fn probe_what_digital_could_emboss() {
    const SIZE: u32 = 200;
    const Y: u32 = 100;
    // ⚠️ Fonte TRANSPARENTE de propósito: numa camada opaca o alfa é 255 em toda parte por
    // construção, e a pergunta "quanto varia o alfa?" já teria a resposta embutida na fixture.
    let cases: [(&str, Falloff, f32, TextureKind); 4] = [
        (
            "disco DURO (Constant, hardness 1)",
            Falloff::Constant,
            1.0,
            TextureKind::None,
        ),
        (
            "macio (Smooth, hardness 0)",
            Falloff::Smooth,
            0.0,
            TextureKind::None,
        ),
        (
            "macio + GRAIN (Noise)",
            Falloff::Smooth,
            0.0,
            TextureKind::Noise,
        ),
        (
            "duro + GRAIN (Noise)",
            Falloff::Constant,
            1.0,
            TextureKind::Noise,
        ),
    ];
    println!("\n=== campo que o Digital oferece a um emboss (traco horizontal, raio 20) ===");
    for (name, falloff, hardness, tex) in cases {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.radius_px = 20.0;
        t.paint.brush.hardness = hardness;
        t.paint.brush.falloff = falloff;
        t.paint.brush.color = [0.2, 0.1, 0.6];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.texture.kind = tex;
        t.paint.brush.texture.size = [0.05, 0.05];
        drag(&mut t, Y as f32, 40.0, 160.0);

        let px = t.canvas_rgba.clone();
        let at = |x: i32, y: i32| -> [u8; 4] {
            if x < 0 || y < 0 || x >= SIZE as i32 || y >= SIZE as i32 {
                return [0; 4];
            }
            let i = ((y as u32 * SIZE + x as u32) * 4) as usize;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };
        let alpha = |x: i32, y: i32| at(x, y)[3] as f64;
        let luma = |x: i32, y: i32| {
            let c = at(x, y);
            0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64
        };

        // MIOLO: a faixa central do traço, longe das duas bordas e das duas pontas.
        let mut interior = Vec::new();
        let mut interior_grad = Vec::new();
        for y in (Y as i32 - 10)..=(Y as i32 + 10) {
            for x in 70..=130 {
                interior.push(alpha(x, y));
                // gradiente cru (sem a constante), em niveis por pixel
                let g = ((alpha(x + 1, y) - alpha(x - 1, y)) * 0.5
                    + (alpha(x, y + 1) - alpha(x, y - 1)))
                .abs();
                interior_grad.push(g);
            }
        }
        // BORDA: a coluna do meio, atravessando o traco de fora a fora.
        let cross: Vec<f64> = ((Y as i32 - 26)..=(Y as i32 + 26))
            .map(|y| alpha(100, y))
            .collect();
        let edge_grad = ((Y as i32 - 26)..=(Y as i32 + 26))
            .map(|y| {
                ((alpha(101, y) - alpha(99, y)) * 0.5 + (alpha(100, y + 1) - alpha(100, y - 1)))
                    .abs()
            })
            .fold(0.0f64, f64::max);

        let a = stats(&interior);
        let g = stats(&interior_grad);
        // A constante que poe um degrau CHEIO (0 -> 255 num pixel) no teto de +-40 niveis.
        let k_alpha = 40.0 / 255.0;
        let emb_edge = emboss_at(&alpha, 100, Y as i32 - 20, k_alpha).abs();
        let emb_in = ((Y as i32 - 8)..=(Y as i32 + 8))
            .flat_map(|y| (80..=120).map(move |x| (x, y)))
            .map(|(x, y)| emboss_at(&alpha, x, y, k_alpha).abs())
            .fold(0.0f64, f64::max);
        let emb_luma = ((Y as i32 - 8)..=(Y as i32 + 8))
            .flat_map(|y| (80..=120).map(move |x| (x, y)))
            .map(|(x, y)| emboss_at(&luma, x, y, k_alpha).abs())
            .fold(0.0f64, f64::max);

        println!("\n-- {name}");
        println!(
            "   alfa no MIOLO: min {:.0}  max {:.0}  media {:.1}  desvio {:.2}",
            a.min, a.max, a.mean, a.sd
        );
        println!(
            "   |grad(alfa)| miolo: max {:.2}  media {:.3}   |  borda: max {:.1}",
            g.max, g.mean, edge_grad
        );
        println!(
            "   emboss (k = 40/255): MIOLO max {emb_in:.2} niveis  |  BORDA {emb_edge:.2} niveis  |  por LUMINANCIA no miolo {emb_luma:.2}"
        );
        let bar: String = cross
            .iter()
            .map(|&v| match v as u32 {
                0 => '.',
                1..=63 => '-',
                64..=190 => '+',
                _ => '#',
            })
            .collect();
        println!("   corte perpendicular (alfa): {bar}");
    }
    println!();
}

/// ⚠️ **O caso que decide se um emboss no Digital precisa de PLANO NOVO.**
///
/// Numa camada OPACA (o caso comum: papel branco) o alfa é 255 em toda parte por construção — mas o
/// composite escreveu `papel·(1−a) + cor·a`, então **a luminância carrega o alfa disfarçado**. Se ela
/// servir de campo, o emboss custa zero bytes de estado: ele lê o pixel que já existe.
///
/// O preço, e é ele que a sonda tem de nomear: a luminância **não distingue tinta de DESENHO**. Uma
/// fronteira entre dois tons dentro de uma região chapada tem gradiente igual ao da borda da tinta.
#[test]
#[ignore = "sonda de estudo: imprime a tabela, nao afirma um bar"]
fn probe_luminance_as_the_field_on_an_opaque_canvas() {
    const SIZE: u32 = 200;
    const Y: u32 = 100;
    let k = 40.0 / 255.0;
    println!("\n=== tela OPACA branca: a luminancia serve de campo? ===");
    for (name, falloff, hardness, tex) in [
        ("disco DURO", Falloff::Constant, 1.0, TextureKind::None),
        ("macio", Falloff::Smooth, 0.0, TextureKind::None),
        ("macio + GRAIN", Falloff::Smooth, 0.0, TextureKind::Noise),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.radius_px = 20.0;
        t.paint.brush.hardness = hardness;
        t.paint.brush.falloff = falloff;
        t.paint.brush.color = [0.2, 0.1, 0.6];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.texture.kind = tex;
        t.paint.brush.texture.size = [0.05, 0.05];
        drag(&mut t, Y as f32, 40.0, 160.0);
        let px = t.canvas_rgba.clone();
        let luma = |x: i32, y: i32| -> f64 {
            if x < 0 || y < 0 || x >= SIZE as i32 || y >= SIZE as i32 {
                return 255.0;
            }
            let i = ((y as u32 * SIZE + x as u32) * 4) as usize;
            0.2126 * px[i] as f64 + 0.7152 * px[i + 1] as f64 + 0.0722 * px[i + 2] as f64
        };
        let mut inner = Vec::new();
        for y in (Y as i32 - 10)..=(Y as i32 + 10) {
            for x in 70..=130 {
                inner.push(emboss_at(&luma, x, y, k).abs());
            }
        }
        let s = stats(&inner);
        let edge = emboss_at(&luma, 100, Y as i32 - 20, k).abs();
        println!(
            "-- {name}: emboss por LUMINANCIA  miolo max {:.2} media {:.2} niveis  |  borda {edge:.2}",
            s.max, s.mean
        );
    }

    // O CUSTO: quatro leituras + uma soma por pixel, sobre uma tela de 2048x2048.
    let n = 2048usize;
    let field: Vec<f32> = (0..n * n).map(|i| ((i % 251) as f32) * 0.7).collect();
    let f = |x: i32, y: i32| -> f64 {
        field[(y.clamp(0, n as i32 - 1) as usize) * n + x.clamp(0, n as i32 - 1) as usize] as f64
    };
    let t0 = std::time::Instant::now();
    let mut acc = 0.0f64;
    for y in 1..n as i32 - 1 {
        for x in 1..n as i32 - 1 {
            acc += emboss_at(&f, x, y, k);
        }
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "\n-- CUSTO do kernel de 4 taps, 2048x2048 serial: {ms:.2} ms  ({:.2} ns/px)  [acc {acc:.0}]\n",
        ms * 1e6 / (n * n) as f64
    );
}
