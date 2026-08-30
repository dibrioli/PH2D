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
pub(crate) fn stacked(inner: &Tree, mods: &[Unary], local: crate::bounds::Ball) -> Tree {
    let mut acc = inner.clone();
    // ⭐⭐ **O bordo anda AO LADO da árvore** (2026-08-30) — a torção precisa de saber quão longe do
    // eixo a peça chega **naquele ponto da pilha**, e um `Array` antes dela muda essa resposta.
    // ⚠️ A lei de cada passo é a do [`crate::bounds::step_mod`], e não uma segunda cópia dela.
    let mut ball = local;
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
        ball = crate::bounds::step_mod(ball, *m);
    }
    acc
}

/// Quão longe do **eixo Z local** a peça chega — o `R` de que a torção tira o divisor.
///
/// ⚠️ O centro de uma bola pode estar fora do eixo (um `Array` empurra-o), e o que conta é o ponto
/// mais distante: `‖(cx, cy)‖ + raio`.
#[allow(dead_code)]
pub(crate) fn axis_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].hypot(b.center[1]) + b.radius.max(0.0))
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

/// ⭐⭐⭐ **A TORÇÃO (twist)** — o segundo operador de espaço deste módulo, e o irmão do [`taper`].
///
/// O ponto vai para o espaço **não torcido** rodando `(x, y)` por `−k·z`, e o valor volta como está:
/// ao contrário da inclinação, cada fatia de `z` sofre uma **rotação**, que é uma isometria — não há
/// escala para desfazer.
///
/// # ⚠️ Onde ela deixa de ser uma distância, e o tecto EXACTO disso
///
/// O jacobiano do mapa inverso tem as duas primeiras colunas ortonormais e a terceira igual a
/// `(k·q_y, −k·q_x, 1)` — o termo que a rotação ganha por variar com `z`. Com `t = k·r`
/// (`r = √(x²+y²)`, que a rotação preserva), a matriz `JᵀJ` restringida ao plano que importa é
/// `[[1, t], [t, 1 + t²]]`, e o maior valor singular sai em forma fechada:
///
/// ```text
/// σ_max(J) = t/2 + √(1 + t²/4)
/// ```
///
/// ⚠️ **E ele é MAIOR do que o `√(1 + t²)` que a intuição sugere** — `1,618` contra `1,414` em
/// `t = 1`. *A derivação à mão do irmão já tinha sido refutada uma vez por medir a coisa errada; aqui
/// a álgebra fecha, e a tabela confirma.*
///
/// Dividir por `σ_max` **pontualmente** torna o campo um minorante onde o tecto é justo, mas o
/// divisor varia com o ponto e a derivada dele reentra em `∇g` — exactamente o que fez o `1 + |s|`
/// do [`taper`] falhar. Por isso o divisor leva um [`TWIST_SAFETY`], e ele é **medido**.

pub(crate) fn twist(inner: &Tree, k: f64) -> Tree {
    twist_with(inner, k, TWIST_SAFETY)
}

/// A mesma lei com o divisor CONSTANTE — ver a varredura em `measure_twist_cost`.
///
/// ⚠️ **Constante e não pontual, e a diferença é medida:** um divisor que varia com o ponto reentra
/// em `∇(f/d) = ∇f/d − f·∇d/d²`, e o segundo termo cresce com o próprio divisor — subir a margem
/// PIORAVA o gradiente (`1,00 → 1,78` contra `4,00 → 2,55` a uma volta por unidade). Uma constante
/// não tem gradiente, e o tecto passa a ser exactamente `σ_max(k·R)/c`.
pub(crate) fn twist_const(inner: &Tree, k: f64, divisor: f64) -> Tree {
    if k == 0.0 || !k.is_finite() {
        return inner.clone();
    }
    let angle = Tree::z() * Tree::constant(-k);
    let (c, s) = (angle.clone().cos(), angle.sin());
    let (x, y) = (Tree::x(), Tree::y());
    let untwisted = inner.remap_xyz(
        x.clone() * c.clone() - y.clone() * s.clone(),
        x * s + y * c,
        Tree::z(),
    );
    untwisted / Tree::constant(divisor.max(1.0))
}

/// O tecto espectral do jacobiano do mapa inverso da torção, em `t = k·r`. Ver [`twist`].
#[must_use]
pub(crate) fn twist_sigma(t: f64) -> f64 {
    t * 0.5 + (1.0 + t * t * 0.25).sqrt()
}

/// A mesma lei com o divisor PONTUAL — a porta que a varredura refutou. Ver [`TWIST_SAFETY`].
pub(crate) fn twist_with(inner: &Tree, k: f64, safety: f64) -> Tree {
    if k == 0.0 || !k.is_finite() {
        return inner.clone();
    }
    let angle = Tree::z() * Tree::constant(-k);
    let (c, s) = (angle.clone().cos(), angle.sin());
    let (x, y) = (Tree::x(), Tree::y());
    let untwisted = inner.remap_xyz(
        x.clone() * c.clone() - y.clone() * s.clone(),
        x.clone() * s + y.clone() * c,
        Tree::z(),
    );
    // `t = k·r`, com o `r` do ponto — a rotação preserva-o, então tanto faz ler antes ou depois.
    let t = crate::ops::safe_sqrt(x.square() + y.square()) * Tree::constant(k.abs());
    let sigma = t.clone() * Tree::constant(0.5)
        + crate::ops::safe_sqrt(Tree::constant(1.0) + t.square() * Tree::constant(0.25));
    untwisted / (Tree::constant(1.0) + (sigma - Tree::constant(1.0)) * Tree::constant(safety))
}

/// Quanto o excesso do tecto da torção é multiplicado antes de dividir — **medido**.
///
/// ⚠️ Escrito como `1 + SAFETY·(σ − 1)` e não como `SAFETY·σ`: o operador tem de degenerar na
/// **identidade ao bit** em `k = 0` (`σ = 1`), senão toda peça já gravada muda de forma no dia em que
/// o modificador nascer.
const TWIST_SAFETY: f64 = 1.0;

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
