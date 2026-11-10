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
pub use oleo_campo::RascunhoCampo;
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
#[path = "oleo_termos.rs"]
mod oleo_termos;
pub use oleo_termos::*;

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
    /// B14.2 (decisão 2 do dono; não é knob: a óptica põe-na por imagem): a luz [`QualidadeDaLuz::Simples`] com o
    /// pincel na tela, a [`QualidadeDaLuz::Cheia`] no repouso — a de fábrica, ao bit.
    pub qualidade: QualidadeDaLuz,
}

/// Quantas amostras a luz do Óleo Real paga por célula (B14.2): as fatias do horizonte, as distâncias dele e os passos
/// da marcha da sombra. A `Simples` corta-as a metade DENTRO do mesmo alcance ([`MARGEM`] e [`ALCANCE_FENDA_MAX`] não
/// mudam: a região suja e a costura são as mesmas).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum QualidadeDaLuz {
    #[default]
    Cheia,
    Simples,
}

impl QualidadeDaLuz {
    /// As fatias do horizonte ([`FATIAS`] ou metade; `FATIAS` divide-se por elas).
    #[must_use]
    pub const fn fatias(self) -> usize {
        match self {
            Self::Cheia => FATIAS,
            Self::Simples => FATIAS / 2,
        }
    }

    /// As distâncias do horizonte: as de [`HORIZONTE`], ou uma sim, uma não (a primeira e a última ficam).
    #[must_use]
    pub const fn horizonte(self) -> &'static [f64] {
        match self {
            Self::Cheia => &HORIZONTE,
            Self::Simples => &HORIZONTE_SIMPLES,
        }
    }

    /// As distâncias `t` (células) da marcha da sombra: `1..=`[`PASSOS_SOMBRA`], ou uma sim, uma não.
    pub fn marcha(self) -> core::iter::StepBy<core::ops::RangeInclusive<usize>> {
        (1..=PASSOS_SOMBRA).step_by(match self {
            Self::Cheia => 1,
            Self::Simples => 2,
        })
    }
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
            qualidade: QualidadeDaLuz::Cheia,
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
    /// Ganho do brilho da lâmpada sobre o microfacet FÍSICO (`1` = o Principled do Cycles; a rugosidade é a
    /// Roughness do material, [`MaterialCartao::rugosidade`]).
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
    /// §Transmission; calibrado contra o Cycles, `oraculo/relevo_papel_cycles.py`).
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
            brilho_max: 1.0,
            antisserrilhado: 1.0,
            antisserrilhado_ambiente: 0.0,
            rebote_papel: 1.0,
            // Cura 02/11c: o Cycles com a tinta assente no dente (`oraculo/relevo_papel_cycles.py`) lê miolo ÷ nu
            // 1,71; o produto (especular microfacet, `e7e49f465b`) 0,256 · 1,396 · 1,788 a 1 · 6 · 8 ⇒ 7,6. Tabela no
            // commit.
            papel_na_tinta: 7.6,
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
/// As de [`HORIZONTE`] da [`QualidadeDaLuz::Simples`]: uma sim, uma não.
const HORIZONTE_SIMPLES: [f64; 5] = [1.0, 3.0, 6.0, 12.0, 24.0];
const _: () = assert!(FATIAS % 2 == 0 && HORIZONTE_SIMPLES[4] == ALCANCE_FENDA_MAX);
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

#[path = "oleo_exato.rs"]
pub(crate) mod exato;

#[cfg(test)]
#[path = "oleo_real_tests.rs"]
mod oleo_real_tests;

#[cfg(test)]
#[path = "oleo_borda_w3b_tests.rs"]
mod oleo_borda_w3b_tests;

#[cfg(test)]
#[path = "oleo_luz_simples_tests.rs"]
mod oleo_luz_simples_tests;
