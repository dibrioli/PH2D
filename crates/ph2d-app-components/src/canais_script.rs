//! **O funil da família SCRIPT** — a curva de uma propriedade que o script DECLARA chega ao `own`
//! do `LuauScript` (`ph2d_ecs::canal_script`; plano `docs/UI_New_and_Simple/F27_E_plano.md`).
//!
//! ⭐ O valor publicado é o que o objecto USA (`ph2d_script::resolve`: o próprio, senão o default da
//! declaração); a escrita é um valor PRÓPRIO — uma chave é um valor que o artista pôs. As
//! declarações vivem na VM, fora do mundo: a shell publica-as em [`Declaracoes`] uma vez por quadro.

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::SCRIPT;
use ph2d_ecs::canal_palavra::NomesDosScripts;
use ph2d_ecs::canal_script as cs;
use ph2d_ecs::scene::property::slot_in;
use ph2d_ecs::scene::registry::ComponentRegistry;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, TextosDosCanais, World};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use ph2d_script::props::{PropDecl, put, resolve};
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptId, ScriptSource, ScriptValue};
use serde_json::Value;

/// O nome do componente no registo (o do catálogo).
const LUAU_SCRIPT: &str = "ph2d::script::LuauScript";

/// ⭐ **O que cada script do projeto declara**, como a VM o colheu (só os prontos).
#[derive(bevy_ecs::resource::Resource, Clone, Debug, Default, PartialEq)]
pub struct Declaracoes(pub BTreeMap<ScriptId, Vec<PropDecl>>);

/// Publica as declarações da VM no mundo — só quando mudam.
pub fn publicar_declaracoes(sim: &mut SimWorld, host: Option<&ScriptHost>) {
    let novas = Declaracoes(
        (sim.world().get_resource::<ProjectScripts>().into_iter())
            .flat_map(|l| l.iter().map(|(id, _)| id).collect::<Vec<_>>())
            .filter_map(|id| Some((id, host?.scene_info(id)?.decls()?.to_vec())))
            .collect(),
    );
    if sim.world().get_resource::<Declaracoes>() != Some(&novas) {
        sim.world_mut().insert_resource(novas);
    }
    // O nome de cada script, para a palavra do canal do ficheiro (F27 F).
    let nomes = NomesDosScripts(
        (sim.world().get_resource::<ProjectScripts>().into_iter())
            .flat_map(|l| l.iter().map(|(id, d)| (id.0, d.name.clone())))
            .collect(),
    );
    if sim.world().get_resource::<NomesDosScripts>() != Some(&nomes) {
        sim.world_mut().insert_resource(nomes);
    }
}

fn declaracoes<'w>(w: &'w World, cfg: &LuauScript) -> Option<&'w [PropDecl]> {
    let ScriptSource::Library(id) = cfg.source else {
        return None;
    };
    w.get_resource::<Declaracoes>()?
        .0
        .get(&id)
        .map(Vec::as_slice)
}

/// As faixas de um valor, como o canal as guarda (o texto entra na tabela).
fn faixas(v: &ScriptValue, textos: &mut TextosDosCanais) -> Vec<f32> {
    #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
    let f = |x: f64| x as f32;
    match v {
        ScriptValue::Number(n) => vec![f(*n)],
        ScriptValue::Bool(b) => vec![if *b { 1.0 } else { 0.0 }],
        // Um objecto é o NOME dele: a chave troca de objecto como um texto troca de palavra.
        ScriptValue::Text(t) | ScriptValue::Object(t) => vec![textos.indice(Some(t))],
        ScriptValue::Vec2(p) => p.iter().copied().map(f).collect(),
        ScriptValue::Color(c) => c.iter().copied().map(f).collect(),
    }
}

/// A faixa `faixa` de `v` sem acrescentar à tabela (o eco: o texto já lá está).
fn faixa_de(v: &ScriptValue, faixa: u16, textos: &TextosDosCanais) -> Option<f32> {
    let i = usize::from(faixa);
    #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
    let f = |x: f64| x as f32;
    match v {
        ScriptValue::Number(n) => Some(f(*n)),
        ScriptValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        ScriptValue::Text(t) | ScriptValue::Object(t) => textos.procura(t),
        ScriptValue::Vec2(p) => p.get(i).copied().map(f),
        ScriptValue::Color(c) => c.get(i).copied().map(f),
    }
}

/// `v` com a faixa `faixa` posta a `x`; `None` = a forma recusa (um texto que a tabela não tem).
fn com_faixa(v: &ScriptValue, faixa: u16, x: f32, textos: &TextosDosCanais) -> Option<ScriptValue> {
    let i = usize::from(faixa);
    let mut v = v.clone();
    match &mut v {
        ScriptValue::Number(n) => *n = f64::from(x),
        ScriptValue::Bool(b) => *b = x >= 0.5,
        ScriptValue::Text(t) | ScriptValue::Object(t) => *t = textos.texto(x)?.to_owned(),
        ScriptValue::Vec2(p) => *p.get_mut(i)? = f64::from(x),
        ScriptValue::Color(c) => *c.get_mut(i)? = f64::from(x),
    }
    Some(v)
}

/// O que o objecto usa agora, por nome (as declarações conhecidas; senão só os próprios).
fn valores(w: &World, cfg: &LuauScript) -> Vec<(String, ScriptValue)> {
    match declaracoes(w, cfg) {
        Some(d) => (resolve(Some(d), &cfg.own).values.into_iter())
            .map(|r| (r.name, r.value))
            .collect(),
        None => cfg
            .own
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

/// Publica o valor de cada propriedade de script das `entidades`.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64], textos: &mut TextosDosCanais) {
    let mut lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = Vec::new();
    for &b in entidades {
        let Some(e) = Entity::try_from_bits(b) else {
            continue;
        };
        let Some(cfg) = sim.world().get::<LuauScript>(e) else {
            continue;
        };
        let mut v = Vec::new();
        for (nome, valor) in valores(sim.world(), cfg) {
            let tipo = crate::script_inspector::tipo_de(&valor);
            v.extend(
                cs::chaves(&nome, tipo, textos)
                    .into_iter()
                    .zip(faixas(&valor, textos)),
            );
        }
        lotes.push((b, v));
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, SCRIPT);
    }
    for (b, valores) in lotes {
        for (k, v) in valores {
            lidos.publicar(b, k, v);
        }
    }
}

/// O `own` inteiro como o documento o guarda (o que o `FieldDrive` lembra).
fn own_guardado(reg: &ComponentRegistry, w: &World, e: Entity) -> Option<Value> {
    let entry = reg.get_by_name(LUAU_SCRIPT)?;
    let mut v = (entry.value_of)(w, e).ok()??;
    slot_in(&mut v, "own").cloned()
}

/// ⭐ **Aplica o que a curva mandou neste quadro** às propriedades de script, regista o eco e declara
/// a escrita à pré-visualização. Devolve `true` se algum objecto mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    reg: &ComponentRegistry,
    drive: &mut PreviewDrive,
    textos: &TextosDosCanais,
) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(SCRIPT))
    else {
        return false;
    };
    escritas.sort_by_key(|&(b, k, _)| (b, k));
    let entry = reg.get_by_name(LUAU_SCRIPT);
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (b, k, x) in escritas {
        let Some(e) = Entity::try_from_bits(b).filter(|&e| sim.world().get_entity(e).is_ok())
        else {
            continue;
        };
        let (Some(nome), Some((_, faixa))) = (cs::nome(k, textos), cs::tipo(k)) else {
            continue;
        };
        let Some(cfg) = sim.world().get::<LuauScript>(e) else {
            continue;
        };
        let agora = valores(sim.world(), cfg)
            .into_iter()
            .find(|(n, _)| n == nome);
        let Some(novo) = agora.and_then(|(_, v)| com_faixa(&v, faixa, x, textos)) else {
            continue;
        };
        if cfg.own.get(nome) != Some(&novo) {
            let antes = own_guardado(reg, sim.world(), e);
            if let Some(mut c) = sim.world_mut().get_mut::<LuauScript>(e) {
                put(&mut c.own, nome, novo);
            }
            let depois = own_guardado(reg, sim.world(), e);
            if let (Some(entry), Some(antes), Some(depois)) = (entry, antes, depois)
                && !ph2d_ecs::is_transient(sim.world(), e)
            {
                drive
                    .fields
                    .driven(e, FieldRef::of(entry, "own"), antes, depois, false);
            }
            mudou = true;
        }
        if let Some(cfg) = sim.world().get::<LuauScript>(e)
            && let Some((_, v)) = valores(sim.world(), cfg)
                .into_iter()
                .find(|(n, _)| n == nome)
            && let Some(ficou) = faixa_de(&v, faixa, textos)
        {
            ecos.push((b, k, x, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, k, x, f) in ecos {
        lidos.eco(b, k, x, f);
    }
    mudou
}

#[cfg(test)]
#[path = "canais_script_tests.rs"]
mod tests;
