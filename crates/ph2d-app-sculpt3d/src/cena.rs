//! ⭐ **A CENA VIVA** — os cinquenta e cinco campos do que uma escultura É.
//!
//! ⚠️⚠️ **Este ficheiro é a cura do tecto de LOC do `lib.rs`, e ela só ficou possível quando a
//! família saiu da shell.** A nota da Fase A dizia — e estava certa — que aquele tecto *«não se
//! cura por corte»*: a struct tem **52 campos PRIVADOS** que ~30 módulos irmãos leem, e mover a
//! declaração para um filho tirava-os da vista deles, porque um irmão não é descendente.
//!
//! ⭐⭐ **O que mudou foi o significado de `pub(crate)`.** Um campo privado declarado no **root**
//! de uma crate é visível no root e em todos os descendentes dele — isto é, na crate inteira:
//! *privado-no-root é exactamente `pub(crate)`*. Enquanto a família vivia na `shells/desktop`,
//! escrever `pub(crate)` abria os 52 campos às 306 mil linhas da shell, e por isso a privacidade
//! de módulo estava a fazer o trabalho de uma fronteira que não existia. Hoje ela existe: o
//! `pub(crate)` aqui alcança a família e **nada mais**, que é exactamente a cerca de ontem.
//!
//! ⇒ a conversão dos 52 campos **não afrouxa uma única visibilidade** — ela só a escreve no
//! vocabulário que a fronteira nova tornou honesto.

use super::*;

/// A cena 3D viva: os objetos, a câmera, o pincel e o pipeline que a desenha.
pub struct Sculpt3dScene {
    /// **A CENA é uma LISTA.** Nunca vazia — a invariante que torna
    /// [`Sculpt3dScene::obj`] total.
    pub(crate) objects: Vec<SceneObject>,
    /// Quem a mão está trabalhando. Sempre `< objects.len()`.
    ///
    /// ⚠️ **Ele NÃO é um modo escondido:** quem o move é o `aim` do PEN-DOWN
    /// (mirar uma peça a torna ativa), então "a ativa" é sempre *a última que
    /// você tocou* — e é por isso que a cena não precisa de um realce de seleção
    /// para ser honesta.
    ///
    /// ⚠️ **E ele não se move DENTRO de um gesto**, o que não é preferência: o
    /// `SculptStroke` dimensiona os planos por-vértice na malha em que o traço
    /// começou, então trocar de peça no meio escreve índices de uma malha noutra
    /// — mudo enquanto a nova for menor, **pânico** assim que for maior. Ver
    /// `Sculpt3dScene::aim`.
    pub(crate) active: usize,
    /// O próximo [`ObjectId`] a cunhar. Ele **nunca reusa**: um id reciclado
    /// faria uma entrada de undo velha nomear uma peça nova, que é exatamente o
    /// que o índice já fazia.
    pub(crate) next_id: u32,
    /// **A peça ISOLADA** — `None` quando a cena inteira está à vista.
    ///
    /// ⚠️ **Um id, e não um `hidden: bool` por peça.** Os dois guardam a mesma
    /// coisa hoje, e só este é incapaz de guardar um estado que ninguém pode
    /// autorar: com bandeiras existe *duas escondidas e uma à vista sem ninguém
    /// ter isolado*, e a pergunta *"o artista está isolado?"* passaria a ser
    /// respondida contando. Aqui a visibilidade é DERIVADA — ver
    /// [`Sculpt3dScene::visible_pieces`].
    ///
    /// ⚠️ **É estado de VISTA, e por isso não entra na história nem no
    /// documento.** Isolar não muda um vértice; é o mesmo lugar em que o onion
    /// da timeline mora (`TimelineState.onion`, não serializado). Um Ctrl+Z que
    /// re-escondesse peças gastaria um passo de undo sem devolver trabalho
    /// nenhum.
    pub(crate) isolated: Option<ObjectId>,
    /// ⭐⭐⭐ **AS PEÇAS QUE O OLHO DA HIERARQUIA FECHOU** — o espelho do
    /// `ph2d_ecs::Visibility`, reconstruído do mundo a cada quadro pelo
    /// [`crate::entities::entities_sync`].
    ///
    /// ⚠️⚠️ **Isto NÃO contradiz a nota do [`Self::isolated`] ao lado, e a
    /// diferença é qual estado é AUTORÁVEL.** Aquela recusa um `hidden: bool`
    /// por peça porque com bandeiras existiria *«duas escondidas e uma à vista
    /// sem ninguém ter isolado»* — um estado que **nenhum gesto de isolamento
    /// pode produzir**. Aqui é o contrário: o olho da Hierarquia é por-linha por
    /// construção, logo esse estado **é exactamente o que o artista autora**, e
    /// um id só não o sabe guardar.
    ///
    /// ⚠️ **CONJUNTO de ids e não um campo no [`super::objects::SceneObject`]**,
    /// pela mesma razão que o `SculptRowsSeen` já é: isto é um **espelho** do
    /// mundo e não estado desta cena, e um campo na peça convidaria alguém a
    /// escrevê-lo aqui — passando a haver duas respostas a *«esta peça
    /// aparece?»*, que divergem no primeiro `Ctrl+Z`.
    ///
    /// ⚠️ **Estado de VISTA**, como o isolamento: não entra na história nem no
    /// documento (o `Visibility` já viaja no projecto, do lado do ECS).
    pub(crate) escondidas: std::collections::BTreeSet<ObjectId>,
    /// **O ARM da topologia dinâmica** — ver [`dyntopo`], que é onde as três
    /// consequências de ligar estão escritas.
    ///
    /// ⚠️ **Da CENA e não da peça**, como o `symmetry` ao lado: é um modo de
    /// TRABALHO, e um interruptor que mudasse de posição ao trocar de peça é o
    /// que faz o artista desenhar o traço errado para descobrir onde ele está.
    pub(crate) dyntopo: dyntopo::Dyntopo,
    /// A malha da peça ativa **como ela estava no pen-down**, e só quando a
    /// topologia dinâmica está armada.
    ///
    /// ⚠️ **É o preço declarado do modo:** um traço que muda a contagem de
    /// vértices não tem janela por-índice para desfazer, então o desfazer dele é
    /// a malha inteira (a entrada `Remeshed`, que já é uma troca simétrica).
    /// Vazio fora do gesto — ele não é um cache, é a metade de trás de UM passo.
    pub(crate) dyn_before: Option<Box<ph2d_mesh::Mesh>>,
    /// **De onde veio cada vértice que o último refino criou** — reusado entre
    /// dabs, e por isso um campo em vez de um `Vec` local.
    ///
    /// ⚠️ Mora aqui e não no [`dyntopo::Dyntopo`] porque aquele é o estado que o
    /// ARTISTA autora (o interruptor e o detalhe) e é `Copy`; isto é rascunho.
    pub(crate) dyn_births: Vec<ph2d_mesh::Birth>,
    /// A renumeração que o último colapso produziu — o canal irmão do
    /// `dyn_births`, e scratch pela mesma razão: reusá-lo entre dabs é o que
    /// mantém a topologia dinâmica sem alocação no caminho quente.
    pub(crate) dyn_remap: ph2d_mesh::Remap,
    /// **Os buffers do passe de região que o refino roda** — mesma razão do
    /// `dyn_births` acima: eles são do tamanho da MALHA e nascer por dab os
    /// tornaria uma alocação por movimento do mouse, que é o que o
    /// [`ph2d_mesh::RegionScratch`] existe para evitar.
    pub(crate) dyn_region: ph2d_mesh::RegionScratch,
    /// ⭐⭐⭐⭐ **A MEMÓRIA DO CAMPO DE POSIÇÃO AO LONGO DO TRAÇO** — a fase da
    /// retícula que atravessa os carimbos.
    ///
    /// ⚠️ **Ela NÃO é rascunho, ao contrário dos três acima: ela é ESTADO DO
    /// GESTO**, e é por isso que tem um dono no pen-down
    /// ([`ph2d_quadflow::regiao::CampoDoTraco::esquece`]) e dois canais de
    /// tamanho — as **mesmas** duas portas por onde o `SculptStroke` sobrevive à
    /// topologia dinâmica. *Uma terceira resposta para «a malha mudou de
    /// tamanho» é como os dois lados passam a descrever vértices diferentes.*
    ///
    /// O mecanismo, e porque é do campo de posição e não do de orientação:
    /// [`ph2d_quadflow::regiao::CampoDoTraco`].
    pub(crate) pente_campo: ph2d_quadflow::regiao::CampoDoTraco,
    /// ⭐⭐ **A QUEIXA DO PASSE DE TOPOLOGIA JÁ FOI DITA NESTE TRAÇO?**
    ///
    /// ⚠️ **Report do dono, 2026-09-14: *«não vejo efeito com density»*.** Ele
    /// tinha razão e o pincel estava certo — o que faltava era o pincel DIZER
    /// porque não fez nada. Um verbo cujo efeito inteiro é sobre a topologia
    /// **parece partido** sempre que o passe não corre, e há três razões
    /// diferentes para isso (o modo desligado · a pilha de multiresolução
    /// montada · não haver aresta curta ao alcance) que o artista vê **iguais**:
    /// nada acontece.
    ///
    /// ⚠️ **Uma vez por TRAÇO, e não por dab** — um dab corre por movimento do
    /// ponteiro, e uma queixa por dab seria um log que ninguém lê. Reposta no
    /// pen-down.
    pub(crate) dyn_queixa_dita: bool,
    /// **A TABELA DE SLOTS DO DEVICE** — quem mora em cada índice do
    /// renderizador. Ver [`slots`], que é onde a lei dela está escrita.
    pub(crate) slots: Vec<ObjectId>,
    pub(crate) camera: Camera3d,
    pub(crate) renderer: MeshRenderer,
    pub(crate) drag: Option<Drag>,
    /// ⭐ **O ESTADO DO CORTE** (`Trim`) — ver [`super::trim_gesto::Trim`].
    pub(crate) trim: super::trim_gesto::Trim,
    pub(crate) last: (f32, f32),
    /// ⭐⭐⭐ **O ESTADO DA JANELA 3D** — a divisão, as câmeras dos quadrantes, e
    /// os dois gizmos que vivem por cima dela.
    ///
    /// ⚠️ **Dez campos num tipo só, e não dez campos aqui**: eles são uma coisa
    /// (*como a peça é OLHADA*), nascem juntos, morrem juntos e são lidos pelos
    /// mesmos três módulos. Espalhados na cena eles eram indistinguíveis dos
    /// sessenta que descrevem *o que a peça É* — e foram eles que levaram este
    /// ficheiro ao tecto de LOC. Ver [`viewports::Janela`].
    pub(crate) janela: viewports::Janela,

    /// **COM QUE LUZ** — ver [`ph2d_mesh_render::Lighting`]. VISTA, e não
    /// documento: escolher com que luz olhar não muda a escultura.
    pub(crate) lighting: ph2d_mesh_render::Lighting,
    /// A malha desenhada por cima da forma. Vista, como o [`Self::lighting`].
    pub(crate) wireframe: bool,
    /// ⭐⭐⭐⭐ **A VISTA DA GRADE** — o arame esconde a diagonal de cada
    /// triângulo ([`ph2d_mesh_render::wire_indices_com`]).
    ///
    /// ⛔⛔ **Ela existe porque o dono não conseguia VER o que a medição via.**
    /// As fileiras que o pente de topologia produz medem `10`–`23` arestas de
    /// mediana contra `2`–`4` do controlo — e afogam-se, porque numa malha
    /// triangulada elas são **uma família de arestas entre três** e o arame
    /// desenha as três com o mesmo traço. *Eu próprio as li como «retalhos».*
    ///
    /// ⚠️ Vista, como o [`Self::wireframe`]: não é salva.
    pub(crate) wire_grade: bool,

    pub(crate) brush: Brush,
    /// **A REFERÊNCIA de cada verbo** (`RefMode`), na ordem do `Verb::ALL`.
    ///
    /// ⚠️ **Ela mora aqui e não só no pincel** porque o `Brush::mode` é o
    /// DERIVADO: o painel reconstrói o `Sculpt3dUi` a cada quadro a partir
    /// destes campos, então uma escolha que vivesse só no pincel seria perdida
    /// na primeira troca de ferramenta — e a troca é justamente o gesto que a
    /// re-resolve.
    /// ⚠️ **O TAMANHO É DERIVADO, e era um `16` literal** — a segunda cópia de
    /// uma contagem que o [`ph2d_sculpt3d::Verb::ALL`] já responde. Ela sobreviveu
    /// enquanto o catálogo não crescia, e o dia em que a W6 acrescentou a faixa
    /// ela virou erro de tipo em dois arquivos do shell.
    pub(crate) verb_slots: [ph2d_panel_sculpt3d::state::VerbSlot; ph2d_sculpt3d::Verb::ALL.len()],
    /// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** — ver
    /// [`ph2d_panel_sculpt3d::UiLevel`].
    ///
    /// ⚠️ **A cena o guarda e NUNCA o lê**, e isso é o desenho e não descuido:
    /// o painel não guarda estado entre quadros (ele recebe um retrato novo a
    /// cada um), então a escolha tem de morar em algum lugar que sobreviva — e
    /// esse lugar é o mesmo que já guarda os outros valores autorados. Renderizar
    /// nada a partir dele é o que o mantém honesto: com que profundidade olhar
    /// não muda a escultura, e por isso ele também não é salvo.
    pub(crate) ui_level: ph2d_panel_sculpt3d::UiLevel,
    /// **O padrão do pincel é mostrado no barro?** — ver
    /// [`sculpt3d_preview::PreviewState`]. Nasce ligado.
    pub(crate) alpha_preview: bool,
    /// **A IMAGEM que o artista armou, e de que sprite ela veio** — o slot que
    /// sobrevive à troca de padrão.
    ///
    /// ⚠️ **Sem ele o chip do slot seria um controle que só sabe DESMARCAR-se:**
    /// escolher `Grain` tira o `Arc<AlphaImage>` do pincel, e o painel — que só
    /// vê o retrato — não teria o que re-armar. Lembrar é o que faz da fileira
    /// um SELETOR: os nove procedurais sobrevivem porque são nomes, e a imagem
    /// precisa de alguém que a segure.
    ///
    /// ⚠️ **UM campo com o par, e não dois `Option`**, porque um nome sem imagem
    /// (ou o contrário) é um estado que ninguém sabe pintar — e dois campos que
    /// têm de concordar é a forma como ele nasce.
    pub(crate) alpha_image: Option<(
        std::sync::Arc<ph2d_sculpt3d::AlphaImage>,
        std::sync::Arc<str>,
    )>,
    /// O raio autorado, em **pixels de tela** — ver [`DEFAULT_RADIUS_PX`]. O raio
    /// de MUNDO é derivado por dab, contra a câmera e o ponto de acerto.
    pub(crate) radius_px: f32,
    /// Onde o traço carimbou pela última vez, em pixels — **a âncora do
    /// espaçamento**, e ela é separada do `last` de propósito: o `last` é o
    /// delta de TODO arrasto (a órbita precisa dele por evento) e esta só anda
    /// quando um dab de fato saiu. Colapsá-las apagaria o carry.
    pub(crate) stroke_anchor: [f32; 2],
    /// ⭐⭐⭐ **O CAMINHO SOBRE A SUPERFÍCIE CONGELADA** — o acumulador da lei do
    /// passo no mundo ([`ph2d_sculpt3d::CaminhoNoMundo`]), que cura o vinco
    /// pontilhado junto à silhueta.
    ///
    /// ⚠️ **Ele é IRMÃO da âncora, não substituto:** a âncora conta píxeis de
    /// ecrã (é ela que decide quais pontos do caminho são CANDIDATOS) e este
    /// conta arco de superfície (é ele que decide quais candidatos CARIMBAM).
    /// Colapsá-los devolveria a pergunta que o defeito é: *quantos píxeis andei*
    /// não é *quanto barro percorri*.
    ///
    /// ⚠️ **Estado de TRAÇO**: nasce vazio, esquece no pen-up — senão o resíduo
    /// de um traço decidiria o primeiro dab do seguinte.
    pub(crate) caminho_no_mundo: ph2d_sculpt3d::CaminhoNoMundo,
    /// Onde a mão **pegou** — o ponto de mundo do pen-down e o pixel dele. É a
    /// âncora dos DOIS grips que puxam, e é dela que os dois derivam o mundo:
    /// o [`Grip::Hold`] mede o puxão total até aqui, o [`Grip::Hook`] mede o
    /// incremento entre dois passos do caminho.
    pub(crate) grab: Option<([f32; 3], (f32, f32))>,
    /// **ONDE O DEDO ESTÁ, ainda não carimbado** — o par do [`Self::grab`], e a
    /// razão de ele existir é que um `Grip::Hold` faz **um dab por EVENTO de
    /// ponteiro**: a ~1000 Hz de mouse e 60 fps são ~16 dabs por quadro, e o
    /// dab do `l-mode` custa 1,05 ms num pincel de 10 % (medido em
    /// `ph2d-sculpt3d/tests/it/measure_field_cost.rs`).
    ///
    /// ⚠️ **Descartar os intermediários é BYTE-IDÊNTICO, e isso é MEDIDO, não
    /// suposto** (`measure_what_coalescing_a_hold_gesture_changes`): 16 eventos
    /// contra 1, com puxões de 50 %, 100 % e 200 % do raio, dão **desvio máximo
    /// 0,000000** e a MESMA contagem de movidos — 17,9 ms viram 1,2.
    ///
    /// O mecanismo já estava escrito no `touched`: o `Grip::Hold` é **frozen**,
    /// então o laço percorre o conjunto congelado no pen-down e o alvo é medido
    /// contra o `pre`. A pegada é consultada nas posições VIVAS e por isso pode
    /// CRESCER — mas o que ela acrescenta pesa zero contra a posição congelada.
    /// *A ressalva que eu tinha escrito (`pode mudar quais vértices entram`)
    /// caiu na medição.*
    ///
    /// ⚠️ **É o mesmo argumento que o `Grip::Turn` já usa** para não andar o
    /// `walk` (`input.rs`): *N dabs com o mesmo total acumulado no
    /// mesmo lugar é trabalho idêntico repetido*. A diferença é que o `Turn`
    /// paga isso uma vez por evento e o `Hold` pagava dezesseis.
    pub(crate) pending_grab: Option<(f32, f32)>,
    /// **O ângulo já varrido** pelo gesto do Twist — ver [`TwistSweep`]. `None`
    /// fora de um gesto; o pen-down o zera.
    pub(crate) twist: Option<TwistSweep>,
    /// **O que o painel ARMOU** — `None` é o estado normal, em que o botão
    /// esquerdo esculpe.
    ///
    /// ⚠️ Estado de FERRAMENTA, não de gesto: ele sobrevive ao pen-up, como o
    /// verbo do pincel. Quem o desarma é o artista, clicando o mesmo botão.
    pub(crate) transform_arm: Option<ph2d_sculpt3d::TransformKind>,
    /// A sessão VIVA — a foto do pen-down. Vazia fora do gesto.
    pub(crate) transform: Option<ph2d_sculpt3d::MaskTransform>,
    /// Onde o dedo pousou: a âncora contra a qual todo gesto é medido.
    pub(crate) transform_from: (f32, f32),
    /// **O FILTRO está armado?** — o botão esquerdo roda o verbo corrente na
    /// malha INTEIRA em vez de esculpir sob o cursor.
    ///
    /// ⚠️ Estado de FERRAMENTA, como o [`Self::transform_arm`] acima, e
    /// **mutuamente exclusivo** com ele: os dois armam o MESMO botão, e um gesto
    /// que significasse as duas coisas não teria como escolher. Quem garante a
    /// exclusão são as duas portas de armar, uma vez cada.
    pub(crate) filter_arm: bool,
    /// **Onde o dedo pousou**, em x — a âncora da força.
    ///
    /// ⚠️ Só o eixo X, e não o par: a força é o arrasto HORIZONTAL, a lei da
    /// referência (o filtro de malha dela). Guardar o `y` seria carregar
    /// um número que ninguém lê.
    pub(crate) filter_from_x: f32,
    /// **QUAL lei o filtro roda** — escolhida no painel, não derivada do verbo.
    ///
    /// ⚠️ **É a mudança de premissa da W9b, e ela é visível:** enquanto a lei
    /// vinha de [`Verb::filter_kind`], três das sete eram INALCANÇÁVEIS por
    /// gesto nenhum (não existe pincel de Scale, de Sphere nem de Random). O
    /// verbo em mãos passa a apenas SEMEAR esta escolha ao armar — quem manda
    /// é o artista, e um verbo sem lei própria deixa a última escolha de pé.
    pub(crate) filter_law: ph2d_sculpt3d::FilterLaw,
    /// ⭐⭐⭐ **O QUE O ARTISTA AFINOU NO FILTRO DE TECIDO** — o referencial, os
    /// quatro números e os três eixos do *Force Axis*.
    ///
    /// ⚠️ **Um tipo e não cinco campos soltos**: eles são uma coisa (*como o
    /// filtro de tecido está afinado*), são GLOBAIS — um filtro não pertence a
    /// ferramenta nenhuma, e três dos cinco tipos não têm verbo — e são lidos
    /// pelo mesmo sítio. Soltos entre os sessenta campos que descrevem *o que a
    /// peça É* eram indistinguíveis do resto, e foram eles que levaram este
    /// ficheiro ao tecto de LOC. Ver [`filter::Tecido`].
    pub(crate) tecido: filter::Tecido,
    pub(crate) symmetry: Symmetry,
    /// **O rig de luz do artista** — as mesmas quatro lâmpadas que acendem a tinta
    /// do Painter (`ph2d-light`).
    ///
    /// ⚠️ A cena guarda uma INSTÂNCIA porque hoje ela é um viewport solto, e o
    /// viewport é o documento dela. Quando a escultura virar uma camada de um
    /// documento do Painter (W3.M4) o rig passa a ser o DELE — a estrutura já tem
    /// um dono só, e o que falta unificar é o dado. Um segundo rig permanente
    /// aqui seria exatamente o que `docs/3D/05.2` proíbe.
    pub(crate) rig: LightRig,
    /// **A CAVIDADE** — quanto a curvatura escurece a fresta e clareia a crista
    /// (`ph2d_mesh_render::shade`, `docs/3D/05.1` §4).
    ///
    /// ⚠️ Nasce em [`ph2d_mesh_render::DEFAULT_CAVITY`], que é **zero**, e o
    /// motivo é o mesmo do `FormRole::Clay` logo abaixo: o barro liso é o que a
    /// W3 entregou e o Enio aprovou, e um canal de sombreamento que se arma
    /// sozinho muda a arte de todo mundo que já esculpiu. O `Shift+C` o liga.
    pub(crate) cavity: f32,
    /// **Quanto do AMBIENTE COM DIREÇÃO entra** — o piso da difusa deixando de
    /// ser o mesmo número em toda direção.
    ///
    /// ⚠️ **Nasce em ZERO, como a `cavity` logo acima e pela mesma razão**: é um
    /// canal que muda a leitura de toda escultura já feita. E dois gates de GPU
    /// cobraram isso — o `the_two_lights_agree_where_the_form_turns_away` afirma
    /// que a luz do barro e a da tinta concordam onde a forma vira, e um piso
    /// direcional só no barro as separa. Levantar o slider diverge, e isso é
    /// aceito (é a classe do matcap); entregá-lo divergido não.
    pub(crate) env: f32,
    /// Quanto do AO ASSADO entra no sombreamento — o irmão da `cavity`, e o
    /// oposto dela na origem: a cavidade é derivada e existe sempre, o AO só
    /// existe depois de o artista pedir um bake.
    pub(crate) ao: f32,
    /// Quanto do AO DE TELA entra — e ele **nasce LIGADO**, ao contrário dos dois
    /// vizinhos, porque é o único dos três que é MEDIDO a cada frame.
    ///
    /// ⚠️ Ligar por default não muda a tela sozinho: o barro amostra um canal que
    /// vale zero enquanto ninguém rodar o passe, então o `1.0` diz *"mostre o que
    /// foi medido"*. E é ele que decide se o passe roda — zero é a resposta
    /// completa para *"não quero pagar por isto"*, sem um segundo interruptor.
    pub(crate) ssao: f32,
    /// **O ESPALHAMENTO SUB-SUPERFICIAL** — quanto, e até onde a luz viaja dentro
    /// do material.
    ///
    /// ⚠️ Ele nasce em **zero**, como o AO assado e ao contrário do AO de tela, e
    /// a razão é de outra família: os dois AOs são MEDIÇÕES da forma (mostrá-las
    /// é honesto), e este é um **MATERIAL**. Barro não é pele; ligar o
    /// espalhamento por padrão mudaria a aparência de toda escultura já feita
    /// para uma que ninguém pediu.
    ///
    /// ⚠️ E o `scatter` **não** mora aqui: ele é semeado pelo tamanho da peça a
    /// cada frame (`Sculpt3dScene::shade`), pela mesma razão do raio do AO de
    /// tela — um número guardado seria uma segunda verdade sobre o tamanho da
    /// peça, a que fica velha exatamente quando o artista a faz crescer.
    pub(crate) sss: f32,
    /// **Até onde a luz viaja, como FRAÇÃO do maior lado da peça.**
    ///
    /// ⚠️ **Fração e não comprimento, e é ela que resolve a tensão entre duas
    /// coisas certas.** Um comprimento absoluto guardado seria uma segunda
    /// verdade sobre o tamanho da escultura — a que fica velha exatamente quando
    /// o artista a faz crescer. Mas o alcance é **o número que decide o LOOK**, e
    /// derivá-lo inteiramente deixava o artista sem como julgá-lo. Guardar a
    /// FRAÇÃO mantém as duas: o alcance segue sendo função da peça, e o slider é
    /// o veredito de aparência, que nenhuma medição responde.
    pub(crate) sss_scatter: f32,
    /// **O que o botão de extract vai fazer** — ver [`ph2d_mesh::Extract`].
    ///
    /// ⚠️ Autorado, e por isso ele mora aqui e não num argumento do gesto: o
    /// artista ajusta a espessura, olha, extrai de novo. O tipo é o do KERNEL —
    /// dois `f32` soltos seriam um segundo lugar para o default morar.
    pub(crate) extract: ph2d_mesh::Extract,
    /// **Em que resolução o botão RECONSTRUIR voxeliza** — o slider da seção
    /// Topology.
    ///
    /// ⚠️ Guardado como CONTAGEM (`u32`), e não como o `f32` da pista: a pista é
    /// contínua porque pistas são contínuas, e a grandeza é um número de células.
    /// O arredondamento e o clamp moram na fronteira (`apply_ui`), não aqui.
    pub(crate) remesh_res: u32,
    /// O lado do quad que a retopologia persegue, em unidades de objeto — ver
    /// [`Sculpt3dScene::quad_remesh`].
    pub(crate) quad_detail: f32,
    /// Quanto a densidade da retopologia segue a curvatura.
    pub(crate) quad_adapt: f32,
    /// ⭐ **QUAL MOTOR de retopologia o botão chama** — ver
    /// [`ph2d_panel_sculpt3d::state::RetopoMode`].
    pub(crate) retopo_mode: ph2d_panel_sculpt3d::state::RetopoMode,
    pub(crate) stroke: SculptStroke,
    /// ⭐⭐⭐ **A SUPERFÍCIE QUE O ARTISTA VIU AO ENCOSTAR A CANETA** — contra
    /// ela é que este traço PICA, e só para os verbos que
    /// [`ph2d_sculpt3d::Verb::pica_na_superficie_do_pen_down`] nomeia.
    ///
    /// ⚠️⚠️ **Não é um cache e não é o [`Self::dyn_before`]**, e ler os dois
    /// como um seria o defeito clássico de duas respostas à mesma pergunta:
    /// aquele é a metade de trás de **um passo de undo** (existe só com a
    /// topologia dinâmica armada, e a sua vida é a da entrada `Remeshed`); este
    /// é o **alvo de uma consulta de raio** e existe por causa da lei do
    /// pincel. Fundi-los faria o pick do projectar depender de um interruptor
    /// de topologia — em silêncio.
    ///
    /// ⚠️ **Ela é escrita a CADA pen-down** (`Some` ou `None`), e não só quando
    /// é precisa: um traço anterior não pode deixar uma fotografia velha para o
    /// seguinte picar contra ela. O `close_stroke` limpa-a por memória, não por
    /// correcção.
    ///
    /// ⛔ **É uma cópia da malha inteira, e o preço está MEDIDO** — a sonda é a
    /// `diag_o_preco_da_fotografia` (em `--release`, mínimo de cinco com a
    /// mediana ao lado; ⚠️ a máquina estava a `load 40`, logo a mediana é o
    /// tecto de uma leitura suja e o mínimo é a que descreve o produto):
    ///
    /// | a peça | mínimo | mediana |
    /// |---|---|---|
    /// | `98 306` verts (fábrica) | **`0,70 ms`** | `4,51` |
    /// | `393 218` (`K`) | `2,55` | `4,55` |
    /// | `1 572 866` (`KK`) | `10,40` | `23,03` |
    ///
    /// ⭐ **E ela corre UMA vez por traço, no pen-down** — não por dab —, ao
    /// lado da cópia que a [`ph2d_sculpt3d::SculptStroke::pecas_da_cena`] já
    /// paga no mesmo instante e da que o [`Self::dyn_before`] paga com a
    /// topologia dinâmica armada: *o orçamento aqui é o do gesto que começa, não
    /// o dos `8 ms` de um dab*. ⭐ O octree viaja na cópia (ele é campo da
    /// [`ph2d_mesh::Mesh`]), logo o primeiro raio **não** reconstrói árvore
    /// nenhuma.
    pub(crate) superficie_do_pen_down: Option<Box<ph2d_mesh::Mesh>>,
    pub(crate) undo: Vec<Entry>,
    /// **O futuro guardado** — o que um Ctrl+Z tirou e um Ctrl+Shift+Z devolve.
    ///
    /// ⚠️ Ela é populada por [`Sculpt3dScene::undo_stroke`] e esvaziada por
    /// [`Sculpt3dScene::record`], nunca por quem edita: uma edição nova torna
    /// este futuro inalcançável, e a lei mora na porta que grava.
    pub(crate) redo: Vec<Entry>,
    /// Quantas vezes a MALHA mudou. Entra no carimbo da doação — ver
    /// `Sculpt3dScene::mesh_changed`, a porta única que o move.
    pub(crate) edits: u64,
    /// **O interruptor da doação** — ver [`FormRole`]. Nasce em `Clay` porque a
    /// primeira coisa que se faz com uma escultura é esculpi-la; a doação é o
    /// passo seguinte, e o `D` o dá.
    pub(crate) role: FormRole,
    /// **A posição ANTERIOR do barro**, a testemunha de que o modo VIROU — ver
    /// [`Sculpt3dScene::take_clay_edge`], a porta única que a lê.
    ///
    /// ⚠️ **Nasce `false` mesmo com o papel nascendo em `Clay`**, e é isso que
    /// apaga o caso especial: o primeiro frame de uma cena nova produz a borda
    /// *entrou*, então "abrir ao nascer" e "abrir ao voltar" deixam de ser duas
    /// regras — são a MESMA.
    pub(crate) clay_was_on: bool,
    /// **O rig de quando alguém perguntou pela última vez** — a testemunha de que o artista MEXEU
    /// na lâmpada; ver [`Sculpt3dScene::take_rig_edge`], a porta única que a lê.
    ///
    /// ⚠️ **Nasce com o rig ATUAL**, ao contrário do [`Self::clay_was_on`] logo acima, e a
    /// assimetria é o conserto: uma cena recém-criada não mexeu em lâmpada nenhuma, e tratar o
    /// nascimento dela como um gesto reescreve a luz de todo objeto já assado no documento.
    pub(crate) rig_was: ph2d_form_donation::baked_form::RigStamp,
    /// O carimbo da última doação entregue — `None` enquanto nada foi doado.
    pub(crate) donated: Option<FormStamp>,
    /// ⭐⭐⭐ **O QUE A ESCULTURA TEM A DIZER** — as recusas e os vereditos que o artista precisa de
    /// LER, à espera de quem tem a fila de avisos.
    ///
    /// ⛔⛔ **Elas viviam só no `eprintln!`**, e o doc do [`RemeshRefusal::explain`] chamava-lhes
    /// por escrito *«a FRASE que o artista lê»*: o botão `Quad Retopology` recusava com a cura
    /// dentro da frase — *ACHATE a pilha antes*, *suba o `Detail`* — e no ecrã não acontecia
    /// **nada**. Um verbo que recusa em silêncio é indistinguível de um verbo partido (§5.0), e o
    /// terminal não é uma superfície do produto: o dono corre o smoke a partir dele, o artista não.
    ///
    /// ⚠️ **Uma CAIXA DE SAÍDA, não uma chamada** (ADR-0075): a cena não conhece a `ToastQueue` — a
    /// shell drena-a uma vez por quadro, no mesmo sítio onde já drena os pedidos do painel. É isso
    /// que deixa as DUAS entradas (a tecla e o botão do painel) falarem pela mesma porta.
    pub(crate) avisos: Vec<String>,
}

impl Sculpt3dScene {
    /// ⭐⭐ **A escultura FALA** — a recusa (ou o veredito) entra na caixa de saída E no terminal.
    ///
    /// ⚠️ **Os dois, de propósito:** o terminal é onde uma bissecção lê a sequência inteira; o ecrã
    /// é onde o artista está. Uma porta só, para que uma delas não possa envelhecer sem a outra.
    pub(crate) fn fala(&mut self, msg: String) {
        eprintln!("[sculpt3d] {msg}");
        self.avisos.push(msg);
    }

    /// O que a escultura tem a dizer, **desarmando** a caixa — a forma que o quadro consome.
    ///
    /// ⚠️ Um `take_*` e não um campo público lido à mão: um aviso que se lê sem se desarmar
    /// reaparece a cada quadro, e um toast repetido 60 vezes por segundo não é um aviso, é um muro.
    pub fn take_avisos(&mut self) -> Vec<String> {
        core::mem::take(&mut self.avisos)
    }
}
