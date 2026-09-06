//! **O ESQUELETO, vivo** (estudo 42 item 5, doc 47) — a forma presa aos ossos é re-cozida a cada
//! quadro a partir da fonte autorada e da pose de agora.
//!
//! Irmão do [`crate::envelope_live`] no padrão (fonte em bytes dentro do componente · `replace_cooked`
//! por quadro · undo e save de graça porque os dois capturam o mundo ECS) e diferente em duas coisas
//! que valem a pena ler antes de mexer:
//!
//! 1. **Não há container.** A gaiola do envelope não é entidade e precisa de um dono; um esqueleto
//!    **já são entidades**, então a forma presa fica onde o artista a pôs.
//! 2. **A cinemática não se escreve.** O mundo de cada osso sai de `parent_world_transform`, que é a
//!    propagação de `Transform` que a casa já corre — logo FK é de borla, e a timeline anima um osso
//!    porque anima um `Transform`.
//!
//! ⚠️ **O ponto onde isto se parte, se alguém o refactorar:** a matriz de um osso é
//! `S_agora⁻¹ ∘ B_agora ∘ rest⁻¹`, e o `rest` guardado **é** `S_bind⁻¹ ∘ B_bind`. Ligar num espaço e
//! cozer noutro devolve uma forma que salta para longe no instante do bind. A composição vive numa
//! porta só ([`ph2d_vec_skin::SkinBone::new`]) por causa disso.

use ph2d_ecs::{ChildOf, Entity, SimWorld, VecBone, VecPathRef, VecSkin, VecSkinBone};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, Xform};
use ph2d_vec_skin::{Skin, SkinBone};

use crate::vec_entities::VecEntityMap;

/// O que sobra quando se solta uma forma do esqueleto — os dois verbos do envelope, pela mesma razão
/// (o artista pode querer **o que vê** ou **o que desenhou**, e adivinhar é que não).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Keep {
    /// A geometria deformada de agora vira o desenho. É o *Expand*.
    Deformed,
    /// A fonte autorada volta. É o *Release*.
    Source,
}

/// O afim local→mundo de uma entidade.
fn world_of(sim: &SimWorld, e: Entity) -> Xform {
    crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, e))
}

/// **Os ossos de um esqueleto, em MUNDO** — `(bits, origem, ponta)`, para o overlay desenhar e para
/// o gesto apontar.
///
/// ⚠️ Devolve **todos** os ossos da cena: quem quer um esqueleto só filtra por
/// [`skeleton_of`]. Uma segunda varredura com outra regra divergiria desta na primeira ramificação.
pub(crate) fn bone_segments(sim: &SimWorld) -> Vec<(u64, [f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for (e, length) in ossos_da_cena(sim) {
        let x = world_of(sim, e);
        out.push((e.to_bits(), x.apply([0.0, 0.0]), x.apply([length, 0.0])));
    }
    out.sort_by_key(|(bits, _, _)| *bits);
    out
}

/// Todo osso da cena, com o comprimento dele.
///
/// ⚠️ **Varre entidades em vez de montar uma `QueryState`**, e a razão é a assinatura: uma query
/// pede `&mut World` para nascer, e os dois consumidores disto — o overlay e o gesto — só têm o
/// mundo emprestado. *Uma função que pede `&mut` só para ler obriga o chamador a arranjar um `&mut`
/// que ele não precisa, e é assim que um `clone` do mundo aparece num caminho de quadro.*
fn ossos_da_cena(sim: &SimWorld) -> Vec<(Entity, f64)> {
    sim.world()
        .iter_entities()
        .filter_map(|er| er.get::<VecBone>().map(|b| (er.id(), b.length)))
        .collect()
}

/// **O esqueleto de que este osso faz parte** — sobe até à raiz (o ancestral mais alto que ainda é
/// osso) e desce recolhendo tudo o que é osso.
///
/// `None` ⇒ **todos** os ossos da cena, que é a leitura certa de *"o artista não apontou nenhum"*
/// quando há um esqueleto só — o caso comum, e o que faz o botão Bind funcionar sem cerimónia.
///
/// ⚠️ **A ORDEM não tem sentido, e mesmo assim tem de ser DETERMINÍSTICA.** Os pesos normalizam-se,
/// então permutar os ossos devolve o mesmo desenho — mas a permutação muda a **ordem da soma** em
/// `f64`, logo o último ULP. Ordenar por `to_bits` (id de alocação) resolve isso *dentro de uma
/// sessão*, e depois do bind quem fixa a ordem é a lista **guardada** no componente. ⛔ Isto **não**
/// é o `canonicalize` que o `CLAUDE.md` §5 proíbe: ali os bits decidiam o CONTEÚDO de um snapshot;
/// aqui decidem só em que ordem se somam parcelas que já foram escolhidas.
pub(crate) fn skeleton_of(sim: &SimWorld, seed: Option<Entity>) -> Vec<Entity> {
    let todos: Vec<Entity> = ossos_da_cena(sim).into_iter().map(|(e, _)| e).collect();
    let Some(seed) = seed.filter(|e| todos.contains(e)) else {
        let mut t = todos;
        t.sort_by_key(|e| e.to_bits());
        return t;
    };
    // Sobe enquanto o PAI também for osso — parar no primeiro pai não-osso é o que permite pendurar
    // um esqueleto inteiro dentro de um grupo sem ele deixar de ser um esqueleto.
    let mut raiz = seed;
    while let Some(p) = sim.world().get::<ChildOf>(raiz).map(ChildOf::parent) {
        if sim.world().get::<VecBone>(p).is_some() {
            raiz = p;
        } else {
            break;
        }
    }
    let mut out = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(e) = pilha.pop() {
        if sim.world().get::<VecBone>(e).is_none() {
            continue;
        }
        out.push(e);
        if let Some(f) = sim.world().get::<ph2d_ecs::Children>(e) {
            pilha.extend(f.iter());
        }
    }
    out.sort_by_key(|e| e.to_bits());
    out
}

/// A pele de uma forma, resolvida para ESTE quadro. `None` quando não há osso vivo nenhum (todos
/// apagados) ou quando a pose da forma é singular — nos dois casos a forma fica em paz.
fn resolve(sim: &SimWorld, skin: &VecSkin, shape: Entity) -> Option<Skin> {
    let shape_inv = world_of(sim, shape).inverse()?;
    let mut ossos = Vec::with_capacity(skin.bones.len());
    for b in &skin.bones {
        let e = Entity::from_bits(b.bone);
        // Um osso apagado é SALTADO e os outros renormalizam-se sozinhos: apagar um osso não pode
        // apagar a forma.
        let Some(vb) = sim.world().get::<VecBone>(e).copied() else {
            continue;
        };
        if let Some(sb) = SkinBone::new(
            Xform(b.rest),
            vb.length,
            vb.strength,
            world_of(sim, e),
            shape_inv,
        ) {
            ossos.push(sb);
        }
    }
    Skin::new(ossos)
}

/// **Um quadro de pele.** Corre depois do `vec_entities::sync` (as entidades existem) e ao lado do
/// `envelope_live::recook`.
pub(crate) fn recook(sim: &SimWorld, scene: &mut VecScene) {
    let alvos: Vec<(Entity, VecSkin, VecPathId)> = sim
        .world()
        .iter_entities()
        .filter_map(|er| {
            Some((
                er.id(),
                er.get::<VecSkin>()?.clone(),
                er.get::<VecPathRef>()?.0,
            ))
        })
        .collect();
    for (e, skin, id) in alvos {
        let Some(pele) = resolve(sim, &skin, e) else {
            continue;
        };
        // Uma fonte corrompida é PULADA (não há o que deformar, e melhor não escrever lixo) — a
        // forma fica com a última geometria boa. Mesma escolha do envelope.
        let Ok(mut src) = postcard::from_bytes::<VecPath>(&skin.source) else {
            continue;
        };
        pele.apply(&mut src);
        if let Some(p) = scene.path_mut(id) {
            p.replace_cooked(src);
        }
    }
}

/// **Prende as formas ao esqueleto.** Devolve quantas prendeu.
///
/// A fonte é a geometria que a forma tem **agora** — o que faz um segundo Bind ser um *re-bind na
/// pose actual*, que é o gesto que todo o pacote de rig oferece. E como a pose de repouso é a
/// identidade por construção (§2.5 do doc 47), **prender não move um pixel**.
pub(crate) fn bind(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    seed: Option<Entity>,
) -> usize {
    let ossos = skeleton_of(sim, seed);
    if ossos.is_empty() {
        return 0;
    }
    let mut feitos = 0;
    for &id in paths {
        let Some(&bits) = map.get(&id) else { continue };
        let shape = Entity::from_bits(bits);
        if sim.world().get_entity(shape).is_err() {
            continue;
        }
        let Some(src) = scene.paths().iter().find(|p| p.id == id) else {
            continue;
        };
        let Ok(bytes) = postcard::to_allocvec(src) else {
            continue;
        };
        let Some(shape_inv) = world_of(sim, shape).inverse() else {
            continue;
        };
        // `rest = S⁻¹ ∘ B` — aplica o mundo do osso primeiro, depois leva ao espaço da forma.
        let tendoes: Vec<VecSkinBone> = ossos
            .iter()
            .map(|&e| VecSkinBone {
                bone: e.to_bits(),
                rest: world_of(sim, e).then(&shape_inv).0,
            })
            .collect();
        sim.world_mut()
            .entity_mut(shape)
            .insert(VecSkin::new(bytes, tendoes));
        feitos += 1;
    }
    feitos
}

/// **Solta as formas seleccionadas do esqueleto.** Devolve quantas soltou.
pub(crate) fn release(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    keep: Keep,
) -> usize {
    let mut feitos = 0;
    for &id in paths {
        let Some(&bits) = map.get(&id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(skin) = sim.world().get::<VecSkin>(e).cloned() else {
            continue;
        };
        if keep == Keep::Source
            && let Ok(src) = postcard::from_bytes::<VecPath>(&skin.source)
            && let Some(p) = scene.path_mut(id)
        {
            p.replace_cooked(src);
        }
        sim.world_mut().entity_mut(e).remove::<VecSkin>();
        feitos += 1;
    }
    feitos
}

#[cfg(test)]
#[path = "skin_live_tests.rs"]
mod tests;
