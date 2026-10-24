//! **Várias imagens no mesmo objecto Macro** (W19, `docs/Macros/17_W19_plano.md` §2.6): cada trilha do
//! Painter de um `Macro.doc` toca no seu palco, chaveado por `(dono, índice da trilha do Painter)`. Duas
//! trilhas que pintam a MESMA imagem (o artista voltou a ela) mostram, no instante `t`, a mais recente que
//! já começou; antes de qualquer começar, a 1.ª.

use super::Lido;
use ph2d_macro::MacroTime;
use std::collections::BTreeMap;

/// A chave de um palco: o objecto Macro (bits) e o índice entre as trilhas do Painter do doc dele.
pub(super) type ChaveDoPalco = (u64, usize);

/// As trilhas do Painter de um `Macro.doc` (com as edições das etapas).
pub(super) fn do_painter(bytes: &[u8]) -> Vec<ph2d_macro::Track> {
    let familia = ph2d_tool_painter::macro_track::FAMILIA;
    ph2d_macro::codec::read(bytes).map_or_else(
        |_| Vec::new(),
        |doc| {
            doc.tracks
                .into_iter()
                .filter(|t| t.family == familia)
                .collect()
        },
    )
}

/// A `k`-ésima trilha do Painter de um `Macro.doc`.
pub(super) fn trilha_do_painter(bytes: &[u8], k: usize) -> Option<ph2d_macro::Track> {
    do_painter(bytes).into_iter().nth(k)
}

/// Os palcos do objecto `bits`.
pub(super) fn do_dono(
    lidos: &mut BTreeMap<ChaveDoPalco, Lido>,
    bits: u64,
) -> impl Iterator<Item = (&ChaveDoPalco, &mut Lido)> {
    lidos.range_mut((bits, 0)..=(bits, usize::MAX))
}

/// **Quem se mostra** entre os palcos de `bits` que tocam em `t`: por imagem, o de início mais tardio
/// que já começou (`≤ t`); nenhum começou = o de menor índice.
pub(super) fn escolhe_os_mostrados(
    lidos: &mut BTreeMap<ChaveDoPalco, Lido>,
    bits: u64,
    t: MacroTime,
) {
    // Por imagem: (índice escolhido, já começou, início).
    let mut melhor: BTreeMap<u64, (usize, bool, MacroTime)> = BTreeMap::new();
    for (&(_, k), l) in do_dono(lidos, bits) {
        let Ok(Some(p)) = &l.palco else {
            continue;
        };
        let Some(s) = p.sprite else {
            continue;
        };
        let inicio = p.em_cena.inicio();
        let comecou = inicio <= t;
        let entra = melhor
            .get(&s)
            .is_none_or(|&(_, c, i)| comecou && (!c || inicio >= i));
        if entra {
            melhor.insert(s, (k, comecou, inicio));
        }
    }
    for (&(_, k), l) in do_dono(lidos, bits) {
        if let Ok(Some(p)) = &mut l.palco {
            p.mostra = p
                .sprite
                .is_none_or(|s| melhor.get(&s).map(|m| m.0) == Some(k));
        }
    }
}
