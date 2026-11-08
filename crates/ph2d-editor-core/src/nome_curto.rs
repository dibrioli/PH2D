//! ⭐ **O nome curto** (dono 07/11, onda 9c: «limite de 12 caracteres para os nomes. se nome maior, ...»): um nome
//! de camada mostra no máximo [`NOME_DE_CAMADA_MAX`] caracteres e «…» depois; o inteiro fica guardado e vai na dica.
//!
//! Conta-se por GRAFEMA aproximado sem dependência nova: um `char` que só ESTENDE o anterior (marca combinante,
//! seletor de variação, tom de pele, etiqueta, ZWJ e o que vem depois dele) não abre caractere novo.

/// Quantos caracteres um nome de camada mostra antes do «…».
pub const NOME_DE_CAMADA_MAX: usize = 12;

/// `c` só estende o caractere anterior?
fn estende(c: char) -> bool {
    matches!(u32::from(c),
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF
        | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F | 0x200D)
}

/// Os índices de byte onde começa cada caractere visível.
fn inicios(s: &str) -> impl Iterator<Item = usize> + '_ {
    let mut depois_de_zwj = false;
    s.char_indices().filter_map(move |(i, c)| {
        let novo = i == 0 || !(estende(c) || depois_de_zwj);
        depois_de_zwj = c == '\u{200D}';
        novo.then_some(i)
    })
}

/// Quantos caracteres visíveis `s` tem.
#[must_use]
pub fn caracteres(s: &str) -> usize {
    inicios(s).count()
}

/// `s` com no máximo `max` caracteres e «…» depois, se passar.
#[must_use]
pub fn encurta(s: &str, max: usize) -> String {
    match inicios(s).nth(max) {
        Some(corte) => format!(
            "{}{}",
            &s[..corte],
            ph2d_i18n::tr("card.physics.name_ellipsis")
        ),
        None => s.to_owned(),
    }
}

/// O nome de camada como se mostra.
#[must_use]
pub fn nome_de_camada(s: &str) -> String {
    encurta(s, NOME_DE_CAMADA_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 12 cabem sem «…»; o 13.º corta com «…»; acento combinante e emoji com ZWJ contam um.
    #[test]
    fn doze_cabem_e_o_decimo_terceiro_corta_com_reticencias() {
        assert_eq!(nome_de_camada("ABCDEFGHIJKL"), "ABCDEFGHIJKL");
        assert_eq!(nome_de_camada("ABCDEFGHIJKLM"), "ABCDEFGHIJKL…");
        let acento = "e\u{301}".repeat(12);
        assert_eq!(caracteres(&acento), 12);
        assert_eq!(nome_de_camada(&acento), acento);
        let familia = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        assert_eq!(caracteres(familia), 1);
        let longo = format!("{}{familia}X", "a".repeat(11));
        assert_eq!(
            nome_de_camada(&longo),
            format!("{}{familia}…", "a".repeat(11))
        );
        assert_eq!(nome_de_camada(""), "");
    }
}
