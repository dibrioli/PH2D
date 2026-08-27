//! **A MARCHA** — o núcleo que anda um raio contra o campo, em fatias de profundidade.
//!
//! Irmão do [`crate`] por responsabilidade (teto de LOC): o `lib.rs` fica com a API pública e as
//! tolerâncias, este com o laço. Ver [`march_slabs`], que é onde a lei das fatias mora.

use crate::{MAX_STEPS, Orbit, Sharpness, T_MAX, slab};

/// ⭐⭐⭐ **Quantas AMOSTRAS de campo a marcha pediu** (W71) — o denominador de *«quantos passos
/// custa um raio»*.
///
/// ⚠️ **Ele existe porque a marcha passou a ser `80 %` do quadro** (§72.1) e ninguém sabia a forma
/// desse custo: um raio que dá 8 passos e um que dá 40 pedem curas opostas — o primeiro é caro
/// **por amostra** (a fita), o segundo é caro **em passos** (a lei da marcha).
///
/// ⚠️ Ela conta **amostras de esfera-marcha**, não as da normal (essas são seis por acerto e saem
/// noutro sítio).
#[doc(hidden)]
pub static STEP_SAMPLES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// ⭐⭐⭐ **Quantos RAIOS a marcha de facto andou** (W81) — o denominador que faltava.
///
/// ⚠️ **A §73 dividiu as amostras pelos PIXELS e leu `8,7`**, e daí concluiu que *«a marcha já está
/// apertada, a sobre-relaxação não tem de onde tirar»*. ⛔ Mas a maior parte de um quadro é **fundo
/// que nunca entra na caixa da peça** e custa exactamente zero amostras: dividir por ele mistura os
/// raios que trabalham com os que não existem. *Duas divisões da mesma medição não são duas
/// medições — só uma delas tem denominador.*
///
/// Ele conta os raios que **entraram no recorte** (ver [`Scene::clip`]) — os únicos que dão sequer
/// um passo.
#[doc(hidden)]
pub static MARCH_RAYS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// ⭐⭐ **As amostras da NORMAL** (W81) — seis por acerto, e elas não estavam em conta nenhuma.
///
/// ⚠️ A [`STEP_SAMPLES`] diz no doc dela que não as conta *«elas saem noutro sítio»* — e o sítio não
/// existia. Sem este contador, o `ns/amostra` da §73 divide o quadro inteiro por um numerador ao
/// qual falta uma parcela, e o preço da diferença central fica invisível.
#[doc(hidden)]
pub static NORMAL_SAMPLES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Quantos degraus o histograma da marcha distingue antes de saturar o último balde.
pub const HIST: usize = 64;

/// Quantas fatias de profundidade os contadores por-fatia distinguem (o último balde satura).
pub const SLABS_COUNTED: usize = 8;

/// ⭐⭐⭐ **As amostras de cada FATIA DE PROFUNDIDADE** (W81) — o par da [`crate::SLAB_SPEC`].
///
/// ⚠️ **As duas fatias de FORA existem sem ninguém as ter pedido**: a `0` vai de `0` a `t_lo` e a
/// última de `t_hi` a `T_MAX`, e elas estão lá porque a faixa dos quatro raios de canto **não**
/// contém os raios interiores (ver [`crate::tiles::slab_bounds`]). O doc delas diz que *«custam zero
/// quando ninguém lá chega»* — ⚠️ mas quando **um** raio lá chega, elas custam uma compilação de JIT
/// inteira, igual à de uma fatia cheia. *Uma fatia preguiçosa é barata em média e não é barata em
/// nenhuma unidade.*
#[doc(hidden)]
pub static SLAB_SAMPLES: [std::sync::atomic::AtomicU64; SLABS_COUNTED] =
    [const { std::sync::atomic::AtomicU64::new(0) }; SLABS_COUNTED];

/// ⭐⭐⭐ **A CURVA DE SOBREVIVÊNCIA da marcha** (W81) — `STEP_HIST[k]` são as amostras dadas ao
/// `k`-ésimo passo (o último balde satura).
///
/// ⚠️ **Uma média não escolhe entre duas curas opostas.** `35` amostras por raio podem ser *todo
/// raio dá 35* (a marcha aproxima-se devagar ⇒ sobre-relaxação) ou *nove em cada dez dão 3 e o
/// décimo dá 300* (uma cauda de raios rasantes ⇒ outra cura inteira). A forma está aqui, e ela
/// custa **um atómico por passo por ladrilho** — a mesma ordem do que a [`STEP_SAMPLES`] já paga,
/// e **nada** por amostra.
#[doc(hidden)]
pub static STEP_HIST: [std::sync::atomic::AtomicU64; HIST] =
    [const { std::sync::atomic::AtomicU64::new(0) }; HIST];

/// ⭐⭐⭐ **Quantas vezes a marcha caiu para a fita NÃO especializada** (W81) — o caminho de recuo.
///
/// ⚠️ **Um recuo custa um `fork`, e um `fork` é uma compilação de JIT inteira** — a W70 mediu-a em
/// `2,89 ms`, que é mais do que um quadro de movimento inteiro. Ele não aparece na
/// [`crate::SPECIALISE_NS`] (que só cronometra a especialização) nem em gate de imagem nenhum: a
/// imagem do recuo é a **certa**, porque a árvore completa é a resposta verdadeira em todo o lado.
/// *É a mesma família do defeito «só de relógio» da W70 — e por isso precisa de um contador.*
#[doc(hidden)]
pub static FORKED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐⭐ **O ESTÊNCIL DA NORMAL** (W81) — quantas amostras de campo uma normal custa.
///
/// ⚠️ **Ele existe porque a normal era `21 %` de TODAS as amostras do quadro e não estava em conta
/// nenhuma** (ver [`NORMAL_SAMPLES`]): seis avaliações por pixel acertado, ao lado das oito que a
/// marcha inteira daquele raio custou.
///
/// ⭐⭐ **As duas leis são a MESMA soma**, e é isso que faz um estêncil ser uma *tabela* e não um
/// caminho: o gradiente é `Σ dᵢ · f(p + ε·dᵢ)` sobre os deslocamentos. Para a diferença central os
/// deslocamentos são `±x, ±y, ±z` e a soma colapsa em `[g₀−g₁, g₂−g₃, g₄−g₅]` — **exactamente** o
/// que o código escrevia à mão. *Um terceiro estêncil passa a ser uma linha de tabela.*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stencil {
    /// A diferença central nos três eixos — **seis** amostras. O que o módulo ship até à W81.
    Central6,
    /// ⭐ Os quatro vértices de um tetraedro regular — **quatro** amostras, `1,5×` menos.
    ///
    /// ⚠️ As amostras ficam a `ε√3` do ponto (e não a `ε`), porque cada deslocamento tem os três
    /// eixos a `±1`. A folga da região é `4ε` ⇒ ela cobre-o, e quem o **prova** é o gate
    /// `every_sample_lies_inside_the_region_that_built_its_tape`, que mede a fronteira a sério.
    Tetra4,
}

impl Stencil {
    /// Os deslocamentos, em unidades de `ε` — e eles são **os dois** papéis: onde amostrar, e com
    /// que peso somar. Ver o doc do tipo.
    #[must_use]
    pub const fn offsets(self) -> &'static [[f32; 3]] {
        match self {
            Stencil::Central6 => &[
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0],
            ],
            Stencil::Tetra4 => &[
                [1.0, -1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [-1.0, 1.0, -1.0],
                [1.0, 1.0, 1.0],
            ],
        }
    }
}

/// **Tudo o que uma marcha precisa de saber, e que não muda entre lotes.**
///
/// Os quatro viajam sempre juntos — a árvore compilada, a câmera, a base dela e as tolerâncias do
/// quadro. Passá-los soltos era o que fazia as duas funções abaixo crescerem para oito parâmetros,
/// e uma lista de oito é onde dois deles trocam de lugar sem o compilador reparar.
pub(crate) struct Scene<'a> {
    pub(crate) shape: &'a ph2d_field_eval::hybrid::Hybrid,
    pub(crate) cam: &'a Orbit,
    pub(crate) basis: ([f32; 3], [f32; 3], [f32; 3]),
    pub(crate) sharp: Sharpness,
    /// ⭐⭐ **A caixa a que a marcha se prende** (W56) — `None` = a marcha de sempre, do plano da
    /// câmera até `T_MAX`.
    ///
    /// ⚠️ **É ela que torna a árvore especializada VÁLIDA em todo ponto avaliado.** Com o recorte, o
    /// raio começa na **entrada** da caixa e pára na saída dela: nenhuma amostra cai fora, e a
    /// especialização — que só vale dentro da região — nunca é perguntada onde ela mente.
    ///
    /// ⭐ E ela paga-se sozinha: os passos de aproximação em espaço vazio deixam de existir.
    pub(crate) clip: Option<([f32; 3], [f32; 3])>,
    /// ⭐⭐ **A fracção da distância que o raio anda** — ver
    /// [`ph2d_field_eval::safe_march_step`], que é quem a deriva.
    ///
    /// ⚠️ Ela viaja na cena, e não é a constante lida directamente, porque a pergunta *"que passo é
    /// seguro?"* é do **documento**: um campo sem operador que infle o gradiente é uma distância
    /// verdadeira, e nele `1,0` é seguro. Medir as duas respostas em processos diferentes não é
    /// medir — a montagem, que não depende disto, mexeu-se `14,4 -> 22,1 ms` entre duas corridas.
    pub(crate) step: f32,
    /// ⭐⭐ **Com que estêncil a normal é lida** — ver [`Stencil`].
    ///
    /// ⚠️ Ele viaja na cena pela mesma razão que o [`Scene::step`]: a pergunta *"quantas amostras
    /// custa uma normal?"* tem de ser respondida **uma vez por quadro** e ser a mesma nas duas
    /// marchas (a linha inteira e as quatro amostras de um pixel de borda). Uma segunda fonte
    /// faria a silhueta re-amostrada ler a normal por outra lei que o interior.
    pub(crate) stencil: Stencil,
}

/// **O núcleo**: marcha um lote arbitrário de raios e devolve `(acertou, normal de vista)`.
///
/// Recebe posições no **plano da câmera** (unidades de mundo), e não índices de pixel, e é isso que
/// o deixa servir às duas passagens — a linha inteira e as quatro amostras espalhadas de um pixel
/// de borda. *Uma marcha, um lugar.*
pub(crate) fn march(
    scene: &Scene<'_>,
    screen: &[(f32, f32)],
) -> (Vec<bool>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
    march_slabs(scene, screen, &[0.0, T_MAX], &mut |_| None)
}

/// ⭐⭐⭐ **O NÚCLEO, EM FATIAS DE PROFUNDIDADE** (W56e) — e a marcha de sempre é o caso `N = 1`.
///
/// # Por que a profundidade é a única dimensão que sobra
///
/// ⛔ **Medido (W56e):** o corte por distância guarda toda aresta a menos de
/// `dmax = min_e (máx distância de um CANTO da região a e)`, e para uma região de diâmetro `D` com
/// a aresta mais próxima a `a` isso é `≈ a + D`. Com `D` da ordem da peça, ele **guarda tudo** — e
/// a região do ladrilho é o tubo do frustum, que atravessa a peça inteira. Medido em três
/// contornos: círculo `77%` guardadas, **estrela `86%`**, pente `80%`. *O corte não é fraco porque
/// a peça é redonda: ele é fraco porque a região é grande.*
///
/// ⚠️ **E encolher o LADO do ladrilho não encolhe a região** — a varredura do [`tiles::TILE`] viu
/// um vale, não uma descida, e é por isto: a região mede `lado + profundidade · |direcção|`, e o
/// segundo termo não sabe o lado. **Só a profundidade sobra.**
///
/// # O que `bounds` e `shape_of` são
///
/// `bounds` são as `k + 1` fronteiras em `t`, da frente para trás. `shape_of(k)` entrega a árvore
/// da fatia `k` — e é chamado **só quando algum raio de facto lá chega**, que é o que impede a
/// montagem de crescer com `N` para quem acerta na primeira. `None` = usar [`Scene::shape`].
///
/// ⚠️ **A normal é avaliada na fita DA FATIA em que o raio parou**, antes de ela morrer. Uma
/// segunda passagem no fim teria de reconstruir todas as fatias — e a fita de outra fatia responde
/// onde não vale (a sonda da normal é `ponto ± ε`, ver [`tiles::tile_region`]).
pub(crate) fn march_slabs(
    scene: &Scene<'_>,
    screen: &[(f32, f32)],
    bounds: &[f32],
    shape_of: &mut dyn FnMut(usize) -> Option<ph2d_field_eval::hybrid::Hybrid>,
) -> (Vec<bool>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let (cam, sharp) = (scene.cam, scene.sharp);
    let n = screen.len();
    let mut hit = vec![false; n];
    let mut normal = vec![[0.0f32; 3]; n];
    // ⭐ **Onde o raio parou, no MUNDO.** Ele sai de graça (a marcha já sabe o `t`), e é o que uma
    // seleção por clique precisa de saber. Devolvê-lo aqui é o que impede uma segunda marcha de
    // existir só para responder à mesma pergunta.
    let mut point = vec![[0.0f32; 3]; n];
    if n == 0 || bounds.len() < 2 {
        return (hit, normal, point);
    }

    // ⭐ **O raio vem da CÂMERA**, e não de uma segunda cópia da conta dela. Este laço reconstruía
    // a aritmética do `Orbit::ray` com um afastamento próprio — duas respostas para *"que raio sai
    // daqui?"*, no mesmo módulo cujo doc promete que a projeção é a mesma do gizmo. Com a lente
    // convergente a direção passou a ser **por raio**, e uma das duas cópias teria ficado paralela.
    let (mut ox, mut oy, mut oz) = (vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]);
    let mut dir = vec![[0.0f32; 3]; n];
    for (i, &(sx, sy)) in screen.iter().enumerate() {
        let (o, d) = cam.ray_at_plane(sx, sy);
        (ox[i], oy[i], oz[i]) = (o[0], o[1], o[2]);
        dir[i] = d;
    }

    let mut t = vec![0.0f32; n];
    // ⭐ **Cada raio entra e sai da caixa** — ver [`Scene::clip`]. Sem recorte, a marcha de sempre.
    let mut t_end = vec![T_MAX; n];
    let mut alive: Vec<u32> = Vec::with_capacity(n);
    for i in 0..n {
        match scene.clip {
            None => alive.push(i as u32),
            Some((lo, hi)) => {
                let o = [ox[i], oy[i], oz[i]];
                if let Some((a, b)) = slab(o, dir[i], lo, hi) {
                    t[i] = a.max(0.0);
                    t_end[i] = b.min(T_MAX);
                    if t[i] < t_end[i] {
                        alive.push(i as u32);
                    }
                }
            }
        }
    }

    MARCH_RAYS.fetch_add(alive.len() as u64, std::sync::atomic::Ordering::Relaxed);

    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    let mut landed: Vec<u32> = Vec::new();
    for k in 0..bounds.len() - 1 {
        if alive.is_empty() {
            break;
        }
        let to = bounds[k + 1];
        // Quem tem trabalho NESTA fatia — e `carry` recolhe quem sai dela pela frente.
        let mut carry: Vec<u32> = Vec::new();
        let mut cur: Vec<u32> = Vec::with_capacity(alive.len());
        for &i in &alive {
            let iu = i as usize;
            if t[iu] < to.min(t_end[iu]) {
                cur.push(i);
            } else if t[iu] < t_end[iu] {
                carry.push(i);
            }
        }
        if cur.is_empty() {
            alive = carry;
            continue;
        }
        // ⚠️ **Só agora** se monta a árvore — a montagem é 96% JIT (medido: 2 334 µs de 2 430 por
        // ladrilho a 168 arestas) e é ela que paga o preço de `N`.
        // Um avaliador POR LOTE: a `fidget` precisa de estado mutável para avaliar, e partilhá-lo
        // entre threads exigiria trava. Criar o próprio mantém a escrita disjunta, que é a condição
        // do ADR-0109.
        //
        // ⭐⭐⭐ **Mas a fita ESPECIALIZADA já é nossa, e forká-la era montá-la duas vezes** (W70).
        // O `shape_of(k)` devolve um `Hybrid` acabado de construir para esta fatia, que ninguém
        // mais vê; o `fork` dele compilava **outra** fita idêntica (medido: `2,89 ms` contra os
        // `2,85` da construção) e deitava a primeira fora. *Um `fork` é para partilhar o que é de
        // outro; o que já é nosso avalia-se directamente.* Só o caminho não especializado forka —
        // ali o `scene.shape` **é** partilhado entre as threads do lote.
        let mut eval = match shape_of(k) {
            Some(s) => s,
            None => {
                FORKED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                scene.shape.fork()
            }
        };
        landed.clear();
        for step in 0..MAX_STEPS {
            if cur.is_empty() {
                break;
            }
            STEP_HIST[step.min(HIST - 1)]
                .fetch_add(cur.len() as u64, std::sync::atomic::Ordering::Relaxed);
            SLAB_SAMPLES[k.min(SLABS_COUNTED - 1)]
                .fetch_add(cur.len() as u64, std::sync::atomic::Ordering::Relaxed);
            xs.clear();
            ys.clear();
            zs.clear();
            for &i in &cur {
                let i = i as usize;
                xs.push(ox[i] + dir[i][0] * t[i]);
                ys.push(oy[i] + dir[i][1] * t[i]);
                zs.push(oz[i] + dir[i][2] * t[i]);
            }
            STEP_SAMPLES.fetch_add(cur.len() as u64, std::sync::atomic::Ordering::Relaxed);
            let Ok(out) = eval.eval(&xs, &ys, &zs) else {
                break;
            };
            let mut next = Vec::with_capacity(cur.len());
            for (j, &i) in cur.iter().enumerate() {
                let iu = i as usize;
                let d = out[j];
                if d < sharp.hit {
                    hit[iu] = true;
                    landed.push(i);
                    continue;
                }
                let lim = to.min(t_end[iu]);
                t[iu] += d * scene.step;
                if t[iu] >= lim {
                    // ⭐ **Nunca grampeado — ele passa, e a fatia certa recolhe-o.** A 1.ª versão
                    // punha `t = lim` "para não sair da região", e uma mutação que o apagou
                    // SOBREVIVEU: o passo é a **distância verdadeira**, então nada existe no
                    // intervalo saltado, e o filtro da fatia seguinte (`t < bounds[k+1]`) manda o
                    // raio para a fatia que de facto o contém — saltando por cima das que ele
                    // atravessou de uma vez, **sem as montar**. *Uma guarda que nenhuma mutação
                    // mata estava a comprar uma avaliação por travessia e nada mais.*
                    if lim < t_end[iu] {
                        carry.push(i);
                    }
                    continue;
                }
                next.push(i);
            }
            cur = next;
        }
        normals_into(
            &mut eval,
            scene,
            &landed,
            &ox,
            &oy,
            &oz,
            &dir,
            &t,
            &mut hit,
            &mut normal,
            &mut point,
        );
        alive = carry;
    }
    (hit, normal, point)
}

/// As normais por diferença central dos raios que acertaram **nesta fatia** — ver [`march_slabs`].
#[allow(clippy::too_many_arguments)]
fn normals_into(
    eval: &mut ph2d_field_eval::hybrid::Hybrid,
    scene: &Scene<'_>,
    idx: &[u32],
    ox: &[f32],
    oy: &[f32],
    oz: &[f32],
    dir: &[[f32; 3]],
    t: &[f32],
    hit: &mut [bool],
    normal: &mut [[f32; 3]],
    point: &mut [[f32; 3]],
) {
    if idx.is_empty() {
        return;
    }
    let offs = scene.stencil.offsets();
    NORMAL_SAMPLES.fetch_add(
        idx.len() as u64 * offs.len() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
    let (right, up, fwd) = scene.basis;
    for &i in idx {
        let i = i as usize;
        point[i] = [
            ox[i] + dir[i][0] * t[i],
            oy[i] + dir[i][1] * t[i],
            oz[i] + dir[i][2] * t[i],
        ];
    }
    let mut gx = Vec::with_capacity(idx.len() * offs.len());
    let mut gy = Vec::with_capacity(idx.len() * offs.len());
    let mut gz = Vec::with_capacity(idx.len() * offs.len());
    for &i in idx {
        let [px, py, pz] = point[i as usize];
        let e = scene.sharp.normal;
        for d in offs {
            gx.push(d[0].mul_add(e, px));
            gy.push(d[1].mul_add(e, py));
            gz.push(d[2].mul_add(e, pz));
        }
    }
    if let Ok(g) = eval.eval(&gx, &gy, &gz) {
        for (k, &i) in idx.iter().enumerate() {
            let i = i as usize;
            let b = k * offs.len();
            // ⭐ **A soma que é as duas leis** — ver [`Stencil`]. Na diferença central os pesos
            // nulos somam zeros exactos e o resultado é o `[g₀−g₁, …]` de sempre.
            let mut world = [0.0f32; 3];
            for (j, d) in offs.iter().enumerate() {
                let v = g[b + j];
                world[0] = d[0].mul_add(v, world[0]);
                world[1] = d[1].mul_add(v, world[1]);
                world[2] = d[2].mul_add(v, world[2]);
            }
            let len = (world[0] * world[0] + world[1] * world[1] + world[2] * world[2]).sqrt();
            if len <= 0.0 {
                hit[i] = false;
                continue;
            }
            let nrm = [world[0] / len, world[1] / len, world[2] / len];
            // Para o espaço de VISTA — é nele que o matcap vive.
            normal[i] = [
                nrm[0] * right[0] + nrm[1] * right[1] + nrm[2] * right[2],
                nrm[0] * up[0] + nrm[1] * up[1] + nrm[2] * up[2],
                nrm[0] * fwd[0] + nrm[1] * fwd[1] + nrm[2] * fwd[2],
            ];
        }
    }
}
