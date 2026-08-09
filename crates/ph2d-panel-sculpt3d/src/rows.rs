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

use crate::state::Sculpt3dUi;

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

/// O teto da pista de **Extract Smooth**, em passadas.
///
/// ⚠️ **OITO, e o número é MEDIDO** (`ph2d-mesh/tests/measure_extract.rs`): o
/// relaxamento da costura **CONVERGE**, e o que ele compra por passada cai
/// rápido. Numa costura serrilhada — a que uma mão pintada deixa — a rugosidade
/// da beira vai de **0,09369 a 0,05117 em oito passadas (−45%)**, e da oitava em
/// diante cada uma compra **0,4%**. Uma pista mais longa seria uma faixa onde
/// arrastar não faz nada, que é o controle morto que esta casa varre a cada wave.
const MAX_EXTRACT_SMOOTH: f32 = 8.0; // LITERAL-PX-OK: contagem de passadas MEDIDA, nao metrica de design

/// Uma row de slider+chip: que número ela edita, sobre que faixa, e **quando ela
/// existe**.
pub struct Row {
    /// Chave i18n do rótulo.
    pub label: &'static str,
    /// Id da pista.
    pub slider: NodeId,
    /// Id do chip numérico ligado a ela.
    pub chip: NodeId,
    /// Mínimo do domínio (o valor em `track = 0`).
    pub min: f32,
    /// Máximo do domínio (o valor em `track = 1`).
    pub max: f32,
    /// Passo do arrasto do chip. ⚠️ Não é decoração: sem faixa+passo registrados
    /// o chip deriva o passo do texto do buffer e percorre ~50 unidades por
    /// PIXEL, o que o transforma num interruptor min↔max (o bug que o painel do
    /// Flip documentou — digitar sempre funcionou, só arrastar estava quebrado).
    pub step: f64,
    /// Quantas casas o readout mostra.
    pub decimals: usize,
    /// Lê o valor desta row do estado autorado.
    pub get: fn(&Sculpt3dUi) -> f32,
    /// Escreve o valor desta row no estado autorado.
    pub set: fn(&mut Sculpt3dUi, f32),
    /// **Esta row existe com este pincel em mãos?**
    ///
    /// ⚠️ O `Plane Offset` só é lido pelos quatro verbos de plano e o `Pinch` só
    /// pelo Crease — pintá-los sempre seriam dois knobs que não fazem nada em
    /// doze das dezesseis ferramentas, que é o controle morto que esta casa
    /// varre a cada wave. A pergunta é feita à porta do MOTOR
    /// (`Verb::uses_plane`), nunca a uma lista paralela de nomes.
    pub show: fn(&Sculpt3dUi) -> bool,
    /// **ONDE, na seção, esta row é desenhada.**
    ///
    /// ⚠️ Ela existe porque *posição na tela* é uma pergunta que a tabela não
    /// respondia, e a resposta errada custou um smoke: a pista de `Alpha Scale`
    /// nasceu no bloco de knobs, ou seja **acima** da fileira de chips que a
    /// governa e separada dela pelo Falloff — um controle órfão, que aparece do
    /// nada e não se liga a nada que o artista acabou de tocar.
    ///
    /// ⚠️ **A row continua na tabela**, e é isso que importa: `populate`, `event`
    /// e a varredura de costura seguem a percorrendo, então ela nasce registrada,
    /// viva e varrida como qualquer outra. O que este campo move é **onde ela é
    /// desenhada**, e só isso — a alternativa (tirá-la da tabela e pintá-la à
    /// mão) a tiraria das três listas de uma vez.
    pub place: Place,
}

/// Em que ponto da seção a row é pintada.
///
/// ⚠️ **Era um `bool`, e o terceiro valor o obrigou a virar isto.** Enquanto
/// havia só *no bloco* × *na cauda*, dois estados bastavam; os dois números do
/// extract são argumentos de um BOTÃO que mora no fim da seção, e pintá-los onde
/// as pistas do alpha moram os separaria do gesto que os lê. Um `bool` com um
/// `if` por id ao lado seria a enumeração que apodrece na quarta row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    /// O bloco de knobs contínuos, no topo da seção.
    Knobs,
    /// Logo abaixo do seletor de padrão, colada aos chips que a governa.
    AfterAlpha,
    /// No fim da seção, colada ao botão de extract que a lê.
    AfterExtract,
}

impl Row {
    /// Pista (`0..=1`) → o valor que ela significa.
    pub fn value_of(&self, track: f32) -> f32 {
        self.min + track * (self.max - self.min)
    }

    /// O valor → a pista dele. A inversa de [`Row::value_of`], e as duas têm de
    /// continuar inversas: o painel publica uma pista a partir do estado a cada
    /// frame e lê um valor de volta a cada arrasto, então um descasamento é um
    /// controle que **anda sozinho enquanto você o segura**
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]).
    pub fn track_of(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// O `link_slider_number_mapped` exprime o mesmo mapa como
    /// `display = track * scale + offset`.
    pub fn scale(&self) -> f32 {
        self.max - self.min
    }

    /// Ver [`Row::scale`].
    pub fn offset(&self) -> f32 {
        self.min
    }
}

/// Um grupo de rows com título. A lista de seções **É** a ordem de pintura.
pub struct Section {
    /// Id do cabeçalho dobrável.
    pub id: NodeId,
    /// Chave i18n do título.
    pub title: &'static str,
    /// As rows dele.
    pub rows: &'static [Row],
}

/// Sempre visível.
fn always(_: &Sculpt3dUi) -> bool {
    true
}

/// **Só com a luz do DOCUMENTO em uso.**
///
/// ⚠️ Um matcap é sombreamento função apenas da normal de VISTA: ele não lê o
/// rig, por definição. Deixar as duas pistas de lâmpada pintadas sob um matcap
/// seriam **dois controles que não fazem nada** — e não um pouco: o artista
/// arrastaria o ângulo da luz olhando uma escultura que não se move, que é a
/// forma mais cara de descobrir o que um modo significa.
fn under_the_rig(u: &Sculpt3dUi) -> bool {
    u.matcap.is_none()
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
        show: |u| u.brush.alpha.is_some(),
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
        show: directional_alpha,
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
        place: Place::AfterExtract,
    },
];

/// **Só com um CARIMBO armado** — as duas pistas de colocação.
///
/// ⚠️ **A pergunta é `is_image`, e não `is_directional`, e a diferença é o que
/// separa um carimbo de um campo:** os três procedurais direcionais apontam para
/// um lado e são HOMOGÊNEOS ao longo dele — um campo infinito não tem posição,
/// só fase, e uma fase é outro controle (uma semente) que este módulo não tem.
/// Oferecer o deslocamento ali seriam duas pistas que o Strata ignora por
/// completo e que o Scratches e o Weave leem como um número sem significado.
///
/// ⚠️ E a neutralidade dos outros não depende desta função: quem a garante é o
/// `Brush::alpha_frame`, que ZERA o deslocamento sem uma imagem armada. Esta
/// decide o que APARECE; aquele decide o que o motor recebe — e é por isso que
/// esconder a row aqui não pode deixar um valor autorado agindo em silêncio.
fn stamp_alpha(u: &Sculpt3dUi) -> bool {
    u.brush
        .alpha
        .as_ref()
        .is_some_and(ph2d_sculpt3d::Alpha::is_image)
}

/// **Só com um padrão DIRECIONAL armado.**
///
/// ⚠️ A pergunta é feita à porta do MOTOR ([`ph2d_sculpt3d::Alpha::is_directional`]),
/// nunca a uma lista de nomes aqui: sob um dos seis isotrópicos o eixo não move
/// um bit — há gate provando —, e duas pistas que desenham e não fazem nada são
/// o controle morto que esta casa varre a cada wave. É a mesma lei do
/// `Plane Offset` e das duas pistas de lâmpada sob um matcap.
fn directional_alpha(u: &Sculpt3dUi) -> bool {
    u.brush
        .alpha
        .as_ref()
        .is_some_and(ph2d_sculpt3d::Alpha::is_directional)
}

/// Um valor de pista → graus inteiros.
///
/// ⚠️ **A pista é `f32` e o ângulo é `u16`**, e a conversão mora AQUI, na
/// fronteira, e não no motor: o rotor deste app anda de grau em grau, então um
/// ângulo fracionário não teria como ser resolvido sem um segundo caminho. É a
/// mesma travessia que o painel já faz para as duas pistas de lâmpada.
/// ⚠️ **ARREDONDA, e o gate de costura pegou o truncamento na hora.** A row
/// mostra zero casas, então `134,625` é lido como **135** no readout; truncando,
/// o padrão iria para 134 e o número na tela discordaria do eixo que o pincel
/// usa — a doença de *seed ≠ sample* que este repo já pagou em quatro módulos.
/// A tolerância de `0,5` do gate `each_row_owns_exactly_one_field` é literalmente
/// o arredondamento que ele espera encontrar aqui.
///
/// ⚠️ **`safe_clamp` e não `.clamp`**, e o `arch_safe_clamp_only` foi quem cobrou:
/// o teto `f32::from(u16::MAX)` **não é um literal**, e o `.clamp` da `std`
/// **panica** com bounds trocados e devolve o valor original com `NaN`. Aqui um
/// `NaN` cairia no `as u16`, que é comportamento definido mas absurdo (zero) — a
/// peneira tem de vir antes.
fn degrees(v: f32) -> u16 {
    ph2d_editor_core::math::safe_clamp(v.round(), 0.0, f32::from(u16::MAX)) as u16
}

/// O zênite do eixo, lido do dono dele.
const MAX_AXIS_ELEV_F32: f32 = ph2d_sculpt3d::MAX_AXIS_ELEV_DEG as f32; // CLAMP-OK: teto do motor

/// Como a forma é LIDA — a cavidade e a lâmpada.
static SHADING: &[Row] = &[
    Row {
        label: "panel.sculpt3d.cavity",
        slider: ids::SCULPT3D_CAVITY,
        chip: ids::SCULPT3D_CAVITY_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.cavity,
        set: |u, v| u.cavity = v,
        show: always,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.light_az",
        slider: ids::SCULPT3D_LIGHT_AZ,
        chip: ids::SCULPT3D_LIGHT_AZ_NUM,
        min: 0.0,
        // 359 e não 360: os dois extremos seriam o MESMO azimute, e uma pista
        // cujas duas pontas significam a mesma coisa tem um degrau invisível.
        max: 359.0, // LITERAL-PX-OK: graus de azimute, nao metrica de design
        step: 5.0,  // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| u.light_az_deg,
        set: |u, v| u.light_az_deg = v,
        show: under_the_rig,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.light_elev",
        slider: ids::SCULPT3D_LIGHT_ELEV,
        chip: ids::SCULPT3D_LIGHT_ELEV_NUM,
        // ⚠️ O piso é o do RESOLVEDOR de luz, não um número escolhido aqui:
        // abaixo dele a resposta plana vai a zero e o modelo relativo dividiria
        // por ~0. Um literal aqui seria a segunda cópia dele.
        min: MIN_ELEV_DEG_F32,
        max: 90.0, // LITERAL-PX-OK: graus de elevacao (o zenite), nao metrica de design
        step: 5.0, // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| u.light_elev_deg,
        set: |u, v| u.light_elev_deg = v,
        show: under_the_rig,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.env",
        slider: ids::SCULPT3D_ENV,
        chip: ids::SCULPT3D_ENV_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.env,
        set: |u, v| u.env = v,
        // ⚠️ **Só sob o rig, e pela razão mais forte desta tabela: um matcap JÁ
        // É um ambiente.** Ele é uma esfera de iluminação capturada — o piso, o
        // céu e o realce dele vêm todos da mesma imagem —, então o termo do
        // estúdio não entra naquele caminho (o `mesh.wgsl` nem chega ao piso) e
        // o slider seria um controle que não faz nada. É a mesma lei das duas
        // pistas de lâmpada logo abaixo.
        show: under_the_rig,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.ao",
        slider: ids::SCULPT3D_AO,
        chip: ids::SCULPT3D_AO_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.ao,
        set: |u, v| u.ao = v,
        show: always,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.ssao",
        slider: ids::SCULPT3D_SSAO,
        chip: ids::SCULPT3D_SSAO_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.ssao,
        set: |u, v| u.ssao = v,
        show: always,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.sss",
        slider: ids::SCULPT3D_SSS,
        chip: ids::SCULPT3D_SSS_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.sss,
        set: |u, v| u.sss = v,
        show: always,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.sss_scatter",
        slider: ids::SCULPT3D_SSS_SCATTER,
        chip: ids::SCULPT3D_SSS_SCATTER_NUM,
        min: 0.0,
        // ⚠️ **O teto é 1,0 = "a luz atravessa a peça inteira"**, e ele não é um
        // limite de recurso: é onde a grandeza deixa de descrever um sólido. O
        // default MEDIDO é 0,25 (`sss::SCATTER_FRACTION`), e a faixa existe para
        // o artista decidir o LOOK — que é a única pergunta que a medição não
        // responde.
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.sss_scatter,
        set: |u, v| u.sss_scatter = v,
        // ⚠️ **Só com o espalhamento LIGADO.** Com a força em zero a tabela nem é
        // consultada, então este slider não moveria um pixel — e um controle que
        // não faz nada é o que esta casa varre a cada wave. É a mesma lei do
        // `Plane Offset`, que só existe nos verbos que leem um plano.
        show: |u| u.sss > 0.0,
        place: Place::Knobs,
    },
];

/// O piso da elevação, lido do dono dele.
const MIN_ELEV_DEG_F32: f32 = ph2d_light::MIN_ELEV_DEG as f32; // CLAMP-OK: piso do resolvedor de luz

/// Toda seção que tem rows de slider, em ordem de pintura.
///
/// ⚠️ **Nem toda seção do painel está aqui** — Tool, Symmetry, Topology e Scene
/// são botões e rádios, não knobs contínuos, e forçá-las nesta tabela pediria uma
/// `Row` que soubesse ser um botão. Elas são pintadas pelo `paint/body.rs`, que é
/// quem conhece a ordem completa.
pub static SECTIONS: &[Section] = &[
    Section {
        id: ids::SCULPT3D_SEC_BRUSH,
        title: "panel.sculpt3d.section.brush",
        rows: BRUSH,
    },
    Section {
        id: ids::SCULPT3D_SEC_SHADING,
        title: "panel.sculpt3d.section.shading",
        rows: SHADING,
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
