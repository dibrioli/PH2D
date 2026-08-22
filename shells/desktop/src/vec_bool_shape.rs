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

/// **O verbo que o painel deve mostrar aceso** — `None` faz a fileira não existir.
///
/// A regra inteira mora aqui, e é o que o `state::set_bool_shape_op` documenta do outro lado:
/// exactamente **uma** forma selecionada, operando de uma booleana viva, que **não** é a base e
/// cujo grupo **não** está numa receita.
#[must_use]
pub(crate) fn shape_op_of_selection(
    sim: &SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
    sel: &[VecPathId],
) -> Option<u8> {
    // ⚠️ **Exactamente uma.** O verbo é propriedade de UMA forma; oferecido sobre um conjunto, o
    // chip aceso teria de responder por várias respostas diferentes — e o clique escreveria em
    // todas sem o artista ter pedido.
    let [id] = sel else {
        return None;
    };
    match role_of(sim, map, bool_live, *id)? {
        BoolRole::Verb(code) => Some(code),
        BoolRole::Base | BoolRole::Recipe => None,
    }
}

/// Escreve o verbo `code` na forma selecionada. Devolve `true` se alguma coisa mudou.
///
/// ⚠️ Ele **repete a triagem** em vez de confiar no que o painel mostrou: entre pintar a fileira e
/// o clique chegar passa um frame, e nele a seleção pode ter mudado. Escrever sem reconferir é
/// como um clique numa forma acaba a mudar outra.
pub(crate) fn set_selected_shape_op(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    bool_live: &BoolLive,
    sel: &[VecPathId],
    code: u8,
) -> bool {
    if shape_op_of_selection(sim, map, bool_live, sel).is_none() {
        return false;
    }
    let [id] = sel else {
        return false;
    };
    let Some(&bits) = map.get(id) else {
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
