//! ⭐ **A barra do transporte em cartões, no molde do Rive** (`docs/Timeline/15_plano_redesenho_molde_rive.md`,
//! onda 1b): duas fileiras — título (abas · migalha | Length | Time · Frame · fps · dono · opções) e
//! clipes (Animating·Done · Marker · um chip por clipe); o tocar e o gravar FLUTUAM acima do painel
//! ([`flutuante`], dono 13/11). Os ids e os eventos são os de sempre; o que não é de todo o dia mora no
//! «⋯» do clipe e na engrenagem (`crate::barra_menus`). `PH2D_CARDS=0` pinta a barra de [`super::paint_bar`].

use super::*;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::banda::{self, Emoldurada, Icone, Sinal, metrics as m};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::CardInk;

/// A largura do chip do dono.
const ANIM_W: f32 = 168.0; // LITERAL-PX-OK: «Animation: Scene» + seta, medido na foto; nomes longos cortam

const DICAS: [(ph2d_a11y::NodeId, &str); 24] = [
    (ids::TIMELINE_STOP, "panel.timeline.tip.stop"),
    (TIMELINE_AUTOKEY, "panel.timeline.tip.autokey"),
    (ids::TIMELINE_LENGTH_NUM, "panel.timeline.tip.length"),
    (TIMELINE_TIME_NUM, "panel.timeline.tip.time"),
    (TIMELINE_FRAME_NUM, "panel.timeline.tip.frame"),
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
    (
        ids::TIMELINE_CURVES_ON_CANVAS,
        "panel.timeline.curves_on_canvas.tip",
    ),
    (ids::TIMELINE_CLIP_MENU, "panel.timeline.tip.clip_menu"),
    (
        ids::TIMELINE_ANIMATING_DONE,
        "panel.timeline.animating.done.tip",
    ),
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
    if id == ids::TIMELINE_CLIPS_AS_LIST {
        return view.clips_as_list;
    }
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
    espelha(ids::TIMELINE_CURVES_ON_CANVAS, view.curves_on_canvas);
    espelha(ids::TIMELINE_CLIPS_AS_LIST, view.clips_as_list);
    for (id, chave) in DICAS {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, ph2d_i18n::tr(chave));
        }
    }
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

/// 1.ª fileira, à esquerda: a migalha (as abas são de pasta na borda esquerda, dono 13/11,
/// [`crate::abas_laterais`]).
fn migalha(ctx: &mut PaintCtx, theme: Theme, f: &mut Fluxo, snap: &TimelineViewSnapshot) {
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
    // «Edit curves on canvas» (dono 10/11): só na aba Keys, antes do dono — no lugar do antigo «Keys | Curves».
    let caixa = Emoldurada {
        id: ids::TIMELINE_CURVES_ON_CANVAS,
        nome: ph2d_i18n::tr("panel.timeline.curves_on_canvas"),
        on: view.curves_on_canvas,
        antes: None,
        fim: None,
    };
    let mut vw = if view.tab == crate::tab::Tab::Keys {
        c.emoldurada_w(&caixa) + m::FIELD_GAP
    } else {
        0.0
    };
    let dono_w = ANIM_W + m::FIELD_GAP + banda::icones_w(1);
    // Numa fileira que não leva a caixa e o dono juntos, a caixa fica na fileira dela.
    if vw > 0.0 && vw + dono_w > f.body.w {
        let (cx, cy) = f.a_direita(vw - m::FIELD_GAP);
        c.emoldurada_discreta_em(&caixa, cx, cy, tons_da_barra(theme));
        vw = 0.0;
    }
    let (x0, y) = f.a_direita(vw + dono_w);
    if vw > 0.0 {
        c.emoldurada_discreta_em(&caixa, x0, y, tons_da_barra(theme));
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

/// Time e Frame (dono 12/11): só a caixa, 20 % mais estreita, com o ícone do que mede (cronómetro ·
/// quadro de filme) no lugar da letra; o nome mora na dica.
fn campo(c: &mut CardCtx<'_>, item: Item, id: ph2d_a11y::NodeId, at: (f32, f32)) {
    let icone = if item == Item::TimeChip {
        IconId::Timer
    } else {
        IconId::FilmFrame
    };
    c.campo_compacto(id, rotulo_do_campo(item), Glyph::Icon(icone), at);
}

/// 1.ª fileira, depois das abas: | Length | Time · Frame · fps (dono 13/11: a duração do clipe entre as
/// abas e o Time, com um separador de cada lado; o tocar flutua acima do painel).
fn grupo_do_tempo(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    snap: &TimelineViewSnapshot,
    body: Rect,
) {
    let ink = CardInk::of_list(theme);
    let w = banda::SEPARADOR_W
        + m::FIELD_GAP
        + banda::CAMPO_COMPACTO_W
        + m::FIELD_GAP
        + banda::SEPARADOR_W;
    f.guarda_para(w);
    let mut c = cartao(ctx, ink, body);
    // Sem nada antes (as abas foram para a borda), a Length encosta ao título sem separador.
    if !f.no_inicio() {
        let (x, y) = f.poe(banda::SEPARADOR_W);
        c.separador_em(x, y);
    }
    let at = f.poe(banda::CAMPO_COMPACTO_W);
    let nome = ph2d_i18n::tr("panel.timeline.card.length_long");
    let glifo = Glyph::Icon(IconId::CardLen);
    match crate::barra_menus::valor_lido_da_dur(snap) {
        Some(lido) => ph2d_editor_core::widget::card_field::com_valores(
            &[(ids::TIMELINE_LENGTH_NUM, lido)],
            || c.campo_compacto(ids::TIMELINE_LENGTH_NUM, nome, glifo, at),
        ),
        None => c.campo_compacto(ids::TIMELINE_LENGTH_NUM, nome, glifo, at),
    }
    let (x, y) = f.poe(banda::SEPARADOR_W);
    c.separador_em(x, y);
    // Time e Frame são um grupo: não se partem entre fileiras.
    let (wt, wf) = (banda::CAMPO_COMPACTO_W, banda::CAMPO_COMPACTO_W);
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

/// 2.ª fileira, no início: [Animating · Done] · Marker (os clipes vêm a seguir).
fn grupo_de_gravar(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    snap: &TimelineViewSnapshot,
    body: Rect,
) {
    // «● Animating jump [Done]» abre a 2.ª fileira (doc 17 item 9, B7): na do título empurrava o
    // Animation para baixo.
    if let Some(texto) = crate::animando::rotulo(snap) {
        let w = crate::animando::pilula_w(ctx, &texto);
        let (x, y) = f.poe(w);
        crate::animando::pilula(ctx, theme, Rect::new(x, y, w, m::FIELD_H), &texto);
    }
    let ink = CardInk::of_list(theme);
    let mut c = cartao(ctx, ink, body);
    let tons = tons_da_barra(theme);
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
        // Dono 12/11 (2.ª vez): Auto Key e Marker «sem contraste» — o texto sobe ao tom das abas
        // não escolhidas (6,6:1 contra a barra; `TextoValor` media 3,7) e o contorno da caixa vazia ao
        // dos títulos.
        discreta_texto: t(T::TextoPropriedade),
        borda_da_caixa: t(T::TextoCabecalho),
        // O corpo dos outros campos da barra (o chip do clip, Time, Frame) e, sob o rato, o degrau acima.
        corpo_da_peca: CardInk::of_list(theme).color(ColorToken::CardField),
        sob_o_rato: CardInk::of_list(theme).color(ColorToken::CardFieldHover),
    }
}

/// A barra (doc 15, onda 1b, dono 09/11): DUAS fileiras. Na tira do cabeçalho `strip`: abas · migalha ·
/// tocar · Time · Frame · fps … dono · engrenagem; a partir de `body`: gravar · marcador · clipes.
pub(crate) fn paint_bar(
    ctx: &mut PaintCtx,
    theme: Theme,
    strip: Rect,
    (body, painel): (Rect, Rect),
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> Barra {
    prepara(ctx.host.store_mut(), snap, view);
    flutuante::pinta(ctx, theme, painel, (snap, view));
    crate::abas_laterais::pinta(ctx, theme, painel, (snap, view.tab));
    let mut f = Fluxo::novo(strip, body);
    migalha(ctx, theme, &mut f, snap);
    grupo_do_tempo(ctx, theme, &mut f, snap, body);
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

#[path = "transport_flutuante.rs"]
pub(crate) mod flutuante;

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

    /// ⭐ Dono 12/11: a caixa de marcar vazia e o texto do Auto Key / Marker têm de se ver contra o
    /// fundo da barra (`KanbanList`) e o recheio da caixa (`CardField`): ≥ 4,5:1 em todo tema. Medido
    /// na foto antes da cura: texto `TextoValor` 3,7:1. *Mutação: `discreta_texto` = `TextoValor` ⇒
    /// vermelho.*
    #[test]
    fn a_caixa_vazia_da_barra_tem_contraste_com_o_fundo() {
        let rgb = |c: ph2d_vector::Color| {
            let [r, g, b, _] = c.to_rgba8().to_u8_array();
            ph2d_tokens::Color::from_hex(u32::from_be_bytes([0, r, g, b]))
        };
        for theme in Theme::ALL {
            let t = tons_da_barra(theme);
            // E o corpo das peças é o dos outros campos da barra (dono 12/11: «soltas no fundo»).
            let campo = CardInk::of_list(theme).color(ColorToken::CardField);
            assert_eq!(rgb(t.corpo_da_peca), rgb(campo), "{theme:?}: corpo da peça");
            for (nome, cor) in [("contorno", t.borda_da_caixa), ("texto", t.discreta_texto)] {
                for fundo in [ColorToken::CardField, ColorToken::KanbanList] {
                    let r = rgb(cor).contrast_ratio(&fundo.resolve(theme));
                    assert!(r >= 4.5, "{theme:?}: {nome} {r:.2}:1 contra {fundo:?}");
                }
            }
        }
    }
}
