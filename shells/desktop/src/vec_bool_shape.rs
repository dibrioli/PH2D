//! **O PAPEL DE CADA FORMA dentro de uma booleana viva** — a porta única de *"o que esta forma faz
//! aqui?"*, e o chip que a edita.
//!
//! Irmão do [`crate::vec_clip_edit`] no padrão: uma projeção que a shell publica por frame (o
//! painel não alcança o mundo ECS e não deve) mais o escritor que o clique dispara.
//!
//! # Por que UMA porta, e não duas respostas parecidas
//!
//! Dois consumidores fazem a mesma pergunta sobre a mesma forma: o **painel** (*"que verbo mostro
//! aceso?"*) e a **linha da hierarquia** (*"que selo pinto?"*). Se cada um a derivasse por si,
//! bastaria um deles usar a ordem de z e o outro a ordem da árvore para o selo dizer `BSE` numa
//! forma que o motor não trata como base — e a UI passaria a mentir sobre a receita sem uma linha
//! vermelha em lado nenhum.
//!
//! ⚠️ **A resposta vem do PLANO que o `bool_live` acabou de cozinhar**, não de uma segunda
//! caminhada da árvore. O plano é o que está na tela: quem é a base, quem são os operandos e em
//! que ordem. É a mesma disciplina do Apply, que materializa o plano em vez de re-chamar o motor.

use ph2d_ecs::{Entity, SimWorld, VecBoolGroup, VecBoolOp};
use ph2d_vec_scene::VecPathId;

use crate::bool_live::{BoolLive, op_of_code};
use crate::vec_entities::VecEntityMap;

/// **O que esta forma É** dentro da booleana viva que a consome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BoolRole {
    /// A forma mais ao FUNDO: o acumulador inicial. ⚠️ Ela **não tem verbo** — não há nada sobre
    /// o que dobrar antes de existir um acumulado —, e é por isso que o papel dela é um valor
    /// próprio em vez de um verbo qualquer.
    Base,
    /// O verbo com que ela dobra sobre o resultado das anteriores (código de `PathfinderOp`).
    Verb(u8),
    /// O grupo está numa **RECEITA** (`Trim`/`Crop`/`Merge`/`MinusBack`), que é verbo da pilha
    /// inteira: nenhuma forma escolhe nada, e é isto que a UI tem de dizer em vez de oferecer um
    /// seletor inerte.
    Recipe,
}

impl BoolRole {
    /// O selo de três letras da linha da hierarquia.
    ///
    /// ⚠️ **Não passa por i18n, de propósito**, e é a convenção dos selos que já existem (`PRF`,
    /// `CAM`, `SPR`): eles são códigos, e a tabela de TOM da linha casa contra a própria string —
    /// traduzir o selo tiraria a cor dele. O app é inglês-only por decisão do Enio.
    #[must_use]
    pub(crate) fn badge(self) -> &'static str {
        match self {
            BoolRole::Base => "BSE",
            BoolRole::Recipe => "RCP",
            BoolRole::Verb(0) => "UNI",
            BoolRole::Verb(1) => "SUB",
            BoolRole::Verb(2) => "INT",
            BoolRole::Verb(3) => "EXC",
            // Um código que este build não conhece herda o do grupo antes de chegar aqui; se
            // ainda assim escapar, o selo não inventa um verbo.
            BoolRole::Verb(_) => "BSE",
        }
    }
}

/// **O papel de `id`**, ou `None` quando ela não é operando de booleana viva nenhuma.
///
/// ⚠️ A ordem das recusas é a lei, e cada uma é uma decisão:
/// 1. sem plano que a consuma ⇒ ela não está numa booleana viva (ou o grupo não cozinhou);
/// 2. o grupo numa **receita** ⇒ `Recipe`, e nenhuma forma escolhe;
/// 3. ela é a **base** ⇒ `Base`, sem verbo;
/// 4. senão, o verbo dela — o override próprio, ou o do grupo por herança.
#[must_use]
pub(crate) fn role_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
    id: VecPathId,
) -> Option<BoolRole> {
    let (group, plan) = bool_live.plan_containing(id)?;
    let group_op = sim.world().get::<VecBoolGroup>(group)?.op;
    let pf = op_of_code(group_op)?;
    if pf.as_bool().is_none() {
        return Some(BoolRole::Recipe);
    }
    if plan.base == id {
        return Some(BoolRole::Base);
    }
    Some(BoolRole::Verb(effective_code(sim, map, id, group_op)))
}

/// O código do verbo que ESTA forma usa: o override dela quando ele é uma das quatro operações de
/// conjunto, senão o do grupo.
///
/// ⚠️ **Espelha `bool_live::operand_verb`, e é o mesmo `op_of_code` que decide** — o que impede as
/// duas de divergirem é partilharem a tabela de tradução, que é a porta única do repo para
/// *"código do painel ⟶ operação do motor"*.
#[must_use]
fn effective_code(sim: &SimWorld, map: &VecEntityMap, id: VecPathId, group_op: u8) -> u8 {
    map.get(&id)
        .and_then(|&bits| sim.world().get::<VecBoolOp>(Entity::from_bits(bits)))
        .filter(|v| op_of_code(v.op).is_some_and(|p| p.as_bool().is_some()))
        .map_or(group_op, |v| v.op)
}

/// **O SELO DE CADA LINHA da hierarquia**, por bits de entidade — a receita inteira legível de uma
/// olhada, num sítio só.
///
/// É a metade que faz este desenho funcionar. Sem ela o verbo só existe no painel lateral, uma
/// forma de cada vez: para entender uma booleana de cinco formas o artista teria de clicar cinco
/// vezes e guardar o resultado de cabeça — que é exactamente a queixa que matou o diagrama
/// (*"confuso de usar"*). A hierarquia já mostra a ORDEM; o selo acrescenta o VERBO, e a ordem
/// mais o verbo **são** a receita.
///
/// ⚠️ **E ele descreve o DOCUMENTO, nunca o quadro** (auditoria de 2026-08-23). Durante uma
/// transição de estados a booleana desenha um MEIO entre duas operações, e o selo continua a dizer
/// o verbo de PARTIDA até a chegada — porque é esse que o componente guarda. As duas coisas são
/// perguntas diferentes: *que receita este documento tem?* responde-se aqui, *o que a tela mostra
/// agora?* responde-se olhando para a tela. ⛔ Ligar o selo ao morph dá-lhe um terceiro estado
/// (*"a caminho de"*) que nenhum gesto pode editar — e um rótulo que muda sozinho num controlo que
/// não se pode mexer é ruído, não informação.
///
/// ⚠️ **Ele lê o plano do quadro ANTERIOR, e isso é deliberado.** A hierarquia publica cedo no
/// frame e a booleana cozinha tarde (`run_render_frame`), então o plano em mãos aqui é o do
/// cozimento passado. O atraso é de um quadro — ~16 ms, abaixo do perceptível — e a alternativa
/// seria mover uma das duas metades na ordem do frame, que é mudança com gates próprios e sem
/// nada a ganhar.
#[must_use]
pub(crate) fn badges(
    sim: &SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
) -> std::collections::BTreeMap<u64, &'static str> {
    map.iter()
        .filter_map(|(&id, &bits)| Some((bits, role_of(sim, map, bool_live, id)?.badge())))
        .collect()
}

/// **A FILEIRA que o painel deve mostrar** — *(verbo aceso, nome da forma)*. `None` faz a fileira
/// não existir.
///
/// # ⚠️ O sujeito é o PRIMÁRIO, nunca "a seleção inteira"
///
/// A primeira versão exigia *"exactamente UMA forma selecionada"*, e **nenhum clique deste editor
/// pode satisfazer isso**: tocar um filho **seleciona o grupo inteiro** — é lei deliberada
/// (`input_dispatch`: *"Tocar um filho seleciona o GRUPO (a árvore é a Hierarquia)"*). A fileira
/// nunca aparecia, e foi por isso que os quatro chips «não responderam ao clique»: eles não
/// estavam na tela.
///
/// A porta certa já existia: `set_object_selection` **preserva o primário** quando ele está na
/// lista (`ph2d-vec-edit/selection.rs`), então depois do clique `selected()` continua a ser *a
/// forma que o dedo apontou*, com os irmãos todos selecionados à volta.
///
/// # E por que ela devolve o NOME
///
/// Com o grupo inteiro aceso no canvas, um rótulo genérico não diz de QUAL das formas ele fala — e
/// o artista escolheria o verbo no escuro. A fileira **nomeia o próprio sujeito**; o selo na linha
/// da hierarquia confirma. ⚠️ Sem `Name` o nome vem vazio, e o painel cai no rótulo genérico: o
/// nome é dado do documento, não copy de UI.
#[must_use]
pub(crate) fn shape_row_of_selection(
    sim: &SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
    sel: &[VecPathId],
    primary: Option<VecPathId>,
) -> Option<(u8, String)> {
    let id = primary?;
    // ⚠️ O primário tem de estar **na seleção**: ele é pegajoso e sobrevive a uma seleção nova que
    // não o contenha, e sem esta conferência a fileira falaria de uma forma que já não está em mãos.
    if !sel.contains(&id) {
        return None;
    }
    let BoolRole::Verb(code) = role_of(sim, map, bool_live, id)? else {
        return None; // a BASE não tem verbo; uma RECEITA é do grupo inteiro
    };
    let name = map
        .get(&id)
        .and_then(|&bits| sim.world().get::<ph2d_ecs::Name>(Entity::from_bits(bits)))
        .map_or_else(String::new, |n| n.as_str().to_owned());
    Some((code, name))
}

/// **O CHIP CLICADO → o código do verbo.** `None` = o id não é um dos quatro.
///
/// Irmão do `vec_layout_edit::layout_edit_for_id` e do `contour_live::join_code_of_id`: a shell
/// tem uma dúzia destas portas, e todas existem pela mesma razão — **um `match` de id enterrado
/// dentro do `render_loop` não é alcançável por teste nenhum**, e o `render_loop` exige janela e
/// GPU para correr.
///
/// ⚠️ Esta porta nasceu de um defeito: os quatro chips shiparam sem um único gate no caminho
/// *id ⟶ componente escrito*, e o que faltava (o sujeito ser o primário) só apareceu no smoke do
/// Enio. O mapeamento estava certo; o que não existia era o sítio onde perguntá-lo.
///
/// ⚠️ **A posição no array É o código** (`PathfinderOp` 0..=3), e não uma segunda tabela: a ordem
/// dos quatro chips é a dos quatro primeiros discriminantes, e uma tabela paralela divergiria dela
/// no dia em que alguém reordenasse a fileira do painel.
#[must_use]
pub(crate) fn shape_op_for_id(id: ph2d_editor::ids::NodeId) -> Option<u8> {
    [
        ph2d_editor::ids::VECTOR_BOOL_SHAPE_UNION,
        ph2d_editor::ids::VECTOR_BOOL_SHAPE_SUBTRACT,
        ph2d_editor::ids::VECTOR_BOOL_SHAPE_INTERSECT,
        ph2d_editor::ids::VECTOR_BOOL_SHAPE_EXCLUDE,
    ]
    .iter()
    .position(|chip| *chip == id)
    .and_then(|i| u8::try_from(i).ok())
}

/// Escreve o verbo `code` na forma **primária**. Devolve `true` se alguma coisa mudou.
///
/// ⚠️ Ele **repete a triagem** em vez de confiar no que o painel mostrou: entre pintar a fileira e
/// o clique chegar passa um frame, e nele a seleção pode ter mudado. Escrever sem reconferir é
/// como um clique numa forma acaba a mudar outra.
pub(crate) fn set_selected_shape_op(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
    sel: &[VecPathId],
    primary: Option<VecPathId>,
    code: u8,
) -> bool {
    if shape_row_of_selection(sim, map, bool_live, sel, primary).is_none() {
        return false;
    }
    let Some(&bits) = primary.and_then(|id| map.get(&id)) else {
        return false;
    };
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(VecBoolOp { op: code });
    true
}

#[cfg(test)]
#[path = "vec_bool_shape_tests.rs"]
mod tests;
