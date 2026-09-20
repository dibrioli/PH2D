//! **A ponte do Inspector para as DUAS secções do ABANÃO** (suplente #25) — o instantâneo que o
//! painel lê e o dreno das edições que ele publica.
//!
//! ⚠️ **O instantâneo mora aqui e não no painel** porque as duas colunas que importam são
//! **varreduras do MUNDO**, que o painel não vê:
//!
//! | coluna | o que ela responde | o silêncio que ela explica |
//! |---|---|---|
//! | `activa` | *esta é a câmera que manda?* | um `CameraShake` numa câmera desligada lê-se igual a um a funcionar |
//! | `ha_camera_que_treme` | *existe QUEM abane?* | ⭐ o emissor ouve, a distância mede-se, e não há ninguém a tremer — a causa está **noutro objecto** |
//!
//! ⚠️⚠️ **A segunda é a mais valiosa da wave**, e é a forma da recusa dos pincéis de escultura:
//! *o artista não a consegue adivinhar olhando para este objecto.*

use ph2d_ecs::{
    CameraShake, CameraShakeRuntime, Entity, ShakeEmitter, ShakeSource, SignalFrom, SimWorld,
};
use ph2d_editor_core::shake_edits::{
    EmitterFieldEdit as EE, InspectorEmitterInfo, InspectorEmitterRow, InspectorShakeInfo,
    ShakeFieldEdit as SE,
};

/// **A cena tem uma câmera que treme?** — a varredura que o aviso do emissor precisa.
///
/// ⚠️ **Ela pergunta pelo COMPONENTE e não pela câmera activa**, de propósito: uma segunda câmera
/// que treme e que o artista vai activar a seguir não é um defeito, e acusá-la mandaria-o
/// arranjar o que já está certo.
fn ha_camera_que_treme(sim: &mut SimWorld) -> bool {
    let mundo = sim.world_mut();
    mundo.query::<&CameraShake>().iter(mundo).next().is_some()
}

/// O instantâneo da secção **CAMERA SHAKE**, ou `None` se o objecto não tiver o componente.
pub fn build_info_camera(
    sim: &mut SimWorld,
    bits: u64,
    clock_playing: bool,
    selected_count: usize,
) -> Option<InspectorShakeInfo> {
    let e = Entity::from_bits(bits);
    let activa = ph2d_ecs::active_camera_of(sim.world_mut()) == Some(e);
    let trauma = sim
        .world()
        .get::<CameraShakeRuntime>(e)
        .map_or(0.0, |rt| rt.trauma);
    let c = sim.world().get::<CameraShake>(e)?;
    Some(InspectorShakeInfo {
        entity_bits: bits,
        amplitude: c.amplitude,
        frequencia: c.frequencia,
        decaimento: c.decaimento,
        expoente: c.expoente,
        semente: c.semente,
        trauma,
        activa,
        clock_playing,
        selected_count,
    })
}

/// O instantâneo da secção **SHAKE EMITTER**, ou `None` se o objecto não tiver o componente.
pub fn build_info_emitter(
    sim: &mut SimWorld,
    bits: u64,
    clock_playing: bool,
    selected_count: usize,
) -> Option<InspectorEmitterInfo> {
    let ha = ha_camera_que_treme(sim);
    let em = sim.world().get::<ShakeEmitter>(Entity::from_bits(bits))?;
    let rows =
        em.0.iter()
            .map(|f| InspectorEmitterRow {
                on: f.on.clone(),
                de: f.de.tag(),
                forca: f.forca,
                dentro: f.dentro,
                fora: f.fora,
            })
            .collect();
    Some(InspectorEmitterInfo {
        entity_bits: bits,
        rows,
        ha_camera_que_treme: ha,
        clock_playing,
        selected_count,
    })
}

/// Aplica as edições da secção **CAMERA SHAKE**. Devolve `true` se alguma coisa mudou.
///
/// ⚠️ **O expoente é COAGIDO à faixa da lei aqui**, e não no painel: o painel manda o que os chips
/// dele produzem, e a faixa é propriedade da `ph2d-shake`. *Uma cerca no painel seria a segunda
/// declaração de um domínio que a lei já tem.*
pub fn apply_shake(sim: &mut SimWorld, edits: &[(u64, SE)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        let e = Entity::from_bits(*bits);
        let Some(mut c) = sim.world_mut().get_mut::<CameraShake>(e) else {
            continue;
        };
        match edit {
            SE::Amplitude(v) => c.amplitude = *v,
            SE::Frequencia(v) => c.frequencia = *v,
            SE::Decaimento(v) => c.decaimento = *v,
            SE::Expoente(n) => {
                c.expoente = (*n).clamp(ph2d_shake::EXPOENTE_MIN, ph2d_shake::EXPOENTE_MAX); // CLAMP-OK: a faixa é da lei
            }
            SE::Semente(v) => c.semente = *v,
            // ⭐⭐⭐ **O PERFIL escreve QUATRO campos e larga a SEMENTE**, e a ausência é a lei (ver
            // o cabeçalho do `ph2d_shake::perfil`): ela é IDENTIDADE e não sensação, e escrevê-la
            // faria duas câmeras que receberam o mesmo clique tremer em UNÍSSONO.
            SE::Perfil(t) => {
                let n = ph2d_shake::Perfil::from_tag(*t).numeros();
                c.amplitude = n.amplitude;
                c.frequencia = n.frequencia;
                c.decaimento = n.decaimento;
                // ⚠️ Pela mesma porta do braço acima: a faixa é propriedade da lei, não do painel.
                c.expoente = n
                    .expoente
                    .clamp(ph2d_shake::EXPOENTE_MIN, ph2d_shake::EXPOENTE_MAX); // CLAMP-OK: a faixa é da lei
            }
        }
        mudou = true;
    }
    mudou
}

/// Aplica as edições da secção **SHAKE EMITTER**. Devolve `true` se alguma coisa mudou.
///
/// ⚠️ **O `Add` põe um [`ShakeSource::default`] e não um vazio**: uma fonte com raios a zero é uma
/// fonte que nunca abana, e o artista leria isso como *«o `+` não fez nada»*.
pub fn apply_emitter(sim: &mut SimWorld, edits: &[(u64, EE)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        let e = Entity::from_bits(*bits);
        let Some(mut em) = sim.world_mut().get_mut::<ShakeEmitter>(e) else {
            continue;
        };
        match edit {
            EE::Add => {
                if em.0.len() < ph2d_ecs::SHAKE_EMITTERS_MAX {
                    em.0.push(ShakeSource::default());
                    mudou = true;
                }
                continue;
            }
            EE::Remove(i) => {
                let i = usize::from(*i);
                if i < em.0.len() {
                    em.0.remove(i);
                    mudou = true;
                }
                continue;
            }
            _ => {}
        }
        // ⚠️ **O índice é conferido UMA vez** — um `get_mut` por braço seria a mesma pergunta em
        // seis sítios, e o dia em que um deles a esquecesse entregaria um `panic`.
        let (EE::On(i, _) | EE::De(i, _) | EE::Forca(i, _) | EE::Dentro(i, _) | EE::Fora(i, _)) =
            edit
        else {
            continue;
        };
        let Some(f) = em.0.get_mut(usize::from(*i)) else {
            continue;
        };
        match edit {
            EE::On(_, s) => f.on = s.clone(),
            EE::De(_, t) => f.de = SignalFrom::from_tag(*t),
            EE::Forca(_, v) => f.forca = *v,
            EE::Dentro(_, v) => f.dentro = *v,
            EE::Fora(_, v) => f.fora = *v,
            EE::Add | EE::Remove(_) => continue,
        }
        mudou = true;
    }
    mudou
}

#[cfg(test)]
#[path = "shake_inspector_tests.rs"]
mod tests;
