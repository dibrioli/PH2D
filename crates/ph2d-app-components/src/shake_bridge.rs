//! **A ponte do ABANÃO** (suplente #25) — o sinal vira trauma, o trauma vira um offset da vista.
//!
//! Irmã do [`crate::tween_bridge`] e do [`crate::hud_bridge`], e a que tem **menos** superfície de
//! todas: ⛔ **ela não escreve um único componente do documento.**
//!
//! # ⭐⭐⭐ Porque ela NÃO passa pelo ledger do `preview_drive`
//!
//! As irmãs escrevem `Transform`/`Sprite` e por isso têm de declarar o que conduziram, senão um
//! fade de meio segundo vira trinta passos de `Ctrl+Z`. Esta escreve **um `CameraShakeRuntime`**,
//! que não é registado, e **devolve** o offset a quem desenha. *Nada do que ela faz entra no
//! ficheiro, no undo ou na captura* — é a mesma lei que a wave das partículas (#18) escreveu um
//! nível acima (*«as partículas não são entidades, logo não passam pelo ledger»*).
//!
//! # ⚠️⚠️ ONDE ela corre, e porque é ANTES da câmera e não depois
//!
//! Ela é chamada da `fase_game_camera` da shell, **imediatamente antes** do `camera_2d::update`:
//!
//! * **antes**, porque a vista DESTE quadro tem de mostrar o trauma DESTE quadro — corrida depois,
//!   o abanão chegaria sempre um quadro atrasado, que é exactamente o defeito que o cabeçalho do
//!   `ph2d-runtime` nomeia por escrito para quem sai da janela do dreno;
//! * ⭐ e isso dá **latência ZERO ao contacto da física**, que é o caso de *«isto explodiu»*: os
//!   relógios do passo fixo correm antes da câmera, logo um `SignalOnHit` publicado neste quadro já
//!   está no outbox quando esta ponte o lê.
//!
//! ⚠️ **Um sinal publicado pela TABELA de acções chega no quadro seguinte**, e isso não é um furo:
//! é a janela de graça de um quadro que o [`ph2d_runtime::SignalOutbox`] dá a **todo** consumidor
//! (`older` + `newer`), e o cursor desta ponte não perde nada por ler cedo.
//!
//! # ⚠️ A distância é medida ao CENTRO DA VISTA, e não à pose da câmera
//!
//! Numa câmera que segue o jogador a pose não se mexe e o centro sim — *«quão longe a explosão está
//! daquilo que estou a ver»* é a pergunta do artista. ⛔ E isto **não são duas respostas**: quando o
//! `CameraRuntime` ainda não existe (o primeiro quadro), a resposta é a pose — que é exactamente
//! de onde o `ensure_runtime` da shell semeia o `center`.

use ph2d_ecs::{
    CameraRuntime, CameraShake, CameraShakeRuntime, Entity, ShakeEmitter, SimWorld, Transform,
};
use ph2d_runtime::{SignalOutbox, SignalReader};

/// O que a ponte fez neste quadro — o que um diagnóstico imprime e um gate lê.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AbanaoReport {
    /// O deslocamento a somar ao centro da vista, em metros.
    pub offset: [f32; 2],
    /// O trauma depois deste quadro (`0..1`).
    pub trauma: f32,
    /// ⭐ **Quantos impulsos CHEGARAM** — os que passaram a cerca de quem falou **e** a distância.
    /// ⚠️ Um sinal que casa o nome e é atenuado a zero **não** conta aqui: *«ninguém gritou»* e
    /// *«gritaram longe demais»* são dois factos, e o painel precisa dos dois.
    pub impulsos: usize,
    /// ⚠️ **Quantos casaram o nome e a cerca e chegaram a ZERO pela distância.** É esta coluna que
    /// separa *«o emissor não está ligado»* de *«o emissor está longe»*.
    pub longe: usize,
}

/// A posição de mundo de uma entidade — `None` quando ela não tem pose.
fn pose_de(sim: &SimWorld, e: Entity) -> Option<[f32; 2]> {
    let t = sim.world().get::<Transform>(e)?;
    Some([t.translation.x, t.translation.y])
}

/// Onde a vista está a olhar — ver o cabeçalho.
fn olhar_de(sim: &SimWorld, cam: Entity) -> [f32; 2] {
    sim.world()
        .get::<CameraRuntime>(cam)
        .map(|rt| rt.center)
        .or_else(|| pose_de(sim, cam))
        .unwrap_or([0.0, 0.0])
}

/// ⭐⭐⭐ **Um quadro de abanão.** Devolve o offset da vista e as colunas do diagnóstico.
///
/// ⚠️ **Zero custo quando não há nenhum:** sem câmera activa, ou sem [`CameraShake`] nela, a função
/// devolve `Default` **antes de ler um único sinal** — que é o caso de toda cena que já existe, e é
/// o que torna esta wave byte-idêntica no caminho de omissão.
///
/// ⚠️ **O cursor avança SEMPRE que a câmera treme**, e nunca quando ela não existe: um leitor que
/// lesse a saída numa cena sem câmera acumularia `missed` que ninguém olha.
pub fn drive_camera_shake(
    sim: &mut SimWorld,
    outbox: &SignalOutbox,
    reader: &mut SignalReader,
    dt: f32,
) -> AbanaoReport {
    let Some(cam) = ph2d_ecs::active_camera_of(sim.world_mut()) else {
        return AbanaoReport::default();
    };
    let Some(lei) = sim.world().get::<CameraShake>(cam).map(CameraShake::lei) else {
        return AbanaoReport::default();
    };
    let olhar = olhar_de(sim, cam);

    // ── Os IMPULSOS ──────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ **A colheita é feita ANTES de tocar no mundo**, porque o `read` segura `&outbox` e a
    // varredura dos emissores segura `&world` — e o passo seguinte precisa de `&mut`.
    let mut report = AbanaoReport::default();
    let mut impulso_total = 0.0f32;
    {
        let sinais: Vec<(std::sync::Arc<str>, Option<u64>)> = outbox
            .read(reader)
            .map(|s| (s.name.clone(), s.origin.quem().map(|b| b.0)))
            .collect();
        if !sinais.is_empty() {
            let mundo = sim.world_mut();
            let fontes: Vec<(Entity, ShakeEmitter)> = mundo
                .query::<(Entity, &ShakeEmitter)>()
                .iter(mundo)
                .map(|(e, em)| (e, em.clone()))
                .collect();
            for (e, em) in &fontes {
                for f in &em.0 {
                    // ⚠️ **Vazio é CALADO** — e o teste é aqui e não no laço dos sinais, porque um
                    // sinal com nome vazio não existe (o `publish` nunca o produz).
                    if f.on.trim().is_empty() {
                        continue;
                    }
                    for (nome, quem) in &sinais {
                        if nome.as_ref() != f.on.as_str() {
                            continue;
                        }
                        let falou = quem.map(Entity::from_bits);
                        if !f.de.deixa_passar(falou, *e) {
                            continue;
                        }
                        // ⭐ A distância sai da pose de QUEM FALOU quando o sinal a traz, e da pose
                        // do EMISSOR quando não traz. ⚠️ Não é a mesma coisa: uma bomba que ouve o
                        // estrondo de outra abana a partir de **onde a outra está**.
                        let onde = falou
                            .and_then(|q| pose_de(sim, q))
                            .or_else(|| pose_de(sim, *e));
                        let Some(onde) = onde else { continue };
                        let d =
                            ((onde[0] - olhar[0]).powi(2) + (onde[1] - olhar[1]).powi(2)).sqrt();
                        let a = ph2d_shake::atenuacao(d, f.dentro, f.fora);
                        if a > 0.0 {
                            impulso_total += f.forca * a;
                            report.impulsos += 1;
                        } else {
                            report.longe += 1;
                        }
                    }
                }
            }
        }
    }

    // ── O TRAUMA ─────────────────────────────────────────────────────────────────────────────
    let mundo = sim.world_mut();
    let mut rt = mundo
        .get::<CameraShakeRuntime>(cam)
        .copied()
        .unwrap_or_default();
    if impulso_total > 0.0 {
        rt.trauma = ph2d_shake::acumula(rt.trauma, impulso_total);
    }
    rt.trauma = ph2d_shake::decai(rt.trauma, lei.decaimento, dt);
    // ⭐ **O relógio só anda com trauma** — ver o doc do [`CameraShakeRuntime::t`]. ⚠️ E ele é
    // avançado DEPOIS do decaimento de propósito: um quadro em que o trauma acabou de chegar a zero
    // é o último em que a vista se mexe, e adiantar o relógio nele só mudaria a fase do seguinte.
    if rt.trauma > 0.0 {
        rt.t += dt;
        report.offset = ph2d_shake::deslocamento(&lei, rt.trauma, rt.t);
    } else {
        // ⚠️ **O relógio NÃO se repõe aqui**, e é o que faz dois estrondos seguidos não terem a
        // mesma cara. Quem o repõe é o `rewind_runtime`, que é onde *«renascer»* se decide.
        report.offset = [0.0, 0.0];
    }
    report.trauma = rt.trauma;
    if let Ok(mut ent) = mundo.get_entity_mut(cam) {
        ent.insert(rt);
    }
    report
}

#[cfg(test)]
#[path = "shake_bridge_tests.rs"]
mod tests;
