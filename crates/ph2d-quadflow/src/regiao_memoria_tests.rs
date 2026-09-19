//! **OS GATES DA MEMÓRIA DO TRAÇO** — a fase da retícula a atravessar os
//! carimbos.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é por responsabilidade: ali mora *a
//! mancha é a peça inteira ao bit*, aqui *o que sobrevive entre duas manchas*.
//! ⚠️ Ele saiu de lá por **TECTO DE LOC** (`797` contra `700`).
//!
//! ```text
//! cargo test -p ph2d-quadflow memoria
//! ```

use super::super::{
    CampoDoTraco, Campos, arruma_na_grelha_lembrando, arruma_na_grelha_por, passo_da_pegada,
};
use super::{ITERACOES, TRACO, peca_densa, pegada};

/// ⭐⭐⭐⭐ **GATE — UMA MEMÓRIA VAZIA DEVOLVE A LEI DE SEMPRE, AO BIT.**
///
/// É o CONTROLO de toda a wave da memória: enquanto ela não tiver conteúdo, a
/// porta nova ([`arruma_na_grelha_lembrando`]) tem de ser **o mesmo
/// programa** que a porta velha. *Sem esta metade, a memória seria um segundo
/// produto disfarçado de alavanca, e nenhuma medição do pente feita antes desta
/// wave continuaria a valer.*
///
/// ⚠️ **E a metade POSITIVA é o que a torna honesta:** com a memória CHEIA o
/// barro tem de ser outro. Sem ela, um `arruma_na_grelha_lembrando` que
/// ignorasse o argumento passava.
#[test]
fn uma_memoria_vazia_devolve_a_lei_de_sempre() {
    let base = peca_densa();
    let (centro, raio) = ([0.0, 0.0, 1.0], 0.55);
    let um = |_p: [f32; 3]| 1.0f32;
    let classe = Campos::UmNivel {
        semente_unica: false,
    };

    let corre = |mem: Option<&mut CampoDoTraco>| {
        let mut mesh = base.clone();
        let mut movidos = Vec::new();
        arruma_na_grelha_lembrando(
            &mut mesh,
            centro,
            raio,
            TRACO,
            &um,
            ITERACOES,
            0.80,
            classe,
            mem,
            &mut movidos,
        );
        (mesh.positions().to_vec(), movidos.len())
    };

    let (sem_porta, n) = {
        let mut mesh = base.clone();
        let mut movidos = Vec::new();
        arruma_na_grelha_por(
            &mut mesh,
            centro,
            raio,
            TRACO,
            &um,
            ITERACOES,
            0.80,
            classe,
            &mut movidos,
        );
        (mesh.positions().to_vec(), movidos.len())
    };
    assert!(
        n > 0,
        "a fixtura nao contem o fenomeno: a lei nao moveu nada"
    );

    let mut vazia = CampoDoTraco::default();
    let (com_vazia, _) = corre(Some(&mut vazia));
    assert_eq!(
        sem_porta, com_vazia,
        "a porta com memoria VAZIA deu outro barro — ela deixou de ser a lei de sempre"
    );

    // A metade POSITIVA: a memória cheia (a do primeiro carimbo) muda o barro.
    let mut cheia = CampoDoTraco::default();
    let _ = corre(Some(&mut cheia));
    assert!(
        cheia.lembrados() > 0,
        "o primeiro carimbo nao guardou nada — a metade positiva nao pode afirmar"
    );
    let (com_cheia, _) = corre(Some(&mut cheia));
    assert_ne!(
        sem_porta, com_cheia,
        "a memoria CHEIA deu o mesmo barro que nao ter memoria — o argumento nao chega"
    );
}

/// ⭐⭐⭐⭐ **GATE — A MEMÓRIA SOBREVIVE ÀS DUAS MUDANÇAS DE TAMANHO DA MALHA.**
///
/// ⛔⛔ **O modo de falha da renumeração é MUDO, e é por isso que ela tem gate
/// próprio:** saltar o [`CampoDoTraco::encolheu`] não deixa a memória
/// incompleta — deixa-a a **MENTIR**. A âncora de um vértice morto passa a
/// descrever quem ocupou a casa dele, e a fase viaja para o sítio errado sem uma
/// linha vermelha em lado nenhum.
///
/// ⚠️ **Os produtores são os do PRODUTO** (`collapse_in_sphere_com` e
/// `refine_in_sphere_sized`), nunca um `Remap` escrito à mão: *um gate que
/// fabrica o plano de compactação afirma sobre a aritmética dele, não sobre a
/// que a malha usa.*
///
/// As três metades, e cada uma reprova por um motivo diferente:
///
/// 1. o tamanho acompanha a malha nos **dois** sentidos;
/// 2. **quem se mudou LEVA a âncora** — a metade que o `encolheu` existe para
///    ter;
/// 3. **um vértice novo HERDA a âncora de um pai**, nunca a média das duas (o
///    ponto médio de dois nós de uma retícula **não é um nó dela**, e semear com
///    ele poria o filho a meia célula de fase de toda a vizinhança — exactamente
///    o `0,375` que fez a hierarquia ser recusada em 20/09).
#[test]
fn a_memoria_sobrevive_as_duas_mudancas_de_tamanho() {
    let mut mesh = peca_densa();
    // ⚠️ **Os dois motores RECUSAM quads** (`Collapse::NotTriangles`), e a
    // [`peca_densa`] é uma esfera UV, que é de quads: sem esta linha a 1.ª
    // metade lia *«o colapso não cortou nada»* sobre um motor que nem correu.
    // É o que o pen-down do produto faz, e pela mesma razão.
    mesh.triangulate();
    // ⚠️⚠️ **A memória cobre a PEÇA INTEIRA aqui, e não uma pegada de pincel.**
    // A renumeração do colapso muda de casa os vértices de ÍNDICE ALTO, e numa
    // esfera UV esses vivem no pólo oposto ao da pegada ⇒ com um raio de
    // pincel, *nenhum vértice lembrado se mudava* e a 2.ª metade media o vazio.
    // A 1.ª redacção reprovou exactamente assim, e é essa a metade do gate.
    let (centro, raio) = ([0.0, 0.0, 1.0], 3.0);
    let um = |_p: [f32; 3]| 1.0f32;
    let mut mem = CampoDoTraco::default();
    let mut movidos = Vec::new();
    arruma_na_grelha_lembrando(
        &mut mesh,
        centro,
        raio,
        TRACO,
        &um,
        ITERACOES,
        0.80,
        Campos::UmNivel {
            semente_unica: false,
        },
        Some(&mut mem),
        &mut movidos,
    );
    let lembrados = mem.lembrados();
    assert!(
        lembrados > 100,
        "a fixtura nao lembrou o suficiente ({lembrados})"
    );

    // ── O COLAPSO ────────────────────────────────────────────────────────────
    let antes: Vec<Option<[f32; 3]>> = (0..mesh.vert_count() as u32)
        .map(|v| mem.ancora(v))
        .collect();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    // ⚠️ **O alvo sai da MALHA e não de um número escolhido** — com um literal a
    // fixtura pode não conter o fenómeno, que foi o que a 1.ª redacção fez.
    let alvo =
        ph2d_mesh::collapse_target(passo_da_pegada(&mesh, &pegada(&mesh, centro, raio)) * 1.6);
    let cortou = matches!(
        ph2d_mesh::collapse_in_sphere_com(
            &mut mesh,
            centro,
            raio,
            alvo,
            None,
            ph2d_mesh::Guarda::ETambemAForma,
            &mut remap,
            &mut region,
        ),
        ph2d_mesh::Collapse::Done { .. }
    );
    assert!(
        cortou,
        "a fixtura nao contem o fenomeno: o colapso nao cortou nada"
    );
    assert!(
        !remap.vert_moves.is_empty(),
        "o colapso nao renumerou ninguem — a metade 2 nao pode afirmar"
    );
    mem.encolheu(&remap);
    assert_eq!(
        mem.len(),
        mesh.vert_count(),
        "a memoria nao encolheu com a malha"
    );
    // ⛔⛔⛔ **A TRADUÇÃO DE UM COLAPSO É UMA CADEIA, e a 1.ª redacção deste
    // gate escreveu-a plana** — foi ele que a apanhou, com `a ancora de 6049 nao
    // seguiu para 5958` sobre uma implementação CERTA. O plano é uma
    // **sequência**: `(11→10)`, `(10→9)` quer dizer que quem começou em `11`
    // acaba em `9`, passando por um endereço que não é dele. É a mesma lei que o
    // §27 desta linha já tinha pago na pegada congelada do polegar.
    //
    // ⇒ a conferência aplica os movimentos **pela mesma ordem** que a memória
    // aplica, e só depois pergunta onde cada um foi parar.
    let mut origem: Vec<Option<usize>> = (0..antes.len()).map(Some).collect();
    for &(de, para) in &remap.vert_moves {
        origem[para as usize] = origem[de as usize];
        origem[de as usize] = None;
    }
    let mut conferidos = 0usize;
    for (j, veio_de) in origem.iter().enumerate().take(remap.verts) {
        let Some(i) = *veio_de else { continue };
        if i == j {
            continue;
        }
        if let Some(a) = antes[i] {
            assert_eq!(
                mem.ancora(u32::try_from(j).expect("cabe")),
                Some(a),
                "a ancora do vertice que comecou em {i} nao seguiu ate' {j}"
            );
            conferidos += 1;
        }
    }
    assert!(
        conferidos > 0,
        "nenhum vertice LEMBRADO se mudou — a metade 2 mediu o vazio"
    );

    // ── O REFINO ─────────────────────────────────────────────────────────────
    let mut births = Vec::new();
    let _ = ph2d_mesh::refine_in_sphere_sized(
        &mut mesh,
        centro,
        raio,
        alvo * 0.25,
        None,
        &mut births,
        &mut region,
    );
    assert!(!births.is_empty(), "o refino nao criou ninguem");
    let com_pai_lembrado = births
        .iter()
        .filter(|b| mem.ancora(b.a).is_some() || mem.ancora(b.b).is_some())
        .count();
    assert!(
        com_pai_lembrado > 0,
        "nenhum vertice novo tem pai lembrado — a metade 3 mediria o vazio"
    );
    mem.cresceu(&mesh, &births);
    assert_eq!(
        mem.len(),
        mesh.vert_count(),
        "a memoria nao cresceu com a malha"
    );
    for b in &births {
        let herdavel = mem.ancora(b.a).or_else(|| mem.ancora(b.b));
        match herdavel {
            Some(a) => assert_eq!(
                mem.ancora(b.vert),
                Some(a),
                "o vertice novo {} nao herdou a ancora de um pai",
                b.vert
            ),
            None => assert_eq!(
                mem.ancora(b.vert),
                None,
                "o vertice novo {} inventou uma ancora sem pai lembrado",
                b.vert
            ),
        }
    }
}
