//! **FASE 4a — TRAÇAR cada saída até à sua parceira.**
//!
//! Do ponto de partida, na direcção da saída, caminha-se **triângulo a triângulo**,
//! acumulando as transições, até o alvo cair dentro do triângulo corrente. O alvo
//! está a **uma célula** de distância: é o ponto de grade vizinho.
//!
//! ⭐ **A regra de escolha da aresta de saída é o que faz os casos especiais
//! desaparecerem.** Quando o segmento passa exactamente por um vértice, as duas
//! arestas candidatas intersectam-no; escolher a que tem **menos vértices sobre o
//! segmento** faz isolinhas que passam por vértices — e triângulos degenerados numa
//! linha — deixarem de precisar de tratamento próprio.
//!
//! ⛔⛔ **A mudança de orientação tem de ser vista e respondida.** Se ao passar de um
//! triângulo para o seguinte o sinal da área muda, origem e alvo trocam e a direcção
//! inverte-se — sem isso o traço atravessa uma dobra e **sai a andar para trás**.
//!
//! ⛔ **Bordo:** uma aresta com uma face só **aborta** o traço, e a saída fica
//! **pendente**. Saídas pendentes são ignoradas na extracção de células.

use crate::exact::{P, Xf, opposite, step};
use crate::ingest::Topo;
use crate::ports::Ports;

/// O tecto de triângulos que um traço pode atravessar.
///
/// ⚠️ **É um tecto de SANIDADE e não de qualidade.** O alvo está a uma célula de
/// distância e um triângulo mede da ordem de uma célula, então um traço são gasta
/// unidades de passos; um que gaste centenas está a andar em círculo sobre um mapa
/// que não fecha, e o que interessa é que ele **pare e seja contado**.
const MAX_STEPS: usize = 256;

#[path = "walk_geom.rs"]
mod geom;
pub(crate) use geom::face_sign;
use geom::{contains, crosses, exit_side, on_edge_side};

#[path = "walk_stats.rs"]
mod stats;
pub(crate) use stats::WalkStats;

/// **O modo do resgate pela gémea.** Ausente/`1` = só `opposite(d2)`, **o comportamento de
/// sempre** · `0` = a regra derivada da dobra · `2` = só `d2` · `3` = a união das duas.
///
/// ⛔⛔ **Nenhum dos três modos alternativos shipa, e os três são a MESMA medição:** todos
/// levam a `sculpt_wrinkled` à perfeição (`χ = +2`, `0` bordo) e a `sculpt_eared` a `4`
/// bordo, e todos pioram a `sculpt_hooked` (`17` bordo, `χ = −1` contra `10`/`0`).
/// *Onde só existe uma candidata, escolher pela dobra e tentar as duas escolhem a mesma —
/// e algumas dessas candidatas são as erradas.*
/// ⭐⭐⭐ **OS PARES EM QUE CADA LADO NOMEIA O OUTRO** — a lei do passe mútuo, sozinha.
///
/// ⚠️ **Está separada da travessia de propósito:** o critério é sobre a TABELA de
/// candidatas e não precisa da malha, então pode ser gateado sem fixtura nenhuma — e é a
/// única parte desta wave que uma mutação consegue matar num teste barato.
///
/// Devolve cada par **uma vez** (`i < j`), e conta as candidatas sem correspondência.
fn mutual_links(cand: &[Vec<(u32, Xf)>], st: &mut WalkStats) -> Vec<(u32, u32, Xf)> {
    let mut out = Vec::new();
    for (i, slot) in cand.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let me = i as u32;
        // ⭐⭐⭐ **VÁRIAS candidatas por porta, e a mutualidade escolhe entre elas.**
        //
        // ⚠️ *A ambiguidade do leque é «qual das rotas», e oferecer só a primeira é
        // escolher* — foi medido: a 1.ª rota é o palpite errado e o passe mútuo recusava-a
        // sempre. ⭐ Com a lista, quem decide é a reciprocidade e não a ordem.
        let hit = slot.iter().find(|&&(j, _)| {
            cand.get(j as usize)
                .is_some_and(|b| b.iter().any(|&(k, _)| k == me))
        });
        let Some(&(j, xf)) = hit else {
            st.rescue_not_mutual += usize::from(!slot.is_empty());
            continue;
        };
        // ⚠️ **Uma vez por par.** Sem isto o segundo lado volta a entrar, e a contagem de
        // pares diz o dobro do que se ligou.
        if me < j {
            st.rescue_mutual += 1;
            out.push((me, j, xf));
        }
    }
    out
}

/// ⭐ **O passe MÚTUO** — ligado por omissão; `PH2D_RESCUE_MUTUAL=0` desliga-o.
/// ⛔⛔ **O leque AMBÍGUO regista candidatas para o passe mútuo** — `PH2D_FAN_MUTUAL=1`
/// liga-o. **Nasce desligado, e é uma recusa MEDIDA.**
///
/// A guarda de 2026-08-26 recusava o leque de uma **singularidade** porque *«escolher uma
/// rota é um palpite»*, e o passe mútuo é a máquina que torna um palpite seguro. ⛔ **Medido
/// (2026-08-27): nem uma candidata do leque é recíproca** — nem a primeira, nem, com a
/// lista, todas elas: `0` pares novos e `0` arestas de bordo a menos nas três peças.
///
/// ⇒ *a rota do leque não aponta ao par certo em direcção nenhuma.* Tabela e mecanismo:
/// `docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` **§23.34**.
fn fan_mutual() -> bool {
    std::env::var("PH2D_FAN_MUTUAL").ok().as_deref() == Some("1")
}

fn mutual_pass() -> bool {
    std::env::var("PH2D_RESCUE_MUTUAL").ok().as_deref() != Some("0")
}

fn rescue_mode() -> u8 {
    std::env::var("PH2D_RESCUE_DIR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

/// ⭐ **O TRAÇADO DE TODAS AS SAÍDAS.**
pub(crate) fn trace_all(topo: &Topo, ports: &mut Ports) -> WalkStats {
    let mut st = WalkStats::default();
    let mut orphan_at: Vec<[f64; 3]> = Vec::new();
    let mut miss_cells: Vec<f64> = Vec::new();
    let mut tri_cells: Vec<f64> = Vec::new();
    let mut cand: Vec<Vec<(u32, Xf)>> = vec![Vec::new(); ports.ports.len()];
    for i in 0..ports.ports.len() {
        if ports.ports[i].link.is_some() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        match trace_one(topo, ports, i as u32, &mut st, &mut cand) {
            Outcome::Linked(j, acc) => {
                // ⛔ **A parceira já tem dona.** Sobrescrever partiria a ligação dela
                // ao meio: uma meia-ligação sem a outra metade.
                if ports.ports[j as usize].link.is_some() {
                    st.contested += 1;
                    continue;
                }
                ports.ports[i].link = Some(j);
                ports.ports[i].link_xf = acc;
                ports.ports[j as usize].link = Some(i.try_into().unwrap_or(u32::MAX));
                ports.ports[j as usize].link_xf = acc.inverse();
                st.linked += 1;
            }
            Outcome::Boundary => st.boundary += 1,
            Outcome::Orphan(no_exit, f, o_outside, entry_only, miss, tri) => {
                st.orphan += 1;
                if !no_exit && entry_only {
                    // ⚠️ No ramo «sem parceira» o 4.º campo carrega **outra** pergunta:
                    // *o alvo caiu sobre uma aresta?* Ver o sítio onde ele é escrito.
                    st.orphan_no_partner_on_edge += 1;
                }
                if !no_exit && o_outside {
                    // ⚠️ No ramo «sem parceira» o 3.º campo carrega **outra** pergunta:
                    // *havia nó naquele ponto?* Ver o sítio onde ele é escrito.
                    st.orphan_no_partner_node_exists += 1;
                }
                if no_exit {
                    st.orphan_no_exit += 1;
                    if face_sign(topo, f) == 0 {
                        st.orphan_no_exit_flat += 1;
                    }
                    if o_outside {
                        st.orphan_no_exit_o_outside += 1;
                    }
                    if entry_only {
                        st.orphan_no_exit_entry_only += 1;
                    }
                    miss_cells.push(miss);
                    tri_cells.push(tri);
                } else {
                    st.orphan_no_partner += 1;
                }
                let t = topo.p3[f];
                orphan_at.push([
                    (t[0][0] + t[1][0] + t[2][0]) / 3.0,
                    (t[0][1] + t[1][1] + t[2][1]) / 3.0,
                    (t[0][2] + t[1][2] + t[2][2]) / 3.0,
                ]);
            }
            Outcome::Runaway => st.runaway += 1,
        }
    }
    // ⭐⭐⭐ **O PASSE MÚTUO** — o desempate por CORRECÇÃO, e ele não precisa de traçar
    // nada de novo.
    //
    // ⛔⛔ A §23.29 mediu e rejeitou ligar a candidata da outra convenção assim que ela
    // aparece: algumas são as **erradas** (a `sculpt_hooked` ia de `χ = 0` para `−1`). E
    // nomeou o critério que falta — *«a parceira certa é a que, traçada de volta, regressa
    // a esta porta»* — parecendo pedir um segundo traçado.
    //
    // ⭐ **Não pede:** as duas pontas já foram traçadas, e cada uma registou a sua
    // candidata. *Se o par é genuíno, cada lado nomeia o outro.* ⇒ liga-se `i ↔ j` só
    // quando `cand[i] = j` **e** `cand[j] = i`.
    if mutual_pass() {
        for (i, j, xf) in mutual_links(&cand, &mut st) {
            if ports.ports[i as usize].link.is_some() || ports.ports[j as usize].link.is_some() {
                continue;
            }
            ports.ports[i as usize].link = Some(j);
            ports.ports[i as usize].link_xf = xf;
            ports.ports[j as usize].link = Some(i);
            ports.ports[j as usize].link_xf = xf.inverse();
            st.linked += 1;
        }
    }
    miss_cells.sort_by(f64::total_cmp);
    #[allow(clippy::cast_possible_truncation)]
    {
        st.orphan_miss_cells_p50 =
            miss_cells.get(miss_cells.len() / 2).copied().unwrap_or(0.0) as f32;
        tri_cells.sort_by(f64::total_cmp);
        st.orphan_tri_cells_p50 = tri_cells.get(tri_cells.len() / 2).copied().unwrap_or(0.0) as f32;
    }
    summarise_orphans(topo, &orphan_at, &mut st);
    st
}

/// ⭐ **ONDE as órfãs morrem** — ver [`WalkStats::orphan_radius_p50`].
///
/// ⚠️ O centro é o das faces e não o dos vértices: aqui só existe a `p3`, e a diferença
/// entre os dois centróides é irrelevante para uma banda de raio.
fn summarise_orphans(topo: &Topo, at: &[[f64; 3]], st: &mut WalkStats) {
    if topo.p3.is_empty() {
        return;
    }
    let mid = |t: &[[f64; 3]; 3]| {
        [
            (t[0][0] + t[1][0] + t[2][0]) / 3.0,
            (t[0][1] + t[1][1] + t[2][1]) / 3.0,
            (t[0][2] + t[1][2] + t[2][2]) / 3.0,
        ]
    };
    let all: Vec<[f64; 3]> = topo.p3.iter().map(mid).collect();
    let c = all
        .iter()
        .fold([0.0f64; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / all.len() as f64;
    let c = [c[0] * inv, c[1] * inv, c[2] * inv];
    let r = |p: &[f64; 3]| {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let mut base: Vec<f64> = all.iter().map(r).collect();
    base.sort_by(f64::total_cmp);
    let med = base[base.len() / 2].max(1.0e-12);
    #[allow(clippy::cast_possible_truncation)]
    {
        st.piece_radius_p99 = (base[base.len() * 99 / 100] / med) as f32;
        let mut v: Vec<f64> = at.iter().map(|p| r(p) / med).collect();
        v.sort_by(f64::total_cmp);
        st.orphan_radius_p50 = v.get(v.len() / 2).copied().unwrap_or(0.0) as f32;
    }
}

enum Outcome {
    Linked(u32, Xf),
    Boundary,
    /// ⚠️ `(nao_achou_saida, face, origem_fora, so_o_lado_de_entrada)` — a face é **onde**
    /// ela morreu e os `bool` dizem qual das avarias foi. *Uma órfã sem isto é um
    /// contador, e um contador não escolhe uma cura.*
    Orphan(bool, usize, bool, bool, f64, f64),
    Runaway,
}

fn trace_one(
    topo: &Topo,
    ports: &Ports,
    id: u32,
    st: &mut WalkStats,
    cand: &mut [Vec<(u32, Xf)>],
) -> Outcome {
    let p = ports.ports[id as usize];
    let mut face = p.face as usize;
    let mut o = p.at;
    let mut t = [
        p.at[0] + step(p.dir, topo.one)[0],
        p.at[1] + step(p.dir, topo.one)[1],
    ];
    let mut dir = p.dir;
    let mut acc = Xf::IDENTITY;
    let mut entry: Option<usize> = None;
    for _ in 0..MAX_STEPS {
        st.steps += 1;
        if contains(topo, face, t) {
            #[allow(clippy::cast_possible_truncation)]
            let key = (face as u32, t[0], t[1], opposite(dir));
            return match ports.by_key.get(&key) {
                Some(&j) if j != id => Outcome::Linked(j, acc),
                // ⭐⭐⭐ **AS DUAS AVARIAS QUE «SEM PARCEIRA» ESCONDE**, e elas pedem curas
                // opostas: ou **não há nó nenhum** naquele ponto de grade (o traço chegou a
                // um sítio que ninguém marcou), ou **há nó e falta-lhe a cardinal de
                // volta** — o leque colapsado do §6.4. *Um contador só não escolhe entre
                // «construir o nó» e «emitir a saída».*
                _ => {
                    #[allow(clippy::cast_possible_truncation)]
                    let node_here =
                        (0..4u8).any(|d| ports.by_key.contains_key(&(face as u32, t[0], t[1], d)));
                    // ⭐⭐⭐ **O RESGATE PELO LEQUE — o TERCEIRO dono.**
                    //
                    // ⛔⛔ Um ponto de grade sobre um **vértice** é um nó `Site::Vertex`, e as
                    // saídas dele estão espalhadas pelo **leque**: cada uma foi emitida com a
                    // **sua** face, e a chave é `(face, u, v, dir)`. ⇒ quem chega ao canto por
                    // uma face qualquer do leque procura com a chave dela e não acha.
                    //
                    // Medido 2026-08-26 na `sculpt_t003`: as **4** órfãs que sobram ao resgate
                    // pela face gémea caem **todas num canto** (`num CANTO: 4` de `4`).
                    //
                    // ⚠️ **A transição é a do leque** (`to_here`), e a **paridade das
                    // inversões de sinal** é contada corner a corner — a mesma lei do laço,
                    // que salta as faces de sinal `0`. *Compor as transições e esquecer o
                    // sinal procura a cardinal oposta em metade dos leques.*
                    if let Some(kk) = (0..3).find(|&i| topo.uv[face][i] == t) {
                        st.orphan_on_corner += 1;
                        let fan = crate::fan::fan_of(topo, crate::fan::Corner::new(face, kk));
                        // ⛔⛔⛔ **SÓ QUANDO A RESPOSTA NÃO DEPENDE DO CAMINHO.**
                        //
                        // Num leque **fechado**, ir de um canto a outro pela ordem do leque ou
                        // pelo outro lado dá transições que diferem pela **holonomia**. Se ela
                        // não é a identidade — que é precisamente o que uma **singularidade**
                        // é — as duas rotas apontam para saídas **diferentes** do mesmo
                        // vértice, e escolher uma é um palpite.
                        //
                        // ⚠️ **Medido 2026-08-26, e foi o `cube` que o disse:** sem esta
                        // guarda o resgate corria `2` vezes ali e as arestas de bordo iam de
                        // `4` para **`6`** — *ligar ao par errado abre mais buracos do que
                        // deixar a órfã em paz*.
                        //
                        // ⚠️ Um leque **aberto** (vértice de bordo) não tem ambiguidade: há um
                        // caminho só.
                        let unambiguous =
                            fan.holonomy.is_none_or(|h| h == crate::exact::Xf::IDENTITY);
                        if !unambiguous {
                            st.fan_ambiguous += 1;
                        }
                        if let Some(i0) = fan
                            .corners
                            .iter()
                            .position(|c| c.f() == face && c.kk() == kk)
                        {
                            let base = fan.to_here[i0].inverse();
                            for i in 0..fan.corners.len() {
                                // ⭐⭐⭐ **O LEQUE AMBÍGUO DEIXA DE RECUSAR — passa a
                                // REGISTAR.**
                                //
                                // ⛔ A guarda existia porque *«escolher uma rota é um
                                // palpite»*: num leque fechado com holonomia ≠ identidade
                                // (uma **singularidade**) as duas rotas apontam a saídas
                                // diferentes, e medido em 2026-08-26 o palpite levava as
                                // arestas de bordo de `4` a `6` no `cube`.
                                //
                                // ⭐ **O passe mútuo dissolve essa razão:** um palpite só
                                // liga se o outro lado apontar de volta. ⇒ aqui regista-se
                                // a candidata e a decisão é lá.
                                if !unambiguous && !fan_mutual() {
                                    break;
                                }
                                if i == i0 {
                                    continue;
                                }
                                let x = base.then(fan.to_here[i]);
                                let t2 = x.apply(t);
                                let mut d2 = x.dir(dir);
                                // A paridade das inversões ao longo do troço do leque.
                                let (lo, hi) = (i0.min(i), i0.max(i));
                                let flips = (lo..hi)
                                    .filter(|&w| {
                                        let (a, b) = (
                                            face_sign(topo, fan.corners[w].f()),
                                            face_sign(topo, fan.corners[w + 1].f()),
                                        );
                                        a != 0 && b != 0 && a != b
                                    })
                                    .count();
                                if flips % 2 == 1 {
                                    d2 = opposite(d2);
                                }
                                #[allow(clippy::cast_possible_truncation)]
                                let key = (fan.corners[i].f() as u32, t2[0], t2[1], opposite(d2));
                                if let Some(&j) = ports.by_key.get(&key)
                                    && j != id
                                {
                                    if unambiguous {
                                        st.orphan_rescued_in_fan += 1;
                                        return Outcome::Linked(j, acc.then(x));
                                    }
                                    // ⚠️ **Primeira candidata só**, e a decisão não é
                                    // aqui: se o outro lado não a nomear de volta, o passe
                                    // mútuo recusa-a.
                                    if !cand[id as usize].iter().any(|&(k, _)| k == j) {
                                        st.fan_candidate += 1;
                                        cand[id as usize].push((j, acc.then(x)));
                                    }
                                }
                            }
                        }
                    }
                    // ⭐⭐⭐ **O RESGATE: a chave é de OUTRA PESSOA, então pergunta-se a ela.**
                    //
                    // ⛔⛔ O comentário abaixo nomeia a avaria desde 2026-08-25 e ninguém a
                    // curou: um nó de aresta nasce **uma vez**, no lado canónico, e fica
                    // registado com a face desse lado. ⇒ **procura-se a mesma chave na face
                    // GÉMEA**, transportada pela transição daquele lado.
                    //
                    // ⚠️ **A troca de direcção quando o sinal da área inverte é a MESMA do
                    // laço** — sem ela o resgate procura a cardinal errada numa dobra, que é
                    // exactamente onde estas órfãs vivem.
                    //
                    // Medido 2026-08-26: na `sculpt_t003` as **4** órfãs «sem parceira» são
                    // **`4` sobre uma aresta**, e na `t001` são `2` de `2`.
                    // ⭐⭐⭐ **POR QUE FALHA O RESGATE?** Três razões, três curas
                    // diferentes: o alvo não está sobre aresta nenhuma · está mas a aresta
                    // não tem gémea · tem gémea e a CHAVE não bate lá. *Um contador único
                    // de «não resgatadas» lê as três como uma.*
                    let side = on_edge_side(topo.uv[face], t);
                    if side.is_none() {
                        st.rescue_no_side += 1;
                    }
                    if let Some(k) = side
                        && topo.twin[face][k].is_none()
                    {
                        st.rescue_no_twin += 1;
                    }
                    if let Some(k) = side
                        && let Some((g, _)) = topo.twin[face][k]
                    {
                        let x = topo.xf[face][k];
                        let t2 = x.apply(t);
                        let mut d2 = x.dir(dir);
                        let (before, after) = (face_sign(topo, face), face_sign(topo, g as usize));
                        if before != 0 && after != 0 && before != after {
                            d2 = opposite(d2);
                        }
                        // ⭐⭐⭐ **A CONVENÇÃO DA DIRECÇÃO AO ATRAVESSAR** — medida, não
                        // adivinhada. `PH2D_RESCUE_DIR=1` usa `d2` cru; o de omissão é o
                        // `opposite(d2)` de sempre. Quantas parceiras cada uma acharia
                        // (2026-08-27, das que ficavam por resgatar):
                        //
                        // ⛔ **Nenhuma convenção única serve, e a regra derivada da
                        // dobra também não** — ela dá o mesmo que tentar as duas, porque
                        // *as duas nunca colidem* (`rescue_ambiguous = 0`). As tabelas, as
                        // quatro tentativas e o que cada uma mediu:
                        // `docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` **§23.27–§23.30**.
                        //
                        // ⭐ O que decide é o **passe mútuo**, no fim do laço: regista-se a
                        // candidata da outra convenção e só se liga se o outro lado a
                        // nomear de volta.
                        // A regra derivada (modo `0`): `opposite(d2)` sse as **duas**
                        // faces estão dobradas. ⛔ Ela existe para a bissecção, não para
                        // shipar — ver a §23.29.
                        let both_folded = before < 0 && after < 0;
                        let order: &[u8] = match rescue_mode() {
                            1 => &[0],
                            2 => &[1],
                            3 => &[0, 1],
                            _ if both_folded => &[0],
                            _ => &[1],
                        };
                        let hit = order.iter().find_map(|&which| {
                            let want = if which == 0 { opposite(d2) } else { d2 };
                            ports
                                .by_key
                                .get(&(g, t2[0], t2[1], want))
                                .copied()
                                .map(|j| (j, want))
                        });
                        match hit.map(|(j, _)| j).as_ref() {
                            Some(&j) if j != id => {
                                st.orphan_rescued_across_edge += 1;
                                return Outcome::Linked(j, acc.then(x));
                            }
                            Some(_) => st.rescue_self += 1,
                            None => {
                                st.rescue_no_key += 1;
                                // ⭐⭐⭐ **A OUTRA CONVENÇÃO TEM CANDIDATA — REGISTA-SE, NÃO
                                // SE LIGA.** Ligar aqui é o que a §23.29 mediu e rejeitou:
                                // algumas dessas candidatas são as **erradas**. O desempate
                                // por CORRECÇÃO faz-se depois, e é barato: *se o par é
                                // genuíno, cada lado nomeia o outro.*
                                let alt = if order[0] == 0 { d2 } else { opposite(d2) };
                                if let Some(&j) = ports.by_key.get(&(g, t2[0], t2[1], alt))
                                    && j != id
                                {
                                    cand[id as usize].push((j, acc.then(x)));
                                }
                                // ⭐⭐⭐ **QUAL CONVENÇÃO acharia a parceira?** Duas
                                // perguntas ortogonais — inverter a direcção ao atravessar
                                // (`opposite`) e trocar a cardinal quando o sinal da área
                                // inverte —, e as quatro combinações medem-se de uma vez.
                                // *Adivinhar uma delas custa uma corrida por tentativa; a
                                // tabela custa uma.*
                                let raw = x.dir(dir);
                                for (bit, cand) in [
                                    (0usize, raw),
                                    (1, opposite(raw)),
                                    (2, d2),
                                    (3, opposite(d2)),
                                ] {
                                    if ports
                                        .by_key
                                        .get(&(g, t2[0], t2[1], cand))
                                        .is_some_and(|&j| j != id)
                                    {
                                        st.rescue_would[bit] += 1;
                                    }
                                }
                                // ⭐ E o ponto transportado tem ALGUMA porta na gémea, com
                                // outra direcção? *Separa «a chave está errada na direcção»
                                // de «não há nada ali».*
                                if (0u8..4)
                                    .any(|d| ports.by_key.contains_key(&(g, t2[0], t2[1], d)))
                                {
                                    st.rescue_wrong_dir += 1;
                                }
                                // ⭐⭐⭐ **QUAL cardinal, RELATIVO ao `d2`?** As duas
                                // convenções são `+0` e `+2`; se o que existe estiver em
                                // `+1`/`+3`, a direcção transportada está a **um quarto de
                                // volta** do que devia — que é outro defeito, com outra
                                // cura, e não mais uma convenção.
                                let mut any = false;
                                for k in 0u8..4 {
                                    if ports
                                        .by_key
                                        .get(&(g, t2[0], t2[1], (d2 + k) & 3))
                                        .is_some_and(|&j| j != id)
                                    {
                                        st.rescue_offset[k as usize] += 1;
                                        any = true;
                                    }
                                }
                                // ⭐⭐⭐ **AS QUE NÃO TÊM PORTA NENHUMA ALI** — a população
                                // que a §23.31 mostrou ser de EMISSÃO e não de
                                // emparelhamento. A emissão dá portas aos dois lados de um
                                // nó de aresta, **mas só se o `sigma` daquele lado não for
                                // zero** ⇒ a 1.ª hipótese é a gémea ser **degenerada** no
                                // domínio. *Se não for, a cura está noutro sítio.*
                                if !any {
                                    st.rescue_no_port += 1;
                                    if face_sign(topo, g as usize) == 0 {
                                        st.rescue_no_port_flat += 1;
                                    }
                                    if face_sign(topo, face) == 0 {
                                        st.rescue_no_port_here_flat += 1;
                                    }
                                    // ⭐⭐⭐ **O PONTO É UM CANTO da gémea?** Um nó sobre um
                                    // VÉRTICE não nasce pelo caminho da aresta (o
                                    // `by_vertex_hit` salta-o) — ele nasce pelo **leque**,
                                    // e se o leque daquele vértice não tiver semente as
                                    // portas dele não existem em face nenhuma.
                                    if topo.uv[g as usize].contains(&t2) {
                                        st.rescue_no_port_corner += 1;
                                        // ⭐⭐⭐ **O NÓ DAQUELE CANTO TEM PORTAS NALGUM
                                        // SÍTIO?** As portas de um nó de vértice nascem
                                        // pelo LEQUE e ficam registadas com a face de cada
                                        // sector. ⇒ *«não há porta nesta face» e «este nó
                                        // não tem porta nenhuma» são dois defeitos
                                        // diferentes*: o 1.º é de indexação, o 2.º é o
                                        // leque a não emitir. As órfãs são poucas, e a
                                        // varredura linear paga-se.
                                        let anywhere = ports
                                            .ports
                                            .iter()
                                            .filter(|q| q.at == t2 && q.face == g)
                                            .count();
                                        let same_node =
                                            ports.ports.iter().filter(|q| q.at == t2).count();
                                        if same_node == 0 {
                                            st.rescue_corner_node_mute += 1;
                                        } else if anywhere == 0 {
                                            st.rescue_corner_other_faces += 1;
                                        }
                                    }
                                    // ⭐ E há porta em ALGUM ponto daquela face? *Separa «a
                                    // face não tem portas» de «não tem NESTE ponto».*
                                    if ports
                                        .by_key
                                        .range(
                                            (g, i64::MIN, i64::MIN, 0)..=(g, i64::MAX, i64::MAX, 3),
                                        )
                                        .next()
                                        .is_some()
                                    {
                                        st.rescue_no_port_face_has_others += 1;
                                    }
                                }
                            }
                        }
                    }
                    // ⭐⭐⭐ **ESTÁ O ALVO SOBRE UMA ARESTA do triângulo?**
                    //
                    // ⚠️ **A hipótese que isto testa:** um nó de aresta nasce **uma vez
                    // por aresta**, no lado *canónico* ([`crate::nodes::is_canonical`]), e
                    // fica registado com a FACE desse lado. Um traço que chegue ao mesmo
                    // ponto pela face **do outro lado** procura `(face, ponto, direcção)`
                    // com a *sua* face — e não acha nada. *O nó existe; a chave é que é
                    // de outra pessoa.*
                    Outcome::Orphan(
                        false,
                        face,
                        node_here,
                        on_edge_side(topo.uv[face], t).is_some(),
                        0.0,
                        0.0,
                    )
                }
            };
        }
        let Some(k) = exit_side(topo, face, entry, o, t) else {
            // ⚠️ As duas perguntas medem-se AQUI, com `o`, `t` e `entry` na mão — depois
            // do retorno já não existem. *Um diagnóstico que precisa do estado local
            // tem de nascer dentro do laço.*
            let o_outside = !contains(topo, face, o);
            // A distância do ALVO ao vértice mais próximo do triângulo, em células.
            let cell = |a: P, b: P| {
                (((a[0] - b[0]) as f64) / topo.one as f64)
                    .hypot(((a[1] - b[1]) as f64) / topo.one as f64)
            };
            let miss = topo.uv[face]
                .iter()
                .map(|v| cell(*v, t))
                .fold(f64::INFINITY, f64::min);
            let tri = (0..3)
                .map(|k| cell(topo.uv[face][k], topo.uv[face][(k + 1) % 3]))
                .fold(0.0f64, f64::max);
            let entry_only = entry.is_some_and(|k| {
                let a = topo.uv[face][k];
                let b = topo.uv[face][(k + 1) % 3];
                crosses(o, t, a, b)
            });
            return Outcome::Orphan(true, face, o_outside, entry_only, miss, tri);
        };
        let Some((g, j)) = topo.twin[face][k] else {
            return Outcome::Boundary;
        };
        let x = topo.xf[face][k];
        let before = face_sign(topo, face);
        let after = face_sign(topo, g as usize);
        acc = acc.then(x);
        o = x.apply(o);
        t = x.apply(t);
        dir = x.dir(dir);
        face = g as usize;
        entry = Some(j as usize);
        // ⛔⛔ **A dobra vista pelo SINAL DA ÁREA.** Sem esta troca o traço
        // atravessa a dobra e passa a andar para trás — e o sintoma não é um erro,
        // é uma malha com faces a menos e ninguém a dizer porquê.
        if before != 0 && after != 0 && before != after {
            core::mem::swap(&mut o, &mut t);
            dir = opposite(dir);
            st.flips += 1;
        }
    }
    Outcome::Runaway
}

#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;
