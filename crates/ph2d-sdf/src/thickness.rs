//! **A ESPESSURA POR VÉRTICE** — quanta matéria há atrás de cada ponto da
//! superfície, e a metade que faltava para a luz ATRAVESSAR a peça.
//!
//! # Por que ela existe
//!
//! O canal pré-integrado do SSS (`ph2d_mesh_render::sss`) **redistribui** a luz
//! que chega pela frente: o teto dele é a média dessa luz sobre a esfera, medido
//! em `1/π ≈ 0,318` do lambert cheio, e ele chega lá virando um **disco cinza
//! sem cor** (a separação R−B cai de 0,0375 em `t = 1,5` para 0,0001 em `t = 24`
//! — `ph2d-mesh-render/tests/measure_sss_curve.rs`).
//!
//! Ou seja: **empurrar aquele eixo não faz cera, faz um disco chapado.** Cera,
//! folha, orelha e mão contra a lanterna são todas a MESMA coisa — luz que entra
//! por trás, atravessa, e sai onde o lambert vale zero. Isso é um termo que
//! **SOMA**, e nenhuma quantidade de difusão pré-integrada o produz.
//!
//! O `docs/3D/05.1` §2b já prescrevia a forma: `exp(-espessura · coeficiente)`,
//! com a espessura **assada por vértice, raio para dentro**.
//!
//! # Por que a MARCHA NO CAMPO, e não o raio nem a curvatura
//!
//! Os três candidatos foram medidos contra um oráculo externo — numa esfera a
//! resposta é `2r`, ao dígito (`tests/measure_thickness.rs`):
//!
//! | método | erro (esfera) | erro (toro) | erro (chapa) | 13 682 vértices |
//! |---|---|---|---|---|
//! | proxy `2/|κ|` (grátis) | **0%** | **420%** | **511%** | 0 ms |
//! | raio contra o octree | 0,02% | exato | exato | **541 ms** |
//! | **marcha no campo** (este) | 0,33% | — | — | **30 ms** |
//!
//! ⚠️ **O proxy pela curvatura é exato na esfera e catastrófico na chapa** — que
//! é justamente a forma pela qual a luz atravessa. Ele teria passado numa
//! fixture de esferas (a cena `=19` é uma!) e mentido no caso de uso. *Um proxy
//! que só acerta onde a fixture olha não é uma medição.*
//!
//! ⚠️ **E o raio perde por ESCALA, não por precisão.** Ele é 16× mais exato — e
//! os 0,33% do campo entram num `exp()`, onde são invisíveis. O que decide é o
//! crescimento: quadruplicar os vértices multiplica o raio por **14** (cada raio
//! atravessa a peça inteira, e mais faces caem no caminho) e o campo por **3,6**.
//! Numa escultura de verdade a diferença deixa de ser 18× e vira *segundos
//! contra minutos*. A minha primeira nota afirmava o contrário (*"~30× mais
//! barato que o AO"*) e a medição a derrubou: ele é **244× mais caro**.
//!
//! ⚠️ **E o campo já existe:** o `bake_ao` o constrói no mesmo gesto. O bake de
//! espessura mora nesta crate por isso — *os dois canais assados saem do mesmo
//! campo*, e pô-los em crates diferentes seria construir a estrutura duas vezes
//! ou fazer uma delas atravessar a fronteira sem motivo.

use ph2d_mesh::Mesh;

use crate::field::VoxelField;

/// A espessura de um vértice que ninguém assou — ou que a marcha não conseguiu
/// medir.
///
/// ⚠️ **O default é o que NÃO acende**, o mesmo argumento do
/// `ph2d_mesh::DEFAULT_AO` pelo lado oposto: um canal ausente tem de ser
/// invisível, e aqui invisível significa **opaco** (`exp(-∞) = 0`). Se a
/// ausência brilhasse, toda malha nasceria de vidro e o artista iria procurar o
/// vidro no shader.
pub const DEFAULT_THICKNESS: f32 = f32::INFINITY;

/// **Assa a espessura de todos os vértices** contra o campo que o [`crate::bake_ao`]
/// já construiu.
///
/// ⚠️ **SERIAL, e a decisão é medida:** 30 ms a 13 682 vértices, num gesto de
/// botão que já paga a voxelização e o flood fill. Um `rayon` novo exige ADR
/// novo pela cerca do ADR-0109, e gastar um ADR aqui seria pagar caro por
/// milissegundos. Se a malha crescer a ponto de o número mudar, o ADR vem então
/// — e o laço já tem a forma que o ADR-0156 autorizou para o AO (gather
/// por-vértice, campo imutável, saídas disjuntas).
///
/// ⚠️ **E ele NÃO tem passe de preenchimento, embora tenha tido.** A primeira
/// versão marchava com o RAIO, e o raio fura: numa malha UV o antípoda de um
/// vértice é outro vértice, o raio sai exatamente pela quina de dois triângulos,
/// e o teste de interseção o recusa nos dois — **5 de 3386 (0,15%)** numa esfera
/// 48×72, cada um um ponto opaco no meio de tinta translúcida. Trocado o motor
/// pelo campo, o furo **deixou de existir** (a marcha ou acha a saída ou anda
/// até o fim), e o passe virou código morto que ninguém tinha reconferido.
/// Quem denunciou foi a mutação: apagá-lo não sangrava gate nenhum.
#[must_use]
pub fn bake(field: &VoxelField, mesh: &Mesh) -> Vec<f32> {
    (0..mesh.positions().len())
        .map(|v| at(field, mesh.positions()[v], mesh.normals()[v]))
        .collect()
}

/// A espessura de UM ponto — a porta que o [`bake`] percorre.
///
/// O raio entra pela superfície ao longo de `-N` e anda até o campo dizer que
/// saiu. ⚠️ **Ele exige nascer DENTRO**, e essa guarda é o que separa *fino* de
/// *não-medido*: numa casca sem volume (uma folha de duas faces, uma superfície
/// aberta) o primeiro passo já cai fora, e devolver a distância andada ali
/// diria *"espessura zero"* — a peça inteira acenderia como vidro. O campo não
/// tem interior a percorrer, e a resposta honesta é [`DEFAULT_THICKNESS`].
#[must_use]
pub fn at(field: &VoxelField, p: [f32; 3], n: [f32; 3]) -> f32 {
    let step = field.step();
    let far = field.far();
    let inside = |d: f32| {
        let q = [p[0] - n[0] * d, p[1] - n[1] * d, p[2] - n[2] * d];
        field.sample(q) < 0.0
    };
    // ⚠️ Meio passo para dentro: exatamente sobre a superfície o campo vale zero,
    // e o sinal ali é ruído de interpolação.
    let first = step * 0.5;
    if !inside(first) {
        return DEFAULT_THICKNESS;
    }
    let mut d = first + step;
    while d < far {
        if !inside(d) {
            return d;
        }
        d += step;
    }
    DEFAULT_THICKNESS
}

#[cfg(test)]
#[path = "thickness_tests.rs"]
mod tests;
