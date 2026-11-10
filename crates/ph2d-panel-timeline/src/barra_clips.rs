//! ⭐ **A fileira dos clipes, no molde do Rive** (`docs/Timeline/15_plano_redesenho_molde_rive.md`, onda 1):
//! um chip por opção da aba (`transport_clips::source_options` — os mesmos ids, o mesmo clique que a
//! lista suspensa tinha), o ▶ no clipe com que o jogo começa, o «⋯» no chip escolhido e o «+».
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

/// O glifo do TIPO no chip (onda 1b, maquete B1): clipe = filme, container = cubo. A lista suspensa
/// de sempre (e `PH2D_CARDS=0`) guarda os glifos dela; uma opção sem tipo fica com o seu.
fn glifo_do_tipo(opcao: &DropdownOption<usize>) -> Option<IconId> {
    match crate::transport_clips::source_at(opcao.id) {
        Some(Ok(_)) => Some(IconId::Cube),
        Some(Err(_)) => Some(IconId::Film),
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

fn largura(ctx: &mut PaintCtx, nome: &str, estrela: bool, menu: bool) -> f32 {
    let mut w = m::SEG_INSET + GLIFO + m::SEG_ICON_GAP + texto_w(ctx, nome) + m::SEG_INSET;
    if estrela {
        w += ESTRELA + m::SEG_ICON_GAP;
    }
    if menu {
        w += m::FIELD_H - m::SEG_INSET;
    }
    w
}

struct Chip<'a> {
    opcao: &'a DropdownOption<usize>,
    on: bool,
    estrela: bool,
    menu: bool,
}

/// Pinta um chip em `r`; devolve o rect do «⋯», se tem.
fn pinta_chip(ctx: &mut PaintCtx, ink: CardInk, r: Rect, ch: &Chip<'_>) -> Option<Rect> {
    let menu_w = if ch.menu { m::FIELD_H } else { 0.0 };
    let corpo = Rect::new(r.x, r.y, r.w - menu_w, r.h);
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
    if ch.on {
        ph2d_editor_core::paint::stroke_rounded_rect(
            ctx.scene,
            r,
            raio,
            StrokeToken::Default.px(),
            ink.color(ColorToken::Accent),
        );
    }
    // As linhas da lista agrupada da Cena só entram no store quando a lista se pinta (a faixa visível);
    // um chip à vista tem de ser um `Button`, senão o clique nunca nasce.
    ctx.host.store_mut().register_if_absent(
        ch.opcao.id,
        ph2d_editor_core::interaction::InteractiveState::Button {
            state: ph2d_editor_core::widget::ButtonState::Normal,
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
    if !ch.menu {
        return None;
    }
    let b = Rect::new(r.x + r.w - menu_w, r.y, menu_w, r.h);
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
    Some(b)
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

/// A 2.ª fileira: «Clips» · um chip por opção · «+».
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    f: &mut Fluxo,
    (snap, view): (&TimelineViewSnapshot, BarView),
    body: Rect,
) -> Clips {
    let ink = CardInk::of_list(theme);
    let chave = if view.tab == Tab::Keys {
        "panel.timeline.card.clips"
    } else {
        "panel.timeline.card.sources"
    };
    let opcoes = crate::transport_clips::source_options(snap, view.tab);
    let escolhida = crate::transport_clips::selected_source(snap, view);
    let mut out = Clips {
        chip: None,
        menu: None,
    };
    // ⭐ UMA fileira só: a Cena com muitos objectos animados chega a ~100 clipes, e chips em fileiras
    // sem fim comeriam a folha. O que não cabe vai para o «+N ▾», que abre a lista de sempre
    // (`TIMELINE_CLIP_DD`, com rolagem) — os mesmos ids, o mesmo clique.
    let pecas: Vec<(usize, Option<Chip<'_>>, f32)> = opcoes
        .iter()
        .enumerate()
        .map(|(i, opcao)| {
            if opcao.disabled || opcao.header {
                let w = ctx
                    .text_system
                    .prefix_width(&opcao.label, m::LABEL_PX)
                    .ceil();
                return (i, None, w);
            }
            let on = i == escolhida;
            let ch = Chip {
                opcao,
                on,
                estrela: comeca_com(snap, view, opcao),
                menu: tem_menu(on, opcao),
            };
            let w = largura(ctx, &opcao.label, ch.estrela, ch.menu);
            (i, Some(ch), w)
        })
        .collect();
    let mais = banda::icones_w(1) + m::FIELD_GAP;
    let tudo: f32 = pecas.iter().map(|p| p.2 + m::FIELD_GAP).sum::<f32>() + mais;
    // Depois do gravar (onda 1b) o grupo inteiro desce se não cabe no resto e cabe numa fileira.
    let nome = ph2d_i18n::tr(chave);
    let nome_w = ctx.text_system.prefix_width(nome, m::LABEL_PX).ceil();
    f.guarda_para(banda::SEPARADOR_W + m::FIELD_GAP + nome_w + m::FIELD_GAP + tudo);
    // O separador do Marker (onda 1c): nunca no início de uma fileira.
    if !f.no_inicio() {
        let (x, y) = f.poe(banda::SEPARADOR_W);
        let (store, hit) = ctx.host.store_and_hit_index_mut();
        CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w)
            .separador_em(x, y);
    }
    rotulo(ctx, ink, f, nome);
    let resta = f.resta();
    let texto_mais = |n: usize| ph2d_i18n::tr_with("panel.timeline.card.more", &[("n", &n)]);
    let mais_w = |ctx: &mut PaintCtx, n: usize| {
        ctx.text_system
            .prefix_width(&texto_mais(n), m::VALUE_PX)
            .ceil()
            + m::FIELD_PAD_R * 2.0
            + m::ICON_GLYPH
    };
    let reserva = mais + mais_w(ctx, opcoes.len()) + m::FIELD_GAP;
    let mut usado = 0.0;
    let mut escondidos = 0;
    let mut cortou = false;
    for (i, ch, w) in &pecas {
        // Depois do primeiro que não cabe, nenhum passa à frente: a ordem é a da lista.
        cortou |= tudo > resta && usado + w + m::FIELD_GAP + reserva > resta;
        if cortou {
            escondidos += usize::from(ch.is_some());
            continue;
        }
        usado += w + m::FIELD_GAP;
        let Some(ch) = ch else {
            rotulo(ctx, ink, f, &opcoes[*i].label);
            continue;
        };
        let (x, y) = f.poe(*w);
        let r = Rect::new(x, y, *w, m::FIELD_H);
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
    if escondidos > 0 {
        let w = mais_w(ctx, escondidos);
        let (x, y) = f.poe(w);
        let r = Rect::new(x, y, w, m::FIELD_H);
        let (store, hit) = ctx.host.store_and_hit_index_mut();
        let open = matches!(
            store.get(ids::TIMELINE_CLIP_DD),
            Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
        );
        let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w);
        c.lista_em(ids::TIMELINE_CLIP_DD, &texto_mais(escondidos), open, r);
        // A lista pende do «+N»; o campo de renomear, do chip escolhido se ele está à vista.
        if open || out.chip.is_none() {
            out.chip = Some(ClipChip {
                id: ids::TIMELINE_CLIP_DD,
                rect: r,
                open,
            });
        }
    }
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
    out
}
