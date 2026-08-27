//! **O TECTO DE UM PASSO, e a medição que o escolheu** — cortado do `lib.rs` pelo teto de LOC
//! (HR-18, 700) e por RESPONSABILIDADE: o pai fica com a lei do bando, este com o número que
//! grampeia o relógio e as duas hipóteses que a medição tratou.
//!
//! ⚠️ O `use` no pai mantém `crate::MAX_DT` a resolver como sempre resolveu — nenhum sítio de
//! chamada mudou de endereço, e o gate do gémeo no device continua a lê-lo por esse nome.

/// **O tecto de um passo — MEDIDO com o grampo LIGADO e LEVANTADO, que é a única forma de o
/// medir.**
///
/// ⚠️ O doc 91 §5.4 registava este `0,1` como *copiado sem derivação*, ao lado do `motion.wave`
/// — e **os dois não eram o mesmo problema**: lá o `dt` nem chega ao passo, aqui chega.
///
/// ## ⛔⛔ A 1.ª redacção desta nota refutava duas hipóteses com evidência que não as podia testar
///
/// Ela citava um varrimento de `dt` com o grampo **ligado** — e o grampo está **dentro** do
/// sistema medido (`dt` só chega ao passo depois de `clamp(0, MAX_DT)`), então acima de `0,1`
/// aquelas linhas eram a linha de `0,1` **repetida**. A própria sonda o imprimia; a nota
/// publicou **⛔ REFUTADA** na mesma. *Uma refutação precisa de uma corrida em que a hipótese
/// pudesse ter sido verdadeira.*
///
/// O controle (o mesmo varrimento com `MAX_DT = 1e9`) foi corrido em 2026-08-27, `load 1,43`,
/// mediana de **5 sementes** com o espalhamento ao lado — porque um bando é caótico e a leitura
/// anterior (`1,284 → 0,763`, a três decimais) era **da ordem do próprio ruído**:
///
/// ```text
///   GRAMPO LEVANTADO   vizinho medio (mediana de 5 sementes)
///   max_speed   dt=1/60      dt=0,1      dt=0,25      dt=0,5    passo/raio a 0,5
///           4     0,582       0,846        0,848       0,629          1,0
///          20     1,284       0,863        0,940       1,268          5,0
///         100     6,585       2,005        1,752       1,819         25,0
/// ```
///
/// ⛔ *«senão a sim explode»* — **REFUTADA, agora com a corrida que a podia refutar**: nem a
/// `passo/raio = 25` a excursão diverge (o `max_speed` limita **todo** passo por construção).
///
/// ⚠️ *«um pássaro salta por cima da vizinhança a que reage»* — **NÃO refutada.** Com o grampo
/// levantado e `max_speed = 20`, ir de `dt = 0,1` para `0,5` (razão `1,0 → 5,0`) **afrouxa o
/// bando 47%** (`0,863 → 1,268`), que é exactamente o que a hipótese prevê. A refutação anterior
/// lia as linhas duplicadas.
///
/// ⚠️ **E «o bando APERTA com um `dt` maior» NÃO é uma lei — ela inverte-se com o `max_speed`,
/// e é FALSA no ajuste de fábrica.** A nota antiga generalizava da linha `20`; no default (`4`)
/// um passo maior **afrouxa** o bando de `0,582` para `0,846` (+45%), com os espalhamentos bem
/// separados. *Uma varredura em que uma linha diz o contrário das outras não escolheu nada.*
///
/// ⇒ **O `0,1` FICA**, e o que ele guarda é o único efeito que sobrevive ao controle: acima da
/// razão `1` o carácter do bando começa a depender do tamanho do salto de playhead em vez de
/// depender dos params. Nos defaults um passo cobre **20% da percepção**.
///
/// ⚠️ **O gémeo no device (`gpu.rs`, `BOIDS_MAX_DT`) tem gate** — ver
/// `the_device_twin_of_the_ceiling_is_the_same_number`. Antes dele, `0,1 → 1000,0` passava em
/// tudo, **inclusive nos 13 gates de paridade com adapter real**, porque toda rota de paridade
/// fixa `dt = 1/60`, abaixo do grampo.
pub(super) const MAX_DT: f32 = 0.1;
