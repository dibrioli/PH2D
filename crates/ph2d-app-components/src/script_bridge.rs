//! ⭐⭐⭐ **A PONTE dos scripts do artista** (TOP-20 #16) — a VM vive no
//! [`ph2d_script::ScriptHost`]; aqui ela encontra o QUADRO: quando corre, o que escreve entra no
//! `Ctrl+Z`, e o que acontece ao rebobinar. Plano: `docs/Components/13_plano_script_properties.md`.
//!
//! # ⚠️ As três leis desta ponte, e onde cada uma foi paga
//!
//! - ⭐ **A corrida é o relógio A ANDAR** (`playhead.is_playing()`), o gate da fábrica e da física.
//!   Parado, o `sync` corre na mesma — é ele que dá ao painel as declarações de um ficheiro no
//!   instante em que o artista o escolhe.
//! - ⭐⭐ **Um passo fixo, UMA chamada a `update`.** Ao contrário da lei pura do relógio, um script
//!   não tem laço de recuperação: `ticks × dt` numa chamada faria o replay depender da taxa de
//!   quadros.
//! - ⭐⭐⭐ **A pose que um script escreve é PRÉ-VISUALIZAÇÃO, e o script é um condutor PERSISTENTE**
//!   (o `Driver::ScriptPose` do `preview_drive`). A tabela do §3.7 do plano, em código:
//!
//!   | momento | o ledger ouve |
//!   |---|---|
//!   | a correr, o script escreveu | `driven(antes, depois)` |
//!   | a correr ou pausado, e o script NÃO escreveu | `driven(agora, agora)` — **só se já havia entrada** |
//!   | rebobinar | `release_to_authored` |
//!
//!   ⛔ **Sem a linha do meio**, um script que PAROU de mexer (chegou ao destino, ou o artista
//!   carregou em pausa) era lido pela `settle` como *«o motor largou»*, a pose da corrida virava
//!   documento, e o rebobinar já não tinha para onde voltar. ⚠️ E ela passa `antes = agora` de
//!   propósito: um arrasto feito na pausa é a **outra mão** do ledger, e fica adoptado como o novo
//!   autorado.
//!
//! ⚠️ **Um script e a física no MESMO corpo são duas mãos no mesmo `Transform`** — a física escreve
//! no passo dela e o script por cima, e o ledger de cada um adopta a escrita do outro como autorada.
//! Não é suportado, e o painel avisa (§3.9 do plano).

use std::collections::BTreeMap;

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, SimWorld, Spawned, StableId, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_script::{LuauScript, SceneDoors, ScriptHost};

/// **O que a ponte fez neste quadro** — o que a shell publica e imprime.
#[derive(Debug, Default)]
pub struct ScriptFrame {
    /// Sinais emitidos por `ph2d.emit`, com os bits de quem emitiu, pela ordem da identidade.
    pub emitted: Vec<(u64, String)>,
    /// Objectos que pararam de correr NESTE quadro, com o diagnóstico (a shell imprime-o UMA vez).
    pub failed: Vec<(u64, ph2d_script::ScriptDiagnostic)>,
    /// Quantos ganchos correram.
    pub calls: usize,
    /// Objectos da corrida que um script mandou apagar — mortes para o dreno único da fábrica.
    pub despawned: Vec<Entity>,
}

/// ⭐⭐ **A porta da `Factory` para os scripts** (`ph2d.spawn("Mestre")`, CodeEditor item 6): a
/// MESMA instanciação das cópias de uma fábrica (`instantiate_master_many`, arte própria) e a mesma
/// marca [`Spawned`] — logo a cópia é TRANSIENTE: não vai para o documento nem para o `Ctrl+Z`, e o
/// rebobinar varre-a (`sweep_spawned`).
pub struct ScriptDoors<'a, 'd> {
    pub registry: &'a ComponentRegistry,
    pub docs: crate::instance_docs::OwnedDocs<'d>,
    /// O tique do passo fixo — o `born_tick` de quem nasce.
    pub tick: u64,
}

impl SceneDoors for ScriptDoors<'_, '_> {
    fn instantiate(
        &mut self,
        sim: &mut SimWorld,
        master: Entity,
        by: Entity,
    ) -> Result<Entity, String> {
        let by = sim.world().get::<StableId>(by).map_or(0, |s| s.0);
        let copia = crate::instantiate::instantiate_master_many(
            sim,
            self.registry,
            master,
            None,
            &mut self.docs,
            crate::instantiate::ArtLink::Own,
            1,
        )
        .map_err(|r| match r {
            crate::instantiate::Refusal::NotAMaster => {
                ph2d_i18n::tr("app.components.script_bridge.it_is_not_a_master").to_string()
            }
            crate::instantiate::Refusal::WouldNestInItself => ph2d_i18n::tr(
                "app.components.script_bridge.the_copy_would_land_inside_its_own_master",
            )
            .to_string(),
        })?
        .remove(0);
        sim.world_mut().entity_mut(copia).insert(Spawned {
            by,
            born_tick: self.tick,
        });
        Ok(copia)
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    /// ⭐ **O script que um script põe numa cópia é um OVERRIDE dela** (ADR-0164): o sync mestre →
    /// instância propaga a AUSÊNCIA de um componente, e sem eco do mestre «o mestre ganha» — a
    /// 1.ª cópia de uma corrida perdia o script no passe seguinte (medido na foto da cena 6).
    fn attached(&mut self, sim: &mut SimWorld, entity: Entity) {
        let Some(piece) = sim
            .world()
            .get::<ph2d_ecs::InstanceOf>(entity)
            .map(|i| i.master)
        else {
            return;
        };
        let Some(script) = self
            .registry
            .iter()
            .find(|e| e.canonical_name == "ph2d::script::LuauScript")
        else {
            return;
        };
        let key = ph2d_ecs::OverrideKey {
            piece,
            type_id: script.type_id,
        };
        let mut em = sim.world_mut().entity_mut(entity);
        match em.get_mut::<ph2d_ecs::ObjectInstance>() {
            Some(mut o) => {
                o.overrides.insert(key);
            }
            None => {
                let mut o = ph2d_ecs::ObjectInstance::default();
                o.overrides.insert(key);
                em.insert(o);
            }
        }
    }
}

/// O «antes» de cada escrita: os objectos com script, e os que um script ainda conduz (um
/// `ph2d.set` num objecto SEM script também é condução — a 1.ª versão só olhava para quem tinha
/// script, e a pose da corrida desses virava documento).
fn poses(sim: &mut SimWorld, drive: &PreviewDrive) -> BTreeMap<Entity, Transform> {
    let world = sim.world_mut();
    let mut out: BTreeMap<Entity, Transform> = world
        .query::<(Entity, &LuauScript, &Transform)>()
        .iter(world)
        .map(|(e, _, t)| (e, *t))
        .collect();
    for e in drive.driven_by(Driver::ScriptPose) {
        if let Some(t) = world.get::<Transform>(e) {
            out.entry(e).or_insert(*t);
        }
    }
    out
}

/// Declara ao ledger o que uma passagem fez: quem mudou, e quem continua conduzido sem mudar.
fn declare(sim: &SimWorld, before: &BTreeMap<Entity, Transform>, drive: &mut PreviewDrive) {
    for (&entity, &was) in before {
        // Uma cópia da corrida não é documento: não há autorado a guardar por baixo dela.
        if ph2d_ecs::is_transient(sim.world(), entity) {
            continue;
        }
        let Some(now) = sim.world().get::<Transform>(entity).copied() else {
            continue; // saiu da cena nesta passagem
        };
        if now != was {
            drive.driven(entity, Driven::ScriptPose(was), Driven::ScriptPose(now));
        } else if drive.still_driving(entity, Driver::ScriptPose) {
            // ⚠️ A linha do meio da tabela do cabeçalho.
            drive.driven(entity, Driven::ScriptPose(now), Driven::ScriptPose(now));
        }
    }
}

/// ⭐⭐⭐ **Um quadro dos scripts.** `playing` é o relógio a andar; `ticks` os passos fixos deste
/// quadro; `heard` os sinais que o outbox entregou a este leitor.
///
/// ⚠️ **O `drive` é obrigatório na assinatura**, pela lei que a `ProjectState::capture` escreve para
/// si mesma: uma função-irmã «sem ledger» seria a segunda porta pela qual cada quadro de uma
/// corrida viraria um passo de `Ctrl+Z`.
#[allow(clippy::too_many_arguments)]
pub fn frame(
    host: &mut ScriptHost,
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    playing: bool,
    ticks: u32,
    fixed_dt: f64,
    heard: &[&str],
    doors: &mut dyn SceneDoors,
) -> ScriptFrame {
    let mut out = ScriptFrame::default();
    host.scene_sync(sim.world_mut());
    let mut before = poses(sim, drive);
    if playing {
        for _ in 0..ticks {
            let r = host.scene_tick_with(sim, fixed_dt, doors);
            absorb(&mut out, &mut before, r);
        }
        let r = host.scene_hear_with(sim, heard, doors);
        absorb(&mut out, &mut before, r);
    } else {
        // Parado, nenhuma borda do teclado ou do rato espera pelo Play.
        host.devices().forget_edges();
    }
    declare(sim, &before, drive);
    out
}

fn absorb(
    out: &mut ScriptFrame,
    before: &mut BTreeMap<Entity, Transform>,
    r: ph2d_script::SceneReport,
) {
    for (e, t) in r.wrote_before {
        before.entry(e).or_insert(t);
    }
    out.despawned.extend(r.despawned);
    out.calls += r.calls;
    out.emitted
        .extend(r.emitted.into_iter().map(|(e, n)| (e.to_bits(), n)));
    out.failed
        .extend(r.failed.into_iter().map(|(e, m)| (e.to_bits(), m)));
}

/// O script de demonstração do arranque (M7): prova que a VM está ligada e só imprime uma vez.
const BOOT_PLACEHOLDER: &str = include_str!("script_boot.luau");

/// **A VM do arranque** (M7) — a falha não é fatal: o app segue sem scripts. `ms` = o relógio do
/// arranque, para a linha do log (só terminal).
#[must_use]
pub fn boot(ms: u128) -> Option<ScriptHost> {
    let mut h = match ScriptHost::new() {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "[{ms:>6}ms] M7 ScriptHost failed: ScriptHost::new: {e} — continuing without scripting"
            );
            return None;
        }
    };
    if let Err(e) = h.load_script(BOOT_PLACEHOLDER) {
        eprintln!(
            "[{ms:>6}ms] M7 ScriptHost failed: load_script: {e} — continuing without scripting"
        );
        return None;
    }
    println!("[{ms:>6}ms] M7: ScriptHost initialized (placeholder script loaded)");
    Some(h)
}

/// ⭐ **REBOBINAR É RENASCER** — o `self` de cada objecto volta a nascer no próximo Play, e a pose
/// que a corrida escreveu volta à que o artista pôs. Devolve quantos objectos tinham vivo.
///
/// ⚠️ **Chamada pelo INVARIANTE do rebobinar da shell** (relógio no início e parado), ao lado da
/// porta da família `Logic` — nunca por um gancho num botão: o transporte tem mais de um caminho até
/// ao zero.
pub fn rewind(host: &mut ScriptHost, sim: &mut SimWorld, drive: &mut PreviewDrive) -> usize {
    let n = host.scene_rewind();
    for entity in drive.driven_by(Driver::ScriptPose) {
        drive.release_to_authored(sim, entity, Driver::ScriptPose);
    }
    n
}

#[cfg(test)]
#[path = "script_bridge_tests.rs"]
mod tests;
