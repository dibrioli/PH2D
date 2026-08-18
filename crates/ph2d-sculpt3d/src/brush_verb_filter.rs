//! **QUE VERBOS PODEM SER UM FILTRO** — a lei e a faixa de cada um, irmãos do
//! [`super`], que responde *o que um verbo É*.
//!
//! ⚠️ **A divisão não é de tamanho, é de assunto.** O arquivo pai é o CATÁLOGO
//! (quais verbos existem, o que cada um lê, com que gesto se pega); aqui mora a
//! resposta a **uma** pergunta nova: *este verbo pode ser dirigido SEM DAB, e em
//! que faixa o arrasto dele vive?*
//!
//! ⚠️ **A tabela é UMA, e é ela que torna o motor do filtro EXAUSTIVO.** O
//! [`crate::SculptStroke::filter`] casa sobre [`FilterKind`], não sobre
//! [`Verb`] — então uma lei nova que entre aqui e não seja implementada lá é um
//! **erro de compilação**, e não um filtro que aparece na lista e não move um
//! vértice. É a mesma razão pela qual a [`crate::GripLaw`] é uma tabela.

use super::*;

/// **A LEI que um filtro roda.** Um por lei, não um por verbo — é isto que dá
/// casa às leis que a referência tem e nós não temos como pincel (o `Sphere`, o
/// `Random`, o `Scale`), sem as empurrar para dentro do [`Verb::ALL`], onde
/// cada uma seria um chip de pincel que ninguém pode carimbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
    /// `calc_smooth_filter` — anda a fração `f` na direção da média do anel.
    ///
    /// ⚠️ **Com `f` NEGATIVO ele AFIA**, e é por isso que a faixa da referência
    /// é `[−1, 1]` e não `[0, 1]`: `smooth(−f)` **é** `sharpen(f)` (a álgebra
    /// está no gate `the_sharpen_filter_is_the_smooth_filter_dragged_backwards`,
    /// que a mede em vez de a afirmar).
    Smooth,
    /// `calc_inflate_filter` — `t = orig_normals × f`, ao pé da letra.
    Inflate,
    /// `calc_relax_filter` — a mesma média, com a componente normal REMOVIDA.
    Relax,
    /// `calc_surface_smooth_filter` — o HC (Vollmer et al.), que devolve parte
    /// do que o laplaciano tirou.
    SurfaceSmooth,
}

/// **A LEI + A FAIXA.** A faixa é `clamp(máscara × arrasto, lo, hi)`, na ordem
/// da referência (`scale_factors` e depois `clamp_factors`).
#[derive(Debug, Clone, Copy)]
pub struct FilterLaw {
    /// Qual das leis acima.
    pub kind: FilterKind,
    /// O piso do fator.
    pub lo: f32,
    /// O teto do fator.
    pub hi: f32,
}

impl Verb {
    /// **Este verbo pode ser dirigido SEM DAB?** — e, se pode, sob que lei e em
    /// que faixa.
    ///
    /// ⚠️ **A propriedade que a lista exprime é MEDÍVEL, não uma opinião:** o
    /// alvo destes quatro é função do vértice, do `pre` congelado e do anel, e
    /// de **mais nada** — nenhum deles lê o centro do dab, a normal de área, o
    /// plano ajustado à pegada, o caminho do traço ou uma âncora. O gate
    /// `a_filtering_verb_reads_nothing_from_the_dab` o afirma pela porta do
    /// produto: o mesmo vértice, o mesmo peso, **dois dabs radicalmente
    /// diferentes**, e o alvo tem de sair byte a byte igual.
    ///
    /// ⚠️ **WHITELIST e não blacklist**, pelo motivo que o
    /// [`Self::honours_invert`] já pagou uma vez: derivada por negação
    /// (`!uses_plane() && !anchors() && !paints_mask()`) ela deixaria de fora
    /// exactamente os verbos que o `uses_plane` **não** nomeia e que mesmo assim
    /// leem o dab — o [`Verb::Draw`] e o [`Verb::ClayStrips`] leem a normal de
    /// ÁREA, o [`Verb::ClayThumb`] lê o caminho, os quatro do
    /// [`crate::kelvinlet`] leem o centro. Um verbo novo nasceria reivindicando
    /// que filtra a malha, **em silêncio**, e o filtro o rodaria com um dab que
    /// não existe.
    ///
    /// ⚠️ **O [`Verb::Sharpen`] fica de FORA, e a exclusão é medida.** Ele
    /// passa na propriedade acima (só lê o anel), mas o alvo dele é
    /// `live + (live − média)·w`, que **é** o `target_smooth` com o sinal do
    /// peso trocado — e num filtro o sinal vem do ARRASTO. Oferecê-lo seria a
    /// segunda porta para o que arrastar o Smooth para a esquerda já faz. No
    /// pincel ele é um chip próprio com razão: ali o gesto **não tem sinal** (o
    /// `Ctrl` é a única fonte, e o [`Self::honours_invert`] não nomeia nem
    /// Smooth nem Sharpen), então sem o chip a lei seria inalcançável.
    ///
    /// ⚠️ **O [`Verb::Layer`] fica de fora por MEDIÇÃO, não por herança.** O
    /// irmão 2-D deste módulo (o *Filter Layer* do Painter) o recusou porque
    /// filtrar uma camada com ele é uma **translação uniforme** de um campo de
    /// altura, e a luz de lá lê `∇h` — não moveria um pixel. Aqui a razão **não
    /// transfere**: a nossa normal é por-vértice, então `base + n·h` sobre a
    /// malha inteira é precisamente o [`FilterKind::Inflate`], com um degrau de
    /// saturação a mais. A recusa fica de pé pela OUTRA razão — é a segunda
    /// porta para o mesmo deslocamento.
    #[must_use]
    pub fn filter_law(self) -> Option<FilterLaw> {
        let (kind, lo, hi) = match self {
            // `clamp_factors(factors, -1.0f, 1.0f)` — `sculpt_filter_mesh.cc:375`.
            Self::Smooth => (FilterKind::Smooth, -1.0, 1.0),
            // ⚠️ **SEM clamp, e a ausência é da referência** (`calc_inflate_filter`
            // não chama `clamp_factors`): o deslocamento é `strength` em
            // unidades de OBJETO, e um teto aqui seria um número que ninguém
            // mediu. Quem calibra é a escala do ARRASTO, que é nossa — ver
            // [`crate::FILTER_DRAG_PER_PX`].
            Self::Inflate => (FilterKind::Inflate, f32::MIN, f32::MAX),
            // `clamp_factors(factors, 0.0f, 1.0f)` — `:1019` e `:1358`.
            //
            // ⚠️ **Arrastar para o lado errado não faz NADA nestes dois, e é a
            // lei da referência:** um relax negativo não é *"desrelaxar"* — não
            // existe a operação inversa de redistribuir —, e o HC negativo
            // amplificaria o próprio erro que ele existe para devolver.
            Self::SlideRelax => (FilterKind::Relax, 0.0, 1.0),
            Self::SurfaceSmooth => (FilterKind::SurfaceSmooth, 0.0, 1.0),
            _ => return None,
        };
        Some(FilterLaw { kind, lo, hi })
    }

    /// **Este verbo é oferecido como filtro?** — uma leitura de
    /// [`Self::filter_law`] em vez de um segundo predicado, o mesmo corte que
    /// [`Self::anchors`] faz sobre o [`Self::grip`].
    ///
    /// É a porta que o plano nomeia (`filters_mesh`), e o consumidor dela é a
    /// UI: *que chips esta lista mostra?*
    #[must_use]
    pub fn filters_mesh(self) -> bool {
        self.filter_law().is_some()
    }
}
