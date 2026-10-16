//! ⭐⭐ **A CENA ABRE INTEIRA** — a vista com que uma cena de demo (`PH2D_GPU_COOK_DEMO=<n>`) nasce.
//!
//! Achado de 15/10: com a câmara de arranque (10 m) as linhas de cima de 46 fichas e das figuras
//! delas ficavam por baixo da barra de ferramentas (`=82..=98`, `=110`, `=118`, `=119`), e a ficha da
//! `=139` colava à borda. A lei:
//!
//! - **A figura é MEDIDA**: o que a bomba pinta em todo tique de `2 · CICLO` s, peça a peça
//!   ([`EXTENSAO`], escrita pela sonda `a_tabela_das_extensoes` e RE-MEDIDA pelo gate
//!   `motion_demo_enquadra_tests`). ⛔ Nenhum número por cena é escolhido aqui.
//! - **O fundo não conta** ([`e_fundo`]): uma peça sozinha mais alta ou mais larga do que a vista de
//!   omissão é o cenário de que a cena fala (o fundo de 28 m das `=140`/`=142`).
//! - **As fichas contam** com o tamanho que TÊM no ecrã (`readout::chip_width` × `CHIP_H_PX`): no
//!   mundo elas crescem quando a câmara se afasta, logo a vista é o ponto fixo de [`vista`].
//! - **A vista de omissão fica quando tudo já cabe** com a margem do *Enquadrar tudo*
//!   (`float_bar::frame_margin`); senão enquadra-se tudo, **nunca mais perto** do que a omissão — o
//!   tamanho das peças de cada cena foi medido nessa câmara.
//! - **Uma cena que é um CAMPO** (a figura em `t = 0` já contém a vista de omissão inteira: as grelhas
//!   de população) abre na omissão — está-se dentro dela. Fica fora da tabela, e o gate confere-o.
//! - **Uma cena cujas peças FOGEM** — a figura do ciclo pede mais de [`FOGE_ACIMA_DE`] vezes a altura
//!   da do ARRANQUE (`t = 0`) junto com o canvas visto da omissão ([`razao_da_fuga`]): a fonte que
//!   cai 70 m, a corda solta que cai 40 m, as plumas da `=95` a ±40 m — abre nesta última: o que
//!   fugiu não afasta a vista (com o ciclo inteiro, a `=50` abria a 79 m com a figura num canto de um
//!   canvas vazio). ⚠️ O corte é de escala, num vão medido, não uma natureza (ver a constante).
//!
//! A shell só APLICA ([`abre_inteira`], uma chamada por quadro na fase da ponte); `PH2D_DEMO_ALTURA`
//! (bissecção do zoom) continua a mandar na altura.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_render::camera_frame::{Area, View, frame_within};

use crate::motion_state::MotionState;

/// A figura MEDIDA de uma cena, `[x0, y0, x1, y1]` m, sem o fundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extensao {
    /// A de todos os tiques de `2 · CICLO` s.
    pub ciclo: [f32; 4],
    /// A do arranque (`t = 0`).
    pub arranque: [f32; 4],
}

/// ⭐ **Quando o ciclo FOGE** — o corte da [`razao_da_fuga`]. O recurso é o TAMANHO das peças no
/// ecrã (medido na câmara de omissão): acima do corte, abrir no ciclo encolhê-las-ia mais do que a
/// cena aguenta, e abre-se no arranque.
///
/// ⛔ **É um corte num VÃO entre amostras, não uma natureza.** Medido a 15/10 (sondas
/// `candidatos_da_fuga` e `a_razao_noutras_janelas`, as 131 cenas da tabela, `2 · CICLO` a 60 Hz):
/// nenhum descritor da DINÂMICA separa as classes — cada um cruza-as (`=15`/`=16` fora):
///
/// | candidato | foge | cresce no sítio |
/// |---|---|---|
/// | alcance: crescimento no 2.º ciclo ÷ no 1.º | 0 (`=113`) … 9,6 (`=52`) | 0 … 1,37 (`=139`) |
/// | instante em que o alcance chega a 95 % ÷ `2·CICLO` | 0,31 (`=113`) … 0,98 | 0 … 0,96 (`=75`) |
/// | peças que batem no 2.º ciclo o recorde do 1.º | 0 (`=113`) … 1 | 0 … 1 (`=139`) |
/// | a caixa em `t = CICLO` contra a de `t = 0` | 0,14 (`=52`) … 504 | 0 … 61 (`=67`) |
/// | o instante preenche a figura do ciclo (máx.) | 0,74 (`=113`) … 1 | 0,58 … 1 |
/// | **esta razão** | **≥ 1,571** (`=138`) | **≤ 1,454** (`=128`) |
///
/// (os discos da `=138` caem e ASSENTAM no 1.º ciclo; a galáxia da `=139` continua a crescer no 2.º:
/// a «fuga» é de ESCALA.) O vão é `1,454` (`=128`, igual nas quatro janelas do gate) → `1,571`
/// (`=138` a 1366×768; 1,581 na foto), ×1,081; o corte é o centro geométrico dele: folga de 3,9 % e
/// 4,1 %. ⚠️ **O risco:** uma cena nova ou uma mudança que caia no vão muda o modo de abrir sem
/// ninguém decidir — o gate (`motion_demo_enquadra_tests`) confere a classe de cada cena em quatro
/// janelas contra a lista decidida (`FOGEM`) e reprova a menos de `PERTO_DO_CORTE` (3 %) do corte.
/// ⚠️ Até 15/10 este doc dizia 1,48 (`=139`) – 1,59 com o corte em 1,55: era a razão SEM o canvas
/// da omissão, não a que a lei corta — nela a `=10` (0,76) e a `=102` (1,33) já abriam no ciclo.
pub const FOGE_ACIMA_DE: f32 = 1.51;

/// ⭐ **A figura MEDIDA de cada cena.** ⛔ Escrita pela sonda (`cargo test -p ph2d-app-motion --lib
/// a_tabela_das_extensoes -- --ignored --nocapture`), nunca à mão: o gate re-mede cada linha.
#[path = "motion_demo_enquadra_tabela.rs"]
mod tabela;
pub(crate) use tabela::EXTENSAO;

/// Uma peça sozinha maior do que a vista de omissão (em altura OU largura) é o FUNDO.
#[must_use]
pub fn e_fundo(p: [f32; 4]) -> bool {
    let h = Camera2d::default().height_world;
    p[2] - p[0] >= h || p[3] - p[1] >= h
}

/// A figura medida do nível `level` — `None` para um campo, uma cena sem figura ou nenhuma cena.
#[must_use]
pub fn extensao(level: Option<&str>) -> Option<Extensao> {
    let l = level?;
    EXTENSAO
        .iter()
        .find(|(n, ..)| *n == l)
        .map(|&(_, ciclo, arranque)| Extensao { ciclo, arranque })
}

/// Uma ficha: a âncora de mundo e a meia-largura × meia-altura em px de ecrã.
pub type Ficha = ([f32; 2], [f32; 2]);

/// O que se enquadra numa vista de `k` px/m: a figura e as fichas, em mundo.
#[must_use]
pub fn caixa(figura: [f32; 4], fichas: &[Ficha], k: f32) -> [f32; 4] {
    fichas.iter().fold(figura, |b, (c, e)| {
        [
            b[0].min(c[0] - e[0] / k),
            b[1].min(c[1] - e[1] / k),
            b[2].max(c[0] + e[0] / k),
            b[3].max(c[1] + e[1] / k),
        ]
    })
}

/// O arredondamento de `f32` de uma vista que encosta EXATAMENTE à margem (o `frame_within` conta em
/// `f64`), em px.
const ARREDONDA_PX: f32 = 1e-2;

/// Cabe `b` (mundo) na `area` (px da janela da câmara) com `margem` px, sob a vista `v`?
#[must_use]
pub fn cabe(b: [f32; 4], v: View, area: Area, janela: WindowSize, margem: f32) -> bool {
    #[expect(clippy::cast_precision_loss, reason = "px de janela")]
    let (ww, wh) = (janela.width as f32, janela.height as f32);
    let k = wh / v.height_world;
    let x = |m: f32| 0.5 * ww + (m - v.center[0]) * k;
    let y = |m: f32| 0.5 * wh - (m - v.center[1]) * k;
    let m = margem - ARREDONDA_PX;
    x(b[0]) >= area.x + m
        && x(b[2]) <= area.x + area.w - m
        && y(b[3]) >= area.y + m
        && y(b[1]) <= area.y + area.h - m
}

/// O canvas visto (`area`, px) em mundo, sob a vista `v`.
#[must_use]
pub fn area_no_mundo(v: View, area: Area, janela: WindowSize) -> [f32; 4] {
    #[expect(clippy::cast_precision_loss, reason = "px de janela")]
    let (ww, wh) = (janela.width as f32, janela.height as f32);
    let k = wh / v.height_world;
    let x = |px: f32| v.center[0] + (px - 0.5 * ww) / k;
    let y = |py: f32| v.center[1] + (0.5 * wh - py) / k;
    [x(area.x), y(area.y + area.h), x(area.x + area.w), y(area.y)]
}

/// A figura do arranque junto com o canvas visto da omissão.
fn arranque_visto(e: Extensao, area: Area, janela: WindowSize) -> [f32; 4] {
    let o = area_no_mundo(Camera2d::default().view(), area, janela);
    let b = e.arranque;
    [
        b[0].min(o[0]),
        b[1].min(o[1]),
        b[2].max(o[2]),
        b[3].max(o[3]),
    ]
}

/// ⭐ **A razão da fuga** — a altura da vista do ciclo sobre a do arranque visto
/// ([`arranque_visto`]): quantas vezes cada peça encolhe no ecrã se a cena abrir no ciclo em vez de
/// no arranque. É o número que [`FOGE_ACIMA_DE`] corta.
#[must_use]
pub fn razao_da_fuga(
    e: Extensao,
    fichas: &[Ficha],
    area: Area,
    janela: WindowSize,
    margem: f32,
) -> f32 {
    let h = |f| vista(f, fichas, area, janela, margem).height_world;
    h(e.ciclo) / h(arranque_visto(e, area, janela))
}

/// ⭐ **O que a cena enquadra** — a figura do ciclo, ou, quando ela FOGE ([`FOGE_ACIMA_DE`]), a do
/// arranque junto com o canvas visto da omissão (abrir nunca mostra menos do que ela mostrava).
#[must_use]
pub fn a_enquadrar(
    e: Extensao,
    fichas: &[Ficha],
    area: Area,
    janela: WindowSize,
    margem: f32,
) -> [f32; 4] {
    if razao_da_fuga(e, fichas, area, janela, margem) > FOGE_ACIMA_DE {
        arranque_visto(e, area, janela)
    } else {
        e.ciclo
    }
}

/// ⭐ **A vista com que a cena abre** — a omissão para um campo ou uma cena fora da tabela.
#[must_use]
pub fn vista_de(
    e: Option<Extensao>,
    fichas: &[Ficha],
    area: Area,
    janela: WindowSize,
    margem: f32,
) -> View {
    e.map_or(Camera2d::default().view(), |e| {
        let f = a_enquadrar(e, fichas, area, janela, margem);
        vista(f, fichas, area, janela, margem)
    })
}

/// ⭐ **A LEI**: a vista de omissão se a figura e as fichas já cabem nela; senão a de
/// `camera_frame` sobre tudo, com a altura entre a omissão e o zoom máximo — o ponto fixo das fichas
/// (que crescem no mundo à medida que a câmara se afasta), que converge por baixo.
#[must_use]
pub fn vista(
    figura: [f32; 4],
    fichas: &[Ficha],
    area: Area,
    janela: WindowSize,
    margem: f32,
) -> View {
    let omissao = Camera2d::default().view();
    #[expect(clippy::cast_precision_loss, reason = "px de janela")]
    let wh = janela.height as f32;
    let k0 = wh / omissao.height_world;
    if cabe(caixa(figura, fichas, k0), omissao, area, janela, margem) {
        return omissao;
    }
    let faixa = (omissao.height_world, Camera2d::ZOOM_MAX_HEIGHT_WORLD);
    let mut v = omissao;
    // A altura só cresce (as fichas no mundo crescem com ela); pára quando deixa de crescer.
    for _ in 0..64 {
        let b = caixa(figura, fichas, wh / v.height_world);
        let bb = ph2d_render::WorldBbox {
            min: [b[0], b[1]],
            max: [b[2], b[3]],
        };
        let Some(n) = frame_within(bb, area, janela, margem, faixa) else {
            return omissao;
        };
        let parou = n.height_world <= v.height_world * (1.0 + f32::EPSILON);
        v = n;
        if parou {
            break;
        }
    }
    v
}

/// As fichas da cena como o produto as pinta.
pub fn fichas_de(motion: &MotionState, ts: &mut ph2d_text::TextSystem) -> Vec<Ficha> {
    fichas(&crate::motion_demo_legend::no_mundo(motion), ts)
}

/// As fichas de legendas já levadas ao mundo.
pub fn fichas(
    legendas: &[crate::motion_demo_legend::Caption],
    ts: &mut ph2d_text::TextSystem,
) -> Vec<Ficha> {
    legendas
        .iter()
        .map(|c| {
            let w = ph2d_editor_core::readout::chip_width(ts, &c.text, 0.0);
            (
                c.world,
                [0.5 * w, 0.5 * ph2d_editor_core::readout::CHIP_H_PX],
            )
        })
        .collect()
}

/// ⭐ **O pedido de abertura** que a cena de demo arma ao nascer — `None` depois de cumprido.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Abertura {
    nivel: String,
    /// A área e a vista do último quadro em que se enquadrou: a mesma área no quadro seguinte =
    /// o layout assentou; a câmara diferente da vista posta = o artista mexeu-lhe.
    ultima: Option<(Area, View)>,
}

impl Abertura {
    /// O pedido da cena `nivel`.
    #[must_use]
    pub fn de(nivel: &str) -> Self {
        Self {
            nivel: nivel.to_owned(),
            ultima: None,
        }
    }
}

/// O pedido da cena que o ambiente montou — `None` sem `PH2D_GPU_COOK_DEMO` ou sem sinks (a mesma
/// cerca do `demo_wants_the_motion_tool`).
#[must_use]
pub fn pedida(sinks: usize) -> Option<Abertura> {
    let n = std::env::var("PH2D_GPU_COOK_DEMO").ok()?;
    (sinks > 0).then(|| Abertura::de(&n))
}

/// ⭐ **A PORTA DA SHELL**, uma vez por quadro: com a ferramenta Motion já a dividir o centro e o
/// layout publicado, põe a vista de [`vista`] (ou a altura do `PH2D_DEMO_ALTURA`) e repete-a enquanto
/// a área visível ainda muda; acaba quando ela assenta ou quando a câmara deixa de ser a posta.
/// `cena` é a janela da CENA — a banda do split (`scene_mapping::janela` na shell), nunca a
/// superfície inteira: a mesma conta mundo↔tela de quem desenha.
pub fn abre_inteira(
    motion: &mut MotionState,
    hero: &HeroScreen,
    camera: &mut Camera2d,
    cena: WindowSize,
    ts: &mut ph2d_text::TextSystem,
) {
    let Some(ab) = motion.abertura.as_ref() else {
        return;
    };
    let split = hero.view.center_split;
    if matches!(split, ph2d_editor_core::screens::layout::CenterSplit::None)
        || hero.last_content.w <= 0.0
    {
        return;
    }
    #[expect(clippy::cast_precision_loss, reason = "px de janela")]
    let (cw, ch) = (cena.width as f32, cena.height as f32);
    let r = ph2d_app_host::canvas_area::frameable(
        hero,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, cw, ch),
    );
    let area = Area {
        x: r.x,
        y: r.y,
        w: r.w.min(cw - r.x).max(0.0),
        h: r.h.min(ch - r.y).max(0.0),
    };
    if let Some((a, v)) = ab.ultima
        && (camera.view() != v || a == area)
    {
        motion.abertura = None;
        return;
    }
    let nivel = ab.nivel.clone();
    let v = match crate::motion_demo_altura::altura_semeada() {
        Some(h) => View {
            center: Camera2d::default().center,
            height_world: h,
        },
        None => {
            let margem =
                ph2d_editor_core::screens::hero::float_bar::frame_margin() * hero.escala().factor();
            let fichas = fichas_de(motion, ts);
            vista_de(extensao(Some(&nivel)), &fichas, area, cena, margem)
        }
    };
    camera.set_view(v);
    if let Some(ab) = motion.abertura.as_mut() {
        ab.ultima = Some((area, v));
    }
}
