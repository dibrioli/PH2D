//! **O que a tool ADOTA do documento** — irmão de [`super::tool`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, e é a fronteira mais nítida que esta crate tem: tudo o mais na
//! tool é estado **AUTORADO** (o que o artista escolheu, e o que a próxima forma vai herdar); aqui
//! está o caminho inverso — **LER** o documento quando a seleção muda, para o painel parar de
//! mentir sobre o que está na tela.
//!
//! ⚠️ **Nenhum método aqui marca `apply_to_selected`**, e essa é a lei da família: armar ao adotar
//! faria a seleção ser reescrita com o que ela já é, todo frame, e um passo de undo com ela.

//! ⚠️ **Módulo FILHO de `tool.rs`, não irmão** (`#[path]` + `use super::*`): os campos que estas
//! quatro escrevem são privados do módulo da tool, e alargá-los para `pub(crate)` só para acomodar
//! um split seria pagar em superfície o que o teto de LOC cobra em organização. É o mesmo padrão
//! que os `*_tests.rs` deste repo já usam.

use super::*;

impl VectorTool {
    /// ADOTA os parâmetros de uma forma (a shell chama ao selecionar uma forma viva):
    /// eles viram os correntes daquela forma, então o painel para de mentir e a próxima
    /// desenhada os herda (modelo Figma "último usado").
    pub fn adopt_shape_values(&mut self, k: ShapeKind, v: ShapeValues) {
        self.shape_values[k.as_u16() as usize] = v;
        crate::shapes::clamp(k, &mut self.shape_values[k.as_u16() as usize]);
    }

    /// **ADOTA o preenchimento** de um caminho — irmão do [`Self::adopt_stroke`], e pela mesma
    /// razão: sem ele a swatch de Fill mostra a última cor autorada em vez da que está na tela.
    /// Alfa 0 é o *sem preenchimento*, a mesma convenção do caminho inverso na ponte.
    ///
    /// ⚠️ **NÃO marca `apply_to_selected`** — adotar é LER o documento.
    pub fn adopt_fill(&mut self, rgba: [u8; 4]) {
        self.fill = rgba;
    }

    /// **ADOTA o traço de um caminho** (a shell chama ao SELECIONAR): tudo o que a tool possui
    /// do stroke passa a ser o daquele caminho, então o painel para de mentir e a próxima forma
    /// desenhada o herda (modelo Figma "último usado").
    ///
    /// ⚠️ **Toma o `StrokeSpec` do DOCUMENTO, e não uma 2ª ficha.** O que a ponte ESCREVE no
    /// apply é um `StrokeSpec` (`StrokeStyle::onto`); ler na seleção pelo MESMO tipo é o que
    /// torna a simetria literal — um campo que exista de um lado e não do outro seria um controle
    /// que mexe no número e não muda nada na tela, que é a doença que aquele arquivo documenta.
    ///
    /// ⚠️ **NÃO marca `apply_to_selected`** — adotar é LER o documento; armar aqui faria a
    /// seleção ser reescrita com o que ela já é, todo frame, e um passo de undo com ela.
    ///
    /// A largura entra em **px de tela** (a unidade que a tool guarda), convertida pelo chamador:
    /// o `StrokeSpec` a traz em MUNDO, e a conversão tem um dono só.
    pub fn adopt_stroke(&mut self, s: &ph2d_vec_scene::StrokeSpec, width_px: f64) {
        self.stroke = [s.color.r, s.color.g, s.color.b, s.color.a];
        // ⚠️ Clampa contra as CONSTANTES, e NÃO pela porta do slider. Rotear um `f64` por
        // `px_to_slider`/`slider_to_px` parecia reuso elegante e **quantiza**: a trilha é `f32`,
        // e 20,0 volta como 20,000000298 (medido pelo gate da lei). Adotar tem de devolver o
        // número do DOCUMENTO, não uma aproximação dele.
        self.stroke_width_px =
            width_px.clamp(crate::params::WIDTH_MIN_PX, crate::params::WIDTH_MAX_PX);
        self.cap = s.cap.into();
        self.join = s.join.into();
        self.align = s.align;
        // `dash` é `Option<(traço, vão)>` em múltiplos da largura; a tool guarda os dois soltos,
        // e `dash = 0` é o contínuo — a mesma convenção que o `dash_lengths` do documento usa.
        let (d, g) = s.dash.unwrap_or((0.0, crate::params::GAP_DEFAULT));
        self.dash = d;
        self.gap = g;
        self.adopt_markers(s.marker_start, s.marker_end, s.marker_scale, s.marker_round);
    }

    /// **ADOTA** as pontas de um caminho (o shell chama ao SELECIONAR um): elas viram as
    /// correntes, e o painel — que pinta a partir da tool — passa a mostrar as pontas
    /// daquele caminho em vez das últimas autoradas. Espelho exato de
    /// [`Self::adopt_shape_values`], e pela mesma razão: sem isto o seletor mente sobre o
    /// que está na tela. **Não** marca `apply_to_selected` — adotar é LER o documento; se
    /// marcasse, o próprio ato de selecionar reescreveria o caminho.
    pub fn adopt_markers(&mut self, start: Marker, end: Marker, scale: f64, round: f64) {
        self.marker_start = start;
        self.marker_end = end;
        self.marker_scale = crate::shapes::clamp_to(&crate::params::MARKER_SCALE, scale);
        self.marker_round = crate::shapes::clamp_to(&crate::params::MARKER_ROUND, round);
    }
}
