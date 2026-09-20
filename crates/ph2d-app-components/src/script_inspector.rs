//! **A secção SCRIPT: o instantâneo e o dreno** (TOP-20 #16, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o molde do `inspector_statemachine` da shell:
//! quem lê o mundo para o painel e quem escreve a edição de volta fazem a MESMA tradução. E vivem
//! AQUI, e não na shell, pelo tecto dela (`the_shell_only_shrinks`).
//!
//! ⭐ **A resposta a «que valor este objecto usa?» é uma só** — a `ph2d_script::resolve`, com as
//! declarações que a VM colheu. O painel recebe-a pronta e pinta.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::script_edits::{
    InspectorScriptInfo, InspectorScriptOrphan, InspectorScriptProp, InspectorScriptStatus,
    InspectorScriptValue, PorqueOrfao, ScriptFieldEdit as E,
};
use ph2d_script::props::{forget, put, resolve};
use ph2d_script::{LuauScript, Origin, OrphanWhy, ScriptHost, ScriptInfo, ScriptValue};

/// Tradução do valor para o vocabulário do painel.
fn para_painel(v: &ScriptValue) -> InspectorScriptValue {
    match v {
        ScriptValue::Number(n) => InspectorScriptValue::Number(*n),
        ScriptValue::Bool(b) => InspectorScriptValue::Bool(*b),
        ScriptValue::Text(t) => InspectorScriptValue::Text(t.clone()),
    }
}

/// **O passo de arrasto de um número sem `step` declarado** — `1` para um default inteiro, `0,1` para
/// um com casas decimais.
///
/// ⚠️ **É pista de edição, não lei** (Q6 do oráculo), e o autor do script manda nela com
/// `{ step = … }`. A regra lê o que o autor ESCREVEU: `ph2d.property("vidas", 3)` pede passos
/// inteiros, `ph2d.property("amplitude", 1.5)` não.
fn passo(default: f64, declarado: Option<f64>) -> f64 {
    declarado.unwrap_or(if default.fract() == 0.0 { 1.0 } else { 0.1 })
}

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub fn build_info(
    sim: &SimWorld,
    host: Option<&ScriptHost>,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorScriptInfo> {
    let e = Entity::from_bits(bits);
    let cfg = sim.world().get::<LuauScript>(e)?;
    let known = host.and_then(|h| h.scene_info(&cfg.source));
    let status = match (host, known) {
        (None, _) => InspectorScriptStatus::Unavailable,
        _ if cfg.source.trim().is_empty() => InspectorScriptStatus::NoFile,
        (Some(_), None) => InspectorScriptStatus::Loading,
        (Some(_), Some(ScriptInfo::Missing(_))) => InspectorScriptStatus::Missing,
        (Some(_), Some(ScriptInfo::Broken(m))) => InspectorScriptStatus::Broken(m.clone()),
        (Some(_), Some(ScriptInfo::Ready(_))) => InspectorScriptStatus::Ready,
    };
    let decls = known.and_then(ScriptInfo::decls);
    let r = resolve(decls, &cfg.own);
    let props = r
        .values
        .iter()
        .map(|p| {
            let default = decls
                .and_then(|d| d.iter().find(|d| d.name == p.name))
                .map(|d| match d.default {
                    ScriptValue::Number(n) => n,
                    _ => 0.0,
                })
                .unwrap_or(0.0);
            InspectorScriptProp {
                name: p.name.clone(),
                value: para_painel(&p.value),
                own: p.origin == Origin::Own,
                // ⭐ **A lista viaja do script para o painel e para mais lado nenhum** — ela não
                // entra no documento: o que o objecto guarda é o texto escolhido.
                options: p.hint.options.clone(),
                min: p.hint.min,
                max: p.hint.max,
                step: Some(passo(default, p.hint.step)),
            }
        })
        .collect();
    let orphans = r
        .orphans
        .iter()
        .map(|o| InspectorScriptOrphan {
            name: o.name.clone(),
            value: para_painel(&o.value),
            wants: match o.why {
                OrphanWhy::Missing => PorqueOrfao::NaoDeclarado,
                OrphanWhy::WrongKind { declared } => PorqueOrfao::OutroTipo(declared.label()),
                OrphanWhy::NotAnOption => PorqueOrfao::ForaDaLista,
            },
        })
        .collect();
    let also_physics = sim
        .world()
        .get::<ph2d_physics_ecs::RigidBody>(e)
        .is_some_and(|b| b.kind == ph2d_physics_ecs::BodyKind::Dynamic);
    Some(InspectorScriptInfo {
        entity_bits: bits,
        source: cfg.source.clone(),
        status,
        props,
        orphans,
        kept: r.kept.len(),
        failure: host.and_then(|h| h.scene_failure(e)).map(str::to_owned),
        clock_playing,
        also_physics,
        selected_count,
    })
}

/// **Aplica uma edição.** `true` = o documento mudou.
///
/// ⚠️ **`Browse` não escreve nada aqui** — o diálogo é da shell, que tem a janela, e ela devolve o
/// caminho escolhido como um `Source`.
///
/// ⚠️ **Decide com uma LEITURA e só depois empresta em escrita** — o `bevy` marca a alteração no
/// `deref_mut`, e um componente tocado por nada é ruído para quem lê `Changed<…>`. ⭐ Mas **pôr o
/// mesmo número pela primeira vez MUDA**: ele passa a ser próprio (a divergência D1).
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let Some(cfg) = sim.world().get::<LuauScript>(e) else {
        return false;
    };
    let posto = |name: &str, v: ScriptValue| (cfg.own.get(name) != Some(&v)).then_some(v);
    enum Mexe {
        Caminho(String),
        Poe(String, ScriptValue),
        Larga(String),
    }
    let mexe = match edit {
        E::Browse => None,
        E::Source(path) => (cfg.source != *path).then(|| Mexe::Caminho(path.clone())),
        E::SetNumber(n, v) => posto(n, ScriptValue::Number(*v)).map(|v| Mexe::Poe(n.clone(), v)),
        E::SetBool(n, b) => posto(n, ScriptValue::Bool(*b)).map(|v| Mexe::Poe(n.clone(), v)),
        E::SetText(n, t) => posto(n, ScriptValue::Text(t.clone())).map(|v| Mexe::Poe(n.clone(), v)),
        E::Forget(n) => cfg.own.contains_key(n).then(|| Mexe::Larga(n.clone())),
    };
    let Some(mexe) = mexe else {
        return false;
    };
    let Some(mut cfg) = sim.world_mut().get_mut::<LuauScript>(e) else {
        return false;
    };
    match mexe {
        Mexe::Caminho(p) => cfg.source = p,
        Mexe::Poe(n, v) => put(&mut cfg.own, &n, v),
        Mexe::Larga(n) => {
            forget(&mut cfg.own, &n);
        }
    }
    true
}

/// **Aplica as edições de um quadro.** `pick` é o diálogo de ficheiro — a shell passa o dela (é ela
/// que tem a janela), e um `Browse` cancelado não escreve nada. `true` = o documento mudou.
pub fn apply_all(
    sim: &mut SimWorld,
    edits: &[(u64, E)],
    mut pick: impl FnMut() -> Option<String>,
) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        let edit = match edit {
            E::Browse => match pick() {
                Some(path) => E::Source(path),
                None => continue,
            },
            other => other.clone(),
        };
        mexeu |= apply(sim, *bits, &edit);
    }
    mexeu
}

/// As extensões que o diálogo `Browse` oferece — a porta única, lida pela shell e pelo gate.
pub const SCRIPT_EXTENSIONS: &[&str] = &["luau", "lua"];

#[cfg(test)]
#[path = "script_inspector_tests.rs"]
mod tests;
