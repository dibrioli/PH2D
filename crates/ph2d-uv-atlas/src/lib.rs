//! ⭐⭐⭐ **O ATLAS** — o mapa de grade da [`ph2d_gridmap`] arrumado em `[0,1]²`.
//!
//! # O que esta crate acrescenta, e o que ela NÃO refaz
//!
//! A `ph2d-gridmap` já resolve `(u, v)` sobre a peça inteira, com as costuras acopladas.
//! O que falta a uma TEXTURA é outra coisa, e são quatro passos:
//!
//! | passo | porquê |
//! |---|---|
//! | **juntar** os patches em ILHAS | ⛔ *um patch não é uma ilha* — onde o salto de período é `0 (mod 4)` os dois lados leem a mesma função a menos de uma translação, e ali não há corte nenhum |
//! | **assentar** cada ilha num plano só | cada patch traz o seu `(u, v)` numa origem própria; pô-los lado a lado é acumular a translação ao longo de uma árvore |
//! | ⭐ **cortar** a ilha que se dobra | ⛔ *uma ilha assentada ao longo de uma árvore pode cair em cima de si mesma*, e aí a tinta aparece em dois sítios — ver [`corte`] |
//! | **arrumar** as peças em `[0,1]²` | é o que um sampler pede, e é onde o desperdício mora |
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

pub mod arrumacao;
pub mod corte;
pub mod empacota;
pub mod orienta;
pub mod sobreposicao;
pub mod topo;

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
    ///
    /// ⚠️ **Com [`Opcoes::colar`] desligada ele é `0` porque NÃO FOI MEDIDO**, e quem o
    /// separa de um `0` de «assentou perfeito» é a [`Self::coladas`] ao lado, que lê `0`
    /// também. *Um zero de «não medido» e um de «perfeito» são o mesmo byte.*
    pub holonomia_max: f32,
    /// ⭐⭐⭐ **Ilhas ANTES do corte** — as componentes ligadas das costuras coladas.
    ///
    /// ⚠️ [`Self::ilhas`] conta o que o empacotador arrumou, que é o que o artista vê;
    /// esta conta o que a superfície dava. *A diferença é o preço do corte, e sem as duas
    /// colunas ele não é observável.*
    pub ilhas_antes_do_corte: usize,
    /// Peças com **uma face só** — ver [`corte::Corte::pecas_de_uma_face`].
    pub pecas_de_uma_face: usize,
    /// Quantas vezes o corte recusou uma face — ver [`corte::Corte::recusas`].
    pub recusas_do_corte: usize,
    /// ⛔ Ver [`corte::Corte::faces_sem_vizinho`].
    pub faces_sem_vizinho: usize,
    /// Pares de peças que a fusão juntou — ver [`corte::Corte::fusoes`].
    pub fusoes_do_corte: usize,
    /// A mediana do tamanho de uma peça, em faces.
    pub peca_p50: usize,
    /// O maior.
    pub peca_max: usize,
    /// ⭐⭐⭐ **A fracção do quadrado que tem TINTA.**
    ///
    /// ⛔⛔ **A PREMISSA DESTE CAMPO MORREU na W3, e a morte fica à vista:** até aqui ele
    /// contava a fracção que as **CAIXAS** ocupavam, e lia `67,7 %` numa peça em que a
    /// tinta ocupava **`18,7 %`**. *Uma régua que mede o invólucro não mede o que está lá
    /// dentro, e era esta a coluna que o relatório publicava.* A do invólucro continua,
    /// com o nome que diz o que ela é: [`Self::caixas_no_quadrado`].
    pub aproveitamento: f32,
    /// A fracção do quadrado que as CAIXAS das peças ocupam. Ver [`Self::aproveitamento`].
    pub caixas_no_quadrado: f32,
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
    /// Por canto, a CARTA (o patch do corte) de onde ele veio.
    ///
    /// ⚠️ **Uma ilha é feita de cartas, e as duas granularidades respondem a perguntas
    /// diferentes:** um cruzamento DENTRO de uma carta acusa o solver contínuo, e um
    /// entre duas cartas da mesma ilha acusa o assentamento. *Guardar só a ilha faz as
    /// duas lerem-se igual* — ver [`sobreposicao::Classe`].
    pub carta: Vec<u32>,
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

/// ⭐ **O que o atlas faz de opcional.**
///
/// ⚠️ **É uma PORTA e não uma variável de ambiente:** uma env lida dentro desta crate
/// alcançaria todo chamador e faria um gate medir a máquina em vez da lei. *Quem quiser
/// bissectar o corte passa-o aqui, e a sonda é o único sítio que o faz.*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcoes {
    /// Partir cada ilha até ela deixar de se pintar duas vezes — ver [`corte`].
    pub cortar: bool,
    /// Rodar cada peça até a caixa dela ser a mínima — ver [`orienta`].
    pub orientar: bool,
    /// Arrumar pela FORMA de cada peça e não pela caixa dela — ver [`empacota`].
    pub empacotar_por_mascara: bool,
    /// ⛔⛔⛔ **Juntar as cartas em ilhas onde a costura não roda — e ela shipa
    /// DESLIGADA, por medição.**
    ///
    /// | `sculpt_antes` CRUA | a colar | sem colar |
    /// |---|---|---|
    /// | tinta / quadrado | `31,8 %` | **`42,6 %`** |
    /// | costura que o pintor sente | `156,5` | **`129,3`** |
    /// | recusas do corte | `369` | `65` |
    /// | resíduo de cruzamento | `40` pares | **`0`** |
    ///
    /// ⇒ **colar perde nos DOIS eixos na malha do artista.** O mecanismo: ela junta `88`
    /// cartas em `4` ilhas que se enrolam pela peça, e o corte volta a retalhá-las em
    /// `129` peças — *a continuidade que a colagem compra nas fronteiras que junta, o
    /// corte paga-a de volta com juros noutro sítio*.
    ///
    /// ⚠️ Na malha remalhada é `41,0 → 47,9 %` de tinta por `+1,8 %` de costura, ou seja
    /// ganha num eixo e empata no outro.
    ///
    /// ⛔ **A porta FICA**, e não é decoração: a lei da colagem é o que diz quais costuras
    /// são cortes de verdade (§3), ela continua gateada, e o meio-termo — *colar só onde
    /// isso não obriga um corte* — é a wave seguinte e precisa dela.
    pub colar: bool,
}

impl Default for Opcoes {
    /// ⚠️ **O corte nasce LIGADO, contra a lei da casa de tudo o que é novo shipar
    /// desligado — e a razão é que aqui não há produto do outro lado.** Nenhum botão
    /// consome este atlas ainda; desligá-lo faria [`build`] devolver, por omissão, um
    /// atlas que se pinta duas vezes e que a régua da [`sobreposicao`] acusa. *Uma
    /// omissão que entrega um resultado sabidamente errado não é conservadora.*
    fn default() -> Self {
        Self {
            cortar: true,
            orientar: true,
            empacotar_por_mascara: true,
            colar: false,
        }
    }
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
    build_com(mesh, cut, map, jumps, Opcoes::default())
}

/// Ver [`build`] e [`Opcoes`].
///
/// # Panics
/// Nunca: toda ausência vira [`Relatorio::orfaos`] ou uma ilha própria.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_com(
    mesh: &Mesh,
    cut: &CutMesh,
    map: &GridMap,
    jumps: &[Option<i32>],
    opcoes: Opcoes,
) -> Atlas {
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
        let colada = opcoes.colar && matches!(jumps.get(s), Some(Some(j)) if j.rem_euclid(4) == 0);
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
    rel.holonomia_max = holonomia(cut, map, jumps, &off, opcoes.colar);

    // Um patch que nenhuma cola alcançou é uma ilha só dele.
    for o in &mut off {
        if o.is_none() {
            *o = Some([0.0, 0.0]);
        }
    }

    // ── 3. A ilha de cada carta.
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
    rel.ilhas_antes_do_corte = ordem.len();

    // ── 4. O PLANO: um `(u, v)` por canto, ainda na origem da ilha.
    //
    // ⭐ Este passo era o ÚLTIMO e passou a ser o do meio: o corte precisa de ver a ilha
    // assentada, e o empacotador precisa de ver as peças que o corte deu. *Empacotar
    // antes de cortar seria arrumar rectângulos que ainda vão mudar de tamanho.*
    let (base, ncantos) = bases_dos_cantos(mesh);
    let mut plano = vec![[0.0f32, 0.0]; ncantos];
    let mut carta = vec![u32::MAX; ncantos];
    let mut ilha = vec![u32::MAX; ncantos];
    let mut posto = vec![false; ncantos];
    for (p, tris) in cut.tris.iter().enumerate() {
        let i = ilha_de[p];
        let o = off[p].unwrap_or([0.0, 0.0]);
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
                plano[c] = [z[0] + o[0], z[1] + o[1]];
                carta[c] = u32::try_from(p).unwrap_or(u32::MAX);
                ilha[c] = i;
                posto[c] = true;
            }
        }
    }

    // ── 5. O CORTE. Ver [`corte`] — a atribuição que o encomendou está no cabeçalho de lá.
    let nfaces = mesh.faces().len();
    let mut ilha_da_face = vec![u32::MAX; nfaces];
    let mut tem_uv = vec![false; nfaces];
    for (f, face) in mesh.faces().iter().enumerate() {
        let n = face.verts().len();
        let b = base[f] as usize;
        // ⛔ **TODOS os cantos, e não «algum»:** uma face meio posta daria um triângulo
        // com um canto em `(0, 0)`, que o corte leria como geometria e o empacotador
        // esticaria a caixa da ilha inteira até à origem.
        tem_uv[f] = n >= 3 && (0..n).all(|k| posto.get(b + k).copied().unwrap_or(false));
        if tem_uv[f] {
            ilha_da_face[f] = ilha[b];
        }
    }
    let ct = if opcoes.cortar {
        corte::corta(mesh, &plano, &ilha_da_face, &tem_uv)
    } else {
        // ⭐ Sem corte, a PEÇA é a ilha: é assim que a bissecção devolve o atlas da W1
        // pelo mesmo caminho, e não por um segundo ramo que envelheceria sozinho.
        corte::Corte {
            peca_da_face: ilha_da_face.clone(),
            pecas: rel.ilhas_antes_do_corte,
            ..corte::Corte::default()
        }
    };
    rel.ilhas = ct.pecas;
    rel.pecas_de_uma_face = ct.pecas_de_uma_face;
    rel.recusas_do_corte = ct.recusas;
    rel.faces_sem_vizinho = ct.faces_sem_vizinho;
    rel.fusoes_do_corte = ct.fusoes;
    rel.peca_p50 = ct.tamanho_p50;
    rel.peca_max = ct.tamanho_max;

    // ── 6. ⭐ ORIENTAR cada peça, e só depois a caixa dela.
    //
    // ⚠️ É um movimento RÍGIDO aplicado ao plano já cortado: ele não pode criar uma
    // sobreposição que o corte tirou, e é por isso que ele vem DEPOIS. *Rodar antes de
    // cortar mudaria a partição sem mudar nada do que importa.*
    if opcoes.orientar {
        arrumacao::orienta_as_pecas(mesh, &base, &mut plano, &ct.peca_da_face, rel.ilhas);
    }

    let (lo, hi) = arrumacao::caixas(mesh, &base, &plano, &ct.peca_da_face, rel.ilhas);

    // ── 7. Arrumar.
    //
    // ⭐ Primeiro pela FORMA (ver [`empacota`]); ⚠️ e o das PRATELEIRAS fica como rede —
    // *um empacotador que devolve «não coube» não resolveu nada*, e a rede corre no dia em
    // que uma peça não caiba num quadrado ao fim de `40` crescimentos.
    let tinta = arrumacao::tinta_do_plano(mesh, &base, &plano, &ct.peca_da_face);
    let (pos, lado, caixas) = if opcoes.empacotar_por_mascara {
        arrumacao::por_mascara(mesh, &base, &plano, &ct.peca_da_face, &lo, &hi, tinta)
            .unwrap_or_else(|| arrumacao::por_prateleiras(&lo, &hi))
    } else {
        arrumacao::por_prateleiras(&lo, &hi)
    };
    let quadrado = lado * lado;
    rel.aproveitamento = if quadrado > 0.0 {
        tinta / quadrado
    } else {
        0.0
    };
    rel.caixas_no_quadrado = if quadrado > 0.0 {
        caixas / quadrado
    } else {
        0.0
    };

    // ── 8. O `(u, v)` de cada canto, em `[0,1]²`.
    let mut uv = vec![[0.0f32, 0.0]; ncantos];
    for (f, face) in mesh.faces().iter().enumerate() {
        let pi = ct.peca_da_face[f];
        let b = base[f] as usize;
        if pi == u32::MAX {
            // ⛔ Uma face que o corte não colocou não tem lugar no atlas, e os cantos dela
            // contam como ÓRFÃOS — *um canto com `(u, v)` de uma face que ninguém arrumou
            // aponta para um sítio do quadrado que não é dela*.
            for k in 0..face.verts().len() {
                if let Some(q) = posto.get_mut(b + k) {
                    *q = false;
                }
            }
            continue;
        }
        let (pi, (px, py), (lx, ly)) = (
            pi as usize,
            (pos[pi as usize][0], pos[pi as usize][1]),
            (lo[pi as usize][0], lo[pi as usize][1]),
        );
        for k in 0..face.verts().len() {
            let q = plano[b + k];
            uv[b + k] = [(q[0] - lx + px) / lado, (q[1] - ly + py) / lado];
            ilha[b + k] = u32::try_from(pi).unwrap_or(u32::MAX);
        }
    }
    rel.cantos = posto.iter().filter(|&&b| b).count();
    rel.orfaos = ncantos - rel.cantos;

    Atlas {
        uv,
        ilha,
        carta,
        relatorio: rel,
    }
}

/// ⛔⛔ **O RASGO na pior costura colada, medido nos PONTOS ASSENTES.**
///
/// ⚠️ **Nunca na fórmula que os assentou.**
///
/// A 1.ª redacção comparava `oa − t` com `ob`, que é literalmente a expressão do laço
/// do assentamento — e uma mutação que trocava o SINAL dele **SOBREVIVEU**, porque a
/// régua errava do mesmo lado. *Um espelho não acusa.* Hoje pergunta-se a coisa que
/// interessa: com as cartas postas no plano da ilha, os dois lados de uma costura
/// colada caem no MESMO ponto?
fn holonomia(
    cut: &CutMesh,
    map: &GridMap,
    jumps: &[Option<i32>],
    off: &[Option<[f32; 2]>],
    colou: bool,
) -> f32 {
    // ⛔⛔ **Sem colagem não há holonomia para medir, e devolver a distância CRUA entre
    // os dois lados seria pior que devolver nada:** ela lia `4,03e1` numa esfera, o que se
    // lê como *«o assentamento falhou»* quando a verdade é *«não houve assentamento»*.
    // ⚠️ O `0` que sai daqui só é honesto porque [`Relatorio::coladas`] o acompanha e lê
    // `0` também — *um zero de «não medido» e um de «perfeito» são o mesmo byte, e o que
    // os separa é o piso de população ao lado*.
    if !colou {
        return 0.0;
    }
    let mut maior = 0.0f32;
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
            maior = maior.max(d[0].mul_add(d[0], d[1] * d[1]).sqrt());
        }
    }
    maior
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;

#[cfg(test)]
#[path = "sobreposicao_tests.rs"]
mod sobreposicao_tests;

#[cfg(test)]
#[path = "corte_tests.rs"]
mod corte_tests;

#[cfg(test)]
#[path = "topo_tests.rs"]
mod topo_tests;

#[cfg(test)]
#[path = "orienta_tests.rs"]
mod orienta_tests;

#[cfg(test)]
#[path = "empacota_tests.rs"]
mod empacota_tests;
