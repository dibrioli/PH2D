//! **A reprodução da família «mundo»** (`docs/Macros/17_W19_plano.md` §2.4): o documento tem o FIM; enquanto
//! a Timeline conduz o objecto Macro (`MacroTempo`), o mundo mostra `S(t)` por blobs conduzidos
//! ([`ph2d_preview_drive::BlobDrive`]) — vê-se, não entra no undo nem no save.
//!
//! - objecto que em `t` não está vivo → `Visibility` escondida;
//! - componente que a trilha nomeia → o blob de `S(t)` (ou ausente); os outros não se tocam;
//! - um escritor por facto: cada corpo da macro passa a `Kinematic` enquanto toca (a ponte global
//!   trata-o como cena); dentro de um `Simula` a pose vem do rapier privado ([`super::palco_fisica`]).

use super::estado::{Lida, ObjetoEm};
use super::formato::{Evento, Mudanca, Quem, RAIZ};
use super::palco_fisica::Recozido;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, Macro, MacroTempo, SimWorld, StableId, Visibility};
use ph2d_editor_core::macro_edits::ErroDaMacroView;
use ph2d_macro::MacroTime;
use ph2d_physics_ecs::{BodyKind, PhysicsSettings, RigidBody};
use ph2d_preview_drive::{BlobRef, PreviewDrive};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const TRANSFORM: &str = "ph2d::ecs::Transform";
pub(super) const VISIBILIDADE: &str = "ph2d::ecs::Visibility";
pub(super) const CORPO: &str = "ph2d::physics::RigidBody";

/// Uma trilha «mundo» lida, com o que se deriva dela uma vez.
pub(super) struct Trilha {
    pub lida: Lida,
    /// Os componentes que a trilha nomeia, por objecto local (em qualquer instante).
    nomes: BTreeMap<u32, BTreeSet<String>>,
    /// Os instantes dos passos que escrevem o `Transform` de cada objecto.
    poses: BTreeMap<u32, Vec<MacroTime>>,
    /// O rapier privado do `Simula` em curso.
    pub recozido: Option<Recozido>,
    /// A conferência de cada `Simula` (pelo índice) que já chegou ao `ate`: `Some` = deu outro bit.
    pub conferidos: BTreeMap<usize, Option<ErroDaMacroView>>,
}

impl Trilha {
    pub(super) fn de(lida: Lida) -> Self {
        let mut nomes: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
        let mut poses: BTreeMap<u32, Vec<MacroTime>> = BTreeMap::new();
        for l in &lida.inicial.linhas {
            nomes
                .entry(l.id)
                .or_default()
                .extend(l.componentes.iter().map(|c| c.nome.clone()));
        }
        for (t, e) in &lida.eventos {
            let Evento::Passo { mudancas, .. } = e else {
                continue;
            };
            for m in mudancas {
                let (id, ns): (u32, Vec<&str>) = match m {
                    Mudanca::Nasce(l) => (
                        l.id,
                        l.componentes.iter().map(|c| c.nome.as_str()).collect(),
                    ),
                    Mudanca::Muda {
                        id, componentes, ..
                    } => (*id, componentes.iter().map(|c| c.0.as_str()).collect()),
                    Mudanca::Morre(_) => continue,
                };
                if ns.contains(&TRANSFORM) {
                    poses.entry(id).or_default().push(*t);
                }
                nomes
                    .entry(id)
                    .or_default()
                    .extend(ns.into_iter().map(str::to_owned));
            }
        }
        Self {
            lida,
            nomes,
            poses,
            recozido: None,
            conferidos: BTreeMap::new(),
        }
    }

    /// O último passo até `t` que escreveu a pose de `id`.
    pub(super) fn ultima_pose(&self, id: u32, t: MacroTime) -> Option<MacroTime> {
        self.poses
            .get(&id)?
            .iter()
            .copied()
            .filter(|p| *p <= t)
            .max()
    }
}

/// O doc de um objecto Macro e as trilhas «mundo» dele, pelo índice da trilha.
struct DoDono {
    doc: Vec<u8>,
    trilhas: BTreeMap<u32, Trilha>,
}

/// **Os palcos do mundo**, um por `(dono, índice da trilha)`; a leitura fica em cache até os bytes do
/// `Macro` mudarem.
#[derive(Default)]
pub struct PalcosDoMundo {
    donos: BTreeMap<u64, DoDono>,
    /// As definições da física do projecto no último quadro (o aplicar recoze com elas).
    pub definicoes: Option<PhysicsSettings>,
}

thread_local! {
    static PALCOS: RefCell<PalcosDoMundo> = RefCell::new(PalcosDoMundo::default());
}

/// A porta da shell (o estado mora aqui; a shell não ganha campo).
pub fn com<R>(f: impl FnOnce(&mut PalcosDoMundo) -> R) -> R {
    PALCOS.with(|p| f(&mut p.borrow_mut()))
}

/// **O quadro da shell**: [`PalcosDoMundo::segue`] sobre o estado da porta.
pub fn segue(
    sim: &mut SimWorld,
    registo: &ComponentRegistry,
    definicoes: PhysicsSettings,
    drive: &mut PreviewDrive,
) {
    com(|p| p.segue(sim, &Contexto::novo(registo, definicoes), drive));
}

/// **O erro da reprodução do mundo** do objecto Macro `bits` (a shell junta-o aos das outras famílias).
#[must_use]
pub fn erro_de(bits: u64) -> Option<ErroDaMacroView> {
    com(|p| p.erro_de(bits))
}

fn le_as_trilhas(doc: &[u8]) -> BTreeMap<u32, Trilha> {
    let Ok(d) = ph2d_macro::codec::read(doc) else {
        return BTreeMap::new();
    };
    d.tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.family == super::formato::FAMILIA)
        // As edições de tempo das etapas (espera, deslocação) ouvem-se: toca a trilha efectiva.
        .filter_map(|(k, t)| {
            let t = t.tocavel(&crate::etapas_mundo::LeitorDoMundo).ok()?;
            Some((u32::try_from(k).ok()?, Trilha::de(Lida::de(&t).ok()?)))
        })
        .collect()
}

/// O blob de `bytes` (um `RigidBody`) como `Kinematic`.
fn cinematico(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut c: RigidBody = postcard::from_bytes(bytes).ok()?;
    c.kind = BodyKind::Kinematic;
    postcard::to_allocvec(&c).ok()
}

fn escondido(o: Option<&ObjetoEm>) -> Vec<u8> {
    let mut v: Visibility = o
        .and_then(|o| o.componentes.get(VISIBILIDADE))
        .and_then(|b| postcard::from_bytes(b).ok())
        .unwrap_or_default();
    v.hidden = true;
    postcard::to_allocvec(&v).expect("postcard de um bool")
}

/// O que um quadro do palco precisa do mundo de fora.
pub struct Contexto<'a> {
    pub registo: &'a ComponentRegistry,
    pub definicoes: PhysicsSettings,
    /// Um ponto de retoma do rapier privado a cada tantos tiques; `None` = recozer do zero (a
    /// variante C da sonda).
    pub retoma: Option<u64>,
}

impl<'a> Contexto<'a> {
    #[must_use]
    pub fn novo(registo: &'a ComponentRegistry, definicoes: PhysicsSettings) -> Self {
        Self {
            registo,
            definicoes,
            retoma: Some(super::palco_fisica::RETOMA),
        }
    }
}

impl PalcosDoMundo {
    /// **Por quadro**, depois de a Timeline escrever o `MacroTempo` e antes do extract: cada objecto
    /// Macro que toca mostra `S(t)`; os que pararam devolvem o documento.
    pub fn segue(&mut self, sim: &mut SimWorld, cx: &Contexto<'_>, drive: &mut PreviewDrive) {
        self.definicoes = Some(cx.definicoes);
        drive.blobs.ronda();
        let mut q = sim
            .world_mut()
            .query::<(Entity, &Macro, Option<&MacroTempo>)>();
        let vistos: Vec<(Entity, Option<f64>)> = q
            .iter(sim.world())
            .map(|(e, _, t)| (e, t.map(|t| t.segundos)))
            .collect();
        self.donos
            .retain(|b, _| vistos.iter().any(|(e, _)| e.to_bits() == *b));
        if vistos.iter().any(|(_, s)| s.is_some()) {
            let sids: BTreeMap<u64, Entity> = sim
                .world_mut()
                .query::<(Entity, &StableId)>()
                .iter(sim.world())
                .map(|(e, s)| (s.0, e))
                .collect();
            for (dono, s) in vistos {
                let Some(s) = s else { continue };
                let Some(m) = sim.world().get::<Macro>(dono) else {
                    continue;
                };
                let d = self.donos.entry(dono.to_bits()).or_insert_with(|| DoDono {
                    doc: Vec::new(),
                    trilhas: BTreeMap::new(),
                });
                if d.doc != m.doc {
                    d.doc.clone_from(&m.doc);
                    d.trilhas = le_as_trilhas(&d.doc);
                }
                let t = MacroTime::from_secs_f64(s);
                for tr in d.trilhas.values_mut() {
                    conduz(tr, sim, cx, drive, dono, t, &sids);
                }
            }
        }
        drive.blobs.devolve_os_largados(sim);
    }

    /// A 1.ª conferência que falhou nas trilhas do objecto Macro `bits`.
    #[must_use]
    pub fn erro_de(&self, bits: u64) -> Option<ErroDaMacroView> {
        self.donos
            .get(&bits)?
            .trilhas
            .values()
            .flat_map(|t| t.conferidos.values())
            .find_map(Clone::clone)
    }

    /// O palco da trilha `k` de `dono` (para os gates).
    #[must_use]
    pub fn recozido(&self, dono: Entity, k: u32) -> Option<&Recozido> {
        self.donos
            .get(&dono.to_bits())?
            .trilhas
            .get(&k)?
            .recozido
            .as_ref()
    }
}

fn conduz(
    tr: &mut Trilha,
    sim: &mut SimWorld,
    cx: &Contexto<'_>,
    drive: &mut PreviewDrive,
    dono: Entity,
    t: MacroTime,
    sids: &BTreeMap<u64, Entity>,
) {
    let agora = tr.lida.em(t);
    let quem: BTreeMap<u32, (Quem, u64)> = tr
        .lida
        .objetos()
        .map(|o| (o.id, (o.quem, o.original)))
        .collect();
    let entidade = |id: u32| match quem.get(&id) {
        _ if id == RAIZ => Some(dono),
        Some((Quem::Macro, _)) => Some(dono),
        Some((_, sid)) => sids.get(sid).copied(),
        None => None,
    };
    let fisica = super::palco_fisica::poses(tr, sim, cx, dono, t, &entidade);
    let porta = |n: &str| cx.registo.get_by_name(n).map(BlobRef::of);
    for (&id, nomes) in &tr.nomes {
        let Some(e) = entidade(id) else { continue };
        // A pose que o solver global escreveu antes de o corpo ser da macro volta ao autorado.
        drive.release_to_authored(sim, e, ph2d_preview_drive::Driver::SolverPose);
        let o = agora.get(&id);
        let vivo = o.is_some_and(|o| o.vivo);
        if vivo {
            for n in nomes.iter().filter(|n| *n != VISIBILIDADE && *n != CORPO) {
                let Some(p) = porta(n) else { continue };
                let v = match fisica.get(&id) {
                    Some(pose) if n == TRANSFORM => Some(pose.as_slice()),
                    _ => o.and_then(|o| o.componentes.get(n)).map(Vec::as_slice),
                };
                drive.blobs.conduz(sim, e, p, v);
            }
        }
        if let Some(p) = porta(VISIBILIDADE) {
            if !vivo {
                drive.blobs.conduz(sim, e, p, Some(&escondido(o)));
            } else if nomes.contains(VISIBILIDADE) {
                let v = o.and_then(|o| o.componentes.get(VISIBILIDADE));
                drive.blobs.conduz(sim, e, p, v.map(Vec::as_slice));
            }
        }
        if let Some(p) = porta(CORPO) {
            // ⚠️ Um escritor por facto: a ponte global trata o corpo da macro como cena.
            let v = if vivo && nomes.contains(CORPO) {
                o.and_then(|o| o.componentes.get(CORPO))
                    .and_then(|b| cinematico(b))
            } else {
                match p.le(sim, e) {
                    Some(Some(b)) => cinematico(&b),
                    _ => continue,
                }
            };
            drive.blobs.conduz(sim, e, p, v.as_deref());
        }
    }
}

#[cfg(test)]
#[path = "palco_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "palco_pilha_tests.rs"]
mod pilha;

#[cfg(test)]
#[path = "palco_sonda.rs"]
mod sonda;
