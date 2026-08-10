//! O **campo**: uma grade de distância à superfície, e o sinal que diz de que
//! lado de cada célula a malha está.
//!
//! Adaptado de `src/editing/Remesh.js` do SculptGL (MIT) — `createVoxelData`,
//! `voxelize` e `floodFill`. Licença em `LICENSES/sculptgl-MIT.txt`.
//!
//! # As duas metades, e por que são duas
//!
//! A **distância** sai da geometria: para cada triângulo, os voxels da caixa
//! dele perguntam *quão longe estou?* e guardam o menor. Isso é local, exato e
//! não sabe o que é dentro ou fora.
//!
//! O **sinal** não é local: saber se uma célula está dentro exige saber se dá
//! para caminhar dela até o infinito sem atravessar a superfície. É um *flood
//! fill* que parte de um canto — garantidamente fora, porque a caixa nasce com
//! folga — e o que ele NÃO alcança é o interior.
//!
//! ⚠️ **A barreira do flood não é a distância, são as ARESTAS ATRAVESSADAS.**
//! Um limiar de distância deixaria a onda passar pela primeira célula fina
//! demais; o voxelizador marca, por célula e por eixo, se o segmento até a
//! célula seguinte de fato fura um triângulo, e é **essa** marca que segura a
//! onda. É também por isso que o raio do [`TriEdges::ray_hit`] é tolerante nas
//! bordas: uma travessia perdida é um furo por onde o dentro vaza para fora.
//!
//! # ⚠️ O que este campo NÃO carrega (divergência declarada)
//!
//! A referência arrasta **cor** e **material** por-célula (24 B extra por
//! voxel, contra os 7 B que gastamos). Aqui o remesh devolve uma malha limpa: o
//! módulo ainda não pinta cor por-vértice, e a **máscara** de escultura também
//! se perde. Carregar qualquer um dos dois é um plano a mais no campo, com o
//! ciclo de vida inteiro atrás — fica nomeado, não contrabandeado.

use ph2d_mesh::{Aabb, Mesh, TriEdges};

/// Quantas células o maior lado da caixa recebe.
///
/// ⚠️ **É o número da referência** (`Remesh.RESOLUTION = 150`), e ele é uma
/// escolha de PRODUTO, não um teto de recurso: quem manda no custo é o cubo
/// dele. A tabela de custo por resolução sai da sonda `measure_remesh`, e é ela
/// que decide se este número sobe — não o conforto do porte.
pub const DEFAULT_RESOLUTION: u32 = 150;

/// Um campo de distância com sinal sobre uma grade regular.
pub struct VoxelField {
    dims: [usize; 3],
    step: f32,
    min: [f32; 3],
    /// Distância à superfície. Positiva fora, **negativa dentro** (depois do
    /// [`VoxelField::flood_fill`]), `INFINITY` onde nenhum triângulo alcançou.
    dist: Vec<f32>,
    /// Três bits por célula: a aresta que sai dela em +x, +y, +z atravessa a
    /// superfície?
    crossed: Vec<u8>,
}

impl VoxelField {
    /// A grade que cobre `bounds` com `resolution` células no maior lado.
    ///
    /// ⚠️ **A folga de 1,51 passos em cada lado é load-bearing**, e não é
    /// arredondamento: ela garante que a célula 0 — a semente do flood fill —
    /// está fora da malha, e que a casca exterior é contígua. Sem ela, uma malha
    /// que encosta na caixa faria o *fora* nascer desconectado, e o campo sairia
    /// do avesso onde o artista não olhou.
    pub fn for_bounds(bounds: Aabb, resolution: u32) -> Self {
        let ext = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        let longest = ext[0].max(ext[1]).max(ext[2]).max(f32::MIN_POSITIVE);
        let step = longest / resolution.max(1) as f32;
        let pad = step * 1.51;

        let min = [
            bounds.min[0] - pad,
            bounds.min[1] - pad,
            bounds.min[2] - pad,
        ];
        let mut dims = [0usize; 3];
        for a in 0..3 {
            let span = ext[a] + 2.0 * pad;
            // `+ 1` porque `dims` conta AMOSTRAS, não caixas: a última amostra
            // tem de cair além do canto, ou a extração perde a fatia final.
            dims[a] = ((span / step).ceil() as usize + 1).max(2);
        }

        let cells = dims[0] * dims[1] * dims[2];
        Self {
            dims,
            step,
            min,
            dist: vec![f32::INFINITY; cells],
            crossed: vec![0u8; cells * 3],
        }
    }

    /// As dimensões da grade, em amostras.
    pub fn dims(&self) -> [usize; 3] {
        self.dims
    }

    /// O lado de uma célula, em unidades de mundo.
    pub fn step(&self) -> f32 {
        self.step
    }

    /// O canto mínimo da grade, em unidades de mundo.
    pub fn origin(&self) -> [f32; 3] {
        self.min
    }

    /// O campo de distância, em ordem `x` mais rápido.
    pub fn distances(&self) -> &[f32] {
        &self.dist
    }

    /// Quantas células a grade tem.
    pub fn cell_count(&self) -> usize {
        self.dist.len()
    }

    /// A distância que vale *"longe o bastante para não importar"* — a diagonal
    /// da própria grade.
    ///
    /// Nada **dentro** da caixa pode estar mais longe da superfície do que isso,
    /// então usá-la no lugar de um infinito não inventa geometria: ela é um
    /// limite superior de verdade, não um número escolhido.
    pub fn far(&self) -> f32 {
        let d = [
            self.dims[0] as f32,
            self.dims[1] as f32,
            self.dims[2] as f32,
        ];
        self.step * (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }

    /// A distância à superfície em `p`, interpolada trilinearmente.
    ///
    /// ⚠️ **O infinito é DOMADO na entrada, e isso não é higiene — é a diferença
    /// entre um número e um `NaN`.** O campo nasce `INFINITY` onde nenhum
    /// triângulo alcançou, e o [`Self::flood_fill`] nega isso para `-INFINITY`
    /// no interior profundo. Interpolar contra um infinito é `0.0 * inf = NaN`
    /// sempre que um peso cai **exatamente** em zero — ou seja, exatamente sobre
    /// as amostras da grade, que é onde uma marcha de esfera mais pousa. Cada
    /// canto atravessa [`Self::far`] antes de entrar na soma, então o **sinal
    /// sobrevive** (aberto continua muito positivo, interior muito negativo) e a
    /// aritmética fica finita.
    ///
    /// Fora da grade a resposta é [`Self::far`]: a caixa nasce com folga (1,51
    /// passos por lado), então ter saído dela é ter saído da vizinhança da malha.
    ///
    /// `NaN` na entrada — ou no campo, que não deveria acontecer — sai como
    /// *aberto*. É a direção segura: um consumidor de oclusão que erra para
    /// aberto deixa de escurecer, e não fabrica uma sombra que a forma não tem.
    pub fn sample(&self, p: [f32; 3]) -> f32 {
        let far = self.far();
        let inv = 1.0 / self.step;

        let mut i0 = [0usize; 3];
        let mut frac = [0.0f32; 3];
        for a in 0..3 {
            let g = (p[a] - self.min[a]) * inv;
            if !(g >= 0.0 && g <= (self.dims[a] - 1) as f32) {
                // Inclui o `NaN`: a comparação negada o pega sem um ramo próprio.
                return far;
            }
            // `dims` é `>= 2` por construção ([`Self::for_bounds`]), então o
            // plano de amostras final sempre tem um vizinho à frente para o
            // canto `+1` da interpolação.
            let i = (g.floor() as usize).min(self.dims[a] - 2);
            i0[a] = i;
            frac[a] = g - i as f32;
        }

        let rx = self.dims[0];
        let rxy = rx * self.dims[1];
        let base = i0[0] + i0[1] * rx + i0[2] * rxy;
        let tame = |v: f32| {
            if v.is_finite() {
                v
            } else if v < 0.0 {
                -far
            } else {
                far
            }
        };
        let c = |dx: usize, dy: usize, dz: usize| tame(self.dist[base + dx + dy * rx + dz * rxy]);
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        let (fx, fy, fz) = (frac[0], frac[1], frac[2]);
        let x00 = lerp(c(0, 0, 0), c(1, 0, 0), fx);
        let x10 = lerp(c(0, 1, 0), c(1, 1, 0), fx);
        let x01 = lerp(c(0, 0, 1), c(1, 0, 1), fx);
        let x11 = lerp(c(0, 1, 1), c(1, 1, 1), fx);
        lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
    }

    /// Escreve a distância de `mesh` no campo, e marca as arestas de voxel que a
    /// superfície atravessa.
    ///
    /// Pode ser chamado mais de uma vez: o campo guarda o **menor**, então
    /// voxelizar duas malhas dá a união delas — que é como um remesh funde uma
    /// cena inteira num corpo só.
    pub fn voxelize(&mut self, mesh: &Mesh) {
        let pos = mesh.positions();
        let mut tris = Vec::new();
        mesh.triangle_indices(&mut tris);

        let inv_step = 1.0 / self.step;
        let (rx, ry, rz) = (self.dims[0], self.dims[1], self.dims[2]);
        let rxy = rx * ry;
        // Os três eixos, como direções de raio. É a mesma ordem dos três bits
        // por célula em `crossed`.
        const AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        for t in &tris {
            let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
            let tri = TriEdges::new(a, b, c);

            // A caixa do TRIÂNGULO, não a da face: é ela que decide quantos
            // voxels este laço visita, e é onde o custo do remesh vive.
            let mut lo = [0usize; 3];
            let mut hi = [0usize; 3];
            for ax in 0..3 {
                let tmin = a[ax].min(b[ax]).min(c[ax]);
                let tmax = a[ax].max(b[ax]).max(c[ax]);
                let i0 = ((tmin - self.min[ax]) * inv_step).floor();
                let i1 = ((tmax - self.min[ax]) * inv_step).ceil();
                // A folga da caixa já garante que isto cai dentro; o grampo é a
                // rede que transforma uma malha com `NaN` ou um vértice absurdo
                // num resultado feio em vez de um pânico.
                lo[ax] = i0.max(0.0) as usize;
                hi[ax] = (i1.max(0.0) as usize).min(self.dims[ax] - 1);
            }

            for k in lo[2]..=hi[2].min(rz - 1) {
                for j in lo[1]..=hi[1] {
                    for i in lo[0]..=hi[0] {
                        let n = i + j * rx + k * rxy;
                        let p = [
                            self.min[0] + i as f32 * self.step,
                            self.min[1] + j as f32 * self.step,
                            self.min[2] + k as f32 * self.step,
                        ];
                        let (sq, closest) = tri.closest_to(p);
                        let d = sq.sqrt();
                        if d < self.dist[n] {
                            self.dist[n] = d;
                        }

                        // Longe do triângulo não há aresta de voxel que ele
                        // possa atravessar: uma aresta mede um passo.
                        if d > self.step {
                            continue;
                        }
                        for (ax, dir) in AXES.iter().enumerate() {
                            // O ponto mais próximo tem de estar À FRENTE, dentro
                            // de um passo, no eixo que estamos testando — senão
                            // o raio nem vale a pena.
                            let along = closest[ax] - p[ax];
                            if along < 0.0 || along > self.step {
                                continue;
                            }
                            let e = n * 3 + ax;
                            if self.crossed[e] == 1 {
                                continue;
                            }
                            if let Some(hit) = tri.ray_hit(p, *dir)
                                && (0.0..=self.step).contains(&hit)
                            {
                                self.crossed[e] = 1;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Decide o sinal: o que a onda alcança a partir do canto é fora, o resto é
    /// dentro.
    ///
    /// ⚠️ **Diverge da referência em duas coisas, as duas por correção:**
    ///
    /// 1. A vizinhança é calculada em `(x, y, z)`, não por deslocamento cru no
    ///    índice linear. No original, `cell ± 1` **envolve de uma linha para a
    ///    outra** e `cell ± rx` de uma fatia para a outra; lá isso é inofensivo
    ///    porque a folga da caixa faz o fora encostar no fora, mas é uma
    ///    propriedade da MARGEM segurando um erro de indexação.
    /// 2. A célula-semente é marcada ao ser semeada. No original ela entra na
    ///    pilha sem rótulo, e se nenhum vizinho a re-empurrar ela termina
    ///    contada como **interior** — de novo, invisível só por causa da folga.
    ///
    /// # Devolve quantas células ficaram DENTRO
    ///
    /// ⚠️ **Quem cria o interior é quem sabe dizer se há um**, e o número é a
    /// única testemunha barata de que a onda não vazou: um vazamento marca a
    /// grade inteira como fora, e a contagem cai a **zero**. Sem ele o vazamento
    /// só aparece lá na frente, como uma malha vazia que o chamador instala
    /// achando que é o resultado — foi assim que ele viveu até aqui.
    ///
    /// Não é `#[must_use]` de propósito: quem só vai AMOSTRAR o campo (o AO, a
    /// espessura) tem o direito de ignorá-lo.
    pub fn flood_fill(&mut self) -> usize {
        let (rx, ry, rz) = (self.dims[0], self.dims[1], self.dims[2]);
        let rxy = rx * ry;
        let cells = self.dist.len();

        let mut outside = vec![false; cells];
        let mut stack: Vec<usize> = Vec::with_capacity(cells / 8);
        outside[0] = true;
        stack.push(0);

        while let Some(cell) = stack.pop() {
            let z = cell / rxy;
            let rem = cell - z * rxy;
            let y = rem / rx;
            let x = rem - y * rx;
            // Perto da superfície a onda só passa por aresta NÃO atravessada;
            // longe dela não há o que barrar.
            let guarded = self.dist[cell] < self.step;

            for ax in 0..3 {
                for step in [-1isize, 1] {
                    let (mut nx, mut ny, mut nz) = (x as isize, y as isize, z as isize);
                    match ax {
                        0 => nx += step,
                        1 => ny += step,
                        _ => nz += step,
                    }
                    if nx < 0
                        || ny < 0
                        || nz < 0
                        || nx >= rx as isize
                        || ny >= ry as isize
                        || nz >= rz as isize
                    {
                        continue;
                    }
                    let next = nx as usize + ny as usize * rx + nz as usize * rxy;
                    if outside[next] {
                        continue;
                    }
                    if guarded {
                        if self.dist[next] == f32::INFINITY {
                            continue;
                        }
                        // A marca de travessia mora na célula de índice MENOR do
                        // par, porque uma aresta é partilhada por duas células e
                        // guardá-la nas duas seria o mesmo fato escrito duas
                        // vezes — e duas cópias divergem.
                        let owner = if step > 0 { cell } else { next };
                        if self.crossed[owner * 3 + ax] == 1 {
                            continue;
                        }
                    }
                    outside[next] = true;
                    stack.push(next);
                }
            }
        }

        let mut inside = 0usize;
        for (id, out) in outside.iter().enumerate() {
            if !out {
                self.dist[id] = -self.dist[id];
                inside += 1;
            }
        }
        inside
    }
}

#[cfg(test)]
#[path = "field_tests.rs"]
mod tests;
