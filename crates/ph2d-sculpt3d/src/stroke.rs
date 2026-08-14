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
use crate::grip::{Amount, Grip};

/// **O QUE UM TOQUE É** — ver [`dab`].
#[path = "dab.rs"]
mod dab;
pub use dab::Dab;
use ph2d_mesh::{Mesh, QueryScratch, RegionScratch};

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
        //
        // ⚠️ **O RAIO da consulta não é o raio do pincel quando há CAMPO**, e a
        // porta é o [`Brush::query_radius`]: um Kelvinlet decide o próprio
        // suporte, e espremê-lo dentro do círculo do cursor é o que fazia o
        // `l-mode` do Grab desenhar uma agulha (ver [`crate::KELVINLET_REACH`]).
        let query_r = brush.query_radius(dab.radius);
        mesh.verts_in_sphere(dab.center, query_r, &mut self.query, &mut self.footprint);
        if self.footprint.is_empty() {
            return 0;
        }
        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            self.capture(mesh, v);
        }

        let plane = self.fit_plane(mesh, brush, dab);
        // ⚠️ **UMA vez por dab, e a assinatura é o que garante isso.** O frame do
        // padrão sai do rotor de um grau ACUMULADO deste app, que é `O(graus)`:
        // derivado por vértice ele custaria mais que o padrão inteiro que
        // orienta. Ver `Brush::alpha_frame`.
        let alpha_frame = brush.alpha_frame();
        let reach = brush.reach(dab.radius);
        let inv_r = 1.0 / dab.radius;
        // ⚠️ **UMA vez por dab**, e as três perguntas que ele responde no laço
        // (que curva usar · onde a pegada acaba · que alvo computar) têm de sair
        // da MESMA resposta: um campo que valesse para o alvo e não para a curva
        // aplicaria o perfil duas vezes, que é o defeito que o [`Verb::Move`]
        // documenta ter pago.
        let field = brush.mode.field(brush.verb);
        // ⚠️ `weight()`, nunca `strength` cru: o slider e o PESO deixaram
        // de ser a mesma coisa quando o `B` entrou (o E13).
        let intensity = brush.weight() * dab.pressure.clamp(0.0, 1.0);
        // ⚠️ **O verbo de MÁSCARA não é freado pela máscara**, e o gate pegou
        // isto: com `w = falloff·(1 − mask)`, uma região totalmente mascarada
        // zerava o peso de qualquer dab — inclusive o que a limparia. A máscara
        // ficava permanente, e o botão "Clear" seria um controle morto que
        // *parece* funcionar em toda região parcial. Ela gateia quem MOVE
        // GEOMETRIA; quem edita o próprio canal a lê como dado, não como freio.
        let gated_by_mask = !brush.verb.paints_mask();
        // ⚠️ **A LEI deste verbo, resolvida UMA vez, numa TABELA** — e a tabela
        // mora no [`Grip`], não aqui. Ela muda exatamente quatro coisas neste
        // laço, e nada mais: a captura, a máscara, a simetria, o refit e o undo
        // são os mesmos para todos. As quatro colunas e o porquê de serem uma
        // tabela estão no doc da [`crate::GripLaw`].
        //
        // ⚠️ **Ela saiu daqui para ser CONSULTÁVEL por um gate.** Enquanto
        // morava neste laço, quem precisava saber *"quais verbos carregam o peso
        // no alvo?"* mantinha a resposta à mão — e a lista de dois grips que o
        // `stroke_apply_tests` guardava ficou INCOMPLETA no instante em que o
        // [`Grip::Stamp`] trocou de lei, sem sangrar nada.
        let crate::GripLaw {
            frozen,
            from_live,
            unit_accum,
            additive,
        } = brush.verb.grip().law(brush.accumulate, field.is_some());
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

        // ⚠️ **UM DAB PODE SER MAIS DE UM PASSE, e a tabela é a porta única**
        // ([`Brush::passes`]). Todo pincel que não é o `L` do Smooth devolve
        // **exatamente um** passe de fator `1.0`, e `x * 1.0 == x` no IEEE-754
        // ⇒ o resto do motor é byte-idêntico **por construção**, não por
        // promessa.
        //
        // ⚠️ **É a única parte ESTRUTURAL da W4, e sem ela o par não existe:**
        // um traço de `N` dabs seria `N` passos `λ` sem nenhum `μ`, e o `L`
        // encolheria exatamente como o `S` — a feature ficaria verde nos gates
        // de unidade e morta no barro.
        //
        // ⚠️ **O PASSE 0 DEFINE O CONJUNTO; os seguintes percorrem ELE.** É a
        // leitura correta do par, não uma conveniência: o passo `μ` existe para
        // desfazer a contração do passo `λ`, então ele tem de alcançar
        // exatamente os vértices que o `λ` moveu — um vértice que recebesse só
        // o `λ` guardaria o encolhimento para sempre. E é também o que mantém a
        // janela publicada (`moved`) sendo um SUPERCONJUNTO do que foi escrito:
        // o guard depende do peso, que num passe posterior pode ter mudado (a
        // distância sai da posição VIVA sob Accumulate), e publicar a lista do
        // ÚLTIMO passe deixaria fora um vértice que o primeiro sujou.
        //
        // ⚠️ Um vértice pulado por um passe posterior não fica meio-escrito: o
        // `target`/`accum` dele guardam o que o passe anterior decidiu, e o
        // aplicador reescreve o MESMO valor sobre a posição que já o tem — um
        // no-op, não um passo perdido.
        let passes = brush.passes();
        for (pi, pass) in passes.iter().enumerate() {
            let first = pi == 0;
            let count = if first { work } else { self.moved.len() };
            for i in 0..count {
                let v = if first {
                    if frozen {
                        self.touched[i]
                    } else {
                        self.footprint[i]
                    }
                } else {
                    self.moved[i]
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
                // ⚠️ **O CANAL TEM CURVA PRÓPRIA, e ela é da referência.** As dez
                // tools de geometria do original multiplicam pela quártica; o
                // `Masking.paint` usa `(1 − d)^{2(1 − hardness)}`
                // (`Masking.js:66-69`), que é uma família CONTÍNUA com um knob —
                // ver [`Brush::mask_weight`]. Perguntar ao [`Falloff`] aqui media
                // uma tool contra a curva de outra, e é literalmente o *"cada tool
                // deve ter seu falloff apropriado"* que o pedido nomeia.
                // ⚠️ **A DUREZA entra AQUI, antes de qualquer curva** — é a ordem
                // do original (`apply_hardness_to_distances` roda antes do
                // `BKE_brush_calc_curve_factors`), e é o que faz as duas curvas
                // abaixo lerem a MESMA distância. Com `hardness = 0`, o default,
                // ela devolve o argumento sem tocar num bit.
                //
                // ⚠️ **COM CAMPO a curva é a INDICADORA do suporte, e é isso que
                // *"o campo É o falloff"* quer dizer em código.** O perfil inteiro
                // — quanto cada vértice anda e para que lado — é do
                // [`crate::kelvinlet`]; o que sobra para a curva é a única coisa
                // que ela ainda decide, que é ONDE o campo é avaliado e onde ele
                // simplesmente não é. Multiplicar as duas aplicaria o perfil duas
                // vezes; deixar a curva fora do `w` largaria a **separação das
                // cópias da simetria**, que é feita pelo `w > 0` medido contra
                // ESTE centro.
                let curve = if field.is_some() {
                    if dist <= query_r { 1.0 } else { 0.0 }
                } else {
                    let t = brush.shaped_distance(dist * inv_r);
                    if brush.verb.paints_mask() {
                        brush.mask_weight(t)
                    } else {
                        brush.falloff.weight(t)
                    }
                };
                // ⚠️ **O FRONT-FACE entra no FATOR, e é aqui que ele pertence** —
                // o `sculpt.cc:7283-7295` faz `factors[i] *= max(dot, 0)`, e o
                // `fall` é o nosso `factors` (ele alimenta o `w` E o `shape`). O
                // filtro BINÁRIO do `fit_plane_over` é outro consumidor e continua
                // onde está: um pesa o dab, o outro pesa a ESTIMATIVA DO PLANO.
                //
                // ⚠️ **O sinal:** o [`Dab::eye`] aponta *do olho para a superfície*,
                // então um vértice de frente tem `n · eye` NEGATIVO — o `max(dot,0)`
                // do original vira `max(−dot, 0)` aqui. Trocar o sinal daria um
                // pincel que só pega o que está de costas, e o gate mede a
                // SILHUETA justamente porque no miolo os dois são indistinguíveis.
                //
                // ⚠️ **A normal é a CONGELADA, e ler a viva era o defeito que o
                // report *"o modo B do grab está bizarro"* nomeou.** O peso de um
                // [`Grip::Hold`] é **recomputado do zero a cada evento**
                // (`accum = w`), então uma normal viva o torna função da
                // deformação que ele próprio produziu: medido no arrasto do
                // produto, o bico caía de `0,9956` para **`0,1418`** do puxão e
                // voltava — o barro agarrado COLAPSA no meio do gesto e salta de
                // volta. É realimentação, não ruído.
                //
                // ⚠️ **E ela é a única entrada do `fall` que era lida VIVA.** A
                // máscara sai do `base_mask`, o alpha é amostrado na posição
                // congelada (com o porquê escrito acima), o [`Verb::Inflate`]
                // empurra pelo `base_nrm` — quatro respostas à mesma pergunta e
                // uma delas divergindo, sem uma linha a justificar. *O peso é um
                // fato sobre o `pre`*, e agora é uma frase só.
                let facing = match brush.mode.kernel().front_face {
                    crate::FrontFace::Ignored => 1.0,
                    crate::FrontFace::Continuous => {
                        let n = self.base_nrm[s];
                        (-(n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2])).max(0.0)
                    }
                };
                let fall = curve * brush.alpha_weight(base, &alpha_frame) * facing;
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
                // ⚠️ **O `flat` MORREU aqui (2026-08-13), e não por higiene:** ele
                // era *"o `w` sem a curva"*, e existia porque a curva atenuava um
                // campo que já traz o próprio perfil. Com a curva virando a
                // INDICADORA do suporte, `curve` vale `1,0` EXATO onde o campo é
                // avaliado — e `1.0 * x == x` no IEEE-754 —, então `w` **é** o
                // `flat`, ao bit. Dois nomes para um número é a segunda porta que
                // este módulo passa o dia a evitar.
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
                // **O CANAL SATURADO não tem o que receber.** É a última coisa que
                // sobrou do early-out do envelope, e ela mudou de pergunta junto com
                // a lei: não é mais *"este dab supera o que já está lá?"* (que numa
                // lei aditiva é sempre sim), é *"ainda cabe alguma coisa?"*. Um
                // vértice em `1.0` receberia `clamp(1 + w) == 1` e seria mandado ao
                // upload por nada.
                //
                // ⚠️ **O `piling` MORREU com a troca de lei do carimbo.** Ele somava
                // `w · ACCUM_PER_DAB` sobre o envelope para exprimir o Accumulate, e
                // o preço estava medido: a primeira passada saía **13,3× mais
                // fraca** que a mesma pincelada com o interruptor desarmado — um
                // controle que promete acumular e começa enfraquecendo. Hoje o
                // Accumulate é `from_live`, que é o que ele é no original, e esta
                // linha não tem mais braço que o consulte.
                if additive && self.accum[s] >= 1.0 {
                    continue;
                }
                // ⚠️ **Quem carrega o peso no ALVO carimba `accum = 1`**, e é isso
                // que o faz caber no MESMO aplicador (`lerp(base, target, 1) ==
                // target`). Sem essa identidade haveria um segundo caminho de
                // escrita de posição — e duas rotas para *"onde este vértice vai
                // parar"* divergem no dia em que uma delas ganhar um caso especial.
                // O `base` continua guardado e intocado, que é o que mantém o undo
                // trivial nas três leis.
                // ⚠️ **A lei ADITIVA satura AQUI e não no aplicador**, e a diferença
                // é observável: o `accum` é o que o dab seguinte lê para decidir se
                // ainda cabe alguma coisa, então deixá-lo crescer sem teto faria o
                // early-out acima disparar cedo demais em dois dabs e tarde demais
                // em vinte. Guardar o valor SATURADO é o que mantém as duas leituras
                // — *quanto foi pintado* e *ainda cabe?* — respondendo o mesmo.
                self.accum[s] = if unit_accum {
                    1.0
                } else if additive {
                    (self.accum[s] + w).min(1.0)
                } else {
                    w
                };
                // ⚠️ **O FATOR DO PASSE entra AQUI, depois dos dois guards e do
                // `accum`, e a ordem é o que mantém a byte-identidade.** O guard
                // `w <= 0.0` pergunta *"este dab tem alguma coisa a dar?"*, que é
                // uma grandeza SEM sinal — aplicado antes, um passe `μ` negativo
                // seria pulado em toda parte e o par nunca rodaria. E o `accum` é
                // *quanto foi pintado*, não *para que lado*: os cinco grips o leem
                // como fração em `[0, 1]`.
                self.target[s] = self.compute_target(
                    mesh,
                    brush,
                    dab,
                    &plane,
                    reach,
                    shape * pass.weight,
                    w * pass.weight,
                    v,
                    s,
                );
                if first {
                    self.moved.push(v);
                }
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
                // ⚠️ **Por PASSE, e é uma DEFESA INERTE hoje — dito em vez de
                // afirmado ao contrário.** Escrevi primeiro que *"o passe
                // seguinte relaxaria contra a vizinhança de antes"*, e isso é
                // FALSO: a média do anel lê `mesh.positions()`, que o
                // `apply_positions` acima já escreveu. Quem o `refresh_region`
                // conserta são as NORMAIS, e o par λ|μ não as lê — o `L` declara
                // `FrontFace::Ignored`, então `facing` vale `1.0` exato nos dois
                // passes.
                //
                // ⇒ Ela fica porque o dia em que um modo declarar
                // `Continuous` **e** mais de um passe, o passe seguinte pesaria
                // pela normal de antes do anterior — e esse é o tipo de erro que
                // não falha, escurece um anel de vértices na borda do pincel e
                // ninguém sabe por quê.
                //
                // ⚠️ **O preço está MEDIDO e é do par inteiro, não só disto:**
                // `0,848 → 1,466 ms/dab` sobre a esfera de 8192 vértices
                // (`what_the_pair_costs`) — **1,73×** e não 2×, porque a
                // consulta da pegada, a captura e o ajuste do plano acontecem
                // UMA vez por dab. Fica com folga sob o kill K1 do ADR-0150.
                mesh.refresh_region(&self.moved, &mut self.region);
            }
        }
        self.moved.len()
    }
}

/// **O `pre` QUE O TRAÇO CONGELA** — ver [`freeze`]. Irmão dos três abaixo, e o
/// assunto que os três já pressupunham sem ter casa própria.
#[path = "stroke_freeze.rs"]
mod freeze;

/// **O QUE UMA SONDA PODE PERGUNTAR** — ver [`probe`]. O corte é *o que um dab
/// FAZ* (aqui) contra *o que se consegue MEDIR dele* (lá).
#[path = "stroke_probe.rs"]
mod probe;

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

/// **O QUE O TRAÇO ESCREVE** — os dois aplicadores. Filho pela mesma razão do
/// [`target`]: eles leem os planos congelados. O corte é *a LEI* (aqui) contra
/// *a ESCRITA* (lá).
#[path = "stroke_apply.rs"]
mod apply;

/// **AS JANELAS QUE UM TRAÇO PUBLICA** — ver [`windows`].
#[path = "stroke_windows.rs"]
mod windows;

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
