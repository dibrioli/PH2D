//! ⭐⭐⭐ **A ROSCA E O SERRILHADO** (W135) — o filete helicoidal varrido num cilindro.
//!
//! # ⭐⭐ O mecanismo: um PERFIL num plano meridiano, e a volta mais próxima
//!
//! Em coordenadas cilíndricas o sólido é `{ perfil(ρ − núcleo, w) ≤ 0 }` com `w = z − b·φ`, e `b`
//! é o avanço por radiano (`starts · pitch / 2π`). O `w` reduz-se ao período `pitch` pelo mesmo
//! `round()` que a espiral da W123 estreou — e o perfil é um **triângulo** assente no cilindro do
//! núcleo, que é o flanco em V que uma porca agarra.
//!
//! ⚠️ **A secção meridiana é EXACTAMENTE o triângulo autorado, a qualquer inclinação** — é assim que
//! uma rosca se especifica na oficina. O que a inclinação muda não é a forma, é a **distância**.
//!
//! # ⭐⭐⭐ A recta tangente outra vez, e desta vez ela dá o factor em FORMA FECHADA
//!
//! A W134 pagou a lei: *decompor o desvio e encolher só o eixo que corre ao longo do fio*. Aqui a
//! peça é uma **face**, não um fio, e a conta fecha-se ainda melhor. Uma face cuja normal no plano
//! meridiano é `(n_r, n_w)` tem, em 3D, o gradiente
//!
//! ```text
//! ‖∇f‖ = √(n_r² + n_w²·(1 + β²)) = √(1 + β²·n_w²) = 1/k ,      β = b/ρ
//! ```
//!
//! porque `∇(ρ − núcleo)` e `∇w` são **ortogonais** (um é `ρ̂`, o outro vive em `φ̂`–`ẑ`). ⇒ multiplicar
//! a face por `k` deixa-a **exactamente 1-Lipschitz**, e:
//!
//! - a face **radial** (o cilindro do núcleo, `n_w = 0`) tem `k = 1` — ela já é exacta;
//! - a face **axial** pura (`n_w = 1`) tem `k = sin β`, que é o factor da mola.
//!
//! ⚠️⚠️ **E o `k` multiplica cada FLANCO ANTES da junta, nunca a junta depois.** É a mesma lei que a
//! W134 pagou pelo lado oposto: uma folga aplicada a um campo que ainda vai ser **subtraído** desloca
//! a superfície; aplicada a uma **distância** que vai ser **juntada**, ela só a torna honesta — e o
//! raio do filete continua a ser em unidades do mundo. *Escalar depois da junta encolheria o filete
//! sem ninguém pedir.*
//!
//! # ⭐⭐ Os três ângulos de aresta saem da mesma conta, e nenhum é suposto
//!
//! A [`Edge::at`] quer `n_a · n_b` das normais EXTERIORES em 3D, e com
//! `N(n_r, n_w) = (n_r, ∓β·n_w, n_w)` elas são, todas em forma fechada:
//!
//! | aresta | `cos_faces` |
//! |---|---|
//! | **crista** (os dois flancos da mesma mão) | `(sin²α − cos²α − β²cos²α)·k²` |
//! | **raiz** (o flanco contra o cilindro) | `sin α · k` |
//! | **cruz** (os flancos de mãos OPOSTAS) | `(sin²α − cos²α + β²cos²α)·k²` |
//!
//! ⚠️ **A `β²` entra com sinal TROCADO nas duas primeiras** — é o que separa a crista, que uma
//! inclinação **afia** (a `β → ∞` os dois flancos ficam anti-paralelos: uma lâmina), do cruzamento,
//! que ela **abre**. A `β = 0` as três degeneram nos valores planos (`−cos 2α`, `sin α`, `−cos 2α`).
//!
//! ⭐ E cada uma é avaliada **no raio onde a aresta vive** — crista e cruz na crista, raiz no núcleo.

use fidget::context::Tree;

use crate::ops::safe_sqrt;
use crate::ops_joint::{Edge, intersection_joint, union_joint};

/// ⭐⭐⭐ **A ROSCA** — o cilindro de raio `radius` na crista, com um filete de altura `depth` e
/// meio-ângulo `flank_deg`, que dá `starts` entradas e cruza `hands` mãos.
///
/// Ver o cabeçalho do módulo para o mecanismo e para os três cossenos de aresta.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn sd_thread(
    radius: f64,
    half_height: f64,
    pitch: f64,
    depth: f64,
    flank_deg: f64,
    starts: u32,
    hands: u32,
    round: f64,
    chamfer: f64,
) -> Tree {
    let tau = std::f64::consts::TAU;
    let alpha = flank_deg.to_radians();
    let (sa, ca) = alpha.sin_cos();
    // A meia-base do triângulo: `a = depth · tan α`. ⚠️ O tecto do documento garante `2a ≤ pitch`.
    let meia_base = depth * sa / ca;
    // ⚠️ **O piso é do avaliador, não do produto** — o documento já recusa um núcleo pequeno
    // ([`ph2d_field::thread_depth_ceiling`]); isto só impede uma divisão por zero se alguém
    // construir a árvore à mão.
    let nucleo = (radius - depth).max(radius * 0.05);
    let b = f64::from(starts.max(1)) * pitch / tau;

    let rho = safe_sqrt(Tree::x().square() + Tree::y().square());
    let phi = Tree::y().atan2(Tree::x());
    // O cilindro do núcleo — **exacto**, e é a face de `k = 1`.
    let dr = rho - Tree::constant(nucleo);

    let beta2 = |r: f64| (b / r) * (b / r);
    let k = |r: f64| 1.0 / (1.0 + beta2(r) * ca * ca).sqrt();
    // ⚠️ **O divisor é tomado no MENOR raio onde há matéria** — a mesma lei da espiral (W123): um
    // `k` local sobrestimaria a distância quando o ponto mais próximo estivesse mais para dentro.
    let k_min = k(nucleo);
    let k_crista = k(radius);
    let e_crista = Edge::at(
        round,
        chamfer,
        (sa * sa - ca * ca - beta2(radius) * ca * ca) * k_crista * k_crista,
    );
    // ⭐⭐⭐ **AS DUAS ARESTAS CÔNCAVAS LEVAM O FILETE E NÃO O CHANFRO**, e a razão não é o gate.
    //
    // ⚠️ **Esta é a primeira forma da casa em que as arestas CÔNCAVAS pesam mais que as convexas:**
    // por volta há **uma** crista e **duas** raízes, e as áreas são as mesmas (`½c²·sin 60°` nas
    // duas, por De Morgan). ⇒ um chanfro na raiz somava o dobro do que a crista tirava, e o
    // `every_shape_that_offers_a_chamfer_is_changed_by_it` mediu-o: `50 290 → 50 358`, **+0,000 %**.
    //
    // ⛔ **A cura não é afrouxar o gate — é o significado das duas palavras.** Um *filete* numa
    // quina côncava é a coisa nomeada e pedida (a raiz redonda de um parafuso, onde a fadiga
    // começa). Um *chanfro* é um **corte recto a 45°**: numa quina côncava ele não corta nada, ele
    // ENCHE — e um controlo chamado «Chamfer» que engorda a peça é um controlo que mente.
    //
    // ⇒ o chanfro age só nas arestas **convexas** (a crista e o aro da laje), e o filete alcança as
    // quatro. ⚠️ **O filete continua a somar volume** nesta forma (duas raízes contra uma crista), e
    // isso é o que um filete de rosca faz em qualquer CAD — está DECLARADO, não é um descuido.
    let e_cruz = Edge::at(
        round,
        0.0,
        (sa * sa - ca * ca + beta2(radius) * ca * ca) * k_crista * k_crista,
    );
    let e_raiz = Edge::at(round, 0.0, sa * k_min);

    let mut cristas: Option<Tree> = None;
    for mao in 0..hands.max(1) {
        // ⭐ A mão: `w = z ∓ b·φ`. ⚠️ **Sem costura** — em `φ → φ − 2π` o `w` anda um número
        // INTEIRO de períodos, e o reduzido não se mexe. Ver [`ph2d_field::thread`].
        let sentido = if mao == 0 { -b } else { b };
        let w = Tree::z() + phi.clone() * Tree::constant(sentido);
        let wr = w.clone() - Tree::constant(pitch) * (w / Tree::constant(pitch)).round();
        let flanco = |lado: f64| {
            (dr.clone() * Tree::constant(sa)
                + (wr.clone() * Tree::constant(lado) - Tree::constant(meia_base))
                    * Tree::constant(ca))
                * Tree::constant(k_min)
        };
        // ⭐⭐⭐ **O FILETE ASSENTA no núcleo, e o terceiro semiespaço é o que o diz.**
        //
        // ⛔ Sem ele o `max` dos dois flancos é uma **cunha INFINITA** que continua para dentro até
        // ao eixo — e lá dentro ela GANHA o `min` contra o cilindro, porque o termo
        // `(|w| − a)·cos α` é negativo e empurra a cunha abaixo de `dr`. Medido: `‖∇f‖` até
        // **`2,46`** a `ρ = 0,013`, onde `|∇w| = b/ρ` explode. *O ponto é fundo dentro da peça, e é
        // por isso que nenhuma régua de FORMA o via.*
        //
        // ⚠️ **Ele não leva `k`** — a face dele é o cilindro (`n_w = 0`), logo já é exacta.
        let crista = intersection_joint(&flanco(1.0), &flanco(-1.0), e_crista).max(-dr.clone());
        cristas = Some(match cristas {
            None => crista,
            Some(outra) => union_joint(&outra, &crista, e_cruz),
        });
    }
    let corpo = match cristas {
        None => dr,
        Some(c) => union_joint(&dr, &c, e_raiz),
    };
    // ⭐⭐⭐ **A LAJE, E O ARRANQUE DA ROSCA — a QUARTA aresta, e a que quase escapou.**
    //
    // ⛔ Ela estava escrita `Edge::square`, *«a parede do cilindro é ortogonal à tampa»* — e é, mas
    // a parede do cilindro **não é a única coisa** que encontra a tampa. Cortar a rosca a meio de
    // uma volta deixa o **ARRANQUE**: uma cunha de flanco que encontra a tampa a `90° − α`, que a
    // `α = 30°` são **`30°`** — uma faca, e é a mesma faca que um parafuso real tem na ponta.
    //
    // ⚠️ **MEDIDO** (`the_fillet_reaches_every_edge_of_every_shape`): com `Edge::square` ficavam
    // **`10,0 %`** da superfície sobre um vinco (pior `73,1°`), tudo em `|z| = h`; com o cosseno do
    // flanco, **`0,1 %`**. *A hipótese foi testada antes da cura: uma peça `4×` mais alta baixou a
    // fracção para `4,4 %` e as coordenadas nunca saíram das tampas.*
    //
    // ⛔⛔⛔ **E DAR-LHE O ÂNGULO VERDADEIRO FOI CONSTRUÍDO, MEDIDO E RECUSADO.**
    //
    // Com `Edge::at(round, chamfer, −cos α · k)` a fracção de vinco cai de **`10,0 %` para `0,1 %`**
    // — a cura funciona. ⛔ E o campo deixa de ser marchável: `intersection_round_at` num diedro de
    // `30°` dá `‖∇f‖ = 3,71` contra o `1` que a marcha exige (`passo 0,7071 × 3,71 = 2,62`), e não
    // é do chanfro — o **filete SOZINHO** mede o mesmo em `round = 0,003`, `0,010` e `0,016`.
    //
    // ⚠️ **A casa afirma que «só o PAR encolhe: cada recuo sozinho já está dentro do balde»**
    // ([`ph2d_field::edge_shrink`], cuja tabela mede `5,02` no prisma e paga divisor `4`) — e a
    // rosca é a primeira forma a exercer um diedro **AGUDO** de verdade (`30°`, contra os `120°` do
    // hexágono). Pagar o divisor aqui custaria **`4×` o quadro sempre que o artista põe um filete**.
    //
    // ⇒ fica a suposição ortogonal, e o arranque entra no `APEX_EXCEPTION` **ao lado da MOLA**, que
    // é a mesma laje a cortar um fio inclinado (`("helix", 6.0)`), com esta medição escrita lá.
    // ⭐ E é a forma certa: um parafuso a sério tem a faca do arranque, e quem a tira é um **chanfro
    // de entrada na ponta** — outra feature, não este filete.
    let laje = Tree::z().abs() - Tree::constant(half_height);
    intersection_joint(&corpo, &laje, Edge::square(round, chamfer))
}
