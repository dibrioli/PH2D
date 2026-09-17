//! ⭐⭐⭐ **O RICOCHETE — a luz que a CENA devolve a cada pixel** (a `W5`, `docs/Render3d/03` §W5).
//!
//! # ⭐⭐ Ele é a outra METADE de um integral que este módulo já calcula
//!
//! A luz que chega a um ponto pelo hemisfério tem duas parcelas: a que vem do **céu** e a que vem
//! das **superfícies**. Este módulo já resolvia a primeira — a [`crate::occlusion`] mede *quanto do
//! céu chega* e o canal dela **atenua**; o que faltava era a segunda, que **soma**.
//!
//! ⇒ *a oclusão que este módulo já ship é a luz indirecta com a cor do ricochete posta a PRETO*, e
//! esta passagem troca esse preto pela superfície que bloqueou.
//!
//! ⚠️ **É por isso que o resultado mora no [`crate::Shadows`]** e não num canal ao lado: o doc
//! daquele tipo escreve a lei — *«ela mora aqui porque é a MESMA pergunta que as lâmpadas
//! respondem: quanto desta fonte chega a este pixel? Um segundo canal ao lado faria o pintor
//! perguntar duas vezes a mesma coisa, e é assim que dois canais divergem.»*
//!
//! # ⭐ O conjunto de direcções é o MESMO da oclusão, e isso não é economia
//!
//! As direcções saem da [`crate::occlusion::cone_dir`] com o mesmo peso `max(0, n·d)`. Dois
//! conjuntos diferentes dariam duas respostas para *«que parte do hemisfério é esta?»* — a soma do
//! céu atenuado com o ricochete deixaria de ser o integral de nada.
//!
//! ⚠️ **A diferença é a PERGUNTA, não o raio:** a oclusão pergunta *se* o raio bateu, esta pergunta
//! *no quê*. É a razão de existir da [`crate::march::march_rays`].
//!
//! # ⭐⭐⭐ E por isso ele refina em FATIAS, pela mesma máquina
//!
//! A medição de `docs/Render3d/08` §5 diz que a passagem inteira não cabe num quadro (`8,8 ms` a
//! `8` direcções no aberto, `376,8` numa caixa fechada) e a §6 diz que ela precisa de **dezenas**
//! de direcções para o erro cair. As duas juntas dão a rota: a [`bounce_slice`] paga **um pedaço
//! da sequência de cada vez**, e o quadro assente vai-a acumulando enquanto a mão está parada —
//! exactamente o que a [`crate::occlusion::occlusion_slice`] faz para o céu.
//!
//! ⚠️⚠️ **E as duas metades avançam com o MESMO índice `k`**, na [`crate::occlusion::refine_sky`]:
//! uma publicação em que o céu já consumiu `k` direcções e o ricochete outras tantas **diferentes**
//! mostraria dois hemisférios distintos somados. *A partilha do conjunto de direcções só é uma lei
//! enquanto as duas metades a percorrerem no mesmo passo.*
//!
//! # ⏳ O que esta passagem NÃO faz, e está declarado
//!
//! - **um ricochete.** A luz que sai do ponto acertado é só a **directa** dele; ela não traz o que
//!   lhe chegou por sua vez. O sangramento de cor — o que a régua mede — nasce no primeiro.
//! - **a parte DIFUSA.** O resultado é uma irradiância por pixel, que não tem direcção: o lóbulo
//!   especular pede *«que luz vem DAQUELA direcção»* e uma média do hemisfério não responde. O
//!   especular indirecto continua a ser o céu.
//! - **o dispositivo.** Ela é CPU, e o quadro assente do modelador é da placa desde a `§36`.

use crate::march::{self, Scene};
use crate::shade_render::ViewBasis;
use crate::{Gbuffer, Orbit, PointLamp, Surfaces};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// ⭐⭐⭐ **O QUE UMA FATIA DE RICOCHETE ENTREGA** — a soma pesada E o peso que a produziu.
///
/// ⚠️ **O denominador é o PESO, nunca a contagem de direcções**, pela mesma razão que o
/// [`crate::ConeSlice`] documenta: duas direcções do reticulado fixo contribuem com `n·d`
/// diferentes para a mesma normal, e as primeiras caem perto do pólo — dividir pela contagem lê
/// **mais luz do que existe** no meio do refinamento.
#[derive(Clone, Debug, Default)]
pub struct BounceSlice {
    /// `Σ w·L`, por pixel.
    pub sum: Vec<[f32; 3]>,
    /// `Σ w`, por pixel — o denominador.
    pub weight: Vec<f32>,
}

impl BounceSlice {
    /// Uma fatia vazia do tamanho do quadro — o que as cercas devolvem quando não há o que marchar.
    #[must_use]
    pub fn empty(pixels: usize) -> Self {
        Self {
            sum: vec![[0.0f32; 3]; pixels],
            weight: vec![0.0f32; pixels],
        }
    }

    /// Acumula outra fatia nesta.
    pub fn add(&mut self, outra: &Self) {
        for (a, b) in self.sum.iter_mut().zip(&outra.sum) {
            for c in 0..3 {
                a[c] += b[c];
            }
        }
        for (a, b) in self.weight.iter_mut().zip(&outra.weight) {
            *a += b;
        }
    }

    /// A irradiância média (`E/π`). ⚠️ Um pixel de FUNDO — e um sem peso nenhum — lê `[0, 0, 0]`,
    /// que é o **OPOSTO** da lei da irmã do céu: uma luz que não foi calculada é **ausência de
    /// luz**, e inventá-la é a única das duas que acende o que devia estar escuro.
    #[must_use]
    pub fn average(&self, hit: &[bool]) -> Vec<[f32; 3]> {
        self.sum
            .iter()
            .zip(&self.weight)
            .zip(hit)
            .map(|((s, w), h)| {
                if *h && *w > 0.0 {
                    s.map(|c| c / w)
                } else {
                    [0.0; 3]
                }
            })
            .collect()
    }
}

/// A luz que a cena devolve a cada pixel, como irradiância normalizada (`E/π`) — a mesma unidade
/// que o [`ph2d_material::Environment::irradiance`] promete.
///
/// `total` é o número de direcções do conjunto de cones; `0` devolve o canal vazio, que o pintor lê
/// como ausência de ricochete (e aí o quadro é o de sempre, ao bit).
///
/// ⭐ Ela é a [`bounce_slice`] da sequência INTEIRA, e há gate a provar que somar as fatias uma a
/// uma dá estes mesmos bytes.
#[must_use]
pub fn bounce_pass(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    total: u32,
) -> Vec<[f32; 3]> {
    bounce_slice(doc, reg, cam, g, surfaces, lampadas, 0, total, total).average(&g.hit)
}

/// ⭐⭐⭐ **UMA FATIA da sequência** — a soma pesada da luz devolvida pelas direcções
/// `primeiro..primeiro+quantos` de um conjunto de `total`, **sem dividir**.
///
/// ⚠️ **Os três números são obrigatórios e nenhum é redundante**, pela mesma lei da
/// [`crate::occlusion::occlusion_slice`]: `total` fixa **o conjunto de direcções** e
/// `primeiro`/`quantos` dizem que pedaço dele esta chamada paga. *Uma API com `(desde, quantos)`
/// não consegue exprimir «a direcção `k` de um conjunto de 48»* — e como o reticulado de Fibonacci
/// é função de `total`, ancorá-lo na FATIA faria cada passagem amostrar um hemisfério diferente.
#[must_use]
#[allow(clippy::too_many_arguments)] // a peça, a vista, os materiais, as luzes e a fatia
pub fn bounce_slice(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    primeiro: u32,
    quantos: u32,
    total: u32,
) -> BounceSlice {
    let pixels = g.hit.len();
    let mut fatia = BounceSlice::empty(pixels);
    if pixels == 0 || quantos == 0 || total == 0 || lampadas.is_empty() || surfaces.all.is_empty() {
        return fatia;
    }

    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: crate::Sharpness::for_frame(cam.half_extent, g.width.min(g.height) as usize),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: crate::Stencil::Tetra4,
    };
    let base = ViewBasis::of(cam);
    let lift = scene.sharp.hit * march::BIAS;
    // ⭐ **Até onde um raio pode bater:** a bola que envolve a peça. Um raio que parte de dentro
    // dela sai, no máximo, pelo diâmetro — e para lá disso não há o que acertar.
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let alcance = 2.0 * bola.radius;
    if alcance <= 0.0 {
        return fatia;
    }

    // ── 1. os raios do hemisfério, um lote só ─────────────────────────────────────────────────
    let mut quais: Vec<usize> = Vec::new();
    let mut pesos: Vec<f32> = Vec::new();
    let mut origens: Vec<[f32; 3]> = Vec::new();
    let mut dirs: Vec<[f32; 3]> = Vec::new();
    // ⚠️ O índice percorre QUATRO tabelas paralelas (a máscara, a normal, o ponto e o peso) — um
    // `enumerate` sobre uma delas esconderia as outras três.
    #[allow(clippy::needless_range_loop)]
    for i in 0..pixels {
        if !g.hit[i] {
            continue;
        }
        let n = base.view_to_world(g.normal[i]);
        let p = g.point[i];
        let erguido = [p[0] + n[0] * lift, p[1] + n[1] * lift, p[2] + n[2] * lift];
        for j in 0..quantos {
            let k = primeiro + j;
            if k >= total {
                break;
            }
            let d = crate::occlusion::cone_dir(k, total);
            let w = n[0] * d[0] + n[1] * d[1] + n[2] * d[2];
            if w <= 0.0 {
                continue;
            }
            quais.push(i);
            pesos.push(w);
            origens.push(erguido);
            dirs.push(d);
            // ⭐ O peso acumula-se AQUI, dentro da fatia, e não no fim: ele é o denominador da
            // média parcial, e um raio que não acerta em nada continua a ser hemisfério medido.
            fatia.weight[i] += w;
        }
    }
    if origens.is_empty() {
        return fatia;
    }

    // ── 2. no quê bateram ─────────────────────────────────────────────────────────────────────
    let (bateu, normal, ponto) =
        march::march_rays(&scene, &origens, &dirs, &[0.0, alcance], &mut |_| None);

    let acertos: Vec<usize> = (0..bateu.len()).filter(|j| bateu[*j]).collect();
    if acertos.is_empty() {
        return fatia;
    }

    // ── 3. quanto de cada lâmpada chega ao ponto acertado — um lote POR LÂMPADA ────────────────
    let mut visivel: Vec<Vec<f32>> = Vec::with_capacity(lampadas.len());
    for lamp in lampadas {
        let mut o = Vec::with_capacity(acertos.len());
        let mut d = Vec::with_capacity(acertos.len());
        let mut ate = Vec::with_capacity(acertos.len());
        for &j in &acertos {
            let q = ponto[j];
            let nq = base.view_to_world(normal[j]);
            let para_luz = [
                lamp.world[0] - q[0],
                lamp.world[1] - q[1],
                lamp.world[2] - q[2],
            ];
            let dist =
                (para_luz[0] * para_luz[0] + para_luz[1] * para_luz[1] + para_luz[2] * para_luz[2])
                    .sqrt()
                    .max(f32::EPSILON);
            o.push([
                q[0] + nq[0] * lift,
                q[1] + nq[1] * lift,
                q[2] + nq[2] * lift,
            ]);
            d.push([para_luz[0] / dist, para_luz[1] / dist, para_luz[2] / dist]);
            ate.push(dist);
        }
        visivel.push(march::march_shadow_to(
            &scene,
            &o,
            &d,
            &ate,
            crate::shadow::HARDNESS,
        ));
    }

    // ── 4. a radiância que sai de cada ponto acertado, e a soma pesada ─────────────────────────
    for (m, &j) in acertos.iter().enumerate() {
        let q = ponto[j];
        let nq = base.view_to_world(normal[j]);
        // ⭐ O observador daquele ponto é **quem lhe perguntou**: o raio veio de `-dirs[j]`.
        let v = [-dirs[j][0], -dirs[j][1], -dirs[j][2]];
        let mat = surfaces.of(q);
        let mut sai = [0.0f32; 3];
        for (l, lamp) in lampadas.iter().enumerate() {
            let dl = [
                lamp.world[0] - q[0],
                lamp.world[1] - q[1],
                lamp.world[2] - q[2],
            ];
            let cru = dl[0] * dl[0] + dl[1] * dl[1] + dl[2] * dl[2];
            // ⚠️ **O braço degenerado é o do sombreador** (abaixo do piso a direcção é a NORMAL,
            // que é o limite finito), e ele está escrito aqui porque a resposta dele é *«a
            // normal»* — uma resposta em MUNDO aqui e em VISTA lá. O que se partilha é a lei
            // física, a [`crate::shade_render::chega_da_lampada`].
            let para_luz = if cru <= crate::shade_render::PISO_DA_LAMPADA {
                nq
            } else {
                let inv = cru.sqrt().recip();
                [dl[0] * inv, dl[1] * inv, dl[2] * inv]
            };
            let chega = crate::shade_render::chega_da_lampada(lamp, cru, visivel[l][m]);
            let c = mat.direct(nq, v, para_luz, chega);
            sai = [sai[0] + c[0], sai[1] + c[1], sai[2] + c[2]];
        }
        let i = quais[j];
        let w = pesos[j];
        fatia.sum[i] = [
            fatia.sum[i][0] + w * sai[0],
            fatia.sum[i][1] + w * sai[1],
            fatia.sum[i][2] + w * sai[2],
        ];
    }
    fatia
}

/// ⭐⭐⭐ **A SUAVIZAÇÃO do ricochete — a MESMA lei do céu, pela mesma porta.**
///
/// ⚠️⚠️ **Ela não é uma escolha de gosto: é a razão que o doc do [`crate::blur_occlusion`] já tem
/// escrita.** O que aquele borrão suaviza hoje não é ruído de amostragem (a oclusão é
/// determinística e há dois gates a afirmá-lo) — são as **estrias do conjunto discreto de
/// direcções**, a assinatura de `48` cones em vez de infinitos. O ricochete corre no **mesmo
/// conjunto** ([`crate::occlusion::cone_dir`]), logo tem a mesma assinatura.
///
/// ⭐ A guarda da normal vive numa porta só ([`crate::occlusion::para_cada_vizinhanca`]) e este
/// canal só soma: *duas cópias da guarda divergiriam no dia em que alguém afinasse o
/// [`crate::OCCLUSION_BLUR_COS`] numa delas, e a quina ficaria esborratada só num dos canais.*
#[must_use]
pub fn blur_bounce(g: &Gbuffer, canal: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut out = canal.to_vec();
    let mut vizinhos = Vec::with_capacity(9);
    crate::occlusion::para_cada_vizinhanca(g, &mut vizinhos, |i, js| {
        let mut soma = [0.0f32; 3];
        for &j in js {
            for c in 0..3 {
                soma[c] += canal[j][c];
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / js.len() as f32;
        out[i] = soma.map(|c| c * inv);
    });
    out
}
