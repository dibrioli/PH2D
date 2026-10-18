//! ⭐⭐⭐ **A APARÊNCIA DO OBJECTO, do lado da shell** (estudo 42 item 2, v19 do schema) — o que o
//! painel mostra da selecção, e o que um gesto dele escreve no documento.
//!
//! Duas propriedades, e as duas são da FORMA e não da tinta: a **opacidade** (a forma inteira
//! compõe-se uma vez e desvanece) e o **modo de mistura** (como ela se compõe com o que está por
//! baixo dela na pilha de z).
//!
//! # A lei da selecção múltipla: MOSTRA «misto» onde diferem, ESCREVE o campo tocado em todas
//!
//! ⭐ **Plano 31 da UI nova (decisão do dono, 18/10)** — revoga o «mostra o primário» que aqui
//! estava. O primário mentia sobre as outras formas, e com o restyle a gravar o estilo INTEIRO ele
//! APAGAVA: mexer na Width de duas formas pintava as duas com a cor do primário. O terceiro estado
//! já existia nos widgets (amostra «misto», número em branco do BulkSelect do Inspector), e a
//! objeção registada — *«o artista fica sem saber o que vai acontecer ao arrastar»* — cai com a
//! escrita POR CAMPO: arrastar dá a todas esse valor, e nada mais muda. O slider fica no primário.
//! A leitura mora em `paint_stack::published` (`mixed_opacity`, `mixed_blend`).

use ph2d_vec_scene::{BlendMode, Opacity, VecPathId, VecScene};

/// ⛔ **A publicação da vista MUDOU-SE para o [`crate::paint_stack`]** (v20, estudo 42 item 4).
///
/// Ela lia duas propriedades; a vista passou a ter três (as duas mais a PILHA), e duas funções a
/// montar o mesmo `Appearance` seriam a segunda porta pela qual as duas metades divergem — uma
/// publicaria a pilha e a outra não, conforme quem chamasse.
///
/// **Escreve a opacidade** nas formas seleccionadas. Devolve se alguma coisa mudou.
///
/// ⚠️ **Devolver «mudou» não é higiene:** o undo desta casa regista por DIFF, e uma escrita que
/// repõe o mesmo valor é invisível para ele — mas o `bool` é o que deixa quem chama decidir sem
/// re-comparar o documento. E o `Opacity::new` prende a faixa numa porta só (⚠️ incluindo o `NaN`,
/// que de outra forma chegaria ao `push_layer` do Vello).
pub fn set_opacity(scene: &mut VecScene, sel: &[VecPathId], v: f32) -> bool {
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
pub fn set_blend(scene: &mut VecScene, sel: &[VecPathId], code: u8) -> bool {
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
#[path = "appearance_tests.rs"]
mod tests;
