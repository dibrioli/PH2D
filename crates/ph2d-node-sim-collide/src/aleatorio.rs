//! **A ALEATORIEDADE DA RESTITUIÇÃO** — a lei de *«nem toda a partícula salta igual»* (doc 89,
//! folha 13), com os dois params que a autoram.
//!
//! ⚠️ Irmã do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali resolve-se o CONTACTO, aqui
//! responde-se *«quão viva é a batida DESTE elemento»* — uma pergunta sobre identidade, não sobre
//! geometria.
//!
//! ⚠️ **Os gates dela FICAM na raiz** (`randomness_tests.rs`): eles exercitam a lei através do
//! `collide` inteiro — um stream, um colisor, uma resposta —, e não a função sozinha. *Os testes
//! moram com o que EXERCITAM, não com o que nomeiam.*

use super::hash;

/// O nome do param da aleatoriedade, e o da semente que a escolhe.
pub(super) const RANDOMNESS: &str = "restitution_randomness";
pub(super) const SEED: &str = "seed";

/// **A RESTITUIÇÃO DE UM ELEMENTO** — a ÚNICA porta, chamada pelo [`collide`] e portada
/// termo a termo para o [`GPU_KERNEL`].
///
/// `rest · (1 − randomness · h)`, com `h ∈ [0, 1)` do hash estável da identidade. Em
/// `randomness = 0` o factor é **exactamente** `1` em IEEE-754 (`1 − 0·h`), então a saída é
/// bit-idêntica à de antes deste param existir — não «quase», o mesmo número.
///
/// ⚠️ **Ela só TIRA, nunca acrescenta**, e é a leitura certa de um material: a restituição
/// autorada é o teto (a batida mais viva que aquele obstáculo devolve), e o acaso diz quanto
/// desta batida se perdeu. Um `±` centrado no valor autorado faria `restitution = 1` devolver
/// **mais** energia do que recebeu em metade dos elementos — a máquina de fazer energia que o
/// clamp do `eval` existe para impedir.
///
/// ⚠️ **A chave é a IDENTIDADE do elemento (`id`), não a posição dele na lista** — pela mesma
/// lei do `pick` do `motion.duplicator`: pôr um `motion.sort` no meio não pode redistribuir
/// quão saltitante cada partícula é. Sem coluna `id`, a posição é a única resposta disponível.
pub(super) fn element_restitution(rest: f32, randomness: f32, seed: u32, key: u32) -> f32 {
    rest * (1.0 - randomness * hash::rand01(seed, key))
}
