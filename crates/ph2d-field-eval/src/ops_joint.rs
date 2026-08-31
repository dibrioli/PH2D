//! ⭐⭐⭐ **A JUNTA DE DUAS SUPERFÍCIES** — o chanfro, o filete por cima dele, e a junta entre duas
//! CÓPIAS de uma repetição.
//!
//! > **Pedido do Enio, 2026-08-30:** *«em radial e outros modificadores que geram cópias da mesma
//! > peça não temos nem filet nem chamfer para a união entre as peças»* e *«em todas as peças temos
//! > fillet para as bordas arredondadas mas não temos um slider para chamfer. Poderíamos ter os 2,
//! > com chamfer antes de fillet»*
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::ops`] estava a `685` das `700` linhas do gate de LOC da workspace. ⛔ *Split, nunca
//! allowlist* — e o corte é por assunto: aqui está tudo o que responde *«que forma tem a junta
//! entre estas duas superfícies»*, e nada do que responde *«que forma é esta primitiva»*.
//!
//! # ⭐ Os dois pedidos são a MESMA lei, e é isso que este arquivo torna verdade
//!
//! A aresta de uma forma é a junta das **peças** dela (a parede com a tampa de um cilindro), que é
//! uma **intersecção**. A costura entre duas cópias é uma **união**. As duas são a mesma conta pelo
//! dual de De Morgan, e por isso o artista lê os mesmos dois números nos dois sítios.
//!
//! # ⛔⛔ A lei que só a medição deu: `blend(a, a) ≠ a`
//!
//! As leis de repetição desta casa **prendem** o índice da cópia vizinha — o `clamp` nas pontas de
//! uma matriz, e o `compare` que devolve `0` no centro exacto de uma célula. Nesses pontos a
//! "vizinha" **é a própria cópia**. Com `min` isso é inofensivo, porque o `min` é idempotente. Com
//! uma mistura não é, e o preço está medido em `spike_join_between_copies`: a lei crua sobe o erro
//! contra o oráculo de `0,1288` para `0,1950` conforme o raio cresce, e uma tentativa ingénua de
//! encolher o **raio** pelo portão deixa `‖∇f‖ = 1,4142` num documento sem junta nenhuma (o
//! `union_round(a, b, 0)` é o `min` **por fora** e não por dentro). ⇒ o portão **escolhe entre as
//! duas leis**, e aí o erro contra o oráculo fica em `0,000000` para todo raio.

use crate::ops::{self, Blended};
use fidget::context::Tree;
use ph2d_field::Joint;

/// ⭐⭐⭐ **OS DOIS RECUOS DE UMA ARESTA, com os campos NOMEADOS** — e o nome é a razão de o tipo
/// existir.
///
/// # ⛔⛔ Ele nasceu de um defeito que os gates apanharam, e a lição é sobre a ASSINATURA
///
/// A 1.ª versão desta família passava `(chamfer: f64, round: f64)` — dois `f64` adjacentes, do mesmo
/// tipo, com significados opostos. Uma reordenação mecânica de parâmetros trocou-os em **oito das
/// nove** chamadas, e o resultado compilou: as formas passaram a ser **chanfradas com o número do
/// filete**, e a peça saía com aresta viva onde o artista pediu arco.
///
/// ⚠️ **Cinco gates da sonda de arestas apanharam-no** (`measure_sharp_edges`), o que é a prova de
/// que valia a pena tê-los — mas o defeito **não podia** ter existido. *Dois argumentos do mesmo
/// tipo, adjacentes e trocáveis, são um erro à espera de uma refactoração;* com os campos nomeados,
/// a troca é **erro de compilação**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// O raio do arco. O que o painel chama **Fillet**.
    pub round: f64,
    /// O recuo do corte reto a 45°, ao longo de cada face. O que o painel chama **Chamfer**.
    pub chamfer: f64,
}

impl Edge {
    /// A aresta viva: os dois recuos a zero.
    pub const SHARP: Self = Self {
        round: 0.0,
        chamfer: 0.0,
    };

    /// A aresta que um [`Joint`] do documento descreve. ⚠️ **É aqui que o `f32` do documento vira o
    /// `f64` do avaliador**, para as duas famílias — a aresta de uma forma e a costura entre cópias.
    #[must_use]
    pub fn of(joint: Joint) -> Self {
        Self {
            round: f64::from(joint.fillet),
            chamfer: f64::from(joint.chamfer),
        }
    }
}

/// ⭐⭐⭐ **A INTERSECÇÃO com chanfro e depois filete** — a aresta convexa de uma forma.
///
/// A intersecção chanfrada é `max(max(a,b), (a+b+c)·√½)` — o dual de De Morgan do
/// [`ops::union_chamfer`] —, logo o `c` **é** o recuo ao longo de cada face, a mesma régua que os
/// chips de carácter partilham. **Medido** (`spike_chamfer_then_fillet`, aro de cilindro): recuo
/// pedido `0,050`/`0,100`/`0,150`/`0,200` ⇒ entregue `0,05000`/`0,10000`/`0,15000`/`0,20000` nas
/// **duas** faces, e `‖∇f‖ = 1,0000` a `ε = 1e-5`.
///
/// ⭐ O plano do chanfro é uma **terceira superfície**, e as arestas novas que ele cria são
/// `a ∩ plano` e `b ∩ plano`. Arredondá-las é o filete de sempre aplicado a essas duas juntas — e a
/// aresta velha `a ∩ b` já não está na fronteira (o plano cortou-a fora), logo o filete não lhe
/// toca.
///
/// ⚠️ **Com o filete ligado o campo passa a inflar até `√2`** (medido `1,4140`), que é o balde que o
/// [`crate::gradient_bound`] já paga por um arredondamento exacto. Quem der chanfro a uma primitiva
/// tem de o dizer ao `ph2d_field::fillet_inflates`, senão a marcha anda a passo cheio sobre um campo
/// que sobe mais depressa que a distância — e o sintoma é a peça furada.
pub fn intersection_joint(a: &Tree, b: &Tree, e: Edge) -> Tree {
    let (chamfer, fillet) = (e.chamfer, e.round);
    if chamfer <= 0.0 {
        // ⭐ **O caminho de sempre, ao bit** — nem um nó a mais na árvore.
        return ops::intersection(a, b, Blended::Exact(fillet));
    }
    let plano = corte(a, b, chamfer, Sentido::Interseccao);
    if fillet <= 0.0 {
        return a.max(b.clone()).max(plano);
    }
    // ⭐⭐⭐ **AS TRÊS DE UMA VEZ** — as duas faces e o plano do corte. Ver a nota da
    // [`ops::intersection_round_n`] para as duas construções que foram medidas e recusadas antes
    // desta (duas misturas encaixadas · encolher-chanfrar-deslocar).
    ops::intersection_round_n(&[a.clone(), b.clone(), plano], fillet)
}

/// ⭐⭐⭐ **A UNIÃO com chanfro e depois filete** — o vinco côncavo entre duas peças.
///
/// O espelho exacto da [`intersection_joint`], e a razão de as duas viverem lado a lado: um artista
/// que aprendeu os dois números numa aresta de forma lê-os iguais na costura entre duas cópias.
pub fn union_joint(a: &Tree, b: &Tree, e: Edge) -> Tree {
    let (chamfer, fillet) = (e.chamfer, e.round);
    if chamfer <= 0.0 {
        return ops::union(a, b, Blended::Exact(fillet));
    }
    let plano = corte(a, b, chamfer, Sentido::Uniao);
    if fillet <= 0.0 {
        return a.min(b.clone()).min(plano);
    }
    // ⭐ O dual exacto da [`intersection_joint`]: as três de uma vez, pela mesma razão.
    ops::union_round_n(&[a.clone(), b.clone(), plano], fillet)
}

/// ⭐⭐⭐ **AS PEÇAS DE UM SÓLIDO E OS CHANFROS DAS ARESTAS QUE ELAS FORMAM, NUMA MISTURA SÓ.**
///
/// # ⛔⛔ Ela nasceu do 3.º report do Enio sobre esta feature (2026-08-30): *«algumas arestas não arredondam no prisma»*
///
/// A [`intersection_joint`] mistura **duas** superfícies. Quando um sólido é feito em etapas — as
/// paredes de um prisma primeiro, o aro depois —, a segunda mistura recebe a **primeira já
/// composta**, e aí ela herda a costura interna dessa composta e põe-na na superfície visível.
///
/// ⚠️ **É a mesma família do defeito que o `intersection_round_n` já curava um nível acima** (duas
/// misturas encaixadas), e a cura é a mesma: *tudo entra ao mesmo tempo.*
///
/// Medido, pior giro da normal com os dois recuos a metade do limite, contra o mesmo filete sem
/// chanfro:
///
/// | forma | só filete | antes | depois |
/// |---|---:|---:|---:|
/// | caixa | `1,8°` | `29,1°` | **`2,1°`** |
/// | cilindro | `1,5°` | `19,0°` | **`1,3°`** |
/// | prisma | `4,2°` | `21,1°` | **`4,5°`** |
/// | moldura | `7,6°` | `31,2°` | **`7,7°`** |
///
/// # ⚠️ As duas coisas que uma leitura rápida entende ao contrário
///
/// 1. **Mais peças NÃO é sempre melhor.** A mistura encolhe o sólido `≈ r(√k − 1)` com `k` peças
///    **activas**, então separar uma dobra cujos dois lados estão activos ao mesmo tempo come
///    material a dobrar — foi assim que a moldura ficou com **`0` de `64 000`** células dentro. Por
///    isso o [`crate::ops::box_with_edge`] só separa a dobra quando `chanfro + filete < 2·meia`.
/// 2. **O tecto de `‖∇f‖` é `√(activas)`, não `√(total)`** — um prisma hexagonal entrega `19` peças
///    e mede `0,6951` depois do divisor, **abaixo** do `1,0852` que a composição encaixada media.
///    *Uma mistura de dezanove que nunca tem mais de três activas é mais barata que duas encaixadas.*
pub fn intersection_joint_n(corpo: &[Tree], arestas: &[(Tree, Tree)], e: Edge) -> Tree {
    let mut pecas: Vec<Tree> = corpo.to_vec();
    for (a, b) in arestas {
        pecas.push(corte(a, b, e.chamfer, Sentido::Interseccao));
    }
    ops::intersection_round_n(&pecas, e.round)
}

/// ⭐⭐ **O DUAL: as peças de uma UNIÃO e os chanfros dos vales que elas formam, numa mistura só.**
///
/// ⚠️ **Ela existe por um custo MEDIDO, não por simetria.** Dobrar `n` uniões duas a duas compõe a
/// lei de Cauchy–Schwarz `n` vezes: a engrenagem de doze dentes subiu para `‖∇f‖ = 1,887` sobre o
/// campo já dividido (`passo × ‖∇f‖ = 1,33`, acima de `1`) e a marcha **atravessaria a superfície**.
/// Numa união n-ária o tecto é `√(activas)` — e num anel de dentes nunca há mais de dois perto.
pub fn union_joint_n(corpo: &[Tree], arestas: &[(Tree, Tree)], e: Edge) -> Tree {
    let mut pecas: Vec<Tree> = corpo.to_vec();
    for (a, b) in arestas {
        pecas.push(corte(a, b, e.chamfer, Sentido::Uniao));
    }
    ops::union_round_n(&pecas, e.round)
}

/// De que lado o corte a 45° recua — o único sinal que separa as duas leis acima.
#[derive(Clone, Copy)]
enum Sentido {
    Uniao,
    Interseccao,
}

/// O plano do chanfro: `(a + b ∓ c)·√½`.
///
/// ⚠️ **Ele é uma distância EXACTA a um plano quando as duas normais são ortogonais**, e um
/// minorante quando o ângulo abre. É a mesma conta do [`ops::union_chamfer`], escrita uma vez para
/// os dois sentidos — *duas cópias dela discordariam sobre o que um `c` negativo faz*.
fn corte(a: &Tree, b: &Tree, c: f64, sentido: Sentido) -> Tree {
    let soma = a.clone() + b.clone();
    let deslocado = match sentido {
        Sentido::Uniao => soma - Tree::constant(c),
        Sentido::Interseccao => soma + Tree::constant(c),
    };
    deslocado * Tree::constant(std::f64::consts::FRAC_1_SQRT_2)
}

/// ⭐⭐⭐ **A UNIÃO ENTRE DUAS CÓPIAS** — a junta que só age onde as cópias são **distintas**.
///
/// `distinct` vale `1` quando as duas avaliações são cópias diferentes e `0` quando a lei de
/// repetição prendeu o índice e devolveu a mesma. Ver a nota do módulo: com `0` esta função é o
/// `min` **exactamente**, dentro e fora, e não um `min` só por fora.
///
/// ⚠️ **A selecção, e não o raio escalado.** As duas formas parecem equivalentes e não são — a
/// segunda deixa `‖∇f‖ = 1,4142` sobre um documento sem junta. Medido em
/// `spike_join_between_copies::measure_two_cells_against_the_oracle`.
pub fn union_between_copies(a: &Tree, b: &Tree, joint: Joint, distinct: &Tree) -> Tree {
    let duro = a.min(b.clone());
    if joint.is_sharp() {
        return duro;
    }
    let misturado = union_joint(a, b, Edge::of(joint));
    duro.clone() + distinct.clone() * (misturado - duro)
}

/// ⭐⭐⭐ **O PORTÃO: as duas avaliações são cópias DISTINTAS?** — `1` quando sim, `0` quando não.
///
/// Para índices inteiros `|vizinha − própria|` já **é** exactamente `0` ou `1`, e é essa a lei
/// inteira. ⚠️ Ela cobre os **dois** sítios em que uma repetição devolve a mesma cópia duas vezes —
/// o `clamp` nas pontas de uma matriz e o `compare` que vale `0` no centro exacto de uma célula —,
/// e é a única porta por onde essa pergunta se faz.
///
/// # ⛔ Eu escrevi DUAS curas para este defeito, e uma delas não tinha gate
///
/// A primeira versão desta wave trazia também um *sinal sem zero* (`+1` / `−1`, nunca `0`) para
/// impedir o `compare` de apontar para a própria célula. ⚠️ **A prova de mutação matou-a**: trocá-lo
/// de volta pelo `compare` cru deixou todos os gates **verdes**, porque o portão já cobria o caso.
/// *Uma segunda cura que nenhum gate defende é código que ninguém pode remover com confiança* —
/// ficou uma lei só.
pub fn distinct_copies(own: &Tree, other: &Tree) -> Tree {
    (other.clone() - own.clone()).abs()
}
