//! **O número da alça da curva do pulo** (ficha GEF 04, 12/11): o readout escreve-se com a conta do
//! campo do cartão (`format_number` sobre o valor na unidade do projecto), arredondado à resolução
//! que o ecrã afirma — px inteiro, ou centímetro em metros. Com o ápice a menos de meio pixel da
//! Jump Height (a lei modela os passos do mundo), os dois textos são o mesmo.

use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::widget::format_number;

/// Um comprimento (ou velocidade) do mundo como o readout o mostra: «200 px», «2.5 m».
#[must_use]
pub fn numero_da_alca(metros: f32, unidade: DisplayUnit, ppm: f32) -> String {
    let v = unidade.from_meters_f64(f64::from(metros), ppm);
    let escala = match unidade {
        DisplayUnit::Pixels => 1.0,
        DisplayUnit::Meters => 100.0, // LITERAL-PX-OK: centímetros por metro (unidade)
    };
    format!(
        "{} {}",
        format_number((v * escala).round() / escala),
        unidade.suffix()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_physics_ecs::jump_feel::Feel;
    use ph2d_physics_ecs::{PhysicsSettings, PlatformPlayer, arc_world};

    /// O texto do campo Jump Height do cartão (`unidade_mundo`: `no_ecra` + `format_number`).
    fn campo(h: f32, u: DisplayUnit, ppm: f32) -> String {
        format!(
            "{} {}",
            format_number(u.from_meters_f64(f64::from(h), ppm)),
            u.suffix()
        )
    }

    /// ⭐ Dono 12/11, «os números não são idênticos»: a alça no topo da curva e o campo Jump Height
    /// dizem o MESMO número, nas três sensações e no padrão, de 0,5 a 2,5 m, em px e em metros.
    #[test]
    fn o_numero_da_alca_e_o_do_campo() {
        let w = arc_world(&PhysicsSettings::default(), 1.0);
        let mut falhas = Vec::new();
        for feel in [
            None,
            Some(Feel::Floaty),
            Some(Feel::Snappy),
            Some(Feel::Heavy),
        ] {
            for h in [0.5_f32, 1.0, 2.0, 2.5] {
                let mut p = PlatformPlayer::default();
                if let Some(f) = feel {
                    p.set_feel(f);
                }
                p.jump_height = h;
                let apice = p.jump_arcs(w).full.apex[1];
                for (u, ppm) in [
                    (DisplayUnit::Pixels, 100.0),
                    (DisplayUnit::Pixels, 64.0),
                    (DisplayUnit::Meters, 100.0),
                ] {
                    let (a, c) = (numero_da_alca(apice, u, ppm), campo(h, u, ppm));
                    if a != c {
                        falhas.push(format!("{feel:?} h={h} {u:?}@{ppm}: alça {a} × campo {c}"));
                    }
                }
            }
        }
        assert!(falhas.is_empty(), "a alça e o campo divergem: {falhas:?}");
    }
}
