//! ⭐⭐⭐ **DESFAZER AS GRAVATAS** — o quad que se auto-cruza, endireitado no sítio.
//!
//! # ⛔⛔⛔ Por que ele existe: UMA gravata deitou fora a melhor candidata que esta peça já teve
//!
//! Medido em 2026-09-03 na escultura do dono (`_base_sculpt`, `Detail 1`, com a calota da fase
//! zero — [`ph2d_remesh_iso::Cap`]), com as três primeiras chaves do selector impressas pela
//! primeira vez:
//!
//! | candidata | furos | ilhas | **gravatas** | pontas amputadas | grade no bico |
//! |---|---|---|---|---|---|
//! | a que o produto escolheu | `0` | `1` | **`0`** | ⛔ **`3` de `5`** | ⛔ `2,81` |
//! | ⭐ a que ele deitou fora | `0` | `1` | **`1`** | ⭐ **`0` de `5`** | ⭐ `0,81` |
//!
//! A chave das gravatas é a **3.ª** e a da amputação a **4.ª**: uma face dobrada ganha a três
//! pontas cortadas. ⚠️ **E a gravata nem estava na ponta** — a `5,7` células do bico mais
//! próximo, um quad dobrado solto no flanco.
//!
//! ⛔ **A saída NÃO é reordenar as chaves** — a ordem foi medida em 30/08 sobre um report do
//! dono (*«destruiu completamente a malha»*, `125` gravatas), e o doc do
//! `sculpt3d_retopo_extract` já escreve a lei: *«a saída não é reordenar o critério, é produzir
//! a candidata que tem as duas coisas»*. É isso que este módulo faz.
//!
//! ⚠️ **Onde não há gravata, ele é o mapa identidade AO BIT** — e há gate. *Uma cura que toca
//! na malha que já estava boa é uma regressão à espera de um smoke.*

use ph2d_mesh::Mesh;

/// Quantas rondas de relaxação local uma gravata tem para se desfazer.
///
/// ⚠️ **Poucas, e de propósito:** isto move `4` vértices por face acusada, com a cerca de
/// viagem por cima — quem precisa de mais do que isto não tem uma face dobrada, tem uma região
/// mal traçada, e essa é a chave da frente do selector, não esta.
const MAX_ROUNDS: usize = 16;

/// ⭐⭐⭐ **A CERCA DESTA REPARAÇÃO — e ela NÃO é a do acabamento**, de propósito.
///
/// ⚠️ **As duas respondem a perguntas diferentes:** a [`crate::EXTRACT_TRAVEL_RESCUE`] (meia
/// aresta) limita um **deslize global** da grade sobre o relevo; esta limita uma **reparação
/// local** de uma face partida. ⛔ E o número não é escolhido: *um quad só se auto-cruza quando
/// um vértice passa PARA LÁ do vizinho*, logo a viagem de volta é da ordem de **uma** aresta —
/// com meia, a cura é impossível por construção. `2` é isso com folga; acima disso não é uma
/// dobra local, é uma região mal traçada, e essa é a chave da frente do selector.
pub const UNTANGLE_TRAVEL: f32 = 2.0;

/// Meio passo — o mesmo amortecimento do alisamento da casa ([`crate::finish`]): um passo
/// inteiro sobre uma umbrella não é contractivo.
const LAMBDA: f32 = 0.5;

/// ⭐⭐⭐ **Quantas dobras juntas fazem uma FENDA** — abaixo disto é vinco da escultura.
///
/// ⚠️ Calibrado no lado **aprovado**: a retopologia que o dono aceitou tem `3` dobras com maior
/// grupo **`1`**; a que ele fotografou tem `5` num grupo **só**. ⛔ Reparar as isoladas apagaria
/// forma que o artista esculpiu.
const GRUPO_MINIMO: usize = 2;

/// ⭐⭐⭐ **Endireita as faces do AVESSO — gravatas e DOBRAS — pousando na `surface`**, e devolve
/// quantas desapareceram.
///
/// `travel` é a cerca de viagem em unidades da aresta mediana; `≤ 0` ou não-finito = sem cerca.
///
/// # ⭐⭐⭐ As DUAS famílias, e por que são a mesma coisa (2026-09-03, 2.º report do dono)
///
/// Uma **gravata** é uma face que se cruza a si própria; uma **dobra** é uma face cuja normal
/// aponta contra a média das vizinhas ([`crate::folded_by_neighbours`]). As duas são *«a face
/// está do avesso em relação à vizinhança»*, e as duas se lêem na foto como uma **fenda escura**.
///
/// ⛔⛔ **E a fenda que ele fotografou eram CINCO dobras no mesmo ponto** — com a malha
/// topologicamente perfeita (`χ = 2`, zero bordo, zero não-manifold, zero gravatas). *Nenhuma
/// régua desta cadeia olhava para dobras*, apesar de as duas leis viverem nesta crate desde
/// sempre e serem consumidas pelo motor legado e por sondas.
///
/// # ⚠️ SÓ os GRUPOS, e a calibração é o lado APROVADO
///
/// Medido com [`crate::quality::folded_faces_by_neighbours`]:
///
/// | malha | dobras | **maior grupo** |
/// |---|---|---|
/// | a retopologia que o dono **APROVOU** (QRemeshify) | `3` | **`1`** — isoladas |
/// | ⛔ a nossa, na foto dele | `5` | **`5`** — todas no mesmo sítio |
///
/// ⇒ **uma dobra ISOLADA fica** (é um vinco real da escultura, e alisá-la seria apagar forma);
/// **um grupo de `≥ 2` é reparado**. *Uma cura calibrada só no nosso lado apagaria a feição de
/// quem está certo.*
///
/// ⚠️ **A aceitação tem DUAS metades**, e cada uma responde a uma pergunta:
///
/// - **desceram?** — o censo é global, logo trocar um defeito por outro noutro sítio não passa;
/// - **a forma não piorou?** — a mesma lei de [`crate::acceptable`] que a passagem de
///   acabamento já usa. *Duas leis de aceitação seriam duas respostas à mesma pergunta.*
///
/// ⛔ Se qualquer das duas falhar, a malha volta **exactamente** ao que era.
#[must_use]
pub fn untangle_bowties(mesh: &mut Mesh, surface: &Mesh, travel: f32) -> usize {
    let grupos = grupos_acusados(mesh);
    if grupos.is_empty() {
        return 0;
    }
    let unit = median_edge(mesh);
    let max_travel = if travel.is_finite() && travel > 0.0 && unit > 0.0 {
        unit * travel
    } else {
        f32::INFINITY
    };
    let seed = crate::finish::bbox_seed(surface);
    let log = std::env::var("PH2D_UNTANGLE_LOG").is_ok();
    let mut curadas = 0usize;
    // ⭐⭐⭐ **UM GRUPO DE CADA VEZ, e a razão é MEDIDA (2026-09-03).**
    //
    // ⛔⛔ A 1.ª versão relaxava TODOS os vértices acusados de uma vez e repunha tudo se o censo
    // não descesse. Quando o report do dono trouxe a 2.ª família (as dobras), isso **acoplou os
    // dois reparos**: o grupo de dobras não cedia, a reposição apagava também a gravata que já
    // tinha sido curada, e a candidata boa voltou a perder — a saída regrediu de `0/5` para
    // `3/5` pontas amputadas. *Um reparo que não cede não pode levar consigo o que cedeu.*
    for grupo in grupos {
        let origin: Vec<[f32; 3]> = mesh.positions().to_vec();
        let forma_antes = crate::quad_shape(mesh);
        let antes = defect_count(mesh);
        let moveis = vertices_de(mesh, &grupo);
        if moveis.is_empty() {
            continue;
        }
        let antes_local = defeitos_no_grupo(mesh, &grupo);
        let mut melhor = antes_local;
        let mut best_pos: Option<Vec<[f32; 3]>> = None;
        for _ in 0..MAX_ROUNDS {
            relax_once(mesh, surface, &moveis, &origin, max_travel, seed);
            let agora = defeitos_no_grupo(mesh, &grupo);
            let total = defect_count(mesh);
            if log {
                let s = crate::quad_shape(mesh);
                eprintln!(
                    "[untangle] grupo de {} face(s): {antes_local} -> {agora} (total {antes} -> {total}) | >60 {} -> {}",
                    grupo.len(),
                    forma_antes.skew_over_60,
                    s.skew_over_60,
                );
            }
            // ⭐⭐⭐ **JULGA-SE O GRUPO, e a guarda é o TOTAL.**
            //
            // ⛔⛔ **Medido 2026-09-03, e é a lei desta porta:** julgar cada grupo pelo censo
            // GLOBAL faz um reparo esperar pelo outro — a gravata desta peça sai e uma dobra
            // vizinha entra no mesmo passo, logo o total não desce e o reparo é recusado, e a
            // candidata boa volta a perder na 3.ª chave do selector. ⚠️ *A relaxação local
            // TROCA a espécie do defeito tanto quanto o apaga* — e é por isso que a guarda
            // certa é «o total não SOBE», e não «o total desce».
            if agora < melhor && total <= antes && aceitavel(&crate::quad_shape(mesh), &forma_antes)
            {
                melhor = agora;
                best_pos = Some(mesh.positions().to_vec());
            }
            if agora == 0 {
                break;
            }
        }
        match best_pos {
            Some(p) => {
                mesh.positions_mut().copy_from_slice(&p);
                mesh.rebuild();
                curadas += antes_local - melhor;
            }
            None => {
                // ⛔ **Repor é incondicional, e SÓ deste grupo** — as rondas que não foram
                // aceites já mexeram na malha, e deixá-las entregaria um alisamento que ninguém
                // pediu.
                mesh.positions_mut().copy_from_slice(&origin);
                mesh.rebuild();
            }
        }
    }
    curadas
}

/// ⭐ **AS FACES ACUSADAS, agrupadas** — cada gravata é um grupo de uma; cada mancha de dobras é
/// um grupo. É a unidade de reparo, e é por isso que uma que não cede não leva as outras.
fn grupos_acusados(mesh: &Mesh) -> Vec<Vec<u32>> {
    let (_, per_face) = crate::local_shape(mesh);
    let mut saida: Vec<Vec<u32>> = Vec::new();
    for (i, d) in per_face.iter().enumerate() {
        if d.kind == crate::QuadKind::Bowtie {
            saida.push(vec![u32::try_from(i).unwrap_or(0)]);
        }
    }
    for grupo in grupos_dobrados(mesh) {
        if grupo.len() >= GRUPO_MINIMO {
            saida.push(grupo);
        }
    }
    saida
}

/// Os vértices de um grupo de faces — e **só** eles.
fn vertices_de(mesh: &Mesh, grupo: &[u32]) -> Vec<u32> {
    let mut v: Vec<u32> = Vec::new();
    for &i in grupo {
        if let Some(f) = mesh.faces().get(i as usize) {
            v.extend_from_slice(f.verts());
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

/// ⭐⭐⭐ **A ACEITAÇÃO DESTE REPARO — e ela NÃO é a do acabamento**, por medição.
///
/// ⛔⛔ **A [`crate::finish_extract::acceptable`] guarda CINCO colunas** (faces péssimas, e as
/// medianas e caudas do enviesamento e do aspecto), e é a lei certa para uma passagem que
/// **desliza a grade inteira**. Aqui ela recusava o reparo:
///
/// | a ronda que desfaz a dobra | faces péssimas | cauda do enviesamento | cauda do aspecto |
/// |---|---|---|---|
/// | antes | `4` | `85,0°` | `4,16` |
/// | depois | ⭐ **`2`** | ⭐ **`68,8°`** | ⛔ `4,38` |
///
/// *Quatro colunas melhoram, a quinta sobe `5 %`, e o preço da recusa é deixar lá uma face do
/// AVESSO* — que é exactamente o que o artista fotografa. ⇒ o que se guarda aqui é a coluna
/// **dura** (quantas faces são péssimas), e não as caudas: uma reparação que mexe em meia dúzia
/// de vértices não pode piorar a peça, e num mesh pequeno as caudas medem o próprio reparo.
///
/// ⚠️ **O censo dos defeitos continua GLOBAL** ([`defect_count`]), logo trocar uma dobra por
/// outra noutro sítio não passa — é essa a metade que impede o reparo de mentir.
fn aceitavel(s: &crate::QuadShape, base: &crate::QuadShape) -> bool {
    s.skew_over_60 <= base.skew_over_60
}

/// ⭐⭐⭐ **AS FACES DO AVESSO** — gravatas (todas) e dobras **em grupo de `≥ 2`**.
///
/// ⚠️ O filtro do grupo é a calibração do lado aprovado — ver [`untangle_bowties`].
fn acusadas(mesh: &Mesh) -> Vec<u32> {
    let (_, per_face) = crate::local_shape(mesh);
    let mut fora: Vec<u32> = Vec::new();
    for (i, d) in per_face.iter().enumerate() {
        if d.kind == crate::QuadKind::Bowtie {
            fora.push(u32::try_from(i).unwrap_or(0));
        }
    }
    for grupo in grupos_dobrados(mesh) {
        if grupo.len() >= GRUPO_MINIMO {
            fora.extend_from_slice(&grupo);
        }
    }
    fora.sort_unstable();
    fora.dedup();
    fora
}

/// Quantas faces DESTE grupo continuam do avesso — gravata ou dobra, sem olhar ao tamanho do
/// grupo (aqui o grupo já foi escolhido).
fn defeitos_no_grupo(mesh: &Mesh, grupo: &[u32]) -> usize {
    let (_, per_face) = crate::local_shape(mesh);
    let dobradas: std::collections::BTreeSet<u32> =
        crate::quality::folded_faces_by_neighbours(mesh)
            .into_iter()
            .collect();
    grupo
        .iter()
        .filter(|&&i| {
            dobradas.contains(&i)
                || per_face
                    .get(i as usize)
                    .is_some_and(|d| d.kind == crate::QuadKind::Bowtie)
        })
        .count()
}

fn defect_count(mesh: &Mesh) -> usize {
    acusadas(mesh).len()
}

/// ⭐⭐⭐ **Quantas faces vivem em GRUPOS de dobras** — a metade da família do avesso que o censo
/// de gravatas não vê, para quem decide entre candidatas.
///
/// ⚠️ A dobra **isolada** não conta: ela é um vinco real da escultura (calibrado no lado que o
/// dono aprovou) — ver [`untangle_bowties`].
#[must_use]
pub fn folded_group_faces(mesh: &Mesh) -> usize {
    grupos_dobrados(mesh)
        .into_iter()
        .filter(|g| g.len() >= GRUPO_MINIMO)
        .map(|g| g.len())
        .sum()
}

/// As faces dobradas, agrupadas por adjacência — *o que se vê é o grupo, não a contagem*.
fn grupos_dobrados(mesh: &Mesh) -> Vec<Vec<u32>> {
    let dobradas = crate::quality::folded_faces_by_neighbours(mesh);
    if dobradas.is_empty() {
        return Vec::new();
    }
    let alvo: std::collections::BTreeSet<u32> = dobradas.iter().copied().collect();
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for &i in &alvo {
        let v = mesh.faces()[i as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut viz: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for quem in por_aresta.values() {
        for &i in quem {
            for &j in quem {
                if i != j {
                    viz.entry(i).or_default().push(j);
                }
            }
        }
    }
    let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut saida: Vec<Vec<u32>> = Vec::new();
    for &s in &alvo {
        if vistos.contains(&s) {
            continue;
        }
        let mut grupo = Vec::new();
        let mut pilha = vec![s];
        vistos.insert(s);
        while let Some(u) = pilha.pop() {
            grupo.push(u);
            for &w in viz.get(&u).map(Vec::as_slice).unwrap_or(&[]) {
                if vistos.insert(w) {
                    pilha.push(w);
                }
            }
        }
        saida.push(grupo);
    }
    saida
}

fn median_edge(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            e.push(dist(a, b));
        }
    }
    if e.is_empty() {
        return 0.0;
    }
    e.sort_by(f32::total_cmp);
    e[e.len() / 2]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

/// Um passo de Laplaciano **tangencial** nos vértices móveis, com reprojeção e cerca.
///
/// ⚠️ **Tangencial pela mesma razão do alisamento da casa:** a componente normal encolhe a peça
/// e a reprojeção a seguir esconde o encolhimento sem o desfazer.
fn relax_once(
    mesh: &mut Mesh,
    surface: &Mesh,
    moveis: &[u32],
    origin: &[[f32; 3]],
    max_travel: f32,
    seed: f32,
) {
    let neighbours: Vec<Vec<u32>> = {
        let adj = mesh.adjacency();
        moveis
            .iter()
            .map(|v| adj.vert_verts.neighbours(*v as usize).to_vec())
            .collect()
    };
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let mut novos: Vec<(u32, [f32; 3])> = Vec::with_capacity(moveis.len());
    {
        let pos = mesh.positions();
        for (i, &v) in moveis.iter().enumerate() {
            let ns = &neighbours[i];
            if ns.len() < 3 {
                continue;
            }
            let p = pos[v as usize];
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let q = pos[w as usize];
                for k in 0..3 {
                    sum[k] += q[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / ns.len() as f32;
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v as usize];
            let along = d[0].mul_add(nv[0], d[1].mul_add(nv[1], d[2] * nv[2]));
            let q = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
            let q = ph2d_remesh_iso::project_onto(surface, q, seed);
            // ⛔ **Fora da cerca, o vértice fica onde está** — e não «encostado à cerca»: um
            // ponto truncado sai da superfície, e a reprojeção seguinte mediria outra coisa.
            if dist(q, origin[v as usize]) <= max_travel {
                novos.push((v, q));
            }
        }
    }
    {
        let pos = mesh.positions_mut();
        for (v, q) in novos {
            pos[v as usize] = q;
        }
    }
    mesh.rebuild();
}

#[cfg(test)]
#[path = "untangle_tests.rs"]
mod tests;

/// ⭐⭐⭐ **APAGA AS ABAS e fecha o buraco** — a cura TOPOLÓGICA do que a relaxação não pode curar.
///
/// # ⛔⛔⛔ O que é uma aba, medido na peça do dono (2026-09-03, 3.º report)
///
/// Uma **língua** de faces dobrada para trás sobre si mesma. Retrato do caso dele: `5` faces,
/// `12` vértices, `1,93 h²` de área **ao todo** (um quad normal é `1`), com ângulos de `174°` a
/// `179°` às vizinhas — *a superfície volta atrás*. Três das cinco faces são migalhas
/// (`0,005`–`0,044 h²`).
///
/// ⛔ **Ela nasce na EXTRACÇÃO** (medido com `PH2D_EXTRACT_FINISH=0`: a malha crua já a traz) e a
/// causa é o **mapa dobrar** ali — `134` triângulos dobrados no domínio da peça dele, `4,5 %`.
/// ⛔⛔ E a cura de fundo **já foi medida e recusada**: o solver injectivo do `ph2d-gridmap`
/// zera as dobras do mapa contínuo e foi ele que produziu o *«destruiu completamente a malha»* de
/// 30/08 (ver `injective_solve::enabled`).
///
/// # A operação, e as três condições que a recusam
///
/// Apagar o grupo deixa um buraco cujo bordo é **um** laço; ele é fechado por um **leque** de
/// `L/2` quads à volta de um vértice novo (a lei clássica do polígono de lados pares).
/// ⛔ Recusa-se — e a malha fica **exactamente** como estava — quando:
///
/// 1. o bordo do grupo **não é um laço só** (a aba não é um disco);
/// 2. o laço tem um número **ímpar** de lados (não há leque de quads);
/// 3. o resultado **não melhora**: as faces do avesso têm de descer, sem furos novos, sem ilhas
///    novas e sem mais faces péssimas.
#[must_use]
pub fn remove_flaps(mesh: &mut Mesh, surface: &Mesh) -> usize {
    let mut curadas = 0usize;
    // ⚠️ **Uma aba de cada vez, e recomeçando o censo**: apagar uma muda os índices de face.
    for _ in 0..MAX_FLAPS {
        let Some(grupo) = grupos_dobrados(mesh)
            .into_iter()
            .filter(|g| g.len() >= GRUPO_MINIMO)
            .max_by_key(Vec::len)
        else {
            break;
        };
        // ⭐⭐⭐ **O DISCO É MAIOR QUE A ABA, e a razão é medida:** os vértices emaranhados ficam
        // na BORDA do grupo, logo apagar só as faces dele deixa o buraco com o mesmo contorno
        // torcido e o remendo volta a dobrar (medido: `avesso 2 -> 2`). Crescer um anel põe-nos
        // no INTERIOR do disco, e eles desaparecem com ele.
        let disco = grow_one_ring(mesh, &grupo);
        if !remove_one_flap(mesh, surface, &disco) {
            break;
        }
        curadas += 1;
    }
    curadas
}

/// Quantas abas se tentam apagar numa passagem — ver [`remove_flaps`].
///
/// ⚠️ Poucas de propósito: uma malha com dezenas de abas não tem um defeito local, tem um mapa
/// partido, e essa é a chave da frente do selector, não esta.
const MAX_FLAPS: usize = 8;

/// As faces do grupo **mais** todas as que partilham um vértice com ele — ver o uso.
fn grow_one_ring(mesh: &Mesh, grupo: &[u32]) -> Vec<u32> {
    let vs: std::collections::BTreeSet<u32> = grupo
        .iter()
        .filter_map(|&i| mesh.faces().get(i as usize))
        .flat_map(|f| f.verts().iter().copied())
        .collect();
    let mut fora: Vec<u32> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(_, f)| f.verts().iter().any(|v| vs.contains(v)))
        .map(|(i, _)| u32::try_from(i).unwrap_or(0))
        .collect();
    fora.sort_unstable();
    fora.dedup();
    fora
}

fn remove_one_flap(mesh: &mut Mesh, surface: &Mesh, grupo: &[u32]) -> bool {
    let antes_avesso = defect_count(mesh);
    let antes_forma = crate::quad_shape(mesh);
    let antes_abertas = open_edges(mesh);
    let antes_ilhas = components(mesh);
    let log = std::env::var("PH2D_UNTANGLE_LOG").is_ok();
    let Some(laco) = hole_loop(mesh, grupo) else {
        if log {
            eprintln!(
                "[aba] grupo de {} face(s): o bordo NAO e' um laco so'",
                grupo.len()
            );
        }
        return false;
    };
    if laco.len() < 4 || laco.len() % 2 != 0 {
        if log {
            eprintln!("[aba] laco de {} lados: curto ou impar", laco.len());
        }
        return false;
    }
    let pos = mesh.positions();
    let mut centro = [0.0f32; 3];
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / laco.len() as f32;
    for &v in &laco {
        let p = pos[v as usize];
        for k in 0..3 {
            centro[k] += p[k] * inv;
        }
    }
    let centro = ph2d_remesh_iso::project_onto(surface, centro, crate::finish::bbox_seed(surface));

    let mut novas_pos = pos.to_vec();
    let c = u32::try_from(novas_pos.len()).unwrap_or(0);
    novas_pos.push(centro);
    let fora: std::collections::BTreeSet<u32> = grupo.iter().copied().collect();
    let mut novas_faces: Vec<ph2d_mesh::Face> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(i, _)| !fora.contains(&u32::try_from(*i).unwrap_or(0)))
        .map(|(_, f)| *f)
        .collect();
    for par in 0..laco.len() / 2 {
        let a = laco[2 * par];
        let b = laco[2 * par + 1];
        let d = laco[(2 * par + 2) % laco.len()];
        novas_faces.push(ph2d_mesh::Face::quad(a, b, d, c));
    }
    // ⛔⛔ **Os vértices INTERIORES ao disco ficam órfãos, e órfão não é neutro:** o `χ` conta-os
    // (esta linha já pagou isso — «doze órfãos, doze unidades»). Compactar antes de montar.
    let (novas_pos, novas_faces) = compactar(novas_pos, novas_faces);
    let Ok(candidata) = ph2d_mesh::Mesh::from_parts(novas_pos, novas_faces) else {
        return false;
    };
    // ⛔ **As quatro colunas, e todas têm de dar** — ver o doc de [`remove_flaps`].
    let depois = crate::quad_shape(&candidata);
    if log {
        eprintln!(
            "[aba] laco de {} lados: avesso {antes_avesso} -> {} | abertas {antes_abertas} -> {} | ilhas {antes_ilhas} -> {} | >60 {} -> {}",
            laco.len(),
            defect_count(&candidata),
            open_edges(&candidata),
            components(&candidata),
            antes_forma.skew_over_60,
            depois.skew_over_60,
        );
    }
    // ⭐⭐⭐ **A GUARDA É DO QUE O ARTISTA VÊ, e o tecto das faces feias é o TAMANHO DO REMENDO.**
    //
    // ⛔⛔ Medido (2026-09-03): com `>60` a não poder subir de todo, o remendo que levava as faces
    // do avesso de `2` para **`0`** era recusado porque o leque acrescentava **uma** face com
    // canto pior que `60°`. *Uma dobra é uma fenda preta na foto dele; uma face enviesada é
    // invisível* — e quem pesa a beleza é o selector, uma camada acima, que vê a saída inteira.
    //
    // ⇒ o que se exige aqui é: **o resto da malha não piora** (o tecto é `L/2`, que é o número
    // de faces que o leque acrescenta — no pior caso todas elas são feias) e **nada do que se
    // vê aparece de novo** (faces do avesso, furos, ilhas).
    if defect_count(&candidata) >= antes_avesso
        || open_edges(&candidata) > antes_abertas
        || components(&candidata) > antes_ilhas
        || depois.skew_over_60 > antes_forma.skew_over_60 + laco.len() / 2
    {
        return false;
    }
    *mesh = candidata;
    true
}

/// O bordo do grupo, como **um** laço orientado para o preenchimento — `None` se não for um.
///
/// ⚠️ **A direcção vem da face de FORA**, e não das faces do grupo: elas estão do avesso, logo a
/// volta delas não diz de que lado fica o buraco.
fn hole_loop(mesh: &Mesh, grupo: &[u32]) -> Option<Vec<u32>> {
    let dentro: std::collections::BTreeSet<u32> = grupo.iter().copied().collect();
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for (i, f) in mesh.faces().iter().enumerate() {
        let i = u32::try_from(i).unwrap_or(0);
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut seguinte: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
    for (aresta, quem) in &por_aresta {
        let de_dentro = quem.iter().filter(|i| dentro.contains(i)).count();
        if de_dentro != 1 || quem.len() != 2 {
            continue;
        }
        let de_fora = *quem.iter().find(|i| !dentro.contains(i))?;
        let v = mesh.faces()[de_fora as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if (a.min(b), a.max(b)) == *aresta {
                // ⭐ O preenchimento percorre ao CONTRÁRIO da face de fora.
                if seguinte.insert(b, a).is_some() {
                    return None;
                }
            }
        }
    }
    if seguinte.len() < 4 {
        return None;
    }
    let inicio = *seguinte.keys().next()?;
    let mut laco = vec![inicio];
    let mut v = *seguinte.get(&inicio)?;
    while v != inicio {
        if laco.len() > seguinte.len() {
            return None;
        }
        laco.push(v);
        v = *seguinte.get(&v)?;
    }
    // ⛔ **Um laço SÓ** — se sobraram arestas de bordo, a aba não é um disco.
    if laco.len() != seguinte.len() {
        return None;
    }
    Some(laco)
}

/// Arestas com uma face só **mais** as não-manifold — a mesma soma que o selector do produto usa.
fn open_edges(mesh: &Mesh) -> usize {
    let mut n: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    n.values().filter(|c| **c != 2).count()
}

/// Quantas peças desligadas a malha tem.
fn components(mesh: &Mesh) -> usize {
    let mut viz: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            viz.entry(a).or_default().push(b);
            viz.entry(b).or_default().push(a);
        }
    }
    let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut n = 0usize;
    for &s in viz.keys() {
        if vistos.contains(&s) {
            continue;
        }
        n += 1;
        let mut pilha = vec![s];
        vistos.insert(s);
        while let Some(u) = pilha.pop() {
            for &w in viz.get(&u).map(Vec::as_slice).unwrap_or(&[]) {
                if vistos.insert(w) {
                    pilha.push(w);
                }
            }
        }
    }
    n
}

/// Deita fora os vértices que nenhuma face usa, e renumera.
fn compactar(
    pos: Vec<[f32; 3]>,
    faces: Vec<ph2d_mesh::Face>,
) -> (Vec<[f32; 3]>, Vec<ph2d_mesh::Face>) {
    let mut mapa = vec![u32::MAX; pos.len()];
    let mut novas_pos: Vec<[f32; 3]> = Vec::with_capacity(pos.len());
    for f in &faces {
        for &v in f.verts() {
            if mapa[v as usize] == u32::MAX {
                mapa[v as usize] = u32::try_from(novas_pos.len()).unwrap_or(0);
                novas_pos.push(pos[v as usize]);
            }
        }
    }
    let novas_faces = faces
        .iter()
        .map(|f| {
            let v = f.verts();
            ph2d_mesh::Face::quad(
                mapa[v[0] as usize],
                mapa[v[1 % v.len()] as usize],
                mapa[v[2 % v.len()] as usize],
                mapa[v[3 % v.len()] as usize],
            )
        })
        .collect();
    (novas_pos, novas_faces)
}
