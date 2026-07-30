//! **A TINTA QUE O PERCURSO DERRUBA** — o defeito que a §22.6 rotulou de "saturação" e não era.
//!
//! ⚠️ **Filho de [`super`] de propósito:** o oráculo é o mesmo `true_area`, e as fixtures (`screen`,
//! `art`, `BLACK`) são as do binning. Um módulo irmão precisaria de uma segunda cópia da referência,
//! e duas referências para uma pergunta é como elas divergem.

use super::*;
use crate::binning::bin_segments;

/// 📏 **SONDA — a SATURAÇÃO aberta: de que o `1 − exp(−τ)` está feito no pixel que mais erra.**
///
/// ⚠️ **O defeito 3a, e a hipótese que ela existe para testar.** Em `hardness = 1` o perfil é um
/// degrau, então a resposta certa é **pura área** e o `1 − exp(−τ)` deveria valer 1 em todo pixel
/// tocado. Ele não vale: medido, 16-27/255 de falta. E a suspeita não é o `F_MAX` — é que a
/// cobertura está sendo atenuada **DUAS vezes pela mesma informação de borda**: uma no `edge`
/// (a área) e outra no `τ`, cujo arco encurta perto da silhueta pelo mesmo motivo geométrico.
///
/// Ela imprime, no pior pixel de cada cena, o `τ` cru e a saturação que ele produz — sem isso
/// "consertar a saturação" é escolher entre `F_MAX`, `SUB` e o empurrão do `p_eval` no escuro.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_saturation_is_made_of() {
    const N: u32 = 64;
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    let cenas: [(&str, crate::pack::FlipGpuData); 3] = [
        (
            "flanco RETO (a borda mais simples que existe)",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
        ),
        (
            "PONTA aguda",
            art(&[(&[[20.0, 20.0], [70.0, 62.0]], 5.0, false, BLACK)]),
        ),
        (
            "QUINA externa",
            art(&[(
                &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
                9.0,
                false,
                BLACK,
            )]),
        ),
    ];
    // ⚠️ **O CONTROLE: o mesmo `sd` no meio do FLANCO.** Sem ele "τ = 0 num pixel raso" não
    // distingue *a lei erra na borda* de *a lei erra perto de uma PONTA*. Num traço inclinado o
    // `sd` assume valores arbitrários ao longo do flanco (num horizontal ele só cai em ±0,5, que é
    // por que o pior pixel das três cenas caiu numa tampa).
    {
        let g = art(&[(&[[20.0, 20.0], [70.0, 62.0]], 9.0, false, BLACK)]);
        let bins = bin_segments(&g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        println!("  CONTROLE — flanco de um traco inclinado (t estritamente interno):");
        let mut vistos = 0;
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
                let seg = run[0];
                let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
                let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
                let (t, _, _) = closest_on_seg(p, sa, sb);
                if !(0.3..=0.7).contains(&t) {
                    continue;
                }
                let Some(sl) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                    continue;
                };
                if !(0.30..=0.49).contains(&sl.sd) || vistos >= 4 {
                    continue;
                }
                vistos += 1;
                let real = true_area(run, &g, &sc, style.tip, p, N);
                let cover = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                println!(
                    "      sd={:+.3}  area={real:.4}  edge={:.4}  cover={cover:.4}  em ({:.1}, {:.1})",
                    sl.sd,
                    sl.planes.coverage(),
                    p[0],
                    p[1]
                );
            }
        }
    }
    println!(
        "  cena                                     sd      area    edge     tau     sat    cover"
    );
    for (nome, g) in &cenas {
        let bins = bin_segments(g, &sc, 16);
        let (mut pior, mut onde) = (0.0_f32, [0.0_f32, 0.0]);
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
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, g, &sc, style.tip, p, N);
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                let d = (cover - real).abs();
                if d > pior {
                    pior = d;
                    onde = p;
                }
            }
        }
        let ti = bins.tile_of_pixel(onde[0], onde[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
        let s = stroke_silhouette(run, g, &sc, style.tip, onde).expect("tocado");
        let real = true_area(run, g, &sc, style.tip, onde, N);
        // O MESMO empurrão que o `stroke_deposit` aplica — a sonda tem de perguntar onde ele
        // pergunta, senão mede um `τ` que o produto nunca computou.
        let p_eval = if s.sd > -0.5 && s.dist > 1e-6 {
            let f = (s.sd + 0.5) * 0.5 / s.dist;
            [
                onde[0] + (s.near[0] - onde[0]) * f,
                onde[1] + (s.near[1] - onde[1]) * f,
            ]
        } else {
            onde
        };
        let tau = crate::tau::stroke_tau(run, g, &sc, style, p_eval).map_or(0.0, |i| i.tau);
        let cover = stroke_deposit(run, g, &sc, onde).map_or(0.0, |d| d.cover);
        println!(
            "  {nome:40} {:+.3}  {real:.4}  {:.4}  {tau:6.3}  {:.4}  {cover:.4}   em ({:.1}, {:.1})",
            s.sd,
            s.planes.coverage(),
            1.0 - (-tau).exp(),
            onde[0],
            onde[1]
        );
    }
}

/// 📏 **SONDA — quanta tinta o percurso DERRUBA, e onde.**
///
/// ⚠️ **O defeito 3a não era saturação.** O `cover` não fica *baixo* nesses pixels: fica **ZERO**,
/// com `stroke_tau` devolvendo `None`. Mecanismo, aberto no pixel: perto de uma tampa ou de uma
/// junta o suporte do integrando encolhe (o pico está EM CIMA do extremo do segmento) — medido,
/// **0,121 px** de suporte contra um passo de quadratura de **0,35** ⇒ a regra do ponto médio não
/// enxerga nada, e a integral inteira dá 0. No FLANCO, com o MESMO `sd`, `cover == área` a quatro
/// casas: a lei está certa, a grade é que não resolve a janela.
///
/// Esta sonda mede o ALCANCE do defeito — quantos pixels e quanto — porque a cura (resolver a
/// janela em vez do segmento) muda os números do miolo também, e esse preço só se paga contra um
/// número.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_much_ink_the_walk_drops() {
    const N: u32 = 64;
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    let cenas: [(&str, crate::pack::FlipGpuData); 3] = [
        (
            "traco reto (2 tampas)",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
        ),
        (
            "L (2 tampas + 2 juntas)",
            art(&[(
                &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
                9.0,
                false,
                BLACK,
            )]),
        ),
        (
            "zigue-zague (24 juntas)",
            art(&[(
                &(0..24)
                    .map(|k| [20.0 + k as f32 * 2.5, if k % 2 == 0 { 40.0 } else { 52.0 }])
                    .collect::<Vec<_>>(),
                6.0,
                false,
                BLACK,
            )]),
        ),
    ];
    println!(
        "  cena                        px c/ tinta   px DERRUBADOS   pior queda   soma perdida"
    );
    for (nome, g) in &cenas {
        let bins = bin_segments(g, &sc, 16);
        let (mut com_tinta, mut zerados, mut pior, mut soma) = (0_u32, 0_u32, 0.0_f32, 0.0_f32);
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
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, g, &sc, style.tip, p, N);
                if real <= 0.0 {
                    continue;
                }
                com_tinta += 1;
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                if cover <= 0.0 && real > 0.02 {
                    zerados += 1;
                    pior = pior.max(real);
                    soma += real;
                }
            }
        }
        println!(
            "  {nome:28} {com_tinta:6}     {zerados:8}      {:7.2}/255   {:8.2} px",
            pior * 255.0,
            soma
        );
    }
}

/// ⭐ **O DEFEITO 3a, PINADO COM O NÚMERO DELE — e ele não é saturação.**
///
/// ⚠️ **Este gate falha quando o defeito for CORRIGIDO, e isso é o desenho** (o padrão do
/// `the_documented_hardening_is_still_there_and_this_is_its_number` do Painter): sem ele o
/// diagnóstico volta a ser re-derivado do zero, e a §22.6 já rotulou este mesmo número de
/// "saturação" uma vez.
///
/// **O que ele afirma, e o que a medição estabeleceu:**
///
/// 1. **no FLANCO a lei é exata** — mesmo `sd`, `cover == área` a quatro casas. Sem esta metade o
///    número da outra não distingue *a lei erra na borda* de *a lei erra perto de um EXTREMO*;
/// 2. **num pixel cuja cobertura só vem de um extremo de segmento o `cover` é ZERO**, não baixo:
///    `stroke_tau` devolve `None`. Perto de uma tampa ou junta o pico do integrando cai EM CIMA da
///    fronteira do domínio e o suporte encolhe — medido, **0,121 px contra um passo de 0,35** ⇒ a
///    regra do ponto médio não pega amostra nenhuma.
///
/// A cura é resolver a JANELA em vez do segmento (o `SUB` é um piso, e hoje ele está no domínio
/// errado). Ela move todo número de tinta do motor, o port WGSL e os gates de look ⇒ **wave
/// própria**, e o alcance medido diz o preço de adiá-la: **4 pixels de 1180** num traço reto,
/// **13 de 1115** num zigue-zague de 24 juntas.
#[test]
fn the_walk_drops_a_sliver_of_ink_at_caps_and_joints_and_this_is_its_number() {
    let sc = screen(96.0, 96.0);
    let ver = |g: &crate::pack::FlipGpuData, p: [f32; 2]| {
        let bins = bin_segments(g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let real = true_area(run, g, &sc, style.tip, p, 64);
        let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
        (real, cover)
    };
    // (1) O FLANCO, com o centro do pixel FORA: a lei acerta a área.
    //
    // ⚠️ **O flanco tem de ser INCLINADO, e a 1ª versão deste gate não era.** Num traço horizontal
    // o `sd` do flanco cai exatamente em ±0,5 — nunca há pixel meio-coberto ali —, então a fixture
    // media `área = 0` e o gate falhava afirmando o vazio. É também a razão de os três piores
    // pixels das sondas terem caído todos numa TAMPA.
    let inclinado = art(&[(&[[20.0, 20.0], [70.0, 62.0]], 9.0, false, BLACK)]);
    let (real_flanco, cover_flanco) = ver(&inclinado, [47.5, 46.5]);
    assert!(
        real_flanco > 0.0 && (cover_flanco - real_flanco).abs() < 2.0 / 255.0,
        "o flanco deveria ser exato: area {real_flanco:.4} vs cover {cover_flanco:.4}"
    );
    // (2) A TAMPA: mesma ordem de cobertura, e o percurso não deposita nada.
    let reto = art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]);
    let (real_tampa, cover_tampa) = ver(&reto, [87.5, 41.5]);
    assert!(
        real_tampa > 0.05,
        "a fixture perdeu a tampa: area {real_tampa:.4} (o gate mediria o vazio)"
    );
    assert!(
        cover_tampa <= 0.0,
        "⭐ O DEFEITO 3a FOI CORRIGIDO — a tampa agora deposita {cover_tampa:.4} contra area \
         {real_tampa:.4}. Atualize a nota da §22.8 do doc 12 e este gate: o numero mudou."
    );
}
