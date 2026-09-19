//! ⭐⭐⭐ **O QUE A PLACA PRECISA DE LER PARA POSAR UMA PELE** — a W2 da F9, metade da CPU.
//!
//! # ⭐⭐⭐ O achado que torna o *vertex shader* trivial
//!
//! A lei da pele é `Σ ŵ_i · (M_i · p)` ([`ph2d_skeleton::Skin::blend`]) — uma mistura LINEAR de
//! afins. Mas os pesos que o bind guarda são **por TENDÃO** (o osso que o artista autorou), e a
//! pele mistura **por OSSO** (um osso que dobra é `N` sub-ossos). A conversão
//! ([`ph2d_skeleton::Skin::weights_from`]) reparte o peso do tendão pelos sub-ossos com uma
//! **quota**:
//!
//! ```text
//! quota = bend::share(sub, n_sub, u)      com  u = projeccao de p no eixo de REPOUSO do osso
//! ```
//!
//! ⭐⭐ **E `u` é função só da posição de REPOUSO.** ⇒ a quota, e portanto a tabela de pesos por
//! OSSO já normalizada, é uma grandeza do **BIND** — ela não muda quando o artista move um osso,
//! só quando ele muda a TOPOLOGIA do rig (segmentos, ou a lista de ossos presos).
//!
//! ⇒ **o shader não precisa de saber o que é um osso que dobra.** Ele lê, por vértice, a posição de
//! repouso e `N` pesos; por quadro, `N` afins. Toda a lei da dobra colapsa na tabela, do lado da
//! CPU, e só se recalcula quando a topologia muda. *Uma complexidade que se resolve uma vez não é
//! uma complexidade do shader.*
//!
//! # O que este módulo é, e o que ele ainda NÃO é
//!
//! Ele é **a metade da CPU da W2**: o empacotamento e a **lei de referência** — a aritmética que o
//! shader vai fazer, escrita aqui para a paridade CPU×GPU ter contra o que medir (o molde é o do
//! Flip: dois motores, uma lei). ⛔ Ele ainda não escreve buffer nenhum nem desenha: isso é a outra
//! metade, e ela precisa de um buffer por-bind com invalidação própria.
//!
//! # ⛔⛔ DÍVIDA NOMEADA: as CORRECÇÕES À MÃO não chegam aqui
//!
//! A tabela que este módulo empacota é a do **BIND**, e a correcção que o artista pinta
//! ([`ph2d_skeleton_ecs::CorreccaoDePeso`], 2026-09-19) é uma **MANCHA no espaço** aplicada por
//! ponto — ela não vive na tabela, é somada depois dela.
//!
//! ⚠️ **Hoje isso não é um defeito observável, e a razão é MEDIDA e não uma promessa:** este
//! caminho **não tem consumidor de produto** (nenhum sítio escreve buffer nem desenha por ele), e
//! as duas mídias vivas passam pela porta corrigida (`ph2d_vec_skin::aplica_corrigido` e
//! `skin_image::posed_sprite_mesh_corrigida`). *Uma lei que falta num caminho que ninguém percorre
//! é dívida, não um bug.*
//!
//! ⚠️⚠️ **Mas ela vira um DEFEITO MUDO no dia em que a outra metade shipar:** a arte desenharia
//! pela placa **sem** as correcções e pela CPU **com** elas, e o sintoma seria *«a correcção
//! funciona e depois some»* — sem um erro. ⇒ quem ligar o buffer tem de decidir onde a mancha
//! entra (a tabela é do bind e a mancha é por ponto: ou ela é **assada na tabela** no bind, e aí
//! muda quando o artista pinta, ou vai ao shader como uma lista), e há gate a lembrá-lo
//! (`crate::skin_gpu_tests::a_pele_da_placa_nao_conhece_as_correccoes_e_isso_esta_nomeado`).

use ph2d_poly2d::Mesh2d;
use ph2d_skeleton::{Skin, Xform};

/// ⭐⭐ **A PELE NO FORMATO DA PLACA** — construída uma vez por bind (ou quando a topologia do rig
/// muda), e imutável enquanto o artista só POSA.
#[derive(Clone, Debug, PartialEq)]
pub struct PeleGpu {
    /// A posição de REPOUSO de cada vértice, **no espaço da coisa deformada** (o que a
    /// [`ph2d_skeleton::Skin`] consome) — e não em pixels da imagem.
    ///
    /// ⚠️ A conversão `pixel → local` fica FORA de propósito: ela é da mídia, e uma segunda cópia
    /// dela aqui seria a segunda resposta a *«o que é um ponto nesta arte»*.
    pub rest: Vec<[f64; 2]>,
    /// **Achatado**: `pesos[v * ossos + i]` é o peso do vértice `v` no OSSO `i` — já repartido
    /// pelos sub-ossos e **já normalizado**.
    pub pesos: Vec<f64>,
    /// Quantos OSSOS (não tendões) a tabela cobre.
    pub ossos: usize,
}

impl PeleGpu {
    /// ⭐⭐⭐ **EMPACOTA** — resolve a quota da dobra e a normalização uma vez, no repouso.
    ///
    /// `por_tendao[v * tendoes + j]` é a tabela que o bind guarda
    /// ([`crate::skinned_mesh::SkinnedMesh::pesos`]); `p2l` leva um ponto da malha (pixels da
    /// imagem) ao espaço da pele.
    ///
    /// ⚠️ **A conta é a do produto, chamada pela porta do produto** ([`Skin::weights_from`]) e não
    /// reescrita aqui: *duas derivações do mesmo peso divergem no dia em que uma ganhar uma cerca*
    /// — a lei que este repo já pagou na ponte da curva do pincel de contorno.
    #[must_use]
    pub fn empacota(pele: &Skin, p2l: Xform, mesh: &Mesh2d, por_tendao: &[f64]) -> Self {
        let ossos = pele.bones().len();
        let tendoes = if mesh.rest.is_empty() {
            0
        } else {
            por_tendao.len() / mesh.rest.len()
        };
        let mut rest = Vec::with_capacity(mesh.rest.len());
        let mut pesos = Vec::with_capacity(mesh.rest.len() * ossos);
        let mut w = pele.scratch();
        for (v, &q) in mesh.rest.iter().enumerate() {
            let p = p2l.apply(q);
            let fatia = por_tendao
                .get(v * tendoes..(v + 1) * tendoes)
                .unwrap_or(&[]);
            pele.weights_from(p, fatia, &mut w);
            rest.push(p);
            pesos.extend_from_slice(&w);
        }
        Self { rest, pesos, ossos }
    }

    /// Os pesos do vértice `v`.
    #[must_use]
    pub fn pesos_de(&self, v: usize) -> &[f64] {
        self.pesos
            .get(v * self.ossos..(v + 1) * self.ossos)
            .unwrap_or(&[])
    }
}

/// ⭐ **AS POSES DO QUADRO** — `N` afins, o único que atravessa a fronteira por quadro.
///
/// ⚠️ **`f32` porque é o que a placa lê**, e a conversão mora aqui para ela ser a MESMA que a
/// paridade mede: uma conversão escrita no sítio do upload seria invisível a este módulo.
#[must_use]
pub fn poses_do_quadro(pele: &Skin) -> Vec<[f32; 6]> {
    pele.bones()
        .iter()
        .map(|b| b.pose.0.map(|x| x as f32))
        .collect()
}

/// ⭐⭐⭐ **A LEI DO SHADER, escrita na CPU** — `Σ w_i · (M_i · p)`, com o mesmo caso degenerado da
/// casa (*«ninguém reclama este ponto»* devolve o ponto intacto).
///
/// É contra ela que a paridade CPU×GPU se mede quando a outra metade da W2 chegar, e é ela que
/// **define** o que o shader tem de fazer — ⛔ não uma prosa a descrevê-lo.
///
/// ⚠️ **A aritmética é em `f32`** de propósito: é a da placa. Medi-la em `f64` mediria outro
/// programa, e a barra da paridade sairia de uma comparação que o produto nunca faz.
#[must_use]
pub fn posa_como_a_placa(pele: &PeleGpu, poses: &[[f32; 6]]) -> Vec<[f32; 2]> {
    pele.rest
        .iter()
        .enumerate()
        .map(|(v, p)| {
            let (px, py) = (p[0] as f32, p[1] as f32);
            let (mut x, mut y, mut soma) = (0.0_f32, 0.0_f32, 0.0_f32);
            for (i, &w) in pele.pesos_de(v).iter().enumerate() {
                let w = w as f32;
                if w == 0.0 {
                    continue;
                }
                let Some([a, b, c, d, e, f]) = poses.get(i).copied() else {
                    continue;
                };
                x += w * a.mul_add(px, c.mul_add(py, e));
                y += w * b.mul_add(px, d.mul_add(py, f));
                soma += w;
            }
            if soma == 0.0 { [px, py] } else { [x, y] }
        })
        .collect()
}

#[cfg(test)]
#[path = "skin_gpu_tests.rs"]
mod tests;
