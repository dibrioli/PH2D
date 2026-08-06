//! **O ALPHA** — o padrão que decide ONDE, dentro da pegada, o verbo age.
//!
//! Um pincel de escultura sem alpha deposita a mesma forma lisa em toda parte, e
//! é por isso que uma peça acabada num app que os tem (ZBrush, Blender, Nomad)
//! lê como pele, escama ou casca, e a nossa lê como barro. O alpha é um número
//! em `[0, 1]` que **multiplica o falloff**: onde ele vale 1 o verbo age cheio,
//! onde vale 0 ele não toca o vértice.
//!
//! ⚠️ **Ele multiplica o FALLOFF, não o peso.** A diferença aparece no Crease,
//! cujo expoente cai sobre `curva × máscara × alpha` — e é assim no original.
//! Multiplicar o peso já formado deixaria o alpha *fora* do expoente, e o verbo
//! afiaria a máscara sem afiar o padrão.
//!
//! # O MAPEAMENTO, e por que ele é 3D
//!
//! O [[04.1-Pinceis]] previa *"uma textura projetada no frame do dab (tangente ×
//! bitangente × normal), com rake opcional"* — o **View Plane** do Blender. Este
//! módulo constrói o outro mapeamento da mesma lista, o **3D**: o padrão é uma
//! função pura da POSIÇÃO do vértice, e não há frame nenhum.
//!
//! Não é preferência, é o que a lei do traço desta casa exige. Nosso envelope é
//! `accum ← max(accum, w)` sobre a lista de dabs, e com o espaçamento em `0,15·r`
//! um vértice cai sob **dezenas** de dabs. Num mapeamento projetado cada um deles
//! amostraria o padrão num `(u,v)` diferente, o `max` tomaria o maior de dezenas
//! de amostras, e o padrão **lavaria** até a sua envoltória superior — o pincel
//! ficaria mais forte, não texturizado. Num mapeamento 3D todos os dabs
//! concordam sobre o valor daquele vértice, então o `max` de valores iguais é o
//! valor: **o padrão sobrevive à lei intacto**, e as três propriedades do traço
//! (independência de espaçamento, idempotência, undo trivial) continuam valendo
//! sem uma linha a mais.
//!
//! ⚠️ **A posição é a CONGELADA no pen-down**, não a viva — senão o padrão
//! escorregaria enquanto o próprio traço move a superfície, e dois dabs
//! discordariam de novo. É a mesma frase que o `stroke.rs` já diz sobre a
//! distância no envelope, e aqui ela cai de graça porque o `pre` já está lá.
//!
//! ⚠️ **O padrão é colado ao ESPAÇO DO OBJETO, não à superfície.** Deformar muito
//! a malha faz a superfície deslizar por baixo do padrão — o comportamento do
//! mapeamento 3D do Blender, e é ele que também garante que girar o objeto na
//! cena não faz a pele nadar. A alternativa (colar ao índice do vértice) não
//! sobrevive a uma subdivisão.
//!
//! # E o que ficou de FORA, com o motivo
//!
//! Os seis padrões daqui são **isotrópicos**: lêem igual de qualquer direção,
//! que é o que os deixa viver sem frame. Um padrão **direcional** — um tecido, um
//! carimbo, um símbolo, qualquer IMAGEM — é precisamente o caso que *quer* a
//! tangente e o rake do `04.1`, porque ele tem um "para cima". Então o desenho
//! daquela nota não está errado: ele é a resposta a outra pergunta, e a porta
//! para ela é uma variante nova deste enum, não uma reescrita desta.
//!
//! ⚠️ E não há imagem para carregar hoje — o mesmo fato que fez os matcaps serem
//! analíticos. Sintetizar uma textura para depois amostrá-la seria a mesma
//! fórmula avaliada duas vezes.
//!
//! # HR-5
//!
//! Zero transcendental: hash inteiro, `floor`, `sqrt` (instrução de hardware) e
//! aritmética. Esta crate é `libm`-free e continua sendo.

/// **OS SEIS PADRÕES**, na ordem em que a UI os lista.
///
/// Cada um é nomeado pelo que ele PARECE, não pelo uso: o mesmo Worley que faz
/// escama faz casco de tartaruga, e um nome de intenção envelheceria no primeiro
/// artista que o usasse para outra coisa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alpha {
    /// fBm de ruído de valor, três oitavas. Irregularidade geral — rocha, massa.
    Noise,
    /// Pontos redondos isolados. Pele.
    Pores,
    /// Domos preenchendo as células, com sulco na fronteira. Réptil, casco.
    Scales,
    /// Só as fronteiras das células, finas. Terra seca, casca, esmalte trincado.
    Cracks,
    /// Mosqueado fino de alto contraste. O *chatter* de uma superfície trabalhada.
    Grain,
    /// fBm dobrado: cristas afiadas em vez de ondas. Ruga, estrato, casca de árvore.
    Ridges,
}

/// O tamanho de feature com que um alpha nasce **quando ninguém perguntou ao
/// modelo** — ver [`recommended_scale`], que é quem o produto usa.
///
/// ⚠️ **O trabalho real desta constante é ser SENTINELA**, não ser um bom
/// tamanho: é comparando contra ela que o painel sabe *"o artista ainda não
/// escolheu"* e pode semear a recomendação — o mesmo papel do `Smooth de fábrica`
/// no `arm_inflate_defaults` do Painter.
///
/// ⚠️ **Ela já foi o default de verdade, e o smoke a reprovou** (*"os poros são
/// gigantescos"*, Enio, 2026-08-05). O número era `0,25`, escolhido como *a
/// escala mais fina que a malha da cena RESOLVE* — e resolver não é parecer:
/// medido, `0,25` põe **oito** features atravessando uma esfera unitária, o que
/// lê como cratera. **Uma escala absoluta não significa nada sem o tamanho do
/// modelo**, e era esse o defeito.
pub const DEFAULT_ALPHA_SCALE: f32 = 0.06;

/// O piso da pista de escala.
///
/// Pela lei das dez arestas ele serve uma malha de aresta `0,0002` — três
/// subdivisões abaixo do que a cena mais densa deste módulo abre. Um piso mais
/// alto tiraria do artista a única faixa em que uma malha de milhões de vértices
/// vale a pena.
pub const MIN_ALPHA_SCALE: f32 = 0.002;

/// O teto da pista: quatro features atravessando um modelo de tamanho 2.
///
/// ⚠️ Acima disto o padrão deixa de ser padrão e vira um multiplicador quase
/// constante — o regime que o smoke reprovou. A pista **não** precisa alcançar
/// "uma célula do tamanho da peça": esse valor não desenha textura nenhuma.
pub const MAX_ALPHA_SCALE: f32 = 0.5;

/// **Quantas features o padrão atravessa o modelo** na recomendação.
///
/// ⚠️ **Medido, não escolhido** (`measure_how_many_features_cross_the_model`):
///
/// | escala numa esfera unitária | features | malha para resolver |
/// |---|---|---|
/// | 0,25 | **8** — a cratera que o smoke reprovou | 17 k verts |
/// | 0,12 | 17 | 86 k |
/// | **0,06** | **33 — textura** | 426 k |
/// | 0,03 | 67 — pele | 959 k |
const FEATURES_ACROSS: f32 = 33.0;

/// **A lei das dez arestas**: quantas arestas uma feature precisa medir para ser
/// amostrada como padrão em vez de como chuvisco. Ver [`DEFAULT_ALPHA_SCALE`].
const EDGES_PER_FEATURE: f32 = 10.0;

/// Quantos vértices a estimativa de aresta amostra.
///
/// ⚠️ **Amostra, e não a mediana verdadeira**, porque o retrato do painel a chama
/// a CADA QUADRO: sobre 425 k vértices a mediana exata ordena ~1,2 M
/// comprimentos, e isso é um imposto por frame para um número que só é lido num
/// clique.
///
/// **MEDIDO** (`measure_the_recommendation_per_frame`), e o número foi escolhido
/// pela varredura em vez de pelo conforto: com 2048 amostras o custo é
/// **0,11 ms/quadro**; com **384** ele cai para **0,017** — e a sonda de viés
/// (`measure_the_edge_estimator_bias`) mostra a estimativa em **1,00×** da
/// mediana verdadeira nas duas. Seis vezes mais barato pela mesma resposta.
///
/// ⚠️ **E ele é PLANO na malha** (0,017 a 153 k e a 425 k) — que é o requisito de
/// projeto, não um bônus: um seed cujo custo crescesse com o modelo seria um
/// imposto que aparece justamente na peça pesada.
const EDGE_SAMPLES: usize = 384;

/// **A escala que ESTE modelo comporta** — o seed do `Alpha Scale`.
///
/// Duas restrições, e o resultado é a mais grossa das duas porque as duas são
/// verdadeiras ao mesmo tempo:
///
/// * **o LOOK** — `maior lado ÷ 33`, que é o que faz um padrão parecer textura
///   em vez de cratera, em qualquer tamanho de modelo;
/// * **a REPRESENTABILIDADE** — `10 × aresta`, a lei das dez arestas: mais fino
///   que isso a malha não amostra o padrão, ela o pica.
///
/// ⚠️ **É por isso que uma escala absoluta precisava de um seed.** O número certo
/// é em unidades de objeto (um poro tem o tamanho de um poro, e trocar o pincel
/// não pode mudá-lo), mas *qual* número depende de duas coisas que só o modelo
/// sabe. Um literal no código acerta uma esfera unitária de uma densidade e erra
/// todo o resto — foi exatamente o que o smoke pegou.
///
/// ⚠️ **Numa malha grossa a segunda restrição VENCE**, e isso é honesto: o padrão
/// sai grosso porque a malha não comporta outro. A cura é subdividir, e é o que
/// o smoke manda fazer — o mesmo fato que faz um escultor de ZBrush subdividir
/// antes de pegar um alpha.
///
/// ⚠️ **E há um regime em que NEM o teto basta**, achado pelo gate: numa esfera
/// 24×36 a aresta mede `0,131`, então a lei das dez arestas pediria `1,31` —
/// mais que o modelo inteiro. Ali não existe escala que sirva: a malha **não
/// carrega padrão nenhum**. A recomendação pousa no teto, que é o estado
/// reconhecível; devolver um valor no meio fingiria que resolveu.
#[must_use]
pub fn recommended_scale(mesh: &ph2d_mesh::Mesh) -> f32 {
    let b = mesh.bounds();
    // ⚠️ **O MAIOR LADO, e não a diagonal da caixa** — e a diferença é um fator
    // `√3` que eu shipei errado uma vez. A tabela que fixou `FEATURES_ACROSS`
    // conta features atravessando uma esfera unitária, ou seja sobre o
    // **diâmetro** (2); a diagonal da caixa dela mede `3,464`, então o mesmo
    // número devolvia uma feature 1,73× maior que a medida. Duas réguas para uma
    // grandeza é a doença de sempre, e aqui ela sai como *"os poros continuam
    // grandes"* depois de um conserto que parecia certo.
    let span = (b.max[0] - b.min[0])
        .max(b.max[1] - b.min[1])
        .max(b.max[2] - b.min[2]);
    let look = span / FEATURES_ACROSS;
    let floor = EDGES_PER_FEATURE * sampled_edge(mesh);
    look.max(floor).clamp(MIN_ALPHA_SCALE, MAX_ALPHA_SCALE)
}

/// A aresta mediana de uma AMOSTRA de vértices — ver [`EDGE_SAMPLES`].
///
/// ⚠️ **Duas armadilhas de amostragem, as duas MEDIDAS e as duas curadas aqui.**
///
/// **(1) O passo constante ressoa com a malha.** A primeira versão andava de
/// `len / 2048` em `len / 2048`, e numa esfera UV — cujos vértices são guardados
/// por linhas de latitude — esse passo entra em ressonância com o comprimento da
/// linha: a sonda `measure_the_edge_estimator_bias` mediu o **MESMO** valor
/// (`0,0105`) para duas malhas de 153 k e 734 k vértices, um viés de **2,34×** na
/// maior. O passo agora é o da razão áurea sobre o índice, que não tem relação
/// nenhuma com a ordem em que a malha guarda os vértices.
///
/// **(2) O PRIMEIRO vizinho não é uma aresta típica.** Ele é o que o CSR pôs
/// primeiro, e numa esfera UV isso é sistematicamente o meridiano ou o raio do
/// polo. A amostra toma o **anel inteiro** de cada vértice sorteado.
fn sampled_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let pos = mesh.positions();
    let ring = mesh.adjacency();
    if pos.is_empty() {
        return 0.0;
    }
    let n = pos.len();
    let mut lens: Vec<f32> = Vec::with_capacity(EDGE_SAMPLES * 6);
    let mut cursor: u64 = 0;
    for _ in 0..EDGE_SAMPLES.min(n) {
        cursor = cursor.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let v = ((cursor >> 33) as usize) % n;
        let a = pos[v];
        for &nb in ring.vert_verts.neighbours(v) {
            let b = pos[nb as usize];
            let e = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            lens.push(e[2].mul_add(e[2], e[0].mul_add(e[0], e[1] * e[1])).sqrt());
        }
    }
    if lens.is_empty() {
        return 0.0;
    }
    lens.sort_by(f32::total_cmp);
    lens[lens.len() / 2]
}

impl Alpha {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 6] = [
        Self::Noise,
        Self::Pores,
        Self::Scales,
        Self::Cracks,
        Self::Grain,
        Self::Ridges,
    ];

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Noise => "Noise",
            Self::Pores => "Pores",
            Self::Scales => "Scales",
            Self::Cracks => "Cracks",
            Self::Grain => "Grain",
            Self::Ridges => "Ridges",
        }
    }

    /// **A frequência RELATIVA do padrão**, em células por unidade de escala.
    ///
    /// ⚠️ Ela existe porque a pista de escala é UMA e os padrões têm densidades
    /// naturais diferentes: o Grain é um chuvisco e as Scales são placas. Sem
    /// este multiplicador o artista teria de re-afinar a escala a cada troca de
    /// padrão, e a pista significaria uma coisa diferente por chip — que é a
    /// definição de um controle que não se aprende.
    fn frequency(self) -> f32 {
        match self {
            // A oitava base do fBm já é a mais grossa das três.
            Self::Noise | Self::Ridges => 1.0,
            Self::Pores | Self::Scales | Self::Cracks => 1.0,
            // Um chuvisco: três vezes mais fino que os demais na mesma escala.
            Self::Grain => 3.0,
        }
    }

    /// **O PESO em `[0, 1]`** no ponto `p` (espaço do objeto), para um tamanho de
    /// feature `scale` em unidades de objeto.
    ///
    /// **Porta única** — o motor, o gate e a sonda perguntam a esta função.
    ///
    /// ⚠️ **A peneira de não-finito é a mesma do [`crate::Falloff::weight`]**, e
    /// pelo mesmo motivo: um `NaN` que escorre daqui vira peso `NaN` num vértice,
    /// e uma malha inteira sai `NaN` a partir de um ponto ruim em algum lugar.
    #[must_use]
    pub fn weight_at(self, p: [f32; 3], scale: f32) -> f32 {
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            return 0.0;
        }
        let s = scale.clamp(MIN_ALPHA_SCALE, MAX_ALPHA_SCALE);
        let k = self.frequency() / s;
        let q = [p[0] * k, p[1] * k, p[2] * k];

        let w = match self {
            Self::Noise => {
                // Centrado em 0,5 e esticado: o fBm cru se aperta em torno da
                // média e sairia como um cinza uniforme.
                contrast(fbm(q), 0.18, 0.82)
            }
            Self::Ridges => {
                // O dobramento (`1 − |2f − 1|`) troca ondas por CRISTAS: onde o
                // fBm cruza a média nasce um vinco de derivada descontínua, que
                // é exatamente o que uma ruga é.
                let f = fbm(q);
                let folded = 1.0 - (2.0f32.mul_add(f, -1.0)).abs();
                contrast(folded, 0.45, 0.98)
            }
            Self::Grain => contrast(value_noise(q), 0.42, 0.62),
            Self::Pores => {
                // Um disco em torno de cada ponto-semente, e nada entre eles.
                let (f1, _) = worley(q);
                1.0 - smoothstep(PORE_CORE, PORE_EDGE, f1)
            }
            Self::Scales => {
                // `f2 − f1` é grande no miolo da célula e vai a ZERO na fronteira
                // dela — é a distância à parede, de graça. Isto é o domo.
                let (f1, f2) = worley(q);
                smoothstep(0.0, SCALE_RIM, f2 - f1)
            }
            Self::Cracks => {
                // O complemento exato do Scales, com a parede muito mais fina:
                // uma trinca é a fronteira, não o interior.
                let (f1, f2) = worley(q);
                1.0 - smoothstep(0.0, CRACK_WIDTH, f2 - f1)
            }
        };

        if w.is_finite() {
            w.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Raio do miolo cheio de um poro, em células.
///
/// ⚠️ **Os dois números saíram de MEDIÇÃO e a primeira escolha estava errada por
/// quase 4×** (`0,10` / `0,30`): a sonda de cobertura deu **média 0,039, com
/// 1,1% dos pontos acima de 0,9** — um pincel que o artista leria como
/// *quebrado*, não como texturizado. A causa é geométrica e vale para todo
/// padrão daqui: as sementes são pontos no **VOLUME**, e a superfície corta esse
/// volume — a fração que ela vê é a fração VOLUMÉTRICA, `(4/3)π r³`, que cai com
/// o CUBO. Um raio que pareceria generoso num desenho 2D é diminuto em 3D.
const PORE_CORE: f32 = 0.22;
/// Raio onde o poro morre, em células.
const PORE_EDGE: f32 = 0.42;
/// Largura do sulco entre duas escamas, na régua de `f2 − f1`.
const SCALE_RIM: f32 = 0.30;
/// Largura de uma trinca, na mesma régua. Bem menor: é o que a faz ser um traço.
const CRACK_WIDTH: f32 = 0.08;

/// `smoothstep` — o degrau C¹ que todo remapeamento daqui usa.
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * 2.0f32.mul_add(-t, 3.0)
}

/// Estica a faixa `[lo, hi]` para `[0, 1]`, com ombro C¹.
///
/// É o `smoothstep` com outro nome, e o nome é o ponto: aqui ele não é um degrau
/// espacial, é o CONTRASTE de um padrão. Os dois usos merecem duas leituras.
fn contrast(v: f32, lo: f32, hi: f32) -> f32 {
    smoothstep(lo, hi, v)
}

/// Hash inteiro de uma célula da grade → `u32` bem misturado.
///
/// ⚠️ **Os três eixos entram por multiplicadores DIFERENTES antes do `xor`**: com
/// o mesmo multiplicador, `(a, b, c)` e `(b, a, c)` colidiriam, e o padrão sairia
/// espelhado em torno da diagonal — visível como uma simetria que ninguém
/// autorou. O avalanche final é o do `splitmix`.
fn hash3(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

/// `u32` → `[0, 1)`, pelos 24 bits ALTOS.
///
/// ⚠️ Os altos e não os baixos: um multiplicador ímpar preserva perfeitamente os
/// bits baixos de uma soma, então `hash & 0xFF` de células vizinhas anda em
/// passos regulares — o padrão sairia listrado.
fn unit(h: u32) -> f32 {
    (h >> 8) as f32 * (1.0 / 16_777_216.0)
}

/// Ruído de valor em `[0, 1]`: hash nos vértices da grade, trilinear entre eles,
/// com a fade cúbica que tira a assinatura da grade das derivadas.
fn value_noise(p: [f32; 3]) -> f32 {
    let f = [p[0].floor(), p[1].floor(), p[2].floor()];
    let i = [f[0] as i32, f[1] as i32, f[2] as i32];
    let mut u = [0.0f32; 3];
    for a in 0..3 {
        let t = (p[a] - f[a]).clamp(0.0, 1.0);
        u[a] = t * t * 2.0f32.mul_add(-t, 3.0);
    }
    let c = |dx: i32, dy: i32, dz: i32| unit(hash3(i[0] + dx, i[1] + dy, i[2] + dz));
    let lerp = |a: f32, b: f32, t: f32| (b - a).mul_add(t, a);

    let x00 = lerp(c(0, 0, 0), c(1, 0, 0), u[0]);
    let x10 = lerp(c(0, 1, 0), c(1, 1, 0), u[0]);
    let x01 = lerp(c(0, 0, 1), c(1, 0, 1), u[0]);
    let x11 = lerp(c(0, 1, 1), c(1, 1, 1), u[0]);
    let y0 = lerp(x00, x10, u[1]);
    let y1 = lerp(x01, x11, u[1]);
    lerp(y0, y1, u[2])
}

/// Três oitavas de [`value_noise`], normalizadas para `[0, 1]`.
///
/// ⚠️ **As razões de frequência são 2,03 e 4,07 e não 2 e 4**, e é essa metade
/// que carrega o peso: com razões inteiras as três oitavas caem sobre os MESMOS
/// vértices de grade em `p` inteiro, e a rede de pontos onde as três concordam
/// vira uma trama visível com o período da oitava mais grossa. Os deslocamentos
/// são a segunda camada — eles tiram as três da origem comum — e ficam por serem
/// de graça, não porque sozinhos bastariam.
fn fbm(p: [f32; 3]) -> f32 {
    const OCTAVES: [([f32; 3], f32, f32); 3] = [
        ([0.0, 0.0, 0.0], 1.0, 0.571_428_6),
        ([19.7, 7.3, 31.1], 2.03, 0.285_714_3),
        ([51.3, 27.9, 11.5], 4.07, 0.142_857_15),
    ];
    let mut acc = 0.0;
    for (off, freq, weight) in OCTAVES {
        let q = [
            p[0].mul_add(freq, off[0]),
            p[1].mul_add(freq, off[1]),
            p[2].mul_add(freq, off[2]),
        ];
        acc = value_noise(q).mul_add(weight, acc);
    }
    acc
}

/// As distâncias aos dois pontos-semente mais próximos, em unidades de célula.
///
/// Uma semente por célula, jogada num canto aleatório dela. ⚠️ Varrer as **27**
/// células vizinhas é o que garante que a mais próxima está entre elas: uma
/// semente pode encostar na parede da sua célula, então a vencedora pode morar a
/// uma célula de distância em cada eixo.
fn worley(p: [f32; 3]) -> (f32, f32) {
    let base = [p[0].floor(), p[1].floor(), p[2].floor()];
    let bi = [base[0] as i32, base[1] as i32, base[2] as i32];
    let (mut f1, mut f2) = (f32::INFINITY, f32::INFINITY);
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell = [bi[0] + dx, bi[1] + dy, bi[2] + dz];
                let h = hash3(cell[0], cell[1], cell[2]);
                // Os três deslocamentos saem de ROTAÇÕES do mesmo hash: um
                // segundo hash por eixo custaria três avalanches por célula, e
                // rotacionar já descorrelaciona o suficiente para o olho — o
                // gate mede a distribuição, não a minha palavra.
                let seed = [
                    cell[0] as f32 - base[0] + unit(h),
                    cell[1] as f32 - base[1] + unit(h.rotate_left(11)),
                    cell[2] as f32 - base[2] + unit(h.rotate_left(22)),
                ];
                let d = [
                    seed[0] - (p[0] - base[0]),
                    seed[1] - (p[1] - base[1]),
                    seed[2] - (p[2] - base[2]),
                ];
                let d2 = d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1]));
                if d2 < f1 {
                    f2 = f1;
                    f1 = d2;
                } else if d2 < f2 {
                    f2 = d2;
                }
            }
        }
    }
    (f1.sqrt(), f2.sqrt())
}

#[cfg(test)]
#[path = "alpha_tests.rs"]
mod tests;
