//! **O PAR λ|μ — o Smooth da LITERATURA, e o que ele NÃO muda.**
//!
//! A sonda irmã (`measure_smooth_shrinkage.rs`) mediu o defeito e a cura antes
//! de uma linha ser escrita; aqui moram os gates que os prendem ao produto.
//!
//! ⚠️ **O oráculo é o RAIO MÉDIO de uma esfera unitária**, que vale `1,0` por
//! construção — todo desvio dele é o encolhimento e nada mais. Um volume por
//! shoelace 3D pediria a orientação das faces e mediria a tesselação junto.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{
    Brush, Dab, Falloff, Pass, RefMode, SculptStroke, Symmetry, TAUBIN_LAMBDA, TAUBIN_MU,
    TAUBIN_PASS_BAND, Verb,
};

/// A função de transferência do par: o ganho que UM dab dá à frequência `k`.
fn transfer(k: f32) -> f32 {
    (1.0 - TAUBIN_LAMBDA * k) * (1.0 - TAUBIN_MU * k)
}

/// **O `λ` É DERIVADO DO ESPECTRO, e o gate afirma a derivação — não o valor.**
///
/// `1/λ = 2` põe o zero do primeiro fator no topo do espectro do laplaciano, e
/// `k = 2` é o **padrão alternado — a ruga de UM vértice**, exactamente o que um
/// artista está a alisar. O zero é **EXACTO em `f32`** (`0.5 * 2.0` é `1.0` sem
/// arredondamento), então a asserção é `== 0.0` e não um épsilon.
///
/// ⚠️ **Escrito assim, e não `assert_eq!(TAUBIN_LAMBDA, 0.5)`, de propósito:**
/// um gate que repete o literal só sabe dizer *"alguém mudou o número"*; este diz
/// **por que ele é esse**, e é o que impede a próxima wave de o mover para um
/// valor bonito sem reconferir o espectro. O valor anterior — `0,33`, uma
/// descrição (*"o Smooth de sempre com um terço do peso"*) e não uma derivação —
/// falha aqui com `f(2) = 0,647`: ele guardava **65 % da ruga** a cada par, que
/// é o *"quase imperceptível"* que o smoke reportou.
#[test]
fn the_lambda_annihilates_the_one_vertex_ripple() {
    assert_eq!(
        transfer(2.0),
        0.0,
        "λ = {TAUBIN_LAMBDA} tem de zerar f(2): o zero de (1 − λk) é k = 1/λ, e \
         o topo do espectro do laplaciano é 2"
    );
    // O CONTROLE: um filtro que zerasse TUDO não é um passa-baixa, é uma
    // borracha. A banda de passagem tem de sobreviver.
    let pb = transfer(TAUBIN_PASS_BAND);
    assert!(
        (pb - 1.0).abs() < 1e-5,
        "a banda de passagem tem de atravessar intacta: f({TAUBIN_PASS_BAND}) = {pb}"
    );
}

/// **O PAR NÃO AMPLIFICA NENHUMA FREQUÊNCIA** — o critério de estabilidade do
/// próprio paper, e o único teto legítimo que este número tem.
///
/// Se algum `k` da banda de corte tem `|f(k)| > 1`, aquela frequência é
/// **devolvida com juros** a cada dab e o traço explode. A fronteira está medida
/// por bisseção em **λ = 0,699984** (`tests/measure_taubin_lambda.rs`); o
/// produto usa `0,5`, a **71 %** dela.
///
/// ⚠️ **A margem não é conservadorismo, é o espectro EFECTIVO:** a fronteira sai
/// do espectro ideal `[0, 2]`, e no operador por cotangentes um triângulo mal
/// formado pode dar peso de aresta negativo e empurrar o espectro para além de 2
/// — o que move a fronteira para BAIXO, de malha para malha. O pico medido de
/// suavização (λ = 0,65) fica a 93 % de um penhasco cuja posição exacta depende
/// da malha, e é por isso que ele não é o ponto de operação.
#[test]
fn the_pair_attenuates_the_whole_stop_band_and_amplifies_nothing() {
    let mut k = TAUBIN_PASS_BAND;
    let mut worst = 0.0f32;
    let mut worst_k = 0.0f32;
    while k <= 2.0 {
        let g = transfer(k).abs();
        if g > worst {
            worst = g;
            worst_k = k;
        }
        k += 1e-3;
    }
    assert!(
        worst <= 1.0 + 1e-4,
        "o par AMPLIFICA k = {worst_k} por {worst}× — acima de 1 a ruga volta com \
         juros a cada dab (λ = {TAUBIN_LAMBDA}, μ = {TAUBIN_MU})"
    );
}

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)
}

fn mean_radius(mesh: &Mesh) -> f64 {
    let p = mesh.positions();
    p.iter()
        .map(|v| f64::from(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2]))).sqrt())
        .sum::<f64>()
        / p.len() as f64
}

/// Um dab que cobre a esfera INTEIRA — o regime do *Filter Layer*, onde o
/// encolhimento é o efeito e não um detalhe de borda.
fn whole_sphere_dab() -> Dab {
    Dab::at([0.0, 0.0, 0.0], 4.0, [0.0, 0.0, -1.0])
}

/// ⚠️ **`Falloff::Constant` de propósito:** com uma curva macia o peso cairia
/// com a distância e o número falaria do FALLOFF junto. O que se mede é o
/// operador.
fn smooth(mode: RefMode) -> Brush {
    Brush {
        verb: Verb::Smooth,
        mode,
        radius: 4.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

/// O encolhimento, em POR CENTO, depois de `n` dabs do MESMO gesto.
fn shrinkage_after(mode: RefMode, n: usize) -> f64 {
    let mut mesh = sphere();
    let r0 = mean_radius(&mesh);
    let b = smooth(mode);
    for _ in 0..n {
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
    }
    (r0 - mean_radius(&mesh)) / r0 * 100.0
}

/// **A ENTREGA DA WAVE: o mesmo gesto, os dois modos.**
///
/// ⚠️ **O oráculo é a RAZÃO e não um número absoluto**, e a escolha é medida,
/// não estética: as duas colunas são **lineares no número de dabs** (o `S`
/// encolhe 0,0894 %/dab, o `L` cresce 0,00102 %/dab), então uma barra absoluta
/// falaria do `n` da fixture e não da lei. A razão é constante em todo `n` — é
/// ela a propriedade.
///
/// ⚠️ **E o `S` é o CONTROLE, medido na MESMA corrida:** se um dia o laplaciano
/// ficar mais suave, os dois lados andam juntos e o gate continua a falar da
/// mesma coisa. Sem ele, um `L` que não faz nada passaria por um `L` que cura.
///
/// Medido (esfera unitária 64×128, `Constant`, força 1, 20 dabs):
/// `S = 1,8062 %` · `L = −0,0373 %` ⇒ **48,4×**.
///
/// ⚠️ **Este número já esteve ERRADO neste doc, e a causa é a que este repo
/// documenta:** ele citou `−0,0206 % ⇒ 87,7×` — a medição de ANTES do operador
/// por cotangentes — durante uma wave inteira, porque eu atualizei a tabela do
/// cabeçalho do módulo e não a medição citada AQUI. *Um doc que cita um número
/// medido tem de ser re-medido por quem move o número*, e é por isso que a sonda
/// `measure_smoothing_power.rs::the_drift_table_the_gate_cites` existe: ela
/// reproduz a fixture de um gate para a tabela dele não envelhecer sozinha.
#[test]
fn the_literature_smooth_holds_the_radius_where_the_reference_one_shrinks() {
    let s = shrinkage_after(RefMode::S, 20);
    let l = shrinkage_after(RefMode::L, 20);
    assert!(
        s > 1.0,
        "o CONTROLE tem de encolher — sem isso o gate não fala de nada: S={s:.4}%"
    );
    // ⚠️ **O SINAL do `L` inverte** (o `μ` sobre-corrige e a esfera CRESCE), e é
    // por isso que a comparação é em módulo: o que a wave entrega é a
    // magnitude da deriva, não a direção dela.
    assert!(
        s.abs() > l.abs() * 40.0,
        "o par λ|μ tem de cortar a deriva por ao menos 40×: S={s:.4}% L={l:.4}%"
    );
}

/// **UM DAB É UM PAR — e é a única parte ESTRUTURAL da wave.**
///
/// ⚠️ **Sem ela a feature fica verde nos gates de unidade e MORTA no barro:**
/// se o λ e o μ se alternassem por DAB em vez de dentro de um, um traço de `N`
/// dabs seria uma sequência `λ μ λ μ …` cujo primeiro passo fica sem par — e um
/// gesto de UM dab (o *Filter Layer*, um clique) seria `λ` puro, encolhendo
/// exatamente como o `S` com um terço da força.
///
/// O gate mede o gesto MAIS CURTO que existe, que é onde a diferença entre as
/// duas leituras é máxima.
#[test]
fn a_single_dab_already_carries_both_halves_of_the_pair() {
    let s = shrinkage_after(RefMode::S, 1);
    let l = shrinkage_after(RefMode::L, 1);
    assert!(s > 0.01, "o CONTROLE tem de encolher num dab só: S={s:.4}%");
    assert!(
        l.abs() < s * 0.1,
        "um dab do L tem de trazer o μ junto: L={l:.4}% contra S={s:.4}%"
    );
}

/// **TODO PINCEL QUE NÃO É O `L` DO SMOOTH É EXATAMENTE UM PASSE — ELE
/// PRÓPRIO.**
///
/// ⚠️ **É esta linha que torna o resto do motor byte-idêntico POR CONSTRUÇÃO:**
/// `x * 1.0 == x` no IEEE-754 para todo finito, então um pincel de um passe
/// atravessa o laço com os mesmos bits que antes da wave. A prova de que o
/// plumbing não moveu nada são os **166 gates** da crate, que passam sem uma
/// linha de fixture mudada — este aqui é a AFIRMAÇÃO, eles são a evidência.
#[test]
fn every_brush_but_the_literature_smooth_is_exactly_one_pass_itself() {
    let sole = [Pass { weight: 1.0 }];
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            let brush = Brush {
                verb,
                mode,
                ..Brush::default()
            };
            let taubin = verb == Verb::Smooth && mode == RefMode::L;
            if taubin {
                let p = brush.passes();
                assert_eq!(p.len(), 2, "o par λ|μ");
                assert!(p[0].weight > 0.0, "λ contrai");
                assert!(
                    p[1].weight < -p[0].weight,
                    "|μ| > λ é a condição do paper: λ={} μ={}",
                    p[0].weight,
                    p[1].weight
                );
            } else {
                assert_eq!(
                    brush.passes(),
                    &sole,
                    "{} × {}: um passe, ele próprio",
                    verb.label(),
                    mode.label()
                );
            }
        }
    }
}

/// **O PAR NÃO DOBRA A JANELA PUBLICADA.**
///
/// ⚠️ **A janela é o que a GPU RE-LÊ**, e ela sai do passe 0 — o conjunto que o
/// dab tocou. Um passe posterior que empurrasse para a mesma lista a duplicaria,
/// e o upload incremental passaria a subir cada vértice duas vezes: um custo que
/// nenhum pixel mostra e que nenhum gate de aparência pode ver.
#[test]
fn the_pair_publishes_one_window_and_not_two() {
    let mut a = sphere();
    let mut b = sphere();
    let mut sa = SculptStroke::default();
    let mut sb = SculptStroke::default();
    sa.begin(&a);
    sb.begin(&b);
    let n_s = sa.dab(
        &mut a,
        &smooth(RefMode::S),
        &whole_sphere_dab(),
        Symmetry::default(),
    );
    let n_l = sb.dab(
        &mut b,
        &smooth(RefMode::L),
        &whole_sphere_dab(),
        Symmetry::default(),
    );
    assert_eq!(n_s, n_l, "o par é UM dab: a contagem é a mesma pegada");
    assert_eq!(
        sa.last_moved().len(),
        sb.last_moved().len(),
        "a janela publicada não pode dobrar com o número de passes"
    );
}

/// **A TABELA que os gates acima citam** — os números, pela porta do produto.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn the_numbers_the_gates_assert() {
    println!("\n  dabs        S            L");
    println!("  ----   ----------   ----------");
    for n in [1, 5, 10, 20, 40] {
        println!(
            "  {n:>4}   {:>9.4}%   {:>9.4}%",
            shrinkage_after(RefMode::S, n),
            shrinkage_after(RefMode::L, n)
        );
    }
}

/// **O PREÇO DO PAR** — um dab do `L` contra um do `S`, pela porta do produto.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn what_the_pair_costs() {
    for mode in [RefMode::S, RefMode::L] {
        let mut mesh = sphere();
        let b = smooth(mode);
        // Aquece: a primeira consulta paga o octree e o first-touch.
        for _ in 0..3 {
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
        }
        let mut best = f64::MAX;
        for _ in 0..9 {
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            let t = std::time::Instant::now();
            let n = s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            best = best.min(t.elapsed().as_secs_f64() * 1e3);
            assert!(n > 0);
        }
        println!("  {} : {best:>7.3} ms/dab", mode.label());
    }
}
