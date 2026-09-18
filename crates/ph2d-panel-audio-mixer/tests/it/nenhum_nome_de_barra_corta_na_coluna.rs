//! ⭐⭐⭐ **O NOME DE UMA BARRA DO MASTER CABE NA COLUNA DELE — nas DUAS línguas.**
//!
//! ⛔⛔ **Medido em 2026-09-18, e o defeito já shipava EM INGLÊS.** A coluna era o literal
//! `FX_LABEL_W = 32,0 px`, e com o sistema de texto REAL (`Xs`):
//!
//! | rótulo | inglês | idioma de teste |
//! |---|---:|---:|
//! | `Low` · `Mid` · `Fbk` | `18,8`–`21,3` | `29,9`–`31,5` |
//! | `Size` | `20,6` | **`33,8`** |
//! | `High` | `24,9` | **`38,2`** |
//! | `Time` | `25,9` | **`39,1`** |
//! | **`Depth`** | **`32,3`** | **`48,5`** |
//! | **`Return`** | **`35,9`** | **`55,1`** |
//!
//! ⇒ **dois cortados hoje, na língua em que o app ship**, e **cinco de oito** sob a tensão do
//! idioma de teste — que é o p90 medido de cinco traduções reais (`pseudo.rs`). A foto do dono de
//! 18/09 mostrava a fileira inteira em `[…]`.
//!
//! ⭐ **A cura não é um número maior: é a coluna DERIVAR dos nomes** que ela vai pintar
//! ([`ph2d_panel_audio_mixer::paint_widgets`]), com piso (o `32` de sempre, para a coluna não
//! encolher onde os nomes são curtos) e tecto (metade da linha — *acima disso o artista lê mais do
//! que arrasta*; ⚠️ medido, ele **não morde** em largura nenhuma que o dock permita).
//!
//! # ⚠️ A régua mede as DUAS línguas, e a razão é que elas reprovam por motivos diferentes
//!
//! O inglês é o que ship; o idioma de teste é a **previsão** de uma tradução. Um gate só sobre o
//! inglês deixaria passar a coluna que parte no dia da primeira língua — que é exactamente o que
//! este painel fez.

use ph2d_i18n::Idioma;
use ph2d_panel_audio_mixer::paint_widgets::{FX_ROW_KEYS, coluna_dos_nomes_em};
use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;

/// A escada de larguras que o dock permite — a mesma do gate irmão do painel de camadas.
const LARGURAS: &[f32] = &[220.0, 245.0, 304.0, 720.0];

fn conteudo(painel: f32) -> f32 {
    painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX
}

#[test]
fn nenhum_nome_de_barra_corta_na_coluna() {
    let mut ts = TextSystem::new();
    let fonte = TypeToken::Xs.px();
    // ⛔ Piso de população: com a lista vazia a coluna cai no piso e tudo «cabe».
    assert!(
        FX_ROW_KEYS.len() >= 12,
        "as barras do master são 8 mais os 4 sub-buses, e a lista tem {}",
        FX_ROW_KEYS.len()
    );

    let mut cortados = Vec::new();
    for &painel in LARGURAS {
        let w = conteudo(painel);
        for idioma in [Idioma::Ingles, Idioma::Teste] {
            // ⚠️⚠️ **A coluna é a DESSA língua** — comparar o texto deformado com uma coluna
            //    calculada em inglês acusa o produto CERTO, e foi a 1.ª redacção deste gate.
            let col = coluna_dos_nomes_em(idioma, &mut ts, w);
            for k in FX_ROW_KEYS {
                let texto = ph2d_i18n::tr_em(idioma, k);
                let largura = ts.prefix_width(texto, fonte);
                if largura > col {
                    cortados.push(format!(
                        "painel {painel} · {idioma:?} · {texto:?} mede {largura:.1} numa coluna de {col:.1}"
                    ));
                }
            }
        }
    }
    assert!(
        cortados.is_empty(),
        "estes nomes de barra saem CORTADOS — a coluna deixou de descrever o que ela pinta:\n  {}",
        cortados.join("\n  ")
    );
}

/// ⚠️ **E o CONTROLO: a coluna de antes reprovaria.**
///
/// Sem esta metade, uma coluna que por acaso ficasse larga (um tecto generoso, um piso alto)
/// passaria o gate acima sem que a MEDIÇÃO dos nomes tivesse alguma coisa a ver com isso — *um
/// gate que passa com a lei desligada não afirma a lei*.
#[test]
fn a_coluna_literal_de_antes_cortava_dois_nomes_ja_em_ingles() {
    let mut ts = TextSystem::new();
    let fonte = TypeToken::Xs.px();
    const ANTES: f32 = 32.0;
    let cortados: Vec<&str> = FX_ROW_KEYS
        .iter()
        .map(|k| ph2d_i18n::tr_em(Idioma::Ingles, k))
        .filter(|t| ts.prefix_width(t, fonte) > ANTES)
        .collect();
    assert_eq!(
        cortados,
        vec!["Depth", "Return"],
        "a fixtura desta wave é a coluna literal de 32 px a cortar DOIS nomes em inglês — se ela \
         deixar de os cortar, os rótulos mudaram e a medição do cabeçalho tem de ser refeita"
    );
}
