//! **A CAIXA SÓLIDA do `sim.collide`** — irmã do `lib.rs` pelo tecto de LOC (HR-18) e por
//! ASSUNTO: ali mora o nó (params, resposta, kernel), aqui a GEOMETRIA de uma das formas dele.
//! ⚠️ Ela é portada termo a termo para o WGSL — mexer aqui é mexer nos dois lados.

/// **O CONTACTO COM A CAIXA SÓLIDA** — a única porta, portada termo a termo para o kernel.
///
/// `half` são as MEIAS extensões (a porta divide as inteiras uma vez, aqui em cima). O teste
/// é o do rectângulo arredondado: leva a peça ao referencial da caixa, acha o ponto mais
/// próximo DENTRO dela e mede.
///
/// ⚠️ **Os dois ramos são geometricamente diferentes e ambos necessários:**
/// - **fora** — a distância ao ponto mais próximo decide, e a normal é a direcção dela. É
///   isto que arredonda as quinas: uma peça na diagonal de um canto sai pela diagonal, não
///   por uma das faces.
/// - **dentro** — não há direcção «para fora» única, então sai pelo eixo de MENOR
///   penetração. Sem este ramo uma peça que nasceu dentro da caixa ficaria presa, e é o
///   mesmo problema que o centro exacto de um disco tem (e que o `[0, 1]` de lá resolve).
///
/// ⚠️ A caixa **CRESCE** pelo raio da peça, como o disco: o centro de uma peça de raio `r`
/// nunca pode estar a menos de `r` da superfície.
pub(super) fn box_contact(
    p: [f32; 2],
    c: [f32; 2],
    half: [f32; 2],
    r: f32,
    n: [f32; 2],
) -> Option<([f32; 2], f32)> {
    // `n` é a normal do plano — `(−sin, cos)` do `angle` —, então o co-seno e o seno saem
    // dela sem recalcular trigonometria (e sem uma segunda resposta a «que ângulo é este?»).
    let (cos, sin) = (n[1], -n[0]);
    let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
    let (lx, ly) = (dx * cos + dy * sin, -dx * sin + dy * cos);
    let (hw, hh) = (half[0].max(0.0), half[1].max(0.0));
    let (qx, qy) = (lx.clamp(-hw, hw), ly.clamp(-hh, hh)); // CLAMP-OK: extensões da caixa
    let (ex, ey) = (lx - qx, ly - qy);
    let d2 = ex * ex + ey * ey;
    let to_world = |v: [f32; 2]| [v[0] * cos - v[1] * sin, v[0] * sin + v[1] * cos];
    if d2 > f32::EPSILON * f32::EPSILON {
        let d = d2.sqrt();
        if d >= r {
            return None;
        }
        return Some((to_world([ex / d, ey / d]), r - d));
    }
    // Dentro: o eixo de menor penetração ganha. `signum` de zero é `1`, e o eixo exacto do
    // centro de uma caixa tem o mesmo empate que o centro de um disco — qualquer saída serve.
    let (px, py) = (hw - lx.abs(), hh - ly.abs());
    let nl = if px < py {
        [if lx < 0.0 { -1.0 } else { 1.0 }, 0.0]
    } else {
        [0.0, if ly < 0.0 { -1.0 } else { 1.0 }]
    };
    Some((to_world(nl), px.min(py) + r))
}
