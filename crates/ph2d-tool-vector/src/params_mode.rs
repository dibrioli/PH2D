//! **O MODO de desenho** — o enum `DrawMode` e as portas que ele responde.
//!
//! Irmão de [`super::params`] pelo teto de 700 LOC, e o corte é por assunto: aqui mora *o que o
//! gesto de canvas está a fazer*, e não os parâmetros numéricos de um estilo. Re-exportado dali,
//! então quem consome não percebe o corte.

use ph2d_vec_scene::ShapeKind;

/// The canvas gesture the Vector tool performs (ADR-0108 Fase 1). `Pen` is the
/// draw + edit-anchor gesture (`PenTool`); the shape modes are drag-to-size
/// (`ShapeTool`). The tool owns the mode; the docked panel's segmented row sets
/// it and highlights the active one from the published snapshot.
/// ⭐⭐⭐ **O QUE O ARRASTO FAZ NO MODO OSSO** — criar ou transformar (Enio, 2026-09-07:
/// *«do modo como está fica confuso para o usuário»*).
///
/// ⛔ **Antes eram os dois ao mesmo tempo, e a ambiguidade era do PONTEIRO:** um arrasto sobre um
/// osso posava-o, um arrasto no vazio criava. Isso torna **inalcançáveis** dois gestos legítimos —
/// começar um osso *em cima* de outro, e posar um osso *sem medo* de criar um por engano — e
/// obriga o artista a saber o que está por baixo do cursor antes de carregar.
///
/// É a mesma partição que o Moho faz com ferramentas separadas (*Add Bone* × *Translate/Rotate
/// Bone*) e o Spine com modos, e a razão é a mesma: **um gesto, um verbo**.
///
/// ⚠️ Ele é um estado do MODO Osso, e não um par de pills na fileira de modos: a fileira responde
/// *«que ferramenta»*, este responde *«o que ela faz»* — e são perguntas de níveis diferentes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BoneAction {
    /// Arrastar **faz** um osso, a partir da ponta do osso aceso (ou do ponto do press).
    /// Carregar num osso apenas o **selecciona** — é assim que se escolhe onde ramificar.
    #[default]
    Create,
    /// Arrastar **posa** o que está sob o cursor: corpo gira, bolinha desloca, quadradinho da
    /// mancha muda a força, anel duplo da ponta dobra a corrente (IK). ⛔ Nunca cria.
    Transform,
    /// ⭐⭐⭐ **Arrastar PINTA o peso** do osso aceso sobre a arte presa — a correcção à mão de
    /// [`ph2d_skeleton::Correccao`], para quando a conta automática erra num sítio.
    ///
    /// ⚠️ **É um verbo do mesmo nível dos outros dois, e não um modo dentro do *Transformar*:** a
    /// pergunta que a fileira responde é *«o que este arrasto faz»*, e pintar peso não é posar nem
    /// criar. ⛔ Escondê-lo atrás de um modificador de teclado seria a meia-porta que o §5.0 nomeia
    /// (*«um gesto que só existe se o artista adivinhar o modificador é meio gesto»*).
    ///
    /// ⚠️ **PARA QUE LADO ele empurra é uma [`WeightDirection`]**, dois botões na secção dele.
    /// ⛔⛔ Até 2026-09-19 esta linha dizia *«o SINAL do valor é a direcção — não há um segundo
    /// verbo a lembrar»*, e a premissa morreu por **ordem do dono** (*«no lugar de valores
    /// negativos em Brush Strength prefiro botões Add e Subtract»*). Ver [`WeightDirection`].
    Weight,
}

impl BoneAction {
    /// As três, na ordem em que o grupo as mostra. ⛔ Fonte única da iteração.
    pub const ALL: [BoneAction; 3] = [
        BoneAction::Create,
        BoneAction::Transform,
        BoneAction::Weight,
    ];

    /// ⭐⭐ **O ÍNDICE deste verbo em [`Self::ALL`]** — a porta que a fileira do painel acende.
    ///
    /// ⛔⛔ **Ela existe porque o índice era um `bool`** (`usize::from(acao == Transform)`), e isso
    /// está certo com dois verbos e **mente em silêncio** com três: o terceiro leria `0` e acenderia
    /// o primeiro segmento. *Um índice derivado de uma comparação é uma tabela escrita à mão com
    /// outra sintaxe.*
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|a| *a == self).unwrap_or(0)
    }
}

/// ⭐⭐⭐ **PARA QUE LADO A PINCELADA DE PESO EMPURRA** — dois botões, e não o sinal de um número.
///
/// ⛔⛔⛔ **Ordem do dono (2026-09-19): *«no lugar de valores negativos em Brush Strength prefiro
/// botões Add e Subtract»*.** A objecção que estava escrita no painel — *«o `Amount` é COM SINAL, e
/// é isso que faz o gesto ser um só; um segundo chip seria a segunda maneira de dizer a mesma
/// coisa»* — fica **registada e não vencida**.
///
/// ⭐ **E a arrumação que ela traz é uma pergunta por controlo:** o número passa a responder
/// *QUANTO* (uma magnitude, que não tem sinal que faça sentido) e os dois botões respondem *PARA QUE
/// LADO*. Enquanto o sinal vivia no número, *«tirar peso»* era um estado invisível — o artista tinha
/// de **ler um menos** para saber o que o próximo arrasto ia fazer.
///
/// ⚠️ **Elas são um SEGMENTO exclusivo e não duas caixas**: as duas ligadas ao mesmo tempo não
/// significam nada, e duas caixas independentes exprimem esse estado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeightDirection {
    /// A pincelada **SOMA** peso ao osso em foco. O valor de fábrica: é o que o artista faz
    /// primeiro, e o que ele espera sem ter escolhido nada.
    #[default]
    Add,
    /// A pincelada **TIRA** peso ao osso em foco.
    Subtract,
}

impl WeightDirection {
    /// As duas, na ordem em que o segmento as mostra. ⛔ Fonte única da iteração.
    pub const ALL: [WeightDirection; 2] = [WeightDirection::Add, WeightDirection::Subtract];

    /// ⭐⭐ **O ÍNDICE desta direcção em [`Self::ALL`]** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ **Derivado da lista e não escrito à mão**, pela mesma razão que o
    /// [`BoneAction::indice`] existe: *um índice derivado de uma comparação é uma tabela escrita à
    /// mão com outra sintaxe*, e ela mente em silêncio no dia em que a lista crescer.
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(0)
    }

    /// ⭐⭐⭐ **O `delta` QUE A LEI RECEBE** — a magnitude com o sinal desta direcção.
    ///
    /// ⚠️⚠️ **Ela é uma PORTA com um chamador só, e isso é de propósito.** A composição
    /// *«magnitude × direcção»* é a lei que a ordem do dono criou; escrita como um `if` dentro do
    /// despacho da shell, ela ficaria num sítio onde nenhum teste lhe chega — que é exactamente
    /// como a escolha do alvo do pincel viveu até 19/09. *Uma lei que só existe num laço de input
    /// é uma lei que ninguém pode contradizer.*
    ///
    /// ⚠️ **A magnitude entra em valor ABSOLUTO** — ela é *quanto*, e um *quanto* negativo não quer
    /// dizer nada. Ver [`WEIGHT_AMOUNT_DEFAULT`] para o que a porta do painel faz com um número
    /// negativo escrito à mão.
    #[must_use]
    pub fn delta(self, magnitude: f64) -> f64 {
        let m = magnitude.abs();
        match self {
            Self::Add => m,
            Self::Subtract => -m,
        }
    }
}

/// ⭐⭐⭐ **COMO A PINCELADA ATRIBUI O PESO** — ordem do dono, 2026-09-19: *«precisamos de 2 modos de
/// atribuir peso aos pontos»*.
///
/// ⚠️⚠️ **A diferença NÃO é de UI — é do modelo de dados**, e está na
/// [`ph2d_skeleton::Especie`]: uma mancha cumulativa SOMA (duas sobrepostas acumulam-se por
/// construção) e uma absoluta FIXA (duas sobrepostas não se somam — vence a última pintada). É por
/// isso que este enum não é uma lente do painel: ele escolhe **que espécie de mancha nasce**.
///
/// ⛔ **No modo absoluto a [`WeightDirection`] fica sem sujeito** — *«para que lado»* não tem
/// resposta quando o gesto põe um valor —, e o painel ESCONDE-a. É a lei da casa (*esconde-se o que
/// se pode; diz-se a razão onde uma cerca de produto proíbe esconder*), e aqui nada proíbe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeightMode {
    /// **CUMULATIVO** — cada pincelada soma (ou tira) o *Brush Strength* ao peso do osso em foco.
    ///
    /// É o valor de fábrica: é a lei que já existia, a que o dono aprovou em smoke, e a que um
    /// artista espera de um pincel sem ter escolhido nada.
    #[default]
    Cumulative,
    /// **ABSOLUTO** — o *Brush Strength* é posto **imediatamente** no osso em foco, e o que sobra
    /// (`1 − v`) reparte-se pelos outros ossos daquele ponto mantendo a proporção entre eles.
    Absolute,
}

impl WeightMode {
    /// Os dois, na ordem em que o segmento os mostra. ⛔ Fonte única da iteração.
    pub const ALL: [WeightMode; 2] = [WeightMode::Cumulative, WeightMode::Absolute];

    /// ⭐⭐ **O ÍNDICE deste modo em [`Self::ALL`]** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ Derivado da lista, pela mesma razão do [`WeightDirection::indice`].
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

    /// ⭐⭐⭐ **A MANCHA QUE ESTA PINCELADA ESCREVE** — a porta que junta o modo, a magnitude e a
    /// direcção numa [`ph2d_skeleton::Especie`].
    ///
    /// ⚠️⚠️ **Ela é UMA e vive aqui, ao lado da [`WeightDirection::delta`] que substitui** — pela
    /// mesma razão escrita lá: *uma lei que só existe num laço de input é uma lei que ninguém pode
    /// contradizer*. ⛔ No modo absoluto a `direccao` é **ignorada**, e isso é a lei e não um
    /// esquecimento: o painel esconde os dois botões exactamente porque eles não têm sujeito aqui.
    ///
    /// ⚠️ **A magnitude entra em valor ABSOLUTO nos dois modos** — ela é *quanto*/*quanto vale*, e
    /// o `Alvo` é coagido a `0..1` pela própria lei ([`ph2d_skeleton::Skin::fixa`]).
    #[must_use]
    pub fn especie(self, magnitude: f64, direccao: WeightDirection) -> ph2d_skeleton::Especie {
        match self {
            Self::Cumulative => ph2d_skeleton::Especie::Soma(direccao.delta(magnitude)),
            Self::Absolute => ph2d_skeleton::Especie::Alvo(magnitude.abs()),
        }
    }
}

/// ⭐ **O RAIO de fábrica do pincel de peso, em PÍXEIS DE ECRÃ.**
///
/// ⛔⛔⛔ **Ele era `20.0` em unidades de MUNDO, e o report do dono mediu o que isso vale**
/// (2026-09-19, *«os pontos não ficam coloridos»*): na barra laranja da cena, a ppm `100`,
///
/// | grandeza | mundo | píxeis |
/// |---|---|---|
/// | dois pontos vizinhos da peça | `0,2761` | `27,6` |
/// | a peça INTEIRA, ponta a ponta | `7,0218` | `702,2` |
/// | **o raio de fábrica de então** | `20,0` | **`2 000`** |
///
/// ⇒ o pincel de fábrica era **`2,85 ×` a peça inteira**: um clique agarrava TODOS os pontos dela
/// ao mesmo tempo, e o anel ficava maior que a janela. *Um pincel que não consegue apontar a um
/// sítio não é um pincel.*
///
/// ⛔⛔ **E a premissa do doc anterior era o defeito:** ele dizia que o número tinha de ser *«visível
/// numa forma do tamanho das que o app desenha»* — mas **um default em unidades de MUNDO não pode
/// saber a escala da cena** (um braço de `6` metros e outro de `600` pedem raios `100 ×`
/// diferentes). O que o artista percebe é o tamanho do pincel **no ecrã**, e é essa a única
/// grandeza que um valor de fábrica pode fixar. ⇒ o raio passa a ser de ECRÃ, e é convertido a
/// mundo no sítio onde é usado — o mesmo idioma do `hit_r` do pick desta casa.
///
/// ⚠️ **O número sai da tabela acima:** `4 ×` o raio de pick da casa (`10` px — abaixo disso o gesto
/// lê-se como apontar, não pintar), `1,45 ×` a distância entre dois pontos vizinhos (logo uma
/// pincelada apanha uma VIZINHANÇA e não um ponto só) e `5,7 %` da peça (logo ela aponta).
pub const WEIGHT_RADIUS_DEFAULT: f64 = 40.0; // LITERAL-PX-OK: raio de ecrã, tabela medida acima

/// ⭐ **QUANTO cada pincelada empurra o peso, de fábrica** — uma MAGNITUDE, em `0..1`.
///
/// ⚠️ **Pequeno de propósito:** o peso vive em `0..1` e a pincelada acumula, logo o artista chega ao
/// extremo insistindo. *Um valor de fábrica que salta para o extremo numa pincelada faz o gesto ser
/// um interruptor.*
///
/// ⛔⛔ **Ele deixou de ter SINAL em 2026-09-19** (ordem do dono — ver [`WeightDirection`]): para
/// que lado a pincelada empurra é agora dois botões. ⚠️ **E um número negativo escrito à mão entra
/// em valor ABSOLUTO, nunca cortado a zero:** cortá-lo deixaria o pincel **inerte e calado**, que é
/// a espécie de defeito que esta casa lê como *«a ferramenta não funciona»*. O campo é re-semeado
/// do estado da ferramenta a cada quadro, logo a tela mostra de volta a magnitude que ela usa — *o
/// ecrã corrige-se à vista em vez de guardar um número que ninguém honra*.
pub const WEIGHT_AMOUNT_DEFAULT: f64 = 0.15;

/// ⭐ **O PISO do raio do pincel de peso, em PÍXEIS DE ECRÃ** — ver [`WEIGHT_RADIUS_DEFAULT`].
///
/// ⚠️ **O recurso é o ECRÃ:** abaixo de um píxel o anel deixa de ser desenhável e o pen-down nunca
/// acha arte — e a recusa (`ForaDaArte`) lê-se exactamente como um pincel partido.
pub const WEIGHT_RADIUS_MIN: f64 = 1.0; // LITERAL-PX-OK: piso de ecrã, um píxel

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrawMode {
    /// Seta preta: seleciona e TRANSFORMA a forma pelo gizmo. Não toca a geometria.
    #[default]
    Select,
    /// Seta branca: edita âncoras e handles do path selecionado. Nunca cria um path,
    /// e o gizmo não aparece (as alças dele comeriam o clique do nó).
    Node,
    /// Caneta: cria path novo e edita os nós que ela mesma pôs. Sem gizmo.
    Pen,
    /// **Lápis**: arrasta e a curva sai — a mão livre. O gesto grava amostras, o decimador as
    /// reduz a nós e o ajuste de Hobby devolve a spline que PASSA por eles
    /// (`ph2d_vec_edit::Pencil`). É um modo e não uma variante da caneta porque o gesto é o
    /// oposto: a caneta é uma sequência de cliques DISCRETOS, o lápis é um arrasto contínuo.
    Pencil,
    /// **Forma**: arrasta para dimensionar a forma ATIVA do catálogo
    /// (`VectorTool::shape_kind`). É UM modo para todas as formas — retângulo, estrela,
    /// seta, balão… — porque a forma é dado, não código. Antes cada forma era um modo, e
    /// vinte e cinco formas seriam vinte e cinco variantes aqui, no painel e no dispatch.
    Shape,
    /// Texto: clica no canvas e digita; cada glyph vira um `VecPath` preenchido
    /// (ADR-0108). Não é uma shape-tool nem cria pelo Pen — o shell trata o gesto.
    Text,
    /// **Shape Builder**: com 2+ formas selecionadas, o cursor arrasta sobre as REGIÕES em
    /// que elas se dividem — o que ele pinta vira uma forma só; com Alt, some.
    ///
    /// É um modo e não um botão de Pathfinder porque a unidade de trabalho não é a FORMA, é
    /// a **face do arranjo**: a região "dentro da A e fora da B" não existe como objeto até o
    /// dedo passar por cima dela. Um Pathfinder obriga a pensar em operações; isto deixa
    /// desenhar o resultado.
    Build,
    /// **Conector**: pressiona sobre uma forma, arrasta, solta sobre outra — nasce uma
    /// linha que gruda nas duas e as SEGUE (soltar no vazio deixa a ponta solta ali;
    /// pressionar e soltar na mesma forma faz um laço).
    ///
    /// Não é uma forma do catálogo, e é por isso que é um MODO: a geometria de um
    /// conector não é autorada, é **derivada** (uma função pura de a quem cada ponta se
    /// prende), e a shell a re-cozinha a cada frame (`connector_live`).
    Connect,
    /// **Pick Shapes** (Blend): coleta as formas fechadas clicadas **na ordem**; o botão Blend as
    /// liga nessa sequência (ADR-0128 C2b). É um modo — como o Build e o Connect — porque o gesto é
    /// escolher formas no canvas, não editar a selecionada; a ORDEM da cadeia é a de clique, não a
    /// de z.
    PickBlend,
    /// **Fillet** (arredondar quina): pressiona sobre uma quina e arrasta — o recuo cresce com o
    /// arrasto e a quina ARREDONDA (arco). Se o ponto clicado não é quina (é suave), a ferramenta
    /// primeiro o transforma em quina. É o Live Corners (ADR-0121) virado ferramenta própria, com
    /// gesto de clicar-e-arrastar, em vez de uma alça escondida no modo Node.
    Fillet,
    /// **Chamfer** (chanfrar quina): idêntico ao [`DrawMode::Fillet`], mas a ligação é uma RETA em
    /// vez de arco (o SINAL do `corner_radius`, ADR-0121). O par Fillet/Chamfer consolida numa
    /// dupla de ferramentas o que estava espalhado entre a alça do Node e o toggle da seção Vertex.
    Chamfer,
    /// **Width**: as alças de LARGURA na curva (plano 25 §5, ADR-0148). Uma alça por parada do
    /// perfil, fora da curva à distância que a fita tem ali; afastar engrossa, aproximar afina,
    /// andar ao longo move a parada. Clicar na curva acrescenta uma parada; o botão direito
    /// sobre uma alça a apaga.
    ///
    /// É um modo pela MESMA razão do Fillet/Chamfer: no Node estas alças competiriam com as
    /// âncoras — uma parada de multiplicador pequeno senta a milímetros da curva, ou seja em
    /// cima delas. O Illustrator também o faz uma ferramenta (Shift+W).
    Width,
    /// ⭐⭐⭐ **Aparar** (plano 38): passa o cursor sobre um pedaço de caminho, o pedaço entre as
    /// duas FRONTEIRAS mais próximas acende, e o clique apaga-o.
    ///
    /// É o `Trim` do Fusion 360 — *"trims to the nearest **crossing or node**"* —, e o *"or node"*
    /// é o *"entre pontos"* do pedido. As quatro espécies de fronteira (cruzamento com outro
    /// caminho · auto-cruzamento · nó · ponta aberta) vivem numa lista só
    /// ([`ph2d_vec_scene::trim_tool::boundaries`]).
    ///
    /// ⚠️ **Não é o [`DrawMode::Cut`], e a diferença é o SUJEITO.** O Corte pede uma lâmina
    /// AUTORADA (desenha-se a linha, e um botão corta com ela); o Trim não pede nada — *tudo o que
    /// está na tela já corta*, e o único gesto é apontar o pedaço. A Autodesk fez exactamente esta
    /// troca no `TRIM` em 2021, e deixou o modelo da lâmina escolhida atrás de uma variável.
    Trim,
    /// ⭐⭐⭐ **Balde** (plano 40): aponta-se uma REGIÃO cercada por traços, ela acende, e o clique
    /// deixa lá uma forma preenchida.
    ///
    /// É o *Smart Fill* do CorelDRAW — **um clique, um objecto novo** —, e não o *Live Paint* do
    /// Illustrator (um tipo de grupo com estado próprio, que é a feature seguinte, não a primeira).
    ///
    /// ⚠️ **Não é o [`DrawMode::Build`], e a diferença é o SUBSTRATO.** O Shape Builder responde
    /// *"que face é esta?"* pela definição conjuntista `região(M) = ∩M − ∪¬M`, que **não existe
    /// para um traço aberto** — uma linha não tem dentro. O balde percorre a rede de ARCOS que o
    /// Soldar produz, e é por isso que ele preenche *"linhas sobrepostas"* (o pedido do Enio) e o
    /// Shape Builder não.
    Bucket,
    /// **Corte** (plano 25 §7, W4): desenha-se uma **LINHA DE CORTE** — com a caneta, exatamente
    /// como se desenha qualquer curva — e um botão do painel corta com ela.
    ///
    /// ⚠️ **A linha não é um gesto transiente, é um OBJETO.** Ela fica na cena depois de
    /// desenhada: move-se com o Select, edita-se no Node, sobrevive ao save, e um segundo botão a
    /// descarta. Esse é o ponto inteiro do modo — a lâmina que some no `release` obriga a acertar
    /// o traço de primeira, e cortar é justamente o gesto em que se quer mirar antes.
    ///
    /// **Um modo, não dois.** Ele substitui a Tesoura (clicar para abrir num ponto) e a Faca
    /// (arrastar uma lâmina reta): as duas produziam peças **ABERTAS**, e a lei do produto é que
    /// uma forma fechada cortada dá formas **FECHADAS** (Enio, 2026-07-31). Abrir um caminho num
    /// nó continua a ser uma operação legítima — mas é outro verbo (*Break Path*, do modo Node),
    /// e não pode vestir a palavra "cortar".
    Cut,
    /// **Moldura** (plano UI/UX W0): arrasta e nasce um CONTÊINER — uma tela, um card, um painel.
    ///
    /// O gesto é o do retângulo, e literalmente: [`Self::shape_kind`] devolve
    /// `ShapeKind::RoundRect` para este modo, então restrição de Shift/Alt, pose, undo e
    /// Live Shape vêm de graça. O que a moldura acrescenta ao nascer é **um componente**
    /// (`ph2d_ecs::VecFrame`) — ela É um retângulo vivo, e é essa decisão que lhe dá fill,
    /// gradiente, traço, raio de quina, efeitos, gizmo, z-order e save sem uma linha a mais.
    ///
    /// ⚠️ **`RoundRect` e não `Rectangle`, desde 2026-08-21** (Enio: *"o Frame é criado como
    /// retângulo de quinas sem a possibilidade de arredondamento"*). A promessa de *"raio de
    /// quina de graça"* acima era **falsa na prática**: a moldura herdava a única forma da
    /// família que não tem campo de raio, e nenhum dos ajustes existia para ela. O raio nasce
    /// zero e `rounded_rect(_, 0)` **é** `rectangle`, então a troca é invisível até o artista
    /// mexer no primeiro ajuste.
    ///
    /// ⚠️ É um MODO e não um botão *"transformar em moldura"* pela razão do Shape: o gesto é
    /// **produzir**, e a tela quer ser desenhada onde vai ficar. Converter uma forma que já existe
    /// (o *Frame selection* do Figma) é a outra metade, e não está construída.
    Frame,
    /// ⭐⭐⭐ **Osso** (estudo 42 item 5, doc 47): arrastar no vazio faz um osso — a origem no press,
    /// o comprimento e o ângulo no arrasto.
    ///
    /// **O pai é o osso SELECCIONADO**, e o osso novo fica seleccionado ⇒ arrasto-arrasto-arrasto é
    /// uma cadeia, sem um único clique de cerimónia (o gesto do Spine, do Moho e do Rive). Clicar
    /// num osso que já existe selecciona-o, e é assim que se ramifica.
    ///
    /// ⚠️ **Ele não desenha uma FORMA** — [`Self::shape_kind`] devolve `None`. Um osso é uma
    /// entidade com `Transform` mais um `ph2d_skeleton_ecs::Bone`; a hierarquia dela **é** o esqueleto, e
    /// que a cinemática directa não precisa de uma linha de código (a propagação de `Transform` da
    /// casa já a faz) e que a timeline anima um osso sem saber que ossos existem.
    Bone,
}

impl DrawMode {
    /// ⭐⭐⭐ **O VOCABULÁRIO INTEIRO** — a lista de onde todo censo de modo se deriva.
    ///
    /// ⚠️⚠️ **Ela nasceu porque três censos escritos à mão deixaram passar o 15º modo em silêncio**
    /// (2026-08-31, a wave do Trim): um deles até dizia *"um modo novo tem de passar por aqui"*, e
    /// tinha um `assert_eq!(lista.len(), 14)` ao lado — que mede o comprimento da PRÓPRIA lista, e
    /// portanto concorda consigo mesma para sempre. *Um censo que se verifica contra a sua própria
    /// cópia não é um censo.*
    ///
    /// ⛔ Um `match` exaustivo protege quem **decide por variante**; não protege quem **itera**. Esta
    /// constante é a resposta para os segundos, e o gate abaixo prende-a ao `match`.
    pub const ALL: &'static [Self] = &[
        Self::Select,
        Self::Node,
        Self::Pen,
        Self::Pencil,
        Self::Shape,
        Self::Text,
        Self::Build,
        Self::Connect,
        Self::PickBlend,
        Self::Fillet,
        Self::Chamfer,
        Self::Width,
        Self::Trim,
        Self::Bucket,
        Self::Cut,
        Self::Frame,
        Self::Bone,
    ];

    /// **A forma que ESTE modo desenha** — `None` quando o gesto não produz forma nenhuma.
    ///
    /// ⚠️ **Uma porta só, e é o campo de VALORES que a exige.** O gesto de forma lê o `kind`
    /// daqui e os parâmetros do **slot desse `kind`** (`VectorTool::draw_config`); enquanto a
    /// moldura era `Rectangle` — a única forma da família que ignora todo parâmetro — as duas
    /// perguntas podiam divergir sem sintoma nenhum, e divergiam: os valores saíam da forma
    /// ATIVA DO CATÁLOGO. Com a moldura em `RoundRect` isso deixa de ser inofensivo — desenhar
    /// uma moldura com a estrela ativa leria o **número de pontas** como raio de quina.
    #[must_use]
    pub fn shape_kind(self, catalog: ShapeKind) -> Option<ShapeKind> {
        match self {
            DrawMode::Shape => Some(catalog),
            // ⚠️ **A moldura é um retângulo ARREDONDÁVEL, e o raio nasce ZERO.** O slot de
            // parâmetros da tool zera todo campo em px (`default_shape_values`: *"nasce em 0
            // (canto vivo) e o usuário autora"*), e `rounded_rect(_, 0)` devolve **literalmente**
            // `rectangle(a, b)` — a mesma função, não uma aproximação. Trocar o kind não move um
            // pixel da moldura que já existia; só abre os cinco ajustes da Round (raio, os três
            // desvios por canto, suavização), que o painel pinta sozinho por ler o kind do alvo.
            DrawMode::Frame => Some(ShapeKind::RoundRect),
            _ => None,
        }
    }

    /// As ferramentas de QUINA (Fillet / Chamfer): clicar-e-arrastar sobre uma quina para
    /// arredondá-la ou chanfrá-la. Uma porta única para os sítios que roteiam o gesto delas.
    #[must_use]
    pub fn is_corner_tool(self) -> bool {
        matches!(self, DrawMode::Fillet | DrawMode::Chamfer)
    }

    /// A ferramenta de quina quer CHANFRO (reta) em vez de arredondado? Só faz sentido quando
    /// [`Self::is_corner_tool`] — o `Chamfer` chanfra, todo o resto arredonda.
    #[must_use]
    pub fn corner_is_chamfer(self) -> bool {
        self == DrawMode::Chamfer
    }
}

/// **A FORMA da região que o marquee do modo Node desenha** — o retângulo de sempre, ou o LAÇO.
///
/// Não é um [`DrawMode`], e a distinção decide o resto: um modo é *o que a ferramenta na mão faz*,
/// e o marquee não é uma ferramenta — é o gesto que acontece ao pressionar o vazio DENTRO do modo
/// Node. Um 15º pill obrigaria a entrar nele, laçar uma vez e sair; a forma da região é uma
/// propriedade do gesto, não um lugar onde se está.
///
/// ⚠️ **Ela é PEGAJOSA e MOMENTÂNEA ao mesmo tempo, por [`Self::for_gesture`]** — o chip diz qual
/// é a de sempre (é a afordância: um atalho que ninguém descobre é uma feature que não existe), e
/// o **Ctrl** troca a de UM gesto (é a saída de fluxo: o laço serve a uma seleção em cinco, e
/// obrigar a ida-e-volta ao painel por causa dela é o que torna um modo pior que um modificador).
/// **Uma pergunta, uma porta** — o que este repo evita são duas *implementações*, não duas
/// entradas na mesma função.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MarqueeShape {
    /// O retângulo entre os dois cantos do arrasto — cobre o caso comum.
    #[default]
    Box,
    /// O **laço**: a região é o caminho que a mão desenhou, fechado da ponta ao começo. É o que
    /// alcança nós entremeados com outros, que nenhum retângulo separa.
    Lasso,
}

impl MarqueeShape {
    /// A outra — o que o Ctrl faz.
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Self::Box => Self::Lasso,
            Self::Lasso => Self::Box,
        }
    }

    /// **A porta única: que forma tem ESTE gesto?** `sticky` é o chip do painel; `ctrl` é o
    /// modificador segurado no PRESS.
    ///
    /// ⚠️ **Perguntada UMA vez, no press, e o resultado congela até soltar** — a mesma lei da
    /// régua do gesto de exposição da tira do Flip. Se ela fosse relida por movimento, largar o
    /// Ctrl no meio do arrasto morfaria a região sob a mão: o artista veria o caminho que desenhou
    /// virar um retângulo entre dois pontos que ele nunca escolheu.
    #[must_use]
    pub fn for_gesture(sticky: Self, ctrl: bool) -> Self {
        if ctrl { sticky.other() } else { sticky }
    }
}

#[cfg(test)]
mod direccao_do_peso_tests {
    use super::WeightDirection;

    /// ⭐⭐⭐ **A DIRECÇÃO COMPÕE A MAGNITUDE COM O SINAL DELA** — a lei que a ordem do dono criou
    /// (2026-09-19), medida na porta e não num `if` de um laço de input.
    ///
    /// ⚠️ **A 3.ª metade é a que impede o pincel de ficar INERTE:** uma magnitude negativa escrita
    /// à mão entra em valor absoluto, logo ela empurra com a mesma força **para o lado que os
    /// botões dizem** — nunca `0`. *Cortar a zero devolveria um pincel que não faz nada e não diz
    /// porquê, que é a família de reports que esta casa já pagou três vezes.*
    #[test]
    fn a_direccao_compoe_a_magnitude_com_o_sinal_dela() {
        assert!(
            (WeightDirection::Add.delta(0.15) - 0.15).abs() < 1e-12,
            "Add deixou de SOMAR"
        );
        assert!(
            (WeightDirection::Subtract.delta(0.15) + 0.15).abs() < 1e-12,
            "Subtract deixou de TIRAR"
        );
        for lado in WeightDirection::ALL {
            let d = lado.delta(-0.4);
            assert!(
                (d.abs() - 0.4).abs() < 1e-12,
                "{lado:?}: uma magnitude negativa deixou de entrar em ABSOLUTO ({d}) — o pincel \
                 fica inerte e calado"
            );
            assert_eq!(
                d.is_sign_negative(),
                lado == WeightDirection::Subtract,
                "{lado:?}: quem manda no sinal deixou de ser o BOTAO"
            );
        }
    }

    /// ⭐⭐ **O ÍNDICE SAI DA LISTA** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ **As duas metades:** cada lado acende o seu, e a lista tem exactamente a população do
    /// enum. *Sem a segunda, uma variante nova fora do `ALL` leria `0` e acenderia o primeiro
    /// segmento — é o defeito que o [`super::BoneAction::indice`] já pagou com três verbos.*
    #[test]
    fn o_indice_da_direccao_sai_da_lista() {
        for (i, lado) in WeightDirection::ALL.iter().enumerate() {
            assert_eq!(lado.indice(), i, "{lado:?} acende o segmento errado");
        }
        let mut vistos = 0usize;
        for lado in WeightDirection::ALL {
            vistos += match lado {
                WeightDirection::Add | WeightDirection::Subtract => 1,
            };
        }
        assert_eq!(
            vistos,
            WeightDirection::ALL.len(),
            "a lista e o enum deixaram de contar a mesma populacao"
        );
    }

    /// ⭐ **DE FÁBRICA ELA SOMA** — é o que o artista faz primeiro, e o que ele espera sem ter
    /// escolhido nada.
    #[test]
    fn de_fabrica_a_pincelada_soma() {
        assert_eq!(WeightDirection::default(), WeightDirection::Add);
    }
}

#[cfg(test)]
mod modo_do_peso_tests {
    use super::{WeightDirection, WeightMode};
    use ph2d_skeleton::Especie;

    /// ⭐⭐⭐ **O MODO ESCOLHE A ESPÉCIE DA MANCHA** — a porta que junta as três respostas do painel
    /// (modo · magnitude · direcção) numa só coisa (F29, ordem do dono de 2026-09-19).
    ///
    /// ⚠️ **As duas metades dizem coisas diferentes:** no cumulativo a direcção **manda no sinal**
    /// (é a lei que a wave anterior trouxe); no absoluto ela é **ignorada**, e isso é a lei e não um
    /// esquecimento — *«para que lado»* não tem resposta quando o gesto põe um valor. É por isso que
    /// o painel esconde os dois botões ali.
    #[test]
    fn o_modo_escolhe_a_especie_da_mancha() {
        assert_eq!(
            WeightMode::Cumulative.especie(0.15, WeightDirection::Add),
            Especie::Soma(0.15),
            "o modo cumulativo deixou de produzir uma mancha que SOMA"
        );
        assert_eq!(
            WeightMode::Cumulative.especie(0.15, WeightDirection::Subtract),
            Especie::Soma(-0.15),
            "no modo cumulativo a direccao deixou de mandar no sinal"
        );
        for lado in WeightDirection::ALL {
            assert_eq!(
                WeightMode::Absolute.especie(0.6, lado),
                Especie::Alvo(0.6),
                "{lado:?}: a direccao chegou ao modo ABSOLUTO, onde ela nao tem sujeito"
            );
        }
    }

    /// ⚠️ **A magnitude entra em ABSOLUTO nos dois modos** — ela é *quanto*/*quanto vale*, e um
    /// número negativo escrito à mão na caixa não pode deixar o pincel inerte nem pedir um peso
    /// negativo. *É a mesma cerca que a [`WeightDirection::delta`] já declara, e ela vale para a
    /// espécie nova pela mesma razão.*
    #[test]
    fn uma_magnitude_negativa_entra_em_absoluto_nos_dois_modos() {
        assert_eq!(
            WeightMode::Absolute.especie(-0.6, WeightDirection::Add),
            Especie::Alvo(0.6),
            "um alvo negativo deixou de ser lido em valor absoluto"
        );
        assert_eq!(
            WeightMode::Cumulative.especie(-0.15, WeightDirection::Add),
            Especie::Soma(0.15),
            "o modo cumulativo deixou de ler a magnitude em absoluto"
        );
    }

    /// ⭐⭐ **O ÍNDICE SAI DA LISTA, e a lista tem a população do enum** — a mesma lei (e o mesmo
    /// defeito medido) do [`WeightDirection::indice`].
    #[test]
    fn o_indice_do_modo_sai_da_lista() {
        for (i, modo) in WeightMode::ALL.iter().enumerate() {
            assert_eq!(modo.indice(), i, "{modo:?} acende o segmento errado");
        }
        let mut vistos = 0usize;
        for modo in WeightMode::ALL {
            vistos += match modo {
                WeightMode::Cumulative | WeightMode::Absolute => 1,
            };
        }
        assert_eq!(
            vistos,
            WeightMode::ALL.len(),
            "a lista e o enum deixaram de contar a mesma populacao"
        );
    }

    /// ⭐ **DE FÁBRICA ELE É CUMULATIVO** — é a lei que já existia e a que o dono aprovou em smoke.
    ///
    /// ⚠️ **Isto é o que mantém um rig já autorado com a mesma aparência:** toda mancha nova nasce
    /// `Soma`, logo a lei corre pelo caminho de sempre até o artista escolher o outro modo.
    #[test]
    fn de_fabrica_o_modo_e_cumulativo() {
        assert_eq!(WeightMode::default(), WeightMode::Cumulative);
    }
}

#[cfg(test)]
mod all_tests {
    use super::DrawMode;

    /// ⭐⭐ **A lista e o `match` contam a MESMA população.**
    ///
    /// A prova é um `match` exaustivo sobre um valor tirado da lista: acrescentar uma variante ao
    /// enum sem a pôr no [`DrawMode::ALL`] deixa este `match` com um braço a mais do que a lista
    /// alcança, e o `assert_eq!` do comprimento acusa. ⛔ Sem a contagem, o `match` sozinho passaria
    /// — ele é exaustivo sobre o ENUM, e a lista pode ser um subconjunto.
    #[test]
    fn the_list_and_the_enum_agree_on_the_population() {
        let mut vistos = 0usize;
        for m in DrawMode::ALL {
            vistos += match m {
                DrawMode::Select
                | DrawMode::Node
                | DrawMode::Pen
                | DrawMode::Pencil
                | DrawMode::Shape
                | DrawMode::Text
                | DrawMode::Build
                | DrawMode::Connect
                | DrawMode::PickBlend
                | DrawMode::Fillet
                | DrawMode::Chamfer
                | DrawMode::Width
                | DrawMode::Trim
                | DrawMode::Bucket
                | DrawMode::Cut
                | DrawMode::Frame
                | DrawMode::Bone => 1,
            };
        }
        assert_eq!(vistos, 17, "o vocabulario mudou — reveja quem o itera");
        // …e sem repetidos: uma variante duplicada na lista faria todo censo medi-la duas vezes.
        let mut ordenada: Vec<String> = DrawMode::ALL.iter().map(|m| format!("{m:?}")).collect();
        ordenada.sort();
        let antes = ordenada.len();
        ordenada.dedup();
        assert_eq!(antes, ordenada.len(), "ha' um modo repetido na lista");
    }
}
