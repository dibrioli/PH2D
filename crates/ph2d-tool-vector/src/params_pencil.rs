//! **Os controles da mão livre** — a faixa do Fidelity e o default do Stabilizer, com a MEDIÇÃO que
//! escolheu cada número ao lado dele (plano 25 W1).

/// **A faixa do slider de Fidelity do lápis**, em px de TELA — a tolerância do decimador (RDP).
///
/// **MEDIDA** (`measure_pencil_fidelity_range` na shell, o S trémulo de 300 px com ±1,5 px de tremor
/// e o estabilizador no default) — nós do traço e desvio máximo da curva IDEAL:
///
/// | fidelity | nós | desvio |
/// |---|---|---|
/// | 0,5 | 133 | 2,66 px |
/// | 1,0 | 66 | 6,38 px |
/// | **2,0** | **14** | **1,23 px** |
/// | 4,0 | 11 | 2,39 px |
/// | 8,0 | 8 | 4,20 px |
/// | 12,0 | 6 | 12,53 px |
/// | 16,0 | 5 | 12,47 px |
/// | 24,0 | 4 | 12,87 px |
///
/// ⚠️ **O desvio é PIOR nas tolerâncias BAIXAS, e isto não é intuitivo:** a 1,0 px o traço fica
/// 6,38 px longe da curva pretendida, contra 1,23 px a 2,0. O decimador guarda extremos locais (é
/// o que salva uma quina), então tolerância baixa guarda o TREMOR — e uma spline INTERPOLANTE
/// através de nós próximos e trémulos **oscila mais que os próprios nós**. Num ajuste que passa
/// pelos pontos, *mais nós não é mais fidelidade*.
///
/// O teto é **8**: a 12 px o S já colapsou (12,53 px de desvio) e a contagem satura em 4 nós — a
/// forma foi comida, não simplificada. O piso é **0,5**: abaixo dele os dois eixos pioram ao mesmo
/// tempo (mais nós E mais desvio), então não há o que oferecer.
pub const PENCIL_FIDELITY_MIN_PX: f64 = 0.5;
pub const PENCIL_FIDELITY_MAX_PX: f64 = 8.0;
/// O default do slider — o joelho MEDIDO da tabela acima.
///
/// ⚠️ **Segunda cópia deliberada** do `ph2d_vec_edit::pencil::DEFAULT_FIDELITY_PX`, que é o
/// *fallback do motor para um chamador que nunca escolhe*. As duas crates não se veem (a shell é
/// quem as liga), e a igualdade é pinada por um gate NA SHELL, que vê as duas — o mesmo padrão dos
/// ranges de knob do Wet Paint. Um `use` cruzado custaria uma aresta de dependência inteira para
/// nomear um `f64`, e inverteria a camada (hoje o tool não conhece o motor de edição).
pub const PENCIL_FIDELITY_DEFAULT_PX: f64 = 2.0;

/// Mapeamento afim do slider de Fidelity (`display_px = track * SCALE + OFFSET`), consumido pelo
/// `link_slider_number_mapped` para o chip espelhar o slider.
pub const PENCIL_FIDELITY_SLIDER_SCALE: f32 =
    (PENCIL_FIDELITY_MAX_PX - PENCIL_FIDELITY_MIN_PX) as f32;
pub const PENCIL_FIDELITY_SLIDER_OFFSET: f32 = PENCIL_FIDELITY_MIN_PX as f32;

/// Track normalizado `0..=1` → Fidelity em px.
#[must_use]
pub fn slider_to_fidelity_px(track: f32) -> f64 {
    PENCIL_FIDELITY_MIN_PX
        + f64::from(track.clamp(0.0, 1.0)) * (PENCIL_FIDELITY_MAX_PX - PENCIL_FIDELITY_MIN_PX)
}

/// Fidelity em px → track normalizado (inverso de [`slider_to_fidelity_px`]), para semear o botão
/// do slider a partir do valor autoritativo do tool.
#[must_use]
pub fn fidelity_px_to_slider(px: f64) -> f32 {
    (((px - PENCIL_FIDELITY_MIN_PX) / (PENCIL_FIDELITY_MAX_PX - PENCIL_FIDELITY_MIN_PX)) as f32)
        .clamp(0.0, 1.0)
}

/// **O default do estabilizador do lápis** — o track do slider É o valor (domínio `0..=1`, o do
/// `lazy_mouse_step`), então não há mapeamento afim a fazer.
///
/// ⚠️ **A faixa é 0..1 e NÃO é capada no joelho de propósito.** A medição mostra o tremor residual
/// com mínimo em 0,75 e a PIORAR até 1,00 (a tabela vive no `vec_pencil_input` da shell, ao lado do
/// filtro) — mas o slider do Painter é 0..1, e capar só o do vetor daria ao app duas respostas para
/// *"o que a estabilização faz?"*. Estabilização pesada é técnica (traço lento e liso), não
/// degenerescência.
pub const PENCIL_STABILIZER_DEFAULT: f32 = 0.5;

/// Mapeamento afim do CHIP do estabilizador: o track é `0..=1` e o chip mostra **por cento**, como
/// os chips de opacidade fazem (`OPACITY_SLIDER_SCALE`). É o chip que converte a unidade, não o
/// código de pintura — assim não há um `* 100.0` solto numa string de readout.
pub const PENCIL_STABILIZER_SLIDER_SCALE: f32 = 100.0;
pub const PENCIL_STABILIZER_SLIDER_OFFSET: f32 = 0.0;
