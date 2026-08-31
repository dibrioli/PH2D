//! ⭐⭐⭐ **OS GATES DA CHAVE DA PONTA** — irmão de [`super::tests`] pelo teto de LOC
//! da shell (HR-18, 600), cortado por RESPONSABILIDADE: aquele ficheiro defende as chaves
//! de **defeito** do [`super::worse`] (furos · peças · gravatas), este a chave de
//! **cobertura** que entrou em 2026-08-30 e a fronteira entre as duas.
//!
//! ⛔⛔⛔ **O report que os exige** (Enio, 28 e 29/08): *«as pontas finas perdem detalhe»*.
//! A cadeia **já produzia** a candidata que o cura e o desempate deitava-a fora.

use ph2d_mesh::{Face, Mesh};

use super::tests::{cubo, um_quad};
/// **UMA NUVEM COM PONTA** — `8` quads de lado `1` perto da origem e `4` quads longe, com o
/// lado da ponta escolhido pelo chamador.
///
/// ⚠️ **Quads SOLTOS de propósito:** as chaves da frente do [`super::worse`] (furos, peças,
/// gravatas) ficam **iguais** entre duas nuvens com a mesma contagem, e é isso que deixa a
/// chave da ponta ser a que decide. *Uma fixtura em que as chaves da frente diferem não testa
/// a chave nova: testa as antigas.*
fn nuvem_com_ponta(lado_da_ponta: f32) -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    let quad = |cx: f32, lado: f32, verts: &mut Vec<[f32; 3]>, faces: &mut Vec<Face>| {
        let b = u32::try_from(verts.len()).expect("a fixtura e' pequena");
        verts.push([cx, 0.0, 0.0]);
        verts.push([cx + lado, 0.0, 0.0]);
        verts.push([cx + lado, lado, 0.0]);
        verts.push([cx, lado, 0.0]);
        faces.push(Face::quad(b, b + 1, b + 2, b + 3));
    };
    for k in 0..8 {
        #[expect(clippy::cast_precision_loss, reason = "k <= 8 nesta fixtura")]
        let cx = k as f32 * 2.0;
        quad(cx, 1.0, &mut verts, &mut faces);
    }
    for k in 0..4 {
        #[expect(clippy::cast_precision_loss, reason = "k <= 4 nesta fixtura")]
        let cx = 100.0 + k as f32 * 2.0;
        quad(cx, lado_da_ponta, &mut verts, &mut faces);
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — a PONTA decide quando todas as chaves de DEFEITO empatam.**
///
/// ⛔⛔⛔ **É o gate do report de 28-29/08** (*«as pontas finas perdem detalhe»*). Medido em
/// 2026-08-30 na `sculpt_antes.obj`: das três candidatas do botão, a de **linhas de feição**
/// empatava em furos, peças, gravatas e faces `>60°` com a escolhida, entregava a ponta
/// **`1,8×` mais fina** (`0,851` contra `1,502`) — e **perdia**, porque a última chave era o
/// enviesamento mediano, a única das grandezas em jogo que o dono nunca nomeou.
#[test]
fn a_ponta_decide_quando_as_chaves_de_defeito_empatam() {
    let grossa = nuvem_com_ponta(2.0);
    let fina = nuvem_com_ponta(0.5);
    // ⚠️ **O CONTROLE vem antes da asserção:** se as chaves da frente não empatarem, o que
    // este gate mede é uma delas e não a nova.
    assert_eq!(
        super::open_edges(&grossa),
        super::open_edges(&fina),
        "⛔ a fixtura tem de empatar nos furos, senao a chave da frente e' que decide"
    );
    assert_eq!(super::components(&grossa), super::components(&fina));
    assert_eq!(super::bowties(&grossa), super::bowties(&fina));
    let (r_grossa, n_grossa) = super::tip_ratio(&grossa);
    let (r_fina, n_fina) = super::tip_ratio(&fina);
    assert!(
        n_grossa > 0 && n_fina > 0,
        "⛔ a amostra da ponta esta' vazia -- a fixtura nao tem ponta nenhuma"
    );
    assert!(
        r_grossa > r_fina,
        "⛔ a fixtura nao separa as duas pontas ({r_grossa:.3} contra {r_fina:.3})"
    );
    // O mesmo `>60` e o mesmo enviesamento dos dois lados: so' a ponta pode decidir.
    assert!(
        super::worse(&grossa, 2, 5.0, &fina, 2, 5.0),
        "⛔ a candidata de ponta GROSSA tem de perder"
    );
    assert!(
        !super::worse(&fina, 2, 5.0, &grossa, 2, 5.0),
        "⛔ e a de ponta FINA tem de ganhar"
    );
}

/// ⭐⭐⭐ **GATE — os FUROS continuam a ganhar da ponta, e isso é a decisão.**
///
/// ⛔ Com `Follow Curvature` ligado, a candidata de linhas de feição chega a `0,543` — o alvo
/// derivado do oráculo aprovado é `0,59` — e traz `6` arestas de bordo contra `4`. *Buracos
/// foram a queixa do dono três vezes; esta chave não os compra.*
#[test]
fn a_ponta_nunca_ganha_de_um_furo() {
    let fina = nuvem_com_ponta(0.5);
    let (cv, cf) = cubo(0.0);
    let fechada = Mesh::from_parts(
        cv,
        cf.into_iter()
            .map(|q| Face::quad(q[0], q[1], q[2], q[3]))
            .collect(),
    )
    .expect("a fixtura e' construida aqui");
    assert_eq!(super::open_edges(&fechada), 0);
    assert!(
        super::open_edges(&fina) > 0,
        "⛔ a nuvem de quads soltos tem de ter furos"
    );
    assert!(
        super::worse(&fina, 2, 5.0, &fechada, 2, 5.0),
        "⛔ a candidata com FUROS perde, por melhor que seja a ponta dela"
    );
}

/// **DOZE QUADS COINCIDENTES** — a casca da ponta sai VAZIA.
///
/// ⚠️ **Todos os centróides no mesmo sítio** ⇒ a distância ao centro é `0` para todos, a casca
/// exterior fica sem uma face e [`ph2d_quadfill::tip_body_ratio`] devolve amostra `0`. ⭐ E as
/// contagens da frente batem certo com a [`nuvem_com_ponta`] (`48` arestas de bordo, `12`
/// peças, `0` gravatas): *sem isso, a chave da frente decidiria e o gate não mediria a guarda.*
fn quads_coincidentes() -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for _ in 0..12 {
        let b = u32::try_from(verts.len()).expect("a fixtura e' pequena");
        verts.push([0.0, 0.0, 0.0]);
        verts.push([1.0, 0.0, 0.0]);
        verts.push([1.0, 1.0, 0.0]);
        verts.push([0.0, 1.0, 0.0]);
        faces.push(Face::quad(b, b + 1, b + 2, b + 3));
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — uma amostra de ponta VAZIA não decide, e a assimétrica é a que PROVA.**
///
/// ⛔⛔ **A 1.ª redacção deste gate usava a MESMA malha dos dois lados**, e a mutação que apaga
/// a guarda `a_n > 0 && b_n > 0` **SOBREVIVEU**: com a amostra vazia dos dois lados as razões
/// valem `0,0` as duas, a comparação dá `Equal` e o código cai no enviesamento **de qualquer
/// maneira**. *Um gate que não distingue a guarda da ausência dela não é um gate.*
///
/// ⭐ **O par assimétrico é o que morde:** sem a guarda, a malha de amostra vazia lê `0,0` —
/// que ao lado de `0,5` é *o melhor resultado possível* — e ganha uma comparação que devia
/// perder. É a armadilha que o doc do [`ph2d_quadfill::tip_body_ratio`] nomeia: *um zero de
/// «não medido» e um de «perfeito» são o mesmo byte.*
#[test]
fn a_amostra_vazia_nao_ganha_de_uma_medida() {
    let vazia = quads_coincidentes();
    let medida = nuvem_com_ponta(0.5);
    // ⚠️ O CONTROLE: as chaves da frente TÊM de empatar.
    assert_eq!(super::open_edges(&vazia), super::open_edges(&medida));
    assert_eq!(super::components(&vazia), super::components(&medida));
    assert_eq!(super::bowties(&vazia), super::bowties(&medida));
    let (r_vazia, n_vazia) = super::tip_ratio(&vazia);
    let (_, n_medida) = super::tip_ratio(&medida);
    assert_eq!(n_vazia, 0, "⛔ a fixtura tem de ter a casca da ponta VAZIA");
    assert!(n_medida > 0, "⛔ e a outra tem de ter amostra");
    assert!(
        r_vazia <= 0.0,
        "⛔ a razao NAO MEDIDA e' `0,0` -- e' isso que a torna perigosa ({r_vazia:.3})"
    );
    assert!(
        super::worse(&vazia, 2, 9.0, &medida, 2, 8.0),
        "⛔ com a amostra vazia de um lado, quem decide e' o enviesamento -- a razao `0,0` de \
         «nao medido» NAO pode ganhar"
    );
}

/// ⭐⭐⭐ **GATE — uma amostra de ponta VAZIA não decide.**
///
/// ⛔⛔ *Um zero de «não medido» e um de «perfeito» são o mesmo byte* — é a armadilha que o
/// doc do [`ph2d_quadfill::tip_body_ratio`] nomeia, e o [`super::worse`] só a evita porque
/// pergunta pela **contagem** da amostra antes de comparar a razão.
#[test]
fn uma_amostra_de_ponta_vazia_nao_decide() {
    let so_um = um_quad(false);
    let (_, n_um) = super::tip_ratio(&so_um);
    assert_eq!(
        n_um, 0,
        "⛔ a fixtura tem de ter a casca da ponta VAZIA para este gate significar algo"
    );
    // Com a amostra vazia dos DOIS lados, quem decide tem de ser o enviesamento.
    assert!(
        super::worse(&so_um, 2, 9.0, &so_um, 2, 8.0),
        "⛔ com a amostra vazia, quem decide e' o enviesamento"
    );
    assert!(
        !super::worse(&so_um, 2, 8.0, &so_um, 2, 9.0),
        "⛔ e no sentido contrario tambem"
    );
}

/// ⭐⭐ **GATE — a ordem da chave nova, lida do FONTE.**
///
/// ⚠️ Irmão do [`a_ordem_das_chaves_e_furos_pecas_gravatas_forma`], e separado dele de
/// propósito: aquele defende a ordem que já existia, este a posição da chave nova. *Um gate
/// que afirma seis coisas de uma vez não diz qual delas partiu.*
#[test]
fn a_ponta_decide_depois_das_faces_ruins_e_antes_do_enviesamento() {
    let src = include_str!("sculpt3d_retopo_rulers.rs");
    let ini = src
        .find("pub(super) fn worse(")
        .expect("a funcao mudou de nome");
    let corpo = &src[ini..];
    let abre = corpo
        .find(") -> bool {")
        .expect("a assinatura de worse mudou");
    let corpo = &corpo[abre..];
    let fim = corpo.find("\n}").expect("o corpo de worse nao fecha");
    // ⚠️ **Descasque os comentários ANTES de procurar**, senão documentar a decisão reprova o
    // portão — é a lição de 2026-08-30, e esta é a segunda vez que ela aparece neste ficheiro.
    let corpo: String = corpo[..fim]
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let em = |agulha: &str| corpo.find(agulha).expect(agulha);
    assert!(
        em("a_over60") < em("a_tip"),
        "⛔ as faces com canto pior que 60 graus decidem ANTES da ponta"
    );
    assert!(
        em("a_tip") < em("a_skew"),
        "⛔ a ponta decide ANTES do enviesamento mediano -- e' a inversao inteira da wave"
    );
}
