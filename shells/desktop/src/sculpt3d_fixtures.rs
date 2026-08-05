//! **AS MALHAS DE FIXTURE** — como cada modelo de smoke é ESCULPIDO.
//!
//! Irmão (`#[path]`) de [`super::scenes`], e o corte é entre *que cena o smoke
//! monta* (lá: quem está armada, que peças entram, o roteiro que o artista lê) e
//! *como uma malha de fixture é FEITA* (aqui). Ele saiu quando o arquivo das
//! cenas cruzou o cap de 600 LOC do HR-18 — split, nunca allowlist.
//!
//! ⚠️ **Toda malha aqui é construída com os VERBOS do produto**, nunca com
//! geometria escrita à mão nos vértices: um relevo fabricado seria uma segunda
//! resposta a *"como uma crista é feita"*, e ela divergiria da primeira no dia em
//! que o depósito mudasse. É a razão de estas funções morarem no shell e não
//! numa crate de teste.

use super::{Brush, Dab, SculptStroke, Symmetry, Verb};

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
pub(super) fn ridged_sphere() -> ph2d_mesh::Mesh {
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

/// A esfera com um BICO puxado longe demais — o modelo que pede um remesh.
///
/// ⚠️ **É a fixture certa porque ela nomeia o problema que o botão resolve**, e
/// uma esfera lisa não nomeia nenhum. Um SnakeHook longo arrasta os mesmos
/// vértices por uma distância grande: a densidade do bico despenca, os
/// triângulos ficam compridos e finos, e a partir de certo ponto **não há mais
/// barro ali para esculpir**. O remesh devolve densidade uniforme, e é isso que
/// o artista vai julgar — não a forma, que tem de sobreviver.
///
/// ⚠️ **Puxado com o VERBO do produto**, como toda fixture deste arquivo: um bico
/// escrito à mão nos vértices seria uma segunda resposta a *"o que um SnakeHook
/// faz"*, e ela divergiria da primeira no dia em que o gancho mudasse.
pub(super) fn hooked_sphere() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let brush = Brush {
        verb: Verb::SnakeHook,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    // O gancho AGARRA e leva: cada passo puxa o que está sob ele mais um pouco,
    // e é a soma deles que estica o barro além do que a malha representa.
    const STEPS: usize = 40;
    let mut tip = [0.0f32, 0.0, 1.0];
    for _ in 0..STEPS {
        let step = [0.0f32, 0.028, 0.028];
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::hooking(tip, brush.radius, [0.0, 0.0, -1.0], step),
            Symmetry::default(),
        );
        for k in 0..3 {
            tip[k] += step[k];
        }
    }
    mesh
}

/// A esfera com um FURO — o modelo que chega quebrado.
///
/// ⚠️ **O furo é feito ARRANCANDO faces, não desenhando uma beira**, e é isso que
/// o torna a fixture certa: a beira resultante é a que uma malha de verdade tem
/// (um contorno irregular, seguindo a grade da esfera), e não uma que eu
/// escolhi. Os vértices que ficam sem face nenhuma sobrevivem soltos, como
/// sobreviveriam num OBJ de terceiro.
pub(super) fn punctured_sphere() -> ph2d_mesh::Mesh {
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

/// **A esfera de RUGAS EM ESCADA** — a fixture da cavidade (`=15`).
///
/// Sete sulcos paralelos de profundidades DECRESCENTES, cavados com o verbo do
/// produto (`Crease`). ⚠️ **A escada é o oráculo inteiro, e uma esfera com um
/// vinco só não serviria:** o que a cavidade entrega é *ver o que a luz sozinha
/// não mostra*, então a cena precisa conter sulcos que a luz JÁ mostra (para o
/// artista ter referência) e sulcos que ela quase não mostra (que é onde o canal
/// se paga). Com uma profundidade só, ligar o canal produz *uma imagem
/// diferente* — e diferente não é a pergunta.
///
/// ⚠️ **Cavados e não escritos à mão nos vértices**, a lei deste arquivo: um
/// sulco fabricado seria uma segunda resposta a *o que um Crease deixa*, e o que
/// o smoke julga é a leitura da forma que o PRODUTO produz.
#[must_use]
pub(super) fn wrinkled_sphere() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let mut stroke = SculptStroke::default();
    const STEPS: usize = 40;
    // ⚠️ **As sete forças saem da MEDIÇÃO, e a primeira escada que escrevi era
    // errada:** com `1,0 … 0,09` os três sulcos de baixo saíam com curvatura
    // 0,026 · 0,008 · 0,0005 — invisíveis mesmo com a cavidade no teto, ou seja
    // três degraus que não ensinam nada. Estas produzem **0,227 → 0,052**, que é
    // a faixa entre *satura o canal* (o ganho de 4,0 satura em 0,25) e *aparece
    // só quando ele está ligado*.
    for (k, strength) in [1.0f32, 0.82, 0.68, 0.56, 0.47, 0.39, 0.32]
        .into_iter()
        .enumerate()
    {
        let brush = Brush {
            verb: Verb::Crease,
            radius: 0.10,
            strength,
            ..Brush::default()
        };
        // Sulcos paralelos ao longo de meridianos vizinhos, no hemisfério que
        // olha para a câmera.
        let v = -0.45 + 0.15 * k as f32;
        stroke.begin(&mesh);
        for i in 0..=STEPS {
            let u = -0.5 + 1.0 * i as f32 / STEPS as f32;
            let r2 = (u * u + v * v).min(0.98);
            let c = [u, v, (1.0 - r2).max(1e-3).sqrt()];
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at(c, brush.radius, [-c[0], -c[1], -c[2]]),
                Symmetry::default(),
            );
        }
    }
    mesh
}
