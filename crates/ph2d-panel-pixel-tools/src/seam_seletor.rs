//! O seletor de cor do painel PIXEL na costura: o clique e o arrasto sobre as áreas pintadas dão a
//! cor (filho de [`crate::seam`] pelo tecto de LOC).

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::PixelRequest;

/// **Um clique numa área do seletor**: a posição do ponteiro no rect pintado dá a cor (esquerdo =
/// frente, direito = fundo, que guarda o alfa dele).
/// **O arrasto no seletor** (smoke do dono 25/10: «os sliders da roda não arrastam, apenas
/// clicam»): com o botão premido sobre o quadro ou uma barra, a cada quadro em que o ponteiro se
/// moveu a cor de frente segue-o — a mesma conta do clique, fora do rect presa à borda.
pub(crate) fn arrasta_seletor(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
) {
    let activo = host.store().active_id();
    let Some(id) = activo.filter(|a| ids::SELECTOR_AREAS.iter().any(|(p, _)| p == a)) else {
        st.seletor.arrasto = None;
        return;
    };
    let ponto = host.store().last_pointer();
    if ponto.is_none() || ponto == st.seletor.arrasto {
        return;
    }
    st.seletor.arrasto = ponto;
    if let Some(r) = escolhe(st, host, id) {
        host.bus_mut().push(EditorAction::Pixel(r));
    }
}

/// Quantas cores mandadas o seletor reconhece como suas (o atraso do instantâneo é de um quadro;
/// a folga cobre um arrasto rápido com vários quadros entre o pedido e o instantâneo).
const MANDADAS: usize = 8;

pub(crate) fn escolhe(
    st: &mut crate::PixelToolsPanelState,
    host: &dyn PanelHostInternal,
    id: NodeId,
) -> Option<PixelRequest> {
    use crate::seletor;
    let (k, fundo) = ids::SELECTOR_AREAS
        .iter()
        .enumerate()
        .find_map(|(k, (a, b))| {
            if *a == id {
                Some((k, false))
            } else if *b == id {
                Some((k, true))
            } else {
                None
            }
        })?;
    let r = st.seletor.rects[k]?;
    let [px, py] = host.store().last_pointer()?;
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let u = f64::from((px - r.x) / r.w).clamp(0.0, 1.0);
    let v = f64::from((py - r.y) / r.h).clamp(0.0, 1.0);
    let (modo, fg, bg, editing) = with_current(|i| (i.options.selector, i.fg, i.bg, i.editing));
    if !editing {
        return None;
    }
    let p = st.seletor.params;
    let [r0, g0, b0, a0] = fg;
    let (rgb, alfa, p) = match k {
        0 => {
            let p = seletor::no_quadro(modo, p, u, v);
            (seletor::cor(modo, p), a0, p)
        }
        1 => {
            let p = seletor::na_barra(modo, p, u);
            (seletor::cor(modo, p), a0, p)
        }
        _ => ([r0, g0, b0], seletor::alfa(u), p),
    };
    let [r, g, b] = rgb;
    if fundo {
        let a = if k == 2 { alfa } else { bg[3] };
        return Some(PixelRequest::Bg([r, g, b, a]));
    }
    let c = [r, g, b, alfa];
    st.seletor.params = p;
    st.seletor.de = Some((modo, c));
    let m = &mut st.seletor.mandadas;
    m.push(c);
    let excesso = m.len().saturating_sub(MANDADAS);
    m.drain(..excesso);
    Some(PixelRequest::Fg(c))
}
