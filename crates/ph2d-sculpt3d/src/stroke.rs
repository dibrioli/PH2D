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
    /// A direção do OLHO no instante do pick — do olho para a superfície,
    /// unitária. É o `dir` do raio que produziu o `center`.
    ///
    /// ⚠️ **Ela é do DAB e não do pincel**, e a razão é a simetria: no original
    /// o espelho é aplicado ao raio **antes** de a direção ser computada
    /// (`Picking.js:211-223`), então a cópia espelhada tem o olho espelhado
    /// junto. Guardá-la no `Brush` daria a MESMA direção às duas cópias, e a
    /// metade espelhada passaria a ser ajustada por um olho que não é o dela.
    ///
    /// ⚠️ **É argumento obrigatório do [`Dab::at`], nunca um builder opcional.**
    /// A lição é do `with_arc_len` do Painter 2D: um campo opcional de dab
    /// chegava em 2 de 7 rotas, e nas outras 5 a feature simplesmente não
    /// acontecia — em silêncio, com o painel dizendo que sim.
    pub eye: [f32; 3],
    /// **O gesto**: quanto o dedo puxou desde o pen-down, em MUNDO.
    ///
    /// ⚠️ **É o deslocamento TOTAL, não o incremento do evento** — e essa é a
    /// escolha que mantém a lei do traço de pé. O alvo do Grab é
    /// `base + pull·falloff`, função do `pre` congelado: puxar de volta devolve
    /// o barro ao lugar, re-carimbar a mesma lista não intensifica nada, e o
    /// undo continua sendo o `base`. Com incrementos, cada um deles seria uma
    /// soma sobre o resultado do anterior — o produto sobre a lista de dabs que
    /// este módulo inteiro existe para não ter.
    ///
    /// Só os verbos que respondem `true` a [`Brush::verb`]`.pulls()` o leem;
    /// para os outros ele é zero e inerte.
    pub pull: [f32; 3],
}

impl Dab {
    /// Um dab de pressão cheia, visto de `eye`.
    #[must_use]
    pub fn at(center: [f32; 3], radius: f32, eye: [f32; 3]) -> Self {
        Self {
            center,
            radius,
            pressure: 1.0,
            eye,
            pull: [0.0; 3],
        }
    }

    /// Um dab que **PUXA** — o gesto do Grab.
    ///
    /// ⚠️ Construtor irmão em vez de um builder opcional: um `with_pull()` é
    /// exatamente a forma que o `with_arc_len` do Painter 2D tinha quando ele
    /// chegava em 2 de 7 rotas e a feature simplesmente não acontecia nas outras
    /// cinco, em silêncio. Quem puxa pede este; quem não puxa não o vê.
    #[must_use]
    pub fn pulling(center: [f32; 3], radius: f32, eye: [f32; 3], pull: [f32; 3]) -> Self {
        Self {
            pull,
            ..Self::at(center, radius, eye)
        }
    }
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
    /// O último dab pintou MÁSCARA? Decide de qual janela a GPU precisa — ver
    /// [`SculptStroke::last_gpu_dirty`]. Um bool escrito no mesmo `if` que já
    /// separa os dois braços; derivá-lo do `Brush` no chamador seria pedir a ele
    /// que soubesse a regra.
    last_paints_mask: bool,
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

    /// Os vértices que a GPU precisa **RE-LER** depois do último dab, em
    /// QUALQUER canal — a janela do upload incremental.
    ///
    /// ⚠️ **Não é o mesmo que [`Self::last_refreshed`], e a diferença é uma
    /// feature inteira.** Aquele responde *de quem eu recomputei a NORMAL*, e um
    /// traço de máscara não move geometria: ele escreve o canal de máscara e
    /// **esquece a região de propósito**. Um chamador que subisse `refreshed`
    /// não subiria byte nenhum de um traço de Mask — a máscara ficaria invisível
    /// na GPU, agora por um segundo motivo, com todos os gates de CPU verdes.
    ///
    /// Os dois casos são exclusivos por construção (o dab ou pinta máscara, ou
    /// move geometria), então a resposta é uma escolha e nunca uma união.
    #[must_use]
    pub fn last_gpu_dirty(&self) -> &[u32] {
        if self.last_paints_mask {
            &self.moved
        } else {
            self.region.refreshed()
        }
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
            let fall = brush.falloff.weight(dist * inv_r);
            // ⚠️ **O `w` fica VERBATIM — mesma ordem, mesmos bits.** A forma
            // "natural" seria derivar um do outro (`w = shape * intensity`), e
            // ela **re-associa** o produto de `(falloff × intensity) × keep`
            // para `(falloff × keep) × intensity`: medido, **30,4% dos triplos
            // divergem**, até ~1 ulp. Em `keep == 1.0` EXATO — o caso comum,
            // porque `DEFAULT_MASK` é 0 — a divergência é ZERO, mas o preço de
            // não arriscar os doze verbos é **uma multiplicação**.
            let w = fall * intensity * keep;
            // A metade SEM intensidade: é ela que o Crease eleva, porque no
            // original o expoente cai sobre `curva × máscara × alpha` e a
            // intensidade entra depois, linear nos dois termos.
            let shape = fall * keep;
            // `<=` e não `<`: um dab que EMPATA não vence. A diferença não é de
            // resultado (o alvo recomputado seria o mesmo) — é de TRABALHO: com
            // `<`, re-carimbar a mesma lista de dabs reescreveria a pegada
            // inteira e a mandaria para o refit do octree e para o upload
            // incremental, todo frame, sem um pixel mudar.
            // ⚠️ **Quem PUXA não pode ser freado pelo early-out**: a pegada
            // do Grab é presa no pen-down, então o peso de cada vértice é o do
            // primeiro dab e nunca mais sobe. Sem esta exceção o barro andaria
            // UM evento e pararia, com o cursor seguindo em frente — e o alvo
            // que mudou não é o peso, é o `pull`. Ver `Verb::pulls`.
            if w <= self.accum[s] && !brush.verb.pulls() {
                continue;
            }
            self.accum[s] = w;
            self.target[s] = self.compute_target(mesh, brush, dab, &plane, reach, shape, v, s);
            self.moved.push(v);
        }

        if self.moved.is_empty() {
            return 0;
        }
        self.last_paints_mask = brush.verb.paints_mask();
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

/// **O ALVO de cada verbo**, e o plano que quatro deles ajustam. Filho para
/// alcançar o `pre` congelado; o corte é *a LEI do traço* (aqui) contra *para
/// onde cada verbo aponta* (lá).
#[path = "stroke_target.rs"]
mod target;

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod tests;
