//! Quais quinas as ferramentas Fillet / Chamfer podem editar neste frame — a política, num
//! ponto testável.
//!
//! O motor (`ph2d_vec_scene::corner_live`) sabe onde há quina; o `PenTool` sabe onde a quina
//! é agarrada. O que **só a shell** sabe é se a geometria daquele path é **derivada** — se
//! algum `*_live::recook` a reescreve — e essa é a pergunta que decide se o raio pode
//! sobreviver. A porta é [`has_derived_verts`], consultada pelo press das ferramentas de
//! quina (`input_dispatch`). (Era a mesma política da antiga alça de raio do Node, que tinha
//! um lado de DESENHO — removido com a alça; hoje o único consumidor é o gesto das tools.)
//!
//! # Por que geometria DERIVADA não pode ganhar raio por-vértice (e o bug que isto evita)
//!
//! Uma forma paramétrica guarda uma RECEITA no `VecShape::Param` (kind + w/h + valores), e
//! a geometria dela é **derivada**: `vec_shape_live::recook_into` reescreve `path.verts`
//! INTEIRO a cada mudança de parâmetro. O `corner_radius` mora dentro do vértice — logo,
//! um raio autorado numa Live Shape sobrevive até o instante em que o usuário encosta num
//! slider do painel, e some **sem erro nenhum**.
//!
//! Isso é pior que não funcionar: funciona, o usuário confia, e um arrasto de slider
//! desfaz o trabalho dele em silêncio. (E não dá para "consertar" preservando os raios no
//! recook: a CONTAGEM de vértices é função dos parâmetros — mexer no slider de lados de um
//! polígono muda quantas quinas existem. Não há para onde levar o raio da quina que
//! deixou de existir.)
//!
//! O raio de uma forma viva é um **campo dela** (o `Radius` do painel, que o polígono, a
//! estrela e o round-rect já têm — inclusive por-canto no round-rect). O raio por-vértice é
//! para caminho **desenhado**. É a mesma divisão do Illustrator: uma live shape tem as
//! propriedades dela; Live Corners é o que se ganha depois de expandir. Aqui "expandir" é
//! **Convert to Curves**, que já existe e é literalmente descartar o `VecShape`.

use ph2d_ecs::{Entity, SimWorld, VecShape};

use crate::vec_entities::VecEntityMap;

/// Este path tem geometria **DERIVADA** — algum `*_live::recook` reescreve os `verts` dele
/// por trás do artista?
///
/// **A pergunta é sobre a ESCRITA, não sobre a contagem de componentes** (ADR-0132 §5). A
/// versão anterior chamava-se `is_live_shape` e enumerava **um** host; a linha ganhou mais
/// quatro desde então, e a alça passou a aparecer sobre conector, morph e filho de envelope,
/// funcionando e morrendo no frame seguinte — o modo de falha que o cabeçalho deste módulo
/// descreve, reproduzido três vezes.
/// [[feedback_a_condition_that_enumerates_its_readers_rots]]
///
/// **O blend está deliberadamente FORA.** Ele é o único cuja escrita é *condicional*: o
/// `blend_live` só reescreve o spine enquanto `!spine_authored`, e a detecção de autoria
/// compara o `verts` inteiro — onde o `corner_radius` mora. Arrastar a alça **é** o gesto
/// que assume o spine, então o raio sobrevive. Há gate a pinar isso nos dois sentidos.
#[must_use]
pub(crate) fn has_derived_verts(
    sim: &SimWorld,
    map: &VecEntityMap,
    id: ph2d_vec_scene::VecPathId,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let w = sim.world();
    // Escrita INCONDICIONAL na própria entidade: a receita (`vec_shape_live`), a rota
    // (`connector_live`) e a forma morfada (`morph_live`).
    if w.get::<VecShape>(e).is_some()
        || w.get::<ph2d_ecs::VecConnector>(e).is_some()
        || w.get::<ph2d_ecs::VecMorph>(e).is_some()
    {
        return true;
    }
    // ENVELOPE: o componente mora no CONTAINER, não no filho — perguntar só à própria
    // entidade devolveria `false` para o caso reportado. Sobe a cadeia pela porta única
    // que o `envelope_live` já é dono (`container_of`), em vez de reandar a árvore aqui.
    crate::envelope_live::container_of(sim, bits).is_some_and(|c| c != bits)
}

// NOTA: `view` (as alças de raio a desenhar) foi REMOVIDO junto com a alça de raio do Node — o
// arredondar/chanfrar quina virou o par de ferramentas Fillet / Chamfer. O que sobrevive deste
// módulo é a POLÍTICA `has_derived_verts`, que o press dessas ferramentas consulta.

// ⚠️ **O PRESS DO MODO NODE, e a metade que fica ABERTA** (W0.2 do plano 25, medido 2026-07-29).
//
// A mesma doença deste módulo vale para arrastar/inserir um NÓ, não só para escrever um raio: a
// geometria derivada é reescrita por trás do artista. A **forma paramétrica** está curada — o press
// do Node congela a receita dentro do gesto (`vec_convert::freeze_shape_recipe`), exatamente como o
// par Fillet/Chamfer, com arch-gate próprio e o repro em
// `vec_convert_tests::a_node_edit_on_a_live_shape_is_wiped_by_the_next_param_edit`.
//
// Os **hosts de RELAÇÃO** (conector · morph · filho de envelope) seguem no estado antigo: o press do
// Node não os consulta, então um nó ali é aceito e o `*_live::recook` o reverte — no envelope e no
// conector isso é **por frame**, na forma paramétrica era só na próxima edição de parâmetro. Não
// curei por dois motivos, e nenhum é "não deu tempo": (a) congelar NÃO serve aqui (é o que este
// módulo argumenta — soltar a relação destrói o que o artista construiu), então a única cura é
// **RECUSAR**, que é mudança de comportamento visível; (b) a recusa tem de deixar o **blend** de
// fora, cuja escrita PARA quando o artista assume o spine (o `has_derived_verts` já o exclui de
// propósito, e arrastar o spine É o gesto de autoria). Decisão de produto — não construída sem
// pedido, com o mecanismo medido escrito aqui para quem a pegar.

#[cfg(test)]
#[path = "corner_handles_tests.rs"]
mod tests;
