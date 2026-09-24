//! ⭐⭐⭐⭐ **A P2 — o `R` POR FACE**: as leis do endereço quando a retícula
//! deixa de ter um lado só.
//!
//! ⚠️ **A propriedade que tudo o resto serve é UMA:** duas faces que se tocam
//! lêem a **mesma célula** na fronteira. Ela é a razão de esta família existir
//! (é a diferença de espécie para o Ptex), e é a primeira a partir-se quando
//! cada face passa a ter a resolução dela.

use crate::{NIVEL_MAX, Tinta, Topologia, amostragem::posicao_tri, interior_por_face, total};

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
/// ⭐⭐⭐⭐ **O PLANO GUARDA O DEGRAU QUE FOI PEDIDO, e ele NÃO é o nível mais
/// fino.**
///
/// ⛔⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE (2026-09-23), e o que
/// ela expôs é que a lei tinha ficado SEM RÉGUA quando o `Even Detail` saiu:**
/// quem a matava era o gate da igualação chegar ao plano, que se foi com o
/// sujeito dele. *Uma lei pode ficar descoberta porque o único gate que a
/// cobria era de outra feature* — e o pré-voo não o vê, porque a âncora dela
/// continua a casar.
///
/// ⚠️⚠️ **E a ida-e-volta do ficheiro é CEGA a isto, por construção:** ela
/// afirma `volta.nivel() == t.nivel()`, e sob a confusão os DOIS lados leem o
/// nível mais fino ⇒ ela passa. *Um oráculo de igualdade não vê um erro que os
/// dois lados cometem.*
///
/// ⭐ **E porque isto importa:** o consumidor compara o `nivel()` com o degrau
/// que a fileira do painel pede. Confundi-los faz a resposta ser **NÃO em todo
/// quadro** num plano onde alguma face subiu — o plano é reconstruído e
/// **re-semeado da cor por vértice**, e a tinta fina do artista desaparece a
/// `60 Hz` (handoff §25.4).
#[test]
fn o_plano_graduado_guarda_o_pedido_e_nao_o_mais_fino() {
    let (pos, faces) = duas_faces();
    let t = Tinta::graduada(pos.len(), it(&faces), &[2, 4], 2).expect("dois níveis, duas faces");

    // ⭐ CONTROLO PRIMEIRO: a fixtura tem de DISCRIMINAR. Com uma lista cujo
    //   máximo é o próprio pedido, as duas leituras coincidem e este gate
    //   passaria a afirmar nada.
    assert_eq!(
        t.topologia().nivel_mais_fino(),
        4,
        "o CONTROLO: a fixtura tem de ter uma face ACIMA do pedido"
    );

    assert_eq!(
        t.nivel(),
        2,
        "o plano guardou {} — o degrau pedido era 2 e o mais fino é 4",
        t.nivel()
    );
}

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

// ⛔⛔⛔ **AQUI VIVIAM OS DOIS GATES DA LEI POR ÁREA** — o da dispersão e o da
// cerca do salto —, e **saíram com a lei** em 2026-09-23, quando o dono
// retirou o `Even Detail`.
//
// ⚠️ A `niveis_por_area` tinha UM consumidor, o escolhedor; sem ele ela ficou
// **viva e órfã** e estes dois passaram a medir só a si mesmos. *Um gate cujo
// único chamador é ele próprio não cobre nada.*
//
// ⭐ **E o que eles mediram fica:** o chão da dispersão é `2×` e não `√2` — *o
// `√2` é o desvio ao alvo de UMA face e a dispersão é uma razão entre DUAS*,
// que é o que desmentia o handoff de 21/09 —, e a cerca do salto era um GUARDA
// com o custo medido (`1`–`6` arestas de `16 k`–`43 k`). A tabela inteira, com
// as três peças do dono, está no handoff §25–§26; é de lá que se parte se a
// graduação voltar, nunca deste ficheiro.

// ⛔⛔⛔⛔ **AQUI VIVIA O GATE DA PORTA DO PRODUTO
// (`a_porta_do_produto_poe_a_face_mediana_no_k_pedido`), e ele saiu com o
// sujeito dele: o `niveis_igualados` foi RETIRADO por ordem do dono em
// 2026-09-23** (*«Even Detail derruba muito a resolução. retiro!»*).
//
// ⚠️ Ele tinha morrido e renascido DUAS vezes em vinte e quatro horas — a
// metade dos «níveis distintos» e a da «dispersão a metade», as duas mortas
// pelo piso —, e o que fica disso é a lição e não o código: *uma premissa que
// morre e renasce num dia é a melhor prova de que tinha de estar num gate*.
//
// ⛔ O que ele mediu NÃO se perdeu: a `niveis_por_area` — a lei por baixo,
// que responde «que nível uma face desta área quer» — continua com os gates
// dela nesta mesma varredura. O que saiu foi quem escolhia o ALVO a partir de
// um chip do painel. Medição inteira: handoff §25–§26.
