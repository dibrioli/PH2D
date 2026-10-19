//! **A tabela das cenas do smoke do modo** (`PH2D_OBJECT_MODE_SMOKE`): o número da cena → as chaves
//! que o [`super::smoke_step`] lê, e a família das alças → o 1.º estágio dela. Irmão do
//! `vector_mode.rs` pelo tecto de 700 LOC; o corte é por ASSUNTO: aqui a TABELA, lá os passos.

/// O que uma cena liga — lido UMA vez, no estágio 0.
pub(super) struct Scene {
    /// O 1.º estágio: `1` = a cena corre (o Add), `9` = feito.
    pub want: u8,
    /// A cena 7 (os furos): duas linhas e o Weld no lugar do rectângulo e da elipse.
    pub weld: bool,
    /// As cenas da nova UI (D10 em diante): no layout *Vetor*.
    pub bars: bool,
    /// A cena 10 (F4): a paleta que ensina — `vector_mode_reach`.
    pub reach: bool,
    /// A cena 11 (F6): os dedos navegam — `vector_mode_nav`.
    pub nav: bool,
    /// A cena 14 (F8): enquadrar tudo — `vector_mode_frame`.
    pub frame: bool,
    /// A família das alças (cenas 15..32) — ver [`handles_first`]; `0` = nenhuma.
    pub handles: u8,
    /// A cena 12 (F7, o QuickShape) = `1`; a 13 (F7e, o Edit Shape) = `2`.
    pub quick: u8,
}

/// As chaves da cena `scene` (o valor da variável; `None` = ausente ou ilegível).
pub(super) fn of(scene: Option<&str>) -> Scene {
    // A ESCRITA da variável, e não o número: `"07"` não é a cena 7.
    let n = scene
        .and_then(|s| s.parse::<u8>().ok().filter(|n| n.to_string() == s))
        .unwrap_or(0);
    Scene {
        want: if matches!(n, 6 | 7 | 9..=34) { 1 } else { 9 },
        weld: n == 7,
        bars: matches!(n, 9..=34),
        reach: n == 10,
        nav: n == 11,
        frame: n == 14,
        handles: if (15..=34).contains(&n) { n - 14 } else { 0 },
        quick: match n {
            12 => 1,
            13 => 2,
            _ => 0,
        },
    }
}

/// O 1.º estágio de cada família das alças: 1 = a 15 (F9, 70–76 em `vector_mode_handles`) · 2 = a
/// 16 (F10, a estrela — 80–84 em `vector_mode_estrela`) · 3 = a 17 (F11, o traço — 90–93 em
/// `vector_mode_stroke`) · 4 = a 18 (F12 A, 100–103 em `vector_mode_tracos`) · 5 = a 19 (F12, as
/// setas — 110–120 em `vector_mode_setas`) · 6 = a 20 (F13, 130–136 em `vector_mode_seleciona`) ·
/// 7 = a 21 (F14, `vector_mode_quinas`) · 8 = a 22 (o Alinhar na ④, `vector_mode_alinhar`) · 9 = a
/// 23 (F16, `vector_mode_controle`) · 10 = a 24 (F16c, o tracejado, `vector_mode_tracejado`) ·
/// 11 e 12 = a 25 e a 26 (F17, as abas, `vector_mode_interface`) · 13 e 14 = a 27 e a 28 (F18,
/// começam pela 25 e seguem em `vector_mode_vestidos`) · 15 e 16 = a 29 e a 30 (F19, começam pela 25
/// e seguem em `vector_mode_le_a_forma`) · 17 e 18 = a 31 e a 32 (F20, `vector_mode_misto`) · 19 e
/// 20 = a 33 e a 34 (F21, `vector_mode_cor`).
/// `None` = fora da família.
pub(super) fn handles_first(handles: u8) -> Option<u8> {
    Some(match handles {
        0 => return None,
        1 => 70,
        2 => 80,
        3 => 90,
        4 => 100,
        5 => 110,
        6 => 130,
        7 => crate::vector_mode_quinas::FIRST,
        8 => crate::vector_mode_alinhar::FIRST,
        9 => crate::vector_mode_controle::FIRST,
        10 => crate::vector_mode_tracejado::FIRST,
        11..=16 => crate::vector_mode_interface::FIRST,
        17 | 18 => crate::vector_mode_misto::FIRST,
        19 | 20 => crate::vector_mode_cor::FIRST,
        _ => 70,
    })
}

#[cfg(test)]
#[path = "vector_mode_scenes_tests.rs"]
mod tests;
