//! ⭐⭐⭐ **OS GATES SOBRE OS MAPAS DE REFERÊNCIA** — as leis que a extracção afirma,
//! medidas nas duas peças verificadas de `docs/3D/cleanroom/fixtures/`.
//!
//! ⚠️ **Uma corrida, muitas afirmações.** A extracção de cada peça custa dezenas de
//! milissegundos e é o mesmo caminho para todos os gates; correr uma por gate seria
//! pagar a mesma conta dez vezes para medir dez faces do mesmo resultado.
//!
//! ⚠️ **As duas peças têm COSTURAS de propósito** (`247` e `138` arestas de rotação
//! não-nula). *Uma peça sem costura aprovaria uma extracção que ignorasse transições
//! por completo* — e a primeira medição desta linha saiu sobre um mapa assim.

mod support;

use ph2d_mesh::Mesh;
use ph2d_quadextract::mapa::Mapa;
use ph2d_quadextract::{ExtractReport, euler_characteristic, extract};

/// `V − E + F` da malha de ENTRADA, lida do próprio fixture.
fn chi_of_input(m: &Mapa) -> i64 {
    use std::collections::BTreeSet;
    let mut e: BTreeSet<(u32, u32)> = BTreeSet::new();
    for t in &m.tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            e.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    let mut used: BTreeSet<u32> = BTreeSet::new();
    for t in &m.tris {
        used.extend(t.iter().copied());
    }
    i64::try_from(used.len()).unwrap() - i64::try_from(e.len()).unwrap()
        + i64::try_from(m.tris.len()).unwrap()
}

/// Arestas da saída usadas **uma** vez (bordo) e **três ou mais** (não-manifold).
fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (
        n.values().filter(|c| **c == 1).count(),
        n.values().filter(|c| **c >= 3).count(),
    )
}

/// As leis que valem em **toda** peça fechada, aplicadas de uma vez.
fn assert_closed_piece(name: &str, m: &Mapa) -> (Mesh, ExtractReport) {
    let (mesh, r) = extract(&m.as_map(), None).unwrap_or_else(|e| panic!("{name}: {e}"));

    // ── GATE 4: toda translação de transição é INTEIRA.
    //
    // ⛔ A extracção **assume-o**. Um mapa cuja costura não seja inteira tem as duas
    // grades desalinhadas, e o saneamento apenas arredonda o erro para dentro.
    assert!(
        r.shift_residual < 1.0e-9,
        "{name}: o mapa nao e' de grade inteira — residuo de translacao {:.3e} celulas",
        r.shift_residual
    );
    assert!(
        r.rot_residual < 1.0e-6,
        "{name}: residuo de rotacao {:.3e} quartos de volta",
        r.rot_residual
    );

    // ── GATE 2: o leque de um vértice regular FECHA, e as transições relêem-se
    // exactamente dos valores saneados.
    //
    // ⚠️ **É esta a forma executável da lei da precisão.** Propagar a imagem a
    // partir de um canto só dá o mesmo valor por qualquer caminho **exactamente**
    // quando a holonomia é a identidade; e reler cada transição dos valores saneados,
    // ao bit, para os dois vértices da aresta ao mesmo tempo, é o que separa um mapa
    // saneado de um mapa quase saneado.
    assert_eq!(
        r.holonomy_broken, 0,
        "{name}: {} leques regulares com holonomia != identidade",
        r.holonomy_broken
    );
    assert_eq!(
        r.inexact_transitions, 0,
        "{name}: {} transicoes nao se releram exactamente depois do saneamento",
        r.inexact_transitions
    );

    // ── GATE 3: numa singularidade a imagem saneada é o PONTO FIXO da holonomia.
    //
    // ⚠️ A afirmação algébrica está em `o_ponto_fixo_e_mesmo_fixo`; o que se cobra
    // aqui é que a peça **contém** o fenómeno — senão o gate mede o vazio.
    assert!(
        r.pinned_fixed > 0,
        "{name}: nenhuma singularidade foi pregada — a peca nao contem o fenomeno"
    );

    // ── A ORDEM DAS SAÍDAS, que é a propriedade load-bearing.
    assert_eq!(
        &r.port_step[..3],
        &[0, 0, 0],
        "{name}: a lista de saidas tem de ser HORARIA — todo passo entre cartas \
         nao-dobradas desce um quarto de volta, e o histograma diz {:?}",
        r.port_step
    );
    assert!(
        r.port_step[3] > 1000,
        "{name}: o balde bom tem de estar CHEIO, e tem {}",
        r.port_step[3]
    );

    // ── §6.4: nenhum leque colapsou abaixo de meia volta sem uma dobra por perto.
    assert_eq!(r.collapsed_fans, 0, "{name}: leques colapsados");

    // ── GATE 6: TODA face da saída é um quad. É o teorema da fusão.
    assert!(
        mesh.faces().iter().all(|f| !f.is_tri()),
        "{name}: saiu um triangulo — a fusao esta' incompleta"
    );
    assert_eq!(r.triangles, 0, "{name}: celulas com tres cantos distintos");
    assert_eq!(
        r.ring_distinct[3], 0,
        "{name}: um triangulo exigiria uma ligacao DIAGONAL no quadrado unitario, \
         e a fusao nao cria ligacoes novas"
    );
    assert!(r.quads > 0, "{name}: saida vazia");
    assert_eq!(mesh.face_count(), r.quads, "{name}: contagem de faces");

    // ── A saída é uma superfície fechada e manifold.
    let (boundary, non_manifold) = edge_census(&mesh);
    assert_eq!(
        boundary, 0,
        "{name}: a saida tem {boundary} arestas de bordo"
    );
    assert_eq!(
        non_manifold, 0,
        "{name}: a saida tem {non_manifold} arestas nao-manifold"
    );

    // ── GATE 7: a característica de Euler da saída IGUALA a da entrada.
    //
    // ⚠️ **É a régua que apanha a asa perdida**, e esta linha ja' pagou por ela: um
    // toro que sai com `χ = 2` passa em todas as outras — 100 % quads, zero bordo,
    // zero nao-manifold — e perdeu a alca.
    assert_eq!(
        euler_characteristic(&mesh),
        chi_of_input(m),
        "{name}: a caracteristica de Euler mudou"
    );
    (mesh, r)
}

#[test]
fn o_gancho_organico_extrai_e_fecha() {
    let m = support::hooked();
    let (mesh, r) = assert_closed_piece("gancho", &m);
    assert_eq!(
        chi_of_input(&m),
        2,
        "o gancho e' uma casca fechada de genero 0"
    );
    // ⭐ **A peça CONTÉM dobras**, e é isso que a torna a fixtura certa para a tese
    // do método: aceitar a dobra e extrair à mesma.
    assert!(r.folded_faces >= 10, "dobras: {}", r.folded_faces);
    assert!(
        r.merged_groups > 0,
        "com dobras, a fusao TEM de colapsar grupos — senao ela nao esta' a correr"
    );
    assert!(mesh.vert_count() > 1000);
}

#[test]
fn o_toro_extrai_com_a_alca_intacta() {
    // ── GATE 10: uma malha de género 1 produz saída com `χ = 0`.
    let m = support::torus();
    let (mesh, _) = assert_closed_piece("toro", &m);
    assert_eq!(chi_of_input(&m), 0, "o fixture e' um toro");
    assert_eq!(
        euler_characteristic(&mesh),
        0,
        "a alca do toro tem de sobreviver"
    );
    assert!(mesh.vert_count() > 2000);
}

#[test]
fn uma_malha_com_bordo_produz_saida() {
    // ── GATE 9: e ⛔ **ele NÃO TEM ORÁCULO** — a integração de referência cai com
    // falha de segmentação em malha com bordo, medido em duas peças. A extracção tem
    // de resolver o bordo **sem gabarito**, e a lei é a do §4: a condição de
    // aceitação de uma isolinha colinear relaxa quando o triângulo não tem vizinho
    // do lado de fora do leque.
    let base = support::hooked();
    let m = support::with_hole(&base, 1000, 40);
    let (mesh, r) = extract(&m.as_map(), None).expect("a peca com bordo foi recusada");
    assert!(
        r.boundary_edges >= 20,
        "a fixtura tem de CONTER bordo, e tem {} arestas",
        r.boundary_edges
    );
    assert!(r.open_fans >= 20, "e leques abertos, e tem {}", r.open_fans);
    assert!(r.quads > 1500, "so' saiu(ram) {} quads", r.quads);
    assert!(
        mesh.faces().iter().all(|f| !f.is_tri()),
        "saiu um triangulo de uma peca com bordo"
    );
    // ⭐ Uma esfera com um disco removido tem `χ = 1`, e é o que a saída tem de dar.
    assert_eq!(
        euler_characteristic(&mesh),
        1,
        "uma casca de genero 0 com UM buraco tem caracteristica 1"
    );
    let (boundary, non_manifold) = edge_census(&mesh);
    assert!(boundary > 0, "a saida perdeu o bordo que a entrada tinha");
    assert_eq!(non_manifold, 0, "a saida ficou nao-manifold no bordo");
    // ⚠️ Saídas que morrem no bordo são **esperadas**; o que não pode haver é órfãs.
    assert!(r.pending_boundary > 0, "nenhuma saida morreu no bordo");
    assert_eq!(r.orphan, 0, "{} saidas orfas", r.orphan);
}

#[test]
fn nenhum_traco_foge_nem_fica_orfao() {
    for (name, m) in [("gancho", support::hooked()), ("toro", support::torus())] {
        let (_, r) = extract(&m.as_map(), None).unwrap();
        assert_eq!(r.runaway, 0, "{name}: {} tracos fugidos", r.runaway);
        assert_eq!(r.orphan, 0, "{name}: {} saidas orfas", r.orphan);
        // ⭐⭐⭐ **E O RESGATE PELA FACE GÉMEA NÃO CORRE AQUI — dizê-lo é a metade honesta.**
        //
        // ⛔⛔ Ele foi construído em 2026-08-26 e **nenhuma fixtura deste repositório o
        // alcança**: estas duas não têm órfã nenhuma, e o resgate só existe dentro do ramo
        // «sem parceira». *Medido, não suposto* — a linha acima é a prova.
        //
        // ⚠️ O caso REAL vive fora da árvore (`sculpt_t001` do corpus): lá ele dispara **uma
        // vez** e leva a peça de `4` arestas de bordo a **`0`**, com `χ` de `1` a **`2`**.
        // ⇒ Esta asserção pina a **INÉRCIA**: se um dia ela ficar vermelha, ou apareceu uma
        // órfã nova nestas peças, ou o resgate passou a correr onde não devia.
        assert_eq!(
            r.orphan_rescued_across_edge, 0,
            "{name}: o resgate correu {} vezes numa peca sem orfas",
            r.orphan_rescued_across_edge
        );
        assert_eq!(
            r.orphan_rescued_in_fan, 0,
            "{name}: o resgate pelo LEQUE correu {} vezes numa peca sem orfas",
            r.orphan_rescued_in_fan
        );
        assert_eq!(
            r.pending_boundary, 0,
            "{name}: uma peca FECHADA nao pode ter saida a morrer no bordo"
        );
        assert_eq!(r.linked * 2, r.ports, "{name}: saidas por emparelhar");
        assert_eq!(r.non_manifold_edges, 0, "{name}: entrada nao-manifold");
        // ⚠️ **O custo do traço, com a contagem ao lado.** O alvo está a UMA célula
        // e um triângulo mede da ordem de uma célula, então um traço são gasta
        // unidades de passos. Uma média que dispare diz que o mapa deixou de ser
        // local muito antes de qualquer gate de forma o notar.
        #[allow(clippy::cast_precision_loss)]
        let mean = r.walk_steps as f64 / r.ports.max(1) as f64;
        assert!(
            mean < 6.0,
            "{name}: o traco medio gastou {mean:.2} passos, e um mapa local gasta unidades"
        );
    }
}
