//! ⭐⭐⭐ **A APARÊNCIA DO OBJECTO, do lado da shell** (estudo 42 item 2, v19 do schema) — o que o
//! painel mostra da selecção, e o que um gesto dele escreve no documento.
//!
//! Duas propriedades, e as duas são da FORMA e não da tinta: a **opacidade** (a forma inteira
//! compõe-se uma vez e desvanece) e o **modo de mistura** (como ela se compõe com o que está por
//! baixo dela na pilha de z).
//!
//! # A lei da selecção múltipla: MOSTRA o primário, ESCREVE em todos
//!
//! ⚠️ É a lei que a fileira de verbos booleanos já paga, e ela veio de um report: *tocar num filho
//! selecciona o GRUPO*, então o sujeito de um readout é o **primário** — a forma que o artista
//! apontou. Escrever, ao contrário, é uma ordem sobre a selecção inteira, como o restyle.
//!
//! ⛔ **A alternativa — mostrar «misto» — foi recusada:** ela obriga um terceiro estado em cada
//! controlo (um slider sem valor, um chip sem nome) e o artista fica sem saber o que vai acontecer
//! ao arrastar. Mostrar o primário responde *o que este objecto é*, e é o que ele aponta.

use ph2d_vec_scene::{BlendMode, Opacity, VecPathId, VecScene};

/// **O que o painel mostra** — `None` sem forma na selecção (a seção some inteira).
pub(crate) fn published(
    scene: &VecScene,
    sel: &[VecPathId],
) -> Option<ph2d_panel_vector::state::Appearance> {
    let primeiro = *sel.first()?;
    let p = scene.paths().iter().find(|p| p.id == primeiro)?;
    Some(ph2d_panel_vector::state::Appearance {
        opacity: p.opacity.get(),
        blend: p.blend,
    })
}

/// **Escreve a opacidade** nas formas seleccionadas. Devolve se alguma coisa mudou.
///
/// ⚠️ **Devolver «mudou» não é higiene:** o undo desta casa regista por DIFF, e uma escrita que
/// repõe o mesmo valor é invisível para ele — mas o `bool` é o que deixa quem chama decidir sem
/// re-comparar o documento. E o `Opacity::new` prende a faixa numa porta só (⚠️ incluindo o `NaN`,
/// que de outra forma chegaria ao `push_layer` do Vello).
pub(crate) fn set_opacity(scene: &mut VecScene, sel: &[VecPathId], v: f32) -> bool {
    let novo = Opacity::new(v);
    let mut mudou = false;
    for id in sel {
        if let Some(p) = scene.path_mut(*id)
            && p.opacity != novo
        {
            p.opacity = novo;
            mudou = true;
        }
    }
    mudou
}

/// **Escreve o modo de mistura** nas formas seleccionadas.
///
/// ⚠️ O que chega do painel é o **código** do modo (`BlendMode::to_u8`), não a linha do popover:
/// a lista é derivada da tradução para o Vello, e reconstruí-la aqui para traduzir um índice seria
/// a segunda cópia dela — a que passa a discordar no primeiro modo novo.
pub(crate) fn set_blend(scene: &mut VecScene, sel: &[VecPathId], code: u8) -> bool {
    let novo = BlendMode::from_u8(code);
    let mut mudou = false;
    for id in sel {
        if let Some(p) = scene.path_mut(*id)
            && p.blend != novo
        {
            p.blend = novo;
            mudou = true;
        }
    }
    mudou
}

#[cfg(test)]
#[path = "vec_appearance_tests.rs"]
mod tests;
