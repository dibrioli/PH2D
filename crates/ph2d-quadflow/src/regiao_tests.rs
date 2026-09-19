//! **OS GATES DA MANCHA** — o domínio local dos dois campos.
//!
//! ⚠️ **O oráculo NÃO é *«o campo parece bom»*.** A propriedade central é uma
//! **igualdade ao bit**: correr a lei na pegada e correr a lei na peça inteira
//! com a mesma fronteira pregada têm de dar **o mesmo número** em cada vértice
//! de miolo. Se isso for verdade, tudo o que a cadeia de retopologia já provou
//! sobre a lei vale aqui sem se medir outra vez; se for falso, a mancha é um
//! segundo motor disfarçado.
//!
//! ```text
//! cargo test -p ph2d-quadflow regiao
//! ```

use ph2d_mesh::{Mesh, QueryScratch, shapes};

use super::{mancha, orientacao_semeada};

/// A peça das medições: curvatura real nas duas direcções, e densidade
/// suficiente para uma pegada ter miolo.
fn peca() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

fn pegada(mesh: &Mesh, centro: [f32; 3], raio: f32) -> Vec<u32> {
    let mut scratch = QueryScratch::default();
    let mut out = Vec::new();
    mesh.verts_in_sphere(centro, raio, &mut scratch, &mut out);
    out
}

/// ⚠️⚠️ **A peça das DUAS últimas: a [`peca`] não contém o fenómeno.**
///
/// Com ela e um raio de `0,45` a mancha tem **`27`** vértices de miolo, e a
/// pegada que o produto de facto forma tem **`790`** — *numa mancha minúscula a
/// fase local também não assenta*, e a primeira redacção destes gates reprovou
/// sobre produto correcto por isso. A regra é a que esta linha já pagou cinco
/// vezes: **a fixtura tem de ser do tamanho do que o produto vê.**
fn peca_densa() -> Mesh {
    shapes::uv_sphere(64, 96, 1.0)
}

const TRACO: [f32; 3] = [0.371, 0.642, -0.183];
const ITERACOES: usize = 6;

/// ⭐⭐⭐ **O MIOLO DE UMA MANCHA É A PEÇA INTEIRA, AO BIT.**
///
/// A metade que torna este módulo um domínio e não um motor. Correm-se as duas:
///
/// - na **mancha**, com a franja pregada;
/// - na **peça inteira**, com **tudo menos o miolo daquela mancha** pregado.
///
/// Os dois percorrem o miolo na mesma ordem relativa (a [`Mancha::ids`] é
/// crescente), lêem os mesmos pesos (um vértice de miolo tem o anel inteiro
/// dentro) e partem da mesma semente. ⇒ **igualdade exacta**, sem epsilon.
///
/// ⚠️ **O controlo POSITIVO está dentro**: a mancha tem de ter miolo, senão a
/// asserção percorre o conjunto vazio e fica verde sobre nada.
#[test]
fn o_miolo_de_uma_mancha_e_a_peca_inteira() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    assert!(
        m.miolo() >= 20,
        "a pegada tem de ter miolo para a asserção medir alguma coisa: {}",
        m.miolo()
    );

    let local = orientacao_semeada(&m, TRACO, ITERACOES);

    // A peça inteira, semeada igual e com tudo fora do miolo pregado.
    let normais = mesh.normals();
    let mut global: Vec<[f32; 3]> = normais
        .iter()
        .map(|&n| crate::orientation::project_tangent(TRACO, n))
        .collect();
    let mut fixos = vec![true; mesh.vert_count()];
    for (i, &v) in m.ids.iter().enumerate() {
        if !m.fronteira[i] {
            fixos[v as usize] = false;
        }
    }
    let adj = crate::im_weights::cotangent_adjacency(&mesh);
    crate::orientation::smooth_on_fixed(&mut global, normais, &adj, &fixos, ITERACOES);

    let mut conferidos = 0usize;
    for ((i, &v), &d) in m.ids.iter().enumerate().zip(&local) {
        if m.fronteira[i] {
            continue;
        }
        assert_eq!(
            d, global[v as usize],
            "o vértice de miolo {v} discorda entre a mancha e a peça"
        );
        conferidos += 1;
    }
    assert_eq!(conferidos, m.miolo());
}

/// ⚠️ **A franja não se move** — ela é condição de fronteira, e a promessa do
/// pincel (*fora da pegada, nem um bit*) começa aqui.
#[test]
fn a_franja_fica_onde_a_semente_a_pos() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    let semente: Vec<[f32; 3]> = m
        .nrm
        .iter()
        .map(|&n| crate::orientation::project_tangent(TRACO, n))
        .collect();
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);

    let mut presos = 0usize;
    for (i, (&d, &s)) in dirs.iter().zip(&semente).enumerate() {
        if !m.fronteira[i] {
            continue;
        }
        assert_eq!(d, s, "a franja {i} moveu-se");
        presos += 1;
    }
    assert!(presos > 0, "a pegada tem de ter franja");
}

/// ⭐ **O CAMPO SEGUE O TRAÇO** — na chapa, onde a resposta se sabe de cabeça.
///
/// Sobre um plano toda normal é a mesma, a projecção do traço é a mesma em todo
/// vértice, e a suavização é um **ponto fixo**: o campo é o traço, exactamente.
/// ⚠️ É a régua que separa *«a lei corre»* de *«a lei corre e faz o que diz»*.
#[test]
fn na_chapa_o_campo_e_o_traco() {
    let mesh = chapa(12);
    let ids: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);
    assert!(m.miolo() >= 40, "miolo: {}", m.miolo());

    let n = m.nrm[0];
    let alvo = crate::orientation::project_tangent(TRACO, n);
    for (i, &d) in dirs.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        // 4-RoSy: a direcção vale a menos de um quarto de volta.
        let c = dot(d, alvo).abs().max(dot(d, cross(n, alvo)).abs());
        assert!(
            c > 0.999_9,
            "o campo desviou do traço no vértice {i}: cos = {c}"
        );
    }
}

/// ⛔ **Sem direcção não há lei** — e o campo sai VAZIO em vez de um eixo
/// inventado. A degenerescência que o pente já declara.
#[test]
fn uma_direccao_nula_nao_inventa_eixo() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    assert!(orientacao_semeada(&m, [0.0, 0.0, 0.0], ITERACOES).is_empty());
}

/// ⚠️ **Os pesos de uma mancha que cobre tudo são os da peça** — a prova de que
/// a porta nova não é uma segunda lei.
#[test]
fn os_pesos_de_uma_mancha_que_cobre_tudo_sao_os_da_peca() {
    let mesh = peca();
    let todas: Vec<u32> = (0..mesh.face_count() as u32).collect();
    let pesos = crate::im_weights::cotangent_edge_weights_on(&mesh, &todas);

    let mut adj: Vec<Vec<crate::im_weights::Link>> = vec![Vec::new(); mesh.vert_count()];
    for ((a, b), w) in pesos {
        adj[a as usize].push(crate::im_weights::Link { id: b, weight: w });
        adj[b as usize].push(crate::im_weights::Link { id: a, weight: w });
    }
    for list in &mut adj {
        list.sort_by_key(|l| l.id);
    }

    assert_eq!(adj, crate::im_weights::cotangent_adjacency(&mesh));
}

/// ⚠️ **Uma mancha pode não ter miolo, e isso é um FACTO, não uma falha** — uma
/// pegada mais fina que uma aresta é toda franja. Quem consome tem de conseguir
/// distinguir *«não fez nada»* de *«não havia o que fazer»*.
#[test]
fn uma_pegada_fina_demais_e_toda_franja() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.05);
    let m = mancha(&mesh, &ids);
    assert!(
        !m.is_empty(),
        "a pegada não pode ser vazia: seria outro caso"
    );
    assert_eq!(m.miolo(), 0, "miolo: {}", m.miolo());
}

/// Uma chapa `n × n` no plano `z = 0`, triangulada por leque de quadrado.
fn chapa(n: usize) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let x = i as f32 / (n - 1) as f32 - 0.5;
            let y = j as f32 / (n - 1) as f32 - 0.5;
            pos.push([x, y, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let b = a + 1;
            let c = a + n as u32;
            let d = c + 1;
            faces.push(ph2d_mesh::Face::tri(a, b, d));
            faces.push(ph2d_mesh::Face::tri(a, d, c));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa é bem formada")
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// ⭐⭐⭐ **O ALVO DA RETÍCULA É TANGENTE E LIMITADO** — as duas propriedades que
/// tornam esta lei utilizável por um pincel.
///
/// Tangente: a malha **desliza**, não incha nem encolhe. Limitada: o alvo é o
/// ponto da grelha **mais perto** do vértice, logo nunca está a mais de meia
/// diagonal — *um pincel não pode atirar barro para o outro lado da peça*.
#[test]
fn o_alvo_da_reticula_e_tangente_e_limitado() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);
    let passo = 0.08;
    let alvos = super::posicao_da_mancha(&m, &dirs, passo, ITERACOES);
    assert_eq!(alvos.len(), m.len());

    let (mut pior_viagem, mut pior_normal) = (0.0f32, 0.0f32);
    for (i, &a) in alvos.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        let d = [a[0] - m.pos[i][0], a[1] - m.pos[i][1], a[2] - m.pos[i][2]];
        pior_viagem = pior_viagem.max(dot(d, d).sqrt() / passo);
        pior_normal = pior_normal.max(dot(d, m.nrm[i]).abs() / passo);
    }
    // Meia diagonal de uma célula quadrada, mais a folga de `f32`.
    assert!(
        pior_viagem <= 0.7072,
        "o alvo saiu da célula: {pior_viagem} passos"
    );
    assert!(
        pior_normal <= 1.0e-5,
        "o alvo saiu do plano tangente: {pior_normal} passos"
    );
}

/// ⭐⭐ **NUMA GRELHA JÁ CERTA A RETÍCULA NÃO MOVE NADA** — o ponto fixo.
///
/// Uma chapa regular de passo `h`, com o traço ao longo de um eixo dela, **já é**
/// a saída desta lei. ⚠️ É a metade que separa *«ela arruma»* de *«ela mexe»*:
/// sem este gate, uma lei que empurrasse tudo meia célula passaria nas outras.
#[test]
fn numa_grelha_ja_certa_a_reticula_nao_move_nada() {
    let n = 12;
    let mesh = chapa(n);
    let h = 1.0 / (n - 1) as f32;
    let ids: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, [1.0, 0.0, 0.0], ITERACOES);
    let alvos = super::posicao_da_mancha(&m, &dirs, h, ITERACOES);

    let mut pior = 0.0f32;
    let mut conferidos = 0usize;
    for (i, &a) in alvos.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        let d = [a[0] - m.pos[i][0], a[1] - m.pos[i][1], a[2] - m.pos[i][2]];
        pior = pior.max(dot(d, d).sqrt() / h);
        conferidos += 1;
    }
    assert!(conferidos >= 40, "miolo: {conferidos}");
    assert!(pior <= 1.0e-4, "a chapa certa moveu-se: {pior} passos");
}

/// ⭐⭐⭐⭐ **UMA PILHA DE UM NÍVEL É A LEI DE HOJE — e sem este gate a recusa
/// da hierarquia não afirma nada.**
///
/// A [`super::campos_em_niveis`] foi construída por ordem do dono, mediu **pior
/// que um nível** em todas as colunas, e a primeira pergunta sobre uma tabela
/// dessas é sempre a mesma: *está a lei errada, ou está o meu código errado?*
///
/// ⇒ com `mais_grosso` acima do tamanho da pegada, a pilha tem **um nível só** e
/// a porta hierárquica tem de devolver **os mesmos `f32`** que a dupla
/// [`super::orientacao_semeada`] + [`super::posicao_da_mancha_com`]. Foi este
/// controlo que transformou aquela tabela numa medição — e ele é o que reprova
/// no dia em que alguém mexer na prolongação, na reposição dos fixos ou na
/// pilha e achar que só tocou no caminho de vários níveis.
///
/// ⚠️ **AO BIT e não «perto»:** a pergunta é *«é a mesma lei?»*, e uma barra de
/// tolerância responderia *«é parecida»*, que é outra pergunta.
#[test]
fn uma_pilha_de_um_nivel_e_a_lei_de_hoje() {
    let mesh = peca_densa();
    let centro = [0.0, 0.0, 1.0];
    let ids = pegada(&mesh, centro, 0.55);
    let m = mancha(&mesh, &ids);
    assert!(m.miolo() > 200, "a pegada precisa de miolo: {}", m.miolo());
    let passo = super::passo_da_pegada(&mesh, &ids);

    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);
    let pos = super::posicao_da_mancha_com(&m, &dirs, passo, ITERACOES, false);
    // ⚠️ Acima do tamanho da mancha ⇒ o `coarsen` nunca corre.
    let (d2, p2) = super::campos_em_niveis(&m, TRACO, passo, ITERACOES, m.len() + 1);

    assert_eq!(d2.len(), dirs.len());
    assert_eq!(p2.len(), pos.len());
    for i in 0..m.len() {
        assert_eq!(
            d2[i], dirs[i],
            "a direccao do vertice {i} nao e' a mesma lei"
        );
        assert_eq!(
            p2[i], pos[i],
            "a reticula do vertice {i} nao e' a mesma lei"
        );
    }
}

// ⚠️⚠️ **O gate do MECANISMO não mora aqui, e a primeira redacção dele morava.**
// Ele precisa de uma malha que já esteja numa retícula — e a única coisa que a
// produz é o TRAÇO (a retícula sozinha, sobre uma esfera de conectividade fixa,
// não converge: medido, `0,400 → 0,367` em oito passagens, porque mover os
// vértices re-deriva o consenso). ⇒ ele vive em `scenes_pente_grelha_tests.rs`,
// pela porta do produto. *A fixtura tem de ser a da cena que o dono usou.*

/// ⭐⭐ **A ÁREA DUAL DE UMA MANCHA SOMA A ÁREA DAS FACES DELA.**
///
/// A coluna nasceu para a hierarquia (o `coarsen` emparelha por razão de áreas
/// e faz a média ponderada por elas), e *uma coluna que ninguém confere é uma
/// coluna que pode estar cheia de uns*. Um terço da área de cada triângulo
/// incidente vai a cada canto ⇒ a soma sobre a mancha é **exactamente** a área
/// das faces colhidas.
///
/// ⚠️ **As faces são as do ANEL de cada vértice, não as contidas na mancha** —
/// é a mesma propriedade que torna os pesos cotangente do miolo exactos, e é
/// por isso que a soma se mede contra as faces COLHIDAS e não contra a pegada.
#[test]
fn a_area_dual_de_uma_mancha_soma_a_area_das_faces() {
    let mesh = peca_densa();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    assert_eq!(m.areas.len(), m.len());
    assert!(
        m.areas.iter().all(|a| *a > 0.0),
        "uma area nula ou negativa"
    );

    // As faces do anel, colhidas como a `mancha` as colhe.
    let dentro: std::collections::BTreeSet<u32> = m.ids.iter().copied().collect();
    let mut faces: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        if face.verts().iter().any(|v| dentro.contains(v)) {
            faces.insert(f as u32);
        }
    }
    let p = mesh.positions();
    let mut area_das_faces = 0.0f64;
    for &f in &faces {
        let face = mesh.faces()[f as usize];
        for t in 0..face.tri_count() {
            let tri = face.tri_at(t);
            let (a, b, c) = (p[tri[0] as usize], p[tri[1] as usize], p[tri[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let x = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            area_das_faces += f64::from(dot(x, x).sqrt()) * 0.5;
        }
    }
    // ⚠️ Só os cantos DENTRO da mancha recebem — um triângulo do anel com dois
    // cantos fora entrega um terço só, logo a soma é MENOR que a das faces.
    let soma: f64 = m.areas.iter().map(|a| f64::from(*a)).sum();
    assert!(
        soma > area_das_faces * 0.5 && soma <= area_das_faces * 1.000_01,
        "a area dual ({soma:.6}) nao descreve as faces colhidas ({area_das_faces:.6})"
    );
}

/// ⭐⭐⭐ **A FRANJA SOBREVIVE À PROLONGAÇÃO — a metade (2) da lei da
/// [`super::campos_em_niveis`], como gate.**
///
/// A prolongação **reescreve todos os vértices do nível filho**, franja
/// incluída; sem a reposição a condição de fronteira desaparece do nível fino,
/// que é o único que o produto lê. Um `fixos` sozinho não chega: ele impede o
/// núcleo de MOVER, não impede a prolongação de ter ESCRITO.
///
/// ⚠️ Ela é a irmã hierárquica da [`a_franja_fica_onde_a_semente_a_pos`], e
/// existe por uma razão que vale mesmo com o caminho de níveis RECUSADO: *a
/// tabela da recusa só descreve esta lei enquanto esta lei for esta*.
#[test]
fn a_franja_sobrevive_a_prolongacao() {
    let mesh = peca_densa();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    let passo = super::passo_da_pegada(&mesh, &ids);
    let (dirs, pos) = super::campos_em_niveis(&m, TRACO, passo, ITERACOES, 24);

    let mut vistos = 0usize;
    for i in 0..m.len() {
        if !m.fronteira[i] {
            continue;
        }
        vistos += 1;
        assert_eq!(pos[i], m.pos[i], "a franja {i} saiu do sitio");
        assert_eq!(
            dirs[i],
            crate::orientation::project_tangent(TRACO, m.nrm[i]),
            "a direccao da franja {i} nao e' a semente"
        );
    }
    assert!(vistos > 20, "a fixtura mal tem franja: {vistos}");
}

/// ⭐⭐ **A CLASSE ESCOLHIDA CHEGA AO BARRO — senão a sonda da recusa mede duas
/// vezes a mesma lei.**
///
/// O [`super::Campos::PorNiveis`] **não tem chamador de produto e não deve
/// ter**; o único que o exercita é a sonda que produziu a tabela da recusa. ⇒
/// um despacho que ignorasse o enum deixaria essa sonda a comparar um nível
/// consigo próprio e a imprimir *«não há diferença»* — que é a conclusão
/// OPOSTA à medida.
#[test]
fn a_classe_escolhida_chega_ao_barro() {
    let base = peca_densa();
    let centro = [0.0, 0.0, 1.0];
    let um = |_p: [f32; 3]| 1.0f32;
    let mut saidas = Vec::new();
    for classe in [
        super::Campos::UmNivel {
            semente_unica: false,
        },
        super::Campos::PorNiveis { mais_grosso: 24 },
    ] {
        let mut mesh = base.clone();
        let mut movidos = Vec::new();
        super::arruma_na_grelha_por(
            &mut mesh,
            centro,
            0.55,
            TRACO,
            &um,
            ITERACOES,
            0.80,
            classe,
            &mut movidos,
        );
        assert!(!movidos.is_empty(), "{classe:?} nao moveu nada");
        saidas.push(mesh.positions().to_vec());
    }
    assert_ne!(
        saidas[0], saidas[1],
        "as duas classes deram o MESMO barro — o despacho nao chega"
    );
}

/// ⭐⭐⭐⭐ **DOIS VIZINHOS NÃO CONSPIRAM NUMA LASCA — a propriedade que a cerca
/// por-vértice NÃO PODIA ter, e que destravou as varreduras.**
///
/// A cerca antiga perguntava *«pôr ESTE vértice aqui afina um triângulo?»*, e o
/// laço é **Jacobi**: todos decidem contra as posições de entrada. ⇒ **dois
/// vizinhos que se aproximam passam os dois, cada um por si**, e juntos fecham
/// o triângulo que partilham. Foi essa cegueira que fez o portão da cena
/// recusar o degrau seguinte das varreduras em 20/09.
///
/// A fixtura é a aritmética do defeito, e ela **nomeia os três ângulos**:
///
/// - o triângulo abre com **`12°`** no canto `C`;
/// - **cada movimento sozinho** deixa-o em `8°` — pior, e **acima** do chão de
///   `5°` ⇒ a cerca antiga aprovava, e a nova também tem de aprovar;
/// - **os dois juntos** deixam-no em `4°` ⇒ abaixo do chão, e os dois são
///   vetados.
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a de baixo, uma cerca que
/// vetasse TUDO passaria na de cima — e uma retícula que não move nada é
/// exactamente o produto que o dono já reprovou quatro vezes.
#[test]
fn dois_vizinhos_nao_conspiram_numa_lasca() {
    // `C` na origem, `A` e `B` a `12°` um do outro. Mover cada um `4°` na
    // direcção do outro deixa `8°`; mover os dois deixa `4°`.
    let g = |graus: f32| -> [f32; 3] {
        let r = graus.to_radians();
        [r.cos(), r.sin(), 0.0]
    };
    let mesh = Mesh::from_parts(
        vec![[0.0, 0.0, 0.0], g(0.0), g(12.0), [0.5, -0.6, 0.0]],
        // ⚠️ A segunda face existe para `A` e `B` terem anel: uma aresta com um
        // triângulo só é bordo, e o veto varre o anel dos MOVIDOS.
        vec![ph2d_mesh::Face::tri(0, 1, 2), ph2d_mesh::Face::tri(0, 3, 1)],
    )
    .expect("a fixtura e' bem formada");

    let (a, b) = ((1u32, g(4.0)), (2u32, g(8.0)));

    // ⭐ As duas metades de BAIXO primeiro: cada um sozinho PASSA.
    for um in [a, b] {
        let ficou = super::veta_combinado(&mesh, vec![um]);
        assert_eq!(
            ficou.len(),
            1,
            "o movimento {um:?} sozinho deixa 8 graus, que esta' ACIMA do chao"
        );
    }

    // ⛔ E a de cima: juntos, os dois caem.
    let ficou = super::veta_combinado(&mesh, vec![a, b]);
    assert!(
        ficou.is_empty(),
        "os dois juntos fecham o canto a 4 graus e mesmo assim {} sobreviveu(ram)",
        ficou.len()
    );
}

/// ⭐⭐⭐ **UMA LASCA QUE JÁ LÁ ESTAVA NÃO PRENDE O VÉRTICE PARA SEMPRE — a
/// metade `pior` da cerca, e ela nasceu de uma MUTAÇÃO SOBREVIVENTE.**
///
/// A cerca é `pior && abaixo do chão`, **nunca só uma das duas**: só *«abaixo
/// do chão»* prenderia para sempre um vértice cujo anel **já nasceu** com uma
/// lasca, e essa é a metade que o produto encontra numa peça **esculpida** —
/// não na esfera lisa da cena, que é por isso que a mutação sobreviveu ao
/// portão dela.
///
/// A fixtura é a aritmética: o canto abre com **`3°`** (já abaixo do chão de
/// `5°`) e o movimento leva-o a **`4°`** — *melhor, e ainda abaixo*. Com as duas
/// metades ele **passa**; só com o chão, seria vetado e a malha ficava
/// congelada ali.
#[test]
fn uma_lasca_que_ja_la_estava_nao_prende_o_vertice() {
    let g = |graus: f32| -> [f32; 3] {
        let r = graus.to_radians();
        [r.cos(), r.sin(), 0.0]
    };
    let mesh = Mesh::from_parts(
        // `C` na origem, `A` e `B` a `3°` — o anel ja' nasce com uma lasca.
        vec![[0.0, 0.0, 0.0], g(0.0), g(3.0), [0.5, -0.6, 0.0]],
        vec![ph2d_mesh::Face::tri(0, 1, 2), ph2d_mesh::Face::tri(0, 3, 1)],
    )
    .expect("a fixtura e' bem formada");

    // Mover `A` para `-1°` abre o canto para `4°`: MELHOR, e ainda abaixo do chao.
    let ficou = super::veta_combinado(&mesh, vec![(1u32, g(-1.0))]);
    assert_eq!(
        ficou.len(),
        1,
        "o vertice ficou preso por uma lasca que ele proprio nao criou"
    );
}
