//! ⭐⭐ **O CONJUNTO DE ESTADOS — um botão, e as formas viram uma máquina** (plano 32 W8).
//!
//! Enio, 2026-08-25:
//!
//! > *"o usuário seleciona todas as peças que estarão envolvidas na máquina de estados do morph.
//! > Com o clique de um único botão um objeto novo surge na hierarquia tendo como filhos as shapes
//! > escolhidas. Todas as setas são atribuídas automaticamente cobrindo todas as morphs possíveis
//! > entre todas as formas (tanto de ida como de volta). As setas são virtuais e ninguém jamais vê.
//! > No canvas uma única shape aparece (a shape do estado atual) e as demais ficam ocultas."*
//!
//! # As quatro coisas que o clique faz, e por que são um só passo
//!
//! 1. nasce o objecto (um `VecPath` vazio + [`VecMorph`], que é o que a cena **desenha**);
//! 2. as formas escolhidas viram **filhos** dele (`ChildOf`), na ordem de z;
//! 3. cada uma ganha `Visibility::hidden()` — *no canvas aparece uma forma só*;
//! 4. o [`VecMorphMachine`] recebe o **grafo completo dirigido** sobre elas.
//!
//! ⚠️ **Um passo de undo, não quatro.** As quatro escritas acontecem no mesmo quadro e o
//! `post_frame_undo` regista por DIFF — um Ctrl+Z desfaz o conjunto inteiro, que é o que o gesto
//! promete. Reparentar num quadro e esconder no seguinte daria dois passos, e o primeiro deixaria
//! o artista com nove formas empilhadas.
//!
//! # ⛔ O que esta wave APAGOU, e não se reconstrói sem ler isto
//!
//! A W3a desenhava as setas no canvas (âmbar, entre as formas) e a W3b tinha um **modo** de arrasto
//! forma→forma que criava uma aresta. As duas morreram aqui, e não por gosto:
//!
//! - *"as setas são virtuais e ninguém jamais vê"* mata o desenho por decisão directa;
//! - o grafo passou a ser **completo por construção**, então o arrasto criaria uma aresta **que já
//!   existe**. Um gesto cujo produto já está lá é um gesto que não faz nada — e o modo dele seria
//!   um pill na fileira a competir com treze irmãos por uma resposta que ninguém pode ver.
//!
//! ⚠️ *Duas portas para a mesma pergunta divergem em silêncio*: com o botão a gerar `n(n-1)` e o
//! arrasto a acrescentar à mão, a lista deixaria de ser derivável e a próxima derivação apagaria
//! o trabalho do arrasto.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecMorph, VecMorphMachine, Visibility};
use ph2d_morph_machine::{MorphGraph, MorphState};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// **O conjunto à espera da entidade dele nascer** — o `sync` do quadro seguinte é que cria a
/// entidade do path novo, e só aí há onde pendurar os componentes.
///
/// ⚠️ **Espelho do `vec_morph_pending`**, e um slot PRÓPRIO porque o payload é outro: aquele leva
/// um componente, este leva a máquina **e** a lista de quem vai ser reparentado.
#[derive(Clone, Debug)]
pub(crate) struct MorphSetPending {
    /// O path do objecto novo (a forma morfada, ainda vazia).
    pub(crate) path: VecPathId,
    /// O nome que a Hierarquia mostra.
    pub(crate) name: String,
    /// As formas-membro, na ordem de z — **a primeira é o estado inicial**.
    pub(crate) members: Vec<VecPathId>,
}

/// **AS FORMAS DA SELEÇÃO QUE PODEM VIRAR ESTADOS.**
///
/// ⚠️ **Um estado é uma forma DESENHADA**, então o que entra é o que a cena sabe desenhar e o mapa
/// conhece.
///
/// ⛔ **Duas exclusões, e cada uma fecha uma porta diferente para o mesmo defeito** — *um conjunto
/// por cima de outro*:
///
/// 1. **uma entidade que já É um Morph** — a máquina de fora teria estados cuja geometria a de
///    dentro re-escreve a cada quadro, por baixo dela;
/// 2. **uma forma que já é MEMBRO de um conjunto** (o pai tem `VecMorphMachine`) — ela está oculta
///    e pertence a outra máquina; um segundo conjunto sobre ela dava dois donos à mesma forma, e o
///    artista alcança-a facilmente pela Hierarquia.
///
/// ⚠️ **A ordem é a da SELEÇÃO**, que é a de z — e ela é load-bearing: o primeiro membro é o
/// `start` do grafo, e é a forma que o artista vê quando o conjunto nasce.
#[must_use]
pub(crate) fn eligible(sim: &SimWorld, map: &VecEntityMap, sel: &[VecPathId]) -> Vec<VecPathId> {
    let mut out: Vec<VecPathId> = Vec::new();
    for id in sel {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let w = sim.world();
        if w.get_entity(e).is_err() || w.get::<VecMorph>(e).is_some() {
            continue;
        }
        let owned = w
            .get::<ChildOf>(e)
            .is_some_and(|c| w.get::<VecMorphMachine>(c.parent()).is_some());
        if owned {
            continue;
        }
        if !out.contains(id) {
            out.push(*id);
        }
    }
    out
}

/// ⭐⭐ **A LISTA DE ESTADOS** sobre `shapes` — uma entrada por forma, **sem tecla nenhuma**.
///
/// # ⛔⛔ Isto era o GRAFO COMPLETO, e a W10 apagou-o
///
/// Até 2026-08-25 esta função devolvia `n(n-1)` arestas, uma por par ordenado — *"todas as morphs
/// possíveis, de ida e de volta"*. Enio, no smoke seguinte:
///
/// > *"em vez de um evento para cada transição, melhor seria um evento por shape (…) se a seta para
/// > cima leva ao retângulo azul, independente de que forma estiver ativa no momento, a seta para
/// > cima vai levar ao retângulo azul (…) assim reduzimos o número de transições no painel para o
/// > número de formas envolvidas."*
///
/// ⚠️ **As passagens não desapareceram — deixaram de ser GUARDADAS.** De qualquer forma continua a
/// dar para ir a qualquer outra; o que era guardado `n(n-1)` vezes é a **condição**, e ela pertence
/// ao destino. Guardá-la por par obrigava o artista a escrever a mesma tecla `n-1` vezes para ela
/// significar sempre a mesma coisa — e nada impedia que as `n-1` discordassem.
///
/// ⭐ **A lista encolhe de `n(n-1)` para `n`:** com 9 formas, de **72** linhas no painel para
/// **9**.
///
/// ⚠️ **A ordem é a dos membros, e a primeira é onde a máquina nasce** (o `start` é derivado —
/// [`MorphGraph::start`]). A lista do painel indexa por posição, então uma ordem que dependesse de
/// iteração de mapa faria o menu de uma linha escrever a tecla noutra depois de um undo.
#[must_use]
pub(crate) fn states_for(shapes: &[VecPathId]) -> MorphGraph {
    MorphGraph {
        states: shapes.iter().map(|&s| MorphState::new(s)).collect(),
    }
}

/// **Cria o conjunto**: põe o path novo na cena e devolve o pendente. `None` se a seleção não dá
/// para um conjunto.
///
/// ⚠️ **O path nasce VAZIO de propósito** — a geometria é DERIVADA pelo `morph_live::recook` de
/// todo quadro, e inventá-la aqui seria uma 2ª porta para a mesma pergunta.
pub(crate) fn create(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    sel: &[VecPathId],
    max_states: usize,
) -> Option<MorphSetPending> {
    let members = eligible(sim, map, sel);
    if members.len() < 2 || members.len() > max_states {
        return None;
    }
    let path = scene.push_path(VecPath::default());
    Some(MorphSetPending {
        // O nome diz **quantos estados**, que é a coisa que o artista quer reconhecer na árvore.
        name: format!("Morph States {}", members.len()),
        path,
        members,
    })
}

/// **Drena o pendente** — pendura os componentes, reparenta os membros e esconde-os.
///
/// Roda entre o `vec_entities::sync` (a entidade do path já existe) e o `morph_live::recook` — o
/// mesmo lugar do `morph_live::upkeep`, e pela mesma razão.
///
/// ⚠️ **Devolve `true` quando consumiu**, e o chamador limpa o slot. Se a forma sumiu entretanto
/// (o artista apagou), consome à mesma: um pendente que nunca resolve ficaria a tentar para sempre.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<MorphSetPending>,
) {
    let Some(p) = pending.as_ref() else { return };
    if !scene.paths().iter().any(|q| q.id == p.path) {
        *pending = None;
        return;
    }
    let Some(&bits) = map.get(&p.path) else {
        return;
    };
    let host = Entity::from_bits(bits);
    if sim.world().get_entity(host).is_err() {
        return;
    }
    let start = p.members[0];
    let graph = states_for(&p.members);
    let members: Vec<Entity> = p
        .members
        .iter()
        .filter_map(|id| map.get(id).map(|&b| Entity::from_bits(b)))
        .collect();
    let name = p.name.clone();

    // ⭐⭐ **ONDE O CONJUNTO FICA, e onde cada estado passa a ficar** (plano 32 W9).
    //
    // Enio, 2026-08-25: *"aqui, diferente da tool morph, todas as peças participantes são
    // alinhadas numa mesma posição e o morph states faz o morph numa mesma posição, não desloca a
    // peça de lugar."*
    //
    // ⚠️ **É a diferença de PRODUTO entre os dois objectos, e ela tem uma razão.** Um Morph
    // autorado entre duas formas em sítios diferentes *quer* que a forma viaje — é um efeito de
    // transição. Um conjunto de ESTADOS é um objecto só que muda de aparência: a personagem que
    // agacha não salta dois metros para a esquerda por isso.
    let centre = align(sim, scene, map, &p.members);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(host) {
        // ⚠️ **`sources = [start, start]`, e `t` no zero.** O `VecMorph::new` nasce a meio caminho
        // de propósito (um morph a `t=0` sobre a forma A não se anuncia), e aqui é o contrário: o
        // conjunto tem de mostrar **exactamente** o estado inicial, senão a primeira coisa que o
        // artista vê é uma forma que ele nunca desenhou.
        e.insert((
            VecMorph {
                sources: [start, start],
                t: 0.0,
            },
            VecMorphMachine { graph },
            Name::new(name),
            // ⭐⭐ **O conjunto tem POSE, e é isso que o torna arrastável** (plano 32 W9).
            //
            // Enio, 2026-08-25: *"o objeto criado como pai tem que ser arrastável no canvas como um
            // objeto qualquer (…) e deve arrastar os filhos junto."*
            //
            // ⚠️ **Um Morph comum vive na IDENTIDADE de propósito** (`morph_live`: a geometria dele
            // é MUNDO, reescrita a cada quadro, e uma pose por cima a deslocaria duas vezes).
            // O conjunto quebra essa regra pelo outro lado: o `recook` coze-o no referencial DELE,
            // a partir das poses **locais** dos filhos, e é por isso que a pose pode existir.
            Transform::from_translation(centre),
        ));
    }
    for m in members {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(m) {
            // ⚠️ **Esconder E reparentar, nesta ordem, na mesma escrita.** O `visible_chain` lê o
            // próprio E os ancestrais, então esconder o membro é o suficiente — e esconder o PAI
            // apagaria o conjunto inteiro, que é o objecto que se quer ver.
            e.insert((Visibility::hidden(), ChildOf(host)));
            // ⛔ O `RootOrder` sai: ele só vale para raízes, e um membro deixou de o ser. É o
            // mesmo par de operações do `vec_entities::group_entities`, e a razão é a mesma.
            e.remove::<ph2d_ecs::RootOrder>();
        }
    }
    *pending = None;
}

/// ⭐⭐ **ALINHA os membros num ponto só, e devolve esse ponto** (plano 32 W9).
///
/// Cada membro passa a ter o **centro do que se vê** exactamente sobre a origem do conjunto, e o
/// conjunto nasce no centro da caixa que os continha a todos — que é onde o artista estava a olhar.
///
/// # A conta, e por que ela preserva rotação, escala e cisalhamento
///
/// Para o membro `i`, com pose de mundo `translate(t) . M` (onde `M` é a parte linear) e centro de
/// mundo `c_i`, a pose LOCAL nova é `translate(t - c_i) . M`. Compondo com o pai (`translate(C)`),
/// o centro vai parar em `C + c_i - c_i = C` — **o mesmo ponto para todos**.
///
/// ⚠️ **Só a translação muda.** Girar ou re-escalar um estado ao alinhá-lo seria destruir o
/// desenho para o pôr no sítio; a diferença entre as formas é o que a máquina existe para mostrar.
///
/// ⚠️ **A caixa é a da CURVA** (`path_curve_bbox`), não a dos pontos de controlo: o artista alinha
/// o que vê, e uma tangente longe da tinta puxaria o centro para fora da forma.
///
/// ⚠️ **Os QUATRO cantos são transformados**, e não só o min/max local: sob rotação a caixa
/// alinhada aos eixos do mundo não é a imagem da caixa local, e usar dois cantos poria uma forma
/// girada fora do centro.
///
/// ⛔ **Um membro sem caixa fica onde está.** Uma forma degenerada não tem centro, e inventar um
/// arrastaria o resto do conjunto por causa dela.
fn align(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    members: &[VecPathId],
) -> ph2d_core::Vec2 {
    // 1) MEDE tudo antes de escrever o que quer que seja: a pose de mundo de cada membro e o
    //    centro de mundo do que ele desenha. Escrever a meio da medição faria o segundo membro ser
    //    medido contra um mundo que o primeiro já mexeu.
    let mut rows: Vec<(Entity, Transform, [f64; 2])> = Vec::new();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for &id in members {
        let Some(&bits) = map.get(&id) else { continue };
        let e = Entity::from_bits(bits);
        let Some((bl, tr)) = scene.path_curve_bbox(id) else {
            continue;
        };
        let world = crate::vec_transform::world_transform(sim, e);
        let x = crate::vec_transform::xform_of_transform(world);
        let (mut cl, mut ch) = ([f64::MAX; 2], [f64::MIN; 2]);
        for corner in [
            [bl[0], bl[1]],
            [tr[0], bl[1]],
            [tr[0], tr[1]],
            [bl[0], tr[1]],
        ] {
            let w = x.apply(corner);
            for k in 0..2 {
                cl[k] = cl[k].min(w[k]);
                ch[k] = ch[k].max(w[k]);
            }
        }
        for k in 0..2 {
            lo[k] = lo[k].min(cl[k]);
            hi[k] = hi[k].max(ch[k]);
        }
        rows.push((e, world, [(cl[0] + ch[0]) * 0.5, (cl[1] + ch[1]) * 0.5]));
    }
    if rows.is_empty() {
        return ph2d_core::Vec2::ZERO;
    }
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];

    // 2) ESCREVE: a pose local de cada membro, com só a translação mexida.
    for (e, world, own) in rows {
        #[allow(clippy::cast_possible_truncation)]
        let t = Transform {
            translation: ph2d_core::Vec2::new(
                world.translation.x - own[0] as f32,
                world.translation.y - own[1] as f32,
            ),
            ..world
        };
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(t);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ph2d_core::Vec2::new(centre[0] as f32, centre[1] as f32)
}

#[cfg(test)]
#[path = "morph_set_tests.rs"]
mod tests;
