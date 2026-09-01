//! **A RÉGUA DO TAMANHO** — a largura média de Cauchy, o oráculo de toda a lei do crescimento.
//!
//! ⚠️ **Módulo irmão do [`crate::turtle`] por RESPONSABILIDADE, não por tamanho.** A tartaruga
//! responde *«que desenho é este?»*; isto responde *«que tamanho tem?»* — e a segunda pergunta
//! é a que o `Growth` faz, a que a escada de tamanhos inverte e a que os gates de ondulação
//! medem. Ela atravessa a tartaruga (chama [`crate::turtle::walk`]) e nenhum passo da
//! interpretação a chama de volta: a dependência é de sentido único, e é por isso que o corte
//! aqui não parte nada.

use crate::grammar::Module;
use crate::turtle::{DEGREES_PER_TURN, Setup, dir, walk};
use ph2d_nodegraph::attr::Column;

/// ⭐⭐⭐ **QUANTAS DIREÇÕES A RÉGUA AMOSTRA — medido, não escolhido** (§0.0).
///
/// Rodando a MESMA figura de `0` a `90°` pelo `Root Angle`, uma régua perfeita não muda nada.
/// Medido pela bancada [`examples/probe_ruler.rs`](../../examples/probe_ruler.rs) sobre os oito
/// moldes, e o custo por A/B contra a régua antiga
/// ([`examples/probe_ceiling.rs`](../../examples/probe_ceiling.rs)):
///
/// | régua | ondulação ao rodar | custo no TECTO |
/// |---|---|---|
/// | `max(w, h)` — a de até 2026-08-30 | **`10,9 %`–`32,5 %`** (varia com a forma) | grátis |
/// | `K = 4` | `7,8 %` | ~0,4 ms |
/// | `K = 8` | `1,9 %` | ~0,7 ms |
/// | **`K = 16`** | **`0,48 %`** | **`1,46 ms`** (8,7 % de um quadro) |
/// | `K = 32` | `0,12 %` | ~2,9 ms |
/// | `K = 64` | `0,06 %` | ~5,8 ms |
///
/// ⚠️ **Só o `K = 16` foi cronometrado; as outras linhas são LINEARES nele** (uma passagem
/// sobre a nuvem com `K` acumuladores), e estão marcadas com `~` por isso.
///
/// ⚠️⚠️ **A ondulação da média é uma constante de `K`, não uma propriedade da figura** — medido,
/// um rectângulo `1×3`, uma agulha, um quadrado e uma agulha de aspecto `100` dão os MESMOS
/// `0,48 %` a `K = 16` (só um círculo desce, a `0,115 %` — e ali o número é o MESMO em `K = 8`,
/// `16` e `32`, porque um círculo tem largura constante e não há nada que a discretização
/// deixe por resolver). É a régua
/// de EIXO que varia com a forma, de `10,9 %` num rectângulo `3:1` a `32,5 %` no pior molde.
///
/// ⛔⛔ **E o TECTO desta tabela foi corrigido pela auditoria de 2026-08-30 — a 1.ª redacção
/// nomeava `65 537` elementos e o alcançável é `262 145`, o QUÁDRUPLO.** Eu tinha derivado o
/// pior caso da cadeia do **Dragon**, e o pior caso é da gramática que mais DESENHA: o Dragon é
/// metade viragens, e um `F -> FFFF` é tudo `F`. ⚠️ E ele está **dentro do arrasto do slider**
/// (`8,5` gerações, e o slider vai a 12), não a três vezes acima dele.
///
/// Medido por A/B no tecto (`F -> FFFF` a `g = 8,5`, 262 145 desenhados): a cozedura custa
/// `11,23 ms` com a régua antiga e `12,69` com esta ⇒ **`+1,46 ms`, `+13 %`**. ⚠️ Os `76 %` de
/// um quadro que a cozedura ali custa são **quase todos pré-existentes** — as três travessias
/// de medição, que a lei do crescimento já pagava antes desta mudança.
///
/// ⇒ o `16` é o joelho: **`23×`–`68×`** melhor que a caixa de eixo por `8,7 %` de um quadro no
/// pior caso, e por `0,05 ms` (`0,3 %`) num molde do catálogo no máximo do slider.
///
/// ⚠️ **O recurso é o QUADRO, e o número diz de que ele é.**
const WIDTH_DIRECTIONS: usize = 16;

/// ⭐⭐⭐ **O TAMANHO desta figura — a LARGURA MÉDIA de Cauchy**, e ela é o oráculo de toda a
/// lei do crescimento.
///
/// `largura(u) = max⟨P,u⟩ − min⟨P,u⟩`, e o que se devolve é a MÉDIA dela sobre
/// [`WIDTH_DIRECTIONS`] direções uniformes no semicírculo. Para um convexo isto é o
/// `perímetro/π`.
///
/// # ⚠️⚠️ Por que não é a caixa alinhada aos eixos (report do Enio, 2026-08-30)
///
/// *"em dragon enquanto cresce (aumentando Generations) parece piscar"*. Ele estava a ver
/// um defeito da RÉGUA: até 2026-08-30 isto devolvia `max(w, h)`, que **não é invariante à
/// rotação** — e a curva do dragão **roda `45°` por geração** por construção. A lei põe o
/// que esta função devolve numa rampa recta; quando a caixa troca de lado longo, a lei passa
/// a fixar a OUTRA dimensão e o tamanho verdadeiro **estagna e depois arranca**. Medido: o
/// menor passo do arrasto era `4,5 %` do passo médio (uma paragem), e a régua de eixo lia
/// `66,6 %` — *cega ao defeito que ela própria causava*.
///
/// # ⛔ Duas réguas invariantes foram MEDIDAS e REJEITADAS, pela mesma causa
///
/// | tentativa | por que caiu |
/// |---|---|
/// | raio de giração (RMS) | é medida de **distribuição**: ao atravessar uma geração a contagem de elementos DUPLICA e os novos nascem coincidentes com os pais ⇒ salto puro de amostragem (Tree: passo `−7 991 %` do médio) |
/// | maior distância ao **centroide** | o centroide salta pela mesma razão (Tree `151×`, Wild `395×` de ondulação) |
///
/// ⇒ a régua tem de ser um EXTENSO **sem centroide**: `max − min` é invariante à translação
/// por construção, e pontos coincidentes não o movem.
///
/// ⚠️ **As direções saem do [`dir`], não de `f32::cos`** — a mesma tabela sem transcendentais
/// que a tartaruga usa, por HR-5. O `0,09 %` de erro de direção é comum às três medições que
/// a lei compara, então cancela na razão.
///
/// ⛔ **E uma terceira hipótese caiu por medição: a figura NÃO salta de sítio.** Um
/// deslocamento lê-se da cadeira como um salto de tamanho, e a lei não olha para onde a figura
/// está — mas medido (`examples/probe_drift.rs`), o pior salto de posição do Dragon num passo do
/// slider é **`0,51 %` do tamanho dele, o MENOR dos oito moldes**, contra `10,75 %` do Tree, de
/// que ninguém se queixou. *O molde acusado é o que menos se mexe.*
///
/// ⚠️ A bancada que mede o defeito e a cura é
/// [`examples/probe_flicker.rs`](../../examples/probe_flicker.rs) (o arrasto, com o observador
/// invariante ao lado da régua de eixo); a que escolhe o `K` é
/// [`examples/probe_ruler.rs`](../../examples/probe_ruler.rs).
///
/// ⚠️ **Chama o `walk` e deita o stream fora, de propósito.** Uma segunda travessia «leve»
/// seria a MESMA lei escrita duas vezes, e é a família de defeito que este módulo já pagou
/// três vezes; o preço de uma alocação vale mais que a divergência.
///
/// ⚠️ **Ela só corre numa geração fraccionária** — numa inteira a âncora não é precisa e
/// ninguém a mede (medido: a inteira é `2,5×`–`2,7×` mais barata). ⛔ **Com uma excepção que a
/// auditoria de 2026-08-30 nomeou: `Growth < 1` torna TODA posição do slider fraccionária**
/// (`g = 12,0` custa `0,114 ms` com `Growth = 1,0` e `0,305 ms` com `0,999`).
pub(crate) fn mean_width(chain: &[Module], set: &Setup) -> f32 {
    let s = walk(chain, set);
    let Some(Column::Vec2(v)) = s.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    // As direções, UMA vez por travessia — nunca por elemento (a mesma cerca do `powf`).
    let mut u = [(0.0f32, 0.0f32); WIDTH_DIRECTIONS];
    for (k, slot) in u.iter_mut().enumerate() {
        let deg = DEGREES_PER_TURN / 2.0 * k as f32 / WIDTH_DIRECTIONS as f32;
        // ⚠️ `sn`, e não `s`: o `s` desta função é o stream, e sombreá-lo aqui já enganou uma
        // leitura.
        let (c, sn, inv) = dir(deg);
        *slot = (c * inv, sn * inv);
    }
    // ⚠️ **Os pontos por FORA e as direções por DENTRO** — uma passagem só sobre a nuvem, com
    // os 32 acumuladores (16 mínimos + 16 máximos) em registos. Ao contrário (`K` passagens
    // completas) paga-se `K` vezes o tráfego de cache: medido, a forma invertida é **1,24×**
    // mais lenta. ⚠️ A bancada `probe_ruler.rs` usa a forma INVERTIDA, então os relógios dela
    // são um tecto — é por isso que a coluna de custo da tabela do [`WIDTH_DIRECTIONS`] vem de
    // um A/B do PRODUTO (`probe_ceiling.rs`) e não dela.
    let mut lo = [f32::MAX; WIDTH_DIRECTIONS];
    let mut hi = [f32::MIN; WIDTH_DIRECTIONS];
    for q in v {
        for k in 0..WIDTH_DIRECTIONS {
            let t = q[0] * u[k].0 + q[1] * u[k].1;
            lo[k] = lo[k].min(t);
            hi[k] = hi[k].max(t);
        }
    }
    // ⛔ **RECUSA MEDIDA (auditoria 2026-08-30): NÃO troque o `f32::min/max` por um `if`.**
    // A asm não vectoriza (o corpo é escalar, 16 `minss` + 16 `maxss` totalmente desenrolados),
    // o que convida à hipótese de que um `if t < lo[k]` seria mais rápido. Medido: ele é
    // **2,4× MAIS LENTO** com bits idênticos — o `minss` é sem ramo, e o `if` gera saltos
    // imprevisíveis sobre dados geométricos.
    let total: f32 = (0..WIDTH_DIRECTIONS).map(|k| hi[k] - lo[k]).sum();
    total / WIDTH_DIRECTIONS as f32
}
