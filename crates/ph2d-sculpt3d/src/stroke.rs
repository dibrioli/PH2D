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

/// **O QUE UM TOQUE É** — ver [`dab`].
#[path = "dab.rs"]
mod dab;
pub use dab::Dab;
use ph2d_mesh::{DEFAULT_MASK, Mesh, QueryScratch, RegionScratch};

/// Um ponto ou vetor refletido pelo trio de sinais de uma cópia da simetria.
fn mirror(v: [f32; 3], s: &[f32; 3]) -> [f32; 3] {
    [v[0] * s[0], v[1] * s[1], v[2] * s[2]]
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
    /// A união, **sobre as cópias de espelho de UMA chamada a
    /// [`SculptStroke::dab`]**, dos vértices escritos e dos vértices cuja normal
    /// foi recomputada.
    ///
    /// ⚠️ **Eles existem porque `moved` e `region` são o rascunho de UMA cópia.**
    /// O `dab_core` zera a lista antes de a encher, e com o espelho armado ele
    /// roda de duas a oito vezes — então quem lia o rascunho depois do laço
    /// recebia a ÚLTIMA cópia e só ela. A malha ficava certa na memória e a
    /// janela de upload descrevia metade dela: o artista tocava um lado, o outro
    /// deformava na tela (report do Enio, 2026-08-05). A distinção entre
    /// *rascunho de uma cópia* e *o que a chamada fez* não era exprimível, e por
    /// isso não podia ser conferida — agora são campos diferentes com nomes
    /// diferentes, e os acessores públicos leem estes.
    call_moved: Vec<u32>,
    call_refreshed: Vec<u32>,
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

    /// Os vértices que o ÚLTIMO [`Self::dab`] de fato moveu — **todas as cópias
    /// de espelho dele**, e não a última.
    ///
    /// ⚠️ *"O último dab"* é a CHAMADA, e a diferença já custou um smoke: com o
    /// espelho armado a chamada aplica de duas a oito cópias, e publicar a
    /// última é publicar o reflexo do que o artista fez. O tamanho desta lista é
    /// exatamente o número que [`Self::dab`] devolve — se os dois divergirem,
    /// dois chamadores do mesmo dab discordam sobre ele.
    #[must_use]
    pub fn last_moved(&self) -> &[u32] {
        &self.call_moved
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
        &self.call_refreshed
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
            &self.call_moved
        } else {
            &self.call_refreshed
        }
    }

    /// Bytes segurados. A sonda de memória o soma: o custo do GESTO não pode
    /// ficar fora da conta só por ser transitório.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        let v3 = size_of::<[f32; 3]>();
        (self.slot.capacity() + self.stamp.capacity()) * size_of::<u32>()
            + (self.touched.capacity()
                + self.footprint.capacity()
                + self.moved.capacity()
                + self.call_moved.capacity()
                + self.call_refreshed.capacity())
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
    ///
    /// ⚠️ **A lei é *a malha para a qual o traço está DIMENSIONADO*, e não *a
    /// malha em que ele começou*** — a diferença nasceu com a topologia
    /// dinâmica: o refino faz a malha crescer NO MEIO do traço, e o
    /// [`Self::grow_to`] re-dimensiona sem jogar fora o `pre`. A igualdade que
    /// este `assert` exige continua exata; o que mudou é quem a mantém.
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
        // ⚠️ **Zerados UMA vez, aqui, e não a cada cópia.** É esta linha que faz
        // as janelas publicadas descreverem a CHAMADA; zerá-las lá dentro é
        // exatamente o defeito que o report de 2026-08-05 desenhou na tela. E
        // zerar aqui — antes do laço, incondicionalmente — é também o que impede
        // um dab que não move nada de herdar a janela do anterior.
        self.call_moved.clear();
        self.call_refreshed.clear();
        for s in signs.iter().take(n) {
            let det = s[0] * s[1] * s[2];
            let mirrored = Dab {
                center: mirror(dab.center, s),
                eye: mirror(dab.eye, s),
                pull: mirror(dab.pull, s),
                amount: if handed { dab.amount * det } else { dab.amount },
                ..*dab
            };
            // ⚠️ **Só uma cópia que TRABALHOU contribui.** O `dab_core` sai cedo
            // quando a pegada é vazia, e nesse caminho ele não chega ao
            // `refresh_region` — a `region` fica com a lista da cópia anterior.
            // Ler o rascunho sem esta pergunta a contaria duas vezes.
            let n = self.dab_core(mesh, brush, &mirrored);
            if n > 0 {
                self.call_moved.extend_from_slice(&self.moved);
                self.call_refreshed
                    .extend_from_slice(self.region.refreshed());
            }
            total += n;
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
        // ⚠️ **UMA vez por dab, e a assinatura é o que garante isso.** O frame do
        // padrão sai do rotor de um grau ACUMULADO deste app, que é `O(graus)`:
        // derivado por vértice ele custaria mais que o padrão inteiro que
        // orienta. Ver `Brush::alpha_frame`.
        let alpha_frame = brush.alpha_frame();
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
                crate::mask_ops::free_weight(self.base_mask[s])
            } else {
                1.0
            };
            // ⚠️ **O alpha multiplica o FALLOFF, e é lido na posição CONGELADA.**
            //
            // No falloff porque é onde ele pertence: o `shape` logo abaixo — o
            // que o Crease eleva à quinta — é `curva × máscara`, e no original o
            // expoente cai sobre `curva × máscara × alpha`. Multiplicar o `w` já
            // formado deixaria o padrão de fora do expoente, e o verbo afiaria a
            // máscara sem afiar o padrão.
            //
            // Na posição congelada porque é o que faz o padrão sobreviver ao
            // ENVELOPE. Um vértice cai sob dezenas de dabs (o espaçamento é
            // `0,15·r`); lido na posição VIVA, cada dab veria um valor diferente
            // — o próprio traço move a superfície — e o `max` tomaria o maior de
            // dezenas de amostras, LAVANDO o padrão até a envoltória superior
            // dele: o pincel ficaria mais forte, não texturizado. Lido no `pre`,
            // todos os dabs concordam sobre aquele vértice, e o `max` de valores
            // iguais é o valor. É a mesma frase que a distância já obedece três
            // parágrafos acima.
            let fall = brush.falloff.weight(dist * inv_r) * brush.alpha_weight(base, &alpha_frame);
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
            // **A LEI, e o interruptor que a troca.** Desarmado, o envelope: um
            // dab que não supera o que já está lá é descartado — mesmo
            // resultado, e sem mandar a pegada inteira ao refit do octree e ao
            // upload por nada. Armado, a SOMA — e ela é normalizada pelo passo
            // do espaçamento (`ACCUM_PER_DAB`), senão o efeito passaria a
            // depender de quantos dabs o motor emitiu.
            //
            // ⚠️ **Quem responde *"este verbo acumula?"* é a PORTA, e não a
            // tabela de grips ao lado.** As duas dizem a mesma coisa hoje — a
            // família do carimbo —, e é exatamente por isso que uma delas tem de
            // ser a resposta: escrito `early_out && brush.accumulate`, o
            // predicado público vira uma segunda cópia que o motor não consulta,
            // e uma mutação que o inverte não sangra (medido: ela sobreviveu aos
            // 90 gates). O `early_out` continua sendo o que ele é — o descarte
            // do envelope —, e a pergunta sobre o interruptor é feita ao verbo.
            let piling = brush.accumulate && brush.verb.accumulates();
            if early_out && !piling && w <= self.accum[s] {
                continue;
            }
            // ⚠️ **Quem carrega o peso no ALVO carimba `accum = 1`**, e é isso
            // que o faz caber no MESMO aplicador (`lerp(base, target, 1) ==
            // target`). Sem essa identidade haveria um segundo caminho de
            // escrita de posição — e duas rotas para *"onde este vértice vai
            // parar"* divergem no dia em que uma delas ganhar um caso especial.
            // O `base` continua guardado e intocado, que é o que mantém o undo
            // trivial nas três leis.
            self.accum[s] = if unit_accum {
                1.0
            } else if piling {
                // ⚠️ **Sem TETO, e é decisão.** `lerp(base, target, accum)` com
                // `accum > 1` passa do alvo — que é precisamente o que o
                // Accumulate significa: o Blender não capa, e capar faria a
                // segunda passada ser um no-op silencioso, que é a forma de "o
                // pincel parou de funcionar". Quem limita é a mão do artista.
                self.accum[s] + w * crate::ACCUM_PER_DAB
            } else {
                w
            };
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
                toward(b[0], t[0], a),
                toward(b[1], t[1], a),
                toward(b[2], t[2], a),
            ];
        }
    }

    /// A MESMA lei, no canal da máscara: [`toward`] entre o `base` e o alvo, onde
    /// o alvo é `1` (mascarar) ou `0` (limpar). Um verbo, uma aritmética.
    fn apply_mask(&self, mesh: &mut Mesh, brush: &Brush) {
        let goal = if brush.invert { 0.0 } else { 1.0 };
        let out = mesh.masks_mut();
        for &v in &self.moved {
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            out[vi] = toward(self.base_mask[s], goal, self.accum[s]);
        }
    }
}

/// **O APLICADOR** — de `b` para `t`, andando a fração `a`.
///
/// ⚠️ **Ele é ancorado no ALVO, e não no `base`.** A forma óbvia — `b + (t−b)·a`,
/// que esteve aqui até 2026-08-11 — e esta são a mesma coisa em aritmética
/// exata, e **nenhuma das três formas possíveis é exata nas três pontas que
/// importam**. Medido em 400 mil pares, contando divergências:
///
/// | forma | `a = 1` → `t` | `a = 0` → `b` | `t = b` → parado |
/// |---|---|---|---|
/// | `b + (t−b)·a` | **139 522** | 0 | 0 |
/// | `b·(1−a) + t·a` | 0 | 0 | **53 315** |
/// | `t − (t−b)·(1−a)` | 0 | **139 697** | 0 |
///
/// A escolha não é de gosto: é de **quais promessas o produto faz**.
///
/// - **`a = 1` devolve `t`** — o [`Grip::Hook`] e o [`Grip::Turn`] põem o peso
///   DENTRO do alvo e carimbam `accum = 1`, então para eles o alvo **é** a
///   posição final. Sem esta exatidão a paridade bit-a-bit com o kernel da
///   referência morre **no aplicador**, e nenhum gate de verbo saberia dizer:
///   todos medem deslocamento com tolerância. Um gate do produto pegou um Twist
///   escrevendo `1,3164502e-8` onde o alvo dizia `1,3164501e-8`.
/// - **`t = b` não move nada** — o `Fill` e o `Scrape` devolvem o próprio `base`
///   para o lado errado do plano, e o `Move` sem gesto devolve `base` para toda
///   a pegada. É uma promessa que o artista vê: *este verbo não toca aquele
///   lado*. A segunda forma a quebra, e o gate `a_dab_with_no_gesture_moves_
///   nothing` ficou VERMELHO nela.
/// - **`a = 0` devolve `b`** — é a que sobra, e é a que o produto **não usa**:
///   `apply_positions` percorre `moved`, e um vértice só entra ali com `w > 0`.
///
/// ⚠️ **E a medição de projeto que dizia que a forma antiga bastava era do
/// REGIME ERRADO.** Eu havia medido `b + (t−b)·1 == t` em 9 M pares gerados como
/// `t = fl(b + d)`, onde a subtração é uma transformação livre de erro e a
/// identidade é **garantida por construção**. O alvo do produto não nasce assim:
/// ele é uma expressão inteira (uma rotação de Rodrigues, uma projeção em plano)
/// arredondada **uma vez** ao `f32`, e contra o `base` ele é um float
/// independente. *Uma fixture que fabrica o valor pela mesma aritmética que vai
/// testar não contém o fenômeno.*
#[inline]
fn toward(b: f32, t: f32, a: f32) -> f32 {
    t - (t - b) * (1.0 - a)
}

/// **A MALHA CRESCEU DEBAIXO DO TRAÇO** — o refino e a lei do `pre`.
///
/// Filho para alcançar os planos congelados; o corte é o mesmo do
/// `stroke_growth_tests.rs`: aqui mora *o que acontece quando a topologia muda
/// no meio de um gesto*, no pai *o que um dab faz*.
#[path = "stroke_growth.rs"]
mod growth;

/// **O ALVO de cada verbo**, e o plano que quatro deles ajustam. Filho para
/// alcançar o `pre` congelado; o corte é *a LEI do traço* (aqui) contra *para
/// onde cada verbo aponta* (lá).
#[path = "stroke_target.rs"]
mod target;

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "stroke_growth_tests.rs"]
mod growth_tests;

#[cfg(test)]
#[path = "stroke_window_tests.rs"]
mod window_tests;

#[cfg(test)]
#[path = "stroke_accum_tests.rs"]
mod accum_tests;

#[cfg(test)]
#[path = "stroke_alpha_tests.rs"]
mod alpha_tests;
