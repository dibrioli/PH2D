//! **O TAMBOR** — uma roldana dirigida é um guincho (W-Pulley W2).
//!
//! Módulo FILHO de [`super`] pelo cap de LOC, e o corte é por assunto: lá mora o
//! que a corda faz com os CORPOS (o impulso, a massa efetiva, a ruptura); aqui, o
//! que um tambor faz com a CORDA — a taxa, o recolhido, e o teto que a correção
//! de posição precisou para o guincho não arremessar a carga.

use std::collections::BTreeMap;

use super::PulleyDesc;

/// **Quantos SUB-PASSOS de recolhimento a correção pode ficar para trás** — o
/// teto do termo de posição, e o único guarda que o guincho precisou.
///
/// Um guincho é uma projeção de velocidade com massa efetiva exata, então ele é
/// **onipotente**: MEDIDO, a mesma carga sobe os mesmos 0,9624 m em um segundo
/// pesando 0,1 kg ou 1000 kg. O que o limita não é força, é **geometria** — e o
/// modo de falha era violento. Recolhido até o gancho alcançar a roldana, a carga
/// **orbita** o eixo, `L(rota)` fica **descontínuo** quando a âncora varre por
/// perto, `C` dá um salto finito num sub-passo e `β·C/dt` o converte em uma
/// velocidade arbitrária:
///
/// | t (s) | \|v\| da carga | y (roldana em 8,0) |
/// |---|---|---|
/// | 2,5 | 2,18 m/s | 6,95 |
/// | 3,0 | 13,3 | 9,55 |
/// | 4,0 | **1266** | −0,37 |
/// | 5,0 | **7360** | 55,8 |
///
/// A cura é bater no termo que amplifica: `C` entra na correção **capado**. O
/// termo de VELOCIDADE (`Ċ`) fica exato — é ele que de fato segura a corda —, e o
/// de POSIÇÃO deixa de tentar apagar num sub-passo um salto que não é erro
/// acumulado, e sim descontinuidade.
///
/// ⚠️ **O teto é relativo à TAXA DO TAMBOR, não ao comprimento da corda** — e a
/// medição escolheu por mim. O aperto normal é uma **defasagem cinemática** de
/// 4,1–4,5 sub-passos de recolhimento, que cresce com a velocidade do tambor e
/// **não** com a massa (0,017201 m nas cinco massas medidas):
///
/// | `ω·r` (m/s) | aperto normal | sub-passos de recolhimento |
/// |---|---|---|
/// | 0,25 | 0,0047 | 4,5 |
/// | 0,50 | 0,0089 | 4,3 |
/// | 1,00 | 0,0172 | 4,1 |
/// | 2,00 | 0,0339 | 4,1 |
///
/// Um teto em metros — ou em fração da corda — **estrangula o tambor rápido e não
/// guarda o lento**, porque a grandeza que ele tenta capar escala com a taxa.
/// Varrido no PRODUTO (`sweep_the_winch`), com a carga levada até a roldana:
///
/// | `C_LAG` | altura aos 2,5 s | \|v\| MÁXIMO |
/// |---|---|---|
/// | 4 | 5,9122 (**15 % lenta**) | 10,0 |
/// | 5 | 6,8851 (0,9 % lenta) | 23,1 |
/// | **6** | **6,9461 (exata)** | **20,3** |
/// | 7 | 6,9461 (exata) | 24,7 |
/// | 8 | 6,9461 (exata) | 29,1 |
/// | ∞ | 6,9461 (exata) | **4422** |
///
/// **6** é o primeiro valor em que o recolhimento normal é EXATO (idêntico ao sem
/// teto, casa decimal por casa decimal) e o mais manso entre os exatos — **218×**
/// abaixo do sem-guarda, com 33 % de folga sobre os 4,5 medidos.
///
/// ⚠️ **Uma corda SEM tambor tem teto ∞**, e isso não é conveniência: é o que
/// mantém esta wave **byte-idêntica** para toda cena que já existia.
///
/// ⛔ **MEDIDO E REJEITADO — não refaça, são três.** **(1) Estolar o tambor por
/// APERTO:** com o teto em 0,5 % da corda o aperto **fica normal até o instante da
/// explosão** (0,033 no tique anterior) e o estol dispara tarde — `|v|` máximo
/// 4148, igual a não ter guarda; abaixo disso (0,1 %) ele dispara o tempo todo e a
/// carga sobe 72 % do que subiria. **(2) Estolar quando o gancho encontra a
/// roldana** (*two-blocking*, régua = o raio da roda): funciona como enunciado — o
/// recolhido congela — mas a carga **passa de largo pela inércia** e o estol
/// PIORA: par casado com o teto em 6, `|v|` máximo **20,33 sem o estol contra
/// 30,66 com ele**, e a altura aos 2,5 s idêntica (6,9461) nos dois. Guarda que
/// não guarda é cerca inventada. **(3) Dar um COLLIDER à
/// roldana** para o gancho bater nela: sem o teto, o impulso da corda é ilimitado
/// e **atravessa o contato** (7360 m/s). ✅ **Com o teto, aí sim** — a roldana
/// sólida faz a carga **assentar** nela (`y` oscila em torno de 6,9 com `|v|`
/// caindo a 5,8 no fim), que é o que uma cadernal de verdade faz. A roldana é uma
/// ENTIDADE com `Transform`, então isso é um gesto que o artista já tem.
pub const PULLEY_CORRECTION_LAG: f32 = 6.0;

/// **Avançar os tambores desta corda** e devolver o total recolhido.
///
/// ⚠️ Uma corda sem tambor nunca entra no mapa, e é isso que mantém a wave do
/// motor byte-neutra: sem entrada, `L0` é o autorado ao bit, e toda cena que já
/// existia continua exatamente onde estava.
///
/// O recolhido é **monótono** — um tambor tem catraca e nunca devolve corda.
pub(super) fn reel(payout: &mut BTreeMap<u64, f32>, p: &PulleyDesc, dt: f32) -> f32 {
    if p.motor_rate == 0.0 {
        return payout.get(&p.id).copied().unwrap_or(0.0);
    }
    let slot = payout.entry(p.id).or_insert(0.0);
    *slot += p.motor_rate * dt;
    *slot
}
