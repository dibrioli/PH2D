//! Os gates de [`super::untangle_bowties`].

use ph2d_mesh::{Face, Mesh};

/// Uma grade de quads sobre um plano — a superfície é ela própria.
fn grade(n: usize) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            #[allow(clippy::cast_precision_loss)]
            let (x, y) = (i as f32 / n as f32, j as f32 / n as f32);
            pos.push([x - 0.5, y - 0.5, 0.0]);
        }
    }
    let mut faces: Vec<Face> = Vec::new();
    #[allow(clippy::cast_possible_truncation)]
    let w = (n + 1) as u32;
    #[allow(clippy::cast_possible_truncation)]
    let nn = n as u32;
    for j in 0..nn {
        for i in 0..nn {
            faces.push(Face::quad(
                j * w + i,
                j * w + i + 1,
                (j + 1) * w + i + 1,
                (j + 1) * w + i,
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("a grade e' valida")
}

/// ⭐ **A gravata sintética:** um vértice interior atirado para o outro lado da diagonal, que é
/// exactamente o que torna dois quads auto-cruzados sem mexer na topologia.
fn grade_com_gravata(n: usize, empurrao: f32) -> (Mesh, Mesh) {
    let plano = grade(n);
    let mut m = plano.clone();
    let alvo = {
        let w = n + 1;
        (n / 2) * w + n / 2
    };
    // ⚠️ **Passar POR CIMA do vizinho, e não pela diagonal:** um vértice atirado ao longo da
    // diagonal deixa o quad **concavo** (ou inteiramente invertido, que a lei lê como
    // `Convex` porque a normal de Newell vira com ele). O que cruza dois lados é o vértice
    // passar para lá do vizinho — medido a construir esta fixtura.
    {
        let pos = m.positions_mut();
        let p = pos[alvo];
        pos[alvo] = [p[0] + empurrao, p[1] + empurrao * 0.35, p[2]];
    }
    m.rebuild();
    (m, plano)
}

#[test]
fn sem_gravata_a_malha_fica_igual_ao_bit() {
    let plano = grade(8);
    let mut m = plano.clone();
    let n = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    assert_eq!(n, 0, "nao havia gravatas para desfazer");
    assert_eq!(m.positions(), plano.positions(), "a malha limpa foi mexida");
    assert_eq!(m.face_count(), plano.face_count());
}

#[test]
fn a_gravata_desfaz_se_e_a_topologia_nao_muda() {
    let (mut m, plano) = grade_com_gravata(8, 0.14);
    let antes = crate::local_shape(&m).0.bowties;
    assert!(antes > 0, "a fixtura tem de ter gravata: {antes}");
    let (v, f) = (m.vert_count(), m.face_count());
    let curadas = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    let depois = crate::local_shape(&m).0.bowties;
    assert_eq!(
        curadas,
        antes - depois,
        "o numero devolvido tem de ser o que desapareceu: {antes} -> {depois}"
    );
    assert_eq!(depois, 0, "sobraram gravatas: {antes} -> {depois}");
    assert_eq!(
        (m.vert_count(), m.face_count()),
        (v, f),
        "a topologia mudou"
    );
}

/// ⛔ **A cerca de viagem MANDA** — com ela a zero-e-tal o vértice não pode voltar ao sítio, e a
/// porta tem de **repor** a malha em vez de entregar um alisamento a meio.
#[test]
fn uma_cerca_apertada_repoe_a_malha_em_vez_de_entregar_meio_caminho() {
    let (mut m, plano) = grade_com_gravata(8, 0.14);
    let antes = m.positions().to_vec();
    let curadas = super::untangle_bowties(&mut m, &plano, 1.0e-4);
    assert_eq!(curadas, 0, "com a cerca fechada nada se cura");
    assert_eq!(m.positions(), &antes[..], "a malha nao voltou ao que era");
}

/// ⭐⭐⭐ **UM GRUPO DE DOBRAS desfaz-se** — a fenda que o dono fotografou em 2026-09-03 era
/// exactamente isto: faces do avesso em relação à vizinhança, com `0` gravatas e a topologia
/// impecável.
///
/// ⚠️ A fixtura é o vértice atirado ao longo da DIAGONAL: ele inverte os quads inteiros (a lei
/// da gravata lê-os como `Convex`, porque a normal de Newell vira com a face) e é a metade da
/// família que o censo de gravatas **não** vê.
#[test]
fn um_grupo_de_dobras_desfaz_se() {
    let plano = grade(8);
    let mut m = plano.clone();
    // ⚠️ **DOIS vértices vizinhos**, e não um: um vértice sozinho inverte **uma** face, e o que
    // esta lei repara é o GRUPO (a dobra isolada é vinco, e tem gate próprio).
    let a = (8 / 2) * 9 + 8 / 2;
    let b = a + 1;
    {
        let pos = m.positions_mut();
        for i in [a, b] {
            let p = pos[i];
            pos[i] = [p[0] + 0.30, p[1] + 0.30, p[2]];
        }
    }
    m.rebuild();
    let dobradas = crate::quality::folded_faces_by_neighbours(&m).len();
    assert!(
        dobradas >= 2,
        "a fixtura tem de ter um GRUPO de dobras: {dobradas}"
    );
    assert_eq!(
        crate::local_shape(&m).0.bowties,
        0,
        "e' a outra metade da familia: sem gravatas"
    );
    let (v, f) = (m.vert_count(), m.face_count());
    let curadas = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    assert!(curadas > 0, "nao curou nada");
    assert_eq!(
        crate::quality::folded_faces_by_neighbours(&m).len(),
        0,
        "sobraram dobras"
    );
    assert_eq!(
        (m.vert_count(), m.face_count()),
        (v, f),
        "a topologia mudou"
    );
}

/// ⛔⛔ **UMA DOBRA ISOLADA FICA** — ela é um vinco real da escultura, e alisá-la seria apagar
/// forma que o artista fez.
///
/// ⚠️ **A calibração é do lado APROVADO:** a retopologia que o dono aceitou tem `3` dobras com
/// maior grupo `1`; a que ele fotografou tem `5` num grupo só.
#[test]
fn uma_dobra_isolada_nao_se_toca() {
    let plano = grade(8);
    let mut m = plano.clone();
    // Inverter a ORDEM de uma face: ela passa a apontar contra as vizinhas, sozinha.
    {
        let faces = m.faces().to_vec();
        let mut novas: Vec<Face> = faces
            .iter()
            .map(|f| {
                let v = f.verts();
                Face::quad(v[0], v[1], v[2], v[3])
            })
            .collect();
        let v = faces[30].verts().to_vec();
        novas[30] = Face::quad(v[3], v[2], v[1], v[0]);
        m = Mesh::from_parts(m.positions().to_vec(), novas).expect("a grade continua valida");
    }
    let antes = m.positions().to_vec();
    let dobradas = crate::quality::folded_faces_by_neighbours(&m).len();
    assert_eq!(dobradas, 1, "a fixtura tem UMA dobra isolada: {dobradas}");
    let curadas = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    assert_eq!(curadas, 0, "uma dobra isolada nao se repara");
    assert_eq!(m.positions(), &antes[..], "a malha foi mexida");
}

/// ⭐⭐⭐ **UM REPARO QUE NÃO CEDE NÃO LEVA CONSIGO O QUE CEDEU** — o defeito que a 1.ª versão
/// desta porta tinha, e que só a peça do dono revelou.
///
/// ⛔⛔ Ela relaxava **todos** os vértices acusados de uma vez e repunha tudo se o censo não
/// descesse: quando o report de 03/09 trouxe as dobras, o grupo teimoso apagava também a gravata
/// já curada, e a saída do botão regrediu de `0/5` para `3/5` pontas amputadas.
///
/// ⚠️ **O que torna um grupo teimoso, de forma determinista, é a CERCA**: a gravata precisa de
/// `≈ 1,1` arestas de viagem e a dobra desta fixtura de `≈ 6,8`; com a cerca do produto
/// (`UNTANGLE_TRAVEL = 2`) só uma das duas é alcançável. ⛔ *Sem isto o gate passa por acidente* — a 1.ª fixtura punha a dobra na
/// borda a contar com a vizinhança curta, e ela cedia na mesma.
#[test]
fn um_grupo_teimoso_nao_apaga_a_cura_do_outro() {
    let plano = grade(8);
    let mut m = plano.clone();
    {
        let pos = m.positions_mut();
        // a gravata, no meio: o vértice passa POR CIMA do vizinho
        let a = 4 * 9 + 4;
        let p = pos[a];
        pos[a] = [p[0] + 0.14, p[1] + 0.14 * 0.35, p[2]];
        // a dobra, na BORDA: dois vértices vizinhos da fileira de baixo, na diagonal
        for i in [2usize, 3] {
            let q = pos[i];
            pos[i] = [q[0] + 0.60, q[1] + 0.60, q[2]];
        }
    }
    m.rebuild();
    let gravatas = crate::local_shape(&m).0.bowties;
    let dobras = crate::quality::folded_faces_by_neighbours(&m).len();
    assert!(
        gravatas > 0 && dobras > 0,
        "a fixtura tem as DUAS: {gravatas} gravata(s), {dobras} dobra(s)"
    );
    let curadas = super::untangle_bowties(&mut m, &plano, super::UNTANGLE_TRAVEL);
    assert!(curadas > 0, "nao curou nada");
    assert_eq!(
        crate::local_shape(&m).0.bowties,
        0,
        "⛔ a GRAVATA tinha de ser curada mesmo que a dobra nao ceda"
    );
    // ⚠️ **E o gate tem de EXERCITAR o caso:** se a dobra da borda tambem cedesse, este teste
    // passaria sem nunca ter havido um grupo teimoso — *um gate que nao vive o caso que nomeia
    // e' verde por acidente.*
    assert!(
        !crate::quality::folded_faces_by_neighbours(&m).is_empty(),
        "a dobra cedeu: a fixtura deixou de exercitar o acoplamento"
    );
}

/// ⭐⭐⭐ **UMA ABA APAGA-SE e o buraco fecha** — a cura topológica do que a relaxação não pode
/// curar (o 3.º report do dono, 2026-09-03).
///
/// ⚠️ A fixtura DOBRA uma língua de faces para trás: dois vértices interiores atirados para lá
/// dos vizinhos ao longo da diagonal, que é o que inverte quads inteiros.
#[test]
fn uma_aba_apaga_se_e_o_buraco_fecha() {
    let plano = grade(8);
    let mut m = plano.clone();
    {
        let pos = m.positions_mut();
        for i in [4 * 9 + 4, 4 * 9 + 5] {
            let p = pos[i];
            pos[i] = [p[0] + 0.60, p[1] + 0.60, p[2]];
        }
    }
    m.rebuild();
    let avesso = crate::quality::folded_faces_by_neighbours(&m).len();
    assert!(avesso >= 2, "a fixtura tem de ter uma ABA: {avesso}");
    let (v0, f0) = (m.vert_count(), m.face_count());
    let abas = super::remove_flaps(&mut m, &plano);
    assert!(abas > 0, "nenhuma aba foi apagada");
    assert_eq!(
        crate::quality::folded_faces_by_neighbours(&m).len(),
        0,
        "sobraram faces do avesso"
    );
    // ⛔ **O buraco FECHOU** — e a conta diz que foi um leque: menos faces do grupo, mais L/2.
    assert!(
        m.face_count() < f0 + 4 && m.vert_count() <= v0 + 4,
        "o remendo cresceu de mais: {f0} -> {} faces, {v0} -> {} verts",
        m.face_count(),
        m.vert_count()
    );
    let mut bordo = std::collections::BTreeMap::new();
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *bordo.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
        }
    }
    let abertas = bordo.values().filter(|c| **c != 2).count();
    let borda_da_grade = 32; // as arestas do contorno do plano
    assert_eq!(
        abertas, borda_da_grade,
        "o remendo abriu (ou fechou) arestas que nao devia"
    );
}

/// ⛔ **Uma malha SÃ não se toca** — a porta é a identidade ao bit onde não há aba.
#[test]
fn sem_aba_a_malha_fica_igual_ao_bit() {
    let plano = grade(8);
    let mut m = plano.clone();
    let abas = super::remove_flaps(&mut m, &plano);
    assert_eq!(abas, 0, "nao havia aba nenhuma");
    assert_eq!(m.positions(), plano.positions(), "a malha sa' foi mexida");
    assert_eq!(m.face_count(), plano.face_count());
}

/// ⭐⭐⭐ **AS TRÊS SAÍDAS DO ACABAMENTO CURAM** — o gate do defeito que custou uma corrida
/// inteira em 2026-09-03.
///
/// ⛔⛔ A cura estava em **duas** das três saídas de
/// [`crate::finish_extracted_travel`], e a peça do dono saía pela terceira: a malha entregue
/// tinha as `6` faces do avesso intactas, e a medição leu *«não melhorou»* sobre código que
/// nunca correu. ⚠️ *Uma cura que vive em algumas das saídas é uma cura que o produto às vezes
/// não corre* — é a mesma família do `CLAUDE.md` §5.0.
///
/// ⚠️ **Ele lê o FONTE** porque as três saídas são condições de topologia que uma fixtura não
/// escolhe: o que se defende aqui é que **nenhuma delas fica sem a linha**.
#[test]
fn as_tres_saidas_do_acabamento_desfazem_o_avesso() {
    let src = include_str!("finish_extract.rs");
    let corpo = src
        .split_once("pub fn finish_extracted_travel")
        .expect("a funcao existe")
        .1;
    let corpo = corpo
        .split_once("\npub(crate) fn ")
        .map_or(corpo, |(antes, _)| antes);
    let retornos = corpo.matches("return ").count() + 1; // os `return` mais a queda no fim
    // ⚠️ **A CHAMADA, e não o nome:** os doc-comments citam as duas portas, e contar o nome
    // fazia o gate ler `4` de `3` — *um censo de fonte tem de saber a forma do que conta*.
    let curas = corpo.matches("remove_flaps(mesh").count();
    assert_eq!(
        curas, retornos,
        "⛔ {retornos} saida(s) e apenas {curas} com a cura -- a peca do dono saiu pela que faltava"
    );
    assert_eq!(
        corpo.matches("untangle_bowties(mesh").count(),
        retornos,
        "⛔ a relaxacao tambem tem de estar nas tres"
    );
}

/// ⛔⛔ **O RELATÓRIO CONTA A MALHA QUE SE ENTREGA** — o gate do estouro de 2026-09-03.
///
/// Enquanto o acabamento não mexia no número de faces, contar os quads pelo relatório da
/// **extracção** e os não-quads por subtração dava o mesmo. O apagador de abas passou a remover
/// um disco e a pôr um leque no lugar (`21 928 → 21 914`), e a subtração **estourou**: o botão
/// imprimiu `18 446 744 073 709 551 602` não-quads.
///
/// ⚠️ *Um relatório que descreve outra malha que não a entregue está errado mesmo quando o
/// número não estoura* — este gate lê o fonte do shell e exige que as duas contagens venham da
/// saída.
#[test]
fn o_relatorio_conta_a_malha_entregue() {
    let src = include_str!("../../../shells/desktop/src/sculpt3d_history_retopo_extract.rs");
    assert!(
        !src.contains("non_quads: out.face_count() - e.quads"),
        "⛔ a subtração que estourou voltou"
    );
    assert!(
        src.contains("non_quads: out.faces().iter().filter(|f| f.verts().len() != 4).count()"),
        "⛔ os nao-quads tem de ser CONTADOS na malha entregue"
    );
}
