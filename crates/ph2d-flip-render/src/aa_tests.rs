//! **A INTEGRAL DE ÁREA DO PIXEL** — o item 3 do padrão-ouro (doc 12 §22.4), MEDIDO.
//!
//! ⚠️ **A premissa do item estava errada, e este arquivo é a correção.** Eu havia escrito que *"a
//! cobertura é amostrada no CENTRO do pixel, sem AA"*, inferido de `sample_count: 1` + `no_msaa()` em
//! todo o pipeline. Os dois fatos são verdade e **irrelevantes**: o AA aqui é **analítico**, não MSAA
//! — o `stroke_deposit` já computa `edge = clamp(0.5 − sd, 0, 1)`, o filtro-caixa da silhueta em
//! PIXELS, com o `min` sobre as passagens EXATO (o rasterizador precisa de `fwidth` de um `min`, que
//! salta na costura, e faz o AA por-passagem). Inferir a ausência de um mecanismo a partir de um
//! proxy, em vez de grepar o mecanismo, é a armadilha que a memória do repo nomeia.
//!
//! **O que sobra, e é o que se mede aqui:** o filtro-caixa é **1-D** (ao longo da normal da
//! silhueta) e o pixel é **2-D**. Onde duas bordas atravessam o mesmo pixel — uma PONTA, uma QUINA,
//! um CRUZAMENTO — a distância com sinal sozinha não determina a área coberta. É a limitação
//! clássica de todo AA por SDF, e a pergunta é só: **quanto ela vale em 1/255?**
//!
//! O oráculo é a **ÁREA de verdade**: em `hardness = 1` a cobertura é um teste dentro/fora puro, então
//! a fração do pixel coberta é `#{sub-amostras com sd < 0} / N²` — exata, sem quadratura nenhuma no
//! caminho (usar o `τ` aqui contaminaria a referência: o próprio `stroke_deposit` documenta que perto
//! da silhueta o arco pode ser mais curto que meio passo de quadratura).

use super::*;
use crate::binning::{ScreenSpace, bin_segments};

/// A fração REAL do pixel coberta pela silhueta, por supersampling `N×N` do teste dentro/fora.
fn true_area(
    run: &[crate::BinSeg],
    g: &crate::pack::FlipGpuData,
    sc: &ScreenSpace,
    tip: crate::tau::TipShape,
    p: [f32; 2],
    n: u32,
) -> f32 {
    let mut dentro = 0u32;
    for j in 0..n {
        for i in 0..n {
            let q = [
                p[0] - 0.5 + (i as f32 + 0.5) / n as f32,
                p[1] - 0.5 + (j as f32 + 0.5) / n as f32,
            ];
            if stroke_silhouette(run, g, sc, tip, q).is_some_and(|(sd, _, _)| sd < 0.0) {
                dentro += 1;
            }
        }
    }
    f32::from(u16::try_from(dentro).unwrap_or(u16::MAX)) / (n * n) as f32
}

/// 📏 **SONDA — o que o filtro-caixa 1-D erra contra a ÁREA 2-D.**
///
/// Quatro cenas escolhidas pelo fenômeno, e a 1ª é o **CONTROLE** (borda reta longe das tampas, onde
/// o filtro-caixa é exato por construção): ela mede **0,00** nas três colunas, e é isso que dá
/// sentido às outras — sem um controle em zero, um número alto pode ser o instrumento.
///
/// **O resultado, e ele nomeia DOIS defeitos independentes:**
///
/// | cena | `edge` vs área | `cover` vs área | saturação |
/// |---|---|---|---|
/// | reta longe das tampas (CONTROLE) | **0,00** | **0,00** | 0,00 |
/// | a mesma com a TAMPA redonda | 8,74 | 24,90 | **29,99** |
/// | PONTA aguda | 10,97 | 18,93 | **21,55** |
/// | QUINA externa | **63,75** | **63,75** | 0,00 |
///
/// - **em CURVATURA (tampa, ponta) o erro é a SATURAÇÃO, não o AA** — o filtro-caixa erra 9-11 e o
///   produto erra 19-25, com o resto vindo de `1 − exp(−τ)` não chegar a 1;
/// - **na QUINA é 100% geométrico** (saturação 0,00): a união cobre ¾ do pixel e o `min` do SDF diz
///   meio. É a limitação clássica do AA por SDF, e aqui ela vale **¼ da cobertura de um pixel**.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_box_filter_owes_the_pixel_area() {
    const N: u32 = 16; // 256 sub-amostras/pixel
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    // ⚠️ **O CONTROLE precisa de uma janela, e a 1ª versão não tinha.** Na cena da borda reta o pior
    // pixel caiu em `(87,5; 41,5)` — 3,5 px DEPOIS do fim do traço, ou seja na **tampa redonda**: a
    // fixture continha a curvatura que ela existia para excluir, e o "controle" media 24,90. Com a
    // janela `x ∈ [20, 76]` (longe das duas tampas) sobra só o lado RETO, onde o filtro-caixa é exato
    // por construção — e é isso que dá sentido às outras duas linhas.
    /// `(nome, arte, janela em x)` — a janela existe para o CONTROLE poder excluir as tampas.
    type Cena = (&'static str, crate::pack::FlipGpuData, Option<(f32, f32)>);
    let cenas: [Cena; 4] = [
        (
            "borda RETA, longe das tampas (CONTROLE)",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
            Some((20.0, 76.0)),
        ),
        (
            "a mesma cena INCLUINDO a tampa redonda",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
            None,
        ),
        (
            "PONTA aguda",
            art(&[(&[[20.0, 20.0], [70.0, 62.0]], 5.0, false, BLACK)]),
            None,
        ),
        (
            // ⚠️ Em PX o Y é INVERTIDO (`point_px`: mundo (16,16) -> px (16,80)), então esta figura é
            // um **L** cuja quina EXTERNA fica em px (48,16) — e o pior pixel é justamente onde as
            // cápsulas horizontal e vertical se cruzam, com `sd = 0` nas duas.
            "QUINA externa (duas capsulas cruzando)",
            art(&[(
                &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
                9.0,
                false,
                BLACK,
            )]),
            None,
        ),
    ];
    // ⚠️ **A 1ª versão desta sonda media os DOIS erros somados e chamava o total de "o filtro-caixa
    // deve".** São dois mecanismos independentes e só um é geométrico:
    //
    // - **`edge` vs ÁREA** — o filtro-caixa 1-D contra a área 2-D. É a pergunta do item 3.
    // - **`1 − exp(−τ)` vs 1** — a SATURAÇÃO. Em `hardness = 1` o `f_of` devolve `F_MAX = 16` e uma
    //   amostra só deposita `dτ = 16·step/pitch = 4` ⇒ `1 − e⁻⁴ = 0,9817`, **4,7/255 de falta** num
    //   pixel que deveria estar cheio. Num pixel raso perto da borda a janela pega poucas amostras,
    //   e é ali que os dois erros se somam.
    //
    // Sem separar, uma "correção do AA" atacaria a metade errada.
    println!(
        "  cena                                      |edge-area|   |cover-area|   satur.   onde (pior \
         cover)"
    );
    for (nome, g, janela) in &cenas {
        // ⚠️ `hardness = 1` é o regime em que a pergunta EXISTE: ali o perfil é um degrau e a
        // cobertura é pura área. Num pincel macio a rampa é larga e o filtro-caixa quase não é
        // chamado a opinar.
        let bins = bin_segments(g, &sc, 16);
        let (mut pior_edge, mut pior_cover, mut satur, mut onde) =
            (0.0_f32, 0.0_f32, 0.0_f32, [0.0_f32, 0.0]);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                if janela.is_some_and(|(lo, hi)| p[0] < lo || p[0] > hi) {
                    continue;
                }
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, g, &sc, style.tip, p, N);
                // O filtro-caixa SOZINHO: a mesma expressão do `stroke_deposit`, sem o perfil.
                let edge = stroke_silhouette(run, g, &sc, style.tip, p)
                    .map_or(0.0, |(sd, _, _)| (0.5 - sd).clamp(0.0, 1.0));
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                pior_edge = pior_edge.max((edge - real).abs() * 255.0);
                let d = (cover - real).abs() * 255.0;
                if d > pior_cover {
                    pior_cover = d;
                    onde = p;
                    // Quanto do erro é a saturação: o que o perfil tirou do filtro-caixa.
                    satur = (edge - cover).abs() * 255.0;
                }
            }
        }
        println!(
            "  {nome:40}  {pior_edge:6.2}        {pior_cover:6.2}       {satur:6.2}   ({:.1}, \
             {:.1})",
            onde[0], onde[1]
        );
        // ⚠️ **O pior pixel, aberto** — `63,75` é exatamente `0,25 × 255`, e um número redondo assim
        // é ou geometria exata ou fixture. Sem imprimir as partes, os dois são indistinguíveis.
        let ti = bins.tile_of_pixel(onde[0], onde[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
        let (sd, _, _) = stroke_silhouette(run, g, &sc, style.tip, onde).expect("tocado");
        println!(
            "        sd = {sd:+.4}  edge = {:.4}  area = {:.4}  segs no tile = {}",
            (0.5 - sd).clamp(0.0, 1.0),
            true_area(run, g, &sc, style.tip, onde, N),
            run.len()
        );
    }
}
/// 📏 **SONDA — a QUINA aberta, sub-amostra por sub-amostra.**
///
/// ⚠️ **Ela existe porque `63,75/255` é exatamente `0,25 × 255`, e um número redondo assim é ou
/// geometria exata ou fixture** — sem imprimir as partes os dois são indistinguíveis. E foi ela que
/// pegou o meu erro de leitura: **o Y do `point_px` é INVERTIDO** (mundo (16,16) → px (16,80), a
/// cicatriz que o §18.4b do doc 12 já registrou), então o que eu lia como "borda reta vertical" é a
/// **quina externa de um L**, com `sd = 0` nas DUAS cápsulas.
///
/// O que ela mostra: a linha de sub-amostras mais próxima da perna horizontal está **cheia**
/// (`################`) e as de baixo estão pela metade (`########........`) ⇒ a união cobre **¾** do
/// pixel enquanto o `min` do SDF diz que o centro está EM CIMA da fronteira (`edge = 0,5`).
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn dump_the_crossing_pixel() {
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    let g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        9.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let p = [52.5_f32, 20.5];
    let ti = bins.tile_of_pixel(p[0], p[1]).unwrap();
    let run = bins.segs_of(ti);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    println!("  pixel {p:?}  segs no tile = {}", run.len());
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let r = sc.radius_px(pa.width) * (1.0 - t) + sc.radius_px(pb.width) * t;
        println!(
            "    seg {:?}->{:?}  px {sa:?}->{sb:?}  t={t:.3} dist={dist:.3} r={r:.3} sd={:+.3}",
            seg.a,
            seg.b,
            dist - r
        );
    }
    for j in [0_u32, 8, 15] {
        let mut linha = String::new();
        for i in 0..16 {
            let q = [
                p[0] - 0.5 + (i as f32 + 0.5) / 16.0,
                p[1] - 0.5 + (j as f32 + 0.5) / 16.0,
            ];
            let d = stroke_silhouette(run, &g, &sc, style.tip, q).map_or(9.9, |(sd, _, _)| sd);
            linha.push(if d < 0.0 { '#' } else { '.' });
        }
        println!("    j={j:2}: {linha}");
    }
}
