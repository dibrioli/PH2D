//! ⭐ **A PILHA DE MODIFICADORES** — o que um nó faz à forma dele depois de ela existir: casca,
//! afastamento, espelho, matriz, repetição radial e inclinação.
//!
//! # Por que ela saiu do `lib.rs`
//!
//! O `lib.rs` desta crate é a **ponte**: documento → árvore → malha. A pilha é uma resposta
//! completa e fechada dentro dela, com as três constantes medidas do [`taper`] ao lado — e o
//! arquivo passou dos **700** do gate de LOC da workspace. ⚠️ **A cura é partir para irmão, nunca
//! uma entrada na allowlist.**

use crate::ops;
use fidget::context::Tree;
use ph2d_field::Unary;

/// ⭐ **A pilha de modificadores de um nó**, aplicada na ordem em que ela está.
///
/// ⚠️ **A ordem importa e é por isso que ela é uma lista**: encascar-e-afastar não é afastar-e-
/// encascar. `|f| − t` seguido de `− d` dá uma parede mais grossa; `f − d` seguido de `| | − t` dá
/// uma parede da mesma espessura noutro sítio. Um conjunto sem ordem teria de escolher uma em
/// silêncio.
pub(crate) fn stacked(inner: &Tree, mods: &[Unary]) -> Tree {
    let mut acc = inner.clone();
    for m in mods {
        acc = match *m {
            // ⭐ A casca inteira: o módulo de uma distância É a distância à mesma superfície vista
            // dos dois lados, e afastá-la meia espessura para cada lado dá a parede.
            Unary::Shell { thickness } => ops::offset(&acc.abs(), f64::from(thickness) * 0.5),
            Unary::Offset { distance } => ops::offset(&acc, f64::from(distance)),
            // ⭐ **Dobra do domínio**: `x → |x|`. O que existe de um lado passa a existir dos dois, e
            // o campo continua uma distância exata — não há costura a fechar, que é o mesmo motivo
            // de a booleana e a casca não poderem falhar.
            Unary::Mirror => acc.remap_xyz(Tree::x().abs(), Tree::y(), Tree::z()),
            // ⭐ Os outros dois eixos, pela MESMA lei — ver [`ph2d_field::Unary::MirrorZ`] para a
            // cerca que caiu.
            Unary::MirrorY => acc.remap_xyz(Tree::x(), Tree::y().abs(), Tree::z()),
            Unary::MirrorZ => acc.remap_xyz(Tree::x(), Tree::y(), Tree::z().abs()),
            Unary::Array { count, spacing } => array(&acc, count, f64::from(spacing)),
            Unary::Radial { count } => radial(&acc, count),
            Unary::Taper { slope } => taper(&acc, f64::from(slope)),
        };
    }
    acc
}

/// ⭐ **A inclinação (draft/taper)** — e o **primeiro operador deste módulo que não é exato**.
///
/// A secção transversal escala por `k(y) = 1 + slope·y`: o ponto vai para o espaço não-inclinado
/// (`x/k`, `y`, `z/k`) e o valor volta multiplicado por `k` — a mesma receita de duas metades que a
/// [`place`] usa para a escala uniforme, e pela mesma razão (sem a segunda metade o campo deixa de
/// ser uma distância).
///
/// # ⚠️ Por que ele não pode ser exato, e o que se paga em vez disso
///
/// A escala **varia com `y`**, e é essa variação que estraga: `∇g` ganha um termo de ordem
/// `slope·f` que a multiplicação por `k` não cancela. Perto da superfície (`f ≈ 0`) o erro
/// desaparece — que é onde a marcha mais precisa dele —, mas longe ele **superestima**, e
/// superestimar é o erro que faz o raio saltar por cima da peça.
///
/// A cura é dividir por `1 + |slope|`, o que torna o campo um **bound conservador**: ele nunca
/// passa da distância verdadeira, e a marcha continua correta. O preço é o número de passos, e ele
/// está medido em `measure_taper_cost` — é dali que sai o
/// [`ph2d_field::mods::MAX_TAPER_SLOPE`].
///
/// ⚠️ **O piso em `k` impede a inversão.** Em `y = −1/slope` a secção colapsa e, passando disso,
/// ela **vira do avesso** — a peça sairia com o interior para fora. Preso a [`TAPER_FLOOR`], o que
/// acontece além do ápice é a secção ficar congelada nele, que é uma forma e não um defeito.
fn taper(inner: &Tree, slope: f64) -> Tree {
    if slope == 0.0 || !slope.is_finite() {
        return inner.clone();
    }
    let k = (Tree::constant(1.0) + Tree::y() * Tree::constant(slope)).max(TAPER_FLOOR);
    let shrunk = inner.remap_xyz(Tree::x() / k.clone(), Tree::y(), Tree::z() / k.clone());
    shrunk * k / Tree::constant(1.0 + TAPER_SAFETY * slope.abs())
}

/// O menor fator de secção que a inclinação admite — ver [`taper`].
///
/// ⚠️ Não é um épsilon de gosto: abaixo dele o `x/k` explode e o campo passa a devolver números que
/// a marcha lê como "muito longe" dentro da própria peça. Um centésimo é duas ordens de grandeza
/// abaixo da secção nominal, o que põe o ápice bem fora de qualquer peça enquadrada.
const TAPER_FLOOR: f64 = 0.01;

/// Quanto o divisor da inclinação cresce por unidade de declive — **medido, e a primeira tentativa
/// estava errada**.
///
/// ⚠️ A conta que eu escrevi primeiro dividia por `1 + |slope|`, derivada à mão. A sonda
/// `measure_taper_cost` **refutou-a**: `‖∇f‖` continuava acima de 1 em todo o alcance, ou seja o
/// campo **superestimava** — exatamente a falha que a divisão existe para evitar.
///
/// | declive | `‖∇f‖` máx com `1 + s` | com `1 + 2s` |
/// |---|---|---|
/// | 0,25 | **1,12** ⛔ | 0,93 ✅ |
/// | 0,50 | **1,20** ⛔ | 0,90 ✅ |
/// | 1,00 | **1,30** ⛔ | 0,87 ✅ |
/// | 2,00 | **1,40** ⛔ | 0,84 ✅ |
///
/// *Uma derivação à mão é uma hipótese; a tabela é o facto.* O `2` é o degrau que a medição deu —
/// com ele `‖∇f‖ ≤ 1` em todo o alcance, que é a condição de a marcha não atravessar a peça.
const TAPER_SAFETY: f64 = 2.0;

/// ⭐ **A matriz radial**: `count` cópias em coroa, em torno do **Z**.
///
/// A conta é a mesma ideia da linear numa coordenada diferente: em vez de dobrar o `x`, dobra-se o
/// **ângulo**. Leva-se o ponto para a fatia dele (`θ − Δ·k`, com `Δ = 2π/count`) e avalia-se **uma**
/// forma — uma coroa de 32 custa o mesmo que uma de 2.
///
/// ⚠️ **Duas fatias**, pelo mesmíssimo motivo da linear: com uma só, uma forma que transborde a
/// fatia faz o campo **superestimar**, e superestimar é o que faz a marcha de raios saltar por cima
/// da superfície. Ver [`array`], onde o mecanismo está escrito por extenso.
///
/// ⚠️ **No eixo (`x = y = 0`) não há ângulo**, e é por isso que a conta não divide por `r`: ela
/// reconstrói o ponto por `r·cos θ'` / `r·sin θ'`, e em `r = 0` isso é a origem — a resposta certa,
/// sem caso especial e sem `NaN`.
fn radial(inner: &Tree, count: u32) -> Tree {
    if count <= 1 {
        return inner.clone();
    }
    let step = std::f64::consts::TAU / f64::from(count);
    let d = Tree::constant(step);
    let r = crate::ops::safe_sqrt(Tree::x().square() + Tree::y().square());
    let theta = Tree::y().atan2(Tree::x());
    let raw = (theta.clone() / d.clone()).round();
    // A fatia vizinha é a do lado para onde o ponto pende — mesma lei da linear.
    let toward = theta.clone() / d.clone() - raw.clone();
    let other = raw.clone() + toward.compare(Tree::constant(0.0));
    let wedge = |k: Tree| {
        let t = theta.clone() - d.clone() * k;
        inner.remap_xyz(r.clone() * t.clone().cos(), r.clone() * t.sin(), Tree::z())
    };
    wedge(raw).min(wedge(other))
}

/// ⭐ **A matriz linear**: `count` cópias espaçadas de `spacing` no X, **sem N cópias da árvore**.
///
/// A conta é a dobra do domínio: leva-se o ponto para a célula dele (`x − s·k`, com `k` o índice da
/// célula preso a `[0, count−1]`) e avalia-se **uma** forma. É a razão de uma matriz de 64 custar o
/// mesmo que uma de 2 — numa malha ela custaria 64 vezes a geometria.
///
/// # ⚠️ Por que DUAS células, e não uma
///
/// A receita clássica (`opRepLim`) olha só a célula do ponto, e ela **superestima** a distância
/// quando a forma transborda a célula: existe uma cópia vizinha mais perto do que a da célula, e o
/// campo não a vê. Superestimar é o erro **caro** numa marcha de raios — o passo salta por cima da
/// superfície, e o sintoma é a peça com buracos, não um erro.
///
/// Olhar a célula do ponto **e a vizinha do lado para onde ele pende** custa duas avaliações da
/// subárvore e devolve a distância exata enquanto a forma couber em **1,5 células**. ⛔ Acima disso
/// o bound volta, e a cura é olhar três — que é o dobro do custo por um caso que o nascimento da
/// matriz (espaçamento = 2× a peça) já põe fora de alcance.
fn array(inner: &Tree, count: u32, spacing: f64) -> Tree {
    if count <= 1 || spacing <= 0.0 || !spacing.is_finite() {
        return inner.clone();
    }
    let s = Tree::constant(spacing);
    let last = f64::from(count - 1);
    // O índice da célula, preso à matriz: `clamp(round(x/s), 0, count−1)`.
    let raw = (Tree::x() / s.clone()).round();
    let k = raw.max(Tree::constant(0.0)).min(Tree::constant(last));
    // ⚠️ **A vizinha é a do lado para onde o ponto PENDE**, e não uma fixa: com o sinal errado a
    // segunda avaliação cai na mesma célula metade das vezes e o gate passaria sem nada a defender.
    let toward = Tree::x() / s.clone() - k.clone();
    let neighbour = (k.clone() + toward.compare(Tree::constant(0.0)))
        .max(Tree::constant(0.0))
        .min(Tree::constant(last));
    let cell = |idx: Tree| inner.remap_xyz(Tree::x() - s.clone() * idx, Tree::y(), Tree::z());
    cell(k).min(cell(neighbour))
}
