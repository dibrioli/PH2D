//! ⭐⭐⭐⭐ **OS GATES DO QUARTO CANAL** — irmão (`#[path]`) do [`super`].
//!
//! ⚠️ **Eles correm SEM adaptador, e isso é a razão de a lei viver ali e não na
//! cena.** A prova de comportamento do `Ctrl+Z` inteiro é `#[ignore]` + placa
//! (`o_ctrl_z_desfaz_a_tinta_fina`, no `tinta_no_produto_tests.rs`), e essa é
//! a população que nem o arnês de mutação nem o CI correm. *O que ESTES
//! afirmam é a lei; o que aquele afirma é o elo até à tecla.*

use super::{IdDoPlano, JanelaFina, PlanoInteiro};
use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_colors::Tinta;
use ph2d_sculpt3d::tinta_fina::TintaDoTraco;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

const COR: [f32; 3] = [0.9, 0.2, 0.1];
const BRANCO: [f32; 3] = [1.0, 1.0, 1.0];

fn plano(mesh: &Mesh, nivel: u8) -> Tinta {
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    Tinta::nova(mesh.vert_count(), faces(), nivel)
}

/// ⭐ **Um traço pela PORTA do motor** — a mesma `SculptStroke::dab` que o
/// produto chama, sem cena e sem device.
fn traco(mesh: &mut Mesh, verb: Verb, nivel: u8) -> TintaDoTraco {
    let brush = Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        color: COR,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(mesh);
    s.tinta_fina = Some(TintaDoTraco::nova(plano(mesh, nivel), 0));
    for i in 0..2u8 {
        let c = [0.06 * f32::from(i), 0.0, 1.0];
        let dab = Dab {
            path: [0.06, 0.0, 0.0],
            ..Dab::at(c, 0.45, c)
        };
        s.dab(mesh, &brush, &dab, Symmetry::default());
    }
    s.tinta_fina
        .take()
        .expect("o plano foi emprestado ao traço")
}

fn pintadas(t: &Tinta) -> usize {
    t.amostras().iter().filter(|c| **c != BRANCO).count()
}

/// ⭐⭐⭐⭐ **GATE — A JANELA DE UM TRAÇO FINO NOMEIA AS AMOSTRAS QUE ELE
/// ESCREVEU, e um traço que não escreveu nenhuma não deixa janela.**
///
/// ⚠️ **O CONTROLO é a metade que a torna uma medição:** um verbo de FORMA no
/// mesmo arranjo empresta o plano, escreve **zero** amostras e devolve-o. Sem
/// ele, uma `do_traco` que devolvesse `Some` sempre passaria — e poria uma
/// entrada de canal em toda pincelada da peça.
#[test]
fn a_janela_de_um_traco_fino_nomeia_as_amostras_que_ele_escreveu() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let fina = traco(&mut m, Verb::Paint, 2);
    let n = pintadas(fina.tinta());
    assert!(n > 0, "a fixtura não contém o fenómeno — nada foi pintado");
    let janela = JanelaFina::do_traco(&fina).expect("o traço de COR escreveu amostras");
    assert_eq!(
        janela.len(),
        n,
        "a janela tem de nomear exactamente as amostras que mudaram de cor"
    );

    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let forma = traco(&mut m, Verb::Draw, 2);
    assert_eq!(
        pintadas(forma.tinta()),
        0,
        "CONTROLO: um verbo de FORMA não escreve uma amostra"
    );
    assert!(
        JanelaFina::do_traco(&forma).is_none(),
        "CONTROLO: sem amostra escrita não há janela — e uma janela vazia poria \
         uma entrada de canal em toda pincelada"
    );
}

/// ⭐⭐⭐⭐ **GATE — A TROCA É INVOLUTIVA: desfazer devolve o plano de antes, e
/// refazer devolve o de depois.**
///
/// ⛔ É o dente do modelo inteiro. Se ela devolvesse o que instalou, o desfazer
/// funcionaria e o refazer seria um no-op que **consome** a entrada — a forma
/// de *«o redo às vezes não faz nada»* que nenhuma contagem vê.
#[test]
fn a_troca_da_janela_fina_e_involutiva() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let fina = traco(&mut m, Verb::Paint, 2);
    let janela = JanelaFina::do_traco(&fina).expect("o traço escreveu amostras");
    let mut t = fina.entregar();
    let depois = t.amostras().to_vec();
    assert!(pintadas(&t) > 0, "a fixtura não contém o fenómeno");

    let inversa = janela.troca(Some(&mut t)).expect("é o MESMO plano");
    assert_eq!(pintadas(&t), 0, "o desfazer não devolveu o plano de antes");

    let outra = inversa.troca(Some(&mut t)).expect("continua a ser o mesmo");
    assert_eq!(
        t.amostras(),
        depois.as_slice(),
        "o refazer não devolveu o plano de depois, ao bit"
    );
    assert!(outra.troca(Some(&mut t)).is_some(), "e ela continua viva");
}

/// ⭐⭐⭐⭐ **GATE — UMA JANELA DE OUTRO PLANO É LARGADA, E NÃO TOCA NUM BIT.**
///
/// ⛔⛔ O endereço de uma amostra é `(face, sítio)`: escrever a cor de antes num
/// plano reconstruído põe a tinta de uma face na face vizinha, sem estourar e
/// sem desenhar lixo óbvio. *É o defeito que ninguém consegue atribuir.*
///
/// ⚠️ **As duas metades:** o plano que já não existe (o artista voltou ao modo
/// `Mesh`) e o plano que existe e é OUTRO. Cada uma sozinha mente — a primeira
/// lê-se como *«sem plano não há nada a fazer»* e a segunda como *«há plano,
/// logo escreve»*.
#[test]
fn uma_janela_de_outro_plano_e_largada_e_nao_toca_num_bit() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let fina = traco(&mut m, Verb::Paint, 2);
    let janela = JanelaFina::do_traco(&fina).expect("o traço escreveu amostras");

    // (a) sem plano nenhum.
    assert!(
        JanelaFina::do_traco(&fina)
            .expect("a mesma janela")
            .troca(None)
            .is_none(),
        "sem plano na peça a janela tem de ser LARGADA"
    );

    // (b) um plano de OUTRO degrau, byte a byte intocado.
    let mut outro = plano(&m, 3);
    let antes = outro.amostras().to_vec();
    assert!(
        janela.troca(Some(&mut outro)).is_none(),
        "um plano de outro degrau não é aquele em que a janela foi escrita"
    );
    assert_eq!(
        outro.amostras(),
        antes.as_slice(),
        "e a recusa não pode ter escrito um único bit"
    );

    // (c) CONTROLO: o MESMO degrau sobre a MESMA malha é aceite.
    let mut mesmo = plano(&m, 2);
    assert!(
        JanelaFina::do_traco(&fina)
            .expect("a mesma janela")
            .troca(Some(&mut mesmo))
            .is_some(),
        "CONTROLO: no plano certo ela tem de ser aplicada"
    );
}

/// ⭐⭐⭐⭐ **GATE — UMA JANELA CUJOS ÍNDICES NÃO CABEM É RECUSADA, E NÃO
/// ESTOURA.**
///
/// ⛔⛔ **É a SEGUNDA cerca da [`JanelaFina::troca`], e ela responde a outra
/// pergunta que a identidade:** a [`super::super::swap_window`] indexa **sem
/// cerca nenhuma**, e um `panic` num `Ctrl+Z` é o pior desfecho de um canal de
/// desfazer.
///
/// ⚠️⚠️ **A identidade do plano BATE de propósito** (`IdDoPlano::de` sobre o
/// próprio plano vivo): sem isso este gate mediria a cerca de cima e a de
/// baixo ficava sem régua. *Foi exactamente esse o defeito da 1.ª redacção — a
/// fixtura sobrescrevia o campo que estava a testar, e a mutação que o apagava
/// SOBREVIVEU.*
#[test]
fn uma_janela_cujos_indices_nao_cabem_e_recusada_e_nao_estoura() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let mut t = plano(&m, 1);
    let antes = t.amostras().to_vec();
    let fora = u32::try_from(t.amostras().len()).expect("cabe") + 7;
    let janela = JanelaFina {
        plano: IdDoPlano::de(&t),
        amostras: vec![fora],
        cores: vec![COR],
    };
    assert!(
        janela.troca(Some(&mut t)).is_none(),
        "a cerca dos ÍNDICES é o que impede o `panic` — a identidade bate"
    );
    assert_eq!(
        t.amostras(),
        antes.as_slice(),
        "e a recusa não pode ter escrito um único bit"
    );
}

/// ⭐⭐⭐⭐ **GATE — LARGAR O DEGRAU E RE-ARMAR O MESMO NÃO TIRA O `Ctrl+Z` AO
/// TRAÇO; DOIS degraus pelo meio tiram.**
///
/// ⚠️⚠️ **Ele mede a COMPOSIÇÃO, que é onde o produto vive e onde nenhum dos
/// irmãos olha:** o [`crate::tinta_da_peca::garante`] tem gates do lado do
/// parque, a [`JanelaFina::troca`] tem-nos do lado da identidade, e *uma lei
/// verificada nas duas pontas ainda pode ser contrariada no meio*.
///
/// ⛔⛔ **A 1.ª asserção é o DISCRIMINADOR e sem ela o gate mede o nada:** sem
/// parque, re-armar constrói um plano **BRANCO**, logo `pintadas == 0` já
/// **antes** do desfazer — e a metade *«o desfazer devolveu o plano de antes»*
/// passaria com a cura apagada. O que separa as duas árvores é o plano voltar
/// **PINTADO**.
///
/// ⚠️ **E a FRONTEIRA é `dois` e não `um`:** a ranhura é uma, logo ir e voltar
/// **directamente** devolve (o degrau de saída ocupa a ranhura que o de volta
/// acabou de esvaziar), e **dois** degraus pelo meio despejam-na. *Era isto
/// que o roteiro da `=52` dizia ao contrário.*
#[test]
fn largar_o_degrau_e_re_armar_o_mesmo_nao_tira_o_desfazer() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let fina = traco(&mut m, Verb::Paint, 2);
    let janela = JanelaFina::do_traco(&fina).expect("o traço escreveu amostras");
    let mut tinta = Some(fina.entregar());
    let depois = tinta.as_ref().expect("armado").amostras().to_vec();
    assert!(
        pintadas(tinta.as_ref().expect("armado")) > 0,
        "a fixtura não contém o fenómeno"
    );

    // (1) Largar a fileira para `Mesh` e voltar ao MESMO degrau.
    let mut parque = None;
    crate::tinta_da_peca::garante(&m, &mut tinta, &mut parque, None);
    assert!(
        tinta.is_none() && parque.is_some(),
        "o CONTROLO: ele foi mesmo parqueado"
    );
    crate::tinta_da_peca::garante(&m, &mut tinta, &mut parque, Some(2));

    let t = tinta.as_mut().expect("re-armado");
    assert!(
        pintadas(t) > 0,
        "o DISCRIMINADOR: o plano tem de voltar PINTADO — se ele volta branco \
         este gate mede o nada e a cura do parque não está lá"
    );
    let inversa = janela
        .troca(Some(t))
        .expect("é o MESMO plano, logo a janela aplica-se");
    assert_eq!(pintadas(t), 0, "o desfazer não devolveu o plano de antes");
    assert!(
        inversa.troca(Some(t)).is_some() && t.amostras() == depois.as_slice(),
        "e o refazer tem de o devolver ao bit"
    );

    // (2) A FRONTEIRA: DOIS degraus pelo meio despejam a ranhura.
    let mut m2 = shapes::uv_sphere(16, 24, 1.0);
    let fina2 = traco(&mut m2, Verb::Paint, 2);
    let mut tinta2 = Some(fina2.entregar());
    let mut parque2 = None;
    for k in [Some(1), Some(3), Some(2)] {
        crate::tinta_da_peca::garante(&m2, &mut tinta2, &mut parque2, k);
    }
    assert_eq!(
        pintadas(tinta2.as_ref().expect("re-armado")),
        0,
        "com dois degraus pelo meio a ranhura já foi ocupada — se isto passar, \
         alguém pôs lá mais do que um plano sem o dizer"
    );
}

/// ⭐⭐ **GATE — A TROCA DO PLANO INTEIRO É INVOLUTIVA, e a primeira volta
/// devolve o plano de antes AO BIT.** A entrada do `Fill` carrega o plano de
/// ANTES; aplicá-la instala-o e devolve o de DEPOIS, que é o que o refazer
/// instala.
///
/// ⚠️ O CONTROLO é o preenchimento ter mudado alguma coisa — sem ele, uma
/// troca que não fizesse nada passaria sobre dois planos iguais.
#[test]
fn a_troca_do_plano_inteiro_e_involutiva() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    let mut t = plano(&mesh, 2);
    let antes = t.amostras().to_vec();
    let entrada = PlanoInteiro::de(&t);
    assert!(ph2d_sculpt3d::preenche::preenche_plano(&mut t, &mesh, COR).expect("descreve"));
    let depois = t.amostras().to_vec();
    assert_ne!(antes, depois, "o controlo: o Fill mudou o plano");
    let inversa = entrada.troca(Some(&mut t)).expect("o mesmo plano");
    assert_eq!(
        t.amostras(),
        &antes[..],
        "desfazer devolve o plano de antes, ao bit"
    );
    let _ = inversa.troca(Some(&mut t)).expect("o mesmo plano");
    assert_eq!(
        t.amostras(),
        &depois[..],
        "refazer devolve o preenchido, ao bit"
    );
}

/// ⛔⛔ **Um plano inteiro de OUTRO plano é LARGADO e não toca num bit** — a
/// mesma cerca da janela, pela mesma razão: escrever as cores de antes num
/// plano de outro degrau é tinta válida no sítio errado.
#[test]
fn um_plano_inteiro_de_outro_degrau_e_largado() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    let entrada = PlanoInteiro::de(&plano(&mesh, 2));
    let mut outro = plano(&mesh, 3);
    let antes = outro.amostras().to_vec();
    assert!(entrada.troca(Some(&mut outro)).is_none());
    assert_eq!(outro.amostras(), &antes[..]);
    // E sem plano nenhum (o artista voltou a `Mesh`) também é largada.
    let entrada = PlanoInteiro::de(&plano(&mesh, 2));
    assert!(entrada.troca(None).is_none());
}

/// ⛔⛔ **GATE — O TECTO DA HISTÓRIA CONTA A JANELA FINA de um traço.**
///
/// O braço do traço no `footprint_bytes` terminava num `..`, que engolia o
/// campo `finas`: um traço fino punha na fila `16 B` por amostra tocada que a
/// poda não via. ⇒ a diferença entre a mesma entrada com e sem a janela é
/// **exactamente** o peso da janela.
#[test]
fn o_tecto_da_historia_conta_a_janela_fina() {
    let mut mesh = shapes::uv_sphere(12, 16, 1.0);
    let j = JanelaFina::do_traco(&traco(&mut mesh, Verb::Paint, 2)).expect("pintou amostras");
    let peso = j.bytes();
    assert!(peso > 0, "o controlo: a janela pesa");
    let entrada = |finas| crate::StrokeUndo::Stroke {
        level: 0,
        verts: Vec::new(),
        positions: Vec::new(),
        masks: None,
        colors: None,
        finas,
    };
    let com = entrada(Some(j)).footprint_bytes();
    let sem = entrada(None).footprint_bytes();
    assert_eq!(com - sem, peso);
}

/// ⭐ **E a entrada do `Fill` pesa os DOIS planos que carrega** — a `16x` o de
/// amostras são ~`300 MB`, e é o tecto em bytes que o poda.
#[test]
fn a_entrada_do_fill_pesa_os_dois_planos() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    let t = plano(&mesh, 2);
    let finas = PlanoInteiro::de(&t);
    let peso_finas = finas.bytes();
    assert_eq!(peso_finas, std::mem::size_of_val(t.amostras()));
    let cores = vec![COR; mesh.vert_count()];
    let peso_cores = cores.capacity() * size_of::<[f32; 3]>();
    let e = crate::StrokeUndo::Fill {
        level: 0,
        colors: Some(cores),
        finas: Some(finas),
    };
    assert_eq!(e.footprint_bytes(), peso_cores + peso_finas);
}
