//! **O MODO de desenho** — o enum `DrawMode` e as portas que ele responde.
//!
//! Irmão de [`super::params`] pelo teto de 700 LOC, e o corte é por assunto: aqui mora *o que o
//! gesto de canvas está a fazer*, e não os parâmetros numéricos de um estilo. Re-exportado dali,
//! então quem consome não percebe o corte.

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
    /// O gesto é o do retângulo, e literalmente: `shape_kind_for_mode` devolve
    /// `ShapeKind::Rectangle` também para este modo, então restrição de Shift/Alt, pose, undo e
    /// Live Shape vêm de graça. O que a moldura acrescenta ao nascer é **um componente**
    /// (`ph2d_ecs::VecFrame`) — ela É um retângulo vivo, e é essa decisão que lhe dá fill,
    /// gradiente, traço, raio de quina, efeitos, gizmo, z-order e save sem uma linha a mais.
    ///
    /// ⚠️ É um MODO e não um botão *"transformar em moldura"* pela razão do Shape: o gesto é
    /// **produzir**, e a tela quer ser desenhada onde vai ficar. Converter uma forma que já existe
    /// (o *Frame selection* do Figma) é a outra metade, e não está construída.
    Frame,
}

impl DrawMode {
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
