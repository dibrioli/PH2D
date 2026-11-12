//! ⭐ **Quem ouve o sinal de um marcador** (doc 17 item 8, maquete B5), só nos cartões: o clique parado
//! na alça de um marcador COM sinal vai até ele (como sempre) e abre o cartão «Signal at frame 24 · land»,
//! o nome só de leitura e a lista «Heard by». A lista vem da shell por esta porta de dados (o idioma do
//! `set_signal_census` do Inspector): o painel diz o nome aberto ([`signal_asked`]), a shell calcula o
//! censo dos ouvintes e devolve-o ([`set_signal_listeners`]). Fecha com Esc, clique de fora, marcador
//! sumido ou sem sinal, outra aba, ou o painel escondido.

use std::cell::RefCell;

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_icon, paint_text_elided, paint_text_title_elided,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::text_elide::largura_do_titulo;
use ph2d_editor_core::zones::Rect;
use ph2d_signal_ends::{Quem, SignalListeners};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::tab::Tab;

/// O cartão aberto: o marcador (índice no armazenamento) e a aba em que abriu.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aberto {
    pub index: usize,
    pub tab: Tab,
    /// O tempo e a contagem de marcadores do último quadro: o índice só vale enquanto o
    /// armazenamento não mudar de tamanho (o marcador não tem id).
    t: f64,
    n: usize,
    /// Aberto neste quadro: o Down que o abriu (na alça) também «fechou pelo clique de fora».
    novo: bool,
}

thread_local! {
    static PEDIDO: RefCell<Option<String>> = const { RefCell::new(None) };
    static OUVINTES: RefCell<Option<(String, SignalListeners)>> = const { RefCell::new(None) };
}

/// ⭐ O nome do sinal do cartão aberto (a shell lê-o a cada quadro); `None` = nenhum cartão aberto.
#[must_use]
pub fn signal_asked() -> Option<String> {
    PEDIDO.with(|p| p.borrow().clone())
}

/// ⭐ A shell entrega quem ouve `name` (o que [`signal_asked`] pediu).
pub fn set_signal_listeners(name: &str, l: SignalListeners) {
    OUVINTES.with(|o| *o.borrow_mut() = Some((name.to_owned(), l)));
}

fn tem_sinal(snap: &TimelineViewSnapshot, index: usize) -> Option<&str> {
    snap.markers
        .get(index)
        .and_then(|m| m.2.as_deref())
        .filter(|s| !s.is_empty())
}

/// O clique parado na alça do marcador `index`: com sinal (e nos cartões) abre o cartão; sem, fecha.
pub(crate) fn abre(state: &mut TimelinePanelState, snap: &TimelineViewSnapshot, index: usize) {
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    state.sinal_aberto = (cartoes && tem_sinal(snap, index).is_some()).then_some(Aberto {
        index,
        tab: state.tab,
        t: snap.markers.get(index).map_or(0.0, |m| m.0),
        n: snap.markers.len(),
        novo: true,
    });
    if state.sinal_aberto.is_some() {
        state.opcoes_abertas = false;
        state.menu_do_clip = false;
    }
}

/// O Esc: fecha o cartão; devolve se havia um.
pub(crate) fn esc(state: &mut TimelinePanelState) -> bool {
    state.sinal_aberto.take().is_some()
}

/// O painel escondido larga o cartão e o pedido.
pub(crate) fn larga(state: &mut TimelinePanelState) {
    state.sinal_aberto = None;
    PEDIDO.with(|p| *p.borrow_mut() = None);
}

/// O clique de fora, o marcador sumido (ou sem sinal) e a troca de aba fecham o cartão.
pub(crate) fn fecha_se_preciso(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    let fora = ctx
        .host
        .store_mut()
        .take_own_list_dismissed(ids::TIMELINE_SIGNAL_CARD);
    let tab = state.tab;
    let Some(a) = state.sinal_aberto.as_mut() else {
        return;
    };
    let novo = std::mem::take(&mut a.novo);
    let vivo = reancora(a, &state::current_snapshot());
    if (fora && !novo) || a.tab != tab || !vivo {
        state.sinal_aberto = None;
    }
}

/// ⭐ **Segue o MESMO marcador**: com a contagem igual o índice vale (arrasto, renome); se mudou
/// (um marcador entrou ou saiu), o marcador de sinal no MESMO tempo é ele — senão sumiu e fecha.
fn reancora(a: &mut Aberto, snap: &TimelineViewSnapshot) -> bool {
    let index = if snap.markers.len() == a.n {
        Some(a.index)
    } else {
        (0..snap.markers.len())
            .find(|&i| tem_sinal(snap, i).is_some() && (snap.markers[i].0 - a.t).abs() < 1e-9)
    };
    let Some(index) = index.filter(|&i| tem_sinal(snap, i).is_some()) else {
        return false;
    };
    a.index = index;
    a.t = snap.markers[index].0;
    a.n = snap.markers.len();
    true
}

/// O rótulo de quem ouve: o nome do componente (ou documento) para o artista. Os padrões são os NOMES
/// DE TIPO que o censo entrega (`stringify!` da tabela `CAMPOS`, o `doc` da `FORA`): identidade, não frase.
fn rotulo(q: Quem) -> &'static str {
    use ph2d_i18n::tr;
    match q {
        Quem::Script => tr("panel.timeline.signal.who.script"),
        Quem::Component("SignalActions") => tr("panel.timeline.signal.who.signal_actions"),
        Quem::Component("WeaponFire") => tr("panel.timeline.signal.who.weapon_fire"),
        Quem::Component("Factory") => tr("panel.timeline.signal.who.factory"),
        Quem::Component("StateMachine") => tr("panel.timeline.signal.who.state_machine"),
        Quem::Component("ShakeEmitter") => tr("panel.timeline.signal.who.shake_emitter"),
        Quem::Component("ParticleEmitter") => tr("panel.timeline.signal.who.particle_emitter"),
        Quem::Component("AnimRules") => tr("panel.timeline.signal.who.anim_rules"),
        Quem::Doc("MotionGraph") => tr("panel.timeline.signal.who.motion_graph"),
        Quem::Doc("StateSets") => tr("panel.timeline.signal.who.state_sets"),
        // Uma linha nova da tabela do censo sem rótulo: o nome do tipo (dado, não frase).
        Quem::Component(t) | Quem::Doc(t) => t,
    }
}

/// Uma linha da lista (maquete B5): «Heard by Camera» em negrito e o «· Camera Shake» na tinta do
/// rótulo; sem objecto (um documento, um script) o rótulo vai no lugar do nome.
struct Ouvinte {
    forte: String,
    resto: Option<String>,
}

/// As linhas «Heard by»: `None` = a shell ainda não respondeu por este nome.
fn linhas(nome: &str) -> Option<(Vec<Ouvinte>, bool)> {
    OUVINTES.with(|o| {
        let o = o.borrow();
        let (n, l) = o.as_ref().filter(|(n, _)| n == nome)?;
        debug_assert_eq!(n, nome);
        let rows = l
            .list
            .iter()
            .map(|x| {
                let ouvido = |quem: &str| {
                    ph2d_i18n::tr_with("panel.timeline.signal.heard_by", &[("who", &quem)])
                };
                match &x.object {
                    Some(obj) => Ouvinte {
                        forte: ouvido(obj),
                        resto: Some(ph2d_i18n::tr_with(
                            "panel.timeline.signal.row",
                            &[("what", &rotulo(x.what))],
                        )),
                    },
                    None => Ouvinte {
                        forte: ouvido(rotulo(x.what)),
                        resto: None,
                    },
                }
            })
            .collect();
        Some((rows, l.script_may))
    })
}

/// Uma linha numa tinta (o «Nobody listens» na do aviso): uma linha só, com «…» se não cabe.
fn linha_tinta(c: &mut CardCtx<'_>, text: &str, cor: ColorToken, y: f32) -> f32 {
    let top = y + m::FIELD_ROW_GAP;
    let ty = top + folga_do_texto(m::FIELD_H, m::VALUE_PX);
    let tinta = c.ink.color(cor);
    paint_text_elided(
        c.text_system,
        c.scene,
        text,
        c.x,
        ty,
        m::VALUE_PX,
        c.w,
        tinta,
    );
    top + m::FIELD_H
}

/// A caixa de uma linha «Heard by» (maquete B5): o fundo dos campos, o ícone e o texto numa linha só.
/// Devolve o `x` e a largura que sobram para o texto.
fn caixa(c: &mut CardCtx<'_>, y: f32, icone: IconId, tinta: ColorToken) -> (f32, f32, f32) {
    let r = Rect::new(c.x, y + m::FIELD_ROW_GAP, c.w, m::FIELD_H);
    fill_rounded_rect(
        c.scene,
        r,
        m::FIELD_RADIUS,
        c.ink.color(ColorToken::CardField),
    );
    let g = m::CHECK_BOX;
    let glifo = Rect::new(r.x + m::FIELD_PAD_R, r.y + (r.h - g) * 0.5, g, g);
    let cor = c.ink.color(tinta);
    paint_icon(c.scene, icone, glifo, cor, StrokeToken::Default.px());
    let tx = glifo.x + g + m::CHECK_GAP;
    (tx, (r.x + r.w - m::FIELD_PAD_R - tx).max(0.0), r.y)
}

/// A largura que uma linha em caixa pede além do texto.
fn caixa_extra() -> f32 {
    m::FIELD_PAD_R * 2.0 + m::CHECK_BOX + m::CHECK_GAP
}

/// «Inimigo · Signal Actions»: o objecto em negrito, o resto na tinta do rótulo, uma linha com «…».
fn linha_ouvinte(c: &mut CardCtx<'_>, o: &Ouvinte, y: f32) -> f32 {
    let (x, w, top) = caixa(c, y, IconId::Success, ColorToken::CardLabel);
    let ty = top + folga_do_texto(m::FIELD_H, m::VALUE_PX);
    let forte = c.ink.color(ColorToken::Text1);
    let fw = largura_do_titulo(c.text_system, &o.forte, m::VALUE_PX).min(w);
    paint_text_title_elided(
        c.text_system,
        c.scene,
        &o.forte,
        x,
        ty,
        m::VALUE_PX,
        w,
        forte,
    );
    if let Some(resto) = &o.resto {
        let rx = x + fw + m::CHECK_GAP * 0.5;
        let rw = (x + w - rx).max(0.0);
        let tinta = c.ink.color(ColorToken::CardLabel);
        paint_text_elided(
            c.text_system,
            c.scene,
            resto,
            rx,
            ty,
            m::VALUE_PX,
            rw,
            tinta,
        );
    }
    top + m::FIELD_H
}

/// «Nobody listens»: a mesma caixa, o ícone e o texto na tinta do aviso.
fn linha_ninguem(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let (x, w, top) = caixa(c, y, IconId::Warning, ColorToken::Warn);
    let ty = top + folga_do_texto(m::FIELD_H, m::VALUE_PX);
    let texto = ph2d_i18n::tr("panel.timeline.signal.nobody");
    let tinta = c.ink.color(ColorToken::Warn);
    paint_text_elided(c.text_system, c.scene, texto, x, ty, m::VALUE_PX, w, tinta);
    top + m::FIELD_H
}

/// O topo do cartão de altura `h`, ou `None` se não cabe inteiro: nunca acima de `topo` (o fundo da
/// barra, o topo da régua e das faixas); acima da alça se cabe, senão debaixo dela dentro do painel.
fn lugar_y(painel: Rect, topo: f32, alca: Rect, h: f32) -> Option<f32> {
    let fundo = painel.y + painel.h;
    let acima = alca.y - m::FIELD_ROW_GAP - h;
    let abaixo = alca.y + alca.h + m::FIELD_ROW_GAP;
    if acima >= topo {
        Some(acima)
    } else if abaixo + h <= fundo {
        Some(abaixo)
    } else {
        None
    }
}

/// Uma linha da lista do cartão, pela ordem em que se pinta.
#[derive(Copy, Clone)]
enum Linha<'a> {
    Ouvinte(&'a Ouvinte),
    Ninguem,
    Script,
}

fn lista(rows: &[Ouvinte], may: bool) -> Vec<Linha<'_>> {
    let mut v: Vec<Linha<'_>> = rows.iter().map(Linha::Ouvinte).collect();
    if rows.is_empty() {
        v.push(Linha::Ninguem);
    }
    if may {
        v.push(Linha::Script);
    }
    v
}

/// Pinta o cartão aberto por cima de tudo, acima da alça do marcador, alinhado à direita da linha
/// dele e preso dentro do painel; publica o pedido para a shell.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let aberto = state
        .sinal_aberto
        .filter(|_| ph2d_editor_core::panel::cards::ferramenta::cards_on())
        .and_then(|a| Some((a.index, tem_sinal(snap, a.index)?.to_owned())));
    PEDIDO.with(|p| *p.borrow_mut() = aberto.as_ref().map(|a| a.1.clone()));
    let alca = aberto.as_ref().and_then(|(i, _)| {
        ctx.host
            .hit_index_mut()
            .rect_for(ids::timeline_marker_hit_id(*i))
    });
    let (Some((index, nome)), Some(alca)) = (aberto, alca) else {
        ctx.host
            .store_mut()
            .publish_own_list(ids::TIMELINE_SIGNAL_CARD, None);
        return;
    };
    let quadro = (snap.markers[index].0 * snap.fps.max(0.0)).round() as i64;
    let titulo = ph2d_i18n::tr_with(
        "panel.timeline.signal.title",
        &[("frame", &quadro.to_string()), ("name", &nome)],
    );
    let ouvintes = linhas(&nome);
    let tr = ph2d_i18n::tr;
    let todas = ouvintes
        .as_ref()
        .map_or_else(Vec::new, |(rows, may)| lista(rows, *may));
    // `k` linhas da lista; as que não cabem dizem-se numa última «+N more».
    let corpo_de = |k: usize| {
        let todas = &todas;
        let titulo = &titulo;
        let nome = &nome;
        move |c: &mut CardCtx<'_>, y: f32| {
            let mut y = c.full(None, Widget::Readout { text: titulo }, y);
            y = c.full(
                Some(tr("panel.timeline.signal.name")),
                Widget::Readout { text: nome },
                y,
            );
            for l in todas.iter().take(k) {
                y = match l {
                    Linha::Ouvinte(o) => linha_ouvinte(c, o, y),
                    Linha::Ninguem => linha_ninguem(c, y),
                    Linha::Script => linha_tinta(
                        c,
                        tr("panel.timeline.signal.script_may"),
                        ColorToken::CardLabel,
                        y,
                    ),
                };
            }
            if k < todas.len() {
                let n = (todas.len() - k).to_string();
                let mais = ph2d_i18n::tr_with("panel.timeline.signal.more", &[("n", &n)]);
                y = linha_tinta(c, &mais, ColorToken::CardLabel, y);
            }
            y
        }
    };
    let painel = ctx
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::TIMELINE_PANEL)
        .unwrap_or(ctx.viewport);
    let mut textos = vec![titulo.as_str(), nome.as_str()];
    let mut larga: f32 = 0.0;
    if let Some((rows, _)) = &ouvintes {
        textos.push(tr("panel.timeline.signal.script_may"));
        let ts = &mut *ctx.text_system;
        let nobody = ts.prefix_width(tr("panel.timeline.signal.nobody"), m::VALUE_PX);
        larga = larga.max(nobody + caixa_extra());
        for o in rows {
            let resto = o.resto.as_deref().map_or(0.0, |r| {
                m::CHECK_GAP * 0.5 + ts.prefix_width(r, m::VALUE_PX)
            });
            let forte = largura_do_titulo(ts, &o.forte, m::VALUE_PX);
            larga = larga.max(forte + resto + caixa_extra());
        }
    }
    let larga = textos
        .iter()
        .map(|t| largura_do_titulo(ctx.text_system, t, m::VALUE_PX))
        .fold(larga, f32::max)
        .ceil();
    let dentro = larga.min((painel.w - m::CARD_PAD_X * 4.0).max(0.0));
    let w = dentro + m::CARD_PAD_X * 2.0;
    let x_linha = alca.x + alca.w * 0.5;
    let x = (x_linha - w).min(painel.x + painel.w - w).max(painel.x);
    let topo = topo_das_faixas(ctx, painel);
    let mut k = todas.len();
    let h = crate::barra_menus::altura(ctx, theme, dentro, &corpo_de(k));
    let (y, h) = match lugar_y(painel, topo, alca, h) {
        Some(y) => (y, h),
        // Nem acima nem abaixo: colado à barra, até ao fundo do painel, as linhas que cabem.
        None => {
            let livre = (painel.y + painel.h - topo).max(0.0);
            let mut h = h;
            while k > 0 && h > livre {
                k -= 1;
                h = crate::barra_menus::altura(ctx, theme, dentro, &corpo_de(k));
            }
            (topo, h.min(livre))
        }
    };
    let rect = Rect::new(x, y, w, h);
    crate::barra_menus::flutua_em(
        ctx,
        theme,
        (ids::TIMELINE_SIGNAL_CARD, rect),
        dentro,
        &corpo_de(k),
    );
}

/// O fundo da barra: o topo da régua e das faixas (o mais alto dos dois), ou o topo do painel.
fn topo_das_faixas(ctx: &mut PaintCtx, painel: Rect) -> f32 {
    let hits = ctx.host.hit_index_mut();
    [ids::TIMELINE_RULER, ids::TIMELINE_LANES]
        .into_iter()
        .filter_map(|id| hits.rect_for(id).map(|r| r.y))
        .reduce(f32::min)
        .unwrap_or(painel.y)
}

/// ⭐ A porta da shell num gesto: com um cartão aberto, `calcula` quem ouve o nome dele e entrega-o.
pub fn ouvintes(calcula: impl FnOnce(&str) -> SignalListeners) {
    if let Some(n) = signal_asked() {
        set_signal_listeners(&n, calcula(&n));
    }
}
