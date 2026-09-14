//! **O painel da cena 3D** (`SCULPT3D_*`) — ADR-0150, W12.
//!
//! A família de ids do `ph2d-panel-sculpt3d`: a lista de ferramentas, os knobs
//! do pincel, o espelho, a topologia, o sombreamento e a lista de peças.
//!
//! Slug pontilhado (`sculpt3d.*`), como a família do painel de física. Hash de
//! string, então nenhum contador de id se move — o `node_id_collisions` varre
//! estas chaves como varre as outras.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/sculpt3d.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Botão de fechar (X).
pub const SCULPT3D_CLOSE: NodeId = hash_node_id("sculpt3d.close");

// ── Cabeçalhos de seção ─────────────────────────────────────────────────────
// Dobráveis, e têm de ser: o `paint_section_header` SEMPRE pinta o chevron,
// então um cabeçalho sem id vivo desenha um "clique para dobrar" que não faz
// nada.
/// A ferramenta em mãos (os 16 verbos).
pub const SCULPT3D_SEC_TOOL: NodeId = hash_node_id("sculpt3d.sec.tool");

/// Os knobs do pincel (raio, força, falloff, e os dois condicionais).
pub const SCULPT3D_SEC_BRUSH: NodeId = hash_node_id("sculpt3d.sec.brush");

/// Os espelhos.
pub const SCULPT3D_SEC_SYMMETRY: NodeId = hash_node_id("sculpt3d.sec.symmetry");

/// A resolução da malha — topologia dinâmica, multires, remesh.
pub const SCULPT3D_SEC_TOPOLOGY: NodeId = hash_node_id("sculpt3d.sec.topology");

/// Como a forma é LIDA — cavidade e luz.
pub const SCULPT3D_SEC_SHADING: NodeId = hash_node_id("sculpt3d.sec.shading");

/// A lista de peças e os verbos que a mexem.
pub const SCULPT3D_SEC_SCENE: NodeId = hash_node_id("sculpt3d.sec.scene");

/// **A ENTREGA** — o que a forma escreve num objeto da cena 2D.
///
/// ⚠️ **Seção própria, e não uma linha na cauda do sombreamento**, porque o
/// assunto é outro: as cinco de cima descrevem *como a escultura é*, esta
/// descreve *o que sai dela para a arte*. E ela é a ÚLTIMA pela mesma lei que
/// pôs a cena no fim — quanto mais raro o gesto, mais fundo ele pode estar.
pub const SCULPT3D_SEC_BAKE: NodeId = hash_node_id("sculpt3d.sec.bake");

// ── A ferramenta ────────────────────────────────────────────────────────────
/// Um chip por verbo, na ordem de `ph2d_sculpt3d::Verb::ALL`.
///
/// ⚠️ **A contagem NÃO é escrita aqui em prosa.** Ela já esteve — dizia
/// *"os 19 verbos"* sobre um array de vinte, porque o polegar e a lâmina em V
/// entraram e ninguém releu a linha de cima. O número que vale é o do `ALL`, e
/// quem o afirma é o gate logo abaixo.
///
/// ⚠️ **O tamanho é o do `Verb::ALL`, e o gate o compara** — um verbo novo sem
/// chip aqui é uma ferramenta que o artista não alcança, que é exatamente o que
/// aconteceu com o `Magnify` antes de ele ganhar a tecla `A`.
pub const SCULPT3D_VERB: [NodeId; 31] = [
    hash_node_id("sculpt3d.verb.0"),
    hash_node_id("sculpt3d.verb.1"),
    hash_node_id("sculpt3d.verb.2"),
    hash_node_id("sculpt3d.verb.3"),
    hash_node_id("sculpt3d.verb.4"),
    hash_node_id("sculpt3d.verb.5"),
    hash_node_id("sculpt3d.verb.6"),
    hash_node_id("sculpt3d.verb.7"),
    hash_node_id("sculpt3d.verb.8"),
    hash_node_id("sculpt3d.verb.9"),
    hash_node_id("sculpt3d.verb.10"),
    hash_node_id("sculpt3d.verb.11"),
    hash_node_id("sculpt3d.verb.12"),
    hash_node_id("sculpt3d.verb.13"),
    hash_node_id("sculpt3d.verb.14"),
    hash_node_id("sculpt3d.verb.15"),
    hash_node_id("sculpt3d.verb.16"),
    hash_node_id("sculpt3d.verb.17"),
    hash_node_id("sculpt3d.verb.18"),
    hash_node_id("sculpt3d.verb.19"),
    hash_node_id("sculpt3d.verb.20"),
    hash_node_id("sculpt3d.verb.21"),
    hash_node_id("sculpt3d.verb.22"),
    hash_node_id("sculpt3d.verb.23"),
    hash_node_id("sculpt3d.verb.24"),
    hash_node_id("sculpt3d.verb.25"),
    hash_node_id("sculpt3d.verb.26"),
    hash_node_id("sculpt3d.verb.27"),
    hash_node_id("sculpt3d.verb.28"),
    hash_node_id("sculpt3d.verb.29"),
    hash_node_id("sculpt3d.verb.30"),
];

/// **A REFERÊNCIA que o verbo corrente segue** — os chips `S` · `B` · `L`.
///
/// ⚠️ **O array tem os TRÊS e o painel pinta só os OFERECIDOS**
/// (`RefMode::offered_for`), e o índice é a posição no `RefMode::ALL` — nunca a
/// posição na fileira desenhada. Indexar pela fileira faria o id de um chip
/// mudar de significado no dia em que um modo passasse a ser oferecido, e o
/// clique do artista pousaria noutro modo sem nada reclamar.
pub const SCULPT3D_REF_MODE: [NodeId; 3] = [
    hash_node_id("sculpt3d.ref_mode.0"),
    hash_node_id("sculpt3d.ref_mode.1"),
    hash_node_id("sculpt3d.ref_mode.2"),
];

/// **A DUREZA DA PONTA da faixa** — o slider e o chip numérico.
pub const SCULPT3D_TIP_ROUNDNESS: NodeId = hash_node_id("sculpt3d.tip_roundness");

/// O chip numérico da dureza da ponta.
pub const SCULPT3D_TIP_ROUNDNESS_NUM: NodeId = hash_node_id("sculpt3d.tip_roundness.num");

/// **O COMPRIMENTO da faixa**, em raios — o slider e o chip numérico.
pub const SCULPT3D_STRIP_LENGTH: NodeId = hash_node_id("sculpt3d.strip_length");

/// O chip numérico do comprimento da faixa.
pub const SCULPT3D_STRIP_LENGTH_NUM: NodeId = hash_node_id("sculpt3d.strip_length.num");

/// **A ABERTURA DO V da lâmina** — o slider e o chip numérico.
pub const SCULPT3D_SCRAPE_ANGLE: NodeId = hash_node_id("sculpt3d.scrape_angle");

/// O chip numérico da abertura do V.
pub const SCULPT3D_SCRAPE_ANGLE_NUM: NodeId = hash_node_id("sculpt3d.scrape_angle.num");

/// **O V É LIDO DA SUPERFÍCIE** — o toggle do modo dinâmico.
pub const SCULPT3D_SCRAPE_DYNAMIC: NodeId = hash_node_id("sculpt3d.scrape_dynamic");

/// **A ESPESSURA DA DEMÃO** — o slider e o chip numérico do
/// `ph2d_sculpt3d::Verb::Layer`.
pub const SCULPT3D_LAYER_HEIGHT: NodeId = hash_node_id("sculpt3d.layer_height");

/// **A FRACÇÃO DO RAIO QUE A NORMAL DO GESTO LÊ** — o slider e o chip numérico
/// do `ph2d_sculpt3d::Brush::normal_radius_frac`, que só o polegar e o empurrão
/// consomem.
pub const SCULPT3D_NORMAL_RADIUS: NodeId = hash_node_id("sculpt3d.normal_radius");

/// A caixa numérica da fracção acima — o par que toda row de knob tem.
pub const SCULPT3D_NORMAL_RADIUS_NUM: NodeId = hash_node_id("sculpt3d.normal_radius.num");

/// **A ÂNCORA CAI NUM VÉRTICE** — a caixa do `Brush::grab_active_vertex`, que só
/// o agarrar oferece.
///
/// ⚠️ **Este bloco já esteve ENTRE a doc da fracção e a const dela**, e o efeito
/// é mudo: a doc passa a documentar o vizinho e a const fica nua. É a família
/// que o split do painel 2D já pagou em 2026-07-19, e ela morde na inserção,
/// não na escrita.
pub const SCULPT3D_GRAB_ANCHOR: NodeId = hash_node_id("sculpt3d.grab_anchor");

/// O chip numérico da espessura da demão.
pub const SCULPT3D_LAYER_HEIGHT_NUM: NodeId = hash_node_id("sculpt3d.layer_height.num");

/// **QUÃO LARGO é o campo elástico** — os chips `Mono` · `Bi` · `Tri`.
///
/// ⚠️ **Só existe onde o campo existe** (`RefMode::field(verb).is_some()`), que
/// é a MESMA porta que o motor pergunta antes de consumir o kernel. Uma fileira
/// oferecida onde o campo não corre seria três chips que não movem um vértice —
/// o controle morto que este painel varre a cada wave.
pub const SCULPT3D_ELASTIC_SCALES: [NodeId; 3] = [
    hash_node_id("sculpt3d.elastic_scales.0"),
    hash_node_id("sculpt3d.elastic_scales.1"),
    hash_node_id("sculpt3d.elastic_scales.2"),
];

/// Carimba a referência do verbo corrente em TODAS as ferramentas.
///
/// ⚠️ **Um gesto, não uma segunda verdade** (§1.3 do plano): o estado é por
/// verbo, e um seletor global ao lado dele seriam duas portas para o mesmo fato.
pub const SCULPT3D_REF_MODE_ALL: NodeId = hash_node_id("sculpt3d.ref_mode.all");

/// **O FILTRO** — arma o botão esquerdo para rodar o verbo corrente na MALHA
/// INTEIRA, com o arrasto horizontal a dar a força.
///
/// ⚠️ **Ele mora no card da FERRAMENTA, ao lado do verbo e da referência, e não
/// junto do transform:** o transform é a sexta coisa que se faz com uma
/// *máscara pintada* e por isso vive lá; o filtro não tem operando próprio — ele
/// **É** a ferramenta na mão, aplicada de uma vez. Quem procura *"e se eu
/// quisesse isto na peça toda?"* procura onde escolheu a ferramenta.
///
/// ⚠️ **`toggle` e não um grupo de um**, e a distinção é a cerca que o
/// [`SCULPT3D_REF_MODE`] já escreve: *um modo só não é uma escolha*. Aqui não há
/// escolha entre irmãos — há um estado ligado ou desligado, e o widget que diz
/// isso é o interruptor, que acende quando armado.
pub const SCULPT3D_FILTER: NodeId = hash_node_id("sculpt3d.filter");

/// **QUAL LEI o filtro roda** — um chip por lei do `FilterKind`.
///
/// ⚠️ **O catálogo NÃO é a projecção dos verbos, e é isso que o justifica.**
/// Quatro das sete leis são verbos que o filtro reusa (Smooth, Inflate, Relax,
/// Surface Smooth) e três **não têm carimbo nenhum** — não existe pincel de
/// Scale, de Sphere nem de Random. Enquanto a lei era derivada do verbo em
/// mãos, essas três eram inalcançáveis por qualquer gesto; a fileira É a porta
/// delas.
///
/// ⚠️ **Grupo e não `toggle`, pelo motivo inverso ao do vizinho de cima:** aqui
/// há irmãos mutuamente exclusivos, que é a definição de um rádio.
///
/// ⚠️ **A CONTAGEM não é citada em prosa** — ela é o comprimento deste array e
/// o do `FilterKind::ALL`, que um gate compara. Um número escrito aqui
/// envelheceria na wave seguinte, como o `Verb::ALL` já pagou duas vezes.
///
/// ⚠️ **UMA convenção, e a ordem desta lista É ela:** o id em `i` nomeia
/// `FilterKind::ALL[i]`, e é assim que o painter, o roteador e o gate a leem.
/// Indexar por DISCRIMINANTE em qualquer um dos três seria uma segunda
/// convenção que coincide com esta só enquanto o `ALL` estiver em ordem de
/// declaração — e o dia em que ele for reordenado, um chip rotulado `Sphere`
/// escreveria `Relax`, pintado, vivo sob o mouse e mentindo.
pub const SCULPT3D_FILTER_KIND: [NodeId; 9] = [
    hash_node_id("sculpt3d.filter.kind.smooth"),
    hash_node_id("sculpt3d.filter.kind.scale"),
    hash_node_id("sculpt3d.filter.kind.inflate"),
    hash_node_id("sculpt3d.filter.kind.sphere"),
    hash_node_id("sculpt3d.filter.kind.random"),
    hash_node_id("sculpt3d.filter.kind.relax"),
    hash_node_id("sculpt3d.filter.kind.surface_smooth"),
    hash_node_id("sculpt3d.filter.kind.enhance_details"),
    hash_node_id("sculpt3d.filter.kind.sharpen"),
];

/// **OS CINCO TIPOS DO FILTRO DE TECIDO** (espec §7) — a segunda fileira do
/// selector do filtro.
///
/// ⚠️ **A mesma convenção da vizinha: o índice é a POSIÇÃO no
/// `ClothFilterKind::ALL`**, nunca o discriminante.
///
/// ⚠️ **Eles são uma fileira PRÓPRIA e não catorze chips numa só**, e a razão é
/// que duas leis se chamam igual dos dois lados — *Inflate* e *Scale* existem nas
/// duas famílias e fazem coisas diferentes. *Um chip cujo rótulo não distingue a
/// lei precisa da fileira para o fazer.*
pub const SCULPT3D_CLOTH_FILTER_KIND: [NodeId; 5] = [
    hash_node_id("sculpt3d.filter.cloth.gravity"),
    hash_node_id("sculpt3d.filter.cloth.inflate"),
    hash_node_id("sculpt3d.filter.cloth.expand"),
    hash_node_id("sculpt3d.filter.cloth.pinch"),
    hash_node_id("sculpt3d.filter.cloth.scale"),
];

/// **O REFERENCIAL do filtro de tecido** (espec §7, *Orientation*).
///
/// ⚠️ **DOIS, e não os três da espec.** O `World` fica de fora por MEDIÇÃO — a
/// `ph2d_mesh::Pose` de uma escultura não tem rotação, logo ele daria os mesmos
/// eixos que o `Local` e seria um chip que o artista descobre vazio clicando.
/// A lista viva é `ClothFilterOrientation::offered()`, e há gate a medi-la.
pub const SCULPT3D_CLOTH_FILTER_ORIENT: [NodeId; 2] = [
    hash_node_id("sculpt3d.filter.cloth.orient.local"),
    hash_node_id("sculpt3d.filter.cloth.orient.view"),
];

/// **COM QUE PROFUNDIDADE OLHAR** — os chips `Basic` · `Pro` (§2 do plano).
///
/// ⚠️ **O nome não é `DETAIL` de propósito:** [`SCULPT3D_DYN_DETAIL`] já existe e
/// é o alvo da topologia dinâmica, e [`SCULPT3D_LEVEL_UP`] é o da multires. Três
/// coisas diferentes disputando a palavra *nível* num painel só é como o próximo
/// leitor abre o array errado.
pub const SCULPT3D_UI_LEVEL: [NodeId; 2] = [
    hash_node_id("sculpt3d.ui_level.0"),
    hash_node_id("sculpt3d.ui_level.1"),
];

/// **QUAL MOTOR DE RETOPOLOGIA** — os chips `Global` · `Local`.
///
/// ⚠️ **O tamanho se CONTA e não se escolhe** — o censo
/// `the_panel_offers_every_retopo_mode_the_engine_has` compara este array com o
/// `RetopoMode::ALL`, então um motor novo que não passe por aqui nasce
/// inalcançável no painel e o gate fica vermelho em vez de o chip sumir em
/// silêncio.
pub const SCULPT3D_RETOPO_MODE: [NodeId; 2] = [
    hash_node_id("sculpt3d.retopo_mode.0"),
    hash_node_id("sculpt3d.retopo_mode.1"),
];

// ── A topologia ─────────────────────────────────────────────────────────────
/// Liga/desliga a topologia dinâmica.
pub const SCULPT3D_DYNTOPO: NodeId = hash_node_id("sculpt3d.dyntopo");

/// **O ALVO DE DENSIDADE do passe de topologia dinâmica** — a pista e o chip.
///
/// ⚠️⚠️ **Eram TRÊS CHIPS com nome** (*grosso · médio · fino*) e passaram a ser
/// uma pista contínua em 2026-09-14, por report do dono: *«porque não temos um
/// slider neste pincel para definir a densidade da malha»*. ⛔ **Os dois não
/// coexistem**, e a razão é a lei desta casa e não gosto: eles escrevem o MESMO
/// número, e duas superfícies sobre um valor só divergem no dia em que uma
/// ganhar clamp e a outra não. A tecla `U` fica — ela cicla os três valores com
/// nome, que é o atalho, exactamente como o `[`/`]` do raio coexiste com a
/// pista dele.
///
/// ⚠️ O doc que morava na tabela de degraus dizia *«três e não um slider
/// contínuo, porque a UI aqui é o teclado»* — **a premissa expirou** quando a
/// secção Topology ganhou os knobs do remesh, e ninguém releu a nota.
pub const SCULPT3D_DYN_DETAIL: NodeId = hash_node_id("sculpt3d.dyn_detail");

/// Ver [`SCULPT3D_DYN_DETAIL`].
pub const SCULPT3D_DYN_DETAIL_NUM: NodeId = hash_node_id("sculpt3d.dyn_detail_num");

/// **O ALVO DE DENSIDADE DO PINCEL** — a pista e o chip, nas propriedades dele.
///
/// ⭐⭐⭐ **ORDEM DO DONO (2026-09-14): *«deixe o slider Detail para o dynamic
/// Retopology e coloque outro slider Detail exclusivo para o pincel, nas
/// propriedades do pincel»*.** São **DOIS** controlos com o mesmo rótulo e
/// assuntos diferentes, e é de propósito: o [`SCULPT3D_DYN_DETAIL`] governa o
/// traço dos outros pincéis (a topologia dinâmica) e este governa o pincel de
/// densidade, que não tem traço nenhum.
///
/// ⚠️ **Eles NÃO são duas superfícies sobre um valor** — a armadilha que os três
/// chips pagaram nesta mesma wave: são **dois campos**, um na cena e outro no
/// pincel, e a porta que escolhe entre eles é a
/// `Brush::offers_density_controls`. *A regra que os separa é a que também
/// decide qual deles a tecla `U` cicla.*
pub const SCULPT3D_DENSITY_DETAIL: NodeId = hash_node_id("sculpt3d.density_detail");

/// Ver [`SCULPT3D_DENSITY_DETAIL`].
pub const SCULPT3D_DENSITY_DETAIL_NUM: NodeId = hash_node_id("sculpt3d.density_detail_num");

/// Desce um nível de multiresolução.
pub const SCULPT3D_LEVEL_DOWN: NodeId = hash_node_id("sculpt3d.level_down");

/// Sobe um nível de multiresolução.
pub const SCULPT3D_LEVEL_UP: NodeId = hash_node_id("sculpt3d.level_up");

/// Subdivide (acrescenta um nível ACIMA).
pub const SCULPT3D_SUBDIVIDE: NodeId = hash_node_id("sculpt3d.subdivide");

/// Reverte (reconstrói um nível ABAIXO).
pub const SCULPT3D_REVERSE: NodeId = hash_node_id("sculpt3d.reverse");

/// **ACHATA a pilha** numa malha só, com todo o detalhe — a saída para os três
/// verbos que recusam com ela montada.
pub const SCULPT3D_FLATTEN: NodeId = hash_node_id("sculpt3d.flatten");

/// Reconstrói a casca (voxel remesh).
pub const SCULPT3D_REMESH: NodeId = hash_node_id("sculpt3d.remesh");

/// Tapa os buracos.
pub const SCULPT3D_CLOSE_HOLES: NodeId = hash_node_id("sculpt3d.close_holes");

/// **RETOPOLOGIA por campo cruzado** (ADR-0160) — a grade corre AO LONGO da
/// forma, ao contrário do voxel remesh, cujos quads seguem os eixos da grade.
pub const SCULPT3D_QUAD_REMESH: NodeId = hash_node_id("sculpt3d.quad_remesh");

// ── A cena ──────────────────────────────────────────────────────────────────
/// As 4 primitivas (esfera, cubo, cilindro, toro).
pub const SCULPT3D_ADD: [NodeId; 4] = [
    hash_node_id("sculpt3d.add.0"),
    hash_node_id("sculpt3d.add.1"),
    hash_node_id("sculpt3d.add.2"),
    hash_node_id("sculpt3d.add.3"),
];

/// Duplica a peça ativa.
pub const SCULPT3D_DUPLICATE: NodeId = hash_node_id("sculpt3d.duplicate");

/// Apaga a peça ativa.
pub const SCULPT3D_DELETE: NodeId = hash_node_id("sculpt3d.delete");

/// Isola a peça ativa (o *local view*).
pub const SCULPT3D_ISOLATE: NodeId = hash_node_id("sculpt3d.isolate");

/// Funde as peças à vista numa só.
pub const SCULPT3D_MERGE: NodeId = hash_node_id("sculpt3d.merge");

// ── A máscara ───────────────────────────────────────────────────────────────
/// As 4 operações de máscara (limpar, inverter, borrar, afiar).
///
/// ⚠️ Elas moram na seção do PINCEL, ao lado do verbo `Mask`, e não numa seção
/// própria: um artista que acabou de pintar máscara procura o que fazer com ela
/// onde ele a pintou.
pub const SCULPT3D_MASK_OP: [NodeId; 4] = [
    hash_node_id("sculpt3d.mask_op.0"),
    hash_node_id("sculpt3d.mask_op.1"),
    hash_node_id("sculpt3d.mask_op.2"),
    hash_node_id("sculpt3d.mask_op.3"),
];

/// **O EXTRACT** — a máscara vira uma PEÇA.
///
/// ⚠️ Ele mora ao lado das quatro operações e **não** na seção da cena, embora
/// o que ele produza seja um objeto: quem acabou de pintar uma máscara procura o
/// que fazer com ela onde a pintou, e é a mesma frase que pôs as outras quatro
/// aqui. A CONSEQUÊNCIA aparece na seção da cena, no número de peças.
pub const SCULPT3D_EXTRACT: NodeId = hash_node_id("sculpt3d.extract");

/// Espessura da casca que o extract produz. Zero é uma folha só.
pub const SCULPT3D_EXTRACT_THICK: NodeId = hash_node_id("sculpt3d.extract_thick");

/// Chip ligado a [`SCULPT3D_EXTRACT_THICK`].
pub const SCULPT3D_EXTRACT_THICK_NUM: NodeId = hash_node_id("sculpt3d.extract_thick_num");

/// A pista da RESOLUÇÃO do remesh.
///
/// ⚠️ A faixa dela é MEDIDA e o recurso é a memória do campo TRANSIENTE — ver
/// [`ph2d_panel_sculpt3d::rows`], onde a tabela mora ao lado do número.
pub const SCULPT3D_REMESH_RES: NodeId = hash_node_id("sculpt3d.remesh_res");

/// O chip numérico da resolução do remesh.
pub const SCULPT3D_REMESH_RES_NUM: NodeId = hash_node_id("sculpt3d.remesh_res_num");

/// O lado do quad que a retopologia persegue, em unidades de objeto.
pub const SCULPT3D_QUAD_DETAIL: NodeId = hash_node_id("sculpt3d.quad_detail");

/// A pista do lado do quad.
pub const SCULPT3D_QUAD_DETAIL_NUM: NodeId = hash_node_id("sculpt3d.quad_detail_num");

/// Quanto a densidade segue a curvatura — `0` uniforme, `1` a faixa inteira.
pub const SCULPT3D_QUAD_ADAPT: NodeId = hash_node_id("sculpt3d.quad_adapt");

/// A pista da adaptação.
pub const SCULPT3D_QUAD_ADAPT_NUM: NodeId = hash_node_id("sculpt3d.quad_adapt_num");

/// Quantas passadas de relaxamento a costura do extract recebe.
pub const SCULPT3D_EXTRACT_SMOOTH: NodeId = hash_node_id("sculpt3d.extract_smooth");

/// Chip ligado a [`SCULPT3D_EXTRACT_SMOOTH`].
pub const SCULPT3D_EXTRACT_SMOOTH_NUM: NodeId = hash_node_id("sculpt3d.extract_smooth_num");

/// **O TRANSFORM** — mover, girar e escalar a parte LIVRE.
///
/// ⚠️ **Rádio com DESLIGADO, e não três comandos:** as quatro operações de
/// máscara ali em cima executam e acabam (nenhuma fica acesa); estes três
/// **ARMAM** o botão esquerdo, então um deles fica aceso enquanto vale — e
/// clicar o aceso desarma. É a diferença entre *um gesto* e *uma ferramenta na
/// mão*, e ela decide o que o `selected` do grupo mostra.
///
/// ⚠️ E eles moram aqui, ao lado do extract, pela frase que já pôs as outras
/// cinco nesta vizinhança: quem acabou de pintar uma máscara procura o que fazer
/// com ela onde a pintou.
pub const SCULPT3D_TRANSFORM: [NodeId; 3] = [
    hash_node_id("sculpt3d.transform.0"),
    hash_node_id("sculpt3d.transform.1"),
    hash_node_id("sculpt3d.transform.2"),
];
