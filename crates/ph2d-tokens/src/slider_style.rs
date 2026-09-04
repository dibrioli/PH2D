//! ⭐⭐⭐ **A APARÊNCIA DA LINHA DE PROPRIEDADE** — o desenho, o raio e a altura, escolhidos pelo
//! artista.
//!
//! Decisão do Enio, 2026-09-02, depois de ver os seis desenhos lado a lado no Widget Lab:
//!
//! > *"O padrão do APP deverá ser Sliders tipo **Underline**, **raio 4**, **linha 22**. Como opções
//! > de customização vamos disponibilizar os 4 primeiros desenhos. Também deixe como opções de
//! > customização raio e linha."*
//!
//! ⚠️ **Irmão exacto do [`TextRendering`](crate::TextRendering)**, e de propósito: os dois são
//! *aparência escolhida pelo artista, ortogonal ao [`Theme`](crate::Theme)*, publicados uma vez por
//! quadro pelo shell e lidos pelo pintor. Quem acrescentar um terceiro eixo de aparência segue esta
//! forma em vez de inventar a terceira.
//!
//! # ⛔ Os DOIS desenhos que foram construídos e NÃO shipam
//!
//! O laboratório estudou **seis**. O Enio escolheu **quatro**. Os outros dois ficam registados aqui
//! para que ninguém os reconstrua julgando-os esquecimento:
//!
//! | desenho | o que era | por que não shipa |
//! |---|---|---|
//! | `Notch` | `Bar` + uma marca vertical na posição do valor | recupera precisão que a `Bar` perde, ao preço de mais um elemento por linha — **não escolhido** |
//! | `Split` | rótulo FORA da caixa, à esquerda (o desenho do Blender) | era o **controlo negativo** do estudo: ele volta a gastar coluna fixa, que é a grandeza que a caixa única põe a zero |
//!
//! ⚠️ **A `Split` não é «uma opção a menos»: ela é a decisão.** Mantê-la na lista de customização
//! deixaria o artista escolher de volta os `154 px` de cromo que o redesenho inteiro existe para
//! apagar (`docs/UI_New_and_Simple/pesquisa/07` §2).

use crate::{Density, Radius};

/// Como uma linha de propriedade desenha o valor.
///
/// ⚠️ **Os nomes são os que o artista lê no ecrã**, e por isso são em inglês (regra do app).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SliderDesign {
    /// O preenchimento é uma linha fina no bordo de baixo. ⭐ **O padrão**, escolhido pelo Enio:
    /// o texto nunca compete com a barra.
    #[default]
    Underline,
    /// O preenchimento é o fundo inteiro da caixa (o do manual do Blender).
    Bar,
    /// Uma cápsula de preenchimento dentro de um sulco recuado.
    Inset,
    /// Sem caixa: preenchimento muito ténue e uma linha de base. O mais plano.
    Ghost,
}

impl SliderDesign {
    /// As quatro opções, **na ordem em que a customização as oferece**.
    ///
    /// ⚠️ A ordem começa no `Underline` porque ele é o padrão — a primeira entrada de um selector é
    /// onde o olho pousa, e pô-la a discordar do default faz o artista pensar que mudou algo.
    pub const ALL: [SliderDesign; 4] = [
        SliderDesign::Underline,
        SliderDesign::Bar,
        SliderDesign::Inset,
        SliderDesign::Ghost,
    ];

    /// O nome que aparece no ecrã.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SliderDesign::Underline => "Underline",
            SliderDesign::Bar => "Bar",
            SliderDesign::Inset => "Inset",
            SliderDesign::Ghost => "Ghost",
        }
    }

    /// A linha de ajuda que o selector mostra — **o que este desenho troca**, não o que ele é.
    #[must_use]
    pub const fn blurb(self) -> &'static str {
        match self {
            SliderDesign::Underline => {
                "2 px fill at the bottom \u{b7} cleanest text \u{b7} quietest at a glance"
            }
            SliderDesign::Bar => {
                "the fill is the whole background \u{b7} reads at a glance \u{b7} competes with the number"
            }
            SliderDesign::Inset => {
                "a capsule in a groove \u{b7} clearly a control \u{b7} spends height on framing"
            }
            SliderDesign::Ghost => {
                "flattest of all \u{b7} vanishes in a long list \u{b7} barely reads as draggable"
            }
        }
    }

    /// A seguinte da lista — para um chip que cicla.
    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    /// A anterior.
    #[must_use]
    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Os raios que a customização oferece, do mais redondo ao quadrado.
///
/// ⚠️ **São TOKENS, não literais** — e o `4` que o Enio escolheu é exactamente o `Radius::Xs`, que
/// é também o `default_corner_radius` do editor do Godot (`editor_theme_manager.h:89`), a
/// referência que ele nomeou.
///
/// ⛔ **Não há `Radius::None`**, e por isso a escada acaba em `Xs`: um `0` aqui seria um literal a
/// contornar o design system, e um token novo é decisão de quem o possui — não deste ficheiro.
pub const SLIDER_RADII: [Radius; 4] = [Radius::Xl, Radius::Lg, Radius::Sm, Radius::Xs];

/// As alturas de linha que a customização oferece. ⭐ O padrão é a **primeira** (`Compact`, 22 px).
pub const SLIDER_DENSITIES: [Density; 3] = [Density::Compact, Density::Cozy, Density::Comfortable];

/// **A aparência activa de uma linha de propriedade.**
///
/// ⚠️ **Guarda TOKENS, nunca px.** Um `radius_px: f32` aqui deixaria o valor sobreviver a uma
/// mudança do design system — e a razão de os tokens existirem é que ninguém guarde o número.
// ⚠️ Sem `Hash`: os dois campos são tokens que não o derivam, e alargar dois enums foundational
// para satisfazer um `derive` que ninguém consome é custo em código alheio por conforto local.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SliderStyle {
    pub design: SliderDesign,
    pub radius: Radius,
    pub density: Density,
}

impl Default for SliderStyle {
    /// ⭐ **O padrão do app, escrito pelo dono:** `Underline` · raio `4` · linha `22`.
    ///
    /// ⚠️ **Escrito à mão, não derivado dos `Default` de cada campo.** O `Density::default()` é
    /// `Comfortable` (32) por causa do Pencil, e herdá-lo aqui contradiria a decisão em silêncio.
    fn default() -> Self {
        Self {
            design: SliderDesign::Underline,
            radius: Radius::Xs,
            density: Density::Compact,
        }
    }
}

impl SliderStyle {
    /// O raio em px, já resolvido.
    #[must_use]
    pub fn radius_px(self) -> f32 {
        self.radius.px()
    }

    /// A altura da linha em px, já resolvida.
    #[must_use]
    pub const fn row_h_px(self) -> f32 {
        self.density.row_h_px()
    }
}

/// ⭐⭐⭐ **A APARÊNCIA do app: a de sempre, ou o redesenho de 2026-09.**
///
/// Enio, 2026-09-03, ao mandar integrar: *«essa nova UI ainda deve ficar desativada até que esteja
/// concluída. Por enquanto permanece a antiga.»*
///
/// ⚠️ **`Classic` é o caminho de OMISSÃO**, e isso não é um detalhe de configuração — é o que
/// permite a linha entrar no `main` sem que ninguém veja um redesenho a meio. O redesenho liga-se
/// com `PH2D_UI_NEW=1`, como o `PH2D_FLIP_NEW_ENGINE` e o `PH2D_RETOPO_EXTRACT` fazem nos módulos
/// deles.
///
/// ⛔ **O que ele NÃO governa, de propósito:** as **correcções** que a linha fez ao caminho antigo
/// — a deriva do cursor, a roda do rato sobre a bancada, o arrastar do corpo de um painel para o
/// rolar. Um defeito curado não é uma aparência nova, e escondê-lo atrás de um interruptor seria
/// deixá-lo por curar para quem não o liga.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UiLook {
    /// A UI que shipa desde sempre: rótulo | trilha | caixa numérica, marca à esquerda,
    /// interruptor deslizante, sem coluna de animação.
    #[default]
    Classic,
    /// O redesenho: a caixa única, a marca à direita, o interruptor fundido, a coluna de animação.
    Redesign,
}

impl UiLook {
    /// Todos, na ordem em que se lêem.
    pub const ALL: [Self; 2] = [Self::Classic, Self::Redesign];

    /// O nome que aparece no ecrã (inglês — regra do app).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Redesign => "Redesign",
        }
    }

    /// A leitura do ambiente. ⚠️ Só `1` liga — qualquer outra coisa é a UI de sempre, incluindo a
    /// variável ausente, vazia ou com lixo. *Um interruptor que liga com o que não percebe é um
    /// interruptor que se liga sozinho.*
    #[must_use]
    pub fn from_env_value(v: Option<&str>) -> Self {
        match v {
            Some("1") => Self::Redesign,
            _ => Self::Classic,
        }
    }
}
