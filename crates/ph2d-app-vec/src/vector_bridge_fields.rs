//! ⭐ **Campo a campo** — a escrita e a leitura da seleção múltipla (plano 31,
//! `docs/UI_New_and_Simple/vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md`).
//!
//! Módulo irmão do [`super::style`] (teto de LOC). Duas metades da MESMA máscara
//! ([`StyleFields`]):
//!
//! - **escrever** ([`StrokeStyle::onto_fields`], [`fill_merge`], [`restyle_selected_fields`]): um
//!   gesto escreve em cada forma selecionada só os campos que o artista mexeu. ⛔ Antes a ponte
//!   gravava o estilo INTEIRO da tool (= o do primário) em todas: mexer na Width de uma forma
//!   vermelha e de uma azul pintava as duas de vermelho.
//! - **ler** ([`mixed_of`]): os campos em que as selecionadas diferem, que o painel pinta como
//!   «misto». ⛔ Revoga a lei «mostra o primário» do `appearance.rs` (decisão do dono, 18/10): o
//!   Inspector do mesmo app já mostrava «misto», e o primário mentia sobre as outras.

use super::style::{StrokeStyle, recolour_stroke_paint, seed_fill_from_paint};
use ph2d_tool_vector::StyleFields as F;
use ph2d_tool_vector::params::DASH_DEFAULT;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{DashPattern, MAX_DASH_PAIRS, Paint, Rgba8, StrokeSpec, VecPathId, VecScene};

/// Os pares do tracejado de um traço — a MESMA leitura que a tool faz ao adotar (`adopt_stroke`).
fn pairs_of(d: Option<DashPattern>) -> [[f64; 2]; MAX_DASH_PAIRS] {
    d.map_or(DASH_DEFAULT, |p| p.pairs)
}

/// O RGB e a alfa de `new` onde `campos` os pede; o resto de `cur`.
fn merge_rgba(cur: Rgba8, new: Rgba8, rgb: F, a: F, campos: F) -> Rgba8 {
    let (r, g, b) = if campos.has(rgb) {
        (new.r, new.g, new.b)
    } else {
        (cur.r, cur.g, cur.b)
    };
    Rgba8::new(r, g, b, if campos.has(a) { new.a } else { cur.a })
}

impl StrokeStyle {
    /// **O `StrokeSpec` de `old` com SÓ os `campos` desta ficha escritos** — o resto do traço
    /// sobrevive. É detector e escritor de uma vez: `onto_fields(s, …) != *s` responde «isto muda
    /// alguma coisa?», então os dois lados não podem divergir num campo esquecido.
    ///
    /// A cor entra DENTRO da tinta que lá está (sólido, padrão ou pincel — a lei do [`Self::onto`]),
    /// e a largura só se `campos` tem `WIDTH`.
    #[must_use]
    pub fn onto_fields(&self, old: &StrokeSpec, width: f64, campos: F) -> StrokeSpec {
        let paint = if campos.any(F::STROKE_RGB | F::STROKE_A) {
            let c = merge_rgba(old.color(), self.color, F::STROKE_RGB, F::STROKE_A, campos);
            recolour_stroke_paint(&old.paint, c)
        } else {
            old.paint.clone()
        };
        let mut pairs = pairs_of(old.dash);
        let novos = pairs_of(self.dash);
        for (k, (par, novo)) in pairs.iter_mut().zip(novos).enumerate() {
            for (gap, (v, n)) in par.iter_mut().zip(novo).enumerate() {
                if campos.has(F::dash(k, gap == 1)) {
                    *v = n;
                }
            }
        }
        let dash = DashPattern { pairs };
        let pick = |f: F| campos.has(f);
        StrokeSpec {
            paint,
            width: if pick(F::WIDTH) { width } else { old.width },
            cap: if pick(F::CAP) { self.cap } else { old.cap },
            join: if pick(F::JOIN) { self.join } else { old.join },
            align: if pick(F::ALIGN) {
                self.align
            } else {
                old.align
            },
            dash: if campos.any(dash_bits()) {
                dash.is_dashed().then_some(dash)
            } else {
                old.dash
            },
            marker_start: if pick(F::MARKER_START) {
                self.marker_start
            } else {
                old.marker_start
            },
            marker_end: if pick(F::MARKER_END) {
                self.marker_end
            } else {
                old.marker_end
            },
            marker_scale_start: if pick(F::MARKER_SCALE_START) {
                self.marker_scale[0]
            } else {
                old.marker_scale_start
            },
            marker_scale_end: if pick(F::MARKER_SCALE_END) {
                self.marker_scale[1]
            } else {
                old.marker_scale_end
            },
            marker_round: if pick(F::MARKER_ROUND) {
                self.marker_round
            } else {
                old.marker_round
            },
        }
    }
}

/// Os seis bits do tracejado.
fn dash_bits() -> F {
    let mut f = F::NONE;
    for k in 0..MAX_DASH_PAIRS {
        f |= F::dash(k, false) | F::dash(k, true);
    }
    f
}

/// **A cor de preenchimento que um gesto pede para ESTA forma** — o RGB e/ou a alfa novos sobre a
/// cor que ela tem, lida pela MESMA porta que a semente usa ([`seed_fill_from_paint`]). O
/// resultado vai ao `apply_fill_colour`, onde mora a lei da espécie de tinta. Um gradiente (sem cor
/// a ler) recebe a cor nova crua, e o `apply_fill_colour` deixa-o quieto.
#[must_use]
pub fn fill_merge(cur: Option<&Paint>, new: Rgba8, campos: F) -> Rgba8 {
    let c = seed_fill_from_paint(cur).map_or(new, |c| Rgba8::new(c[0], c[1], c[2], c[3]));
    merge_rgba(c, new, F::FILL_RGB, F::FILL_A, campos)
}

/// **Reestiliza SÓ os `campos` do traço de cada caminho selecionado.** Um caminho sem traço fica
/// como está (ganhar um traço do nada seria a UI a inventar geometria). Devolve quantos traços
/// foram reescritos.
pub fn restyle_selected_fields(
    scene: &mut VecScene,
    selection: &[VecPathId],
    style: &StrokeStyle,
    width: f64,
    campos: F,
) -> usize {
    let mut n = 0;
    for &id in selection {
        let Some(path) = scene.path_mut(id) else {
            continue;
        };
        let Some(old) = path.stroke.as_ref() else {
            continue;
        };
        path.stroke = Some(style.onto_fields(old, width, campos));
        n += 1;
    }
    n
}

/// **Os campos em que as formas selecionadas DIFEREM** — vazio com menos de duas. O preenchimento
/// conta só as formas FECHADAS (as únicas que a amostra Fill pinta) e o traço só as que têm traço.
#[must_use]
pub fn mixed_of(pen: &PenTool, scene: &VecScene) -> F {
    let sel = pen.selected_paths();
    if sel.len() < 2 {
        return F::NONE;
    }
    let paths: Vec<_> = sel.iter().filter_map(|&id| scene.path(id)).collect();
    let mut mixed = F::NONE;
    let fills: Vec<_> = paths
        .iter()
        .filter(|p| p.closed)
        .map(|p| seed_fill_from_paint(p.fill.as_ref()))
        .collect();
    if let Some(first) = fills.first() {
        for f in &fills[1..] {
            match (first, f) {
                (Some(a), Some(b)) => {
                    if a[..3] != b[..3] {
                        mixed |= F::FILL_RGB;
                    }
                    if a[3] != b[3] {
                        mixed |= F::FILL_A;
                    }
                }
                (None, None) => {}
                // Um gradiente ao lado de uma cor: a amostra não tem UMA cor a mostrar.
                _ => mixed |= F::FILL,
            }
        }
    }
    let strokes: Vec<&StrokeSpec> = paths.iter().filter_map(|p| p.stroke.as_ref()).collect();
    if let Some(a) = strokes.first() {
        for b in &strokes[1..] {
            mixed |= stroke_diff(a, b);
        }
    }
    mixed
}

/// Os campos do traço em que `a` e `b` diferem.
pub(super) fn stroke_diff(a: &StrokeSpec, b: &StrokeSpec) -> F {
    let (ca, cb) = (a.color(), b.color());
    let mut f = F::NONE;
    let mut se = |cond: bool, bit: F| {
        if cond {
            f |= bit;
        }
    };
    se((ca.r, ca.g, ca.b) != (cb.r, cb.g, cb.b), F::STROKE_RGB);
    se(ca.a != cb.a, F::STROKE_A);
    se((a.width - b.width).abs() > 1e-9, F::WIDTH);
    se(a.cap != b.cap, F::CAP);
    se(a.join != b.join, F::JOIN);
    se(a.align != b.align, F::ALIGN);
    se(a.marker_start != b.marker_start, F::MARKER_START);
    se(a.marker_end != b.marker_end, F::MARKER_END);
    se(
        (a.marker_scale_start - b.marker_scale_start).abs() > 1e-9,
        F::MARKER_SCALE_START,
    );
    se(
        (a.marker_scale_end - b.marker_scale_end).abs() > 1e-9,
        F::MARKER_SCALE_END,
    );
    se(
        (a.marker_round - b.marker_round).abs() > 1e-9,
        F::MARKER_ROUND,
    );
    for (k, (pa, pb)) in pairs_of(a.dash).iter().zip(pairs_of(b.dash)).enumerate() {
        se((pa[0] - pb[0]).abs() > 1e-9, F::dash(k, false));
        se((pa[1] - pb[1]).abs() > 1e-9, F::dash(k, true));
    }
    f
}

thread_local! {
    /// O «misto» do quadro anterior — para devolver o número à caixa no quadro em que a seleção
    /// volta a concordar.
    static LAST_MIXED: std::cell::Cell<F> = const { std::cell::Cell::new(F::NONE) };
}

/// **As caixas numéricas em «misto» ficam em BRANCO** (o BulkSelect do Inspector: o valor e o
/// último gravado continuam os do primário, então focar e sair sem digitar não escreve nada). O
/// slider ao lado fica no primário: o arrasto parte de um sítio previsível. Quando a seleção volta
/// a concordar, a caixa recebe o número outra vez.
pub fn blank_mixed_numbers(
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    tool: &ph2d_tool_vector::VectorTool,
    mixed: F,
) {
    let antes = LAST_MIXED.with(|c| c.replace(mixed));
    if mixed.has(F::WIDTH) {
        store.blank_number_input(ph2d_tool_vector::ids::VECTOR_WIDTH_NUM);
    } else if antes.has(F::WIDTH) {
        store.set_number_value(
            ph2d_tool_vector::ids::VECTOR_WIDTH_NUM,
            tool.stroke_width_px(),
        );
    }
}
