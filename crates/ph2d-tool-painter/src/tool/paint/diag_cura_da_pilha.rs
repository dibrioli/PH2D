//! MEDIÇÃO — **o que a CURA da pilha entregou** (2026-09-21).
//!
//! Corte por RESPONSABILIDADE do irmão [`super::diag_auditoria_da_pilha`] (que bateu `1 001`
//! linhas contra o tecto de `700`): lá mora o DIAGNÓSTICO — o preço do replay, o carimbo, e as
//! réguas que provaram que as fixturas estavam no ponto neutro. Aqui mora o que a acumulação
//! ([`super::composite_acumulado`]) custa e o que ela muda.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_cura -- --ignored --nocapture --test-threads=1
//! ```

use super::diag_auditoria_da_pilha::{
    SIZE, caminho_rabisco, caminho_rapido, corre, diff_visivel, grava_ppm, pilha_do_dono,
    tela_com_arte,
};
use super::*;

/// ⭐⭐⭐ **AS DUAS ROTAS** — a acumulação (omissão) contra o REPLAY que ela substituiu.
///
/// Exacta em Brush e Erase por álgebra; **divergente no Blur por desenho** (uma passagem contra
/// `N`). Esta sonda é quem põe o número na divergência declarada.
#[test]
#[ignore = "sonda: corre à mão, em --release"]
fn diag_cura_as_duas_rotas() {
    println!("\n  AS DUAS ROTAS — acumulação contra replay (o que se VÊ, sobre branco)\n");
    println!("  caminho  | pilha                          | pior |  médio | px visíveis");
    println!("  ---------+--------------------------------+------+--------+------------");
    let casos: [(&str, fn(&mut PainterTool)); 5] = [
        ("dois Brushes (deve ser EXACTO)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 0.5,
                color: Some([0.0, 0.0, 1.0]),
                ..CompositeLayer::default()
            };
        }),
        ("Erase Tudo sobre Brush (EXACTO)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Erase,
                strength: 0.7,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("Smear sob Brush (EXACTO)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Smear,
                strength: 1.0,
                ..CompositeLayer::default()
            };
        }),
        ("Blur sobre Brush (DIVERGE)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Blur,
                strength: 1.0,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("a pilha do DONO", pilha_do_dono),
    ];
    for (nome_c, pts) in [
        ("rabisco", caminho_rabisco(120)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        for (nome, monta) in casos {
            let img = |replay: bool| {
                let mut t = tela_com_arte(40.0, 0);
                t.paint.pilha_por_replay = replay;
                monta(&mut t);
                corre(&mut t, &pts);
                (*t.canvas_rgba).clone()
            };
            let (pior, medio, nz, _) = diff_visivel(&img(true), &img(false));
            println!("  {nome_c:8} | {nome:30} | {pior:4} | {medio:6.3} | {nz:11}");
        }
    }
}

/// A nitidez média na faixa pintada — o Laplaciano absoluto. **Menor = mais borrado.**
fn nitidez(rgba: &[u8], r: Region) -> f64 {
    let stride = SIZE as usize * 4;
    let lum = |i: usize| {
        let a = f64::from(rgba[i + 3]) / 255.0;
        (0.299 * f64::from(rgba[i])
            + 0.587 * f64::from(rgba[i + 1])
            + 0.114 * f64::from(rgba[i + 2]))
        .mul_add(a, 255.0 * (1.0 - a))
    };
    let (mut s, mut n) = (0f64, 0f64);
    for y in (r.y as usize + 1)..(r.y + r.h) as usize - 1 {
        for x in (r.x as usize + 1)..(r.x + r.w) as usize - 1 {
            let i = y * stride + x * 4;
            let l = 4.0f64.mul_add(
                -lum(i),
                lum(i - 4) + lum(i + 4) + lum(i - stride) + lum(i + stride),
            );
            s += l.abs();
            n += 1.0;
        }
    }
    s / n
}

/// ⭐⭐⭐ **QUANTO o Blur perdeu** — a divergência declarada, com direcção e tamanho.
#[test]
#[ignore = "sonda: corre à mão, em --release"]
fn diag_cura_quanto_o_blur_perdeu() {
    let destino = std::env::var("PH2D_AUDIT_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    let r = Region {
        x: 300,
        y: 330,
        w: 460,
        h: 400,
    };
    println!("\n  QUANTO O BLUR PERDEU  (nitidez: MENOR = mais borrado)\n");
    println!("  força |   replay | acumulação |  razão");
    println!("  ------+----------+------------+--------");
    for forca in [0.362f32, 1.0] {
        let img = |replay: bool| {
            let mut t = tela_com_arte(40.0, 0);
            t.paint.pilha_por_replay = replay;
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Blur,
                strength: forca,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            corre(&mut t, &caminho_rabisco(120));
            (*t.canvas_rgba).clone()
        };
        let (a, b) = (img(true), img(false));
        let (na, nb) = (nitidez(&a, r), nitidez(&b, r));
        println!("  {forca:5.3} | {na:8.3} | {nb:10.3} | {:6.2}×", nb / na);
        if (forca - 1.0).abs() < 1e-6 {
            grava_ppm(&format!("{destino}/blur_replay.ppm"), &a);
            grava_ppm(&format!("{destino}/blur_acumulado.ppm"), &b);
        }
    }
}

/// **A LARGURA DA BORDA** — quantos pixels de transição há entre a tinta e o fundo.
///
/// ⚠️ É a régua que a nitidez MÉDIA não é: o borrão espalha na BORDA, e uma média sobre a região
/// inteira é dominada pelo miolo chapado.
fn largura_da_borda(rgba: &[u8], y: u32) -> u32 {
    let stride = SIZE as usize * 4;
    let mut n = 0;
    for x in 0..SIZE as usize {
        let i = y as usize * stride + x * 4;
        let (r, g, b, a) = (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]);
        if a < 8 {
            continue;
        }
        // Nem vermelho puro nem azul puro = pixel de transição.
        let puro_vermelho = r > 200 && g < 60 && b < 60;
        let puro_azul = b > 150 && r < 80;
        if !puro_vermelho && !puro_azul {
            n += 1;
        }
    }
    n
}

/// ⭐⭐⭐ **O ESPALHAMENTO DO BORRÃO** — a régua da BORDA, replay contra acumulação.
#[test]
#[ignore = "sonda: corre à mão, em --release"]
fn diag_cura_o_espalhamento() {
    println!("\n  O ESPALHAMENTO DO BORRÃO — pixels de transição na linha y\n");
    println!("      y |  replay | acumulação |  razão");
    println!("  ------+---------+------------+--------");
    let img = |replay: bool| {
        let mut t = tela_com_arte(40.0, 0);
        t.paint.pilha_por_replay = replay;
        t.paint.composite[0] = CompositeLayer {
            op: CompositeOp::Blur,
            strength: 1.0,
            ..CompositeLayer::default()
        };
        t.paint.composite[1] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            color: Some([1.0, 0.0, 0.0]),
            ..CompositeLayer::default()
        };
        corre(&mut t, &caminho_rabisco(120));
        (*t.canvas_rgba).clone()
    };
    let (a, b) = (img(true), img(false));
    let (mut sa, mut sb) = (0u32, 0u32);
    for y in [420u32, 470, 520, 570, 620] {
        let (la, lb) = (largura_da_borda(&a, y), largura_da_borda(&b, y));
        sa += la;
        sb += lb;
        println!(
            "  {y:5} | {la:7} | {lb:10} | {:6.2}×",
            f64::from(lb) / f64::from(la.max(1))
        );
    }
    println!(
        "  TOTAL | {sa:7} | {sb:10} | {:6.2}×",
        f64::from(sb) / f64::from(sa.max(1))
    );
}

/// ⭐⭐⭐ **O RECTÂNGULO MORREU?** — a região contra a composição GLOBAL, nas DUAS rotas.
#[test]
#[ignore = "sonda: corre à mão, em --release"]
fn diag_cura_o_rectangulo_morreu() {
    println!("\n  O LIMITE REGIONAL CUSTA O QUÊ À IMAGEM? (contra a composição GLOBAL)\n");
    println!("  rota         | caminho  | pior |  médio | px visíveis | caixa");
    println!("  -------------+----------+------+--------+-------------+-------------------");
    for (nome_c, pts) in [
        ("rabisco", caminho_rabisco(120)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        for (rota, replay) in [("replay", true), ("acumulação", false)] {
            let img = |global: bool| {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                let mut t = tela_com_arte(40.0, 0);
                t.paint.pilha_por_replay = replay;
                pilha_do_dono(&mut t);
                corre(&mut t, &pts);
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                (*t.canvas_rgba).clone()
            };
            let (pior, medio, nz, caixa) = diff_visivel(&img(true), &img(false));
            println!("  {rota:12} | {nome_c:8} | {pior:4} | {medio:6.3} | {nz:11} | {caixa}");
        }
    }
}

/// ⭐ **O RESÍDUO da rota nova** — de que camada ele é.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_cura_o_residuo() {
    println!("\n  O RESÍDUO — cada pilha contra a composição GLOBAL, na rota de ACUMULAÇÃO\n");
    println!("  pilha                          | sem limite | pior | px  | caixa");
    println!("  -------------------------------+------------+------+-----+------------------");
    let pts = caminho_rabisco(120);
    let casos: [(&str, fn(&mut PainterTool)); 4] = [
        ("Brush só", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 0.3,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([0.0, 1.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("Blur + Brush", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Blur,
                strength: 0.4,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 0.3,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("Brush + Smear", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 0.3,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Smear,
                strength: 0.6,
                ..CompositeLayer::default()
            };
        }),
        ("os três (o do gate)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Blur,
                strength: 0.4,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 0.3,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[3] = CompositeLayer {
                op: CompositeOp::Smear,
                strength: 0.6,
                ..CompositeLayer::default()
            };
        }),
    ];
    for (nome, monta) in casos {
        for sem_limite in [false, true] {
            let img = |global: bool| {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                super::composite_pilha::SMEAR_SEM_LIMITE.with(|c| c.set(sem_limite));
                let mut t = tela_com_arte(40.0, 0);
                monta(&mut t);
                corre(&mut t, &pts);
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                super::composite_pilha::SMEAR_SEM_LIMITE.with(|c| c.set(false));
                (*t.canvas_rgba).clone()
            };
            let (pior, _, nz, caixa) = diff_visivel(&img(true), &img(false));
            println!(
                "  {nome:30} | {:10} | {pior:4} | {nz:3} | {caixa}",
                if sem_limite { "sim" } else { "não" }
            );
        }
    }
}
