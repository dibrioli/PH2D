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
    /// **Esta row é pintada pela CAUDA da seção, e não no bloco de knobs.**
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
    pub in_tail: bool,
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
        in_tail: false,
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
        in_tail: false,
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
        in_tail: false,
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
        in_tail: false,
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
        in_tail: true,
    },
];

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
        in_tail: false,
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
        in_tail: false,
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
        in_tail: false,
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
