//! ⭐ **A barra do transporte em cartões, no molde do Rive** (`docs/Timeline/15_plano_redesenho_molde_rive.md`,
//! onda 1b): duas fileiras — título (abas · migalha · tocar · Time · Frame · fps · dono · opções) e
//! gravar + clipes (Auto Key · Record · Marker · um chip por clipe). Os ids e os eventos são os de sempre; o que não é de todo o dia mora no
//! «⋯» do clipe e na engrenagem (`crate::barra_menus`). `PH2D_CARDS=0` pinta a barra de [`super::paint_bar`].

use super::*;
use ph2d_editor_core::panel::cards::banda::{self, Emoldurada, Icone, Sinal, metrics as m};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::CardInk;

/// A largura do chip do dono.
const ANIM_W: f32 = 168.0; // LITERAL-PX-OK: «Animation: Scene» + seta, medido na foto; nomes longos cortam

const DICAS: [(ph2d_a11y::NodeId, &str); 18] = [
    (TIMELINE_GO_START, "panel.timeline.tip.go_start"),
    (TIMELINE_PREV_FRAME, "panel.timeline.tip.prev_frame"),
    (TIMELINE_PLAY, "panel.timeline.tip.play"),
    (TIMELINE_NEXT_FRAME, "panel.timeline.tip.next_frame"),
    (TIMELINE_GO_END, "panel.timeline.tip.go_end"),
    (ids::TIMELINE_ADD_CLIP, "panel.timeline.tip.add_clip"),
    (ids::TIMELINE_DUP_CLIP, "panel.timeline.tip.dup_clip"),
    (ids::TIMELINE_RENAME_CLIP, "panel.timeline.tip.rename_clip"),
    (ids::TIMELINE_DELETE_CLIP, "panel.timeline.tip.delete_clip"),
    (ids::TIMELINE_REVERSE_CLIP, "panel.timeline.tip.reverse"),
    (TIMELINE_ADD_MARKER, "panel.timeline.tip.add_marker"),
    (ids::TIMELINE_STRETCH_CLEAR, "panel.timeline.tip.stretch"),
    (
        ids::TIMELINE_ONION_SETTINGS,
        "panel.timeline.tip.onion_settings",
    ),
    (ids::TIMELINE_ONION_MODE, "panel.timeline.tip.onion_keys"),
    (ids::TIMELINE_OPTIONS, "panel.timeline.tip.options"),
    (ids::TIMELINE_VIEW_CURVES, "panel.timeline.view.tip"),
    (ids::TIMELINE_CLIP_MENU, "panel.timeline.tip.clip_menu"),
    (
        ids::TIMELINE_CLIP_AUTOPLAY,
        "panel.timeline.tip.starts_with",
    ),
];

/// O valor da caixa `id`, pela mesma tabela que o store espelha.
pub(crate) fn valor_da_caixa(
    snap: &TimelineViewSnapshot,
    view: BarView,
    id: ph2d_a11y::NodeId,
) -> bool {
    caixas(snap, view).iter().any(|(_, i, on)| *i == id && *on)
}

/// O estado de cada caixa: id, nome (`Item`) e valor. Um valor de TAB não mora no documento.
fn caixas(snap: &TimelineViewSnapshot, view: BarView) -> [(Item, ph2d_a11y::NodeId, bool); 10] {
    let laco = snap.loop_range.is_some();
    [
        (Item::Loop, TIMELINE_LOOP, laco && !snap.loop_ping_pong),
        (
            Item::PingPong,
            TIMELINE_PINGPONG,
            laco && snap.loop_ping_pong,
        ),
        (Item::Physics, TIMELINE_PHYSICS, snap.simulate_physics),
        (Item::AutoKey, TIMELINE_AUTOKEY, snap.auto_key),
        (Item::Record, TIMELINE_RECORD, snap.performing),
        (
            Item::MotionPath,
            TIMELINE_MOTION_PATH,
            snap.position_is_path,
        ),
        (Item::Snap, TIMELINE_SNAP, snap.frame_snap),
        (Item::Speed, ids::TIMELINE_SPEED, view.speed_view),
        (Item::Onion, ids::TIMELINE_ONION, snap.onion.enabled),
        (
            Item::OnionMode,
            ids::TIMELINE_ONION_MODE,
            snap.onion.mode == ph2d_timeline::OnionMode::Keys,
        ),
    ]
}

/// O que o store precisa de saber antes de a fila se pintar com ele emprestado.
fn prepara(store: &mut WidgetStore, snap: &TimelineViewSnapshot, view: BarView) {
    let fps = if snap.fps > 0.0 {
        snap.fps
    } else {
        DEFAULT_FPS
    };
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, valor, passo, casas) in [
        (TIMELINE_TIME_NUM, snap.time_seconds, 1.0 / fps, 2),
        (TIMELINE_FRAME_NUM, snap.frame as f64, 1.0, 0),
        (
            ids::TIMELINE_LENGTH_NUM,
            snap.view_length_seconds,
            chips::DUR_STEP_SECONDS,
            2,
        ),
    ] {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step: passo,
                scrub: passo,
                default: Some(0.0),
            },
        );
        store.set_number_range(id, 0.0, f64::INFINITY, passo);
        store.set_number_drag_rate(id, passo);
        if arrastado != Some(id) {
            mirror_number(store, id, valor, casas);
        }
    }
    let mut espelha = |id, on| {
        if let Some(InteractiveState::Toggle { on: no_store, .. }) = store.get_mut(id) {
            *no_store = on;
        }
    };
    espelha(
        ids::TIMELINE_CLIP_AUTOPLAY,
        crate::transport_entry::ligado(snap),
    );
    for (_, id, on) in caixas(snap, view) {
        espelha(id, on);
    }
    for (id, chave) in DICAS {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, ph2d_i18n::tr(chave));
        }
    }
}

fn segs_das_abas<'a>(celulas: &'a [(ph2d_a11y::NodeId, String)], escolhida: usize) -> Vec<Seg<'a>> {
    celulas
        .iter()
        .enumerate()
        .map(|(i, (id, nome))| Seg {
            id: *id,
            icon: None,
            text: nome,
            on: i == escolhida,
        })
        .collect()
}

fn fileira_do_transporte(snap: &TimelineViewSnapshot, view: BarView) -> [Icone; 5] {
    let vivo = |id, icon| Icone {
        id: Some(id),
        icon,
        on: false,
    };
    let play = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    // A lista de Containers não tem playback: o Play apaga-se e não regista alvo nenhum.
    let sem_tempo = crate::tab::rows(view.tab, snap) == crate::tab::Rows::Containers;
    [
        vivo(TIMELINE_GO_START, IconId::SkipBack),
        vivo(TIMELINE_PREV_FRAME, IconId::ChevronLeft),
        if sem_tempo {
            Icone {
                id: None,
                icon: play,
                on: false,
            }
        } else {
            vivo(TIMELINE_PLAY, play)
        },
        vivo(TIMELINE_NEXT_FRAME, IconId::ChevronRight),
        vivo(TIMELINE_GO_END, IconId::SkipForward),
    ]
}

/// O rótulo do Time e do Frame, cada um à largura dele (o Length mora nas opções).
fn rotulo_do_campo(item: Item) -> &'static str {
    match item {
        Item::TimeChip => ph2d_i18n::tr("panel.timeline.card.time"),
        _ => chip_rotulo(item),
    }
}

/// «24 fps · project»: só leitura — os quadros por segundo são do projeto.
fn texto_do_fps(snap: &TimelineViewSnapshot) -> String {
    let fps = if snap.fps > 0.0 {
        snap.fps
    } else {
        DEFAULT_FPS
    };
    let n = if (fps - fps.round()).abs() < 1e-6 {
        format!("{fps:.0}")
    } else {
        format!("{fps:.2}")
    };
    ph2d_i18n::tr_with("panel.timeline.card.fps", &[("fps", &n)])
}

/// A fila das fileiras: cada peça pede a largura dela e recebe onde pintar; o que não cabe desce.
/// A 1.ª fileira mora na tira do cabeçalho; as outras, a partir de `body`.
pub(crate) struct Fluxo {
    body: Rect,
    lim: (f32, f32),
    x: f32,
    y: f32,
    fileiras: usize,
}

impl Fluxo {
    fn novo(strip: Rect, body: Rect) -> Self {
        Self {
            body,
            lim: (strip.x, strip.w),
            x: strip.x,
            y: strip.y,
            fileiras: 0,
        }
    }

    /// Começa uma fileira nova no interior.
    pub(crate) fn quebra(&mut self) {
        self.fileiras += 1;
        self.lim = (self.body.x, self.body.w);
        let passo = m::FIELD_H + m::FIELD_ROW_GAP;
        self.y = self.body.y + (self.fileiras - 1) as f32 * passo - banda::SOBRA_DO_CABECALHO;
        self.x = self.lim.0;
    }

    /// Onde vai uma peça de largura `w`, na fileira de agora ou na seguinte.
    pub(crate) fn poe(&mut self, w: f32) -> (f32, f32) {
        if self.x > self.lim.0 && self.x + w > self.lim.0 + self.lim.1 {
            self.quebra();
        }
        let at = (self.x, self.y);
        self.x += w + m::FIELD_GAP;
        at
    }

    /// O que sobra da fileira de agora.
    pub(crate) fn resta(&self) -> f32 {
        self.lim.0 + self.lim.1 - self.x
    }

    /// Nada pintado ainda na fileira de agora.
    pub(crate) fn no_inicio(&self) -> bool {
        self.x <= self.lim.0
    }

    /// Um grupo de largura `w` que não cabe no resto da fileira mas cabe numa inteira desce inteiro.
    pub(crate) fn guarda_para(&mut self, w: f32) {
        if self.x > self.lim.0 && w > self.resta() && w <= self.body.w {
            self.quebra();
        }
    }

    /// Uma peça encostada à DIREITA da fileira de agora (ou da seguinte, se não cabe).
    fn a_direita(&mut self, w: f32) -> (f32, f32) {
        if self.x > self.lim.0 && self.x + w > self.lim.0 + self.lim.1 {
            self.quebra();
        }
        let fim = self.lim.0 + self.lim.1;
        let at = ((fim - w).max(self.x), self.y);
        self.x = fim + m::FIELD_GAP;
        at
    }

    fn depois(&self) -> f32 {
        if self.fileiras > 0 {
            self.y + m::FIELD_H + banda::VAO_ATE_A_FOLHA
        } else {
            self.body.y
        }
    }
}

/// O que a barra devolve: onde a folha começa, os chips de que pendem as listas e o campo de
/// renomear, e as âncoras das duas janelinhas.
pub(crate) struct Barra {
    pub(crate) depois: f32,
    pub(crate) clip: Option<ClipChip>,
    pub(crate) anim: Option<ClipChip>,
    pub(crate) opcoes: Option<Rect>,
    pub(crate) menu_do_clip: Option<Rect>,
}

fn cartao<'a>(ctx: &'a mut PaintCtx, ink: CardInk, body: Rect) -> CardCtx<'a> {
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w)
}

/// 1.ª fileira, à esquerda: abas · migalha (o tocar vem a seguir, [`grupo_de_tocar`]).
fn abas_e_migalha(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    (snap, view): (&TimelineViewSnapshot, BarView),
    body: Rect,
) {
    let ink = CardInk::of_list(theme);
    let celulas = crate::transport_tabs::items(snap);
    {
        let mut c = cartao(ctx, ink, body);
        let segs = segs_das_abas(&celulas, crate::transport_tabs::selected(view.tab, snap));
        let at = f.poe(c.segmentos_em_fila_w(&segs));
        c.abas_em(&segs, at.0, at.1, tons_da_barra(theme));
    }
    let w = crate::breadcrumb::width(snap);
    if w > 0.0 {
        let (x, y) = f.poe(w);
        crate::breadcrumb::paint_selo(ctx, theme, Rect::new(x, y, w, m::FIELD_H), snap);
    }
}

/// 1.ª fileira, à direita: o dono e a engrenagem (o «×» é da moldura).
fn dono_e_opcoes(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    (snap, view): (&TimelineViewSnapshot, BarView),
    body: Rect,
) -> (ClipChip, Rect) {
    let ink = CardInk::of_list(theme);
    let mut c = cartao(ctx, ink, body);
    // «Keys | Curves» (doc 17, item 5): só na aba Keys, antes do dono (maquete B1).
    let segs = crate::curvas_vista::segs(view.curvas);
    let vw = if view.tab == crate::tab::Tab::Keys {
        c.segmentos_em_fila_w(&segs) + m::FIELD_GAP
    } else {
        0.0
    };
    let (x0, y) = f.a_direita(vw + ANIM_W + m::FIELD_GAP + banda::icones_w(1));
    if vw > 0.0 {
        c.segmentos_em_fila(&segs, x0, y);
    }
    let x = x0 + vw;
    let rect = Rect::new(x, y, ANIM_W, m::FIELD_H);
    let open = matches!(
        c.store.get(ids::TIMELINE_ANIM_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    c.lista_em(
        ids::TIMELINE_ANIM_DD,
        &crate::transport_animation::chip_text(snap),
        open,
        rect,
    );
    let gx = x + ANIM_W + m::FIELD_GAP;
    c.icones_em_fila(
        &[Icone {
            id: Some(ids::TIMELINE_OPTIONS),
            icon: IconId::Settings,
            on: false,
        }],
        gx,
        y,
    );
    let chip = ClipChip {
        id: ids::TIMELINE_ANIM_DD,
        rect,
        open,
    };
    (chip, Rect::new(gx, y, banda::icones_w(1), m::FIELD_H))
}

fn campo(c: &mut CardCtx<'_>, item: Item, id: ph2d_a11y::NodeId, at: (f32, f32)) {
    let nome = rotulo_do_campo(item);
    let col = c.coluna_de_rotulos(&[nome]);
    c.campo_em_fila(id, nome, Glyph::Text(letra_da_grandeza(nome)), col, at);
}

/// 1.ª fileira, depois das abas: tocar · Time · Frame · fps.
fn grupo_de_tocar(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    (snap, view): (&TimelineViewSnapshot, BarView),
    body: Rect,
) {
    let ink = CardInk::of_list(theme);
    let mut c = cartao(ctx, ink, body);
    // O separador das abas (onda 1c) só entra se o tocar vem logo a seguir na MESMA fileira.
    if f.resta() >= banda::SEPARADOR_W + m::FIELD_GAP + banda::icones_w(TRANSPORT_BTNS) {
        let (x, y) = f.poe(banda::SEPARADOR_W);
        c.separador_em(x, y);
    }
    let at = f.poe(banda::icones_w(TRANSPORT_BTNS));
    c.icones_em_fila(&fileira_do_transporte(snap, view), at.0, at.1);
    // Time e Frame são um grupo: não se partem entre fileiras.
    let wt = CardCtx::campo_em_fila_w(c.coluna_de_rotulos(&[rotulo_do_campo(Item::TimeChip)]));
    let wf = CardCtx::campo_em_fila_w(c.coluna_de_rotulos(&[rotulo_do_campo(Item::FrameChip)]));
    let (x, y) = f.poe(wt + m::FIELD_GAP + wf);
    campo(&mut c, Item::TimeChip, TIMELINE_TIME_NUM, (x, y));
    campo(
        &mut c,
        Item::FrameChip,
        TIMELINE_FRAME_NUM,
        (x + wt + m::FIELD_GAP, y),
    );
    let fps = texto_do_fps(snap);
    let (x, y) = f.poe(c.rotulo_com_sinal_w(&fps));
    c.rotulo_com_sinal(Sinal::Icone(IconId::Fps, ColorToken::CardLabel), &fps, x, y);
}

/// As duas caixas de gravar: Auto Key (o ponto acende-se vermelho quando liga) e Record while playing
/// (o anel vermelho de gravação). A cor mora no sinal, nunca no quadradinho.
fn caixas_de_gravar(snap: &TimelineViewSnapshot) -> [Emoldurada<'static>; 2] {
    // Apagados desligados; o vermelho só quando LIGA (doc 15 onda 2i: as abas mandam na barra).
    let acende = |on: bool| {
        if on {
            ColorToken::Danger
        } else {
            ColorToken::TextDisabled
        }
    };
    let ponto = acende(snap.auto_key);
    [
        Emoldurada {
            id: TIMELINE_AUTOKEY,
            nome: ph2d_i18n::tr("panel.timeline.card.autokey"),
            on: snap.auto_key,
            antes: None,
            fim: Some(Sinal::Ponto(ponto)),
        },
        Emoldurada {
            id: TIMELINE_RECORD,
            nome: ph2d_i18n::tr("panel.timeline.card.record"),
            on: snap.performing,
            antes: Some(Sinal::Icone(IconId::Circle, acende(snap.performing))),
            fim: None,
        },
    ]
}

/// 2.ª fileira, no início: Auto Key · Record while playing · Marker (os clipes vêm a seguir).
fn grupo_de_gravar(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    snap: &TimelineViewSnapshot,
    body: Rect,
) {
    let ink = CardInk::of_list(theme);
    let mut c = cartao(ctx, ink, body);
    let tons = tons_da_barra(theme);
    for caixa in caixas_de_gravar(snap) {
        let (x, y) = f.poe(c.emoldurada_w(&caixa));
        c.emoldurada_discreta_em(&caixa, x, y, tons);
    }
    let nome = ph2d_i18n::tr("panel.timeline.card.add_marker");
    let w = c.botao_com_icone_w(nome);
    let (x, y) = f.poe(w);
    c.botao_com_icone_discreto_em(
        TIMELINE_ADD_MARKER,
        IconId::Plus,
        nome,
        Rect::new(x, y, w, m::FIELD_H),
        tons,
    );
    // O trecho a tocar (doc 17 item 2): «takeoff → top» com ✕; clicar volta ao clipe todo.
    if let Some(nome) = crate::ruler_trecho::rotulo(snap) {
        let w = crate::faixa_marcadores::pilula_w(ctx, &nome);
        let (x, y) = f.poe(w);
        crate::faixa_marcadores::pilula(ctx, theme, Rect::new(x, y, w, m::FIELD_H), &nome);
    }
}

/// ⭐ **A hierarquia da barra** (dono 09/11, onda 2i): as abas são a navegação principal e as mais
/// fortes (pílula no azul do chip da maquete, texto claro em negrito); Auto Key, Record e Marker um
/// degrau abaixo, sem caixa. As tintas são as da maquete B1 (`TimelineTinta`).
pub(crate) fn tons_da_barra(theme: Theme) -> banda::TonsDaBarra {
    use ph2d_tokens::TimelineTinta as T;
    let t = |x| crate::folha_tinta::tinta(x, theme);
    banda::TonsDaBarra {
        ativa_fundo: t(T::ChipEscolhido),
        ativa_texto: t(T::TextoCabecalho),
        inativa_texto: t(T::TextoPropriedade),
        sob_o_rato: t(T::NomesCabecalho),
        discreta_texto: t(T::TextoValor),
    }
}

/// A barra (doc 15, onda 1b, dono 09/11): DUAS fileiras. Na tira do cabeçalho `strip`: abas · migalha ·
/// tocar · Time · Frame · fps … dono · engrenagem; a partir de `body`: gravar · marcador · clipes.
pub(crate) fn paint_bar(
    ctx: &mut PaintCtx,
    theme: Theme,
    strip: Rect,
    body: Rect,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> Barra {
    prepara(ctx.host.store_mut(), snap, view);
    let mut f = Fluxo::novo(strip, body);
    abas_e_migalha(ctx, theme, &mut f, (snap, view), body);
    grupo_de_tocar(ctx, theme, &mut f, (snap, view), body);
    let (anim, opcoes) = dono_e_opcoes(ctx, theme, &mut f, (snap, view), body);
    f.quebra();
    grupo_de_gravar(ctx, theme, &mut f, snap, body);
    let clips = crate::barra_clips::paint(ctx, theme, &mut f, (snap, view), body);
    Barra {
        depois: f.depois(),
        clip: clips.chip,
        anim: Some(anim),
        opcoes: Some(opcoes),
        menu_do_clip: clips.menu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O fps lê-se inteiro quando é inteiro. *Mutação: sempre duas casas ⇒ «24.00 fps».*
    #[test]
    fn o_fps_le_se_inteiro_quando_e_inteiro() {
        let snap = TimelineViewSnapshot {
            fps: 24.0,
            ..TimelineViewSnapshot::default()
        };
        assert!(
            texto_do_fps(&snap).starts_with("24 fps"),
            "{}",
            texto_do_fps(&snap)
        );
        let snap = TimelineViewSnapshot {
            fps: 29.97,
            ..TimelineViewSnapshot::default()
        };
        assert!(texto_do_fps(&snap).starts_with("29.97 fps"));
    }
}
