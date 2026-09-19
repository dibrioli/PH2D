//! **AS DUAS CERCAS DO LAÇO DE SEPARAÇÃO** — irmã do [`super`] pelo tecto de LOC (HR-18) e por
//! ASSUNTO: ali mora a LEI, aqui moram os dois números que decidem **quando o laço muda de
//! comportamento**, cada um com a tabela de onde saiu.
//!
//! ⚠️ Os dois nasceram de reports do dono em 2026-09-18, e nenhum deles é «um valor razoável»: um é
//! o ponto onde o `fork/join` passa a pagar, o outro é o joelho onde a saída deixa de depender do
//! limiar. *Um limite sem a medição ao lado é um palpite à espera de um smoke* (§0.0).

/// ⭐⭐ **A partir de quantas peças uma varredura paga o fork/join** — MEDIDO na sonda
/// [`custo_probe::onde_o_paralelo_passa_a_pagar`], não herdado.
///
/// ⚠️ O [`ph2d_nodegraph::attr::PAR_THRESHOLD`] (`8192`) é o equilíbrio de um nó que corre UMA
/// passagem por quadro com um corpo por-elemento pequeno. Aqui o corpo é o vizinhado mais o SAT de
/// cada par, e a passagem repete-se `varreduras` vezes — *o mesmo `n` tem outro ponto de
/// equilíbrio*, e usar o de lá deixava `500` peças num núcleo com 31 parados.
pub const PECAS_PARA_PARALELIZAR: usize = 128;

/// ⭐⭐⭐ **O REPOUSO VISÍVEL** — abaixo desta fracção do ALCANCE de uma peça, mais uma varredura não
/// muda o que se vê, e o [`separate`] pára.
///
/// ⚠️ **O número é MEDIDO** ([`custo_probe::parar_quando_nada_mais_se_ve`]), e a régua não é o
/// resíduo de uma varredura — é o **DESVIO da saída** contra varrer o tecto inteiro, mais a
/// contagem de pares ainda sobrepostos, que é o que o artista vê:
///
/// | limiar | varreduras (campo de 500) | desvio | pares sobrepostos |
/// |---|---|---|---|
/// | `1e-3` | 50 | `2,1e-2` | **178** (era 177) ⇠ já muda a resposta |
/// | `1e-4` | 100 | `2,1e-3` | 177 |
/// | **`1e-5`** | **149** | **`2,4e-4`** | **177** (era 177) |
/// | `1e-6` | 207 | `2,7e-5` | 177 |
///
/// ⇒ `1e-5` é o joelho: a última coluna **não muda** e a conta cai `6,9×`. ⛔ A `1e-3` a resposta
/// já é outra — *o número não é «um epsilon razoável», é onde a saída deixa de depender dele*.
pub const REPOUSO_VISIVEL: f32 = 1e-5;
