//! **O RIG SAI DA HIERARQUIA** (W-Rig) — a pergunta de TOPOLOGIA, pura.
//!
//! A rota por seleção liga uma **sequência**: N corpos marcados viram N−1 joints
//! em fila (`joint_draw::join_chain`, W-J4). Um ragdoll não é uma fila — a pelve
//! tem três filhos —, e uma corrente **não consegue expressá-lo**. O que ele é,
//! o artista já desenhou: a **árvore da Hierarquia**.
//!
//! A lei desta wave cabe numa frase: **uma aresta pai→filho da hierarquia É um
//! joint.**
//!
//! ## Por que o ancestral MAIS PRÓXIMO, e não o pai literal
//!
//! Um nó sem desenho é um **grupo** — a metade ORGANIZACIONAL da árvore, o
//! *grupo* do Harmony (a mesma distinção que o [ADR-0133] faz ao dizer que o
//! container é a metade TEMPORAL). Um grupo não é um osso: dar-lhe corpo
//! plantaria um collider invisível no meio do personagem, e pular a aresta sem
//! mais nada **desconectaria** o filho do rig.
//!
//! Então o grupo é **transparente**: quem não é parte não interrompe a linhagem,
//! e o filho se liga ao avô. É por isso que a função pergunta *"qual o ancestral
//! mais próximo que TAMBÉM é parte?"* em vez de olhar só o pai.
//!
//! ## O que esta função NÃO decide
//!
//! Quem é parte. Isso depende de haver DESENHO (um `Sprite`), e esta crate não
//! conhece `ph2d-render` — nem deve. O chamador entrega a lista; aqui só mora a
//! topologia, que é o que se pode provar sobre uma árvore sem saber o que os nós
//! são. É o mesmo corte do [`crate::jointed_group`]: função pura sobre o ECS
//! **AUTORADO**, headless-testável, sem um dispatch no caminho.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use std::collections::BTreeSet;

use ph2d_ecs::{ChildOf, Entity, World};

use crate::JointKind;

/// As arestas que um rig criaria sobre `parts`: para cada parte, o **ancestral
/// mais próximo que também é parte**, na forma `(pai, filho)`.
///
/// A ordem da saída segue a ordem de `parts` — a função não reordena nada, então
/// um chamador determinístico produz um rig determinístico.
///
/// Uma parte cuja linhagem inteira está fora da lista **não gera aresta**: ela é
/// uma raiz. Duas raízes na mesma chamada são dois rigs independentes, e é isso
/// que mantém dois personagens selecionados juntos sem um joint entre eles.
///
/// ⚠️ **Sem guarda de ciclo, de propósito.** A subida é um `while` sobre
/// `ChildOf`, exatamente como o `parent_world_transform_into` que a ponte usa
/// para toda pose de corpo-filho — uma hierarquia cíclica já trava aquele
/// caminho antes de chegar aqui, e uma segunda resposta a *"a árvore pode
/// ciclar?"* seria pior que nenhuma.
#[must_use]
pub fn rig_edges(world: &World, parts: &[Entity]) -> Vec<(Entity, Entity)> {
    let set: BTreeSet<Entity> = parts.iter().copied().collect();
    let mut edges = Vec::new();
    for &child in parts {
        let mut cur = world.get::<ChildOf>(child).map(ChildOf::parent);
        while let Some(p) = cur {
            if set.contains(&p) {
                edges.push((p, child));
                break;
            }
            cur = world.get::<ChildOf>(p).map(ChildOf::parent);
        }
    }
    edges
}

/// Todos os descendentes de `roots` (inclusive as próprias raízes) que satisfazem
/// `is_part`, em ordem **estável** dentro do frame.
///
/// ⚠️ **A varredura é para CIMA, e não para baixo, porque `ChildOf` é a única
/// aresta que existe** — não há índice de filhos no ECS. Cada candidato sobe a
/// própria linhagem procurando uma raiz; é `O(N · profundidade)`, o que numa
/// cena de editor é ruído, e evita construir um índice que teria de concordar
/// com o `ChildOf` (uma segunda resposta a *"quem é filho de quem?"*).
///
/// `candidates` vem de uma query do chamador, cuja ordem é de arquétipo e
/// portanto **não** é estável; a saída é ordenada para que o rig produza os
/// mesmos joints, com os mesmos nomes, em duas corridas iguais.
#[must_use]
pub fn subtree_parts(
    world: &World,
    roots: &[Entity],
    candidates: impl IntoIterator<Item = Entity>,
    is_part: impl Fn(Entity) -> bool,
) -> Vec<Entity> {
    let roots: BTreeSet<Entity> = roots.iter().copied().collect();
    let mut out: Vec<Entity> = candidates
        .into_iter()
        .filter(|&e| is_part(e) && in_subtree(world, e, &roots))
        .collect();
    out.sort_unstable();
    out
}

/// `e` é uma das raízes, ou desce de uma delas?
fn in_subtree(world: &World, e: Entity, roots: &BTreeSet<Entity>) -> bool {
    if roots.contains(&e) {
        return true;
    }
    let mut cur = world.get::<ChildOf>(e).map(ChildOf::parent);
    while let Some(p) = cur {
        if roots.contains(&p) {
            return true;
        }
        cur = world.get::<ChildOf>(p).map(ChildOf::parent);
    }
    false
}

/// **A meia-faixa de batente que um joint de RIG nasce com, em GRAUS.**
///
/// ⚠️ **MEDIDO, não escolhido por gosto.** Sem batente nenhum, o boneco da cena 67
/// dobra a cabeça **176°** relativos ao tronco — ela termina atrás do peito — e um
/// braço 170°: o *ragdoll-macarrão*, o modo de falha clássico de um wizard que só
/// resolve topologia. A varredura, 3 s de queda, pior ângulo relativo por junta:
///
/// | faixa | pescoço · braços · pernas | queda do tronco |
/// |---|---|---|
/// | **sem batente** | 176 · 170 · 136 · 70 · 124 | 3,30 m |
/// | ±90° | 87 · 90 · 90 · **67** · 90 | 3,01 m |
/// | **±60°** | 60 · 60 · 60 · 60 · 60 | **3,42 m** |
/// | ±45° | 45 · 45 · 45 · **14** · **6** | 2,60 m |
/// | ±30° | 30 · 30 · 30 · 30 · 30 | 2,62 m |
///
/// **±60° é a maior faixa em que TODA junta é de fato limitada** (as cinco batem
/// exatamente nela, logo todas teriam ido além) — em ±90° uma perna nem alcança o
/// batente, e em ±45° duas não alcançam. E o desabamento continua vivo: 3,42 m de
/// percurso, mais do que sem batente, porque o corpo se segura e escorrega em vez
/// de se desmontar.
///
/// ⚠️ **A faixa é simétrica em torno da pose AUTORADA, não em torno de zero**, e
/// isso sai de graça: `PhysicsWorld::axis_locals` alinha os DOIS frames locais à
/// rotação do joint, então o ângulo relativo vale 0 na pose em que o artista
/// desenhou. Um braço desenhado a 30° ganha limites centrados em 30°.
pub const RIG_LIMIT_DEG: f32 = 60.0;

/// Os batentes que um joint criado por um RIG nasce com — `None` quando o tipo
/// não tem batente, ou quando o batente dele é uma DISTÂNCIA.
///
/// ⚠️ **A segunda metade é uma armadilha de UNIDADE, não uma sutileza:** num
/// `Slider` (ou numa `Wheel`) o limite é o CURSO, em **metros**
/// (`JointKind::limits_in_metres`) — escrever `±60°` ali daria a um trilho
/// **±1,05 metro** de curso, um número que ninguém pediu e que não se lê como
/// erro. É a mesma classe que a W-JointCopy nomeou ao explicar por que o TIPO
/// viaja junto com os números.
#[must_use]
pub fn rig_limits(kind: JointKind) -> Option<(f32, f32)> {
    if !kind.has_limits() || kind.limits_in_metres() {
        return None;
    }
    let half = RIG_LIMIT_DEG.to_radians();
    Some((-half, half))
}
