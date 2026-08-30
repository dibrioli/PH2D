//! ⭐ **As dimensões de uma forma** — o que ela mede, e o que se pode escrever nela.
//!
//! # Por que isto existe
//!
//! Até aqui a única coisa editável de uma primitiva era o **raio do filete**. Um modelador em que
//! não se consegue dizer *"este cilindro tem 20 de raio e 50 de altura"* não é um modelador de
//! precisão — é um de escala uniforme, que é o gesto que sobra quando não há números.
//!
//! # A divisão: o documento dá a PAREDE, a vista dá o CONFORTO
//!
//! Cada grandeza diz o que **admite** ([`Dim::span`]) — o `round` de uma caixa não pode chegar à
//! meia-extensão dela, porque a fonte encolhida deixaria de existir. Isso é do documento e não se
//! negoceia.
//!
//! ⛔ **O teto de um slider NÃO é isso.** A largura de uma caixa não tem limite nenhum: escrever um
//! aqui seria inventar um número que a física não pede — o que o [`CLAUDE.md §0`] proíbe. Quem
//! escolhe até onde o **gesto** vai é a vista, e a resposta natural é *o que cabe no enquadramento*
//! — uma dimensão maior do que o quadro é uma cujo efeito não se vê. O campo numérico continua sem
//! teto, porque digitar 1000 é uma afirmação sobre a peça e não sobre a janela.
//!
//! # ⚠️ Uma faixa tem DUAS pontas, e o piso não é sempre zero
//!
//! Foi o que faltou à primeira versão: [`Dim`] só dizia o **teto**, e o painel punha o piso em zero
//! para todas as linhas. Numa largura isso está certo (o documento recusa ≤ 0); numa **posição** é um
//! defeito com sintoma mudo — digitar `-0,5` era reescrito para `0` pelo espelho do controle, e a
//! peça ia para a origem. O smoke não o apanhou porque o número experimentado foi positivo.
//!
//! Daí o [`Span`]: cada grandeza diz a **forma** da sua faixa e de que recurso vem cada ponta, e
//! quem fecha as pontas abertas é a vista, num sítio só.
//!
//! # Meias-extensões não aparecem
//!
//! O documento guarda **meias**-extensões (é a forma que a distância assinada quer). Ninguém diz que
//! uma caixa tem «meia-largura 5»: [`dims`] devolve a largura **inteira** e [`set_dim`] volta a
//! dividir. A conversão mora aqui, num sítio, e não em cada painel que a mostre.
//!
//! [`CLAUDE.md §0`]: ../../../CLAUDE.md

/// ⭐ **Que número autorado de um nó** — a identidade de uma linha do painel.
///
/// ⚠️ Um `usize` cru serviria, com o painel a saber que «0..2 é a posição e o resto são dimensões».
/// Uma convenção implícita entre duas crates é o tipo de coisa que sobrevive até alguém acrescentar
/// uma linha no meio — e aí o controle passa a escrever noutro número, em silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    /// A translação **local** do nó, por eixo (0 = X).
    ///
    /// ⚠️ **Local, e é a convenção da casa**: o Inspector dela mostra o `Transform.translation`, que
    /// é local, e o readout do gizmo 2D diz por extenso que o delta é local *"porque é isso que o
    /// Inspector mostra"*. Um painel que mostrasse mundo contradiria o número ao lado no dia em que
    /// alguém agrupasse.
    Pos(u8),
    /// Um dos três ângulos da rotação **local**, por eixo (0 = X), em **graus**.
    ///
    /// ⚠️ A pose guarda um **quaternion**; estes três são o nome canónico dele. Ver
    /// [`crate::xform::set_rotation_degree`], que é onde a lei (e o que ela recusou) está escrita.
    Rot(u8),
    /// A escala **uniforme** do nó. Ver a nota de [`crate::Xform::scale`].
    Scale,
    /// Uma dimensão da forma — a posição na lista de [`dims`].
    Dim(u16),
    /// Um número de um **modificador** — `slot` é a posição na pilha, `field` é qual dos números
    /// dele. Ver [`crate::mods`].
    ///
    /// ⚠️ **A posição, e não a natureza**: a pilha pode ter duas cascas, e uma chave por natureza
    /// não as distinguiria — escrever numa escreveria na outra.
    ///
    /// ⚠️ **E DOIS índices, não um**: uma matriz tem quantas cópias *e* que espaçamento. Um índice
    /// só obrigaria a achatar a pilha inteira numa lista de números, e aí inserir um modificador no
    /// meio renumeraria tudo o que vem depois — com um arrasto a meio a escrever noutro campo.
    Mod { slot: u16, field: u8 },
    /// ⭐⭐ **O NÍVEL DE RESOLUÇÃO de uma forma que ainda está ligada ao desenho** (W55).
    ///
    /// ⚠️ **Não é uma dimensão da FORMA, e é por isso que tem chave própria.** Um `Dim` diz o que a
    /// peça mede — largura, altura, filete —, e mexer nele muda a peça. Este número não muda a peça
    /// nenhuma: muda **com que finura o contorno desenhado é convertido** nela. As duas coisas vivem
    /// em sítios diferentes (a forma no nó, o vínculo ao lado dele) e sobrevivem a gestos
    /// diferentes — largar o vínculo apaga este número e deixa a forma intacta.
    ///
    /// O teto é [`crate::MAX_PROFILE_RESOLUTION`], e ele é medido.
    Resolution,
    /// ⭐⭐⭐ **O RAIO DA JUNÇÃO desta forma** (W98) — com que arredondamento ela se encontra com o
    /// resultado das anteriores.
    ///
    /// ⚠️ **Não é o [`Param::Dim`] do filete da forma, e a diferença é o SUJEITO.** O `Dim` do
    /// arredondamento é das arestas **dela própria** — as 12 de uma caixa, o aro de um cilindro — e
    /// existe mesmo numa peça de uma forma só. Este é do **encontro**, e só existe porque há alguma
    /// coisa antes. Uma caixa arredondada que corta com aresta viva precisa dos dois números ao
    /// mesmo tempo, e uma chave só não os saberia distinguir.
    ///
    /// ⭐ **Escrever aqui MATERIALIZA o verbo** quando a forma o estava a herdar: pedir um raio de
    /// junção próprio *é* pronunciar-se. O painel mostra isso na hora — o chip `Inherit` apaga-se e
    /// acende o verbo que ela agora tem por escrito.
    ///
    /// ⚠️ A **base** não tem esta chave: ela semeia o acumulado e não se junta a nada
    /// ([`crate::fold_verb`]).
    Joint,
}

/// ⭐ **O que uma grandeza admite** — a forma da faixa, e de que recurso vem cada ponta.
///
/// ⚠️ Nenhuma variante escolhe um número por conforto: ou a ponta é do **documento** (a peça deixa
/// de existir acima dela), ou é da **representação** (um ângulo canónico não passa de meia volta),
/// ou está **aberta** e quem a fecha é a vista — que é a única a saber o que cabe no quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Span {
    /// Positiva e sem parede: uma largura, um raio, uma escala. O documento recusa `≤ 0`; o teto é
    /// o alcance da **vista**.
    Positive,
    /// Positiva, com **parede** do documento. Acima de `wall` a forma degenera — um dente de
    /// engrenagem de largura zero, uma moldura sem espessura. ⛔ **O zero NÃO passa**, e é essa a
    /// diferença para a [`Span::WallFromZero`].
    Wall(f32),
    /// ⭐⭐⭐ **Com parede E com o zero dentro** — a faixa dos DOIS RECUOS de uma aresta, o filete e o
    /// chanfro.
    ///
    /// # ⛔ Ela cura um defeito PRÉ-EXISTENTE, e a lei que o nomeia já estava neste arquivo
    ///
    /// O filete usava [`Span::Wall`], e o painel mapeia essa faixa para um slider que **começa em
    /// zero** — mas a porta de escrita recusa o zero, porque `Wall` promete «positiva». ⇒ o artista
    /// arredondava uma aresta e **não conseguia desarredondá-la**: o controle descia até ao fundo e
    /// o número parava logo acima dele, sem dizer porquê.
    ///
    /// ⚠️ É exactamente o que o doc da [`Span::Count`] já descreve, e por isso ela ganhou o `min`:
    /// *«uma faixa que oferece o que a porta recusa é uma affordance que mente»*. A cura ali foi um
    /// piso declarado; aqui é o zero declarado.
    ///
    /// ⭐ E para o **chanfro** isto não é conforto: zero é o estado de nascimento dele, e uma faixa
    /// que não o alcança faria um knob que só liga.
    ///
    /// ⛔ **A `Wall` fica como está**, e a distinção é o que ela protege: num dente de engrenagem ou
    /// numa espessura de moldura o zero é a forma a deixar de existir, não um estado que se pede.
    WallFromZero(f32),
    /// Simétrica e sem parede nenhuma: uma **posição**. As duas pontas são o alcance da vista, e a
    /// de baixo é negativa — a origem não é um canto do mundo.
    Free,
    /// **Periódica**: um ângulo. As pontas são `±half` e são a própria **representação** — nem o
    /// documento nem a vista têm voto, e um número além delas não é recusado, é renomeado.
    Turn(f32),
    /// ⭐ **Não há faixa nenhuma agora**: a grandeza existe, tem valor, e **não é editável neste
    /// estado**.
    ///
    /// ⚠️ É diferente de *"não aparece"*. O valor continua a ser um facto que o artista precisa de
    /// ler — e esconder a linha faria o painel saltar de tamanho a cada travessia. O que ela perde é
    /// o **controle**: quem a recebe pinta um facto, não um slider (*uma affordance que não pode ser
    /// honrada é pior do que nenhuma*).
    ///
    /// O caso de hoje é o terceiro ângulo na trava de cardan — ver
    /// [`crate::xform::rotation_axis_is_free`], que é a **mesma** porta que recusa a escrita.
    Locked,
    /// ⭐ **Simétrica, e fechada pelo DOCUMENTO**: `±max`, sem a vista ter voto.
    ///
    /// ⚠️ É a irmã da [`Span::Free`] com as pontas fechadas, e a diferença é de onde vem o número:
    /// numa posição não há limite nenhum e a vista escolhe o alcance; aqui o limite é um **facto**
    /// do documento — hoje, o custo de marcha que a inclinação paga
    /// ([`crate::mods::MAX_TAPER_SLOPE`]).
    Walls(f32),
    /// ⭐ **Uma CONTAGEM**: inteira, de `min` a `max`. Quantas cópias uma matriz tem, quantos lados
    /// um prisma tem.
    ///
    /// ⚠️ É uma faixa **própria** e não uma `Positive` disfarçada, porque três coisas mudam de uma
    /// vez: o passo do arrasto é **1** (e não um centésimo do curso), o número mostra-se **sem
    /// casas decimais** (não existe meia cópia), e o piso não é zero.
    ///
    /// ⚠️ **O `min` é um campo desde a W101**, e ele nasceu de um caso concreto: uma matriz começa
    /// em **1** (zero cópias é a peça a desaparecer, e apagar já tem botão) e um prisma começa em
    /// **3** (abaixo disso não há polígono). Com o piso fixo em `1`, o slider do prisma descia a 1,
    /// a escrita era recusada, e o controle **saltava para trás debaixo do dedo** — *uma recusa é
    /// informação, mas uma faixa que oferece o que a porta recusa é uma affordance que mente.*
    Count { min: u32, max: u32 },
    /// ⭐ **Positiva OU ZERO**, com o teto vindo da vista — a irmã da [`Span::Positive`] com o zero
    /// dentro.
    ///
    /// ⚠️ Ela existe por **uma** grandeza, e ela é a razão de ser da forma: o raio do TOPO de um
    /// [`crate::Primitive::Cone`], cujo zero **é o cone fechado**. Com `Positive` o documento recusa
    /// o zero e a forma que dá nome à primitiva fica indigitável; com `Free` o slider oferece
    /// negativo, que não quer dizer nada.
    FromZero,
}

/// Uma grandeza editável de um nó.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dim {
    /// A chave i18n do nome. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    pub key: &'static str,
    /// O valor que o artista vê — já em unidades inteiras (ver o doc do módulo).
    pub value: f32,
    /// **O que ela admite**, e de onde vem cada ponta. Ver [`Span`].
    pub span: Span,
}

/// ⭐ A metade de **escrita** — ver [`dims_write`].
#[path = "dims_write.rs"]
mod dims_write;

pub use crate::dims_scale::scale_primitive;
pub use dims_write::{clamp_round, set_dim};

/// ⭐ A tabela por-forma — ver [`dims_table`].
#[path = "dims_table.rs"]
mod dims_table;
pub use dims_table::dims;
