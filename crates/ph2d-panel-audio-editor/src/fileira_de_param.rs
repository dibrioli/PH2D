//! ⭐⭐⭐ **A FILEIRA DE PARAMETRO DESTE PAINEL — uma porta, cinco consumidores.**
//!
//! ⛔⛔⛔ **Report do dono, 2026-09-19, com foto:** *«por que esses sliders nao sao colocados
//! no padrao do app?»* (a seccao *Effects*) e, na resposta, a pergunta que fecha o assunto:
//! *«qual proxima etapa?»*. Medido, a MESMA forma estava escrita **cinco** vezes neste painel
//! — *Effects* · *Loop* · *Delivery* · *Spectral* · *Variation* — cada uma com um
//! rotulo CENTRADO numa linha e a pista nua na linha de BAIXO.
//!
//! ⚠️⚠️ **Isso poe o nome POR CIMA do controlo**, que e exactamente o que o dono recusou em
//! 2026-09-14 (*«Label acima do campo numerico! Muito ruim!»*) — a frase que fez o
//! Inspector inteiro mudar de lei. E as cinco sao anteriores a essa lei: nenhuma foi convertida.
//!
//! ⭐ **Converter as cinco a mao seria escrever a lei cinco vezes.** *Uma lei escrita em dois
//! sitios ainda nao e uma lei — so uma PORTA e*, e esta casa ja pagou essa por tres vezes
//! (a caneta eliptica, a coluna de nomes da roldana, o par `M | S` do mixer).
//!
//! # ⚠️ O que a porta decide, e o que ela NAO decide
//!
//! Ela decide a GEOMETRIA (a caixa unica da casa) e o RECORTE (o envelope de rolagem deste
//! painel). Ela **nao** decide o valor nem a unidade: quem os tem e o motor, e eles entram como
//! `norm` e `valor`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::clipped_hits::ClippedHits;

/// ⭐⭐⭐ **Uma fileira de parametro na caixa unica.** Devolve o `y` seguinte.
///
/// - `norm` e a fraccao `0..1` que o preenchimento mostra;
/// - `valor` e a leitura ja formatada pelo motor (`20.0 kHz`, `-6.0 dB`); `None` deixa a porta
///   formatar a propria fraccao;
/// - ⚠️⚠️ **`vivo = false` pinta a fileira e NAO a regista** (`NodeId(0)`, que os dois
///   aspectos da casa tratam como *pintado e nao agarravel*). E a recusa que duas destas seccoes
///   ja praticavam por escrito — *um slider que parece vivo e nao faz nada e uma mentira*
///   — e agora ela e a MESMA em todas.
///   ⛔ **Diferenca DECLARADA na aparencia CLASSICA** (`PH2D_UI_NEW=0`, que nao e o caminho
///   de omissao): la a fileira inerte desenha o polegar parado onde antes desenhava uma pista nua.
///   Ele nunca acende (o par visual de um id zero e o neutro) e continua a nao responder; o que se
///   ganha em troca e o NUMERO, que a pista nua nao tinha.
///
/// ⚠️ **O chip vai sempre a `NodeId(0)`:** o valor destes parametros e uma string do motor,
/// com uma curva propria por efeito, e torna-lo editavel obriga a ler a string de volta pela
/// curva. Divida NOMEADA, nao esquecimento.
///
/// ⚠️ **A altura e a que a porta da casa DEVOLVE**, nunca uma constante: numa coluna
/// estreita ela promove o rotulo para uma fileira propria, e um `y` fixo poria a linha seguinte
/// por cima.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fileira_de_param(
    y: f32,
    x: f32,
    w: f32,
    label: &str,
    norm: f32,
    valor: Option<&str>,
    id: NodeId,
    vivo: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let n = norm.clamp(0.0, 1.0);
    let slider_id = if vivo { id } else { NodeId(0) };
    let alto = hit_index.com_recorte(|store, hits| {
        ph2d_editor_core::widget::paint_slider_with_chip_layout_adaptive(
            Rect::new(x, y, w, ph2d_tokens::ROW_H_PX),
            label,
            n,
            f64::from(n),
            valor,
            slider_id,
            NodeId(0),
            ph2d_editor_core::widget::DEFAULT_LABEL_W,
            ph2d_editor_core::widget::DEFAULT_CHIP_W,
            store,
            hits,
            scene,
            text_system,
            theme,
        )
    });
    y + alto
}
