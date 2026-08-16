//! **A LEI DO TRAÇO** — a peça que separa este módulo de um port ingênuo.
//!
//! > O efeito de um traço é função do **CAMINHO**, nunca de quão fino o motor
//! > amostrou o caminho. (`docs/3D/04.1`)
//!
//! No ZBrush e no Blender cada dab soma sobre o RESULTADO do anterior, então o
//! que sobrevive a `n` dabs é um **produto sobre a lista de dabs** — e a lista
//! depende da taxa de amostragem do mouse. Passar devagar deposita mais que
//! passar rápido pelo mesmo caminho. A `line/Painter` pagou esse bug **quatro
//! vezes** em 2D (mordida do arado · cápsula do relevo · campo de smear · gate
//! de proteção) até formular a cura, e ela vale igual em 3D:
//!
//! ```text
//! pen-down:  base[v] ← positions[v]          // congela o "pre"
//! por dab:   accum[v] ← max(accum[v], w)     // ENVELOPE, não `+=`
//! por dab:   target[v] ← alvo(verbo)         // do dab que VENCEU
//! aplica:    positions[v] ← lerp(base, target, accum)
//! ```
//!
//! Três propriedades caem disso, e as três são visíveis para o artista:
//!
//! 1. **Independência de espaçamento** — devagar ou rápido dá o mesmo resultado.
//! 2. **Idempotência sob re-stamp** — repetir a mesma lista de dabs não
//!    intensifica nada, o que é o que permitiria editar parâmetros do traço
//!    *depois* dele.
//! 3. **Undo trivial** — `base` **é** o estado anterior e `touched` **é** a
//!    janela; não há um segundo sistema a construir.
//!
//! ⚠️ **O `target` guarda o VENCEDOR, não uma média.** Quando um dab novo eleva
//! o `accum` de um vértice, ele também reescreve o alvo — o mesmo desenho do
//! envelope do impasto 2D, que guarda *os ingredientes do dab mais carregado*.
//! Sem isso, um verbo cujo alvo depende do dab (todos os de plano) teria de
//! recomputar a pegada inteira a cada dab, e o gesto deixaria de ser limitado
//! pela pegada.
//!
//! ⚠️ **Um vértice NÃO capturado tem `pre == posição viva`** — porque só quem foi
//! capturado é escrito. É isso que deixa o Smooth ler a vizinhança sem capturar
//! o anel inteiro, e é por isso que `base_pos_of` cai na malha viva sem mentir.

use crate::brush::{Brush, Symmetry, Verb};
use crate::grip::{Amount, Grip};

/// **O QUE UM TOQUE É** — ver [`dab`].
#[path = "dab.rs"]
mod dab;
pub use dab::Dab;
use ph2d_mesh::{Mesh, QueryScratch, RegionScratch};

/// Um ponto ou vetor refletido pelo trio de sinais de uma cópia da simetria.
fn mirror(v: [f32; 3], s: &[f32; 3]) -> [f32; 3] {
    [v[0] * s[0], v[1] * s[1], v[2] * s[2]]
}

/// O estado vivo de UM traço de escultura.
///
/// Os dois vetores do tamanho da malha (`slot`/`stamp`) vivem aqui e são
/// **reusados entre traços** — carimbados por época, como o `QueryScratch`. O
/// resto é do tamanho da PEGADA do traço, e é isso que mantém a memória de um
/// gesto proporcional ao que o artista tocou e não ao que ele abriu.
#[derive(Clone, Debug, Default)]
pub struct SculptStroke {
    slot: Vec<u32>,
    stamp: Vec<u32>,
    epoch: u32,
    touched: Vec<u32>,
    base_pos: Vec<[f32; 3]>,
    base_nrm: Vec<[f32; 3]>,
    base_mask: Vec<f32>,
    /// **A saída por-índice do map do dab** — ver [`super::stroke_map`]. ⚠️
    /// Campo e não local: alocar `count` entradas por dab devolveria ao
    /// alocador o que o paralelismo economiza.
    par_out: Vec<Option<DabOut>>,
    /// **A porta de ABLAÇÃO do piso do pool** — só em teste. ⚠️ Sem ela os
    /// gates ficariam verdes sobre uma rota que NUNCA roda: as fixtures deste
    /// módulo ficam sob o piso (a armadilha do ADR-0120).
    #[cfg(test)]
    pub(crate) par_floor_override: Option<usize>,
    accum: Vec<f32>,
    target: Vec<[f32; 3]>,
    footprint: Vec<u32>,
    moved: Vec<u32>,
    query: QueryScratch,
    region: RegionScratch,
    /// A união, **sobre as cópias de espelho de UMA chamada a
    /// [`SculptStroke::dab`]**, dos vértices escritos e dos vértices cuja normal
    /// foi recomputada.
    ///
    /// ⚠️ **Eles existem porque `moved` e `region` são o rascunho de UMA cópia.**
    /// O `dab_core` zera a lista antes de a encher, e com o espelho armado ele
    /// roda de duas a oito vezes — então quem lia o rascunho depois do laço
    /// recebia a ÚLTIMA cópia e só ela. A malha ficava certa na memória e a
    /// janela de upload descrevia metade dela: o artista tocava um lado, o outro
    /// deformava na tela (report do Enio, 2026-08-05). A distinção entre
    /// *rascunho de uma cópia* e *o que a chamada fez* não era exprimível, e por
    /// isso não podia ser conferida — agora são campos diferentes com nomes
    /// diferentes, e os acessores públicos leem estes.
    call_moved: Vec<u32>,
    call_refreshed: Vec<u32>,
    /// O último dab pintou MÁSCARA? Decide de qual janela a GPU precisa — ver
    /// [`SculptStroke::last_gpu_dirty`]. Um bool escrito no mesmo `if` que já
    /// separa os dois braços; derivá-lo do `Brush` no chamador seria pedir a ele
    /// que soubesse a regra.
    last_paints_mask: bool,
    /// **O CENTRO DO DAB ANTERIOR**, para derivar [`Dab::path`].
    ///
    /// ⚠️ Guardado ANTES da expansão do espelho: o que interessa é o caminho que
    /// a mão percorreu, e as cópias de simetria são todas do mesmo gesto.
    last_center: Option<[f32; 3]>,
    /// **A INCLINAÇÃO ACUMULADA do [`Verb::ClayThumb`]**, em graus.
    ///
    /// ⚠️ **Irmão do [`Self::last_center`], e pela mesma razão:** é um fato
    /// sobre o GESTO, não sobre o dab. O `Dab` diz onde a mão apertou; quantos
    /// dabs já passaram é do traço, e o `clay_thumb.cc` o guarda exatamente
    /// aqui — no `StrokeCache`, não no evento.
    ///
    /// ⚠️ **Avançado UMA vez por chamada a [`Self::dab`], antes do espelho** —
    /// a referência o gateia em `stroke_is_main_symmetry_pass`, e a nossa
    /// fronteira de chamada É essa passada: avançá-lo por cópia faria a
    /// inclinação correr de duas a oito vezes mais rápido com o espelho armado,
    /// e o artista veria a ferramenta mudar de lei ao ligar a simetria.
    thumb_tilt_deg: f32,
    /// **A ABERTURA DO V do [`Verb::MultiplaneScrape`]**, em graus — o
    /// `cache->multiplane_scrape_angle`.
    ///
    /// ⚠️ **Ele é do TRAÇO porque o modo dinâmico o SUAVIZA contra o dab
    /// anterior** ([`crate::MULTIPLANE_ANGLE_SMOOTH`]); no modo fixo ele é
    /// simplesmente reescrito, e é isso que impede um valor lido numa passada
    /// dinâmica de ressuscitar depois de o artista desmarcar o modo.
    scrape_angle_deg: f32,
    /// **A LÂMINA EM V deste dab** — hoisted, como o `alpha_frame` e a
    /// silhueta.
    ///
    /// ⚠️ **Estado de DAB num campo de TRAÇO, e a razão é a assinatura:** ela
    /// nasce de `&mut self` (o modo dinâmico tem memória) e é lida do `&self`
    /// que o alvo por-vértice recebe — as duas metades não cabem no mesmo
    /// empréstimo, e passá-la por argumento significaria enfiar um `Option` a
    /// mais em `compute_target`, que dezanove verbos não leem. É a mesma rota
    /// que o `thumb_tilt_deg` já usa.
    ///
    /// ⚠️ **`None` é *este dab não deposita*.** Ela é reescrita no topo de todo
    /// dab, então nunca sobrevive ao seguinte.
    scrape: Option<plane::ScrapePlanes>,
    /// **O `b` do HC, por SLOT** — ver [`super::stroke_hc`].
    ///
    /// ⚠️ **Vazio para vinte e um dos vinte e dois verbos**, e é o `fill_hc_disp`
    /// que o dimensiona: um plano por-slot a mais no `capture` cobraria 12 bytes
    /// por vértice tocado a quem nunca escolheu este pincel.
    hc_b: Vec<[f32; 3]>,
}

impl SculptStroke {
    /// **Quantos vértices o último dab tocou** — sonda, para uma medição de
    /// custo poder dizer se duas colunas fizeram o mesmo trabalho.
    #[doc(hidden)]
    #[must_use]
    pub fn footprint_len(&self) -> usize {
        self.footprint.len()
    }
}

impl SculptStroke {
    /// Congela o `pre`: começa um traço novo sobre `mesh`.
    ///
    /// Não copia a malha — a captura é **preguiçosa, por vértice tocado**. Um
    /// traço numa malha de 5 M vértices que toca 20 mil paga 20 mil, não 5 M.
    pub fn begin(&mut self, mesh: &Mesh) {
        let n = mesh.vert_count();
        if self.slot.len() != n {
            self.slot = vec![u32::MAX; n];
            self.stamp = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — a mesma regra do `QueryScratch`, e sem ela um
        // traço a cada 4 bilhões nasceria achando que já capturou tudo.
        if self.epoch == 0 {
            self.epoch = 1;
            self.stamp.fill(0);
        }
        self.touched.clear();
        self.base_pos.clear();
        self.base_nrm.clear();
        self.base_mask.clear();
        self.accum.clear();
        self.target.clear();
        // ⚠️ **Um traço novo não herda a direção do anterior** — sem esta linha o
        // primeiro dab apontaria para onde a mão ia no gesto passado, que é um
        // lugar arbitrário.
        self.last_center = None;
        // ⚠️ **Nem a inclinação**, e a referência escreve a mesma linha
        // (`clay_thumb.cc:166`, `front_angle = 0` no primeiro passo do traço).
        // Sem ela o segundo traço começaria de onde o primeiro parou, e o
        // artista veria a mesma ferramenta cavar mais fundo por ter sido usada
        // antes.
        self.thumb_tilt_deg = 0.0;
        // ⚠️ **Nem a abertura do V**, pela mesma razão — e no modo dinâmico ela
        // é MEMÓRIA, então herdá-la faria o primeiro dab do traço novo raspar
        // com o ângulo que a superfície tinha noutro lugar.
        self.scrape_angle_deg = 0.0;
        self.scrape = None;
        // ⚠️ **O `b` do HC morre com o traço**, e é o que faz dele o *"array
        // zerado no início do traço"* do `surface_smooth.cc`: dentro de um gesto
        // ele PERSISTE entre dabs (a lei da referência), entre gestos não.
        self.hc_b.clear();
    }
}

/// **A ANATOMIA DE UM DAB** — a sequência que amarra os módulos acima. Ver
/// [`dab_core`].
#[path = "stroke_dab_core.rs"]
mod dab_core;

/// **O `pre` QUE O TRAÇO CONGELA** — ver [`freeze`]. Irmão dos três abaixo, e o
/// assunto que os três já pressupunham sem ter casa própria.
/// **QUANTAS VEZES UM DAB ACONTECE** — ver [`symmetry`].
#[path = "stroke_symmetry.rs"]
mod symmetry;

#[path = "stroke_map.rs"]
mod stroke_map;
pub(crate) use stroke_map::DabOut;

#[path = "stroke_freeze.rs"]
mod freeze;

/// **O QUE UMA SONDA PODE PERGUNTAR** — ver [`probe`]. O corte é *o que um dab
/// FAZ* (aqui) contra *o que se consegue MEDIR dele* (lá).
#[path = "stroke_probe.rs"]
mod probe;

/// **O PLANO que quatro verbos ajustam** — ver [`plane`]. O corte é o `e` que o
/// cabeçalho do [`target`] carregava: *para onde um verbo aponta* difere entre
/// os dezasseis, *que forma a superfície tem* é uma pergunta só.
#[path = "stroke_plane.rs"]
mod plane;

/// **A SUPERFÍCIE LOCAL do `l-mode`** — ver [`surface`]. Irmão do [`plane`], e o
/// corte são dois PAPERS: lá o `calc_area_normal_and_center` da referência, aqui
/// a projeção MLS de Alexa et al. 2003.
#[path = "stroke_surface.rs"]
mod surface;

/// **QUE SILHUETA ESTE DAB TEM** — ver [`shape`].
#[path = "stroke_shape.rs"]
mod shape;

/// **A MALHA CRESCEU DEBAIXO DO TRAÇO** — o refino e a lei do `pre`.
///
/// Filho para alcançar os planos congelados; o corte é o mesmo do
/// `stroke_growth_tests.rs`: aqui mora *o que acontece quando a topologia muda
/// no meio de um gesto*, no pai *o que um dab faz*.
#[path = "stroke_growth.rs"]
mod growth;

/// **O ALVO de cada verbo**, e o plano que quatro deles ajustam. Filho para
/// alcançar o `pre` congelado; o corte é *a LEI do traço* (aqui) contra *para
/// onde cada verbo aponta* (lá).
#[path = "stroke_target.rs"]
mod target;

/// **O QUE O ANEL DIZ** — a média congelada e a normal que o relax remove.
/// Filho pelo mesmo motivo do [`target`]: os dois leem o `pre`.
#[path = "stroke_ring.rs"]
mod ring;

/// **O `b` DO HC** — o buffer que o [`Verb::SurfaceSmooth`] exige. Filho pelo
/// mesmo motivo dos dois acima, e o corte é *o que a vizinhança DIZ* (ali)
/// contra *o que o passo laplaciano TIROU* (lá) — a segunda pergunta só existe
/// para um verbo, e o buffer que ela obriga é a única parte estrutural dele.
#[path = "stroke_hc.rs"]
mod hc;
pub use hc::{HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, HC_VERTEX_MIN};

/// **O QUE O TRAÇO ESCREVE** — os dois aplicadores. Filho pela mesma razão do
/// [`target`]: eles leem os planos congelados. O corte é *a LEI* (aqui) contra
/// *a ESCRITA* (lá).
#[path = "stroke_apply.rs"]
mod apply;

/// **AS JANELAS QUE UM TRAÇO PUBLICA** — ver [`windows`].
#[path = "stroke_windows.rs"]
mod windows;

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "stroke_growth_tests.rs"]
mod growth_tests;

#[cfg(test)]
#[path = "stroke_window_tests.rs"]
mod window_tests;

#[cfg(test)]
#[path = "stroke_accum_tests.rs"]
mod accum_tests;

#[cfg(test)]
#[path = "stroke_alpha_tests.rs"]
mod alpha_tests;
