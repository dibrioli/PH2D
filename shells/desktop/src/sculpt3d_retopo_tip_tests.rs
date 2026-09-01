//! ⭐⭐⭐ **OS GATES DA CHAVE DA PONTA** — irmão de [`super::tests`] pelo teto de LOC
//! da shell (HR-18, 600), cortado por RESPONSABILIDADE: aquele ficheiro defende as chaves
//! de **defeito** do [`super::worse`] (furos · peças · gravatas), este a chave de
//! **cobertura** que entrou em 2026-08-30 e a fronteira entre as duas.
//!
//! ⛔⛔⛔ **O report que os exige** (Enio, 28 e 29/08): *«as pontas finas perdem detalhe»*.
//! A cadeia **já produzia** a candidata que o cura e o desempate deitava-a fora.

use ph2d_mesh::{Face, Mesh};

use super::tests::{cubo, um_quad};

/// **UMA MEDIÇÃO DE PONTA VAZIA** — `tips = 0`, que a chave lê como *«não medido»* e não
/// como *«perfeito»*. ⚠️ É o valor com que os gates das OUTRAS chaves entram: uma fixtura
/// que trouxesse uma contagem de pontas passaria a testar a chave nova por acidente.
fn sem() -> ph2d_quadfill::TipDeviation {
    ph2d_quadfill::TipDeviation::default()
}

/// **UMA MEDIÇÃO DE DENSIDADE DA PONTA VAZIA** — a irmã da [`sem`] para a 5.ª chave
/// (2026-09-01). ⚠️ Pela mesma razão: `tips = 0` é *«não medido»*, logo a chave da grade na
/// ponta não decide, e os gates das outras chaves continuam a medir o que sempre mediram.
fn sem_den() -> ph2d_quadfill::TipDensity {
    ph2d_quadfill::TipDensity::default()
}

/// **UMA MEDIÇÃO DE PONTA COM `n` DE `total` ACIMA DA BARRA.**
fn pontas(acima: usize, total: usize) -> ph2d_quadfill::TipDeviation {
    ph2d_quadfill::TipDeviation {
        over: acima,
        tips: total,
        ..ph2d_quadfill::TipDeviation::default()
    }
}

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
        super::super::decide::worse(
            &grossa,
            2,
            5.0,
            sem(),
            sem_den(),
            &fina,
            2,
            5.0,
            sem(),
            sem_den()
        ),
        "⛔ a candidata de ponta GROSSA tem de perder"
    );
    assert!(
        !super::super::decide::worse(
            &fina,
            2,
            5.0,
            sem(),
            sem_den(),
            &grossa,
            2,
            5.0,
            sem(),
            sem_den()
        ),
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
        super::super::decide::worse(
            &fina,
            2,
            5.0,
            sem(),
            sem_den(),
            &fechada,
            2,
            5.0,
            sem(),
            sem_den()
        ),
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
        super::super::decide::worse(
            &vazia,
            2,
            9.0,
            sem(),
            sem_den(),
            &medida,
            2,
            8.0,
            sem(),
            sem_den()
        ),
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
        super::super::decide::worse(
            &so_um,
            2,
            9.0,
            sem(),
            sem_den(),
            &so_um,
            2,
            8.0,
            sem(),
            sem_den()
        ),
        "⛔ com a amostra vazia, quem decide e' o enviesamento"
    );
    assert!(
        !super::super::decide::worse(
            &so_um,
            2,
            8.0,
            sem(),
            sem_den(),
            &so_um,
            2,
            9.0,
            sem(),
            sem_den()
        ),
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
    // ⚠️ **O ficheiro medido mudou em 2026-09-01:** a ESCOLHA passou para o irmão
    // [`super::super::decide`] quando o tecto de LOC do `rulers` estourou (`614`). *Um gate
    // que lê o fonte segue o fonte — e é por isso que ele reprova no corte em vez de ficar
    // mudo a medir um ficheiro que já não tem a função.*
    let src = include_str!("sculpt3d_retopo_decide.rs");
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

/// **DUAS NUVENS IGUAIS, uma com a ponta ENCURTADA** — a fixtura da amputação.
///
/// ⚠️ **Tudo o resto empata por construção** (mesma contagem de quads soltos ⇒ mesmos furos,
/// mesmas peças, zero gravatas): *se as chaves da frente diferissem, o gate mediria uma delas.*
fn nuvem_com_ponta_a(distancia: f32) -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    let quad = |cx: f32, verts: &mut Vec<[f32; 3]>, faces: &mut Vec<Face>| {
        let b = u32::try_from(verts.len()).expect("a fixtura e' pequena");
        verts.push([cx, 0.0, 0.0]);
        verts.push([cx + 1.0, 0.0, 0.0]);
        verts.push([cx + 1.0, 1.0, 0.0]);
        verts.push([cx, 1.0, 0.0]);
        faces.push(Face::quad(b, b + 1, b + 2, b + 3));
    };
    for k in 0..8 {
        #[expect(clippy::cast_precision_loss, reason = "k <= 8 nesta fixtura")]
        let cx = k as f32 * 2.0;
        quad(cx, &mut verts, &mut faces);
    }
    for k in 0..4 {
        #[expect(clippy::cast_precision_loss, reason = "k <= 4 nesta fixtura")]
        let cx = distancia + k as f32 * 2.0;
        quad(cx, &mut verts, &mut faces);
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — a candidata com MAIS pontas partidas perde, mesmo com a forma melhor.**
///
/// ⛔⛔⛔ **Medido em 2026-08-31 e é a razão desta chave:** numa varredura do teto de graduação
/// da fase zero, a célula `ADAPT_RATIO = 8` entregou uma fase zero **perfeita** (`0` de `4`
/// pontas cortadas) e a **saída** cortou a ponta mais longa em **`−43 %`**. As duas candidatas
/// estavam limpas na topologia e o `worse` escolheu a que comia o espinho, *porque nada aqui
/// olhava para as pontas*.
///
/// ⚠️ **A régua mudou no mesmo dia, e a nova CONTA:** a primeira redacção comparava o
/// **alcance** das duas candidatas — um extremo global, e ainda por cima tirado do centroide
/// por vértice, que mede a amostragem. Medido na `_base_sculpt` a `Detail 0,40`, onde as duas
/// primeiras candidatas empatam em bordo: o alcance escolhia a de `2` pontas acima da barra
/// contra a de `1`. *Ele defendia a ponta que sobrevivia nas duas.*
#[test]
fn a_candidata_com_mais_pontas_partidas_perde_mesmo_com_a_forma_melhor() {
    let inteira = nuvem_com_ponta_a(100.0);
    let cortada = nuvem_com_ponta_a(60.0);
    // ⚠️ **O CONTROLE:** as chaves da frente TÊM de empatar.
    assert_eq!(super::open_edges(&inteira), super::open_edges(&cortada));
    assert_eq!(super::components(&inteira), super::components(&cortada));
    assert_eq!(super::bowties(&inteira), super::bowties(&cortada));
    // A forma da partida é dada PERFEITA e a da inteira PÉSSIMA: só a contagem pode decidir.
    assert!(
        super::super::decide::worse(
            &cortada,
            0,
            0.0,
            pontas(2, 4),
            sem_den(),
            &inteira,
            999,
            89.0,
            pontas(0, 4),
            sem_den()
        ),
        "⛔ a candidata com mais pontas partidas tem de perder -- e' o espinho que o dono fotografou"
    );
    assert!(
        !super::super::decide::worse(
            &inteira,
            999,
            89.0,
            pontas(0, 4),
            sem_den(),
            &cortada,
            0,
            0.0,
            pontas(2, 4),
            sem_den()
        ),
        "⛔ e a relacao tem de ser ANTI-SIMETRICA"
    );
}

/// ⭐⭐ **GATE — um EMPATE na contagem não decide, e a chave seguinte é que fala.**
///
/// ⛔ Duas candidatas com o **mesmo número** de pontas acima da barra não se distinguem por
/// esta chave — nem quando os desvios medianos diferem. *A grandeza que ela decide é discreta
/// de propósito: um `p50` contínuo competiria com o enviesamento em toda peça, incluindo as
/// que não têm ponta nenhuma partida.*
#[test]
fn um_empate_na_contagem_de_pontas_nao_decide() {
    let a = nuvem_com_ponta_a(100.0);
    let b = nuvem_com_ponta_a(99.5);
    assert!(
        super::super::decide::worse(
            &a,
            9,
            0.0,
            pontas(1, 4),
            sem_den(),
            &b,
            2,
            0.0,
            pontas(1, 4),
            sem_den()
        ),
        "⛔ com a contagem empatada quem decide e' a chave seguinte (as faces `>60°`)"
    );
    assert!(
        !super::super::decide::worse(
            &b,
            2,
            0.0,
            pontas(1, 4),
            sem_den(),
            &a,
            9,
            0.0,
            pontas(1, 4),
            sem_den()
        ),
        "⛔ e no sentido contrario tambem"
    );
}

/// ⭐⭐ **GATE — uma medição de ponta VAZIA não decide.**
///
/// ⛔⛔ `tips = 0` é *«não medido»*, e `over = 0` lê-se igual a *«nenhuma ponta partida»*.
/// *São o mesmo byte, e sem esta guarda uma candidata que a régua não conseguiu medir ganhava
/// de uma medida e partida — que é o contrário do que a chave existe para fazer.*
#[test]
fn uma_medicao_de_ponta_vazia_nao_decide_a_amputacao() {
    let a = nuvem_com_ponta_a(100.0);
    assert!(
        super::super::decide::worse(
            &a,
            9,
            0.0,
            sem(),
            sem_den(),
            &a,
            2,
            0.0,
            pontas(3, 4),
            sem_den()
        ),
        "⛔ uma medicao vazia nao pode GANHAR de uma medida e partida"
    );
    assert!(
        super::super::decide::worse(
            &a,
            9,
            0.0,
            pontas(3, 4),
            sem_den(),
            &a,
            2,
            0.0,
            sem(),
            sem_den()
        ),
        "⛔ nem PERDER dela -- em ambos os sentidos decide a chave seguinte"
    );
}

/// ⭐⭐ **GATE — os FUROS continuam a ganhar das pontas partidas.**
#[test]
fn as_pontas_partidas_nunca_ganham_de_um_furo() {
    let cortada_fechada = {
        let (cv, cf) = cubo(0.0);
        Mesh::from_parts(
            cv,
            cf.into_iter()
                .map(|q| Face::quad(q[0], q[1], q[2], q[3]))
                .collect(),
        )
        .expect("a fixtura e' construida aqui")
    };
    let inteira_furada = nuvem_com_ponta_a(100.0);
    assert_eq!(super::open_edges(&cortada_fechada), 0);
    assert!(super::open_edges(&inteira_furada) > 0);
    assert!(
        super::super::decide::worse(
            &inteira_furada,
            0,
            0.0,
            pontas(0, 4),
            sem_den(),
            &cortada_fechada,
            999,
            89.0,
            pontas(4, 4),
            sem_den()
        ),
        "⛔ o FURO decide antes das pontas -- foi a queixa mais antiga do dono"
    );
}
