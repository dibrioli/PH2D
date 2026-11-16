//! **O instantâneo e o dreno da secção MOTION INPUT** (ciclo 19 do Motion).
//!
//! A ponte `u8` ⇄ `MotionSource` vive AQUI e só aqui (o painel não vê a `ph2d-ecs`).

use crate::motion_input_fonte as fonte;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_ecs::scene::registry::ComponentRegistry;
use ph2d_ecs::{MotionInput, MotionInputs, MotionSource, Name, SimWorld};
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MOTION_INPUT_SOURCES, MotionInputAoVivo as AoVivo,
    MotionInputFieldEdit as E, MotionInputOutro as Outro,
};

const _: () = assert!(MotionSource::ALL.len() == MOTION_INPUT_SOURCES);

/// O índice que atravessa a fronteira ⇄ a fonte. Fora da faixa ⇒ `None` (edição inerte).
#[must_use]
pub fn source_de_u8(v: u8) -> Option<MotionSource> {
    MotionSource::ALL.get(usize::from(v)).copied()
}

/// ⭐ **Constrói o instantâneo** da entidade escolhida, ou `None` se ela não tem o componente.
///
/// `listas` = `(nomes do grafo, acções do Input Map)`, que esta crate não vê (quem as passa é a
/// shell); só correm quando o componente existe. `reg` dá os campos do From (Property).
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    (selected_count, a_correr): (usize, bool),
    reg: Option<&ComponentRegistry>,
    listas: (impl FnOnce() -> Vec<String>, impl FnOnce() -> Vec<String>),
) -> Option<InspectorMotionInputInfo> {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let world = sim.world();
    let m = world.get::<MotionInput>(e)?;
    // ⚠️ As MESMAS portas que a ponte lê — senão o painel diria «tem vida» sobre uma que ela ignora.
    let alvo = fonte::alvo(world, e, m.from);
    let vida = alvo.and_then(|a| crate::health_bar_bridge::vida_da_barra(world, a));
    let tem_corpo = alvo.is_some_and(|a| match m.source {
        MotionSource::Grounded => fonte::personagem(world, a).is_some(),
        _ => fonte::corpo(world, a).is_some(),
    });
    let campos_do_alvo = match (reg, alvo) {
        (Some(r), Some(a)) => fonte::campos_numericos(r, world, a),
        _ => Vec::new(),
    };
    let (ao_vivo, tapado_por) = ao_vivo(sim, e, m, a_correr, (vida, reg, alvo));
    Some(InspectorMotionInputInfo {
        entity_bits: bits,
        name: m.name.clone(),
        source: m.source.index(),
        value: m.value,
        color: m.color,
        nomes_do_grafo: (listas.0)(),
        tem_vida: vida.is_some(),
        from: ph2d_ray_sensor::gotas::nomes(world, &[m.from])
            .pop()
            .unwrap_or_default(),
        alvo_achado: alvo.is_some(),
        tem_corpo,
        action: m.action.clone(),
        acoes_do_mapa: (listas.1)(),
        property: m.property.clone(),
        campos_do_alvo,
        remap: m.remap,
        remap_from: m.remap_from,
        remap_to: m.remap_to,
        curve: m.curve.clone(),
        smooth: m.smooth,
        selected_count,
        a_correr,
        ao_vivo,
        tapado_por,
    })
}

const fn ao_vivo_de(v: Valor) -> AoVivo {
    match v {
        Valor::Numero(n) => AoVivo::Numero(n),
        Valor::Cor(c) => AoVivo::Cor(c),
    }
}

/// MI-11: o que este objecto manda agora e, em Play, quem o tapa. A vida lê-se também parada (é a
/// mesma razão que a ponte publica); o resto só existe com o jogo a correr, pelo recurso que o grafo
/// lê — nunca uma segunda conta.
fn ao_vivo(
    sim: &SimWorld,
    e: ph2d_ecs::Entity,
    m: &MotionInput,
    a_correr: bool,
    (vida, reg, alvo): (
        Option<(f32, f32)>,
        Option<&ComponentRegistry>,
        Option<ph2d_ecs::Entity>,
    ),
) -> (Option<AoVivo>, Option<(Outro, AoVivo)>) {
    let world = sim.world();
    let eu = Escritor::Objeto {
        nome: world
            .get::<Name>(e)
            .map(|n| n.0.clone())
            .unwrap_or_default(),
        bits: e.to_bits(),
    };
    let jogo = world.get_resource::<MotionInputs>().filter(|_| a_correr);
    let nome = m.name.trim();
    let proprio = match m.source {
        MotionSource::Health => match jogo {
            Some(j) => j.valor_de(nome, &eu).map(ao_vivo_de),
            None => vida.map(|(agora, max)| AoVivo::Numero(agora / max)),
        },
        MotionSource::Damage => {
            jogo.map(|j| AoVivo::Disparos(j.disparos_de(nome, &eu).unwrap_or(0)))
        }
        MotionSource::Property => match jogo {
            Some(j) => j.valor_de(nome, &eu).map(ao_vivo_de),
            None => alvo
                .and_then(|a| fonte::propriedade(reg, world, a, &m.property))
                .map(AoVivo::Numero),
        },
        MotionSource::Value
        | MotionSource::Color
        | MotionSource::Speed
        | MotionSource::Grounded
        | MotionSource::Action => jogo.and_then(|j| j.valor_de(nome, &eu)).map(ao_vivo_de),
    };
    let tapado = jogo
        .and_then(|j| j.primeira_linha(nome))
        .filter(|(quem, _)| **quem != eu)
        .and_then(|(quem, v)| {
            let outro = match quem {
                Escritor::Script => Outro::Script,
                Escritor::Linha => Outro::Linha,
                Escritor::Objeto { nome, .. } => Outro::Objeto(nome.clone()),
                Escritor::Sinal => return None,
            };
            Some((outro, ao_vivo_de(v)))
        });
    (proprio, tapado)
}

/// ⭐ **A semente do nome** (anexar): o primeiro nome do grafo que nenhum outro `Motion Input` usa
/// (senão o primeiro do grafo; sem grafo, fica o de fábrica). Só troca o nome de FÁBRICA — nunca o
/// que o artista escreveu.
pub fn semeia_o_nome(sim: &mut SimWorld, bits: u64) {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let world = sim.world_mut();
    let Some(grafo) = world.get_resource::<ph2d_ecs::MotionGraphNames>() else {
        return;
    };
    let grafo = grafo.0.clone();
    let fabrica = MotionInput::default().name;
    if world
        .get::<MotionInput>(e)
        .is_none_or(|m| m.name != fabrica)
    {
        return;
    }
    let mut q = world.query::<(ph2d_ecs::Entity, &MotionInput)>();
    let usados: Vec<String> = q
        .iter(world)
        .filter(|(o, _)| *o != e)
        .map(|(_, m)| m.name.trim().to_string())
        .collect();
    let escolhido = grafo
        .iter()
        .find(|n| !usados.contains(n))
        .or_else(|| grafo.first());
    if let (Some(n), Some(mut m)) = (escolhido.cloned(), world.get_mut::<MotionInput>(e)) {
        m.name = n;
    }
}

/// Aplica uma edição. `true` = o mundo mudou (escrever o MESMO valor não é mudança: o undo é por
/// diff).
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = ph2d_ecs::Entity::from_bits(bits);
    if let E::From(nome) = edit {
        // O NOME vira `StableId` pela porta do conta-gotas do Ray Sensor; ninguém com ele = inerte.
        let Some(id) = ph2d_ray_sensor::gotas::resolve(sim.world(), nome) else {
            return false;
        };
        let Some(mut m) = sim.world_mut().get_mut::<MotionInput>(e) else {
            return false;
        };
        if m.from == id {
            return false;
        }
        m.from = id;
        return true;
    }
    let Some(mut m) = sim.world_mut().get_mut::<MotionInput>(e) else {
        return false;
    };
    match edit {
        E::Name(n) => {
            if m.name == *n {
                return false;
            }
            m.name.clone_from(n);
        }
        E::Source(i) => {
            let Some(s) = source_de_u8(*i) else {
                return false;
            };
            if m.source == s {
                return false;
            }
            m.source = s;
        }
        E::Value(v) => {
            if !v.is_finite() || m.value.to_bits() == v.to_bits() {
                return false;
            }
            m.value = *v;
        }
        E::Color(c) => {
            if m.color == *c {
                return false;
            }
            m.color = *c;
        }
        E::From(_) => return false, // resolvido acima
        E::Action(t) => return troca(&mut m.action, t),
        E::Property(t) => return troca(&mut m.property, t),
        E::Curve(t) => return troca(&mut m.curve, t),
        E::Remap(b) => {
            if m.remap == *b {
                return false;
            }
            m.remap = *b;
        }
        E::RemapFrom(v) => return troca_numero(&mut m.remap_from, *v),
        E::RemapTo(v) => return troca_numero(&mut m.remap_to, *v),
        E::Smooth(v) => return troca_numero(&mut m.smooth, v.max(0.0)),
    }
    true
}

fn troca(campo: &mut String, t: &str) -> bool {
    if campo == t {
        return false;
    }
    t.clone_into(campo);
    true
}

fn troca_numero(campo: &mut f32, v: f32) -> bool {
    if !v.is_finite() || campo.to_bits() == v.to_bits() {
        return false;
    }
    *campo = v;
    true
}

/// Aplica todas as edições do quadro. `true` = alguma coisa mudou (`|`: todas correm).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    edits
        .iter()
        .fold(false, |mudou, (bits, edit)| apply(sim, *bits, edit) | mudou)
}

#[cfg(test)]
#[path = "motion_input_inspector_tests.rs"]
mod tests;
