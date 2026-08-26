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
use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, ObjectInstance, SimWorld, StableId};
use ph2d_physics_ecs::PhysicsBridge;
use std::collections::BTreeMap;

/// ⛔ **Nunca sai da receita, esteja onde estiver.**
///
/// O `MasterRoot` marca *«isto é uma biblioteca»*, e a instância que o recebesse ficaria inerte.
/// O `InstanceOf` é o elo da própria instância: o mestre não o tem, então a regra de remoção
/// (*o que o mestre não tem, a instância perde*) apagá-lo-ia — e a instância deixaria de existir
/// como instância no quadro seguinte.
const NEVER_PROPAGATES: &[&str] = &[
    "ph2d::ecs::MasterRoot",
    "ph2d::ecs::InstanceOf",
    "ph2d::ecs::ObjectInstance",
];

/// ⭐⭐⭐ **O ECO do mestre — como o passe sabe QUEM se mexeu.**
///
/// # ⚠️ Um diff sozinho NÃO atribui, e é isto que refuta o plano
///
/// O plano diz *«override por campo capturado por diff»*. Um diff só responde *«estão
/// diferentes»*: se o mestre mudou, `mestre != instância`; se a instância mudou, **também**. Ler o
/// diff como *«a instância mexeu-se»* transformaria cada edição da receita num override em todas
/// as instâncias; lê-lo como *«o mestre mexeu-se»* desfaria toda edição do artista no quadro
/// seguinte, calado. *É a mesma diferença de bytes para as duas causas.*
///
/// ⛔ **E o instrumento óbvio — o change tick — é CEGO à operação que mais dói:** a
/// [refutação 3](https://github.com/dibrioli/PH2D/blob/main/docs/Components/pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md)
/// mediu-o (*«remover componente não muda tick de ninguém: `remove::<Sprite>` em 1 % ⇒ **0** linhas
/// re-serializadas»*), e tirar um componente da receita é exatamente o que tem de chegar às
/// instâncias.
///
/// ⇒ o passe guarda o que o mestre tinha **no passe anterior**. Aí as duas perguntas separam-se:
/// `mestre_mexeu = eco != agora`, e o resto da diferença é da instância.
///
/// ⚠️ **O eco é do MESTRE, e por isso custa o mestre — não as instâncias.** Mil instâncias da mesma
/// receita partilham uma entrada. *É a mesma razão por que a biblioteca existe.*
///
/// ⚠️ **Ele responde por um lado só, e chega:** o eco diz se o MESTRE mexeu. O lado da instância
/// lê-se comparando com o mestre — excepto para quem carrega referência, e aí a resposta é *não
/// capturar* (ver o corpo do passe, onde a medição está escrita).
#[derive(Default)]
pub(crate) struct MasterEcho {
    /// `(peça do mestre, type_id)` → bytes que o mestre tinha no passe anterior.
    master: BTreeMap<(u64, u64), Option<Vec<u8>>>,
}

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

/// Uma peça da instância e a peça do mestre de que ela nasceu, com as duas identidades ao lado —
/// elas são a chave de tudo (override, eco, remap) e re-buscá-las por entidade custaria uma
/// travessia por componente.
struct Pair {
    inst: Entity,
    master: Entity,
    /// `StableId` da peça do MESTRE — a `piece` de um [`ph2d_ecs::OverrideKey`].
    master_id: u64,
}

/// Uma instância viva: a raiz, os pares peça↔peça, e o mapa de identidade do remap.
struct Live {
    root: Entity,
    /// A raiz primeiro.
    pairs: Vec<Pair>,
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
                    && let Some(inst_id) = sim.world().get::<StableId>(e).map(|s| s.0)
                {
                    pairs.push(Pair {
                        inst: e,
                        master: m,
                        master_id: link.master,
                    });
                    ids.insert(link.master, inst_id);
                }
                if let Some(kids) = sim.world().get::<Children>(e) {
                    stack.extend(kids.iter().copied());
                }
            }
            // A raiz primeiro, e o resto por identidade do mestre — determinismo outra vez.
            pairs.sort_by_key(|p| (p.inst != root, p.master_id));
            Live { root, pairs, ids }
        })
        .collect()
}

/// ⭐ **Propaga o mestre para cada instância dele.** Devolve quantos componentes de facto mudaram
/// — `0` é o estado normal, e é o que faz deste passe um ponto fixo.
///
/// # As três respostas por `(peça, componente)`
///
/// 1. **A instância possui** (há override) ⇒ não se toca. O valor dela é ela.
/// 2. **O mestre mexeu-se** (o eco discorda do agora) ⇒ propaga, mesmo que a instância também
///    tenha mexido. ⚠️ *Editar a receita é uma difusão deliberada*, e no empate ela ganha —
///    declarado, não descoberto.
/// 3. **Só a instância mexeu-se** ⇒ **nasce um override**, e o passe deixa o valor dela em paz.
pub(crate) fn sync_instances(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut MasterEcho,
) -> usize {
    let mut wrote = 0;
    // ⚠️ O eco novo é montado à parte e só entra no fim: várias instâncias partilham o mesmo
    // mestre, e atualizá-lo a meio faria a 2.ª instância ler *«o mestre não mexeu»*.
    let mut next_master: BTreeMap<(u64, u64), Option<Vec<u8>>> = BTreeMap::new();

    for live in live_instances(sim) {
        let mut overrides = sim
            .world()
            .get::<ObjectInstance>(live.root)
            .cloned()
            .unwrap_or_default();
        let before_overrides = overrides.clone();
        // Os que carregam REFERÊNCIA e por isso não se decidem por bytes — ver abaixo.
        let mut pending: Vec<(u64, Entity, Option<Vec<u8>>)> = Vec::new();

        for pair in &live.pairs {
            let (inst, master) = (pair.inst, pair.master);
            let is_root = inst == live.root;
            for entry in registry.iter() {
                let name = entry.canonical_name;
                if NEVER_PROPAGATES.contains(&name)
                    || (is_root && ROOT_IS_ITS_OWN.contains(&name))
                    || entry.desc.is_some_and(|d| d.owned_document)
                {
                    continue;
                }
                // ⚠️ A pose de quem o runtime possui não sincroniza **nem vira override** — a
                // condição (b) da refutação 1, nas duas metades.
                if name == TRANSFORM && !bridge.document_owns_pose(sim.world(), inst) {
                    continue;
                }
                let key = ph2d_ecs::OverrideKey {
                    piece: pair.master_id,
                    type_id: entry.type_id,
                };
                let want = (entry.serialize)(sim.world(), master).unwrap_or_default();
                let echo_key = (pair.master_id, entry.type_id);
                let master_moved = echo.master.get(&echo_key).is_some_and(|p| *p != want);
                // Regista o eco novo (uma vez por peça do mestre — instâncias irmãs repetem-no).
                next_master.entry(echo_key).or_insert_with(|| want.clone());

                if overrides.overrides.contains(&key) {
                    continue; // (1) a instância possui este componente
                }
                let have = (entry.serialize)(sim.world(), inst).unwrap_or_default();

                // ⚠️⚠️ **Para quem carrega REFERÊNCIA, a comparação por bytes não é decidível
                // aqui** — e isto foi medido, não previsto: a junta do mestre nomeia os corpos do
                // mestre e a da instância nomeia os dela, **de propósito**. Comparar os bytes dá
                // *«diferente»* para sempre, e o passe reescrevia a junta todo o quadro.
                //
                // ⛔⛔ **E por isso eles PROPAGAM mas nunca CAPTURAM override** — medido, não
                // suposto: o solver **escreve dentro do `PhysicsJoint`** (ele semeia `local_a`/
                // `local_b` e vira o `anchored` no 1.º reconcile), e do lado de fora isso é
                // indistinguível de o artista ter mexido na junta. A 1.ª versão capturava, e
                // **toda instância com uma junta ganhava um override no primeiro tique** —
                // ficando surda à receita para sempre.
                //
                // ⚠️ É a família do `pose_owner` um nível acima: *o runtime escreve mais do que a
                // pose de um corpo*. A porta da ponte responde por um corpo, e não por um campo
                // derivado dentro de um componente de config (ADR-0131 diz *«config, nunca estado
                // vivo»*, e o `anchored` é a excepção que esta wave encontrou).
                //
                // ⇒ **DECLARADO:** editar a junta de uma instância vale até o mestre mexer na
                // dele. São dois tipos hoje (`PhysicsJoint`, `PulleyWheel`).
                if crate::instance_refs::carries_object_ref(name) {
                    if !master_moved && echo.master.contains_key(&echo_key) {
                        continue;
                    }
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
                // ⚠️ **Sem eco não há atribuição** — é o 1.º passe, ou o 1.º depois de um load.
                // Aí o mestre ganha: inventar um override a partir de um estado que ninguém viu
                // mudar seria congelar contra a receita algo que o artista nunca pediu.
                if !master_moved && echo.master.contains_key(&echo_key) {
                    overrides.overrides.insert(key); // (3)
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
        let entities: Vec<Entity> = live.pairs.iter().map(|p| p.inst).collect();
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

        // O conjunto de overrides é AUTORIA: escrito só quando muda, para que o undo registe o
        // gesto *«esta instância passou a ter uma excepção»* e mais nada.
        if overrides != before_overrides {
            sim.world_mut().entity_mut(live.root).insert(overrides);
            wrote += 1;
        }
    }
    // ⚠️ Só as chaves que este passe visitou sobrevivem — um mestre apagado leva o eco dele.
    echo.master = next_master;
    wrote
}

/// ⭐ **DEVOLVER a peça ao mestre** — o inverso do override (ADR-0164 / F4.4).
///
/// Tira a chave do conjunto; o passe seguinte volta a propagar o valor da receita. Devolve `true`
/// se havia o que devolver.
///
/// ⚠️ **Ele não escreve o valor do mestre aqui** — quem propaga é o sync, e escrever também neste
/// sítio seria a segunda porta que discorda da primeira no dia em que a regra do `pose_owner`
/// mudar. *Um verbo, um efeito.*
///
/// ⚠️⚠️ **Mas tirar a chave NÃO chega, e foi um gate que o disse:** no passe seguinte a peça ainda
/// difere do mestre e o mestre não mexeu — que é exatamente a assinatura de *«a instância
/// mexeu-se»*. O override renascia no quadro a seguir ao revert, e o verbo era um **no-op
/// visível**.
///
/// ⇒ o revert também **apaga o eco daquela chave**. Sem memória, o passe cai na regra do 1.º
/// encontro — *o mestre ganha* — que já estava escrita e justificada. *A saída não precisou de uma
/// regra nova: precisou de esquecer.*
pub(crate) fn revert_override(
    sim: &mut SimWorld,
    echo: &mut MasterEcho,
    instance_root: Entity,
    key: ph2d_ecs::OverrideKey,
) -> bool {
    let Some(mut ov) = sim.world().get::<ObjectInstance>(instance_root).cloned() else {
        return false;
    };
    if !ov.overrides.remove(&key) {
        return false;
    }
    echo.master.remove(&(key.piece, key.type_id));
    sim.world_mut().entity_mut(instance_root).insert(ov);
    true
}

/// ⭐ **Devolve TODA a instância à receita** — o verbo que o menu da Hierarquia chama.
///
/// `None` quando a entidade não é a raiz de uma instância (o menu é plano e o item aparece em toda
/// linha — ver o dreno); `Some(n)` com quantas excepções foram apagadas.
///
/// ⚠️ **A pergunta é «é a RAIZ de uma instância?»**, e a resposta é o `ObjectInstance`: as peças
/// não o têm, e uma instância sem excepção nenhuma tem-no vazio ou não o tem — os dois casos
/// respondem `Some(0)`, que é *«nada a devolver»* e não *«não se aplica»*.
pub(crate) fn revert_all_overrides(
    sim: &mut SimWorld,
    echo: &mut MasterEcho,
    instance_root: Entity,
) -> Option<usize> {
    // A raiz de uma instância é a peça cujo mestre é um `MasterRoot` — a mesma pergunta do passe.
    let link = sim.world().get::<InstanceOf>(instance_root).copied()?;
    let master = {
        let mut q = sim.world_mut().query::<(Entity, &StableId)>();
        q.iter(sim.world())
            .find(|(_, s)| s.0 == link.master)
            .map(|(e, _)| e)?
    };
    sim.world().get::<MasterRoot>(master)?;

    let keys: Vec<ph2d_ecs::OverrideKey> = sim
        .world()
        .get::<ObjectInstance>(instance_root)
        .map(|o| o.overrides.iter().copied().collect())
        .unwrap_or_default();
    let mut n = 0;
    for key in keys {
        if revert_override(sim, echo, instance_root, key) {
            n += 1;
        }
    }
    Some(n)
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
        // Três campos disjuntos do `AppGfx` (+ o eco, que é do `App`) — sem clonar nada.
        sync_instances(
            &mut gfx.sim,
            &gfx.component_registry,
            &gfx.physics,
            &mut self.instance_echo,
        );
    }
}

#[cfg(test)]
#[path = "instance_sync_tests.rs"]
mod tests;
