//! ⭐⭐⭐ **O RENDER POR MALHA** — a peça do modelador vira OBJETOS DE TRIÂNGULOS (ordem do dono,
//! 2026-10-02: *«ao entrar no modo render os objetos compostos tornam-se objetos simples de malha;
//! crie tantos objetos quanto estiverem separados»*).
//!
//! # As três perguntas, e a porta de cada uma
//!
//! | pergunta | porta |
//! |---|---|
//! | o que é UM objeto do Render? | um SÓLIDO conexo — [`ph2d_field_eval::extract::extract_parts`]. O que se toca numa booleana funde; o que está solto separa (a regra do dono, 02/10) |
//! | quem o artista MOVE? | as [`unidades`]: a árvore aberta só por uniões SECAS sem modificador cujos filhos somam todos. Mover um objeto do Render escreve o `FieldPose` delas — a peça move-se DE VERDADE, e o undo é o de sempre |
//! | de que cor é cada triângulo? | a folha dona do centro dele ([`ph2d_field_eval::owners::Owners`]), na ordem de [`crate::materials::folhas`] — o índice é o do material |
//!
//! ⭐ **Um objeto é MÓVEL quando as unidades dele não aparecem em mais nenhum.** Metade de um toro
//! cortado, ou a cópia de um espelho, partilha a unidade com outro objeto — movê-la sozinha não tem
//! tradução na árvore, e o gizmo diz porquê em vez de mover outra coisa.
//!
//! ⭐ **Os grupos de extração** são as unidades cujas bolas de bordo se tocam: uma bola que não toca
//! nenhuma outra não pode tocar sólido nenhum, e cada grupo extrai na SUA grade — a célula encolhe
//! com o grupo, em vez de uma grade da cena inteira gastar resolução no vazio entre os objetos.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, NodeShape, Op, Xform};
use ph2d_field_ecs::{FieldMods, FieldNode, FieldVerb};

/// A resolução de cada grupo: `2^DEPTH` células no diâmetro da bola dele.
pub const DEPTH: u8 = 7;

/// ⭐ **As unidades MÓVEIS da peça**, na ordem da Hierarquia.
#[must_use]
pub fn unidades(world: &World, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if ph2d_field_ecs::is_hidden(world, e) || !ph2d_field_ecs::contributes(world, e) {
            continue;
        }
        match abre(world, e) {
            Some(kids) => stack.extend(kids.into_iter().rev()),
            None => out.push(e),
        }
    }
    out
}

/// Os filhos de `e` quando ele é uma união seca que se pode abrir sem mudar a forma; senão `None`.
fn abre(world: &World, e: Entity) -> Option<Vec<Entity>> {
    let node = world.get::<FieldNode>(e)?;
    if !matches!(node.shape, NodeShape::Combine(Op::Union(Blend::Sharp))) {
        return None;
    }
    if world.get::<FieldMods>(e).is_some_and(|m| !m.stack.is_empty()) {
        return None;
    }
    let kids: Vec<Entity> = world
        .get::<bevy_ecs::hierarchy::Children>(e)
        .map(|c| c.iter().copied().collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|&k| {
            world.get::<FieldNode>(k).is_some()
                && !ph2d_field_ecs::is_hidden(world, k)
                && ph2d_field_ecs::contributes(world, k)
        })
        .collect();
    // ⚠️ O verbo do PRIMEIRO só semeia o acumulado (ver `FieldVerb`); os outros têm de somar.
    let todos_somam = kids.iter().skip(1).all(|&k| {
        world
            .get::<FieldVerb>(k)
            .is_none_or(|v| matches!(v.op, Op::Union(Blend::Sharp)))
    });
    todos_somam.then_some(kids)
}

/// A união SECA de documentos já postos no mundo. ⚠️ O verbo da raiz de cada um é limpo: dentro da
/// peça ele era relativo aos irmãos de lá, e aqui todos somam.
#[must_use]
pub fn uniao(docs: &[FieldDoc]) -> Option<FieldDoc> {
    if docs.len() == 1 {
        return Some(docs[0].clone());
    }
    let mut nodes: Vec<Node> = Vec::new();
    let mut raizes = Vec::new();
    for d in docs {
        let base = nodes.len() as u32;
        for n in d.nodes() {
            let mut n = n.clone();
            if let NodeKind::Combine { children, .. } = &mut n.kind {
                for c in children.iter_mut() {
                    c.0 += base;
                }
            }
            nodes.push(n);
        }
        let raiz = base + d.root().0;
        nodes[raiz as usize].verb = None;
        raizes.push(NodeId(raiz));
    }
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: raizes,
        },
    ));
    let root = NodeId(nodes.len() as u32 - 1);
    FieldDoc::new(nodes, root).ok()
}

/// ⭐ **Um objeto do Render**: a malha, quem ele move, e a pose de referência com que foi extraído.
#[derive(Clone, Debug)]
pub struct ObjetoRender {
    /// As unidades cuja superfície está neste objeto, na ordem de [`unidades`].
    pub unidades: Vec<Entity>,
    /// As unidades dele não aparecem em mais nenhum objeto.
    pub movel: bool,
    pub malha: crate::malha_render_tri::MalhaPronta,
    /// A pose de MUNDO de `unidades[0]` quando a malha foi extraída: o quadro desenha com
    /// `pose_agora ∘ pose_extraida⁻¹`.
    pub pose_extraida: Xform,
}

/// ⭐⭐⭐ **A PEÇA → OS OBJETOS DO RENDER.**
#[must_use]
pub fn extrair(world: &World, root: Entity, reg: &ph2d_field_eval::hybrid::Registry) -> Vec<ObjetoRender> {
    let us = unidades(world, root);
    let postos: Vec<(Entity, FieldDoc)> = us
        .iter()
        .filter_map(|&u| Some((u, ph2d_field_ecs::cook(world, u)?.ok()?)))
        .collect();
    let bolas: Vec<ph2d_field_eval::bounds::Ball> = postos
        .iter()
        .map(|(_, d)| {
            ph2d_field_eval::bounds::bounding_ball(d, reg)
                .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY)
        })
        .collect();
    let grupos = agrupa(&bolas);

    let folhas = crate::materials::folhas(world, root);
    let unidade_da_folha: Vec<Option<usize>> = folhas
        .iter()
        .map(|(e, _, _)| ancestral_em(world, *e, &postos))
        .collect();
    let placed: Vec<FieldDoc> = folhas.iter().map(|(_, _, d)| d.clone()).collect();

    let feitos: Vec<Vec<(Vec<usize>, crate::malha_render_tri::MalhaPronta)>> =
        std::thread::scope(|s| {
            let tarefas: Vec<_> = grupos
                .iter()
                .map(|g| {
                    let docs: Vec<FieldDoc> = g.iter().map(|&i| postos[i].1.clone()).collect();
                    let unidade_da_folha = &unidade_da_folha;
                    // As folhas DESTE grupo: compilar as de fora seria pagar JIT por quem não
                    // pode ser dono de ponto nenhum aqui.
                    let mapa: Vec<usize> = (0..placed.len())
                        .filter(|&i| unidade_da_folha[i].is_some_and(|u| g.contains(&u)))
                        .collect();
                    let sub: Vec<FieldDoc> = mapa.iter().map(|&i| placed[i].clone()).collect();
                    s.spawn(move || {
                        let Some(doc) = uniao(&docs) else {
                            return Vec::new();
                        };
                        let cell = ph2d_field_eval::extract::cell_size(&doc, reg, DEPTH) as f32;
                        let donos = ph2d_field_eval::owners::Owners::new(&sub, reg, cell);
                        let Ok(partes) =
                            ph2d_field_eval::extract::extract_parts(&doc, reg, DEPTH)
                        else {
                            return Vec::new();
                        };
                        let mut campo = ph2d_field_eval::hybrid::Hybrid::new(&doc, reg);
                        partes
                            .iter()
                            .map(|m| {
                                crate::malha_render_tri::prepara(
                                    m,
                                    &donos,
                                    &mapa,
                                    unidade_da_folha,
                                    &mut campo,
                                    cell,
                                )
                            })
                            .collect()
                    })
                })
                .collect();
            tarefas
                .into_iter()
                .map(|t| t.join().unwrap_or_default())
                .collect()
        });

    let todos: Vec<(Vec<usize>, crate::malha_render_tri::MalhaPronta)> =
        feitos.into_iter().flatten().filter(|(_, m)| !m.indices.is_empty()).collect();
    let mut vezes = vec![0usize; postos.len()];
    for (us, _) in &todos {
        for &u in us {
            vezes[u] += 1;
        }
    }
    todos
        .into_iter()
        .filter(|(us, _)| !us.is_empty())
        .map(|(us, malha)| ObjetoRender {
            movel: us.iter().all(|&u| vezes[u] == 1),
            pose_extraida: ph2d_field_ecs::world_xform(world, postos[us[0]].0),
            unidades: us.iter().map(|&u| postos[u].0).collect(),
            malha,
        })
        .collect()
}

/// Os grupos de extração: as bolas que se tocam, por união-busca. Ordem estável (HR-5).
fn agrupa(bolas: &[ph2d_field_eval::bounds::Ball]) -> Vec<Vec<usize>> {
    let mut pai: Vec<usize> = (0..bolas.len()).collect();
    fn acha(pai: &mut [usize], mut a: usize) -> usize {
        while pai[a] != a {
            pai[a] = pai[pai[a]];
            a = pai[a];
        }
        a
    }
    for i in 0..bolas.len() {
        for j in i + 1..bolas.len() {
            let (a, b) = (bolas[i], bolas[j]);
            let d2: f32 = (0..3).map(|k| (a.center[k] - b.center[k]).powi(2)).sum();
            if d2.sqrt() <= a.radius + b.radius {
                let (ra, rb) = (acha(&mut pai, i), acha(&mut pai, j));
                pai[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    let mut grupos: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for i in 0..bolas.len() {
        let r = acha(&mut pai, i);
        grupos.entry(r).or_default().push(i);
    }
    grupos.into_values().collect()
}

/// Qual unidade (índice em `postos`) é antepassada de `e` — `None` se a folha está fora de todas
/// (escondida, ou numa subárvore que não rende).
fn ancestral_em(world: &World, e: Entity, postos: &[(Entity, FieldDoc)]) -> Option<usize> {
    let mut cur = Some(e);
    while let Some(c) = cur {
        if let Some(i) = postos.iter().position(|(u, _)| *u == c) {
            return Some(i);
        }
        cur = world.get::<bevy_ecs::hierarchy::ChildOf>(c).map(|p| p.0);
    }
    None
}

/// ⭐ **A pose com que o quadro desenha o objeto**: `agora ∘ extraída⁻¹` (o inverso é o
/// [`Xform::local_under`] da casa, com o mundo na identidade).
#[must_use]
pub fn delta(agora: Xform, extraida: Xform) -> Xform {
    agora.compose(Xform::local_under(extraida, Xform::IDENTITY))
}

#[cfg(test)]
#[path = "malha_render_tests.rs"]
mod tests;
