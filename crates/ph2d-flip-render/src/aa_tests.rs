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
            if stroke_silhouette(run, g, sc, tip, q).is_some_and(|s| s.sd < 0.0) {
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
/// **O resultado, com a lei ANTIGA (a rampa `0,5 − sd`) e a de hoje (a ÁREA) lado a lado:**
///
/// | cena | RAMPA vs área | ÁREA vs área | `cover` vs área | saturação |
/// |---|---|---|---|---|
/// | reta longe das tampas (CONTROLE) | **0,00** | **0,00** | 0,00 | 0,00 |
/// | a mesma com a TAMPA redonda | 10,18 | **2,59** | 26,21 | **26,56** |
/// | PONTA aguda | 10,65 | **4,50** | 20,30 | **21,15** |
/// | QUINA externa | **63,75** | **3,20** | 16,68 | **16,89** |
///
/// - a coluna RAMPA é o defeito que a wave da área fechou — **63,75/255 na quina**, que é
///   exatamente ¼ de pixel, e ~10 em qualquer borda diagonal;
/// - o que sobra na coluna ÁREA (2,6-4,5) é a **CURVATURA**: cada passagem entra como o plano
///   TANGENTE, e uma tampa de raio 7 px não é reta dentro do pixel. É o resíduo que o `pixel_area`
///   declara deliberado, e o controle em 0,00 é o que prova que ele não é o instrumento;
/// - `|cover − área|` continua grande **e não é mais AA**: a saturação ao lado explica quase tudo
///   dela. É o defeito **3a**, o outro eixo, e ele encosta no `F_MAX`, que tem racional próprio.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_box_filter_owes_the_pixel_area() {
    // ⚠️ **`N = 16` MENTIA aqui, e a sonda irmã é que mostrou.** O erro do oráculo cresce com
    // quantas sub-células a fronteira atravessa — máximo em diagonal — e a 45° ele sozinho valia
    // 5,4/255. Com 64 os dois oráculos concordam.
    const N: u32 = 64;
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
        "  cena                                      RAMPA(era)  AREA(e')   |cover-area|  satur.  \
         onde (pior cover)"
    );
    for (nome, g, janela) in &cenas {
        // ⚠️ `hardness = 1` é o regime em que a pergunta EXISTE: ali o perfil é um degrau e a
        // cobertura é pura área. Num pincel macio a rampa é larga e o filtro-caixa quase não é
        // chamado a opinar.
        let bins = bin_segments(g, &sc, 16);
        let (mut pior_rampa, mut pior_edge, mut pior_cover, mut satur, mut onde) =
            (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, [0.0_f32, 0.0]);
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
                let s = stroke_silhouette(run, g, &sc, style.tip, p);
                // A LEI ANTIGA (congelada aqui como referência) e a NOVA, lado a lado.
                let rampa = s.as_ref().map_or(0.0, |s| (0.5 - s.sd).clamp(0.0, 1.0));
                let edge = s.as_ref().map_or(0.0, |s| s.planes.coverage());
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                pior_rampa = pior_rampa.max((rampa - real).abs() * 255.0);
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
            "  {nome:40}  {pior_rampa:6.2}    {pior_edge:6.2}      {pior_cover:6.2}     \
             {satur:6.2}   ({:.1}, {:.1})",
            onde[0], onde[1]
        );
        // ⚠️ **O pior pixel, aberto** — `63,75` é exatamente `0,25 × 255`, e um número redondo assim
        // é ou geometria exata ou fixture. Sem imprimir as partes, os dois são indistinguíveis.
        let ti = bins.tile_of_pixel(onde[0], onde[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
        let s = stroke_silhouette(run, g, &sc, style.tip, onde).expect("tocado");
        let sd = s.sd;
        println!(
            "        sd = {sd:+.4}  edge = {:.4}  area = {:.4}  segs no tile = {}",
            s.planes.coverage(),
            true_area(run, g, &sc, style.tip, onde, N),
            run.len()
        );
    }
}
/// 📏 **SONDA — o filtro-caixa contra o ÂNGULO da borda, sem quina nenhuma.**
///
/// ⚠️ **Ela nasceu de uma dúvida sobre a minha própria atribuição.** Eu havia escrito que na cena da
/// PONTA *"o erro é a saturação, não o AA"* — mas o `|edge−area|` dela foi **10,97**, e uma conta de
/// meia linha diz que uma borda RETA a 45° já deve esse tanto: o filtro-caixa é 1-D **ao longo da
/// normal**, e a área de um quadrado unitário cortado por um semi-plano só é `0,5 − sd` quando a
/// borda é **paralela a um eixo**. A 45° a área exata é `(sd√2 + 1)²/2` para `sd ∈ [−√2/2, 0]`, que
/// em `sd = −0,25` vale `0,2090` contra `0,25` da rampa ⇒ **10,4/255**. Se a medição confirmar, o
/// item 3 tem uma TERCEIRA metade, e ela não tem quina nenhuma.
///
/// A fixture varre o ângulo com um traço reto e **só olha o flanco** (`t ∈ [0,25; 0,75]` no
/// segmento) — sem isso a tampa redonda entra e mede curvatura, o erro que já derrubou o controle da
/// sonda irmã.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_box_filter_owes_a_slanted_edge() {
    // ⚠️ **A 1ª rodada usou `N = 16` e o número saiu ACIMA do teto teórico** (15,11 contra os 10,9
    // que um semi-plano puro pode dever a 45°). A causa é o próprio ORÁCULO: a estimativa por
    // sub-amostras erra na proporção de quantas sub-células a fronteira atravessa, e isso é MÁXIMO a
    // 45° — exatamente o ângulo em questão. *Um oráculo cujo erro é função do parâmetro que a sonda
    // VARIA não é um oráculo* (a lição que a antiderivada desta mesma jornada já pagou). A sonda
    // agora imprime DUAS resoluções: se as duas concordarem, o número é do produto.
    for n in [16_u32, 96] {
        println!("  --- oraculo com {n}x{n} sub-amostras ---");
        slanted_edge_sweep(n);
    }
}

fn slanted_edge_sweep(n_sub: u32) {
    let n = n_sub;
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    println!("  angulo   RAMPA(era)  AREA(e')   pior pixel (pela rampa)");
    for deg in [0, 15, 30, 45, 60, 75, 90] {
        let a = (deg as f32).to_radians();
        let (c, s) = (a.cos(), a.sin());
        let (cx, cy) = (48.0_f32, 48.0);
        let g = art(&[(
            &[
                [cx - 40.0 * c, cy - 40.0 * s],
                [cx + 40.0 * c, cy + 40.0 * s],
            ],
            13.0,
            false,
            BLACK,
        )]);
        let bins = bin_segments(&g, &sc, 16);
        let (mut pior, mut pior_area, mut onde) = (0.0_f32, 0.0_f32, [0.0_f32, 0.0]);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                // Só o FLANCO: um pixel cujo ponto mais próximo cai perto de uma ponta está medindo
                // a tampa redonda, e a curvatura dela é outro fenômeno.
                let seg = run[0];
                let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
                let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
                let (t, _, _) = closest_on_seg(p, sa, sb);
                if !(0.25..=0.75).contains(&t) {
                    continue;
                }
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, &g, &sc, style.tip, p, n);
                let s = stroke_silhouette(run, &g, &sc, style.tip, p);
                let rampa = s.as_ref().map_or(0.0, |s| (0.5 - s.sd).clamp(0.0, 1.0));
                let area = s.as_ref().map_or(0.0, |s| s.planes.coverage());
                pior_area = pior_area.max((area - real).abs() * 255.0);
                let d = (rampa - real).abs() * 255.0;
                if d > pior {
                    pior = d;
                    onde = p;
                }
            }
        }
        // ⚠️ **O pior pixel, aberto** — a 1ª rodada mediu 15,11 a 45° e a conta de meia linha diz
        // 10,9 para um semi-plano puro. Um número que passa do próprio limite teórico é ou fixture
        // ou premissa errada, e as duas se distinguem OLHANDO as partes.
        let ti = bins
            .tile_of_pixel(onde[0], onde[1])
            .filter(|t| !bins.segs_of(*t).is_empty());
        if let Some(ti) = ti {
            let run = bins.segs_of(ti);
            let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
            let s = stroke_silhouette(run, &g, &sc, style.tip, onde).expect("tocado");
            let sd = s.sd;
            println!(
                "  {deg:3}   {pior:8.2} {pior_area:8.2}    ({:.1}, {:.1})   sd={sd:+.4} \
                 edge={:.4} area={:.4} segs={}",
                onde[0],
                onde[1],
                s.planes.coverage(),
                true_area(run, &g, &sc, style.tip, onde, n),
                run.len()
            );
        } else {
            println!("  {deg:3}   {pior:8.2} {pior_area:8.2}    (sem pixel)");
        }
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
            let d = stroke_silhouette(run, &g, &sc, style.tip, q).map_or(9.9, |s| s.sd);
            linha.push(if d < 0.0 { '#' } else { '.' });
        }
        println!("    j={j:2}: {linha}");
    }
}

/// ⭐ **O GATE DO PRODUTO — num pixel de quina o depósito passa MAIS do que a borda mais próxima
/// sozinha permite.**
///
/// ⚠️ **Ele existe porque os gates de `pixel_area` vivem na LEI, e uma reversão dentro do
/// `stroke_deposit` passaria por todos eles.** E o oráculo não precisa modelar a saturação: o
/// `cover` é `sat · edge`, com o mesmo `sat` nas duas leis (o `τ` não sabe do `edge`), então num
/// pixel cuja passagem mais próxima tem `sd ≥ 0` a rampa dá `edge ≤ 0,5` e portanto
/// **`cover ≤ 0,5` por aritmética** — nenhum ajuste de constante alcança o outro lado. A área diz
/// ¾ ali, e o que se mede é o depósito atravessando essa barreira.
#[test]
fn at_a_corner_the_deposit_passes_what_the_nearest_edge_alone_allows() {
    let sc = screen(96.0, 96.0);
    // O mesmo L da sonda: em PX o Y é invertido, então a quina EXTERNA fica em (48, 16) e o pixel
    // (52,5; 20,5) tem `sd = 0` nas DUAS cápsulas.
    let g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        9.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let p = [52.5_f32, 20.5];
    let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro da tela");
    let run = bins.segs_of(ti);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    let s = stroke_silhouette(run, &g, &sc, style.tip, p).expect("tocado");
    // A premissa da fixture, declarada: se a quina deixar de ser uma quina, o gate não mede o que
    // diz medir. (A fixture que não contém o fenômeno é a falha recorrente desta jornada.)
    assert!(
        s.sd >= -0.01,
        "a fixture perdeu a quina: sd = {} (a rampa nem estaria capada em 0,5)",
        s.sd
    );
    let area = true_area(run, &g, &sc, style.tip, p, 64);
    assert!(
        (area - 0.75).abs() < 0.02,
        "a fixture mudou: a uniao cobre {area}, nao 3/4"
    );
    let cover = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
    assert!(
        cover > 0.55,
        "o deposito na quina deu {cover} — a rampa nao passa de 0,5 ali, entao isto e' a lei antiga \
         de volta"
    );
}

/// 📏 **SONDA — o que a 3ª e a 4ª vaga de plano COMPRAM.**
///
/// ⚠️ **Ela existe porque o tamanho do conjunto é um TETO, e um teto entra medido** (§0.0). No
/// device o custo é dominado pelo array de recorte — 4 vagas / 8-gono custam **4,71 ms** por frame
/// a 200 traços contra **3,50** com 2 vagas / 6-gono, a MESMA lei —, então a pergunta não é
/// "quantas cabem" e sim "quantas o desenho usa".
///
/// A cena é um ZIGUE-ZAGUE de passo sub-pixel: o único jeito de um traço só atravessar o mesmo
/// pixel com três bordas. Imprime, por número de vagas, o pior desvio contra a área supersampleada.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_third_and_fourth_plane_buy() {
    let sc = screen(96.0, 96.0);
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for k in 0..24 {
        let x = 20.0 + k as f32 * 2.0;
        pts.push([x, if k % 2 == 0 { 40.0 } else { 44.0 }]);
    }
    let g = art(&[(&pts, 3.0, false, BLACK)]);
    let bins = bin_segments(&g, &sc, 16);
    let (mut pior, mut onde, mut n_planos) = (0.0_f32, [0.0_f32, 0.0], 0_usize);
    for y in 0..96_u32 {
        for x in 0..96_u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                continue;
            };
            let run = bins.segs_of(ti);
            if run.is_empty() {
                continue;
            }
            let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
            let Some(s) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                continue;
            };
            let real = true_area(run, &g, &sc, style.tip, p, 64);
            let d = (s.planes.coverage() - real).abs() * 255.0;
            if d > pior {
                pior = d;
                onde = p;
                n_planos = s.planes.len();
            }
        }
    }
    println!(
        "  MAX_PLANES = {}: pior |area-real| = {pior:.2}/255 em ({:.1}, {:.1}), {n_planos} planos \
         em alcance ali",
        crate::pixel_area::MAX_PLANES,
        onde[0],
        onde[1]
    );
}
