//! ⭐⭐⭐ **O ATLAS** — o mapa de grade da [`ph2d_gridmap`] arrumado em `[0,1]²`.
//!
//! # O que esta crate acrescenta, e o que ela NÃO refaz
//!
//! A `ph2d-gridmap` já resolve `(u, v)` sobre a peça inteira, com as costuras acopladas.
//! O que falta a uma TEXTURA é outra coisa, e são três passos de aritmética:
//!
//! | passo | porquê |
//! |---|---|
//! | **juntar** os patches em ILHAS | ⛔ *um patch não é uma ilha* — onde o salto de período é `0 (mod 4)` os dois lados leem a mesma função a menos de uma translação, e ali não há corte nenhum |
//! | **assentar** cada ilha num plano só | cada patch traz o seu `(u, v)` numa origem própria; pô-los lado a lado é acumular a translação ao longo de uma árvore |
//! | **arrumar** as ilhas em `[0,1]²` | é o que um sampler pede, e é onde o desperdício mora |
//!
//! ⭐ **O tamanho do trabalho foi MEDIDO antes da primeira linha**
//! (`docs/3D/26_a_parametrizacao_como_atlas.md`): nas peças do dono são **`4` a `13`**
//! ilhas sobre `88`–`116` patches, com metade do comprimento das costuras a ser corte de
//! verdade. *Sem essa medição eu teria escrito um empacotador para cem rectângulos.*
//!
//! # ⚠️ A premissa do assentamento, e ela é GATEADA
//!
//! Numa costura colada (`jump ≡ 0`) a relação entre os dois lados é uma **translação
//! constante ao longo da cadeia** — é isso que permite somar um deslocamento por patch em
//! vez de um por vértice. [`Relatorio::cola_max`] mede a dispersão dessa translação e
//! [`Relatorio::holonomia_max`] mede o que sobra ao fechar um ciclo dentro de uma ilha.
//! *Sem as duas, «as ilhas ficaram bem» é uma afirmação sobre uma imagem que ninguém viu.*

use ph2d_gridmap::{CutMesh, GridMap};
use ph2d_mesh::Mesh;

/// ⭐ **O VÃO entre duas ilhas, em texels de uma textura de referência.**
///
/// ⚠️ O recurso tem nome: **a cadeia de mips**. Cada nível divide a textura por dois, logo
/// um vão de `2^k` texels é o que sobrevive a `k` níveis antes de duas ilhas se misturarem
/// — `8` sobrevive a três. ⛔ Ele **não** é o raio da dilatação de costura (essa é outra
/// wave e vive do lado da textura); é o espaço que a dilatação vai ter para correr.
pub const VAO_EM_TEXELS: f32 = 8.0;

/// A textura de referência em que [`VAO_EM_TEXELS`] é contado.
///
/// ⚠️ **Medida e não escolhida:** a `§5` do doc 26 mede que a `2048²` um texel vale
/// `1/25`–`1/33` do quad que a retopologia pede nas três peças do dono.
pub const TEXTURA_DE_REFERENCIA: f32 = 2048.0;

/// O que o atlas mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Relatorio {
    /// Ilhas arrumadas.
    pub ilhas: usize,
    /// Patches que entraram.
    pub patches: usize,
    /// Costuras coladas (a transição não roda) e rodadas (o corte de verdade).
    pub coladas: usize,
    /// Ver [`Self::coladas`].
    pub rodadas: usize,
    /// ⛔⛔ **A dispersão da translação DENTRO de uma costura colada**, em células de
    /// grade. A premissa do assentamento é que ela é constante; isto mede-a.
    pub cola_max: f32,
    /// ⭐⭐⭐ **Costuras coladas que FECHAM UM CICLO — os cortes que o atlas teve de fazer
    /// por sua conta.**
    ///
    /// ⚠️ **Elas não são um defeito, são o preço de uma superfície não ser plana:** o
    /// assentamento percorre uma ÁRVORE de patches, e toda costura colada que sobra depois
    /// da árvore é uma aresta que não cabe. *Uma esfera não se desenrola sem um corte, e
    /// isto é quantos ela precisou.*
    ///
    /// ⛔ O que seria um defeito é elas existirem e ninguém as contar — aí a [`Self::holonomia_max`]
    /// lê-se como *«o solver falhou»* em vez de *«a peça tem género»*.
    pub ciclos: usize,
    /// ⛔⛔ **O RASGO na pior dessas costuras**, em células de grade. É a distância a que
    /// os dois lados de um ciclo ficam um do outro depois de a árvore os assentar.
    pub holonomia_max: f32,
    /// A fracção do quadrado que as caixas das ilhas ocupam.
    pub aproveitamento: f32,
    /// ⛔ Cantos da malha que não receberam `(u, v)`.
    ///
    /// ⚠️ **É o piso de população desta crate:** sem ele um atlas vazio lê-se como um
    /// atlas perfeito, que é o defeito que a sonda do doc 26 apanhou em si mesma.
    pub orfaos: usize,
    /// Cantos que receberam.
    pub cantos: usize,
}

/// ⭐ O atlas: um `(u, v)` por CANTO da malha.
///
/// ⚠️ **Por canto e não por vértice, e a razão é a costura:** um vértice sobre um corte
/// tem `(u, v)` diferente de cada lado, e um plano por-vértice não o sabe dizer. *O canto
/// é a menor unidade em que um atlas é exprimível.*
#[derive(Debug, Clone, Default)]
pub struct Atlas {
    /// Por canto (faces em ordem, os vértices de cada face em ordem), o `(u, v)`.
    pub uv: Vec<[f32; 2]>,
    /// Por canto, a ilha a que ele pertence — é o que dá cor a um desenho do atlas.
    pub ilha: Vec<u32>,
    /// Ver [`Relatorio`].
    pub relatorio: Relatorio,
}

/// O índice do primeiro canto de cada face, e o total.
///
/// ⭐ **É a única definição de «canto» desta casa**, e por isso ela é pública: quem
/// consumir o atlas tem de indexá-lo do mesmo jeito, e duas contagens divergem no dia em
/// que alguém escrever um quad.
#[must_use]
pub fn bases_dos_cantos(mesh: &Mesh) -> (Vec<u32>, usize) {
    let mut base = Vec::with_capacity(mesh.faces().len());
    let mut n = 0usize;
    for f in mesh.faces() {
        base.push(u32::try_from(n).unwrap_or(u32::MAX));
        n += f.verts().len();
    }
    (base, n)
}

/// Raiz de um conjunto disjunto, com compressão de caminho.
fn raiz(pai: &mut [usize], mut x: usize) -> usize {
    while pai[x] != x {
        pai[x] = pai[pai[x]];
        x = pai[x];
    }
    x
}

/// A translação de uma costura colada, e a dispersão dela ao longo da cadeia.
///
/// ⚠️ Devolve `None` quando nenhuma posição da cadeia tem os dois lados — *uma costura sem
/// par não é uma translação de zero, é uma ausência*, e somá-la como zero colaria duas
/// ilhas por engano.
fn cola(map: &GridMap, seam: &ph2d_gridmap::Seam) -> Option<([f32; 2], f32)> {
    let (a, b) = (&seam.side[0], &seam.side[1]);
    let (uva, uvb) = (map.uv.get(a.patch as usize)?, map.uv.get(b.patch as usize)?);
    let mut soma = [0.0f64, 0.0];
    let mut n = 0usize;
    let mut ts: Vec<[f32; 2]> = Vec::new();
    for k in 0..a.local.len().min(b.local.len()) {
        let (Some(la), Some(lb)) = (a.local[k], b.local[k]) else {
            continue;
        };
        let (Some(&za), Some(&zb)) = (uva.get(la as usize), uvb.get(lb as usize)) else {
            continue;
        };
        let t = [zb[0] - za[0], zb[1] - za[1]];
        soma[0] += f64::from(t[0]);
        soma[1] += f64::from(t[1]);
        n += 1;
        ts.push(t);
    }
    if n == 0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    let media = [(soma[0] / n as f64) as f32, (soma[1] / n as f64) as f32];
    let disp = ts
        .iter()
        .map(|t| {
            let d = [t[0] - media[0], t[1] - media[1]];
            d[0].mul_add(d[0], d[1] * d[1]).sqrt()
        })
        .fold(0.0f32, f32::max);
    Some((media, disp))
}

/// ⭐⭐⭐ **Constrói o atlas.**
///
/// As entradas são o que a cadeia já produz — a malha **triangulada**, o corte, o mapa
/// contínuo e os saltos de período. ⛔ *Esta crate não corre o campo nem o traçado*: ela
/// não os conhece, e assim não pode medir um programa diferente do que o chamador correu.
///
/// # Panics
/// Nunca: toda ausência vira [`Relatorio::orfaos`] ou uma ilha própria.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build(mesh: &Mesh, cut: &CutMesh, map: &GridMap, jumps: &[Option<i32>]) -> Atlas {
    let np = cut.origin.len();
    let mut rel = Relatorio {
        patches: np,
        ..Relatorio::default()
    };

    // ── 1. As ilhas, e a translação de cada costura colada.
    let mut pai: Vec<usize> = (0..np).collect();
    // Por costura colada: `(patch_a, patch_b, translação)`.
    let mut colas: Vec<(usize, usize, [f32; 2])> = Vec::new();
    for (s, seam) in cut.seams.iter().enumerate() {
        let colada = matches!(jumps.get(s), Some(Some(j)) if j.rem_euclid(4) == 0);
        if !colada {
            rel.rodadas += 1;
            continue;
        }
        let Some((t, disp)) = cola(map, seam) else {
            rel.rodadas += 1;
            continue;
        };
        rel.coladas += 1;
        rel.cola_max = rel.cola_max.max(disp);
        let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        colas.push((pa, pb, t));
        let (ra, rb) = (raiz(&mut pai, pa), raiz(&mut pai, pb));
        if ra == rb {
            // ⭐ A aresta que a árvore não usa — ver [`Relatorio::ciclos`].
            rel.ciclos += 1;
        } else {
            pai[ra] = rb;
        }
    }

    // ── 2. O assentamento: um deslocamento por patch, acumulado por travessia.
    //
    // ⚠️ A ordem é por COSTURA e não por patch, e repete-se até estabilizar: uma travessia
    // em largura escrita à mão precisaria da lista de vizinhos, que é a mesma informação
    // por outro caminho. *Duas respostas à mesma pergunta divergem.*
    let mut off = vec![None::<[f32; 2]>; np];
    for (p, o) in off.iter_mut().enumerate() {
        if raiz(&mut pai, p) == p {
            *o = Some([0.0, 0.0]);
        }
    }
    let mut mexeu = true;
    while mexeu {
        mexeu = false;
        for &(pa, pb, t) in &colas {
            match (off[pa], off[pb]) {
                (Some(oa), None) => {
                    // `uv_b = uv_a + t` ⇒ para o lado B cair no plano de A, ele desloca-se
                    // de `oa − t`.
                    off[pb] = Some([oa[0] - t[0], oa[1] - t[1]]);
                    mexeu = true;
                }
                (None, Some(ob)) => {
                    off[pa] = Some([ob[0] + t[0], ob[1] + t[1]]);
                    mexeu = true;
                }
                _ => {}
            }
        }
    }
    // ⛔⛔ **A HOLONOMIA mede-se nos PONTOS ASSENTES, nunca na fórmula que os assentou.**
    //
    // A 1.ª redacção comparava `oa − t` com `ob`, que é literalmente a expressão do laço
    // acima — e uma mutação que trocava o SINAL do assentamento **SOBREVIVEU**, porque a
    // régua errava do mesmo lado. *Um espelho não acusa.* Hoje pergunta-se a coisa que
    // interessa: com as cartas postas no plano da ilha, os dois lados de uma costura
    // colada caem no MESMO ponto?
    for (s, seam) in cut.seams.iter().enumerate() {
        if !matches!(jumps.get(s), Some(Some(j)) if j.rem_euclid(4) == 0) {
            continue;
        }
        let (a, b) = (&seam.side[0], &seam.side[1]);
        let (pa, pb) = (a.patch as usize, b.patch as usize);
        let (Some(oa), Some(ob)) = (off[pa], off[pb]) else {
            continue;
        };
        let (Some(uva), Some(uvb)) = (map.uv.get(pa), map.uv.get(pb)) else {
            continue;
        };
        for k in 0..a.local.len().min(b.local.len()) {
            let (Some(la), Some(lb)) = (a.local[k], b.local[k]) else {
                continue;
            };
            let (Some(&za), Some(&zb)) = (uva.get(la as usize), uvb.get(lb as usize)) else {
                continue;
            };
            let d = [
                (za[0] + oa[0]) - (zb[0] + ob[0]),
                (za[1] + oa[1]) - (zb[1] + ob[1]),
            ];
            rel.holonomia_max = rel
                .holonomia_max
                .max(d[0].mul_add(d[0], d[1] * d[1]).sqrt());
        }
    }
    // Um patch que nenhuma cola alcançou é uma ilha só dele.
    for o in &mut off {
        if o.is_none() {
            *o = Some([0.0, 0.0]);
        }
    }

    // ── 3. A caixa de cada ilha, no plano dela.
    let mut ilha_de = vec![0u32; np];
    let mut ordem: Vec<usize> = Vec::new();
    for (p, slot) in ilha_de.iter_mut().enumerate() {
        let r = raiz(&mut pai, p);
        if let Some(i) = ordem.iter().position(|&q| q == r) {
            *slot = u32::try_from(i).unwrap_or(0);
        } else {
            *slot = u32::try_from(ordem.len()).unwrap_or(0);
            ordem.push(r);
        }
    }
    rel.ilhas = ordem.len();
    let mut lo = vec![[f32::MAX; 2]; rel.ilhas];
    let mut hi = vec![[f32::MIN; 2]; rel.ilhas];
    for p in 0..np {
        let i = ilha_de[p] as usize;
        let o = off[p].unwrap_or([0.0, 0.0]);
        for z in &map.uv[p] {
            let q = [z[0] + o[0], z[1] + o[1]];
            lo[i][0] = lo[i][0].min(q[0]);
            lo[i][1] = lo[i][1].min(q[1]);
            hi[i][0] = hi[i][0].max(q[0]);
            hi[i][1] = hi[i][1].max(q[1]);
        }
    }

    // ── 4. Arrumar: prateleiras, mais altas primeiro, com o vão do mip.
    let fraccao = VAO_EM_TEXELS / TEXTURA_DE_REFERENCIA;
    let mut tam: Vec<(usize, f32, f32)> = (0..rel.ilhas)
        .map(|i| {
            let (w, h) = if lo[i][0] <= hi[i][0] {
                (hi[i][0] - lo[i][0], hi[i][1] - lo[i][1])
            } else {
                (0.0, 0.0)
            };
            (i, w, h)
        })
        .collect();
    tam.sort_by(|a, b| b.2.total_cmp(&a.2));
    let area: f32 = tam.iter().map(|t| t.1 * t.2).sum();
    let mut lado = area.sqrt().max(1.0e-6);
    let mut pos = vec![[0.0f32, 0.0]; rel.ilhas];
    // ⚠️ O laço CRESCE o quadrado até caber. *Um empacotador que devolve «não coube» a
    // quem lhe deu rectângulos não resolveu nada* — e o preço de crescer é medido pelo
    // aproveitamento, que é a coluna que este relatório publica.
    for _ in 0..200 {
        let vao = lado * fraccao;
        let (mut x, mut y, mut alt) = (vao, vao, 0.0f32);
        let mut coube = true;
        for &(i, w, h) in &tam {
            // ⛔⛔ **A 1.ª redacção não tinha esta linha, e o gate do quadrado apanhou-a:**
            // sem ela uma ilha mais LARGA que o quadrado era «colocada» na primeira
            // prateleira e o laço declarava que coube — a fixtura de uma ilha só (`2 × 1`
            // num quadrado de `1,41`) saía com `u = 1,42`. *Um empacotador que só verifica
            // a altura mede metade do problema.*
            if w + 2.0 * vao > lado || h + 2.0 * vao > lado {
                coube = false;
                break;
            }
            if x + w + vao > lado {
                x = vao;
                y += alt + vao;
                alt = 0.0;
            }
            if y + h + vao > lado {
                coube = false;
                break;
            }
            pos[i] = [x, y];
            x += w + vao;
            alt = alt.max(h);
        }
        if coube {
            break;
        }
        lado *= 1.05;
    }
    rel.aproveitamento = if lado > 0.0 {
        area / (lado * lado)
    } else {
        0.0
    };

    // ── 5. O `(u, v)` de cada canto, em `[0,1]²`.
    let (base, ncantos) = bases_dos_cantos(mesh);
    let mut uv = vec![[0.0f32, 0.0]; ncantos];
    let mut ilha = vec![u32::MAX; ncantos];
    let mut posto = vec![false; ncantos];
    for (p, tris) in cut.tris.iter().enumerate() {
        let i = ilha_de[p] as usize;
        let o = off[p].unwrap_or([0.0, 0.0]);
        let (px, py) = (pos[i][0], pos[i][1]);
        let (lx, ly) = (lo[i][0], lo[i][1]);
        for (ti, t) in tris.iter().enumerate() {
            let Some(&fi) = cut.tri_face[p].get(ti) else {
                continue;
            };
            let Some(face) = mesh.faces().get(fi as usize) else {
                continue;
            };
            let verts = face.verts();
            for &l in t {
                let Some(&g) = cut.origin[p].get(l as usize) else {
                    continue;
                };
                let Some(k) = verts.iter().position(|&v| v == g) else {
                    continue;
                };
                let Some(&z) = map.uv[p].get(l as usize) else {
                    continue;
                };
                let c = base[fi as usize] as usize + k;
                if c >= ncantos {
                    continue;
                }
                uv[c] = [
                    (z[0] + o[0] - lx + px) / lado,
                    (z[1] + o[1] - ly + py) / lado,
                ];
                ilha[c] = u32::try_from(i).unwrap_or(u32::MAX);
                posto[c] = true;
            }
        }
    }
    rel.cantos = posto.iter().filter(|&&b| b).count();
    rel.orfaos = ncantos - rel.cantos;

    Atlas {
        uv,
        ilha,
        relatorio: rel,
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
