//! Os gates da REAMOSTRAGEM (a Catmull-Rom centrípeta do `flip_smooth`) e a sonda que os
//! mediu. Irmão por LOC cap (HR-18, 600) — segue módulo FILHO via `#[path]`, então
//! `use super::*` alcança os privados do `flip_smooth`.

use super::*;

/// A distância do ponto `p` à POLILINHA que o artista de fato desenhou.
fn dist_to_polyline(p: Vec2, pts: &[Vec2]) -> f32 {
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let l2 = ab.x * ab.x + ab.y * ab.y;
        let t = if l2 < 1e-12 {
            0.0
        } else {
            (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
        };
        let d = Vec2::new(p.x - a.x - t * ab.x, p.y - a.y - t * ab.y);
        best = best.min((d.x * d.x + d.y * d.y).sqrt());
    }
    best
}

/// O maior GIRO (graus) entre segmentos consecutivos da saida -- o detector direto de
/// CUSPIDE: uma cuspide e uma reversao de ~180 graus, e e exatamente a propriedade que a
/// parametrizacao centripeta prova nao existir (Yuksel et al. 2011).
fn max_turn_deg(pts: &[Vec2]) -> f32 {
    let mut worst = 0.0f32;
    for w in pts.windows(3) {
        let a = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
        let b = Vec2::new(w[2].x - w[1].x, w[2].y - w[1].y);
        let la = (a.x * a.x + a.y * a.y).sqrt();
        let lb = (b.x * b.x + b.y * b.y).sqrt();
        if la < 1e-9 || lb < 1e-9 {
            continue;
        }
        let c = ((a.x * b.x + a.y * b.y) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.max(c.acos().to_degrees());
    }
    worst
}

/// O comprimento de arco de uma polilinha.
fn arc(pts: &[Vec2]) -> f32 {
    pts.windows(2)
        .map(|w| {
            let d = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
            (d.x * d.x + d.y * d.y).sqrt()
        })
        .sum()
}

/// As fixtures das duas metades desta wave — o CONTROLE (espaçamento par, passa nos dois
/// mundos) e três graus de espaçamento DESIGUAL com giro GENTIL, que é o que o RDP produz.
/// UMA lista, dois consumidores: o gate e a sonda medem o MESMO traço.
fn fixtures() -> [(&'static str, Vec<Vec2>); 4] {
    [
        (
            "CONTROLE  espacamento par, giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(20.0, 3.0),
                Vec2::new(30.0, 9.0),
            ],
        ),
        (
            "desigual 50:1  giro gentil (dot 0.71)",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(102.0, 2.0),
                Vec2::new(104.0, 20.0),
            ],
        ),
        (
            "desigual 20:1  giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(60.0, 0.0),
                Vec2::new(63.0, 3.0),
                Vec2::new(66.0, 12.0),
            ],
        ),
        (
            "desigual 8:1   giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(40.0, 0.0),
                Vec2::new(45.0, 5.0),
                Vec2::new(50.0, 15.0),
            ],
        ),
    ]
}

/// **Suavizar não pode entortar MAIS do que o traço já entortava** — e nenhuma barriga
/// pode ser maior que o span mais curto em que ela vive.
///
/// As duas metades são PROPRIEDADES do que "suavizar" significa, não modelos da
/// implementação: um alisador redistribui o giro de uma quina por vários vértices, então
/// o giro de CADA junta da saída é no máximo o giro da quina da entrada (com igualdade
/// quando o span é curto demais para subdividir — daí o `<=` e a folga de meio grau, que é
/// ruído de `f32`, não margem escolhida). E o desvio à polilinha que o artista desenhou
/// mede a BARRIGA: uma que passa do span mais curto é tinta num lugar onde ninguém passou.
///
/// ⚠️ **O regime que isto pega não é a quina — é o ESPAÇAMENTO DESIGUAL com giro GENTIL**,
/// que é o que o RDP produz (ele apaga os pontos do trecho reto e mantém os da curva). O
/// CONTROLE é o espaçamento par: ele passa nos dois mundos, então são os desiguais que
/// carregam o gate. Medido, giro máximo da saída — centrípeta vs uniforme:
/// par `4,7° / 4,8°` · `8:1` `14,5° / 46,5°` · `20:1` `19,4° / 135,3°` ·
/// `50:1` **`29,6° / 158,5°`** (uma reversão de 158° É uma cúspide) — e o desvio no `50:1`
/// vai de `1,51` para `5,55` num span de corda `2,83`.
///
/// Mutação que sangra: `knot_step` devolver `1.0` (α = 0, a parametrização UNIFORME).
#[test]
fn the_resampled_curve_never_bends_harder_than_the_polyline_it_smooths() {
    for (name, pts) in fixtures() {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        let turn_in = max_turn_deg(&pts);
        let turn_out = max_turn_deg(&out);
        assert!(
            turn_out <= turn_in + 0.5,
            "{name}: a reamostragem entortou MAIS que a polilinha \
             — giro de saída {turn_out:.1}°, entrada {turn_in:.1}° (cúspide da uniforme)"
        );
        let shortest = pts
            .windows(2)
            .map(|w| {
                let d = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
                (d.x * d.x + d.y * d.y).sqrt()
            })
            .fold(f32::MAX, f32::min);
        let dev = out
            .iter()
            .map(|&p| dist_to_polyline(p, &pts))
            .fold(0.0f32, f32::max);
        assert!(
            dev <= shortest,
            "{name}: a barriga ({dev:.3}) passou do span mais curto ({shortest:.3})"
        );
    }
}

/// SONDA — o comentário do `centered_tangent` afirma que *"uniforme basta porque as
/// quinas são tratadas à parte"*. A afirmação é sobre um número: a Catmull-Rom
/// UNIFORME dispara com ESPAÇAMENTO DESIGUAL, e não só em quina — e espaçamento
/// desigual é o que o RDP PRODUZ (ele apaga os pontos do trecho reto e mantém os
/// da curva). Rode com `--nocapture`.
#[test]
fn measure_the_uniform_parameterisation_on_uneven_spacing() {
    let cases = fixtures();
    println!("  caso                                  | desvio max | inflacao | giro max");
    for (name, pts) in cases {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        let dev = out
            .iter()
            .map(|&p| dist_to_polyline(p, &pts))
            .fold(0.0f32, f32::max);
        println!(
            "  {name:37} | {dev:10.3} | {:8.3} | {:7.1}",
            arc(&out) / arc(&pts),
            max_turn_deg(&out),
        );
    }
}

/// SONDA — **quantos pontos a reamostragem inventa, e quantos deles não dizem nada.**
///
/// O passo é FIXO em arco (`0,4 × largura`), então um span RETO é subdividido com a mesma
/// densidade de uma curva fechada. E span reto longo é o que o RDP DEIXA (ele existe para
/// colapsar o trecho reto a dois pontos) — ou seja, o par RDP→reamostragem apaga pontos de
/// uma reta e depois repõe pontos na MESMA reta. Cada um custa um quad na GPU e, pior, um
/// SLOT na lista de vizinhos (o orçamento da fita local existe porque ela satura).
///
/// "Não diz nada" = o ponto está a menos de `1e-3` da corda do span que ele subdivide, isto
/// é: apagá-lo não moveria a tinta.
#[test]
fn measure_how_many_resampled_points_say_nothing() {
    let mut cases: Vec<(&str, Vec<Vec2>)> = vec![
        (
            "reta longa (o que o RDP deixa)",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 0.0),
                Vec2::new(100.0, 0.0),
            ],
        ),
        (
            "L: duas retas longas + quina",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(60.0, 0.0),
                Vec2::new(60.0, 60.0),
            ],
        ),
    ];
    // Um arco de verdade: 9 pontos num quarto de circulo de raio 40 (todo ponto informa).
    let arc_pts: Vec<Vec2> = (0..=8)
        .map(|i| {
            let a = i as f32 / 8.0 * std::f32::consts::FRAC_PI_2;
            Vec2::new(40.0 * a.cos(), 40.0 * a.sin())
        })
        .collect();
    cases.push(("arco de 90 graus (9 pontos)", arc_pts));
    cases.extend(fixtures());

    // ⚠️ Os pontos de ENTRADA estão na polilinha por definição, então contam como
    // "mudos" sempre — o DESPERDÍCIO é o que sobra depois de descontá-los.
    println!("  caso                                  | entra | sai  | desperdicio");
    for (name, pts) in cases {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        // Um ponto de saida e MUDO se cai sobre a corda do span de entrada mais proximo.
        let mute = out
            .iter()
            .filter(|&&p| dist_to_polyline(p, &pts) < 1e-3)
            .count();
        println!(
            "  {name:37} | {:5} | {:4} | {:11}",
            pts.len(),
            out.len(),
            mute.saturating_sub(pts.len()),
        );
    }
}

/// As sondas e o gate da PRECISÃO da autoria — quão longe o traço guardado fica da mão. Módulo
/// irmão porque é outro assunto: aqui não se mede a reamostragem, mede-se **quanto do gesto
/// sobrevive** aos três estágios (captura · smoothing · simplificação).
#[path = "flip_authoring_precision_tests.rs"]
mod precisao;
