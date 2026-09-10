//! ⭐⭐⭐ **A MALHA DE QUADRÍCULAS, GRADUADA PELAS ARTICULAÇÕES** — o que uma deformação precisa.
//!
//! # O report que a trouxe (dono, 2026-09-10, com três fotos)
//!
//! > *«a malha criada automaticamente é de péssima qualidade. deveria ser um quadmesh inteligente
//! > com maior densidade nas áreas das articulações»*
//!
//! Ele tem razão e o número diz quanto. A malha anterior era o **contorno** triangulado por
//! *ear-clipping*, e sobre a cápsula do smoke ela media:
//!
//! | | contorno | esta grelha |
//! |---|---:|---:|
//! | vértices | `18` | ver o gate |
//! | **no MIOLO** | **`0`** | a maioria |
//! | aspecto mediano | **`17,4`** | `~2` |
//! | pior aspecto | **`53,1`** | limitado |
//!
//! ⛔⛔ **Zero vértices no interior é a causa inteira:** toda a deformação tinha de passar pela
//! borda, e as lascas do leque cisalhavam a arte — é literalmente o que as fotos mostram.
//!
//! # ⚠️ O que «quadmesh» quer dizer aqui, e o que ele NÃO muda
//!
//! O que a qualidade da deformação pede é a **DISPOSIÇÃO DOS VÉRTICES** — células regulares, com
//! miolo, mais densas onde a dobra acontece. ⛔ O *primitivo guardado* não pode ser um quadrilátero:
//! o desenho é **um afim por triângulo**, e um afim não leva um quadrilátero qualquer a outro
//! qualquer (quatro pontos são oito equações para seis incógnitas). ⇒ cada célula é guardada como
//! **dois triângulos**, e a grelha vive na disposição.
//!
//! # A construção, e porque é uma GRELHA-PRODUTO e não uma quadtree
//!
//! Os cortes são escolhidos **eixo a eixo**: uma lista de `x` e uma lista de `y`, densas perto das
//! articulações e largas longe delas. A malha é o produto das duas.
//!
//! ⭐⭐⭐ **Ela CONFORMA por construção** — dois vizinhos partilham a aresta inteira, sempre. ⛔ Uma
//! *quadtree* graduada (a resposta «óbvia») deixa **nós pendurados** na transição entre níveis, e um
//! nó pendurado abre **fenda** numa deformação: ele move-se pelos pesos dele enquanto a aresta do
//! vizinho grosso se move linearmente entre as pontas. Curá-los pede a tabela de moldes de
//! transição (5 casos a menos de rotação) — e a grelha-produto entrega o mesmo adensamento sem
//! nenhum deles.
//!
//! ⚠️ **A fronteira DECLARADA da grelha-produto:** a densidade é o produto de dois campos de uma
//! dimensão, então uma articulação adensa a **coluna** e a **linha** inteiras dela, e não só a
//! vizinhança. Para um membro — que é o caso deste módulo — isso é o que se quer: articulações ao
//! longo de um braço dão colunas finas em cada dobra, e as linhas ficam largas porque o membro é
//! fino de través.

use crate::Mesh2d;

/// ⭐ **Os números da grelha.** Todos em **pixels da imagem**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridOptions {
    /// O passo **junto de uma articulação** — o mais fino que a malha fica.
    pub fine: f64,
    /// O passo **longe de todas** — o mais largo.
    ///
    /// ⚠️ `coarse < fine` é coagido para `fine` na porta: uma grelha mais grossa perto da dobra do
    /// que longe dela é o oposto do que o dono pediu, e recusar em silêncio seria pior.
    pub coarse: f64,
    /// Até que distância de uma articulação a malha adensa.
    pub radius: f64,
    /// A partir de que alfa um pixel conta como tinta. Ver [`crate::MeshOptions`].
    pub alpha_threshold: u8,
    /// ⭐ **Quanto a malha passa da tinta**, em pixels — o *Expansion* do *Puppet* do After Effects.
    ///
    /// ⚠️ **A malha NÃO segue a silhueta, ela COBRE-A.** O recorte fino é do **alfa da própria
    /// arte**, que já o faz de graça e ao sub-pixel; obrigar a grelha a seguir o contorno traria de
    /// volta as células deformadas da borda — que é exactamente o defeito que esta wave cura.
    pub expand: f64,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            // ⚠️ Números de PRODUTO, não tectos de recurso: eles são o ponto de partida e o smoke é
            // quem os julga. O que está medido é a FORMA da resposta (os gates da monotonia e do
            // adensamento), nunca estes três valores.
            fine: 10.0,
            coarse: 26.0,
            radius: 40.0,
            alpha_threshold: 1,
            expand: 2.0,
        }
    }
}

/// ⭐⭐⭐ **OS CORTES DE UM EIXO** — densos perto de um foco, largos longe dele.
///
/// `min`/`max` são os extremos do eixo; `focos` são as coordenadas (no MESMO eixo) das
/// articulações. Devolve os cortes por ordem, começando em `min` e acabando em `max`.
///
/// ⚠️⚠️ **A marcha nunca SALTA um foco.** Sem essa guarda, um passo largo que comece pouco antes de
/// uma articulação atravessa-a inteira, e a dobra fica exactamente no meio de uma célula grande —
/// *o adensamento existiria na tabela e não no sítio que interessa*.
#[must_use]
pub fn axis_samples(min: f64, max: f64, focos: &[f64], opts: GridOptions) -> Vec<f64> {
    let fine = opts.fine.max(0.5);
    let coarse = opts.coarse.max(fine);
    let radius = opts.radius.max(f64::MIN_POSITIVE);
    // ⚠️ Pela `partial_cmp` de propósito: o caso a apanhar é `max <= min` **e** o `NaN`, e um
    // `max <= min` sozinho deixaria um eixo `NaN` marchar para sempre.
    if min.partial_cmp(&max) != Some(core::cmp::Ordering::Less) {
        return vec![min, min];
    }
    // O passo local: `fine` sobre um foco, `coarse` a partir de `radius`, recta entre os dois.
    let passo = |x: f64| -> f64 {
        let d = focos
            .iter()
            .map(|f| (x - f).abs())
            .fold(f64::INFINITY, f64::min);
        if !d.is_finite() {
            return coarse;
        }
        let t = (d / radius).clamp(0.0, 1.0);
        (coarse - fine).mul_add(t, fine)
    };
    let mut xs = vec![min];
    let mut x = min;
    loop {
        let h = passo(x);
        let mut nx = x + h;
        // ⚠️ Não saltar um foco — se há um dentro do passo, o corte cai NELE.
        for &f in focos {
            if f > x + fine * 0.25 && f < nx {
                nx = nx.min(f);
            }
        }
        // O último vão funde-se com o `max` em vez de deixar uma tira fininha, que daria uma
        // célula de aspecto enorme mesmo com a grelha inteira certa.
        if nx >= max - fine * 0.5 {
            break;
        }
        xs.push(nx);
        x = nx;
    }
    xs.push(max);
    xs
}

/// ⭐⭐⭐ **A MALHA DE UMA IMAGEM, graduada pelas articulações** — a porta da 2.ª mídia.
///
/// `focos` são as articulações em **pixels da imagem** (quem as tem é o esqueleto, e é ele que as
/// entrega — este leaf não sabe o que é um osso). Uma lista vazia dá uma grelha **uniforme** a
/// `coarse`, que é a leitura certa de *«não há dobra nenhuma para adensar»*.
///
/// `None` quando nenhuma célula tem tinta.
#[must_use]
pub fn grid_mesh_of(
    alpha: &[u8],
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: GridOptions,
) -> Option<Mesh2d> {
    let (w, h) = (width as usize, height as usize);
    if w == 0 || h == 0 || alpha.len() < w * h {
        return None;
    }
    let fx: Vec<f64> = focos.iter().map(|p| p[0]).collect();
    let fy: Vec<f64> = focos.iter().map(|p| p[1]).collect();
    let xs = axis_samples(0.0, width.into(), &fx, opts);
    let ys = axis_samples(0.0, height.into(), &fy, opts);

    let tem_tinta = |x0: f64, y0: f64, x1: f64, y1: f64| -> bool {
        let e = opts.expand.max(0.0);
        let (a, b) = ((x0 - e).max(0.0), (y0 - e).max(0.0));
        let (c, d) = ((x1 + e).min(width.into()), (y1 + e).min(height.into()));
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "os quatro foram limitados à grelha nas linhas acima"
        )]
        let (ix0, iy0, ix1, iy1) = (
            a as usize,
            b as usize,
            (c.ceil() as usize).min(w),
            (d.ceil() as usize).min(h),
        );
        (iy0..iy1).any(|y| (ix0..ix1).any(|x| alpha[y * w + x] >= opts.alpha_threshold))
    };

    // Índice do vértice `(i, j)` da grelha, criado só quando uma célula viva o pede — ⛔ emitir a
    // grelha inteira deixaria vértices órfãos, que o esqueleto pesaria e ninguém desenharia.
    let mut idx: Vec<Option<u32>> = vec![None; xs.len() * ys.len()];
    let mut rest: Vec<[f64; 2]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for j in 0..ys.len().saturating_sub(1) {
        for i in 0..xs.len().saturating_sub(1) {
            if !tem_tinta(xs[i], ys[j], xs[i + 1], ys[j + 1]) {
                continue;
            }
            let mut no = |i: usize, j: usize| -> u32 {
                let k = j * xs.len() + i;
                if let Some(v) = idx[k] {
                    return v;
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "uma grelha de imagem não passa de 2^32 nós muito antes de a imagem caber em memória"
                )]
                let v = rest.len() as u32;
                rest.push([xs[i], ys[j]]);
                idx[k] = Some(v);
                v
            };
            let (a, b, c, d) = (no(i, j), no(i + 1, j), no(i + 1, j + 1), no(i, j + 1));
            // ⚠️ **A diagonal é SEMPRE a mesma** (`a–c`), escolhida no repouso. Escolhê-la pela
            // célula DEFORMADA (a mais curta das duas) faria a malha trocar de diagonal a meio de
            // um gesto — e o desenho **piscaria** exactamente enquanto o artista dobra.
            tris.push([a, b, c]);
            tris.push([a, c, d]);
        }
    }
    (!tris.is_empty()).then_some(Mesh2d {
        rest,
        tris,
        size: [width, height],
    })
}
