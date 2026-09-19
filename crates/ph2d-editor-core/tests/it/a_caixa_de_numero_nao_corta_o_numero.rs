//! ⛔⛔⛔ **UM NÚMERO QUE SE LÊ `0.…` MENTE SOBRE SI PRÓPRIO.**
//!
//! Achado pela varredura das elisões em 2026-09-18 (`ph2d-panel-registry-init`): o painel de
//! *Design Tokens* pintava `0.500` e `1.500` como **`0.…`** e **`1.…`**, e o de *Color
//! Equalization* pintava `+0.00 EV` como `+0.00…`.
//!
//! # A conta, que é a lei
//!
//! | grandeza | valor |
//! |---|---:|
//! | caixa do chip | `56,0 px` |
//! | coluna do stepper (`(h·0,6).clamp(16,22)`) | `16,0` |
//! | área de texto | `40,0` |
//! | **orçamento que ela pedia** (`label_budget`) | **`24,0`** |
//! | `0.500` na fonte do chip | **`31,0`** (`28,2` com as fontes do sistema) |
//!
//! ⛔ **A área de texto já está recuada da borda direita pela coluna INTEIRA do stepper**, e pedir
//! por cima dela o orçamento de um RÓTULO conta essa borda **duas vezes**: `40 − 8 − 8 = 24`.
//!
//! ⇒ a caixa de número conta **uma** borda (`crate::paint::paint_text_centered_com_orcamento`), e
//! com o texto centrado sobram `Md/2 = Xs` de cada lado — que é o recuo que ela **já usa** na
//! vertical para a selecção. Orçamento `24,0 → 32,0`, e os três números passam a ler-se inteiros.
//!
//! ⚠️ **O que NÃO muda:** a caixa continua do mesmo tamanho, o stepper continua na coluna dele, e
//! um valor genuinamente longo (`-141.881`, `42,6 px`) continua elidido — *a reticência é mais
//! honesta que um recorte a meio de um dígito, que se lê como outro número*.

use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::widget::paint_number_chip;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;

/// A caixa que o painel de Design Tokens usa (`CHIP_W = 56`) e a altura de uma linha.
const CAIXA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 56.0,
    h: 22.0,
};

fn pinta(display: &str) -> Vec<ph2d_editor_core::text_elide::elisao::Medido> {
    let mut ts = TextSystem::without_system_fonts();
    let mut cena = ph2d_vector::VectorScene::new();
    elisao::medindo(|| {
        paint_number_chip(
            CAIXA,
            ph2d_editor_core::widget::TextInputState::Normal,
            0.5,
            Some(display),
            None,
            0,
            None,
            &mut cena,
            &mut ts,
            Theme::default(),
        );
    })
    .1
}

/// ⭐⭐⭐ **O número sai INTEIRO — medido pelo CENSO, no caminho do produto.**
///
/// ⚠️ A régua é o instrumento que achou o defeito, e não uma conta escrita ao lado: *um gate e a
/// cura que ele confere nunca podem partilhar a suposição* — a lei que esta linha pagou em 18/09,
/// quando o gate da coluna do mixer media a mesma grandeza errada que a cura.
#[test]
fn a_caixa_de_numero_mostra_o_numero_todo() {
    // ⚠️ O corpus são os valores que a varredura MEDIU no produto (`0.500`, `1.500`) mais o
    //    vizinho da mesma família — e não valores inventados: a fronteira do que cabe é a
    //    LARGURA DA CAIXA que cada painel escolhe, e ela está medida no teste abaixo.
    for display in ["0.500", "1.500", "0.000"] {
        let medidos = pinta(display);
        let cortado: Vec<_> = medidos.iter().filter(|m| !m.coube()).collect();
        assert!(
            cortado.is_empty(),
            "a caixa de número cortou {display:?}: {cortado:?}"
        );
    }
}

/// ⛔ **E o CONTROLO: um valor genuinamente longo continua a ser elidido.**
///
/// Sem esta metade, um orçamento infinito passaria o gate acima e o número transbordaria para a
/// coluna das setas — *uma cura que apaga a lei não é uma cura*.
/// ⚠️ **Uma pintura centrada deixa DOIS registos** — o corte contra o orçamento da caixa e a
/// re-medição do que sobrou contra a largura do rect —, logo a régua procura o corte em vez de
/// assumir que há um registo só.
#[test]
fn um_valor_longo_de_mais_continua_a_avisar_que_foi_cortado() {
    let medidos = pinta("-141.881");
    let cortes: Vec<_> = medidos.iter().filter(|m| !m.coube()).collect();
    assert_eq!(cortes.len(), 1, "{medidos:?}");
    assert!(
        cortes[0].pintado.ends_with('…'),
        "o valor longo tem de sair com reticências, nunca recortado a meio de um dígito: {medidos:?}"
    );
}

/// ⛔⛔ **A FRONTEIRA que fica, com o número: um valor NEGATIVO de três decimais não cabe numa
/// caixa de `56 px`, nem depois da cura.**
///
/// `-0.500` mede `36,1 px` contra um orçamento de `32,0` — e a alavanca aqui **não** é o respiro
/// (já está no mínimo honesto): é a **largura da caixa**, que é do painel que a escolhe. Esta
/// metade existe para o dia em que alguém a alargar: ela reprova, e a cura é apagá-la.
#[test]
fn um_valor_negativo_de_tres_decimais_ainda_nao_cabe_em_56px() {
    let medidos = pinta("-0.500");
    assert!(
        medidos.iter().any(|m| !m.coube()),
        "`-0.500` passou a caber — a caixa alargou, e esta fronteira já não descreve nada: \
         {medidos:?}"
    );
}

/// ⭐⭐ **A conta da cura, escrita como ela é lida:** a área de texto menos UMA borda.
///
/// ⚠️ Ela mora aqui porque é o que separa esta caixa de um rótulo — e porque o número `32,0` do
/// cabeçalho tem de ser **derivado** dos tokens, senão ele envelhece no dia em que o `Spacing::Md`
/// mudar.
#[test]
fn o_respiro_da_caixa_de_numero_conta_uma_borda_e_nao_duas() {
    // ⭐⭐ **As DUAS grandezas lêem-se do PRODUTO, e não de uma segunda cópia da conta:** uma
    //   pintura centrada que ELIDE deixa dois registos — o corte contra o **orçamento** e a
    //   re-medição do que sobrou contra a **área de texto**. O menor é um, o maior é a outra.
    let medidos = pinta("-141.881");
    assert_eq!(
        medidos.len(),
        2,
        "a régua precisa dos dois registos (orçamento e área): {medidos:?}"
    );
    let orcamento = medidos
        .iter()
        .map(|m| m.largura)
        .fold(f32::INFINITY, f32::min);
    let area = medidos.iter().map(|m| m.largura).fold(0.0_f32, f32::max);

    // ⛔ METADE DE BAIXO: a caixa não paga o respiro de um RÓTULO (a borda do stepper contada
    //   duas vezes). Se voltar a pagá-lo, o número volta a sair `0.…`.
    let de_rotulo = ph2d_editor_core::paint::label_budget(area);
    assert!(
        orcamento > de_rotulo,
        "a caixa está a pagar o respiro de um rótulo ({orcamento} contra {de_rotulo}) — a borda do \
         stepper está contada duas vezes"
    );
    // ⛔⛔ METADE DE CIMA, e ela nasceu de uma mutação SOBREVIVENTE: um orçamento igual à área
    //   inteira passa a metade de baixo e encosta o número na borda da caixa. *Uma cura só é uma
    //   cura entre dois limites.*
    assert!(
        orcamento < area,
        "a caixa deixou de ter respiro nenhum ({orcamento} de {area}) — o número encosta na borda"
    );

    let mut ts = TextSystem::without_system_fonts();
    let fonte = ph2d_tokens::TypeToken::Xs.px();
    assert!(
        ts.prefix_width("0.500", fonte) <= orcamento,
        "o caso MEDIDO do report tem de caber: {:.2} px contra {orcamento:.1}",
        ts.prefix_width("0.500", fonte)
    );
}
