//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

// ⛔ **Havia um `INSP_INSTANCE_SECTION` e ele SAIU** (Enio, 2026-08-27): a superfície deixou de ser
// uma seção e passou a ser um CARTÃO no topo, que não tem cabeçalho, nem recolher, nem âncora de
// nota — logo não tem o que registar. *Um id que nada regista é um id que nada resolve.*

/// Quantos valores por eixo. ⚠️ Mesmo teto e mesma razão do irmão do vetor.
pub const MAX_INSTANCE_AXIS_VALUES: usize = 8;

/// ⭐⭐⭐ **Quantos DEGRAUS da escada do *Aplicar* o cartão endereça** (F5 critério 4).
///
/// ⚠️ **É um teto de TABELA DE IDS, e ele diz de que recurso é** — o mesmo do
/// [`MAX_INSTANCE_AXIS_VALUES`]: os ids são `const` para o censo do
/// `hit_indexed_ids_are_registered` os poder **ver**, e uma tabela `const` tem um tamanho.
///
/// ⛔ **Não é o limite de aninhamento do produto.** Uma cena com nove receitas encaixadas continua
/// a funcionar; o que acontece é que o 9.º degrau fica **fora do cartão**, e a fileira diz quantos
/// ficaram — ⛔ escrito, nunca truncado em silêncio. O *Aplicar ao mestre* do menu (o degrau mais
/// externo) alcança-se sempre, porque não passa por esta tabela.
pub const MAX_INSTANCE_APPLY_LEVELS: usize = 8;
