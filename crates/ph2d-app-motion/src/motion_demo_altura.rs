//! **A altura de câmara com que uma cena de demo do Motion nasce** — uma porta de BISSECÇÃO
//! (`PH2D_DEMO_ALTURA=<metros>`), irmã do `PH2D_CARIMBO_CORNER`. Ver o doc da [`altura_semeada`].

/// **A ALTURA DE CÂMERA com que uma cena de demo nasce** — `PH2D_DEMO_ALTURA=<metros>`.
///
/// ⛔⛔ **Isto é um INSTRUMENTO DE BISSECÇÃO e não um knob de produto** — o irmão do
/// `PH2D_CARIMBO_CORNER`, pela mesma razão: o report do dono de 2026-09-23 (*«estrelas com corner
/// radius de 1 provoca queda de raw»*) depende do ZOOM (o recorte por câmara e o LOD da forma
/// decidem quantas estrelas vão CRISP à placa), e o gesto que o produz é a roda do rato — que
/// **não é alcançável de uma corrida sem interface** (o XTest é ignorado na Xwayland virtual e o
/// `ydotool` mexe no rato REAL do dono). Sem esta porta a única régua do regime de zoom era eu
/// clicar, que esta casa não faz.
///
/// ⚠️ **Valor ausente, ilegível, não finito ou `<= 0` ⇒ `None`, e `None` é o arranque de sempre
/// AO BIT** — quem lê não toca na câmara. *Uma porta de bissecção que mude a cena quando ninguém
/// lhe toca deixa de bissectar coisa nenhuma.* A faixa é a da câmara
/// ([`ph2d_ecs::camera_2d::CAMERA_MIN_HEIGHT_WORLD`] .. `MAX`), a mesma que o zoom do artista
/// respeita — uma altura fora dela seria uma vista que a roda não alcança.
#[must_use]
pub fn altura_semeada() -> Option<f32> {
    altura_por(std::env::var("PH2D_DEMO_ALTURA").ok().as_deref())
}

/// A LEI da porta acima, **pura** — o que a variável significa, sem a ler (*um gate que lê o
/// ambiente mede a MÁQUINA em que corre*).
#[must_use]
pub fn altura_por(valor: Option<&str>) -> Option<f32> {
    valor
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| {
            v.clamp(
                ph2d_ecs::camera_2d::CAMERA_MIN_HEIGHT_WORLD,
                ph2d_ecs::camera_2d::CAMERA_MAX_HEIGHT_WORLD,
            )
        })
}

#[cfg(test)]
#[path = "motion_demo_altura_tests.rs"]
mod tests;
