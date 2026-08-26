//! **DO TRAÇADO PARA OS PATCHES** — recortar, achar cantos, cortar em arcos.
//!
//! # Os quatro passos
//!
//! 1. **Inundação**: duas faces são do mesmo patch se a aresta entre elas não é
//!    parede.
//! 2. **Fronteira**: percorrida **pivotando em torno do vértice dentro do
//!    patch**, nunca saltando de aresta em aresta.
//! 3. **Cantos**: o **ângulo interno daquele patch naquele vértice**.
//! 4. **Arcos**: a fronteira partida em todo vértice que seja canto de **algum**
//!    patch — é isso que faz nascer a junção em T.
//!
//! ⚠️ **O passo 2 é onde a versão ingénua se parte.** Andar de aresta em aresta
//! escolhendo *"a próxima que ainda não usei"* é ambíguo num vértice onde quatro
//! arestas de fronteira se cruzam, e ali o laço salta para o patch errado — sem
//! erro nenhum, só com uma fronteira que descreve outra coisa. Pivotar dentro do
//! patch não tem essa escolha para fazer.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ph2d_mesh::Mesh;

use crate::TraceReport;
use crate::walk::Walls;

/// ⚠️ **Quantos quartos de volta um vértice de fronteira tem de valer para NÃO
/// ser canto.** Um lado reto passa `180°`, que são **dois** quartos; qualquer
/// outra contagem é uma quina.
pub(crate) const FLAT_QUARTERS: i32 = 2;

/// **A DECOMPOSIÇÃO** — o que o F4 vai consumir.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchLayout {
    /// Por face, o patch a que ela pertence.
    pub face_patch: Vec<u32>,
    /// ⭐⭐⭐ **A DIREÇÃO DA CRUZ em cada face da malha** — o campo que produziu
    /// este layout, a viajar com ele.
    ///
    /// ⛔ **Ele está aqui, e não num parâmetro do F5, por uma razão medida**
    /// (2026-08-22): a montagem enviesava o interior dos patches porque
    /// interpolava a fronteira, e a causa apurada foi que ela **nem sequer recebia
    /// o campo**. Um parâmetro novo pode ser esquecido em qualquer um dos dezoito
    /// sítios que chamam o F5; um campo do layout **não pode** — quem tem o layout
    /// tem, por construção, o campo que o gerou.
    ///
    /// ⚠️ **Vazio é uma resposta legítima**: [`decompose`] é chamada directamente
    /// por gates que não têm campo nenhum, e ali o F5 volta ao achatamento
    /// harmónico. *O que não pode existir é o caminho do produto sem ele*, e é o
    /// [`crate::trace_patches`] quem o preenche.
    pub face_dir: Vec<[f32; 3]>,
    /// Por patch, por lado, os arcos que o compõem: o id e **se o lado o
    /// percorre ao contrário** da ordem canónica do arco.
    ///
    /// ⚠️ **O sentido é obrigatório e não é decoração.** Os dois patches que
    /// partilham um arco percorrem-no em sentidos opostos; sem o registar, o F5
    /// amostraria os pontos de um deles ao contrário e os dois lados da divisa
    /// deixariam de coincidir — a malha sairia **rasgada** ao longo de cada
    /// fronteira de patch, sem nenhum erro a acusar.
    pub side_arcs: Vec<Vec<Vec<(u32, bool)>>>,
    /// Por arco, a cadeia ORDENADA de vértices de malha, na ordem canónica.
    pub arc_chain: Vec<Vec<u32>>,
    /// Por patch, os vértices-canto, na ordem da fronteira.
    pub corners: Vec<Vec<u32>>,
    /// Por arco, o comprimento geométrico.
    ///
    /// ⚠️ **Ele é a régua da PRÉ-CONDIÇÃO e não a da densidade** — o
    /// `ph2d_quadfill` mede-o na malha que recebe e compara com este número para
    /// provar que o layout é daquela malha. Quem decide **quantos quads** um arco
    /// leva é o [`Self::arc_tau`].
    pub arc_length: Vec<f32>,
    /// ⭐⭐ **O COMPRIMENTO EFECTIVO cumulativo ao longo de cada arco** — o `τ`, um
    /// valor por vértice da cadeia, começando em `0`.
    ///
    /// ⭐ **Ele é a fonte ÚNICA da densidade, e as duas fases a leem.** O F4 tira
    /// dele o alvo de quads do arco (`τ_fim / aresta_alvo`) e o F5 reamostra a
    /// cadeia por ele — então *é inexprimível* o alvo dizer uma densidade e a
    /// amostragem realizar outra. ⛔ Duas contas separadas foi o que destruiu o
    /// produto em 2026-08-21, noutra fase.
    ///
    /// ⚠️ **Sem graduação ele É o comprimento cumulativo** (`τ_fim == arc_length`),
    /// e o resultado é byte-idêntico ao de uma reamostragem por comprimento de
    /// arco. Com [`PatchLayout::grade`] cada troço passa a valer
    /// `|Δ| × alvo/tamanho_local`: onde o campo de tamanho pede quads menores, o
    /// troço "mede" mais e recebe mais segmentos. *A densidade vira uma integral,
    /// não um caso especial.*
    pub arc_tau: Vec<Vec<f32>>,
    /// Por arco, as arestas de malha que o compõem. ⚠️ É o que permite **desfazer**
    /// uma parede: sem isto, um patch degenerado só se pode contar, não curar.
    pub arc_edges: Vec<BTreeSet<(u32, u32)>>,
    /// ⭐ **Quantas fronteiras cada patch tem.** `1` é um disco; mais do que isso é
    /// um anel, e ele **não é um patch**.
    ///
    /// ⚠️ **A ausência deste número custou o produto uma segunda vez**
    /// (2026-08-21): os lados das DUAS fronteiras iam para a mesma lista, então o
    /// último lado de uma não encadeava no primeiro da outra — e a montagem
    /// recusava com `Broken`, três fases depois, nomeando a fase errada. O relatório
    /// já contava `non_disk`; o que faltava era **quem** para se poder curar.
    pub loops_per_patch: Vec<usize>,
    /// ⭐⭐⭐ **AS FRONTEIRAS de cada patch, em vértices** — e não só a contagem delas.
    ///
    /// ⛔⛔ **O layout calculava-as e deitava-as fora**, guardando apenas o `len()`. ⚠️ *A
    /// contagem diz que o patch é um anel; só os laços dizem POR ONDE o cortar* — e a
    /// reparação por corte precisa exactamente disto (2026-08-25).
    pub loops: Vec<Vec<Vec<u32>>>,
    /// ⭐⭐ **A CARACTERÍSTICA DE EULER da região de faces de cada patch** —
    /// `V − E + F` sobre as faces que ele contém. **Um disco dá `1`.**
    ///
    /// ⛔ **Ela existe porque o [`Self::loops_per_patch`] é CEGO AO GÉNERO, e isso
    /// custou o produto** (2026-08-22): num toro, um patch engolia a asa inteira e
    /// saía com **uma** fronteira — a única cerca que havia deixava-o passar — e a
    /// malha final vinha com `χ = 2` onde a topologia exige `0`. *Uma peça pode
    /// passar em toda asserção e ter deixado de ser um toro.*
    ///
    /// ⚠️ **A régua completa é `χ = 2 − 2g − b`.** Contar fronteiras dá o `b` e
    /// apanha o anel (`b = 2 ⇒ χ = 0`); o género só aparece quando se mede o `χ`
    /// também. Um disco é `b = 1` **e** `χ = 1`; a asa engolida é `b = 1` e
    /// `χ = −1`, que é `g = 1`.
    ///
    /// ⚠️ **Ela é da REGIÃO DE FACES, não do passeio da fronteira** — de propósito.
    /// Perguntar ao passeio se ele é um disco seria pedir-lhe que se auto-conferisse
    /// com a mesma informação que já o deixou passar.
    ///
    /// ⛔⛔ **E ela é DIAGNÓSTICO, nunca gatilho de [`Self::degenerate`]** — medido
    /// no mesmo dia. A cura dos degenerados é `dissolve`, que **apaga uma parede** e
    /// faz o patch CRESCER; para uma asa isso é a direcção errada, e o laço de
    /// limpeza comeu a decomposição inteira: **27 patches viraram 1**. Uma asa
    /// cura-se **acrescentando** um corte, e esse trabalho ainda não existe.
    ///
    /// ⚠️ **E `χ(região) == 1` nem sequer é a condição certa para o corte**: uma
    /// asa cortada por um laço não-separante continua a ser a mesma região de faces
    /// (o `flood` não duplica vértices), logo o `χ` dela **não muda**. A régua que
    /// de facto decide é a do COMPLEXO — `V − E + F` sobre cantos, arcos e patches
    /// — e é ela que o [`super::PatchLayout::to_layout`] usa como cerca. *A estrutura
    /// CW mínima de um toro é UM patch com uma aresta dupla, e ela é válida.*
    pub chi: Vec<i64>,
    /// ⭐⭐ **A CARACTERÍSTICA DE EULER da malha que entrou** — `2` numa esfera,
    /// `0` num toro.
    ///
    /// ⚠️ **Ela viaja no layout porque a CERCA precisa dela e não tem a malha.** O
    /// [`super::PatchLayout::to_layout`] compara o complexo de patches com este
    /// número; guardá-lo aqui é o que torna a comparação possível sem passar a
    /// malha por mais uma porta — e sem uma segunda conta que pudesse discordar.
    pub mesh_chi: i64,
    /// O que o traçado e a decomposição mediram.
    pub report: TraceReport,
}

/// **RECORTA a malha nas paredes** e devolve a decomposição.
#[must_use]
pub fn decompose(mesh: &Mesh, walls: &Walls, report: TraceReport) -> PatchLayout {
    decompose_with(mesh, walls, report, false)
}

/// ⭐⭐ **A MESMA COISA, com a PONTE do anel honrada ou não** — ver `cut_open`.
///
/// ⚠️ **Ponto de extensão append-only** (`CLAUDE.md` §0.2): com `cut_open = false`
/// ela **é** o [`decompose`], que a chama assim — a identidade é por construção, não
/// por asserção. O que **tem** gate é a decisão de quando a ligar
/// (`the_bridge_is_only_adopted_when_it_strictly_helps`).
#[must_use]
pub fn decompose_with(
    mesh: &Mesh,
    walls: &Walls,
    mut report: TraceReport,
    cut_open: bool,
) -> PatchLayout {
    let faces = mesh.faces();
    let pos = mesh.positions();
    // Aresta dirigida -> face que a contém na sua orientação. Numa malha fechada
    // e coerentemente orientada, cada uma pertence a exatamente uma face.
    let mut half: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (fi, f) in faces.iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            half.insert(
                (v[k], v[(k + 1) % v.len()]),
                u32::try_from(fi).unwrap_or(u32::MAX),
            );
        }
    }

    let face_patch = flood(faces, walls, &half);
    let n_patches = face_patch
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m as usize + 1);

    // ⭐ **O `χ` de cada patch vem ANTES das fronteiras**, e a ordem é obrigatória:
    // é ele que diz a quem a ponte se aplica. Ele sai só do `face_patch`, então não
    // depende de nada que as fronteiras produzam.
    let chi = crate::topology::patch_chi(faces, &face_patch, n_patches);

    // As fronteiras, patch a patch.
    let loops: Vec<Vec<Vec<u32>>> = (0..n_patches)
        .map(|p| {
            // ⭐⭐ **A PONTE só se honra num patch que NÃO É DISCO.** Medido em
            // 2026-08-22: nas três fixturas com parede interior, ela vive **sempre**
            // no patch anel (`χ = 0`, duas fronteiras) e é um **caminho entre as duas
            // fronteiras dele** — ou seja, *a ponte que abre o anel em disco já está
            // traçada*, e o passeio é que se recusava a vê-la.
            crate::boundary::boundary_loops(
                faces,
                &half,
                &face_patch,
                walls,
                u32::try_from(p).unwrap_or(0),
                cut_open && chi.get(p).copied().unwrap_or(1) != 1,
            )
        })
        .collect();

    // ⭐ **UM CANTO SÓ EXISTE ONDE A PAREDE SE RAMIFICA** — ver [`is_corner`].
    let branching = walls.branching();

    // ⚠️ **O canto é do par (patch, vértice)**, mas o CORTE em arcos é global: um
    // vértice que seja canto de qualquer patch parte a fronteira de todos os que
    // passam por ele. É isso que faz nascer a junção em T — o lado de um patch
    // com dois arcos, porque o vizinho tem um canto no meio dele.
    let mut any_corner: BTreeSet<u32> = BTreeSet::new();
    // ⚠️ **O canto PROMOVIDO é do patch que o promoveu, e só dele.** Ele parte a
    // fronteira de todos (é `any_corner`), mas só fecha um lado de quem precisou
    // dele — para o vizinho, a fronteira passa direito por ali. *É exactamente a
    // relação de uma junção em T, e por isso a lista é por patch.*
    let mut mine_corner: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); n_patches];
    let mut promoted = 0usize;
    for (pi, ls) in loops.iter().enumerate() {
        let p = u32::try_from(pi).unwrap_or(0);
        for lp in ls {
            let structural: Vec<u32> = lp
                .iter()
                .copied()
                .filter(|&v| crate::boundary::is_corner(mesh, faces, &face_patch, &branching, p, v))
                .collect();
            any_corner.extend(&structural);
            mine_corner[pi].extend(&structural);
            // ⭐ **A PROMOÇÃO — e ela é o degrau, não a regra.** Ver
            // [`crate::boundary::MIN_PATCH_CORNERS`].
            if structural.len() < crate::boundary::MIN_PATCH_CORNERS {
                let got = crate::boundary::promote(
                    mesh,
                    faces,
                    &face_patch,
                    p,
                    lp,
                    crate::boundary::MIN_PATCH_CORNERS - structural.len(),
                    &mut mine_corner[pi],
                );
                promoted += got;
            }
            any_corner.extend(mine_corner[pi].iter().copied());
        }
    }
    report.promoted = promoted;

    // Os arcos: a fronteira partida em TODO canto de QUALQUER patch.
    let mut arc_id: BTreeMap<BTreeSet<(u32, u32)>, u32> = BTreeMap::new();
    let mut arc_length: Vec<f32> = Vec::new();
    let mut arc_tau: Vec<Vec<f32>> = Vec::new();
    let mut arc_edges: Vec<BTreeSet<(u32, u32)>> = Vec::new();
    let mut arc_chain: Vec<Vec<u32>> = Vec::new();
    let mut side_arcs: Vec<Vec<Vec<(u32, bool)>>> = Vec::with_capacity(n_patches);
    let mut corners: Vec<Vec<u32>> = Vec::with_capacity(n_patches);
    let loops_per_patch: Vec<usize> = loops.iter().map(Vec::len).collect();
    // ⚠️ **Aqui, e não numa sonda:** é o único sítio onde as paredes e a
    // decomposição que elas produziram existem ao mesmo tempo. Ver
    // [`crate::TraceReport::interior_walls`].
    report.interior_walls = walls
        .edges
        .iter()
        .filter(|&&(a, b)| match (half.get(&(a, b)), half.get(&(b, a))) {
            (Some(&f), Some(&g)) => face_patch[f as usize] == face_patch[g as usize],
            _ => false,
        })
        .count();
    report.with_genus = chi.iter().filter(|c| **c != 1).count();
    for (p, ls) in loops.iter().enumerate() {
        let _ = p;
        if ls.len() != 1 {
            report.non_disk += 1;
        }
        let mut patch_sides: Vec<Vec<(u32, bool)>> = Vec::new();
        let mut patch_corners: Vec<u32> = Vec::new();
        let mut flat = 0usize;
        for lp in ls {
            let mine: Vec<bool> = lp.iter().map(|v| mine_corner[p].contains(v)).collect();
            let cuts: Vec<usize> = (0..lp.len())
                .filter(|&i| any_corner.contains(&lp[i]))
                .collect();
            if cuts.is_empty() {
                continue;
            }
            let mut open: Vec<(u32, bool)> = Vec::new();
            for k in 0..cuts.len() {
                let (i, j) = (cuts[k], cuts[(k + 1) % cuts.len()]);
                let chain = chain_between(lp, i, j);
                // ⚠️ **A chave é o CONJUNTO de arestas, não a sequência.** Os
                // dois patches que partilham um arco percorrem-no em sentidos
                // OPOSTOS (a fronteira de cada um roda no seu próprio sentido),
                // então a lista ordenada de um é a do outro ao contrário — e o
                // mesmo arco ganharia dois ids. O sintoma é `ArcUse { uses: 1 }`
                // no F4: um arco de bordo onde não há bordo nenhum.
                let key: BTreeSet<(u32, u32)> = chain
                    .windows(2)
                    .map(|w| (w[0].min(w[1]), w[0].max(w[1])))
                    .collect();
                let id = *arc_id.entry(key.clone()).or_insert_with(|| {
                    // ⭐ O `τ` nasce como o comprimento cumulativo — a graduação
                    // neutra. Ver [`PatchLayout::arc_tau`].
                    let mut tau = Vec::with_capacity(chain.len());
                    let mut run = 0.0f32;
                    tau.push(0.0);
                    for w in chain.windows(2) {
                        run += crate::boundary::dist(pos[w[0] as usize], pos[w[1] as usize]);
                        tau.push(run);
                    }
                    let len = run;
                    arc_tau.push(tau);
                    arc_length.push(len);
                    arc_edges.push(key);
                    arc_chain.push(chain.clone());
                    u32::try_from(arc_length.len() - 1).unwrap_or(u32::MAX)
                });
                // ⚠️ O sentido: este lado percorre o arco na ordem canónica ou ao
                // contrário? A resposta é a ponta por onde ele entra.
                let reversed = arc_chain[id as usize].first() != chain.first();
                open.push((id, reversed));
                // O lado FECHA num canto DESTE patch, não num canto qualquer.
                if mine[j] {
                    patch_sides.push(std::mem::take(&mut open));
                    patch_corners.push(lp[j]);
                }
            }
            if !open.is_empty() {
                // O resto emenda no primeiro lado do mesmo laço.
                if let Some(first) = patch_sides.first_mut() {
                    let mut merged = std::mem::take(&mut open);
                    merged.append(first);
                    *first = merged;
                } else {
                    patch_sides.push(std::mem::take(&mut open));
                }
            }
            flat += lp.len();
        }
        let _ = flat;
        *report.valence.entry(patch_sides.len()).or_default() += 1;
        side_arcs.push(patch_sides);
        corners.push(patch_corners);
    }
    report.patches = n_patches;
    report.arcs = arc_length.len();

    PatchLayout {
        face_patch,
        // ⚠️ **VAZIO aqui de propósito** — esta função não recebe o campo. Quem o
        // preenche é o [`crate::trace_patches`], que o tem. Um `decompose` chamado
        // de um gate sem campo continua a devolver um layout válido, e o F5 volta
        // ao achatamento harmónico.
        face_dir: Vec::new(),
        side_arcs,
        arc_chain,
        corners,
        arc_length,
        arc_tau,
        arc_edges,
        loops_per_patch,
        loops,
        chi,
        mesh_chi: crate::topology::mesh_euler(faces),
        report,
    }
}

impl PatchLayout {
    /// Por patch, por lado, só os **ids** de arco — o que o F4 consome.
    #[must_use]
    pub fn sides(&self) -> Vec<Vec<Vec<u32>>> {
        self.side_arcs
            .iter()
            .map(|sides| {
                sides
                    .iter()
                    .map(|side| side.iter().map(|&(a, _)| a).collect())
                    .collect()
            })
            .collect()
    }

    /// **OS PATCHES DEGENERADOS** — menos de três lados.
    ///
    /// ⚠️ **Um patch de dois lados não é um erro de contagem, é uma lasca**: duas
    /// separatrizes que correram quase juntas e prenderam uma tira entre elas. Ele
    /// não tem cantos que cheguem para a lei do F4 (`L_i = e_{i-1} + e_{i+1}` pede
    /// pelo menos três), e é por isso que o layout inteiro é recusado por causa de
    /// **um** deles.
    #[must_use]
    pub fn degenerate(&self) -> Vec<usize> {
        (0..self.side_arcs.len())
            .filter(|&p| {
                // ⭐ **Duas razões, e a segunda foi acrescentada em 2026-08-21.**
                // Menos de três lados não satisfaz a lei do F4; e **mais de uma
                // fronteira não é um disco** — os lados das duas iam para a mesma
                // lista, e o último de uma não encadeava no primeiro da outra.
                self.side_arcs[p].len() < 3
                    || self.loops_per_patch.get(p).copied().unwrap_or(1) != 1
            })
            .collect()
    }
}

/// **DISSOLVE** a parede que separa cada patch degenerado do vizinho mais barato.
///
/// ⚠️ **Apagar a parede é a cura certa, e apagar o patch não seria.** A lasca não
/// é uma região a mais na superfície: ela é uma parede a mais. Removida a parede,
/// as faces dela juntam-se ao vizinho na inundação seguinte e a contagem de
/// patches cai sozinha — nenhuma face muda de sítio, nenhuma geometria se toca.
///
/// Devolve `false` quando não havia nada para dissolver, que é o sinal de paragem.
#[must_use]
pub fn dissolve(walls: &mut Walls, layout: &PatchLayout, victims: &[usize]) -> bool {
    let mut removed = false;
    for &p in victims {
        // O lado mais CURTO: dissolver o maior mudaria a forma do vizinho muito
        // mais do que o necessário para a lasca desaparecer.
        let target = layout.side_arcs[p]
            .iter()
            .min_by(|a, b| {
                let la: f32 = a.iter().map(|&(i, _)| layout.arc_length[i as usize]).sum();
                let lb: f32 = b.iter().map(|&(i, _)| layout.arc_length[i as usize]).sum();
                la.total_cmp(&lb)
            })
            .cloned();
        let arcs: Vec<u32> = match target {
            Some(side) => side.into_iter().map(|(a, _)| a).collect(),
            // ⛔⛔ **ESTE RAMO É MORTO, e o comentário dizia o contrário.** Ele só é
            // alcançado quando `side_arcs[p]` está **vazio** (o `min_by` de um iterador
            // vazio é `None`) — e o filtro logo abaixo pergunta a esse **mesmo** vector
            // vazio quais arcos ele contém, logo o `any` é sempre `false` e a lista sai
            // vazia. ⚠️ *A intenção escrita era «a fronteira inteira sai»; o que o código
            // faz é não tirar nada.*
            //
            // ⚠️ **Medido 2026-08-25: não é ele que trava a limpeza.** Na peça do artista
            // existe exactamente um patch de `0` lados, e mesmo assim a `dissolve` devolve
            // `true` (os outros quatro degenerados removem parede) — quem trava é a guarda
            // de topologia, uma linha acima do laço que chama isto
            // (`TraceReport::cleanup_stop == 2`). ⛔ Curar este ramo **não** cura a peça, e
            // é por isso que ele fica aqui NOMEADO em vez de remendado a caminho de outra
            // coisa. *Um defeito verdadeiro que não é o defeito em mãos merece um nome, não
            // um desvio.*
            None => (0..layout.arc_edges.len())
                .map(|i| u32::try_from(i).unwrap_or(0))
                .filter(|&i| layout.side_arcs[p].iter().flatten().any(|&(j, _)| j == i))
                .collect(),
        };
        for a in arcs {
            for e in &layout.arc_edges[a as usize] {
                removed |= walls.edges.remove(e);
            }
        }
    }
    removed
}

/// ⭐⭐⭐ **ABRE OS ANÉIS — a reparação por CORTE, que é a metade que faltava.**
///
/// Um patch com **duas ou mais fronteiras** é um anel: ele não é um disco, e a fase seguinte
/// não o consegue parametrizar. ⭐ A cura publicada é **cortar** entre duas fronteiras dele,
/// não fundir com o vizinho.
///
/// # ⛔⛔ Por que a [`dissolve`] não serve, e está medido
///
/// A `dissolve` cura a **lasca** — um patch de poucos lados, que de facto é *uma parede a
/// MAIS*. ⚠️ **Um anel é o contrário: uma parede a MENOS**, e fundi-lo com o vizinho só o
/// torna maior e mais complexo. Medido em 2026-08-25 na peça do artista (`chain_info` com
/// `PH2D_CLEANUP_FORCE=1`): forçar a fusão por cima da guarda leva as transições inexactas
/// de `8` a `4` **e** o bordo de `8` a `10`, com o enviesamento de `7,3°` a `7,6°`. *A cura
/// certa para o defeito errado paga em geometria o que ganha em topologia.*
///
/// ⚠️ **E não há ponte traçada para honrar:** o modo `decompose_with(cut_open)` devolve
/// `(1, 5)` contra os `(1, 5)` do normal nesta peça — *o caminho tem de ser CONSTRUÍDO*.
///
/// Devolve `false` quando não havia anel nenhum ou nenhum caminho — o sinal de paragem.
#[must_use]
pub fn open_rings(mesh: &ph2d_mesh::Mesh, walls: &mut Walls, layout: &PatchLayout) -> bool {
    let mut cut = false;
    for (p, loops) in layout.loops.iter().enumerate() {
        if loops.len() < 2 {
            continue;
        }
        let faces: Vec<u32> = layout
            .face_patch
            .iter()
            .enumerate()
            .filter(|(_, q)| **q as usize == p)
            .filter_map(|(f, _)| u32::try_from(f).ok())
            .collect();
        let from: BTreeSet<u32> = loops[0].iter().copied().collect();
        let to: BTreeSet<u32> = loops[1].iter().copied().collect();
        // ⚠️ **O caminho anda pelas faces DESTE patch e mais nenhuma** — um corte que
        // saísse dele partiria um vizinho que ninguém pediu para partir.
        let Some(chain) = path_inside(mesh, &faces, &from, &to) else {
            continue;
        };
        // ⛔⛔ **O VÃO É A PORTA.** Um caminho de uma aresta diz que as duas fronteiras se
        // TOCAM — o patch está estrangulado, e cortar ali acrescenta um toco em vez de
        // abrir um anel (medido: `(1,5)` ⇒ `(2,6)`). ⭐ Só um vão a sério é um anel.
        if chain.len() - 1 < MIN_RING_GAP {
            continue;
        }
        for w in chain.windows(2) {
            let (a, b) = (w[0].min(w[1]), w[0].max(w[1]));
            if walls.edges.insert((a, b)) {
                *walls.degree.entry(a).or_default() += 1;
                *walls.degree.entry(b).or_default() += 1;
                cut = true;
            }
        }
    }
    cut
}

/// ⭐⭐⭐ **O VÃO MÍNIMO para um patch multi-fronteira ser tratado como ANEL.**
///
/// ⛔ Abaixo dele o patch está **estrangulado** e o corte é a cura errada — ver a tabela em
/// [`crate::OPEN_RINGS`]. O número sai da [`ring_gaps`], e ⚠️ **`0` reabre a experiência
/// que foi rejeitada**.
pub const MIN_RING_GAP: usize = 2;

/// ⭐⭐⭐ **A RÉGUA QUE FALTAVA: a que DISTÂNCIA as duas fronteiras de um patch passam uma
/// da outra**, em arestas.
///
/// ⛔⛔ **A contagem de fronteiras junta duas avarias com curas opostas** (medido
/// 2026-08-25): um **anel gordo** tem um buraco no meio e cura-se **cortando**; um patch
/// **ESTRANGULADO** tem as duas fronteiras a um triângulo uma da outra, e cortar ali só
/// acrescenta um toco. *«Duas fronteiras» chama-se anel na topologia, e o nome trouxe
/// consigo a cura errada.*
///
/// Devolve, por patch multi-fronteira: `(patch, lados, fronteiras, VÃO em arestas, faces)`.
/// ⚠️ O vão `1` é o estrangulamento; é ele que a `open_rings` **não** deve tocar.
#[must_use]
pub fn ring_gaps(
    mesh: &ph2d_mesh::Mesh,
    layout: &PatchLayout,
) -> Vec<(usize, usize, usize, usize, usize)> {
    let mut out = Vec::new();
    for (p, loops) in layout.loops.iter().enumerate() {
        if loops.len() < 2 {
            continue;
        }
        let faces: Vec<u32> = layout
            .face_patch
            .iter()
            .enumerate()
            .filter(|(_, q)| **q as usize == p)
            .filter_map(|(f, _)| u32::try_from(f).ok())
            .collect();
        let from: BTreeSet<u32> = loops[0].iter().copied().collect();
        let to: BTreeSet<u32> = loops[1].iter().copied().collect();
        // ⚠️ **O vão é o número de ARESTAS do caminho**, que é `len - 1`: uma cadeia de
        // dois vértices é uma aresta, e é o estrangulamento.
        let gap = path_inside(mesh, &faces, &from, &to).map_or(0, |c| c.len() - 1);
        out.push((p, layout.side_arcs[p].len(), loops.len(), gap, faces.len()));
    }
    out
}

/// O caminho mais curto (em nº de arestas) de `from` a `to`, andando **só** por arestas das
/// faces dadas.
///
/// ⚠️ **Devolve a cadeia inteira, pontas incluídas** — é ela que vira parede.
fn path_inside(
    mesh: &ph2d_mesh::Mesh,
    faces: &[u32],
    from: &BTreeSet<u32>,
    to: &BTreeSet<u32>,
) -> Option<Vec<u32>> {
    let mut adj: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for &f in faces {
        let v = mesh.faces()[f as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        }
    }
    let mut prev: BTreeMap<u32, u32> = BTreeMap::new();
    let mut seen: BTreeSet<u32> = from.iter().copied().collect();
    let mut queue: VecDeque<u32> = from.iter().copied().collect();
    let mut hit = None;
    while let Some(x) = queue.pop_front() {
        if to.contains(&x) && !from.contains(&x) {
            hit = Some(x);
            break;
        }
        for &y in adj.get(&x).into_iter().flatten() {
            if seen.insert(y) {
                prev.insert(y, x);
                queue.push_back(y);
            }
        }
    }
    let mut cur = hit?;
    let mut chain = vec![cur];
    while let Some(&q) = prev.get(&cur) {
        chain.push(q);
        cur = q;
    }
    chain.reverse();
    (chain.len() >= 2).then_some(chain)
}

/// **INUNDAÇÃO** — faces vizinhas por uma aresta que não é parede são o mesmo
/// patch.
fn flood(faces: &[ph2d_mesh::Face], walls: &Walls, half: &BTreeMap<(u32, u32), u32>) -> Vec<u32> {
    let mut patch = vec![u32::MAX; faces.len()];
    let mut next = 0u32;
    let mut queue: VecDeque<u32> = VecDeque::new();
    for start in 0..faces.len() {
        if patch[start] != u32::MAX {
            continue;
        }
        patch[start] = next;
        queue.push_back(u32::try_from(start).unwrap_or(0));
        while let Some(f) = queue.pop_front() {
            let v = faces[f as usize].verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                if walls.blocks(a, b) {
                    continue;
                }
                // A face do outro lado é a que contém a aresta ao contrário.
                let Some(&g) = half.get(&(b, a)) else {
                    continue;
                };
                if patch[g as usize] == u32::MAX {
                    patch[g as usize] = next;
                    queue.push_back(g);
                }
            }
        }
        next += 1;
    }
    patch
}

/// A cadeia de vértices do laço, de `i` até `j`, dando a volta se preciso.
fn chain_between(lp: &[u32], i: usize, j: usize) -> Vec<u32> {
    let n = lp.len();
    let mut out = Vec::new();
    let mut t = i;
    loop {
        out.push(lp[t]);
        if t == j && out.len() > 1 {
            break;
        }
        t = (t + 1) % n;
        if out.len() > n {
            break;
        }
    }
    out
}
