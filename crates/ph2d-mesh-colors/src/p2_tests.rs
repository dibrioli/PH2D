//! ⭐⭐⭐⭐ **A P2 — o `R` POR FACE**: as leis do endereço quando a retícula
//! deixa de ter um lado só.
//!
//! ⚠️ **A propriedade que tudo o resto serve é UMA:** duas faces que se tocam
//! lêem a **mesma célula** na fronteira. Ela é a razão de esta família existir
//! (é a diferença de espécie para o Ptex), e é a primeira a partir-se quando
//! cada face passa a ter a resolução dela.

use crate::{
    NIVEL_MAX, Tinta, Topologia, amostragem::posicao_tri, interior_por_face, niveis_por_area, total,
};

/// Dois triângulos que partilham a aresta `(1, 2)`.
///
/// ```text
///   2 ---- 3        face 0 = [0, 1, 2]
///   | \    |        face 1 = [1, 3, 2]
///   |  \   |        partilhada: (1, 2), a diagonal
///   0 ---- 1
/// ```
fn duas_faces() -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    (pos, vec![vec![0, 1, 2], vec![1, 3, 2]])
}

fn it(f: &[Vec<u32>]) -> impl Iterator<Item = &[u32]> + Clone {
    f.iter().map(Vec::as_slice)
}

/// Uma chave de posição estável — as duas faces calculam a MESMA aritmética
/// afim sobre os MESMOS cantos, logo o ponto partilhado é igual ao bit.
fn chave(p: [f32; 3]) -> [u32; 3] {
    [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]
}

/// Todas as amostras de uma face, com a posição delas.
fn amostras_da_face(
    t: &Tinta,
    pos: &[[f32; 3]],
    faces: &[Vec<u32>],
    f: usize,
) -> Vec<([u32; 3], u32)> {
    let c = &faces[f];
    let (a, b, cc) = (pos[c[0] as usize], pos[c[1] as usize], pos[c[2] as usize]);
    let l = t.lado_da_face(f);
    let mut out = Vec::new();
    t.para_cada_amostra_tri(f, c, |idx, ijk| {
        out.push((chave(posicao_tri(a, b, cc, l, ijk)), idx));
    });
    out
}

/// ⭐⭐⭐⭐ **A FRONTEIRA CONTINUA PARTILHADA com níveis diferentes dos dois
/// lados** — a lei que faz esta família não ter costura.
///
/// ⛔⛔ **O mecanismo que ela defende:** a face grossa conta `t` na retícula
/// DELA e a aresta guarda as amostras na retícula DELA (o máximo dos dois
/// vizinhos). Sem a multiplicação pelo passo, a amostra `t = 1` de uma face de
/// lado `2` cairia na célula `1` de uma aresta de lado `8` — *um oitavo do
/// caminho em vez de metade* —, e as duas faces pintariam sítios diferentes
/// com o mesmo gesto.
///
/// ⚠️ **O CONTROLO está dentro:** a fixtura tem de PRODUZIR uma amostra
/// partilhada que não seja um canto, senão o gate mede os dois vértices e fica
/// verde sobre um endereço partido.
#[test]
fn as_duas_faces_concordam_na_fronteira_com_niveis_diferentes() {
    let (pos, faces) = duas_faces();
    for (ka, kb) in [(1u8, 3u8), (3, 1), (0, 2), (2, 2)] {
        let t = Tinta::graduada(pos.len(), it(&faces), &[ka, kb], ka.max(kb))
            .expect("dois níveis, duas faces");

        let a = amostras_da_face(&t, &pos, &faces, 0);
        let b = amostras_da_face(&t, &pos, &faces, 1);

        let mut partilhadas = 0usize;
        let mut cantos = 0usize;
        for (chave_a, idx_a) in &a {
            for (chave_b, idx_b) in &b {
                if chave_a == chave_b {
                    assert_eq!(
                        idx_a, idx_b,
                        "níveis {ka}/{kb}: a mesma posição da fronteira tem \
                         dois endereços ({idx_a} contra {idx_b})"
                    );
                    partilhadas += 1;
                    // Os dois cantos da aresta partilhada são vértices.
                    if *idx_a < pos.len() as u32 {
                        cantos += 1;
                    }
                }
            }
        }
        // ⚠️⚠️ **Controlo, e ele tem uma cerca que a 1.ª redacção não tinha:**
        //    sem uma amostra de ARESTA partilhada este gate mediria só os dois
        //    vértices, que são triviais — ⛔ mas a `lado = 1` a face NÃO TEM
        //    amostras de aresta nenhumas (é o caso base, uma por canto), logo
        //    ali a lei da partilha não tem sobre o que falar. *Exigir o
        //    controlo nessa célula reprova sobre produto correcto*, e foi o
        //    que ele fez à primeira.
        if ka.min(kb) >= 1 {
            assert!(
                partilhadas - cantos >= 1,
                "níveis {ka}/{kb}: a fixtura não produziu uma amostra de aresta \
                 partilhada ({partilhadas} partilhadas, {cantos} cantos)"
            );
        } else {
            assert_eq!(
                partilhadas, cantos,
                "níveis {ka}/{kb}: a `lado = 1` só os cantos são partilhados"
            );
        }
        assert_eq!(cantos, 2, "níveis {ka}/{kb}: a aresta tem dois cantos");
    }
}

/// ⭐⭐⭐ **A ARESTA leva o MÁXIMO dos vizinhos, e o bloco dela é a SOMA.**
///
/// ⚠️ **A metade do UNIFORME é o que prova que nada se mexeu:** com um nível só
/// os prefixos valem `id × (lado − 1)` e `f × interior(lado)` **exactamente**,
/// que é a aritmética que o shader ainda faz. *Sem ela, esta wave podia ter
/// mudado o caminho da placa sem uma linha de WGSL se mexer.*
#[test]
fn a_aresta_leva_o_maximo_e_o_uniforme_fica_na_aritmetica_de_antes() {
    let (pos, faces) = duas_faces();
    let base = Topologia::nova(pos.len(), it(&faces), 0);

    // --- graduado: a aresta partilhada é a mais fina das duas ---
    let g = base.regraduada(&[1, 3]).expect("regradua");
    for f in 0..g.faces() {
        for s in 0..g.cantos_de(f) {
            let (id, _) = g.aresta(f, s);
            let vizinhas: Vec<u8> = (0..g.faces())
                .filter(|&o| (0..g.cantos_de(o)).any(|t| g.aresta(o, t).0 == id))
                .map(|o| g.nivel_de(o))
                .collect();
            let maior = 1u32 << vizinhas.iter().copied().max().expect("tem vizinha");
            assert_eq!(
                g.aresta_lado(id),
                maior,
                "a aresta {id} devia levar o máximo dos vizinhos {vizinhas:?}"
            );
        }
    }
    // A soma do bloco das arestas é a soma dos comprimentos, uma a uma.
    let soma: u32 = (0..g.arestas() as u32)
        .map(|id| g.aresta_lado(id) - 1)
        .sum();
    assert_eq!(g.arestas_amostras(), soma);

    // --- uniforme: os prefixos voltam a ser PRODUTOS ---
    for k in [0u8, 1, 3] {
        let u = base.regraduada(&[k, k]).expect("regradua");
        let lado = 1u32 << k;
        for id in 0..u.arestas() as u32 {
            assert_eq!(u.aresta_off(id), id * (lado - 1), "nível {k}, aresta {id}");
        }
        assert_eq!(
            total(&u),
            pos.len() + u.arestas() * (lado as usize - 1) + 2 * interior_por_face(3, lado) as usize,
            "nível {k}: a contagem uniforme tem forma fechada"
        );
        assert_eq!(u.nivel_uniforme(), Some(k));
        // E a `nova` com o mesmo nível dá EXACTAMENTE a mesma topologia.
        assert_eq!(Topologia::nova(pos.len(), it(&faces), k), u, "nível {k}");
    }
    assert_eq!(g.nivel_uniforme(), None, "um plano graduado não é uniforme");
}

/// ⛔⛔ **Uma lista de níveis que não descreve esta malha é RECUSADA.**
///
/// *Um plano com o tamanho errado instalado numa malha é tinta no sítio
/// errado* — a mesma lei que a [`Topologia::descreve`] impõe às faces.
#[test]
fn uma_lista_de_niveis_do_tamanho_errado_e_recusada() {
    let (pos, faces) = duas_faces();
    let base = Topologia::nova(pos.len(), it(&faces), 0);
    assert!(base.regraduada(&[1]).is_none(), "curta de mais");
    assert!(base.regraduada(&[1, 2, 3]).is_none(), "comprida de mais");
    assert!(base.regraduada(&[1, 2]).is_some(), "o controlo passa");
    // E o nível é CORTADO, nunca aceite acima da escada.
    let alto = base.regraduada(&[9, 9]).expect("regradua");
    assert_eq!(alto.nivel_de(0), NIVEL_MAX);
}

/// ⭐⭐⭐ **A LEI DA ÁREA baixa a dispersão, e o CHÃO dela é `2×`.**
///
/// ⚠️⚠️ **É este gate que desmente o handoff de 21/09**, que escreveu
/// *«o pior caso é `√2 = 1,41×`»*: o `√2` é o desvio ao alvo de UMA face e a
/// dispersão é uma razão entre DUAS. Ver [`niveis_por_area`].
#[test]
fn o_nivel_por_area_baixa_a_dispersao_e_o_chao_e_dois() {
    // Uma tira de triângulos cujas áreas crescem 64× de ponta a ponta.
    let n = 32usize;
    let mut pos = Vec::new();
    let mut faces = Vec::new();
    let mut x = 0.0f32;
    for i in 0..n {
        let h = 0.1 * (1.0 + i as f32 * 0.25);
        pos.push([x, 0.0, 0.0]);
        pos.push([x, h, 0.0]);
        x += h;
    }
    pos.push([x, 0.0, 0.0]);
    pos.push([x, 0.1 * (1.0 + n as f32 * 0.25), 0.0]);
    for i in 0..n {
        let a = (2 * i) as u32;
        faces.push(vec![a, a + 2, a + 1]);
    }
    let areas: Vec<f32> = faces
        .iter()
        .map(|f| {
            let (a, b, c) = (pos[f[0] as usize], pos[f[1] as usize], pos[f[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1]];
            let v = [c[0] - a[0], c[1] - a[1]];
            0.5 * (u[0] * v[1] - u[1] * v[0]).abs()
        })
        .collect();

    let densidade = |k: &[u8]| -> (f32, f32) {
        let mut d: Vec<f32> = k
            .iter()
            .zip(&areas)
            .map(|(k, a)| f32::from(1u16 << k) / a.sqrt())
            .collect();
        d.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
        (d[0], d[d.len() - 1])
    };

    let base = Topologia::nova(pos.len(), it(&faces), 0);
    let uniforme = vec![3u8; faces.len()];
    let (u0, u1) = densidade(&uniforme);
    let disp_uniforme = u1 / u0;

    // O alvo é a MEDIANA do uniforme — orçamento parecido, não «mais ganha».
    let mut d: Vec<f32> = areas.iter().map(|a| 8.0 / a.sqrt()).collect();
    d.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    let alvo = d[d.len() / 2];

    let k = niveis_por_area(&base, &areas, alvo, 1);
    let (p0, p1) = densidade(&k);
    let disp_por_face = p1 / p0;

    assert!(
        disp_uniforme > 4.0,
        "a fixtura tem de CONTER o defeito: dispersão uniforme {disp_uniforme:.2}×"
    );
    assert!(
        disp_por_face <= 2.0 + 1e-3,
        "o chão é 2× (duas faces, cada uma a meia escada do alvo): {disp_por_face:.3}×"
    );
    assert!(
        disp_por_face < disp_uniforme / 2.0,
        "a lei tem de COMPRAR alguma coisa: {disp_uniforme:.2}× → {disp_por_face:.2}×"
    );
    // E ela produz mesmo níveis diferentes, senão o número acima é um acidente.
    let mut distintos = k.clone();
    distintos.sort_unstable();
    distintos.dedup();
    assert!(distintos.len() >= 3, "níveis distintos: {distintos:?}");
}

/// ⭐⭐ **A CERCA do salto entre vizinhas chega ao PONTO FIXO e nunca DESCE
/// ninguém.**
///
/// ⚠️ **Ela é um GUARDA e isso é declarado:** medida no corpus do dono ela é
/// quase inerte (o salto natural já é `≤ 2`), e a fixtura aqui é construída
/// para o fenómeno — uma tira onde a área salta de uma ponta à outra.
#[test]
fn a_cerca_do_salto_sobe_a_vizinha_e_chega_ao_ponto_fixo() {
    // ⛔⛔ **Uma TIRA a sério, e a 1.ª redacção NÃO era uma:** ela emitia um
    //    triângulo por coluna, e dois deles partilham só um VÉRTICE — a cerca
    //    corre por ARESTA, logo não propagava nada e o gate ficava verde sobre
    //    uma corrente que não existia. Quem o apanhou foi o clippy, na
    //    asserção final: `NIVEL_MAX - 5` é `0`, e `x >= 0` num `u8` é
    //    **sempre** verdade. *Uma asserção que não pode falhar é comentário
    //    com sintaxe de código.*
    //
    //    A tira certa alterna `A` e `B`: `A_i` e `B_i` partilham `(2i+1,2i+2)`
    //    e `B_i` e `A_{i+1}` partilham `(2i+2,2i+3)`.
    let n = 8usize;
    let mut pos = Vec::new();
    let mut faces = Vec::new();
    for i in 0..=n {
        pos.push([i as f32, 0.0, 0.0]);
        pos.push([i as f32, 1.0, 0.0]);
    }
    for i in 0..n {
        let a = (2 * i) as u32;
        faces.push(vec![a, a + 2, a + 1]);
        faces.push(vec![a + 1, a + 2, a + 3]);
    }
    let n = faces.len();
    let base = Topologia::nova(pos.len(), it(&faces), 0);

    // Um degrau brutal escrito à mão: a primeira no topo, as outras no chão.
    let mut cru = vec![0u8; n];
    cru[0] = NIVEL_MAX;
    let areas: Vec<f32> = cru
        .iter()
        .map(|k| {
            // A área que devolve exactamente este nível com `alvo = 1`.
            let lado = f32::from(1u16 << k);
            lado * lado
        })
        .collect();

    let sem = niveis_por_area(&base, &areas, 1.0, NIVEL_MAX);
    assert_eq!(sem, cru, "sem cerca a lei devolve o degrau cru");

    let com = niveis_por_area(&base, &areas, 1.0, 1);
    // Ponto fixo: nenhuma aresta tem salto acima da cerca.
    let g = base.regraduada(&com).expect("regradua");
    for f in 0..g.faces() {
        for s in 0..g.cantos_de(f) {
            let (id, _) = g.aresta(f, s);
            let vizinhas: Vec<u8> = (0..g.faces())
                .filter(|&o| (0..g.cantos_de(o)).any(|t| g.aresta(o, t).0 == id))
                .map(|o| com[o])
                .collect();
            let (lo, hi) = (
                vizinhas.iter().copied().min().expect("tem"),
                vizinhas.iter().copied().max().expect("tem"),
            );
            assert!(
                hi - lo <= 1,
                "aresta {id}: salto {} em {vizinhas:?}",
                hi - lo
            );
        }
    }
    // ⭐ Ela SOBE e nunca DESCE — descer apagaria detalhe que o artista pediu.
    for (f, (antes, depois)) in cru.iter().zip(&com).enumerate() {
        assert!(depois >= antes, "face {f}: {antes} desceu para {depois}");
    }
    // ⭐⭐ **E a escada é EXACTA ao longo da corrente** — é isto que prova que a
    //    cerca é transitiva e que a fixtura é mesmo uma tira. ⛔ A asserção que
    //    aqui esteve (`com[5] >= NIVEL_MAX - 5`) era **vácua** num `u8`.
    for (j, &k) in com.iter().enumerate().take(usize::from(NIVEL_MAX) + 2) {
        let esperado = NIVEL_MAX.saturating_sub(u8::try_from(j).unwrap_or(u8::MAX));
        assert_eq!(k, esperado, "face {j} da corrente: a escada é {com:?}");
    }
}

/// ⭐⭐⭐⭐ **A PORTA DO PRODUTO ANCORA NA MEDIANA E O `k` É UM PISO — a face
/// típica fica ao nível que o artista pediu, e NINGUÉM desce abaixo dele.**
///
/// ⛔⛔⛔ **A metade do PISO nasceu de um REPORT do dono (23/09) e ela MATOU
/// duas premissas deste gate, que ficam aqui escritas porque a morte delas é a
/// wave:**
///
/// 1. ***«os níveis são DISTINTOS, `≥ 3` deles»*** — com o piso, tudo o que
///    ficava abaixo de `k` sobe para `k`, logo numa peça cuja dispersão cabe
///    num degrau da escada saem **dois** níveis e não três. Medido nesta
///    fixtura: `[k, k+1]` em todo `k` de `0` a `4`, com `7` das `32` faces
///    acima. ⇒ a metade honesta é **«há faces ACIMA de `k`»**, que é o que
///    aquela queria dizer (*isto não é um plano uniforme*) sem exigir a metade
///    de baixo que a ordem do dono proíbe.
/// 2. ***«a dispersão cai para menos de METADE»*** — ela caía porque a lei
///    descia as faces pequenas **e** subia as grandes. Com só uma direcção ela
///    cai de `8,75×` para `7,00×` aqui, e exigir metade seria exigir de volta
///    exactamente o que o dono reprovou. ⇒ a barra é **descer estritamente**, e
///    a lei que a torna forte é a monotonia: subir uma face nunca sobe o
///    MÁXIMO (ele pertence à face mais pequena, que já está em `k`).
///
/// ⚠️ **A `k = NIVEL_MAX` a metade `2` é inexprimível** — no topo da escada não
/// há para onde subir —, e a população do gate di-lo em vez de a contornar.
///
/// ⚠️ E a `k = 0` ela **não** é obrigada a devolver zeros: a densidade é
/// `lado/√área`, logo quem satura contra o chão são as faces PEQUENAS, e as
/// grandes sobem na mesma (medido: `7` acima em `k = 0`).
#[test]
fn a_porta_do_produto_poe_a_face_mediana_no_k_pedido() {
    // A mesma tira do gate da dispersão: áreas que crescem de ponta a ponta.
    let n = 32usize;
    let (mut pos, mut faces) = (Vec::new(), Vec::new());
    let mut x = 0.0f32;
    for i in 0..n {
        let h = 0.1 * (1.0 + i as f32 * 0.25);
        pos.push([x, 0.0, 0.0]);
        pos.push([x, h, 0.0]);
        x += h;
    }
    pos.push([x, 0.0, 0.0]);
    pos.push([x, 0.1 * (1.0 + n as f32 * 0.25), 0.0]);
    for i in 0..n {
        let a = (2 * i) as u32;
        faces.push(vec![a, a + 2, a + 1]);
    }
    let areas: Vec<f32> = faces
        .iter()
        .map(|f| {
            let (a, b, c) = (pos[f[0] as usize], pos[f[1] as usize], pos[f[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1]];
            let v = [c[0] - a[0], c[1] - a[1]];
            0.5 * (u[0] * v[1] - u[1] * v[0]).abs()
        })
        .collect();
    let base = Topologia::nova(pos.len(), it(&faces), 0);

    for k in 1..=4u8 {
        let niveis = super::niveis_igualados(&base, &areas, k, 1);
        assert_eq!(niveis.len(), faces.len(), "k={k}: uma entrada por face");

        // (1) A MEDIANA é o `k` pedido.
        let mut ord = niveis.clone();
        ord.sort_unstable();
        let mediana = ord[(ord.len() - 1) / 2];
        assert_eq!(
            mediana, k,
            "k={k}: a face típica saiu em {mediana} e o artista pediu {k} ({ord:?})"
        );

        // (2) ⭐⭐⭐ **O PISO: ninguém abaixo do que o artista pediu.** Esta é a
        //     metade que o report de 23/09 comprou, e a única cuja violação o
        //     dono VÊ (*«a resolução fica bem baixa, inclusive a 16x»*).
        assert!(
            niveis.iter().all(|n| *n >= k),
            "k={k}: {niveis:?} — alguma face saiu MAIS GROSSA do que o degrau pedido"
        );

        // (3) ⭐ CONTROLO: há faces ACIMA — senão isto é um plano uniforme.
        //     ⚠️ Inexprimível no topo da escada, e a população di-lo.
        if k < NIVEL_MAX {
            assert!(
                niveis.iter().any(|n| *n > k),
                "k={k}: {niveis:?} — a porta devolveu um plano uniforme"
            );
        }

        // (4) E a dispersão cai contra o uniforme do MESMO `k` — ESTRITAMENTE,
        //     que é tudo o que uma lei de um sentido só pode prometer.
        let disp = |ks: &[u8]| -> f32 {
            let mut v: Vec<f32> = ks
                .iter()
                .zip(&areas)
                .map(|(k, a)| f32::from(1u16 << k) / a.sqrt())
                .collect();
            v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
            v[v.len() - 1] / v[0]
        };
        let (uni, por_face) = (disp(&vec![k; faces.len()]), disp(&niveis));
        if k < NIVEL_MAX {
            assert!(
                por_face < uni,
                "k={k}: a porta tem de COMPRAR alguma coisa — {uni:.2}× → {por_face:.2}×"
            );
        }
    }

    // ⭐⭐⭐ **O CHÃO, e a PREMISSA QUE ESTE GATE DERRUBOU.** Eu escrevi aqui
    //   *«a `k = 0` não há como igualar, logo tudo devolve zeros»* e a corrida
    //   respondeu `[0 × 25, 1 × 7]`. ⛔ Estava ao contrário: a densidade é
    //   `lado / √área`, logo **uma face GRANDE precisa de MAIS subdivisões**
    //   para chegar ao alvo, e quem satura contra o piso são as PEQUENAS.
    //
    //   ⇒ o que se afirma é o que a lei de facto faz: a mediana continua no
    //   `k` pedido, **ninguém desce abaixo do chão**, e o único sentido
    //   disponível ali é para CIMA. ⚠️ *Isto é o mesmo mecanismo que refutou a
    //   leitura do TECTO (handoff §28.3) visto do outro lado da escada.*
    let zero = super::niveis_igualados(&base, &areas, 0, 1);
    let mut ord = zero.clone();
    ord.sort_unstable();
    assert_eq!(ord[(ord.len() - 1) / 2], 0, "k=0: a mediana saiu de zero");
    assert!(zero.iter().any(|k| *k > 0), "k=0: ninguém subiu — {zero:?}");
    assert!(
        zero.iter().all(|k| *k <= NIVEL_MAX),
        "k=0: alguém passou o tecto da escada — {zero:?}"
    );
}
