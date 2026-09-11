//! **O FILTRO `Scale` DA REFERÊNCIA JÁ É EXPRIMÍVEL?** — a pergunta que a §8 do
//! plano manda fazer ANTES de escrever um kernel novo.
//!
//! A W9b da tabela pede três leis que a referência tem como FILTRO e nós não
//! temos como pincel: `Random`, `Sphere` e `Scale`. Esta sonda julga só a
//! terceira, porque ela é a única cuja resposta pode já estar na árvore: o
//! `MaskTransform` + [`Gesture::Scale`] (a W15) é uma homotetia **ponderada pela
//! máscara**, que é a definição do filtro.
//!
//! **As duas leis, lado a lado:**
//!
//! | | centro | peso |
//! |---|---|---|
//! | referência (`calc_scale_filter`) | a **ORIGEM do objeto** (`t = orig_positions × f`) | `p·(1 + w·f)`, **linear** |
//! | nós (`Gesture::Scale`) | o **CENTROIDE PONDERADO** do que se move | `pivot + (p−pivot)·s^w`, **exponencial** |
//!
//! ⚠️ **A diferença de PESO não é um defeito nosso, e a nossa é a que compõe:**
//! `s^0 = 1`, `s^1 = s`, e dois gestos seguidos dão o PRODUTO dos fatores — que
//! é o que uma escala É. A da referência é o primeiro termo da nossa (elas
//! concordam em `w ∈ {0, 1}` e em `f → 0`), e a sonda mede onde ela diverge.
//!
//! ⚠️ **A diferença de CENTRO é REAL e é a que decide a wave**, porque não é
//! uma aproximação de nada: escalar em torno da origem do objeto e em torno do
//! centro do que está livre são operações diferentes assim que as duas coisas
//! não coincidem — e é exatamente por isso que a sonda mede a peça DESLOCADA
//! como caso separado.
//!
//! Ela **imprime e não afirma**. O que ela produz é o número com que o Enio
//! decide se a W9b constrói um `FilterKind::Scale` ou se a linha da tabela é
//! fechada por composição, como o `platform_floor_layers` da física foi.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_scale_filter --release
//! -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes::uv_sphere};
use ph2d_sculpt3d::{Gesture, MaskTransform};

/// A lei da referência, `calc_scale_filter` + `apply_translations`:
/// `t = orig_positions × (máscara_livre × strength)`, somada à pose congelada.
///
/// ⚠️ **Escrita à mão de propósito** — chamar a nossa seria o oráculo-espelho
/// que esta casa já documentou: ela devolveria a nossa resposta com outro nome.
fn reference_scale(pre: &[[f32; 3]], free: &[f32], strength: f32) -> Vec<[f32; 3]> {
    pre.iter()
        .zip(free)
        .map(|(p, &w)| {
            let f = w * strength;
            [p[0] + p[0] * f, p[1] + p[1] * f, p[2] + p[2] * f]
        })
        .collect()
}

/// A pose que o nosso gesto produz, para o MESMO `strength`.
///
/// O casamento é o de primeira ordem: em `w = 1` a referência multiplica por
/// `1 + f`, então é esse o `factor` que se pede ao gesto.
fn ours_scale(mesh: &Mesh, strength: f32) -> Vec<[f32; 3]> {
    let mut m = mesh.clone();
    let mut session = MaskTransform::begin(&m).expect("nada livre para mover");
    session.apply(
        &mut m,
        &Gesture::Scale {
            factor: 1.0 + strength,
        },
    );
    m.positions().to_vec()
}

/// A malha da sonda: uma esfera com uma **máscara em gradiente**, para o peso
/// variar de verdade — com a máscara cheia ou vazia as duas leis coincidem em
/// `w ∈ {0, 1}` e a comparação seria vácua.
fn fixture(offset: f32) -> Mesh {
    let mut m = uv_sphere(24, 32, 1.0);
    let n = m.vert_count();
    if offset != 0.0 {
        for p in m.positions_mut() {
            p[0] += offset;
        }
    }
    let ys: Vec<f32> = m.positions().iter().map(|p| p[1]).collect();
    let masks = m.masks_mut();
    for i in 0..n {
        // 0 em baixo (livre), 1 em cima (pregado), rampa no meio.
        masks[i] = ((ys[i] + 1.0) * 0.5).clamp(0.0, 1.0);
    }
    m
}

fn worst(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// O tamanho do que a lei move — a régua contra a qual o desvio é lido.
fn worst_travel(pre: &[[f32; 3]], post: &[[f32; 3]]) -> f32 {
    worst(pre, post)
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma"]
fn measure_whether_our_transform_already_is_the_scale_filter() {
    println!("\n=== O CENTRO: a peça na ORIGEM (onde as duas leis podem coincidir) ===");
    for &s in &[0.05_f32, 0.2, 0.5, 1.0] {
        let m = fixture(0.0);
        let free: Vec<f32> = m.masks().unwrap().iter().map(|&x| 1.0 - x).collect();
        let pre = m.positions().to_vec();
        let r = reference_scale(&pre, &free, s);
        let o = ours_scale(&m, s);
        let d = worst(&r, &o);
        let t = worst_travel(&pre, &r);
        println!(
            "  strength {s:>4} | referência anda {t:>7.4} | desvio {d:>7.4} = {:>5.1}% do movimento",
            if t > 0.0 { 100.0 * d / t } else { 0.0 }
        );
    }

    println!("\n=== O CENTRO: a peça DESLOCADA 3 unidades em X ===");
    println!("  (a origem do objeto deixa de ser o centro do que se move)");
    for &s in &[0.05_f32, 0.2, 0.5] {
        let m = fixture(3.0);
        let free: Vec<f32> = m.masks().unwrap().iter().map(|&x| 1.0 - x).collect();
        let pre = m.positions().to_vec();
        let r = reference_scale(&pre, &free, s);
        let o = ours_scale(&m, s);
        let d = worst(&r, &o);
        let t = worst_travel(&pre, &r);
        println!(
            "  strength {s:>4} | referência anda {t:>7.4} | desvio {d:>7.4} = {:>5.1}% do movimento",
            if t > 0.0 { 100.0 * d / t } else { 0.0 }
        );
    }

    println!("\n=== O PESO isolado: mesmo centro, só a lei do expoente ===");
    println!("  linear `1 + w·f` contra exponencial `(1+f)^w`, por peso");
    for &s in &[0.2_f32, 1.0] {
        print!("  strength {s:>4} |");
        for &w in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let lin = 1.0 + w * s;
            let exp = (1.0 + s).powf(w);
            print!(" w={w:.2}: {:+.2}%", 100.0 * (exp - lin) / lin);
        }
        println!();
    }

    println!("\n=== O CONTROLE: com a máscara VAZIA as duas TÊM de coincidir ===");
    let mut m = uv_sphere(24, 32, 1.0);
    let n = m.vert_count();
    for i in 0..n {
        m.masks_mut()[i] = 0.0;
    }
    let free = vec![1.0_f32; n];
    let pre = m.positions().to_vec();
    let r = reference_scale(&pre, &free, 0.5);
    let o = ours_scale(&m, 0.5);
    println!("  desvio com máscara vazia: {:.6e}", worst(&r, &o));
}
