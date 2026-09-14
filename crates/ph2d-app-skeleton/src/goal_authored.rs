//! ⭐⭐⭐ **A POSE AUTORADA DE UMA CORRENTE** — irmão do [`super::goal`] pelo tecto de LOC, cortado
//! por responsabilidade: ali mora *quem resolve a restrição*, aqui *o que o artista desenhou*.
//!
//! ⛔⛔ **Por que ela existe, e só para o modo MISTO** (ordem do dono, 2026-09-14: *«cada osso
//! mantém sua direção inicial»*): o `Ccw` e o `Cw` levam o lado num **campo do documento**, logo
//! sobrevivem a tudo; o misto lê o lado de **cada junta** da pose que lhe chega, e a pose que chega
//! é a que o solver deixou no quadro anterior. ⚠️ Enquanto o alvo está ao alcance isso é estável (o
//! modo é um ponto fixo, e há gate) — mas **um arrasto que leve o alvo para fora do alcance deita a
//! corrente na recta**, e uma corrente recta não tem lado nenhum para ler: os lados evaporavam-se,
//! para sempre, e nada na tela o dizia.
//!
//! ⇒ *«inicial» é o que está no DOCUMENTO*, e quem o guarda enquanto o motor escreve por cima é o
//! [`ph2d_preview_drive`] — *pré-visualização não é autoria*. Esta porta reconstrói a corrente a
//! partir das rotações autoradas e devolve a pose de que os lados se lêem.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// A rotação LOCAL autorada de um osso — a do documento se o motor a deslocou, a viva se não.
fn rotacao_autorada(sim: &SimWorld, preview: &PreviewDrive, e: Entity) -> Option<f64> {
    let viva = f64::from(sim.world().get::<Transform>(e)?.rotation);
    match preview.authored(e.to_bits(), Driver::SolverPose) {
        Some(Driven::SolverPose(t)) => Some(f64::from(t.rotation)),
        _ => Some(viva),
    }
}

/// ⭐⭐⭐ **A corrente como o artista a desenhou**, nas mesmas coordenadas em que a viva chega.
///
/// ⭐⭐ **O truque é que a junta É a rotação local do osso de baixo.** Num encadeamento
/// pai→filho o ângulo de mundo acumula (`mundo_i = mundo_{i−1} + local_i`), logo o ângulo da junta
/// entre os ossos `i−1` e `i` — a diferença dos dois ângulos de mundo — **é** o `local_i`. ⇒ não é
/// preciso pose de mundo autorada nenhuma, nem percorrer pais: bastam as rotações locais.
///
/// ⚠️ **A RAIZ da corrente fica onde está**, e o ângulo de partida é lido da pose VIVA menos a
/// rotação local viva do 1.º osso — isso dá o ângulo do PAI, que não é governado e por isso é o
/// mesmo nas duas poses. ⛔ Começar do zero poria a corrente autorada noutra direcção e os sinais
/// sairiam de uma pose que ninguém desenhou.
///
/// ⚠️ Os COMPRIMENTOS são os vivos, de propósito: eles não são o que este modo defende, e um osso
/// esticado por outro motor tem de continuar esticado.
pub(crate) fn authored_joints(
    sim: &SimWorld,
    preview: &PreviewDrive,
    corrente: &[Entity],
    vivas: &[[f64; 2]],
    comps: &[f64],
) -> Option<Vec<[f64; 2]>> {
    if corrente.is_empty() || vivas.len() != corrente.len() + 1 || comps.len() != corrente.len() {
        return None;
    }
    let viva0 = (vivas[1][1] - vivas[0][1]).atan2(vivas[1][0] - vivas[0][0]);
    let local0 = f64::from(sim.world().get::<Transform>(corrente[0])?.rotation);
    let mut ang = viva0 - local0; // o ângulo de mundo do PAI da corrente
    let mut juntas = Vec::with_capacity(vivas.len());
    juntas.push(vivas[0]);
    for (i, &e) in corrente.iter().enumerate() {
        ang += rotacao_autorada(sim, preview, e)?;
        let (sen, cos) = ang.sin_cos();
        let u = juntas[i];
        juntas.push([u[0] + comps[i] * cos, u[1] + comps[i] * sen]);
    }
    Some(juntas)
}
