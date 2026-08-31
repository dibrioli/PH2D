//! ⭐ **A LEI DO DIVISOR do centro** — a fracção, os limites dela e o sub-rectângulo que a cena
//! ocupa quando o grafo de nós parte a área ao meio.
//!
//! ⚠️ **Cortado do `screens/layout.rs` em 2026-08-31 pelo tecto de LOC (711/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro é *a geometria de um quadro* (onde cada banda fica); isto é
//! *a lei de uma fracção* — o clamp, o `t`, e o contrato entre o `set_viewport` da cena e o chrome
//! que projecta mundo↔tela. Ela não tem um `Rect` do quadro em lado nenhum, e é por isso que sai
//! inteira.

/// Split of the center region into the scene viewport and the Motion Nodes
/// graph, applied while the Motion tool is active (Motion Nodes M0.T4).
///
/// `t` is the fraction of the center band that the **scene** occupies — the top
/// slice in [`Self::Horizontal`], the left slice in [`Self::Vertical`] — clamped
/// to [`Self::T_MIN`]..=[`Self::T_MAX`]. `None` = no split (the scene fills the
/// whole center; the graph is hidden), the default for every non-Motion tool.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CenterSplit {
    /// No split — scene fills the center, graph hidden.
    None,
    /// Horizontal divider: scene on top, graph on the bottom (Cavalry layout).
    Horizontal { t: f32 },
    /// Vertical divider: scene on the left, graph on the right (TouchDesigner).
    Vertical { t: f32 },
}

impl CenterSplit {
    /// Divider clamp — the scene (and graph) always keep at least a quarter.
    pub const T_MIN: f32 = 0.25; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.
    pub const T_MAX: f32 = 0.75; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.
    /// Default split fraction — the scene gets 55 % of the center (plan §2.1).
    pub const T_DEFAULT: f32 = 0.55; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.

    /// Clamp a raw fraction into the legal divider range. NaN-aware
    /// (`safe_clamp`): a divider drag that produced a NaN `t` collapses to the
    /// lower bound instead of poisoning the layout.
    pub fn clamp_t(t: f32) -> f32 {
        crate::math::safe_clamp(t, Self::T_MIN, Self::T_MAX)
    }

    /// `true` for a horizontal or vertical split (the graph is visible).
    pub fn is_split(self) -> bool {
        !matches!(self, Self::None)
    }

    /// O sub-retângulo `[x, y, w, h]` (px do alvo, **ancorado no topo-esquerda**) que a
    /// CENA ocupa quando o centro está dividido — a MESMA fração que o painel do grafo
    /// usa (top para `Horizontal`, esquerda para `Vertical`). `None` quando não há split
    /// (a cena é a janela cheia).
    ///
    /// **PORTA ÚNICA (2026-07-25):** o *render* da cena (`present`, via
    /// `Camera2d::uniform_for_subrect` + `set_viewport`) E todo o *chrome* que mapeia
    /// mundo↔tela sobre a cena (a grade do mundo, o gizmo de field e o drag dele)
    /// derivam a projeção DAQUI. Antes, o present calculava o sub-retângulo e o chrome
    /// projetava a janela CHEIA — duas cópias que discordavam, e um ponto de mundo caía
    /// comprimido na banda (cena) e cheio (grade+gizmo). Era o **drift crônico do Motion**
    /// (a cena divide o centro, mas a grade/gizmo ignoravam). Como o sub-retângulo é
    /// ancorado em `(0,0)`, o chrome só precisa das DIMS: `view_proj_for_subrect(w,h)` é
    /// idêntico a `view_proj(WindowSize{w,h})`, então passar `[r[2], r[3]]` como a janela
    /// do `world_to_screen`/`screen_to_world` casa o chrome com o `set_viewport` da cena.
    ///
    /// ⚠️⚠️ **O SUB-RETÂNGULO É UMA CONTAGEM DE PIXELS, e por isso ele sai INTEIRO daqui**
    /// (report do Enio, 2026-08-25: *«no modo motion a imagem de referência sofre um drift
    /// no pan com o mouse»*, refinado para *«acontece para Object e Chip, não para Star»*).
    ///
    /// `h · t` quase nunca é inteiro — `768 · 0,55 = 422,4`, `1022 · 0,55 = 562,1` —, e a
    /// fracção fazia esta função dar **duas respostas à mesma pergunta**:
    ///
    /// | quem pergunta | o que recebia |
    /// |---|---|
    /// | `set_viewport` do passe de sprites | **422,4** (o `f32` cru) |
    /// | `set_scissor_rect`, ao lado dele | 422 (`as u32`) |
    /// | `scene_camera_window` → o Vello e o pan | 422 (`as u32`) |
    ///
    /// ⇒ o conteúdo RASTER era desenhado com `422,4/10` pixels por unidade de mundo e o
    /// VECTORIAL com `422/10` — uma diferença de escala de **0,095 %**. Estática ela é
    /// sub-pixel; **num pan ela é um movimento**: a imagem anda `0,095 %` mais que o cursor
    /// e o traço anda exacto, então as duas separam-se enquanto se arrasta e voltam a juntar-se
    /// quando se volta. Foi exactamente o que o Enio viu, e é por isso que a `Star` (vectorial)
    /// não derivava. Medido nos dois tamanhos de janela dos logs dele: `0,18 px` por 1000 px
    /// de arrasto a 1022, e `0,95 px` a 768 — *quanto MENOR a janela, pior*.
    ///
    /// ⚠️ **A cura é o arredondamento estar AQUI e não em cada consumidor.** Um `as u32` no
    /// `scene_camera_window` já existia e não bastou: enquanto a porta devolvesse a fracção,
    /// quem a usasse crua discordava de quem a truncasse. *Um valor que é pixels não pode
    /// sair fraccionário da porta que o define.*
    #[must_use]
    pub fn scene_viewport(self, w: f32, h: f32) -> Option<[f32; 4]> {
        // `floor`, e não `round`: o `scene_window_wh` faz `as u32` (que trunca) sobre este
        // mesmo número, e as duas conversões TÊM de dar o mesmo inteiro. Arredondar aqui e
        // truncar lá reintroduziria a divergência num degrau diferente.
        match self {
            Self::Horizontal { t } => Some([0.0, 0.0, w, (h * t).floor().max(1.0)]),
            Self::Vertical { t } => Some([0.0, 0.0, (w * t).floor().max(1.0), h]),
            Self::None => None,
        }
    }

    /// `true` for a vertical split (side-by-side, divider drawn vertically) —
    /// the shell reads this to pick the divider's resize cursor (`EwResize` for
    /// a vertical divider, `NsResize` for a horizontal one).
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// The current divider fraction (or [`Self::T_DEFAULT`] when not split).
    pub fn t(self) -> f32 {
        match self {
            Self::Horizontal { t } | Self::Vertical { t } => t,
            Self::None => Self::T_DEFAULT,
        }
    }

    /// Same orientation, new (clamped) fraction. `None` stays `None`.
    pub fn with_t(self, t: f32) -> Self {
        let t = Self::clamp_t(t);
        match self {
            Self::Horizontal { .. } => Self::Horizontal { t },
            Self::Vertical { .. } => Self::Vertical { t },
            Self::None => Self::None,
        }
    }

    /// Switch to a horizontal split (scene on top), preserving the fraction.
    pub fn to_horizontal(self) -> Self {
        Self::Horizontal {
            t: Self::clamp_t(self.t()),
        }
    }

    /// Switch to a vertical split (scene on the left), preserving the fraction.
    pub fn to_vertical(self) -> Self {
        Self::Vertical {
            t: Self::clamp_t(self.t()),
        }
    }
}
