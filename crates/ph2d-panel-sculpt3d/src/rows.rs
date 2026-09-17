//! **A tabela de knobs — a lista que todo o resto do painel percorre.**
//!
//! Um controle contínuo aparece em quatro lugares: ele é pintado, é registrado
//! (senão o clique é descartado em silêncio), vira um valor quando o artista o
//! arrasta, e é varrido pelo teste de costura. Quatro listas escritas à mão
//! derivam, e a deriva é MUDA — uma row pintada e não registrada está morta sob
//! o mouse.
//!
//! Então há UMA lista. `paint`, `populate`, `event` e o `tests/seam.rs` percorrem
//! [`ROWS`]; acrescentar um knob é acrescentar uma linha, e ele nasce pintado,
//! registrado, vivo e varrido.

use ph2d_sculpt3d::Verb;

use crate::state::UiLevel;

/// O teto do raio que o SLIDER oferece, em pixels de tela — e a pista é CÚBICA ([`Row::curva`]).
///
/// ⚠️ **Ele não é o teto do produto:** quem aperta é a DIAGONAL DA VISTA (`radius_ceiling_px`, na
/// família: 2 203 px a 1920×1080, 2 779 px a 2560×1080), e este tem de a alcançar — `5000` passa a
/// diagonal de um ecrã 4K (4 406 px) e o digitável do alvo (5 000 px de raio). Numa janela menor o valor
/// **volta** ao encostar no teto real, porque o retrato que o painel pinta é o número JÁ CLAMPADO
/// que o dab usa. Uma pista que anda além do teto é honesta; uma que mostra um número que o pincel
/// não usa não é. ⛔ Era `200`, com o teto em `1/8` da altura, e o dono não chegava à peça (16/09).
const RADIUS_TRACK_MAX_PX: f32 = 5000.0; // LITERAL-PX-OK: extensao da PISTA, nao metrica de design (o teto real e a diagonal da vista)

#[path = "rows_types.rs"]
mod types;

/// Os knobs da LEITURA da forma — ver o doc do módulo.
#[path = "rows_shading.rs"]
mod shading;

/// Os knobs da RESOLUÇÃO da malha — ver o doc do módulo.
#[path = "rows_topology.rs"]
mod topology;
pub use topology::TOPOLOGY;

/// Os cinco números do pincel de TECIDO — ver o doc do módulo.
#[path = "rows_cloth.rs"]
mod cloth;
/// Os cinco números do PINCEL DE PLANO — ver [`plano`].
/// **OS DOIS KNOBS DO HC** — ver [`hc`]. Irmão pelo mesmo corte de ASSUNTO dos
/// vizinhos, e ele foi FORÇADO pelo teto de LOC deste ficheiro quando o pincel
/// afiado entrou: a tabela cresce uma linha por pincel e a prosa de cada knob
/// cresce um parágrafo, logo o que sai são os GRUPOS que já se lêem sozinhos.
#[path = "rows_hc.rs"]
mod hc;

#[path = "rows_plano.rs"]
mod plano;
/// Os três números do pincel de POSE — ver [`pose`].
#[path = "rows_pose.rs"]
mod pose;

/// O número do pincel de CONTORNO — ver [`boundary`].
#[path = "rows_boundary.rs"]
mod boundary;

/// ⭐ Os quatro números do FILTRO de tecido, e a *Quality* do pincel — ver o doc
/// do módulo. ⚠️ **Irmão do [`cloth`] por SUJEITO**: os números do filtro não são
/// os do pincel, e a pergunta de visibilidade deles é outra.
#[path = "rows_cloth_filter.rs"]
mod cloth_filter;

/// Os dois números do EXTRACT — ver o doc do módulo.
#[path = "rows_extract.rs"]
mod extract;

/// As perguntas que o PADRÃO faz — ver o doc do módulo.
#[path = "rows_alpha.rs"]
mod alpha;

/// **QUAIS SEÇÕES existem e que CABEÇALHOS elas têm** — ver o doc do módulo.
///
/// ⚠️ Nenhum caminho de chamador muda: `rows::SECTIONS`, `rows::rows()`,
/// `rows::row_for()` e `rows::section_headers()` continuam onde estavam.
#[path = "rows_sections.rs"]
mod sections;
pub use sections::{BUTTON_SECTIONS, SECTIONS, row_for, rows, section_headers};

pub use types::{Place, Row, Section};

/// O teto da pista de **Extract Smooth**, em passadas.
///
/// ⚠️ **OITO, e o número é MEDIDO** (`ph2d-mesh/tests/it/measure_extract.rs`): o
/// relaxamento da costura **CONVERGE**, e o que ele compra por passada cai
/// rápido. Numa costura serrilhada — a que uma mão pintada deixa — a rugosidade
/// da beira vai de **0,09369 a 0,05117 em oito passadas (−45%)**, e da oitava em
/// diante cada uma compra **0,4%**. Uma pista mais longa seria uma faixa onde
/// arrastar não faz nada, que é o controle morto que esta casa varre a cada wave.
const MAX_EXTRACT_SMOOTH: f32 = 8.0; // LITERAL-PX-OK: contagem de passadas MEDIDA, nao metrica de design

/// **AS PERGUNTAS QUE UMA FILEIRA FAZ** — ver [`show`]; o corte foi forçado
/// pelo tecto de LOC e o cabeçalho dele diz porquê.
#[path = "rows_show.rs"]
mod show;
pub(super) use show::{always, shapes_the_distance, suaviza_o_traco, tem_raio};

/// O pincel: o que se ajusta antes de encostar no barro.
static BRUSH: &[Row] = &[
    Row {
        label: "panel.sculpt3d.radius",
        slider: crate::ids::SCULPT3D_RADIUS,
        chip: crate::ids::SCULPT3D_RADIUS_NUM,
        min: 1.0,
        max: RADIUS_TRACK_MAX_PX,
        step: 1.0,
        decimals: 0,
        get: |u| u.radius_px,
        set: |u, v| u.radius_px = v,
        // ⛔ **O Box Trim não tem raio** — o que delimita o efeito dele é a
        // FORMA que a mão desenha. Quem o achou foi o censo dos knobs mortos.
        show: tem_raio,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // ⭐⭐⭐ **O `Detail` DO PINCEL DE DENSIDADE, logo abaixo do raio** — ordem do
    // dono (14/09): *«deixe o slider Detail para o dynamic Retopology e coloque
    // outro slider Detail exclusivo para o pincel, nas propriedades do
    // pincel»*.
    //
    // ⚠️ **Ele fica COLADO ao raio de propósito:** para este pincel os dois são
    // a ferramenta inteira — *o raio diz ONDE, este diz QUÃO FINO* —, e separá-los
    // por um knob que ele não lê faria o artista procurar o segundo.
    //
    // ⚠️⚠️ **Há DOIS controlos com o rótulo `Detail` neste painel, e é de
    // propósito:** o da secção *Topology* governa a **topologia dinâmica** (o
    // traço dos outros pincéis) e este governa este pincel, que não tem traço. ⛔
    // Eles **não** são duas superfícies sobre um valor — a armadilha que os três
    // chips pagaram nesta mesma wave: são **dois campos**, um na cena e outro no
    // `Brush`, e quem escolhe entre eles é a
    // [`ph2d_sculpt3d::Brush::offers_density_controls`], a mesma porta que este
    // `show` consulta. *Um slider visível a governar outra coisa é o que essa
    // porta única existe para impedir.*
    //
    // ⚠️ A faixa e a unidade são as MESMAS do irmão (uma contagem de triângulos
    // ancorada na ÁREA, logo independente do zoom e do tamanho da peça): *dois
    // sliders, uma régua*.
    Row {
        label: "panel.sculpt3d.density_detail",
        slider: crate::ids::SCULPT3D_DENSITY_DETAIL,
        chip: crate::ids::SCULPT3D_DENSITY_DETAIL_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
        decimals: 2,
        get: |u| u.brush.density_detail,
        set: |u, v| u.brush.density_detail = v,
        show: |u| u.brush.offers_density_controls(),
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // ⭐⭐ **A SUAVIZAÇÃO DO TRAÇO DO LAÇO** — ordem do dono (2026-09-15): *«Em
    // laço um parâmetro para suavizar o traço»*.
    //
    // ⚠️ **`0` é o traço CRU, byte-idêntico** — a lei ([`ph2d_trim::suaviza`])
    // empresta o anel em vez de o copiar, e há gate. ⛔ E ela só é pintada com
    // o LAÇO na mão: a caixa e o círculo saem de dois pontos, e uma pista que
    // suavizasse dois pontos era o controlo morto que esta casa varre a cada
    // wave.
    Row {
        label: "panel.sculpt3d.trim_smooth",
        slider: crate::ids::SCULPT3D_TRIM_SMOOTH,
        chip: crate::ids::SCULPT3D_TRIM_SMOOTH_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
        decimals: 2,
        get: |u| u.brush.trim_suavizacao,
        set: |u, v| u.brush.trim_suavizacao = v,
        show: suaviza_o_traco,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    plano::PLANO_ALTURA,
    plano::PLANO_PROFUNDIDADE,
    plano::PLANO_FIRMEZA_NORMAL,
    plano::PLANO_FIRMEZA_CENTRO,
    plano::PLANO_AREA,
    // ⭐⭐ **A FOLGA DA PROJECÇÃO** (espec §6.3.4). ⛔ O rótulo diz «vão» e não
    // «distância mínima», e a escolha é MEDIDA: ela só é mínima no sentido de
    // AVANÇO — o porquê e as duas medições vivem no campo que ela escreve
    // ([`ph2d_sculpt3d::Brush::project_min_distance`]). ⚠️ A faixa é em unidades
    // do OBJECTO, e o tecto `1,0` é o lado da peça de omissão.
    Row {
        label: "panel.sculpt3d.project_min_dist",
        slider: crate::ids::SCULPT3D_PROJECT_MIN_DIST,
        chip: crate::ids::SCULPT3D_PROJECT_MIN_DIST_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de uma distancia de objecto, nao metrica de layout
        decimals: 2,
        get: |u| u.brush.project_min_distance,
        set: |u, v| u.brush.project_min_distance = v,
        show: |u| u.brush.offers_project_controls(),
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.strength",
        slider: crate::ids::SCULPT3D_STRENGTH,
        chip: crate::ids::SCULPT3D_STRENGTH_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.strength,
        set: |u, v| u.brush.strength = v,
        // ⭐⭐ **E ela deixou de ser `always` em 2026-09-15** — ver
        // [`ph2d_sculpt3d::Verb::a_forca_chega_ao_barro`], onde a medição está:
        // o censo dos knobs lê o `Density` a arrastar a força de `0,1` a `1,0`
        // com desvio `0,000e0` no barro.
        show: |u| u.brush.verb.a_forca_chega_ao_barro(),
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // **A DUREZA** — logo abaixo da força, que é onde o Blender a põe, e não por
    // costume: as duas moldam o MESMO peso em eixos ortogonais (a força diz
    // *quanto*, a dureza diz *até onde o cheio vai antes de a curva começar*), e
    // lê-las juntas é o que faz a segunda ser aprendida.
    //
    // ⚠️ **Ela NÃO é a `mask_hardness` logo abaixo**, embora os nomes se
    // pareçam: aquela é o expoente da curva PRÓPRIA do canal de máscara
    // (`Masking.js:66`), esta remapeia a DISTÂNCIA que qualquer falloff consome
    // (no Blender, um remapeamento de distância aplicado antes de toda queda). Um verbo pode oferecer
    // as duas ao mesmo tempo, e é por isso que elas não podem compartilhar um
    // controle.
    Row {
        label: "panel.sculpt3d.hardness",
        slider: crate::ids::SCULPT3D_HARDNESS,
        chip: crate::ids::SCULPT3D_HARDNESS_NUM,
        min: 0.0,
        // ⚠️ **UM é o disco duro, e ele é alcançável de propósito** — o
        // `shaped_distance` tem braço próprio para ele justamente porque a
        // fórmula geral divide por `1 − h`. O teto do Blender é o mesmo.
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.hardness,
        set: |u, v| u.brush.hardness = v,
        // ⚠️ **Ela some onde o dab não LÊ a distância** — ver
        // [`shapes_the_distance`]. Era `always`, e sob um campo elástico isso
        // eram dois controles (pista + chip) que não movem um vértice: medido
        // ao bit em `measure_where_the_curve_knobs_reach`.
        show: shapes_the_distance,
        // ⚠️ **O caso mais limpo de Pro que esta tabela tem:** o valor de fábrica
        // é `0`, que é o NEUTRO da etapa de dureza da referência (com dureza zero
        // a etapa não corre), então escondê-la no Basic não tira capacidade
        // nenhuma de ninguém — ela só some
        // de vista com o pincel exatamente como estava.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    // **O ALISAMENTO DE CADA DAB** — logo abaixo da dureza, que é onde o Blender
    // o põe (são vizinhos no declarador de propriedades dele), e pelo mesmo motivo que pôs a
    // dureza abaixo da força: os dois trocam **borda dura** por **superfície que
    // a malha consegue carregar**, e é ao lado um do outro que a troca se lê.
    //
    // ⚠️ **Ele não é oferecido no Smooth nem na Máscara**, e as duas exclusões
    // são do original: alisar um alisamento é o mesmo verbo duas vezes, e um
    // passe que mexesse na posição durante um gesto de MÁSCARA moveria o barro
    // num gesto cuja razão de existir é não movê-lo. A pergunta é feita à PORTA
    // do motor ([`ph2d_sculpt3d::Brush::auto_smooth_brush`], que devolve `None`
    // nos dois casos) e não a uma lista de nomes aqui — duas cópias divergiriam
    // num knob que aparece e não muda um vértice.
    Row {
        label: "panel.sculpt3d.auto_smooth",
        slider: crate::ids::SCULPT3D_AUTO_SMOOTH,
        chip: crate::ids::SCULPT3D_AUTO_SMOOTH_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.auto_smooth,
        set: |u, v| u.brush.auto_smooth = v,
        show: |u| {
            ph2d_sculpt3d::Brush {
                auto_smooth: 1.0,
                ..u.brush.clone()
            }
            .auto_smooth_brush()
            .is_some()
        },
        // ⚠️ **Pro pela MESMA razão que a dureza, e a regra é a mesma:** o valor
        // de fábrica é `0`, que é o neutro do próprio Blender, então escondê-lo
        // não tira capacidade de ninguém — ele some de vista com o pincel
        // exactamente como estava. (O falloff saiu do Pro em 2026-08-16 porque a
        // REFERÊNCIA o mostra sempre; aqui ela o guarda no avançado.)
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.plane_offset",
        slider: crate::ids::SCULPT3D_PLANE_OFFSET,
        chip: crate::ids::SCULPT3D_PLANE_OFFSET_NUM,
        // Com SINAL, e é o que separa Flatten de Clay sem inventar um verbo:
        // positivo adiciona matéria, negativo raspa.
        min: -1.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de uma fração do raio, não métrica de layout
        decimals: 2,
        get: |u| u.brush.plane_offset,
        set: |u, v| u.brush.plane_offset = v,
        show: |u| u.brush.verb.uses_plane(),
        // ⚠️ **Pro, e o teste é *"esconder deixa a ferramenta sem o que o nome
        // dela promete?"*.** Não: os quatro verbos de plano rodam na referência
        // EXATA com o knob em zero (o Clay levanta o plano dele pelo
        // `CLAY_PLANE_FRACTION`, no kernel, e este número SOMA àquele), e quem
        // quer o outro lado tem o Fill e o Scrape como chips. É afinação sobre
        // um default que a referência escolheu, que é a definição de Pro.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.pinch",
        slider: crate::ids::SCULPT3D_PINCH,
        chip: crate::ids::SCULPT3D_PINCH_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.pinch,
        set: |u, v| u.brush.pinch = v,
        show: |u| u.brush.verb == Verb::Crease,
        // ⚠️ **Pro sobre um valor ARMADO**: o `Brush::default().pinch` nasce em
        // `0,5` e o Crease aperta desde o primeiro traço. Um knob que nascesse
        // em zero seria amputação — o verbo se chamaria *vincar* e só cavaria.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    hc::HC_SHAPE,
    hc::HC_VERTEX,
    // **A PONTA DA FAIXA**, os dois knobs que fazem dela uma faixa.
    //
    // ⚠️ **A pergunta é a MESMA que o motor faz** (`verb == ClayStrips`, o que a
    // [`ph2d_sculpt3d::Footprint`] consome) — uma segunda lista de verbos aqui
    // seria um par de sliders que não move um vértice no dia em que a moldura
    // ganhasse um segundo consumidor.
    Row {
        label: "panel.sculpt3d.tip_roundness",
        slider: crate::ids::SCULPT3D_TIP_ROUNDNESS,
        chip: crate::ids::SCULPT3D_TIP_ROUNDNESS_NUM,
        min: 0.0,
        // ⚠️ **UM é o disco, e é alcançável de propósito** — a caixa totalmente
        // arredondada É a distância euclidiana, então o teto do knob é a
        // ferramenta a colapsar no Clay com portão de profundidade. Quem quiser
        // isso pode; o que ele não pode ser é o DEFAULT (foi, e o smoke o pegou:
        // *"parece redondo"*).
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.tip_roundness,
        set: |u, v| u.brush.tip_roundness = v,
        show: |u| u.brush.verb == Verb::ClayStrips,
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.strip_length",
        slider: crate::ids::SCULPT3D_STRIP_LENGTH,
        chip: crate::ids::SCULPT3D_STRIP_LENGTH_NUM,
        // ⚠️ **O piso é `1`, e não `0`:** o número é *quantos raios a faixa mede
        // ao longo do caminho*, então abaixo de um a pegada seria mais CURTA que
        // larga — uma faixa atravessada, que é o oposto do que o nome diz. O
        // motor recusa `0` de qualquer forma (`Strip::new` devolve `None`), e um
        // slider que alcança um valor que o motor recusa é um controle que
        // mente.
        min: 1.0,
        // O teto é MEDIDO pela consulta que ele paga: a pegada alcança
        // `√(1 + L²)` raios, então `4` já pede uma consulta de 4,1 raios — 17×
        // a área de um disco. Além disso a tira deixa de caber num traço curto.
        max: 4.0,   // LITERAL-PX-OK: teto de um knob adimensional, não métrica de layout
        step: 0.25, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.strip_length,
        set: |u, v| u.brush.strip_length = v,
        show: |u| u.brush.verb == Verb::ClayStrips,
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    // **A ABERTURA DO V**, o único knob que faz da lâmina uma lâmina.
    Row {
        label: "panel.sculpt3d.scrape_angle",
        slider: crate::ids::SCULPT3D_SCRAPE_ANGLE,
        chip: crate::ids::SCULPT3D_SCRAPE_ANGLE_NUM,
        // ⚠️ **ZERO é alcançável e ali a ferramenta fica INERTE** — os dois
        // meios-planos coincidem com o plano TANGENTE, e num convexo não há nada
        // acima dele (medido: zero vértices movidos). Um piso acima de zero
        // esconderia uma continuidade que a física tem; o que ele não pode ser é
        // o default, e não é.
        min: 0.0,
        // ⚠️ **O teto é o da REFERÊNCIA** (o declarador de propriedades dela), não nosso — ver
        // [`ph2d_sculpt3d::MULTIPLANE_ANGLE_MAX_DEG`], que traz a tabela do que
        // de facto acontece lá em cima.
        max: ph2d_sculpt3d::MULTIPLANE_ANGLE_MAX_DEG,
        step: 5.0, // LITERAL-PX-OK: passo em GRAUS, não métrica de layout
        decimals: 0,
        get: |u| u.brush.scrape_angle_deg,
        set: |u, v| u.brush.scrape_angle_deg = v,
        show: |u| u.brush.verb == Verb::MultiplaneScrape,
        // ⚠️ **Basic, e é a única row desta wave que não é Pro:** esconder este
        // knob deixa a ferramenta sem o que o nome dela promete — *multiplane* É
        // o ângulo entre os planos. O teste do nível é *"esconder deixa a
        // ferramenta sem o que o nome dela promete?"*, e aqui a resposta é sim.
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // **A ESPESSURA DA DEMÃO** — ver [`ph2d_sculpt3d::Brush::layer_height`],
    // onde o default e as duas faixas têm a fonte e a medição ao lado.
    Row {
        label: "panel.sculpt3d.layer_height",
        slider: crate::ids::SCULPT3D_LAYER_HEIGHT,
        chip: crate::ids::SCULPT3D_LAYER_HEIGHT_NUM,
        // ⚠️ **ZERO é alcançável e ali a demão é INERTE** — uma camada de
        // espessura nenhuma não move um vértice —, e é a faixa declarada da
        // propriedade na referência (de `0` a `1`). Um piso acima de zero
        // esconderia uma continuidade que a lei tem.
        min: 0.0,
        // ⚠️ **O slider para na faixa de UI da referência e a caixa alcança a
        // DURA** — os dois números saem dela e nenhum é nosso; ver
        // [`ph2d_sculpt3d::LAYER_HEIGHT_UI_MAX`].
        max: ph2d_sculpt3d::LAYER_HEIGHT_UI_MAX,
        step: 0.01, // LITERAL-PX-OK: passo em unidades de OBJETO, não de layout
        decimals: 3,
        get: |u| u.brush.layer_height,
        set: |u, v| u.brush.layer_height = v,
        show: |u| u.brush.verb == Verb::Layer,
        // ⚠️ **Basic, pelo mesmo teste do ângulo do V:** esconder este knob
        // deixa a ferramenta sem o que o nome dela promete — uma *demão* É a
        // espessura, e sem ela sobra um Draw que satura num número que o artista
        // não escolheu.
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // **A FRACÇÃO DO RAIO QUE A NORMAL DO GESTO LÊ** — ver
    // [`ph2d_sculpt3d::Brush::normal_radius_frac`], onde o `0,5` de fábrica tem
    // a proveniência ao lado.
    Row {
        label: "panel.sculpt3d.normal_radius",
        slider: crate::ids::SCULPT3D_NORMAL_RADIUS,
        chip: crate::ids::SCULPT3D_NORMAL_RADIUS_NUM,
        // ⚠️ **ZERO é alcançável e a degeneração é NOMEADA**: sem amostra
        // nenhuma dentro da fracção, o gesto cai na normal do plano do carimbo —
        // que existe sempre. Um piso acima de zero esconderia uma continuidade
        // que a lei tem, e a lei já responde por ela.
        min: 0.0,
        // A fracção é do RAIO: acima de `1` ela deixaria de ser uma fracção, e a
        // pegada inteira já é o que o plano do carimbo lê.
        max: 1.0,
        step: 0.01, // LITERAL-PX-OK: fracção adimensional, não layout
        decimals: 2,
        get: |u| u.brush.normal_radius_frac,
        set: |u, v| u.brush.normal_radius_frac = v,
        // ⭐ **O afiado é o TERCEIRO a lê-la** (espec §2.3): a normal da área
        // dele soma sobre esta fracção do raio, e a ablação mede-a como knob
        // OBSERVÁVEL — as quatro amostras do corpus dão saídas que diferem entre
        // `1,9e-3` e `1,5e-2`.
        show: |u| matches!(u.brush.verb, Verb::Thumb | Verb::Nudge | Verb::DrawSharp),
        // ⚠️ **Pro, e o teste é o mesmo do vizinho com resposta OPOSTA:**
        // escondê-lo não deixa a ferramenta sem o que o nome promete — um
        // polegar continua a espalmar —, ele afina *de que superfície* o gesto
        // se declara paralelo.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    // ⚠️ **Ela NÃO é um seletor de falloff, e a distinção é da REFERÊNCIA.** O
    // canal de máscara do original tem curva PRÓPRIA — `(1 − d)^{2(1 − hardness)}`
    // (`Masking.js:66`) — enquanto as dez tools de geometria multiplicam pela
    // quártica que o nosso `Falloff` estende. É o *"cada tool deve ter seu
    // falloff apropriado"* onde ele não é escolha de produto: `hardness` é uma
    // família CONTÍNUA (expoente `2` a `0`, o topo sendo um disco duro), e o
    // seletor discreto ao lado governa outra pergunta.
    Row {
        label: "panel.sculpt3d.mask_hardness",
        slider: crate::ids::SCULPT3D_MASK_HARDNESS,
        chip: crate::ids::SCULPT3D_MASK_HARDNESS_NUM,
        min: 0.0,
        // ⚠️ O teto sai do MOTOR (`ph2d_sculpt3d::MAX_MASK_HARDNESS`), onde a
        // lei que o torna um disco duro está escrita; um literal aqui seria a
        // segunda cópia dele.
        max: ph2d_sculpt3d::MAX_MASK_HARDNESS,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.mask_hardness,
        set: |u, v| u.brush.mask_hardness = v,
        show: |u| u.brush.verb.paints_mask(),
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // ⭐ **AS SEIS PISTAS DO PADRÃO vivem no módulo do padrão** — as perguntas
    // que elas fazem (`directional_alpha`, `stamp_alpha`) e os números que elas
    // lêem (`MAX_AXIS_ELEV_F32`, `degrees`) já moravam lá; as fileiras ficavam
    // aqui, e era a única metade do assunto fora de casa.
    alpha::ALPHA_SCALE,
    alpha::STAMP_SCALE,
    alpha::ALPHA_OFF_X,
    alpha::ALPHA_OFF_Y,
    alpha::ALPHA_AZ,
    alpha::ALPHA_ELEV,
    // ── Os CINCO números do pincel de TECIDO ────────────────────────────────
    //
    // ⚠️⚠️ **Eles existiam na lei e não existiam no painel.** A tradução
    // `Brush → Pincel` escrevia os cinco como omissão literal, e o corpus do
    // oráculo tem fixture para cada um (`massa2`, `amort05`, `amort1`,
    // `plast05`, `pino`, `preset` com limite `5`). *Uma lei medida na bancada e
    // não ligada no produto é uma lei que o artista não tem* — é a mesma conta
    // que os oito modos e as três áreas pagaram em 06/09.
    //
    // ⚠️ **A pergunta de visibilidade é ao VERBO**, como a das duas fileiras de
    // chip: com outro pincel na mão eles não movem um vértice.
    cloth::CLOTH_LIMIT,
    cloth::CLOTH_FALLOFF,
    cloth::CLOTH_MASS,
    cloth::CLOTH_DAMPING,
    cloth::CLOTH_PLASTICITY,
    // ── Os TRÊS números do pincel de POSE ───────────────────────────────────
    //
    // ⚠️ A pergunta de visibilidade é ao VERBO, como a do tecido: com outro
    // pincel na mão eles não movem um vértice. ⭐ E os OUTROS três controlos
    // próprios deste pincel não são rows — o modo é uma fileira de chips e os
    // dois interruptores são caixas; ver `paint::brush` e `event_toggles`.
    pose::POSE_SEGMENTS,
    pose::POSE_OFFSET,
    pose::POSE_TRANSITION,
    // ⭐ **O único NÚMERO do contorno** — os outros dois controlos próprios
    // dele são fileiras de chips; ver `paint::brush` e `event`.
    boundary::BOUNDARY_OFFSET,
    // ⭐⭐⭐ **A *Quality* do pincel** — as varreduras que o ALVO FIXA em `5`.
    // Ver `Brush::cloth_sweeps` para a tabela do que ela compra.
    cloth_filter::CLOTH_SWEEPS,
    // ── Os QUATRO números do FILTRO de tecido ───────────────────────────────
    //
    // ⚠️⚠️ **Eles NÃO são os do pincel, e tratá-los como se fossem foi o
    // defeito** (pergunta do dono, 2026-09-08): o filtro lia
    // `brush.cloth_mass`/`_damping`/`_plasticity`, cuja omissão e faixa são
    // outras — o amortecimento dele nasce em `0` e o do pincel em `0,01`, com a
    // faixa a começar aí, então o filtro **nunca alcançava o próprio valor de
    // omissão**.
    //
    // ⚠️ **E a pergunta de visibilidade é ao FILTRO, não ao verbo**: desde a W9b
    // ele corre com qualquer pincel na mão, então com o Draw na mão os três
    // mexiam na simulação sem nada na tela os mostrar.
    // ⭐⭐⭐ **Os dois de 08/09 vêm PRIMEIRO, e são os únicos `Basic` do bloco** —
    // eles decidem se o pano é um pano; os outros quatro afinam um comportamento
    // que já está certo.
    // ⭐ **A força vem primeiro de todas** — é o único deles que o alvo também
    // tem, e é o dial que decide a MAGNITUDE de tudo o resto.
    cloth_filter::CFILTER_STRENGTH,
    cloth_filter::CFILTER_STRETCH,
    cloth_filter::CFILTER_VOLUME,
    cloth_filter::CFILTER_BEND,
    cloth_filter::CFILTER_MASS,
    cloth_filter::CFILTER_DAMPING,
    cloth_filter::CFILTER_PLASTICITY,
    cloth_filter::CFILTER_SWEEPS,
    // ── Os dois números do EXTRACT ──────────────────────────────────────────
    //
    // ⚠️ **Eles são os ARGUMENTOS de um botão, e ficam colados nele** — não são
    // knobs do pincel. É a mesma decisão que trouxe a pista de `Alpha Scale`
    // para a cauda: um controle e o que ele governa têm de estar no campo de
    // visão um do outro.
    extract::EXTRACT_THICKNESS,
    extract::EXTRACT_SMOOTH,
];
