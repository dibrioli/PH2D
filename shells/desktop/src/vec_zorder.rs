//! **A ordem de z do vetor** — a projeção da árvore, e quem a reescreve.
//!
//! Módulo irmão do [`crate::vec_entities`] (teto de 600 LOC da shell). Os dois vivem juntos por
//! assunto: a ponte doc↔árvore é quem sabe **onde cada forma está na pilha**, porque a pilha é uma
//! **projeção da árvore** (ADR-0110) e não uma propriedade do documento.
//!
//! É a regra que este arquivo inteiro serve: **quem quiser mandar no z escreve no ESTADO
//! AUTORADO** — nunca na ordem do vetor da cena, que a projeção reescreve a cada frame.
//!
//! # A LEI (Enio, 2026-08-04, segunda rodada)
//!
//! > *"O objeto não deve ser movido na hierarquia, apenas o Z muda, e o Z determina na frente de
//! > quem ele é mostrado. A ordem na hierarquia só define a ordem se o Z for igual."*
//!
//! Logo a pilha tem **duas chaves** — `(Z efetivo, índice do DFS)` — e só a **primeira** é
//! autorável por esta porta. Os quatro botões Arrange **escrevem o Z e mais nada**: a árvore é
//! do artista, e só a Hierarquia a move.
//!
//! ⚠️ **A versão anterior mexia na árvore** (`RootOrder` das raízes, sequência do `Children` dos
//! filhos) e caía no Z como plano B. Isso dava aos botões **duas** maneiras de mudar a mesma
//! pilha, e a que o artista não pediu era a destrutiva: pressionar *Forward* re-arrumava a
//! Hierarquia dele por baixo. `siblings`/`sibling_move`/`write_sibling_order` morreram com ela.

use super::VecEntityMap;
use ph2d_ecs::scene::{HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld, Without, ZIndexOverride};
use ph2d_vec_scene::VecPathId;

/// Reempilha um GRUPO de paths numa sequência contígua de z (`run` vem **fundo → topo**),
/// mantendo a ordem relativa de todo o resto.
///
/// # Por que isto existe
///
/// A ordem de z é a projeção da ÁRVORE (ADR-0110) filtrada pelo Z: quem quiser mandar nela tem de
/// escrever no estado autorado, não na ordem do vetor da cena — essa é reescrita a cada frame pela
/// projeção. (É a mesma armadilha do "duas portas para a mesma pergunta": `VecScene::reorder_path`
/// mexe na porta ERRADA e o frame seguinte desfaz. Os botões Arrange chamavam-na — e por isso
/// estavam MORTOS até 2026-08-04; hoje passam pelo [`reorder`], que escreve o Z.)
///
/// ⚠️ **Esta função é a exceção que escreve `RootOrder`, e continua certa:** ela não move uma
/// forma que o artista pôs, ela **coloca** um grupo recém-criado — o Blend precisa que os passos
/// nasçam entre as fontes, e nascer é escolher um lugar na árvore, não mudar de lugar nela.
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

/// **A pilha que o artista VÊ, com as duas chaves à vista** — fundo → topo.
///
/// Reconstruir o snapshot aqui é caro por frame e irrelevante num gesto: os botões Arrange são
/// user-paced. O que não seria irrelevante é derivá-la por outra via — os botões passariam a mover
/// a forma numa pilha que não é a desenhada.
fn final_view(sim: &mut SimWorld) -> (HierarchySnapshot, Vec<StackRow>) {
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    let rows = keyed_stack(sim.world(), &snap);
    (snap, rows)
}

/// **Quantos caminhos vetoriais a sub-árvore de `id` contém, além dele próprio.**
///
/// ⚠️ Existe porque **um descendente é incruzável**: o Z é uma CASCATA (Godot), então subir o meu
/// número sobe o dos meus filhos pelo mesmo tanto e a distância entre nós não muda. Mirar num
/// deles seria um botão que escreve um número, devolve `true`, e não move um pixel — um passo de
/// undo gasto por nada.
///
/// A sub-árvore é contígua no DFS e a filtragem por `vec_path` preserva a contiguidade, então os
/// descendentes de `id` são exactamente os `k` índices seguintes ao dele.
fn descendant_count(snap: &HierarchySnapshot, id: VecPathId) -> u32 {
    let mut it = snap.entries.iter();
    let Some(depth) = it.find(|e| e.vec_path == Some(id)).map(|e| e.depth) else {
        return 0;
    };
    u32::try_from(
        it.take_while(|e| e.depth > depth)
            .filter(|e| e.vec_path.is_some())
            .count(),
    )
    .unwrap_or(u32::MAX)
}

/// **Move `id` na pilha que se VÊ, escrevendo o Z — e SÓ o Z.** `true` = o número mudou.
///
/// # Os quatro verbos são UMA regra com uma REFERÊNCIA diferente
///
/// A pilha é totalmente ordenada por `(Z efetivo, índice do DFS)` e o DFS é intocável (a árvore é
/// do artista), então mover-se resume-se a *"que número me põe do outro lado daquela forma?"* — e
/// os verbos só discordam sobre **qual** forma é essa:
///
/// | verbo | referência |
/// |---|---|
/// | Forward | o vizinho imediatamente à frente |
/// | To Front | o da FRENTE de tudo |
/// | Backward | o vizinho imediatamente atrás |
/// | To Back | o do FUNDO de tudo |
///
/// Um `match` que desse a cada verbo a sua própria aritmética teria quatro sítios onde o sinal
/// pode estar trocado; aqui o sinal é escrito uma vez.
///
/// ⚠️ **O passo é o MENOR que entrega:** empatar já basta quando a árvore me favorece (o desempate
/// é dela), senão é preciso um degrau. Empatar de propósito não é fraqueza — é o que impede o Z de
/// inflar um número por clique e o que faz do *Backward* o inverso exacto do *Forward*.
///
/// ⚠️ **E há um limite que é ARITMÉTICA, não desleixo:** com três formas empatadas em `z = 0` não
/// existe inteiro que ponha a do fundo entre as outras duas — `0` deixa-a atrás das duas e `1`
/// põe-na à frente das duas. Nesse regime o *Forward* passa mais de um lugar. A alternativa seria
/// renumerar os vizinhos (mexer no número de um objeto que o artista não selecionou) ou mexer na
/// árvore, que é precisamente o que esta lei proíbe.
pub(crate) fn reorder(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    order: ph2d_vec_scene::ZOrder,
) -> bool {
    use ph2d_vec_scene::ZOrder;
    let (snap, rows) = final_view(sim);
    let Some(i) = rows.iter().position(|r| r.id == id) else {
        return false;
    };
    let me = rows[i];
    // Os meus DESCENDENTES saem da lista de candidatos: eles viajam comigo pela cascata, logo
    // nenhum número os cruza (ver `descendant_count`).
    let kin = descendant_count(&snap, id);
    let mine = |r: &StackRow| r.dfs > me.dfs && r.dfs <= me.dfs + kin;
    let ahead = || rows[i + 1..].iter().filter(|r| !mine(r));
    let behind = || rows[..i].iter().filter(|r| !mine(r));
    let goal = match order {
        ZOrder::Raise => ahead().next(),
        ZOrder::ToFront => ahead().next_back(),
        ZOrder::Lower => behind().next_back(),
        ZOrder::ToBack => behind().next(),
    };
    // Não há ninguém cruzável na direção pedida. ⚠️ A recusa é o que impede um passo de undo
    // vazio: o undo global regista por DIFF, e o artista gastaria um Ctrl+Z sem ver nada
    // acontecer. Ela subsume o "já está no extremo" — e cobre também o caso em que tudo o que
    // está à frente é a minha própria sub-árvore.
    let Some(goal) = goal.copied() else {
        return false;
    };
    let forward = matches!(order, ZOrder::ToFront | ZOrder::Raise);
    let target = if forward {
        if me.dfs > goal.dfs {
            goal.z
        } else {
            goal.z.saturating_add(1)
        }
    } else if me.dfs < goal.dfs {
        goal.z
    } else {
        goal.z.saturating_sub(1)
    };
    // O componente guarda o AUTORADO; somar o delta a ele soma o mesmo delta ao efetivo (a
    // cascata dos pais é um termo comum).
    let own = authored_z(sim, map, id).unwrap_or(0);
    set_authored_z(
        sim,
        map,
        id,
        own.saturating_add(target.saturating_sub(me.z)),
    )
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
    keyed_stack(world, snap).into_iter().map(|r| r.id).collect()
}

/// Uma linha da pilha: **as duas chaves e o caminho**.
///
/// O `dfs` é a posição da forma na varredura da Hierarquia — 0 é a primeira linha da lista. Ele é
/// **intocável por esta porta**: quem o move é o artista, arrastando na Hierarquia.
#[derive(Copy, Clone)]
struct StackRow {
    z: i32,
    dfs: u32,
    id: VecPathId,
}

/// **A pilha com as duas chaves à vista**, fundo → topo.
///
/// ⚠️ É a **porta única** da pergunta *"quem está na frente de quem?"*: a projeção do frame
/// consome-a (via [`z_order`]) e os botões Arrange raciocinam sobre ela. Uma segunda derivação
/// faria os botões moverem a forma numa pilha que não é a desenhada — e o artista clicaria em
/// *Forward* para ver a forma passar outra coisa que não a que está à frente dela.
///
/// A ordenação é pelo **par** (e não por `sort_by_key` estável sobre o Z): o resultado é o mesmo,
/// mas a chave que os botões usam para calcular passa a estar escrita onde a ordem é decidida.
fn keyed_stack(world: &bevy_ecs::world::World, snap: &HierarchySnapshot) -> Vec<StackRow> {
    let mut keyed: Vec<StackRow> = snap
        .entries
        .iter()
        .filter_map(|e| e.vec_path.map(|p| (e.entity, p)))
        .enumerate()
        .map(|(dfs, (bits, id))| StackRow {
            z: ph2d_ecs::effective_z_index(world, Entity::from_bits(bits)),
            dfs: u32::try_from(dfs).unwrap_or(u32::MAX),
            id,
        })
        .collect();
    keyed.sort_by_key(|r| (r.z, r.dfs));
    keyed
}
