//! ⭐⭐ **O ESTADO DO RENDER POR MALHA** — quando extrair, o que mostrar enquanto extrai, e com que
//! pose cada objeto se desenha. Corre no `ecs_bridge` (o sítio do quadro que tem o mundo).
//!
//! # As três regras
//!
//! 1. **A FORMA mudou** (a chave de [`crate::malha_render::Entrada::chave`]) ⇒ extrai de novo, noutra
//!    thread; o quadro continua a mostrar os objetos de antes até a nova chegar.
//! 2. **Só as POSES mudaram** ⇒ nada se extrai durante o gesto: cada objeto desenha-se com
//!    `pose_agora ∘ pose_extraída⁻¹` — mover é custo ZERO, como num jogo.
//! 3. **Acabou o gesto e as poses não são as verificadas** ⇒ uma extração de VERIFICAÇÃO: se ela
//!    partir a peça de outra maneira (dois objetos passaram a tocar-se, ou o que se move junto deixou
//!    de se mover junto), troca; senão fica com as malhas que já estão na placa (nada pisca).
//!
//! ⚠️ O estado vive aqui, numa `thread_local` da thread do modelador, e não no `Smoke`: o
//! `smoke_state.rs` está no tecto de linhas, e nada aqui é estado do documento.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use bevy_ecs::entity::Entity;
use ph2d_field::{FieldDoc, Xform};

use crate::malha_render::{Entrada, ObjetoRender};

type Chave = Vec<(Entity, FieldDoc)>;

/// ⭐ O que o quadro desenha.
#[derive(Default)]
pub struct Estado {
    /// Sobe a cada extração adoptada — o desenhista re-sobe as malhas quando ela muda.
    pub geracao: u64,
    pub objetos: Arc<Vec<ObjetoRender>>,
    chave: Option<Chave>,
    /// As poses (por unidade) que a última extração adoptada ou verificada viu.
    verificadas: BTreeMap<Entity, Xform>,
    em_voo: Option<(Chave, BTreeMap<Entity, Xform>, Receiver<Vec<ObjetoRender>>)>,
    /// A matriz de modelo de cada objeto, na ordem de `objetos` (coluna a coluna).
    pub modelos: Vec<[[f32; 4]; 4]>,
    /// A raiz da peça destes objetos. ⚠️ Outra raiz (outro documento) recomeça o estado NA HORA:
    /// medido (02/10, a sonda de duas cenas seguidas), sem isto a 2.ª peça mostrava as malhas da 1.ª
    /// até a extração dela chegar.
    raiz: Option<Entity>,
}

thread_local! {
    static ESTADO: RefCell<Option<Estado>> = const { RefCell::new(None) };
}

/// ⭐ **O Render por malha está ligado?** `PH2D_FIELD_RENDER_TRACADO=1` volta ao traçado (bissecção).
#[must_use]
pub fn ligado() -> bool {
    std::env::var("PH2D_FIELD_RENDER_TRACADO").map_or(true, |v| v != "1")
}

/// Lê o estado (se houver).
pub fn com<R>(f: impl FnOnce(&Estado) -> R) -> Option<R> {
    ESTADO.with(|c| c.borrow().as_ref().map(f))
}

/// A matriz coluna a coluna de uma pose (escala uniforme).
#[must_use]
pub fn matriz(x: Xform) -> [[f32; 4]; 4] {
    let c = |d: [f32; 3]| x.apply_dir(d);
    let (a, b, k) = (c([1.0, 0.0, 0.0]), c([0.0, 1.0, 0.0]), c([0.0, 0.0, 1.0]));
    let t = x.translation;
    [
        [a[0], a[1], a[2], 0.0],
        [b[0], b[1], b[2], 0.0],
        [k[0], k[1], k[2], 0.0],
        [t[0], t[1], t[2], 1.0],
    ]
}

fn perto(a: Xform, b: Xform) -> bool {
    let d = |u: &[f32], v: &[f32]| u.iter().zip(v).all(|(x, y)| (x - y).abs() <= 1.0e-6);
    d(&a.translation, &b.translation)
        && (d(&a.rotation, &b.rotation) || d(&a.rotation, &b.rotation.map(|q| -q)))
        && (a.scale - b.scale).abs() <= 1.0e-6
}

/// As unidades de cada objeto, ordenadas — a PARTIÇÃO da peça.
fn particao(objs: &[ObjetoRender]) -> Vec<Vec<Entity>> {
    let mut p: Vec<Vec<Entity>> = objs
        .iter()
        .map(|o| {
            let mut u = o.unidades.clone();
            u.sort_unstable();
            u
        })
        .collect();
    p.sort_unstable();
    p
}

/// ⭐⭐ **O sincronismo do quadro.** `em_render` = algum viewport está no Render; `gesto` = um
/// arrasto mexeu nas poses neste quadro.
pub fn sync(sim: &mut ph2d_ecs::SimWorld, em_render: bool, gesto: bool) {
    if !em_render || !ligado() {
        ESTADO.with(|c| *c.borrow_mut() = None);
        return;
    }
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let Some(root) = q.iter(world).next().map(|(e, _)| e) else {
        ESTADO.with(|c| *c.borrow_mut() = None);
        return;
    };
    let entrada = crate::malha_render::colhe(sim.world(), root);
    let chave = entrada.chave();
    let poses: BTreeMap<Entity, Xform> = entrada
        .postos
        .iter()
        .map(|(e, _)| *e)
        .zip(entrada.poses.iter().copied())
        .collect();
    ESTADO.with(|c| {
        let mut slot = c.borrow_mut();
        let st = slot.get_or_insert_with(Estado::default);
        if st.raiz != Some(root) {
            *st = Estado {
                geracao: st.geracao,
                raiz: Some(root),
                ..Estado::default()
            };
        }
        st.colhe_o_voo(&chave);
        let mesmas = st.verificadas.len() == poses.len()
            && st
                .verificadas
                .iter()
                .all(|(e, x)| poses.get(e).is_some_and(|y| perto(*x, *y)));
        let precisa = st.chave.as_ref() != Some(&chave) || (!mesmas && !gesto);
        if precisa && st.em_voo.is_none() {
            st.lanca(entrada, chave, poses.clone());
        }
        st.modelos = st
            .objetos
            .iter()
            .map(|o| {
                let agora = poses.get(&o.unidades[0]).copied().unwrap_or(o.pose_extraida);
                matriz(crate::malha_render::delta(agora, o.pose_extraida))
            })
            .collect();
    });
}

impl Estado {
    /// Ainda não há objetos e a extração deles está a caminho: o quadro espera (mostra o anterior).
    #[must_use]
    pub fn esperando(&self) -> bool {
        self.geracao == 0 || (self.objetos.is_empty() && self.em_voo.is_some())
    }

    fn lanca(&mut self, entrada: Entrada, chave: Chave, poses: BTreeMap<Entity, Xform>) {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let objs = crate::malha_render::processa(&entrada, &crate::smoke::sampled_registry());
            let _ = tx.send(objs);
        });
        self.em_voo = Some((chave, poses, rx));
    }

    /// Recolhe a extração em voo, se chegou, e decide se a adopta.
    fn colhe_o_voo(&mut self, chave_agora: &Chave) {
        let Some((chave, poses, rx)) = &self.em_voo else {
            return;
        };
        let Ok(objs) = rx.try_recv() else {
            if matches!(rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)) {
                self.em_voo = None;
            }
            return;
        };
        let (chave, poses) = (chave.clone(), poses.clone());
        self.em_voo = None;
        if &chave != chave_agora {
            // Uma forma mais nova já existe: esta resposta é velha, e a próxima será lançada.
            return;
        }
        let forma_nova = self.chave.as_ref() != Some(&chave);
        if forma_nova || particao(&objs) != particao(&self.objetos) || self.algum_se_partiu(&poses) {
            self.objetos = Arc::new(objs);
            self.geracao += 1;
            self.chave = Some(chave);
        }
        self.verificadas = poses;
    }

    /// Um objeto com várias unidades que deixaram de se mover JUNTAS desde a verificação (uma delas
    /// foi movida sozinha, pela Hierarquia): a malha dele já não é a forma.
    fn algum_se_partiu(&self, poses: &BTreeMap<Entity, Xform>) -> bool {
        let movimento = |u: &Entity| match (poses.get(u), self.verificadas.get(u)) {
            (Some(a), Some(v)) => Some(crate::malha_render::delta(*a, *v)),
            _ => None,
        };
        self.objetos.iter().any(|o| {
            let Some(m0) = movimento(&o.unidades[0]) else {
                return true;
            };
            o.unidades
                .iter()
                .skip(1)
                .any(|u| movimento(u).is_none_or(|m| !perto(m, m0)))
        })
    }
}

#[cfg(test)]
#[path = "malha_render_estado_tests.rs"]
mod tests;

/// ⭐ **O objeto do Render que contém `e`** — `e` é uma unidade ou está dentro de uma. Devolve as
/// unidades dele e se ele é móvel.
#[must_use]
pub fn objeto_de(world: &bevy_ecs::world::World, e: Entity) -> Option<(Vec<Entity>, bool)> {
    com(|st| {
        let mut cur = Some(e);
        while let Some(c) = cur {
            if let Some(o) = st.objetos.iter().find(|o| o.unidades.contains(&c)) {
                return Some((o.unidades.clone(), o.movel));
            }
            cur = world.get::<bevy_ecs::hierarchy::ChildOf>(c).map(|p| p.0);
        }
        None
    })
    .flatten()
}

/// ⭐ **No Render, um clique escolhe o OBJETO inteiro** — todas as unidades dele.
#[must_use]
pub fn selecao_por_objeto(
    world: &bevy_ecs::world::World,
    req: crate::scene::SelectRequest,
    atual: &[Entity],
) -> crate::scene::SelectRequest {
    use crate::scene::SelectRequest as R;
    let bits = |v: Vec<Entity>| v.into_iter().map(Entity::to_bits).collect::<Vec<u64>>();
    let todas = |all: &[u64]| {
        let mut out: Vec<Entity> = Vec::new();
        for b in all {
            let e = Entity::from_bits(*b);
            for u in objeto_de(world, e).map_or_else(|| vec![e], |(us, _)| us) {
                if !out.contains(&u) {
                    out.push(u);
                }
            }
        }
        out
    };
    match req {
        R::Entity(b) => objeto_de(world, Entity::from_bits(b)).map_or(R::Entity(b), |(us, _)| R::Many(bits(us))),
        R::Toggle(b) => match objeto_de(world, Entity::from_bits(b)) {
            Some((us, _)) if us.iter().all(|u| atual.contains(u)) => R::RemoveMany(bits(us)),
            Some((us, _)) => R::AddMany(bits(us)),
            None => R::Toggle(b),
        },
        R::AddMany(all) => R::AddMany(bits(todas(&all))),
        R::RemoveMany(all) => R::RemoveMany(bits(todas(&all))),
        outro => outro,
    }
}

/// ⭐ **A seleção pode mover-se no Render?** `None` = pode; `Some(chave)` = não, e a frase diz
/// porquê. Pode quando ela é feita de objetos INTEIROS e MÓVEIS.
#[must_use]
pub fn trava(world: &bevy_ecs::world::World, sel: &[Entity]) -> Option<&'static str> {
    if sel.is_empty() || com(|st| st.geracao).unwrap_or(0) == 0 {
        return None;
    }
    for e in sel {
        let Some((us, movel)) = objeto_de(world, *e) else {
            // Uma luz, ou algo fora da peça: o gizmo dele não é deste assunto.
            continue;
        };
        if !us.contains(e) || !us.iter().all(|u| sel.contains(u)) {
            return Some("app.field3d.malha_render.whole_objects_only");
        }
        if !movel {
            return Some("app.field3d.malha_render.part_of_a_cut");
        }
    }
    None
}

#[cfg(test)]
#[path = "malha_render_costura_tests.rs"]
mod costura_tests;

#[cfg(test)]
#[path = "malha_render_sondas.rs"]
mod sondas;
