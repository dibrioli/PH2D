//! ⭐ G9 (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.7): um acorde (Ctrl+M, Ctrl+W) nunca cai num atalho
//! de letra solta. Só `std`: o gate `despacho::ctrl_letra_nunca_cai_no_atalho_solto` inclui-o.

/// As letras com um braço de atalho SOLTO no despacho (o gate conta-as na fonte).
pub(crate) const LETRAS_SOLTAS: [char; 9] = ['A', 'B', 'F', 'G', 'K', 'L', 'M', 'T', 'W'];

/// Um acorde (`cmd` = Ctrl ou Super, ou o Alt) sobre a letra de um atalho solto (`keycode` do
/// `crate::keymap::winit_to_input_keycode`: ASCII maiúsculo): nada acontece.
pub(crate) fn acorde_em_letra_solta(keycode: Option<u32>, cmd: bool, alt: bool) -> bool {
    (cmd || alt)
        && keycode
            .and_then(char::from_u32)
            .is_some_and(|l| LETRAS_SOLTAS.contains(&l))
}
