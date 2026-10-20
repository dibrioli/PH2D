//! **As linhas do dope-sheet** — uma por faixa, ou UMA por grupo de canais (plano 35 §5.1).
//!
//! Os campos que se gravam juntos ([`ph2d_ecs::FieldKey::grupo`]: a cor inteira, a forma inteira)
//! chegam como uma faixa por campo — 6 para um preenchimento, 81 para uma estrela de 10 pontas. O
//! artista lê UMA linha («Star · Fill») com um losango por instante em que o grupo tem chave; o ▶
//! abre os membros, cada um com o seu nome («R», «Point X 3») e o seu editor de curva.
//!
//! Só o painel agrupa: o instantâneo continua uma faixa por ligação, e o losango do grupo agarra as
//! chaves dos MEMBROS (a coluna do Summary restrita ao grupo, [`Escopo`]), de modo que mover, apagar,
//! copiar e mudar a interpolação vão pelos intents de sempre. Um grupo de um só membro não ganha
//! cabeçalho: a faixa já tem o nome do grupo.

use std::collections::BTreeMap;

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{PropKind, SelectedKey, TimelineViewSnapshot, TrackView};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::graph::TimeView;
use crate::state::TimelinePanelState;
use crate::summary::columns_of;
use crate::tab::Tab;
use crate::tracks::{DIAMOND_H, KEY_HIT_HW, TWIRL_W, paint_diamond, paint_twirl, track_label};
use crate::{geom, ids};

/// O endereço DURÁVEL de um grupo de canais: de que objecto, e o [`ph2d_ecs::FieldKey::grupo`].
/// É o que o painel lembra como «aberto» — sobrevive a um membro apagado e à ordem das ligações.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrupoChave {
    pub entity: u64,
    pub familia: u16,
    pub grupo: u16,
    pub i: u16,
}

impl GrupoChave {
    /// O grupo desta faixa, ou `None` se ela não é um campo de família.
    pub(crate) fn de(t: &TrackView) -> Option<Self> {
        let PropKind::Field(k) = t.prop else {
            return None;
        };
        let (familia, grupo, i) = k.grupo()?;
        Some(Self {
            entity: t.entity,
            familia,
            grupo,
            i,
        })
    }

    /// O grupo de um alvo de rato: o `handle` é o `AnimTarget` de um membro qualquer.
    pub(crate) fn do_handle(snap: &TimelineViewSnapshot, handle: u64) -> Option<Self> {
        snap.tracks
            .iter()
            .find(|t| t.target.get() == handle)
            .and_then(Self::de)
    }
}

/// Que faixas uma coluna de losangos abrange: todas (o Summary) ou as de um grupo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Escopo {
    Tudo,
    Grupo(GrupoChave),
}

impl Escopo {
    pub(crate) fn contem(self, t: &TrackView) -> bool {
        match self {
            Self::Tudo => true,
            Self::Grupo(c) => GrupoChave::de(t) == Some(c),
        }
    }
}

/// Um grupo de dois ou mais membros, com os membros (índices em `snap.tracks`) na ordem de leitura:
/// contorno, ponto, campo — «Point X 1, Point Y 1, In X 1…».
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Grupo {
    pub chave: GrupoChave,
    pub membros: Vec<usize>,
}

impl Grupo {
    /// O handle que o alvo do rato carrega (o primeiro membro).
    pub(crate) fn handle(&self, snap: &TimelineViewSnapshot) -> u64 {
        snap.tracks[self.membros[0]].target.get()
    }
}

/// O que uma linha mostra: uma faixa (`grupo` = o grupo aberto a que ela pertence) ou o cabeçalho
/// de um grupo (índice em [`Layout::grupos`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Linha {
    Faixa { i: usize, grupo: Option<usize> },
    Grupo(usize),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Banda {
    pub linha: Linha,
    pub y: f32,
    pub h: f32,
}

/// ⭐ **A fonte única da disposição das linhas** — a pintura, o hit das chaves, o retângulo de
/// seleção e a altura do scroll leem-na, de modo que um losango nunca discorda da linha onde a
/// curva dele é desenhada.
#[derive(Debug, Default)]
pub(crate) struct Layout {
    pub bandas: Vec<Banda>,
    pub grupos: Vec<Grupo>,
}

/// As linhas deste separador, de cima para baixo, a partir do topo da banda das faixas (depois das
/// pistas de clips e do Summary). A aba Arrange não tem faixas.
pub(crate) fn layout(
    snap: &TimelineViewSnapshot,
    tab: Tab,
    expanded: &[u64],
    abertos: &[GrupoChave],
    graph_h: f32,
    rows_top: f32,
    scroll_y: f32,
) -> Layout {
    let mut out = Layout::default();
    if !tab.shows_keys() {
        return out;
    }
    let mut por_chave: BTreeMap<GrupoChave, usize> = BTreeMap::new();
    let mut de = vec![None; snap.tracks.len()];
    for (i, t) in snap.tracks.iter().enumerate() {
        let Some(c) = GrupoChave::de(t) else { continue };
        let g = *por_chave.entry(c).or_insert_with(|| {
            out.grupos.push(Grupo {
                chave: c,
                membros: Vec::new(),
            });
            out.grupos.len() - 1
        });
        out.grupos[g].membros.push(i);
        de[i] = Some(g);
    }
    for g in &mut out.grupos {
        g.membros.sort_by_key(|&m| match snap.tracks[m].prop {
            PropKind::Field(k) => (k.i, k.j, k.campo),
            _ => (0, 0, 0),
        });
    }
    let mut y = rows_top - scroll_y + geom::stack_h(snap, tab) + geom::summary_h(snap, tab);
    let mut posto = vec![false; out.grupos.len()];
    let faixa = |out: &mut Layout, y: &mut f32, i: usize, grupo| {
        let h = geom::row_h(expanded.contains(&snap.tracks[i].target.get()), graph_h);
        out.bandas.push(Banda {
            linha: Linha::Faixa { i, grupo },
            y: *y,
            h,
        });
        *y += h;
    };
    for (i, d) in de.iter().copied().enumerate() {
        match d.filter(|&g| out.grupos[g].membros.len() > 1) {
            None => faixa(&mut out, &mut y, i, None),
            Some(g) if !posto[g] => {
                posto[g] = true;
                out.bandas.push(Banda {
                    linha: Linha::Grupo(g),
                    y,
                    h: ROW_H_PX,
                });
                y += ROW_H_PX;
                if abertos.contains(&out.grupos[g].chave) {
                    for n in 0..out.grupos[g].membros.len() {
                        let m = out.grupos[g].membros[n];
                        faixa(&mut out, &mut y, m, Some(g));
                    }
                }
            }
            Some(_) => {}
        }
    }
    out
}

/// O nome de um membro dentro do grupo aberto: o do vocabulário, e o índice quando o mesmo campo
/// se repete (um por ponto, por paragem, por par do tracejado) — `contorno.ponto` se há mais de um
/// contorno.
pub(crate) fn rotulo_membro(snap: &TimelineViewSnapshot, g: &Grupo, i: usize) -> String {
    let campo = |m: usize| match snap.tracks[m].prop {
        PropKind::Field(k) => Some(k),
        _ => None,
    };
    let Some(k) = campo(i) else {
        return crate::tracks::prop_label(snap.tracks[i].prop).to_owned();
    };
    let nome = ph2d_i18n::tr(k.membro().unwrap_or("panel.timeline.prop.field"));
    let irmaos: Vec<_> = g
        .membros
        .iter()
        .filter_map(|&m| campo(m))
        .filter(|o| o.campo == k.campo)
        .collect();
    if irmaos.len() < 2 {
        nome.to_owned()
    } else if irmaos.iter().any(|o| o.i != k.i) {
        format!("{nome} {}.{}", k.i + 1, k.j + 1)
    } else {
        format!("{nome} {}", k.j + 1)
    }
}

/// O cabeçalho de um grupo: ▶, «Objecto · Grupo» e um losango por coluna dos membros (cheio quando
/// TODAS as chaves da coluna estão selecionadas). `linha` = `(grupo, y, aberto)`.
pub(crate) fn paint_grupo(
    ctx: &mut PaintCtx,
    theme: Theme,
    geo: &geom::Geom,
    view: TimeView,
    preview_dx: f32,
    snap: &TimelineViewSnapshot,
    linha: (&Grupo, f32, bool),
) {
    let (g, y, aberto) = linha;
    let x = geo.rows.x;
    let handle = g.handle(snap);
    paint_twirl(
        ctx,
        theme,
        aberto,
        (x, y),
        ids::timeline_group_twirl_id(handle),
        TimelineHitKind::GroupTwirl { group: handle },
    );
    let rep = &snap.tracks[g.membros[0]];
    let font = TypeToken::Sm.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &track_label(snap.object_name(rep.entity), rep.entity, rep.prop),
        x + TWIRL_W,
        y + (ROW_H_PX - font) * 0.5,
        font,
        (geo.label_w - TWIRL_W - Spacing::Xs.px()).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    let lane = crate::tracks::lane_rect(geo, view);
    let cy = y + ROW_H_PX * 0.5;
    for c in columns_of(g.membros.iter().map(|&m| &snap.tracks[m])) {
        let base = view.time_x + ((c.t_seconds - view.view_start) * view.px_per_s) as f32;
        let kx = if c.all_selected {
            base + preview_dx
        } else {
            base
        };
        if kx < view.time_x - DIAMOND_H || kx > view.right + DIAMOND_H {
            continue;
        }
        let tok = if c.all_selected {
            ColorToken::TimelineKeySelected
        } else {
            ColorToken::TimelineKey
        };
        paint_diamond(ctx, kx, cy, DIAMOND_H, resolve(tok, theme));
        let id = ids::timeline_group_key_id(handle, c.t_bits());
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::GroupKey {
                    group: handle,
                    t_bits: c.t_bits(),
                },
                canvas: lane,
            },
        );
        ctx.host.hit_index_mut().register(
            id,
            Rect::new(kx - KEY_HIT_HW, y, KEY_HIT_HW * 2.0, ROW_H_PX),
        );
    }
}

/// Every key whose diamond **centre** falls inside `sel` (a marquee in global px) — a group row
/// contributes every member key under its diamonds. Mirrors the row/column math of the painters
/// exactly: a key the user can see inside the box is a key that gets selected.
///
/// Rows scrolled out of `rows` never match: `sel` is intersected with the band first, so a marquee
/// dragged past the bottom edge cannot reach into the clipped rows below it.
pub(crate) fn keys_in_rect(
    rows: Rect,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    sel: Rect,
) -> Vec<SelectedKey> {
    let (top_y, bot_y) = (sel.y.max(rows.y), (sel.y + sel.h).min(rows.y + rows.h));
    let mut out = Vec::new();
    if top_y > bot_y {
        return out;
    }
    let lay = layout(
        snap,
        state.tab,
        &state.expanded,
        &state.grupos_abertos,
        state.graph_h,
        rows.y,
        state.scroll_y,
    );
    for b in &lay.bandas {
        // The diamonds sit on the row's dope-sheet STRIP, never in its graph band — a marquee
        // over the curve must not grab the keys under it.
        let cy = b.y + ROW_H_PX * 0.5;
        if cy < top_y || cy > bot_y {
            continue;
        }
        let faixas: &[usize] = match &b.linha {
            Linha::Faixa { i, .. } => std::slice::from_ref(i),
            Linha::Grupo(g) => &lay.grupos[*g].membros,
        };
        for &i in faixas {
            let track = &snap.tracks[i];
            for k in &track.keys {
                let kx = view.time_x + ((k.t_seconds - view.view_start) * view.px_per_s) as f32;
                if kx >= sel.x && kx <= sel.x + sel.w {
                    out.push(SelectedKey::new(track.target.get(), k.id.get()));
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "linhas_tests.rs"]
mod tests;
