//! **QUANTAS VEZES UM DAB ACONTECE** — a porta pública do traço, e a expansão
//! do espelho, cortadas por ASSUNTO do [`super`].
//!
//! O pai responde *o que um dab FAZ a um vértice* (a pegada, a silhueta, o peso,
//! o alvo). Aqui mora o que acontece **antes de haver um vértice**: de onde vem
//! a direção do gesto e quantas cópias dele o espelho produz — e é por isso que
//! um verbo novo herda a simetria de graça, a lição literal do
//! `stamp_dabs_inner` do Painter 2D.

use super::*;

impl SculptStroke {
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
        // ⚠️ **A DIREÇÃO DO TRAÇO nasce AQUI, e é a diferença entre CENTROS** —
        // ver [`Dab::path`] para a lição de 2D que escolheu os centros em vez de
        // uma tangente suavizada. Ela é derivada antes do espelho porque as
        // cópias são todas do mesmo gesto; o `mirrored` abaixo a leva a cada uma
        // como leva o `pull`.
        let dab = &Dab {
            path: match self.last_center {
                Some(p) => [
                    dab.center[0] - p[0],
                    dab.center[1] - p[1],
                    dab.center[2] - p[2],
                ],
                None => [0.0; 3],
            },
            ..*dab
        };
        // ⚠️ **A INCLINAÇÃO DO POLEGAR avança aqui, e a ORDEM é a da
        // referência** (`clay_thumb.cc:165-176`): o primeiro passo do traço
        // **zera e sai** — e para nós ele já sai por outra via, porque o `path`
        // dele é nulo —, e só do segundo em diante o ângulo cresce. Ler
        // `last_center` ANTES da linha abaixo é o que distingue os dois casos;
        // depois dela a informação já não existe.
        //
        // ⚠️ **E ela é gateada no VERBO** — o `front_angle` da referência só
        // avança dentro do `do_clay_thumb_brush`. Sem o gate, um traço de Draw
        // deixaria a inclinação carregada para o próximo traço de polegar sem
        // que nada na tela dissesse porquê.
        if brush.verb == Verb::ClayThumb && self.last_center.is_some() {
            self.thumb_tilt_deg = (self.thumb_tilt_deg + crate::CLAY_THUMB_TILT_STEP_DEG)
                .clamp(0.0, crate::CLAY_THUMB_TILT_MAX_DEG);
        }
        self.last_center = Some(dab.center);
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
                // Um VETOR, como o `eye` e o `pull` — a cópia espelhada vai para
                // o outro lado e o traço dela vai junto.
                path: mirror(dab.path, s),
                amount: if handed { dab.amount * det } else { dab.amount },
                ..*dab
            };
            // ⚠️ **Só uma cópia que TRABALHOU contribui.** O `dab_core` sai cedo
            // quando a pegada é vazia, e nesse caminho ele não chega ao
            // `refresh_region` — a `region` fica com a lista da cópia anterior.
            // Ler o rascunho sem esta pergunta a contaria duas vezes.
            let n = self.dab_core(mesh, brush, &mirrored, brush.passes());
            if n > 0 {
                self.call_moved.extend_from_slice(&self.moved);
                self.call_refreshed
                    .extend_from_slice(self.region.refreshed());
            }
            total += n;
            // ⚠️ **O SEGUNDO PASSE — o `autosmooth_factor` do Blender**, e a
            // posição é a dele: **depois** do verbo e **dentro** da passada de
            // simetria (`sculpt.cc:3635`, no fim do `do_brush_action`, que é
            // chamado uma vez por cópia). Fora do laço ele alisaria só a última
            // cópia; antes do verbo ele alisaria a superfície que o verbo está
            // prestes a substituir.
            //
            // ⚠️ **A porta devolve `None` no neutro** ([`Brush::auto_smooth_brush`]),
            // então com o default de fábrica nem uma consulta à árvore acontece —
            // é isso que torna este passe invisível no produto até alguém o pedir.
            //
            // ⚠️ **Ele NÃO entra no `total`.** O contador diz *quantos vértices
            // este dab moveu*, e o alisamento percorre os MESMOS: somá-lo faria
            // um dab relatar até o dobro da própria pegada, e quem lê esse número
            // decide refino e orçamento por ele.
            //
            // ⚠️ **A FORÇA vive nas PASSADAS, não no `strength` do filho** — o
            // `autosmooth_factor` é um ORÇAMENTO de até quatro laplacianos
            // completos, não o coeficiente de um lerp
            // ([`crate::auto_smooth`]). O filho nasce com `strength = 1,0` e
            // quem escala é o `Pass::weight`, que é o
            // `scale_factors(factors, strength)` da referência ao pé da letra.
            //
            // ⚠️ **E é por isso que o modo não contamina o número:** o
            // `weight()` do filho passa pela curva do slider do modo vigente, e
            // tanto `Linear` como `Squared` devolvem `1,0` para `1,0`. Pôr a
            // força ali faria o `b-mode` elevá-la ao quadrado — e o autosmooth
            // do Blender é **linear**, ao contrário da ferramenta Smooth, cujo
            // `bstrength` traz `alpha = strength²`.
            if let Some(smoother) = brush.auto_smooth_brush() {
                let budget = crate::auto_smooth::iteration_strengths(brush.auto_smooth);
                let m = self.dab_core(mesh, &smoother, &mirrored, budget.as_slice());
                if m > 0 {
                    self.call_moved.extend_from_slice(&self.moved);
                    self.call_refreshed
                        .extend_from_slice(self.region.refreshed());
                }
            }
        }
        total
    }
}
