//! **O instantâneo e o dreno da secção MOTION INPUT** (ciclo 19 do Motion).
//!
//! A ponte `u8` ⇄ `MotionSource` vive AQUI e só aqui (o painel não vê a `ph2d-ecs`).

use ph2d_ecs::motion_inputs::{Escritor, Valor};
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
/// `nomes_do_grafo` só corre quando o componente existe — é a varredura do grafo do Motion, que esta
/// crate não vê (quem a passa é a shell).
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    selected_count: usize,
    a_correr: bool,
    nomes_do_grafo: impl FnOnce() -> Vec<String>,
) -> Option<InspectorMotionInputInfo> {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let world = sim.world();
    let m = world.get::<MotionInput>(e)?;
    let vida = crate::health_bar_bridge::vida_da_barra(world, e);
    let (ao_vivo, tapado_por) = ao_vivo(sim, e, m, a_correr, vida);
    Some(InspectorMotionInputInfo {
        entity_bits: bits,
        name: m.name.clone(),
        source: m.source.index(),
        value: m.value,
        color: m.color,
        nomes_do_grafo: nomes_do_grafo(),
        // ⚠️ A MESMA porta que a ponte lê — senão o painel diria «tem vida» sobre uma que ela ignora.
        tem_vida: vida.is_some(),
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
    vida: Option<(f32, f32)>,
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
        MotionSource::Value | MotionSource::Color => {
            jogo.and_then(|j| j.valor_de(nome, &eu)).map(ao_vivo_de)
        }
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
    }
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
