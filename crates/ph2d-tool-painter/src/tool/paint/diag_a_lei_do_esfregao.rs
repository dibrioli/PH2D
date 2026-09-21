//! SONDA — **a LEI do esfregão numa curva**, dissecada (2026-09-21).
//!
//! Irmã de [`super::diag_auditoria_do_aspecto`], cortada dela pelo tecto de LOC e melhor por
//! isso: *o que o dono VÊ* e *porque a lei o faz* são duas perguntas. Aqui mede-se a segunda —
//! e a resposta foi uma **RECUSA**, não uma cura (ver
//! [`ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`]).
//!
//! A ordem em que elas caíram, que é o que poupa a próxima janela:
//!
//! | # | pergunta | veredito |
//! |---|---|---|
//! | A3 | a costura segue o arco? | é um DEGRAU, não uma costura: o 1.º quarto fica intacto e o resto perde `23 %` |
//! | A4 | é a CURVATURA? | ⛔ **refutada** — a perda CRESCE com o raio |
//! | A5 | arco fixo ou ângulo fixo? | ⛔ nenhum dos dois |
//! | A6 | o campo | `\|disp\|` sobe de `35` a `66 px` e satura; **radial `36 px`** = `1,2` raios |
//! | A7 | a caminhada do perímetro salta? | ⛔ **ilibada** — `226` dabs, passo `6,00`, raio `215,0` |
//!
//! ⚠️ **Uma fixtura NÃO continha o fenómeno e está declarado:** a `StrokeMethod::Line` desta
//! bancada pintou **`0`** texels nos dois lados, logo a (A4) não pôde ler nada da recta — quem
//! respondeu por ela foi a (A8), na irmã.
//!
//! ⇒ O escopo (é a lei ou a fonte?) e o tecto que a curaria vivem em
//! [`super::diag_o_tecto_do_esfregao`].

use super::diag_composite_e_as_formas::cp2;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const L: u32 = 700;

/// (A3) A RÉGUA POR ÂNGULO — a costura do esfregão num círculo fechado.
///
/// ⚠️ Toda régua de alfa desta linha soma a tela inteira, e um arco com outro aspecto não move
/// nenhuma delas. A auditoria de 2026-09-21 disse que a costura **segue o arco** (ela não é uma
/// linha alinhada aos eixos), logo é o ÂNGULO que a tem de indexar.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_costura_do_esfregao -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_costura_do_esfregao() {
    use composite::CompositeOp::{Brush, Smear};
    let (c, r) = ([350.0f32, 350.0], 215.0f32);

    // `tinta(θ)` = a soma do alfa ao longo do raio que sai do centro no ângulo θ, na banda que
    // contém o anel. Um anel uniforme dá uma função CONSTANTE em θ.
    let perfil = |t: &PainterTool| -> Vec<f32> {
        let mut v = Vec::with_capacity(720);
        for i in 0..720 {
            let a = (i as f32) * std::f32::consts::TAU / 720.0;
            let (sn, cs) = a.sin_cos();
            let mut s = 0.0f32;
            // A banda: do interior ao exterior do anel, com folga de 3 raios de pincel.
            let mut rr = r - 90.0;
            while rr <= r + 90.0 {
                let x = (c[0] + cs * rr).round() as i32;
                let y = (c[1] + sn * rr).round() as i32;
                if (0..L as i32).contains(&x) && (0..L as i32).contains(&y) {
                    s += f32::from(t.canvas_rgba[((y as u32 * L + x as u32) * 4 + 3) as usize]);
                }
                rr += 0.5;
            }
            v.push(s / 255.0);
        }
        v
    };

    let cena = |camadas: &[(composite::CompositeOp, f32)]| -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        for (op, s) in camadas {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, *s);
        }
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        for i in 1..=8 {
            let u = i as f32 / 8.0;
            t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        t
    };

    for (nome, camadas) in [
        ("CONTROLO so Brush", vec![(Brush, 1.0)]),
        ("Smear/Brush", vec![(Smear, 1.0), (Brush, 1.0)]),
    ] {
        let t = cena(&camadas);
        let p = perfil(&t);
        let med = p.iter().sum::<f32>() / p.len() as f32;
        let mn = p.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = p.iter().cloned().fold(0.0f32, f32::max);
        // O salto ENTRE VIZINHOS, com o da costura (θ = 0) ao lado do pior de todos os outros.
        let salto = |i: usize| (p[i] - p[(i + 719) % 720]).abs();
        let na_costura = salto(0);
        let (mut pior_outro, mut onde) = (0.0f32, 0usize);
        for i in 1..720 {
            if salto(i) > pior_outro {
                pior_outro = salto(i);
                onde = i;
            }
        }
        println!(
            "{nome:>18} · media {med:8.1} · min {mn:8.1} ({:.0} %) · max {mx:8.1} · \
             salto NA COSTURA {na_costura:7.2} · pior outro {pior_outro:7.2} (θ={:.0}°)",
            100.0 * mn / med,
            onde as f32 / 2.0
        );
        // O perfil em doze pontos, para ver a FORMA e não só os extremos.
        let doze: Vec<String> = (0..12).map(|k| format!("{:.0}", p[k * 60])).collect();
        println!("{:>18}   θ 0..330 em 30°: {}", "", doze.join(" "));
    }
}

/// (A4) O DISCRIMINADOR — a perda de tinta do esfregão é função da CURVATURA?
///
/// A (A3) mediu que numa elipse o esfregão perde `23 %` da tinta em quase todo o anel. Duas
/// famílias de causa explicam isso e pedem curas OPOSTAS:
///   · a FRONTEIRA (a corrente não dá a volta numa figura fechada) ⇒ o defeito é local, no ângulo 0;
///   · a CURVATURA (o deslocamento acumulado é uma soma de CORDAS, que numa curva sai do anel)
///     ⇒ o defeito é global e tem de encolher com o raio.
///
/// A experiência varre o raio e põe a LINHA ao lado — numa recta a curvatura é zero por construção.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_perda_e_a_curvatura -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_perda_e_a_curvatura() {
    use composite::CompositeOp::{Brush, Smear};
    let c = [350.0f32, 350.0];

    let alfa_total = |t: &PainterTool| -> f64 {
        (0..(L * L) as usize)
            .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
            .sum::<f64>()
            / 255.0
    };

    let cena = |camadas: &[(composite::CompositeOp, f32)],
                metodo: ph2d_painter_brush::StrokeMethod,
                r: f32|
     -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = metodo;
        t.paint.composite_enabled = true;
        for (op, s) in camadas {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, *s);
        }
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        for i in 1..=8 {
            let u = i as f32 / 8.0;
            t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        t
    };

    println!("  forma      raio   so Brush   Smear/Brush   guardado   curvatura(1/r)");
    for (nome, metodo, raios) in [
        (
            "elipse",
            ph2d_painter_brush::StrokeMethod::Ellipse,
            vec![40.0f32, 80.0, 160.0, 215.0, 300.0],
        ),
        (
            "linha ",
            ph2d_painter_brush::StrokeMethod::Line,
            vec![215.0f32],
        ),
    ] {
        for r in raios {
            let so = alfa_total(&cena(&[(Brush, 1.0)], metodo, r));
            let sm = alfa_total(&cena(&[(Smear, 1.0), (Brush, 1.0)], metodo, r));
            let k = if nome.starts_with("elipse") {
                format!("{:.4}", 1.0 / r)
            } else {
                "0 (recta)".into()
            };
            println!(
                "  {nome}  {r:6.0}  {so:9.0}  {sm:12.0}   {:6.1} %   {k}",
                100.0 * sm / so
            );
        }
    }
}

/// (A5) ARCO ou ÂNGULO? — onde é que a tinta cai, medido em três raios.
///
/// A (A4) refutou a CURVATURA (a perda cresce com o raio). Sobra o comprimento percorrido, e há
/// duas leituras dele: a transição acontece a um **ARCO** fixo do princípio da figura (um efeito
/// de corrente/cobertura) ou a um **ÂNGULO** fixo (um efeito da geometria da elipse). Elas só se
/// separam variando o raio.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_arco_ou_angulo -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_arco_ou_angulo() {
    use composite::CompositeOp::{Brush, Smear};
    let c = [350.0f32, 350.0];

    for r in [80.0f32, 160.0, 300.0] {
        let mut linha = String::new();
        let mut so_brush = 0.0f32;
        for camadas in [vec![(Brush, 1.0)], vec![(Smear, 1.0), (Brush, 1.0)]] {
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
            t.paint.brush.radius_px = 30.0;
            t.paint.brush.color = [0.75, 0.12, 0.12];
            t.paint.brush.space_attenuation = false;
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
            t.paint.composite_enabled = true;
            for (op, s) in &camadas {
                t.acrescenta_camada(op.to_u8());
                let pos = t.composite_len() - 1;
                t.set_composite_layer_strength(pos, *s);
            }
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            for i in 1..=8 {
                let u = i as f32 / 8.0;
                t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));

            let perfil = |t: &PainterTool, ang: f32| -> f32 {
                let (sn, cs) = ang.sin_cos();
                let mut s = 0.0f32;
                let mut rr = r - 90.0;
                while rr <= r + 90.0 {
                    let x = (c[0] + cs * rr).round() as i32;
                    let y = (c[1] + sn * rr).round() as i32;
                    if (0..L as i32).contains(&x) && (0..L as i32).contains(&y) {
                        s += f32::from(t.canvas_rgba[((y as u32 * L + x as u32) * 4 + 3) as usize]);
                    }
                    rr += 0.5;
                }
                s / 255.0
            };
            if camadas.len() == 1 {
                so_brush = perfil(&t, 1.5); // um ângulo longe da costura, como referência
                continue;
            }
            // Onde é que o perfil cai abaixo de 90 % da referência, pela PRIMEIRA vez?
            let mut queda_ang = f32::NAN;
            for i in 0..720 {
                let a = (i as f32) * std::f32::consts::TAU / 720.0;
                if perfil(&t, a) < 0.9 * so_brush {
                    queda_ang = a;
                    break;
                }
            }
            linha = format!(
                "r {r:5.0} · perimetro {:7.0} · referencia {so_brush:6.1} · \
                 cai a {:5.1}° = ARCO {:7.1} px",
                std::f32::consts::TAU * r,
                queda_ang.to_degrees(),
                queda_ang * r
            );
        }
        println!("  {linha}");
    }
}

/// (A6) O CAMPO DE DESLOCAMENTO, por ângulo — a medição que decide, em vez de a inferir.
///
/// Para cada ângulo: o módulo do deslocamento no anel, e a decomposição dele em **tangencial**
/// (transporte ao longo do anel, que num anel uniforme não devia custar tinta nenhuma) e
/// **RADIAL** (que o tira do anel e é onde a tinta morre).
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_campo_do_esfregao -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_o_campo_do_esfregao() {
    use composite::CompositeOp::{Brush, Smear};
    let c = [350.0f32, 350.0];
    let r = 215.0f32;

    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.color = [0.75, 0.12, 0.12];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.paint.composite_enabled = true;
    for (op, s) in [(Smear, 1.0f32), (Brush, 1.0)] {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, s);
    }
    t.on_canvas_pointer(cp2(c, PointerPhase::Down));
    for i in 1..=8 {
        let u = i as f32 / 8.0;
        t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
    }
    // ⚠️ O pen-up é OBRIGATÓRIO: sob a mão a figura é um RASCUNHO e a pilha não corre — sem ele
    // a sonda apanha ZERO e lê-se como «o esfregão não passa por aqui». A cópia do campo é tirada
    // ANTES de o re-amostrar o consumir, logo ela sobrevive ao fecho da sessão.
    t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
    let disp = super::smear_warp::espia::ultimo();
    assert_eq!(disp.len(), (L * L) as usize, "a sonda não apanhou o campo");

    println!("   θ    |disp|   tangencial    radial   (px; raio do pincel = 30, anel = 60 px)");
    for k in 0..12 {
        let a = (k as f32) * std::f32::consts::TAU / 12.0;
        let (sn, cs) = a.sin_cos();
        // A média sobre a espessura do anel, no ângulo θ.
        let (mut m, mut tg, mut rd, mut n) = (0.0f32, 0.0f32, 0.0f32, 0u32);
        let mut rr = r - 30.0;
        while rr <= r + 30.0 {
            let x = (c[0] + cs * rr).round() as i32;
            let y = (c[1] + sn * rr).round() as i32;
            if (0..L as i32).contains(&x) && (0..L as i32).contains(&y) {
                let d = disp[(y as u32 * L + x as u32) as usize];
                m += d[0].hypot(d[1]);
                // versor radial (cs, sn); tangencial (-sn, cs)
                rd += d[0] * cs + d[1] * sn;
                tg += -d[0] * sn + d[1] * cs;
                n += 1;
            }
            rr += 1.0;
        }
        let n = n.max(1) as f32;
        println!(
            "{:5.0}°  {:7.2}   {:9.2}   {:7.2}",
            a.to_degrees(),
            m / n,
            tg / n,
            rd / n
        );
    }
}

/// (A7) A LISTA DE DABS — o passo entre dabs consecutivos é uniforme, ou a caminhada SALTA?
///
/// Um `|disp|` radial de `36 px` (A6) não sai de uma soma de cordas num raio de `215` (a flecha
/// seria `2 px`). A hipótese que sobra é a **ORDEM da caminhada**: se o perímetro for emitido por
/// simetria (os algoritmos clássicos de elipse cospem 4 pontos de cada vez), dois dabs
/// consecutivos ficam em lados OPOSTOS da figura e o «passo» do esfregão é uma corda gigante.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_caminhada_do_perimetro -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_caminhada_do_perimetro() {
    use composite::CompositeOp::{Brush, Smear};
    let (c, r) = ([350.0f32, 350.0], 215.0f32);
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.color = [0.75, 0.12, 0.12];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.paint.composite_enabled = true;
    for (op, s) in [(Smear, 1.0f32), (Brush, 1.0)] {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, s);
    }
    t.on_canvas_pointer(cp2(c, PointerPhase::Down));
    for i in 1..=8 {
        let u = i as f32 / 8.0;
        t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));

    let dabs = super::smear_warp::espia::ultimos_dabs();
    println!("  dabs entregues ao esfregao: {}", dabs.len());
    let mut passos: Vec<f32> = Vec::new();
    for i in 1..dabs.len() {
        let (a, b) = (dabs[i - 1].0, dabs[i].0);
        passos.push((b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let mut ord = passos.clone();
    ord.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f32| ord[((ord.len() - 1) as f32 * f) as usize];
    println!(
        "  passo entre dabs consecutivos: p05 {:.2} · p50 {:.2} · p95 {:.2} · MAX {:.2}",
        q(0.05),
        q(0.50),
        q(0.95),
        ord[ord.len() - 1]
    );
    println!(
        "  primeiros 12 passos: {:?}",
        &passos[..12.min(passos.len())]
    );
    println!(
        "  arc_len: primeiro {:.2} · ultimo {:.2} · perimetro teorico {:.2}",
        dabs[0].1,
        dabs[dabs.len() - 1].1,
        std::f32::consts::TAU * r
    );
    // O RAIO de cada dab, para ver se a caminhada anda mesmo sobre o anel.
    let raios: Vec<f32> = dabs
        .iter()
        .map(|(p, _)| (p[0] - c[0]).hypot(p[1] - c[1]))
        .collect();
    let rmin = raios.iter().cloned().fold(f32::INFINITY, f32::min);
    let rmax = raios.iter().cloned().fold(0.0f32, f32::max);
    println!("  raio dos centros: min {rmin:.1} · max {rmax:.1} (pedido {r:.0})");
}
