//! ⛔⛔⛔⛔ **A HIERARQUIA CONTRA UM NÍVEL — a wave *«pagar o cálculo por
//! níveis»*, medida e RECUSADA.**
//!
//! O dono ordenou-a em 20/09 e ela mede **pior em todas as colunas**, com o
//! controlo de um nível a bater ao dígito. Aqui vivem o **gate do mecanismo**
//! (que precisa de uma malha que só o TRAÇO produz) e as três sondas que
//! produziram a tabela; o controlo ao bit vive na crate da lei
//! (`uma_pilha_de_um_nivel_e_a_lei_de_hoje`), e o registo da recusa no
//! cabeçalho de [`ph2d_quadflow::regiao::campos_em_niveis`].
//!
//! ⚠️ Ele saiu do irmão por **TECTO DE LOC** (`707` contra `700`), e o corte é
//! por responsabilidade: ali fica a retícula que shipa, aqui a classe recusada.

use super::*;

/// ⭐⭐⭐⭐ **SONDA — A HIERARQUIA CONTRA UM NIVEL, nas QUATRO colunas.**
///
/// A ordem do dono foi *«pagar o cálculo por níveis»*, e a régua da FILEIRA
/// (escrita como passo zero dela) **corrigiu a pergunta**: a retícula de um
/// nível já produz fileiras (`p50 10`–`23`, máximo `63`–`74`) — o que a
/// hierarquia compraria é *mais fileiras COMPLETAS*, nunca as primeiras.
///
/// ⇒ a coluna que decide esta wave é a **`fil`**, e as outras estão aqui para
/// provar que ela não é comprada à custa delas (a `grade` é o alinhamento, o
/// `vinco` é o que a LUZ mostra, e `V` tem de ficar comparável — *duas malhas
/// de tamanhos diferentes não são duas leis, são duas peças*).
///
/// ⚠️ **O `mais_grosso` é a variável**: `0` é a lei que shipa (um nível), e
/// `24` é o [`ph2d_quadflow::hierarchy::COARSEST`] que a cadeia de retopologia
/// já usa. `8` está aqui para ver se descer abaixo dele muda alguma coisa — uma
/// pegada tem centenas de vértices, não milhões, e *uma pilha que pára cedo de
/// mais não tem nível grosso nenhum onde o acoplamento aconteça*.
///
/// ⛔ Ela **imprime**; quem decide é quem lê.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_a_hierarquia_contra_um_nivel -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela das duas classes, nao afirma nada"]
fn diag_a_hierarquia_contra_um_nivel() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("rumo   classe       grade%   vinco_p50  vinco_p90  fil50 fil90 filmax      V");
    // ⭐ **A ESCADA das varreduras, as duas classes lado a lado** — ela exclui a
    // saida barata: *se o nivel fino, com mais varreduras, re-adaptar a fase
    // localmente, a hierarquia nao comprou nada; se nao re-adaptar, ela custa*.
    // ⚠️⚠️ **A escada corre os QUATRO rumos e imprime a SOMA**, porque a coluna
    // da fileira SALTA entre degraus adjacentes num rumo so (`fil90` leu `63` na
    // ronda 3 e `34` na 4): o percurso e' guloso, e uma aresta a entrar ou sair
    // do balde alinhado funde ou parte duas cadeias longas de uma vez.
    // *Uma coluna de alta variancia lida numa amostra so fabrica uma tendencia.*
    println!("-- escada das varreduras, SOMA dos quatro rumos --");
    for varr in [2usize, 4, 8, 16, 32] {
        for (classe, n) in [("1 nivel ", 0usize), ("niveis24", 24)] {
            let (mut g, mut v50, mut v90, mut f50, mut f90, mut fmax) =
                (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
            // ⛔⛔ **A coluna das LASCAS faltava a esta escada, e é ela que o
            // portão da cena lê.** A primeira redacção mediu grade, vinco e
            // fileira, concluiu que `4` era melhor que `2`, e o gate
            // `a_cena_do_pente_tem_o_que_mostrar` reprovou com `2` triângulos
            // abaixo de `5°` — *uma escada sem a coluna da cerca que o produto
            // aplica recomenda um degrau que o produto recusa*.
            let (mut finas, mut pior) = (0usize, 180.0f64);
            for (_, e) in RUMOS.iter() {
                MAIS_GROSSO.with(|c| c.set(n));
                let (m, c) = traco_com_grelha(*e, raio, alvo, varr, LADO_DA_CELULA);
                MAIS_GROSSO.with(|c| c.set(0));
                let (bal, nb) = grade_da_faixa(&m, &c, raio);
                g += 100.0 * bal[0] as f64 / nb.max(1) as f64;
                let (a, b, _, _) = vinco_da_faixa(&m, &c, raio);
                v50 += a;
                v90 += b;
                let (x, y, z, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
                f50 += x;
                f90 += y;
                fmax = fmax.max(z);
                let (lf, _) = lascas(&m, &c, raio, LIMIAR_DA_LASCA);
                finas += lf;
                pior = pior.min(pior_angulo(&m, &c, raio).0);
            }
            println!(
                "varr{varr:<3} {classe}  lascas {finas:2} pior {pior:5.2}  grade {:6.2}  vinco {:6.3} {:6.3}  fil {:5.1} {:5.1} {fmax:4}",
                g / 4.0,
                v50 / 4.0,
                v90 / 4.0,
                f50 / 4.0,
                f90 / 4.0
            );
        }
    }
    println!("-- os quatro rumos --");
    for (nome, e) in RUMOS.iter().map(|(n, e)| (*n, *e)) {
        // ⭐⭐⭐ **O CONTROLO vem PRIMEIRO, e sem ele a tabela nao afirma nada:**
        // com `mais_grosso` acima do tamanho da pegada a pilha tem UM nivel so,
        // logo a `campos_em_niveis` tem de devolver a lei de hoje — *uma sonda
        // de duas classes sem o caso em que elas coincidem mede outro programa*.
        for (classe, n) in [
            ("1 nivel ", 0usize),
            ("pilha  1", 1_000_000),
            ("niveis24", 24),
            ("niveis 8", 8),
            ("niveis64", 64),
            ("niv. 128", 128),
        ] {
            MAIS_GROSSO.with(|c| c.set(n));
            let (m, c) = traco_com_grelha(e, raio, alvo, RONDAS_DA_GRELHA, LADO_DA_CELULA);
            MAIS_GROSSO.with(|c| c.set(0));
            println!(
                "{nome:<6} {classe}  {}",
                linha_da_grelha(&m, &c, raio, RONDAS_DA_GRELHA, LADO_DA_CELULA)
            );
        }
    }
}

/// ⛔⛔⛔ **SONDA — PORQUE a hierarquia mede pior: o retrato da pilha.**
///
/// Ela corre a [`ph2d_quadflow::regiao::retrato_da_pilha`] sobre a pegada REAL
/// do meio do traco, e separa as duas explicacoes: a franja a saturar os niveis
/// grossos, ou a FASE a chegar de longe (o pedido de deslocamento, em passos).
#[test]
#[ignore = "sonda: imprime o retrato da pilha, nao afirma nada"]
fn diag_o_retrato_da_pilha() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let e = RUMOS[0].1;
    // A malha a meio do traco, pela porta do produto.
    let (malha, centros) = traco_com_grelha(e, raio, alvo, RONDAS_DA_GRELHA, LADO_DA_CELULA);
    let centro = centros[centros.len() / 2];
    let mut faces = Vec::new();
    malha.octree().faces_in_sphere(centro, raio, &mut faces);
    let r2 = raio * raio;
    let mut ids: Vec<u32> = Vec::new();
    {
        let p = malha.positions();
        let todas = malha.faces();
        for &f in &faces {
            for &v in todas[f as usize].verts() {
                let q = p[v as usize];
                let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
                if d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])) <= r2 {
                    ids.push(v);
                }
            }
        }
    }
    let passo = ph2d_quadflow::regiao::passo_da_pegada(&malha, &ids) * LADO_DA_CELULA;
    let m = ph2d_quadflow::regiao::mancha(&malha, &ids);
    println!(
        "\npegada: {} vertices, {} miolo, {} franja ({:.0}%), passo {passo:.5}",
        m.len(),
        m.miolo(),
        m.len() - m.miolo(),
        100.0 * (m.len() - m.miolo()) as f64 / m.len().max(1) as f64
    );
    for (nome, n) in [
        ("1 nivel ", 1_000_000usize),
        ("niveis24", 24),
        ("niveis 8", 8),
    ] {
        let (niveis, p50, p90) = ph2d_quadflow::regiao::retrato_da_pilha(
            &m,
            [1.0, 0.0, 0.0],
            passo,
            RONDAS_DA_GRELHA,
            n,
        );
        let pilha: Vec<String> = niveis.iter().map(|(v, f)| format!("{v}/{f}")).collect();
        println!(
            "{nome}  pedido p50 {p50:6.3} p90 {p90:6.3} passos   pilha (V/fixos): {}",
            pilha.join("  ")
        );
    }
}

/// ⛔⛔⛔⛔ **A HIERARQUIA PROPÕE UMA RETÍCULA QUE NÃO É A QUE A MALHA TEM — o
/// mecanismo da recusa da wave *«pagar o cálculo por níveis»*, como gate.**
///
/// O dono ordenou a hierarquia (20/09) e ela mediu **pior em todas as colunas**,
/// com o controlo de um nível a bater ao dígito. Este gate é o *porquê*, e ele
/// tem de viver aqui porque a malha que o revela só o TRAÇO a produz.
///
/// A régua é o **pedido**: a distância de cada vértice ao nó de retícula que a
/// lei lhe aponta, em passos de célula. Sobre a malha a meio de um traço já
/// arrumado por um nível:
///
/// | classe | pedido `p50` | leitura |
/// |---|---|---|
/// | **um nível** | **`0,022`** | *«a retícula que propões é a que já tens»* |
/// | níveis (`24`) | `0,375` | *«propõe-te outra, a meia célula daqui»* |
///
/// ⚠️ **`0,375` é o máximo que duas retículas do mesmo passo podem discordar** —
/// meia célula é `0,5`, e a distância de um ponto qualquer ao nó mais perto de
/// uma grade rígida tem `p50 ≈ 0,40` (por simulação). ⇒ *a fase que a
/// hierarquia propõe não tem relação nenhuma com a que a malha tem.*
///
/// ⭐⭐⭐ **E é isso que parte as fileiras, porque um pincel aplica a lei DEZENAS
/// de vezes sobre uma pegada que anda.** O emparelhamento do `coarsen` é outro
/// a cada dab, logo cada dab propõe uma grade sem relação com a do anterior e
/// nada se acumula. Um nível semeia cada vértice consigo próprio, logo propõe
/// sempre a grade de que a malha está mais perto — e os dabs **reforçam-se**.
///
/// ⛔⛔ **DUAS explicações minhas caíram antes desta**, e ficam escritas para não
/// voltarem: a **saturação da franja** (refutada — `11 %` no nível `0` e `52 %`
/// no mais grosso, os níveis grossos resolvem) e a **convergência por repetição**
/// (refutada — a retícula sozinha sobre uma esfera de conectividade fixa não
/// converge em classe nenhuma, `0,400 → 0,367` em oito passagens: quem converge
/// é o PAR retícula + dyntopo).
#[test]
fn a_hierarquia_propoe_outra_reticula() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let (malha, centros) =
        traco_com_grelha(RUMOS[0].1, raio, alvo, RONDAS_DA_GRELHA, LADO_DA_CELULA);
    let centro = centros[centros.len() / 2];

    let mut faces = Vec::new();
    malha.octree().faces_in_sphere(centro, raio, &mut faces);
    let r2 = raio * raio;
    let mut ids: Vec<u32> = Vec::new();
    {
        let p = malha.positions();
        let todas = malha.faces();
        for &f in &faces {
            for &v in todas[f as usize].verts() {
                let q = p[v as usize];
                let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
                if d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])) <= r2 {
                    ids.push(v);
                }
            }
        }
    }
    let passo = ph2d_quadflow::regiao::passo_da_pegada(&malha, &ids) * LADO_DA_CELULA;
    let m = ph2d_quadflow::regiao::mancha(&malha, &ids);
    assert!(
        m.miolo() > 300,
        "a pegada do meio do traco precisa de miolo: {}",
        m.miolo()
    );
    let direccao = [1.0, 0.0, 0.0];

    let local =
        ph2d_quadflow::regiao::retrato_da_pilha(&m, direccao, passo, RONDAS_DA_GRELHA, m.len() + 1)
            .1;
    let (niveis, global, _) =
        ph2d_quadflow::regiao::retrato_da_pilha(&m, direccao, passo, RONDAS_DA_GRELHA, 24);

    // ⚠️ Sem varios niveis isto nao mede hierarquia nenhuma.
    assert!(niveis.len() >= 4, "a pilha e' rasa demais: {niveis:?}");
    // ⛔ A FRANJA nao satura — a explicacao candidata que caiu, aqui para nao voltar.
    let (v, f) = niveis[niveis.len() - 1];
    assert!(
        f * 2 < v * 3,
        "o nivel mais grosso esta' quase todo pregado ({f} de {v})"
    );
    assert!(
        local < 0.10,
        "um nivel devia reconhecer a propria reticula, e pede {local:.3} passos"
    );
    assert!(
        global > 0.25,
        "a hierarquia devia propor outra fase, e pede so' {global:.3} passos"
    );
}

/// ⛔ **SONDA — DESENHA a hierarquia contra um nível, com a grandeza REALÇADA.**
///
/// *Uma leitura a olho de um arame CHEIO não é uma medição* — foi assim que eu
/// li «retalhos» numa grade em 20/09 — e a recíproca também vale: **uma tabela
/// que decide uma wave olha-se antes de se acreditar nela.** Ela escreve os
/// dois lados com as arestas da fileira a preto e as outras a cinzento.
///
/// ```text
/// PH2D_PENTE_DUMP=/tmp cargo test -p ph2d-app-sculpt3d --release --lib diag_desenha_os_niveis -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: escreve .ppm para se olhar"]
fn diag_desenha_os_niveis() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    for (nome, n) in [("1nivel", 0usize), ("niveis", 24)] {
        MAIS_GROSSO.with(|c| c.set(n));
        let (m, c) = traco_com_grelha(RUMOS[0].1, raio, alvo, RONDAS_DA_GRELHA, LADO_DA_CELULA);
        MAIS_GROSSO.with(|c| c.set(0));
        let caminho = format!("{dir}/niveis_{nome}.ppm");
        crate::scenes::pente::tests::desenhos::desenha_fileiras(&m, &c, raio, &caminho);
        crate::scenes::pente::tests::desenhos::desenha_so_a_grade(
            &m,
            &c,
            raio,
            &format!("{dir}/niveis_grade_{nome}.ppm"),
        );
        let (f50, f90, fmax, fn_) =
            ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
        println!("{nome}: fileira p50 {f50} p90 {f90} max {fmax} ({fn_} cadeias) -> {caminho}");
    }
}
