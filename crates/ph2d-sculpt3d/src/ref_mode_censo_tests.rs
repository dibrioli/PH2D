//! ⛔⛔⛔ **O CENSO DAS REFERÊNCIAS QUE NINGUÉM DECLARA** — irmão (`#[path]`) do
//! [`super::tests`], e o corte é o ASSUNTO: lá *que lei cada modo declara*, aqui
//! *quem fica SEM modo que o declare* e por isso cai no slider cru.
//!
//! ⚠️ Ele nasceu de um defeito que **22 fixturas verdes não viam**, e o cabeçalho
//! do próprio gate conta-o.

use super::*;

/// ⛔⛔⛔ **O CENSO DAS REFERÊNCIAS QUE NINGUÉM DECLARA** — quais verbos caem no
/// **slider cru** porque o modo que os declara não tem perfil para eles.
///
/// ⚠️⚠️ **A tabela do [`RefMode::declares`] é uma LISTA NEGRA, e a direcção é o
/// defeito:** um verbo novo nasce a reivindicar o `S`, o [`Verb::profile`]
/// devolve `None` naquele modo, e o [`crate::Brush::weight`] cai no `map_or` —
/// entregando **força LINEAR** onde a referência que de facto governa o verbo
/// pede o **quadrado**.
///
/// ⛔ **E ele é MUDO à força cheia**, que é o que o torna caro: `s² = s` em `1`,
/// logo um corpus na força máxima não o vê. Foi assim que ele chegou ao pincel de
/// plano com **22 fixturas verdes** — só a célula de força `0,5` o acusou, e por
/// um factor exacto de `2×` (`0,104798` contra `0,052399` do alvo).
///
/// ⇒ este censo lista quem está nessa posição **hoje**, e a lista é uma catraca
/// que só **encolhe**: cada entrada tem de dizer porque é inofensiva.
#[test]
fn o_censo_das_referencias_que_ninguem_declara() {
    let caem: Vec<&str> = Verb::ALL
        .iter()
        .copied()
        .filter(|v| {
            // O `weight` pergunta ao modo que o verbo declara; se esse modo não
            // tem perfil, o peso é o slider CRU.
            let modo = RefMode::default().for_verb(*v);
            v.profile(modo).is_none()
        })
        .map(Verb::label)
        .collect();
    // ⭐⭐ **O PINCEL DE PLANO NÃO ESTÁ NA LISTA, e é a prova da cura:** ele entrou
    // na lista negra do `S`, logo o `for_verb` recua para o `B`, que **tem** a
    // ferramenta — e é de lá que vem a força ao quadrado que o corpus exige.
    //
    // ⚠️ **A lista é NOMEADA, e cada membro tem de ser inofensivo por uma razão
    // ESCRITA** — não por ninguém ter olhado:
    //
    // | verbo | porque o slider cru não o machuca |
    // |---|---|
    // | `Cloth` · `Pose` · `Boundary` | desviam antes do dab ([`Verb::resolve_a_propria_regiao`]) e a lei deles lê `brush.strength` **directo** — o `weight` não está no caminho |
    // | `Density` · `Box Trim` | não têm lei por-vértice ([`Verb::sem_lei_por_vertice`]): não há barro a pesar |
    //
    // ⛔⛔ **E os QUATRO que ficam são DÍVIDA NOMEADA, não isenção** — os três
    // pincéis de multiresolução/projecção e o **afiar**:
    //
    // - o **apagador**, o **esfregão** e a **projecção** atravessam o dab (são
    //   [`crate::Grip::Stamp`]), logo o peso deles é o slider cru — força
    //   **linear** onde a espec deles diz **quadrado**
    //   (`SPEC_unblocked_brushes.md` §1.1, com o `0,5²` medido no corpus da
    //   projecção). ⚠️ **O corpus dos três é todo à força CHEIA**, onde `s² = s`
    //   ⇒ nenhum gate deles pode ver isto hoje. *A cura é uma fixtura de meia
    //   força em cada, que é acto de E — a mesma que apanhou o pincel de plano.*
    // - ⭐ o **AFIAR** é o achado desta passagem, e ninguém o procurava: o `S`
    //   **declara-o** (ele não está na lista negra) e o
    //   `the_reference_has_no_sharpen_and_the_table_says_so` afirma, no mesmo
    //   ficheiro, que aquela referência **não tem** esta ferramenta. ⇒ as duas
    //   tabelas discordam, e o preço é o mesmo: força linear, muda à força cheia.
    //
    // ⚠️ **A ORDEM é a do [`Verb::ALL`]** e não alfabética: ela é o que torna o
    // diff legível quando um verbo entra ou sai.
    let esperado = [
        "Sharpen",
        "Cloth",
        "Pose",
        "Boundary",
        "Density",
        "Erase Displacement",
        "Smear Displacement",
        "Scene Project",
        "Box Trim",
    ];
    assert_eq!(
        caem, esperado,
        "a lista de quem cai no slider CRU mudou — se um verbo ENTROU, a forca \
         dele passou a ser linear em silencio; se SAIU, apague-o daqui"
    );
}
