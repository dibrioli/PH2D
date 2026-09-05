//! ⭐⭐⭐ **A PILHA DE APARÊNCIA** — N preenchimentos e N contornos numa forma (estudo 42, item 4).
//!
//! É o *Appearance panel* do Illustrator, os *Fills*/*Strokes* do Rive e a pilha do Affinity. Até
//! aqui, cada camada de estilo obrigava a **duplicar o objecto** — e duas cópias de uma forma são
//! duas geometrias que divergem no primeiro ponto que o artista mexe.
//!
//! # ⭐ A metade que faltava de um mecanismo que já shipa
//!
//! Esta struct já tem a pilha da **geometria** ([`crate::effect::FxEntry`], ADR-0132: uma lista
//! ordenada de efeitos, cada um com o próprio interruptor). Isto é a pilha do **estilo**, com a
//! mesma forma e pelas mesmas razões — *o Inkscape separa geometria de estilo e o manual deste
//! repo nomeia isso como a limitação a superar*.
//!
//! # As três decisões, e de onde vieram
//!
//! 1. **INTERCALADA.** Uma lista só, onde cada entrada é um preenchimento **ou** um contorno — e
//!    não duas listas. ⚠️ É o que separa o modelo do Illustrator/Rive do do **Figma**, que tem N
//!    tintas de contorno partilhando **uma** geometria de traço: lá, dois contornos de larguras
//!    diferentes (a «etiqueta», o «carril de comboio») são inexprimíveis sem duplicar a forma, que
//!    é exactamente a lacuna que esta wave fecha.
//! 2. **DE BAIXO PARA CIMA.** O índice `0` é a camada mais próxima da base, e é a primeira a
//!    desenhar. ⚠️ O painel mostra a pilha ao contrário (o topo em cima, como o Illustrator), e
//!    **essa inversão vive num sítio só** — senão as duas convenções divergem e o artista arrasta
//!    uma camada para cima e ela desce.
//! 3. **A BASE É O CHÃO DA PILHA.** O `fill` e o `stroke` do documento continuam onde estavam e
//!    são as duas camadas mais baixas, nesta ordem. ⛔ Não há migração escondida para a lista, e
//!    não há duas fontes que possam discordar: *a base é o chão, e a lista é o que está por cima
//!    dele*, exactamente como `verts` é o contorno primário e `subpaths` os outros.
//!
//! ⚠️ **E a base não fica sem opacidade nem sem mistura — elas são as do OBJECTO** (v19, estudo 42
//! item 2). Isso não é uma assimetria: a camada de baixo compõe-se com o que está **atrás da
//! forma**, e é precisamente isso que a mistura do objecto quer dizer. As camadas de cima
//! compõem-se com o que está **dentro** dela, e por isso a opacidade e a mistura delas são da
//! ENTRADA.

use crate::{Opacity, Paint, StrokeSpec, VecPath};
use ph2d_blend_mode::BlendMode;

/// ⭐⭐⭐ **Quantas camadas uma forma pode ter.**
///
/// ⚠️ **O número saiu de uma MEDIÇÃO, e ela nomeia dois recursos** (§0.0):
///
/// | camadas | caminhos emitidos | recortes | encode (debug) |
/// |---|---|---|---|
/// | 4 | 14 | 8 | 20 µs |
/// | 16 | 50 | 32 | 72 µs |
/// | **32** | 98 | 64 | **167 µs** |
/// | 64 | 194 | 128 | 439 µs |
///
/// O custo é **linear** (~6,8 µs por camada em debug), então o encode não é o que trava: a `32`,
/// UMA forma custa `167 µs` de um quadro de `16 700` — **1,0 %**, num build sem optimização.
///
/// O recurso que de facto manda é o **espaço de ids do painel**: cada linha tem cinco controlos, e
/// a resolução de um clique varre esse espaço (a mesma lei que o `MAX_BLEND_MODES` impõe à lista de
/// modos). ⛔ Um `Vec` sem tecto tornaria um clique irresolúvel para as camadas acima do que o
/// painel endereça, e uma camada que se desenha e não se toca é pior que uma ausente.
///
/// ⚠️ Para escala: as artes de referência do Illustrator usam **2 a 5** atributos.
pub const MAX_PAINT_LAYERS: usize = 32;

/// O que uma entrada da pilha PINTA.
///
/// ⚠️ **Append-only**: o postcard serializa o índice do variant, então um variant no meio relê
/// saves antigos como a outra coisa, em silêncio.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PaintKind {
    /// Mais um preenchimento, com a mesma tinta que o `fill` de base aceita.
    Fill(Paint),
    /// Mais um contorno — **com a largura, a ponta, a junção e o tracejado dele**. É isto que faz
    /// a «etiqueta» (um traço branco largo por baixo de um preto fino) caber numa forma só.
    Stroke(StrokeSpec),
}

/// **Uma entrada da pilha** — o que ela pinta, se está ligada, e como se compõe com o que está por
/// baixo dela DENTRO da forma.
///
/// ⚠️ O «ligado» é propriedade da ENTRADA e não da tinta, pela mesma razão que o
/// [`crate::effect::FxEntry`] o tem: *desarmar não pode custar os parâmetros* — pôr o alfa a zero
/// para esconder uma camada e depois querer a cor de volta é a definição de uma feature que obriga
/// o artista a lembrar-se de números.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PaintEntry {
    pub kind: PaintKind,
    /// Desligada = a pilha SALTA-a, e os parâmetros ficam.
    pub enabled: bool,
    /// Opacidade **desta camada**, sobre o que já está pintado dentro da forma.
    pub opacity: Opacity,
    /// Como esta camada se compõe com o que está por baixo dela **dentro** da forma.
    pub blend: BlendMode,
}

impl PaintEntry {
    /// Uma entrada nova, ligada, opaca e sem mistura — o neutro.
    #[must_use]
    pub fn new(kind: PaintKind) -> Self {
        Self {
            kind,
            enabled: true,
            opacity: Opacity::default(),
            blend: BlendMode::Normal,
        }
    }

    /// Um preenchimento novo.
    #[must_use]
    pub fn fill(paint: Paint) -> Self {
        Self::new(PaintKind::Fill(paint))
    }

    /// Um contorno novo.
    #[must_use]
    pub fn stroke(spec: StrokeSpec) -> Self {
        Self::new(PaintKind::Stroke(spec))
    }

    /// Esta entrada chega a pintar alguma coisa?
    ///
    /// ⚠️ Um contorno de largura **zero** não desenha nada, e contá-lo como camada faria o
    /// renderer abrir uma camada de composição por nada — o mesmo teste que o desenho da base já
    /// faz (`s.width > 0.0`).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled
            && match &self.kind {
                PaintKind::Fill(_) => true,
                PaintKind::Stroke(s) => s.width > 0.0,
            }
    }

    /// **A cor que a swatch desta linha mostra** — a sólida, a 1.ª parada de um gradiente, ou a
    /// `fallback` de um padrão.
    ///
    /// ⚠️ É a MESMA pergunta que o [`Paint::primary_color`] e o [`crate::StrokePaint::color`] já
    /// respondem, e por isso ela é delegada: uma terceira transcrição de *"de que cor é isto?"*
    /// divergiria na primeira tinta nova.
    #[must_use]
    pub fn swatch_color(&self) -> crate::Rgba8 {
        match &self.kind {
            PaintKind::Fill(p) => p.primary_color(),
            PaintKind::Stroke(s) => s.color(),
        }
    }

    /// A largura, quando esta camada é um contorno.
    #[must_use]
    pub fn width(&self) -> Option<f64> {
        match &self.kind {
            PaintKind::Stroke(s) => Some(s.width),
            PaintKind::Fill(_) => None,
        }
    }

    /// O nome que o painel mostra nesta linha.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.kind {
            PaintKind::Fill(_) => "Fill",
            PaintKind::Stroke(_) => "Stroke",
        }
    }
}

/// O que uma camada pinta, **emprestado** — o que a porta [`VecPath::paint_stack`] devolve.
#[derive(Clone, Copy, Debug)]
pub enum PaintRef<'a> {
    Fill(&'a Paint),
    Stroke(&'a StrokeSpec),
}

/// **Uma camada a desenhar**, já resolvida: a base e as entradas saem daqui iguais, e é isso que
/// impede quem desenha de conhecer a diferença entre as duas.
#[derive(Clone, Copy, Debug)]
pub struct DrawnPaint<'a> {
    pub paint: PaintRef<'a>,
    /// `1.0` para a base — a opacidade dela é a do OBJECTO, e essa é aplicada uma vez sobre a
    /// forma inteira.
    pub opacity: f32,
    /// `Normal` para a base, pela mesma razão.
    pub blend: BlendMode,
    /// Esta camada é a base? Quem desenha não precisa de saber; quem **mede** precisa (o
    /// hit-test da largura, o exportador que marca o preenchimento do balde).
    pub is_base: bool,
}

impl VecPath {
    /// ⭐⭐⭐ **A PILHA INTEIRA, na ordem de desenho** — a base primeiro (preenchimento, depois
    /// contorno), depois cada entrada ligada de baixo para cima.
    ///
    /// ⛔ **É a porta ÚNICA de *"o que é que esta forma pinta?"*.** Quem lê `path.fill` e
    /// `path.stroke` directamente vê só o chão da pilha — o que está certo para uma swatch («de
    /// que cor é esta forma?») e **errado** para quem desenha, exporta ou mede.
    pub fn paint_stack(&self) -> impl Iterator<Item = DrawnPaint<'_>> {
        let base_fill = self.fill.as_ref().map(|p| DrawnPaint {
            paint: PaintRef::Fill(p),
            opacity: 1.0,
            blend: BlendMode::Normal,
            is_base: true,
        });
        let base_stroke = self
            .stroke
            .as_ref()
            .filter(|s| s.width > 0.0)
            .map(|s| DrawnPaint {
                paint: PaintRef::Stroke(s),
                opacity: 1.0,
                blend: BlendMode::Normal,
                is_base: true,
            });
        let extras = self.paints.iter().filter(|e| e.is_active()).map(|e| {
            let paint = match &e.kind {
                PaintKind::Fill(p) => PaintRef::Fill(p),
                PaintKind::Stroke(s) => PaintRef::Stroke(s),
            };
            DrawnPaint {
                paint,
                opacity: e.opacity.get(),
                blend: e.blend,
                is_base: false,
            }
        });
        base_fill.into_iter().chain(base_stroke).chain(extras)
    }

    /// **A LARGURA que esta forma de facto ocupa para lá do caminho** — o maior meio-traço entre a
    /// base e as camadas activas.
    ///
    /// ⚠️ É o que o hit-test e a moldura de exportação têm de perguntar: um contorno extra de `20`
    /// por baixo de um de `2` faz a forma ser **dez vezes** mais gorda do que o `stroke.width` diz,
    /// e medir só a base devolveria uma caixa que corta o desenho.
    #[must_use]
    pub fn widest_stroke(&self) -> f64 {
        self.paint_stack()
            .filter_map(|d| match d.paint {
                PaintRef::Stroke(s) => Some(s.width),
                PaintRef::Fill(_) => None,
            })
            .fold(0.0_f64, f64::max)
    }

    /// Há mais alguma coisa na pilha para lá do chão?
    #[must_use]
    pub fn has_extra_paints(&self) -> bool {
        self.paints.iter().any(PaintEntry::is_active)
    }
}
