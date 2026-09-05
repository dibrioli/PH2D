//! **COMO UM VERBO PEGA O BARRO** — o gesto, e a lei que ele impõe ao laço do
//! dab.
//!
//! Irmão do [`super::brush`], e o corte é de RESPONSABILIDADE: lá mora *que
//! ferramenta está na mão* (o verbo, a curva, os ganhos, o `Brush`), aqui mora
//! *como o gesto chega e o que ele faz com o que já está no barro*. O pai
//! cruzou o teto de LOC quando a [`GripLaw`] nasceu, e esta era a linha de
//! corte que já existia no assunto.

/// **A grandeza escalar que um gesto de [`Grip::Turn`] entrega**, e o que um
/// espelho faz com ela.
///
/// ⚠️ **Ela mora DENTRO do grip, e não num predicado ao lado, porque as duas
/// perguntas têm sempre a mesma resposta**: um verbo que gira em torno de uma
/// âncora *tem* de dizer em que unidade o gesto chega, e um verbo sem unidade
/// não pode ser um `Turn`. Carregada aqui, a contradição é **inexprimível** —
/// e os dois consumidores (o espelho no kernel, o gesto no shell) casam
/// exaustivamente sobre o mesmo dado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amount {
    /// Um **ÂNGULO** em radianos, com sinal.
    ///
    /// ⚠️ **É um PSEUDOESCALAR: um espelho troca o sinal dele.** Um redemoinho
    /// visto no espelho gira ao contrário — a mesma geometria que separa força
    /// de torque no módulo de física, e a razão de o espelho da simetria não
    /// poder tratar as duas grandezas igual.
    Angle,
    /// Uma **FRAÇÃO** de escala (`0` não mexe, `+1` dobra, `−1` colapsa).
    ///
    /// É um escalar comum: um espelho não a toca. Uma esfera que dobra de
    /// tamanho dobra de tamanho no espelho também.
    Fraction,
}

/// **Como um verbo consome o GESTO** — a pergunta que separa o Draw do Grab e o
/// Grab do Snake Hook, feita **uma vez**.
///
/// As três perguntas que pareciam independentes são facetas desta: *o pincel
/// carimba ao longo do caminho ou segura uma pegada?* · *o verbo lê
/// [`crate::Dab::pull`]?* · *o alvo é função do `pre` congelado ou um
/// revezamento sobre o que já está lá?* Respondê-las em três predicados soltos
/// seria a falha de três portas — a `line/Painter` a pagou em 2D quando uma
/// condição **enumerava seus leitores** e o terceiro nasceu morto.
///
/// Um `match` exaustivo sobre isto é o que faz um verbo NOVO ser impossível de
/// esquecer: quem acrescentar um quarto grip não compila até dizer, em cada
/// consumidor, o que ele significa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grip {
    /// **CARIMBA.** O caminho é percorrido a passos de
    /// [`crate::min_spacing`] e cada dab é um toque que **SOMA sobre o que o
    /// anterior deixou** — `vivo + n·(ganho·peso)`, a estrutura do kernel da
    /// referência. Dez verbos.
    ///
    /// ⚠️ **Ele era a lei do ENVELOPE e trocou em 2026-08-11** (a metade 2 do
    /// plano de paridade). O envelope tomava o `max` sobre a lista de dabs
    /// ancorado no `pre` congelado, o que dava idempotência sob re-stamp de
    /// graça; a lei que compõe a compra por outra via — **a LISTA é função do
    /// caminho** (o passo exato do `walk`, `6,485 % → 0,000 %`) —, e sem essa
    /// garantia esta linha não podia ser escrita.
    ///
    /// ⚠️ **O que ele PERDE, com nome:** duas passadas pelo mesmo caminho
    /// somam (é o que uma pincelada faz) ⇒ **re-carimbar a mesma lista NÃO é
    /// no-op**, e nenhuma rota deste módulo re-carimba um traço. O `base`
    /// continua sendo o `pre` congelado de todo vértice tocado, então o **undo
    /// segue trivial** nas cinco leis.
    /// **SIMULA.** A região é escolhida no pen-down, o anel de fora é PREGADO,
    /// e o gesto entra como FORÇA — o resto do pano responde por ser pano.
    ///
    /// ⚠️⚠️ **É o único grip cujo estado sobrevive ao evento** (posição *e*
    /// velocidade). Os outros cinco respondem `alvo = f(pre, dab)` e são função
    /// pura do gesto; aqui o resultado do evento *N* é a entrada do *N+1*, e é
    /// isso que faz uma prega existir. ⛔ A lei abaixo NUNCA é lida por ele: o
    /// [`crate::SculptStroke::dab`] desvia antes, para o `stroke_cloth`. Ela
    /// está preenchida com o que o congelamento significa (a região é medida
    /// uma vez) para o dia em que alguém a consulte.
    Simulate,
    Stamp,
    /// **SEGURA.** A pegada é presa no pen-down e o [`crate::Dab::pull`] é o
    /// deslocamento **TOTAL** desde então. O alvo continua sendo função do
    /// `pre` (`base + pull`, atenuado pelo `accum`), então puxar de volta
    /// devolve o barro ao lugar.
    ///
    /// ⚠️ **Não percorre o caminho**: o espaçamento existe para não deixar vão
    /// entre dois carimbos, e isto não carimba — rodar o walk daria N dabs
    /// idênticos no mesmo lugar.
    Hold,
    /// **ARRASTA.** A pegada anda com o cursor, o [`crate::Dab::pull`] é o
    /// **INCREMENTO** entre dois dabs, e cada um entrega ao seguinte.
    ///
    /// ⚠️ **É a outra lei, e a diferença é visível na primeira pincelada:** o
    /// alvo é `posição VIVA + incremento·peso`, não `base + algo`. Puxar para
    /// fora e voltar deixa um espinho (a matéria foi transportada), onde o
    /// [`Grip::Hold`] devolveria o barro. Isso não é um descuido — **é o que
    /// esticar significa**, e nenhum app entrega um Snake Hook idempotente.
    ///
    /// ⚠️ **O que a lei do traço perde e o que ela mantém, com nome:** a
    /// independência de amostragem vira uma **INTEGRAL DE LINHA** (`Σ dir·peso`
    /// converge com o espaçamento, e o walk fixa o espaçamento no caminho, não
    /// no polling) · a idempotência sob re-stamp **cai**, e nada a consome hoje
    /// (nenhuma rota deste módulo re-carimba um traço) · **o undo continua
    /// trivial**, porque `base` segue sendo o `pre` congelado de todo vértice
    /// que a pegada tocou em qualquer momento do gesto.
    Hook,
    /// **GIRA (ou ESCALA) em torno de uma ÂNCORA.** A pegada é congelada no
    /// pen-down como no [`Grip::Hold`], a distância sai do `pre` congelado, e o
    /// gesto que chega é o **TOTAL** desde então — o ângulo varrido, ou a fração
    /// de escala acumulada.
    ///
    /// ⚠️ **A lei do traço fica INTEIRA, e é por isso que este grip existe em
    /// vez de o Twist ser um [`Grip::Hook`].** O original aplica o incremento de
    /// cada evento sobre o resultado do anterior (`Twist.js:47` lê
    /// `_lastMouseX`), que é o **produto sobre a lista de dabs** que este módulo
    /// existe para não ter — e no Local Scale ele morde de verdade, porque
    /// `Π(1 + sᵢ·wᵢ) ≠ 1 + Σsᵢ·wᵢ`: arrastar devagar cresceria mais que arrastar
    /// rápido pelo mesmo caminho. Ancorado no `pre` com o gesto TOTAL, o alvo é
    /// função do estado congelado ⇒ independência de amostragem, idempotência
    /// sob re-stamp e *varrer de volta devolve o barro ao lugar*, as três
    /// propriedades que o cabeçalho do `stroke` promete.
    ///
    /// ⚠️ **O alvo já traz o PESO (`accum = 1`), e isso NÃO é cosmética.**
    /// Interpolar entre o `base` e a posição girada cortaria pela **CORDA** do
    /// arco: o vértice andaria por dentro da circunferência e o barro
    /// **encolheria** na direção da âncora quanto maior o giro — o mesmo defeito
    /// que o tween do Flip pagou antes de trocar o lerp pela espiral. O peso
    /// entra no ÂNGULO (`θ·w`), onde ele pertence.
    ///
    /// ⚠️ **O [`Amount`] viaja aqui dentro** — ver o doc dele.
    Turn(Amount),
    /// **PINTA UM CANAL.** Não move geometria: escreve a máscara, e o alvo de
    /// posição é o próprio lugar.
    ///
    /// ⚠️ **Ele nasceu quando o [`Grip::Stamp`] trocou de lei** (2026-08-11), e
    /// nasceu para NÃO trocar junto. A família do carimbo passou a compor sobre
    /// o vivo com `accum = 1`, e um canal que recebesse `accum = 1` seria
    /// mascarado por inteiro **num dab** — o oposto de esfregar para construir.
    /// A referência confirma que são duas leis e não uma: o `Masking.paint` tem
    /// curva própria, acumula e satura (`clamp(m + f, 0, 1)`), e **não tem
    /// `accumulate`** (ver [`crate::ref_kernels::mask`]).
    ///
    /// ⚠️ **E ele DEIXOU o envelope em 2026-08-12, que é a wave que o doc
    /// acima prometia.** A lei era `max` sobre a lista de dabs, e o preço
    /// estava medido pela porta do produto: esfregar o MESMO lugar **dezesseis
    /// vezes** deixava o canal exatamente onde uma esfregada o deixava
    /// (`0,500045` nas duas pontas), enquanto a referência chega à proteção
    /// total em **quatro**. Um envelope de dabs idênticos no mesmo lugar é
    /// constante por construção — *a máscara não acumulava, ela SATURAVA no
    /// primeiro toque*, e era isso que fazia dela um canal que não se
    /// constrói esfregando.
    ///
    /// Hoje ele é **ADITIVO e satura em 1** (`clamp(m + f, 0, 1)`,
    /// `Masking.js:70`), com a curva PRÓPRIA do canal
    /// ([`crate::Brush::mask_weight`]) em vez do [`crate::Falloff`] da
    /// geometria.
    Paint,
}

/// **A LEI que um grip impõe ao laço do dab** — as quatro colunas, lado a lado.
///
/// ⚠️ **Uma tabela e não quatro predicados soltos:** as colunas são *facetas de
/// uma escolha só*, e os grips têm combinações que se cruzam (o [`Grip::Turn`]
/// congela como o [`Grip::Hold`] e carimba `accum = 1` como o [`Grip::Hook`]).
/// Escritas como predicados independentes, um grip novo nasceria com a
/// combinação de quem alguém lembrou de atualizar; escritas aqui, ele **não
/// compila** até a linha dele existir.
///
/// ⚠️ **E ela é PÚBLICA porque um gate a consome.** O `stroke_apply_tests`
/// precisa saber *quais verbos carregam o peso no alvo*, e a resposta que ele
/// mantinha à mão era uma lista de dois grips — que a troca de lei do
/// [`Grip::Stamp`] tornou INCOMPLETA no dia em que ela landou, sem sangrar
/// nada. Perguntada aqui, ela não pode envelhecer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GripLaw {
    /// Trabalha sobre a pegada CONGELADA no pen-down em vez da consulta deste
    /// dab.
    pub frozen: bool,
    /// A distância sai da posição VIVA em vez do `pre` congelado.
    pub from_live: bool,
    /// O alvo já traz o peso, então o `accum` vale 1.
    pub unit_accum: bool,
    /// Cada dab **SOMA** o próprio peso ao que já está acumulado, saturando em
    /// `1`, em vez de escrever um valor.
    ///
    /// ⚠️ **Esta coluna era o `early_out` do ENVELOPE, e a troca de significado
    /// é a wave da máscara** (2026-08-12): com o [`Grip::Paint`] aditivo,
    /// **nenhum grip é mais um envelope**, e uma coluna que respondesse `false`
    /// para os cinco seria uma coluna morta. O que sobrou dela é o oposto — a
    /// única lei que ainda acumula ao longo da lista de dabs.
    pub additive: bool,
    /// O acúmulo satura **ASSINTOTICAMENTE** — o `d + f·força·(1,05 − |d|)` do
    /// `layer.cc` — em vez de somar linearmente, e o teto é a máscara livre em
    /// vez de `1`.
    ///
    /// ⚠️ **É a TERCEIRA lei de acumulação, e as três são mutuamente
    /// exclusivas** ([`Self::unit_accum`] · [`Self::additive`] · esta). Elas são
    /// três booleanos e não um enum porque cada uma nasceu numa wave diferente e
    /// um enum churnaria os quatro sítios que já as leem; o que as mantém
    /// honestas é o gate `the_three_accumulation_laws_are_mutually_exclusive`,
    /// que varre TODO verbo contra TODA combinação das duas flags.
    ///
    /// ⚠️ **Nenhum [`Grip`] a declara** — ela é do VERBO, e chega pela porta
    /// [`crate::Verb::grip_law`], que é onde a faixa já sobrescreve o
    /// `from_live`. Um `Grip::Coat` teria sido a leitura errada: os grips
    /// respondem *o que o GESTO faz*, e o gesto de uma demão é um carimbo — pô-la
    /// num grip próprio faria o [`crate::Verb::anchors`], que é *"uma leitura
    /// de `grip` em vez de um segundo predicado"*, passar a dizer que a demão
    /// tem âncora.
    pub coat: bool,
}

impl Grip {
    /// A [`GripLaw`] deste grip.
    ///
    /// ⚠️ **`accumulate` entra aqui porque é a ÚNICA coluna que ele move**, e
    /// move exatamente uma: o `from_live` do [`Grip::Stamp`]. Passá-lo como
    /// parâmetro é o que mantém a tabela sendo a resposta inteira em vez de
    /// *quase* a resposta, com o chamador colando o resto por fora.
    ///
    /// ⚠️ **`carries_field` entra pela MESMA razão, e move exatamente uma
    /// coluna também** — o `unit_accum` do [`Grip::Hold`]. Um campo elástico
    /// ([`crate::Field`]) **é** o falloff, não um fator a multiplicar por ele:
    /// quem o recebe já traz o peso no alvo, e deixar o aplicador atenuar de
    /// novo aplicaria o perfil duas vezes (o `0,12226` contra `0,22500` que o
    /// [`crate::Verb::Move`] documenta ter pago).
    ///
    /// ⚠️ **Ele move UMA coluna e não quatro, e isso é medição e não desenho:**
    /// os outros quatro grips já carimbam `unit_accum = true` por razões
    /// próprias, então um campo entregue a eles não muda linha nenhuma desta
    /// tabela. O `Hold` é o único que atenua no aplicador — e é por isso que
    /// era ele que ia aplicar o perfil duas vezes.
    #[must_use]
    pub fn law(self, accumulate: bool, carries_field: bool) -> GripLaw {
        let (frozen, from_live, unit_accum, additive) = match self {
            // ⚠️ **O CARIMBO COMPÕE** (2026-08-11) — a metade 2 do plano de
            // paridade. Cada dab soma o próprio incremento sobre o que o
            // anterior deixou, que é a estrutura do kernel da referência
            // (`vAr[ind] = vx + anx * fallOff`), e é por isso que o alvo passa a
            // trazer o peso (`unit_accum`) e não há envelope a superar
            // (`early_out`).
            //
            // ⚠️ **E isto SÓ é seguro porque a metade 1 fechou.** Compor sobre a
            // lista de dabs é a dependência de amostragem que o `04.1` proíbe —
            // a menos que a LISTA seja função do caminho, que é exatamente o que
            // o passo exato do `walk` passou a garantir (`6,485 % → 0,000 %`).
            // Invertida a ordem, esta linha shipava a 17,3 % e o smoke julgaria
            // a lei carregando um defeito de outra.
            //
            // ⚠️ **`from_live` É o Accumulate**, e não uma segunda leitura dele:
            // com a distância saindo da posição VIVA o centro do pincel e o
            // vértice sobem juntos e o pincel não se esgota; saindo do `pre`
            // congelado o vértice atravessa `dist >= 1` e sai da pegada sozinho.
            // É o mecanismo inteiro do checkbox do original — ver
            // [`crate::ref_kernels::Origin`], que o modela num tipo justamente
            // para ninguém achar que ele multiplica alguma coisa.
            // ⛔ **Inalcançável pelo laço do dab** — ver a variante. Congelado
            // porque a região É medida uma vez, e `unit_accum` porque o solver
            // escreve a posição inteira, não uma fração dela.
            Self::Simulate => (true, false, true, false),
            Self::Stamp => (false, accumulate, true, false),
            Self::Hold => (true, false, carries_field, false),
            Self::Hook => (false, true, true, false),
            Self::Turn(_) => (true, false, true, false),
            // O canal: ADITIVO e saturante, o `clamp(m + f)` da referência.
            Self::Paint => (false, false, false, true),
        };
        GripLaw {
            frozen,
            from_live,
            unit_accum,
            additive,
            // ⚠️ **Nenhum grip a declara, de propósito** — ver o doc do campo.
            coat: false,
        }
    }
}
