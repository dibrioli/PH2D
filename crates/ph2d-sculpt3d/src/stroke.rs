//! **A LEI DO TRAÇO** — a peça que separa este módulo de um port ingênuo.
//!
//! > O efeito de um traço é função do **CAMINHO**, nunca de quão fino o motor
//! > amostrou o caminho. (`docs/3D/04.1`)
//!
//! No ZBrush e no Blender cada dab soma sobre o RESULTADO do anterior, então o
//! que sobrevive a `n` dabs é um **produto sobre a lista de dabs** — e a lista
//! depende da taxa de amostragem do mouse. Passar devagar deposita mais que
//! passar rápido pelo mesmo caminho. A `line/Painter` pagou esse bug **quatro
//! vezes** em 2D (mordida do arado · cápsula do relevo · campo de smear · gate
//! de proteção) até formular a cura, e ela vale igual em 3D:
//!
//! ```text
//! pen-down:  base[v] ← positions[v]          // congela o "pre"
//! por dab:   accum[v] ← max(accum[v], w)     // ENVELOPE, não `+=`
//! por dab:   target[v] ← alvo(verbo)         // do dab que VENCEU
//! aplica:    positions[v] ← lerp(base, target, accum)
//! ```
//!
//! Três propriedades caem disso, e as três são visíveis para o artista:
//!
//! 1. **Independência de espaçamento** — devagar ou rápido dá o mesmo resultado.
//! 2. **Idempotência sob re-stamp** — repetir a mesma lista de dabs não
//!    intensifica nada, o que é o que permitiria editar parâmetros do traço
//!    *depois* dele.
//! 3. **Undo trivial** — `base` **é** o estado anterior e `touched` **é** a
//!    janela; não há um segundo sistema a construir.
//!
//! ⚠️ **O `target` guarda o VENCEDOR, não uma média.** Quando um dab novo eleva
//! o `accum` de um vértice, ele também reescreve o alvo — o mesmo desenho do
//! envelope do impasto 2D, que guarda *os ingredientes do dab mais carregado*.
//! Sem isso, um verbo cujo alvo depende do dab (todos os de plano) teria de
//! recomputar a pegada inteira a cada dab, e o gesto deixaria de ser limitado
//! pela pegada.
//!
//! ⚠️ **Um vértice NÃO capturado tem `pre == posição viva`** — porque só quem foi
//! capturado é escrito. É isso que deixa o Smooth ler a vizinhança sem capturar
//! o anel inteiro, e é por isso que `base_pos_of` cai na malha viva sem mentir.

use crate::brush::{Brush, Symmetry, Verb};
use ph2d_mesh::{DEFAULT_MASK, Mesh, QueryScratch, RegionScratch};

/// Um toque de pincel: **onde a mão estava e com que força apertou**. O que a
/// ferramenta É vive no [`Brush`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Centro, em coordenadas de mundo (tipicamente o `Hit::point` do pick).
    pub center: [f32; 3],
    /// Raio de influência, em unidades de mundo.
    pub radius: f32,
    /// Pressão do dispositivo em `[0, 1]`. Sem tablet, `1.0`.
    pub pressure: f32,
}

impl Dab {
    /// Um dab de pressão cheia.
    #[must_use]
    pub fn at(center: [f32; 3], radius: f32) -> Self {
        Self {
            center,
            radius,
            pressure: 1.0,
        }
    }
}

/// O plano ajustado à pegada de um dab.
///
/// ⚠️ **Inclinado, nunca horizontal** — um ajuste horizontal *cava uma cratera
/// na encosta* em vez de achatá-la (lição paga no `plane.rs` do Painter 2D).
/// O estimador é a média ponderada pelo falloff das posições e das normais da
/// pegada, que é o `calc_area_normal_and_center` do Blender; ele difere de um
/// ajuste por mínimos quadrados de verdade numa sela, e a divergência está
/// registrada aqui em vez de escondida.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PlaneFit {
    point: [f32; 3],
    normal: [f32; 3],
}

/// O estado vivo de UM traço de escultura.
///
/// Os dois vetores do tamanho da malha (`slot`/`stamp`) vivem aqui e são
/// **reusados entre traços** — carimbados por época, como o `QueryScratch`. O
/// resto é do tamanho da PEGADA do traço, e é isso que mantém a memória de um
/// gesto proporcional ao que o artista tocou e não ao que ele abriu.
#[derive(Clone, Debug, Default)]
pub struct SculptStroke {
    slot: Vec<u32>,
    stamp: Vec<u32>,
    epoch: u32,
    touched: Vec<u32>,
    base_pos: Vec<[f32; 3]>,
    base_nrm: Vec<[f32; 3]>,
    base_mask: Vec<f32>,
    accum: Vec<f32>,
    target: Vec<[f32; 3]>,
    footprint: Vec<u32>,
    moved: Vec<u32>,
    query: QueryScratch,
    region: RegionScratch,
}

impl SculptStroke {
    /// Congela o `pre`: começa um traço novo sobre `mesh`.
    ///
    /// Não copia a malha — a captura é **preguiçosa, por vértice tocado**. Um
    /// traço numa malha de 5 M vértices que toca 20 mil paga 20 mil, não 5 M.
    pub fn begin(&mut self, mesh: &Mesh) {
        let n = mesh.vert_count();
        if self.slot.len() != n {
            self.slot = vec![u32::MAX; n];
            self.stamp = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — a mesma regra do `QueryScratch`, e sem ela um
        // traço a cada 4 bilhões nasceria achando que já capturou tudo.
        if self.epoch == 0 {
            self.epoch = 1;
            self.stamp.fill(0);
        }
        self.touched.clear();
        self.base_pos.clear();
        self.base_nrm.clear();
        self.base_mask.clear();
        self.accum.clear();
        self.target.clear();
    }

    /// Os vértices que este traço tocou — **a janela do undo**.
    #[must_use]
    pub fn touched(&self) -> &[u32] {
        &self.touched
    }

    /// As posições de antes do traço, na ordem de [`Self::touched`] — **o
    /// estado anterior do undo**. Não há um segundo sistema a construir: o
    /// congelamento que a lei exige já É a entrada de undo.
    #[must_use]
    pub fn base_positions(&self) -> &[[f32; 3]] {
        &self.base_pos
    }

    /// As máscaras de antes do traço, na ordem de [`Self::touched`].
    #[must_use]
    pub fn base_masks(&self) -> &[f32] {
        &self.base_mask
    }

    /// Os vértices que o ÚLTIMO dab de fato moveu.
    #[must_use]
    pub fn last_moved(&self) -> &[u32] {
        &self.moved
    }

    /// Os vértices que o último dab deixou **obsoletos na GPU** — a janela do
    /// upload incremental.
    ///
    /// ⚠️ **É um SUPERCONJUNTO de [`Self::last_moved`], e confundir os dois é um
    /// defeito visível.** Mover um vértice muda a normal de todo vizinho que
    /// compartilha uma face com ele, mesmo que o vizinho não tenha andado —
    /// `refresh_region` já os conserta na CPU, e subir só os movidos deixa a
    /// malha iluminada por normais velhas numa faixa de um anel de largura, bem
    /// na BORDA do pincel. Um gate de GPU pegou isto comparando o quadro
    /// incremental com o quadro do upload cheio.
    #[must_use]
    pub fn last_refreshed(&self) -> &[u32] {
        self.region.refreshed()
    }

    /// Bytes segurados. A sonda de memória o soma: o custo do GESTO não pode
    /// ficar fora da conta só por ser transitório.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        let v3 = size_of::<[f32; 3]>();
        (self.slot.capacity() + self.stamp.capacity()) * size_of::<u32>()
            + (self.touched.capacity() + self.footprint.capacity() + self.moved.capacity())
                * size_of::<u32>()
            + (self.base_pos.capacity() + self.base_nrm.capacity() + self.target.capacity()) * v3
            + (self.base_mask.capacity() + self.accum.capacity()) * size_of::<f32>()
            + self.query.capacity_bytes()
            + self.region.capacity_bytes()
    }

    /// Aplica um dab, **com a simetria expandida aqui e em lugar nenhum mais**.
    ///
    /// Devolve quantos vértices se moveram. As cópias espelhadas caem no mesmo
    /// núcleo, então um verbo novo herda simetria de graça — a lição literal do
    /// `stamp_dabs_inner` do Painter 2D.
    ///
    /// ⚠️ O plano do espelho passa pela **origem do mundo**. Quando uma malha
    /// ganhar `Transform` próprio (W8), é o frame local dela que entra aqui —
    /// e será uma mudança nesta função, não nos doze verbos.
    pub fn dab(&mut self, mesh: &mut Mesh, brush: &Brush, dab: &Dab, sym: Symmetry) -> usize {
        let (signs, n) = sym.signs();
        let mut total = 0;
        for s in signs.iter().take(n) {
            let mirrored = Dab {
                center: [
                    dab.center[0] * s[0],
                    dab.center[1] * s[1],
                    dab.center[2] * s[2],
                ],
                ..*dab
            };
            total += self.dab_core(mesh, brush, &mirrored);
        }
        total
    }

    fn dab_core(&mut self, mesh: &mut Mesh, brush: &Brush, dab: &Dab) -> usize {
        self.moved.clear();
        if dab.radius <= 0.0 || brush.strength <= 0.0 || dab.pressure <= 0.0 {
            return 0;
        }
        // A pegada sai das posições VIVAS: o pincel age onde a superfície está
        // agora, não onde ela estava no pen-down. É só o ALVO que vem do `pre`.
        mesh.verts_in_sphere(dab.center, dab.radius, &mut self.query, &mut self.footprint);
        if self.footprint.is_empty() {
            return 0;
        }
        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            self.capture(mesh, v);
        }

        let plane = self.fit_plane(brush, dab);
        let reach = brush.reach(dab.radius);
        let inv_r = 1.0 / dab.radius;
        let intensity = brush.strength * dab.pressure.clamp(0.0, 1.0);
        // ⚠️ **O verbo de MÁSCARA não é freado pela máscara**, e o gate pegou
        // isto: com `w = falloff·(1 − mask)`, uma região totalmente mascarada
        // zerava o peso de qualquer dab — inclusive o que a limparia. A máscara
        // ficava permanente, e o botão "Clear" seria um controle morto que
        // *parece* funcionar em toda região parcial. Ela gateia quem MOVE
        // GEOMETRIA; quem edita o próprio canal a lê como dado, não como freio.
        let gated_by_mask = !brush.verb.paints_mask();

        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let base = self.base_pos[s];
            let d = [
                base[0] - dab.center[0],
                base[1] - dab.center[1],
                base[2] - dab.center[2],
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            // A máscara é lida do estado CONGELADO: um traço de Mask não pode
            // mudar o quanto ele próprio já mascarou no meio do gesto.
            let keep = if gated_by_mask {
                1.0 - self.base_mask[s]
            } else {
                1.0
            };
            let w = brush.falloff.weight(dist * inv_r) * intensity * keep;
            // `<=` e não `<`: um dab que EMPATA não vence. A diferença não é de
            // resultado (o alvo recomputado seria o mesmo) — é de TRABALHO: com
            // `<`, re-carimbar a mesma lista de dabs reescreveria a pegada
            // inteira e a mandaria para o refit do octree e para o upload
            // incremental, todo frame, sem um pixel mudar.
            if w <= self.accum[s] {
                continue;
            }
            self.accum[s] = w;
            self.target[s] = self.compute_target(mesh, brush, dab, &plane, reach, v, s);
            self.moved.push(v);
        }

        if self.moved.is_empty() {
            return 0;
        }
        if brush.verb.paints_mask() {
            self.apply_mask(mesh, brush);
            // Nada de geometria mudou: quem lê `last_refreshed` tem de ver
            // vazio, não a lista do dab anterior.
            self.region.forget();
        } else {
            self.apply_positions(mesh);
            mesh.refresh_region(&self.moved, &mut self.region);
        }
        self.moved.len()
    }

    /// Guarda o `pre` de um vértice, se ainda não guardou. Idempotente.
    fn capture(&mut self, mesh: &Mesh, v: u32) {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            return;
        }
        self.stamp[vi] = self.epoch;
        self.slot[vi] = self.touched.len() as u32;
        self.touched.push(v);
        self.base_pos.push(mesh.positions()[vi]);
        self.base_nrm.push(mesh.normals()[vi]);
        self.base_mask
            .push(mesh.masks().map_or(DEFAULT_MASK, |m| m[vi]));
        self.accum.push(0.0);
        // Alvo neutro: sem dab que vença, `lerp(base, base, 0)` não move nada.
        self.target.push(mesh.positions()[vi]);
    }

    /// A posição de `v` ANTES do traço.
    ///
    /// Um vértice não capturado nunca foi escrito por este traço, logo a posição
    /// viva dele **é** o `pre`. É isso que torna o Smooth barato: ele lê o anel
    /// inteiro sem obrigar a captura de vizinhos que ninguém vai mover.
    fn base_pos_of(&self, mesh: &Mesh, v: u32) -> [f32; 3] {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            self.base_pos[self.slot[vi] as usize]
        } else {
            mesh.positions()[vi]
        }
    }

    fn fit_plane(&self, brush: &Brush, dab: &Dab) -> PlaneFit {
        let inv_r = 1.0 / dab.radius;
        let mut acc_p = [0.0f64; 3];
        let mut acc_n = [0.0f64; 3];
        let mut sum = 0.0f64;
        for &v in &self.footprint {
            let s = self.slot[v as usize] as usize;
            let p = self.base_pos[s];
            let n = self.base_nrm[s];
            let d = [
                p[0] - dab.center[0],
                p[1] - dab.center[1],
                p[2] - dab.center[2],
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            // A ponderação é só o FALLOFF: o plano descreve a superfície sob o
            // pincel, e força/pressão/máscara dizem o quanto agir sobre ela, não
            // que forma ela tem.
            let w = f64::from(brush.falloff.weight(dist * inv_r));
            if w <= 0.0 {
                continue;
            }
            sum += w;
            for k in 0..3 {
                acc_p[k] += f64::from(p[k]) * w;
                acc_n[k] += f64::from(n[k]) * w;
            }
        }
        if sum <= 0.0 {
            // Pegada inteira na borda do falloff (Sharper com raio grande, por
            // exemplo). O centro do dab e a normal dele são a melhor resposta
            // disponível, e ela nunca é usada para mais que um plano degenerado.
            return PlaneFit {
                point: dab.center,
                normal: [0.0, 1.0, 0.0],
            };
        }
        let inv = 1.0 / sum;
        let mut point = [0.0f32; 3];
        let mut normal = [0.0f32; 3];
        for k in 0..3 {
            point[k] = (acc_p[k] * inv) as f32;
            normal[k] = (acc_n[k] * inv) as f32;
        }
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if len > 1e-12 {
            for n in &mut normal {
                *n /= len;
            }
        } else {
            // Normais que se cancelam (uma dobra fechada sob o pincel): sem
            // direção defensável, o plano vira o do próprio dab.
            normal = [0.0, 1.0, 0.0];
        }
        // O offset move o PLANO, não os vértices — é o knob que faz do Flatten
        // um Clay sem um segundo verbo.
        let off = brush.plane_offset * dab.radius;
        for k in 0..3 {
            point[k] += normal[k] * off;
        }
        PlaneFit { point, normal }
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_target(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        plane: &PlaneFit,
        reach: f32,
        v: u32,
        s: usize,
    ) -> [f32; 3] {
        let base = self.base_pos[s];
        let n_area = plane.normal;
        match brush.verb {
            Verb::Draw => add(base, n_area, reach),
            Verb::Inflate => add(base, self.base_nrm[s], reach),
            Verb::Smooth => self.neighbour_average(mesh, v, base),
            Verb::Sharpen => {
                let avg = self.neighbour_average(mesh, v, base);
                // Reflete a média através do próprio vértice: o oposto exato do
                // Smooth, com a mesma magnitude.
                [
                    base[0] * 2.0 - avg[0],
                    base[1] * 2.0 - avg[1],
                    base[2] * 2.0 - avg[2],
                ]
            }
            Verb::Flatten => project(base, plane),
            Verb::Fill => {
                if signed_distance(base, plane) < 0.0 {
                    project(base, plane)
                } else {
                    base
                }
            }
            Verb::Scrape => {
                if signed_distance(base, plane) > 0.0 {
                    project(base, plane)
                } else {
                    base
                }
            }
            // Achata E acrescenta: o barro que se adiciona, sem uma constante
            // escondida — o `reach` é o mesmo knob de todo verbo aditivo.
            Verb::Clay => add(project(base, plane), n_area, reach),
            Verb::Pinch => add_vec(base, tangential(base, dab.center, n_area), 1.0),
            Verb::Magnify => add_vec(base, tangential(base, dab.center, n_area), -1.0),
            Verb::Crease => {
                let t = tangential(base, dab.center, n_area);
                // Aperta lateralmente E cava: o `-reach` é o que faz um vinco
                // ser um vinco. Com `invert`, `reach` já chega negativo e o
                // mesmo verbo levanta uma crista.
                add(add_vec(base, t, brush.pinch), n_area, -reach)
            }
            // O alvo de posição de um verbo de máscara é o próprio lugar: ele
            // não move geometria, e `apply_mask` é quem escreve o canal dele.
            Verb::Mask => base,
        }
    }

    /// A média das posições **congeladas** do anel de `v`.
    ///
    /// Ler o `pre` e não o vivo é o que torna o Smooth idempotente: um traço que
    /// passa duas vezes no mesmo lugar suaviza uma vez, e a superfície não
    /// derrete enquanto o artista segura o botão parado.
    fn neighbour_average(&self, mesh: &Mesh, v: u32, base: [f32; 3]) -> [f32; 3] {
        let ring = mesh.adjacency().vert_verts.neighbours(v as usize);
        if ring.is_empty() {
            return base;
        }
        let mut acc = [0.0f32; 3];
        for &nb in ring {
            let p = self.base_pos_of(mesh, nb);
            for k in 0..3 {
                acc[k] += p[k];
            }
        }
        let inv = 1.0 / ring.len() as f32;
        [acc[0] * inv, acc[1] * inv, acc[2] * inv]
    }

    fn apply_positions(&self, mesh: &mut Mesh) {
        let out = mesh.positions_mut();
        for &v in &self.moved {
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let (b, t, a) = (self.base_pos[s], self.target[s], self.accum[s]);
            out[vi] = [
                b[0] + (t[0] - b[0]) * a,
                b[1] + (t[1] - b[1]) * a,
                b[2] + (t[2] - b[2]) * a,
            ];
        }
    }

    /// A MESMA lei, no canal da máscara: `lerp(base, alvo, accum)`, onde o alvo
    /// é `1` (mascarar) ou `0` (limpar). Um verbo, uma aritmética.
    fn apply_mask(&self, mesh: &mut Mesh, brush: &Brush) {
        let goal = if brush.invert { 0.0 } else { 1.0 };
        let out = mesh.masks_mut();
        for &v in &self.moved {
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let (b, a) = (self.base_mask[s], self.accum[s]);
            out[vi] = b + (goal - b) * a;
        }
    }
}

fn add(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

fn add_vec(p: [f32; 3], v: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + v[0] * k, p[1] + v[1] * k, p[2] + v[2] * k]
}

fn signed_distance(p: [f32; 3], plane: &PlaneFit) -> f32 {
    (p[0] - plane.point[0]) * plane.normal[0]
        + (p[1] - plane.point[1]) * plane.normal[1]
        + (p[2] - plane.point[2]) * plane.normal[2]
}

fn project(p: [f32; 3], plane: &PlaneFit) -> [f32; 3] {
    let d = signed_distance(p, plane);
    add(p, plane.normal, -d)
}

/// A parte de `centro − p` que corre ao longo da superfície.
///
/// ⚠️ **Divergência deliberada do Blender**, que faz o Pinch mover para o centro
/// em 3D (`co += (center − co) * fade`). Aquele vetor tem componente ao longo da
/// normal, então o Pinch dele também ACHATA um pouco — dois efeitos num knob. Ao
/// remover a componente normal, apertar é apertar; quem quer achatar tem quatro
/// verbos para isso.
fn tangential(p: [f32; 3], center: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = [center[0] - p[0], center[1] - p[1], center[2] - p[2]];
    let along = d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2];
    [
        d[0] - normal[0] * along,
        d[1] - normal[1] * along,
        d[2] - normal[2] * along,
    ]
}

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod tests;
