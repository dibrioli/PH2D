//! **AS LEITURAS DERIVADAS DO RETRATO** — perguntas cuja resposta sai do
//! [`super::Sculpt3dSnapshot`] e que não são estado nenhum.
//!
//! ⚠️ Irmão (`#[path]`) do [`super`], cortado pelo tecto de LOC do painel e
//! pelo ASSUNTO — a mesma forma dos `state_modes`, `state_channel` e
//! `state_luz`. *O `state.rs` é o MODELO; «qual chip está aceso?» é uma
//! LEITURA dele, e as duas coisas crescem por razões diferentes.*

use super::Sculpt3dSnapshot;

/// **Qual chip da fileira de padrão está aceso**, dado o retrato.
///
/// `0` é o pincel liso, `1..=9` são os `Alpha::ALL` deslocados de um, e o
/// último é o slot de IMAGEM.
///
/// ⚠️ **Ela é `pub` para o GATE poder perguntar ao produto.** O `event` não a
/// chama — ele resolve a pergunta INVERSA (*este índice arma o quê?*) —, então
/// isto não é uma porta compartilhada, é o retrato responder *«qual está
/// aceso?»* em vez de um teste re-derivar a aritmética por conta própria. Um
/// gate com a sua terceira cópia concordaria consigo mesmo enquanto o painel
/// pintasse outra coisa, que é o oráculo-espelho que esta casa recusa.
#[must_use]
pub fn alpha_chip_index(snap: &Sculpt3dSnapshot) -> usize {
    match snap.ui.brush.alpha.as_ref() {
        None => 0,
        Some(a) if a.is_image() => ph2d_sculpt3d::Alpha::ALL.len() + 1,
        Some(a) => ph2d_sculpt3d::Alpha::ALL
            .iter()
            .position(|x| x == a)
            .map_or(0, |i| i + 1),
    }
}
