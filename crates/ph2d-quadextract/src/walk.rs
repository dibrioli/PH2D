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

use crate::exact::{P, Xf, opposite, orient, step};
use crate::ingest::Topo;
use crate::ports::Ports;

/// O tecto de triângulos que um traço pode atravessar.
///
/// ⚠️ **É um tecto de SANIDADE e não de qualidade.** O alvo está a uma célula de
/// distância e um triângulo mede da ordem de uma célula, então um traço são gasta
/// unidades de passos; um que gaste centenas está a andar em círculo sobre um mapa
/// que não fecha, e o que interessa é que ele **pare e seja contado**.
const MAX_STEPS: usize = 256;

/// O que o traçado mediu de si próprio.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalkStats {
    /// Saídas emparelhadas.
    pub linked: usize,
    /// ⚠️ Saídas que morreram no **bordo** — esperado numa malha aberta.
    pub boundary: usize,
    /// ⛔ Saídas que chegaram e **não** acharam a parceira no destino.
    pub orphan: usize,
    /// ⛔⛔ **A METADE das órfãs que chegou ao ponto de grade e não achou lá ninguém.**
    ///
    /// ⚠️ **A contagem única não distingue duas avarias diferentes**, e elas têm curas
    /// opostas: esta diz que o NÓ do outro lado não emitiu a cardinal de volta (leque
    /// colapsado, §6.4); a irmã diz que o traço nem conseguiu sair do triângulo.
    pub orphan_no_partner: usize,
    /// ⛔⛔ A outra metade: a [`exit_side`] não achou por onde sair do triângulo — o
    /// sintoma de uma carta **dobrada**, onde o segmento não cruza lado nenhum.
    pub orphan_no_exit: usize,
    /// ⛔ Destas, quantas tinham a ORIGEM já **fora** do triângulo em que estavam.
    ///
    /// ⚠️⚠️ **MEDIDO e sem valor de diagnóstico: é o caso NORMAL.** O `o` nunca é
    /// actualizado para o ponto de travessia — ele é a origem do segmento, transportada
    /// carta a carta —, logo depois do primeiro salto ele está **sempre** do lado de fora
    /// do triângulo em que o traço entrou. *Um contador que dá 100 % não separa nada, e a
    /// leitura de que «a origem estar fora é a avaria» estava errada.* Fica como controlo.
    pub orphan_no_exit_o_outside: usize,
    /// ⭐⭐⭐ **Das órfãs «sem parceira», quantas chegaram a um ponto que TEM nó** — e
    /// portanto ao qual só falta a cardinal de volta (leque colapsado, §6.4).
    ///
    /// ⚠️ A diferença para o total de «sem parceira» são as que chegaram a um ponto **sem
    /// nó nenhum**. *As duas leem-se igual no relatório antigo e pedem curas opostas:
    /// construir o nó, ou fazê-lo emitir a saída.*
    pub orphan_no_partner_node_exists: usize,
    /// ⭐⭐⭐ **Das órfãs «sem parceira», quantas caíram sobre uma ARESTA do triângulo.**
    ///
    /// ⚠️ Um nó de aresta nasce **uma vez por aresta**, no lado canónico, e fica registado
    /// com a face desse lado. *Quem chega pela outra face procura com a chave dele e não
    /// acha — o nó existe, a chave é que é de outra pessoa.*
    pub orphan_no_partner_on_edge: usize,
    /// ⭐⭐⭐ **Quantas órfãs o RESGATE salvou** — o nó existia na face gémea.
    ///
    /// ⚠️ **Zero aqui não é bom nem mau sozinho:** ele só quer dizer que nenhuma órfã caiu
    /// numa aresta nesta peça. *A régua é este número contra
    /// [`WalkStats::orphan_no_partner_on_edge`], que é quantas ficaram por salvar.*
    pub orphan_rescued_across_edge: usize,
    /// ⛔ Por que o resgate pela gémea **não** disparou: o alvo não está sobre aresta …
    pub rescue_no_side: usize,
    /// … está, mas a aresta não tem gémea (é bordo).
    pub rescue_no_twin: usize,
    /// … tem gémea, e a chave transportada **não existe** lá.
    pub rescue_no_key: usize,
    /// … dessas, quantas têm ALGUMA porta no mesmo ponto, com **outra direcção**.
    pub rescue_wrong_dir: usize,
    /// … a chave existe e é a **própria** porta.
    pub rescue_self: usize,
    /// ⭐⭐⭐ Quantas seriam resgatadas por cada convenção de direcção:
    /// `[x.dir(dir), oposta, com a troca do sinal da área, oposta dessa]`.
    /// ⚠️ *O índice `3` é o que o código usa hoje — e ele conta `0` por construção aqui,
    /// porque este ramo só corre quando ele falhou.*
    pub rescue_would: [usize; 4],
    /// ⭐ **Das órfãs «sem parceira», quantas caíram num CANTO do triângulo.**
    ///
    /// ⚠️ Um canto é um nó de **vértice**, registado com a face canónica do leque — um
    /// terceiro dono possível, que o resgate por um lado só não alcança.
    pub orphan_on_corner: usize,
    /// ⭐⭐⭐ **Quantas órfãs o resgate pelo LEQUE salvou** — o nó era de vértice.
    pub orphan_rescued_in_fan: usize,
    /// ⭐ **O DIÂMETRO do triângulo em que a órfã morreu**, em células — a régua com que
    /// a linha de baixo se lê.
    ///
    /// ⛔⛔ **Sem ela a distância é ambígua e a 1.ª leitura foi errada:** `3,0` células de
    /// distância ao vértice mais próximo é enorme num triângulo de `0,2` células e é
    /// *estar quase lá* num de `6`. *Uma distância sem a escala do objecto a que se mede
    /// não é uma medição.*
    pub orphan_tri_cells_p50: f32,
    /// ⭐⭐⭐ **A QUE DISTÂNCIA o segmento passa do triângulo, em CÉLULAS de grade.**
    ///
    /// ⚠️ **É a régua que separa as duas curas possíveis:** se o segmento falha o triângulo
    /// por uma fracção de célula, a avaria é de **fronteira** (um `<` onde devia estar um
    /// `<=`, uma travessia por um vértice); se falha por células inteiras, o transporte
    /// levou-o para outra parte da peça e a avaria é **estrutural**. *Sem esta coluna as
    /// duas leem-se igual — «não achou saída» — e a cura errada é barata de escrever.*
    pub orphan_miss_cells_p50: f32,
    /// ⛔⛔ Destas, quantas TERIAM saída se o lado de ENTRADA fosse permitido.
    ///
    /// ⚠️ **É a hipótese de que a exclusão do lado de entrada é forte de mais numa
    /// dobra**: ali o traço tem de voltar por onde veio, e a regra que impede o
    /// ping-pong impede também isso.
    pub orphan_no_exit_entry_only: usize,
    /// ⛔⛔⛔ **Destas, quantas morreram num triângulo de ÁREA ZERO no domínio.**
    ///
    /// ⚠️ **A [`contains`] devolve `false` para SEMPRE quando a área é zero** (o `s == 0`
    /// sai logo à entrada), e nenhum lado de um triângulo achatado é cruzado por um
    /// segmento — *as duas portas fecham-se ao mesmo tempo, e o traço não tem para onde
    /// ir.* Separar esta contagem é o que distingue «o mapa dobrou» de «o mapa
    /// COLAPSOU», que são avarias diferentes com curas diferentes.
    pub orphan_no_exit_flat: usize,
    /// ⭐⭐⭐ **ONDE as órfãs morrem**, em raios normalizados pelo raio mediano da peça.
    ///
    /// ⛔ O report do artista de 2026-08-25 é sobre POSIÇÃO (*«furos nas pontas»*), e a
    /// órfã é o sintoma MAIS A MONTANTE da cadeia que produz um furo: órfã ⇒ saída
    /// pendente ⇒ célula abandonada ⇒ aresta de bordo.
    pub orphan_radius_p50: f32,
    /// O `p99` do raio normalizado de toda a peça — a régua da linha de cima.
    /// ⚠️ **É o p99, NÃO o máximo** — e o rótulo do instrumento dizia *«a peça vai até»*,
    /// que se lê como o máximo. Em 2026-08-26 isso fez uma célula colapsada a `1,54×` ser
    /// lida como *«um nó FORA da peça»* quando ela está no **1% mais externo**, que é a
    /// ponta. *Uma coluna cujo nome promete outra estatística lê-se ao contrário.*
    pub piece_radius_p99: f32,
    /// ⛔ Traços que estouraram o tecto de passos.
    pub runaway: usize,
    /// ⛔⛔ **Traços que chegaram a uma parceira JÁ EMPARELHADA com outra.**
    ///
    /// ⚠️ Acontece onde duas cartas se sobrepõem: dois talos diferentes chegam ao
    /// mesmo ponto de grade pela mesma direcção. *A primeira redacção sobrescrevia a
    /// ligação da outra e deixava o par ANTIGO a apontar para uma saída que já não
    /// apontava de volta* — uma meia-ligação assimétrica, que faz a extracção de
    /// células virar à esquerda para dentro de uma célula alheia. Era daí que saíam as
    /// quatro células de TRÊS lados que o teorema proíbe.
    pub contested: usize,
    /// Passos gastos, somados — a régua do custo.
    pub steps: usize,
    /// Quantas vezes o traço atravessou uma mudança de orientação.
    pub flips: usize,
}

/// ⛔ **A convenção alternativa da direcção no resgate pela gémea** — `PH2D_RESCUE_DIR=1`.
/// Ver a tabela no sítio onde ela é lida.
fn rescue_dir_raw() -> bool {
    std::env::var("PH2D_RESCUE_DIR").ok().as_deref() == Some("1")
}

/// ⭐ **O TRAÇADO DE TODAS AS SAÍDAS.**
pub(crate) fn trace_all(topo: &Topo, ports: &mut Ports) -> WalkStats {
    let mut st = WalkStats::default();
    let mut orphan_at: Vec<[f64; 3]> = Vec::new();
    let mut miss_cells: Vec<f64> = Vec::new();
    let mut tri_cells: Vec<f64> = Vec::new();
    for i in 0..ports.ports.len() {
        if ports.ports[i].link.is_some() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        match trace_one(topo, ports, i as u32, &mut st) {
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

fn trace_one(topo: &Topo, ports: &Ports, id: u32, st: &mut WalkStats) -> Outcome {
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
                        if let Some(i0) = fan
                            .corners
                            .iter()
                            .position(|c| c.f() == face && c.kk() == kk)
                        {
                            let base = fan.to_here[i0].inverse();
                            for i in 0..fan.corners.len() {
                                if !unambiguous {
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
                                    st.orphan_rescued_in_fan += 1;
                                    return Outcome::Linked(j, acc.then(x));
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
                        // | peça | `x.dir` | oposta | `d2` | `opposite(d2)` (hoje) | total |
                        // |---|---|---|---|---|---|
                        // | `sculpt_wrinkled` | `0` | `7` | **`7`** | `0` | `8` |
                        // | `sculpt_hooked` | `1` | `2` | **`3`** | `0` | `8` |
                        // | `sculpt_eared` | `0` | `2` | **`2`** | `0` | `3` |
                        //
                        // ⚠️ *«Acha uma porta» não é «acha a porta certa»* — ligar uma
                        // parceira errada parte a malha em silêncio. ⛔⛔⛔ **E o veredito
                        // pelo RESULTADO desmente a coluna acima:**
                        //
                        // | peça | `opposite(d2)` (omissão) | ⛔ `d2` |
                        // |---|---|---|
                        // | `sculpt_wrinkled` | `10` bordo · `χ = +1` · `8` órfãs | ⭐ **`0` bordo · `χ = +2` · `0` órfãs** |
                        // | `sculpt_hooked` | `10` bordo · `χ = 0` | ⛔ `17` bordo · `χ = −1` |
                        // | `sculpt_eared` | `6` bordo · `χ = +1` · `7` órfãs | ⛔ `8` bordo · `11` órfãs |
                        // | `sphere_uv_96x144` | `0` bordo · `χ = +2` | = (inerte) |
                        //
                        // ⇒ **uma escolha GLOBAL conserta uma peça e parte duas**: a
                        // convenção não é uma constante, depende da transição. *A coluna
                        // «quantas acharia» conta candidatas; o `χ` conta as CERTAS, e as
                        // duas discordam.* ⛔ Fica em omissão o comportamento de sempre;
                        // o desempate tem de ser derivado (a parceira certa é a que, ao
                        // ser traçada de volta, regressa a esta porta) — e isso é obra.
                        let want_dir = if rescue_dir_raw() { d2 } else { opposite(d2) };
                        match ports.by_key.get(&(g, t2[0], t2[1], want_dir)) {
                            Some(&j) if j != id => {
                                st.orphan_rescued_across_edge += 1;
                                return Outcome::Linked(j, acc.then(x));
                            }
                            Some(_) => st.rescue_self += 1,
                            None => {
                                st.rescue_no_key += 1;
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

/// O sinal da área da imagem de uma face.
/// ⭐ **Sobre QUAL lado do triângulo o ponto cai** — `None` se está no interior.
///
/// ⚠️ O índice do lado é o do laço da travessia (`uv[k] → uv[(k+1) % 3]`), e é ele que
/// indexa [`Topo::twin`] e [`Topo::xf`]. *Uma convenção de lado escrita duas vezes é duas
/// convenções.*
fn on_edge_side(tri: [P; 3], t: P) -> Option<usize> {
    let [a, b, c] = tri;
    if crate::exact::orient(a, b, t) == 0 {
        Some(0)
    } else if crate::exact::orient(b, c, t) == 0 {
        Some(1)
    } else if crate::exact::orient(c, a, t) == 0 {
        Some(2)
    } else {
        None
    }
}

pub(crate) fn face_sign(topo: &Topo, f: usize) -> i8 {
    let [a, b, c] = topo.uv[f];
    orient(a, b, c)
}

/// O ponto está **dentro ou sobre** o triângulo-imagem?
fn contains(topo: &Topo, f: usize, q: P) -> bool {
    let [a, b, c] = topo.uv[f];
    let s = orient(a, b, c);
    if s == 0 {
        return false;
    }
    let e = [orient(a, b, q), orient(b, c, q), orient(c, a, q)];
    e.iter().all(|&x| x == 0 || x == s)
}

/// ⭐ **A ARESTA POR ONDE O SEGMENTO SAI** — e o desempate que apaga os casos
/// especiais.
fn exit_side(topo: &Topo, f: usize, entry: Option<usize>, o: P, t: P) -> Option<usize> {
    let mut best: Option<(usize, u8)> = None;
    for k in 0..3usize {
        if entry == Some(k) {
            continue;
        }
        let a = topo.uv[f][k];
        let b = topo.uv[f][(k + 1) % 3];
        if !crosses(o, t, a, b) {
            continue;
        }
        let n = u8::from(on_segment(o, t, a)) + u8::from(on_segment(o, t, b));
        if best.is_none_or(|(_, c)| n < c) {
            best = Some((k, n));
        }
    }
    best.map(|(k, _)| k)
}

/// Os dois segmentos tocam-se ou cruzam-se? Fechado nos extremos, de propósito: um
/// segmento que passa por um vértice **atravessa** as duas arestas que ali se
/// encontram, e é o desempate que decide qual delas serve.
fn crosses(o: P, t: P, a: P, b: P) -> bool {
    let (d1, d2) = (orient(o, t, a), orient(o, t, b));
    let (d3, d4) = (orient(a, b, o), orient(a, b, t));
    d1 * d2 <= 0 && d3 * d4 <= 0
}

/// O ponto está no segmento `[o, t]`, extremos incluídos?
fn on_segment(o: P, t: P, q: P) -> bool {
    orient(o, t, q) == 0
        && q[0] >= o[0].min(t[0])
        && q[0] <= o[0].max(t[0])
        && q[1] >= o[1].min(t[1])
        && q[1] <= o[1].max(t[1])
}

#[cfg(test)]
mod tests {
    use super::on_edge_side;

    /// ⭐⭐⭐ **A CONVENÇÃO DO LADO — `k` é a aresta do canto `k` para o `k+1`.**
    ///
    /// ⛔⛔ **É ela que indexa [`Topo::twin`] e [`Topo::xf`]**, e um índice trocado manda o
    /// resgate perguntar à face errada. ⚠️ *Uma convenção de lado escrita duas vezes é duas
    /// convenções, e nenhum tipo as separa: as três são `usize`.*
    #[test]
    fn the_side_index_is_the_corner_it_starts_from() {
        let tri = [[0, 0], [6, 0], [0, 6]];
        // O ponto médio de cada lado tem de dar o índice desse lado.
        assert_eq!(on_edge_side(tri, [3, 0]), Some(0), "lado 0 = canto 0 -> 1");
        assert_eq!(on_edge_side(tri, [3, 3]), Some(1), "lado 1 = canto 1 -> 2");
        assert_eq!(on_edge_side(tri, [0, 3]), Some(2), "lado 2 = canto 2 -> 0");
    }

    /// ⭐⭐ **O interior não é aresta nenhuma** — e sem esta metade a lei aceitaria tudo.
    #[test]
    fn a_point_inside_is_on_no_side() {
        let tri = [[0, 0], [6, 0], [0, 6]];
        assert_eq!(on_edge_side(tri, [1, 1]), None);
        assert_eq!(on_edge_side(tri, [2, 2]), None);
    }

    /// ⭐⭐⭐ **UM CANTO pertence a DOIS lados, e a resposta é o de índice MENOR.**
    ///
    /// ⚠️ Não é uma preferência: é o que torna a escolha **determinista**. *Um empate
    /// resolvido de outra maneira em cada chamada faria o resgate perguntar a uma face
    /// diferente a cada corrida* — e o hash da grade é contrato (HR-5).
    #[test]
    fn a_corner_belongs_to_the_lower_side() {
        let tri = [[0, 0], [6, 0], [0, 6]];
        assert_eq!(on_edge_side(tri, [0, 0]), Some(0), "canto 0: lados 0 e 2");
        assert_eq!(on_edge_side(tri, [6, 0]), Some(0), "canto 1: lados 0 e 1");
        assert_eq!(on_edge_side(tri, [0, 6]), Some(1), "canto 2: lados 1 e 2");
    }

    /// ⭐⭐ **Um ponto sobre o PROLONGAMENTO de um lado também é colinear** — e a função
    /// diz que sim, de propósito.
    ///
    /// ⚠️ **Ela é um predicado de COLINEARIDADE, não de pertença ao segmento**, e quem a
    /// chama já sabe que o ponto está *dentro* do triângulo ([`contains`] correu antes).
    /// *Dizer isto aqui é mais barato que alguém a reutilizar noutro sítio e descobrir.*
    #[test]
    fn the_predicate_is_collinearity_not_membership() {
        let tri = [[0, 0], [6, 0], [0, 6]];
        assert_eq!(on_edge_side(tri, [99, 0]), Some(0));
    }
}
