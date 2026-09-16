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

/// A cobertura de uma IMAGEM: algum pixel da célula (mais a folga) passa do limiar de alfa?
#[expect(
    clippy::too_many_arguments,
    reason = "é o predicado de cobertura completo: a fatia, o tamanho, a célula, a folga e o limiar"
)]
fn tinta_na_celula(
    alpha: &[u8],
    width: u32,
    height: u32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    e: f64,
    limiar: u8,
) -> bool {
    let (w, h) = (width as usize, height as usize);
    if alpha.len() < w * h {
        return false;
    }
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
    (iy0..iy1).any(|y| (ix0..ix1).any(|x| alpha[y * w + x] >= limiar))
}

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
    /// ⭐⭐⭐ **QUANTOS TRIÂNGULOS A MALHA DEVE TER** — o passo passa a ser CONTADO, nunca escolhido.
    ///
    /// ⛔⛔ **Um passo em pixels NÃO é um orçamento, e a diferença é toda a arte que não é a do
    /// smoke:** a contagem de células é `área / passo²`, então a MESMA configuração entrega `~2 900`
    /// triângulos numa sprite de `512×320` e **`~12 400`** numa de `1024×1024` — `4,2×` o custo do
    /// quadro por a arte ser maior, que é literalmente *«o caminho mais lento define o tecto do mais
    /// rápido»* (§0.0). O botão `Quad Retopology` pagou esta mesma lição em 2026-08-28 e a cura foi
    /// a mesma: **ancorar na ÁREA e contar**.
    ///
    /// ⚠️ **O `fine`/`coarse` continuam a mandar na FORMA da graduação** (quantas vezes mais fina a
    /// malha fica junto de uma articulação); o que este número faz é **escalar os dois** até a
    /// contagem bater. ⇒ mexer neste valor muda o custo, mexer naqueles muda o desenho.
    ///
    /// ⭐ **De onde o `3 000` vem, e de que recurso ele é:** o TEMPO do quadro. Medido em 2026-09-15
    /// (`measure_the_cpu_cost_of_a_skinned_frame`, o MÍNIMO de 40 corridas), uma peça entregue custa
    /// **`0,44 µs`** no caminho que de facto corre (`Smooth` com o refinamento inerte) — plano em
    /// `1 152`, `4 608` e `10 368` peças. A fatia da pele é `1/10` de um quadro de 60 fps ⇒
    /// `1,667 ms / 0,44 µs` = **`3 788` peças** para UMA imagem sozinha. `3 000` deixa `20 %` de
    /// folga para a segunda imagem de uma cena e é onde a faceta desta arte cai a **`1,16 px`**,
    /// abaixo da barra de `1,5` (a tabela está no gate `a_arte_nao_sai_facetada`).
    ///
    /// ⚠️ `0` **desliga a contagem** e devolve o passo literal — é o que as fixturas de geometria
    /// pura usam para pedir uma grelha exacta.
    pub target_tris: usize,
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
            // ⭐ A CONTAGEM é que é o orçamento; o `fine`/`coarse` acima são a FORMA da graduação e
            // o ponto de partida da escala. Ver o doc do campo para de que recurso este número é.
            target_tris: 3_000,
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
    grid_mesh_com(
        &|x0, y0, x1, y1, e| {
            tinta_na_celula(
                alpha,
                width,
                height,
                x0,
                y0,
                x1,
                y1,
                e,
                opts.alpha_threshold,
            )
        },
        width,
        height,
        focos,
        opts,
    )
}

/// ⭐⭐⭐ **A MESMA GRELHA, com a COBERTURA injectada** — *«onde há arte?»* tem duas respostas neste
/// app e uma lei só.
///
/// `cobre(x0, y0, x1, y1, folga)` responde *«esta célula tem arte, com esta folga à volta?»*. Uma
/// imagem responde pelo **alfa**; um caminho vectorial responde por **estar dentro do contorno**.
///
/// ⛔⛔ **Ela existe para as duas mídias não terem duas malhas.** A graduação pelas articulações, a
/// conformidade (dois vizinhos partilham a aresta inteira), o orçamento em triângulos e a
/// renormalização são do GRID, não da mídia — e uma segunda grelha escrita para o vector divergiria
/// desta no primeiro ajuste, com o sintoma a ser *«a forma vectorial dobra diferente da imagem»*.
#[must_use]
pub fn grid_mesh_com(
    cobre: &Cobertura<'_>,
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: GridOptions,
) -> Option<Mesh2d> {
    let bruta = grelha(cobre, width, height, focos, opts)?;
    if opts.target_tris == 0 {
        return Some(bruta);
    }
    // ⭐⭐⭐ **UMA correcção, e a lei é a da ÁREA:** a contagem de células vai com `1/passo²`, logo
    // `passo_novo = passo × √(previsto / alvo)`. ⛔ Nunca um laço até bater o número exacto — a
    // silhueta recorta células, então o alvo não é alcançável por construção e um laço ficaria a
    // perseguir o último por cento. *É a mesma renormalização que a `SizingGrid` do botão de quads
    // aprendeu, e pela mesma razão.*
    #[expect(
        clippy::cast_precision_loss,
        reason = "contagens de triângulos de uma malha de imagem, muito abaixo do limite exacto do f64"
    )]
    let escala = ((bruta.tris.len() as f64) / (opts.target_tris as f64)).sqrt();
    if !escala.is_finite() || escala <= 0.0 {
        return Some(bruta);
    }
    let escalada = GridOptions {
        fine: opts.fine * escala,
        coarse: opts.coarse * escala,
        // ⚠️ **O raio do adensamento escala TAMBÉM**, e é o que mantém a FORMA da graduação: sem
        // isso uma malha duas vezes mais fina teria a banda fina a cobrir metade da arte.
        radius: opts.radius * escala,
        ..opts
    };
    grelha(cobre, width, height, focos, escalada).or(Some(bruta))
}

/// ⭐ **«Esta célula tem arte?»** — `(x0, y0, x1, y1, folga)` em coordenadas da malha.
pub type Cobertura<'a> = dyn Fn(f64, f64, f64, f64, f64) -> bool + 'a;

/// A grelha com os passos **literais** — sem a contagem. É a lei de sempre.
fn grelha(
    cobre: &Cobertura<'_>,
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: GridOptions,
) -> Option<Mesh2d> {
    if width == 0 || height == 0 {
        return None;
    }
    let fx: Vec<f64> = focos.iter().map(|p| p[0]).collect();
    let fy: Vec<f64> = focos.iter().map(|p| p[1]).collect();
    let xs = axis_samples(0.0, width.into(), &fx, opts);
    let ys = axis_samples(0.0, height.into(), &fy, opts);

    let tem_tinta =
        |x0: f64, y0: f64, x1: f64, y1: f64| cobre(x0, y0, x1, y1, opts.expand.max(0.0));

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
