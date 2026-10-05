//! Os gates das leis puras da secção SKELETON (saíram do `section.rs` pelo tecto de 600 do painel).

use super::aceso;

/// ⭐⭐⭐ **NADA ACESO ATÉ O ARTISTA CARREGAR** — a ordem do dono, palavra por palavra.
///
/// ⛔ *«Se não há ossos no mundo nenhum botão fica selecionado até o usuário apertar Create»*
/// (2026-09-09). ⚠️ **A metade que importa é a PRIMEIRA:** um default aceso faria o painel
/// afirmar um verbo que a ferramenta não tem armado — e o arrasto no canvas faria outra coisa
/// que a fileira diz.
#[test]
fn nothing_is_lit_until_the_artist_arms_it() {
    assert!(
        !aceso(None, 0) && !aceso(None, 1),
        "um segmento nasceu aceso sem nada armado"
    );
    assert!(aceso(Some(0), 0) && !aceso(Some(0), 1), "*Create* armado");
    assert!(
        !aceso(Some(1), 0) && aceso(Some(1), 1),
        "*Transform* armado"
    );
}

/// ⭐⭐⭐ **O ÍNDICE QUE A SHELL PUBLICA ACENDE O SEGMENTO CERTO** — a direcção do pincel de
/// peso (ordem do dono, 2026-09-19).
///
/// ⚠️ **As três metades:** o segmento publicado acende, o outro **não** (senão os dois acesos
/// leem-se como «nenhum escolhido»), e a fileira sai da lista de ids na ordem dela — *um par
/// trocado acenderia o `Add` quando a ferramenta está a tirar peso, e o artista veria o
/// contrário do que o próximo arrasto faz.*
#[test]
fn o_indice_publicado_acende_o_lado_do_pincel() {
    for publicado in 0..2 {
        let fileira = super::segmentos_da_direccao(publicado);
        for (i, (id, chave, on)) in fileira.iter().enumerate() {
            assert_eq!(
                *id,
                crate::ids::VECTOR_BONE_WEIGHT_DIR_IDS[i],
                "a fileira e a lista que o `populate` regista deixaram de concordar na ORDEM — \
                 um segmento passa a carregar o id do outro"
            );
            // ⭐ E o RÓTULO segue o id: sem isto, o botao que diz «Add» pode mandar «Subtract».
            assert_eq!(
                *chave,
                [
                    "panel.vector.bone.weight.add",
                    "panel.vector.bone.weight.subtract"
                ][i],
                "o rotulo do segmento {i} deixou de emparelhar com o id dele"
            );
            assert_eq!(
                *on,
                i == publicado,
                "publicado {publicado}: o segmento {i} devia estar {}",
                if i == publicado { "ACESO" } else { "apagado" }
            );
        }
    }
}
