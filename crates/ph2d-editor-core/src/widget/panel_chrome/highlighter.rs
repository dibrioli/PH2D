//! A paleta de marcador — ver o `mod highlighter` no [`super`].

/// Highlighter palette used by user-placed notes + section-outline
/// markings inside panel bodies, selectable from the right-click context menu.
///
/// Indices 0..4 map to Yellow / Pink / Green / Blue / Orange (os tons de marcador); 5..8 são as
/// quatro VIVAS (ordem do dono, 2026-09-30: *«mais 4 cores vivas para outline — vermelho, azul,
/// verde e amarelo»*). ⚠️ As vivas são escolhidas para o texto ESCURO da nota continuar legível
/// sobre elas (o pintor da nota não troca a cor do texto): as quatro ficam acima de `4,5:1`
/// contra `#212121`.
///
/// ⚠️ **Acrescenta-se no FIM, nunca a meio** — o índice é o que a nota e o contorno guardam, e uma
/// cor inserida a meio trocaria a cor de toda nota já pintada.
/// Hoisted to `panel_chrome` so panel crates can paint notes
/// without reaching into `ph2d_editor::screens::hero::context_menu_overlay`.
pub const HIGHLIGHTER_RGBA: [[u8; 4]; 9] = [
    // LITERAL-COLOR-OK: highlighter palette — user-pickable note + outline colors.
    [0xFF, 0xF5, 0x9D, 0xFF], // yellow
    [0xF8, 0xBB, 0xD0, 0xFF], // pink
    [0xC8, 0xE6, 0xC9, 0xFF], // green
    [0xBB, 0xDE, 0xFB, 0xFF], // blue
    [0xFF, 0xE0, 0xB2, 0xFF], // orange
    [0xFF, 0x3B, 0x30, 0xFF], // vivid red
    [0x3D, 0x8B, 0xFF, 0xFF], // vivid blue
    [0x22, 0xC5, 0x5E, 0xFF], // vivid green
    [0xFF, 0xD6, 0x0A, 0xFF], // vivid yellow
];

/// ⭐ **A cor de marcador `idx`** — a porta que o contorno e a nota leem. Um índice fora da paleta
/// (um ficheiro de uma versão com mais cores) cai na ÚLTIMA, nunca estoura.
#[must_use]
pub fn highlighter_rgba(idx: u8) -> [u8; 4] {
    HIGHLIGHTER_RGBA[usize::from(idx).min(HIGHLIGHTER_RGBA.len() - 1)]
}
