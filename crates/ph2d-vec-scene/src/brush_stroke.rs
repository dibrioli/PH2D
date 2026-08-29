//! ⭐⭐⭐ **O PINCEL DE CONTORNO** (plano 36, W2) — a arte que PERCORRE a linha, em vez da tinta que
//! ela revela.
//!
//! # Os dois modelos, e por que este ficheiro existe ao lado do padrão
//!
//! O [`crate::StrokePaint::Pattern`] é **normativo em SVG 2**: um traço com paint server é a
//! silhueta dele PREENCHIDA, então um tracejado são **buracos** no papel de parede e a arte não os
//! conhece. Isto é o outro modelo — o *Pattern Brush* do Illustrator —, e ele responde ao contrário
//! às MESMAS três perguntas: escala com a largura, reinicia em cada traço, e tem quinas.
//!
//! # ⭐ O motor já existia, e está pago
//!
//! [`crate::pattern_along`] (plano 23) copia um motivo ao longo de um [`ArcPath`], cada cópia
//! rodada para a tangente — **0,597 ms para 200 cópias × 40 vértices**, ~13× de folga sob o *kill*.
//! O que faltava não era o motor: era **endereçá-lo como uma propriedade do traço** em vez de uma
//! relação entre dois objectos.
//!
//! # ⚠️ A guia é o contorno COZIDO, e a lei é a mesma do traço
//!
//! Quem desenha o traço percorre a geometria **viva** (cantos vivos, largura viva, booleana). O
//! pincel tem de correr sobre a mesma — senão a arte anda por um caminho que ninguém vê.

use crate::arc_path::ArcPath;
use crate::pattern_path::{PatternSpec, pattern_along};
use crate::{BrushStroke, Contour, VecPath};

/// **A ALTURA que uma cópia recebe** — derivada da largura da faixa, e multiplicada pelo `scale`.
///
/// ⚠️⚠️ **É aqui que os dois modelos divergem, e é deliberado.** O plano 35 §2.3 fixou que uma
/// TINTA **não** escala com a largura (*"a largura decide a faixa; o padrão decide o que a
/// preenche"*) — a queixa clássica do Illustrator, do lado certo. Um pincel é o oposto **porque ele
/// É a faixa**: engrossar o traço engrossa a arte, que é o que o *Pattern Brush* faz.
#[must_use]
pub fn brush_height(b: &BrushStroke, width: f64) -> f64 {
    let h = width * b.scale;
    if h.is_finite() && h > 0.0 { h } else { 0.0 }
}

/// A arte **escalada** para a altura da faixa, centrada em zero.
///
/// ⚠️ **Um factor ÚNICO nos dois eixos.** Escalar só a altura esmagaria o motivo, e um pincel que
/// deforma a arte é o *Art Brush*, não o *Pattern Brush* — outra ferramenta, com outro nome, que o
/// plano 36 §3.4 deixa fora de propósito.
///
/// `None` quando a arte não tem altura que se meça (um ponto, um caminho vazio): não há factor
/// honesto, e desenhar nada é melhor que dividir por quase-zero.
#[must_use]
fn art_at_height(art: &VecPath, h: f64) -> Option<VecPath> {
    // ⚠️ `<=` e não `!(_ > _)`: os dois recusam o NaN, e é a forma que o `dash_fit`
    // desta crate já escolheu — *duas formas de recusar a mesma coisa lêem-se como duas leis*.
    if h <= 0.0 || h.is_nan() {
        return None;
    }
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in art.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    let alt = hi[1] - lo[1];
    if !seen || alt <= 0.0 || alt.is_nan() {
        return None;
    }
    let k = h / alt;
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let map = |p: [f64; 2]| [(p[0] - c[0]) * k, (p[1] - c[1]) * k];
    let mut out = art.clone();
    let escala = |verts: &mut Vec<crate::VecVertex>| {
        for v in verts.iter_mut() {
            v.anchor = map(v.anchor);
            v.in_handle = map(v.in_handle);
            v.out_handle = map(v.out_handle);
            // ⚠️ **O raio de quina é um COMPRIMENTO LOCAL** e escala junto — esquecê-lo faria a
            // quina viva de uma cópia grande ficar do tamanho da de uma pequena.
            v.corner_radius *= k;
        }
    };
    escala(&mut out.verts);
    for c in &mut out.subpaths {
        escala(&mut c.verts);
    }
    Some(out)
}

/// ⭐⭐ **AS CÓPIAS que um pincel põe sobre `guia`** — a porta única do modelo B.
///
/// - `guia` é o contorno **cozido** da forma (o que o traço de facto percorre);
/// - `art` é a forma que se repete;
/// - `width` é a largura do traço, de que a altura da arte é derivada.
///
/// ⚠️ **O encaixe é LIGADO** (`fit_to_guide`): num contorno fechado, a cauda que sobra é um vão
/// encostado a uma cópia inteira **sempre na mesma quina** — o defeito que o `dash_fit` já curou
/// para o tracejado, e a mesma porta o cura aqui.
///
/// Vazio quando não há o que desenhar (arte degenerada, contorno sem comprimento, largura zero) —
/// e quem chama pinta a **cor de recurso**, que é desenho certo, não desistência.
#[must_use]
pub fn brush_copies(guia: &Contour, art: &VecPath, b: &BrushStroke, width: f64) -> Vec<VecPath> {
    let Some(escalada) = art_at_height(art, brush_height(b, width)) else {
        return Vec::new();
    };
    let Some(arc) = ArcPath::from_contour(&guia.verts, guia.closed) else {
        return Vec::new();
    };
    pattern_along(
        &escalada,
        &arc,
        &PatternSpec {
            spacing: b.spacing,
            offset: b.offset,
            flip: b.flip,
            rotation_deg: b.rotation_deg,
            fit_to_guide: true,
            ..PatternSpec::default()
        },
    )
}

/// **Todas as cópias de um pincel sobre TODOS os contornos de `path`.**
///
/// ⚠️ **Contorno a contorno, e não o mais longo.** O `dash_fit` escolhe o contorno mais longo
/// porque o traçador recebe **um** par `[traço, vão]` para o caminho inteiro; aqui não há essa
/// restrição — cada contorno recebe as suas cópias e **fecha exactamente**, o que é estritamente
/// melhor. *Uma limitação herdada sem se perguntar se ela ainda existe é uma limitação inventada.*
#[must_use]
pub fn brush_along_path(
    path: &VecPath,
    art: &VecPath,
    b: &BrushStroke,
    width: f64,
) -> Vec<VecPath> {
    let mut out = Vec::new();
    let principal = Contour {
        verts: path.verts.clone(),
        closed: path.closed,
    };
    for c in std::iter::once(&principal).chain(path.subpaths.iter()) {
        out.extend(brush_copies(c, art, b, width));
    }
    out
}

#[cfg(test)]
#[path = "brush_stroke_engine_tests.rs"]
mod tests;
