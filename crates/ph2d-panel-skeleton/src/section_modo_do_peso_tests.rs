//! ⭐⭐⭐ **OS GATES DO MODO DO PINCEL DE PESO NO PAINEL** (F29, ordem do dono de 2026-09-19).
//!
//! ⚠️ **O que eles medem é o que uma chamada de pintura esconde:** qual segmento acende, se a
//! fileira da direcção chega a ser pintada, e que rótulo o número carrega. As três decisões vivem
//! em funções de propósito — *uma lei que só existe dentro de uma chamada de pintura não tem como
//! ser medida*.

use super::{pinta_a_direccao, rotulo_do_numero, segmentos_do_modo};
use ph2d_tool_vector::WeightMode;

/// ⭐⭐⭐ **O ÍNDICE QUE A SHELL PUBLICA ACENDE O SEGMENTO CERTO** — a mesma lei (e o mesmo gate) da
/// fileira da direcção, uma wave depois.
///
/// ⚠️ **As três metades:** o segmento publicado acende, o outro **não** (dois acesos leem-se como
/// «nenhum escolhido»), e o par id↔rótulo sai emparelhado — *duas listas na mesma ordem trocam-se
/// uma sem a outra, e o sintoma é um botão que faz o contrário do que diz*.
#[test]
fn o_indice_publicado_acende_o_modo_do_pincel() {
    for publicado in 0..2 {
        let fileira = segmentos_do_modo(publicado);
        for (i, (id, chave, on)) in fileira.iter().enumerate() {
            assert_eq!(
                *id,
                crate::ids::VECTOR_BONE_WEIGHT_MODE_IDS[i],
                "a fileira e a lista que o `populate` regista deixaram de concordar na ORDEM"
            );
            assert_eq!(
                *chave,
                [
                    "panel.vector.bone.weight.mode.cumulative",
                    "panel.vector.bone.weight.mode.absolute"
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

/// ⭐⭐⭐ **A FILEIRA DA DIRECÇÃO SOME NO MODO ABSOLUTO** — ordem do dono: *«neste modo os botões Add
/// e Subtract ficam inactivos»*.
///
/// ⚠️ ***Inactivo* aqui é AUSENTE**, que é a leitura mais forte: não há botão para carregar. É a lei
/// da casa (*esconde-se o que se pode*), e o precedente é o `Density` da escultura.
///
/// ⛔ **A metade do CONTROLO é obrigatória:** sem ela, uma lei que escondesse a fileira SEMPRE
/// passaria — e o modo cumulativo ficaria sem escolher o lado.
#[test]
fn a_fileira_da_direccao_some_no_modo_absoluto() {
    assert!(
        !pinta_a_direccao(WeightMode::Absolute.indice()),
        "no modo ABSOLUTO a fileira da direccao continua pintada — os dois botoes nao tem sujeito \
         ali, e um controlo que nao faz nada e' a especie de morto que o §5.0 nomeia"
    );
    assert!(
        pinta_a_direccao(WeightMode::Cumulative.indice()),
        "no modo CUMULATIVO a fileira sumiu — o artista deixou de poder escolher o lado"
    );
}

/// ⭐⭐⭐ **O RÓTULO DO NÚMERO SEGUE O MODO** — *Brush Strength* no cumulativo, *Target Weight* no
/// absoluto.
///
/// ⚠️⚠️ **O mesmo controlo, dois significados, e é a ordem do dono que o exige** (*«o valor de Brush
/// Strength é posto imediatamente no osso»*): ali o número deixa de ser *quanto empurrar* e passa a
/// ser *que valor pôr*. ⛔ Dois campos seriam duas superfícies sobre um valor — a armadilha que os
/// três chips do `Detail` da escultura pagaram —, e um rótulo fixo seria um controlo a mentir sobre
/// metade do curso.
///
/// ⚠️ **As chaves têm de ser DIFERENTES**, senão isto passa sobre um rótulo fixo.
#[test]
fn o_rotulo_do_numero_segue_o_modo() {
    let cumul = rotulo_do_numero(WeightMode::Cumulative.indice());
    let abs = rotulo_do_numero(WeightMode::Absolute.indice());
    assert_eq!(cumul, "panel.vector.bone.weight.amount");
    assert_eq!(abs, "panel.vector.bone.weight.target");
    assert_ne!(
        cumul, abs,
        "o rotulo do numero deixou de seguir o modo — ele mente em metade do curso"
    );
    // ⭐ E os dois textos existem de facto: uma chave sem entrada pinta o NOME dela na tela.
    for chave in [cumul, abs] {
        assert_ne!(
            ph2d_i18n::tr(chave),
            chave,
            "a chave `{chave}` nao tem texto — o painel pintaria o nome dela"
        );
    }
}
