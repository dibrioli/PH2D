//! **AS DIFERENÇAS de uma instância** — a porta do override, o absorver e a troca de mestre
//! (plano UI/UX W5b).
//!
//! Irmão do [`crate::vec_component_edit`] pelo teto de LOC, e o corte é o assunto: ali moram os
//! verbos que fazem uma instância EXISTIR (criar, colocar, destacar), aqui os que a fazem DIFERIR.
//!
//! # A W5a shipou o modelo do override sem uma porta que o produzisse
//!
//! `OverrideSlot::Fill` e `OverrideSlot::Hidden` existiam, com gates, e **nada no editor podia
//! criar um**: o único escritor era um teste. O *Reset Overrides* era um botão que nunca podia ser
//! preciso. É a forma exata de [[feedback_a_capability_without_a_door_passes_every_gate]] — e
//! nenhum gate a vê, porque cada metade está certa sozinha.
//!
//! # A lista de peças é a sub-árvore INTEIRA, e isso decide o gesto
//!
//! Uma linha por peça do mestre, com um interruptor e uma cor. ⚠️ Se a lista fosse
//! [`crate::instance_live::visible_pieces`], esconder uma peça **tirar-lhe-ia a linha** e o gesto
//! seria de mão única: o artista perderia a peça sem um erro, e a única volta seria o *Reset*, que
//! também apaga tudo o resto.
//!
//! # O endereço é a peça do MESTRE, nunca o índice da linha
//!
//! A linha `row` é só onde ela foi pintada. O que fica guardado é o `VecPathId` da peça no mestre
//! ([`ph2d_ecs::InstanceOverride::sub`]), e é isso que faz o override sobreviver a editar o mestre
//! — e a acrescentar-lhe peças, que reordenaria as linhas.

use ph2d_ecs::{Entity, InstanceOverride, Name, OverrideSlot, SimWorld, VecInstance};
use ph2d_vec_scene::{Paint, Rgba8, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// A espécie `Fill` de [`OverrideSlot`] — o mesmo número que `OverrideSlot::kind` devolve.
const FILL_KIND: u8 = 0;
/// A espécie `Hidden`.
const HIDDEN_KIND: u8 = 1;

/// **As peças que a lista do painel endereça**, e quantas ficaram além do teto.
///
/// A porta é UMA: quem PUBLICA as linhas e quem RESOLVE um clique perguntam a mesma coisa, no
/// mesmo frame, ao mesmo mundo. Uma segunda travessia daria uma ordem que poderia diferir — e o
/// sintoma seria o clique na linha 2 a recolorir a peça 3, sem erro nenhum.
#[must_use]
pub(crate) fn addressed_pieces(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main_e: Entity,
) -> (Vec<VecPathId>, usize) {
    let all = crate::instance_live::subtree_paths(scene, sim, map, main_e);
    let cap = ph2d_editor::ids::MAX_INSTANCE_PIECES;
    let beyond = all.len().saturating_sub(cap);
    (all.into_iter().take(cap).collect(), beyond)
}

/// **As linhas que o painel pinta** para a instância `inst` do mestre `main_e`.
#[must_use]
pub(crate) fn piece_rows(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main_e: Entity,
    inst: &VecInstance,
) -> (Vec<ph2d_panel_vector::state::InstancePiece>, usize) {
    let (pieces, beyond) = addressed_pieces(sim, scene, map, main_e);
    let rows = pieces
        .iter()
        .map(|&piece| {
            let own = inst.get(piece, FILL_KIND);
            let colour = match own {
                Some(OverrideSlot::Fill(rgba)) => rgba,
                // Sem override, a cor EFETIVA é a do mestre. Um gradiente (ou nenhuma tinta) não
                // tem uma cor: a swatch mostra transparente, e autorar um override ali é
                // legítimo — o override É sólido, por decisão do `OverrideSlot::Fill`.
                _ => main_solid(scene, piece).unwrap_or([0; 4]),
            };
            ph2d_panel_vector::state::InstancePiece {
                name: piece_name(sim, map, piece),
                colour,
                visible: inst.get(piece, HIDDEN_KIND) != Some(OverrideSlot::Hidden),
                overridden: own.is_some(),
            }
        })
        .collect();
    (rows, beyond)
}

/// A cor sólida de um caminho, se a tinta dele for uma.
fn main_solid(scene: &VecScene, piece: VecPathId) -> Option<[u8; 4]> {
    match scene
        .paths()
        .iter()
        .find(|p| p.id == piece)?
        .fill
        .as_ref()?
    {
        Paint::Solid(c) => Some([c.r, c.g, c.b, c.a]),
        _ => None,
    }
}

/// O nome que a Hierarquia mostra. Uma peça sem `Name` é apontada pelo id — feio, mas honesto:
/// inventar *"Piece 3"* faria duas peças trocarem de nome quando uma terceira nascesse no meio.
fn piece_name(sim: &SimWorld, map: &VecEntityMap, piece: VecPathId) -> String {
    map.get(&piece)
        .and_then(|&bits| sim.world().get::<Name>(Entity::from_bits(bits)))
        .map_or_else(|| format!("#{piece}"), |n| n.0.to_string())
}

/// **Este alvo de picker é a swatch de uma peça?** — a linha, se for.
///
/// Irmão do `fx_live::colour_target`: o picker é UM e partilhado, então quem o abriu é uma
/// pergunta ao ID do alvo, nunca um flag ao lado (que seria o segundo lugar do mesmo facto).
#[must_use]
pub(crate) fn colour_target(target: ph2d_editor::NodeId) -> Option<usize> {
    (0..ph2d_editor::ids::MAX_INSTANCE_PIECES)
        .find(|&r| ph2d_editor::ids::vector_instance_piece_colour_id(r) == target)
}

/// **O interruptor da linha `row`**: mostra/esconde a peça NESTA instância. `true` = mundo mudou.
pub(crate) fn toggle_piece_visible(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    e: Entity,
    row: usize,
) -> bool {
    let Some((mut inst, piece)) = piece_at(sim, scene, map, e, row) else {
        return false;
    };
    if inst.get(piece, HIDDEN_KIND) == Some(OverrideSlot::Hidden) {
        // Voltar a mostrar é REMOVER o override, e não gravar um *"visível"*: um segundo estado
        // para *"igual ao mestre"* faria a instância carregar diferenças que não existem, e o
        // `has_overrides` acenderia o *Reset* sobre uma cópia idêntica ao mestre.
        inst.overrides
            .retain(|o| !(o.sub == piece && o.slot.kind() == HIDDEN_KIND));
    } else {
        inst.set(piece, OverrideSlot::Hidden);
    }
    write_back(sim, e, inst)
}

/// **A cor da linha `row`** — o override de preenchimento desta peça nesta instância.
pub(crate) fn set_piece_colour(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    e: Entity,
    row: usize,
    rgba: [u8; 4],
) -> bool {
    let Some((mut inst, piece)) = piece_at(sim, scene, map, e, row) else {
        return false;
    };
    if inst.get(piece, FILL_KIND) == Some(OverrideSlot::Fill(rgba)) {
        return false; // o picker publica a mesma cor a cada frame: sem isto, um passo de undo por frame
    }
    inst.set(piece, OverrideSlot::Fill(rgba));
    write_back(sim, e, inst)
}

/// A instância de `e` e a peça da linha `row` — a resolução que os dois escritores partilham.
fn piece_at(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    e: Entity,
    row: usize,
) -> Option<(VecInstance, VecPathId)> {
    let inst = sim.world().get::<VecInstance>(e)?.clone();
    let main_e = Entity::from_bits(*map.get(&inst.main)?);
    let (pieces, _) = addressed_pieces(sim, scene, map, main_e);
    let piece = *pieces.get(row)?;
    Some((inst, piece))
}

fn write_back(sim: &mut SimWorld, e: Entity, inst: VecInstance) -> bool {
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.insert(inst);
        return true;
    }
    false
}

/// **Update Main** — as diferenças desta instância passam a ser o MESTRE.
///
/// Devolve `(absorvidos, recusados)`. As irmãs herdam por construção (o desenho delas é derivado);
/// uma irmã que tenha override PRÓPRIO no mesmo slot mantém-no, que é o que o Figma faz e a única
/// leitura coerente: um override é uma decisão da cópia, e absorver no mestre não a revoga.
///
/// ⚠️ **O `Hidden` não sobe, e a recusa é por ESPÉCIE.** Um mestre não tem *"peça escondida"* — a
/// única forma de a esconder lá seria apagá-la, e apagar arte não é o que *"atualizar o mestre"*
/// significa. Recusar no BOTÃO (escondê-lo quando há um `Hidden`) levaria junto as cores que ele
/// pode absorver.
pub(crate) fn update_main(sim: &mut SimWorld, scene: &mut VecScene, e: Entity) -> (usize, usize) {
    let Some(inst) = sim.world().get::<VecInstance>(e).cloned() else {
        return (0, 0);
    };
    let (mut taken, mut refused) = (0, 0);
    let mut keep: Vec<InstanceOverride> = Vec::new();
    for o in &inst.overrides {
        match o.slot {
            OverrideSlot::Fill(rgba) => {
                if let Some(p) = scene.path_mut(o.sub) {
                    p.fill = Some(Paint::Solid(Rgba8 {
                        r: rgba[0],
                        g: rgba[1],
                        b: rgba[2],
                        a: rgba[3],
                    }));
                    taken += 1;
                } else {
                    // A peça já não existe no mestre: o override é lixo, e mantê-lo faria o
                    // *Reset* continuar aceso sobre uma diferença que ninguém vê.
                    refused += 1;
                }
            }
            OverrideSlot::Hidden => {
                refused += 1;
                keep.push(*o);
            }
        }
    }
    if taken == 0 && keep.len() == inst.overrides.len() {
        return (0, refused); // nada mudou: um passo de undo vazio é ruído
    }
    let mut next = inst;
    next.overrides = keep;
    write_back(sim, e, next);
    (taken, refused)
}

/// **Swap** — a instância passa a derivar de `new_main`.
///
/// Devolve `(trocou, overrides_descartados)`. ⚠️ **A regra de compatibilidade é a única honesta
/// que este endereço permite:** um override aponta o `VecPathId` de uma peça **do mestre antigo**,
/// e esses ids não existem no novo — então os que não resolvem para uma peça do novo mestre são
/// descartados. Casar por NOME seria o endereço que a W5a recusou (duas respostas a *"que peça é
/// esta?"*), e mantê-los seria guardar diferenças que nada desenha.
pub(crate) fn swap_main(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    e: Entity,
    new_main: VecPathId,
) -> Option<(bool, usize)> {
    let inst = sim.world().get::<VecInstance>(e)?.clone();
    // Trocar por si mesma é o laço que o produtor recusa; trocar pelo mesmo mestre é um no-op.
    let at = map
        .iter()
        .find(|(_, b)| Entity::from_bits(**b) == e)
        .map(|(k, _)| *k)?;
    if new_main == at || new_main == inst.main {
        return Some((false, 0));
    }
    // O alvo tem de se declarar mestre — o mesmo contrato que o produtor exige para desenhar.
    let new_e = Entity::from_bits(*map.get(&new_main)?);
    sim.world().get::<ph2d_ecs::VecComponentMain>(new_e)?;
    let (pieces, _) = addressed_pieces(sim, scene, map, new_e);
    let before = inst.overrides.len();
    let mut next = VecInstance::new(new_main);
    for o in inst.overrides.iter().filter(|o| pieces.contains(&o.sub)) {
        next.set(o.sub, o.slot);
    }
    let dropped = before - next.overrides.len();
    write_back(sim, e, next);
    Some((true, dropped))
}

#[cfg(test)]
#[path = "vec_component_pieces_tests.rs"]
mod tests;
