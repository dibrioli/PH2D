//! **AS CENAS DO SMOKE** — com que malha cada uma abre, e o que ela declara.
//!
//! Filho (`#[path]`) de [`super`], e o corte é entre *o que a cena VIVA é* (lá:
//! a malha, a câmera, o pincel, o passe) e *que fixture cada cena de smoke
//! monta* (aqui). São assuntos diferentes: uma é o produto, a outra é o que se
//! põe na frente do Enio para ele julgar o produto — e a segunda cresce uma
//! entrada por wave.
//!
//! ⚠️ **Toda fixture aqui é construída com os VERBOS do produto**, nunca com
//! geometria fabricada à mão: um relevo escrito direto nos vértices seria uma
//! segunda resposta a *"como uma crista é feita"*, e ela divergiria da primeira
//! no dia em que o depósito mudasse.

use super::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// A cena está armada? (`PH2D_SCULPT3D_SMOKE` em `1`..`5`.)
pub(crate) fn smoke_armed() -> bool {
    matches!(
        std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref(),
        Some("1" | "2" | "3" | "4" | "5")
    )
}

/// `=5` — a cena do **TWIST e do LOCAL SCALE**: uma esfera com CRISTAS.
pub(crate) fn turn_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("5")
}

/// A esfera com uma CRUZ de cristas — o modelo em que um giro se VÊ.
///
/// ⚠️ **Uma esfera lisa é a fixture errada para o Twist, e não por gosto:** ela é
/// invariante por rotação em torno de qualquer eixo que passe pelo centro, então
/// torcer a calota frontal a deixa quase idêntica a ela mesma. O gesto
/// funcionaria e o smoke não teria como dizer. As cristas dão ao redemoinho uma
/// forma que o olho segue — e ao inchaço do Local Scale uma referência de
/// tamanho.
///
/// ⚠️ **Elas são desenhadas com o VERBO do produto**: um relevo escrito à mão nos
/// vértices seria uma segunda resposta a *"como uma crista é feita"*, e ela
/// divergiria da primeira no dia em que o depósito mudasse.
///
/// ⚠️ **São TRÊS traços, e a contagem sai da lei do envelope.** Um traço deixa
/// exatamente um `reach` de altura por mais dabs que ele carimbe — é essa a
/// promessa —, e um `reach` aqui é `raio × 0,2 = 0,028`, cerca de 1,4% do
/// diâmetro: no enquadramento padrão isso são ~5 px, uma ondulação que o olho
/// não segue. Cada `begin` re-congela o `pre`, então os traços empilham; três é
/// o que a medição pôs em ~4% do diâmetro (gate abaixo).
fn ridged_sphere() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.14,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    const STEPS: usize = 48;
    for _ in 0..3 {
        stroke.begin(&mesh);
        for i in 0..=STEPS {
            let u = -0.6 + 1.2 * i as f32 / STEPS as f32;
            for c in [
                [u, 0.0, (1.0 - u * u).max(1e-3).sqrt()],
                [0.0, u, (1.0 - u * u).max(1e-3).sqrt()],
            ] {
                stroke.dab(
                    &mut mesh,
                    &brush,
                    // O olho aponta da superfície para o centro: é o raio que
                    // teria produzido este acerto vindo de fora.
                    &Dab::at(c, brush.radius, [-c[0], -c[1], -c[2]]),
                    Symmetry::default(),
                );
            }
        }
    }
    mesh
}

/// `=3` — a cena da **REVERSÃO**: um modelo denso que É uma subdivisão.
pub(crate) fn reversion_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("3")
}

/// `=4` — a cena de **FECHAR BURACO**: uma esfera com um pedaço arrancado.
pub(crate) fn holes_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("4")
}

/// A esfera com um FURO — o modelo que chega quebrado.
///
/// ⚠️ **O furo é feito ARRANCANDO faces, não desenhando uma beira**, e é isso que
/// o torna a fixture certa: a beira resultante é a que uma malha de verdade tem
/// (um contorno irregular, seguindo a grade da esfera), e não uma que eu
/// escolhi. Os vértices que ficam sem face nenhuma sobrevivem soltos, como
/// sobreviveriam num OBJ de terceiro.
fn punctured_sphere() -> ph2d_mesh::Mesh {
    let sphere = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    let keep: Vec<ph2d_mesh::Face> = sphere
        .faces()
        .iter()
        .copied()
        .filter(|f| {
            let vs = f.verts();
            let n = vs.len() as f32;
            let mut c = [0.0f32; 3];
            for &v in vs {
                let p = sphere.positions()[v as usize];
                for k in 0..3 {
                    c[k] += p[k] / n;
                }
            }
            // Uma calota lateral, bem visível no enquadramento padrão.
            !(c[0] > 0.45 && c[1] > 0.15)
        })
        .collect();
    ph2d_mesh::Mesh::from_parts(sphere.positions().to_vec(), keep)
        .expect("arrancar face não inventa índice")
}

/// A malha com que cada cena abre.
///
/// ⚠️ **Porta única, e ela existe para o gate.** A cena `=3` só significa alguma
/// coisa se a malha dela de fato reverter, e isso é um fato sobre a GEOMETRIA
/// que nenhum arch-gate de fonte enxerga. Um gate que reconstruísse a malha por
/// conta própria estaria medindo outra malha no dia em que esta mudasse.
#[must_use]
pub(crate) fn smoke_mesh() -> ph2d_mesh::Mesh {
    if turn_scene() {
        return ridged_sphere();
    }
    if holes_scene() {
        return punctured_sphere();
    }
    if reversion_scene() {
        // ⚠️ **Ela é DUAS vezes subdividida de propósito**: um modelo denso que
        // chega pronto não tem um nível embaixo, e a cena só demonstra a
        // reversão se houver mais de um para reconstruir. A esfera UV mistura
        // quads no corpo com triângulos nos polos, que é o caso que exercita os
        // dois ramos do reconhecedor de uma vez.
        let coarse = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
        ph2d_mesh::subdivide(&ph2d_mesh::subdivide(&coarse))
    } else {
        ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
    }
}

/// `=2` — a cena da **DOAÇÃO**: a esfera E uma tela branca para pintar.
///
/// ⚠️ Cena própria, e não um passo a mais na `=1`: julgar a escultura e julgar a
/// doação são duas perguntas, e a segunda precisa de uma tela que a primeira não
/// quer ver. Misturá-las faria o smoke do barro abrir com um retângulo branco
/// atrás dele sem nada explicando por quê.
pub(crate) fn donation_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("2")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **A cena `=5` só significa alguma coisa se a esfera dela TIVER cristas**,
    /// e isso é um fato sobre geometria que nenhum arch-gate de fonte enxerga —
    /// o mesmo argumento do gate da cena `=3`, que pina que ela é construída
    /// subdividindo.
    ///
    /// ⚠️ **O oráculo tem duas metades, e a segunda é a que importa:** a crista
    /// tem de subir E a região LISA tem de ficar lisa. Só a primeira ficaria
    /// verde se o traço vazasse pela esfera inteira — e aí a fixture não teria
    /// forma a seguir, que é exatamente o que ela existe para dar.
    #[test]
    fn the_turn_scene_opens_with_a_sphere_that_has_ridges() {
        let mesh = ridged_sphere();
        let (mut on, mut off) = (0.0f32, 0.0f32);
        for p in mesh.positions() {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            // A cruz vive na calota `+Z`, ao longo dos planos `y = 0` e `x = 0`.
            if p[2] < 0.7 {
                continue;
            }
            if p[0].abs() < 0.05 || p[1].abs() < 0.05 {
                on = on.max(r - 1.0);
            } else if p[0].abs() > 0.3 && p[1].abs() > 0.3 {
                off = off.max((r - 1.0).abs());
            }
        }
        assert!(
            on > 0.04,
            "a crista subiu só {on:.4} do raio — numa esfera de diâmetro 2 isso não se segue com o olho"
        );
        assert!(
            off < 0.005,
            "a região LISA subiu {off:.4}: o traço vazou, e a fixture perdeu a forma que ela existe para dar"
        );
    }
}
