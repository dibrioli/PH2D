//! ⭐ **A fileira dos clipes, no molde do Rive** (`docs/Timeline/15_plano_redesenho_molde_rive.md`, onda 1):
//! um chip por opção da aba (`transport_clips::source_options` — os mesmos ids, o mesmo clique que a
//! lista suspensa tinha), o ▶ no clipe com que o jogo começa, o «⋯» no chip escolhido e o «+». Nas abas
//! de faixas (onda 3, `docs/Timeline/16_onda3_containers_arrange.md`, maquete B4) os containers são um
//! 2.º grupo rotulado na MESMA fileira, cada chip com o uso dele («used 2×» / «not placed»).
//!
//! O chip escolhido devolve-se como o `ClipChip` de `TIMELINE_CLIP_DD`: é sobre ele que o campo de
//! renomear flutua, como flutuava sobre a lista.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, folga_do_texto, paint_icon, paint_text_elided};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{self, Icone, metrics as m};
use ph2d_editor_core::panel::cards::{CardCtx, dicas};
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::chrome::{TIMELINE_CHIP_ARRASTADO_ALPHA, TIMELINE_CHIP_ARRASTADO_ANEL_PX};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};

use crate::ids;
use crate::tab::Tab;
use crate::transport::cartoes::Fluxo;
use crate::transport::{BarView, ClipChip};

/// O nome mais largo que um chip mostra antes de cortar.
const TEXTO_MAX: f32 = 140.0; // LITERAL-PX-OK: «Animation: Scene» cabe; um nome maior corta com reticência
/// O glifo do tipo (clipe · container) e o ▶ do início.
const GLIFO: f32 = m::HEAD_ICON_GLYPH;
const ESTRELA: f32 = m::KEY_MARK_BOX;

/// O que a fileira devolve: o chip escolhido e onde o «⋯» dele ficou.
pub(crate) struct Clips {
    pub(crate) chip: Option<ClipChip>,
    pub(crate) menu: Option<Rect>,
}

/// O clipe da linha `i` (na aba Keys) é aquele com que a animação começa?
fn comeca_com(snap: &TimelineViewSnapshot, view: BarView, opcao: &DropdownOption<usize>) -> bool {
    if view.tab != Tab::Keys {
        return false;
    }
    match crate::transport_clips::source_at(opcao.id) {
        Some(Err(linha)) => snap
            .clip_index
            .get(linha)
            .is_some_and(|&i| snap.animation.entry == Some(i)),
        _ => false,
    }
}

/// O «⋯» vive no chip escolhido, se ele é um CLIPE: o menu age sobre o clipe ativo.
fn tem_menu(on: bool, opcao: &DropdownOption<usize>) -> bool {
    on && !matches!(crate::transport_clips::source_at(opcao.id), Some(Ok(_)))
}

/// O glifo do TIPO no chip (onda 1b, maquete B1) — o MESMO da lista aberta (dono 12/11): uma fonte
/// só, `transport_clips::{CLIP_GLYPH, CONTAINER_GLYPH}`. Uma opção sem tipo fica com o seu.
pub(crate) fn glifo_do_tipo(opcao: &DropdownOption<usize>) -> Option<IconId> {
    use crate::transport_clips::{CLIP_GLYPH, CONTAINER_GLYPH};
    match crate::transport_clips::source_at(opcao.id) {
        Some(Ok(_)) => Some(CONTAINER_GLYPH),
        Some(Err(_)) => Some(CLIP_GLYPH),
        None => opcao.icon,
    }
}

/// A largura do texto do chip, já cortada.
fn texto_w(ctx: &mut PaintCtx, nome: &str) -> f32 {
    ctx.text_system
        .prefix_width(nome, m::VALUE_PX)
        .ceil()
        .min(TEXTO_MAX)
}

/// O texto secundário de um chip (o «used 2×» do container): um degrau abaixo do nome.
const USO_PX: f32 = m::LABEL_PX;

fn uso_w(ctx: &mut PaintCtx, uso: &str) -> f32 {
    ctx.text_system.prefix_width(uso, USO_PX).ceil()
}

fn largura(ctx: &mut PaintCtx, ch: &Chip<'_>) -> f32 {
    let mut w =
        m::SEG_INSET + GLIFO + m::SEG_ICON_GAP + texto_w(ctx, &ch.opcao.label) + m::SEG_INSET;
    if let Some(uso) = &ch.uso {
        w += m::SEG_ICON_GAP + uso_w(ctx, uso);
    }
    if ch.estrela {
        w += ESTRELA + m::SEG_ICON_GAP;
    }
    if ch.menu {
        w += m::FIELD_H - m::SEG_INSET;
    }
    if ch.done {
        w += m::FIELD_H;
    }
    w
}

struct Chip<'a> {
    opcao: &'a DropdownOption<usize>,
    on: bool,
    estrela: bool,
    menu: bool,
    /// O Done (verde, ✓) no fim do chip: o clipe escolhido enquanto se anima (dono 13/11).
    done: bool,
    /// Onde o container é usado (`crate::uso::rotulo`); `None` num clipe.
    uso: Option<String>,
    /// A cor da fonte quando este chip está na mão (`chip_drag`).
    arrastado: Option<ph2d_tokens::TimelineTinta>,
}

/// A frase do uso, se a opção é um CONTAINER.
fn uso_de(snap: &TimelineViewSnapshot, opcao: &DropdownOption<usize>) -> Option<String> {
    match crate::transport_clips::source_at(opcao.id) {
        Some(Ok(c)) => snap.containers.get(c).map(|c| crate::uso::rotulo(c.uses)),
        _ => None,
    }
}

/// Pinta um chip em `r`; devolve o rect do «⋯», se tem.
fn pinta_chip(ctx: &mut PaintCtx, ink: CardInk, r: Rect, ch: &Chip<'_>) -> Option<Rect> {
    let menu_w = if ch.menu { m::FIELD_H } else { 0.0 };
    let done_w = if ch.done { m::FIELD_H } else { 0.0 };
    let corpo = Rect::new(r.x, r.y, r.w - menu_w - done_w, r.h);
    let hover = ctx.host.store().hover_live(ch.opcao.id) > 0.0;
    let fundo = if ch.on {
        ColorToken::AccentSoft
    } else if hover {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    // Uma pílula, como a maquete B1; a escolhida leva o contorno do acento.
    let raio = r.h * 0.5;
    // O chip escolhido no azul da maquete B1 (onda 2g).
    let cor_fundo = if ch.on {
        crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::ChipEscolhido, ctx.host.theme())
    } else {
        ink.color(fundo)
    };
    fill_rounded_rect(ctx.scene, r, raio, cor_fundo);
    if let Some(t) = ch.arrastado {
        // O chip na mão (`.dr` da B4): a cor da fonte a 25 % e o anel dela por dentro.
        let cor = crate::folha_tinta::tinta(t, ctx.host.theme());
        fill_rounded_rect(
            ctx.scene,
            r,
            raio,
            cor.multiply_alpha(TIMELINE_CHIP_ARRASTADO_ALPHA),
        );
        let w = TIMELINE_CHIP_ARRASTADO_ANEL_PX;
        let dentro = Rect::new(r.x + w * 0.5, r.y + w * 0.5, r.w - w, r.h - w);
        // FRAME-RAW-OK: o anel do chip na mão É a marca do arrasto (`.dr`), na cor da fonte.
        ph2d_editor_core::paint::stroke_rounded_rect(ctx.scene, dentro, raio - w * 0.5, w, cor);
    } else if ch.on {
        // FRAME-RAW-OK: o contorno do acento É a marca do chip escolhido (como a célula sob a mão).
        ph2d_editor_core::paint::stroke_rounded_rect(
            ctx.scene,
            r,
            raio,
            StrokeToken::Default.px(),
            ink.color(ColorToken::Accent),
        );
    }
    // ⭐ O chip é uma superfície da Timeline (doc 17 item 6): o clique escolhe como a lista escolhia,
    // o arrasto leva-o a uma faixa (`chip_drag`). Registado sempre: a lista agrupada da Cena só põe as
    // linhas dela no store quando se pinta.
    ctx.host.store_mut().register(
        ch.opcao.id,
        ph2d_editor_core::interaction::InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind: crate::chip_drag::superficie(ch.opcao.id),
            canvas: corpo,
        },
    );
    ctx.host.hit_index_mut().register(ch.opcao.id, corpo);
    let tinta = ink.color(if ch.on || hover {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    });
    let mut x = r.x + m::SEG_INSET;
    if let Some(icon) = glifo_do_tipo(ch.opcao) {
        let g = Rect::new(x, r.y + (r.h - GLIFO) * 0.5, GLIFO, GLIFO);
        paint_icon(ctx.scene, icon, g, tinta, StrokeToken::Default.px());
    }
    x += GLIFO + m::SEG_ICON_GAP;
    let tw = texto_w(ctx, &ch.opcao.label);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &ch.opcao.label,
        x,
        r.y + folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        tw,
        tinta,
    );
    x += tw;
    if let Some(uso) = &ch.uso {
        x += m::SEG_ICON_GAP;
        let uw = uso_w(ctx, uso);
        let cor =
            crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoValor, ctx.host.theme());
        paint_text_elided(
            ctx.text_system,
            ctx.scene,
            uso,
            x,
            r.y + folga_do_texto(r.h, USO_PX),
            USO_PX,
            uw,
            cor,
        );
        x += uw;
    }
    if ch.estrela {
        x += m::SEG_ICON_GAP;
        let e = Rect::new(x, r.y + (r.h - ESTRELA) * 0.5, ESTRELA, ESTRELA);
        paint_icon(
            ctx.scene,
            IconId::Play,
            e,
            ink.color(ColorToken::Warn),
            StrokeToken::Default.px(),
        );
    }
    if ch.done {
        crate::animando::botao_done_no_chip(ctx, r);
    }
    if !ch.menu {
        return None;
    }
    let b = Rect::new(r.x + r.w - done_w - menu_w, r.y, menu_w, r.h);
    pinta_reticencias(ctx, ink, b);
    Some(b)
}

/// O «⋯» do clipe ativo em `b` (o menu do clipe pende dele).
fn pinta_reticencias(ctx: &mut PaintCtx, ink: CardInk, b: Rect) {
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CLIP_MENU, b);
    dicas::icone(ids::TIMELINE_CLIP_MENU);
    let aceso = ctx.host.store().hover_live(ids::TIMELINE_CLIP_MENU) > 0.0;
    let g = m::ICON_GLYPH;
    paint_icon(
        ctx.scene,
        IconId::MoreHorizontal,
        Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
        ink.color(if aceso {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        }),
        StrokeToken::Default.px(),
    );
}

/// ⭐ «Show clips as a list» (dono 12/11, na engrenagem): a fileira vira UMA lista suspensa — a de
/// sempre (`TIMELINE_CLIP_DD`, com rolagem), com o nome do escolhido —, o «⋯» dele e o «+». Os ids e o
/// clique são os dos chips.
fn como_lista(
    ctx: &mut PaintCtx,
    ink: CardInk,
    f: &mut Fluxo,
    body: Rect,
    (opcao, animando_done): (Option<&DropdownOption<usize>>, bool),
) -> Clips {
    let texto = opcao.map_or("", |o| o.label.as_str());
    // O glifo do tipo também na lista FECHADA (dono 12/11): o mesmo do chip e da lista aberta.
    let icone = opcao.and_then(glifo_do_tipo);
    let li = ph2d_editor_core::widget::card_chip_icone_w(icone);
    let w = li + texto_w(ctx, texto) + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH;
    let menu = opcao.is_some_and(|o| tem_menu(true, o));
    let done = animando_done;
    let fim = if menu { m::FIELD_H + m::FIELD_GAP } else { 0.0 }
        + if done { m::FIELD_H + m::FIELD_GAP } else { 0.0 };
    f.guarda_para(w + m::FIELD_GAP + fim + banda::icones_w(1));
    let (x, y) = f.poe(w);
    let r = Rect::new(x, y, w, m::FIELD_H);
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let open = matches!(
        store.get(ids::TIMELINE_CLIP_DD),
        Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
    );
    CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w)
        .lista_com_icone_em(ids::TIMELINE_CLIP_DD, (icone, texto), open, r);
    let menu = menu.then(|| {
        let (x, y) = f.poe(m::FIELD_H);
        let b = Rect::new(x, y, m::FIELD_H, m::FIELD_H);
        pinta_reticencias(ctx, ink, b);
        b
    });
    if done {
        let (x, y) = f.poe(m::FIELD_H);
        let b = Rect::new(x, y, m::FIELD_H, m::FIELD_H);
        crate::animando::botao_done(ctx, ctx.host.theme(), b, m::FIELD_RADIUS);
    }
    pinta_o_mais(ctx, ink, f, body);
    Clips {
        chip: Some(ClipChip {
            id: ids::TIMELINE_CLIP_DD,
            rect: r,
            open,
        }),
        menu,
    }
}

/// Pinta o rótulo da fileira (ou o cabeçalho de um dono, na lista agrupada da Cena).
fn rotulo(ctx: &mut PaintCtx, ink: CardInk, f: &mut Fluxo, texto: &str) {
    let w = ctx.text_system.prefix_width(texto, m::LABEL_PX).ceil();
    let (x, y) = f.poe(w);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        texto,
        x,
        y + folga_do_texto(m::FIELD_H, m::LABEL_PX),
        m::LABEL_PX,
        w,
        ink.color(ColorToken::CardLabel),
    );
}

/// Uma peça da fileira, na ordem da lista: um chip, ou o cabeçalho de um dono (lista agrupada da Cena).
struct Peca<'a> {
    i: usize,
    chip: Option<Chip<'a>>,
    w: f32,
    /// Um container: mora no 2.º grupo («Containers»).
    container: bool,
}

/// As larguras fixas da fileira, medidas antes de pintar.
struct Medida {
    /// O «+» e o vão dele.
    mais: f32,
    /// O rótulo «Containers» e o vão dele (0 sem containers).
    grupo: f32,
    /// O «+N» mais largo que pode sair, e o vão dele.
    reserva: f32,
    /// A fileira inteira, sem dobra.
    tudo: f32,
    /// Só o grupo dos clipes e o «+»: é ele que decide se a fileira desce — o 2.º grupo dobra no «+N»,
    /// nunca acrescenta uma fileira.
    clipes: f32,
    /// A primeira peça do 2.º grupo.
    primeiro: Option<usize>,
}

/// O que cabe: cada peça à vista ou dobrada no «+N», e quantos chips dobraram.
struct Corte {
    visivel: Vec<bool>,
    escondidos: usize,
}

fn pecas<'a>(
    ctx: &mut PaintCtx,
    (snap, view): (&TimelineViewSnapshot, BarView),
    opcoes: &'a [DropdownOption<usize>],
    escolhida: usize,
) -> Vec<Peca<'a>> {
    opcoes
        .iter()
        .enumerate()
        .map(|(i, opcao)| {
            if opcao.disabled || opcao.header {
                let w = ctx
                    .text_system
                    .prefix_width(&opcao.label, m::LABEL_PX)
                    .ceil();
                return Peca {
                    i,
                    chip: None,
                    w,
                    container: false,
                };
            }
            let on = i == escolhida;
            let ch = Chip {
                opcao,
                on,
                estrela: comeca_com(snap, view, opcao),
                menu: tem_menu(on, opcao),
                done: on && crate::animando::rotulo(snap).is_some(),
                uso: uso_de(snap, opcao),
                arrastado: crate::chip_drag::tinta_do_chip(opcao.id, snap, &opcao.label),
            };
            let w = largura(ctx, &ch);
            Peca {
                i,
                container: matches!(crate::transport_clips::source_at(opcao.id), Some(Ok(_))),
                chip: Some(ch),
                w,
            }
        })
        .collect()
}

fn texto_mais(n: usize) -> String {
    ph2d_i18n::tr_with("panel.timeline.card.more", &[("n", &n)])
}

fn mais_n_w(ctx: &mut PaintCtx, n: usize) -> f32 {
    ctx.text_system
        .prefix_width(&texto_mais(n), m::VALUE_PX)
        .ceil()
        + m::FIELD_PAD_R * 2.0
        + m::ICON_GLYPH
}

fn mede(ctx: &mut PaintCtx, pecas: &[Peca<'_>], opcoes: usize) -> Medida {
    let mais = banda::icones_w(1) + m::FIELD_GAP;
    let primeiro = pecas.iter().position(|p| p.container);
    let grupo = if primeiro.is_some() {
        let nome = ph2d_i18n::tr("panel.timeline.card.containers");
        ctx.text_system.prefix_width(nome, m::LABEL_PX).ceil() + m::FIELD_GAP
    } else {
        0.0
    };
    let soma = |ps: &[Peca<'_>]| ps.iter().map(|p| p.w + m::FIELD_GAP).sum::<f32>();
    let ate = primeiro.unwrap_or(pecas.len());
    Medida {
        mais,
        grupo,
        reserva: mais_n_w(ctx, opcoes) + m::FIELD_GAP,
        tudo: soma(pecas) + mais + grupo,
        clipes: soma(&pecas[..ate]) + mais,
        primeiro,
    }
}

/// ⭐ UMA regra de dobra para os dois grupos: na ordem da lista, depois do primeiro que não cabe nenhum
/// passa à frente; o «+» (fecha os clipes) e o rótulo «Containers» (abre o 2.º grupo) contam onde se pintam.
fn corta(pecas: &[Peca<'_>], me: &Medida, resta: f32) -> Corte {
    let mut c = Corte {
        visivel: vec![false; pecas.len()],
        escondidos: 0,
    };
    let mut usado = 0.0;
    let mut cortou = false;
    for (k, p) in pecas.iter().enumerate() {
        let antes = if Some(k) == me.primeiro {
            me.mais + me.grupo
        } else {
            0.0
        };
        let depois = if p.container { 0.0 } else { me.mais };
        cortou |=
            me.tudo > resta && usado + antes + p.w + m::FIELD_GAP + depois + me.reserva > resta;
        if cortou {
            c.escondidos += usize::from(p.chip.is_some());
            continue;
        }
        usado += antes + p.w + m::FIELD_GAP;
        c.visivel[k] = true;
    }
    c
}

/// O «+N ▾» com os `n` que dobraram: abre a lista de sempre (`TIMELINE_CLIP_DD`).
fn pinta_mais_n(
    ctx: &mut PaintCtx,
    ink: CardInk,
    f: &mut Fluxo,
    body: Rect,
    n: usize,
    out: &mut Clips,
) {
    if n == 0 {
        return;
    }
    let w = mais_n_w(ctx, n);
    let (x, y) = f.poe(w);
    let r = Rect::new(x, y, w, m::FIELD_H);
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let open = matches!(
        store.get(ids::TIMELINE_CLIP_DD),
        Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
    );
    let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w);
    c.lista_em(ids::TIMELINE_CLIP_DD, &texto_mais(n), open, r);
    // A lista pende do «+N»; o campo de renomear, do chip escolhido se ele está à vista.
    if open || out.chip.is_none() {
        out.chip = Some(ClipChip {
            id: ids::TIMELINE_CLIP_DD,
            rect: r,
            open,
        });
    }
}

/// O «+» que cria um clipe: fecha o grupo dos clipes.
fn pinta_o_mais(ctx: &mut PaintCtx, ink: CardInk, f: &mut Fluxo, body: Rect) {
    let (x, y) = f.poe(banda::icones_w(1));
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w);
    c.icones_em_fila(
        &[Icone {
            id: Some(ids::TIMELINE_ADD_CLIP),
            icon: IconId::Plus,
            on: false,
        }],
        x,
        y,
    );
}

/// A 2.ª fileira: «Clips» · um chip por clipe · «+»; nas abas de faixas, a seguir, «Containers» · um
/// chip por container com o uso dele (doc 16, maquete B4). Os ids e o clique são os da lista de sempre.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    (snap, view): (&TimelineViewSnapshot, BarView),
    body: Rect,
) -> Clips {
    let ink = CardInk::of_list(theme);
    let opcoes = crate::transport_clips::source_options(snap, view.tab);
    let escolhida = crate::transport_clips::selected_source(snap, view);
    let mut out = Clips {
        chip: None,
        menu: None,
    };
    // ⭐ UMA fileira só (a Cena chega a ~100 clipes): o que não cabe vai para o «+N ▾» (`TIMELINE_CLIP_DD`).
    let pecas = pecas(ctx, (snap, view), &opcoes, escolhida);
    let medida = mede(ctx, &pecas, opcoes.len());
    let nome = ph2d_i18n::tr("panel.timeline.card.clips");
    let nome_w = ctx.text_system.prefix_width(nome, m::LABEL_PX).ceil();
    // O grupo desce inteiro se cabe numa fileira; se nem o 1.º chip cabe no resto, desce também.
    let um = pecas
        .first()
        .map_or(0.0, |p| p.w + m::FIELD_GAP + medida.mais + medida.reserva);
    let minimo = medida.clipes.min(um.max(medida.mais));
    if !view.clips_as_list {
        f.guarda_para(banda::SEPARADOR_W + m::FIELD_GAP + nome_w + m::FIELD_GAP + medida.clipes);
        f.guarda_para(banda::SEPARADOR_W + m::FIELD_GAP + nome_w + m::FIELD_GAP + minimo);
    }
    // Nem um chip ao lado do fim do clipe: ele desce.
    if f.resta() < banda::SEPARADOR_W + m::FIELD_GAP * 2.0 + nome_w + minimo {
        f.larga_reserva();
    }
    if !f.no_inicio() {
        // O separador do Marker nunca abre uma fileira.
        let (x, y) = f.poe(banda::SEPARADOR_W);
        let (store, hit) = ctx.host.store_and_hit_index_mut();
        CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w)
            .separador_em(x, y);
    }
    rotulo(ctx, ink, f, nome);
    if view.clips_as_list {
        let done = crate::animando::rotulo(snap).is_some();
        return como_lista(ctx, ink, f, body, (opcoes.get(escolhida), done));
    }
    let corte = corta(&pecas, &medida, f.resta());
    for (k, p) in pecas.iter().enumerate() {
        if Some(k) == medida.primeiro {
            // Os clipes acabam: o «+N» (se a dobra começou neles) e o «+»; abre-se o 2.º grupo.
            if !corte.visivel[k] {
                pinta_mais_n(ctx, ink, f, body, corte.escondidos, &mut out);
            }
            pinta_o_mais(ctx, ink, f, body);
            if corte.visivel[k] {
                rotulo(ctx, ink, f, ph2d_i18n::tr("panel.timeline.card.containers"));
            }
        }
        if !corte.visivel[k] {
            continue;
        }
        let Some(ch) = &p.chip else {
            rotulo(ctx, ink, f, &opcoes[p.i].label);
            continue;
        };
        let (x, y) = f.poe(p.w);
        let r = Rect::new(x, y, p.w, m::FIELD_H);
        let menu = pinta_chip(ctx, ink, r, ch);
        if ch.on {
            out.chip = Some(ClipChip {
                id: ids::TIMELINE_CLIP_DD,
                rect: r,
                open: false,
            });
            out.menu = menu;
        }
    }
    match medida.primeiro {
        None => {
            pinta_mais_n(ctx, ink, f, body, corte.escondidos, &mut out);
            pinta_o_mais(ctx, ink, f, body);
        }
        Some(k) if corte.visivel[k] => pinta_mais_n(ctx, ink, f, body, corte.escondidos, &mut out),
        Some(_) => {}
    }
    out
}
