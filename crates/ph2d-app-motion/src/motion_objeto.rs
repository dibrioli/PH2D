//! ⭐⭐⭐ **O MOTION COMO OBJECTO** (doc ObjectsAndComponents 01 §13, F5; escolha do dono 9.3: *«Motion
//! vira objecto na árvore, com o que cria como filhos; pode haver vários»*).
//!
//! O objecto é uma entidade com o marcador [`ph2d_ecs::MotionGenerator`]; os nós dele são os do
//! `MotionDoc` cujo dono é o `StableId` dele (`ph2d_motion_doc::owner`). Este módulo é a manutenção
//! por quadro que mantém as duas metades de acordo, no molde da Animação da onda 2
//! (`ph2d-timeline-persist::upkeep`):
//!
//! | caso | lei |
//! |---|---|
//! | nó sem dono, ou dono que NUNCA viveu no mundo | **migração**: o objecto em edição (ou o 1.º vivo) adopta-o; sem nenhum, nasce um «Motion» |
//! | dono que estava vivo e morreu | os nós dele saem num passo do undo do Motion — ⛔ nunca com o relógio a andar (o jogo destrói objectos) |
//! | cópia do editor de um objecto Motion | a cópia ganha um grafo PRÓPRIO (`copy_owner`) num passo do undo |
//! | selecção | o dono em edição segue-a: o objecto Motion seleccionado, ou o 1.º ancestral que o seja |
//!
//! ⚠️ O dono em edição NÃO salta para outro quando a selecção sai de um Motion: o artista pode
//! seleccionar o molde (filho) ou outro objecto sem perder o grafo que está a editar.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, MotionGenerator, StableId, World};

use crate::motion_state::MotionState;

/// ⭐ Motion ▸ Edit — a família de modos.
#[path = "motion_objeto_modo.rs"]
pub mod modo;
/// ⭐ Add ▸ Motion.
#[path = "motion_object_add.rs"]
pub mod object_add;
/// ⭐ A origem: a afim do dono aplicada ao que os sinks dele geram.
#[path = "motion_objeto_origem.rs"]
pub mod origem;

#[path = "motion_objeto_entradas.rs"]
pub mod entradas;
/// `PH2D_MOTION_OBJETO_SMOKE=1|2` — a cena do smoke do dono.
#[path = "motion_objeto_smoke.rs"]
pub mod smoke;
/// ⭐ O editor de nós mostra o grafo do objecto em edição.
#[cfg(feature = "panel-motion-graph")]
#[path = "motion_objeto_view.rs"]
pub mod view;

/// Uma afim 2D `[a, b, c, d, tx, ty]` (colunas `(a, b)` e `(c, d)`, translação `(tx, ty)`) — a
/// forma do `GlobalTransform::affine`.
pub type Afim = [f32; 6];

/// A identidade.
pub const IDENTIDADE: Afim = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Um objecto Motion vivo neste quadro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Dono {
    /// Os bits da entidade.
    pub entity: u64,
    /// O olho (a cadeia de pais incluída): escondido, os sinks dele não cozinham nem desenham.
    pub visible: bool,
    /// Local → mundo: a origem do que os sinks dele geram.
    pub afim: Afim,
}

/// O estado do Motion-como-objecto, guardado no `MotionState`.
#[derive(Debug, Default)]
pub struct ObjetoState {
    /// ⭐ O objecto Motion cujo grafo o editor mostra, e a quem os nós novos pertencem (`StableId`).
    pub editing: Option<u64>,
    /// Os objectos Motion vivos neste quadro, por `StableId`.
    pub donos: BTreeMap<u64, Dono>,
    /// Os vivos do quadro anterior — só quem estava vivo pode MORRER (a purga).
    vivos_antes: BTreeSet<u64>,
    /// ⭐ A origem aplicada às cópias (ver [`origem`]).
    pub origem: origem::Origem,
    /// As ligações `(dono, nome do molde)` já vistas — o molde vira filho só no MOMENTO da ligação.
    ligados: BTreeSet<(u64, String)>,
    /// O 1.º quadro semeia as ligações sem mexer na árvore (o artista pode ter arrastado um molde
    /// para fora, e abrir o projecto não o pode desfazer).
    semeado: bool,
    /// ⭐ As covas das purgas ([`ph2d_ecs::graveyard`]): o undo global que devolve o objecto devolve
    /// os nós dele no mesmo quadro.
    pub cemiterio: ph2d_ecs::Graveyard<ph2d_motion_doc::MotionDoc>,
}

impl ObjetoState {
    /// Há algum objecto Motion visível? Então o Motion cozinha e desenha mesmo sem a ferramenta.
    #[must_use]
    pub fn desenha(&self) -> bool {
        self.donos.values().any(|d| d.visible)
    }

    /// O dono de um nó está escondido? (Um nó sem dono, ou de um dono que não está vivo, não está.)
    #[must_use]
    pub fn escondido(&self, owner: Option<u64>) -> bool {
        owner
            .and_then(|o| self.donos.get(&o))
            .is_some_and(|d| !d.visible)
    }
}

/// O que a manutenção fez neste quadro (para a shell marcar o título sujo e os testes medirem).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manutencao {
    /// O objecto Motion que nasceu para adoptar nós órfãos.
    pub nasceu: Option<u64>,
    /// Os donos cujos nós saíram (o objecto morreu).
    pub purgados: Vec<u64>,
    /// `(original, cópia)` cujos grafos foram copiados.
    pub copiados: Vec<(u64, u64)>,
    /// O undo global devolveu um objecto, e o cemitério devolveu os nós dele.
    pub ressuscitados: bool,
}

/// ⭐ **A manutenção por quadro** — antes do `dispatch`, com o mundo na mão.
///
/// `selection` são os bits do objecto seleccionado; `playing` é o relógio a andar (o Play).
pub fn upkeep(
    motion: &mut MotionState,
    world: &mut World,
    selection: Option<u64>,
    playing: bool,
) -> Manutencao {
    let mut feito = Manutencao::default();
    ph2d_ecs::assign_missing_stable_ids(world);
    motion.objeto.donos = vivos(world);
    copias(motion, world, &mut feito);
    ressuscita(motion, &mut feito);
    mortes(motion, playing, &mut feito);
    let migrou = orfaos(motion, world, playing, &mut feito);
    if feito.nasceu.is_some() {
        motion.objeto.donos = vivos(world);
    }
    motion.objeto.vivos_antes = motion.objeto.donos.keys().copied().collect();
    if !playing {
        liga_moldes(motion, world, migrou);
    }
    ph2d_ecs::assign_motion_templates(world);
    segue_a_seleccao(motion, world, selection);
    feito
}

/// ⭐ **A forma que o Motion copia vira FILHA dele** (escolha do dono, 08/10) — no momento em que um
/// `source.object` do objecto passa a nomeá-la, ou na migração. Uma vez: arrastá-la para fora depois
/// é escolha do artista, e não é desfeita. Um nome com dois donos possíveis (dois objectos com o mesmo
/// nome) não liga nada — escolher um seria adivinhar.
fn liga_moldes(motion: &mut MotionState, world: &mut World, migrou: bool) {
    let tipo = ph2d_node_source_object::MANIFEST.name;
    let pares: BTreeSet<(u64, String)> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == tipo)
        .filter_map(|n| {
            let dono = motion.doc.owner_of(n.id)?;
            let nome = motion
                .doc
                .graph
                .node_text_param_overrides(n.id)?
                .get(ph2d_node_source_object::OBJECT_PARAM)?
                .trim()
                .to_string();
            (!nome.is_empty()).then_some((dono, nome))
        })
        .collect();
    let semear = !motion.objeto.semeado && !migrou;
    motion.objeto.semeado = true;
    if semear {
        motion.objeto.ligados = pares;
        return;
    }
    let novos: Vec<(u64, String)> = pares.difference(&motion.objeto.ligados).cloned().collect();
    for (dono, nome) in &novos {
        if let Some(d) = motion.objeto.donos.get(dono) {
            adopta_molde(world, d.entity, nome);
        }
    }
    motion.objeto.ligados = pares;
}

/// Põe o objecto chamado `nome` sob `dono` (se é UM só, não é o dono nem um antepassado dele).
fn adopta_molde(world: &mut World, dono: u64, nome: &str) {
    let Some(pai) = Entity::try_from_bits(dono) else {
        return;
    };
    let mut q = world.query::<(Entity, &ph2d_ecs::Name)>();
    let achados: Vec<Entity> = q
        .iter(world)
        .filter(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .collect();
    let [molde] = achados[..] else {
        return;
    };
    let mut cur = Some(pai);
    while let Some(e) = cur {
        if e == molde {
            return;
        }
        cur = world.get::<ph2d_ecs::ChildOf>(e).map(|c| c.parent());
    }
    if world.get::<ph2d_ecs::ChildOf>(molde).map(|c| c.parent()) != Some(pai) {
        world.entity_mut(molde).insert(ph2d_ecs::ChildOf(pai));
    }
}

/// Os objectos Motion vivos, com o olho e a afim de cada um.
fn vivos(world: &mut World) -> BTreeMap<u64, Dono> {
    let mut q = world.query::<(Entity, &StableId, &MotionGenerator)>();
    let ids: Vec<(Entity, u64)> = q
        .iter(world)
        .filter(|(_, s, _)| !s.is_none())
        .map(|(e, s, _)| (e, s.0))
        .collect();
    ids.into_iter()
        .map(|(e, sid)| {
            let t = pose_de(world, e);
            (
                sid,
                Dono {
                    entity: e.to_bits(),
                    visible: !ph2d_ecs::visibility::is_hidden_in_tree(world, e),
                    afim: ph2d_ecs::GlobalTransform::from_transform(t).affine(),
                },
            )
        })
        .collect()
}

/// ⭐ Local → mundo de `entity`: os `Transform` da cadeia de pais compostos de cima para baixo (a
/// mesma composição da propagação, `Transform::compose`).
#[must_use]
pub fn afim_de(world: &World, entity: Entity) -> Afim {
    ph2d_ecs::GlobalTransform::from_transform(pose_de(world, entity)).affine()
}

/// A pose composta (local → mundo) de `entity`.
fn pose_de(world: &World, entity: Entity) -> ph2d_ecs::Transform {
    let mut cadeia = Vec::new();
    let mut cur = Some(entity);
    while let Some(e) = cur {
        if cadeia.len() > 256 {
            break;
        }
        if let Some(t) = world.get::<ph2d_ecs::Transform>(e) {
            cadeia.push(*t);
        }
        cur = world.get::<ph2d_ecs::ChildOf>(e).map(|c| c.parent());
    }
    let Some(raiz) = cadeia.pop() else {
        return ph2d_ecs::Transform::IDENTITY;
    };
    cadeia
        .into_iter()
        .rev()
        .fold(raiz, ph2d_ecs::Transform::compose)
}

/// ⭐ A afim do objecto Motion em EDIÇÃO (identidade sem ele) — o espaço em que os gizmos do editor
/// de nós desenham e arrastam.
#[must_use]
pub fn afim_em_edicao(motion: &MotionState) -> Afim {
    motion
        .objeto
        .editing
        .and_then(|s| motion.objeto.donos.get(&s))
        .map_or(IDENTIDADE, |d| d.afim)
}

/// ⭐ A câmara da cena vista do espaço do objecto Motion em edição — a dos gizmos do Motion (warp,
/// pivô, colisor, pontos): o que eles desenham em coordenadas do grafo cai onde o objecto o pôs.
#[must_use]
pub fn camera_do_dono(
    camera: &ph2d_render::Camera2d,
    motion: &MotionState,
) -> ph2d_render::Camera2d {
    camera.with_model(afim_em_edicao(motion))
}

/// As cópias do editor de um objecto Motion levam um grafo próprio (um passo do undo do Motion).
fn copias(motion: &mut MotionState, world: &mut World, feito: &mut Manutencao) {
    let pares = ph2d_ecs::take_copies(world, ph2d_ecs::CopyReader::Motion);
    if pares.is_empty() {
        return;
    }
    let sid_de = |bits: u64| -> Option<u64> {
        let e = Entity::try_from_bits(bits)?;
        world.get::<MotionGenerator>(e)?;
        world
            .get::<StableId>(e)
            .filter(|s| !s.is_none())
            .map(|s| s.0)
    };
    let pares: Vec<(u64, u64)> = pares
        .into_iter()
        .filter_map(|(o, c)| Some((sid_de(o)?, sid_de(c)?)))
        .filter(|(o, _)| motion.doc.owner_ids().contains(o))
        .collect();
    if pares.is_empty() {
        return;
    }
    let pre = motion.doc.clone();
    for (o, c) in pares {
        motion.doc.copy_owner(o, c);
        feito.copiados.push((o, c));
    }
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

/// ⭐ O Ctrl+Z da ferramenta Motion pertence ao undo GLOBAL? Sim quando o topo da fila do Motion é a
/// purga de um objecto apagado, enterrada à espera dele ([`ph2d_ecs::Graveyard::holds`]): desfazê-la
/// ali daria os nós a outro objecto (a migração), e o objecto nunca voltaria.
#[must_use]
pub fn undo_e_do_global(motion: &MotionState) -> bool {
    motion
        .history
        .top()
        .is_some_and(|t| motion.objeto.cemiterio.holds(t))
}

/// ⭐ O objecto que o undo global devolveu traz os nós dele de volta, no mesmo Ctrl+Z (o cemitério).
fn ressuscita(motion: &mut MotionState, feito: &mut Manutencao) {
    if motion.objeto.cemiterio.is_empty() {
        return;
    }
    let (donos, history, doc) = (&motion.objeto.donos, &mut motion.history, &mut motion.doc);
    let abriu = motion.objeto.cemiterio.rise(
        |sid| donos.contains_key(&sid),
        |antes| {
            let Some(d) = history.take_top_if(&antes) else {
                return false;
            };
            *doc = d;
            true
        },
    );
    if abriu {
        feito.ressuscitados = true;
        motion.pump.mark_dirty();
    }
}

/// ⛔ O objecto que estava vivo e morreu leva os nós dele — nunca com o relógio a andar.
fn mortes(motion: &mut MotionState, playing: bool, feito: &mut Manutencao) {
    if playing {
        return;
    }
    let mortos: Vec<u64> = motion
        .objeto
        .vivos_antes
        .iter()
        .copied()
        .filter(|s| !motion.objeto.donos.contains_key(s))
        .filter(|s| motion.doc.owner_ids().contains(s))
        .collect();
    if mortos.is_empty() {
        return;
    }
    let pre = motion.doc.clone();
    motion.objeto.cemiterio.bury(mortos.clone(), pre.clone());
    for s in mortos {
        motion.doc.remove_owner(s);
        feito.purgados.push(s);
        if motion.objeto.editing == Some(s) {
            motion.objeto.editing = None;
        }
    }
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

/// ⭐ **A migração**: nó sem dono, ou de um dono que nunca viveu, vai para o objecto em edição (ou o
/// 1.º vivo); sem nenhum, nasce um objecto «Motion». O `MotionDoc` global de antes do F5 vira UM
/// objecto — zero perda.
fn orfaos(
    motion: &mut MotionState,
    world: &mut World,
    playing: bool,
    feito: &mut Manutencao,
) -> bool {
    let fantasmas: Vec<u64> = motion
        .doc
        .owner_ids()
        .into_iter()
        .filter(|s| !motion.objeto.donos.contains_key(s) && !motion.objeto.vivos_antes.contains(s))
        .collect();
    let sem_dono = !motion.doc.unowned().is_empty()
        || motion
            .doc
            .backdrops
            .iter()
            .any(|b| !motion.doc.backdrop_owners.contains_key(&b.id));
    if (fantasmas.is_empty() && !sem_dono) || playing {
        return false;
    }
    let alvo = motion
        .objeto
        .editing
        .filter(|s| motion.objeto.donos.contains_key(s))
        .or_else(|| motion.objeto.donos.keys().next().copied())
        .or_else(|| {
            let sid = nasce(world);
            feito.nasceu = sid;
            sid
        });
    let Some(alvo) = alvo else {
        return false;
    };
    for f in fantasmas {
        motion.doc.rehome(f, alvo);
    }
    motion.doc.adopt_unowned(alvo);
    if motion.objeto.editing.is_none() {
        motion.objeto.editing = Some(alvo);
    }
    true
}

/// Nasce um objecto Motion na origem do mundo (a do grafo de antes: zero deslocamento).
fn nasce(world: &mut World) -> Option<u64> {
    let e = world
        .spawn((
            ph2d_ecs::Name::new(ph2d_i18n::tr("object_add.motion")),
            ph2d_ecs::Transform::IDENTITY,
            MotionGenerator,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(world);
    let sid = world.get::<StableId>(e).map(|s| s.0)?;
    Some(sid)
}

/// O dono em edição segue a selecção: o objecto Motion seleccionado, ou o 1.º ancestral que o seja.
/// Sem nenhum na cadeia, fica o de antes (se vivo), senão o 1.º vivo.
fn segue_a_seleccao(motion: &mut MotionState, world: &World, selection: Option<u64>) {
    if let Some(sid) = selection.and_then(|b| motion_na_cadeia(world, b)) {
        motion.objeto.editing = Some(sid);
        return;
    }
    let vivo = motion
        .objeto
        .editing
        .is_some_and(|s| motion.objeto.donos.contains_key(&s));
    if !vivo {
        motion.objeto.editing = motion.objeto.donos.keys().next().copied();
    }
}

/// O `StableId` do objecto Motion que é `bits` ou o 1.º ancestral dele que o seja.
#[must_use]
pub fn motion_na_cadeia(world: &World, bits: u64) -> Option<u64> {
    let mut cur = Entity::try_from_bits(bits);
    let mut passos = 0;
    while let Some(e) = cur {
        if world.get::<MotionGenerator>(e).is_some() {
            return world
                .get::<StableId>(e)
                .filter(|s| !s.is_none())
                .map(|s| s.0);
        }
        passos += 1;
        if passos > 256 {
            return None;
        }
        cur = world.get::<ph2d_ecs::ChildOf>(e).map(|c| c.parent());
    }
    None
}

/// ⭐ **Os sinks que cozinham**: os de todos os objectos visíveis (o olho esconde os do dono).
pub fn sinks_visiveis(
    motion: &MotionState,
    sinks: Vec<ph2d_nodegraph::graph::NodeId>,
) -> Vec<ph2d_nodegraph::graph::NodeId> {
    sinks
        .into_iter()
        .filter(|s| !motion.objeto.escondido(motion.doc.owner_of(*s)))
        .collect()
}

#[cfg(test)]
#[path = "motion_objeto_tests.rs"]
mod tests;
