//! ⭐⭐⭐ **O NÓ DE TORO** (W134) — as cercas da corda que anda na superfície de um toro, e por que
//! cada uma delas é geometria e não gosto.
//!
//! # Por que a prosa vive aqui, e não no [`crate::primitive`]
//!
//! Aquele arquivo é **um `enum` no tecto de LOC**, e o doc dele já escreveu a saída: *«a próxima
//! primitiva não cabe, e a saída é a que a `Polygon` usou — o doc da variante fica com o essencial
//! e a prosa vai para o módulo do mecanismo»*. Este é esse módulo.
//!
//! # ⭐⭐ `p` e `q` são CONTAGENS, e é a representação que apaga o caso especial
//!
//! A W128 pagou uma lição: *um `m` fraccionário não faz forma nova, faz uma peça rachada*, e a cura
//! ali foi **coagir** o número na porta de escrita. Aqui não há nada a coagir — `p` é o número de
//! ramos do `min` que constrói o campo ([`ph2d_field_eval::ops_knot`]), logo um `p` fraccionário não
//! é exprimível. *Quando a representação certa existe, a validação deixa de ter trabalho.*
//!
//! # ⚠️ `gcd(p, q) > 1` NÃO é recusado
//!
//! Ali a curva fecha antes de gastar os `p` ramos e o desenho degenera para o nó `(p/g, q/g)`
//! percorrido `g` vezes — uma peça **válida**, com fios coincidentes. ⛔ Recusá-la seria proibir uma
//! forma por causa do nome que a matemática lhe dá.

/// O menor número de voltas ao eixo. ⚠️ **Um**, e não dois: `(1, q)` é uma argola enrolada — não é
/// um nó, e é uma forma.
pub const MIN_KNOT_WINDS: u32 = 1;

/// O menor número de voltas ao tubo. Com `q = 0` a corda não sai do plano e a peça é um toro fino,
/// que já tem primitiva própria.
pub const MIN_KNOT_LOOPS: u32 = 1;

/// ⭐ **O TECTO DE `p`** — e o recurso dele é o **relógio do quadro**, porque `p` é a contagem de
/// ramos da árvore.
///
/// Medido pelo traçado (`the_price_of_the_torus_knot`, uma peça a 640×360, `q = 3`, `load 4,06`):
///
/// | `p` | 1 | 2 | 3 | 4 | 6 | 8 | 10 | **12** | 14 | 16 | 20 | 24 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
/// | ms | `17,8` | `18,2` | `15,0` | `21,8` | `19,3` | `31,0` | `36,0` | **`32,1`** | `49,2` | `61,1` | `78,1` | `80,9` |
///
/// Calibração da mesma sonda: esfera `2,4` · toro `2,5` · caixa `8,9` · **só um desenho na cena
/// `14,3`** · desenho + esfera `18,9`. ⇒ até `p = 12` um nó custa o que uma cena com um desenho já
/// custa; de `14` para cima ele **dobra** e passa a ser a peça mais cara do módulo.
pub const MAX_KNOT_WINDS: u32 = 12;

/// ⭐⭐ **O TECTO DE `q` É RELATIVO A `p`**, e o recurso dele é OUTRO: a **marcha**.
///
/// A árvore não cresce com `q` — o que cresce é o divisor do minorante, que vale `~q/p`, e com ele o
/// número de passos que a marcha dá dentro da coroa. Medido no pior `q/p` que existe (`p = 1`,
/// mesma sonda):
///
/// | `q` (com `p = 1`) | 1 | 2 | 3 | **4** | 6 | 8 | 10 | 12 | 16 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
/// | ms | `8,4` | `14,1` | `21,2` | **`23,5`** | `39,5` | `46,0` | `50,0` | `50,2` | `60,3` |
///
/// ⇒ o joelho está em `q/p = 4`: ali um nó custa `23,5 ms`, a classe de *desenho + esfera*; a `6` ele
/// já é `39,5`. ⛔ **Um tecto ABSOLUTO em `q` seria o caminho lento a mandar no rápido** (§0): com
/// `p = 12` a mesma razão custa o mesmo, e proibi-la seria proibir metade da família por causa do
/// caso `p = 1`.
pub const MAX_KNOT_LOOPS_OVER_WINDS: u32 = 4;

/// O tecto de `q` para este `p` — ver [`MAX_KNOT_LOOPS_OVER_WINDS`].
#[must_use]
pub fn max_knot_loops(winds: u32) -> u32 {
    winds.clamp(MIN_KNOT_WINDS, MAX_KNOT_WINDS) * MAX_KNOT_LOOPS_OVER_WINDS
}

/// ⭐⭐ **O TECTO DA CORDA** — metade da menor distância **PERPENDICULAR** entre dois fios, e as
/// duas candidatas são geometria fechada.
///
/// ⚠️⚠️ **A primeira redacção media a CORDA no plano meridiano, e isso é generoso por `1/c`.** Dois
/// fios vizinhos não são dois pontos: são duas rectas **inclinadas do mesmo lado**, e a distância
/// entre rectas paralelas é a componente do deslocamento **perpendicular à direcção delas**. Medir
/// no plano do corte conta a parte que corre ao longo dos dois, que não os aproxima.
///
/// Com `sin β = ρ/√(ρ² + (r·q/p)²)` (a fracção da direcção do fio que é azimutal, tomada no raio do
/// anel):
///
/// 1. **Fios vizinhos no mesmo `φ`** — o deslocamento é ao longo do círculo do tubo, que faz ângulo
///    `β` com a direcção deles ⇒ perpendicular `= 2·r·sin(π/p) · sin β`. Com `p = 1` não há vizinho,
///    e o tecto é o próprio raio do tubo (mais do que isso e a corda engole o furo).
/// 2. **Passagens sucessivas pelo mesmo `ψ`** — o deslocamento é azimutal, que faz `90° − β` com a
///    direcção deles ⇒ perpendicular `= 2π(R − r)·p/q · cos β`.
///
/// ⚠️ **É geometria, não conforto**: o número sai de `R`, `r`, `p` e `q`. Acima dele os fios
/// fundem-se e a peça deixa de ser um nó — a mesma lei do `MAX_SPIRAL_FILL` da mola, escrita para
/// duas direcções em vez de uma.
///
/// ⛔ **É um MINORANTE da meia-distância verdadeira, e há gate a prová-lo** (`the_cord_ceiling_is_
/// below_the_curve_that_measures_itself`): as duas candidatas são as duas famílias de vizinhança de
/// um nó de toro, e a varredura da curva contra ela própria confirma-o em toda a grelha `(p, q)`.
#[must_use]
pub fn knot_cord_ceiling(radius: f32, tube: f32, winds: u32, loops: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let (p, q) = (winds.max(1) as f32, loops.max(1) as f32);
    let k = tube * q / p;
    // ⚠️⚠️ **O `sin β` é tomado no lado de DENTRO do toro (`R − r`), e não no raio do anel.** A
    // inclinação do fio depende do raio a que ele passa, e o lado de dentro é o mais apertado — foi
    // ali que a varredura da curva contra ela própria apanhou o tecto a exceder o alcance real em
    // `1,26×` (`p = 3, q = 10`). *Um `sin β` no raio médio descreve o fio no sítio em que ele está
    // mais folgado.*
    let dentro = (radius - tube).max(f32::EPSILON);
    let sin_b = dentro / dentro.hypot(k);
    let cos_b = k / dentro.hypot(k);
    let entre_fios = if winds <= 1 {
        tube
    } else {
        tube * (std::f32::consts::PI / p).sin() * sin_b
    };
    let entre_voltas = std::f32::consts::PI * (radius - tube).max(0.0) * p / q * cos_b;
    // ⛔⛔⛔ **A CURVATURA DA PRÓPRIA CORDA foi construída, medida e RETIRADA.** Ela é uma cerca
    //    legítima em teoria — um tubo mais gordo do que o raio de curvatura da linha média cruza-se
    //    **sozinho** — e **nunca decide**: medido sobre `486 720` células (`R` em `0,05..2,0`, `r`
    //    em `2,5 %..97,5 %` de `R`, todos os `(p, q)` que o painel oferece) ela é o mínimo em
    //    **ZERO**. A álgebra diz porquê: `1/κ < R − r` exige `R − r > r`, e `1/κ < r` exige o
    //    contrário — as duas nunca se dão ao mesmo tempo, e uma delas está sempre no `min`.
    //    ⚠️ **Ela SOBREVIVEU a duas rondas de mutação antes de eu perguntar «quantas células ela
    //    decide?»** — *num `min` de cercas, uma candidata que nunca é o mínimo é invisível a toda
    //    régua de resultado.*
    //
    // 4. ⭐⭐ **ATRAVÉS DO FURO** — os dois lados de dentro do anel distam `2(R − r)`, e a corda de um
    //    lado encontra a do outro a metade disso. ⚠️ **É a família que só um toro GORDO revela**: com
    //    `r` pequeno ela nunca decide, e foi preciso pôr uma segunda proporção no corpus da régua
    //    para a ver (`R = 0,55`, `r = 0,40`: o alcance medido é **exactamente** `R − r`).
    //
    // ⚠️⚠️ **Esta candidata já cá esteve com `(R − r)/2` e foi RETIRADA** — metade do valor certo,
    //    escolhida e não derivada, e por isso ela era a mais apertada de todas: mascarava a
    //    curvatura, cuja mutação SOBREVIVIA. *Uma cerca que não morde esconde a que morde* — e o
    //    número errado escondia a família certa.
    let furo = dentro;
    KNOT_CORD_MARGIN * entre_fios.min(entre_voltas).min(furo)
}

/// ⭐ **O que as três candidatas ainda SOBRAM, medido.**
///
/// As três são a geometria de **rectas paralelas**, e a corda de um nó não é uma recta. Medido
/// pelo alcance da curva (`the_cord_ceiling_is_below_the_curve_that_measures_itself`, sobre as
/// células com `q ≤ 4p`, nas **duas** proporções de toro do corpus):
///
/// | célula | `r=0,40 (1,3)` | `r=0,40 (1,4)` | `r=0,24 (1,4)` | as restantes |
/// |---|---:|---:|---:|---:|
/// | tecto ÷ alcance, com margem `0,90` | **`1,041`** | `0,997` | `0,967` | `≤ 0,91` |
///
/// ⇒ com `0,90` sobra `4,1 %` na pior, e o que shipa é **`0,85`**: *um máximo AMOSTRADO que vira
/// limite de segurança erra sempre PARA BAIXO*.
pub const KNOT_CORD_MARGIN: f32 = 0.85;
