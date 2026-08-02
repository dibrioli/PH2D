//! **A MULTIRESOLUÇÃO** — os níveis, e o ir-e-voltar entre eles.
//!
//! Port de `reference/sculptgl/src/mesh/multiresolution/` (`Multimesh.js`,
//! `MeshResolution.js`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! Subdividir já dá resolução; o que a multiresolução dá é **descer**. O artista
//! esculpe o detalhe fino no nível 3, volta ao nível 0 para mover a forma
//! GRANDE, e sobe de novo com o detalhe intacto — que é a única maneira de
//! corrigir uma proporção depois de já ter feito a pele.
//!
//! # O detalhe é um DESLOCAMENTO, e ele vive num frame LOCAL
//!
//! Guardar as posições do nível de cima seria inútil: mover a base embaixo não
//! as moveria. O que se guarda é a **diferença** entre onde o vértice está e
//! onde a subdivisão o PORIA — e essa diferença é expressa nos eixos
//! `(normal, tangente, binormal)` daquele vértice. Assim, quando a base é
//! entortada, o detalhe **gira junto**: uma verruga continua saindo perpendicular
//! à pele em vez de apontar para onde a pele apontava antes.
//!
//! ```text
//! descer:  base ← os V primeiros do topo      (a subdivisão põe os PARES ali)
//!          previsão = subdivide(base)
//!          detalhe = (topo − previsão) no frame de cada vértice do topo
//!
//! subir:   topo ← subdivide(base)             (a previsão, de novo)
//!          topo += detalhe, no MESMO frame
//! ```
//!
//! ⚠️ **A ida e a volta são EXATAS quando nada muda embaixo**, e isso não é
//! aspiração: `previsão + (topo − previsão) = topo` ao bit, desde que o frame
//! seja o mesmo dos dois lados. É o gate que decide este módulo.
//!
//! ⚠️ **E é por isso que o frame tem UMA porta** ([`local_frame`]). Encode e
//! decode são as duas metades de uma inversa; escritos duas vezes, eles
//! divergem — e a divergência não aparece como erro, aparece como a escultura
//! escorregando um pouco a cada viagem.
//!
//! ⚠️ **A NORMAL do frame é a do topo, e ela NÃO é recomputada entre descer e
//! subir.** É o que torna a viagem exata. Enquanto o artista trabalha embaixo,
//! ninguém toca a malha de cima; ao subir, o frame lido é literalmente o que
//! codificou. Recompor as normais do topo no meio do caminho — o que um
//! `rebuild` distraído faria — quebraria a inversa em silêncio, e o sintoma
//! seria o detalhe **derivando** a cada subida.
//!
//! ⚠️ **A tangente sai do PRIMEIRO vizinho do anel.** A escolha é arbitrária e
//! o que importa é ser a MESMA nos dois lados — ela é, porque a adjacência do
//! topo não muda entre as duas chamadas.

use crate::mesh::Mesh;
use crate::subdivide::{Predicted, predict, subdivide};

/// O detalhe de um nível em relação ao de baixo.
///
/// ⚠️ O de POSIÇÃO vive no frame local; os de canal são deltas simples. Um
/// canal não tem orientação — girar a base não gira uma cor.
#[derive(Clone, Debug, Default)]
struct Details {
    /// `(normal, tangente, binormal)` por vértice do nível de cima.
    xyz: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
}

/// **A pilha de níveis.** O nível 0 é a base; cada nível acima é uma subdivisão
/// do anterior mais o detalhe que o artista pôs nele.
#[derive(Clone, Debug)]
pub struct Multires {
    levels: Vec<Mesh>,
    /// `details[i]` é o detalhe do nível `i` contra o `i − 1`; o do nível 0 é
    /// vazio e nunca lido.
    details: Vec<Details>,
    sel: usize,
}

impl Multires {
    /// Uma pilha de um nível só — o estado de toda malha que ninguém subdividiu.
    #[must_use]
    pub fn new(base: Mesh) -> Self {
        Self {
            levels: vec![base],
            details: vec![Details::default()],
            sel: 0,
        }
    }

    /// Em que nível o artista está.
    #[must_use]
    pub fn level(&self) -> usize {
        self.sel
    }

    /// Quantos níveis existem.
    #[must_use]
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// A malha do nível atual — a que o artista vê e esculpe.
    #[must_use]
    pub fn mesh(&self) -> &Mesh {
        &self.levels[self.sel]
    }

    /// A malha do nível atual, para esculpir.
    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.levels[self.sel]
    }

    /// Troca a malha do nível atual — a porta do undo, que devolve uma malha
    /// inteira.
    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.levels[self.sel] = mesh;
    }

    /// **Acrescenta um nível acima**, subdividindo o atual. Devolve `false` se
    /// não estamos no TOPO.
    ///
    /// ⚠️ **Só do topo, e a recusa é estrutural.** Subdividir do meio criaria um
    /// segundo nível `n + 1` sem dizer o que fazer com o que já existe lá — e a
    /// resposta honesta (*jogar fora o detalhe acima*) é destruição silenciosa
    /// de trabalho. O original recusa igual.
    pub fn add_level(&mut self) -> bool {
        if self.sel + 1 != self.levels.len() {
            return false;
        }
        let up = subdivide(&self.levels[self.sel]);
        let n = up.vert_count();
        self.levels.push(up);
        // O nível novo nasce SEM detalhe: ele é exatamente a subdivisão do de
        // baixo, então a diferença é zero em todo vértice.
        self.details.push(Details {
            xyz: vec![[0.0; 3]; n],
            colors: None,
            masks: None,
        });
        self.sel += 1;
        true
    }

    /// **Desce um nível**, guardando o detalhe do de cima. `false` no nível 0.
    pub fn lower(&mut self) -> bool {
        if self.sel == 0 {
            return false;
        }
        let up = self.sel;
        let down = up - 1;

        // 1. A base recebe o que o artista fez nos vértices que ela COMPARTILHA
        //    com o topo.
        //
        // ⚠️ **Isto só funciona porque a subdivisão põe os vértices PARES em
        //    `[0, V)`** — o vértice `i` de baixo É o vértice `i` de cima. É uma
        //    das três divergências de forma que o `subdivide` tomou (o original
        //    aloca por ordem de visita e precisa de um mapa), e é aqui que ela
        //    se paga. Há gate afirmando a identidade.
        self.copy_shared_down(up, down);
        self.levels[down].rebuild();

        // 2. O que a subdivisão da base PORIA agora, e a diferença.
        let predicted = predict(&self.levels[down]);
        self.details[up] = encode(&self.levels[up], &predicted);
        self.sel = down;
        true
    }

    /// **Sobe um nível**, devolvendo o detalhe. `false` no topo.
    pub fn higher(&mut self) -> bool {
        if self.sel + 1 >= self.levels.len() {
            return false;
        }
        let down = self.sel;
        let up = down + 1;
        let predicted = predict(&self.levels[down]);
        decode(&mut self.levels[up], &self.details[up], &predicted);
        // ⚠️ O `rebuild` vem DEPOIS do decode, nunca antes: é ele que troca as
        // normais do topo, e o decode precisa das que codificaram.
        self.levels[up].rebuild();
        self.sel = up;
        true
    }

    /// **Descarta o nível do TOPO** e desce a seleção — o desfazer do
    /// [`Multires::add_level`].
    ///
    /// ⚠️ Só do topo, e só se houver mais de um: descartar do meio deixaria uma
    /// pilha cujos detalhes descrevem um nível que não existe mais.
    pub fn drop_top(&mut self) -> bool {
        if self.levels.len() < 2 || self.sel + 1 != self.levels.len() {
            return false;
        }
        self.levels.pop();
        self.details.pop();
        self.sel -= 1;
        true
    }

    /// Vai para o nível `target`, subindo ou descendo o que for preciso.
    pub fn select(&mut self, target: usize) {
        while self.sel > target && self.lower() {}
        while self.sel < target && self.higher() {}
    }

    /// Bytes segurados pela pilha inteira — os detalhes, que a malha não conta.
    #[must_use]
    pub fn detail_bytes(&self) -> usize {
        self.details
            .iter()
            .map(|d| {
                d.xyz.capacity() * size_of::<[f32; 3]>()
                    + d.colors.as_ref().map_or(0, |c| c.capacity() * 12)
                    + d.masks.as_ref().map_or(0, |m| m.capacity() * 4)
            })
            .sum()
    }

    /// Copia para a base os canais dos vértices que ela compartilha com o topo.
    fn copy_shared_down(&mut self, up: usize, down: usize) {
        let n = self.levels[down].vert_count();
        let (lo, hi) = self.levels.split_at_mut(up);
        let (dst, src) = (&mut lo[down], &hi[0]);
        dst.positions_mut().copy_from_slice(&src.positions()[..n]);
        if let Some(c) = src.colors() {
            dst.colors_mut().copy_from_slice(&c[..n]);
        }
        if let Some(m) = src.masks() {
            dst.masks_mut().copy_from_slice(&m[..n]);
        }
    }
}

/// **O frame local de um vértice** — a porta única de que encode e decode são as
/// duas metades.
///
/// `normal` é a do vértice no nível de cima; `at` e `neighbour` são a posição
/// PREVISTA dele e a do primeiro vizinho do anel. Devolve `None` quando o frame
/// é degenerado (normal nula, ou o vizinho exatamente sobre a normal): sem eixos
/// não há como escrever o deslocamento, e o original também desiste.
#[must_use]
fn local_frame(
    normal: [f32; 3],
    at: [f32; 3],
    neighbour: [f32; 3],
) -> Option<([f32; 3], [f32; 3], [f32; 3])> {
    let n = normalize(normal)?;
    let mut t = [
        neighbour[0] - at[0],
        neighbour[1] - at[1],
        neighbour[2] - at[2],
    ];
    // Projeta a corda no plano da normal — é o que torna o frame ortonormal.
    let along = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
    for k in 0..3 {
        t[k] -= n[k] * along;
    }
    let t = normalize(t)?;
    let bi = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    Some((n, t, bi))
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len2 == 0.0 {
        return None;
    }
    let inv = 1.0 / len2.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}

/// A diferença entre onde o topo ESTÁ e onde a subdivisão o poria.
fn encode(up: &Mesh, predicted: &Predicted) -> Details {
    let n = up.vert_count();
    let adj = up.adjacency();
    let mut xyz = vec![[0.0f32; 3]; n];
    for (v, out) in xyz.iter_mut().enumerate().take(n) {
        let Some(&first) = adj.vert_verts.neighbours(v).first() else {
            continue;
        };
        let Some((nrm, tan, bi)) = local_frame(
            up.normals()[v],
            predicted.positions[v],
            predicted.positions[first as usize],
        ) else {
            continue;
        };
        let d = sub(up.positions()[v], predicted.positions[v]);
        *out = [dot(nrm, d), dot(tan, d), dot(bi, d)];
    }
    Details {
        xyz,
        colors: up
            .colors()
            .zip(predicted.colors.as_ref())
            .map(|(a, b)| (0..n).map(|i| sub(a[i], b[i])).collect()),
        masks: up
            .masks()
            .zip(predicted.masks.as_ref())
            .map(|(a, b)| (0..n).map(|i| a[i] - b[i]).collect()),
    }
}

/// Instala a previsão e soma o detalhe de volta, no MESMO frame.
fn decode(up: &mut Mesh, details: &Details, predicted: &Predicted) {
    let n = up.vert_count();
    // Os canais primeiro: eles não participam do frame.
    if let (Some(src), Some(d)) = (predicted.colors.as_ref(), details.colors.as_ref()) {
        let out = up.colors_mut();
        for i in 0..n {
            for k in 0..3 {
                out[i][k] = (src[i][k] + d[i][k]).clamp(0.0, 1.0);
            }
        }
    }
    if let (Some(src), Some(d)) = (predicted.masks.as_ref(), details.masks.as_ref()) {
        let out = up.masks_mut();
        for i in 0..n {
            out[i] = (src[i] + d[i]).clamp(0.0, 1.0);
        }
    }

    // ⚠️ O frame lê a PREVISÃO (não as posições vivas, que ainda são as da
    // viagem anterior) e as NORMAIS do topo, que ninguém recomputou desde o
    // encode. As duas metades da inversa perguntam à mesma porta.
    let mut moved = vec![[0.0f32; 3]; n];
    {
        let adj = up.adjacency();
        let normals = up.normals();
        for v in 0..n {
            moved[v] = predicted.positions[v];
            let Some(&first) = adj.vert_verts.neighbours(v).first() else {
                continue;
            };
            let Some((nrm, tan, bi)) = local_frame(
                normals[v],
                predicted.positions[v],
                predicted.positions[first as usize],
            ) else {
                continue;
            };
            let d = details.xyz.get(v).copied().unwrap_or([0.0; 3]);
            for k in 0..3 {
                moved[v][k] += nrm[k] * d[0] + tan[k] * d[1] + bi[k] * d[2];
            }
        }
    }
    up.positions_mut().copy_from_slice(&moved);
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
#[path = "multires_tests.rs"]
mod tests;
