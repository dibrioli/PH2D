//! ⭐ **A INCLINAÇÃO (draft/taper)** — a lei, os dois pisos e o divisor.
//!
//! # Por que um módulo irmão
//!
//! Ela é o **primeiro** operador deste módulo que não devolve uma distância exacta, e por isso
//! carrega três medições por extenso: por que o campo não pode ser exacto, de onde sai a constante
//! de segurança, e — desde 2026-08-31 — por que o piso do `k` é da **peça** e não um épsilon. Com as
//! três escritas o `stack.rs` passava dos **700** do gate de LOC da workspace.
//! ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist.**

use fidget::context::Tree;

/// ⭐⭐⭐ **O PISO DO `k` DA INCLINAÇÃO** — o valor que a própria peça produz no ápice dela.
///
/// ⚠️ **A bola é a LOCAL, e é essa a escolha:** a corrente já vem inflada pelos deformadores
/// anteriores, e é justamente aí que o `k` ia a negativo. A local diz até onde a **matéria
/// autorada** chega, e dentro dela o `k` nunca desce abaixo deste número — logo o piso não toca no
/// material, só congela o que está para lá dele.
///
/// ⚠️ **O [`TAPER_FLOOR`] fica como última rede**, para o caso em que a própria peça já passa do
/// ápice (`slope·alcance > 1`): ali não há valor positivo que a matéria produza, e a única coisa a
/// fazer é não inverter.
pub(crate) fn taper_floor(slope: f64, b: crate::bounds::Ball) -> f64 {
    let alcance = f64::from(b.center[1].abs() + b.radius.max(0.0));
    slope.abs().mul_add(-alcance, 1.0).max(TAPER_FLOOR)
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
/// ela **vira do avesso** — a peça sairia com o interior para fora. Preso ao piso, o que acontece
/// além do ápice é a secção ficar congelada nele, que é uma forma e não um defeito.
///
/// # ⛔⛔⛔ E o piso ERA UM ÉPSILON, não uma medida (2026-08-31)
///
/// Ele era [`TAPER_FLOOR`] — `0,01`, escolhido —, e ali `σ = 1/k = 100`. O divisor cobre
/// `1 + 2·|slope|·alcance`, que numa peça normal dá `2,2`: **24× a menos**. Enquanto o envelope era
/// pequeno o `k` nunca lá chegava e ninguém pagava; com dois deformadores antes dele o envelope
/// cresce e o `k` **vai a NEGATIVO dentro do recorte**.
///
/// Medido (caixa `0,35³`, `slope 0,6`, sonda a `40³`):
///
/// | pilha | raio do envelope | `k` mínimo | `‖∇f‖` |
/// |---|---:|---:|---:|
/// | `[Taper]` | `0,78` | `0,53` | `0,78` |
/// | `[Taper, Twist, Taper]` | `1,14` | `0,31` | **`1,88`** |
/// | `[Bend, Twist, Taper]` | `2,59` | **`−0,55`** ⇒ o piso | **`2,21`** |
///
/// ⭐ **A cura é a mesma da dobra: o piso é da PEÇA.** Ele passa a ser o `k` no ápice da bola
/// **local** — o que a matéria de facto produz —, e é isso que o torna gratuito: dentro da peça o
/// `k` nunca esteve abaixo dele, logo **o material sai bit a bit igual**. O que muda é só a região
/// de fora, onde a secção já estava congelada — noutro valor.
pub(crate) fn taper(inner: &Tree, slope: f64, piso: f64) -> Tree {
    if slope == 0.0 || !slope.is_finite() {
        return inner.clone();
    }
    let k = (Tree::constant(1.0) + Tree::y() * Tree::constant(slope)).max(piso);
    let shrunk = inner.remap_xyz(Tree::x() / k.clone(), Tree::y(), Tree::z() / k.clone());
    // ⚠️ **A divisão saiu daqui e é feita UMA vez no fim da pilha** — ver [`stacked`], e a medição
    // que a obrigou. O factor continua a ser este, e continua a ser dele.
    shrunk * k
}

/// Por quanto a inclinação divide — ver [`TAPER_SAFETY`] e o doc do [`taper`].
///
/// ⛔⛔ **O `alcance` entrou em 2026-08-30, e ele faltava desde a W18.** A tabela que escolheu o
/// `TAPER_SAFETY` foi medida numa peça **centrada e de tamanho um**; o termo que ela corrige cresce
/// com a distância ao eixo `Y` (é `x·s/k²` que reentra no gradiente), logo uma peça larga — ou uma
/// **matriz** antes da inclinação — passa por cima dele. Medido: `[Array, Taper]` dava
/// `‖∇f‖ = 1,5049` **dentro da caixa de recorte**, alcançável em dois cliques.
///
/// ⚠️ **`max(1, alcance)`**: nunca menos do que a tabela original concedeu, senão a cura tornaria
/// uma peça pequena MENOS segura do que ela é hoje.
pub(crate) fn taper_divisor(slope: f64, alcance: f64) -> f64 {
    if slope == 0.0 || !slope.is_finite() {
        1.0
    } else {
        TAPER_SAFETY.mul_add(slope.abs() * alcance.abs().max(1.0), 1.0)
    }
}

/// Quão longe do **eixo Y** a peça chega — a inclinação escala `x` e `z` em torno dele.
///
/// ⚠️ Irmão do [`axis_reach`], e num eixo diferente: cada modificador nomeia o seu, que é a lei que
/// as primitivas deste módulo já seguem.
///
/// # ⛔ Apertá-lo para a CAIXA não compra nada, e carrega o risco do irmão (2026-09-01)
///
/// A `bounds::axis_distance` sabe ler a caixa, e daria `0,658` onde a esfera dá `0,780`. ⛔ Medido:
/// o divisor da inclinação **não se mexe** (`2,20` nos dois), porque o `max(1, alcance)` do
/// [`taper_divisor`] domina em qualquer peça enquadrada. ⇒ zero ganho.
///
/// ⚠️ E o preço é o mesmo que o [`axis_reach`] pagou no mesmo dia: o [`TAPER_SAFETY`] foi **medido**
/// numa peça centrada e de tamanho um, com esta esfera lá dentro. *Apertar a entrada de uma
/// constante calibrada consome a margem que a fazia bastar* — e ali o resultado foi um pixel a
/// furar. Uma mudança sem ganho e com risco não se faz.
pub(crate) fn taper_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].hypot(b.center[2]) + b.radius.max(0.0))
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
