//! **A ordem de z do vetor** — a projeção da árvore, e quem a reescreve.
//!
//! Módulo irmão do [`crate::vec_entities`] (teto de 600 LOC da shell). Os dois vivem juntos por
//! assunto: a ponte doc↔árvore é quem sabe **onde cada forma está na pilha**, porque a pilha é uma
//! **projeção da árvore** (ADR-0110) e não uma propriedade do documento.
//!
//! É a regra que este arquivo inteiro serve: **quem quiser mandar no z escreve na ÁRVORE.** A
//! ordem do vetor da cena é reescrita a cada frame pela projeção — mexer nela é falar com a porta
//! errada, e o frame seguinte desfaz.
//!
//! ⚠️ *"A árvore"* são **dois** lugares, e um deles não é o `RootOrder`: a ordem das RAÍZES mora
//! nele, a dos FILHOS mora na sequência do `Children`. Uma nota anterior deste cabeçalho dizia só
//! *"escreva no `RootOrder`"* e estava certa pela metade — um filho não tem `RootOrder`, e
//! escrever um nele não move nada.

use super::VecEntityMap;
use ph2d_ecs::scene::{HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld, Without, ZIndexOverride};
use ph2d_vec_scene::VecPathId;

/// Reempilha um GRUPO de paths numa sequência contígua de z (`run` vem **fundo → topo**),
/// mantendo a ordem relativa de todo o resto.
///
/// # Por que isto existe
///
/// A ordem de z é a projeção da ÁRVORE (ADR-0110): quem quiser mandar no z tem de escrever no
/// `RootOrder`, não na ordem do vetor da cena — essa é reescrita a cada frame pela projeção.
/// (É a mesma armadilha do "duas portas para a mesma pergunta": `VecScene::reorder_path` mexe na
/// porta ERRADA e o frame seguinte desfaz. Os botões Arrange chamavam-na — e por isso estavam
/// MORTOS até 2026-08-04; hoje passam pelo [`reorder`].)
///
/// O Blend precisa disto: os passos que ele cria só ganham entidade no `sync` do frame seguinte,
/// e ele quer a sequência inteira (fontes inclusas) empilhada na ordem certa.
pub(crate) fn restack(sim: &mut SimWorld, map: &VecEntityMap, run: &[VecPathId]) {
    let members: Vec<Entity> = run
        .iter()
        .filter_map(|id| map.get(id).copied())
        .map(Entity::from_bits)
        .filter(|e| sim.world().get_entity(*e).is_ok())
        .filter(|e| sim.world().get::<ChildOf>(*e).is_none()) // só raízes: um filho vive no pai
        .collect();
    if members.len() < 2 {
        return;
    }
    // A pilha de z de TODAS as raízes, fundo → topo — que desde a lei de Godot (2026-08-04) é a
    // ordem CRESCENTE de `RootOrder`: a primeira linha da Hierarquia é a de TRÁS.
    let mut stack: Vec<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &RootOrder), Without<ChildOf>>();
        let mut roots: Vec<(Entity, u32)> = q.iter(sim.world()).map(|(e, r)| (e, r.0)).collect();
        roots.sort_by_key(|(e, o)| (*o, e.to_bits()));
        roots.into_iter().map(|(e, _)| e).collect()
    };
    // O grupo entra na fatia de z da mais de TRÁS dele — o resultado não salta para o topo do
    // documento, que é o que o Illustrator faz com um blend.
    let anchor = stack
        .iter()
        .take_while(|e| !members.contains(e))
        .filter(|e| !members.contains(e))
        .count();
    stack.retain(|e| !members.contains(e));
    let at = anchor.min(stack.len());
    for (k, e) in members.iter().enumerate() {
        stack.insert(at + k, *e);
    }
    for (i, e) in stack.iter().enumerate() {
        // fundo (i=0) = MENOR `RootOrder` — a lista da Hierarquia é fundo → topo.
        if let Ok(mut em) = sim.world_mut().get_entity_mut(*e) {
            em.insert(RootOrder(u32::try_from(i).unwrap_or(u32::MAX)));
        }
    }
}

/// **Os IRMÃOS de `e` na ordem em que a Hierarquia os lista** — e desde a lei de Godot
/// (2026-08-04) o **último** é o da FRENTE.
///
/// Uma raiz tem por irmãos as outras raízes (ordenadas por `RootOrder`, com os bits a desempatar —
/// o mesmo critério do `build_hierarchy_snapshot`, e é **por ser o mesmo** que o número que o
/// painel mostra é o lugar em que a forma de facto está). Um filho tem os irmãos do `Children` do
/// pai, na ordem de inserção.
fn siblings(sim: &mut SimWorld, e: Entity) -> Vec<Entity> {
    if let Some(c) = sim.world().get::<ChildOf>(e) {
        return sim
            .world()
            .get::<bevy_ecs::hierarchy::Children>(c.parent())
            .map(|k| k.iter().copied().collect())
            .unwrap_or_default();
    }
    let mut q = sim
        .world_mut()
        .query_filtered::<(Entity, Option<&RootOrder>), Without<ChildOf>>();
    let mut roots: Vec<(Entity, u32)> = q
        .iter(sim.world())
        .map(|(e, r)| (e, r.map_or(u32::MAX, |r| r.0)))
        .collect();
    roots.sort_by_key(|(e, o)| (*o, e.to_bits()));
    roots.into_iter().map(|(e, _)| e).collect()
}

/// **O Z-INDEX AUTORADO desta forma** — o número que o artista escreve, `None` se ela não tem um.
///
/// ⚠️ **É o `ph2d_ecs::ZIndexOverride`, o MESMO componente dos sprites** (Enio, 2026-08-04: *"o Z
/// index deve ser global e sobrepõe a ordem na hierarquia"*). Um componente próprio do vetor seria
/// uma segunda resposta a *"quem está na frente?"* — e como um caminho pode ser filho de um sprite
/// (ADR-0110), as duas respostas conviveriam na MESMA árvore.
///
/// ⚠️ **Autorado, não efetivo.** O campo do painel edita o número desta forma; o que ORDENA é o
/// efetivo (`ph2d_ecs::effective_z_index`, que soma a cascata dos pais como no Godot). Mostrar o
/// efetivo num campo editável faria o artista escrever `5` e ler `8`.
#[must_use]
pub(crate) fn authored_z(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<i32> {
    let e = Entity::from_bits(*map.get(&id)?);
    Some(sim.world().get::<ZIndexOverride>(e).map_or(0, |z| z.0))
}

/// Escreve o Z autorado de `id`. **Zero DESTACA o componente** — a mesma política de todo override
/// deste repo: um arquivo não guarda o neutro.
pub(crate) fn set_authored_z(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    z: i32,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let z = ZIndexOverride::clamped(z);
    if authored_z(sim, map, id) == Some(z) {
        return false; // um passo de undo vazio é ruído
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    if z == 0 {
        em.remove::<ZIndexOverride>();
    } else {
        em.insert(ZIndexOverride(z));
    }
    true
}

/// **A pilha que o artista VÊ**, fundo → topo — a mesma que o desenho consome.
///
/// Reconstruir o snapshot aqui é caro por frame e irrelevante num gesto: os botões Arrange são
/// user-paced. O que não seria irrelevante é derivá-la por outra via — os botões passariam a mover
/// a forma numa pilha que não é a desenhada.
fn final_stack(sim: &mut SimWorld) -> Vec<VecPathId> {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    z_order(sim.world(), &snap)
}

/// **Move `id` um lugar na pilha que se VÊ.** `true` = alguma coisa mudou.
///
/// # A regra é UMA, e ela é MEDIDA em vez de enumerada
///
/// > **Tenta a ÁRVORE; se a forma não se mexeu na pilha, escreve o Z.**
///
/// A pilha tem duas chaves — o Z efetivo manda, a árvore desempata — e um `match` que decidisse
/// *"este caso quer árvore, aquele quer Z"* teria de enumerar tie/não-tie, irmão/não-irmão,
/// raiz/filho. Em vez disso o gesto **faz** o movimento de árvore e **olha** o resultado na mesma
/// pilha que o renderer desenha; se ela não andou, quem manda é o Z e é ele que se escreve.
///
/// ⚠️ **O movimento de árvore que não pegou FICA.** Ele não é lixo: a forma passou de facto para a
/// frente dos irmãos, e é isso que o artista quer no dia em que o Z voltar a empatar. Desfazê-lo
/// custaria uma segunda escrita para não deixar rasto de nada.
///
/// ⚠️ **"Um lugar" só existe entre IRMÃOS.** Duas formas podem ser vizinhas na pilha e viver em
/// pais diferentes; movê-la "um lugar" ali seria reparentar. Por isso o passo de Z é *estritamente
/// além do balde do vizinho* (`+1`), e não uma tentativa de empatar com ele — empatar devolveria a
/// decisão ao desempate de árvore, que é justamente quem não conseguiu.
pub(crate) fn reorder(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    order: ph2d_vec_scene::ZOrder,
) -> bool {
    use ph2d_vec_scene::ZOrder;
    let forward = matches!(order, ZOrder::ToFront | ZOrder::Raise);
    let before = final_stack(sim).iter().position(|x| *x == id);
    // Já está no extremo que o botão pede: a recusa é aqui, e vale para as DUAS metades.
    if let Some(i) = before {
        let n = final_stack(sim).len();
        if (forward && i + 1 >= n) || (!forward && i == 0) {
            return false;
        }
    }
    let tree = sibling_move(sim, map, id, order);
    let stack = final_stack(sim);
    let after = stack.iter().position(|x| *x == id);
    // ⚠️ **O que "entregou" significa depende do VERBO**, e colapsar os dois foi um defeito real:
    // com um `a > b` para todos, o *To Front* dava-se por satisfeito ao andar UM lugar e a forma
    // parava no meio da pilha — o botão diz *frente*, não *um passo*.
    if let (Some(b), Some(a)) = (before, after) {
        let done = match order {
            ZOrder::ToFront => a + 1 == stack.len(),
            ZOrder::ToBack => a == 0,
            ZOrder::Raise => a > b,
            ZOrder::Lower => a < b,
        };
        if done {
            return true;
        }
    }
    bump_z(sim, map, id, order) || tree
}

/// A metade de ÁRVORE: reordena `id` entre os irmãos dele. `true` = a árvore mudou.
fn sibling_move(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    order: ph2d_vec_scene::ZOrder,
) -> bool {
    use ph2d_vec_scene::ZOrder;
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let mut sibs = siblings(sim, e);
    let Some(i) = sibs.iter().position(|x| *x == e) else {
        return false;
    };
    if sibs.len() < 2 {
        return false; // filho único: não há pilha em que andar
    }
    // A lista é FUNDO → frente, então "à frente" é para o ÍNDICE MAIOR.
    let to = match order {
        ZOrder::ToFront => sibs.len() - 1,
        ZOrder::Raise => (i + 1).min(sibs.len() - 1),
        ZOrder::Lower => i.saturating_sub(1),
        ZOrder::ToBack => 0,
    };
    if to == i {
        return false;
    }
    let moved = sibs.remove(i);
    sibs.insert(to, moved);
    write_sibling_order(sim, e, &sibs);
    true
}

/// A metade do **Z**: leva `id` estritamente além do balde do vizinho na direção pedida.
fn bump_z(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    order: ph2d_vec_scene::ZOrder,
) -> bool {
    use ph2d_vec_scene::ZOrder;
    let stack = final_stack(sim);
    let Some(i) = stack.iter().position(|x| *x == id) else {
        return false;
    };
    let eff = |sim: &SimWorld, p: VecPathId| -> i32 {
        map.get(&p).map_or(0, |&b| {
            ph2d_ecs::effective_z_index(sim.world(), Entity::from_bits(b))
        })
    };
    let mine = eff(sim, id);
    let target = match order {
        ZOrder::Raise => match stack.get(i + 1) {
            Some(&n) => eff(sim, n) + 1,
            None => return false,
        },
        ZOrder::Lower => match i.checked_sub(1).and_then(|k| stack.get(k)) {
            Some(&n) => eff(sim, n) - 1,
            None => return false,
        },
        ZOrder::ToFront => stack.iter().map(|p| eff(sim, *p)).max().unwrap_or(mine) + 1,
        ZOrder::ToBack => stack.iter().map(|p| eff(sim, *p)).min().unwrap_or(mine) - 1,
    };
    let delta = target - mine;
    if delta == 0 {
        return false;
    }
    // O componente guarda o AUTORADO; somar o delta a ele soma o mesmo delta ao efetivo (a
    // cascata dos pais é um termo comum).
    let own = authored_z(sim, map, id).unwrap_or(0);
    set_authored_z(sim, map, id, own.saturating_add(delta))
}

/// Grava a ordem `sibs` (FUNDO → frente) de volta na árvore.
///
/// ⚠️ **Duas escritas diferentes para dois lugares diferentes**, e não é acidente: a ordem das
/// RAÍZES mora no `RootOrder` (é o que o snapshot ordena) e a dos FILHOS mora na sequência do
/// `Children` — que o `ph2d_ecs::reinsert_children_in_order` reescreve. ⚠️ **Ela é a porta única
/// dessa segunda metade**, partilhada com o drop da Hierarquia e com o arrasto dentro de um fluxo;
/// a primeira versão desta função reimplementou o truque do re-inserir `ChildOf` à mão, que é
/// exactamente o detalhe de motor que a porta existe para esconder.
fn write_sibling_order(sim: &mut SimWorld, e: Entity, sibs: &[Entity]) {
    if let Some(parent) = sim.world().get::<ChildOf>(e).map(ChildOf::parent) {
        ph2d_ecs::reinsert_children_in_order(sim.world_mut(), parent, sibs);
        return;
    }
    for (i, &root) in sibs.iter().enumerate() {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(root) {
            em.insert(RootOrder(u32::try_from(i).unwrap_or(u32::MAX)));
        }
    }
}

/// Os gates do ponto fixo (o conserto do "undo só faz uma etapa") — módulo irmão,
/// pelo teto de 600 LOC por arquivo da shell (HR-18).
#[cfg(test)]
#[path = "vec_zorder_fixpoint_tests.rs"]
mod zorder_fixpoint_tests;

/// Os gates do Z-index e dos botões Arrange — irmão pelo mesmo teto.
#[cfg(test)]
#[path = "vec_zorder_arrange_tests.rs"]
mod arrange_tests;

/// A ordem de z que a árvore dita: **fundo → topo**, pronta para
/// `VecScene::reorder_to`.
///
/// # A LEI (Enio, 2026-08-04): a ordem do DFS **É** a ordem de desenho
///
/// *"Em Godot os objetos mais abaixo na hierarquia aparecem na frente"* — e o DFS lista o pai
/// antes dos filhos, logo **o filho desenha sobre o pai** de graça, sem ninguém antecipar nada.
///
/// ⚠️ **A versão anterior INVERTIA** (`entries … .rev()`, a convenção Illustrator/Figma de *"a
/// primeira linha é a da frente"*), e pagava por isso em três lugares: o renderer tinha de
/// **antecipar** o desenho de todo pai para não cobrir os filhos, o apontar tinha de **demover**
/// cada pai antecipado, e a INSTÂNCIA — que percorre a cena e não tem renderer por trás — não
/// tinha como fazer nem uma coisa nem outra (*"ao criar a instância, os filhos que no mestre
/// aparecem na frente dos pais vão para trás dos pais"*). Uma lei imposta por dois remendos que o
/// terceiro consumidor não conhecia. Invertida a projeção, os três somem.
///
/// # E o **Z** manda sobre ela
///
/// *"A ordem só conta se o Z dos objetos for igual. O Z index deve ser global e sobrepõe a ordem
/// na hierarquia."* — a semântica exacta do `CanvasItem.z_index`: ordena-se pelo Z EFETIVO, e o
/// DFS é o **desempate**. O `sort_by_key` é estável, então quem não tem Z fica exactamente onde a
/// árvore o pôs — e um documento sem um único `ZIndexOverride` produz a MESMA lista que o DFS cru.
///
/// ⚠️ **Consequência honesta: o Z tira uma forma da sub-árvore em que ela vive.** Um filho com Z
/// alto sai de dentro do intervalo da moldura que o contém e **deixa de ser recortado** — é o
/// significado literal de *"sobrepõe a ordem na hierarquia"*, e é por isso que o intervalo de
/// recorte se resolve contra ESTA lista e não contra o DFS (ver `vec_frame_spans`).
///
/// **A fonte é o snapshot da ÁRVORE, não a lista do painel** — e isso não é
/// arrumação, é o conserto de BUGS #15. O painel publica a lista dele no prólogo do
/// frame, **antes** de o [`sync`] dar entidade à forma recém-criada; projetar por
/// ela deixa a forma nova de fora, e quem o `reorder_to` não conhece recebe chave 0 e
/// vai pro **FUNDO**. A cena só convergia um frame depois — e como o snapshot do undo
/// é tirado no fim do frame da AÇÃO, ele capturava um estado que **não é ponto fixo
/// dos sistemas**: restaurá-lo e deixar o frame rodar produzia outra coisa, o diff
/// por-frame lia a diferença como ação do usuário, e nascia um passo espúrio que
/// limpava o redo. Era o "o undo só faz uma etapa e não funciona mais".
///
/// O snapshot vem de `build_hierarchy_snapshot` — a **mesma** função que alimenta o
/// painel, chamada num momento diferente (depois do `sync`). Um DFS próprio aqui
/// seria uma segunda porta para a mesma pergunta, e duas portas divergem.
#[must_use]
pub(crate) fn z_order(world: &bevy_ecs::world::World, snap: &HierarchySnapshot) -> Vec<VecPathId> {
    let mut keyed: Vec<(i32, VecPathId)> = snap
        .entries
        .iter()
        .filter_map(|e| {
            e.vec_path.map(|p| {
                (
                    ph2d_ecs::effective_z_index(world, Entity::from_bits(e.entity)),
                    p,
                )
            })
        })
        .collect();
    // ESTÁVEL: o empate mantém a ordem do DFS, que é o desempate que a lei manda usar.
    keyed.sort_by_key(|(z, _)| *z);
    keyed.into_iter().map(|(_, p)| p).collect()
}
