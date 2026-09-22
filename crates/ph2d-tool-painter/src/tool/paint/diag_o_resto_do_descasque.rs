//! **O RESÍDUO DO DESCASQUE** — a sonda do report de 2026-09-21 (*«artefatos de imagem nas margens
//! retangulares»*, duas fotos).
//!
//! ⭐ O oráculo é a SEGUNDA foto dele, e é o mais limpo que este defeito admite: **com uma figura
//! em voo a tinta SOME de propósito** ([`super::shape_draft`]) — o preview é descascado e nada é
//! carimbado até ao pen-up. ⇒ *depois de N quadros de arrasto sem largar, a tela tem de estar
//! BYTE-IDÊNTICA à de antes do pen-down.* Tudo o que sobrar é resíduo que o descasque não alcançou.
//!
//! ⛔⛔ **A 1.ª redacção desta sonda leu `0` em todas as células e não afirmava nada:** ela corria
//! sobre uma tela VAZIA, e a §1 da [auditoria de hoje](../../../../../docs/Painter/40_auditoria_da_pilha_2026-09-21.md)
//! já tinha medido que ali o Smear move `0` pixels — *não há nada para esfregar*. A fixtura honesta
//! pinta ARTE antes, e leva o controlo positivo ao lado.
//!
//! ⚠️ A régua que já existia não podia ver isto: o [`super::composite_formas_tests`] mede a tinta
//! DENTRO da figura, e a [`super::diag_auditoria_da_pilha`] mede um A/B de duas rotas que
//! **partilham** a caixa do descasque — *um A/B entre dois caminhos que cometem o mesmo erro lê
//! zero*.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_o_resto -- --ignored --nocapture --test-threads=1
//! ```

use super::diag_auditoria_da_pilha::{Caso, cp, pilha_do_dono, tela_com};
use super::*;

const S: u32 = 1024;
const C: [f32; 2] = [512.0, 512.0];

/// A tela do dono: vazia, com a pilha dele e **arte por baixo** — sem ela o esfregão é inerte.
fn cena() -> PainterTool {
    let mut t = tela_com(24.0, 0);
    pilha_do_dono(&mut t);
    // Arte: um traço à mão livre COMMITADO, que é o que as camadas de baixo vão ler.
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.on_canvas_pointer(cp([300.0, 512.0], PointerPhase::Down));
    for i in 1..=100 {
        t.on_canvas_pointer(cp([300.0 + i as f32 * 4.0, 512.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([700.0, 512.0], PointerPhase::Up));
    t
}

/// Um gesto de Ellipse; `soltar` decide se ele é largado (carimba) ou fica **em voo**.
fn elipse(t: &mut PainterTool, raios: &[f32], soltar: bool) {
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp(C, PointerPhase::Down));
    for r in raios {
        t.on_canvas_pointer(cp([C[0] + r, C[1]], PointerPhase::Move));
    }
    if soltar {
        let r = *raios.last().unwrap();
        t.on_canvas_pointer(cp([C[0] + r, C[1]], PointerPhase::Up));
    }
}

/// O gesto que o DONO nomeou (2026-09-21): **Anchored** — um carimbo só, que CRESCE e depois
/// ENCOLHE no mesmo movimento. ⭐ É a reprodução mais barata deste defeito, e o *«ao crescer
/// desenha corretamente»* é o diagnóstico: a crescer, o quadro seguinte TAPA o resíduo do
/// anterior; a encolher, ele fica à vista.
fn ancorado(t: &mut PainterTool, raios: &[f32], soltar: bool) {
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Anchored;
    t.on_canvas_pointer(cp(C, PointerPhase::Down));
    for r in raios {
        t.on_canvas_pointer(cp([C[0] + r, C[1]], PointerPhase::Move));
    }
    if soltar {
        let r = *raios.last().unwrap();
        t.on_canvas_pointer(cp([C[0] + r, C[1]], PointerPhase::Up));
    }
}

/// Os texels em que a tela DIFERE do que era antes do gesto.
fn difere(antes: &[u8], t: &PainterTool) -> Vec<(u32, u32, u8)> {
    let mut v = Vec::new();
    for y in 0..S {
        for x in 0..S {
            let i = ((y * S + x) * 4) as usize;
            let d = (0..4)
                .map(|k| antes[i + k].abs_diff(t.canvas_rgba[i + k]))
                .max()
                .unwrap_or(0);
            if d > 0 {
                v.push((x, y, d));
            }
        }
    }
    v
}

/// A caixa e o perfil ANGULAR — é o perfil que decide entre as duas explicações.
pub(super) fn conta(nome: &str, v: &[(u32, u32, u8)]) {
    if v.is_empty() {
        println!("  {nome:36} |      0 |    - |         - | -");
        return;
    }
    let (x0, y0) = (
        v.iter().map(|p| p.0).min().unwrap(),
        v.iter().map(|p| p.1).min().unwrap(),
    );
    let (x1, y1) = (
        v.iter().map(|p| p.0).max().unwrap(),
        v.iter().map(|p| p.1).max().unwrap(),
    );
    let pior = v.iter().map(|p| p.2).max().unwrap();
    let mut oit = [0usize; 8];
    for &(x, y, _) in v {
        let (dx, dy) = (f64::from(x) - 512.0, f64::from(y) - 512.0);
        let mut a = dy.atan2(dx).to_degrees() + 360.0 + 22.5;
        a %= 360.0;
        oit[(a / 45.0) as usize % 8] += 1;
    }
    println!(
        "  {nome:36} | {:6} | {pior:4} | {:9} | E{} SE{} S{} SO{} O{} NO{} N{} NE{}",
        v.len(),
        format!("{}x{}", x1 - x0 + 1, y1 - y0 + 1),
        oit[0],
        oit[1],
        oit[2],
        oit[3],
        oit[4],
        oit[5],
        oit[6],
        oit[7]
    );
}

/// Um gesto de **Free Hand** ao longo de um caminho; `soltar` fecha a figura (ela fica editável).
pub(super) fn mao_livre(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], soltar: bool) {
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::FreeHand;
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    for i in 1..=24 {
        let u = i as f32 / 24.0;
        t.on_canvas_pointer(cp(
            [de[0] + (ate[0] - de[0]) * u, de[1] + (ate[1] - de[1]) * u],
            PointerPhase::Move,
        ));
    }
    if soltar {
        t.on_canvas_pointer(cp(ate, PointerPhase::Up));
    }
}

/// Uma tela com ARTE OPACA pintada pelo pincel SIMPLES (composite desligado) e só depois a pilha.
///
/// ⚠️ ⛔ **A 1.ª régua deste report media uma coisa legítima:** sobre arte pintada COM a pilha (alfa
/// `~80`) qualquer queda de alfa conta, e uma camada `Blur` **baixa o alfa do miolo por lei**.
/// *Uma régua de «o alfa caiu» não distingue o borrão do apagão.* Com arte OPACA a pergunta passa a
/// ser inequívoca: **quantos texels de alfa `255` foram a QUASE ZERO**, que é o «completamente
/// branco» da foto.
fn cena_opaca() -> PainterTool {
    let mut t = tela_com(24.0, 0);
    t.paint.composite_enabled = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.paint.brush.color = [0.1, 0.1, 0.1];
    t.on_canvas_pointer(cp([300.0, 512.0], PointerPhase::Down));
    for i in 1..=100 {
        t.on_canvas_pointer(cp([300.0 + i as f32 * 4.0, 512.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([700.0, 512.0], PointerPhase::Up));
    pilha_do_dono(&mut t);
    t.paint.composite_enabled = true;
    t
}

/// A mesma cena com um CAMPO largo de arte, e não uma tira.
///
/// ⚠️ A foto do dono tem a arte a cobrir metade da tela e o rectângulo branco mede `~390×140`; uma
/// tira de `48 px` de altura **não pode conter um rectângulo desses**. *Uma fixtura mais pequena que
/// o fenómeno mede sempre a borda dele.*
fn cena_opaca_larga() -> PainterTool {
    let mut t = tela_com(24.0, 0);
    t.paint.composite_enabled = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.paint.brush.color = [0.1, 0.1, 0.1];
    for fila in 0..10 {
        let y = 400.0 + fila as f32 * 26.0;
        t.on_canvas_pointer(cp([280.0, y], PointerPhase::Down));
        for i in 1..=110 {
            t.on_canvas_pointer(cp([280.0 + i as f32 * 4.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([720.0, y], PointerPhase::Up));
    }
    pilha_do_dono(&mut t);
    t.paint.composite_enabled = true;
    t
}

/// Os texels de arte OPACA que foram a QUASE ZERO — o «completamente branco».
fn branqueado(antes: &[u8], t: &PainterTool) -> Vec<(u32, u32, u8)> {
    let mut v = Vec::new();
    for y in 0..S {
        for x in 0..S {
            let i = ((y * S + x) * 4 + 3) as usize;
            let (a, b) = (antes[i], t.canvas_rgba[i]);
            if a >= 200 && b < 40 {
                v.push((x, y, a - b));
            }
        }
    }
    v
}

/// Um mapa grosso da região `(x0,y0)-(x1,y1)`: `#` arte que FICOU, `·` arte APAGADA, ` ` sem arte.
///
/// ⭐ *Uma contagem não distingue um rasto fino de um RECTÂNGULO*, e é a forma que o dono vê.
fn mapa(antes: &[u8], t: &PainterTool, x0: u32, y0: u32, x1: u32, y1: u32, passo: u32) {
    println!("\n  mapa  x {x0}..{x1}  y {y0}..{y1}  (1 char = {passo} px)");
    for by in (y0..y1).step_by(passo as usize) {
        let mut linha = String::new();
        for bx in (x0..x1).step_by(passo as usize) {
            let (mut arte, mut ido) = (0u32, 0u32);
            for y in by..(by + passo).min(y1) {
                for x in bx..(bx + passo).min(x1) {
                    let i = ((y * S + x) * 4 + 3) as usize;
                    if antes[i] >= 200 {
                        arte += 1;
                        if t.canvas_rgba[i] < 40 {
                            ido += 1;
                        }
                    }
                }
            }
            linha.push(if arte == 0 {
                ' '
            } else if ido * 2 >= arte {
                '#'
            } else if ido > 0 {
                '+'
            } else {
                '.'
            });
        }
        println!("    |{linha}|");
    }
    println!("    ('#' maioria APAGADA · '+' alguma · '.' intacta · ' ' sem arte)");
}

/// ⛔⛔⛔ **O RECTÂNGULO BRANCO** — report do dono de 2026-09-21, com foto:
/// *«usando o stroke freehand os retângulos ficaram completamente brancos cobrindo a imagem pintada
/// anteriormente»*.
#[test]
#[ignore = "sonda: corre à mão, --release --nocapture"]
fn diag_o_rectangulo_branco() {
    println!("\n  O RECTÂNGULO BRANCO — quanta ARTE um gesto de figura APAGA");
    println!("  canvas {S}² · pilha do dono · ARTE por baixo\n");
    println!("  caso                                 | texels | pior |     caixa | por oitante");
    println!("  -------------------------------------+--------+------+-----------+------------");

    // ⚠️ O alias é o da sonda irmã — um segundo aqui seria a mesma resposta escrita duas vezes.
    let casos: [Caso; 4] = [
        ("FreeHand: 1 figura, largada", |t| {
            mao_livre(t, [380.0, 470.0], [640.0, 560.0], true);
        }),
        ("FreeHand: 1 figura, EM VOO", |t| {
            mao_livre(t, [380.0, 470.0], [640.0, 560.0], false);
        }),
        ("FreeHand: 2 figuras na mesma sessão", |t| {
            mao_livre(t, [380.0, 470.0], [640.0, 560.0], true);
            mao_livre(t, [380.0, 560.0], [640.0, 470.0], true);
        }),
        ("Ellipse: 2 figuras na mesma sessão", |t| {
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
            for c in [[420.0f32, 512.0], [600.0, 512.0]] {
                t.on_canvas_pointer(cp(c, PointerPhase::Down));
                for i in 1..=4 {
                    t.on_canvas_pointer(cp([c[0] + 20.0 * i as f32, c[1]], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([c[0] + 80.0, c[1]], PointerPhase::Up));
            }
        }),
    ];
    for (nome, gesto) in casos {
        for antiga in [false, true] {
            super::region::CAIXA_DO_PINCEL.with(|c| c.set(antiga));
            let mut t = cena_opaca();
            let antes = t.canvas_rgba.to_vec();
            gesto(&mut t);
            let marca = if antiga { " [caixa ANTIGA]" } else { "" };
            conta(&format!("{nome}{marca}"), &branqueado(&antes, &t));
        }
        super::region::CAIXA_DO_PINCEL.with(|c| c.set(false));
    }
    // ⭐ ATRIBUIÇÃO: calar uma camada de cada vez, sobre o caso mais forte.
    println!();
    let pior_caso = |t: &mut PainterTool| {
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        for c in [[420.0f32, 512.0], [600.0, 512.0]] {
            t.on_canvas_pointer(cp(c, PointerPhase::Down));
            for i in 1..=4 {
                t.on_canvas_pointer(cp([c[0] + 20.0 * i as f32, c[1]], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([c[0] + 80.0, c[1]], PointerPhase::Up));
        }
    };
    for alvo in 0..composite::N_CAMADAS {
        let op = cena_opaca().paint.composite[alvo].op;
        if cena_opaca().paint.composite[alvo].strength <= 0.0 {
            continue;
        }
        let mut t = cena_opaca();
        t.paint.composite[alvo].strength = 0.0;
        let antes = t.canvas_rgba.to_vec();
        pior_caso(&mut t);
        conta(
            &format!("  ablação: camada {alvo} ({op:?}) calada"),
            &branqueado(&antes, &t),
        );
    }
    // ⭐⭐⭐ E O TECTO DO TRANSPORTE: é o mesmo defeito que a wave da CURVA mediu?
    //
    // ⛔ O esfregão não APAGA tinta: ele reamostra uma fonte congelada em `p − disp`. Onde o `disp`
    // é grande, um texel de arte passa a mostrar o que a fonte tem LONGE dali — tela vazia. O
    // alcance foi medido nesta linha em **`2,2`–`9,5` raios de pincel**
    // ([`ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`]), e a fronteira do que é reamostrado
    // é a REGIÃO: um rectângulo.
    for tecto in [
        ph2d_painter_brush::SEM_TECTO,
        ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
        1.0,
    ] {
        super::smear_warp::espia::poe_tecto(tecto);
        let mut t = cena_opaca();
        let antes = t.canvas_rgba.to_vec();
        pior_caso(&mut t);
        let nome = if tecto.is_finite() {
            format!("  tecto do transporte = {tecto:.1} raios")
        } else {
            "  tecto do transporte = NENHUM (o que shipa)".to_string()
        };
        conta(&nome, &branqueado(&antes, &t));
    }
    super::smear_warp::espia::poe_tecto(ph2d_painter_brush::SEM_TECTO);

    // ⭐ O CAMPO LARGO — a fixtura do tamanho do fenómeno.
    println!();
    for (nome, gesto) in [
        ("CAMPO largo · Ellipse 2 figuras", 0u8),
        ("CAMPO largo · FreeHand 1 figura", 1),
        ("CAMPO largo · FreeHand 2 figuras", 2),
    ] {
        let mut t = cena_opaca_larga();
        let antes = t.canvas_rgba.to_vec();
        match gesto {
            0 => pior_caso(&mut t),
            1 => mao_livre(&mut t, [340.0, 460.0], [660.0, 600.0], true),
            _ => {
                mao_livre(&mut t, [340.0, 460.0], [660.0, 600.0], true);
                mao_livre(&mut t, [340.0, 600.0], [660.0, 460.0], true);
            }
        }
        conta(nome, &branqueado(&antes, &t));
        if gesto == 2 {
            mapa(&antes, &t, 260, 380, 760, 660, 8);
        }
    }

    // E a UMA figura só, para separar «entre figuras» de «dentro de uma».
    let mut t = cena_opaca();
    let antes = t.canvas_rgba.to_vec();
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([420.0, 512.0], PointerPhase::Down));
    for i in 1..=4 {
        t.on_canvas_pointer(cp([420.0 + 20.0 * i as f32, 512.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([500.0, 512.0], PointerPhase::Up));
    conta("  Ellipse: UMA figura só", &branqueado(&antes, &t));

    // CONTROLO: sem composite, o mesmo gesto não pode apagar nada.
    let mut t = cena_opaca();
    t.paint.composite_enabled = false;
    let antes = t.canvas_rgba.to_vec();
    mao_livre(&mut t, [380.0, 470.0], [640.0, 560.0], true);
    conta("CONTROLO: sem composite, 1 figura", &branqueado(&antes, &t));
    // CONTROLO POSITIVO: a arte EXISTE e é opaca — senão um zero acima não afirma nada.
    let t = cena_opaca();
    let opacos = (0..(S * S) as usize)
        .filter(|&i| t.canvas_rgba[i * 4 + 3] >= 200)
        .count();
    println!("\n  CONTROLO+: texels de arte OPACA na cena: {opacos}");
}

/// ⭐⭐⭐ **A medição: o que o descasque deixa para trás, e de quem é.**
#[test]
#[ignore = "sonda: corre à mão, --release --nocapture"]
fn diag_o_resto_do_descasque() {
    println!("\n  O RESÍDUO DO DESCASQUE — figura EM VOO, a tela tem de ficar intacta");
    println!("  canvas {S}² · Ellipse · pilha do dono · ARTE por baixo · raios 200 -> 120 -> 60\n");
    println!("  caso                                 | texels | pior |     caixa | por oitante");
    println!("  -------------------------------------+--------+------+-----------+------------");

    let raios = [200.0f32, 120.0, 60.0];

    // ⭐ O CONTROLO POSITIVO vem PRIMEIRO: sem ele, um zero abaixo não distingue
    //   «o descasque está limpo» de «o gesto não pinta».
    let mut t = cena();
    let antes = t.canvas_rgba.to_vec();
    elipse(&mut t, &raios, true);
    conta("CONTROLO+: o mesmo gesto LARGADO", &difere(&antes, &t));

    // 1. A pilha do dono, em voo.
    let mut t = cena();
    let antes = t.canvas_rgba.to_vec();
    elipse(&mut t, &raios, false);
    let v = difere(&antes, &t);
    conta("pilha do dono, EM VOO", &v);

    // 2. CONTROLO: sem composite nenhum.
    let mut t = cena();
    t.paint.composite_enabled = false;
    let antes = t.canvas_rgba.to_vec();
    elipse(&mut t, &raios, false);
    conta("CONTROLO-: sem composite, em voo", &difere(&antes, &t));

    // 3. ABLAÇÃO: todos os tamanhos a 1,0.
    let mut t = cena();
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].size = 1.0;
    }
    let antes = t.canvas_rgba.to_vec();
    elipse(&mut t, &raios, false);
    conta("ablação: todo tamanho = 1,0", &difere(&antes, &t));

    // 4. ABLAÇÃO: uma camada de cada vez CALADA.
    for alvo in 0..composite::N_CAMADAS {
        let mut t = cena();
        if t.paint.composite[alvo].strength <= 0.0 {
            continue;
        }
        let op = t.paint.composite[alvo].op;
        t.paint.composite[alvo].strength = 0.0;
        let antes = t.canvas_rgba.to_vec();
        elipse(&mut t, &raios, false);
        conta(
            &format!("ablação: camada {alvo} ({op:?}) calada"),
            &difere(&antes, &t),
        );
    }

    // 4-bis. ⭐⭐⭐ O GESTO QUE O DONO NOMEOU, com o ORÁCULO certo, e ATRIBUÍDO.
    //
    // ⚠️ Num Anchored a tinta EM VOO é legítima (o artista VÊ o carimbo a crescer), logo comparar
    // contra a tela de antes mede o carimbo e não o defeito. O oráculo é **o mesmo estado final
    // alcançado sem passar pelo grande**: se o descasque fosse completo, `40→200→40` e `40` dariam
    // a MESMA tela.
    println!();
    let residuo = |ajusta: &dyn Fn(&mut PainterTool)| {
        let mut a = cena();
        ajusta(&mut a);
        ancorado(&mut a, &[40.0, 120.0, 200.0, 120.0, 40.0], false);
        let mut b = cena();
        ajusta(&mut b);
        ancorado(&mut b, &[40.0], false);
        let so_b = b.canvas_rgba.to_vec();
        difere(&so_b, &a)
    };
    let v = residuo(&|_t| {});
    conta("Anchored 40->200->40 vs 40 directo", &v);
    conta(
        "  CONTROLO: sem composite",
        &residuo(&|t| t.paint.composite_enabled = false),
    );
    conta(
        "  ablação: todo tamanho = 1,0",
        &residuo(&|t| {
            for pos in 0..composite::N_CAMADAS {
                t.paint.composite[pos].size = 1.0;
            }
        }),
    );
    for alvo in 0..composite::N_CAMADAS {
        let op = cena().paint.composite[alvo].op;
        if cena().paint.composite[alvo].strength <= 0.0 {
            continue;
        }
        conta(
            &format!("  ablação: camada {alvo} ({op:?}) calada"),
            &residuo(&move |t| t.paint.composite[alvo].strength = 0.0),
        );
    }
    // O perfil RADIAL: o resíduo mora numa banda, e a banda diz de quem ele é.
    if !v.is_empty() {
        let mut faixa = std::collections::BTreeMap::new();
        for &(x, y, _) in &v {
            let d = (f64::from(x) - 512.0).hypot(f64::from(y) - 512.0);
            *faixa.entry((d / 20.0) as u32 * 20).or_insert(0usize) += 1;
        }
        println!("\n  perfil RADIAL do resíduo (raio px -> texels):");
        for (r, n) in &faixa {
            println!(
                "    {r:4}..{:4} | {n:6} {}",
                r + 20,
                "#".repeat((*n / 400).min(60))
            );
        }
    }

    // 5. A PREVISÃO: o resíduo mora entre o raio do PINCEL e o da CAMADA MAIOR.
    let mut t = cena();
    let maior = (0..composite::N_CAMADAS)
        .filter(|&p| t.paint.composite[p].strength > 0.0)
        .map(|p| t.tamanho_da_camada(p))
        .fold(1.0f32, f32::max);
    let antes = t.canvas_rgba.to_vec();
    elipse(&mut t, &raios, false);
    let v = difere(&antes, &t);
    let dentro = v
        .iter()
        .filter(|&&(x, y, _)| {
            let d = (f64::from(x) - 512.0).hypot(f64::from(y) - 512.0);
            raios.iter().any(|&r| {
                let r = f64::from(r);
                d >= r + 24.0 - 2.0 && d <= r + 24.0 * f64::from(maior) + 2.0
            })
        })
        .count();
    println!(
        "\n  maior escala de camada VIVA: {maior:.3}×  (pincel 24 px ⇒ a camada alcança {:.1} px)",
        24.0 * maior
    );
    println!(
        "  resíduo na banda prevista [r+24 , r+{:.1}]: {dentro} de {} ({:.1} %)",
        24.0 * maior,
        v.len(),
        100.0 * dentro as f64 / v.len().max(1) as f64
    );
}

/// ⭐⭐⭐ **UM RE-CARIMBO QUE ENCOLHE NÃO DEIXA RASTO** — o gate do report de 2026-09-21
/// (*«artefatos de imagem nas margens retangulares»*, três fotos) e da **dica** do dono, que é o
/// diagnóstico inteiro: *«com Anchored, ao crescer ele desenha corretamente, mas se no mesmo
/// movimento reduzir, vários artefatos retangulares aparecem»*.
///
/// ⛔⛔ **A causa é uma assimetria de REGIÃO:** o descasque salvava a caixa do raio do **PINCEL** e
/// a pilha escreve a caixa do raio da **CAMADA** — a [`super::composite::PainterTool::camada_dabs`]
/// faz `radius_px *= escala`, e a escala do dono é `1,904`. O anel entre as duas nunca era
/// restaurado. ⇒ *a crescer, o quadro seguinte TAPA o anel do anterior; a encolher, ele fica à
/// vista.* Cura: [`super::region::PainterTool::caixa_do_lote`], a porta ÚNICA de *«que região este
/// lote escreve»* — a mesma pergunta que a `watercolor_preview_footprint` responde ao lado.
///
/// ⭐ **A forma prevista bateu com a terceira foto do dono antes de eu a ver:** um anel recortado
/// pelo rectângulo que o contém só escapa onde o círculo **TOCA** o rectângulo — os quatro pontos
/// cardeais, nunca os cantos. A foto tem exactamente quatro fantasmas, um em cada ponto cardeal.
///
/// **Medido** (pilha do dono, pincel `24`, `40→200→40`): `290 534` texels de rasto contra `0`, e o
/// resíduo acabava em `raio 380` = `200 × 1,904`, o alcance exacto da camada maior.
///
/// ⚠️⚠️ **O CONTROLO é o interruptor de bissecção, e sem ele este gate não afirma nada** — com
/// `size = 1` as duas respostas coincidem e os dois lados leem `0`. É a mesma lei que a §1 da
/// auditoria desta data pagou: *uma fixtura no ponto neutro é verde a afirmar nada*.
#[test]
fn um_recarimbo_que_encolhe_nao_deixa_rasto() {
    let corre = |raios: &[f32], caixa_antiga: bool| {
        super::region::CAIXA_DO_PINCEL.with(|c| c.set(caixa_antiga));
        let mut t = tela_com(24.0, 0);
        pilha_do_dono(&mut t);
        ancorado(&mut t, raios, false);
        super::region::CAIXA_DO_PINCEL.with(|c| c.set(false));
        t
    };
    let difs = |a: &PainterTool, b: &PainterTool| {
        a.canvas_rgba
            .iter()
            .zip(b.canvas_rgba.iter())
            .filter(|(x, y)| x != y)
            .count()
    };
    let cresce = [40.0f32, 120.0, 200.0];
    let vaivem = [40.0f32, 120.0, 200.0, 120.0, 40.0];

    // CONTROLO POSITIVO: o carimbo grande pinta, e pinta ALÉM do raio do pincel — senão «não deixa
    // rasto» seria verdade sobre uma tela onde nada aconteceu.
    let grande = corre(&cresce, false);
    let linha = (0..S)
        .filter(|&x| grande.canvas_rgba[((512 * S + x) * 4 + 3) as usize] > 4)
        .count();
    assert!(
        linha > 2 * 200,
        "a camada não escreveu além do raio do pincel ({linha} px na linha do centro) — \
         a fixtura está no ponto neutro e este gate não afirma nada"
    );

    // ⛔ O CONTROLO DA CURA: com a caixa ANTIGA o rasto EXISTE.
    let rasto = difs(&corre(&vaivem, true), &corre(&[40.0], true));
    assert!(
        rasto > 10_000,
        "a fixtura não contém o fenómeno: com a caixa antiga o rasto foi de {rasto} bytes"
    );

    // ⭐ A LEI: passar pelo grande e voltar dá a MESMA tela que nunca lá ter ido.
    let n = difs(&corre(&vaivem, false), &corre(&[40.0], false));
    assert_eq!(
        n, 0,
        "encolher deixou {n} bytes de rasto (com a caixa antiga são {rasto})"
    );
}
