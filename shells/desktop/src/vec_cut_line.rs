//! **A LINHA DE CORTE** — o caminho que a tesoura usa como lâmina (plano 25 §7).
//!
//! Espelho exato do `connector_live::upkeep` e do `blend_live::upkeep`: um caminho novo já está
//! na cena desde o press, mas a **entidade** dele só nasce no `vec_entities::sync`, mais adiante
//! no mesmo frame — então o componente que o marca como lâmina não pode ser pendurado no press.
//! A fila de um item (`App::vec_cut_pending`) é o que atravessa esse vão.
//!
//! # Uma lâmina de cada vez
//!
//! Desenhar uma linha nova **substitui** a anterior. A alternativa — recusar enquanto houver uma
//! — deixaria o modo Cut morto exatamente quando ele parece estar funcionando (o pill aceso, a
//! caneta a não desenhar nada), e o botão *Discard* existe para o outro caso: tirar a lâmina da
//! tela quando se acabou de cortar.
//!
//! # A lâmina não é obra
//!
//! Ela perde `fill` e `stroke` ao ser adotada, e quem a desenha é o **overlay** (hachurada, com
//! a tesoura na ponta). Duas razões, e as duas são de produto: uma lâmina que herdasse a cor e a
//! espessura do traço corrente seria indistinguível de um desenho, e uma lâmina com estilo sairia
//! no export — que é o mesmo que dizer que ela não é uma ferramenta.

use ph2d_ecs::{Entity, SimWorld, VecCutPath};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// Instala o `VecCutPath` na entidade da lâmina recém-desenhada e garante que ela é a ÚNICA.
///
/// Chamado **depois** do `sync` (a entidade tem de existir) e **antes** do `settle_origins`, pela
/// mesma razão dos irmãos: o `settle` decide a pose de um caminho novo, e queremos que a lâmina
/// já esteja identificada quando isso acontece.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    pending: &mut Option<VecPathId>,
) {
    if let Some(id) = pending.take() {
        if let Some(&bits) = map.get(&id) {
            // A lâmina anterior morre — uma de cada vez (ver o doc do módulo).
            for old in cut_lines(sim, map) {
                if old != id {
                    scene.remove_path(old);
                }
            }
            if let Ok(mut em) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
                em.insert(VecCutPath);
            }
        } else {
            // A entidade ainda não nasceu (o `sync` deste frame não a viu). Devolve à fila: o
            // caminho já está na cena, e desistir deixaria uma lâmina que se desenha como arte.
            *pending = Some(id);
        }
    }
    strip_style(scene, &cut_lines(sim, map));
}

/// **A lâmina não tem estilo, e a regra é re-afirmada a CADA frame** — não estampada uma vez.
///
/// Enio, 2026-07-31: *"o traço do CUT não pode receber um Fill"*. Estampar só na adoção não basta,
/// e há pelo menos **duas** portas que devolvem estilo a um caminho depois dela: a caneta, que
/// **preenche ao FECHAR** (`on_press`: fechar um caminho é o gesto que o torna uma forma, e uma
/// forma nasce com o `fill` corrente), e o recolorir da seleção. Nenhuma delas sabe o que é uma
/// lâmina, e ensiná-las seria espalhar a regra por tantos sítios quantas portas existirem —
/// exactamente a enumeração que apodrece quando nasce a terceira.
///
/// ⚠️ **Isto não polui o undo**: o passo é registado por DIFF no fim do frame, e a limpeza acontece
/// no mesmo frame em que o estilo foi escrito — o líquido é zero, e o Ctrl+Z não vê nada.
fn strip_style(scene: &mut VecScene, lines: &[VecPathId]) {
    for id in lines {
        if let Some(p) = scene.path_mut(*id)
            && (p.fill.is_some() || p.stroke.is_some())
        {
            p.fill = None;
            p.stroke = None;
        }
    }
}

/// Todos os caminhos marcados como lâmina. Devolve `Vec` (e não `Option`) de propósito: o
/// invariante "uma de cada vez" é ESTABELECIDO pelo [`upkeep`], não assumido por quem lê — se um
/// dia dois existirem, quem pergunta vê os dois em vez de escolher um em silêncio.
#[must_use]
pub(crate) fn cut_lines(sim: &SimWorld, map: &VecEntityMap) -> Vec<VecPathId> {
    map.iter()
        .filter(|(_, bits)| {
            sim.world()
                .get::<VecCutPath>(Entity::from_bits(**bits))
                .is_some()
        })
        .map(|(&id, _)| id)
        .collect()
}

/// A lâmina corrente, se houver.
#[must_use]
pub(crate) fn cut_line(sim: &SimWorld, map: &VecEntityMap) -> Option<VecPathId> {
    cut_lines(sim, map).first().copied()
}

/// **Descarta** a lâmina. Devolve `true` se havia uma (o chamador só abre passo de undo então).
pub(crate) fn discard(sim: &SimWorld, scene: &mut VecScene, map: &VecEntityMap) -> bool {
    let lines = cut_lines(sim, map);
    for id in &lines {
        scene.remove_path(*id);
    }
    // A entidade morre sozinha no `sync` do frame seguinte (a direção 2 dele: path que sumiu do
    // documento leva a entidade junto). Despawná-la aqui seria a segunda porta da mesma regra.
    !lines.is_empty()
}

/// **Executa o CORTE** com a lâmina desenhada. Devolve quantas formas foram cortadas.
///
/// # A fonte é ASSADA no mundo, e é isso que mata o deslocamento
///
/// As peças entram na cena em coordenadas de MUNDO, e a entidade de um caminho novo nasce com
/// `Transform` IDENTIDADE (`vec_entities::sync`). Guardar as peças em coordenadas LOCAIS da fonte
/// — o que o corte fazia antes desta wave — as fazia aparecer deslocadas por **exactamente o
/// `Transform` da fonte**: o defeito que o Enio fotografou em 2026-07-31 (*"a parte cortada se
/// desloca do lugar"*).
///
/// Assar é a regra da casa deste módulo, não uma invenção: a booleana, o merge e o offset já
/// assam os operandos em mundo (ADR-0111). A fonte é CONSUMIDA pelas peças, então o `Transform`
/// dela não é perdido — ele é absorvido pela geometria, e o `settle_origins` dá a cada peça um
/// pivô no centro dela própria.
///
/// # O escopo: TUDO o que a lâmina atravessa
///
/// A seleção **não estreita os alvos** (Enio, 2026-07-31: *"o corte deve acontecer em qualquer
/// forma sobreposta e não apenas na forma selecionada"*) — e não é só preferência, é coerência:
/// a seleção que este gesto exige é a da **própria lâmina**, então usá-la também para escolher
/// alvos daria dois significados ao mesmo estado, e o artista não teria como pedir um deles.
///
/// A lâmina **nunca** é alvo de si mesma, e quem a exclui é o marcador — não uma lista de ids que
/// alguém teria de manter.
///
/// # As duas condições para cortar
///
/// `armed` (o pill **Cut** está escolhido) **e** a lâmina está na seleção. As duas juntas são o
/// que impede um corte acidental: a linha fica na cena depois de usada, e sem elas qualquer
/// clique no botão — ou um atalho futuro — cortaria a cena outra vez com uma lâmina que o artista
/// esqueceu lá.
pub(crate) fn apply_cut(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    armed: bool,
) -> usize {
    if !armed {
        return 0;
    }
    let Some(line_id) = cut_line(sim, map) else {
        return 0;
    };
    if !selected.contains(&line_id) {
        return 0;
    }
    let xforms = crate::vec_transform::build(sim, map);
    let Some(line) = world_copy(scene, &xforms, line_id) else {
        return 0;
    };
    // Escondida ou TRAVADA não é cortada: as duas dizem *"não mexa nisto agora"*, e uma faca que
    // as ignorasse seria a única ferramenta do editor a fazê-lo.
    let view = crate::vec_entities::view_state(sim, map);

    let targets: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| *id != line_id)
        .filter(|id| view.is_pickable(*id))
        .collect();

    let mut cut = 0usize;
    for id in targets {
        let Some(source) = world_copy(scene, &xforms, id) else {
            continue;
        };
        // Uma fonte ABERTA não é assunto do corte fechado — ela fica intacta por ora, e a
        // recusa é do motor, não desta função (uma segunda decisão aqui divergiria dele).
        let Ok(pieces) = ph2d_vec_boolean::cut_closed(&source, &line) else {
            continue;
        };
        let Some(z) = scene.paths().iter().position(|p| p.id == id) else {
            continue;
        };
        scene.remove_path(id);
        // As peças entram NO LUGAR da fonte, na ordem em que saíram: nenhuma salta para a frente
        // da cena, e as duas metades ficam vizinhas na Hierarquia.
        for (k, piece) in pieces.into_iter().enumerate() {
            scene.insert_path(z + k, piece);
        }
        cut += 1;
    }
    cut
}

/// Uma cópia do caminho `id` em coordenadas de MUNDO (a geometria autorada, com o afim assado).
fn world_copy(
    scene: &VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    id: VecPathId,
) -> Option<ph2d_vec_scene::VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    ph2d_vec_scene::bake_xform(&mut p, &ph2d_vec_scene::xform_of(xforms, id));
    Some(p)
}

#[cfg(test)]
#[path = "vec_cut_line_tests.rs"]
mod tests;
