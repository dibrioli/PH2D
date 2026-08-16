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

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_sculpt3d::Verb;

use crate::state::{Sculpt3dUi, UiLevel};

/// O teto do raio que o SLIDER oferece, em pixels de tela.
///
/// ⚠️ **Ele não é o teto do produto, e a diferença está medida:** quem aperta é
/// a ALTURA DO VIEWPORT (`RADIUS_MAX_FRAC_OF_HEIGHT = 1/8` da tela, no shell) —
/// 180 px a 1440p, 90 px a 720p. Um número fixo aqui seria um segundo teto livre
/// para divergir daquele, então este é só a extensão da pista: numa janela baixa
/// o valor **volta** ao encostar no teto real, porque o retrato que o painel
/// pinta é o número JÁ CLAMPADO que o dab usa. Uma pista que anda além do teto
/// é honesta; uma que mostra um número que o pincel não usa não é.
const RADIUS_TRACK_MAX_PX: f32 = 200.0; // LITERAL-PX-OK: extensao da PISTA, nao metrica de design (o teto real e 1/8 da altura do viewport)

#[path = "rows_types.rs"]
mod types;

/// Os knobs da LEITURA da forma — ver o doc do módulo.
#[path = "rows_shading.rs"]
mod shading;

/// Os knobs da RESOLUÇÃO da malha — ver o doc do módulo.
#[path = "rows_topology.rs"]
mod topology;
pub use topology::TOPOLOGY;

/// As perguntas que o PADRÃO faz — ver o doc do módulo.
#[path = "rows_alpha.rs"]
mod alpha;
use alpha::{MAX_AXIS_ELEV_F32, degrees, directional_alpha, stamp_alpha};

pub use types::{Place, Row, Section};

/// O teto da pista de **Extract Smooth**, em passadas.
///
/// ⚠️ **OITO, e o número é MEDIDO** (`ph2d-mesh/tests/measure_extract.rs`): o
/// relaxamento da costura **CONVERGE**, e o que ele compra por passada cai
/// rápido. Numa costura serrilhada — a que uma mão pintada deixa — a rugosidade
/// da beira vai de **0,09369 a 0,05117 em oito passadas (−45%)**, e da oitava em
/// diante cada uma compra **0,4%**. Uma pista mais longa seria uma faixa onde
/// arrastar não faz nada, que é o controle morto que esta casa varre a cada wave.
const MAX_EXTRACT_SMOOTH: f32 = 8.0; // LITERAL-PX-OK: contagem de passadas MEDIDA, nao metrica de design

/// Sempre visível. ⚠️ `pub(super)` porque a tabela do sombreamento é um
/// módulo FILHO e as duas a partilham — duas cópias divergiriam no dia em que
/// *sempre* ganhasse uma exceção.
pub(super) fn always(_: &Sculpt3dUi) -> bool {
    true
}

/// O pincel: o que se ajusta antes de encostar no barro.
static BRUSH: &[Row] = &[
    Row {
        label: "panel.sculpt3d.radius",
        slider: ids::SCULPT3D_RADIUS,
        chip: ids::SCULPT3D_RADIUS_NUM,
        min: 1.0,
        max: RADIUS_TRACK_MAX_PX,
        step: 1.0,
        decimals: 0,
        get: |u| u.radius_px,
        set: |u, v| u.radius_px = v,
        show: always,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.strength",
        slider: ids::SCULPT3D_STRENGTH,
        chip: ids::SCULPT3D_STRENGTH_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.strength,
        set: |u, v| u.brush.strength = v,
        show: always,
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
    // (`apply_hardness_to_distances`, `sculpt.cc:7549`). Um verbo pode oferecer
    // as duas ao mesmo tempo, e é por isso que elas não podem compartilhar um
    // controle.
    Row {
        label: "panel.sculpt3d.hardness",
        slider: ids::SCULPT3D_HARDNESS,
        chip: ids::SCULPT3D_HARDNESS_NUM,
        min: 0.0,
        // ⚠️ **UM é o disco duro, e ele é alcançável de propósito** — o
        // `shaped_distance` tem braço próprio para ele justamente porque a
        // fórmula geral divide por `1 − h`. O teto do Blender é o mesmo.
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.hardness,
        set: |u, v| u.brush.hardness = v,
        show: always,
        // ⚠️ **O caso mais limpo de Pro que esta tabela tem:** o valor de fábrica
        // é `0`, que é o NEUTRO do próprio original (o
        // `apply_hardness_to_distances` abre com `if (hardness == 0.0f) return;`),
        // então escondê-la não tira capacidade nenhuma de ninguém — ela só some
        // de vista com o pincel exatamente como estava.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    // **O ALISAMENTO DE CADA DAB** — logo abaixo da dureza, que é onde o Blender
    // o põe (`rna_brush.cc:3450` contra `:3457`), e pelo mesmo motivo que pôs a
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
        slider: ids::SCULPT3D_AUTO_SMOOTH,
        chip: ids::SCULPT3D_AUTO_SMOOTH_NUM,
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
        slider: ids::SCULPT3D_PLANE_OFFSET,
        chip: ids::SCULPT3D_PLANE_OFFSET_NUM,
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
        slider: ids::SCULPT3D_PINCH,
        chip: ids::SCULPT3D_PINCH_NUM,
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
    // **OS DOIS KNOBS DO HC** — o que ele DEVOLVE, e de onde a devolução vem.
    //
    // ⚠️ **A pergunta é `verb == SurfaceSmooth`, e não `uses_neighbours()`** — o
    // irmão [`Verb::Smooth`] também lê o anel e não tem `b` nenhum para
    // devolver; oferecer-lhe estes dois seria um par de sliders que não move um
    // vértice, que é exactamente o knob morto que este arquivo evita.
    Row {
        label: "panel.sculpt3d.hc_shape",
        slider: ids::SCULPT3D_HC_SHAPE,
        chip: ids::SCULPT3D_HC_SHAPE_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.hc_shape,
        set: |u, v| u.brush.hc_shape = v,
        show: |u| u.brush.verb == Verb::SurfaceSmooth,
        // ⚠️ **Pro, e o default está no MEIO de um fator** — a varredura mediu
        // uma troca monótona e SUAVE em toda a faixa (segurar mais a pose custa
        // 1,6× menos deriva por 7% menos alisamento, sem joelho e sem cliff),
        // então não há óptimo a escolher: o `0,5` é o meio, dito e não vestido
        // de medição. Ver `ph2d_sculpt3d::HC_SHAPE_DEFAULT`.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.hc_vertex",
        slider: ids::SCULPT3D_HC_VERTEX,
        chip: ids::SCULPT3D_HC_VERTEX_NUM,
        // ⚠️ **O piso é `0,5` e ele NÃO é gosto:** abaixo dele o operador
        // AMPLIFICA em vez de contrair, e a faixa do Blender (`[0, 1]`) alcança
        // o disfuncional. A forma fechada e a tabela medida vivem em
        // `ph2d_sculpt3d::HC_VERTEX_DEFAULT`; aqui fica só a consequência — o
        // dedo não chega lá, e o clamp do MOTOR cobre o que um documento traga.
        min: ph2d_sculpt3d::HC_VERTEX_MIN,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.brush.hc_vertex,
        set: |u, v| u.brush.hc_vertex = v,
        show: |u| u.brush.verb == Verb::SurfaceSmooth,
        // ⚠️ **O default É o piso**, e é onde ele alisa MAIS (0,6806 da
        // rugosidade removível contra 0,6903 em 0,650). O outro extremo, `1`, é
        // o `strength = 0` deste verbo escrito no outro eixo: a correção passa a
        // ser exactamente o que o passo laplaciano somou.
        level: UiLevel::Pro,
        place: Place::Knobs,
    },
    // **A PONTA DA FAIXA**, os dois knobs que fazem dela uma faixa.
    //
    // ⚠️ **A pergunta é a MESMA que o motor faz** (`verb == ClayStrips`, o que a
    // [`ph2d_sculpt3d::Footprint`] consome) — uma segunda lista de verbos aqui
    // seria um par de sliders que não move um vértice no dia em que a moldura
    // ganhasse um segundo consumidor.
    Row {
        label: "panel.sculpt3d.tip_roundness",
        slider: ids::SCULPT3D_TIP_ROUNDNESS,
        chip: ids::SCULPT3D_TIP_ROUNDNESS_NUM,
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
        slider: ids::SCULPT3D_STRIP_LENGTH,
        chip: ids::SCULPT3D_STRIP_LENGTH_NUM,
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
        slider: ids::SCULPT3D_SCRAPE_ANGLE,
        chip: ids::SCULPT3D_SCRAPE_ANGLE_NUM,
        // ⚠️ **ZERO é alcançável e ali a ferramenta fica INERTE** — os dois
        // meios-planos coincidem com o plano TANGENTE, e num convexo não há nada
        // acima dele (medido: zero vértices movidos). Um piso acima de zero
        // esconderia uma continuidade que a física tem; o que ele não pode ser é
        // o default, e não é.
        min: 0.0,
        // ⚠️ **O teto é o da REFERÊNCIA** (`rna_brush.cc:3382`), não nosso — ver
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
        slider: ids::SCULPT3D_LAYER_HEIGHT,
        chip: ids::SCULPT3D_LAYER_HEIGHT_NUM,
        // ⚠️ **ZERO é alcançável e ali a demão é INERTE** — uma camada de
        // espessura nenhuma não move um vértice —, e é a faixa da referência
        // (`RNA_def_property_range(prop, 0, 1.0f)`). Um piso acima de zero
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
    // ⚠️ **Ela NÃO é um seletor de falloff, e a distinção é da REFERÊNCIA.** O
    // canal de máscara do original tem curva PRÓPRIA — `(1 − d)^{2(1 − hardness)}`
    // (`Masking.js:66`) — enquanto as dez tools de geometria multiplicam pela
    // quártica que o nosso `Falloff` estende. É o *"cada tool deve ter seu
    // falloff apropriado"* onde ele não é escolha de produto: `hardness` é uma
    // família CONTÍNUA (expoente `2` a `0`, o topo sendo um disco duro), e o
    // seletor discreto ao lado governa outra pergunta.
    Row {
        label: "panel.sculpt3d.mask_hardness",
        slider: ids::SCULPT3D_MASK_HARDNESS,
        chip: ids::SCULPT3D_MASK_HARDNESS_NUM,
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
    Row {
        label: "panel.sculpt3d.alpha_scale",
        slider: ids::SCULPT3D_ALPHA_SCALE,
        chip: ids::SCULPT3D_ALPHA_SCALE_NUM,
        // ⚠️ Os dois extremos são do MOTOR, não escolhidos aqui: eles saem da lei
        // das dez arestas (`ph2d_sculpt3d::DEFAULT_ALPHA_SCALE`), e um literal
        // nesta tabela seria a segunda cópia deles.
        min: ph2d_sculpt3d::MIN_ALPHA_SCALE,
        max: ph2d_sculpt3d::MAX_ALPHA_SCALE,
        step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
        decimals: 2,
        get: |u| u.brush.alpha_scale,
        set: |u, v| u.brush.alpha_scale = v,
        // ⚠️ **A row some sem padrão armado**, e não é cosmético: o número é o
        // tamanho de uma feature que não existe. É o mesmo mecanismo das duas
        // pistas de lâmpada sob um matcap — uma row condicional é PULADA, nunca
        // pintada apagada, porque um controle que desenha e não responde mente.
        //
        // ⚠️ **E ela some também sob um CARIMBO**, porque a pergunta muda de
        // régua: um estêncil é medido na TELA, não no modelo, e quem responde
        // por ele é a row seguinte. Reusar este número com duas unidades faria
        // ele trocar de significado em silêncio ao trocar de padrão.
        show: |u| u.brush.alpha.is_some() && !stamp_alpha(u),
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    Row {
        label: "panel.sculpt3d.stamp_scale",
        slider: ids::SCULPT3D_STAMP_SCALE,
        chip: ids::SCULPT3D_STAMP_SCALE_NUM,
        // ⚠️ **A faixa é em FRAÇÃO DA ALTURA DA TELA**, e por isso ela não fala
        // do modelo: `1,0` é um ladrilho ocupando a tela inteira e `0,02` são
        // cinquenta atravessando-a. Um estêncil não sabe o tamanho da peça — é
        // justamente essa independência que o artista pediu —, então herdar a
        // pista de `Pattern Size` (unidades de OBJETO, semeada pela densidade da
        // malha) seria herdar a régua errada.
        min: 0.02,  // LITERAL-PX-OK: fração da altura da vista, não métrica de layout
        max: 1.0,   // LITERAL-PX-OK: idem
        step: 0.01, // LITERAL-PX-OK: idem
        decimals: 2,
        get: |u| u.brush.alpha_stencil_scale,
        set: |u, v| u.brush.alpha_stencil_scale = v,
        show: stamp_alpha,
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    Row {
        label: "panel.sculpt3d.alpha_off_x",
        slider: ids::SCULPT3D_ALPHA_OFF_X,
        chip: ids::SCULPT3D_ALPHA_OFF_X_NUM,
        // ⚠️ **A faixa é SIMÉTRICA e mede um LADO do modelo.** Uma primitiva
        // nasce cabendo na esfera unitária (span 2), então ±1 leva o carimbo de
        // uma ponta à outra; e o zero tem de cair no MEIO da pista, porque
        // *nenhum deslocamento* é o estado neutro e não um extremo.
        min: -1.0,
        max: 1.0,
        step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
        decimals: 2,
        get: |u| u.brush.alpha_offset[0],
        set: |u, v| u.brush.alpha_offset[0] = v,
        show: stamp_alpha,
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    Row {
        label: "panel.sculpt3d.alpha_off_y",
        slider: ids::SCULPT3D_ALPHA_OFF_Y,
        chip: ids::SCULPT3D_ALPHA_OFF_Y_NUM,
        min: -1.0,
        max: 1.0,
        step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
        decimals: 2,
        get: |u| u.brush.alpha_offset[1],
        set: |u, v| u.brush.alpha_offset[1] = v,
        show: stamp_alpha,
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    Row {
        label: "panel.sculpt3d.alpha_az",
        slider: ids::SCULPT3D_ALPHA_AZ,
        chip: ids::SCULPT3D_ALPHA_AZ_NUM,
        min: 0.0,
        // 359 e não 360 — os dois extremos seriam o MESMO azimute, e uma pista
        // cujas duas pontas significam a mesma coisa tem um degrau invisível. É
        // a mesma régua do `light_az`, e de propósito: um artista que aprendeu a
        // apontar a luz não devia reaprender a apontar o padrão.
        max: 359.0, // LITERAL-PX-OK: graus de azimute, nao metrica de design
        step: 5.0,  // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| f32::from(u.brush.alpha_az_deg),
        set: |u, v| u.brush.alpha_az_deg = degrees(v),
        show: directional_alpha,
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    Row {
        label: "panel.sculpt3d.alpha_elev",
        slider: ids::SCULPT3D_ALPHA_ELEV,
        chip: ids::SCULPT3D_ALPHA_ELEV_NUM,
        // ⚠️ **Sem o piso que a LÂMPADA tem.** Lá o `MIN_ELEV_DEG` existe porque
        // uma luz rasante degenera a resposta plana; um EIXO não degenera em
        // lugar nenhum — o frame é ortonormal por identidade em qualquer
        // elevação. Copiar o piso do vizinho seria um limite herdado por
        // analogia, que é o que esta casa varre a cada wave.
        min: 0.0,
        // ⚠️ O zênite vem do MOTOR, não é escolhido aqui: acima dele o eixo
        // desceria do outro lado e o azimute já cobre esse hemisfério — dois
        // caminhos para a mesma direção.
        max: MAX_AXIS_ELEV_F32,
        step: 5.0, // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| f32::from(u.brush.alpha_elev_deg),
        set: |u, v| u.brush.alpha_elev_deg = degrees(v),
        // ⚠️ **Ela some sob um CARIMBO, e é o modo inteiro numa linha:** o eixo
        // de um estêncil é a VISTA, por definição. Um controle que o inclinasse
        // tiraria o carimbo da frente — exatamente o que este modo existe para
        // impedir —, então ele não é oferecido em vez de ser oferecido e
        // ignorado.
        show: |u| directional_alpha(u) && !stamp_alpha(u),
        level: UiLevel::Basic,
        place: Place::AfterAlpha,
    },
    // ── Os dois números do EXTRACT ──────────────────────────────────────────
    //
    // ⚠️ **Eles são os ARGUMENTOS de um botão, e ficam colados nele** — não são
    // knobs do pincel. É a mesma decisão que trouxe a pista de `Alpha Scale`
    // para a cauda: um controle e o que ele governa têm de estar no campo de
    // visão um do outro.
    Row {
        label: "panel.sculpt3d.extract_thickness",
        slider: ids::SCULPT3D_EXTRACT_THICK,
        chip: ids::SCULPT3D_EXTRACT_THICK_NUM,
        // ⚠️ **A faixa é sobre a ESCALA LOCAL da malha, e as primitivas desta
        // casa nascem com raio 1** — meia unidade é meia peça, e é a faixa
        // confortável do arrasto. O sinal escolhe o lado: para fora é armadura,
        // para dentro é forro. **Zero é uma folha só**, e é ele que está no meio
        // da pista de propósito.
        min: -0.5,
        max: 0.5,
        step: 0.01, // LITERAL-PX-OK: passo de uma espessura em unidades de malha
        decimals: 3,
        get: |u| u.extract.thickness,
        set: |u, v| u.extract.thickness = v,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::AfterExtract,
    },
    Row {
        label: "panel.sculpt3d.extract_smooth",
        slider: ids::SCULPT3D_EXTRACT_SMOOTH,
        chip: ids::SCULPT3D_EXTRACT_SMOOTH_NUM,
        min: 0.0,
        max: MAX_EXTRACT_SMOOTH,
        step: 1.0, // LITERAL-PX-OK: uma passada e' inteira
        decimals: 0,
        get: |u| u.extract.smooth as f32,
        // ⚠️ O `round` é a fronteira de DISPLAY: a pista fala em `f32` como toda
        // row desta tabela, e o que o kernel conta é uma passada inteira.
        set: |u, v| u.extract.smooth = v.round().max(0.0) as u32,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::AfterExtract,
    },
];

/// Toda seção que tem rows de slider, em ordem de pintura.
///
/// ⚠️ **Nem toda seção do painel está aqui** — Tool, Symmetry e Scene são botões
/// e rádios, não knobs contínuos, e forçá-las nesta tabela pediria uma `Row` que
/// soubesse ser um botão. Elas são pintadas pelo `paint/body.rs`, que é quem
/// conhece a ordem completa.
///
/// ⚠️ **A Topology ENTROU quando ganhou o primeiro knob contínuo** (a resolução
/// do remesh), e o que a traz para cá não é a pintura — ela continua sendo
/// desenhada à mão, porque o resto dela são botões — e sim as outras três listas:
/// `populate`, `event` e a varredura de costura percorrem esta tabela, então uma
/// row que mora nela nasce registrada, viva sob o mouse e varrida. Uma row
/// pintada à mão FORA daqui seria o controle morto que esta casa varre a cada
/// wave.
pub static SECTIONS: &[Section] = &[
    Section {
        id: ids::SCULPT3D_SEC_BRUSH,
        title: "panel.sculpt3d.section.brush",
        rows: BRUSH,
    },
    Section {
        id: ids::SCULPT3D_SEC_SHADING,
        title: "panel.sculpt3d.section.shading",
        rows: shading::SHADING,
    },
    Section {
        id: ids::SCULPT3D_SEC_TOPOLOGY,
        title: "panel.sculpt3d.section.topology",
        rows: topology::TOPOLOGY,
    },
];

/// Toda row, achatada — o que `populate`, `event` e a varredura de costura
/// percorrem.
pub fn rows() -> impl Iterator<Item = &'static Row> {
    SECTIONS.iter().flat_map(|s| s.rows.iter())
}

/// A row a que um id pertence, se alguma (a pista ou o chip dela).
pub fn row_for(id: NodeId) -> Option<&'static Row> {
    rows().find(|r| r.slider == id || r.chip == id)
}
