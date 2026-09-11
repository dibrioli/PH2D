//! Sonda (`#[ignore]`): qual é a exactidão do par `exp`/`ln` DESTE avaliador?
use fidget::context::Tree;
use ph2d_field_eval::Field;

#[test]
#[ignore]
fn probe_exp_ln_accuracy() {
    // `exp(ln(x))` devia ser `x`.
    let ida_volta = Tree::x().ln().exp();
    let f = Field::from_tree(&ida_volta);
    let mut pior = 0.0_f64;
    for i in 1..400 {
        let x = f64::from(i) * 0.01;
        let e = ((f.at(x, 0.0, 0.0) - x) / x).abs();
        pior = pior.max(e);
    }
    println!("  exp(ln(x)) contra x: pior erro RELATIVO = {pior:.3e}");

    // `exp(ln(x)/2)` devia ser `sqrt(x)` — o que a norma-2 faz.
    let raiz = (Tree::x().ln() * Tree::constant(0.5)).exp();
    let f = Field::from_tree(&raiz);
    let mut pior = 0.0_f64;
    for i in 1..400 {
        let x = f64::from(i) * 0.01;
        let v = x.sqrt();
        pior = pior.max(((f.at(x, 0.0, 0.0) - v) / v).abs());
    }
    println!("  exp(ln(x)/2) contra sqrt(x): pior erro RELATIVO = {pior:.3e}");

    // E o `sqrt` nativo da árvore, para comparar.
    let f = Field::from_tree(&Tree::x().sqrt());
    let mut pior = 0.0_f64;
    for i in 1..400 {
        let x = f64::from(i) * 0.01;
        let v = x.sqrt();
        pior = pior.max(((f.at(x, 0.0, 0.0) - v) / v).abs());
    }
    println!("  sqrt NATIVO contra sqrt(x):  pior erro RELATIVO = {pior:.3e}");
}
