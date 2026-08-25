//! **O RIG SAI DA HIERARQUIA** (W-Rig) — a metade de SHELL.
//!
//! O kernel (`ph2d_physics_ecs::{subtree_parts, rig_edges}`) responde a
//! TOPOLOGIA e nada mais; aqui mora o que ele deliberadamente não sabe:
//!
//! - **quem é parte** — precisa de `Sprite`, que é da `ph2d-render`, e a
//!   `ph2d-physics-ecs` não a conhece nem deve;
//! - **como uma parte vira corpo** — pela porta da §11 (`PhysicsFieldEdit::Add`),
//!   que já sabe tirar o collider da CAIXA DO SPRITE. Uma segunda regra aqui
//!   seria um rig cujos colliders discordam dos que o botão *Add Body* faz;
//! - **o que já existe** — uma aresta que já tem joint é pulada, e é isso que
//!   torna o gerador **re-executável**: acrescente um braço ao personagem,
//!   clique de novo, e só o braço novo ganha joint.
//!
//! ⚠️ **Dois joints sobre o mesmo par são LEGÍTIMOS em geral** (é metade do
//! motivo de um joint ser ENTIDADE, W3) — o que esta wave recusa é o GERADOR
//! produzi-los, porque um rig com duas restrições no mesmo par é o solver
//! brigando consigo mesmo sobre um par que ninguém autorou duas vezes.

use std::collections::BTreeSet;

#[cfg(test)]
use ph2d_ecs::Name;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld, Transform, World};
use ph2d_physics_ecs::{JointKind, PhysicsJoint, RigidBody, rig_edges, subtree_parts};

/// O que um clique em *Rig* faria — contado ANTES de fazer, porque o rótulo do
/// botão diz o número (a lei do `Bake 5.0s to Timeline` e do `Paste to 3
/// Joints`: um clique que muda N objetos tem de dizer N antes).
pub(crate) struct RigPlan {
    /// As partes da subárvore, em ordem estável.
    pub(crate) parts: Vec<Entity>,
    /// `(pai, filho)` por aresta que AINDA não tem joint.
    pub(crate) edges: Vec<(Entity, Entity)>,
}

impl RigPlan {
    /// Vazio, para o caso comum de não haver seleção.
    pub(crate) const fn empty() -> Self {
        Self {
            parts: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// O botão é oferecido? Só com uma aresta a criar — sem isso, dois irmãos
    /// selecionados (que não têm ancestral um do outro) veriam um botão que não
    /// faz nada.
    pub(crate) fn is_offered(&self) -> bool {
        !self.edges.is_empty()
    }
}

/// O que o rig FARIA sobre a subárvore de `roots`.
///
/// ⚠️ **A subárvore, não a seleção literal** — selecionar o tronco é o gesto de
/// *"rigar este personagem"*, e é o mesmo raciocínio de expansão que o
/// `jointed_group` usa no Bake (assar meio rig é um rig quebrado). A contagem no
/// rótulo é a divulgação: se você esperava três partes e ele diz seis, você vê.
pub(crate) fn plan(sim: &mut SimWorld, roots: &[u64]) -> RigPlan {
    if roots.is_empty() {
        return RigPlan::empty();
    }
    let roots: Vec<Entity> = roots.iter().map(|&b| Entity::from_bits(b)).collect();
    // As duas queries ANTES do empréstimo compartilhado: uma query precisa de
    // `&mut World`, e o predicado de parte precisa de `&World` ao mesmo tempo.
    let candidates: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Transform)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    let joined: BTreeSet<(u64, u64)> = {
        let mut q = sim.world_mut().query::<&PhysicsJoint>();
        q.iter(sim.world())
            .map(|j| pair_key(j.body_a, j.body_b))
            .collect()
    };

    let world = sim.world();
    // **Uma parte tem EXTENSÃO DESENHADA, ou já é um corpo.** O sprite é o que dá
    // ao collider um tamanho que casa com o que se vê; um corpo sem sprite é uma
    // parte física deliberada (um pé invisível) e entra pelo mesmo direito. O que
    // fica de fora é o nó puramente organizacional — e ele é TRANSPARENTE, não um
    // corte: o kernel liga o filho ao avô.
    let parts = subtree_parts(world, &roots, candidates, |e| {
        world.get::<ph2d_render::Sprite>(e).is_some() || world.get::<RigidBody>(e).is_some()
    });
    let edges = rig_edges(world, &parts)
        .into_iter()
        .filter(|&(a, b)| match (name_id(world, a), name_id(world, b)) {
            // Sem nome dos dois lados, nenhum joint pode estar apontando para
            // este par — um joint guarda hashes de `Name` (W3).
            (Some(na), Some(nb)) => !joined.contains(&pair_key(na, nb)),
            _ => true,
        })
        .collect();
    RigPlan { parts, edges }
}

/// O que um rig FEZ — para o toast, e para o `last` que a §12 passa a mostrar.
pub(crate) struct RigOutcome {
    pub(crate) bodies: usize,
    pub(crate) joints: usize,
    pub(crate) last: Option<Entity>,
    /// A fila de comandos recusou algo? O chamador é quem tem o deck de toasts.
    pub(crate) error: Option<String>,
}

/// Executa o plano: corpo a quem não tem, **flush**, um joint por aresta.
///
/// ⚠️ **O FLUSH ENTRE AS DUAS METADES É LOAD-BEARING, e este doc já afirmou o
/// contrário.** Ele dizia que a ordem não importava porque *"o `Add` enfileira
/// `RigidBody`+`Collider` e o `create_joint` lê `Transform` e `Name` — conjuntos
/// disjuntos"*. Era verdade, e deixou de ser no instante em que a criação passou a
/// ancorar na **EMENDA** (`ph2d_physics_ecs::seam_between`): a emenda mede as duas
/// SILHUETAS, e a silhueta é o `Collider` que ainda está na fila.
///
/// O sintoma não seria um erro — seria a emenda caindo em silêncio no fallback do
/// ponto médio, ou seja o rig inteiro ancorando pelo desenho antigo enquanto o
/// resto do app usa o novo. Medido: o pescoço do boneco da cena 67 nascia em
/// `y = 3,35` (o meio) em vez de `y = 3,50` (a junta).
///
/// ⚠️ **Um passo de undo, e de graça:** tudo cai no MESMO frame, e o undo global
/// é por DIFF de fim de frame — ele vê um estado, não N operações (nem duas
/// drenagens). É a mesma linha que a `join_chain` escreveu.
pub(crate) fn apply(
    sim: &mut SimWorld,
    plan: &RigPlan,
    kind: JointKind,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> RigOutcome {
    let mut error = None;
    let mut bodies = 0;
    for &e in &plan.parts {
        // ⚠️ **Só quem NÃO tem corpo.** O `Add` reescreve o `Collider` a partir da
        // caixa do sprite, então passá-lo por cima de uma parte já autorada
        // apagaria a forma, o offset e o material que o artista escolheu — o
        // gerador desfazendo trabalho no clique que deveria acrescentar.
        if sim.world().get::<RigidBody>(e).is_none() {
            crate::render_loop::inspector_physics::apply_physics_edit(
                sim,
                e.to_bits(),
                ph2d_editor::PhysicsFieldEdit::Add,
                queue,
                registry,
            );
            bodies += 1;
        }
    }
    // ⚠️ AQUI, e não no chamador: o `create_joint` abaixo lê o `Collider` que o
    // laço acima acabou de enfileirar.
    if let Err(e) = ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), queue, registry) {
        error = Some(e.to_string());
    }
    let mut made = 0;
    let mut last = None;
    for &(parent, child) in &plan.edges {
        // **A = o PAI.** O filho pende do pai, e é o lado A que o pivô segue
        // (`sync_joint_pivots`, W-AnchorFollow) — invertido, o dot de um braço
        // seguiria a mão.
        if let Some(j) = crate::render_loop::inspector_joint::create_joint(
            sim,
            parent.to_bits(),
            child.to_bits(),
            kind,
        ) {
            // **O rig nasce com BATENTES, e as outras duas rotas não.** Não é uma
            // propriedade do "Pin" — é uma propriedade de *isto é um RIG*: sem
            // batente o boneco dobra a cabeça 176° para dentro do peito (medido),
            // e o wizard existe justamente para o artista não afinar N juntas à
            // mão. O botão *Join* faz UM joint e você já está olhando para a §12;
            // este faz cinco de uma vez.
            if let Some((lo, hi)) = ph2d_physics_ecs::rig_limits(kind)
                && let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(j)
            {
                c.limits_enabled = true;
                c.limit_min = lo;
                c.limit_max = hi;
            }
            made += 1;
            last = Some(j);
        }
    }
    RigOutcome {
        bodies,
        joints: made,
        last,
        error,
    }
}

/// A chave de um par, sem direção — um joint `A→B` e um `B→A` ligam o mesmo par.
const fn pair_key(a: u64, b: u64) -> (u64, u64) {
    if a <= b { (a, b) } else { (b, a) }
}

/// A identidade de `e` — a chave por que um joint o nomeia (ADR-0164 F1).
///
/// ⚠️ Era o **hash do nome**, e por isso o rig tratava um corpo sem nome como inexistente
/// (`filter(|n| !n.as_str().is_empty())`). Com identidade essa condição desapareceu: todo
/// objeto tem id, com ou sem nome. O `None` fica para a entidade que já não existe.
fn name_id(world: &World, e: Entity) -> Option<u64> {
    world.get::<ph2d_ecs::StableId>(e).map(|s| s.0)
}

#[cfg(test)]
#[path = "joint_rig_tests.rs"]
mod tests;
