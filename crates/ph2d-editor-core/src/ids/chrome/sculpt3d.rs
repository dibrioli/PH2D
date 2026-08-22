//! **O painel da cena 3D** (`SCULPT3D_*`) — ADR-0150, W12.
//!
//! A família de ids do `ph2d-panel-sculpt3d`: a lista de ferramentas, os knobs
//! do pincel, o espelho, a topologia, o sombreamento e a lista de peças.
//!
//! Slug pontilhado (`sculpt3d.*`), como a família do painel de física. Hash de
//! string, então nenhum contador de id se move — o `node_id_collisions` varre
//! estas chaves como varre as outras.

use super::{NodeId, hash_node_id};

/// Retângulo externo do painel (para o `z_order` + a barreira de hit).
pub const SCULPT3D_PANEL: NodeId = hash_node_id("sculpt3d.panel");
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
pub const SCULPT3D_VERB: [NodeId; 23] = [
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

/// **COM QUE PROFUNDIDADE OLHAR** — os chips `Basic` · `Pro` (§2 do plano).
///
/// ⚠️ **O nome não é `DETAIL` de propósito:** [`SCULPT3D_DETAIL`] já existe e é
/// o degrau da topologia dinâmica, e [`SCULPT3D_LEVEL_UP`] é o da multires. Três
/// coisas diferentes disputando a palavra *nível* num painel só é como o próximo
/// leitor abre o array errado.
pub const SCULPT3D_UI_LEVEL: [NodeId; 2] = [
    hash_node_id("sculpt3d.ui_level.0"),
    hash_node_id("sculpt3d.ui_level.1"),
];

/// **QUAL MOTOR DE RETOPOLOGIA** — os chips `Global` · `Local`.
///
/// ⚠️ **O tamanho se CONTA e não se escolhe** — o seam
/// `the_panel_offers_every_retopo_mode_the_engine_has` compara este array com o
/// `RetopoMode::ALL`, então um motor novo que não passe por aqui nasce
/// inalcançável no painel e o gate fica vermelho em vez de o chip sumir em
/// silêncio.
pub const SCULPT3D_RETOPO_MODE: [NodeId; 2] = [
    hash_node_id("sculpt3d.retopo_mode.0"),
    hash_node_id("sculpt3d.retopo_mode.1"),
];

// ── O pincel ────────────────────────────────────────────────────────────────
/// As curvas de `ph2d_sculpt3d::Falloff::ALL`.
///
/// ⚠️ **O tamanho se CONTA, não se escolhe** — o seam
/// `the_panel_offers_every_falloff_the_engine_has` compara este array com o
/// `ALL` do motor, então uma curva nova que não passe por aqui nasce
/// inalcançável no painel e o gate fica vermelho em vez de o botão sumir em
/// silêncio.
pub const SCULPT3D_FALLOFF: [NodeId; 12] = [
    hash_node_id("sculpt3d.falloff.0"),
    hash_node_id("sculpt3d.falloff.1"),
    hash_node_id("sculpt3d.falloff.2"),
    hash_node_id("sculpt3d.falloff.3"),
    hash_node_id("sculpt3d.falloff.4"),
    hash_node_id("sculpt3d.falloff.5"),
    hash_node_id("sculpt3d.falloff.6"),
    hash_node_id("sculpt3d.falloff.7"),
    hash_node_id("sculpt3d.falloff.8"),
    hash_node_id("sculpt3d.falloff.9"),
    hash_node_id("sculpt3d.falloff.10"),
    hash_node_id("sculpt3d.falloff.11"),
];
/// Raio do pincel, em **pixels de tela**.
pub const SCULPT3D_RADIUS: NodeId = hash_node_id("sculpt3d.radius");
/// Chip ligado a [`SCULPT3D_RADIUS`].
pub const SCULPT3D_RADIUS_NUM: NodeId = hash_node_id("sculpt3d.radius_num");
/// Força do dab, em `[0, 1]`.
pub const SCULPT3D_STRENGTH: NodeId = hash_node_id("sculpt3d.strength");
/// Chip ligado a [`SCULPT3D_STRENGTH`].
pub const SCULPT3D_STRENGTH_NUM: NodeId = hash_node_id("sculpt3d.strength_num");
/// Deslocamento do plano, em fração do raio (só os verbos de plano o leem).
pub const SCULPT3D_PLANE_OFFSET: NodeId = hash_node_id("sculpt3d.plane_offset");
/// Chip ligado a [`SCULPT3D_PLANE_OFFSET`].
pub const SCULPT3D_PLANE_OFFSET_NUM: NodeId = hash_node_id("sculpt3d.plane_offset_num");
/// Quanto o Crease aperta lateralmente.
pub const SCULPT3D_PINCH: NodeId = hash_node_id("sculpt3d.pinch");
/// Chip ligado a [`SCULPT3D_PINCH`].
pub const SCULPT3D_PINCH_NUM: NodeId = hash_node_id("sculpt3d.pinch_num");
/// **α do Surface Smooth** — quanto o `b` se ancora na pose do PEN-DOWN em vez
/// da posição de agora (`surface_smooth_shape_preservation`).
pub const SCULPT3D_HC_SHAPE: NodeId = hash_node_id("sculpt3d.hc_shape");
/// Chip ligado a [`SCULPT3D_HC_SHAPE`].
pub const SCULPT3D_HC_SHAPE_NUM: NodeId = hash_node_id("sculpt3d.hc_shape_num");
/// **β do Surface Smooth** — que fração da correção vem do `b` do PRÓPRIO
/// vértice em vez da média dos vizinhos (`surface_smooth_current_vertex`).
///
/// ⚠️ **A faixa deste knob começa em `0,5` e o piso NÃO é dele** — ver
/// `ph2d_sculpt3d::HC_VERTEX_MIN`: abaixo disso o operador AMPLIFICA (medido,
/// `β = 0,3` leva a rugosidade a 43,8× a da base em dezasseis dabs), e quem
/// impede a malha de rebentar é o clamp no MOTOR. O `min` da row existe para o
/// artista não alcançar o disfuncional com o dedo; o clamp existe para um
/// documento que traga o valor errado ser corrigido em vez de explodir.
pub const SCULPT3D_HC_VERTEX: NodeId = hash_node_id("sculpt3d.hc_vertex");
/// Chip ligado a [`SCULPT3D_HC_VERTEX`].
pub const SCULPT3D_HC_VERTEX_NUM: NodeId = hash_node_id("sculpt3d.hc_vertex_num");
/// **A DUREZA DO DAB** — o platô de peso cheio no miolo da pegada.
///
/// ⚠️ Ele NÃO é o [`SCULPT3D_MASK_HARDNESS`], embora os nomes se pareçam: aquele
/// é o expoente da curva PRÓPRIA do canal de máscara, este remapeia a DISTÂNCIA
/// que qualquer falloff consome (`apply_hardness_to_distances` do Blender). Dois
/// controles, duas perguntas, e o gate de costura pinta os dois para o mesmo
/// verbo nunca oferecer um pelo outro.
pub const SCULPT3D_HARDNESS: NodeId = hash_node_id("sculpt3d.hardness");
/// Chip ligado a [`SCULPT3D_HARDNESS`].
pub const SCULPT3D_HARDNESS_NUM: NodeId = hash_node_id("sculpt3d.hardness_num");
/// **O alisamento que corre depois de cada dab** — o `autosmooth_factor` do
/// Blender (`sculpt.cc:3636`), e o VIZINHO do [`SCULPT3D_HARDNESS`] no RNA dele
/// (`rna_brush.cc:3450` contra `:3457`).
///
/// ⚠️ A adjacência não é acaso e a fileira a honra: são os dois knobs que trocam
/// **borda dura** por **superfície que a malha consegue carregar**, e lê-los
/// juntos é o que faz o segundo ser aprendido quando o primeiro morde.
pub const SCULPT3D_AUTO_SMOOTH: NodeId = hash_node_id("sculpt3d.auto_smooth");
/// Chip ligado a [`SCULPT3D_AUTO_SMOOTH`].
pub const SCULPT3D_AUTO_SMOOTH_NUM: NodeId = hash_node_id("sculpt3d.auto_smooth_num");
/// A dureza da borda do canal de MÁSCARA — o `_hardness` da tool `Masking` do
/// SculptGL. ⚠️ Ele NÃO é um falloff: o canal tem curva própria
/// (`(1 − d)^{2(1 − hardness)}`), e o seletor de [`Falloff`] governa a
/// geometria.
pub const SCULPT3D_MASK_HARDNESS: NodeId = hash_node_id("sculpt3d.mask_hardness");
/// Chip ligado a [`SCULPT3D_MASK_HARDNESS`].
pub const SCULPT3D_MASK_HARDNESS_NUM: NodeId = hash_node_id("sculpt3d.mask_hardness_num");

/// **O PADRÃO que decide onde, dentro da pegada, o verbo age** — a primeira
/// opção é NENHUM e as outras são os padrões de `ph2d_sculpt3d::Alpha::ALL`.
///
/// ⚠️ O tamanho é `Alpha::ALL.len() + 2`, e os DOIS a mais não são padrões: o
/// primeiro é o pincel LISO e o último é o slot de IMAGEM, que carrega o nome do
/// sprite em vez de um nome de fórmula. É a mesma aritmética do
/// [`SCULPT3D_MATCAP`] com um degrau a mais, e pelo mesmo motivo: um chip
/// sobrando pinta uma opção que o motor não tem, um faltando deixa um padrão
/// inalcançável. Gateado.
///
/// ⚠️ **O chip da imagem é o ÚLTIMO, e a posição é load-bearing:** os índices
/// `1..=9` são um deslocamento sobre `Alpha::ALL`, então pôr a imagem no meio
/// re-numeraria os nove e todo clique passaria a armar o padrão vizinho.
pub const SCULPT3D_ALPHA: [NodeId; 11] = [
    hash_node_id("sculpt3d.alpha.none"),
    hash_node_id("sculpt3d.alpha.0"),
    hash_node_id("sculpt3d.alpha.1"),
    hash_node_id("sculpt3d.alpha.2"),
    hash_node_id("sculpt3d.alpha.3"),
    hash_node_id("sculpt3d.alpha.4"),
    hash_node_id("sculpt3d.alpha.5"),
    hash_node_id("sculpt3d.alpha.6"),
    hash_node_id("sculpt3d.alpha.7"),
    hash_node_id("sculpt3d.alpha.8"),
    hash_node_id("sculpt3d.alpha.image"),
];
/// **O TAMANHO DO CARIMBO, em fração da ALTURA DA TELA** — ver
/// `ph2d_sculpt3d::Brush::alpha_stencil_scale`.
///
/// ⚠️ **Id PRÓPRIO, e não o do `Pattern Size`.** Uma imagem é um estêncil preso
/// ao viewport e é medida na TELA; os nove procedurais são campos 3-D e são
/// medidos no MODELO. O mesmo widget com duas réguas trocaria de significado em
/// silêncio ao trocar de padrão — e o artista não teria como saber qual das duas
/// está segurando.
pub const SCULPT3D_STAMP_SCALE: NodeId = hash_node_id("sculpt3d.stamp_scale");
/// Chip ligado a [`SCULPT3D_STAMP_SCALE`].
pub const SCULPT3D_STAMP_SCALE_NUM: NodeId = hash_node_id("sculpt3d.stamp_scale_num");
/// **ONDE o carimbo POUSA**, ao longo da tangente do frame e em fração da
/// ALTURA DA TELA — ver `ph2d_sculpt3d::Brush::alpha_offset`.
///
/// ⚠️ **Dois ids e não um par XY num controle só**, porque as duas pistas deste
/// painel são de UM número: um widget de dois eixos seria o primeiro do painel e
/// pediria hit-test, arrasto e chip próprios — trabalho que não compra nada que
/// duas pistas irmãs não deem.
pub const SCULPT3D_ALPHA_OFF_X: NodeId = hash_node_id("sculpt3d.alpha_off_x");
/// Chip ligado a [`SCULPT3D_ALPHA_OFF_X`].
pub const SCULPT3D_ALPHA_OFF_X_NUM: NodeId = hash_node_id("sculpt3d.alpha_off_x_num");
/// A outra metade da colocação — ver [`SCULPT3D_ALPHA_OFF_X`].
pub const SCULPT3D_ALPHA_OFF_Y: NodeId = hash_node_id("sculpt3d.alpha_off_y");
/// Chip ligado a [`SCULPT3D_ALPHA_OFF_Y`].
pub const SCULPT3D_ALPHA_OFF_Y_NUM: NodeId = hash_node_id("sculpt3d.alpha_off_y_num");

/// Tamanho de uma feature do alpha, em unidades de objeto.
pub const SCULPT3D_ALPHA_SCALE: NodeId = hash_node_id("sculpt3d.alpha_scale");
/// Chip ligado a [`SCULPT3D_ALPHA_SCALE`].
pub const SCULPT3D_ALPHA_SCALE_NUM: NodeId = hash_node_id("sculpt3d.alpha_scale_num");

/// **O AZIMUTE do eixo de um padrão DIRECIONAL.**
///
/// ⚠️ **Não é a lâmpada, e a distinção importa mais do que parece:** os dois
/// pares de pistas falam a mesma língua (azimute + elevação em graus, o rotor do
/// app) e descrevem coisas diferentes — um aponta a LUZ, o outro aponta o
/// PADRÃO. Ids separados são o que impede um clique de virar o outro.
pub const SCULPT3D_ALPHA_AZ: NodeId = hash_node_id("sculpt3d.alpha_az");
/// Chip ligado a [`SCULPT3D_ALPHA_AZ`].
pub const SCULPT3D_ALPHA_AZ_NUM: NodeId = hash_node_id("sculpt3d.alpha_az_num");
/// A ELEVAÇÃO do eixo — ver [`SCULPT3D_ALPHA_AZ`].
pub const SCULPT3D_ALPHA_ELEV: NodeId = hash_node_id("sculpt3d.alpha_elev");
/// Chip ligado a [`SCULPT3D_ALPHA_ELEV`].
pub const SCULPT3D_ALPHA_ELEV_NUM: NodeId = hash_node_id("sculpt3d.alpha_elev_num");
/// **O preview do padrão NO BARRO** — o interruptor do tinto que mostra, na
/// peça, o que o próximo traço vai depositar.
pub const SCULPT3D_ALPHA_PREVIEW: NodeId = hash_node_id("sculpt3d.alpha_preview");

// ── O espelho ───────────────────────────────────────────────────────────────
// TRÊS botões e não um rádio: os eixos são independentes (o ZBrush espelha em
// dois ao mesmo tempo), e um segmented é *um de N* por construção.
/// Espelho em X.
pub const SCULPT3D_SYM_X: NodeId = hash_node_id("sculpt3d.sym.x");
/// Espelho em Y.
pub const SCULPT3D_SYM_Y: NodeId = hash_node_id("sculpt3d.sym.y");
/// Espelho em Z.
pub const SCULPT3D_SYM_Z: NodeId = hash_node_id("sculpt3d.sym.z");

// ── A topologia ─────────────────────────────────────────────────────────────
/// Liga/desliga a topologia dinâmica.
pub const SCULPT3D_DYNTOPO: NodeId = hash_node_id("sculpt3d.dyntopo");
/// Os 3 degraus de detalhe (grosso/médio/fino).
pub const SCULPT3D_DETAIL: [NodeId; 3] = [
    hash_node_id("sculpt3d.detail.0"),
    hash_node_id("sculpt3d.detail.1"),
    hash_node_id("sculpt3d.detail.2"),
];
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

// ── O sombreamento ──────────────────────────────────────────────────────────
/// **A CAVIDADE** — quanto a curvatura escurece a fresta e clareia a crista.
pub const SCULPT3D_CAVITY: NodeId = hash_node_id("sculpt3d.cavity");
/// Chip ligado a [`SCULPT3D_CAVITY`].
pub const SCULPT3D_CAVITY_NUM: NodeId = hash_node_id("sculpt3d.cavity_num");

/// **QUANTO DO AMBIENTE COM DIREÇÃO ENTRA** — o piso da difusa dizendo de onde
/// a luz de preenchimento vem.
///
/// ⚠️ **Ele NÃO é uma segunda luz**, e é por isso que o rótulo diz *ambiente* e
/// não *intensidade*: o número é o MESMO `ph2d_light::AMBIENT` de sempre,
/// redistribuído — céu em cima, ricochete do chão embaixo —, com a média sobre
/// todas as normais preservada. Subi-lo não clareia a peça; ele tira luz de baixo
/// e põe em cima.
pub const SCULPT3D_ENV: NodeId = hash_node_id("sculpt3d.env");
/// Chip ligado a [`SCULPT3D_ENV`].
pub const SCULPT3D_ENV_NUM: NodeId = hash_node_id("sculpt3d.env_num");

/// **QUANTO DO AO ASSADO ENTRA** — irmão da cavidade no painel, e o oposto dela
/// na origem: a cavidade é derivada e existe sempre, o AO só existe depois de um
/// bake explícito.
pub const SCULPT3D_AO: NodeId = hash_node_id("sculpt3d.ao");
/// Chip ligado a [`SCULPT3D_AO`].
pub const SCULPT3D_AO_NUM: NodeId = hash_node_id("sculpt3d.ao_num");
/// **QUANTO DO AO DE TELA ENTRA** — o irmão MEDIDO do de cima.
///
/// ⚠️ Dois knobs e não um, e a diferença não é gosto: o assado é exato, viaja no
/// arquivo e ENVELHECE a cada pincelada; este é medido todo frame, nunca fica
/// velho e só vê o que está na tela. Colapsá-los num knob só obrigaria o artista
/// a escolher entre a oclusão que ele VÊ enquanto trabalha e a que ele EXPORTA.
pub const SCULPT3D_SSAO: NodeId = hash_node_id("sculpt3d.ssao");
/// Chip ligado a [`SCULPT3D_SSAO`].
pub const SCULPT3D_SSAO_NUM: NodeId = hash_node_id("sculpt3d.ssao_num");
/// **Quanto do espalhamento sub-superficial entra** (`ph2d_mesh_render::sss`).
pub const SCULPT3D_SSS: NodeId = hash_node_id("sculpt3d.sss");
/// Chip ligado a [`SCULPT3D_SSS`].
pub const SCULPT3D_SSS_NUM: NodeId = hash_node_id("sculpt3d.sss_num");
/// **Até onde a luz viaja dentro do material**, como FRAÇÃO do maior lado da
/// peça — nunca um comprimento absoluto (ver a row).
pub const SCULPT3D_SSS_SCATTER: NodeId = hash_node_id("sculpt3d.sss_scatter");
/// Chip ligado a [`SCULPT3D_SSS_SCATTER`].
pub const SCULPT3D_SSS_SCATTER_NUM: NodeId = hash_node_id("sculpt3d.sss_scatter_num");

/// **ASSAR O AO** — o botão que mede quanto do céu cada vértice enxerga.
///
/// ⚠️ É um BOTÃO e não um passe automático porque o bake não cabe num pen-up:
/// ~338 ms na malha que a cena `=16` abre (`ph2d-sdf/tests/measure_ao.rs`).
pub const SCULPT3D_BAKE_AO: NodeId = hash_node_id("sculpt3d.bake_ao");

/// **ASSAR A FORMA NO SPRITE** — o objetivo 2 do módulo (`docs/3D/02.2`).
///
/// ⚠️ **Não confundir com o [`SCULPT3D_BAKE_AO`] acima:** aquele mede um canal e
/// o escreve **na MALHA**, este escreve o G-buffer inteiro **num objeto da cena
/// 2D**, que passa a acender pela forma e a sobreviver à escultura. Os dois
/// carregam a palavra *bake* e são gestos diferentes — é por isso que moram em
/// seções diferentes e os rótulos dizem o ALVO, nunca só o verbo.
///
/// ⚠️ **Ele existe porque o gesto tinha uma porta só, e ela era um atalho**
/// (`Shift+B`). Um verbo cuja única forma de ser pedido é uma combinação de
/// teclas que nada na tela menciona é um verbo que só quem o escreveu alcança.
pub const SCULPT3D_BAKE_SPRITE: NodeId = hash_node_id("sculpt3d.bake_sprite");

/// **USAR O SPRITE SELECIONADO COMO PADRÃO** — o alpha por IMAGEM.
///
/// ⚠️ **Um BOTÃO e não um chip, e a diferença não é de gosto:** a fileira de
/// chips lista NOMES (os nove padrões que são fórmulas), e uma imagem não é um
/// nome — é uma coisa para a qual se aponta. Um chip *"Image"* teria de existir
/// antes de haver pixels, e é exatamente esse estado que o
/// [`ph2d_sculpt3d::Alpha::Image`] torna inexprimível ao carregar a imagem
/// dentro de si.
///
/// ⚠️ **Ele é o irmão do *"Use as Brush Shape"* do Painter 2D**, e o gesto é o
/// mesmo: o artista seleciona um sprite no canvas e aperta. Sem sprite
/// selecionado o botão **não é pintado** — um botão que só pode falhar é a
/// forma de o artista aprender que ele não funciona.
pub const SCULPT3D_ALPHA_SPRITE: NodeId = hash_node_id("sculpt3d.alpha_sprite");

/// **COM QUE LUZ o barro é mostrado** — a primeira opção é o RIG DO ARTISTA e as
/// outras são os matcaps de [`ph2d_mesh_render::MATCAPS`].
///
/// ⚠️ O tamanho é `MATCAPS.len() + 1`, e o `+ 1` é o rig — que **não** é um
/// matcap. A igualdade das duas contagens é gateada: um chip a mais pinta uma
/// opção que o shader não tem, um a menos deixa um material inalcançável.
pub const SCULPT3D_MATCAP: [NodeId; 11] = [
    hash_node_id("sculpt3d.matcap.rig"),
    hash_node_id("sculpt3d.matcap.0"),
    hash_node_id("sculpt3d.matcap.1"),
    hash_node_id("sculpt3d.matcap.2"),
    hash_node_id("sculpt3d.matcap.3"),
    hash_node_id("sculpt3d.matcap.4"),
    hash_node_id("sculpt3d.matcap.5"),
    hash_node_id("sculpt3d.matcap.6"),
    hash_node_id("sculpt3d.matcap.7"),
    hash_node_id("sculpt3d.matcap.8"),
    hash_node_id("sculpt3d.matcap.9"),
];

/// **ACUMULAR na mesma pincelada** — o `BRUSH_ACCUMULATE` do Blender.
pub const SCULPT3D_ACCUMULATE: NodeId = hash_node_id("sculpt3d.accumulate");

/// **SÓ AS FACES DA FRENTE** — o `BRUSH_FRONTFACE` do Blender
/// (`use_frontface`, `properties_paint_common.py:1354`).
pub const SCULPT3D_FRONT_FACES: NodeId = hash_node_id("sculpt3d.front_faces");

/// A malha de arestas desenhada por cima da forma.
pub const SCULPT3D_WIREFRAME: NodeId = hash_node_id("sculpt3d.wireframe");
/// Azimute da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_AZ: NodeId = hash_node_id("sculpt3d.light_az");
/// Chip ligado a [`SCULPT3D_LIGHT_AZ`].
pub const SCULPT3D_LIGHT_AZ_NUM: NodeId = hash_node_id("sculpt3d.light_az_num");
/// Elevação da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_ELEV: NodeId = hash_node_id("sculpt3d.light_elev");
/// Chip ligado a [`SCULPT3D_LIGHT_ELEV`].
pub const SCULPT3D_LIGHT_ELEV_NUM: NodeId = hash_node_id("sculpt3d.light_elev_num");

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
