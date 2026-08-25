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
    /// **Setas do Morph**: pressiona sobre uma forma, arrasta, solta sobre outra — nasce uma
    /// **seta** na máquina de estados do Morph selecionado (plano 32 W3b).
    ///
    /// ⚠️ **É o gesto do [`DrawMode::Connect`] com outro produto**, e por isso é um MODO próprio e
    /// não uma variante dele: o conector produz uma **linha no documento** (que exporta, imprime e
    /// se selecciona); esta seta produz uma **aresta num grafo** — chrome, a explicação de uma
    /// regra. Dois produtos diferentes atrás do mesmo movimento da mão precisam de dois modos,
    /// senão o artista não tem como dizer qual deles quer.
    MorphLink,
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
}

impl DrawMode {
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
