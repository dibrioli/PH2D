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

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_script::{LuauScript, ScriptHost};

/// **O que a ponte fez neste quadro** — o que a shell publica e imprime.
#[derive(Debug, Default)]
pub struct ScriptFrame {
    /// Sinais emitidos por `ph2d.emit`, com os bits de quem emitiu, pela ordem da identidade.
    pub emitted: Vec<(u64, String)>,
    /// Objectos que pararam de correr NESTE quadro, com a mensagem (a shell imprime-a UMA vez).
    pub failed: Vec<(u64, String)>,
    /// Quantos ganchos correram.
    pub calls: usize,
}

/// Os objectos com script e a pose deles agora — o «antes» de cada escrita.
fn poses(sim: &mut SimWorld) -> Vec<(Entity, Transform)> {
    let world = sim.world_mut();
    world
        .query::<(Entity, &LuauScript, &Transform)>()
        .iter(world)
        .map(|(e, _, t)| (e, *t))
        .collect()
}

/// Declara ao ledger o que uma passagem fez: quem mudou, e quem continua conduzido sem mudar.
fn declare(sim: &SimWorld, before: &[(Entity, Transform)], drive: &mut PreviewDrive) {
    for &(entity, was) in before {
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
pub fn frame(
    host: &mut ScriptHost,
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    playing: bool,
    ticks: u32,
    fixed_dt: f64,
    heard: &[&str],
) -> ScriptFrame {
    let mut out = ScriptFrame::default();
    host.scene_sync(sim.world_mut());
    let before = poses(sim);
    if playing {
        for _ in 0..ticks {
            let r = host.scene_tick(sim.world_mut(), fixed_dt);
            absorb(&mut out, r);
        }
        let r = host.scene_hear(sim.world_mut(), heard);
        absorb(&mut out, r);
    }
    declare(sim, &before, drive);
    out
}

fn absorb(out: &mut ScriptFrame, r: ph2d_script::SceneReport) {
    out.calls += r.calls;
    out.emitted
        .extend(r.emitted.into_iter().map(|(e, n)| (e.to_bits(), n)));
    out.failed
        .extend(r.failed.into_iter().map(|(e, m)| (e.to_bits(), m)));
}

/// ⭐ **REBOBINAR É RENASCER** — o `self` de cada objecto volta a nascer no próximo Play, e a pose
/// que a corrida escreveu volta à que o artista pôs. Devolve quantos objectos tinham vivo.
///
/// ⚠️ **Chamada pelo INVARIANTE do rebobinar da shell** (relógio no início e parado), ao lado da
/// porta da família `Logic` — nunca por um gancho num botão: o transporte tem mais de um caminho até
/// ao zero.
pub fn rewind(host: &mut ScriptHost, sim: &mut SimWorld, drive: &mut PreviewDrive) -> usize {
    let n = host.scene_rewind();
    for (entity, _) in poses(sim) {
        drive.release_to_authored(sim, entity, Driver::ScriptPose);
    }
    n
}

#[cfg(test)]
#[path = "script_bridge_tests.rs"]
mod tests;
