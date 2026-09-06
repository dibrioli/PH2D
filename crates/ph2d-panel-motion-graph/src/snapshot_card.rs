//! ⭐⭐⭐ **O QUE UM CARTÃO CONTROLA** — a faixa de params desenhada dentro do nó e as secções
//! que a dobram (ciclo 1 da dinâmica, [doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⚠️ Irmão de [`super`] por RESPONSABILIDADE (HR-18): o pai descreve o que o grafo **É** (nós,
//! pinos, fios, molduras) e este o que cada cartão **CONTROLA**. `super` é o `snapshot`, e o
//! `use super::*` traz o `ParamUiHint` e os vizinhos.

use super::*;

/// ⭐ **UMA SECÇÃO da faixa de params do cartão** — o «painel dentro do nó» do Blender 4.x.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardSection {
    /// O nome do grupo, tal como o registry o declara.
    pub title: &'static str,
    /// O índice, **em [`GraphNodeView::params`]**, da primeira row desta secção — o cabeçalho é
    /// desenhado imediatamente antes dela. Numa secção FECHADA aponta para onde as rows
    /// estariam (a próxima row visível, ou o fim da lista).
    pub at: u16,
    /// Aberta: as rows dela estão em `params`. Fechada: não estão.
    pub open: bool,
    /// Quantas rows a secção esconde quando fechada — o número que o cabeçalho mostra, para
    /// uma secção dobrada não parecer uma secção vazia.
    pub hidden: u16,
}

/// ⭐ **UM PARAM NO CARTÃO** — o hint `&'static` do registry mais o valor vivo.
///
/// `Copy` de propósito: [`ph2d_node_registry::ParamUiHint`] já é `Copy` (os seus `param` e
/// `label` são `&'static str`), então uma row do cartão custa **um `memcpy`, nunca um
/// `String`** — ver o doc de [`GraphNodeView::params`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardParam {
    /// Rótulo, faixa, passo e widget — tal como o registry os declara (`register_param_ui`).
    pub hint: ParamUiHint,
    /// O valor **AUTORADO** (override do grafo, senão o default do manifesto). ⚠️ Não é o
    /// valor do cook: um param dirigido por fio só tem valor DURANTE o cozimento, e desenhar
    /// esse faria a row tremer a meio de um quadro.
    pub value: f32,
    /// Um fio (doc 58) dirige este param: a row mostra a proveniência e **não se arrasta** —
    /// o número vem de fora.
    pub driven: bool,
    /// A COR, em **bytes sRGB**, quando o widget é [`ph2d_node_registry::ParamWidget::Color`] —
    /// a row pinta uma **amostra**, nunca um número.
    ///
    /// ⚠️ Um hint de cor ancora QUATRO params (`channels`), e a shell suprime os quatro do
    /// resto da lista — a mesma lei do painel. Sem isso o cartão mostrava cinco rows para uma
    /// cor: a amostra e os canais `r`/`g`/`b`/`a` crus, que é exactamente o que o
    /// `ParamWidget::Color` existe para não fazer.
    ///
    /// ⚠️ **Bytes e não `f32`, porque a conversão é da SHELL:** os params guardam RGBA
    /// **linear**, e quem sabe passar isso a sRGB é o `linear_rgba_to_srgb8` que o bridge de
    /// cor já usa para semear o picker. Converter aqui seria a segunda cópia dessa lei.
    pub swatch: Option<[u8; 4]>,
    /// ⭐⭐ **A FAIXA DO ARRASTO, RESOLVIDA** — e ⛔ **não** é `hint.min`/`hint.max`.
    ///
    /// A faixa de uma MAGNITUDE depende do canal que ela conduz (o `Amount` do `motion.drive`
    /// mede graus na Rotation e unidades de mundo no X/Y), e o hint estático só descreve um
    /// deles. O painel resolve isso desde 2026-08-14 — *«Scale não aceita mais que 4 em sua
    /// caixa de texto e 4 não é quase nada para rot»*, report do Enio — e o cartão tem de
    /// resolver **pela mesma porta**, senão a saída do painel traz o defeito de volta.
    ///
    /// ⚠️ **Alargada para CONTER o valor vivo**, sempre: uma faixa que não contém o número que
    /// está lá pinta um nível saturado e, ao primeiro toque, escreve o limite por cima do que
    /// o artista autorou.
    pub min: f32,
    pub max: f32,
    pub step: f32,
    /// ⭐⭐ **A FAIXA DIGITÁVEL** — o piso e o tecto que o teclado aceita, que **não** são os do
    /// arrasto (`hint.min`/`hint.max`).
    ///
    /// É a lei do doc 88 que o painel já tem, trazida para o cartão inteira: *arrastar e ser
    /// legal são perguntas diferentes* (o *soft* contra o *hard* do Blender). O `Radius` de uma
    /// forma arrasta-se de `1` a `20` porque é aí que ele é útil, e escreve-se até `1 000 000`
    /// porque é aí que ele ainda é um número.
    ///
    /// ⚠️ **Contém sempre a faixa do arrasto e o valor vivo** — um tecto digitável ABAIXO do que
    /// o dedo alcança desfaria em silêncio um valor que o slider ainda produz.
    pub hard_min: f32,
    pub hard_max: f32,
    /// ⭐⭐⭐ **A FACE: `mostrado = guardado × face_scale`, com `face_suffix` ao lado.**
    ///
    /// Um comprimento do mundo é **guardado em metros** e o artista trabalha em **pixels** — o
    /// painel mostra `94 px` onde o documento tem `0,94`. ⚠️ **Medido em 2026-09-05: `109` de
    /// `454` rows escalares do catálogo (24 %) têm escala de face, e `135` têm sufixo** — sem
    /// isto o cartão mostraria outro número que o painel para um quarto dos controlos, e a caixa
    /// de escrita escreveria `94` onde o painel escreve `0,94`.
    ///
    /// ⚠️ **`value`, `min`, `max`, `step` e os dois `hard_*` já vêm NA FACE** (é o que
    /// `ScalarRow::in_display` faz do lado do painel): tudo o que desenha, arrasta e escreve
    /// trabalha no número do artista, e a volta ao documento acontece **num sítio só**,
    /// [`CardParam::to_stored`].
    pub face_scale: f32,
    pub face_suffix: &'static str,
}

impl CardParam {
    /// ⭐⭐ **A ÚNICA porta de volta ao documento** — o número que o artista vê, no valor que o
    /// cook lê. Os quatro sítios que emitem um `SetParam` de uma row de cartão passam por aqui.
    ///
    /// ⚠️ Uma escala não-finita ou não-positiva cai no neutro: ela chegaria aqui como uma
    /// divisão que manda o número do artista para o infinito.
    #[must_use]
    pub fn to_stored(&self, shown: f32) -> f32 {
        if self.face_scale.is_finite() && self.face_scale > 0.0 {
            shown / self.face_scale
        } else {
            shown
        }
    }

    /// A row de um param **sem nada declarado além do hint** — as duas faixas são a dele.
    ///
    /// ⚠️ É o caso NEUTRO, e é por isso que ele é uma porta: quem resolve o canal, o fio, o
    /// `contain` e os limites digitáveis é a shell (`stamp_card_params`), que tem o registry e
    /// o documento. Uma sonda ou um teste que precise de uma row constrói-a por aqui em vez de
    /// escrever cinco campos à mão — e assim um campo NOVO não passa a ser copiado errado em
    /// seis sítios.
    #[must_use]
    pub fn from_hint(hint: ParamUiHint, value: f32) -> Self {
        Self {
            hint,
            value,
            driven: false,
            swatch: None,
            min: hint.min,
            max: hint.max,
            step: hint.step,
            hard_min: hint.min,
            hard_max: hint.max,
            // A face NEUTRA de uma grandeza é a que o próprio widget já decide (um ângulo diz
            // `deg` sem perguntar a ninguém). Só o comprimento precisa do projeto, e é por isso
            // que ele é a shell quem o resolve.
            face_scale: 1.0,
            face_suffix: ph2d_node_registry::unit_of(hint.widget, None)
                .fixed_suffix()
                .unwrap_or(""),
        }
    }

    /// A mesma, marcada como **dirigida por fio** (o valor vem de fora e a row não se arrasta).
    #[must_use]
    pub fn driven_by_wire(mut self) -> Self {
        self.driven = true;
        self
    }
}
