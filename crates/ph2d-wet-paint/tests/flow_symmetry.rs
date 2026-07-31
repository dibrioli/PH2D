//! **QUAL DOS DOIS CAMPOS DE FLUXO ESTÁ CERTO** — a mesma pergunta que o
//! `solver_symmetry.rs` faz ao advect e à secagem, agora ao terceiro passe.
//!
//! O [`build_flow_field`](ph2d_wet_paint::solver::build_flow_field) serial
//! carimba a umidade (`wet[c] = g(paper[c])` onde o filme é fundo) **no meio
//! da varredura**, e algumas linhas depois o FREIO de uma outra célula lê
//! `wet[probe]` alguns pixels adiante. Numa cena cujo fluxo diverge do centro,
//! a metade da ESQUERDA sonda células que a varredura já reescreveu e a da
//! DIREITA sonda células ainda intocadas — o mesmo freio, dois valores.
//!
//! Isso não é física: é a direção do laço. E como a fixture é simétrica por
//! construção, a assinatura é mensurável.
//!
//! ⚠️ **O oráculo é a ANTISSIMETRIA de `flow_x`**, não um valor absoluto: sob
//! `x → W+1−x` toda entrada do passe ou é simétrica (film, paper, wet) ou
//! antissimétrica (as velocidades), e cada termo preserva isso — o gradiente
//! do nivelamento troca de lado, a viscosidade é uma média de vizinhos
//! antissimétricos, e o freio é um escalar cujo probe espelha junto. Logo a
//! saída TEM de ser antissimétrica, e o quanto ela não é mede exatamente a
//! dependência de ordem.

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::{Params, Sim};
use ph2d_wet_paint::solver::{
    build_flow_field, build_flow_field_jacobi, build_flow_field_jacobi_rows, rebuild_active_region,
};
use ph2d_wet_paint::tuning::{Knob, Tuning};

const W: usize = 96;
const H: usize = 24;

/// Uma poça FUNDA e simétrica em `x`, sob um campo de velocidade
/// **antissimétrico** (divergente a partir do centro) e sem componente
/// vertical.
///
/// Três coisas a fixture PRECISA ter, ou o gate fica verde por vácuo:
///
/// * **filme > 3** em boa parte da faixa — é a condição do carimbo de umidade;
///   sem ele nada é escrito no meio do passe e os dois modelos coincidem;
/// * **`paper` que produz um byte de umidade ≠ do inicial** (`wet` nasce em 0,
///   e `wet_byte_from_paper(0.2) = 255`) — carimbar o mesmo valor que já
///   estava lá é indistinguível de não carimbar;
/// * **estrutura** no perfil (um platô com dentes) — um campo chato é
///   simétrico sob qualquer coisa.
fn symmetric_pool() -> Grid {
    let mut g = Grid::new(W, H);
    let s = g.s;
    let cx = (W as f64 + 1.0) / 2.0; // o eixo do espelho, entre as colunas
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * s;
            let d = (x as f64 - cx).abs();
            let bump = if d < 30.0 { 1.0 - d / 30.0 } else { 0.0 };
            // ⚠️ Os dentes medem a DISTÂNCIA AO EIXO, nunca `x` — a lição que o
            // controle do `solver_symmetry.rs` pagou.
            let teeth = if (d.floor() as i32) % 7 < 3 {
                1.3
            } else {
                0.85
            };
            g.film[i] = (7.0 * bump * teeth) as f32;
            g.susp[i] = (400.0 * bump) as f32;
            g.susp_rgb[i] = [200.0, 40.0, 90.0];
            g.paper[i] = (0.35 + 0.25 * teeth) as f32;
            // Velocidade DIVERGENTE: negativa à esquerda, positiva à direita,
            // exatamente espelhada. `vel_y = 0` mantém o problema 1-D.
            g.vel_x[i] = (0.9 * (x as f64 - cx) / 24.0).clamp(-0.9, 0.9) as f32;
            g.vel_y[i] = 0.0;
        }
    }
    g.expand_bbox(1, 1, W as i32, H as i32);
    rebuild_active_region(&mut g);
    assert!(g.has_fluid, "a fixture tem de ter agua");
    g
}

fn params() -> Params {
    Sim::default().gather_params(&Tuning::default())
}

/// O pior desvio de ANTISSIMETRIA de `flow_x`.
fn mirror_error(g: &Grid) -> f64 {
    let s = g.s;
    let mut worst = 0.0f64;
    for y in 1..=H {
        for x in 1..=W / 2 {
            let a = f64::from(g.flow_x[x + y * s]);
            let b = f64::from(g.flow_x[(W + 1 - x) + y * s]);
            let d = (a + b).abs();
            if d > worst {
                worst = d;
            }
        }
    }
    worst
}

/// A fixture de fato dispara o carimbo — sem isto o gate mediria dois passes
/// que nunca escrevem nada no meio da varredura, e ficaria verde por vácuo.
#[test]
fn the_fixture_actually_stamps_wetness_mid_pass() {
    let g = symmetric_pool();
    let s = g.s;
    let deep = (1..=H)
        .flat_map(|y| (1..=W).map(move |x| (x, y)))
        .filter(|(x, y)| f64::from(g.film[x + y * s]) > 3.0)
        .count();
    assert!(
        deep > 400,
        "so {deep} celulas tem filme > 3 — o carimbo mal roda e o gate nao mede nada"
    );
    assert!(
        g.wet.iter().all(|w| *w == 0),
        "a fixture ja nasce umida — carimbar o mesmo valor e indistinguivel de nao carimbar"
    );
}

/// **NO PONTO DE OPERAÇÃO QUE SHIPA, OS DOIS MODELOS SÃO O MESMO — AO BYTE.**
///
/// E a razão é aritmética, não sorte. O carimbo só escreve onde `film > 3`; o
/// freio de quem sonda aquela célula vale
/// `clamp(film + 3·wet/255 − brake, 0.05, 1)`, e com o `brake` **default de
/// 1,5** já `film − 1,5 > 1,5 > 1` ⇒ **satura em 1,0 e o termo de umidade não
/// entra**. Dito de outro modo: *a única célula cujo `wet` este passe pode
/// mudar é uma célula funda demais para que o `wet` dela importe.*
///
/// ⚠️ **É isto que mantém o fingerprint do ADR-0134 INTACTO** — a wave é uma
/// reescrita de laço quente no ponto de operação do produto, e vira mudança de
/// modelo só onde o gate irmão abaixo mede.
#[test]
fn at_the_shipping_knobs_the_two_models_are_the_same_to_the_byte() {
    let p = params();
    let mut gs = symmetric_pool();
    build_flow_field(&mut gs, &p, 0.0, 0.6, false);
    let mut ji = symmetric_pool();
    build_flow_field_jacobi(&mut ji, &p, 0.0, 0.6, false);
    assert_eq!(
        gs.flow_x, ji.flow_x,
        "flow_x divergiu no ponto de operação default"
    );
    assert_eq!(
        gs.flow_y, ji.flow_y,
        "flow_y divergiu no ponto de operação default"
    );
    assert_eq!(gs.wet, ji.wet, "o carimbo de umidade divergiu");
}

/// **E ONDE ELES DIFEREM, O SERIAL É QUEM ESTÁ TORTO.**
///
/// Com o freio no máximo (`brake = 4`) a saturação some para o filme na faixa
/// `(3, 5)`: ali o `wet` decide entre frear e não frear, e o serial responde
/// diferente conforme a varredura já tenha passado pela célula sondada. Numa
/// cena simétrica por construção isso tem assinatura.
///
/// Mutação: apontar as duas rotas para o mesmo `build_flow_field` faz os dois
/// números coincidirem e o gate morre — é a DIFERENÇA que carrega a afirmação.
#[test]
fn at_a_high_brake_the_gauss_seidel_leans_left_to_right_and_the_jacobi_does_not() {
    let mut t = Tuning::default();
    t.set(Knob::Brake, 4.0);
    let p = Sim::default().gather_params(&t);

    let mut gs = symmetric_pool();
    build_flow_field(&mut gs, &p, 0.0, 0.0, false);
    let e_gs = mirror_error(&gs);

    let mut ji = symmetric_pool();
    build_flow_field_jacobi(&mut ji, &p, 0.0, 0.0, false);
    let e_ji = mirror_error(&ji);

    println!("  viés de espelho do flow_x: gauss-seidel {e_gs:.6}  jacobi {e_ji:.6}");
    assert_eq!(
        e_ji, 0.0,
        "o campo independente de ordem devia ser exatamente antissimetrico, e desviou {e_ji}"
    );
    assert!(
        e_gs > 1e-3,
        "o gauss-seidel nao mostrou vies ({e_gs}) — a fixture nao contem o fenomeno"
    );
}

/// O carimbo de umidade **sai do passe, mas não sai do produto**: os dois
/// modelos deixam o MESMO `wet` no fim.
///
/// ⚠️ É o que separa *"o freio lê o estado de entrada"* de *"o carimbo
/// sumiu"* — a extração seria uma regressão silenciosa se o plano de umidade
/// parasse de ser escrito, e nenhum gate de fluxo veria.
#[test]
fn both_models_leave_the_same_wetness_behind() {
    let p = params();
    let mut gs = symmetric_pool();
    build_flow_field(&mut gs, &p, 0.0, 0.0, false);
    let mut ji = symmetric_pool();
    build_flow_field_jacobi(&mut ji, &p, 0.0, 0.0, false);
    assert_eq!(
        gs.wet, ji.wet,
        "o carimbo de umidade divergiu entre os modelos"
    );
    assert!(
        gs.wet.iter().any(|w| *w != 0),
        "nenhuma celula foi carimbada — o gate esta comparando dois planos vazios"
    );
}

/// **O FREIO LÊ A UMIDADE DE ANTES DO PASSE** — não a de depois.
///
/// ⚠️ Sem este gate, mover o carimbo para ANTES do núcleo passaria em tudo: o
/// resultado continuaria independente de ordem (todo mundo veria a folha
/// já carimbada), só seria **outro modelo**. O oráculo é a diferença entre
/// rodar o passe sobre a poça crua e sobre a MESMA poça já carimbada à mão:
/// se o freio lesse a umidade pós-carimbo, os dois campos coincidiriam.
#[test]
fn the_brake_reads_the_wetness_from_before_the_pass() {
    let mut t = Tuning::default();
    t.set(Knob::Brake, 4.0); // onde o termo de umidade sai da saturação
    let p = Sim::default().gather_params(&t);

    let mut raw = symmetric_pool();
    build_flow_field_jacobi(&mut raw, &p, 0.0, 0.0, false);

    // A MESMA poça, com o carimbo já aplicado à mão antes do passe.
    let mut pre = symmetric_pool();
    for i in 0..pre.cells {
        if f64::from(pre.film[i]) > 3.0 {
            pre.wet[i] = ph2d_wet_paint::grid::wet_byte_from_paper(f64::from(pre.paper[i]));
        }
    }
    build_flow_field_jacobi(&mut pre, &p, 0.0, 0.0, false);

    assert_ne!(
        raw.flow_x, pre.flow_x,
        "o campo nao mudou com a folha ja carimbada — o freio esta lendo a umidade DEPOIS do carimbo"
    );
}

/// **O BACKRUN É PURE CODE MOTION — ao byte.**
///
/// O levante é `F^n` com um `F` que não conhece o gatilho, então espalhar (o
/// serial) e contar (o gather) pousam no MESMO número. O que muda de modelo é
/// só o campo de fluxo, cujo portão capilar passa a ler o `susp+sett` de antes
/// dos levantes — e é por isso que este gate compara os planos de PIGMENTO e
/// não o fluxo.
#[test]
fn the_backrun_lift_lands_on_the_same_pigment_as_the_serial() {
    let mut t = Tuning::default();
    t.set(Knob::ExtBackrun, 1.5);
    let p = Sim::default().gather_params(&t);

    let mut gs = symmetric_pool();
    seed_settled(&mut gs);
    build_flow_field(&mut gs, &p, 0.0, 0.6, false);

    let mut ji = symmetric_pool();
    seed_settled(&mut ji);
    build_flow_field_jacobi(&mut ji, &p, 0.0, 0.6, false);

    assert_eq!(gs.sett, ji.sett, "o pigmento assentado divergiu");
    assert_eq!(gs.susp, ji.susp, "o pigmento em suspensao divergiu");
    assert_eq!(gs.susp_rgb, ji.susp_rgb, "a cor em suspensao divergiu");
    assert_eq!(gs.bloom, ji.bloom, "o orcamento de floracao divergiu");
}

/// A fixture do backrun precisa de pigmento ASSENTADO com um degrau de filme
/// ao lado — sem isso o levante nunca dispara e o gate acima fica verde por
/// vácuo.
fn seed_settled(g: &mut Grid) {
    let s = g.s;
    let (mut lifted, mut spent) = (0usize, 0usize);
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * s;
            // Uma faixa seca de pigmento onde o filme é raso: é ali que o
            // vizinho fundo consegue ver o degrau.
            if f64::from(g.film[i]) < 2.0 {
                g.sett[i] = 500.0;
                g.sett_rgb[i] = [30.0, 120.0, 200.0];
                lifted += 1;
            }
            // ⚠️ **Metade dos gatilhos nasce SEM orçamento de floração.** Sem
            // isto `bloom` é 0 em toda parte, `bloom < 6` é sempre verdade, e
            // o predicado fica INERTE — a mutação que o apaga passava em tudo.
            if y % 2 == 0 && f64::from(g.film[i]) >= 2.0 {
                g.bloom[i] = 6;
                spent += 1;
            }
        }
    }
    assert!(
        lifted > 200,
        "so {lifted} celulas assentadas — o backrun nao teria o que levantar"
    );
    assert!(
        spent > 200,
        "so {spent} gatilhos sem orcamento — o `bloom < 6` fica inerte"
    );
}

/// O gate acima só vale se o levante DE FATO rodou.
#[test]
fn the_backrun_fixture_actually_lifts() {
    let mut t = Tuning::default();
    t.set(Knob::ExtBackrun, 1.5);
    let p = Sim::default().gather_params(&t);
    let mut g = symmetric_pool();
    seed_settled(&mut g);
    let before: f64 = g.sett.iter().map(|v| f64::from(*v)).sum();
    build_flow_field_jacobi(&mut g, &p, 0.0, 0.6, false);
    let after: f64 = g.sett.iter().map(|v| f64::from(*v)).sum();
    assert!(
        before - after > 1.0,
        "o assentado nao se moveu ({before} -> {after}) — a fixture nao contem o fenomeno"
    );
}

/// **As duas rotas de caminhada são a MESMA resposta** (ADR-0145): o pool não
/// tem voto sobre a aritmética, só sobre quem avalia qual linha.
#[test]
fn the_serial_and_parallel_walks_agree_to_the_byte() {
    for backrun in [false, true] {
        let mut t = Tuning::default();
        if backrun {
            t.set(Knob::ExtBackrun, 1.0);
        }
        let p = Sim::default().gather_params(&t);
        let mut a = symmetric_pool();
        let mut b = symmetric_pool();
        // A gravidade entra para o fingering/backrun terem um eixo.
        build_flow_field_jacobi_rows(
            &mut a,
            &p,
            0.0,
            0.6,
            false,
            ph2d_wet_paint::par::Rows::Serial,
        );
        build_flow_field_jacobi_rows(
            &mut b,
            &p,
            0.0,
            0.6,
            false,
            ph2d_wet_paint::par::Rows::Parallel,
        );
        assert_eq!(
            a.flow_x, b.flow_x,
            "backrun={backrun}: flow_x divergiu entre as rotas"
        );
        assert_eq!(
            a.flow_y, b.flow_y,
            "backrun={backrun}: flow_y divergiu entre as rotas"
        );
        assert_eq!(
            a.wet, b.wet,
            "backrun={backrun}: wet divergiu entre as rotas"
        );
        assert_eq!(
            a.susp, b.susp,
            "backrun={backrun}: susp divergiu entre as rotas"
        );
        assert_eq!(
            a.sett, b.sett,
            "backrun={backrun}: sett divergiu entre as rotas"
        );
        assert_eq!(
            a.bloom, b.bloom,
            "backrun={backrun}: bloom divergiu entre as rotas"
        );
    }
}
