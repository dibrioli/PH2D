//! **O MODO GUIADO** — a gramática deixa de ser DIGITADA e passa a ser DERIVADA de meia
//! dúzia de números de forma.
//!
//! # A pergunta do Enio, e o que a medição dela respondeu
//!
//! > *"eu havia te pedido o L-System estado da arte. (…) O Blender e Houdini usam Axiom e
//! > Rules?"* (2026-08-29)
//!
//! As duas referências respondem **coisas diferentes**, e é a discordância delas que decide
//! o desenho:
//!
//! | ferramenta | como se faz uma planta lá | tem gramática à vista? |
//! |---|---|---|
//! | **Houdini** (L-System SOP) | `Premise` + `Rule 1..N`, campos de texto | **SIM** — é a mesma forma que este nó tinha |
//! | **Blender** | não tem L-System nenhum; a árvore vem do *Sapling Tree Gen* (Weber & Penn 1995) | **NÃO** — são sliders |
//! | **SpeedTree** (o padrão da indústria) | uma HIERARQUIA de geradores (Tronco → Ramo → Folha), cada um com sliders | **NÃO** |
//!
//! ⇒ *A gramática é o estado da arte do MOTOR, e não da INTERFACE.* Quem faz plantas para
//! viver não escreve `F[+F]F[-F]F`: mexe em «quantos ramos», «que ângulo», «quanto encolhe».
//! O nó tinha a metade certa e a errada trocadas.
//!
//! ⚠️ **E a cura NÃO é apagar a gramática** — ela é o que compra a samambaia, o coral, o
//! raio e o floco de Koch, formas que nenhum conjunto de sliders alcança. A cura é a lei
//! desta casa para exactamente isto: **fonte ≠ cozido** (ADR-0121). Os números são a FONTE;
//! a gramática é DERIVADA deles; e mudar para `Grammar` **assa** a derivada no texto, que
//! passa a ser a fonte a partir dali (o verbo `Detach` do ADR-0164, aplicado a texto).
//!
//! ⭐ **É por isso que assar é o momento em que se APRENDE a notação**: o artista vê a
//! gramática exacta que os sliders dele faziam, com os nomes dos params lá dentro.
//!
//! # A gramática que sai daqui referencia os params pelo NOME
//!
//! `A(s) -> F(s)![+A(s*length_scale)][-A(s*length_scale)]` — e não `s*0.9`. Uma expressão
//! vê os params do nó pelo nome, então **os sliders continuam vivos depois de assados**: o
//! *Length Scale* mexe na planta autorada tal como mexia na guiada. Um literal aqui faria a
//! conversão para `Grammar` **matar metade dos knobs** em silêncio — o defeito que o doc 90
//! caça.
//!
//! ⚠️ **`+` sozinho já quer dizer `+(angle)`** ([`crate::turtle`]: `m.arg(0)
//! .unwrap_or(set.angle)`), então os dois ramos extremos saem com o símbolo nu — que é
//! como o ABOP os escreve, e é o que o artista vai reconhecer nos exemplos que encontrar.

/// Quantos ramos, no máximo, saem de um nó.
///
/// ⚠️ **É um tecto de LEGIBILIDADE, não de recurso**, e a distinção é o §0.0: a derivação
/// aguenta muito mais (o tecto real é o [`crate::MAX_MODULES`], medido). O que satura aqui é
/// o leque: com 5 ramos a abrir `2·angle`, dois vizinhos ficam a `angle/2` um do outro, e
/// acima disso os ramos sobrepõem-se antes de o segundo nível existir. Quem quer mais
/// escreve-o na gramática — que é precisamente para isso que o modo `Grammar` fica.
pub const MAX_BRANCHES: f32 = 5.0;

/// Quantos segmentos rectos, no máximo, entre duas bifurcações.
///
/// O tronco limpo é a coisa que um L-System de brinquedo não tem: com `segments = 1` toda
/// aresta bifurca e a planta é um arbusto. O tecto é a mesma leitura do
/// [`MAX_BRANCHES`] — `6` já dá um tronco de `6·step` antes do primeiro ramo, e a partir
/// daí quem manda é o `Generations`.
pub const MAX_SEGMENTS: f32 = 6.0;

/// Quanto a variação abre (e fecha) o ângulo nas duas regras alternativas.
///
/// ⚠️ **A variação é ESTOCÁSTICA e não ruído por-ramo**, porque é isso que o substrato tem:
/// a dimensão aleatória de um L-System são **pesos de regra** (ABOP §1.7), e o motor já a
/// tem. Uma regra alternativa com o ângulo `1 ± 0,55` é o que a torna visível de longe sem
/// desmontar a silhueta — a `0,25` a planta lê-se igual, a `1,0` ela desfaz-se.
const JITTER: f32 = 0.55;

/// O axioma do modo guiado — o mesmo de fábrica, e a igualdade é um gate.
pub const AXIOM: &str = crate::DEFAULT_AXIOM;

/// Os números de FORMA, lidos do painel.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    /// Quantos rebentos saem de cada nó (1 = um caule que só se estende).
    pub branches: f32,
    /// Quantos `F` seguidos antes de bifurcar.
    pub segments: f32,
    /// `0` = toda a planta obedece à mesma regra; `1` = nenhum nó usa o ângulo nominal.
    pub variation: f32,
    /// Uma curvatura constante por segmento, em graus. `0` = ramos rectos.
    pub bend: f32,
}

/// Uma contagem vinda do painel, coagida à faixa `1..=max`.
///
/// ⚠️⚠️ **O `NaN` tem de ser apanhado ANTES do `clamp`, e o gate apanhou-me nisto.** O
/// `f32::clamp` **propaga** `NaN` (é o que a IEEE manda) e `NaN as u32` **satura em 0** — ou
/// seja, um param conduzido por um fio que produzisse `NaN` dava `segments = 0`, e a regra
/// saía `A(s) -> !A(s*length_scale)`: uma planta que deriva, gasta o orçamento todo e **não
/// desenha um único traço**. *Um `clamp` não é uma coerção: ele defende as duas pontas e
/// entrega o meio intacto, `NaN` incluído.*
fn count(v: f32, max: f32) -> u32 {
    if !v.is_finite() {
        return 1;
    }
    v.round().clamp(1.0, max) as u32
}

/// Quantos ramos, coagido ao inteiro da faixa.
fn n_branches(sh: &Shape) -> u32 {
    count(sh.branches, MAX_BRANCHES)
}

/// Quantos segmentos, coagido ao inteiro da faixa.
fn n_segments(sh: &Shape) -> u32 {
    count(sh.segments, MAX_SEGMENTS)
}

/// O coeficiente do ângulo do `k`-ésimo de `n` ramos: `+1` no de fora, `−1` no do outro
/// lado, distribuídos por igual.
///
/// ⚠️ **`n − 1` no denominador, e é o que põe os EXTREMOS em `±1`.** Com `n/2` o leque
/// abriria menos do que o slider diz e o número mentiria — o artista pede `25°` e mede `20`.
fn coeff(k: u32, n: u32) -> f32 {
    if n <= 1 {
        return 0.0;
    }
    1.0 - 2.0 * k as f32 / (n - 1) as f32
}

/// O símbolo de viragem para um coeficiente do `angle`.
///
/// Os três casos redondos saem com a escrita do ABOP (`+`, `-`, nada) e os do meio com a
/// expressão explícita — que é a forma que ensina, porque diz de onde o número vem.
fn turn(c: f32) -> String {
    // ⚠️ A comparação é FROUXA de propósito: o coeficiente sai de uma divisão inteira, e
    // `1 - 2·2/4` é exactamente `0` em `f32`, mas `1 - 2·1/3` não é `1/3` exacto. Um
    // `== 0.0` estrito faria o ramo do meio de um leque de 5 sair como `+(angle*0.000)`.
    const NEAR: f32 = 1e-4;
    if c.abs() < NEAR {
        String::new()
    } else if (c - 1.0).abs() < NEAR {
        "+".to_string()
    } else if (c + 1.0).abs() < NEAR {
        "-".to_string()
    } else if c > 0.0 {
        format!("+(angle*{c:.3})")
    } else {
        format!("-(angle*{:.3})", -c)
    }
}

/// O sucessor de uma regra, com os ângulos do leque multiplicados por `mul`.
fn body(sh: &Shape, mul: f32) -> String {
    let mut s = String::new();
    let bend = sh.bend.abs() > 1e-4;
    for _ in 0..n_segments(sh) {
        s.push_str("F(s)");
        if bend {
            // Pelo NOME: o slider *Bend* fica vivo mesmo depois de assado.
            s.push_str("+(bend)");
        }
    }
    // ⚠️ **O `!` vem DEPOIS do desenho**: ele afina a espessura para o que vem a seguir, e
    // pô-lo antes faria o tronco nascer já fino. É a mesma ordem da gramática de fábrica.
    s.push('!');
    let n = n_branches(sh);
    if n <= 1 {
        // Um caule: nada de parênteses rectos, senão a cadeia empilha um ramo por geração
        // sem nunca voltar — e um `[` que nunca fecha é memória gasta a não desenhar nada.
        s.push_str("A(s*length_scale)");
        return s;
    }
    for k in 0..n {
        s.push('[');
        s.push_str(&turn(coeff(k, n) * mul));
        s.push_str("A(s*length_scale)");
        s.push(']');
    }
    s
}

/// **As regras DERIVADAS da forma** — uma, ou três se houver variação.
///
/// ⚠️ **Três e não duas**: a variação tem de abrir E fechar o ângulo. Só uma alternativa
/// mais aberta enviesaria a planta inteira para fora, e o artista leria isso como *"o
/// Variation também abre o leque"* — que é uma segunda resposta a uma pergunta que o slider
/// *Angle* já responde.
#[must_use]
pub fn rules(sh: &Shape) -> String {
    let v = sh.variation.clamp(0.0, 1.0);
    if v <= 0.0 {
        return format!("A(s) -> {}", body(sh, 1.0));
    }
    let half = v * 0.5;
    format!(
        "A(s) -> ({:.3}) {} ; A(s) -> ({half:.3}) {} ; A(s) -> ({half:.3}) {}",
        1.0 - v,
        body(sh, 1.0),
        body(sh, 1.0 + JITTER),
        body(sh, 1.0 - JITTER),
    )
}

#[cfg(test)]
#[path = "shape_tests.rs"]
mod tests;
