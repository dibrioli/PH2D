//! ⭐ **O SYNC VIVO mestre → instância** (ADR-0164 / plano F4.3).
//!
//! Editar a receita muda todas as instâncias dela. O passe corre uma vez por quadro, compara
//! **bytes** e escreve só o que difere — é isso que o torna um ponto fixo, e sem isso ele
//! registaria um passo de undo por quadro para sempre.
//!
//! # A correspondência é DURÁVEL, e não posicional
//!
//! Cada peça de uma instância guarda `InstanceOf { master }` — a identidade da peça do mestre de
//! que ela nasceu (F4.2). Emparelhar por posição na árvore seria mais barato e **errado**: no dia
//! em que o mestre ganha uma peça no meio, os índices deslizam e cada peça passa a receber os
//! bytes da vizinha, em silêncio.
//!
//! # ⚠️ As quatro coisas que ele NÃO escreve, e porquê
//!
//! 1. **A pose de quem o runtime possui.** Um corpo dinâmico tem o `Transform` escrito pelo
//!    solver; o sync escrever ali poria dois autores na mesma célula, e por tique o readback
//!    marcaria a diferença como se fosse autoria. A pergunta tem UMA porta —
//!    [`ph2d_physics_ecs::PhysicsBridge::document_owns_pose`] —, e ela é a condição (b) da
//!    [refutação 1].
//! 2. **O que é da RAIZ da instância** ([`ROOT_IS_ITS_OWN`]) — onde ela está, como se chama, em
//!    que ordem aparece. São os *"default overrides"* do Unity: nunca contam como override,
//!    porque nunca vieram do mestre.
//! 3. **O marcador de MESTRE** ([`NEVER_PROPAGATES`]). Propagá-lo faria a instância virar receita
//!    e **parar de cair** — o defeito da F4.1 ressuscitado pelo próprio sync.
//! 4. **Os documentos POSSUÍDOS** (`owned_document`), pela razão da cópia profunda: o id é opaco e
//!    copiá-lo poria duas entidades a escrever no mesmo documento.
//!
//! # ⚠️ E ele REMAPEIA, porque propagar bytes propaga referências
//!
//! A junta do mestre nomeia os corpos do mestre. Copiar os bytes dela para a instância sem
//! reescrever as pontas desligaria a instância do próprio rig — o mesmo defeito que a F4.2 curou
//! na instanciação, de volta pela porta do sync. É a mesma tabela ([`crate::instance_refs`]).
//!
//! [refutação 1]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, SimWorld, StableId};
use ph2d_physics_ecs::PhysicsBridge;
use std::collections::BTreeMap;

/// ⛔ **Nunca sai da receita, esteja onde estiver.**
///
/// O `MasterRoot` marca *«isto é uma biblioteca»*, e a instância que o recebesse ficaria inerte.
/// O `InstanceOf` é o elo da própria instância: o mestre não o tem, então a regra de remoção
/// (*o que o mestre não tem, a instância perde*) apagá-lo-ia — e a instância deixaria de existir
/// como instância no quadro seguinte.
const NEVER_PROPAGATES: &[&str] = &["ph2d::ecs::MasterRoot", "ph2d::ecs::InstanceOf"];

/// **O que a RAIZ de uma instância possui** — os *default overrides*.
///
/// ⚠️ A lista é do SÍTIO, não do tipo: o `Transform` de uma **peça** propaga (é a receita), e o da
/// raiz é onde o artista a largou. *Um tipo, duas respostas, escolhidas pelo lugar* — é o que o
/// descritor do `Transform` já diz, deixando a escolha para este passe.
const ROOT_IS_ITS_OWN: &[&str] = &[
    "ph2d::ecs::Transform",
    "ph2d::ecs::Name",
    "ph2d::ecs::RootOrder",
    "ph2d::ecs::SiblingOrder",
];

/// O componente cuja escrita depende de quem possui a pose.
const TRANSFORM: &str = "ph2d::ecs::Transform";

/// Uma instância viva: a raiz, os pares peça↔peça, e o mapa de identidade do remap.
struct Live {
    root: Entity,
    /// `(peça da instância, peça do mestre)`, a raiz primeiro.
    pairs: Vec<(Entity, Entity)>,
    /// `StableId do mestre → StableId da instância`.
    ids: BTreeMap<u64, u64>,
}

/// As instâncias da cena, em ordem determinística (por identidade da raiz).
fn live_instances(sim: &mut SimWorld) -> Vec<Live> {
    let by_id: BTreeMap<u64, Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &StableId)>();
        q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
    };
    // As RAÍZES: uma entidade cujo elo aponta para um mestre. ⚠️ Ordenadas pela identidade, e não
    // pelos bits: os bits mudam a cada respawn do undo, e a ordem de escrita tem de ser a mesma
    // em qualquer máquina (o `physics_ecs_c9` compara os 3 SO entre si).
    let mut roots: Vec<(u64, Entity)> = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf, &StableId)>();
        q.iter(sim.world())
            .filter(|(_, link, _)| {
                by_id
                    .get(&link.master)
                    .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some())
            })
            .map(|(e, _, s)| (s.0, e))
            .collect()
    };
    roots.sort_unstable();

    roots
        .into_iter()
        .map(|(_, root)| {
            let mut pairs = Vec::new();
            let mut ids = BTreeMap::new();
            let mut stack = vec![root];
            while let Some(e) = stack.pop() {
                if let Some(link) = sim.world().get::<InstanceOf>(e).copied()
                    && let Some(&m) = by_id.get(&link.master)
                {
                    pairs.push((e, m));
                    if let Some(s) = sim.world().get::<StableId>(e) {
                        ids.insert(link.master, s.0);
                    }
                }
                if let Some(kids) = sim.world().get::<Children>(e) {
                    stack.extend(kids.iter().copied());
                }
            }
            // A raiz primeiro, e o resto por identidade do mestre — determinismo outra vez.
            pairs.sort_by_key(|&(e, m)| (e != root, sim.world().get::<StableId>(m).map(|s| s.0)));
            Live { root, pairs, ids }
        })
        .collect()
}

/// ⭐ **Propaga o mestre para cada instância dele.** Devolve quantos componentes de facto mudaram
/// — `0` é o estado normal, e é o que faz deste passe um ponto fixo.
pub(crate) fn sync_instances(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    bridge: &PhysicsBridge,
) -> usize {
    let mut wrote = 0;
    for live in live_instances(sim) {
        // Os que carregam REFERÊNCIA e por isso não se decidem por bytes — ver abaixo.
        let mut pending: Vec<(u64, Entity, Option<Vec<u8>>)> = Vec::new();
        for &(inst, master) in &live.pairs {
            let is_root = inst == live.root;
            for entry in registry.iter() {
                let name = entry.canonical_name;
                if NEVER_PROPAGATES.contains(&name)
                    || (is_root && ROOT_IS_ITS_OWN.contains(&name))
                    || entry.desc.is_some_and(|d| d.owned_document)
                {
                    continue;
                }
                if name == TRANSFORM && !bridge.document_owns_pose(sim.world(), inst) {
                    continue;
                }
                let want = (entry.serialize)(sim.world(), master).unwrap_or_default();
                let have = (entry.serialize)(sim.world(), inst).unwrap_or_default();
                // ⚠️⚠️ **Para quem carrega REFERÊNCIA, a comparação por bytes não é decidível
                // aqui** — e isto foi medido, não previsto: a junta do mestre nomeia os corpos do
                // mestre e a da instância nomeia os dela, **de propósito**. Comparar os bytes dá
                // *«diferente»* para sempre, e o passe reescrevia a junta todo o quadro.
                //
                // ⇒ estes escrevem-se, RELIGAM-SE, e só então se pergunta se alguma coisa de facto
                // mudou (a fase de baixo). A conta continua honesta e o passe continua ponto fixo.
                if crate::instance_refs::carries_object_ref(name) {
                    match &want {
                        Some(bytes) => {
                            let _ = (entry.insert_from_bytes)(sim.world_mut(), inst, bytes);
                        }
                        None => (entry.remove)(sim.world_mut(), inst),
                    }
                    pending.push((entry.type_id, inst, have));
                    continue;
                }
                // ⚠️ **A comparação por BYTES é o `set_if_neq`** — sem ela toda escrita marcaria
                // change detection e o readback sujaria 100% dos corpos por tique (condição (e)
                // da refutação 1).
                if want == have {
                    continue;
                }
                match want {
                    Some(bytes) => {
                        if (entry.insert_from_bytes)(sim.world_mut(), inst, &bytes).is_ok() {
                            wrote += 1;
                        }
                    }
                    // O mestre perdeu o componente ⇒ a instância perde-o também. Sem esta metade,
                    // tirar um `Collider` da receita deixaria as instâncias a colidir para sempre.
                    None => {
                        (entry.remove)(sim.world_mut(), inst);
                        wrote += 1;
                    }
                }
            }
        }
        // ⚠️ **Depois de escrever, RELIGAR.** Os bytes que acabaram de chegar nomeiam os corpos do
        // MESTRE; sem isto a junta da instância larga o rig dela na primeira propagação.
        //
        // ⚠️⚠️ **E o remap salta exatamente o que o passe não propaga** — *o que não propaga não se
        // remapeia*. O `InstanceOf.master` da raiz É a identidade do mestre, logo uma chave do
        // mapa: remapeá-lo apontava a instância para SI PRÓPRIA, e a partir do 2.º quadro o sync
        // deixava de a encontrar. Ver [`crate::instance_refs::remap_object_refs_except`], onde a
        // medição está escrita.
        let entities: Vec<Entity> = live.pairs.iter().map(|&(e, _)| e).collect();
        crate::instance_refs::remap_object_refs_except(
            sim.world_mut(),
            &entities,
            &live.ids,
            NEVER_PROPAGATES,
        );
        // ⭐ **Só AGORA se pode perguntar se a referência mudou** — depois de religada, ela está no
        // espaço da instância, que é onde estava antes. Igual ⇒ o passe não mudou nada, e o gate
        // do ponto fixo continua a valer.
        for (type_id, inst, before) in pending {
            let Some(entry) = registry.get_by_id(type_id) else {
                continue;
            };
            if (entry.serialize)(sim.world(), inst).unwrap_or_default() != before {
                wrote += 1;
            }
        }
    }
    wrote
}

impl crate::App {
    /// O passe, uma vez por quadro.
    ///
    /// ⚠️ **Depois do quadro e ANTES do `post_frame_undo`**, e as duas metades são a razão:
    ///
    /// - *antes da captura*, senão a escrita do sync chega ao mundo depois da fotografia e vira um
    ///   passo de undo espúrio no quadro seguinte — um passo que o artista não deu;
    /// - *depois do quadro*, porque é aí que as edições do Inspector já foram aplicadas ao mundo
    ///   (`apply_editor_commands` corre no fim do laço). Pô-lo antes faria as instâncias andarem
    ///   **um quadro atrás do mestre**: o artista veria a peça da receita mudar e as cópias
    ///   seguirem depois, que é exatamente o que *«mudam no mesmo quadro»* proíbe.
    pub(crate) fn sync_instances(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Três campos disjuntos do `AppGfx` — sem clonar o registo nem a ponte.
        sync_instances(&mut gfx.sim, &gfx.component_registry, &gfx.physics);
    }
}

#[cfg(test)]
#[path = "instance_sync_tests.rs"]
mod tests;
