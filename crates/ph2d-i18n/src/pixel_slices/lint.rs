//! **As palavras da janela LINT do PIXEL ART, P13 U-Lint** — as regras do linter e a lista de avisos.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel_lint.title" => "Lint",
        "pixel.lint.sec_rules" => "Rules",
        "pixel.lint.sec_issues" => "Issues",
        "pixel.lint.rule.orphan" => "Orphan pixels",
        "pixel.lint.rule.doubles" => "Doubles",
        "pixel.lint.rule.jaggies" => "Jaggies",
        "pixel.lint.rule.banding" => "Banding",
        "pixel.lint.rule.pillow" => "Pillow shading",
        "pixel.lint.rule.too_many_colors" => "Too many colors",
        "pixel.lint.rule.orphan.tip" => "A lone pixel with nothing of its color around it.",
        "pixel.lint.rule.doubles.tip" => {
            "An L-shaped corner in a 1-pixel line where a diagonal step is enough."
        }
        "pixel.lint.rule.jaggies.tip" => "Line steps of uneven length that break the slope.",
        "pixel.lint.rule.banding.tip" => {
            "A 1-pixel band between two parallel edges that blurs them into a staircase."
        }
        "pixel.lint.rule.pillow.tip" => {
            "Shading that only follows the outline instead of the light."
        }
        "pixel.lint.rule.too_many_colors.tip" => "More colors than the color limit allows.",
        "pixel.lint.severity.low" => "Low",
        "pixel.lint.severity.medium" => "Medium",
        "pixel.lint.severity.high" => "High",
        "pixel.lint.color_limit" => "Color limit",
        "pixel.lint.none" => "No issues found",
        "pixel.lint.no_document" => "Open a Pixel Sprite in Draw to check it.",
        "pixel.lint.count" => "issues",
        _ => return None,
    })
}
