//! **O PAR λ|μ — o Smooth da LITERATURA, e o que ele NÃO muda.**
//!
//! A sonda irmã (`measure_smooth_shrinkage.rs`) mediu o defeito e a cura antes
//! de uma linha ser escrita; aqui moram os gates que os prendem ao produto.
//!
//! ⚠️ **O oráculo é o RAIO MÉDIO de uma esfera unitária**, que vale `1,0` por
//! construção — todo desvio dele é o encolhimento e nada mais. Um volume por
//! shoelace 3D pediria a orientação das faces e mediria a tesselação junto.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, Pass, RefMode, SculptStroke, Symmetry, Verb};

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
/// `S = 1,8062 %` · `L = −0,0206 %` ⇒ **87,7×**.
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
