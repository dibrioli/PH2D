//! **Os TEXTOS do texto vetorial nos canais** — a família da fonte e o conteúdo
//! (`canal_vetor::TEXTO_FAMILIA`/`TEXTO_CONTEUDO`). O canal é `f32`: o texto entra pelo índice em
//! [`TextosDosCanais`] (`0` = nenhum). Irmão de [`crate::canais_campos`], à parte porque precisa
//! da tabela: ler ACRESCENTA um texto novo, escrever resolve o índice — um índice que a tabela não
//! tem não escreve nada.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{FieldKey, TextosDosCanais, VecShape};

/// O campo `kk` é um texto do texto vetorial?
#[must_use]
pub fn e_de_texto(kk: FieldKey) -> bool {
    kk.familia == VETOR && matches!(kk.campo, c::TEXTO_FAMILIA | c::TEXTO_CONTEUDO)
}

/// Os campos de texto que a forma tem (só um `VecShape::Text`).
#[must_use]
pub fn campos(shape: Option<&VecShape>) -> Vec<FieldKey> {
    match shape {
        Some(VecShape::Text(_)) => [c::TEXTO_FAMILIA, c::TEXTO_CONTEUDO]
            .map(|campo| FieldKey::new(VETOR, campo, 0, 0))
            .to_vec(),
        _ => Vec::new(),
    }
}

/// O índice do texto de `kk` (acrescentando-o à tabela se é novo).
pub fn ler(shape: Option<&VecShape>, kk: FieldKey, textos: &mut TextosDosCanais) -> Option<f32> {
    let Some(VecShape::Text(t)) = shape else {
        return None;
    };
    if !e_de_texto(kk) || kk.i != 0 || kk.j != 0 {
        return None;
    }
    Some(match kk.campo {
        c::TEXTO_FAMILIA => textos.indice(t.family.as_deref()),
        _ => textos.indice(Some(&t.text)),
    })
}

/// Aplica o texto do índice `v`. A família `0` volta à fonte por omissão; o conteúdo nunca é
/// «nenhum» (um índice sem texto não escreve).
pub fn escrever(shape: Option<&mut VecShape>, kk: FieldKey, v: f32, textos: &TextosDosCanais) {
    let Some(VecShape::Text(t)) = shape else {
        return;
    };
    if !e_de_texto(kk) || kk.i != 0 || kk.j != 0 {
        return;
    }
    let novo = textos.texto(v);
    match kk.campo {
        c::TEXTO_FAMILIA => {
            if novo.is_some() || v.round() < 1.0 {
                let f = novo.map(str::to_owned);
                if t.family != f {
                    t.family = f;
                }
            }
        }
        _ => {
            if let Some(s) = novo
                && t.text != s
            {
                s.clone_into(&mut t.text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::VecTextParams;

    fn k(campo: u16) -> FieldKey {
        FieldKey::new(VETOR, campo, 0, 0)
    }

    fn texto() -> VecShape {
        VecShape::Text(VecTextParams {
            text: "Olá".to_owned(),
            origin: [0.0, 0.0],
            family: Some("Inter".to_owned()),
            size: 24.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: 0,
            axes: Vec::new(),
            wrap_width: None,
        })
    }

    /// Ler acrescenta à tabela e escrever o que se leu é identidade; um índice novo troca o texto;
    /// a família `0` volta à fonte por omissão; um índice desconhecido não escreve.
    #[test]
    fn o_texto_le_o_indice_e_escreve_o_texto_dele() {
        let mut textos = TextosDosCanais::default();
        let mut sh = texto();
        let fam = ler(Some(&sh), k(c::TEXTO_FAMILIA), &mut textos).unwrap();
        let con = ler(Some(&sh), k(c::TEXTO_CONTEUDO), &mut textos).unwrap();
        assert_eq!(textos.texto(fam), Some("Inter"));
        assert_eq!(textos.texto(con), Some("Olá"));
        let antes = sh.clone();
        escrever(Some(&mut sh), k(c::TEXTO_FAMILIA), fam, &textos);
        escrever(Some(&mut sh), k(c::TEXTO_CONTEUDO), con, &textos);
        assert_eq!(sh, antes);

        let roboto = textos.indice(Some("Roboto"));
        let adeus = textos.indice(Some("Adeus"));
        escrever(Some(&mut sh), k(c::TEXTO_FAMILIA), roboto, &textos);
        escrever(Some(&mut sh), k(c::TEXTO_CONTEUDO), adeus, &textos);
        let VecShape::Text(t) = &sh else { panic!() };
        assert_eq!(
            (t.family.as_deref(), t.text.as_str()),
            (Some("Roboto"), "Adeus")
        );

        escrever(Some(&mut sh), k(c::TEXTO_CONTEUDO), 99.0, &textos);
        escrever(Some(&mut sh), k(c::TEXTO_CONTEUDO), 0.0, &textos);
        escrever(Some(&mut sh), k(c::TEXTO_FAMILIA), 0.0, &textos);
        let VecShape::Text(t) = &sh else { panic!() };
        assert_eq!((t.family.as_deref(), t.text.as_str()), (None, "Adeus"));
        assert_eq!(campos(Some(&sh)).len(), 2);
        assert!(campos(None).is_empty());
        assert_eq!(ler(None, k(c::TEXTO_FAMILIA), &mut textos), None);
    }
}
