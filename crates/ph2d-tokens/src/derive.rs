//! ⭐⭐⭐ **A DERIVAÇÃO — um tema moderno nasce de CINCO entradas, e nenhum slot é escrito à mão.**
//!
//! Porte das regras de cor do tema *Modern* do Godot 4.6 (`editor/themes/theme_modern.cpp`,
//! `populate_shared_styles` + `_get_base_color` — **MIT**, vendorizado em
//! `docs/UI_New_and_Simple/referencias/godot-editor-src/`). A decisão do Enio (2026-09-04) está
//! em [`crate::theme`]; a pesquisa que a fundamenta em `pesquisa/08_modelos_com_codigo_para_seguir.md`.
//!
//! # As regras, uma a uma (as do Godot, com o nome dele ao lado)
//!
//! | papel | como nasce |
//! |---|---|
//! | `mono` | branco num tema escuro, preto num claro — *«will be used to generate the rest»* |
//! | `dark_color_1` | `base.lerp(preto, contraste · 1,15)` |
//! | `dark_color_3` | `base` com **V** de HSV puxado para 0 por `contraste · 0,8` e **S** × `0,9` |
//! | `contrast_color_1` | `base.lerp(mono, max(contraste, 0,3) · 1,15)` |
//! | `contrast_color_2` | `base.lerp(mono, max(contraste, 0,3) · 1,725)` |
//! | `highlight` | `acento @ 0,275` |
//! | `font` · `font_secondary` · `font_disabled` | `mono @ 0,75` · `@ 0,55` · `@ 0,35` |
//! | `info` · `success` · `warning` · `error` | fixos: `(0,7 0,8 1)` · `(0,45 0,95 0,5)` · `(0,83 0,78 0,62)` · `(1 0,47 0,42)` — e escurecidos num tema claro, como lá |
//! | `extra_border_1/2` | `mono @ 0,4` · `@ 0,2` — só com *Draw Extra Borders* (o preset OLED) |
//!
//! ⚠️ **O `lerp` é o do Godot: componente a componente em sRGB, não em luz linear.** Portar
//! «melhor» daria outras cores, e o que se quer é *as dele*.
//!
//! # ⚠️ ACHATAR o alfa é decisão deste porte, não do Godot
//!
//! O Godot escreve `font_color = mono × (1,1,1,0,75)` e deixa o alfa viajar até ao pintor. Aqui
//! os slots que hoje são **opacos** no `tokens.json` continuam opacos: a cor com alfa é
//! **composta sobre a base** na derivação (`over`). Duas razões, e nenhuma é gosto: o gate de
//! contraste ([`crate::contrast`]) mede a cor do token e não a compõe, e há pintores que
//! constroem o `Color` do Vello a partir dos três canais. Os slots que hoje carregam alfa
//! (`bg-scrim`, `rail-bg`, `focus-ring`, `grid-*`, `graph-grid`, `graph-backdrop-*`,
//! `graph-marquee`, `graph-inert`) continuam a carregá-lo.
//!
//! # ⚠️ As cores de DADO não se derivam — emprestam-se
//!
//! `node-cat-*`, `port-*`, `curve-*`, `wire-fire-glow`, `graph-backdrop-*`, `attr-write`: são as
//! únicas com matiz por direito (a lei que os quatro modelos planos partilham — `pesquisa/08 §3`),
//! e já estão calibradas em OKLCH *dark-safe* no `tokens.json`. Um tema moderno escuro lê-as da
//! tabela do `forge`, um claro da do `sunstone`. ⭐ Os eixos `axis-x/y/z` são os do Godot.

use crate::color::{Color, ColorToken};
use crate::theme::Theme;

/// As cinco entradas de um tema moderno (as três de cor aqui; raio e espaçamento vivem em
/// [`crate::visuals::Chrome`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Inputs {
    /// `interface/theme/base_color`.
    pub base: Rgb,
    /// `interface/theme/accent_color`.
    pub accent: Rgb,
    /// `interface/theme/contrast` — negativo num tema claro (a «elevação» escurece).
    pub contrast: f32,
    /// Tema escuro ⇒ `mono` é branco.
    pub dark: bool,
    /// `interface/theme/draw_extra_borders` — o preset OLED liga-o, porque num fundo preto o
    /// contraste já não separa nada.
    pub extra_borders: bool,
}

/// Uma cor sRGB em `0..1`, o espaço em que o Godot deriva.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// `Color::lerp` do Godot — componente a componente.
    #[must_use]
    pub fn lerp(self, to: Self, t: f32) -> Self {
        Self::new(
            self.r + (to.r - self.r) * t,
            self.g + (to.g - self.g) * t,
            self.b + (to.b - self.b) * t,
        )
        .clamp()
    }

    #[must_use]
    pub(crate) fn clamp(self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    /// `self` com alfa `a`, **composto sobre** `under` — o achatamento descrito no topo.
    #[must_use]
    pub fn over(self, under: Self, a: f32) -> Self {
        under.lerp(self, a)
    }

    /// `_get_base_color` do Godot: V de HSV puxado para 0 por `contrast · dim`, S multiplicado.
    #[must_use]
    pub fn dimmed(self, contrast: f32, dim: f32, sat_mult: f32) -> Self {
        let final_contrast = if dim < 0.0 {
            contrast.clamp(-0.1, 0.5)
        } else {
            contrast
        };
        let (h, s, v) = self.to_hsv();
        let v = (v + (0.0 - v) * (final_contrast * dim)).clamp(0.0, 1.0);
        Self::from_hsv(h, s * sat_mult, v)
    }

    fn to_hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let v = max;
        let s = if max > 0.0 { delta / max } else { 0.0 };
        let h = if delta <= f32::EPSILON {
            0.0
        } else if (max - self.r).abs() <= f32::EPSILON {
            ((self.g - self.b) / delta).rem_euclid(6.0)
        } else if (max - self.g).abs() <= f32::EPSILON {
            (self.b - self.r) / delta + 2.0
        } else {
            (self.r - self.g) / delta + 4.0
        } / 6.0;
        (h, s, v)
    }

    fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let s = s.clamp(0.0, 1.0);
        if s <= 0.0 {
            return Self::new(v, v, v);
        }
        let h6 = (h.rem_euclid(1.0)) * 6.0;
        let i = h6.floor();
        let f = h6 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        match i as i32 {
            0 => Self::new(v, t, p),
            1 => Self::new(q, v, p),
            2 => Self::new(p, v, t),
            3 => Self::new(p, q, v),
            4 => Self::new(t, p, v),
            _ => Self::new(v, p, q),
        }
    }

    /// Opaca.
    #[must_use]
    pub fn color(self) -> Color {
        self.with_alpha(1.0)
    }

    /// Com o alfa que o slot carrega hoje.
    #[must_use]
    pub fn with_alpha(self, a: f32) -> Color {
        let c = self.clamp();
        Color {
            r: (c.r * 255.0).round() as u8,
            g: (c.g * 255.0).round() as u8,
            b: (c.b * 255.0).round() as u8,
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }
}

/// O `default_contrast` do Godot — o piso das duas cores de contraste.
const DEFAULT_CONTRAST: f32 = 0.3;

/// Os papéis DERIVADOS — a tabela intermédia entre as cinco entradas e os ~83 slots.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Roles {
    pub base: Rgb,
    pub accent: Rgb,
    pub mono: Rgb,
    pub mono_inv: Rgb,
    pub dark_1: Rgb,
    pub dark_3: Rgb,
    /// ⭐⭐ **O FUNDO DE UM PAINEL — mais fundo que o `dark_1`, e por uma razão medida.**
    ///
    /// O `Bg1` responde a DUAS perguntas neste app: *«de que cor é o fundo do CANVAS»*
    /// ([`crate::ColorToken::Bg1`], via `hero::canvas_backdrop`) e *«de que cor é um CARTÃO dentro
    /// de um painel»* (os sete cartões do Painter, as fileiras do Inspector). Com o painel em
    /// `dark_1` os dois ficavam a **4/255** um do outro — *«o fundo dos cards tem tão pouco
    /// contraste com o fundo dos painéis que quase não podem ser diferenciados»* (Enio,
    /// 2026-09-05).
    ///
    /// ⛔ **Mover o `Bg1` para cima foi a tentativa ERRADA** (2026-09-05): ele clareia o CANVAS
    /// junto — *«mudou a cor do canvas»*. E subir o cartão para o `Bg2` apagaria os botões, que
    /// pintam `Bg2` em repouso. ⇒ quem desce é o **painel**, que é cromo e não tem outra
    /// pergunta agarrada; o canvas fica byte a byte no que o dono aprovou. É também o que o
    /// Blender faz: os painéis são mais escuros que a área de trabalho.
    pub panel: Rgb,
    /// ⭐⭐ **O CHÃO da janela** — um degrau abaixo do painel, na mesma família multiplicativa.
    ///
    /// É o que faz cada área ler-se como um cartão: sem ele o painel É o tom mais escuro, e a
    /// divisória entre duas áreas não tem nada para mostrar.
    pub ground: Rgb,
    /// ⭐⭐ **O fundo que o PINTOR DOS CARTÕES usa numa secção.** É o `dark_3` (o `bg-1`) em todo
    /// tema cuja escada separa — e um degrau ABSOLUTO acima do painel onde ela COLAPSA.
    ///
    /// ⛔ **Não é o `bg-1`**: esse slot é também o fundo do CANVAS (`hero::canvas_backdrop`), que o
    /// dono aprovou byte a byte e já devolveu uma vez (*«mudou a cor do canvas»*, 2026-09-05). O
    /// degrau vive só onde a pergunta é «de que cor é o cartão?» — [`card_surface`].
    ///
    /// ⛔ Report do dono (2026-09-30): *«No tema Black o fundo do painel e o fundo dos cards é
    /// igual e desse modo os cards não são visíveis»*. Com a base preta e `contrast = 0` a família
    /// multiplicativa devolve preto para o painel, o cartão e o sub-cartão, e a *Draw Extra
    /// Borders* que devia separá-los **nunca chegava ao cartão**: o pintor dos cartões só
    /// PREENCHE. *O tema declarava uma moldura; o cartão não a recebia.*
    pub card: Rgb,
    /// O fundo de um cartão de **subsecção** — a `base` (o `bg-2`) onde a escada separa, e um degrau
    /// acima do [`Self::card`] onde ela colapsa.
    pub subcard: Rgb,
    pub contrast_1: Rgb,
    pub contrast_2: Rgb,
    pub font: Rgb,
    pub font_secondary: Rgb,
    pub font_hover: Rgb,
    pub font_disabled: Rgb,
    pub info: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub error: Rgb,
    /// A alfa das bordas extra (0 sem *Draw Extra Borders*).
    pub extra_border_a: f32,
    pub contrast: f32,
}

/// **O degrau entre DUAS SUPERFÍCIES vizinhas**, em fracção de canal (≈ 10/255).
///
/// ⛔ Absoluto de propósito — ver a nota no `ground`. O valor é o piso a que a separação ainda se
/// lê nos quatro temas modernos, e o gate `the_ground_stands_under_every_panel` mede-o.
///
/// ⭐ **Ele nasceu para o CHÃO e serve a mesma pergunta para o CAMPO** (2026-09-14): *«duas
/// superfícies que se tocam têm de se ler como duas»*. A `visuals::Chrome::modern` afasta o fundo
/// de um campo das três superfícies em que ele pode assentar por este mesmo degrau — ⛔ e isso
/// **não** é herdar a resposta de outra pergunta: é a MESMA pergunta (separação entre superfícies
/// nos mesmos quatro temas), que é a cerca que o gate dos cartões planta ao recusar herdar os
/// `12/255` do par *cartão-contra-painel*.
pub const SURFACE_STEP: f32 = 0.04;

impl Inputs {
    /// As entradas de um tema moderno; `None` para a família clássica.
    #[must_use]
    pub fn of(theme: Theme) -> Option<Self> {
        // ⚠️ Os quatro primeiros NASCERAM da tabela `color_preset` do Godot 4.6 (MIT) — todos
        //    cinzentos com um azul de acento. ⭐ Em 2026-09-30 o dono pediu-os «interessantes como
        //    os novos» (*«ficaram um pouco sem graça»*), e cada um ganhou uma MATIZ na base e um
        //    acento mais vivo, guardando o carácter do nome: o `Dark` continua o mais escuro e
        //    azul, o `Gray` o meio-tom, o `Light` o claro e o `Oled` o preto puro com bordas. O
        //    Godot fica como a ORIGEM da regra (as cinco entradas e a derivação), não das cores.
        Some(match theme {
            // `#1e2433` (meia-noite azulada) + `#4aa3ff` (azul eléctrico) — era `#292929` + `#569eff`.
            Theme::Dark => Self {
                base: Rgb::new(0.118, 0.141, 0.2),
                accent: Rgb::new(0.29, 0.64, 1.0),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            // `#3d3833` (grafite quente) + `#2ec4b0` (verde-água) — era `#3d3d3d` + `#70bafa`.
            Theme::Gray => Self {
                base: Rgb::new(0.24, 0.22, 0.2),
                accent: Rgb::new(0.18, 0.77, 0.69),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            // `#e6dccb` (papel) + `#4150d8` (índigo) — era `#e6e6e6` + `#2e80ff`. ⚠️ A base não
            //    pode ser mais clara que a do Godot: o painel sobe acima dela e satura no `255`,
            //    e o degrau cartão/painel encolhe (medido: `#ede4d3` deu `10/255` contra `12`).
            Theme::Light => Self {
                base: Rgb::new(0.902, 0.863, 0.796),
                accent: Rgb::new(0.255, 0.314, 0.847),
                contrast: -0.06,
                dark: false,
                extra_borders: false,
            },
            // preto puro (fica — é o que o nome promete) + `#b57bff` (violeta néon) — era `#73bfff`.
            Theme::Oled => Self {
                base: Rgb::BLACK,
                accent: Rgb::new(0.71, 0.482, 1.0),
                contrast: 0.0,
                dark: true,
                extra_borders: true,
            },
            // ⭐⭐ **Os quatro COLORIDOS** (2026-09-30, ver [`Theme::PlumberRed`]). ⚠️ Estes NÃO
            //    vêm do Godot — são as primeiras cores escolhidas à mão da família moderna, por
            //    ordem do dono. O que continua da regra é o resto: cinco entradas, a mesma
            //    derivação, e o mesmo gate de contraste WCAG (`contrast_tests.rs`) a medi-los.
            // `#b3202a` (o boné) + `#ffd23c` (os botões).
            Theme::PlumberRed => Self {
                base: Rgb::new(0.70, 0.125, 0.165),
                accent: Rgb::new(1.0, 0.824, 0.235),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            // `#17702f` (o boné) + `#6db8ff` (o macacão, clareado para se ler sobre o verde).
            Theme::PlumberGreen => Self {
                base: Rgb::new(0.09, 0.44, 0.184),
                accent: Rgb::new(0.427, 0.722, 1.0),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            // `#4a2166` + `#ff8a3d`.
            Theme::Sunset => Self {
                base: Rgb::new(0.29, 0.13, 0.40),
                accent: Rgb::new(1.0, 0.541, 0.239),
                contrast: 0.3,
                dark: true,
                extra_borders: false,
            },
            // `#e3bdd1` + `#c0187e` — claro. ⚠️ Era `#f7cfe3` com o contraste `−0,06` do `light`, e o
            //    cartão ficava a `9/255` do painel (medido em 2026-09-30, quando o gate
            //    `a_card_stands_off_its_panel` passou a varrer os oito modernos): o painel satura no
            //    `255` do vermelho — a cerca que o `Light` já escreve acima — e numa base SATURADA o
            //    `dimmed` do cartão tira saturação, o que CLAREIA o verde e o azul e come o degrau.
            //    Medido: só escurecer a base dá `10`, só o contraste `−0,08` dá `11`; as duas juntas passam.
            Theme::Candy => Self {
                base: Rgb::new(0.89, 0.74, 0.82),
                accent: Rgb::new(0.753, 0.094, 0.494),
                contrast: -0.08,
                dark: false,
                extra_borders: false,
            },
            _ => return None,
        })
    }

    /// Os papéis, pelas regras do Godot.
    #[must_use]
    pub fn roles(self) -> Roles {
        let mono = if self.dark { Rgb::WHITE } else { Rgb::BLACK };
        let mono_inv = if self.dark { Rgb::BLACK } else { Rgb::WHITE };
        let c = self.contrast;
        let c_floor = c.max(DEFAULT_CONTRAST);
        let base = self.base;
        let (info, success, warning, error) = if self.dark {
            (
                Rgb::new(0.7, 0.8, 1.0),
                Rgb::new(0.45, 0.95, 0.5),
                Rgb::new(0.83, 0.78, 0.62),
                Rgb::new(1.0, 0.47, 0.42),
            )
        } else {
            // «Darken some colors to be readable on a light background.»
            (
                Rgb::new(0.35, 0.6, 0.9),
                Rgb::new(0.45, 0.95, 0.5).lerp(mono, 0.35),
                Rgb::new(0.83, 0.49, 0.01),
                Rgb::new(0.8, 0.22, 0.22),
            )
        };
        // ⭐ O painel sai antes do `Roles` porque o CHÃO deriva DELE, e não da base: no tema claro
        //    a escada da base satura (o painel já está a 1/255 do branco), e um degrau «mais um
        //    passo na mesma direcção» não separa nada. Derivar do painel dá a mesma leitura nos
        //    quatro: uma superfície um degrau ABAIXO daquela em que as áreas assentam.
        let panel = base.lerp(Rgb::BLACK, c * 1.8);
        let dark_3 = base.dimmed(c, 0.8, 0.9);
        let (card, subcard) = card_surfaces(panel, dark_3, base, self.dark);
        Roles {
            base,
            accent: self.accent,
            mono,
            mono_inv,
            dark_1: base.lerp(Rgb::BLACK, c * 1.15),
            dark_3,
            // O `1.8` é o degrau MEDIDO: com ele o cartão (`Bg1`) fica a 12/255 do painel no
            // Dark, 19 no Gray e 14 no Light (gate `a_card_stands_off_its_panel`), contra os 4
            // que o dono viu. ⛔ O OLED tem base preta: a família multiplicativa colapsa lá, e
            // quem separa é a *Draw Extra Borders*, como no Godot.
            panel,
            // ⚠️ O degrau seguinte da mesma escada. ⛔ No OLED a base é preta e a família colapsa
            //    — lá quem separa é a *Draw Extra Borders*, como no Godot e como o `panel` já nota.
            // ⚠️ **Um degrau ABSOLUTO, e não uma fracção.** Uma fracção do painel dá 9/255 no
            //    Dark e **114** no Light — porque o painel claro está a 1/255 do branco e a mesma
            //    fracção cobre uma distância enorme. *Uma escada relativa mede-se em passos
            //    diferentes conforme onde se está nela.*
            ground: Rgb::new(
                panel.r - SURFACE_STEP,
                panel.g - SURFACE_STEP,
                panel.b - SURFACE_STEP,
            )
            .clamp(),
            card,
            subcard,
            contrast_1: base.lerp(mono, c_floor * 1.15),
            contrast_2: base.lerp(mono, c_floor * 1.725),
            font: mono.over(base, 0.8),
            // ⭐ **`0.70`, não o `0.55` do Godot** — *«as fonts dos cards podem ser um pouco mais
            //    claras para aumentar contraste»* (Enio, 2026-09-05): os títulos e os rótulos dos
            //    cartões do Painter são todos `Text2`. O `font` sobe junto (`0.75 → 0.8`) para a
            //    hierarquia entre o rótulo e o valor continuar a ler-se.
            font_secondary: mono.over(base, 0.7),
            font_hover: mono.over(base, 0.85),
            font_disabled: mono.over(base, if self.dark { 0.35 } else { 0.5 }),
            info,
            success,
            warning,
            error,
            extra_border_a: if self.extra_borders { 0.2 } else { 0.0 },
            contrast: c,
        }
    }
}

/// ⭐⭐ **Quantos degraus de [`SURFACE_STEP`] o cartão sobe acima do painel quando a escada
/// COLAPSA** — e o sub-cartão sobe [`COLLAPSED_SUBCARD_STEPS`].
///
/// Medido contra o que o dono aprovou nos temas que separam: no `Dark` o cartão fica a `12/255`
/// do painel e o sub-cartão `10` acima do cartão. Sobre PRETO a mesma distância lê-se menos (o
/// olho é logarítmico no escuro), logo o OLED ganha dois degraus (`20/255`, `#141414`) e o
/// sub-cartão três (`31/255`, `#1f1f1f`).
const COLLAPSED_CARD_STEPS: f32 = 2.0;
/// Ver [`COLLAPSED_CARD_STEPS`].
const COLLAPSED_SUBCARD_STEPS: f32 = 3.0;

/// ⭐⭐ **Os fundos dos dois cartões**, com a cerca do colapso.
///
/// A escada separa ⇒ `(dark_3, base)`, byte a byte o que shipava. A escada COLAPSA (o cartão a
/// menos de um [`SURFACE_STEP`] do painel no pior canal — o `Oled`, e qualquer tema futuro de base
/// preta ou branca pura) ⇒ degraus ABSOLUTOS a partir do painel, na direcção da elevação do tema.
fn card_surfaces(panel: Rgb, dark_3: Rgb, base: Rgb, dark: bool) -> (Rgb, Rgb) {
    let gap = (dark_3.r - panel.r)
        .abs()
        .max((dark_3.g - panel.g).abs())
        .max((dark_3.b - panel.b).abs());
    if gap >= SURFACE_STEP {
        return (dark_3, base);
    }
    let dir = if dark { 1.0 } else { -1.0 };
    let lift = |steps: f32| {
        let d = dir * steps * SURFACE_STEP;
        Rgb::new(panel.r + d, panel.g + d, panel.b + d).clamp()
    };
    (lift(COLLAPSED_CARD_STEPS), lift(COLLAPSED_SUBCARD_STEPS))
}

/// ⭐⭐ **O fundo de um cartão pintado pelo livro dos cartões — `Some` só onde a escada COLAPSA.**
///
/// `None` diz *«pinte o token de sempre»* (e o chamador resolve-o, com as sobreposições do
/// projecto), logo os temas cuja escada separa ficam byte a byte o que eram. `Some` é o degrau
/// absoluto do [`card_surfaces`] — hoje só o `Oled`, e qualquer tema futuro de base preta ou branca
/// pura. A família clássica não tem cartões (desenha o risco) e devolve `None`.
#[must_use]
pub fn card_surface(theme: Theme, subsection: bool) -> Option<Color> {
    let r = Inputs::of(theme)?.roles();
    let (natural, lifted) = if subsection {
        (r.base, r.subcard)
    } else {
        (r.dark_3, r.card)
    };
    (lifted != natural).then(|| lifted.color())
}

/// A alfa do `highlight_color` do Godot — `Color(accent, 0.275)`.
const HIGHLIGHT_A: f32 = 0.275;

/// **A cor de um slot num tema moderno.**
///
/// ⚠️ Cobre TODA chave de [`ColorToken`] — o gate `every_token_derives_in_every_modern_theme`
/// percorre `ColorToken::ALL` × [`Theme::MODERN`], então uma chave nova que não entre aqui
/// reprova em vez de estourar no primeiro quadro.
#[must_use]
pub(crate) fn colour(theme: Theme, token: ColorToken) -> Color {
    let inputs = Inputs::of(theme).expect("um tema moderno tem entradas");
    let r = inputs.roles();
    let key = token.key();
    // As cores de DADO: emprestadas da família clássica (ver o topo).
    let borrowed = if inputs.dark {
        Theme::Forge
    } else {
        Theme::Sunstone
    };
    let borrow = || token.factory(borrowed);
    let c = r.contrast.max(DEFAULT_CONTRAST);
    match key {
        // ── superfícies ──
        "bg-0" => r.dark_1.color(),
        "bg-1" => r.dark_3.color(),
        "bg-2" => r.base.color(),
        "bg-3" => r.base.lerp(r.mono, c * 0.5).color(),
        "bg-elev" => r.base.lerp(r.mono, c * 0.3).color(),
        "panel-bg" => r.panel.color(),
        "window-ground" => r.ground.color(),
        // O trilho é da mesma família do painel: mais claro que ele faria o cromo lateral saltar
        // à frente do conteúdo.
        "rail-bg" => r.panel.with_alpha(0.85),
        "canvas" => r.base.lerp(Rgb::BLACK, c * 1.5).color(),
        "bg-scrim" => Rgb::BLACK.with_alpha(if inputs.dark { 0.6 } else { 0.4 }),
        // ── bordas ──
        "border" => r
            .mono
            .with_alpha(r.extra_border_a.max(if inputs.dark { 0.08 } else { 0.12 })),
        "border-strong" => r
            .mono
            .with_alpha((r.extra_border_a * 2.0).max(if inputs.dark { 0.2 } else { 0.25 })),
        "border-emph" => r.accent.color(),
        // ── texto ──
        "text-1" => r.font.color(),
        "text-2" => r.font_secondary.color(),
        "text-3" => r.mono.over(r.base, 0.45).color(),
        "text-disabled" => r.font_disabled.color(),
        // ── acento ──
        "accent" => r.accent.color(),
        "accent-hover" => r.accent.lerp(r.mono, 0.15).color(),
        "accent-press" => r.accent.lerp(r.mono_inv, 0.15).color(),
        "accent-soft" => r.accent.over(r.base, HIGHLIGHT_A).color(),
        "accent-fg" => r.mono_inv.color(),
        "selection" => r.accent.over(r.base, HIGHLIGHT_A).color(),
        "focus-ring" => r.accent.with_alpha(0.55),
        // ── estado ──
        "danger" => r.error.color(),
        "danger-soft" => r.error.over(r.base, HIGHLIGHT_A).color(),
        "success" => r.success.color(),
        "success-soft" => r.success.over(r.base, HIGHLIGHT_A).color(),
        "warn" => r.warning.color(),
        "warn-soft" => r.warning.over(r.base, HIGHLIGHT_A).color(),
        "info" => r.info.color(),
        "info-soft" => r.info.over(r.base, HIGHLIGHT_A).color(),
        // ── grelha e eixos ──
        "grid-line" => r.mono.with_alpha(0.12),
        "grid-axis" => r.accent.with_alpha(0.30),
        "axis-x" => Rgb::new(0.96, 0.20, 0.32).color(),
        "axis-y" => Rgb::new(0.53, 0.84, 0.01).color(),
        "axis-z" => Rgb::new(0.16, 0.55, 0.96).color(),
        // ── o grafo de nós ──
        "graph-bg" => r.dark_1.color(),
        "graph-grid" => r.mono.with_alpha(0.06),
        "graph-marquee" => r.accent.with_alpha(0.18),
        "graph-inert" => r.base.with_alpha(0.62),
        // ── a timeline: os 16 apelidos, por construção ──
        "timeline-curve"
        | "timeline-handle"
        | "timeline-key-selected"
        | "timeline-loop-brace"
        | "timeline-playhead"
        | "timeline-summary-ring" => r.accent.color(),
        "timeline-handle-line" | "timeline-loop-region" => {
            r.accent.over(r.base, HIGHLIGHT_A).color()
        }
        "timeline-row-alt" | "timeline-ruler-bg" => r.base.color(),
        "timeline-marker" | "timeline-summary-key" => r.warning.color(),
        "timeline-key-active" => r.accent.lerp(r.mono_inv, 0.15).color(),
        "timeline-missing" => r.error.color(),
        "timeline-key" => r.font.color(),
        "timeline-ruler-tick" => r.mono.over(r.base, 0.45).color(),
        // ── as cores de DADO: emprestadas ──
        k if k.starts_with("node-cat-")
            || k.starts_with("port-")
            || k.starts_with("curve-")
            || k.starts_with("graph-backdrop-")
            || k == "wire-fire-glow"
            || k == "attr-write" =>
        {
            borrow()
        }
        other => panic!(
            "color token {other:?} nao tem regra de derivacao para o tema {theme:?} — \
             acrescente-a em ph2d-tokens/src/derive.rs"
        ),
    }
}
