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
    MaterialCartao, PERFIL_SUBSUPERFICIE, RUGOSIDADE_PADRAO, alcance_subsuperficie,
};

#[path = "oleo_material.rs"]
mod oleo_material;

/// Os knobs do Óleo Real. `1.0` em profundidade, sombra e brilho = o look das capturas `A_base`
/// (Rebelle: impasto 8, SoftShadows 10, Gloss 10, luz 42°).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OleoReal {
    /// Ganho da altura: `1` = a escala medida em W0(b) (≈ 15–17 px de relevo na faca).
    pub profundidade: f64,
    /// De ONDE vem a luz, em radianos, no plano da tela (x para a direita, y para BAIXO): `225°` = topo-esquerda.
    pub azimute: f64,
    /// Altura da luz sobre a tela, em radianos (o Rebelle não desce de 42°; nós sim).
    pub altitude: f64,
    /// Força da sombra projectada, `0..=1` (linear no slider — M6).
    pub sombra: f64,
    /// Força do brilho especular, `0..=1` (M4: só o termo somado; nunca o volume).
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
    MARGEM + 2 * o.af.raio_suave + 1 + sss
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

impl CampoAltura {
    /// A janela `[x0..=x1]×[y0..=y1]` (células, `1..=W`) alargada pela margem e presa à grade.
    pub fn de(
        camadas: &[(&Grid, bool)],
        o: &OleoReal,
        ratio: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
    ) -> Self {
        Self::de_com(&EmSerie, camadas, o, ratio, (x0, y0, x1, y1))
    }

    /// [`CampoAltura::de`] com as linhas de cada passagem repartidas por `rep` — o mesmo campo bit a bit
    /// (gate `o_campo_em_faixas_e_bit_identico_ao_em_serie`).
    pub fn de_com(
        rep: &impl Reparte,
        camadas: &[(&Grid, bool)],
        o: &OleoReal,
        ratio: usize,
        (x0, y0, x1, y1): (usize, usize, usize, usize),
    ) -> Self {
        let g = camadas[0].0;
        let af = &o.af;
        // A janela leva a margem da marcha e as duas passagens do desfoque (e o alcance da Subsurface).
        let sss = oleo_material::raio_subsuperficie_max(camadas, o, ratio, (x0, y0, x1, y1))
            .map(|r| oleo_material::alcance_subsuperficie(r, ratio));
        let m = MARGEM + 2 * af.raio_suave + sss.unwrap_or(0);
        let (ax0, ay0) = (x0.saturating_sub(m).max(1), y0.saturating_sub(m).max(1));
        let (ax1, ay1) = ((x1 + m).min(g.w), (y1 + m).min(g.h));
        let (w, h) = (ax1 + 1 - ax0, ay1 + 1 - ay0);
        let escala = af.px_por_massa * o.profundidade / ratio.max(1) as f64;
        let mut massa = vec![0f64; w * h];
        let mut fios = vec![0f64; w * h];
        let com_material = camadas.iter().any(|&(lg, v)| v && !lg.material.is_empty());
        rep.por_linhas(w, &mut massa, &mut fios, &|y, massa, fios| {
            for x in 0..w {
                let i = (ax0 + x) + (ay0 + y) * g.s;
                let mut cobre = 0.0;
                for &(lg, visivel) in camadas {
                    if !visivel {
                        continue;
                    }
                    let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                    let k = crate::grid::espessura::factor(lg, i);
                    massa[x] += m * k;
                    let alfa = crate::opacity::alpha_of_mass(m);
                    cobre += alfa * (1.0 - cobre);
                    // Os sulcos dos fios fundam-se com a tinta que se empilhou (a amplitude ∝ espessura).
                    if !lg.altura.is_empty() {
                        fios[x] += f64::from(lg.altura[i]) * k;
                    }
                }
                // Na borda a altura vai a zero com a cobertura: a orla fina não é um degrau.
                massa[x] *= cobre;
                fios[x] *= cobre;
            }
        });
        // A CRISTA da faca (`Grid::crista`, K3a) entra na altura da SOMBRA (K6): projecta a banda escura por dentro
        // da borda (`49_realshader/E4_lamina.md` §Rebordo). Só dela — os fios do Oil Bristle ficam na normal.
        let crista = camadas
            .iter()
            .any(|&(lg, v)| v && !lg.crista.is_empty())
            .then(|| {
                let mut crista = vec![0f64; w * h];
                por_linha(rep, w, &mut crista, &|y, linha| {
                    for (x, c) in linha.iter_mut().enumerate() {
                        let i = (ax0 + x) + (ay0 + y) * g.s;
                        let mut cobre = 0.0;
                        for &(lg, visivel) in camadas {
                            if !visivel {
                                continue;
                            }
                            let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                            let k = crate::grid::espessura::factor(lg, i);
                            *c += f64::from(crate::grid::crista::em(lg, i)) * k;
                            cobre += crate::opacity::alpha_of_mass(m) * (1.0 - cobre);
                        }
                        *c *= cobre;
                    }
                });
                crista
            });
        let material = if com_material {
            let mut material = vec![MaterialCelula::NEUTRO; w * h];
            por_linha(rep, w, &mut material, &|y, linha| {
                for (x, q) in linha.iter_mut().enumerate() {
                    let i = (ax0 + x) + (ay0 + y) * g.s;
                    for &(lg, visivel) in camadas {
                        let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                        if visivel && m > 0.0 {
                            let alfa = crate::opacity::alpha_of_mass(m);
                            *q = MaterialCelula::mistura(*q, lg.material_da_celula(i), alfa as f32);
                        }
                    }
                }
            });
            material
        } else {
            Vec::new()
        };
        let d = |v: &[f64], r: usize| desfoca(rep, v, w, h, r);
        let suave = match marca_do_nivelado(rep, camadas, (ax0, ay0, w, h)) {
            Some((mu, peso)) => {
                forma_viva(&massa, (&mu, &peso), &|v| d(&d(v, af.raio_suave), af.raio_suave))
            }
            None => d(&d(&massa, af.raio_suave), af.raio_suave),
        };
        let k_fios = af.fios * o.profundidade;
        let v = plano_f32(rep, w, h, &|k| (suave[k] * escala) as f32);
        let (massa_d, fios_d) = match af.raio_detalhe {
            0 => (massa.clone(), fios),
            r => (d(&massa, r), d(&fios, r)),
        };
        let detalhe = plano_f32(rep, w, h, &|k| {
            (af.grao * (massa_d[k] - suave[k]) * escala + fios_d[k] * k_fios) as f32
        });
        let alta = match af.raio_sombra {
            0 => massa,
            r => d(&d(&massa, r), r),
        };
        let sombra = plano_f32(rep, w, h, &|k| {
            let s = (alta[k] * escala) as f32;
            match &crista {
                Some(c) => s + (c[k] * k_fios) as f32,
                None => s,
            }
        });
        let albedo = if af.rebote_papel > 0.0 && o.sombra > 0.0 {
            let mut albedo = vec![[1.0f32; 3]; w * h];
            por_linha(rep, w, &mut albedo, &|y, linha| {
                for (x, q) in linha.iter_mut().enumerate() {
                    *q = albedo_da_celula(camadas, (ax0 + x) + (ay0 + y) * g.s);
                }
            });
            albedo
        } else {
            Vec::new()
        };
        let mut campo = Self {
            x0: ax0,
            y0: ay0,
            w,
            h,
            v,
            detalhe,
            sombra,
            material,
            sss: Vec::new(),
            luz_ceu: Vec::new(),
            luz_ceu_janela: (0, 0, 0, 0),
            luz_ceu_chave: None,
            albedo,
        };
        if let Some(rr) = sss {
            let (sss, luz_ceu, janela) =
                oleo_material::espalha(rep, &campo, o, (ratio, rr), (x0, y0, x1, y1), (g.w, g.h));
            campo.sss = sss;
            campo.luz_ceu = luz_ceu;
            campo.luz_ceu_janela = janela;
            campo.luz_ceu_chave = Some(ChaveLuz::de(o));
        }
        campo
    }

    /// O que a Subsurface soma ao difuso de cada canal na célula `(x, y)`, antes do peso (`0` sem ela).
    #[inline]
    #[must_use]
    pub fn subsuperficie_em(&self, x: usize, y: usize) -> [f64; 3] {
        if self.sss.is_empty() {
            return [0.0; 3];
        }
        self.sss[self.idx(x as isize, y as isize)].map(f64::from)
    }

    /// A [`LuzCeu`] guardada da célula `(x, y)`, se a Subsurface a calculou com a mesma óptica `o`.
    #[inline]
    fn luz_ceu_em(&self, o: &OleoReal, x: isize, y: isize) -> Option<LuzCeu> {
        let (jx, jy, jw, jh) = self.luz_ceu_janela;
        let (dx, dy) = (x - jx as isize, y - jy as isize);
        if self.luz_ceu.is_empty()
            || dx < 0
            || dy < 0
            || dx >= jw as isize
            || dy >= jh as isize
            || self.luz_ceu_chave != Some(ChaveLuz::de(o))
        {
            return None;
        }
        Some(self.luz_ceu[dx as usize + dy as usize * jw])
    }

    /// Larga a luz e o céu guardados (o sombreado volta a contá-los célula a célula; a MESMA imagem — gate
    /// `a_luz_do_ceu_guardada_e_a_de_sempre`). Para medir o que eles poupam.
    pub fn esquece_luz_do_ceu(&mut self) {
        self.luz_ceu = Vec::new();
        self.luz_ceu_chave = None;
    }

    /// O material que se vê na célula `(x, y)` ([`MaterialCelula::NEUTRO`] sem material em camada nenhuma).
    #[inline]
    #[must_use]
    pub fn material_em(&self, x: usize, y: usize) -> MaterialCelula {
        if self.material.is_empty() {
            return MaterialCelula::NEUTRO;
        }
        self.material[self.idx(x as isize, y as isize)]
    }

    /// A altura da SOMBRA na célula `(x, y)` (presa à janela): a da marcha da luz.
    #[inline]
    pub fn at_sombra(&self, x: isize, y: isize) -> f64 {
        f64::from(self.sombra[self.idx(x, y)])
    }

    /// A altura da SOMBRA num ponto qualquer, bilinear: a marcha diagonal pela célula mais próxima
    /// desenhava em escada a borda de cada estria (o pente da folha W1b).
    #[inline]
    fn at_sombra_em(&self, x: f64, y: f64) -> f64 {
        let (fx, fy) = (libm::floor(x), libm::floor(y));
        let (tx, ty) = (x - fx, y - fy);
        let (ix, iy) = (fx as isize, fy as isize);
        let l0 =
            self.at_sombra(ix, iy) + (self.at_sombra(ix + 1, iy) - self.at_sombra(ix, iy)) * tx;
        let l1 = self.at_sombra(ix, iy + 1)
            + (self.at_sombra(ix + 1, iy + 1) - self.at_sombra(ix, iy + 1)) * tx;
        l0 + (l1 - l0) * ty
    }

    #[inline]
    fn idx(&self, x: isize, y: isize) -> usize {
        let x = (x - self.x0 as isize).clamp(0, self.w as isize - 1) as usize;
        let y = (y - self.y0 as isize).clamp(0, self.h as isize - 1) as usize;
        x + y * self.w
    }

    /// A altura da FORMA na célula `(x, y)` (presa à janela).
    #[inline]
    pub fn at(&self, x: isize, y: isize) -> f64 {
        f64::from(self.v[self.idx(x, y)])
    }

    /// A altura com o DETALHE (a que dá a normal).
    #[inline]
    pub fn at_fina(&self, x: isize, y: isize) -> f64 {
        let i = self.idx(x, y);
        f64::from(self.v[i]) + f64::from(self.detalhe[i])
    }
}

/// A cor LINEAR da célula `i` sobre o papel branco: as camadas visíveis por cima umas das outras pela cobertura
/// (a lei [`crate::assenta::Ecra::Porto`]; o rebote só precisa do tom da parede).
fn albedo_da_celula(camadas: &[(&Grid, bool)], i: usize) -> [f32; 3] {
    let (mut p, mut pa) = ([0.0f64; 3], 0.0f64);
    for &(lg, visivel) in camadas {
        if !visivel {
            continue;
        }
        let a_s = crate::opacity::alpha_of_mass(f64::from(lg.sett[i]));
        let a_f = crate::opacity::alpha_of_mass(f64::from(lg.susp[i]));
        if a_s + a_f * (1.0 - a_s) <= 0.0 {
            continue;
        }
        let (c, la) = crate::assenta::cor_e_alfa(
            crate::assenta::Ecra::Porto.com_vela(lg.vela[i]),
            lg.sett_rgb[i].map(f64::from),
            lg.susp_rgb[i].map(f64::from),
            a_s,
            a_f,
        );
        for k in 0..3 {
            p[k] = p[k] * (1.0 - la) + c[k] * la;
        }
        pa += la * (1.0 - pa);
    }
    if pa <= 0.0 {
        return [1.0; 3];
    }
    p.map(|v| (srgb255_to_linear(v / pa) * pa + (1.0 - pa)) as f32)
}

/// A marca do nivelamento ([`crate::grid::nivelado`], K7) na janela `(ax0, ay0, w, h)` — a maior das camadas
/// visíveis — e o PESO dela, `marca × cobertura`. `None` quando nenhuma a tem: a forma de antes, ao bit.
fn marca_do_nivelado(
    rep: &impl Reparte,
    camadas: &[(&Grid, bool)],
    (ax0, ay0, w, h): (usize, usize, usize, usize),
) -> Option<(Vec<f64>, Vec<f64>)> {
    if !camadas.iter().any(|&(lg, v)| v && !lg.nivelado.is_empty()) {
        return None;
    }
    let s = camadas[0].0.s;
    let (mut mu, mut peso) = (vec![0f64; w * h], vec![0f64; w * h]);
    rep.por_linhas(w, &mut mu, &mut peso, &|y, mu, peso| {
        for x in 0..w {
            let i = (ax0 + x) + (ay0 + y) * s;
            let mut cobre = 0.0;
            for &(lg, visivel) in camadas {
                if visivel {
                    mu[x] = mu[x].max(f64::from(crate::grid::nivelado::em(lg, i)));
                    let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                    cobre += crate::opacity::alpha_of_mass(m) * (1.0 - cobre);
                }
            }
            peso[x] = mu[x] * cobre;
        }
    });
    Some((mu, peso))
}

/// ⭐ **A parede nivelada fica viva** (W3 K7, `49_realshader/oraculo/README.md` §K7): a massa (já `× cobertura`)
/// reparte-se pela marca `mu`. A parte por marcar leva o desfoque largo de sempre (`desfoque`); a marcada é a
/// ESPESSURA média normalizada pelo peso `mu × cobertura` — `desfoque(massa·mu) ÷ desfoque(peso) · peso` —, que alisa
/// o miolo do planalto sem o espalhar para fora dele nem puxar para dentro a orla fina: a parede é a da cobertura
/// (1–2 células, o Rebelle ~2 px) em vez das ~10 do desfoque largo.
fn forma_viva(
    massa: &[f64],
    (mu, peso): (&[f64], &[f64]),
    desfoque: &dyn Fn(&[f64]) -> Vec<f64>,
) -> Vec<f64> {
    let livre: Vec<f64> = massa.iter().zip(mu).map(|(m, u)| m * (1.0 - u)).collect();
    let presa: Vec<f64> = massa.iter().zip(mu).map(|(m, u)| m * u).collect();
    let (livre, presa, suporte) = (desfoque(&livre), desfoque(&presa), desfoque(peso));
    (0..massa.len())
        .map(|k| {
            let viva = if suporte[k] > 1e-9 {
                presa[k] / suporte[k] * peso[k]
            } else {
                0.0
            };
            livre[k] + viva
        })
        .collect()
}

/// Média móvel separável de raio `r` (caixa `2r+1`), presa às bordas; cada passagem por linhas de `rep`.
fn desfoca(rep: &impl Reparte, v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let n = (2 * r + 1) as f64;
    let mut tmp = vec![0f64; w * h];
    por_linha(rep, w, &mut tmp, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += v[(x + k).saturating_sub(r).min(w - 1) + y * w];
            }
            *q = s / n;
        }
    });
    let mut out = vec![0f64; w * h];
    por_linha(rep, w, &mut out, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += tmp[x + (y + k).saturating_sub(r).min(h - 1) * w];
            }
            *q = s / n;
        }
    });
    out
}

/// O vector unitário PARA a luz (x direita, y baixo, z para fora da tela).
fn para_a_luz(o: &OleoReal) -> [f64; 3] {
    let c = libm::cos(o.altitude);
    [
        c * libm::cos(o.azimute),
        c * libm::sin(o.azimute),
        libm::sin(o.altitude),
    ]
}

/// Os termos da óptica numa célula, antes da cor: a visibilidade do céu `0..1` (GTAO pelo cosseno; plano
/// aberto = 1), a luz directa que chega `0..1` (1 − sombra), o difuso da lâmpada (plano iluminado =
/// `1 − céu`) e o brilho somado por canal (linear; o do AMBIENTE tem a cor dele).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Termos {
    pub vis: f64,
    pub luz: f64,
    pub sol: f64,
    pub espec: [f64; 3],
    /// O GANHO do céu visível por canal (W2 E2): a irradiância do HDRI no sector que cada fatia vê,
    /// ÷ a visibilidade (céu uniforme = 1), ÷ a do plano aberto (a tinta lisa = a cor dela). `None` no
    /// Clássico (céu uniforme).
    pub dir: Option<[f64; 3]>,
    /// W2 E3-B: o peso do METAL (o difuso vai com `1 − metal`), o que o VERNIZ deixa passar à camada de
    /// baixo (`1` sem verniz).
    pub metal: f64,
    pub sob_verniz: f64,
    /// W3 M1: o difuso que a SUBSURFACE soma a cada canal (já com o peso; `0` sem ela).
    pub sss: [f64; 3],
}

impl Termos {
    /// O difuso relativo de uma tinta cinza `0,5` (o mapa `t_e` da sonda, comparável ao render cinza do
    /// Cycles; plano iluminado = 1), média dos canais.
    #[must_use]
    pub fn e(&self, o: &OleoReal) -> f64 {
        (0..3).map(|c| difuso(o, self, 0.5, c)).sum::<f64>() / 3.0
    }
}

/// O difuso relativo (plano iluminado = 1) do canal `c` de albedo linear `a`: o ambiente pelo sector
/// visível com o multi-ricochete de Jimenez, mais a lâmpada. No Clássico é a lei de antes, bit a bit.
#[must_use]
pub fn difuso(o: &OleoReal, t: &Termos, a: f64, c: usize) -> f64 {
    let ceu = o.forca_ambiente;
    match t.dir {
        None => ceu * multi_ricochete(t.vis, a) + t.sol,
        // O céu visto DIRECTAMENTE tem o ganho do sector; o que as paredes devolvem (`mr − V`) é o céu
        // médio delas (`oraculo/README.md` §E2: o ganho do sector também no rebote e o rebote da lâmpada
        // pioraram a régua).
        Some(d) => ceu * (d[c] * t.vis + (multi_ricochete(t.vis, a) - t.vis)) + t.sol,
    }
}

/// A visibilidade com a FORÇA `f` da fenda: as CHANCES de oclusão `(1−V)/V` multiplicam-se por `f`
/// (`V' = V / (V + f·(1−V))`). `0` = céu inteiro, `1` = `V` (bit a bit), acima escurece o meio-tom e a fenda
/// funda devagar: `V^f` levava a fenda vermelha aos pontos pretos já em `f = 2` (`oraculo/README.md` §E1).
#[must_use]
pub fn forca_da_fenda(v: f64, f: f64) -> f64 {
    if f == 1.0 {
        return v;
    }
    if f <= 0.0 || v >= 1.0 {
        return 1.0;
    }
    v / (v + f * (1.0 - v))
}

/// O multi-ricochete do céu pela cor da tinta (Jimenez 2016): a fenda recebe a luz das paredes da MESMA
/// tinta e fica no tom dela. Devolve a visibilidade CORRIGIDA (substitui `v`; nunca `v × isto`). Contra o Cycles no vermelho (`oraculo/calibra_ao.py`, 26/10), em V 0,78 ·
/// 0,70 · 0,56 · 0,47: ganho 1,16 · 1,23 · 1,36 · 1,47 × esta fórmula 1,17 · 1,25 · 1,43 · 1,58.
#[must_use]
pub fn multi_ricochete(v: f64, albedo: f64) -> f64 {
    let a = 2.0404 * albedo - 0.3324;
    let b = -4.7951 * albedo + 0.6417;
    let c = 2.7552 * albedo + 0.6903;
    v.max(((v * a + b) * v + c) * v)
}

/// O realce enche a FOLGA até 1 em vez de cortar: sem brilho a base fica intacta (neutralidade); o brilho
/// aproxima-se do branco sem degrau. O Cycles com o HDRI de estúdio põe 23 % da tinta acima de 1.
#[must_use]
pub fn rola(base: f64, espec: f64) -> f64 {
    let folga = 1.0 - base;
    if folga <= 1e-9 {
        return base.min(1.0);
    }
    1.0 - folga * libm::exp(-espec.max(0.0) / folga)
}

/// Ilumina a cor `rgb` (sRGB 0..255, sem pré-multiplicar) da célula `(cx, cy)`, de `cobertura` `0..=1`.
/// Devolve sRGB 0..255. O brilho é do filme de tinta: na orla fina ele vai com a cobertura.
pub fn ilumina(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
) -> [f64; 3] {
    ilumina_com(o, campo, cx, cy, rgb, cobertura, &campo.material_em(cx, cy))
}

/// [`ilumina`] com o MATERIAL da célula (W2 E3-B), combinado com o do cartão ([`MaterialCartao`]).
pub fn ilumina_com(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
    mat: &MaterialCelula,
) -> [f64; 3] {
    let a = rgb.map(srgb255_to_linear);
    cor_dos_termos(o, &termos_com(o, campo, cx, cy, mat, a), a, cobertura)
}

/// [`ilumina`] e a [`sombra_no_papel`] da mesma célula pintada, com UMA conta dos termos.
pub fn ilumina_e_sombra(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
) -> ([f64; 3], f64) {
    let a = rgb.map(srgb255_to_linear);
    let t = termos_com(o, campo, cx, cy, &campo.material_em(cx, cy), a);
    (
        cor_dos_termos(o, &t, a, cobertura),
        menos_rebote(o, campo, cx, cy, sombra_dos_termos(o, &t)),
    )
}

/// A cor sRGB `0..255` da tinta de albedo linear `a` e `cobertura` pelos termos `t` da célula.
fn cor_dos_termos(o: &OleoReal, t: &Termos, a: [f64; 3], cobertura: f64) -> [f64; 3] {
    let t = *t;
    let cob = cobertura.clamp(0.0, 1.0);
    let mut out = [0.0; 3];
    for c in 0..3 {
        let mut base = a[c] * difuso(o, &t, a[c], c);
        if t.sss[c] != 0.0 {
            base = a[c] * (difuso(o, &t, a[c], c) + t.sss[c]);
        }
        if t.metal > 0.0 || t.sob_verniz != 1.0 {
            base = base * (1.0 - t.metal) * t.sob_verniz;
        }
        out[c] = srgb255_of_linear(rola(base, t.espec[c] * cob).clamp(0.0, 1.0));
    }
    out
}

/// A luz directa que chega à célula `(x, y)` de altura `h0`, `0..=1`: marcha para a luz, o raio sobe
/// `tan(alt)` por célula; a penumbra é uma faixa de ELEVAÇÃO (um sol com tamanho), por isso alarga-se com
/// a distância ao obstáculo; `sombra` escala-a linearmente (M6).
fn luz_directa(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize, h0: f64) -> f64 {
    if o.sombra <= 0.0 {
        return 1.0;
    }
    let l = para_a_luz(o);
    let lxy = libm::hypot(l[0], l[1]).max(1e-9);
    let (dx, dy) = (l[0] / lxy, l[1] / lxy);
    // O horizonte na direcção da luz (tangente da elevação do obstáculo mais alto visto daqui); a luz
    // apaga-se ao longo de `penumbra` (em tangente) abaixo da elevação dela. Em papel livre o horizonte
    // é 0 e a luz é cheia — a forma `k·folga/t` limitava-a a `k·tan(alt)` também no papel livre.
    let sobe = libm::tan(o.altitude);
    let mut horizonte = f64::NEG_INFINITY;
    for t in 1..=PASSOS_SOMBRA {
        let t = t as f64;
        let q = campo.at_sombra_em(x as f64 + dx * t, y as f64 + dy * t);
        horizonte = horizonte.max((q - h0) / t);
    }
    let luz = ((sobe - horizonte) / o.af.penumbra.max(1e-6)).clamp(0.0, 1.0);
    1.0 - o.sombra * (1.0 - luz)
}

/// A normal da altura FINA na célula `(x, y)` (Sobel 3×3).
fn normal(campo: &CampoAltura, x: isize, y: isize) -> [f64; 3] {
    let h = |dx: isize, dy: isize| campo.at_fina(x + dx, y + dy);
    let fx = (h(1, -1) + 2.0 * h(1, 0) + h(1, 1) - h(-1, -1) - 2.0 * h(-1, 0) - h(-1, 1)) / 8.0;
    let fy = (h(-1, 1) + 2.0 * h(0, 1) + h(1, 1) - h(-1, -1) - 2.0 * h(0, -1) - h(1, -1)) / 8.0;
    let inv = 1.0 / libm::sqrt(fx * fx + fy * fy + 1.0);
    [-fx * inv, -fy * inv, inv]
}

/// A visibilidade do céu pelo cosseno (GTAO, Jimenez 2016, vista ortográfica +z) sobre a altura FINA:
/// [`FATIAS`] fatias, cada uma com o horizonte dos dois lados nas distâncias [`HORIZONTE`], preso à
/// horizontal (a tela tapa o chão). Plano aberto = 1. Contra o Cycles (`oraculo/candidatas_ao.py`):
/// viés p50 +0,03 no Dabber, +0,11 no Oil Bristle (os sulcos dos fios tapam mais do que se vê aqui).
fn visibilidade_e_ceu(
    campo: &CampoAltura,
    x: isize,
    y: isize,
    n: [f64; 3],
    escala: f64,
    ceu: Option<&CeuDoMapa<'_>>,
) -> (f64, Option<[f64; 3]>) {
    use core::f64::consts::{FRAC_PI_2, PI, TAU};
    let h0 = campo.at_fina(x, y);
    let mut acc = 0.0;
    let mut e = [0.0f64; 4];
    for k in 0..FATIAS {
        let phi = PI * k as f64 / FATIAS as f64;
        let (dx, dy) = (libm::cos(phi), libm::sin(phi));
        let mut s = [0.0f64; 2];
        for (j, sgn) in [1.0, -1.0].into_iter().enumerate() {
            for r in HORIZONTE {
                let r = r * escala;
                let (ox, oy) = (libm::round(sgn * dx * r), libm::round(sgn * dy * r));
                let d = libm::hypot(ox, oy);
                if d == 0.0 {
                    continue;
                }
                let dh = campo.at_fina(x + ox as isize, y + oy as isize) - h0;
                s[j] = s[j].max(dh / libm::sqrt(dh * dh + d * d));
            }
        }
        // A normal projectada no plano da fatia: o comprimento pesa a fatia, o ângulo `g` mede-se do zénite.
        let no = -n[0] * dy + n[1] * dx;
        let np = [n[0] + no * dy, n[1] - no * dx, n[2]];
        let len = libm::sqrt(np[0] * np[0] + np[1] * np[1] + np[2] * np[2]);
        if len < 1e-9 {
            continue;
        }
        let lado = np[0] * dx + np[1] * dy;
        let g = libm::acos((np[2] / len).clamp(-1.0, 1.0)).copysign(lado);
        let (t1, t2) = (FRAC_PI_2 - libm::asin(s[0]), FRAC_PI_2 - libm::asin(s[1]));
        let a1 = g + t1.min(FRAC_PI_2 - g);
        let a2 = g - t2.min(FRAC_PI_2 + g);
        let (cg, sg) = (libm::cos(g), libm::sin(g));
        let v = 0.25 * (-libm::cos(2.0 * a1 - g) + cg + 2.0 * a1 * sg)
            + 0.25 * (-libm::cos(2.0 * a2 - g) + cg + 2.0 * a2 * sg);
        acc += len * v;
        // O GANHO do HDRI no arco FÍSICO de cada lado — do zénite até ao horizonte medido, preso ao hemisfério
        // da normal: `∫ L·cos(θ−γ)·sinθ dθ` em forma fechada pelas somas, ÷ o mesmo arco sob céu uniforme
        // (o canal 3). O arco da `v` acima (o de Jimenez, deslocado de `γ` e calibrado contra o Cycles no céu
        // uniforme) deixava a encosta virada para a janela vê-la através da parede: difuso da fenda 1,76 × 0,66.
        if let Some(cm) = ceu {
            let u = cm.u0 + phi / TAU;
            for (a, ud, sinal) in [
                (t1.min(FRAC_PI_2 + g), u, 1.0),
                (t2.min(FRAC_PI_2 - g), u + 0.5, -1.0),
            ] {
                let [c1, c2] = cm.s.ate(ud, a);
                for ch in 0..4 {
                    e[ch] += len * (cg * c1[ch] + sinal * sg * c2[ch]);
                }
            }
        }
    }
    let vis = (acc / FATIAS as f64).clamp(0.0, 1.0);
    let dir = ceu.map(|cm| {
        if e[3] > 1e-9 {
            [0, 1, 2].map(|ch| e[ch] / e[3] / cm.cima[ch].max(1e-9))
        } else {
            [1.0; 3]
        }
    });
    (vis, dir)
}

/// O céu do HDRI visto pelas fatias (W2 E2): as [`Sectores`] do mapa, o azimute do mapa da fatia `φ = 0`
/// e a irradiância do plano ABERTO pelas MESMAS 8 direcções (a tinta lisa sai exactamente com a cor dela).
struct CeuDoMapa<'a> {
    s: &'a Sectores,
    u0: f64,
    cima: [f64; 3],
}

impl<'a> CeuDoMapa<'a> {
    fn novo(o: &OleoReal, claro: f64, s: &'a Sectores) -> Self {
        let u0 = u_do_mapa(o, claro);
        let mut cima = [0.0; 3];
        let lados = (1.0 / CUNHA) as usize;
        for k in 0..lados {
            let [c1, _] = s.ate(u0 + k as f64 * CUNHA, core::f64::consts::FRAC_PI_2);
            for ch in 0..3 {
                cima[ch] += c1[ch] / c1[3] / (2 * FATIAS) as f64;
            }
        }
        Self { s, u0, cima }
    }

    fn de(o: &OleoReal) -> Option<CeuDoMapa<'static>> {
        let m = o.ambiente.mapa()?;
        Some(CeuDoMapa::novo(o, m.claro, m.sectores.as_ref()?))
    }
}

/// O `u` do mapa (fracção da volta, a de `Mapa::amostra`) da direcção da tela de ângulo `0` (x para a
/// direita); a de ângulo `φ` (y para BAIXO) é `u + φ/2π` — o espelho de y e a rotação de [`para_o_mapa`].
fn u_do_mapa(o: &OleoReal, claro: f64) -> f64 {
    let l = para_a_luz(o);
    let rot = claro - libm::atan2(-l[1], l[0]);
    1.0 - rot / core::f64::consts::TAU
}

/// A fracção LIVRE do reflexo do ambiente: o horizonte da altura fina na direcção reflectida `r`
/// (distâncias [`HORIZONTE`]); o lobo de largura `rug` passa de tapado a livre à volta dele. Com a
/// visibilidade média no lugar disto a fenda reflectia o estúdio: luminância 74 × 47 do Cycles.
fn reflexo_livre(
    campo: &CampoAltura,
    x: isize,
    y: isize,
    r: [f64; 3],
    rug: f64,
    escala: f64,
) -> f64 {
    let rxy = libm::hypot(r[0], r[1]);
    if rxy < 1e-6 {
        return 1.0;
    }
    let (dx, dy) = (r[0] / rxy, r[1] / rxy);
    let h0 = campo.at_fina(x, y);
    let mut sen = 0.0f64;
    for d in HORIZONTE {
        let d = d * escala;
        let (ox, oy) = (libm::round(dx * d), libm::round(dy * d));
        let dist = libm::hypot(ox, oy);
        if dist == 0.0 {
            continue;
        }
        let dh = campo.at_fina(x + ox as isize, y + oy as isize) - h0;
        sen = sen.max(dh / libm::sqrt(dh * dh + dist * dist));
    }
    ((r[2] - sen) / rug.max(0.05) + 0.5).clamp(0.0, 1.0)
}

/// **A sombra no PAPEL** (M6: cai também fora da tinta): a fracção `0..1` de escurecimento de uma
/// célula SEM tinta — a luz directa que a tinta à volta lhe tira, `(1 − céu)·(1 − luz)`, e o céu que ela
/// lhe tapa ao pé, `céu·(1 − V)` (o Cycles: visibilidade 0,68 no p5 do papel). O difuso, a fenda e o
/// brilho são da tinta.
pub fn sombra_no_papel(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    let (x, y) = (cx as isize, cy as isize);
    if o.sombra > 0.0
        && let Some(lc) = campo.luz_ceu_em(o, x, y)
    {
        return menos_rebote(
            o,
            campo,
            cx,
            cy,
            sombra_de(o, lc.luz, forca_da_fenda(lc.v, o.fenda), lc.dir),
        );
    }
    let luz = luz_directa(o, campo, x, y, campo.at_sombra(x, y));
    let (vis, dir) = if o.sombra <= 0.0 {
        (1.0, None)
    } else {
        let ceu = CeuDoMapa::de(o);
        let (v, dir) = visibilidade_e_ceu(
            campo,
            x,
            y,
            normal(campo, x, y),
            escala_do_horizonte(o),
            ceu.as_ref(),
        );
        (forca_da_fenda(v, o.fenda), dir)
    };
    menos_rebote(o, campo, cx, cy, sombra_de(o, luz, vis, dir))
}

/// A sombra `s` do papel menos o [`rebote_no_papel`] (sem rebote, `s` tal e qual).
fn menos_rebote(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize, s: f64) -> f64 {
    if campo.albedo.is_empty() || s <= 0.0 {
        return s;
    }
    (s - rebote_no_papel(o, campo, cx, cy)).max(0.0)
}

/// **O rebote de UM salto da tinta vizinha no papel** (K10, `oraculo/README.md` §K10), em fracção da luz do plano
/// aceso. Em cada lado de cada fatia do GTAO (a mesma marcha da visibilidade), pelo cosseno da normal do papel:
/// o arco da horizontal até ao horizonte é a PAREDE, que devolve a cor dela × a luz da FACE virada para o papel
/// (o céu de uma face vertical, `½`, e o sol que lhe bate, `0` do lado escuro); o arco abaixo da horizontal (o
/// papel inclinado) é o CHÃO vizinho, aceso pela normal dele. A sombra projectada atrás de um empaste alto
/// continua escura: a face que ela vê é a de costas para a luz.
#[must_use]
pub fn rebote_no_papel(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    use core::f64::consts::{FRAC_PI_2, PI};
    if campo.albedo.is_empty() || o.sombra <= 0.0 {
        return 0.0;
    }
    let (x, y) = (cx as isize, cy as isize);
    let h0 = campo.at_fina(x, y);
    let n = normal(campo, x, y);
    let l = para_a_luz(o);
    let forca = o.forca_ambiente;
    let escala = escala_do_horizonte(o);
    let lum = |q: (isize, isize)| {
        let a = campo.albedo[campo.idx(q.0, q.1)];
        0.2126 * f64::from(a[0]) + 0.7152 * f64::from(a[1]) + 0.0722 * f64::from(a[2])
    };
    // `∫₀^a cos(θ − g)·sin θ dθ`: o peso pelo cosseno de um arco da fatia (o da visibilidade).
    let int = |a: f64, g: f64| 0.25 * (-libm::cos(2.0 * a - g) + libm::cos(g) + 2.0 * a * libm::sin(g));
    let mut soma = 0.0;
    for k in 0..FATIAS {
        let phi = PI * k as f64 / FATIAS as f64;
        let (dx, dy) = (libm::cos(phi), libm::sin(phi));
        let no = -n[0] * dy + n[1] * dx;
        let np = [n[0] + no * dy, n[1] - no * dx, n[2]];
        let len = libm::sqrt(np[0] * np[0] + np[1] * np[1] + np[2] * np[2]);
        if len < 1e-9 {
            continue;
        }
        let g = libm::acos((np[2] / len).clamp(-1.0, 1.0)).copysign(np[0] * dx + np[1] * dy);
        for sgn in [1.0, -1.0] {
            let gs = sgn * g;
            // O limite da meia-esfera da normal deste lado (abaixo da horizontal se ela pende para cá).
            let fim = FRAC_PI_2 + gs;
            let (mut e, mut parede) = (0.0f64, (x, y));
            for r in HORIZONTE {
                let r = r * escala;
                let (ox, oy) = (libm::round(sgn * dx * r), libm::round(sgn * dy * r));
                let d = libm::hypot(ox, oy);
                if d == 0.0 {
                    continue;
                }
                let q = (x + ox as isize, y + oy as isize);
                let dh = campo.at_fina(q.0, q.1) - h0;
                let sen = dh / libm::sqrt(dh * dh + d * d);
                if sen > e {
                    (e, parede) = (sen, q);
                }
            }
            let mut luz = 0.0;
            if e > 0.0 {
                let (a, b) = (FRAC_PI_2 - libm::asin(e), fim.min(FRAC_PI_2));
                if b > a {
                    // A face olha de volta para o papel: normal horizontal `−(dx, dy)·sgn`.
                    let ndl = -(sgn * dx * l[0] + sgn * dy * l[1]);
                    let face = 0.5 * forca + (1.0 - forca) * ndl.max(0.0) / l[2];
                    luz += (int(b, gs) - int(a, gs)) * lum(parede) * face;
                }
            }
            // O papel INCLINADO (a saia da forma desfocada) vê abaixo da horizontal o chão vizinho, aceso pela
            // normal DELE e com a sombra DELE: nos vãos entre lascas a encosta da frente apanha o sol que esta não
            // vê; na sombra projectada o vizinho também está à sombra. O céu dele é o aberto: tapá-lo pelo céu
            // deste papel apagava o vão (§K10, recusa).
            if fim > FRAC_PI_2 {
                let q = (x + libm::round(sgn * dx) as isize, y + libm::round(sgn * dy) as isize);
                let m = normal(campo, q.0, q.1);
                let ndl = (m[0] * l[0] + m[1] * l[1] + m[2] * l[2]).max(0.0);
                let sol = if ndl > 0.0 {
                    ndl * luz_directa(o, campo, q.0, q.1, campo.at_sombra(q.0, q.1))
                } else {
                    0.0
                };
                let chao = forca * 0.5 * (1.0 + m[2]) + (1.0 - forca) * sol / l[2];
                luz += (int(fim, gs) - int(FRAC_PI_2, gs)) * lum(q) * chao;
            }
            soma += len * luz;
        }
    }
    o.sombra * o.af.rebote_papel * soma / FATIAS as f64
}

/// A [`sombra_no_papel`] de uma célula PINTADA a partir dos [`Termos`] que a cor dela já calculou (a luz
/// directa, a visibilidade e o ganho do céu são as MESMAS peças): bit a bit a conta por célula, a metade do
/// custo (gate `a_sombra_dos_termos_e_bit_identica_a_do_papel`).
#[must_use]
pub fn sombra_dos_termos(o: &OleoReal, t: &Termos) -> f64 {
    sombra_de(o, t.luz, t.vis, t.dir)
}

/// A sombra no papel em `u8` (o plano que o composite guarda): a MESMA conversão nos dois caminhos. `s` é uma
/// fracção de LUZ (linear); o composite escurece o byte sRGB por ela, por isso guarda-se a fracção do byte que
/// tira essa luz: `1 − sRGB(1 − s)` (exacta no papel branco). Guardar `s` no byte era tirar ~2,2× a luz — o halo
/// cinza à volta de todo traço (K4, `oraculo/README.md` §K4).
#[must_use]
pub fn sombra_em_u8(s: f64) -> u8 {
    (255.0 - srgb255_of_linear(1.0 - s.clamp(0.0, 1.0))).round() as u8
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
    let hn = libm::sqrt(hv[0] * hv[0] + hv[1] * hv[1] + hv[2] * hv[2]);
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
        o.brilho * af.brilho_max * fres * d * ndl * luz * mistura
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
        let rug = libm::sqrt(libm::sqrt(a2));
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
            lampada[c] + o.brilho * forca * pre[c] * ab * livre
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
        };
        let d_metal = ggx(a2_amb);
        for c in 0..3 {
            let f0m = a[c].clamp(0.0, 1.0);
            let ab = f0m * xa + yb;
            let viz = a[c] * difuso(o, &sem_brilho, a[c], c) * (1.0 + (a[c] - 1.0) * m.metal);
            let em = lampada_de(f0m, 1.0, d_metal)
                + o.brilho * (forca * pre[c] * livre + (1.0 - livre) * viz) * ab;
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
            let ev = lv + o.brilho * forca * pre[c] * (F0 * xa + yb) * livre;
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
