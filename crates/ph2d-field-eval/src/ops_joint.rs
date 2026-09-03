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
///
/// # ⭐⭐⭐ E o terceiro campo é o ÂNGULO, pela mesma razão (W107)
///
/// O filete desta casa só era um arco de raio `r` a **90°** — fora dali o operador entregava até
/// `2,29×` menos do que o número no slider dizia ([`ops::union_round_at`]). A cura precisa de saber
/// **em que ângulo as duas faces se encontram**, e esse facto é do par de faces, não do acabamento.
///
/// ⚠️ **Ele entra como campo, e não como argumento com valor por omissão, de propósito:** os 32
/// sítios que constroem uma `Edge` passam a **declarar** que ângulo assumem em vez de o supor —
/// [`Edge::square`] para quem é de facto ortogonal, [`Edge::at`] para quem não é. *A suposição que
/// esteve escondida durante toda a vida deste módulo passa a estar escrita em cada chamada.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// O raio do arco. O que o painel chama **Fillet**.
    pub round: f64,
    /// O recuo do corte reto a 45°, ao longo de cada face. O que o painel chama **Chamfer**.
    pub chamfer: f64,
    /// ⭐ **`n_a · n_b`, o cosseno entre as normais EXTERIORES das duas faces** — `0` = ortogonais.
    ///
    /// Para um diedro interno `2α` vale `−cos 2α`; ver [`ops::union_round_at`], que é quem o lê.
    pub cos_faces: f64,
}

impl Edge {
    /// A aresta viva: os dois recuos a zero.
    pub const SHARP: Self = Self {
        round: 0.0,
        chamfer: 0.0,
        cos_faces: 0.0,
    };

    /// ⭐ **A aresta de duas faces ORTOGONAIS** — a laje contra a parede de um cilindro, dois planos
    /// de uma caixa. ⚠️ Ela é o caminho de sempre **ao bit**: com `cos_faces = 0` o operador é
    /// exactamente a fórmula publicada.
    #[must_use]
    pub const fn square(round: f64, chamfer: f64) -> Self {
        Self {
            round,
            chamfer,
            cos_faces: 0.0,
        }
    }

    /// ⭐⭐ **A aresta de duas faces que se encontram num ângulo qualquer**, dado pelo cosseno das
    /// normais exteriores. Ver [`ops::union_round_at`] para a conta e para o que ela cura.
    #[must_use]
    pub const fn at(round: f64, chamfer: f64, cos_faces: f64) -> Self {
        Self {
            round,
            chamfer,
            cos_faces,
        }
    }

    /// A aresta que um [`Joint`] do documento descreve. ⚠️ **É aqui que o `f32` do documento vira o
    /// `f64` do avaliador**, para as duas famílias — a aresta de uma forma e a costura entre cópias.
    ///
    /// ⚠️ **O ângulo fica em `0`, e isso é uma AUSÊNCIA declarada:** a junta que o artista autora é
    /// entre duas *subárvores arbitrárias*, cujas normais no encontro variam ao longo da curva de
    /// contacto e não são conhecidas ao compilar. As formas que **conhecem** a própria geometria
    /// usam a [`Edge::at`].
    #[must_use]
    pub fn of(joint: Joint) -> Self {
        Self {
            round: f64::from(joint.fillet),
            chamfer: f64::from(joint.chamfer),
            cos_faces: 0.0,
        }
    }
}

/// ⭐⭐⭐ **A INTERSECÇÃO com chanfro e depois filete** — a aresta convexa de uma forma.
///
/// A intersecção chanfrada é `max(max(a,b), plano)` — o dual de De Morgan do
/// [`ops::union_chamfer`] —, e o `c` **é** o recuo ao longo de cada face, a mesma régua que os
/// chips de carácter partilham. A `90°` o plano é `(a+b+c)·√½`; a lei geral está no [`corte`].
///
/// **Medido** (`spike_chamfer_then_fillet`, aro de cilindro): recuo
/// pedido `0,050`/`0,100`/`0,150`/`0,200` ⇒ entregue `0,05000`/`0,10000`/`0,15000`/`0,20000` nas
/// **duas** faces, e `‖∇f‖ = 1,0000` a `ε = 1e-5`.
///
/// ⚠️⚠️ **E essa medição foi feita num aro de CILINDRO, que é ORTOGONAL** — a mesma armadilha em
/// que o filete viveu toda a vida deste módulo. Fora dos 90° o corte descia `c/sin 2α`, que numa
/// ponta de estrela é `1,61×` o número pedido. ⭐ **A W111 fechou-o:** o corte recua `c` em qualquer
/// quina, e o gate que o mede — pelo caminho do produto, não por uma fórmula copiada — é
/// `the_chamfer_recess_is_the_number_the_slider_says`.
///
/// ⭐ O plano do chanfro é uma **terceira superfície**, e as arestas novas que ele cria são
/// `a ∩ plano` e `b ∩ plano`. Arredondá-las é o filete de sempre aplicado a essas duas juntas — e a
/// aresta velha `a ∩ b` já não está na fronteira (o plano cortou-a fora), logo o filete não lhe
/// toca.
///
/// ⚠️ **Pelo caminho n-ário, com o filete ligado o campo infla até `√2`** (medido `1,4140`) — pelo
/// caminho por PARES ele **não infla**, porque um `max` de dois campos exactos é 1-Lipschitz
/// enquanto `a` e `b` forem distâncias. ⚠️ O `√2` é o balde que o
/// [`crate::gradient_bound`] já paga por um arredondamento exacto. Quem der chanfro a uma primitiva
/// tem de o dizer ao `ph2d_field::fillet_inflates`, senão a marcha anda a passo cheio sobre um campo
/// que sobe mais depressa que a distância — e o sintoma é a peça furada.
///
/// # ⭐⭐⭐ W111 — a cura tem TRÊS metades, e nenhuma delas funciona sozinha
///
/// A W110 mediu duas saídas e recusou as duas, cada uma **sozinha**. A célula que faltava era a das
/// três juntas, e é ela que shipa:
///
/// 1. **O recuo** — o corte desce `c` ao longo de cada face, e não `c/sin 2α` (ver [`corte`]).
/// 2. **A normalização** — o plano passa a ser uma distância (`‖∇plano‖ = 1` contra `0,4644` numa
///    ponta de estrela). ⚠️ Ela **tem** de vir com a 1.ª: honrar só o recuo deixa a mistura
///    `2,15×` mais larga do que a geometria pede, e foi isso que fez a cura parcial medir pior.
/// 3. **O filete POR PARES, com o ângulo das arestas NOVAS** — o corte tira `a ∩ b` da fronteira,
///    logo as que restam são `a ∩ plano` e `b ∩ plano`, as duas com o [`novo_cosseno`].
///
/// ⭐⭐ **E a 4.ª peça é o que torna a 3.ª verdadeira: a faceta pode não sobreviver ao filete.**
/// Acima do [`facet_fillet_limit`] os dois arcos sobrepunham-se e nascia uma crista — que é
/// exactamente o que a W110 mediu na coluna «por PARES». Com o limite no sítio, a mesma coluna cai
/// de `17,52` · `20,06` para **`4,11` · `0,79`**.
///
/// A `2×2` da W110 mais a célula nova, na estrela (fracção de superfície sobre um vinco, nos três
/// pontos de trabalho `c=.5 r=.2` / `.4 .4` / `.3 .5`):
///
/// | plano \ filete | n-ário | por PARES | **por PARES + limite da faceta** |
/// |---|---|---|---|
/// | **ortogonal** (o que shipava) | `5,02` · `3,80` · `0,59` | `6,96` · `15,02` · `22,79` | — |
/// | **honesto** (com o ângulo) | `15,33` · `8,66` · `1,14` | `12,30` · `17,52` · `20,06` | **`12,30` · `4,11` · `0,79`** |
///
/// # ⚠️ O que PIOROU, e por que isso é a medição a ficar honesta
///
/// O pior giro da estrela sobe de `45,8°` para `82,4°`. ⛔ **Não é o operador: é o vértice que o
/// corte antigo escondia.** Numa ponta, o chanfro do aro deixa duas facetas com normais
/// `(nᵢ + ẑ)/√2`, e o ângulo entre elas é `arccos((κ+1)/2) = 83,81°` — a sonda lê `82,4°`–`84,9°` em
/// **toda** a varredura do chanfro (oito posições). Os `45,8°` eram comprados a cortar a ponta
/// `1,61×` mais fundo do que o slider dizia: *a ponta saía romba, e o vértice já não existia quando
/// o aro lá chegava.*
///
/// ⇒ o gate deixou de ter um tecto em graus e passou a ter uma **igualdade analítica**
/// (`the_star_pair_crease_is_exactly_the_angle_the_two_rim_facets_make`), que reprova nos dois
/// sentidos — encolher aquele número significa voltar a cortar a mais.
///
/// # ⛔ O que FICA de fora, e a medição que o mantém lá
///
/// A mistura **n-ária** ([`intersection_joint_n`], e o caso `cos_faces == 0`) continua a supor todos
/// os pares ortogonais; generalizá-la pede a matriz de Gram inteira, cujo recorte não tem forma
/// fechada em `N ≥ 3`. Ver [`SEM_ANGULO_FICA_N_ARIO`].
pub fn intersection_joint(a: &Tree, b: &Tree, e: Edge) -> Tree {
    let (chamfer, fillet) = (e.chamfer, e.round);
    if chamfer <= 0.0 {
        if fillet <= 0.0 || e.cos_faces == 0.0 {
            // ⭐ **O caminho de sempre, ao bit** — nem um nó a mais na árvore.
            return ops::intersection(a, b, Blended::Exact(fillet));
        }
        return ops::intersection_round_at(a, b, fillet, e.cos_faces);
    }
    let plano = corte(a, b, chamfer, e.cos_faces, Sentido::Interseccao);
    if fillet <= 0.0 {
        return a.max(b.clone()).max(plano);
    }
    if e.cos_faces == 0.0 {
        // ⭐⭐⭐ **AS TRÊS DE UMA VEZ** — ver [`SEM_ANGULO_FICA_N_ARIO`] para a medição que manda
        // este caso ficar aqui, e a nota da [`ops::intersection_round_n`] para as duas construções
        // recusadas antes dela.
        return ops::intersection_round_n(&[a.clone(), b.clone(), plano], fillet);
    }
    if fillet >= facet_fillet_limit(chamfer, e.cos_faces) {
        // ⭐⭐⭐ **A FACETA NÃO SOBREVIVE À EROSÃO ⇒ o plano é REDUNDANTE.** Ver o doc da
        // [`facet_fillet_limit`]: acima do limite o filete é o da quina VIVA, e as duas leis
        // coincidem exactamente no limite.
        return ops::intersection_round_at(a, b, fillet, e.cos_faces);
    }
    let novo = novo_cosseno(e.cos_faces);
    ops::intersection_round_at(a, &plano, fillet, novo)
        .max(ops::intersection_round_at(b, &plano, fillet, novo))
}

/// ⭐⭐⭐ **A UNIÃO com chanfro e depois filete** — o vinco côncavo entre duas peças.
///
/// O espelho exacto da [`intersection_joint`], e a razão de as duas viverem lado a lado: um artista
/// que aprendeu os dois números numa aresta de forma lê-os iguais na costura entre duas cópias.
pub fn union_joint(a: &Tree, b: &Tree, e: Edge) -> Tree {
    let (chamfer, fillet) = (e.chamfer, e.round);
    if chamfer <= 0.0 {
        if fillet <= 0.0 || e.cos_faces == 0.0 {
            return ops::union(a, b, Blended::Exact(fillet));
        }
        return ops::union_round_at(a, b, fillet, e.cos_faces);
    }
    let plano = corte(a, b, chamfer, e.cos_faces, Sentido::Uniao);
    if fillet <= 0.0 {
        return a.min(b.clone()).min(plano);
    }
    if e.cos_faces == 0.0 {
        return ops::union_round_n(&[a.clone(), b.clone(), plano], fillet);
    }
    if fillet >= facet_fillet_limit(chamfer, e.cos_faces) {
        return ops::union_round_at(a, b, fillet, e.cos_faces);
    }
    // ⭐ O dual exacto da [`intersection_joint`], por De Morgan: as duas arestas NOVAS que o corte
    // criou, cada uma com o cosseno dela, e nunca as três ao mesmo tempo.
    let novo = novo_cosseno(e.cos_faces);
    ops::union_round_at(a, &plano, fillet, novo).min(ops::union_round_at(b, &plano, fillet, novo))
}

/// ⭐⭐⭐ **O COSSENO DAS ARESTAS QUE O CORTE CRIA** — `n_a · n_p`, onde `n_p` é a normal do plano do
/// chanfro.
///
/// Com `n_p = (n_a + n_b)/‖n_a + n_b‖` e `‖n_a + n_b‖ = √(2 + 2κ)`, sai
/// `n_a·n_p = (1 + κ)/√(2 + 2κ) = √((1+κ)/2)` — que é o `sin α` da cunha de meio-ângulo `α`.
///
/// ⚠️ **É o MESMO valor nos dois sentidos.** Sob complementação as três normais viram, e um produto
/// escalar de duas delas não muda — é por isso que a [`union_joint`] e a [`intersection_joint`] lhe
/// passam o mesmo número.
fn novo_cosseno(cos_faces: f64) -> f64 {
    (0.5 * (1.0 + cos_faces.clamp(-1.0, 1.0))).sqrt()
}

/// ⭐⭐⭐ **ATÉ QUE RAIO A FACETA DO CHANFRO SOBREVIVE À EROSÃO** — `c·sin α·(1 + sin α)/cos α`.
///
/// # A lei, e por que ela não é uma cerca de segurança
///
/// Um filete de raio `r` é a **abertura** morfológica: erodir por `r`, dilatar por `r`. Erodir
/// desloca os três planos (as duas faces e o corte) para dentro por `r`; a faceta sobrevive
/// enquanto o plano do corte deslocado ainda cortar a quina que as duas faces deslocadas formam.
///
/// Com o vértice na origem e a bissectriz em `+x`, a quina das faces deslocadas está em
/// `x = r/sin α` e o corte deslocado em `x = c·cos α + r`. Eles coincidem em
/// `r = c·cos α·sin α/(1 − sin α)`, que é a expressão acima depois de racionalizar.
///
/// ⭐⭐ **E no limite as DUAS leis dão a MESMA peça, o que torna a fronteira contínua:** ali os três
/// planos deslocados são concorrentes, logo os centros dos dois arcos coincidem e os dois arcos
/// **são o mesmo arco** — que é exactamente o filete da quina viva. *A transição não tem degrau
/// para haver, não porque alguém a suavizou.*
///
/// # ⚠️ Acima do limite o CHANFRO é que desaparece, e isso é a geometria, não uma desistência
///
/// Se o corte deslocado já não corta a quina deslocada, ele não pertence ao erodido — logo a
/// abertura do sólido chanfrado é **idêntica** à abertura do sólido vivo. O artista vê o arco comer
/// o chanfro, que é o que as ferramentas de CAD fazem e é legível.
///
/// ⛔ **A alternativa — prender `r` no limite — foi recusada por desenho, não por medição:** ela
/// deixa o *Fillet* inerte acima de um ponto que nada na tela nomeia, e um controlo que pára de
/// responder é o modo de falha que esta casa mede desde 2026-08-30. *A lei que fica mantém os dois
/// controlos vivos: um deles come o outro, à vista.*
///
/// ⚠️ A `90°` isto vale `1,7071·c`, então o ponto de trabalho `r = 0,5·c` dos gates fica **dentro**
/// da faceta em toda forma ortogonal; numa ponta de estrela (`α = 19,2°`) vale `0,4629·c`, e é por
/// isso que ali o mesmo ponto de trabalho cai do outro lado.
fn facet_fillet_limit(chamfer: f64, cos_faces: f64) -> f64 {
    let c = cos_faces.clamp(-COS_FACES_MAX_JOINT, COS_FACES_MAX_JOINT);
    let (sin_a, cos_a) = ((0.5 * (1.0 + c)).sqrt(), (0.5 * (1.0 - c)).sqrt());
    chamfer * sin_a * (1.0 + sin_a) / cos_a
}

/// A mesma cerca da [`ops::union_round_at`], repetida aqui porque as duas leis a partilham: um
/// `cos_faces` a `±1` é uma cunha degenerada (duas faces paralelas ou coincidentes) e o divisor
/// `cos α` iria a zero.
const COS_FACES_MAX_JOINT: f64 = 0.9999;

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
        pecas.push(corte(
            a,
            b,
            e.chamfer,
            SEM_ANGULO_FICA_N_ARIO,
            Sentido::Interseccao,
        ));
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
        pecas.push(corte(
            a,
            b,
            e.chamfer,
            SEM_ANGULO_FICA_N_ARIO,
            Sentido::Uniao,
        ));
    }
    ops::union_round_n(&pecas, e.round)
}

/// ⛔⛔ **SEM O ÂNGULO, A MISTURA FICA N-ÁRIA — e a razão é MEDIDA em DOIS sítios.**
///
/// As duas metades da cura andam juntas: honrar o recuo do corte **exige** honrar a normalização
/// dele, e o campo `2,15×` mais fiel estreita `2,15×` a região onde o filete mistura sobre o plano.
/// Numa mistura **por pares** isso é exactamente o que se quer, porque o par sabe o ângulo da
/// aresta nova. Numa mistura n-ária **não há** quem o saiba: a [`ops::intersection_round_n`] supõe
/// todos os pares ortogonais, e a matriz de Gram de `N ≥ 3` geradores não tem recorte de forma
/// fechada.
///
/// ⇒ dar-lhe o plano honesto é a célula que a W110 mediu e recusou: na estrela a fracção de
/// superfície sobre um vinco ia de `5,02 %` para `15,33 %`. *Meia cura desta família deixa a metade
/// que ficou pior do que estava.*
///
/// # ⚠️ E este valor responde por DOIS sítios, com a mesma medição
///
/// 1. **A [`intersection_joint_n`] e a [`union_joint_n`]** — a mistura é n-ária por construção (é a
///    razão de elas existirem: a costura interna de uma composta aflorava na aresta seguinte), logo
///    o plano tem de ficar ortogonal. ⛔ **Medido na forma que o exprime**, e não extrapolado da
///    estrela: o **octaedro** é a única cujas doze arestas partilham um ângulo (`κ = 1/3`), logo é a
///    única que pode dizer «todas as minhas arestas são assim». Com o plano honesto o vinco residual
///    dela sobe de `10,03 %` para `15,12 %` da superfície, e nenhum gate melhora. *A mistura estreita
///    a região onde mistura sem saber o ângulo das arestas novas — o mesmo mecanismo, um nível
///    acima.*
/// 2. **O `cos_faces == 0` das juntas por PARES** — que é a *ausência declarada* de ângulo
///    ([`Edge::square`] e [`Edge::of`]). ⛔ A W111 experimentou levar também este caso à decomposição
///    por pares e a medição recusou: a cruz foi de `3,8°` para `12,2°` de pior giro (`3,19×`, com a
///    barra em `2,60×`). *Um `a` que é a UNIÃO de meia peça não é uma face, e o `max` de dois
///    pares só é exacto entre superfícies.*
///
/// ⚠️ **Ele é `0.0` e não um `Option`, de propósito:** o sentinela já existe no contrato da
/// [`Edge`], e um segundo vocabulário para *«não sei o ângulo»* poria duas respostas na mesma
/// pergunta.
const SEM_ANGULO_FICA_N_ARIO: f64 = 0.0;

/// De que lado o corte recua — o único sinal que separa as duas leis acima.
#[derive(Clone, Copy)]
enum Sentido {
    Uniao,
    Interseccao,
}

/// ⭐⭐⭐ **O PLANO DO CHANFRO** — `(a + b)/√(2+2κ) ± c·√((1−κ)/2)`, e a `κ = 0` isso é
/// `(a + b ± c)·√½` termo a termo.
///
/// # A derivação, em duas linhas
///
/// Com o vértice na origem, a bissectriz em `+x` e meio-ângulo interno `α`, as normais exteriores
/// são `(−sin α, ±cos α)`, logo `κ = n_a·n_b = −cos 2α`. O corte tem de tocar cada face à distância
/// `c` **medida ao longo da face**, isto é, em `c·(cos α, ±sin α)`; a normal dele é
/// `(n_a + n_b)/‖n_a + n_b‖` com `‖n_a + n_b‖ = √(2 + 2κ)`, e a distância do vértice ao plano é
/// `c·cos α = c·√((1−κ)/2)`.
///
/// # ⭐⭐ As DUAS coisas que isto corrige, e por que uma sem a outra mede pior
///
/// | | a lei até W110 (`·√½`) | esta |
/// |---|---|---|
/// | recuo ao longo da face | `c/sin 2α` — **`1,61×`** o pedido numa ponta de estrela | **`c`** |
/// | `‖∇plano‖` na mesma ponta | `0,4644` — subestima **`2,15×`** | **`1,0000`** |
///
/// ⛔⛔ **A W110 construiu a 1.ª metade sozinha e a medição recusou-a**, e a explicação estava na
/// 2.ª: a região onde o filete mistura sobre o plano é `{|plano| < r}`, logo um campo `2,15×` menor
/// torna-a `2,15×` mais larga — *a lei antiga escondia o vinco da ponta porque errava numa segunda
/// coisa, na direcção que compensava a primeira.* A varredura que o provou está no doc da
/// [`intersection_joint`]: com o recuo honesto e a escala antiga, o pior giro fica em `~85°` do
/// princípio ao fim do chanfro, e *uma curva plana exclui «ela corta menos, logo sobra mais ponta»*.
///
/// ⚠️ **E nem as duas juntas bastam:** o filete que vem por cima tem de ser aplicado **por pares**,
/// com o [`novo_cosseno`] das arestas que o corte cria, e travado pelo [`facet_fillet_limit`].
/// Quatro peças, uma cura.
///
/// # ⚠️ `κ = 0` devolve a expressão ANTIGA, e não a fórmula nova avaliada em zero
///
/// As duas concordam em aritmética exacta e **não ao bit** — `(a+b)·s + c·s` e `(a+b+c)·s` são somas
/// em ordens diferentes. O ramo curto mantém intocado o caminho de omissão de 25 sítios de chamada,
/// que é o mesmo motivo por que a [`Edge::square`] existe.
fn corte(a: &Tree, b: &Tree, c: f64, cos_faces: f64, sentido: Sentido) -> Tree {
    if cos_faces == 0.0 {
        // ⭐ **O caminho de sempre, ao bit** — e ele é o caso `κ = 0` da lei abaixo, termo a termo.
        let soma = a.clone() + b.clone();
        let deslocado = match sentido {
            Sentido::Uniao => soma - Tree::constant(c),
            Sentido::Interseccao => soma + Tree::constant(c),
        };
        return deslocado * Tree::constant(std::f64::consts::FRAC_1_SQRT_2);
    }
    let k = cos_faces.clamp(-COS_FACES_MAX_JOINT, COS_FACES_MAX_JOINT);
    let escala = (2.0 + 2.0 * k).sqrt().recip();
    let recuo = c * (0.5 * (1.0 - k)).sqrt();
    let normalizado = (a.clone() + b.clone()) * Tree::constant(escala);
    match sentido {
        Sentido::Uniao => normalizado - Tree::constant(recuo),
        Sentido::Interseccao => normalizado + Tree::constant(recuo),
    }
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
