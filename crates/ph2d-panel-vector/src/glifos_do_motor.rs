//! ⭐ **O GLIFO de cada tipo que o motor publica** (F43, layout do Figma): o «+» e as linhas das
//! secções Effects e Filters mostram o ícone do tipo ao lado do nome.
//!
//! Par do [`crate::nomes_do_motor`] e pela mesma lei: o rótulo INGLÊS do motor é um identificador
//! (o braço de um `match`), lido no ponto único onde o painel recebe a tabela — antes da tradução.
//! O gate `every_kind_the_engine_publishes_has_its_own_glyph` varre `PathEffect::KINDS` e
//! `FxOp::SPECS` e exige um glifo PRÓPRIO por tipo (o `None` nunca acontece, e dois tipos não
//! partilham desenho).

use ph2d_editor_core::icons::IconId;

/// O glifo de um tipo de EFEITO (`PathEffect::KINDS`), pelo rótulo do motor.
#[must_use]
pub fn efeito(rotulo: &str) -> Option<IconId> {
    Some(match rotulo {
        "Trim Path" => IconId::FxTrimPath,
        "Zig Zag" => IconId::FxZigzag,
        "Repeater" => IconId::FxRepeater,
        "Pucker & Bloat" => IconId::FxPuckerBloat,
        "Arc" => IconId::FxArc,
        "Arc Upper" => IconId::FxArcUpper,
        "Arc Lower" => IconId::FxArcLower,
        "Bulge" => IconId::FxBulge,
        "Flag" => IconId::FxFlag,
        "Wave" => IconId::FxWave,
        "Squeeze" => IconId::FxSqueeze,
        "Fisheye" => IconId::FxFisheye,
        "Rise" => IconId::FxRise,
        "Falloff Radial" => IconId::FxFalloffRadial,
        "Falloff Linear" => IconId::FxFalloffLinear,
        "Falloff Rect" => IconId::FxFalloffRect,
        "Falloff Sweep" => IconId::FxFalloffSweep,
        "Twist" => IconId::FxTwist,
        "Knot" => IconId::FxKnot,
        "Sketch" => IconId::FxSketch,
        "Hatch" => IconId::FxHatch,
        _ => return None,
    })
}

/// O glifo de um tipo de FILTRO (`FxOp::SPECS`), pelo rótulo do motor.
#[must_use]
pub fn filtro(rotulo: &str) -> Option<IconId> {
    Some(match rotulo {
        "Blur" => IconId::FilterBlur,
        "Outer Glow" => IconId::FilterOuterGlow,
        "Drop Shadow" => IconId::FilterDropShadow,
        "Inner Shadow" => IconId::FilterInnerShadow,
        "Inner Glow" => IconId::FilterInnerGlow,
        "Outline" => IconId::FilterOutline,
        "Feather" => IconId::FilterFeather,
        "Bevel" => IconId::FilterBevel,
        "Color Overlay" => IconId::FilterColorOverlay,
        "Turbulence" => IconId::FilterTurbulence,
        "Grow / Shrink" => IconId::FilterGrowShrink,
        "Color Adjust" => IconId::FilterColorAdjust,
        "Duotone" => IconId::FilterDuotone,
        "Luma to Alpha" => IconId::FilterLumaAlpha,
        "Gradient Map" => IconId::FilterGradientMap,
        _ => return None,
    })
}

/// O glifo a PINTAR: o do tipo, ou o genérico de um tipo que a tabela não conhece (as tabelas
/// sintéticas dos testes de costura). A cobertura real é o gate abaixo, contra o motor.
#[must_use]
pub fn ou_generico(g: Option<IconId>) -> IconId {
    g.unwrap_or(IconId::WandSparkles)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **Cada tipo publicado pelo motor tem o SEU glifo** — nenhum sem desenho, nenhum par a
    /// partilhar, e nenhum glifo de efeito igual a um de filtro. *Mutação: trocar o braço de
    /// `"Wave"` para `FxFlag` ⇒ VERMELHO.*
    #[test]
    fn every_kind_the_engine_publishes_has_its_own_glyph() {
        let fx: Vec<IconId> = ph2d_vec_scene::effect::PathEffect::KINDS
            .iter()
            .map(|k| efeito(k).unwrap_or_else(|| panic!("efeito sem glifo: {k}")))
            .collect();
        let fl: Vec<IconId> = ph2d_fx_op::FxOp::SPECS
            .iter()
            .map(|s| filtro(s.name).unwrap_or_else(|| panic!("filtro sem glifo: {}", s.name)))
            .collect();
        assert_eq!(
            fx.len(),
            ph2d_tool_vector::ids::MAX_FX_KINDS,
            "controlo: os 21 efeitos"
        );
        assert_eq!(
            fl.len(),
            crate::ids::MAX_FILTER_KINDS,
            "controlo: os 15 filtros"
        );
        let todos: Vec<IconId> = fx.iter().chain(fl.iter()).copied().collect();
        for (i, a) in todos.iter().enumerate() {
            for b in &todos[i + 1..] {
                assert_ne!(a, b, "dois tipos partilham o glifo {}", a.slug());
            }
        }
        assert!(
            !todos.contains(&ou_generico(None)),
            "o genérico é de quem não tem glifo"
        );
    }
}
