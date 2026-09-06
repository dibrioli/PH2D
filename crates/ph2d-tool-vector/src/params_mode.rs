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
    /// entidade com `Transform` mais um `VecBone`; a hierarquia dela **é** o esqueleto, e é por isso
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
