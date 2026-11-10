//! **A CRT da Preview** (P13 V/U-CRT, `docs/PixelArt/P13_plano.md` §11–§12): preferência de VISTA
//! da sessão — nunca o documento, nunca o ficheiro. A janela do jogo lê a MESMA ([`vista`]).
//!
//! O passe é o `crt_cpu` da `ph2d-pixel-crt` (o gabarito que a placa iguala a 0 níveis), corrido só
//! quando muda `(seq, escala inteira, params)` — a Preview compõe só quando o quadro muda — e numa
//! escala ≤ a de caber na janela, o que prende a saída à área do corpo (o custo é por píxel de saída:
//! 25,7 ms em 4,2 MP ⇒ ~6 ns/px a 1 fio, passo 0b). Desligada, nada disto corre: a imagem é a de hoje.

use std::sync::Arc;

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::tr;
use ph2d_pixel_crt::{CURVATURA_MAXIMA, CrtParams, LADO_MAXIMO, crt_cpu};

use crate::{PixelPreviewPanelState, canal::PreviewFrame, ids};

/// A preferência da vista: ligada e os cinco números.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VistaCrt {
    pub ligada: bool,
    pub params: CrtParams,
}

/// Os params ao ligar (Scanlines, Mask, Curvature, Brightness, Bloom) e os passos do campo
/// (seta, arrasto por píxel, faixa) — dados da lei CRT, não medidas de UI.
#[rustfmt::skip]
const FABRICA: [f32; 5] = [0.5, 0.25, 0.1, 1.2, 0.15]; // LITERAL-PX-OK: params da lei CRT, não UI
#[rustfmt::skip]
const PASSOS: [f64; 3] = [0.05, 0.005, 0.01]; // LITERAL-PX-OK: passos de um campo de fracção, não UI

impl Default for VistaCrt {
    /// Desligada; ao ligar, um ecrã de TV que se lê (dado da lei, não medida de UI).
    fn default() -> Self {
        Self {
            ligada: false,
            params: CrtParams {
                scanline: FABRICA[0],
                mascara: FABRICA[1],
                curvatura: FABRICA[2],
                brilho: FABRICA[3],
                bloom: FABRICA[4],
            },
        }
    }
}

thread_local! {
    static VISTA: std::cell::Cell<VistaCrt> = std::cell::Cell::new(VistaCrt::default());
}

/// A CRT da vista agora (a Preview e a janela do jogo).
#[must_use]
pub fn vista() -> VistaCrt {
    VISTA.with(std::cell::Cell::get)
}

/// Os params que um leitor aplica: `None` = desligada (o leitor salta a chamada).
#[must_use]
pub fn params_ligados() -> Option<CrtParams> {
    let v = vista();
    v.ligada.then_some(v.params)
}

pub fn define(v: VistaCrt) {
    VISTA.with(|c| c.set(v));
}

const CHAVES: [&str; 5] = [
    "panel.pixel.preview.crt.scanlines",
    "panel.pixel.preview.crt.mask",
    "panel.pixel.preview.crt.curvature",
    "panel.pixel.preview.crt.brightness",
    "panel.pixel.preview.crt.bloom",
];

/// O tecto de cada número, com o recurso: scanline/máscara/bloom são fracções; a curvatura é o tecto
/// de `i32` da lei Q14; o brilho 2 já satura um nível 128.
const TECTOS: [f32; 5] = [1.0, 1.0, CURVATURA_MAXIMA, 2.0, 1.0];

fn valores(p: &CrtParams) -> [f32; 5] {
    [p.scanline, p.mascara, p.curvatura, p.brilho, p.bloom]
}

pub(crate) fn populate(store: &mut WidgetStore) {
    let fabrica = valores(&VistaCrt::default().params);
    for (i, id) in ids::CRT_CAMPOS.into_iter().enumerate() {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: f64::from(fabrica[i]),
                buffer: String::new(),
                caret: 0,
                last_committed: f64::from(fabrica[i]),
                selection_anchor: None,
            },
        );
        let spec = CardFieldSpec {
            step: PASSOS[0],
            scrub: PASSOS[1],
            default: Some(f64::from(fabrica[i])),
        };
        store.mark_card_field(id, spec);
        store.set_number_range(id, 0.0, f64::from(TECTOS[i]), PASSOS[2]);
    }
}

/// Um número do cartão mudou: a vista recebe-o, preso ao tecto.
pub(crate) fn numero(store: &WidgetStore, id: ph2d_a11y::NodeId) -> bool {
    let Some(i) = ids::CRT_CAMPOS.iter().position(|c| *c == id) else {
        return false;
    };
    let Some(n) = store.number_value(id) else {
        return true;
    };
    #[expect(clippy::cast_possible_truncation, reason = "params da CRT ∈ [0, 2]")]
    let n = (n as f32).clamp(0.0, TECTOS[i]);
    let mut v = vista();
    let p = &mut v.params;
    *[
        &mut p.scanline,
        &mut p.mascara,
        &mut p.curvatura,
        &mut p.brilho,
        &mut p.bloom,
    ][i] = n;
    define(v);
    true
}

pub(crate) fn alterna() {
    let mut v = vista();
    v.ligada = !v.ligada;
    define(v);
}

/// Escreve os números da vista nos campos (salvo o que está a ser arrastado).
pub(crate) fn prepara(store: &mut WidgetStore) {
    let arrasto = store.number_input_drag().map(|d| d.id);
    for (id, v) in ids::CRT_CAMPOS.into_iter().zip(valores(&vista().params)) {
        if arrasto != Some(id) {
            store.set_number_value(id, f64::from(v));
        }
    }
}

/// O cartão na faixa `x..x+w` a partir de `y`, com os números já escritos. Devolve o fim em `y`.
pub(crate) fn pinta_cartao(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    prepara(ctx.host.store_mut());
    let ink = ph2d_editor_core::widget::card_ink::CardInk::of_list(ctx.host.theme());
    let (store, hits) = ctx.host.store_and_hit_index_mut();
    let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hits, ink, x, w);
    cartao(&mut c, y)
}

/// O cartão: a caixa CRT e, ligada, os cinco números aos pares. Devolve o fim em `y`.
pub(crate) fn cartao(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let v = vista();
    let rotulo = Rotulo::vista(TextKey::new("panel.pixel.preview.crt"));
    let mut y = c.check(ids::CRT, rotulo, v.ligada, y);
    if !v.ligada {
        return y;
    }
    let cells: Vec<Cell<'_>> = ids::CRT_CAMPOS
        .into_iter()
        .zip(CHAVES)
        .map(|(id, k)| {
            Cell::new(Widget::Field {
                id,
                glyph: Glyph::Text(""),
            })
            .label(tr(k))
        })
        .collect();
    for par in cells.chunks(2) {
        y = c.row(par, y);
    }
    y
}

/// A imagem que a Preview mostra: a publicada (desligada) ou a da CRT na escala inteira `k`.
pub(crate) fn imagem(
    st: &mut PixelPreviewPanelState,
    f: &PreviewFrame,
    escala: f32,
) -> Option<ph2d_vector::StableImage> {
    let Some(p) = params_ligados() else {
        st.crt = None;
        st.mostrada = Some(f.rgba.clone());
        return st.imagem.as_ref().map(|(_, i)| i.clone());
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "escala ≥ 1"
    )]
    let k = (escala.floor().max(1.0) as u32).min(LADO_MAXIMO / f.w.max(f.h).max(1));
    let k = k.max(1);
    let chave = (f.seq, k, valores(&p).map(f32::to_bits));
    if st.crt.as_ref().is_none_or(|(c, _, _)| *c != chave) {
        let rgba = Arc::new(crt_cpu(&f.rgba, f.w, f.h, k, &p));
        let img = ph2d_vector::StableImage::from_rgba(rgba.clone(), f.w * k, f.h * k)?;
        st.passes_crt += 1;
        st.crt = Some((chave, img, rgba));
    }
    let (_, img, rgba) = st.crt.as_ref()?;
    st.mostrada = Some(rgba.clone());
    Some(img.clone())
}
