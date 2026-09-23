//! ⭐⭐⭐ **A ÁREA DE UMA MALHA** — a da peça inteira e a de cada face, e a lei
//! do triângulo que as duas partilham.
//!
//! ⚠️ Ele saiu do [`super::mesh`] por TECTO DE LOC em 2026-09-23, e o corte é
//! por RESPONSABILIDADE: aqui mora *quanto de superfície há*, e nada mais.

use super::Mesh;

/// ⭐⭐ **A ÁREA DE UM TRIÂNGULO — a lei que a peça inteira e cada face
/// partilham**, e a única cópia dela nesta crate.
///
/// ⚠️ Em `f64` de propósito: os dois consumidores somam muitos destes, e uma
/// malha de milhões de triângulos minúsculos perde a cauda numa soma de `f32`.
fn area_do_triangulo(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f64 {
    let (u, w) = (
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    );
    let n = [
        u[1].mul_add(w[2], -(u[2] * w[1])),
        u[2].mul_add(w[0], -(u[0] * w[2])),
        u[0].mul_add(w[1], -(u[1] * w[0])),
    ];
    f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()) * 0.5
}

impl Mesh {
    /// ⭐⭐ **A ÁREA DA SUPERFÍCIE** — a soma dos triângulos de toda face, com
    /// os n-gons abertos em leque a partir do primeiro canto.
    ///
    /// ⚠️⚠️ **É a PORTA ÚNICA, e ela nasceu com DOIS consumidores de crates
    /// diferentes** (2026-09-14): o botão de retopologia, que já a usava para
    /// ancorar a contagem de quads, e o alvo da **topologia dinâmica**, que
    /// passou a ancorar-se nela pelo mesmo motivo. *Duas somas de triângulos em
    /// duas crates seriam a segunda resposta à mesma pergunta.*
    ///
    /// ⭐ **Ela é da SUPERFÍCIE e não da tesselação** — é isso que a torna
    /// invariante a remalhar, e por consequência ao ZOOM e à escala da peça.
    ///
    /// ⚠️ **A soma é em `f64`** e só desce a `f32` no fim: uma malha de milhões
    /// de triângulos minúsculos perde a cauda numa soma de `f32`.
    ///
    /// ⚠️⚠️ **E este doc-comment ROUBOU o `#[must_use]` do vizinho ao nascer** —
    /// ele foi inserido entre o atributo e o `face_count`, que ficou sem ele. É
    /// a armadilha que este repo já regista duas vezes, e quem a apanhou foi o
    /// `clippy`, não uma leitura.
    #[must_use]
    pub fn surface_area(&self) -> f32 {
        let p = self.positions();
        let mut sum = 0.0f64;
        for f in self.faces() {
            let v = f.verts();
            for k in 1..v.len() - 1 {
                sum += area_do_triangulo(p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
            }
        }
        sum as f32
    }

    /// ⭐⭐⭐ **A ÁREA DE CADA FACE, uma por face** — a irmã da
    /// [`Self::surface_area`], com a MESMA aritmética por triângulo.
    ///
    /// ⚠️⚠️ **Elas partilham o triângulo e NÃO a soma, e isso é deliberado:**
    /// a [`Self::surface_area`] acumula triângulo a triângulo num `f64` só, e
    /// somar aqui por FACE e depois somar as faces dá outra ordem — *logo
    /// outros últimos bits*. Ela alimenta o tecto de quads da retopologia e o
    /// alvo da topologia dinâmica, e mudar-lhe um ULP move gates que nada têm a
    /// ver com esta porta. ⇒ a lei partilhada é
    /// [`area_do_triangulo`]; a acumulação é de cada um.
    ///
    /// ⭐ O consumidor é a GRADUAÇÃO da tinta fina (a P2): o nível de uma face
    /// sai da área dela, e sem esta porta a família teria de reescrever a soma
    /// do leque — *a segunda resposta à mesma pergunta*.
    #[must_use]
    pub fn face_areas(&self) -> Vec<f32> {
        let p = self.positions();
        self.faces()
            .iter()
            .map(|f| {
                let v = f.verts();
                let mut a = 0.0f64;
                for k in 1..v.len() - 1 {
                    a +=
                        area_do_triangulo(p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
                }
                a as f32
            })
            .collect()
    }
}
