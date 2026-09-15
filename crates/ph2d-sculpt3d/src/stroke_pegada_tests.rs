//! Os gates da **pegada congelada** a atravessar uma mudança de topologia.
//!
//! ⚠️ Irmão do `stroke_growth_tests.rs` pelo mesmo corte do módulo que eles
//! julgam: lá o `pre` de cada vértice, aqui o CONJUNTO de vértices do gesto.

use super::*;

/// ⛔⛔ **A TRADUÇÃO DE UM COLAPSO SEGUE UMA CADEIA, E NÃO UMA TABELA PLANA.**
///
/// Este gate existe porque a primeira redacção da
/// [`SculptStroke::encolhe_a_pegada_congelada`] supôs que as trocas do plano
/// eram independentes — que um índice era origem **ou** destino, nunca os dois.
/// **São os dois**, e o produto pagou-o com um `index out of bounds` no primeiro
/// dab a seguir a um colapso.
///
/// ⭐ **O oráculo é a aplicação SEQUENCIAL das trocas** — exactamente o que a
/// malha faz, e o mesmo que o `shrink_with` faz sobre os slots. A pergunta é
/// feita para **todos** os vértices de **todos** os planos, e não para uma
/// amostra.
///
/// # ⚠️⚠️ O corpus tem de conter as TRÊS espécies, e o gate exige-o
///
/// | espécie | como um vértice a encontra |
/// |---|---|
/// | **sobrescrito** | alguém se mudou para cima dele |
/// | **truncado** | ficou acima do novo tamanho e ninguém o foi buscar — ⛔ ele **não** aparece em troca nenhuma |
/// | **em cadeia** | mudou-se duas ou mais vezes, e os endereços do meio não são dele |
///
/// ⛔ **A segunda espécie foi achada por uma mutação SOBREVIVENTE:** apagar a
/// cerca do novo tamanho não era observável, porque o plano de três mortos
/// seguidos resolve todos os seus mortos por sobreposição. *Uma régua que não vê
/// o fenómeno acontecer não prova que ele não aconteceu* — e a terceira espécie
/// tem o mesmo problema ao contrário: um plano que não encadeia é respondido de
/// igual maneira pela tabela plana que estava errada.
#[test]
fn a_traducao_de_um_colapso_segue_a_cadeia_e_nao_uma_tabela_plana() {
    /// `(quem morre, de quantos)` — ver a tabela das três espécies no doc.
    const PLANOS: [(&[u32], usize); 4] = [
        // Três seguidos no fim: `(11→10)`, `(10→9)`, `(9→8)` ⇒ o vértice que
        // começou em `11` acaba em `8`. É a CADEIA.
        (&[8, 9, 10], 12),
        // O topo morre sozinho: o plano fica VAZIO, e o vértice `11` some sem
        // aparecer em troca nenhuma. É a TRUNCAGEM.
        (&[11], 12),
        // Um morto no meio: uma troca só, sem cadeia.
        (&[5], 12),
        // Os dois extremos: truncagem e sobreposição no mesmo plano.
        (&[0, 11], 12),
    ];

    let (mut sobrescritos_vistos, mut truncados_vistos, mut cadeias_vistas) = (0, 0, 0);
    for (mortos_verts, len) in PLANOS {
        let remap = ph2d_mesh::Remap::plan(&[], 0, mortos_verts, len);

        // ⭐ O ORÁCULO: as trocas POR ORDEM. `ident[i]` passa a dizer que vértice
        // ORIGINAL vive hoje em `i`.
        let mut ident: Vec<u32> = (0..u32::try_from(len).expect("cabe")).collect();
        for &(de, para) in &remap.vert_moves {
            ident[para as usize] = ident[de as usize];
        }
        ident.truncate(remap.verts);

        let mut de_para = remap.vert_moves.clone();
        de_para.sort_unstable_by_key(|&(de, _)| de);
        let mut sobrescritos: Vec<u32> = remap.vert_moves.iter().map(|&(_, para)| para).collect();
        sobrescritos.sort_unstable();
        let verts = u32::try_from(remap.verts).expect("cabe");

        for v in 0..u32::try_from(len).expect("cabe") {
            let esperado = ident
                .iter()
                .position(|&original| original == v)
                .map(|i| u32::try_from(i).expect("cabe"));
            assert_eq!(
                pegada::onde_parou(v, &de_para, &sobrescritos, verts),
                esperado,
                "o vértice {v} do plano {mortos_verts:?}/{len}: a cadeia \
                 {:?} diz outra coisa",
                remap.vert_moves
            );
            // O censo das espécies, para o piso lá em baixo.
            if sobrescritos.binary_search(&v).is_ok() {
                sobrescritos_vistos += 1;
            } else if esperado.is_none() {
                truncados_vistos += 1;
            }
            let mut saltos = 0;
            let mut actual = v;
            while let Ok(i) = de_para.binary_search_by_key(&actual, |&(de, _)| de) {
                actual = de_para[i].1;
                saltos += 1;
            }
            if saltos >= 2 {
                cadeias_vistas += 1;
            }
        }
    }

    assert!(
        sobrescritos_vistos > 0,
        "nenhum vértice morreu SOBRESCRITO no corpus"
    );
    assert!(
        truncados_vistos > 0,
        "nenhum vértice morreu TRUNCADO no corpus — e sem essa espécie a cerca \
         do novo tamanho não é observável (uma mutação sobreviveu por isto)"
    );
    assert!(
        cadeias_vistas > 0,
        "nenhum vértice VIAJOU EM CADEIA no corpus — e sem ela a tabela plana \
         que estava errada responde igual"
    );
}

/// ⭐⭐⭐ **A PEGADA CONGELADA DO POLEGAR ATRAVESSA O REFINO E O COLAPSO** — a
/// ordem do dono (*«Thumb se for possível, deveria subdividir»*) medida de
/// ponta a ponta na crate, com a malha a mexer-se **nos dois sentidos**.
///
/// As três metades, e nenhuma basta sozinha:
///
/// 1. **Nenhum índice fica fora da malha.** É a que o produto pagou: um índice
///    obsoleto é lido no dab seguinte e estoura — ou, pior, aponta para barro
///    noutro sítio da peça, em silêncio.
/// 2. **Quem sobreviveu continua a nomear O MESMO VÉRTICE.** Sem ela, largar a
///    pegada inteira a cada colapso passaria na primeira.
/// 3. **Ela ACOLHEU quem nasceu lá dentro.** Sem ela, uma pegada que só encolhe
///    passaria nas duas de cima e deixaria os vértices novos parados entre
///    vizinhos deslocados.
///
/// ⚠️ **Os pisos de população são três**, um por fenómeno: a pegada tem de
/// existir, o refino tem de ter dado à luz, e o colapso tem de ter mexido — *um
/// gate sobre uma malha que não muda de tamanho afirma o nada*.
#[test]
fn a_pegada_congelada_do_polegar_atravessa_a_topologia() {
    let mut m = ph2d_mesh::shapes::uv_sphere(14, 20, 1.0);
    m.triangulate();
    let mut s = SculptStroke::default();
    s.begin(&m);

    // O polegar é o ÚNICO verbo que congela a pegada — ver `stroke_dab_core`.
    let brush = Brush {
        verb: Verb::Thumb,
        ..Brush::default()
    };
    let centro = [0.0, 0.0, 1.0];
    s.dab(
        &mut m,
        &brush,
        &Dab::pulling(centro, 0.9, [0.0, 0.0, -1.0], [0.05, 0.0, 0.0]),
        Symmetry::default(),
    );
    assert_eq!(
        s.pegada_ancorada.len(),
        1,
        "o controlo: sem simetria o polegar congela EXACTAMENTE uma pegada"
    );
    assert!(
        s.pegada_ancorada[0].verts.len() >= 10,
        "o controlo: a pegada tem de conter o fenómeno, e tem {}",
        s.pegada_ancorada[0].verts.len()
    );

    // A identidade de cada vértice, na numeração de HOJE. Os recém-nascidos
    // entram com uma identidade própria, que é o que permite distingui-los.
    let mut ident: Vec<u32> = (0..u32::try_from(m.vert_count()).expect("cabe")).collect();
    let antes: Vec<u32> = s.pegada_ancorada[0]
        .verts
        .iter()
        .map(|&v| ident[v as usize])
        .collect();

    // ── O REFINO ──────────────────────────────────────────────────────────────
    let fino = 0.5 * mean_edge_pegada(&m);
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    let r = ph2d_mesh::refine_in_sphere(&mut m, centro, 0.9, fino, &mut births, &mut region);
    assert!(
        matches!(r, ph2d_mesh::Refine::Done { .. }),
        "o controlo: a fixtura tem de refinar ({r:?})"
    );
    assert!(!births.is_empty(), "o controlo: alguém tem de nascer");
    let nascidos: Vec<u32> = births.iter().map(|b| b.vert).collect();
    ident.resize(m.vert_count(), 0);
    for (i, &v) in nascidos.iter().enumerate() {
        // Uma identidade que não colide com nenhuma das antigas.
        ident[v as usize] = u32::MAX - u32::try_from(i).expect("cabe");
    }
    // A pegada ANTES do refino, em índices — e eles não se mexem, porque o
    // refino APENDA.
    let antes_do_refino: Vec<u32> = s.pegada_ancorada[0].verts.clone();
    s.grow_with(&m, &births);

    // ── A LEI DO CRESCIMENTO, com oráculo INDEPENDENTE ───────────────────────
    // ⭐ Reconstruída à mão pela regra escrita na lei: entra quem nasce entre
    // DOIS membros, e um recém-nascido acolhido passa a ser membro para os que
    // nascerem depois dele no mesmo passe.
    let mut esperado: Vec<u32> = antes_do_refino.clone();
    esperado.sort_unstable();
    // ⚠️⚠️ **O piso que torna esta metade observável:** sem um nascimento com UM
    // pai dentro e outro fora, a regra «os dois» e a regra «um basta» dão a
    // MESMA pegada, e este gate ficaria verde sobre as duas.
    let mut meio_dentro = 0usize;
    for b in &births {
        let (pa, pb) = (
            esperado.binary_search(&b.a).is_ok(),
            esperado.binary_search(&b.b).is_ok(),
        );
        if pa != pb {
            meio_dentro += 1;
        }
        if pa && pb {
            // Os nascimentos chegam por ordem e o índice é sempre maior.
            esperado.push(b.vert);
        }
    }
    assert!(
        meio_dentro > 0,
        "a fixtura não contém o fenómeno: nenhum vértice nasceu a cavalo na \
         fronteira da pegada, logo «os dois pais» e «um pai basta» são \
         indistinguíveis aqui"
    );
    assert_eq!(
        s.pegada_ancorada[0].ordenada, esperado,
        "a pegada acolheu outro conjunto que não «quem nasceu entre DOIS \
         membros» ({meio_dentro} nasceram a cavalo na fronteira)"
    );

    // ── O COLAPSO ─────────────────────────────────────────────────────────────
    let grosso = 1.2 * mean_edge_pegada(&m);
    let mut remap = ph2d_mesh::Remap::default();
    let c = ph2d_mesh::collapse_in_sphere(&mut m, centro, 0.9, grosso, &mut remap, &mut region);
    assert!(
        matches!(c, ph2d_mesh::Collapse::Done { .. }),
        "o controlo: a fixtura tem de colapsar ({c:?})"
    );
    assert!(
        !remap.vert_moves.is_empty(),
        "o controlo: o colapso tem de ter MEXIDO alguém, senão a renumeração \
         não é exercitada"
    );
    for &(de, para) in &remap.vert_moves {
        ident[para as usize] = ident[de as usize];
    }
    ident.truncate(remap.verts);
    s.shrink_with(&remap);

    // ── AS TRÊS METADES ───────────────────────────────────────────────────────
    let p = &s.pegada_ancorada[0];
    for &v in &p.verts {
        assert!(
            (v as usize) < m.vert_count(),
            "a pegada guarda o índice {v} e a malha tem {} vértices — é este o \
             índice obsoleto que estoura no dab seguinte",
            m.vert_count()
        );
    }
    assert_eq!(
        p.verts.len(),
        p.ordenada.len(),
        "as duas ordens descrevem o MESMO conjunto"
    );
    let mut copia = p.verts.clone();
    copia.sort_unstable();
    assert_eq!(copia, p.ordenada, "o índice deixou de descrever a pegada");

    let hoje: Vec<u32> = p.verts.iter().map(|&v| ident[v as usize]).collect();
    // (2) Quem sobreviveu continua a nomear o mesmo vértice — e há quem tenha.
    let sobreviventes: Vec<u32> = antes.iter().copied().filter(|a| hoje.contains(a)).collect();
    assert!(
        sobreviventes.len() >= antes.len() / 2,
        "a pegada perdeu mais de metade num colapso local: {} de {}",
        sobreviventes.len(),
        antes.len()
    );
    // ⛔ E nenhum ESTRANHO entrou: tudo o que está lá ou já lá estava, ou nasceu.
    let bebes: Vec<u32> = nascidos
        .iter()
        .enumerate()
        .map(|(i, _)| u32::MAX - u32::try_from(i).expect("cabe"))
        .collect();
    for h in &hoje {
        assert!(
            antes.contains(h) || bebes.contains(h),
            "um vértice que nunca esteve na pegada apareceu nela"
        );
    }
    // (3) Ela ACOLHEU quem nasceu entre dois membros.
    let acolhidos = hoje.iter().filter(|h| bebes.contains(h)).count();
    assert!(
        acolhidos > 0,
        "a pegada não acolheu nenhum dos {} nascimentos — os vértices novos \
         ficariam parados entre vizinhos deslocados, que é a cratera de agulhas",
        births.len()
    );

    // E o dab seguinte não estoura — que é o modo de falha que o produto viu.
    let _ = s.dab(
        &mut m,
        &brush,
        &Dab::pulling(centro, 0.9, [0.0, 0.0, -1.0], [0.08, 0.0, 0.0]),
        Symmetry::default(),
    );
}

fn mean_edge_pegada(m: &ph2d_mesh::Mesh) -> f32 {
    let pos = m.positions();
    let (mut soma, mut n) = (0.0f32, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            soma += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
    }
    soma / n.max(1) as f32
}
