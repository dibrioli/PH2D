//! ⭐⭐⭐ **A ROSCA — as cercas do filete helicoidal** (W135), num arquivo só.
//!
//! # ⭐ Por que ela é uma primitiva, e não uma composição
//!
//! `Union(Cylinder, Helix)` **já se faz hoje**, e dá um parafuso de filete **REDONDO** — o que a
//! composição não alcança é o que faz uma rosca ser uma rosca: o **flanco em V** (a face inclinada
//! que uma porca agarra), as **entradas** (`starts`) e o cruzamento das duas mãos, que é o
//! **serrilhado** de um punho. ⚠️ *A pergunta do `CLAUDE.md` §5.0 foi feita e a resposta é «metade»*
//! — a mesma resposta que o polígono da W132 deu.
//!
//! # ⭐⭐ O modelo, e a cerca que sai dele
//!
//! O sólido é `{ (ρ, φ, z) : perfil(ρ − núcleo, w) ≤ 0 }` com `w = z − b·φ` reduzido ao período
//! `pitch`, e `b = starts · pitch / 2π`. ⚠️ **É um CONJUNTO, não uma varredura** — ele não pode
//! auto-intersectar-se, e por isso a inclinação não tem cerca de forma. O que tem cerca é o
//! **período**: o campo mede a volta MAIS PRÓXIMA, então se o triângulo do perfil for mais largo que
//! `pitch` ele invade a volta vizinha e o campo passa a dizer *«fora»* dentro da peça.
//!
//! ⇒ `2·depth·tan α ≤ pitch`, que é [`thread_depth_ceiling`]. ⭐ **No limite exacto os filetes
//! tocam-se na raiz** — é a rosca de profundidade cheia, e a faixa do painel ([`crate::Span::Wall`])
//! pára mesmo antes dela, deixando sempre uma nesga de terra.
//!
//! # ⚠️ A costura do `atan2` não existe, e a razão é a mesma do nó (W134) — mas por outra via
//!
//! Em `φ → φ − 2π` o `w` anda `+starts·pitch`, que é um número **INTEIRO** de períodos: o `w`
//! reduzido não se mexe. ⇒ **`starts` é `u32` porque é isso que o torna contínuo** — uma entrada
//! fraccionária rachava a peça de alto a baixo, que é a lição da W128 paga pela representação.

use std::f32::consts::PI;

/// O mínimo de entradas — uma rosca tem pelo menos um fio.
pub const MIN_THREAD_STARTS: u32 = 1;

/// **O máximo de entradas.**
///
/// ⚠️⚠️ **O recurso NÃO é a árvore — é a MARCHA.** `starts` não acrescenta um ramo: ele só engorda
/// `b`, o avanço por radiano. Quem cresce é o **divisor** (`1/k = √(1 + (b/núcleo)²·cos²α)`), o
/// campo fica mais conservador, e o passo da marcha encurta. ⛔ *Uma sonda que contasse nós da
/// árvore diria que isto é grátis* — por isso a régua é o **QUADRO** (a lei da W128/W134).
///
/// ⚠️ **MEDIDO** (`the_price_of_the_thread`, 07/09, `load 4,96`, uma peça a 640×360; esfera `2,7`,
/// cilindro `2,6`):
///
/// | `starts` | 1 | 8 | 32 | 64 | 96 | **128** | 192 | 256 | 384 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
/// | ms | 2,6 | 3,4 | 4,3 | 5,3 | 6,5 | **8,3** | 8,8 | 11,0 | 13,2 |
///
/// ⭐ **O critério é *duas peças ainda cabem num quadro*** — a `128` uma peça sozinha é metade dos
/// `16,7 ms`, e acima disso uma cena com duas roscas deixa de fechar. ⛔ **Não** é «até rebentar»:
/// a `384` a peça sozinha ainda cabe (`79 %`), e um tecto ali entregaria uma forma que só existe
/// se for a única coisa no ecrã.
///
/// ⚠️ **E a FORMA não põe tecto nenhum aqui** — o gradiente foi medido `≤ 1,0107` até `starts = 96`
/// e a secção meridiana continua exacta: é mesmo só o relógio.
pub const MAX_THREAD_STARTS: u32 = 128;

/// ⭐ **Uma mão, ou as duas.** `1` é a rosca; `2` cruza a família da mão direita com a da esquerda,
/// que é o **serrilhado** (o losango de um punho).
///
/// ⚠️ **Não há uma terceira**, e é por isso que o tecto é uma constante da GEOMETRIA e não uma
/// medição: uma hélice sobe ou desce, e o conjunto das mãos tem dois elementos.
pub const MIN_THREAD_HANDS: u32 = 1;
/// Ver [`MIN_THREAD_HANDS`].
pub const MAX_THREAD_HANDS: u32 = 2;

/// **O meio-ângulo do V, em graus** — o ângulo entre o flanco e a direcção radial. A rosca métrica
/// ISO é `30` (60° de ângulo incluído).
///
/// ⚠️ **As duas pontas degeneram, e é isso que as põe aqui:** a `0` o filete é uma lâmina de largura
/// nula; a `90` a base é infinita e [`thread_depth_ceiling`] devolve zero.
pub const MIN_THREAD_FLANK_DEG: f32 = 5.0;
/// Ver [`MIN_THREAD_FLANK_DEG`].
pub const MAX_THREAD_FLANK_DEG: f32 = 75.0;

/// ⚠️ **A fracção do raio que o NÚCLEO nunca desce abaixo.** O divisor do campo é tomado no menor
/// raio onde há matéria, e ele explode quando o núcleo vai a zero: `1/k = √(1 + (b/núcleo)²·cos²α)`.
///
/// ⚠️ **MEDIDO** (mesma corrida, `starts = 8`, o passo aberto para o período não ser quem corta):
///
/// | núcleo | `0,60·R` | `0,45·R` | `0,35·R` | **`0,25·R`** | `0,15·R` | `0,08·R` | `0,04·R` |
/// |---|---:|---:|---:|---:|---:|---:|---:|
/// | ms | 4,6 | 6,9 | 6,3 | **7,9** | 10,7 | 11,6 | **17,4** |
///
/// ⭐ **O mesmo critério das entradas — duas peças num quadro**: a `0,25·R` uma rosca custa `7,9 ms`
/// e duas fecham; a `0,15·R` já não. ⚠️ E a `0,04·R` **uma sozinha passa o quadro** (`17,4`), que é
/// a parede que este piso existe para não deixar alcançar.
///
/// ⭐ Ele deixa o filete ir até `0,75·R` de profundidade, que cobre um **sem-fim** — muito além dos
/// `0,05`–`0,1·R` de um parafuso real.
pub const THREAD_CORE_FLOOR: f32 = 0.25;

/// ⭐⭐ **ATÉ ONDE A PROFUNDIDADE PODE IR** — o `min` de duas paredes, cada uma de um recurso:
///
/// 1. **o PERÍODO** (`pitch / (2·tan α)`): acima dele o triângulo de uma volta invade a vizinha, e o
///    campo — que só mede a volta mais próxima — passa a mentir. É a cerca do **MODELO**.
/// 2. **o NÚCLEO** (`radius · (1 − `[`THREAD_CORE_FLOOR`]`)`): o divisor vive no menor raio com
///    matéria. É a cerca da **MARCHA**.
///
/// ⭐ **É a mesma função que o painel, a validação e a porta de escrita usam** — um painel que
/// calculasse o próprio tecto ofereceria o que o documento recusa.
#[must_use]
pub fn thread_depth_ceiling(radius: f32, pitch: f32, flank_deg: f32) -> f32 {
    let alpha = flank_deg.clamp(MIN_THREAD_FLANK_DEG, MAX_THREAD_FLANK_DEG) * PI / 180.0;
    let periodo = pitch * 0.5 / alpha.tan().max(f32::EPSILON);
    let nucleo = radius * (1.0 - THREAD_CORE_FLOOR);
    periodo.min(nucleo).max(0.0)
}

/// ⭐⭐ **ATÉ ONDE O FILETE (o `round`) PODE IR NESTA FORMA** — o `min` de duas alturas, e as duas
/// são de uma aresta diferente:
///
/// 1. **a CRISTA** (`depth · sin α`): o círculo inscrito no vértice do V, tangente aos dois flancos,
///    tem de caber entre a crista e a raiz.
/// 2. **a TERRA** (`(pitch/2 − depth·tan α) · cos α`): a distância do meio da terra ao plano do
///    flanco. ⚠️ **Acima dela a mistura da união alcança o meio da terra, e ali o campo tem um
///    vinco** — o ponto onde a volta mais próxima muda de identidade. *O `min` esconde-o enquanto a
///    superfície ali for a do núcleo; um filete que chegue lá desenterra-o.*
#[must_use]
pub fn thread_round_limit(pitch: f32, depth: f32, flank_deg: f32) -> f32 {
    let alpha = flank_deg.clamp(MIN_THREAD_FLANK_DEG, MAX_THREAD_FLANK_DEG) * PI / 180.0;
    let (sa, ca) = alpha.sin_cos();
    let crista = depth * sa;
    let terra = (pitch * 0.5 - depth * alpha.tan()) * ca;
    crista.min(terra).max(0.0)
}
