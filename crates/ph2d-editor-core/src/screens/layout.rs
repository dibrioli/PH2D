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

/// A lei do divisor do centro — [`crate::screens::center_split`].
///
/// ⚠️ Re-exportada aqui porque todo chamador do [`HeroLayout`] a importa junto com ele: o tipo
/// mudou de ficheiro, não de dono.
pub use crate::screens::center_split::CenterSplit;

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
    /// ⭐⭐⭐ **O CABEÇALHO DA ÁREA** (D2, metade 2) — a faixa do editor, a primeira região da área
    /// de cima para baixo. Zero-altura enquanto ninguém a pede.
    ///
    /// ⚠️ Ela é irmã da fila e da régua, não da barra de menus: sai da **área**, entre as colunas.
    /// A ordem do modelo (`spec/01 §4`) é **cabeçalho → ferramentas → régua → conteúdo**.
    pub area_header: Rect,
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
    /// ⭐⭐⭐ **A BANDA que o divisor parte** — e a única régua contra a qual a fracção `t` se
    /// mede.
    ///
    /// ⛔⛔ **Sem ela, o arrasto do divisor tinha OFFSET e TREMOR** (Enio, 2026-08-31: *«segurar e
    /// arrastar o topo do canvas de nós tem um bug, um offset e um tremor»*). O painel do grafo
    /// reconstruía a banda somando `center_viewport + motion_graph` — o que era verdade até a
    /// **timeline docar dentro do split** (W4.T4) e passar a comer o fundo do `motion_graph`:
    ///
    /// | quem | denominador de `t` |
    /// |---|---|
    /// | o painel, ao arrastar | `chrome_h − altura_da_timeline` |
    /// | o layout, ao aplicar (`top_h = band.h · t`) | `chrome_h` |
    ///
    /// ⇒ **offset** de `chrome_h / (chrome_h − 240) ≈ 1,32` no alvo de referência (arrastar para o
    /// meio punha o divisor a 66 %), e **tremor** porque a altura da timeline é ela própria
    /// clampada pela altura do grafo: mover o divisor mudava o denominador, que movia o divisor.
    /// *Uma fracção medida contra uma banda RECONSTRUÍDA oscila assim que uma das parcelas
    /// depender do resultado.*
    ///
    /// ⚠️ Ela existe mesmo sem split (é a coluna da área) — quem não divide nada simplesmente não
    /// a lê.
    pub split_band: Rect,
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
}

/// Default docked height of the general timeline panel (px).
pub const TIMELINE_DOCK_H: f32 = 240.0; // LITERAL-PX-OK: timeline dock default height

/// ⛔ **`MOTION_TIMELINE_H` MORREU em 2026-08-31, e a morte é a cura.** Ela dizia que o timeline é
/// *«mais baixo dentro do Motion, porque o grafo é aquilo a que o artista veio»* — e era uma
/// SEGUNDA altura ao lado da faixa docada. No dia em que a do fundo passou a ser autorada (o topo
/// dela é uma costura), a segunda tornava o arrasto do artista invisível dentro do Motion:
/// *arrastava-se a borda e a banda não mexia, porque quem mandava ali era uma constante.*
///
/// ⇒ hoje [`HeroLayout::dock_timeline_into_motion`] lê `self.timeline.h`, que **já é** a altura
/// autorada. *A faixa tem UMA altura; docá-la dentro do split não pode inventar outra.*
///
/// ⭐⭐ **O tecto pertence ao GRAFO, e é o mesmo piso de uma faixa docada** — nunca uma fracção.
///
/// ⛔⛔ **A fracção (`0,45` do grafo) matava METADE do gesto que o Enio pediu.** Ela dava, no alvo
/// de referência, um tecto de `450 × 0,45 ≈ 202` px — **abaixo dos `240` de omissão da faixa**. ⇒
/// a costura nascia **já no tecto**: arrastar para BAIXO funcionava e arrastar para CIMA era
/// inerte, e nada na tela dizia porquê. *Um limite que corta o valor de fábrica não é um limite:
/// é metade do controlo desligada de origem.*
///
/// ⚠️ Hoje o que se defende é o **hospedeiro**: o grafo nunca fica com menos do que
/// [`crate::interaction::WidgetStore::dock_bottom_h`] aceita para uma faixa (o mesmo `120` px
/// abaixo do qual um dock deixa de ser usável). No alvo de referência isso dá `330` de tecto e
/// `120` de piso — **os dois lados do arrasto vivos**, com a de fábrica a `240` no meio.
const MOTION_GRAPH_MIN_H: f32 = 120.0; // LITERAL-PX-OK: = o piso de uma faixa docada (`DOCK_H_MIN`)

// ⛔⛔ **`flip_strip` e `FLIP_STRIP_H` MORRERAM em 2026-08-31.** A tira do Flip declara o encaixe
// `Bottom` e pinta em `ctx.slot` — a MESMA banda do timeline —, então este rect de 132 px era uma
// segunda geometria que ninguém pintava e que só a **reserva** da área de desenho ainda lia. ⇒ a
// área ficava reservada até `y = 892` e a tira pintava desde `784`: **147 528 px² de painel por
// cima do desenho**, invisíveis a todos os gates porque a `ph2d-panel-registry-init` não liga a
// feature `flip_frames` nas features de omissão dela. *Um rect que ninguém pinta não é uma reserva:
// é a resposta errada à pergunta «onde é que este painel está».*

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
            area_header_h,
            bottom_dock_h,
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
        // flutuar. ⚠️ **Ele NÃO foi zerado global** — continua a separar a barra de topo da borda.
        // ⚠️ Em 2026-08-31 a faixa do fundo (timeline / tira do Flip) também o perdeu, e a razão é
        // a mesma: *duas aritméticas para a mesma borda divergem no dia em que só uma é corrigida*
        // (a que aqui ficou tinha 20 px de buraco entre a coluna e a faixa, e o doc do
        // `reserve_bottom_strip` já dizia que os dois coincidiam).
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
        // Onde a coluna da esquerda acaba e a da direita começa. Sob `mirrored` os dois painéis
        // trocam de lado, então a resposta segue o LADO e não o nome (a mesma razão do
        // `side_columns`). ⚠️ Está aqui em cima, antes do split e das faixas do fundo, porque as
        // TRÊS coisas que dividem a janela na horizontal — o split do centro, o timeline e a
        // tira do Flip — vivem todas entre estas duas colunas.
        let (left_col_right, right_col_left) = if mirrored {
            (inspector_x + insp_w, hierarchy_x)
        } else {
            (hierarchy_x + hier_w, inspector_x)
        };
        // ⚠️ Uma coluna VAZIA não é reservada — a área cresce para dentro dela; uma OCUPADA é,
        // seja quem for que lá esteja. Quem responde é o `DockSides::from_published`.
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
        // ⭐⭐ **O SPLIT DO CENTRO PARTE A ÁREA, NÃO A JANELA** (Enio, 2026-08-31, com foto e duas
        // setas: *«o canvas dos Nodes deve ficar bem encaixado entre os painéis laterais como na
        // Godot, sem espaços»*).
        //
        // ⛔ Ele partia `viewport.x .. viewport.x + viewport.w`, e por isso a banda do grafo
        // atravessava o ecrã inteiro: ela nascia **por baixo** das duas colunas e — como o painel
        // do grafo é pintado depois delas — comia o terço de baixo da Hierarquia e do dock da
        // direita. No alvo de referência isso são **`2 × 300 × 430 px²`** de painel tapado por uma
        // região que não é irmã de ninguém.
        //
        // ⇒ D5, a mesma lei da [`Self::draw_area`]: *regiões são IRMÃS numa fila, nunca camadas
        // empilhadas.* A banda do split é a coluna da área, e as colunas laterais deixam de a
        // partilhar por construção.
        //
        // ⚠️ **Só o x/w muda; o y/h fica.** A fracção `t` continua a ser a da banda de chrome na
        // vertical, que é o que o `CenterSplit::scene_viewport` (o renderizador da cena) lê — mexer
        // nela poria o `set_viewport` e o painel a discordar, que é o *drift* que aquele doc conta.
        // ⚠️ E as DUAS metades encolhem juntas: o painel do grafo deteta a orientação por
        // `rect.x > center.x` e mede o arrasto do divisor contra `center + rect`, então narrar uma
        // só faria um split horizontal ler-se como vertical.
        let band = Rect::new(area_x0, chrome_top, area_w, chrome_h);
        let (center_viewport, motion_graph) = match split {
            CenterSplit::None => (canvas, Rect::new(band.x, band.y, 0.0, 0.0)),
            CenterSplit::Horizontal { t } => {
                let top_h = band.h * CenterSplit::clamp_t(t);
                (
                    Rect::new(band.x, band.y, band.w, top_h),
                    Rect::new(band.x, band.y + top_h, band.w, band.h - top_h),
                )
            }
            CenterSplit::Vertical { t } => {
                let left_w = band.w * CenterSplit::clamp_t(t);
                (
                    Rect::new(band.x, band.y, left_w, band.h),
                    Rect::new(band.x + left_w, band.y, band.w - left_w, band.h),
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
        // ⭐⭐ **A FAIXA DO FUNDO ENCOSTA NAS COLUNAS** (Enio, 2026-08-31, com foto e duas setas:
        // *«a timeline deve ficar bem encaixada entre os painéis laterais, sem espaços»*).
        //
        // ⛔ Ela era `left_col_right + EDGE_PAD` e acabava `EDGE_PAD` antes da outra coluna — dois
        // buracos de 20 px, um de cada lado, entre o painel e a faixa. ⚠️ **É um resíduo com
        // data:** em 2026-08-30 o `EDGE_PAD` saiu do `area_x0` (as colunas ficaram *flush*) e
        // ficou aqui, e o doc do [`Self::reserve_bottom_strip`] escrito nesse dia já afirmava que
        // o timeline nascia *«literalmente no `area_x0`»* — o que era falso por 20 px. *Duas
        // aritméticas para a mesma borda divergem no dia em que só uma é corrigida.*
        //
        // ⚠️ Ela segue as COLUNAS e não a `draw_area`: uma coluna fechada deixa a área crescer
        // para dentro dela, e a faixa cresce junto porque `area_x0` já responde por isso.
        let timeline_x = area_x0;
        let timeline_w = area_w;
        // ⭐⭐ **A ÁREA de desenho** (D5) — as regiões são IRMÃS numa fila, e é por não
        // partilharem coordenada que nada aqui pode tapar nada. Horizontalmente ela começa
        // depois da coluna da esquerda (o trilho, mais o painel **se estiver aberto**) e acaba
        // antes da coluna da direita; verticalmente é a banda de chrome, que já exclui a barra
        // de topo e o HUD. ⚠️ Uma coluna fechada não é reservada: a área cresce para dentro
        // dela, senão a régua da esquerda ficaria a flutuar sobre o desenho.
        //
        // ⚠️ **A área ENCOSTA na coluna** (Enio, 2026-08-30, com seta: *«a régua deve ficar
        // colada na hierarquia, e a nossa tem um espaço ruim»*). O `EDGE_PAD` que aqui estava era
        // o último dos quatro espaços mortos — e o pior, porque a régua nasce na borda da área e
        // o buraco ficava entre ela e o painel, onde salta à vista. (O `area_x0`/`area_w` estão
        // calculados lá em cima, antes do split, que é o outro consumidor deles.)
        // ⭐⭐ **A FILA DE FERRAMENTAS é uma REGIÃO da área, e por isso corta a ÁREA e não a
        // janela** (spec §4, D5). Ela e a régua são irmãs numa fila vertical: a fila fica em cima,
        // a régua começa por baixo dela, e nenhuma das duas pode tapar a outra porque não
        // partilham coordenada. ⛔ Uma barra de ferramentas que atravessasse o ecrã passaria por
        // cima das colunas — que é o modelo que o trilho `x = 0` tinha, e o defeito que ele deu.
        // ⭐⭐ **A ORDEM da área é cabeçalho → ferramentas → régua → conteúdo** (`spec/01 §4`), e
        // cada uma SUBTRAI da seguinte. ⛔ Uma faixa que flutuasse sobre a de baixo reproduziria,
        // num modelo novo, o defeito das duas réguas que o modelo existe para curar.
        let area_header = Rect::new(area_x0, chrome_top, area_w, area_header_h.min(chrome_h));
        let tool_bar = Rect::new(
            area_x0,
            chrome_top + area_header.h,
            area_w,
            tool_bar_h.min((chrome_h - area_header.h).max(0.0)),
        );
        let draw_area = Rect::new(
            area_x0,
            chrome_top + area_header.h + tool_bar.h,
            area_w,
            (chrome_h - area_header.h - tool_bar.h).max(0.0),
        );
        // ⭐ A altura é a AUTORADA (a costura do topo escreve-a), apertada contra a banda de
        // chrome — numa janela baixa a faixa não pode ser mais alta do que o sítio onde vive.
        let band_h = bottom_dock_h.min(chrome_h);
        let timeline = Rect::new(
            timeline_x,
            (chrome_bot - band_h).max(chrome_top),
            timeline_w,
            band_h,
        );
        // Flip frame strip (ADR-0114 W3): a BAIXA faixa inferior do Flip — a tira de
        // quadros da camada ativa + transporte. Compartilha a coluna do timeline
        // (mesma largura, entre as colunas de chrome), mas é MUITO mais baixa: é uma
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
            area_header,
            tool_bar,
            docks,
            center_viewport,
            motion_graph,
            split_band: band,
            motion_timeline_slot,
            timeline,
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
        // ⭐ A altura AUTORADA da faixa (ver o `MOTION_TIMELINE_MAX_FRAC`) — nunca uma constante
        // própria, senão o arrasto do artista fica invisível aqui dentro.
        // ⭐ A altura AUTORADA da faixa, com o **grafo** a guardar o piso dele — ver
        // `MOTION_GRAPH_MIN_H`. ⛔ Nunca uma constante própria, senão o arrasto fica invisível.
        let h = self
            .timeline
            .h
            .min((self.motion_graph.h - MOTION_GRAPH_MIN_H).max(0.0));
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
    /// ⛔ **Achado da auditoria de 2026-08-30:** o `timeline` nasce no `area_x0` (⚠️ *hoje* — em
    /// 2026-08-30 ele nascia 20 px à direita dele, e este doc já afirmava a coincidência que só
    /// passou a ser verdade em 31/08) e tem
    /// `TIMELINE_DOCK_H = 240` de altura no fundo da banda. Com os dois docks abertos ele
    /// partilhava **4 800 px² (20 × 240)** com a régua da esquerda — a mesma família de defeito
    /// que a wave existe para curar, num dock que o próprio ficheiro chama de *«General timeline
    /// dock»*.
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
