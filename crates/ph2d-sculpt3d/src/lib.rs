#![forbid(unsafe_code)]
//! `ph2d-sculpt3d` — **os verbos de escultura**.
//!
//! Na W1/M3 há **um**: o `Draw`, que empurra a superfície ao longo da normal.
//! Ele existe agora porque é o mínimo que responde à pergunta do M3 — *quanto
//! custa um dab?* —, e essa resposta é o que decide o kill-criterion K1 do
//! `docs/3D/03.5`. Os outros 19 verbos, o modelo de traço, a máscara e a
//! simetria são a W2, cada um com o gate dele.
//!
//! # Sobre a "porta única" que o `03.5` promete
//!
//! Aquele documento desenha `sculpt_kernel_device(vertices) -> Device`, e ela
//! **não está construída aqui, de propósito**. Uma porta com uma resposta só e
//! um variant inalcançável é um controle que não faz nada — o apodrecimento que
//! este repositório já nomeou várias vezes. Ela nasce **quando a medição pedir**:
//! se o K1 disparar, o caminho de GPU e a porta chegam juntos, com o caminho de
//! CPU virando o oráculo de paridade dele. É isso que a sonda
//! `tests/measure_brush_kernel.rs` existe para decidir.

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch};

/// Um toque de pincel: onde, de que tamanho, com que força.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Centro, em coordenadas de mundo.
    pub center: [f32; 3],
    /// Raio de influência, em unidades de mundo.
    pub radius: f32,
    /// Deslocamento no CENTRO, em unidades de mundo. Negativo cava.
    pub strength: f32,
}

/// O peso do dab a uma distância normalizada `t ∈ [0,1]` do centro.
///
/// `(1 − t²)²` — o falloff *Smooth* que ZBrush, Blender e SculptGL
/// compartilham. Ele é **C¹ na borda** (valor e derivada zeram em `t = 1`), e é
/// isso que faz um traço não deixar um degrau na fronteira do pincel. Sem
/// transcendental (HR-5): dois multiplicadores e uma subtração.
#[must_use]
pub fn falloff(t: f32) -> f32 {
    if t >= 1.0 {
        return 0.0;
    }
    let u = 1.0 - t * t;
    u * u
}

/// Buffers que atravessam os dabs de um traço. Alocar por dab é o que
/// transforma um gesto em serrilhado.
#[derive(Clone, Debug, Default)]
pub struct DabScratch {
    query: QueryScratch,
    region: RegionScratch,
    affected: Vec<u32>,
}

impl DabScratch {
    /// Quantos vértices o último dab tocou — a grandeza que decide o custo, e a
    /// que a sonda do M3 imprime ao lado dos milissegundos.
    #[must_use]
    pub fn affected(&self) -> usize {
        self.affected.len()
    }

    /// Bytes segurados. A sonda de memória o soma: o custo do GESTO não pode
    /// ficar fora da conta só por ser transitório.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        self.query.capacity_bytes()
            + self.region.capacity_bytes()
            + self.affected.capacity() * size_of::<u32>()
    }
}

/// Aplica **um** dab de `Draw`: cada vértice sob o pincel anda ao longo da
/// própria normal, pesado pelo falloff. Devolve quantos vértices se moveram.
///
/// ⚠️ **Um dab, não um traço.** Esta função lê a normal VIVA, o que é a resposta
/// certa para um toque isolado e a **errada** para uma sequência: encadeada, ela
/// realimenta a própria saída, e o que sobrevive a `n` dabs vira um produto
/// sobre a lista de dabs — a doença que a `line/Painter` curou quatro vezes no
/// relevo 2D. O modelo de traço da W2 (`docs/3D/04.1`) congela o estado no
/// pen-down e acumula um ENVELOPE; ele muda **qual** normal se lê, não **quanto
/// trabalho** se faz, e é por isso que a medição do M3 vale para os dois.
pub fn apply_dab(mesh: &mut Mesh, dab: &Dab, scratch: &mut DabScratch) -> usize {
    if dab.radius <= 0.0 {
        return 0;
    }
    mesh.verts_in_sphere(
        dab.center,
        dab.radius,
        &mut scratch.query,
        &mut scratch.affected,
    );
    if scratch.affected.is_empty() {
        return 0;
    }

    let inv_r = 1.0 / dab.radius;
    // As normais são lidas ANTES de qualquer escrita de posição: dentro de um
    // mesmo dab, mover um vértice não pode mudar a direção em que o vizinho vai.
    for i in 0..scratch.affected.len() {
        let v = scratch.affected[i] as usize;
        let n = mesh.normals()[v];
        let p = mesh.positions()[v];
        let d = [
            p[0] - dab.center[0],
            p[1] - dab.center[1],
            p[2] - dab.center[2],
        ];
        let dist2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        let w = falloff(dist2.sqrt() * inv_r) * dab.strength;
        let out = &mut mesh.positions_mut()[v];
        out[0] += n[0] * w;
        out[1] += n[1] * w;
        out[2] += n[2] * w;
    }

    mesh.refresh_region(&scratch.affected, &mut scratch.region);
    scratch.affected.len()
}

#[cfg(test)]
#[path = "dab_tests.rs"]
mod tests;
