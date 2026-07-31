//! O PINCEL — o verbo, o falloff e os knobs que o artista gira.
//!
//! Ver `docs/3D/04.1`. A divisão que este arquivo faz, e que decide tudo o
//! resto: **o `Dab` é onde e com que força a mão apertou; o `Brush` é que
//! ferramenta está na mão.** Um traço é uma lista de dabs contra UM brush.

/// A curva de peso do pincel, do centro (`t = 0`) à borda (`t = 1`).
///
/// A MESMA família que o Painter 2D já expõe, e de propósito: um artista que
/// aprendeu *Sharper* pintando não devia reaprendê-la esculpindo. A curva
/// **customizada** (o `ParamWidget::Curve` que o repo já possui) é a 6ª e entra
/// quando houver painel — construir aqui um segundo editor de curva seria a
/// segunda resposta que o `04.1` proíbe em letra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Falloff {
    /// `(1 − t²)²` — **C¹ na borda** (valor e derivada zeram em `t = 1`), e é
    /// isso que faz um traço não deixar degrau na fronteira do pincel.
    #[default]
    Smooth,
    /// `√(1 − t²)` — o perfil de uma esfera: cheio no miolo, tangente vertical
    /// na borda. Deposita mais massa que o Smooth com o mesmo raio.
    Sphere,
    /// `(1 − t²)⁴` — pico estreito, ombro que morre cedo. É o falloff de quem
    /// quer detalhe pequeno com pincel grande.
    Sharper,
    /// `1` até a borda, e nada além. Um disco duro; o degrau é a feature.
    Constant,
    /// `√(1 − t)` — sobe rápido na borda e achata no miolo, o oposto do Sharper.
    Root,
}

impl Falloff {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 5] = [
        Self::Smooth,
        Self::Sphere,
        Self::Sharper,
        Self::Constant,
        Self::Root,
    ];

    /// O peso a uma distância normalizada `t`. **Porta única** — todo verbo, a
    /// simetria e o cursor perguntam a esta função.
    ///
    /// Sem transcendental exceto a raiz (HR-5): `sqrt` é instrução de hardware,
    /// não chamada de libm.
    #[must_use]
    pub fn weight(self, t: f32) -> f32 {
        // O NaN é peneirado explicitamente: sem isso ele escorre pelas fórmulas
        // e sai como peso NaN num vértice, que é como uma malha inteira vira
        // `NaN` a partir de um dab com raio zero em algum lugar.
        if !t.is_finite() || t >= 1.0 {
            return 0.0;
        }
        let t = t.max(0.0);
        match self {
            Self::Smooth => {
                let u = 1.0 - t * t;
                u * u
            }
            Self::Sphere => (1.0 - t * t).sqrt(),
            Self::Sharper => {
                let u = 1.0 - t * t;
                let u2 = u * u;
                u2 * u2
            }
            Self::Constant => 1.0,
            Self::Root => (1.0 - t).sqrt(),
        }
    }

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Sphere => "Sphere",
            Self::Sharper => "Sharper",
            Self::Constant => "Constant",
            Self::Root => "Root",
        }
    }
}

/// O que o pincel FAZ. Ver `docs/3D/04.1` para a família de cada um.
///
/// ⚠️ **`Layer` não está aqui, e a ausência é uma decisão medida.** Sob a lei do
/// traço (`accum` é um ENVELOPE em `[0,1]`, nunca uma soma) o Draw já é limitado
/// a um `reach` por traço, por mais que o artista demore — que é exatamente o
/// que o Layer do ZBrush existe para garantir. Os dois colapsam, e o Painter 2D
/// registrou o mesmo colapso pelo mesmo motivo (`docs/Painter/18` §"Draw Sharp
/// colapsa no Layer"). Quem quiser empilhar solta e desenha de novo — o
/// `pre` se re-congela e o traço seguinte soma.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Verb {
    /// Empurra ao longo da normal **da ÁREA** (uma direção para o dab inteiro) —
    /// é isso que faz um domo liso em vez de um ouriço.
    #[default]
    Draw,
    /// Empurra cada vértice ao longo da **própria** normal: a forma ENGORDA,
    /// que é uma palavra diferente de "sobe". (A distinção Draw×Inflate é
    /// exatamente esta, no Blender e no ZBrush.)
    Inflate,
    /// Puxa cada vértice para a média dos vizinhos — o laplaciano.
    Smooth,
    /// O laplaciano com o sinal trocado: afia o que o Smooth arredondaria.
    Sharpen,
    /// Projeta no plano ajustado à pegada; move nos dois sentidos.
    Flatten,
    /// O plano, só para CIMA — enche vale sem raspar crista.
    Fill,
    /// O plano, só para BAIXO — raspa crista sem encher vale.
    Scrape,
    /// O plano deslocado para fora: o barro que se ADICIONA (é o Flatten com
    /// Offset > 0; os dois knobs ficam na tela, e por isso não é um verbo que
    /// esconde um número).
    Clay,
    /// Puxa tangencialmente para o centro do dab: afia uma aresta.
    Pinch,
    /// O oposto do Pinch: empurra para fora, alarga.
    Magnify,
    /// Pinch + deslocamento negativo — cava um vinco.
    Crease,
    /// Não move geometria: escreve `mask[v]`, que **todos** os outros respeitam.
    Mask,
}

impl Verb {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 12] = [
        Self::Draw,
        Self::Inflate,
        Self::Smooth,
        Self::Sharpen,
        Self::Flatten,
        Self::Fill,
        Self::Scrape,
        Self::Clay,
        Self::Pinch,
        Self::Magnify,
        Self::Crease,
        Self::Mask,
    ];

    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// O sinal (o `Ctrl` de todo app de escultura) muda o que este verbo faz?
    ///
    /// `Smooth`/`Sharpen` e `Pinch`/`Magnify` são pares de verbos **separados**,
    /// então inverter não tem o que negar neles — e um controle que não faz nada
    /// é pior que um que falta. A UI pergunta aqui antes de oferecer o chip.
    #[must_use]
    pub fn honours_invert(self) -> bool {
        !matches!(
            self,
            Self::Smooth | Self::Sharpen | Self::Pinch | Self::Magnify
        )
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    #[must_use]
    pub fn uses_plane(self) -> bool {
        matches!(self, Self::Flatten | Self::Fill | Self::Scrape | Self::Clay)
    }

    /// Este verbo lê o anel de vizinhos? (Quem responde `true` custa a
    /// travessia do CSR por vértice, e é o que decide se o `vert_verts` pode um
    /// dia virar preguiçoso.)
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(self, Self::Smooth | Self::Sharpen)
    }

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Inflate => "Inflate",
            Self::Smooth => "Smooth",
            Self::Sharpen => "Sharpen",
            Self::Flatten => "Flatten",
            Self::Fill => "Fill",
            Self::Scrape => "Scrape",
            Self::Clay => "Clay",
            Self::Pinch => "Pinch",
            Self::Magnify => "Magnify",
            Self::Crease => "Crease",
            Self::Mask => "Mask",
        }
    }
}

/// Quanto do RAIO do pincel um dab de força cheia desloca.
///
/// ⚠️ **É fração do raio, nunca uma distância absoluta** — a lição que o impasto
/// do Painter pagou em 2026-07-14: com altura absoluta, um pincel pequeno e um
/// grande picam no mesmo valor, e o grande vira uma poça chata porque a razão
/// *altura ÷ largura* despenca. Amarrando ao raio, a razão de aspecto do domo é
/// constante em toda escala e o falloff lê igual com pincel de 1 mm e de 1 m.
///
/// O NÚMERO é decisão de **smoke**, como o `ORBIT_RAD_PER_PX` da câmera: ele não
/// é teto de recurso nenhum, é o quanto de barro uma pincelada move.
pub const REACH_FRACTION: f32 = 0.2;

/// A ferramenta na mão. Um traço inteiro corre contra um destes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brush {
    pub verb: Verb,
    pub falloff: Falloff,
    /// Raio de influência, em unidades de MUNDO.
    pub radius: f32,
    /// Intensidade em `[0, 1]` — o que multiplica o falloff para virar o peso.
    pub strength: f32,
    /// O `Ctrl` de todo app de escultura: cava em vez de levantar.
    pub invert: bool,
    /// Desloca o plano ao longo da normal da área, em fração do raio. Positivo
    /// adiciona matéria (é o que faz do Flatten um Clay). Só os verbos de plano
    /// o leem — ver [`Verb::uses_plane`].
    pub plane_offset: f32,
    /// Quanto o `Crease` aperta lateralmente, em `[0, 1]`.
    pub pinch: f32,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            verb: Verb::Draw,
            falloff: Falloff::Smooth,
            radius: 0.25,
            strength: 0.5,
            invert: false,
            plane_offset: 0.0,
            pinch: 0.5,
        }
    }
}

impl Brush {
    /// O deslocamento, com sinal, que um dab deste pincel alcança — em unidades
    /// de mundo. Porta única: Draw, Inflate, Clay e Crease perguntam aqui.
    #[must_use]
    pub fn reach(&self, radius: f32) -> f32 {
        let s = if self.invert && self.verb.honours_invert() {
            -1.0
        } else {
            1.0
        };
        radius * REACH_FRACTION * s
    }
}

/// Os espelhos que multiplicam um dab.
///
/// ⚠️ **A simetria é aplicada na LISTA DE DABS, num ponto único** — nunca dentro
/// de um verbo. É o que faz um verbo novo herdá-la de graça, e é literalmente a
/// lição que o `stamp_dabs_inner` do Painter 2D deixou escrita.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Symmetry {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Symmetry {
    /// Só o eixo X — o caso que 95% da escultura de personagem usa.
    pub const MIRROR_X: Self = Self {
        x: true,
        y: false,
        z: false,
    };

    #[must_use]
    pub fn any(self) -> bool {
        self.x || self.y || self.z
    }

    /// Os sinais por eixo de cada cópia, incluindo o original. De 1 a 8
    /// entradas; a primeira é sempre `[1, 1, 1]`.
    ///
    /// Devolve um array fixo + comprimento para não alocar por dab — um traço
    /// emite dezenas de dabs por segundo, e este é o caminho quente.
    #[must_use]
    pub fn signs(self) -> ([[f32; 3]; 8], usize) {
        let mut out = [[1.0f32; 3]; 8];
        let mut n = 1;
        for (axis, on) in [self.x, self.y, self.z].into_iter().enumerate() {
            if !on {
                continue;
            }
            for i in 0..n {
                let mut s = out[i];
                s[axis] = -s[axis];
                out[n + i] = s;
            }
            n *= 2;
        }
        (out, n)
    }
}

#[cfg(test)]
#[path = "brush_tests.rs"]
mod tests;
