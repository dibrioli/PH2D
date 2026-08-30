//! ⭐⭐ **QUEM SE JUNTA A QUEM, E COM QUE FORMA** — os operadores booleanos e o carácter deles.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::ops`] responde a *«que forma cada primitiva é»*; este responde a *«como duas formas se
//! encontram»*. O arquivo passou as `700` linhas do gate de LOC da workspace quando o **chanfro por
//! forma** entrou (Enio, 2026-08-30). ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **Intersecção e subtração são De Morgan**, e não fórmulas próprias: `A ∩ B = ¬(¬A ∪ ¬B)`. Uma
//! fórmula a mais seria a segunda resposta à mesma pergunta — e as duas divergiriam no dia em que um
//! carácter novo nascesse.

use crate::ops::{length2, neg};
use fidget::context::Tree;
use std::f64::consts::FRAC_1_SQRT_2;

pub fn union_sharp(a: &Tree, b: &Tree) -> Tree {
    a.min(b.clone())
}

/// União com filete **exato** de raio `r`.
///
/// `max(r, min(a,b)) − ‖(max(r−a, 0), max(r−b, 0))‖`
///
/// Preserva a propriedade de distância onde `a` e `b` a têm, e por isso **o raio pedido é o raio
/// entregue** — medido a 0,00 % nos gates desta crate.
pub fn union_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    let ux = (Tree::constant(r) - a.clone()).max(0.0);
    let uy = (Tree::constant(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - length2(&ux, &uy)
}

/// União **suave** (smooth-min polinomial), alcance `k`.
///
/// ⚠️ **NÃO preserva a propriedade de distância**, e o `k` **não é um raio**: medido, entrega 3/4
/// do número pedido. Quem o levar à UI com a etiqueta "raio" mente 25 %, sempre.
pub fn union_smooth(a: &Tree, b: &Tree, k: f64) -> Tree {
    let half = Tree::constant(0.5);
    let h = (half.clone() + half * (b.clone() - a.clone()) / Tree::constant(k))
        .max(0.0)
        .min(1.0);
    let mixed = b.clone() + (a.clone() - b.clone()) * h.clone();
    mixed - Tree::constant(k) * h.clone() * (Tree::constant(1.0) - h)
}

/// ⭐⭐⭐ **União com CHANFRO de alcance `r`** (W99) — o corte reto a 45°, em vez do arco.
///
/// `min(min(a, b), (a + b − r) · √½)`
///
/// # ⚠️ Por que ela é UMA linha, e por que isso não é sorte
///
/// O plano do chanfro num canto de 90° é `a + b = r`, e a distância de um ponto a ele é
/// `(a + b − r)/√2` — **exacta**, não aproximada. O `min` com o canto vivo é o que a limita à região
/// onde ela de facto é a superfície mais próxima. *No CAD, filete e chanfro são duas máquinas com
/// modos de falha diferentes; aqui são a mesma conta com um termo trocado, e nenhuma pode falhar.*
///
/// # ⚠️ Ela é sempre um MINORANTE, e a marcha depende disso
///
/// O resultado é `min(min(a,b), …)`, logo **nunca maior** que `min(a, b)` — que já é um minorante
/// da distância à união. ⇒ andar o valor do campo continua a ser seguro, mesmo onde o termo do
/// chanfro tem gradiente acima de `1` (ele tem, quando as duas normais se alinham). *Um passo é
/// seguro porque o valor é menor que a distância, não porque o gradiente é unitário.*
pub fn union_chamfer(a: &Tree, b: &Tree, r: f64) -> Tree {
    let corte = (a.clone() + b.clone() - Tree::constant(r)) * Tree::constant(FRAC_1_SQRT_2);
    a.min(b.clone()).min(corte)
}

/// Intersecção, com o mesmo caráter de mistura da união — **por De Morgan**.
pub fn intersection(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    neg(&union(&neg(a), &neg(b), blend))
}

/// Subtração (`a` menos `b`): intersectar `a` com o complemento de `b`.
pub fn difference(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    intersection(a, &neg(b), blend)
}

/// O caráter de mistura já resolvido em número, para os três operadores partilharem um caminho só.
#[derive(Clone, Copy, Debug)]
pub enum Blended {
    Sharp,
    Exact(f64),
    /// ⭐⭐⭐ **O CHANFRO** (W99) — o corte reto a 45º. Ver [`union_chamfer`].
    Chamfer(f64),
    Organic(f64),
}

/// A união, escolhendo a fórmula pelo caráter. **Os outros dois operadores passam por aqui** — é o
/// que garante que "arredondar" signifique a mesma coisa nas três operações.
pub fn union(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    match blend {
        // ⚠️ Raio zero cai no caminho DURO de propósito: `union_round(_, _, 0.0)` seria
        // algebricamente equivalente, mas passaria por um `max`/`length` a mais em cada avaliação,
        // e o traçado avalia milhões de vezes por quadro.
        Blended::Sharp => union_sharp(a, b),
        Blended::Exact(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Chamfer(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Organic(k) if k <= 0.0 => union_sharp(a, b),
        Blended::Exact(r) => union_round(a, b, r),
        Blended::Chamfer(r) => union_chamfer(a, b, r),
        Blended::Organic(k) => union_smooth(a, b, k),
    }
}
