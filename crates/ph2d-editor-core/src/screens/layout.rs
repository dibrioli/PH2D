//! Hero screen 4-zone layout — chrome constants + `HeroLayout` struct.
//!
//! ADR-0029 Phase B.1: moved from `ph2d-editor::screens::hero::style`
//! (chrome consts) and `ph2d-editor::screens::hero` (HeroLayout
//! struct + ctor) to editor-core so `PaintCtx` (also in editor-core)
//! can hold `&HeroLayout` without referring back to `ph2d-editor`.
//!
//! Hero-screen-specific (TopBar + LeftRail + Hierarchy + Inspector +
//! BottomHud). Panel-specific chrome (paint_panel_surface, etc.) lives
//! in `widget::panel_chrome`.

use crate::zones::Rect;
use ph2d_tokens::{
    EDGE_PAD_PX, HERO_VIEWPORT_H_PX, HERO_VIEWPORT_W_PX, HIER_ROW_H_PX, HIERARCHY_W_PX,
    HUD_BOTTOM_PAD_PX, HUD_H_PX, INSPECTOR_W_PX, TOPBAR_GAP_PX, TOPBAR_H_PX,
};

/// Default mockup viewport (iPad 12.9 landscape).
pub const HERO_VIEWPORT_W: f32 = HERO_VIEWPORT_W_PX;
pub const HERO_VIEWPORT_H: f32 = HERO_VIEWPORT_H_PX;

/// Padding from the screen edge to chrome (TopBar inset,
/// Hierarchy pinned-right inset, etc).
pub const EDGE_PAD: f32 = EDGE_PAD_PX;
pub const TOPBAR_H: f32 = TOPBAR_H_PX;
pub const TOPBAR_GAP: f32 = TOPBAR_GAP_PX;
/// Mirrors `crate::widget::tool_rail_width_px()`.
pub fn rail_w() -> f32 {
    crate::widget::tool_rail_width_px()
}
pub const INSPECTOR_W: f32 = INSPECTOR_W_PX;

/// **O ORÇAMENTO de altura que um painel do dock pode assumir** — não a altura dele.
///
/// ⛔ **Até 2026-08-30 isto era um TECTO GEOMÉTRICO** (`chrome_h.min(INSPECTOR_MAX_H)`) e era ele
/// que deixava a coluna da direita a parar **80 px antes do fundo** no viewport de referência —
/// metade do *«muitos espaços em todos os lugares»* que o Enio apontou com quatro setas. Um tecto
/// de altura é coisa de painel que FLUTUA; uma coluna ANCORADA vai de ponta a ponta.
///
/// ⚠️ **O número fica, e o consumidor dele também** — o `ph2d-panel-motion-params` mede contra
/// isto quantas linhas cabem, e é a única coisa que o lê. Ele deixa de ser *«a altura do dock»* e
/// passa a ser *«a altura que um painel pode contar ter»*: conservador no alvo de referência (a
/// banda tem 960), e ⚠️ **optimista numa janela baixa** — o que já era verdade antes, porque o
/// `min` dava a banda quando ela era menor. Quem quiser a altura REAL lê `layout.inspector.h`.
///
/// ⚠️ Nomeado (era literal solto no `Rect::new` abaixo) porque um painel docado precisa saber
/// **quanta altura existe** para decidir se o conteúdo dele cabe: o `motion-params` não rola, e
/// um teto de linhas que não é conferido contra esta altura só troca o corte do `.take()` pelo
/// corte da borda da tela. Uma segunda cópia deste número num gate divergiria no dia em que o
/// dock mudar de tamanho — então há um número, com um dono.
pub const INSPECTOR_MAX_H: f32 = 880.0; // LITERAL-PX-OK: Inspector max height cap
pub const HIERARCHY_W: f32 = HIERARCHY_W_PX;
pub const HUD_H: f32 = HUD_H_PX;
pub const HUD_BOTTOM_PAD: f32 = HUD_BOTTOM_PAD_PX;
pub const HIER_ROW_H: f32 = HIER_ROW_H_PX;

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

/// Quais colunas laterais estão ocupadas — [`crate::screens::dock_sides`].
///
/// Re-exportado aqui porque o `HeroLayout` o recebe por argumento e todo chamador o importa
/// junto com ele; o tipo mudou de ficheiro, não de dono.
pub use crate::screens::dock_seam::{ChromeBands, DOCK_SEAM_PX, DockSide};
pub use crate::screens::dock_sides::DockSides;

/// Pre-computed sub-region rects for one frame. Built once per
/// frame from a viewport rect — cheap.
#[derive(Copy, Clone, Debug)]
pub struct HeroLayout {
    pub viewport: Rect,
    pub top_bar: Rect,
    pub left_rail: Rect,
    pub inspector: Rect,
    /// Background-Removal panel slot — shares the Inspector's right-dock
    /// geometry. Only painted while the `bgremoval` tool is active (the
    /// panel's own visibility gate keys off `panel_visible("bgremoval")`,
    /// which the shell drives from the active-tool id).
    pub bgremoval: Rect,
    /// Padding panel slot — shares the Inspector's right-dock geometry,
    /// like [`Self::bgremoval`]. Only painted while the `padding` tool is
    /// active (the panel's own visibility gate keys off
    /// `panel_visible("padding")`, which the shell drives from the
    /// active-tool id).
    pub padding: Rect,
    /// Painter sidebar slot — shares the Inspector's right-dock geometry
    /// (W2.T2.1 plan §5). Only painted while the `painter` tool is active
    /// (the panel's own visibility gate keys off
    /// `panel_visible("painter_sidebar")`, which the shell drives from
    /// the active-tool id).
    pub painter_sidebar: Rect,
    /// Painter layers panel slot — shares the Inspector's right-dock
    /// geometry too (W3.T3.4 plan §6, mirror do [`Self::painter_sidebar`]).
    /// Only painted while the `painter` tool is active (the panel's own
    /// visibility gate keys off `panel_visible("painter_layers")`, which
    /// the shell drives from the active-tool id).
    pub painter_layers: Rect,
    pub hierarchy: Rect,
    pub bottom_hud: Rect,
    /// Visible canvas region (between rail/inspector on the left
    /// and hierarchy on the right, between TopBar and HUD vertically).
    ///
    /// ⚠️ **O doc acima descreve um layout ANCORADO e o código implementa full-bleed** — este
    /// rect É a viewport inteira, e os painéis flutuam por cima dele. A contradição é
    /// pré-existente e fica NOMEADA aqui até a docagem (A2) a resolver; quem quer a área em
    /// que o desenho de facto se vê usa [`Self::draw_area`].
    pub canvas: Rect,
    /// ⭐⭐ **A ÁREA de desenho — o que sobra da janela depois de o chrome DOCADO tirar a sua
    /// faixa**, e o hospedeiro das duas réguas.
    ///
    /// É a primeira peça do modelo de áreas (`docs/UI_New_and_Simple/spec/01_modelo_de_areas.md`
    /// §4, decisão D5 do Enio): *regiões são IRMÃS numa fila, nunca camadas empilhadas.* A régua
    /// e o trilho deixam de partilhar a origem `(0, 0)` da janela e passam a ocupar faixas
    /// disjuntas — e é por isso que o defeito desaparece **por construção**, sem uma verificação
    /// a defendê-lo: duas regiões não se tapam porque não partilham coordenada.
    ///
    /// | | tapado por chrome docado |
    /// |---|---:|
    /// | régua da esquerda ancorada em `canvas` (até 2026-08-30) | **86,8 %** — o trilho cobre-a inteira |
    /// | régua de cima ancorada em `canvas` | **29,4 %** — a barra de topo |
    /// | as duas ancoradas em `draw_area` | **0,0 %** |
    ///
    /// (medição em `docs/UI_New_and_Simple/medicoes/02_a_area_tapada.md`, alvo 1366 × 1024 =
    /// iPad Pro 12,9", que é o viewport de referência dos tokens.)
    ///
    /// ⚠️⚠️ **A PROJECÇÃO NÃO MUDA, e é isso que torna a cura barata.** O
    /// [`crate::grid::world_bounds`] deriva de `window_w`/`window_h`, nunca deste rect — o rect
    /// só decide **onde a faixa é desenhada** e quais traços entram nela (`ruler::in_band` já
    /// filtrava). ⇒ um traço marcado em 100 continua a cair no mesmo pixel de tela; ele apenas
    /// deixa de o fazer debaixo do trilho. *Uma régua que se mudasse e levasse a projecção
    /// consigo passaria a apontar para outro sítio, que é pior do que estar tapada.*
    ///
    /// ⛔ **Não é (ainda) o rect da CENA.** O sub-rectângulo da cena é ancorado em `(0, 0)` por
    /// construção em toda a cadeia (`CenterSplit::scene_viewport` devolve `[0, 0, w, h·t]` e
    /// todo consumidor lê só as DIMS) — dar-lhe uma ORIGEM é a obra da docagem (A2), e está
    /// nomeada com esse preço. Aqui a cena continua full-bleed, por baixo das réguas.
    pub draw_area: Rect,
    /// ⭐ **A fila de ferramentas** — a faixa que os chips do trilho ocupam quando estão na
    /// horizontal, por cima da área de desenho (Godot). Zero-altura enquanto ninguém a pede.
    ///
    /// ⚠️ Ela é irmã da régua, não da barra de menus: sai da **área**, entre as colunas, e não da
    /// janela inteira.
    pub tool_bar: Rect,
    /// ⚠️ **Quais colunas estavam OCUPADAS quando este layout foi construído.** O rect de uma
    /// coluna existe sempre (a geometria não depende do estado); o que depende é haver alguém lá.
    /// Sem isto, `dock_seam` oferecia agarre numa coluna vazia — chrome vivo e invisível.
    pub docks: DockSides,
    /// Scene viewport sub-rect (Motion Nodes M0.T4). Equals the full
    /// [`Self::canvas`] with no split; the top slice (`Horizontal`) or left
    /// slice (`Vertical`) of the center band when the Motion tool splits it.
    /// The scene is rendered into this rect via scissor + a sub-rect camera
    /// (M0.T13) when it differs from `canvas`.
    pub center_viewport: Rect,
    /// Motion Nodes graph sub-rect — the complement of [`Self::center_viewport`]
    /// in the center band. Zero-sized when the split is `None`.
    pub motion_graph: Rect,
    /// The timeline's band **inside the Motion workspace** (W4.T4). Zero-height until
    /// [`Self::dock_timeline_into_motion`] carves it out of the bottom of
    /// [`Self::motion_graph`] — which is what the shell does when the Motion tool and the
    /// timeline are both on screen. It was a reserved seam for a whole module's worth of time
    /// (Motion Nodes M0.T4); this is the module that landed in it.
    pub motion_timeline_slot: Rect,
    /// General timeline panel slot — a bottom-docked strip spanning the center
    /// band (between the two side chrome columns), floating over the lower edge
    /// of the scene. Painted only while `panel_visible("timeline")` (the shell
    /// drives that from the timeline toggle). Height [`TIMELINE_DOCK_H`].
    pub timeline: Rect,
    /// ⭐⭐ **A faixa de ABAS de cada encaixe**, na ordem de [`crate::screens::slot::Slot::ALL`] —
    /// **zero-altura** onde o encaixe tem menos de dois ocupantes, que é o estado de omissão do
    /// app. Escrita por [`Self::reserve_slot_tabs`]; ver `screens::hero::slot_tabs`.
    pub slot_tabs: [Rect; 6],
    /// Flip frame-strip slot (ADR-0114 W3) — a low bottom-docked band spanning the
    /// same column as the timeline. Painted only while the `flip` tool is active
    /// (`panel_visible("flip_frames")`, bridge-driven). Height [`FLIP_STRIP_H`].
    pub flip_strip: Rect,
}

/// Default docked height of the general timeline panel (px).
pub const TIMELINE_DOCK_H: f32 = 240.0; // LITERAL-PX-OK: timeline dock default height

/// The timeline's height when it docks **inside the Motion workspace** (W4.T4) — shorter than its
/// free-standing dock, because it is sharing a band with the node graph and the graph is the thing
/// the artist came for.
///
/// Under Motion the dope-sheet is mostly empty anyway: its tracks bind to ECS **objects**, and a
/// Motion parameter is not one (keyframing them is deferred). What the artist actually needs here
/// is the **transport, the ruler and the scrub** — the graph cooks on the playhead's tick — and
/// that fits.
pub const MOTION_TIMELINE_H: f32 = 200.0; // LITERAL-PX-OK: timeline height inside the Motion split

/// …but never more than this much of the graph. On a short window a fixed 200 px would leave the
/// node editor a sliver, and a dock that eats its host is a dock nobody wants.
const MOTION_TIMELINE_MAX_FRAC: f32 = 0.45; // LITERAL-PX-OK: a FRACTION of the graph, not a design token

/// Docked height of the Flip frame strip (px): title + toolbar row + the cells
/// row. It is a STRIP (one layer's cells), not a dope-sheet — the multi-layer
/// view is the global timeline's job (W6, deferred).
pub const FLIP_STRIP_H: f32 = 132.0; // LITERAL-PX-OK: Flip frame strip dock height

impl HeroLayout {
    pub fn for_viewport(viewport: Rect) -> Self {
        Self::for_viewport_mirrored(viewport, false)
    }

    pub fn for_viewport_mirrored(viewport: Rect, mirrored: bool) -> Self {
        Self::for_viewport_mirrored_with_rail_w(viewport, mirrored, rail_w())
    }

    /// Layout constructor with an explicit rail-column width. Used by
    /// the hero orchestrator when the user picks a non-default
    /// [`crate::widget::RailButtonSize`] preset in the Themes menu
    /// (2026-05-24) — the rail shrinks/grows and Inspector/Hierarchy
    /// x-positions follow.
    pub fn for_viewport_mirrored_with_rail_w(viewport: Rect, mirrored: bool, rail_w: f32) -> Self {
        Self::for_viewport_split(viewport, mirrored, rail_w, CenterSplit::None)
    }

    /// Layout constructor that also splits the center region for the Motion
    /// tool (M0.T4). With [`CenterSplit::None`] this is identical to
    /// [`Self::for_viewport_mirrored_with_rail_w`] — `center_viewport == canvas`,
    /// `motion_graph`/`motion_timeline_slot` zero-sized. Chrome (rail, panels,
    /// HUD) floats over the center exactly as before; only the scene/graph split
    /// is new.
    pub fn for_viewport_split(
        viewport: Rect,
        mirrored: bool,
        rail_w: f32,
        split: CenterSplit,
    ) -> Self {
        Self::for_viewport_docked(viewport, mirrored, rail_w, split, DockSides::BOTH)
    }

    /// O construtor que também sabe **quais colunas laterais estão abertas** — a única entrada
    /// que consegue resolver a [`Self::draw_area`] correctamente.
    ///
    /// ⚠️ Todos os outros construtores delegam aqui com [`DockSides::BOTH`], que é o estado do
    /// mockup de referência. O caminho de produção (`screens/hero/paint.rs`) pergunta ao
    /// `is_panel_visible`, e há gate a exigi-lo — reservar a coluna de um painel fechado põe a
    /// régua da esquerda a flutuar no meio do desenho.
    pub fn for_viewport_docked(
        viewport: Rect,
        mirrored: bool,
        rail_w: f32,
        split: CenterSplit,
        docks: DockSides,
    ) -> Self {
        Self::for_viewport_bands(
            viewport,
            mirrored,
            ChromeBands {
                rail_w,
                ..ChromeBands::DEFAULT
            },
            split,
            docks,
        )
    }

    /// ⭐⭐ **O construtor que recebe as BANDAS por medida, e não por modo** (2026-08-30).
    ///
    /// `rail_w` e `top_bar_h` são larguras/alturas, não interruptores — *«sem chrome legado»* é
    /// simplesmente `0.0` nos dois, e o layout não precisa de saber porquê. ⚠️ Um parâmetro
    /// `legacy_chrome: bool` faria este ficheiro conhecer uma fase de migração; uma medida a
    /// zero é a mesma aritmética de sempre.
    #[allow(clippy::too_many_arguments)]
    pub fn for_viewport_bands(
        viewport: Rect,
        mirrored: bool,
        bands: ChromeBands,
        split: CenterSplit,
        docks: DockSides,
    ) -> Self {
        let ChromeBands {
            rail_w,
            top_bar_h,
            left_dock_w,
            right_dock_w,
            tool_bar_h,
        } = bands;
        // ⭐⭐ **A BARRA DE TOPO É FLUSH e a BANDA vai até ao fundo** (Enio, 2026-08-30, com
        // quatro setas na foto: *«muitos espaços em todos os lugares»*). Ela era inset em
        // `EDGE_PAD` e a banda de chrome perdia `TOPBAR_GAP` por cima e a reserva do HUD por
        // baixo — 94 px em cima e 60 em baixo de espaço morto, com as colunas a boiar no meio.
        //
        // ⚠️ **O HUD continua a flutuar**, e é de propósito: ele é centrado (`x ∈ [443, 923]` no
        // alvo de referência) e as colunas vivem nas pontas, logo não se tocam — reservar-lhe uma
        // faixa custaria a altura das duas colunas para nada.
        let top_bar = Rect::new(viewport.x, viewport.y, viewport.w, top_bar_h);
        let chrome_top = top_bar.y + top_bar.h;
        let chrome_bot = viewport.y + viewport.h;
        let chrome_h = (chrome_bot - chrome_top).max(0.0);
        let left_rail = Rect::new(viewport.x, chrome_top, rail_w, chrome_h);
        // ⭐⭐ **AS COLUNAS SÃO FLUSH** (Enio, 2026-08-30, com foto): encostadas à borda da janela
        // de um lado e ao trilho do outro, sem o `EDGE_PAD` que as fazia ler como cartões a
        // flutuar. ⚠️ **Só ESTA das quatro utilizações do `EDGE_PAD` mudou** — ele continua a
        // separar a barra de topo da borda, a afastar o timeline e a dar o respiro entre a coluna
        // e a área de desenho. Zerá-lo global seriam quatro decisões numa.
        // ⚠️ **A largura segue o LADO, não o painel** — sob espelho a Hierarchy muda-se para a
        // coluna da direita e herda a largura DELA. Escrever `hierarchy_w` seria a inversão que o
        // compilador não vê, e o `side_columns()` existe pelo mesmo motivo.
        let (hier_w, insp_w) = if mirrored {
            (right_dock_w, left_dock_w)
        } else {
            (left_dock_w, right_dock_w)
        };
        let (hierarchy_x, inspector_x) = if mirrored {
            (viewport.x + viewport.w - hier_w, viewport.x + rail_w)
        } else {
            (viewport.x + rail_w, viewport.x + viewport.w - insp_w)
        };
        let inspector = Rect::new(inspector_x, chrome_top, insp_w, chrome_h);
        // Bg Removal panel shares the Inspector's right-dock x/width;
        // it replaces the Inspector visually while the tool is active.
        let bgremoval = inspector;
        // Padding panel shares the Inspector's right-dock geometry too.
        let padding = inspector;
        // Painter sidebar shares the Inspector's right-dock geometry —
        // takeover mode replaces Inspector visually (W2.T2.1 plan §5).
        let painter_sidebar = inspector;
        // Painter layers panel shares the Inspector's right-dock geometry
        // too (W3.T3.4 plan §6, mirror do painter_sidebar).
        let painter_layers = inspector;
        let hierarchy = Rect::new(hierarchy_x, chrome_top, hier_w, chrome_h);
        let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h);
        // Center split (M0.T4): partition the chrome band (chrome_top..chrome_bot,
        // full width — panels float over it) into the scene sub-rect and the graph
        // sub-rect. `None` keeps the legacy full-bleed scene (== canvas).
        let (center_viewport, motion_graph) = match split {
            CenterSplit::None => (canvas, Rect::new(viewport.x, chrome_top, 0.0, 0.0)),
            CenterSplit::Horizontal { t } => {
                let top_h = chrome_h * CenterSplit::clamp_t(t);
                (
                    Rect::new(viewport.x, chrome_top, viewport.w, top_h),
                    Rect::new(viewport.x, chrome_top + top_h, viewport.w, chrome_h - top_h),
                )
            }
            CenterSplit::Vertical { t } => {
                let left_w = viewport.w * CenterSplit::clamp_t(t);
                (
                    Rect::new(viewport.x, chrome_top, left_w, chrome_h),
                    Rect::new(
                        viewport.x + left_w,
                        chrome_top,
                        viewport.w - left_w,
                        chrome_h,
                    ),
                )
            }
        };
        // The timeline's seam: zero-height until the shell docks the timeline into it (W4.T4).
        let motion_timeline_slot = Rect::new(
            motion_graph.x,
            motion_graph.y + motion_graph.h,
            motion_graph.w,
            0.0,
        );
        let bottom_hud = Rect::new(
            viewport.x + (viewport.w - 480.0) * 0.5, // LITERAL-PX-OK: HUD strip width
            viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H,
            480.0, // LITERAL-PX-OK: HUD strip width
            HUD_H,
        );
        // General timeline dock: bottom strip of the center band, between the two
        // side chrome columns (so it never overlaps Inspector/Hierarchy), floating
        // over the scene's lower edge. Side columns swap under `mirrored`.
        let (left_col_right, right_col_left) = if mirrored {
            (inspector_x + insp_w, hierarchy_x)
        } else {
            (hierarchy_x + hier_w, inspector_x)
        };
        let timeline_x = left_col_right + EDGE_PAD;
        let timeline_w = (right_col_left - EDGE_PAD - timeline_x).max(0.0);
        // ⭐⭐ **A ÁREA de desenho** (D5) — as regiões são IRMÃS numa fila, e é por não
        // partilharem coordenada que nada aqui pode tapar nada. Horizontalmente ela começa
        // depois da coluna da esquerda (o trilho, mais o painel **se estiver aberto**) e acaba
        // antes da coluna da direita; verticalmente é a banda de chrome, que já exclui a barra
        // de topo e o HUD. ⚠️ Uma coluna fechada não é reservada: a área cresce para dentro
        // dela, senão a régua da esquerda ficaria a flutuar sobre o desenho.
        //
        // ⚠️ Uma coluna VAZIA não é reservada — a área cresce para dentro dela; uma OCUPADA é,
        // seja quem for que lá esteja. Quem responde é o `DockSides::from_published`.
        // ⚠️ **A área ENCOSTA na coluna** (Enio, 2026-08-30, com seta: *«a régua deve ficar
        // colada na hierarquia, e a nossa tem um espaço ruim»*). O `EDGE_PAD` que aqui estava era
        // o último dos quatro espaços mortos — e o pior, porque a régua nasce na borda da área e
        // o buraco ficava entre ela e o painel, onde salta à vista.
        let area_x0 = if docks.left {
            left_col_right
        } else {
            viewport.x + rail_w
        };
        let area_x1 = if docks.right {
            right_col_left
        } else {
            viewport.x + viewport.w
        };
        let area_w = (area_x1 - area_x0).max(0.0);
        // ⭐⭐ **A FILA DE FERRAMENTAS é uma REGIÃO da área, e por isso corta a ÁREA e não a
        // janela** (spec §4, D5). Ela e a régua são irmãs numa fila vertical: a fila fica em cima,
        // a régua começa por baixo dela, e nenhuma das duas pode tapar a outra porque não
        // partilham coordenada. ⛔ Uma barra de ferramentas que atravessasse o ecrã passaria por
        // cima das colunas — que é o modelo que o trilho `x = 0` tinha, e o defeito que ele deu.
        let tool_bar = Rect::new(area_x0, chrome_top, area_w, tool_bar_h.min(chrome_h));
        let draw_area = Rect::new(
            area_x0,
            chrome_top + tool_bar.h,
            area_w,
            (chrome_h - tool_bar.h).max(0.0),
        );
        let timeline = Rect::new(
            timeline_x,
            (chrome_bot - TIMELINE_DOCK_H).max(chrome_top),
            timeline_w,
            TIMELINE_DOCK_H.min(chrome_h),
        );
        // Flip frame strip (ADR-0114 W3): a BAIXA faixa inferior do Flip — a tira de
        // quadros da camada ativa + transporte. Compartilha a coluna do timeline
        // (mesma largura, entre as colunas de chrome), mas é MUITO mais baixa: é uma
        // tira, não um dope-sheet. Só é pintada com a tool Flip ativa; quando o
        // timeline global está aberto, o painel se empilha ACIMA dele (o offset é
        // decidido no `paint`, que é quem sabe da visibilidade).
        let flip_strip = Rect::new(
            timeline_x,
            (chrome_bot - FLIP_STRIP_H).max(chrome_top),
            timeline_w,
            FLIP_STRIP_H.min(chrome_h),
        );
        Self {
            viewport,
            top_bar,
            left_rail,
            inspector,
            bgremoval,
            padding,
            painter_sidebar,
            painter_layers,
            hierarchy,
            bottom_hud,
            canvas,
            draw_area,
            tool_bar,
            docks,
            center_viewport,
            motion_graph,
            motion_timeline_slot,
            timeline,
            flip_strip,
            // ⚠️ Sem ocupação nenhuma não há abas: quem as reserva é `reserve_slot_tabs`, que só
            // o hero chama — os construtores públicos devolvem a geometria SEM abas, de propósito.
            slot_tabs: [Rect::new(0.0, 0.0, 0.0, 0.0); 6],
        }
    }

    /// **The timeline moves INTO the Motion workspace** (W4.T4) — it stops floating over the node
    /// graph and becomes the band under it.
    ///
    /// Everything the timeline needs to be useful to Motion was already there: **one clock** (the
    /// `MotionTransport` died in W4.T7 — the graph cooks on the `Playhead`'s tick), a bridge that
    /// runs every frame whether the panel is visible or not, and a snapshot published every frame.
    /// The gap was **geometry**: `motion_graph` ran all the way down to the chrome, `timeline` was
    /// the same bottom strip it always is, and the two **overlapped completely** — the timeline
    /// painted *on top of* the graph, because it is drawn later.
    ///
    /// So this does not move a panel; it **carves a band**. The graph gives up its bottom edge and
    /// the timeline's rect becomes that band — which means the timeline panel needs **no change at
    /// all**: it already draws into `layout.timeline`, and `layout.timeline` is now somewhere else.
    /// One rect, decided in one place; a panel that had to ask *"which rect am I in today?"* would
    /// be a second place for the two to disagree.
    ///
    /// Idempotent, and inert without a split (`motion_graph` is zero-sized then — there is nothing
    /// to carve).
    pub fn dock_timeline_into_motion(&mut self) {
        if self.motion_graph.h <= 0.0 || self.motion_graph.w <= 0.0 {
            return; // no Motion split on screen: the timeline keeps its own dock
        }
        let h = MOTION_TIMELINE_H.min(self.motion_graph.h * MOTION_TIMELINE_MAX_FRAC);
        self.motion_graph.h -= h;
        self.motion_timeline_slot = Rect::new(
            self.motion_graph.x,
            self.motion_graph.y + self.motion_graph.h,
            self.motion_graph.w,
            h,
        );
        self.timeline = self.motion_timeline_slot;
    }

    /// **Uma faixa docada no FUNDO come a altura da área de desenho** — e sem isto a régua da
    /// esquerda corre por baixo dela.
    ///
    /// ⛔ **Achado da auditoria de 2026-08-30:** o `timeline` nasce em
    /// `timeline_x = left_col_right + EDGE_PAD`, que é **literalmente** o `area_x0`, e tem
    /// `TIMELINE_DOCK_H = 240` de altura no fundo da banda. Com os dois docks abertos ele
    /// partilhava **4 800 px² (20 × 240)** com a régua da esquerda — a mesma família de defeito
    /// que a wave existe para curar, num dock que o próprio ficheiro chama de *«General timeline
    /// dock»*. O `flip_strip` é o irmão dele.
    ///
    /// ⚠️ Idempotente e inerte para uma faixa vazia ou que já esteja abaixo da área. Chame-a uma
    /// vez por faixa **visível** — quem sabe da visibilidade é o `screens/hero/paint.rs`, como no
    /// [`Self::dock_timeline_into_motion`].
    pub fn reserve_bottom_strip(&mut self, strip: Rect) {
        if strip.w <= 0.0 || strip.h <= 0.0 {
            return;
        }
        let area_bottom = self.draw_area.y + self.draw_area.h;
        if strip.y >= area_bottom {
            return; // a faixa está fora da área — nada a reclamar
        }
        self.draw_area.h = (strip.y - self.draw_area.y).max(0.0);
    }

    /// **A região em que um popover flutuante pode nascer** — a BANDA DE CHROME, e nunca a janela
    /// inteira.
    ///
    /// Um popover longo não cabe em lado nenhum, e aí
    /// [`crate::widget::Dropdown::popover_rect_clamped`] toma *o lado com mais espaço, clampado à
    /// região que lhe deram*. Dada a JANELA, "mais espaço" é quase sempre para CIMA, e o clamp
    /// encosta a lista na borda de cima da janela — medido no frame real com o picker de token
    /// (81 linhas, `PH2D_BUILD_SMOKE=50`): a primeira linha nascia em **`y ∈ [2, 30]`**, ou seja
    /// **2 px da borda da janela**, 64 px ACIMA do painel a que pertence, por cima da barra de topo
    /// (`(14, 14, 1338, 64)`) — e `WidgetStore::panel_at` respondia `None` ali, porque ali não há
    /// painel nenhum. O `HitIndex` resolvia o clique (o popover regista por último), mas quem
    /// pergunta *"isto está sobre um painel?"* — e o gestor de janelas, que é dono da faixa de
    /// borda — não tem por que concordar. A cura não é disputar aquela faixa: é não nascer lá.
    ///
    /// A banda devolvida é **exatamente** a que o rail e os painéis docados ocupam (o `left_rail`
    /// abrange-a por construção), então um popover aterra onde o painel dele vive. Nenhuma
    /// constante nova: mover a banda de chrome move o popover junto.
    ///
    /// Largura = a da viewport, de propósito — um popover pode transbordar para o canvas na
    /// horizontal (é o que a lista de 182 px faz por cima da cena), e é só na VERTICAL que ele
    /// saía do sítio.
    ///
    /// ⛔⛔ **A âncora MUDOU em 2026-08-30, e a antiga era uma armadilha nomeada.** Ela era o
    /// `left_rail` — *«ele abrange a banda por construção»* —, e isso deixou de ser verdade no
    /// instante em que o trilho passou a poder estar **fora**: com `rail_w = 0` o rect dele é
    /// `(0, 0, 0, h)` e a região virava **a janela inteira**, que é exactamente o defeito que ela
    /// existe para curar (a 1.ª linha do picker a nascer a 2 px da borda).
    ///
    /// ⭐ Hoje ela sai da **coluna do dock**, que é a banda por construção e não por acaso: o
    /// popover nasce onde o painel dele vive, e o painel vive ali. *Uma região derivada de um
    /// chrome que pode desaparecer não é uma região — é uma coincidência.*
    #[must_use]
    pub fn popover_region(&self) -> Rect {
        Rect::new(
            self.viewport.x,
            self.inspector.y,
            self.viewport.w,
            self.inspector.h,
        )
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod split_tests;
