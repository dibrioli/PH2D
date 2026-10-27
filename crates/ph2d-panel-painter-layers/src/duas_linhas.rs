//! ⭐ **O nome em DUAS linhas** (Óleo Real W3 K9): os nomes do material do Real Oil são os do Blender por ordem do
//! dono (27/10: Metallic · Roughness · IOR Level · Coat · Coat Roughness · Sheen · Subsurface · Subsurface Radius) —
//! nem se encurtam nem se traduzem. Quando um deles não cabe na coluna da secção (o mínimo do dock, `220`; o
//! `Subsurface Radius` ainda a `245`), parte-se num ESPAÇO e pinta-se em duas linhas, na mesma coluna e alinhado à
//! direita, com a caixa ao meio das duas — em vez de cortar. ⛔ Não é o «nome por cima da caixa» que o dono recusou
//! (14/09): o nome continua à ESQUERDA do valor.
//!
//! Uma decisão, dois leitores: o pintor ([`crate::card::card_row`]) e o censo das elisões
//! (`cada_nome_deste_painel_cabe_na_coluna_da_seccao`) perguntam aqui.

use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;

/// As chaves cujo nome se parte em duas linhas quando não cabe — o material do Real Oil, pela ordem do cartão.
pub const EM_DUAS_LINHAS: &[&str] = &crate::paint_wetpaint::MATERIAL;

/// As duas linhas de `nome` (a chave `chave`) numa coluna de `col` px: `None` quando cabe numa só, quando a chave
/// não se parte, ou quando nenhum espaço dá duas metades que caibam (aí o nome corta, como qualquer outro). A 1.ª
/// linha é a mais longa que cabe.
#[must_use]
pub fn partido<'a>(
    ts: &mut TextSystem,
    chave: &str,
    nome: &'a str,
    col: f32,
) -> Option<(&'a str, &'a str)> {
    if !EM_DUAS_LINHAS.contains(&chave) {
        return None;
    }
    let fonte = TypeToken::Sm.px();
    if ts.prefix_width(nome, fonte) <= col {
        return None;
    }
    let mut espacos: Vec<usize> = nome.match_indices(' ').map(|(i, _)| i).collect();
    espacos.reverse();
    espacos.into_iter().find_map(|i| {
        let (a, b) = (&nome[..i], &nome[i + 1..]);
        (ts.prefix_width(a, fonte) <= col && ts.prefix_width(b, fonte) <= col).then_some((a, b))
    })
}
