//! **O Óleo Real** (doc 49, [`README`](../../../docs/Painter/49_realshader/README.md) §4): a óptica da
//! tinta sobre o campo de ALTURA que a tinta já é — em luz linear, normal por Sobel, a sombra macia por
//! marcha na direcção da luz, e (W1c, medida contra o Cycles em [`oraculo/`](../../../docs/Painter/49_realshader/oraculo/))
//! o céu de um AMBIENTE ([`Ambiente`]) tapado pela visibilidade GTAO com multi-ricochete pela cor da tinta
//! (a fenda fica no tom, nunca preta), o reflexo desse ambiente e o realce que enche a folga até 1 em vez
//! de cortar. Referência em CPU; o passe da placa espelha-a.
//!
//! Lei de neutralidade: com a sombra e o brilho a zero, a tinta lisa sai com a cor que tinha
//! (`E_plano = 1`): a claridade continua a ser o pigmento.

use crate::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
use crate::grid::Grid;
pub use crate::material::MaterialCelula;
pub use crate::oleo_ambiente::Ambiente;
use crate::oleo_ambiente::{CUNHA, Sectores};
pub use oleo_material::{
    MaterialCartao, N_CAMPOS, PERFIL_SUBSUPERFICIE, RUGOSIDADE_PADRAO, alcance_subsuperficie,
};

#[path = "oleo_cache.rs"]
mod oleo_cache;
#[path = "oleo_material.rs"]
mod oleo_material;
pub use oleo_cache::{CacheCampo, ChaveCampo};
#[path = "oleo_borda.rs"]
mod oleo_borda;
#[path = "oleo_campo.rs"]
mod oleo_campo;
pub(crate) use oleo_borda::*;
#[path = "oleo_luz.rs"]
mod oleo_luz;
pub use oleo_luz::*;
#[path = "oleo_horizonte.rs"]
mod oleo_horizonte;
use oleo_horizonte::*;
#[path = "oleo_sombra.rs"]
mod oleo_sombra;
pub use oleo_sombra::*;
#[path = "oleo_pontinhos.rs"]
mod oleo_pontinhos;
pub use oleo_pontinhos::*;

/// Os knobs do Óleo Real. `1.0` em profundidade, sombra e brilho = o look das capturas `A_base`
/// (Rebelle: impasto 8, SoftShadows 10, Gloss 10, luz 42°).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OleoReal {
    /// Ganho GLOBAL da altura (`1` = a escala medida em W0(b), ≈ 15–17 px de relevo na faca). NÃO é knob do
    /// artista desde 28/10: o Depth do artista é do PINCEL e mora na célula (`Grid::oleo`); o produto deixa este a
    /// `1` — fica para as réguas e os testes do motor.
    pub profundidade: f64,
    /// De ONDE vem a luz, em radianos, no plano da tela (x para a direita, y para BAIXO): `225°` = topo-esquerda.
    pub azimute: f64,
    /// Altura da luz sobre a tela, em radianos (o Rebelle não desce de 42°; nós sim).
    pub altitude: f64,
    /// Força da sombra projectada, `0..=1` (linear no slider — M6).
    pub sombra: f64,
    /// Ganho GLOBAL do brilho especular, `0..=1` (M4: só o termo somado; nunca o volume). Como a
    /// [`Self::profundidade`]: o Gloss do artista é do pincel, na célula; o produto deixa este a `1`.
    pub brilho: f64,
    /// O céu que a tinta recebe e reflecte, rodado para o lado claro ficar no azimute da luz.
    pub ambiente: Ambiente,
    /// Força da sombra de FENDA (o céu que a tinta vizinha tapa), `0..=`[`FENDA_MAX`]: `0` = sem, `1` = a
    /// física (GTAO + multi-ricochete), acima = dramática — pela lei de [`forca_da_fenda`].
    pub fenda: f64,
    /// Até que distância (células) uma pincelada tapa o céu da vizinha, `1..=`[`ALCANCE_FENDA_MAX`]: as
    /// distâncias de [`HORIZONTE`] escalam por `alcance_fenda / ALCANCE_FENDA_MAX` (o custo não muda).
    pub alcance_fenda: f64,
    /// **Força do ambiente** (W2 E2), `0..=1`: a fracção da luz do plano que vem do AMBIENTE (o céu ou o
    /// HDRI, pelo sector visível de cada fatia); o resto é da lâmpada (difuso e brilho dela). `0` = só a
    /// lâmpada, `1` = só o ambiente (o `hdri_*` do oráculo), [`FORCA_AMBIENTE_FABRICA`] = a mistura de antes.
    pub forca_ambiente: f64,
    /// O material GLOBAL do cartão (W2 E3-B), combinado com o de cada célula ([`MaterialCartao`]).
    pub material: MaterialCartao,
    /// As constantes da lei, afinadas contra as capturas (não são knobs do artista).
    pub af: Afinacao,
    /// W3b (não é knob: o composite põe-no a cada recomposição): a cor LINEAR do papel do documento sob a tinta — a
    /// que a Transmission tinge ([`crate::material::transmite`]). Branco sem papel.
    pub papel: [f64; 3],
    /// W3b (não é knob, idem): a amplitude do dente do papel (px de relevo, `Relief × MAX_TOOTH_PX` do substrato) que
    /// o Relevo do papel põe na superfície da tinta; `0` sem papel.
    pub dente_papel: f64,
    /// D1 W3b (não é knob, idem): o raio (px) do **Smooth Edges** da TELA — a silhueta amaciada antes do campo
    /// ([`amacia_a_silhueta`]): a cobertura, as três alturas, a normal e a sombra seguem-na. `0` = desligado, ao bit.
    pub suave_px: f64,
}

impl Default for OleoReal {
    fn default() -> Self {
        Self {
            profundidade: 1.0,
            azimute: 225f64.to_radians(),
            altitude: 42f64.to_radians(),
            sombra: 1.0,
            brilho: 1.0,
            ambiente: Ambiente::default(),
            fenda: 1.0,
            alcance_fenda: ALCANCE_FENDA_MAX,
            forca_ambiente: FORCA_AMBIENTE_FABRICA,
            material: MaterialCartao::NEUTRO,
            af: Afinacao::default(),
            papel: [1.0; 3],
            dente_papel: 0.0,
            suave_px: 0.0,
        }
    }
}

/// As constantes da óptica. Ponto de partida = README §2b (W0); cada uma afina-se pela régua contra as
/// capturas do Rebelle (`49_realshader/capturas/`), nunca a olho sem a régua ao lado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Afinacao {
    /// px de relevo por unidade de massa (`sett+susp`) a profundidade 1 (W0(b)/(c): massa p50 ≈ 2 600 na
    /// faca ↔ ≈ 15 px de relevo no Rebelle).
    pub px_por_massa: f64,
    /// Raio (células) das duas passagens de média que dão a FORMA da pincelada a partir da massa.
    pub raio_suave: usize,
    /// Raio (células) da altura que faz a SOMBRA projectada (W1b): menor que o da forma, para a crista de uma
    /// passagem por cima projectar sombra dentro da tinta. A cavidade continua na forma.
    pub raio_sombra: usize,
    /// Raio (células) do pré-filtro do DETALHE antes da normal (W1b): o ruído de uma célula (grão e fios)
    /// derivado pelo Sobel serrilhava as estrias em pente claro/escuro.
    pub raio_detalhe: usize,
    /// Fracção do grão de célula a célula da massa que fica no relevo (à escala cheia faz uma lixa).
    pub grao: f64,
    /// Ganho da altura dos fios (`Grid::altura`, px).
    pub fios: f64,
    /// Largura da penumbra, em tangente de elevação: a luz vai de cheia a nada quando o horizonte sobe
    /// `penumbra` acima da elevação dela. Maior = sombra mais macia.
    pub penumbra: f64,
    /// Intensidade do brilho (a rugosidade é a Roughness do material, [`MaterialCartao::rugosidade`]).
    pub brilho_max: f64,
    /// Ganho do alargamento do lobo pela curvatura dentro da célula (Kaplanyan; `1` = o de W1c).
    pub antisserrilhado: f64,
    /// O mesmo ganho no reflexo do AMBIENTE. `0` (W2 E3-B): o pré-filtrado já É um lobo filtrado (o mapa não
    /// serrilha como o pico analítico da lâmpada); alargá-lo de novo pela curvatura deixava o brilho do
    /// Estúdio em p99 0,26 × 0,48 do Cycles — a `0`: 0,53 × 0,48, área > 0,3 2,12 % × 2,13 %
    /// (`oraculo/README.md` §E3). `1` = o de W1c.
    pub antisserrilhado_ambiente: f64,
    /// K10: o peso do REBOTE de um salto da tinta vizinha no papel ([`rebote_no_papel`]); `0` = sem (a sombra no
    /// papel de antes, bit a bit), `1` = a física (`oraculo/README.md` §K10).
    pub rebote_papel: f64,
    /// W3b: o ganho do dente do papel na altura FINA da tinta com o Relevo do papel 1 (`oraculo/README.md`
    /// §Transmission, calibrado contra a tela de antes de `ca26a7738a`).
    pub papel_na_tinta: f64,
    /// Paper Press (dono 28/10): o ganho do dente do papel na ALTURA BASE e na SOMBRA da tinta com o Paper Press 0
    /// (`medida_paper_press_calibra`, Dabber, Cold Relief 1, o dente no miolo da tinta grossa ÷ o do papel nu, em
    /// níveis: 0,23 · 0,44 · 0,65 · 0,86 · 1,04 a 0,5 · 1 · 1,5 · 2 · 2,5 — linear). `2,4` = o papel mexe na tinta
    /// tantos níveis como no papel nu (o que o olho compara; o RELATIVO ÷ a luminância média daria 0,85, e na foto
    /// `J6_paper_press.jpg` com 0,85 o dente quase não se via sob a tinta azul).
    pub papel_na_base: f64,
}

impl Default for Afinacao {
    fn default() -> Self {
        Self {
            // Régua M6 (`regua_sombra.py`, faca vertical H3, 42°): 0,0015 + penumbra 0,6 dá 53·35·6 → com
            // raio 3: 42·18·0,8 contra o Rebelle 51·12·3; 0,002 já salta para 66·55·32 (a parede passa de
            // 4,5 a 6 px). O 0,006 de W0 (sombra ÷ tan) era a altura da CÚPULA do Rebelle, não do planalto.
            px_por_massa: 0.0015,
            raio_suave: 3,
            raio_sombra: 1,
            raio_detalhe: 1,
            grao: 0.15,
            fios: 1.0,
            penumbra: 0.6,
            brilho_max: 1.6,
            antisserrilhado: 1.0,
            antisserrilhado_ambiente: 0.0,
            rebote_papel: 1.0,
            // `medida_relevo_do_papel_calibra`: miolo ÷ nu 0,449 · 0,866 a 1 · 2 (linear); 0,67 (a tela de antes de
            // `ca26a7738a`, Dabber de fábrica com o óleo) a 1,5.
            papel_na_tinta: 1.5,
            papel_na_base: 2.4,
        }
    }
}

/// Passos da marcha da sombra, em células. Custo medido em W0(a) (README §2b): K=24·N=8 cabe na RTX a
/// 4K; na CPU paga-se só na região suja.
pub const PASSOS_SOMBRA: usize = 24;
/// As distâncias (células) do horizonte da visibilidade, geométricas até à margem; as fatias (cada uma
/// olha para os dois lados). Contra o Cycles (`oraculo/candidatas_ao.py`, 26/10): 4 fatias = 8 (viés
/// +0,068 × +0,064); 64 células só ajudam o papel ao pé de paredes altas (a pirâmide de máximos da placa).
const HORIZONTE: [f64; 9] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0];
const FATIAS: usize = 4;
/// O tecto do [`OleoReal::alcance_fenda`] = o horizonte de fábrica: além dele a amostra sai da
/// [`MARGEM`] da região suja (a costura no recompor) — mais longe é a pirâmide de máximos da placa
/// (`oraculo/README.md`, recusa «64 células na CPU»).
pub const ALCANCE_FENDA_MAX: f64 = HORIZONTE[HORIZONTE.len() - 1];
const _: () = assert!(ALCANCE_FENDA_MAX < MARGEM as f64);
/// O tecto do [`OleoReal::fenda`], MEDIDO na cena do dono (`oraculo/README.md` §E1): o último valor em que
/// a fenda vermelha não tem pixel com `max(R,G,B) < 70` (os pontos pretos da lei M2).
pub const FENDA_MAX: f64 = 3.0;
/// A [`OleoReal::forca_ambiente`] de fábrica: a mistura céu × lâmpada de antes da W2 (o Clássico bit a bit).
pub const FORCA_AMBIENTE_FABRICA: f64 = 0.6;
/// A margem (células) que o render de uma região precisa à volta dela.
pub const MARGEM: usize = PASSOS_SOMBRA + 1;
/// O ALCANCE (células) de uma mudança de tinta: a célula `c` muda a forma até `2·raio_suave` à volta, e a
/// cavidade/sombra de quem a amostra até [`MARGEM`] (+1 do Sobel) — quanto a região suja tem de crescer.
#[must_use]
pub fn alcance(o: &OleoReal) -> usize {
    // A célula pode ter raio próprio até ao tecto: o alcance do cartão é o do tecto.
    let sss = if o.material.subsuperficie > 0.0 {
        oleo_material::alcance_subsuperficie(f64::INFINITY, 1)
    } else {
        0
    };
    MARGEM + 2 * o.af.raio_suave + 1 + sss + alcance_do_suave(o.suave_px)
}

/// Quantas células o Smooth Edges de raio `r` (células; px é um tecto, `ratio ≥ 1`) espalha uma mudança: a tenda.
#[must_use]
fn alcance_do_suave(r: f64) -> usize {
    if r > 0.0 {
        exato::ceil(r) as usize + 1
    } else {
        0
    }
}

/// O [`alcance`] de uma mudança na região `(x0, y0, x1, y1)` também pela Subsurface das CÉLULAS (K4): a célula `q`
/// lê a irradiância até ao alcance do raio DELA, por isso com Subsurface de pincel (cartão a 0) a região cresce pelo
/// maior raio de quem a pode ler. O do cartão já é o do tecto.
#[must_use]
pub fn alcance_na(
    o: &OleoReal,
    camadas: &[(&Grid, bool)],
    ratio: usize,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
) -> usize {
    let a = alcance(o);
    if o.material.subsuperficie > 0.0 {
        return a;
    }
    let (gw, gh) = (camadas[0].0.w, camadas[0].0.h);
    let m = a + oleo_material::alcance_subsuperficie(f64::INFINITY, ratio);
    let janela = (
        x0.saturating_sub(m).max(1),
        y0.saturating_sub(m).max(1),
        (x1 + m).min(gw),
        (y1 + m).min(gh),
    );
    oleo_material::raio_subsuperficie_max(camadas, o, ratio, janela)
        .map_or(a, |r| a + oleo_material::alcance_subsuperficie(r, ratio))
}

/// O alargamento máximo do lobo (α²) pelo antisserrilhado — o tecto de Kaplanyan 2016 (`0,18`).
const VARIANCIA_MAX_BRILHO: f64 = 0.18;

/// `F0` do ligante (dieléctrico).
const F0: f64 = 0.04;

/// A altura da tinta (px de relevo ÷ px por célula) numa janela de células, de TODAS as camadas
/// visíveis, pela ESPESSURA ([`crate::grid::espessura`]) pesada pela cobertura, em três escalas: a FORMA
/// (raio [`Afinacao::raio_suave`]) e o DETALHE (grão e fios) dão a normal; a altura da SOMBRA (raio
/// [`Afinacao::raio_sombra`]) faz a marcha da luz e a cavidade. Com o detalhe na marcha a tinta inteira
/// lia-se como fenda (folha de 26/10: oclusão em «sal e pimenta», mediana 0,24).
pub struct CampoAltura {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    v: Vec<f32>,
    detalhe: Vec<f32>,
    sombra: Vec<f32>,
    /// O material que se VÊ em cada célula (W2 E3-B): o de cada camada visível por cima do de baixo, pela
    /// cobertura dela (`Grid::material_da_celula`). Vazio quando nenhuma camada tem material = tudo NEUTRO.
    material: Vec<MaterialCelula>,
    /// W3 M1: a luz que a Subsurface traz da vizinhança, por canal (`oleo_material::espalha`). Vazio sem ela.
    sss: Vec<[f32; 3]>,
    /// K4: a luz directa e o céu de cada célula da janela da irradiância (as peças que a Subsurface já calculou),
    /// para o sombreado da MESMA célula não as refazer — com a [`ChaveLuz`] da óptica que as fez.
    luz_ceu: Vec<LuzCeu>,
    luz_ceu_janela: (usize, usize, usize, usize),
    luz_ceu_chave: Option<ChaveLuz>,
    /// K10: a cor LINEAR que cada célula devolve ao papel (a tinta sem luz sobre o papel branco, pela cobertura),
    /// a do [`rebote_no_papel`]. Vazio sem rebote.
    albedo: Vec<[f32; 3]>,
    /// §BUG-bordas: a fracção da célula DENTRO do contorno sub-célula ([`borda_suave`]); `1` fora da orla e vazio
    /// sem orla — a cobertura da tinta e a altura das três escalas vão com ela.
    borda: Vec<f32>,
    /// O Gloss da tinta que se vê em cada célula (dono 28/10, `Grid::oleo`); vazio = `1` em toda a parte.
    brilho: Vec<f32>,
    /// A fracção da tinta que se vê com o Real Oil (dono 28/10, `Grid::oleo`); vazio = `1` (toda a tinta é óleo).
    oleo: Vec<f32>,
}

/// A luz directa, a visibilidade BRUTA (antes da força da fenda) e o ganho do céu de uma célula.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LuzCeu {
    pub luz: f64,
    pub v: f64,
    pub dir: Option<[f64; 3]>,
}

/// O que a [`LuzCeu`] lê da óptica (além do campo): com outra chave a célula volta a ser calculada.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChaveLuz {
    azimute: f64,
    altitude: f64,
    sombra: f64,
    penumbra: f64,
    ambiente: Ambiente,
    alcance_fenda: f64,
}

impl ChaveLuz {
    fn de(o: &OleoReal) -> Self {
        Self {
            azimute: o.azimute,
            altitude: o.altitude,
            sombra: o.sombra,
            penumbra: o.af.penumbra,
            ambiente: o.ambiente,
            alcance_fenda: o.alcance_fenda,
        }
    }
}

/// A [`LuzCeu`] da célula `(x, y)` contada de raiz.
pub(crate) fn luz_ceu(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize) -> LuzCeu {
    let n = normal(campo, x, y);
    let luz = luz_directa(o, campo, x, y, campo.at_sombra(x, y));
    let ceu = CeuDoMapa::de(o);
    let (v, dir) = visibilidade_e_ceu(campo, x, y, n, escala_do_horizonte(o), ceu.as_ref());
    LuzCeu { luz, v, dir }
}

/// Quem corre as LINHAS de uma passagem do [`CampoAltura`]: `por_linhas(w, a, b, f)` chama `f(y, a_y, b_y)`
/// para cada linha `y` de largura `w` de `a` e de `b` (o mesmo comprimento), em qualquer ordem e em qualquer
/// fio — cada linha só escreve a dela e lê o que a passagem anterior deixou, por isso o campo é o MESMO bit a bit
/// ([`EmSerie`] aqui; o composite reparte-as pelo rayon, ADR-0109).
pub trait Reparte {
    fn por_linhas<A: Send, B: Send>(
        &self,
        w: usize,
        a: &mut [A],
        b: &mut [B],
        f: &(dyn Fn(usize, &mut [A], &mut [B]) + Sync),
    );
}

/// As linhas uma a uma, por ordem.
pub struct EmSerie;

impl Reparte for EmSerie {
    fn por_linhas<A: Send, B: Send>(
        &self,
        w: usize,
        a: &mut [A],
        b: &mut [B],
        f: &(dyn Fn(usize, &mut [A], &mut [B]) + Sync),
    ) {
        for (y, (la, lb)) in a.chunks_mut(w).zip(b.chunks_mut(w)).enumerate() {
            f(y, la, lb);
        }
    }
}

/// Uma passagem de UM plano: `f(y, linha)`.
fn por_linha<T: Send>(
    rep: &impl Reparte,
    w: usize,
    v: &mut [T],
    f: &(dyn Fn(usize, &mut [T]) + Sync),
) {
    let mut nada = vec![(); v.len()];
    rep.por_linhas(w, v, &mut nada, &|y, l, _| f(y, l));
}

/// Um plano `f32` de `w×h` célula a célula, `f(k)`.
fn plano_f32(
    rep: &impl Reparte,
    w: usize,
    h: usize,
    f: &(dyn Fn(usize) -> f32 + Sync),
) -> Vec<f32> {
    let mut v = vec![0f32; w * h];
    por_linha(rep, w, &mut v, &|y, l| {
        for (x, q) in l.iter_mut().enumerate() {
            *q = f(x + y * w);
        }
    });
    v
}

/// O ganho da altura de óleo da célula `i` de `lg` (dono 28/10: `Depth × Real Oil` da tinta). `todo_oleo` = nenhuma
/// camada tem o campo: a chamada de antes (réguas, testes do motor), em que toda a tinta é óleo de Depth `1`.
#[inline]
fn ganho_da_celula(lg: &Grid, i: usize, todo_oleo: bool) -> f64 {
    if todo_oleo {
        1.0
    } else {
        crate::grid::oleo_celula::ganho(&lg.oleo, i)
    }
}

/// As operações EXACTAS do IEEE 754 (arredondar, chão, tecto e a raiz, correctamente arredondada): o mesmo bit em
/// todo o SO pela própria norma — a lei do porte só pede o `libm` para as TRANSCENDENTES. As do `libm` são em
/// software e eram ~20 % do tempo do óleo por quadro (perf, J6: `round` 11 %, `from_parts` 7 %).
pub(crate) mod exato {
    #[inline(always)]
    pub(crate) fn round(v: f64) -> f64 {
        v.round()
    }
    #[inline(always)]
    pub(crate) fn floor(v: f64) -> f64 {
        v.floor()
    }
    #[inline(always)]
    pub(crate) fn ceil(v: f64) -> f64 {
        v.ceil()
    }
    #[inline(always)]
    pub(crate) fn sqrt(v: f64) -> f64 {
        v.sqrt()
    }
}

/// O fim da [`sombra_no_papel`]: `luz` directa, a visibilidade `vis` já com a força da fenda, e o `dir` do céu.
fn sombra_de(o: &OleoReal, luz: f64, vis: f64, dir: Option<[f64; 3]>) -> f64 {
    let directa = (1.0 - o.forca_ambiente) * (1.0 - luz);
    let tapado = if o.sombra <= 0.0 {
        0.0
    } else {
        // Com HDRI o papel recebe o céu do SECTOR que vê: ao pé da parede que tapa a janela, a sombra macia
        // vem do lado dela (a média dos canais: o plano da sombra é um só).
        let recebe = dir.map_or(vis, |d| vis * (d[0] + d[1] + d[2]) / 3.0);
        o.sombra * o.forca_ambiente * (1.0 - recebe)
    };
    (directa + tapado).clamp(0.0, 1.0)
}

/// O ambiente roda-se em torno de z para o lado claro dele ficar no azimute da luz; `d` na moldura da
/// tela (y para BAIXO) vai para a do mapa (a do Blender, y para cima).
fn para_o_mapa(o: &OleoReal, claro: f64, d: [f64; 3]) -> [f64; 3] {
    let l = para_a_luz(o);
    let rot = claro - libm::atan2(-l[1], l[0]);
    let (c, s) = (libm::cos(rot), libm::sin(rot));
    let b = [d[0], -d[1], d[2]];
    [b[0] * c - b[1] * s, b[0] * s + b[1] * c, b[2]]
}

/// O factor das distâncias de [`HORIZONTE`] pelo [`OleoReal::alcance_fenda`] (`1` no de fábrica).
fn escala_do_horizonte(o: &OleoReal) -> f64 {
    o.alcance_fenda.clamp(1.0, ALCANCE_FENDA_MAX) / ALCANCE_FENDA_MAX
}

/// Os [`Termos`] da célula `(cx, cy)` com a célula de material NEUTRO (o do cartão vale) e a tinta cinza
/// `0,5` (a que o metal e o tint reflectem).
pub fn termos(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> Termos {
    termos_com(o, campo, cx, cy, &MaterialCelula::NEUTRO, [0.5; 3])
}

/// Os [`Termos`] da célula `(cx, cy)` com o material `mat` dela e a cor LINEAR `a` (o F0 do metal e o tint).
pub fn termos_com(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    mat: &MaterialCelula,
    a: [f64; 3],
) -> Termos {
    let af = &o.af;
    let m = crate::material::efectivo(mat, &o.material);
    // O Gloss: o da tinta da célula (dono 28/10) sobre o ganho do motor (`1` no produto).
    let brilho = o.brilho * campo.brilho_em(cx, cy);
    let (x, y) = (cx as isize, cy as isize);
    let n = normal(campo, x, y);
    let l = para_a_luz(o);
    let escala = escala_do_horizonte(o);
    // W2 E2: com HDRI o DIFUSO do céu é a irradiância do SECTOR que cada fatia vê (nunca a do HDRI × a
    // visibilidade média: a janela do estúdio entrava nas fendas que as paredes tapam, 1,09 × 0,55).
    let LuzCeu {
        luz,
        v: v_bruta,
        dir,
    } = campo
        .luz_ceu_em(o, x, y)
        .unwrap_or_else(|| luz_ceu(o, campo, x, y));
    let vis = forca_da_fenda(v_bruta, o.fenda);
    let forca = o.forca_ambiente;

    // Difuso da lâmpada relativo: plano iluminado = 1 − força do ambiente.
    let ndl = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
    let sol = (1.0 - forca) * (ndl / l[2]) * luz;

    // Brilho da lâmpada: GGX + Schlick, vista ortográfica (0,0,1), só onde a luz directa chega.
    let hv = [l[0], l[1], l[2] + 1.0];
    let hn = exato::sqrt(hv[0] * hv[0] + hv[1] * hv[1] + hv[2] * hv[2]);
    let ndh = ((n[0] * hv[0] + n[1] * hv[1] + n[2] * hv[2]) / hn).max(0.0);
    // Antisserrilhado do brilho (Kaplanyan 2016): o lobo alarga-se pela variação da normal dentro da célula
    // (a hessiana da altura fina) — numa encosta estreita o pico de GGX saturava em pontos brancos.
    let hf = |dx: isize, dy: isize| campo.at_fina(x + dx, y + dy);
    let hxx = hf(1, 0) - 2.0 * hf(0, 0) + hf(-1, 0);
    let hyy = hf(0, 1) - 2.0 * hf(0, 0) + hf(0, -1);
    let hxy = (hf(1, 1) - hf(1, -1) - hf(-1, 1) + hf(-1, -1)) / 4.0;
    let var = 0.25 * (hxx * hxx + hyy * hyy + 2.0 * hxy * hxy) * n[2] * n[2];
    // A Roughness ABSOLUTA do material (célula ∘ cartão; no neutro a `RUGOSIDADE_PADRAO`, bit a bit).
    let aa = (2.0 * var * af.antisserrilhado).min(VARIANCIA_MAX_BRILHO);
    let aa_amb = (2.0 * var * af.antisserrilhado_ambiente).min(VARIANCIA_MAX_BRILHO);
    let rugo = m.rugosidade;
    let a2 = rugo * rugo * rugo * rugo + aa;
    let ggx = |a2: f64| {
        let den = ndh * ndh * (a2 - 1.0) + 1.0;
        a2 / (core::f64::consts::PI * den * den)
    };
    let d_ggx = ggx(a2);
    let vdh = hv[2] / hn;
    let p5 = libm::pow(1.0 - vdh, 5.0);
    // O F0 dieléctrico: IOR Level `0,5` = IOR 1,5 (`F0 = 0,08·s`; o F90 é branco).
    let f0d = F0 * (m.nivel_ior / 0.5);
    // IOR Level abaixo de `0,5` apaga também a BORDA (o Principled a `0` não reflecte nada: IOR 1).
    let borda = (m.nivel_ior / 0.5).min(1.0);
    // O brilho da lâmpada vai com a parte dela na mistura (`1` na de fábrica, bit a bit).
    let mistura = (1.0 - forca) / (1.0 - FORCA_AMBIENTE_FABRICA);
    let lampada_de = |f0: f64, borda: f64, d: f64| {
        let fres = f0 + (1.0 - f0) * p5 * borda;
        brilho * af.brilho_max * fres * d * ndl * luz * mistura
    };
    let lampada = [lampada_de(f0d, borda, d_ggx); 3];

    // Brilho do ambiente (split-sum: pré-filtrado × F0·A + B, Karis 2014), tapado pela visibilidade. Sem mapa
    // (Clássico) o metal e o verniz reflectem o céu uniforme.
    let mapa = o.ambiente.mapa();
    // `(pré-filtrado por canal, A, B, fracção livre)` do lobo de rugosidade `a2` com a normal `nr`.
    let ambiente_de = |a2: f64, nr: [f64; 3]| {
        let r = [
            2.0 * nr[2] * nr[0],
            2.0 * nr[2] * nr[1],
            2.0 * nr[2] * nr[2] - 1.0,
        ];
        let rug = exato::sqrt(exato::sqrt(a2));
        let nv = nr[2].max(1e-4);
        let r0 = -rug + 1.0;
        let a004 = (r0 * r0).min(libm::exp2(-9.28 * nv)) * r0 + (-0.0275 * rug + 0.0425);
        let xa = a004 * -1.04 + (-0.572 * rug + 1.04);
        let yb = a004 * 1.04 + (0.022 * rug - 0.04);
        let pre = mapa.map_or([1.0; 3], |mp| mp.brilho(para_o_mapa(o, mp.claro, r), rug));
        let livre = reflexo_livre(campo, x, y, r, rug, escala);
        (pre, xa, yb, livre)
    };
    let a2_amb = rugo * rugo * rugo * rugo + aa_amb;
    let base = (mapa.is_some() || m.metal > 0.0).then(|| ambiente_de(a2_amb, n));
    let mut espec = match (mapa, base) {
        (Some(_), Some((pre, xa, yb, livre))) => [0, 1, 2].map(|c| {
            let ab = f0d * xa + yb * borda;
            lampada[c] + brilho * forca * pre[c] * ab * livre
        }),
        _ => lampada,
    };
    // METAL: F0 = a cor da tinta, sem difuso. O que a parede vizinha TAPA no reflexo não é
    // preto: é a tinta dela (a mesma tinta: o difuso dela, ou o que ela reflecte com o F0 dela se é metal,
    // `a·difuso·mix(1, a, metal)`). O lobo da lâmpada do metal alarga-se como o do AMBIENTE (o alargamento
    // pela curvatura do dieléctrico punha o sol em toda a chapa lisa: R 0,18 × 0,05 do Cycles) — §E3.
    if let (true, Some((pre, xa, yb, livre))) = (m.metal > 0.0, base) {
        let sem_brilho = Termos {
            vis,
            luz,
            sol,
            espec: [0.0; 3],
            dir,
            metal: 0.0,
            sob_verniz: 1.0,
            sss: [0.0; 3],
            transmissao: 0.0,
            vidro_claro: 0.0,
        };
        let d_metal = ggx(a2_amb);
        for c in 0..3 {
            let f0m = a[c].clamp(0.0, 1.0);
            let ab = f0m * xa + yb;
            let viz = a[c] * difuso(o, &sem_brilho, a[c], c) * (1.0 + (a[c] - 1.0) * m.metal);
            let em = lampada_de(f0m, 1.0, d_metal)
                + brilho * (forca * pre[c] * livre + (1.0 - livre) * viz) * ab;
            espec[c] = (1.0 - m.metal) * espec[c] + m.metal * em;
        }
    }
    // VERNIZ: um dieléctrico liso por cima (IOR 1,5), de rugosidade própria; a camada de baixo recebe o que
    // o Fresnel dele deixa passar.
    let mut sob_verniz = 1.0;
    if m.verniz > 0.0 {
        let rc = m.verniz_rugosidade.clamp(0.0, 1.0);
        let a2c = rc * rc * rc * rc + aa;
        let (pre, xa, yb, livre) = ambiente_de(rc * rc * rc * rc + aa_amb, n);
        let fv = F0 + (1.0 - F0) * libm::pow(1.0 - n[2].clamp(0.0, 1.0), 5.0);
        sob_verniz = 1.0 - m.verniz * fv;
        let lv = lampada_de(F0, 1.0, ggx(a2c));
        for c in 0..3 {
            let ev = lv + brilho * forca * pre[c] * (F0 * xa + yb) * livre;
            espec[c] = espec[c] * sob_verniz + m.verniz * ev;
        }
    }
    let mut t = Termos {
        vis,
        luz,
        sol,
        espec,
        dir,
        metal: m.metal,
        sob_verniz,
        sss: if m.subsuperficie > 0.0 {
            campo
                .subsuperficie_em(cx, cy)
                .map(|s| m.subsuperficie.min(1.0) * s)
        } else {
            [0.0; 3]
        },
        transmissao: m.transmissao,
        vidro_claro: m.vidro_claro,
    };
    // VELUDO: o sheen branco, mais forte onde a normal foge da vista, à luz que a célula recebe.
    if m.veludo > 0.0 {
        for c in 0..3 {
            let e = difuso(o, &t, 1.0, c);
            t.espec[c] += oleo_material::veludo(m.veludo, n[2], e) * sob_verniz;
        }
    }
    t
}

#[cfg(test)]
#[path = "oleo_real_tests.rs"]
mod oleo_real_tests;

#[cfg(test)]
#[path = "oleo_borda_w3b_tests.rs"]
mod oleo_borda_w3b_tests;
