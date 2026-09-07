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
/// Medido pelo traçado (`the_price_of_the_torus_knot`, uma peça a 640×360, `q = 3`, `load 4,15`):
///
/// | `p` | 1 | 2 | 3 | 4 | **6** | 8 | 10 | 12 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|
/// | ms | `12,5` | `13,2` | `11,1` | `18,0` | **`16,9`** | `34,0` | `41,2` | `35,9` |
///
/// Calibração da mesma sonda: esfera `3,0` · toro `2,8`; e a base do módulo é `26,7 ms` num quadro de
/// **movimento** (doc 06 §13.0). ⇒ até `p = 6` um nó custa menos do que a base; de `8` para cima ele
/// **dobra**.
///
/// ⛔ **Este número desceu de `12` para `8` em 07/09**, e a razão é o `CLAUDE.md` §0: *quem move o
/// número que tornava algo inalcançável tem de reconferir a nota*. A tabela de `12` foi medida com o
/// campo de ontem; o de hoje é outro (a correcção de curvatura, o tecto da pegada) e o joelho
/// mudou de sítio. ⚠️ O `8` é o degrau em que a tabela ainda foi medida — o `6` é onde ela é barata,
/// e a folga entre os dois é a que a sonda não distingue de ruído.
pub const MAX_KNOT_WINDS: u32 = 8;

/// ⭐⭐ **O TECTO DE `q` É RELATIVO A `p`**, e o recurso dele é OUTRO: a **FORMA**.
///
/// A árvore não cresce com `q`; o que cresce é o quão **inclinado** o fio corre, e o modelo do campo
/// (um cruzamento por plano meridiano, com o eixo encolhido pela inclinação) é de 1.ª ordem nisso.
/// Medido pelo salto máximo da normal na secção da corda, no topo do controlo:
///
/// | `q/p` | 1 | 2 | 3 | **4** | 5 | 6 |
/// |---|---:|---:|---:|---:|---:|---:|
/// | salto da normal | `17,2°` | `19,1°` | `21,4°` | **`27,5°`** | `128,5°` | `142,0°` |
///
/// (uma secção lisa amostrada em `24` direcções dá `15,0°` — é esse o chão.)
///
/// ⇒ o joelho está entre `4` e `5`, e não é gradual: **a peça passa de `27°` para `128°`**. E o
/// relógio concorda que ali ainda é barato (`p = 1, q = 4` custa `14,0 ms`).
///
/// ⛔ **Um tecto ABSOLUTO em `q` seria o caminho lento a mandar no rápido** (§0): com `p` grande a
/// mesma razão dá a mesma forma, e proibi-la seria proibir metade da família por causa do `p = 1`.
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
    // ⚠️ A pegada mede-se no raio do ANEL, que é onde a elipse é maior.
    let sin_b_fora = radius / radius.hypot(k);
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
    // 4. ⭐⭐⭐ **A PEGADA DA CORDA NO TUBO** — e é esta que o report de 07/09 obrigou a escrever.
    //    A secção da corda no plano meridiano é uma elipse com semi-eixo `corda/sin β` ao longo do
    //    círculo do tubo. Quando essa pegada deixa de ser um arco PEQUENO do tubo, o modelo (um
    //    cruzamento por plano meridiano, com o eixo encolhido pela inclinação) sai da validade dele
    //    e a corda deixa de ser redonda. Medido pelo salto da normal na secção: `ζ ≈ 0,3` dá
    //    `15°`–`22°`; `ζ = 0,6` dá `31°`; `ζ = 0,77` dá `95°`.
    let pegada = KNOT_FOOTPRINT * tube * sin_b_fora;
    KNOT_CORD_MARGIN * entre_fios.min(entre_voltas).min(furo).min(pegada)
}

/// ⭐⭐⭐ **QUANTO DO TUBO A CORDA PODE OCUPAR** — a fracção do raio do tubo que a pegada dela cobre.
///
/// ⛔ **Este é o tecto que o report do Enio de 07/09 obrigou a escrever**, e ele é do **MODELO**, não
/// da geometria: a secção da corda no plano meridiano é uma elipse de semi-eixo `corda/sin β`, e
/// quando ela deixa de ser um arco pequeno do tubo o campo (um cruzamento por plano meridiano, com o
/// eixo encolhido pela inclinação) sai da validade dele e a corda deixa de ser redonda.
///
/// Medido pelo salto máximo da normal na secção, com a corda a **`95 %` do tecto** (o pior sítio a
/// que o controlo chega), sobre nove pares:
///
/// | `ζ` | `0,20` | **`0,30`** | `0,40` | `0,55` |
/// |---|---:|---:|---:|---:|
/// | pior salto | `18,0°` | **`21,4°`** | `29,5°` | `92,2°` |
///
/// (uma secção lisa amostrada em `24` direcções dá `15,0°` — é esse o chão, e não zero.)
///
/// ⇒ `0,30` mantém a família inteira a menos de `6,4°` do chão **no topo do controlo**; a `0,55` a
/// pior peça vai a `92°`, que é o risco que o dono viu. ⚠️ **Uma corda mais gorda continua
/// alcançável** — por *Thickness*, porque a pegada é proporcional ao raio do tubo.
pub const KNOT_FOOTPRINT: f32 = 0.30;

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
