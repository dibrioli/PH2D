//! ⭐⭐⭐ **AS QUATRO FORMAS DE UMA LINHA PARTILHAM A REPARTIÇÃO** — e a escada é impressa.
//!
//! ⛔⛔⛔ **Auditoria de 2026-09-19, sobre o report *«widgets embolados»*.** O painel tinha o rótulo
//! em três sítios diferentes ao mesmo tempo, e nenhum gate o via: os que existiam mediam o que a
//! linha DESPACHA, nunca onde ela é pousada.
//!
//! ⚠️ **A régua é o PRODUTO** — a [`super::colunas_da_fileira`], que é a mesma porta que os quatro
//! pintores chamam. ⛔ Um número escrito aqui mediria uma coluna que o painel já não tem.

use super::{casas_da_linha, colunas_da_fileira, passo_da_linha};
use crate::state::ParamRow;
use ph2d_editor_core::widget::{
    NUMBER_INPUT_MIN_W_PX, property_label_col_w, slider_with_chip_chip_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_field::Bound;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

/// ⭐ **O curso que o dock permite** — e a largura do dono como AMOSTRA DATADA, nunca uma cerca.
///
/// ⚠️ A lição está escrita no gate irmão do Inspector: *a largura do artista é um ESTADO* — o
/// `~/.ph2d/layout.txt` dele dizia `dock_w_right = 296,89` em 2026-09-19, e pregá-la seria escolher
/// um ponto que envelhece em horas.
const LARGURAS: &[f32] = &[220.0, 245.0, 296.89, 304.0, 400.0, 720.0];

fn linha(painel: f32) -> (f32, f32) {
    let x = ph2d_tokens::PANEL_HEAD_PAD_PX;
    (x, (painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX).max(0.0))
}

/// Onde a fileira VIVA pousa o valor — pela porta do produto, não por uma conta paralela.
fn valor_da_viva(x: f32, w: f32, y: f32) -> Rect {
    slider_with_chip_chip_rect(
        Rect::new(x, y, w, ROW_H_PX),
        property_label_col_w(x, w),
        NUMBER_INPUT_MIN_W_PX,
    )
}

/// ⭐⭐⭐ **A COLUNA DO VALOR DE UMA LINHA TRAVADA É A DA LINHA VIVA** — em toda a largura do dock.
///
/// ⛔⛔ **Medido ANTES da cura** (a travada usava `property_label_col_w` como coluna do valor): o
/// desvio **não era constante** — ele cresce com a largura, porque a coluna do formulário é uma
/// FRACÇÃO da linha e a da caixa única é um piso FIXO.
///
/// | painel | valor da VIVA | valor da TRAVADA (antes) | desvio |
/// |---:|---:|---:|---:|
/// | `220,00` | `108,00` | `118,00` | `+10,00` |
/// | `245,00` | `133,00` | `130,50` | `−2,50` |
/// | `296,89` | `184,89` | `156,45` | `−28,44` |
/// | `304,00` | `192,00` | `160,00` | `−32,00` |
/// | `400,00` | `288,00` | `208,00` | `−80,00` |
/// | `720,00` | `608,00` | `368,00` | `−240,00` |
///
/// ⚠️⚠️ **E o `16 px` que a auditoria publicou é o ecrã CLÁSSICO** (`PH2D_UI_NEW=0`), onde a caixa
/// única não corre: ali o pintor lê o `label_w`, e `w − Lc` menos `Lc` dá `2 × Spacing::Md`. *Uma
/// medição feita sobre a fórmula, e não sobre o pintor, mede a aparência que o artista não tem.*
#[test]
fn as_quatro_formas_de_linha_partilham_a_coluna() {
    println!("  painel |  VIVA valor.x |  colunas_da_fileira.x |  nome.x |  nome.w");
    for painel in LARGURAS {
        let (x, w) = linha(*painel);
        let viva = valor_da_viva(x, w, 0.0);
        let (nome, coluna) = colunas_da_fileira(x, w, 0.0);
        println!(
            "{painel:>8.2} | {:>13.2} | {:>21.2} | {:>7.2} | {:>7.2}",
            viva.x, coluna.x, nome.x, nome.w
        );
        assert!(
            (coluna.x - viva.x).abs() < 0.01 && (coluna.w - viva.w).abs() < 0.01,
            "a {painel:.2}: a coluna do valor das formas travada/amostra ({:.2}+{:.2}) deixou de ser \
             a da fileira viva ({:.2}+{:.2}) — atravessar a trava volta a mover o número de sítio",
            coluna.x,
            coluna.w,
            viva.x,
            viva.w
        );
        // ⚠️ **A metade que o `paint_property_label` apagava:** o nome acabava EXACTAMENTE onde o
        // valor começa (vão `0,00 px` por construção), e é isso que a foto do dono mostra como
        // «embolado». O vão tem de ser o `pad` da caixa única.
        let vao = coluna.x - (nome.x + nome.w);
        assert!(
            (vao - Spacing::Md.px()).abs() < 0.01,
            "a {painel:.2} o vão entre o fim do nome e o início do valor é {vao:.2} px e o pad da \
             caixa única é {:.2}",
            Spacing::Md.px()
        );
    }
}

/// ⛔ **CONTROLO: a coluna do formulário NÃO é a da caixa única** — sem isto o gate acima ficaria
/// verde se alguém trocasse as duas portas por uma só, e a escada de cima deixaria de descrever
/// nada.
#[test]
fn a_coluna_do_formulario_e_uma_grandeza_diferente() {
    let (x, w) = linha(720.0);
    let (_, coluna) = colunas_da_fileira(x, w, 0.0);
    let formulario = x + (w - property_label_col_w(x, w));
    assert!(
        (coluna.x - formulario).abs() > 100.0,
        "a `720` a coluna da caixa única ({:.2}) e a do formulário ({formulario:.2}) passaram a \
         concordar — a escada do gate irmão mede uma diferença que já não existe",
        coluna.x
    );
}

fn fileira(valor: f32, lo: f32, teto: f32, integral: bool) -> ParamRow {
    ParamRow {
        entity: 0,
        param: ph2d_field::Param::Dim(0),
        key: "field.dim.round",
        value: valor,
        lo,
        bound: Bound::Soft(teto),
        inert: None,
        integral,
        choices: &[],
        swatch: None,
        section: None,
        subject: None,
    }
}

/// ⭐⭐⭐ **ATRAVESSAR A TRAVA NÃO MUDA A PRECISÃO DO NÚMERO.**
///
/// ⛔⛔ **Medido:** a `paint_fact` formatava com `decimals_for_step(1.0)` = **uma** casa, e o
/// `Coat IOR` de `1,6` da foto do dono é a prova do lado que se vê. Numa faixa `0..1` o passo é
/// `0,01` ⇒ **três** casas, e `0,375` lia-se `0.4`.
///
/// ⚠️ **A fixtura é uma linha de faixa APERTADA**: numa faixa larga as duas leis dariam o mesmo
/// número de casas, e o gate ficava verde sobre o defeito.
#[test]
fn a_precisao_de_uma_linha_nao_muda_ao_travar() {
    // `0..1`: passo `0,01` ⇒ `ceil(2) + 1 = 3` casas.
    let fino = fileira(0.375, 0.0, 1.0, false);
    assert_eq!(casas_da_linha(&fino), 3, "passo {}", passo_da_linha(&fino));
    assert_eq!(
        format!("{:.d$}", f64::from(fino.value), d = casas_da_linha(&fino)),
        "0.375",
        "a fileira travada volta a arredondar o número que a viva mostra inteiro"
    );
    // ⛔ **O controlo**: a lei antiga (`decimals_for_step(1.0)`) dá `1` casa nesta mesma linha.
    assert_eq!(
        crate::paint::decimals_for_step(1.0),
        1,
        "a lei que o defeito usava deixou de dar uma casa — a tabela deste gate já não descreve o \
         que ele cura"
    );
    // ⚠️ Uma linha INTEIRA continua sem casas: meia cópia não existe.
    let contagem = fileira(3.0, 1.0, 64.0, true);
    assert_eq!(casas_da_linha(&contagem), 0);
}

/// ⭐⭐⭐ **O RÓTULO MAIS LARGO DESTE PAINEL CABE NA COLUNA, À LARGURA DO DONO.**
///
/// ⛔⛔ **Report de 2026-09-19:** `Subsurface Radius Scale` media `142,79 px` a `TypeToken::Sm`
/// contra uma coluna de `138,45` na largura real do dono ⇒ **cortava por `4,34 px`**, e ele é uma
/// fileira TRAVADA (o `Thin Walled` desliga as quatro do raio da subsuperfície).
///
/// ⭐⭐ **A cura da coluna resolveu-o de graça, e por isso a cura PRESCRITA não entrou:** o
/// empréstimo por secção (`property_label_col_w_for`) daria `{EMPRESTIMO}` e a repartição da caixa
/// única dá **mais** — ela não reparte a linha ao meio, ela dá ao nome *tudo menos a coluna do
/// campo*. Os números estão impressos por este gate.
///
/// ⚠️ **A barra é a largura do DONO e não a de omissão** — foi essa a lição do gate irmão do
/// Inspector, e a que o deixava verde sobre o que ele via.
#[test]
fn o_rotulo_mais_largo_cabe_na_coluna_a_largura_do_dono() {
    const O_MAIS_LARGO: &str = "Subsurface Radius Scale";
    let mut ts = TextSystem::new();
    let fonte = TypeToken::Sm.px();
    let medido = ts.prefix_width(O_MAIS_LARGO, fonte);
    println!(
        "  painel |  coluna da CAIXA |  empréstimo por secção | «{O_MAIS_LARGO}» = {medido:.2}"
    );
    for painel in LARGURAS {
        let (x, w) = linha(*painel);
        let (nome, _) = colunas_da_fileira(x, w, 0.0);
        let emprestimo =
            ph2d_editor_core::widget::property_label_col_w_for(x, w, Some(medido), None);
        println!("{painel:>8.2} | {:>16.2} | {emprestimo:>22.2} |", nome.w);
        if *painel >= 296.89 {
            assert!(
                nome.w >= medido,
                "a {painel:.2} o rótulo mais largo do painel ({medido:.2} px) não cabe na coluna \
                 ({:.2} px) — ele volta a cortar",
                nome.w
            );
        }
    }
}
