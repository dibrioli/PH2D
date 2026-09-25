//! ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO** — a outra metade do chão que só recebe.
//!
//! O [`crate::ground`] desenha o que a peça **TIRA** (a sombra das lâmpadas, o escurecimento de
//! contacto do céu) e nunca desenhou o que ela **PÕE**: um vaso vermelho pousa numa sombra cinzenta,
//! onde a verdade é um avermelhado à volta da base. Este módulo é essa metade.
//!
//! # ⛔⛔⛔ Porque NÃO são as sondas da peça, e o número que o decide
//!
//! A pergunta óbvia é *«o chão não pode consultar a grelha de sondas que já está assada?»*. Medido
//! (a sonda `tests::chao_ricochete`, uma bola vermelha de raio `0,5` pousada, luz de lado), sobre a
//! irradiância devolvida no canal vermelho:
//!
//! | `x` (raios do centro) | VERDADE (8 192 direcções) | a grelha 3D devolve | `E·r²` |
//! |---:|---:|---:|---:|
//! | `1,00` | `0,002697` | `0,001941` | `0,00135` |
//! | `1,50` | `0,009411` | `0,003075` | `0,00765` |
//! | **`1,75`** | **`0,009772`** (o PICO) | `0,003063` | `0,00993` |
//! | `2,50` | `0,007167` | `0,003052` | `0,01299` |
//! | `4,00` | `0,002962` | `0,003046` | `0,01259` |
//! | `5,75` | `0,001165` | `0,003044` | `0,00992` |
//!
//! Duas leituras, e as duas mandam:
//!
//! - ⛔⛔ **a grelha 3D SATURA**: ela cobre a bola da peça com margem [`crate::probes::PROBE_MARGIN`]
//!   (`1,05` raios) e a consulta **agarra-se à borda** — daí a coluna dela ser uma CONSTANTE de
//!   `1,5` raios em diante. Num plano que vai até ao horizonte isso pinta o mundo inteiro de
//!   vermelho, com a mesma força a `5` raios e a `500`;
//! - ⛔⛔⛔ **e o PICO do sangramento fica FORA dela** (`1,75` raios contra os `1,05` que ela cobre).
//!   *Uma extrapolação ancorada na borda não pode reproduzir um máximo que acontece depois dela* —
//!   logo nem alargar a margem nem multiplicar por `1/r²` servem: a primeira rouba resolução à peça,
//!   e a segunda decai a partir do sítio errado.
//!
//! ⇒ **o chão é um PLANO, logo o campo dele é 2D** — e é isso que torna a lei própria barata:
//! [`GROUND_BOUNCE_GRID`]² pontos contra os `32³` da peça.
//!
//! # ⭐⭐ O estimador: as direcções vão todas para dentro do CONE da peça
//!
//! Um ponto de chão longe vê a peça num cone estreito. Com direcções uniformes sobre o hemisfério
//! **quase nenhuma acerta**, e o que se mede é a contagem de acertos: por isso a coluna VERDADE da
//! tabela precisou de `8 192` direcções para deixar de saltar (a `1 024` ela lia `0,0067` num ponto
//! e `0,0037` no seguinte — ruído de `±50 %` sobre uma curva lisa).
//!
//! ⇒ aqui as direcções são sorteadas **dentro do cone** que a bola da peça subtende, e o integral é
//! corrigido pelo ângulo sólido desse cone. **Nada no resto da cena devolve luz** (o chão é
//! invisível e não entra na marcha), logo fora do cone a radiância devolvida é ZERO **por
//! construção**, e o estimador continua sem viés.
//!
//! ⚠️ **A base do cone é CONTÍNUA sobre o chão, e isso é uma cerca:** a tangente sai de
//! `cross(eixo, X)`, que nunca degenera porque o eixo aponta do chão para o centro da peça e tem
//! `y > 0` **sempre** — logo o eixo nunca é paralelo a `X`. ⛔ Uma base construída pelo truque
//! habitual (o ramo que troca de fórmula quando a componente dominante muda) tem uma
//! **descontinuidade**, e uma descontinuidade num campo interpolado é uma COSTURA desenhada — a
//! recusa que o [`crate::occlusion`] já pagou.

use crate::{Ground, Orbit, PointLamp, Surfaces};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;
use rayon::prelude::*;

/// ⭐⭐⭐ **Pontos por aresta do campo — e o tecto é a MESMA saída de 8 bits das direcções.**
///
/// Desvio contra a VERDADE convergida (`8 192` direcções uniformes) ao longo do perfil, com o
/// relógio da assadura ao lado (`--release`, `load 21`, mínimo de 5):
///
/// | grelha | pior desvio | em bytes | relógio |
/// |---:|---:|---:|---:|
/// | `16²` | `0,002406` | `8` | `1,51 ms` |
/// | `24²` | `0,001027` | `3` | `3,52 ms` |
/// | **`32²`** | **`0,000736`** | **`2`** | **`5,33 ms`** |
/// | `48²` | `0,000492` | `2` | `12,00 ms` |
/// | `64²` | `0,000401` | `1` | `20,65 ms` |
///
/// ⚠️ **Esta tabela foi medida com a lâmpada LONGE e o nó a ler um PONTO** (antes do pré-filtro de
/// 2026-09-24). Com a lâmpada encostada a verdade tem riscas mais finas que qualquer célula e a
/// resolução deixa de ser a alavanca (`128²` erra o mesmo que `32²` — `docs/Render3d/09` §10).
///
/// ⇒ **`32²` é o JOELHO: ele lê os mesmos `2` bytes que `48²` por `44 %` do relógio**, e descer ao
/// byte seguinte custa `4×`. ⛔⛔ **E esta constante esteve em `48` sem uma medição por baixo** — o
/// §0.0 manda medir ANTES de escrever um limite, e o número não medido estava caro no lado errado:
/// ele pagava `6,7 ms` do quadro assente por nada que a saída soubesse representar.
pub const GROUND_BOUNCE_GRID: usize = 32;

/// ⭐⭐⭐ **Direcções por ponto, todas dentro do cone — e o tecto sai do RECURSO, que é a SAÍDA DE
/// 8 BITS.**
///
/// Medido contra o mesmo campo assado com `4 096` direcções, com o erro em unidades do **pico** do
/// campo (`0,015114`, que é `33/255` no vermelho sobre o fundo transparente do modelador) — *uma
/// célula onde a verdade é quase zero dá erro relativo enorme e contribui zero para a imagem, logo
/// a régua é absoluta*:
///
/// | direcções | pior desvio / pico | em bytes | desvio médio | em bytes |
/// |---:|---:|---:|---:|---:|
/// | `16` | `8,73 %` | `4` | `0,42 %` | `0` |
/// | `32` | `6,93 %` | `3` | `0,30 %` | `0` |
/// | `64` | `3,53 %` | `2` | `0,21 %` | `0` |
/// | **`128`** | **`0,96 %`** | **`0`** | `0,07 %` | `0` |
/// | `256` | `1,27 %` | `1` | `0,06 %` | `0` |
///
/// ⇒ **`128` é onde a pior mancha passa a valer menos de UM byte**, e subir mais não compra nada
/// que a saída saiba representar.
///
/// ⚠️ **Esta tabela foi medida a `48²`** (a grelha de então) — o que ela mede é o ESTIMADOR, e ele
/// não depende de quantas células há. O custo que shipa é o da grelha medida ao lado:
/// `32² × 128 = 131 072` raios, **`1,6 %`** dos `8,4 M` que a grelha da peça já paga.
///
/// ⚠️⚠️ **E a 1.ª régua desta tabela media SINAL e não ruído:** ela era a 2.ª diferença ao longo de
/// uma linha, normalizada pelo valor local, e leu `0,78 · 1,96 · 1,38 · 1,66 · 1,44` sobre a mesma
/// escada de direcções — *sem tendência nenhuma*. O que não cai com a amostragem não é ruído da
/// amostragem: era a queda a pique do campo debaixo da peça, que é a curva CERTA.
pub const GROUND_BOUNCE_DIRS: u32 = 128;

/// Meia-largura do campo, em raios da bola da peça.
pub const GROUND_BOUNCE_SPAN: f32 = 6.0;

/// A fracção EXTERNA do campo em que ele esmorece para zero.
pub const GROUND_BOUNCE_FADE: f32 = 0.25;

/// Raios por lote da assadura paralela — ver a nota em [`bake_ground_bounce`].
const LOTE: usize = 2048;

/// ⭐ **O campo 2D do chão** — uma irradiância por célula, na convenção do ricochete desta casa
/// (média da radiância devolvida pesada pelo cosseno; uma radiância uniforme `L` devolve `L`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroundBounce {
    /// O canto do campo, em mundo — `(x, z)`.
    pub origin: [f32; 2],
    /// A distância entre células vizinhas.
    pub step: f32,
    /// Células por aresta.
    pub n: usize,
    /// A altura do plano.
    pub height: f32,
    /// `n²` irradiâncias, em ordem `z * n + x` — a MÉDIA de cada célula (ver o pré-filtro em
    /// [`bake_ground_bounce`]).
    pub value: Vec<[f32; 3]>,
}

impl GroundBounce {
    /// O campo vazio — o que um chão sem peça, sem luz ou sem material devolve.
    #[must_use]
    pub fn vazio() -> Self {
        Self::default()
    }

    /// ⭐⭐ **A consulta** — a **B-spline cúbica uniforme** sobre os `4×4` nós à volta de `q`, vezes o
    /// esmorecimento da orla.
    ///
    /// ⛔⛔ **Até 2026-09-24 era BILINEAR, e a foto do dono mostrou o preço** (*«áreas retangulares
    /// ruins»*, a luz encostada ao nó da cena `=28`): a bilinear é contínua e a DERIVADA dela salta
    /// em cada linha da grelha, logo um campo com contraste desenha os VINCOS das células — losangos
    /// do tamanho de uma célula no chão. A B-spline é `C²` (sem vincos), tem pesos **não negativos**
    /// que somam `1` (um campo não negativo continua não negativo e um campo constante continua o
    /// mesmo) e é a reconstrução que acompanha a assadura PRÉ-FILTRADA de [`bake_ground_bounce`]: as
    /// duas metades juntas são o *prefiltro + reconstrução suave* das grelhas de irradiância.
    ///
    /// ⚠️ Ela **aproxima** e não interpola — num nó ela devolve `(v₋ + 4v + v₊)/6` e não `v`. Para
    /// um campo de irradiância é o sítio certo de errar (ela nunca inventa um máximo), e o gate
    /// `o_campo_do_chao_concorda_com_a_convergida` mede o custo no miolo.
    ///
    /// ⚠️ **Fora do campo devolve ZERO**, e isso é a metade conservadora de uma aproximação
    /// DECLARADA: a verdade continua a decair (`~1/r²`) e nós cortamo-la. *O erro escurece o chão
    /// longe da peça e nunca desenha uma aresta* — ver [`GROUND_BOUNCE_FADE`].
    ///
    /// ⚠️⚠️ **A saída antecipada de fora-do-campo é uma GUARDA DE ÍNDICE, e não a lei** — a lei é a
    /// [`GroundBounce::orla`], que chega a zero exactamente na borda; e os índices do estêncil
    /// prendem-se à borda pela mesma razão (ali a orla já pôs zero). *Quem apagar a ORLA parte o
    /// `a_orla_do_campo_esmorece_em_vez_de_cortar`.*
    #[must_use]
    pub fn sample(&self, q: [f32; 3]) -> [f32; 3] {
        if self.n < 2 || self.value.is_empty() || self.step.is_nan() || self.step <= 0.0 {
            return [0.0; 3];
        }
        #[allow(clippy::cast_precision_loss)]
        let lado = (self.n - 1) as f32;
        let u = [
            (q[0] - self.origin[0]) / self.step,
            (q[2] - self.origin[1]) / self.step,
        ];
        if u[0] < 0.0 || u[1] < 0.0 || u[0] > lado || u[1] > lado {
            return [0.0; 3];
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c0 = [
            (u[0].floor() as usize).min(self.n - 2),
            (u[1].floor() as usize).min(self.n - 2),
        ];
        #[allow(clippy::cast_precision_loss)]
        let f = [
            (u[0] - c0[0] as f32).clamp(0.0, 1.0),
            (u[1] - c0[1] as f32).clamp(0.0, 1.0),
        ];
        let (wx, wz) = (bspline_pesos(f[0]), bspline_pesos(f[1]));
        let ultimo = self.n - 1;
        let mut soma = [0.0f32; 3];
        for (dz, pz) in wz.iter().enumerate() {
            // O nó `c0 - 1 + dz`, preso à borda.
            let iz = (c0[1] + dz).saturating_sub(1).min(ultimo);
            for (dx, px) in wx.iter().enumerate() {
                let ix = (c0[0] + dx).saturating_sub(1).min(ultimo);
                let w = pz * px;
                let v = self.value[iz * self.n + ix];
                soma = [soma[0] + w * v[0], soma[1] + w * v[1], soma[2] + w * v[2]];
            }
        }
        let esmorece = self.orla(q);
        [soma[0] * esmorece, soma[1] * esmorece, soma[2] * esmorece]
    }

    /// O esmorecimento da ORLA: `1` no miolo, a descer para `0` na borda do campo.
    ///
    /// ⚠️ Ele é medido no **QUADRADO** do campo (a distância de Chebyshev ao centro) e não num
    /// círculo: o campo é quadrado, e um esmorecimento circular deixaria os quatro cantos a cair a
    /// pique na borda.
    fn orla(&self, q: [f32; 3]) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let meia = self.step * (self.n - 1) as f32 * 0.5;
        if meia.is_nan() || meia <= 0.0 {
            return 0.0;
        }
        let centro = [self.origin[0] + meia, self.origin[1] + meia];
        let d = (q[0] - centro[0]).abs().max((q[2] - centro[1]).abs()) / meia;
        ((1.0 - d) / GROUND_BOUNCE_FADE).clamp(0.0, 1.0)
    }
}

/// Os quatro pesos da **B-spline cúbica uniforme** na fracção `t ∈ [0, 1]` da célula — para os nós
/// `−1, 0, +1, +2`. Não negativos, somam `1`, e a curva é `C²`.
#[must_use]
pub fn bspline_pesos(t: f32) -> [f32; 4] {
    let s = 1.0 - t;
    let (t2, t3) = (t * t, t * t * t);
    [
        s * s * s / 6.0,
        (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0,
        (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0,
        t3 / 6.0,
    ]
}

/// ⭐⭐ **Onde o raio `j` de um nó nasce DENTRO da célula** — a sequência `R2` (o número plástico),
/// em fracções de célula em `[−½, ½)²`. Determinística: o mesmo campo sai sempre dos mesmos bits.
#[must_use]
pub fn desvio_na_celula(j: u32) -> [f32; 2] {
    const A1: f64 = 0.754_877_666_246_692_8;
    const A2: f64 = 0.569_840_290_998_053_3;
    let j = f64::from(j);
    #[allow(clippy::cast_possible_truncation)]
    let a = ((0.5 + j * A1).fract() - 0.5) as f32;
    #[allow(clippy::cast_possible_truncation)]
    let b = ((0.5 + j * A2).fract() - 0.5) as f32;
    [a, b]
}

/// ⭐⭐⭐ **ASSAR o campo do chão** — a MESMA lei que uma sonda da peça assa
/// ([`crate::bounce::radiancia_devolvida`]), com as direcções dentro do cone.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn bake_ground_bounce(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    ground: Ground,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    n: usize,
    dirs: u32,
    lado_px: usize,
) -> GroundBounce {
    assa(
        doc, reg, cam, ground, surfaces, lampadas, n, dirs, lado_px, true,
    )
}

/// ⚠️ **A lei ANTIGA — o nó a ler um PONTO** —, viva só para o CONTROLO do gate
/// `cada_no_da_grelha_do_chao_vale_a_media_da_celula`: sem ela a barra dele podia estar a medir uma
/// fixtura onde a amostragem pontual não dobra nada.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn bake_ground_bounce_pontual(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    ground: Ground,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    n: usize,
    dirs: u32,
    lado_px: usize,
) -> GroundBounce {
    assa(
        doc, reg, cam, ground, surfaces, lampadas, n, dirs, lado_px, false,
    )
}

#[allow(clippy::too_many_arguments)]
fn assa(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    ground: Ground,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    n: usize,
    dirs: u32,
    lado_px: usize,
    pre_filtra: bool,
) -> GroundBounce {
    let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, reg) else {
        return GroundBounce::vazio();
    };
    if lampadas.is_empty() || surfaces.all.is_empty() || n < 2 || dirs == 0 {
        return GroundBounce::vazio();
    }
    let raio = bola.radius.max(1e-3);
    let meia = raio * GROUND_BOUNCE_SPAN;
    #[allow(clippy::cast_precision_loss)]
    let step = 2.0 * meia / (n - 1) as f32;
    let origin = [bola.center[0] - meia, bola.center[2] - meia];
    let mut campo = GroundBounce {
        origin,
        step,
        n,
        height: ground.height,
        value: vec![[0.0; 3]; n * n],
    };

    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let scene = crate::probes::scene_solta(&shape, doc, reg, cam, lado_px);
    let base = crate::shade_render::ViewBasis::of(cam);
    let lift = scene.sharp.hit * crate::march::BIAS;
    // Até onde um raio procura: a diagonal do campo mais o diâmetro da peça.
    let alcance = 2.0 * meia * core::f32::consts::SQRT_2 + 2.0 * raio;

    // ── 1. os raios de cada célula, num lote só ───────────────────────────────────────────────
    let mut origens: Vec<[f32; 3]> = Vec::new();
    let mut raios: Vec<[f32; 3]> = Vec::new();
    let mut quais: Vec<usize> = Vec::new();
    // O ângulo sólido do cone de CADA raio: com a origem espalhada pela célula, o cone muda de raio
    // para raio, e o integral pesa cada um pelo seu.
    let mut omega: Vec<f32> = Vec::new();
    for iz in 0..n {
        for ix in 0..n {
            let k = iz * n + ix;
            for j in 0..dirs {
                // ⭐⭐⭐ **O PRÉ-FILTRO: o raio `j` nasce num ponto DIFERENTE da célula.** O nó passa a
                // valer a MÉDIA da irradiância sobre a célula dele, e não a irradiância no ponto do
                // nó — é o que uma grelha precisa para não DOBRAR (*alias*) uma feição mais fina que
                // ela. Medido na cena `=28` com a lâmpada encostada: a verdade tem riscas de sombra
                // finas (a mancha acesa da peça age como uma segunda lâmpada), e a amostragem
                // pontual apanhava uma risca ou um vão ao acaso em cada nó — era esse o losango.
                // ⚠️ **Custo ZERO:** os mesmos `dirs` raios por nó, só com origens diferentes.
                let dv = if pre_filtra {
                    desvio_na_celula(j)
                } else {
                    [0.0; 2]
                };
                #[allow(clippy::cast_precision_loss)]
                let q = [
                    origin[0] + (ix as f32 + dv[0]) * step,
                    ground.height,
                    origin[1] + (iz as f32 + dv[1]) * step,
                ];
                let Some((eixo, cos_alfa)) = cone_para(&bola, q, raio) else {
                    continue;
                };
                let (t, b) = base_do_cone(eixo);
                let d = dir_no_cone(j, dirs, cos_alfa, eixo, t, b);
                // ⚠️ Abaixo do horizonte não conta: o integral é do HEMISFÉRIO do chão.
                if d[1] <= 0.0 {
                    continue;
                }
                origens.push(q);
                raios.push(d);
                quais.push(k);
                omega.push(core::f32::consts::TAU * (1.0 - cos_alfa));
            }
        }
    }
    if origens.is_empty() {
        return campo;
    }

    // ── 2. a radiância devolvida por cada raio — a lei do produto ─────────────────────────────
    //
    // ⭐⭐ **Em LOTES paralelos, e isso não muda um bit:** a [`crate::march::march_rays`] é
    // SEQUENCIAL (ela serve passes que já vêm paralelizados de fora), e cada raio deste campo é
    // independente de todos os outros — repartir o lote calcula exactamente os mesmos números.
    // Medido a `48² × 128`: **`106,5 ms` num lote só**, que é metade de um quadro assente a pagar
    // por `1,6 %` dos raios que a grelha da peça paga no dispositivo.
    let sai: Vec<[f32; 3]> = origens
        .par_chunks(LOTE)
        .zip(raios.par_chunks(LOTE))
        .flat_map_iter(|(o, d)| {
            crate::bounce::radiancia_devolvida(
                &scene, &base, lift, alcance, o, d, surfaces, lampadas,
            )
        })
        .collect();

    // ── 3. o integral: `E = 1/(π·N) · Σ Ω_j·L·cos` — cada raio com o ângulo sólido do SEU cone ──
    let mut soma = vec![[0.0f32; 3]; n * n];
    for (m, k) in quais.iter().enumerate() {
        let c = raios[m][1] * omega[m];
        let l = sai[m];
        let s = &mut soma[*k];
        *s = [s[0] + c * l[0], s[1] + c * l[1], s[2] + c * l[2]];
    }
    #[allow(clippy::cast_precision_loss)]
    let a = 1.0 / (dirs as f32 * core::f32::consts::PI);
    for (v, s) in campo.value.iter_mut().zip(&soma) {
        *v = [s[0] * a, s[1] * a, s[2] * a];
    }
    campo
}

/// O cone que a bola da peça subtende a partir de `q` — `(eixo, cos α)`, ou `None` quando não há
/// cone nenhum (o ponto está dentro da bola, ou colado ao centro dela).
fn cone_para(
    bola: &ph2d_field_eval::bounds::Ball,
    q: [f32; 3],
    raio: f32,
) -> Option<([f32; 3], f32)> {
    let d = [
        bola.center[0] - q[0],
        bola.center[1] - q[1],
        bola.center[2] - q[2],
    ];
    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if r.is_nan() || r <= raio {
        // Dentro da bola: o cone é o hemisfério inteiro.
        return Some(([0.0, 1.0, 0.0], 0.0));
    }
    let sin_alfa = (raio / r).clamp(0.0, 1.0);
    let cos_alfa = (1.0 - sin_alfa * sin_alfa).max(0.0).sqrt();
    Some(([d[0] / r, d[1] / r, d[2] / r], cos_alfa))
}

/// A base do cone — ver a cerca da CONTINUIDADE no cabeçalho do módulo.
fn base_do_cone(eixo: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let t = cross(eixo, [1.0, 0.0, 0.0]);
    let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-12);
    let t = [t[0] / len, t[1] / len, t[2] / len];
    (t, cross(eixo, t))
}

/// A `j`-ésima direcção dentro do cone — a espiral de Fibonacci da casa, mapeada no cone.
fn dir_no_cone(
    j: u32,
    total: u32,
    cos_alfa: f32,
    eixo: [f32; 3],
    t: [f32; 3],
    b: [f32; 3],
) -> [f32; 3] {
    #[allow(clippy::cast_precision_loss)]
    let u = (j as f32 + 0.5) / total as f32;
    let cos_t = 1.0 - u * (1.0 - cos_alfa);
    let sin_t = (1.0 - cos_t * cos_t).max(0.0).sqrt();
    // O ângulo áureo, o mesmo de [`crate::occlusion::cone_dir`].
    #[allow(clippy::cast_precision_loss)]
    let phi = core::f32::consts::TAU * (j as f32) * 0.618_034;
    let (s, c) = phi.sin_cos();
    [
        eixo[0] * cos_t + sin_t * (c * t[0] + s * b[0]),
        eixo[1] * cos_t + sin_t * (c * t[1] + s * b[1]),
        eixo[2] * cos_t + sin_t * (c * t[2] + s * b[2]),
    ]
}
