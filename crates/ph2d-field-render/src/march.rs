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
            None => scene.shape.fork(),
        };
        landed.clear();
        for _ in 0..MAX_STEPS {
            if cur.is_empty() {
                break;
            }
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
    let (right, up, fwd) = scene.basis;
    for &i in idx {
        let i = i as usize;
        point[i] = [
            ox[i] + dir[i][0] * t[i],
            oy[i] + dir[i][1] * t[i],
            oz[i] + dir[i][2] * t[i],
        ];
    }
    let mut gx = Vec::with_capacity(idx.len() * 6);
    let mut gy = Vec::with_capacity(idx.len() * 6);
    let mut gz = Vec::with_capacity(idx.len() * 6);
    for &i in idx {
        let [px, py, pz] = point[i as usize];
        let e = scene.sharp.normal;
        for (dx, dy, dz) in [
            (e, 0.0, 0.0),
            (-e, 0.0, 0.0),
            (0.0, e, 0.0),
            (0.0, -e, 0.0),
            (0.0, 0.0, e),
            (0.0, 0.0, -e),
        ] {
            gx.push(px + dx);
            gy.push(py + dy);
            gz.push(pz + dz);
        }
    }
    if let Ok(g) = eval.eval(&gx, &gy, &gz) {
        for (k, &i) in idx.iter().enumerate() {
            let i = i as usize;
            let b = k * 6;
            let world = [g[b] - g[b + 1], g[b + 2] - g[b + 3], g[b + 4] - g[b + 5]];
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
