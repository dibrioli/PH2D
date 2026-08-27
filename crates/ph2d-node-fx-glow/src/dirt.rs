//! **A MÁSCARA DE SUJIDADE do halo** — o *Dirt Texture* / *Dirt Intensity* do Unity URP e o
//! *Bloom Dirt Mask* do Unreal (doc 89 folha 11, a última célula P2 da folha).
//!
//! ⚠️ **Ela é DUAS perguntas, e por isso são dois params de tipos diferentes:** *qual imagem*
//! (um NOME, no canal de texto do `Graph` — doc 32) e *quanto ela acende* (um `f32` do
//! manifesto). Pôr as duas no mesmo canal era impossível nas duas direcções: um nome não é um
//! `f32`, e um `ParamSpec` é `&'static` num manifesto congelado (§6).
//!
//! ## Por que a imagem se escolhe pelo NOME de um objecto da cena
//!
//! O grafo não vê nada que a aplicação possua — é essa a propriedade que o torna memoizável e
//! reproduzível ao bit. O que atravessa é o **canal externo** (doc 65), e a shell já publica
//! toda sprite NOMEADA da cena nele, como uma instância que carrega a aparência dela
//! (`uv_rect` + `texture_id`). O `source.object` já lê exactamente isso, e o
//! [`ParamWidget::Source`](ph2d_node_registry::ParamWidget::Source) já pinta a lista viva dos
//! nomes publicados.
//!
//! ⇒ **Escolher a sujidade é escolher um objecto pelo nome**, com a mesma UI, a mesma
//! resolução de textura e as mesmas três fontes (`Atlas` / `Individual` / `CookedTexture`) que
//! toda sprite deste app tem. ⚠️ A célula precificava justamente essa resolução como *"a fiação
//! cara"*; ela shipou pela folha 14 — ver o cabeçalho de `ph2d-render/src/motion_fx_dirt.rs`.
//!
//! ⚠️ **Um objecto usado como DADO fica visível na cena, e isso é do artista resolver** (o olho
//! da Hierarquia desliga-o, e o publicador não filtra por visibilidade — de propósito: ele
//! responde *"o que a cena tem"*, não *"o que a câmara vê"*). A alternativa seria um selector de
//! ASSET, que hoje não existe como widget e é uma wave de UI própria.

use ph2d_node_registry::ParamGateText;
use ph2d_nodegraph::graph::Graph;

/// A chave do param de TEXTO em que o nome do objecto viaja.
///
/// ⚠️ **`dirt` e não `dirt_texture`:** o valor não é uma textura, é o nome de uma coisa da cena
/// de que a textura sai — e um nome de param carrega contrato. Chamá-lo `*_texture` prometeria
/// um id de textura, que é o que este canal justamente não carrega.
pub const DIRT_KEY: &str = "dirt";

/// **QUANTO a máscara acende** — o `ParamSpec` (`f32`), `0` = o passe de sempre.
pub const DIRT_INTENSITY: &str = "dirt_intensity";

/// O nome do objecto que serve de máscara, ou `None` — o primeiro `fx.glow` do grafo, a mesma
/// escolha que [`super::from_graph`] e [`super::bake_halo_lut`] fazem (e que o `Deficit::Shadowed`
/// diagnostica quando há mais de um).
///
/// ⚠️ **Vazio e só-espaços contam como AUSENTE — e é a MESMA regra que o painel usa.**
/// O `has_text` do `motion_bridge_params` (que decide se a linha `Dirt Intensity` é
/// pintada) faz `!v.trim().is_empty()`, exactamente como isto. Se as duas divergissem, o
/// artista veria o knob para uma escolha que a máscara considera ausente — um controle
/// pintado sobre nada, que é o defeito que o gate de texto existe para não ter. O campo de texto do painel deixa apagar o nome
/// até à string vazia, e um external chamado `""` não existe — então a diferença entre *"apaguei
/// a escolha"* e *"escolhi uma coisa sem nome"* não é uma diferença: as duas são *sem máscara*.
#[must_use]
pub fn source(graph: &Graph) -> Option<String> {
    let node = graph
        .nodes()
        .iter()
        .find(|n| n.type_name == super::TYPE_NAME)?
        .id;
    let name = graph
        .node_text_param_overrides(node)
        .and_then(|m| m.get(DIRT_KEY))?;
    // ⚠️⚠️ **O trim decide a AUSÊNCIA; ele não reescreve o VALOR.** A 1.ª versão devolvia
    // `name.trim().to_string()`, e isso tornava um nome com espaço **impossível de casar**:
    //   - `motion_bridge_objects::publish` publica a chave CRUA (só salta as vazias-após-trim),
    //   - `source_options` devolve `externals().keys()` verbatim ⇒ o chip mostra `"Lens Dirt "`,
    //   - clicar escreve `"Lens Dirt "` no text param,
    //   - isto aparava para `"Lens Dirt"`, e o `resolve` compara com igualdade EXACTA
    //     (deliberadamente — aparar lá faria `"Dirt"` e `"Dirt "` colidirem só nesta feature).
    // ⇒ o artista escolhia da lista que o próprio app pintou e o diagnóstico respondia
    // *«nenhuma sprite NOMEADA "Lens Dirt" na cena»* com ela na Hierarquia. E o espaço é
    // **invisível** ali: `Name::new` não apara e o `unique_name_excluding` também não.
    //
    // ⚠️ **Toda a família irmã já fazia assim** — `motion.look_at` filtra por `trim().is_empty()`
    // e passa o `n` **não aparado** ao `position_of`; `source.object` e `motion.path` passam o
    // `text_param` cru. *Um valor aparado num sítio e cru no outro são duas respostas à mesma
    // pergunta, e a que o artista vê é a que envelhece.*
    (!name.trim().is_empty()).then(|| name.clone())
}

/// O gate que esconde a intensidade enquanto não há imagem.
///
/// ⚠️ **A imagem vem PRIMEIRO no painel e a intensidade é gateada por ela** — a ordem das
/// perguntas: sem imagem escolhida o knob não faz nada, e *um controle que não faz nada não é
/// pintado* (a mesma lei do `angle` sob `stretch = 1`, com a condição do outro lado da fronteira
/// `f32` — é para isso que o [`ParamGateText`] existe). As duas linhas moram no `PARAM_HINTS` do
/// pai porque o registry guarda **uma** fatia por nó (`insert`, não `extend`), e duas chamadas
/// deixariam a segunda a apagar a primeira em silêncio.
pub static GATES: &[ParamGateText] = &[ParamGateText {
    param: DIRT_INTENSITY,
    when_text: DIRT_KEY,
    when_present: true,
}];

/// O teto DIGITÁVEL da intensidade — o mesmo `64` da `intensity`, e pela mesma razão.
///
/// ⚠️ **Este número é AUTORAL, não de recurso, e o doc diz qual é qual.** A contribuição final
/// é `glow · (tint + dirt·isto) · intensity`, e o teto de RECURSO daquele produto é o do formato
/// (`Rgba16Float`, 65 504), que já é onde o `clamp` mora — a montante, sobre a entrada do
/// bright-pass. Pôr aqui um número derivado do formato seria fingir que este knob sozinho
/// decide a saturação, quando quem a decide é a composição de três. O curso do slider é o da
/// MÃO (a faixa em que a referência trabalha); o `64` é o alcance da MÁQUINA por digitação.
///
/// ⚠️ **Esta frase dizia «o `4` do slider» e o slider é `8`** (auditoria de 2026-08-27): a prosa
/// ficou parada na 1.ª versão, e o slider subiu quando a régua por-pixel restrita ao halo o
/// mediu. O gate que existe — `the_typed_ceiling_is_wider_than_the_hand` — afirma só
/// `hard.max > hint.max`, verdade para `4` **e** para `8`, então a frase velha era invisível.
/// ⇒ o número deixa de ser repetido aqui: quem o quer lê o `PARAM_HINTS`, que é a fonte.
/// *Uma prosa que repete um número de outro sítio é a cópia que envelhece primeiro.*
pub const HARD_MAX: f32 = 64.0;

#[cfg(test)]
#[path = "dirt_tests.rs"]
mod tests;
