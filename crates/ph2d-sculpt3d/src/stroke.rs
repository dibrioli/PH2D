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

use crate::brush::{Amount, Brush, Grip, Symmetry, Verb};
use ph2d_mesh::{DEFAULT_MASK, Mesh, QueryScratch, RegionScratch};

/// Um ponto ou vetor refletido pelo trio de sinais de uma cópia da simetria.
fn mirror(v: [f32; 3], s: &[f32; 3]) -> [f32; 3] {
    [v[0] * s[0], v[1] * s[1], v[2] * s[2]]
}

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
    /// **O gesto**: o vetor de MUNDO que este dab tem para dar.
    ///
    /// ⚠️ **A leitura é do [`crate::Grip`], e há duas** — o construtor nomeia
    /// qual, e é ele o guarda:
    ///
    /// - [`Dab::pulling`] ([`Grip::Hold`]): o deslocamento **TOTAL** desde o
    ///   pen-down. É o que mantém a lei do traço de pé — o alvo é `base + pull`,
    ///   função do `pre` congelado, então puxar de volta devolve o barro ao
    ///   lugar e re-carimbar não intensifica nada. Com incrementos aqui, cada um
    ///   seria uma soma sobre o resultado do anterior: o produto sobre a lista
    ///   de dabs que este módulo existe para não ter.
    /// - [`Dab::hooking`] ([`Grip::Hook`]): o **INCREMENTO** desde o dab
    ///   anterior. Ali a soma sobre a lista **é** a feature (esticar é
    ///   transportar matéria), e o que a torna um fato do CAMINHO em vez da taxa
    ///   de polling é o walk do espaçamento, que fixa o passo na geometria.
    ///
    /// Um campo com duas leituras é um risco declarado, e a alternativa é pior:
    /// dois campos deixariam **um deles morto em doze verbos**, e o outro morto
    /// no décimo terceiro. Nenhum tipo distingue um total de um incremento —
    /// quem distingue é o nome de quem constrói.
    ///
    /// Só os verbos que respondem `true` a [`Brush::verb`]`.anchors()` o leem;
    /// para os outros ele é zero e inerte.
    pub pull: [f32; 3],
    /// **O gesto ESCALAR**: quanto o [`Grip::Turn`] girou ou cresceu desde o
    /// pen-down. A unidade é a que o [`crate::Amount`] do verbo nomeia — radianos
    /// para o [`crate::Verb::Twist`], fração para o [`crate::Verb::LocalScale`].
    ///
    /// ⚠️ **É o TOTAL desde o pen-down, nunca um incremento**, e é essa escolha
    /// que mantém a lei do traço de pé para os dois verbos que o leem — ver
    /// [`Grip::Turn`]. Os construtores [`Dab::turning`] e [`Dab::scaling`] são
    /// quem nomeia a unidade, exatamente como [`Dab::pulling`] e
    /// [`Dab::hooking`] nomeiam a leitura do [`Self::pull`]: nenhum tipo
    /// distingue um radiano de uma fração.
    ///
    /// ⚠️ **Um campo a mais, e não dois**, pelo motivo que o [`Self::pull`] já
    /// pagou: dois campos deixariam cada um morto em quinze dos dezesseis
    /// verbos, e o dia em que entrasse o terceiro gesto escalar seriam três.
    pub amount: f32,
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
            amount: 0.0,
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

    /// Um dab que **ARRASTA** — o gesto do Snake Hook, e `step` é o
    /// **INCREMENTO** desde o dab anterior, não o total.
    ///
    /// ⚠️ Irmão de [`Dab::pulling`] e não um flag nele: a diferença entre um
    /// total e um incremento não é visível no tipo, e um `Dab { pull, .. }`
    /// construído à mão pode carregar qualquer um dos dois sem o compilador
    /// piscar. O nome no sítio de construção é a única barreira que existe, e
    /// por isso ela tem de estar lá.
    #[must_use]
    pub fn hooking(center: [f32; 3], radius: f32, eye: [f32; 3], step: [f32; 3]) -> Self {
        Self::pulling(center, radius, eye, step)
    }

    /// Um dab que **TORCE** — o gesto do Twist, e `radians` é o ângulo varrido
    /// **TOTAL** desde o pen-down.
    ///
    /// ⚠️ **O EIXO é o olho, e ele entra unitário daqui**: a rotação é em torno
    /// da reta que passa pela âncora na direção de quem olha, e Rodrigues exige
    /// um eixo de norma 1 (com norma `k` ele devolveria uma rotação *mais uma
    /// escala*, e o barro encolheria com o giro). Um espelho preserva
    /// comprimento, então normalizar no sítio de construção basta para todas as
    /// cópias da simetria.
    ///
    /// ⚠️ **Sem eixo não há giro:** um olho degenerado zera o ÂNGULO em vez de
    /// produzir um eixo inventado. É a diferença entre um dab que não faz nada e
    /// um dab que colapsa a pegada na âncora.
    #[must_use]
    pub fn turning(center: [f32; 3], radius: f32, eye: [f32; 3], radians: f32) -> Self {
        let len = (eye[0] * eye[0] + eye[1] * eye[1] + eye[2] * eye[2]).sqrt();
        let (axis, amount) = if len.is_finite() && len > 1e-12 {
            ([eye[0] / len, eye[1] / len, eye[2] / len], radians)
        } else {
            (eye, 0.0)
        };
        Self {
            amount,
            ..Self::at(center, radius, axis)
        }
    }

    /// Um dab que **INFLA ou ENCOLHE** — o gesto do Local Scale, e `fraction` é
    /// a fração **TOTAL** desde o pen-down (`0` não mexe, `+1` dobra o raio da
    /// pegada, `−1` a colapsa na âncora).
    #[must_use]
    pub fn scaling(center: [f32; 3], radius: f32, eye: [f32; 3], fraction: f32) -> Self {
        Self {
            amount: fraction,
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
    /// e será uma mudança nesta função, não nos dezesseis verbos.
    ///
    /// ⚠️ **O espelho alcança o DAB INTEIRO, e até 2026-08-02 ele alcançava só o
    /// centro** — o `eye` e o `pull` atravessavam sem tocar, e o preço está
    /// medido: um Grab com espelho puxava as duas metades na MESMA direção de
    /// mundo, **0,343574** de erro de simetria num pincel de raio 0,4 (a metade
    /// espelhada ia parar em `x = +1,1540` onde a simetria pede `+0,8076`). O
    /// doc do [`Dab::eye`] já **afirmava** que a cópia espelhada tinha o olho
    /// dela; a linha que o faria nunca existiu.
    ///
    /// ⚠️ **E o `eye` só é observável onde a pegada atravessa a TERMINADOR** —
    /// medido `0,047`–`0,094` ali e **0,000001** em qualquer outro lugar, porque
    /// longe dela o conjunto frontal é *tudo* (e o fallback do `fit_plane` cobre
    /// o caso em que ele é *nada*). Era a fixture que não continha o fenômeno,
    /// não o defeito que era pequeno.
    ///
    /// # A lei, e por que ela não é um `for` sobre os campos
    ///
    /// Cada canal do dab tem uma **espécie geométrica**, e um espelho trata as
    /// três de maneiras diferentes:
    ///
    /// - o `center` é um **PONTO** e o `eye`/`pull` são **VETORES** — todos
    ///   componente a componente pelo sinal do eixo;
    /// - o [`Amount::Angle`] é um **PSEUDOESCALAR**: ele troca de sinal com o
    ///   **determinante** do espelho (um redemoinho no espelho gira ao
    ///   contrário), e espelhar os TRÊS eixos não é uma reflexão — é uma rotação
    ///   de 180°, e o produto dos sinais devolve `+1` sozinho, sem caso especial;
    /// - o [`Amount::Fraction`] é um **ESCALAR comum** e o espelho não a toca.
    ///
    /// # ⚠️ A malha tem de ser a MESMA em que o traço COMEÇOU
    ///
    /// Os planos por-vértice (`slot`, `stamp`) são dimensionados no
    /// [`Self::begin`], então um dab noutra malha os indexa com índices que não
    /// são deles. Enquanto a malha nova for MENOR o defeito é mudo — escreve na
    /// vizinhança errada e ninguém vê; assim que for maior, estoura.
    ///
    /// Este `assert` custa uma comparação de `usize` por dab e troca *"index out
    /// of bounds"* por uma frase. Ele já se pagou: numa cena com mais de um
    /// objeto, o pen-down começava o traço na peça ATIVA e o pick escolhia a
    /// peça sob o cursor — tocar um cubo de 8 vértices e depois uma esfera de
    /// 6050 panicava. A lei mora no chamador (*um traço pertence a uma peça*), e
    /// isto é o que a nomeia quando alguém a quebrar de novo.
    pub fn dab(&mut self, mesh: &mut Mesh, brush: &Brush, dab: &Dab, sym: Symmetry) -> usize {
        assert_eq!(
            self.slot.len(),
            mesh.vert_count(),
            "um dab tem de cair na malha em que o traço COMEÇOU: \
             `begin` dimensionou {} vértices e esta malha tem {}",
            self.slot.len(),
            mesh.vert_count()
        );
        let (signs, n) = sym.signs();
        let handed = matches!(brush.verb.grip(), Grip::Turn(Amount::Angle));
        let mut total = 0;
        for s in signs.iter().take(n) {
            let det = s[0] * s[1] * s[2];
            let mirrored = Dab {
                center: mirror(dab.center, s),
                eye: mirror(dab.eye, s),
                pull: mirror(dab.pull, s),
                amount: if handed { dab.amount * det } else { dab.amount },
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
        // ⚠️ **A LEI deste verbo, resolvida UMA vez, numa TABELA onde os quatro
        // grips aparecem lado a lado.** Ela muda exatamente quatro coisas neste
        // laço, e nada mais — a captura, a máscara, a simetria, o refit e o undo
        // são os mesmos para todos:
        //
        // - `frozen` — trabalha sobre a pegada CONGELADA no pen-down (`touched`)
        //   em vez da consulta deste dab;
        // - `from_live` — a distância sai da posição VIVA em vez do `pre`;
        // - `unit_accum` — o alvo já traz o peso, então o `accum` vale 1;
        // - `early_out` — um dab que não supera o envelope é descartado.
        //
        // ⚠️ **Uma tabela e não três predicados soltos:** as quatro colunas são
        // *facetas* de uma escolha só, e os quatro grips têm combinações que se
        // cruzam (o `Turn` congela como o `Hold` e carimba `accum = 1` como o
        // `Hook`). Escritas como predicados independentes, um grip novo nasceria
        // com a combinação de quem alguém lembrou de atualizar; escritas aqui,
        // ele **não compila** até a linha dele existir.
        let (frozen, from_live, unit_accum, early_out) = match brush.verb.grip() {
            Grip::Stamp => (false, false, false, true),
            Grip::Hold => (true, false, false, false),
            Grip::Hook => (false, true, true, false),
            Grip::Turn(_) => (true, false, true, false),
        };
        // ⚠️ **Quem SEGURA trabalha sobre o que já TOCOU, não sobre a consulta
        // deste dab — e sem isto o Grab PERDE barro.** A consulta sai das
        // posições vivas, então um vértice arrastado para além do raio SAI da
        // esfera e deixa de ser escrito: ele congela onde estava. Medido em
        // `tests/measure_pull_profile.rs`, um grab de raio 0,4 puxado a 0,6 e
        // trazido de volta deixava **0,52994** de resíduo — e *"puxar de volta
        // devolve o barro ao lugar"* é a propriedade que o `Grip::Hold`
        // PROMETE. O original congela o conjunto no pen-down
        // (`Move.js:initMoveData`, chamado do `startSculpt`) e nunca o
        // reconsulta.
        //
        // ⚠️ **O gate que existia não podia ver**: ele mede o vértice do MIOLO,
        // que só escapa quando o puxão passa do raio. Quem vê é o gesto de ida e
        // volta com puxão maior que a pegada.
        //
        // `touched` **é** esse conjunto congelado: depois do primeiro dab ele
        // contém a pegada inteira, e ninguém a remove. Ele pode CRESCER (um
        // vizinho que escorregou para dentro da esfera é capturado), e o filtro
        // `w > 0` — medido contra a posição CONGELADA — devolve exatamente o
        // conjunto do pen-down. É também ele que separa as cópias da simetria,
        // que compartilham este vetor: um vértice da cópia oposta está fora do
        // raio deste centro, logo pesa zero.
        //
        // ⚠️ **O [`Grip::Turn`] congela pelo MESMO motivo, e nele o efeito é
        // mais forte:** um Local Scale empurra os vértices para longe da âncora,
        // então a pegada consultada a cada dab ENCOLHE em contagem enquanto a
        // forma cresce — o barro da borda sairia do raio e congelaria a meio
        // caminho, deixando um degrau na fronteira do pincel.
        let work = if frozen {
            self.touched.len()
        } else {
            self.footprint.len()
        };

        for i in 0..work {
            let v = if frozen {
                self.touched[i]
            } else {
                self.footprint[i]
            };
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let base = self.base_pos[s];
            // ⚠️ **De ONDE se mede a distância, e as duas respostas são certas
            // para leis diferentes.** No envelope o peso tem de ser função do
            // estado CONGELADO: se ele fosse recomputado sobre a superfície que
            // o próprio traço moveu, dois dabs no mesmo lugar dariam pesos
            // diferentes e a idempotência — a propriedade que o envelope existe
            // para dar — cairia junto.
            //
            // No revezamento é o oposto, e é o que faz um espinho ser um
            // espinho: a pegada ANDA, e um vértice que já foi arrastado para
            // perto do novo centro **tem** de continuar sendo arrastado. Medido
            // pelo `base`, esse mesmo vértice ficaria com peso ~0 (o `pre` dele
            // está lá atrás), a ponta pararia de crescer e o gesto viraria um
            // Grab com centro móvel. `Drag.js:99` lê `vAr[ind]`, a posição viva.
            //
            // A pegada (`verts_in_sphere`) já sai das posições vivas nos dois
            // casos, então no revezamento pegada e peso concordam.
            let from = if from_live {
                mesh.positions()[vi]
            } else {
                base
            };
            let d = [
                from[0] - dab.center[0],
                from[1] - dab.center[1],
                from[2] - dab.center[2],
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
            // ⚠️ **Quem tem ÂNCORA não pode ser freado pelo early-out**: a
            // pegada dos três é presa no pen-down (ou o `accum` deles vale 1),
            // então o peso de cada vértice nunca mais sobe. Sem esta exceção o
            // barro andaria UM evento e pararia, com o cursor seguindo em frente
            // — e o que mudou não é o peso, é o gesto. Ver a tabela dos grips.
            // ⚠️ **Peso zero não é um dab: ele não tem nada a dar.** Para os
            // doze verbos de carimbo isto já saía do early-out abaixo (`0 <= 0`)
            // e é redundante; para os que têm âncora ele é a linha que os mantém
            // corretos, porque eles **dispensam** aquele early-out. Sem ela, um
            // vértice de peso zero levaria `accum ← 0` e `target ← base`, e o
            // dab seguinte de um Grab **desfaria** o que a cópia espelhada
            // acabou de fazer — as duas cópias compartilham o `touched`.
            if w <= 0.0 {
                continue;
            }
            if early_out && w <= self.accum[s] {
                continue;
            }
            // ⚠️ **Quem carrega o peso no ALVO carimba `accum = 1`**, e é isso
            // que o faz caber no MESMO aplicador (`lerp(base, target, 1) ==
            // target`). Sem essa identidade haveria um segundo caminho de
            // escrita de posição — e duas rotas para *"onde este vértice vai
            // parar"* divergem no dia em que uma delas ganhar um caso especial.
            // O `base` continua guardado e intocado, que é o que mantém o undo
            // trivial nas três leis.
            self.accum[s] = if unit_accum { 1.0 } else { w };
            self.target[s] = self.compute_target(mesh, brush, dab, &plane, reach, shape, w, v, s);
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
