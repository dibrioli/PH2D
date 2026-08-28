//! **ASSAR UM AFIM na geometria de um caminho** — módulo irmão de [`super::path_ops`] pelo teto de
//! LOC, e o corte é por RESPONSABILIDADE: o `path_ops` **move e mede** um caminho no frame dele;
//! aqui um afim entra na geometria e o frame **desaparece**.
//!
//! ⚠️ É o que reconcilia frames diferentes antes de uma operação de geometria (booleana, merge,
//! offset): os operandos vêm de entidades com `Transform` distintos, e um resultado só pode viver
//! num frame.

use crate::{Paint, VecPath};

/// **Assa** o afim `x` na geometria de `path`: âncoras, handles e a geometria do
/// gradiente passam a estar no frame de destino.
///
/// É o que reconcilia frames diferentes antes de uma operação de geometria
/// (booleana, merge, offset): os operandos vêm de entidades com `Transform`
/// distintos, e um resultado só pode viver num frame. Assando os operandos no
/// MUNDO, o resultado nasce em world-space — e a entidade nova dele, na identidade,
/// o desenha exatamente onde as formas de origem estavam.
///
/// Identidade é no-op.
pub fn bake_xform(path: &mut VecPath, x: &crate::Xform) {
    if x.is_identity() {
        return;
    }
    let f = |p: [f64; 2]| x.apply(p);
    path.for_each_vert_mut(|v| {
        v.anchor = f(v.anchor);
        v.in_handle = f(v.in_handle);
        v.out_handle = f(v.out_handle);
    });
    transform_fill_geometry(path, f, x.mean_scale());
}

/// Aplica a transformação de ponto `f` (a MESMA das âncoras) à geometria world-space
/// do gradiente do fill, e escala por `radius_scale` **todo comprimento escalar do
/// path**: o raio do gradiente radial e o `corner_radius` de cada vértice.
///
/// Os dois são a mesma espécie de quantidade — **um comprimento que não é um ponto** —
/// e é por isso que moram na mesma função. Um op que transforma a geometria transforma
/// os pontos com `f` e os comprimentos com `radius_scale`; se os dois comprimentos
/// vivessem em funções separadas, o próximo op novo escalaria um e esqueceria o outro,
/// e a quina de uma forma escalada arredondaria errado sem nada ficar vermelho.
///
/// Sob escala NÃO-uniforme um raio escalar é indefinido (a quina viraria elíptica); o
/// fator médio dos eixos é a mesma aproximação que o gradiente radial já faz.
/// No-op para `Solid` / sem fill.
pub(crate) fn transform_fill_geometry(
    path: &mut VecPath,
    f: impl Fn([f64; 2]) -> [f64; 2],
    radius_scale: f64,
) {
    if radius_scale != 1.0 {
        path.for_each_vert_mut(|v| v.corner_radius *= radius_scale);
    }
    match &mut path.fill {
        Some(Paint::Linear { start, end, .. }) => {
            *start = f(*start);
            *end = f(*end);
        }
        Some(Paint::Radial { center, radius, .. }) => {
            *center = f(*center);
            *radius *= radius_scale;
        }
        Some(Paint::MultiPoint { points }) => {
            for p in points {
                p.pos = f(p.pos);
            }
        }
        // ⭐⭐ **O padrão SONDA o afim, e por isso é o único preenchimento desta casa que conserva a
        // ORIENTAÇÃO.** As imagens dos dois eixos unitários dizem tudo: o ângulo do eixo x é a
        // rotação, e o comprimento de cada imagem é a escala NAQUELE eixo. É exacto para qualquer
        // afim, e melhor do que o `radius_scale` médio que o gradiente radial recebe — não por
        // esmero, mas porque um radial do peniko **é circular** e não tem onde guardar um ângulo,
        // enquanto o padrão tem.
        //
        // ⚠️ **Um espelho vira uma meia-volta**, e a aproximação é nomeada: `atan2` de um eixo
        // invertido dá `π`, e o `PatternFill` não tem campo de reflexão onde guardar a diferença.
        Some(Paint::Pattern(pat)) => {
            let o = f(pat.origin);
            let ax = f([pat.origin[0] + 1.0, pat.origin[1]]);
            let ay = f([pat.origin[0], pat.origin[1] + 1.0]);
            let (dx, dy) = ([ax[0] - o[0], ax[1] - o[1]], [ay[0] - o[0], ay[1] - o[1]]);
            pat.angle += dx[1].atan2(dx[0]);
            pat.size = [
                pat.size[0] * dx[0].hypot(dx[1]),
                pat.size[1] * dy[0].hypot(dy[1]),
            ];
            pat.origin = o;
        }
        Some(Paint::Solid(_)) | None => {}
    }
}
