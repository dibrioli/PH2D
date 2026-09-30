//! ⭐⭐⭐ **A SILHUETA DE UMA FORMA QUE SE SOBREPÕE A SI MESMA** — o contacto resolvido pela regra
//! do preenchimento.
//!
//! # O defeito, medido
//!
//! Numa dobra forte (`100°`–`150°`) a parte de DENTRO de dois membros presos a ossos passa uma por
//! cima da outra — é geometria, não erro da lei: dois pedaços rígidos que rodam em torno de uma
//! junta sobrepõem-se do lado côncavo assim que `tan(θ/2)` passa a distância da junta ao início do
//! pedaço rígido sobre a meia-espessura. O PREENCHIMENTO (regra não-zero) pinta a região certa —
//! a união dos dois membros, com um canto em «V» no cotovelo —, e o TRAÇO desenha o contorno
//! inteiro, **com o «olho» da sobreposição por dentro**. Medido na barra da cena, dobra em S
//! (`skinned_mesh_arap_sonda_tests`): o contorno cruza-se `2` vezes de `100°` a `150°` e a
//! silhueta tem `0` cruzamentos, com a área do preenchimento intacta onde não há contacto.
//!
//! ⭐ **A imagem presa já faz isto de graça** (um membro desenha-se por cima do outro, como num
//! boneco de recorte, e o canto fica em «V») — esta porta é o que põe o desenho vectorial a
//! concordar com ela.
//!
//! # O estado da arte, e porque é ESTA a resposta
//!
//! O que a literatura faz no contacto de um cotovelo é pôr a pele na FRONTEIRA DA UNIÃO dos
//! membros (*Implicit Skinning*, Vaillant et al. 2013: cada vértice é projectado na iso-superfície
//! da composição dos campos por osso, e o contacto vira um vinco). Para um caminho vectorial a
//! fronteira da união **é exacta e barata**: é a união do caminho com o VAZIO, pelo motor que esta
//! crate já tem — cúbicas de verdade, sem malha nenhuma.
//!
//! ⛔ **Duas curas foram construídas, medidas e RECUSADAS antes desta** (sonda citada acima):
//! - **ARAP onde a lei esmaga** (Sorkine–Alexa 2007, a família do Plastic do OpenToonz): desfaz os
//!   triângulos virados e **cria** laços a `90°` — ela resiste à compressão, e o lado de dentro de
//!   um cotovelo TEM de comprimir.
//! - **A menor correcção sem inversão** (barreira sobre `det J`, à la IPC/Garanzha 2021): zero
//!   triângulos virados em toda a dobra, e o contorno continua a cruzar-se — **o defeito não é a
//!   pele virar do avesso, é o contacto**. Acrescentar a barreira do ângulo da borda trava o
//!   optimizador (colapsa triângulos a área zero).
//!
//! # A lei desta porta
//!
//! - **Só corre quando o contorno se cruza** ([`crosses_itself`]) — fora do contacto a forma sai
//!   **byte-idêntica**, e o custo é o de uma varredura de segmentos.
//! - **O estilo é o da forma** (preenchimento, traço, opacidade, mistura, camadas); só a geometria
//!   muda. ⚠️ Não é o `compound_from` da booleana, que devolve o estilo MÍNIMO de um resultado
//!   de edição: aqui a forma é a mesma, desenhada.
//! - **Um caminho ABERTO não tem silhueta** (não há interior) ⇒ `None`, e quem chama desenha-o
//!   como sempre.

use kurbo::{BezPath, PathEl, Shape};
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{Contour, FillRule, VecPath};

/// ⭐ **A tolerância do achatamento na detecção, em fracção da diagonal da forma** — `1e-4`.
///
/// ⚠️ Ela decide só se a varredura CORRE, nunca a geometria que sai (essa é exacta, do motor). Uma
/// corda de uma curva sem auto-intersecção só cruza outra quando os dois fios estão a menos da
/// tolerância um do outro — e aí o contacto existe à escala do desenho.
pub const DETECTION_TOLERANCE: f64 = 1e-4;

/// ⭐⭐⭐ **A silhueta de `path`** quando ele se sobrepõe a si mesmo; `None` quando não se
/// sobrepõe, é aberto, ou o motor recusa (e aí quem chama desenha a forma como estava).
#[must_use]
pub fn resolve_overlap(path: &VecPath) -> Option<VecPath> {
    if !path.closed || path.subpaths.iter().any(|c| !c.closed) {
        return None;
    }
    let bez = crate::to_bez(path);
    if !crosses_itself(&bez) {
        return None;
    }
    let rule = match path.fill_rule {
        FillRule::NonZero => LsFillRule::NonZero,
        FillRule::EvenOdd => LsFillRule::EvenOdd,
    };
    // ⭐ `A ∪ ∅` e NÃO `A ∪ A` — a regra de multiplicidade do `linesweeper` 0.4 (ver
    // `expand.rs`, `Region::of`): uma aresta com multiplicidade par não se dissolve.
    let groups = crate::binary_grouped(&bez, &BezPath::new(), rule, BinaryOp::Union)?;
    let mut contornos = groups.iter().flatten().filter_map(crate::verts_from_bez);
    let outer = contornos.next()?;
    let resto: Vec<Contour> = contornos.map(Contour::new_closed).collect();
    let mut out = path.clone();
    out.verts = outer;
    out.closed = true;
    // ⚠️ Os contornos do motor saem ORIENTADOS (de fora primeiro, os buracos dentro), logo as
    // duas regras concordam; `EvenOdd` é a que não depende da orientação de nenhum deles.
    out.fill_rule = if resto.is_empty() {
        FillRule::NonZero
    } else {
        FillRule::EvenOdd
    };
    out.subpaths = resto;
    Some(out)
}

/// ⭐⭐ **O contorno cruza-se?** — os contornos achatados, e um par de segmentos que se ATRAVESSA.
/// Poda por varredura em `x`, para não pagar o quadrado inteiro.
///
/// ⚠️ **Não há salto de vizinhos, e não é descuido:** dois segmentos consecutivos partilham o
/// extremo com os MESMOS bits, logo um dos quatro testes de lado subtrai um ponto de si mesmo e dá
/// **zero exacto** — e o teste estrito de [`atravessa`] recusa-o. Um salto explícito foi escrito e
/// uma mutação que o apagava SOBREVIVEU: *uma linha que a mutação não consegue matar não é lei.*
#[must_use]
pub fn crosses_itself(bez: &BezPath) -> bool {
    let caixa = bez.bounding_box();
    let diag = caixa.width().hypot(caixa.height());
    if !diag.is_finite() || diag <= 0.0 {
        return false;
    }
    let mut segs: Vec<([f64; 2], [f64; 2])> = Vec::new();
    let (mut ini, mut ult) = ([0.0; 2], [0.0; 2]);
    let mut aberto = false;
    let fecha = |segs: &mut Vec<_>, ult: [f64; 2], ini: [f64; 2]| {
        if ult != ini {
            segs.push((ult, ini));
        }
    };
    kurbo::flatten(bez.iter(), diag * DETECTION_TOLERANCE, |el| match el {
        PathEl::MoveTo(p) => {
            if aberto {
                fecha(&mut segs, ult, ini);
            }
            aberto = true;
            ini = [p.x, p.y];
            ult = ini;
        }
        PathEl::LineTo(p) => {
            let q = [p.x, p.y];
            if q != ult {
                segs.push((ult, q));
                ult = q;
            }
        }
        PathEl::ClosePath => {
            if aberto {
                fecha(&mut segs, ult, ini);
                aberto = false;
            }
        }
        // O `flatten` só emite rectas.
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
    });
    if aberto {
        fecha(&mut segs, ult, ini);
    }
    segs.sort_by(|a, b| a.0[0].min(a.1[0]).total_cmp(&b.0[0].min(b.1[0])));
    for (i, s) in segs.iter().enumerate() {
        let xmax = s.0[0].max(s.1[0]);
        for t in &segs[i + 1..] {
            if t.0[0].min(t.1[0]) > xmax {
                break;
            }
            if atravessa(s.0, s.1, t.0, t.1) {
                return true;
            }
        }
    }
    false
}

/// Dois segmentos atravessam-se de verdade (os extremos de cada um em lados estritamente opostos
/// do outro). ⚠️ Um toque num extremo NÃO conta — dois contornos que se tocam num ponto não
/// pedem silhueta.
fn atravessa(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> bool {
    let lado = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
        (u[0] - o[0]).mul_add(v[1] - o[1], -((v[0] - o[0]) * (u[1] - o[1])))
    };
    let (d1, d2) = (lado(a1, a2, b1), lado(a1, a2, b2));
    let (d3, d4) = (lado(b1, b2, a1), lado(b1, b2, a2));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

#[cfg(test)]
#[path = "overlap_tests.rs"]
mod tests;
