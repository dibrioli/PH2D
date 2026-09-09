//! ⭐⭐⭐ **O FILTRO DE TECIDO — os portões do GESTO**, irmão do [`cloth_tests`].
//!
//! O corte entre os dois é o que cada um afirma: lá *a LEI do pano sob um
//! carimbo*, aqui *o que muda quando o pano deixa de ter pincel* — a peça
//! inteira, o ponto congelado, e a diferença de gesto que separa este filtro do
//! [`super::stroke_filter`].
//!
//! ⚠️ **Nenhum destes gates tem lado APROVADO, e isso é uma escolha e não uma
//! falta:** o corpus do filtro EXISTE desde 2026-09-07
//! (`fixtures/cloth/filtro/`, `17` corridas) e quem o mede é o
//! `ph2d-cloth/tests/oraculo_do_filtro.rs` — **a lei**. O que se afirma aqui é a
//! nossa COSTURA (a peça inteira, o ponto congelado, o gesto que acumula), que o
//! oráculo não observa porque ela não é lei nenhuma dele. *Uma barra calibrada
//! sem o lado aprovado mediria os nossos próprios defeitos — e é por isso que
//! nenhum destes gates põe uma.*

use super::cloth_tests::plano;
use super::{ClothFilterStep, SculptStroke};
use crate::ClothFilterKind;
use crate::ClothFilterProps;

fn passo(s: f32) -> ClothFilterStep {
    ClothFilterStep {
        s,
        gravity_axis: [0.0, 0.0, -1.0],
        frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        axes: [true; 3],
        eye: [0.0, 0.0, 1.0],
    }
}

/// Corre `n` passos do filtro e devolve a malha resultante.
fn corre(kind: ClothFilterKind, s: f32, n: usize, ponto: [f32; 3]) -> Vec<[f32; 3]> {
    let mut mesh = plano();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&mesh, ClothFilterProps::default(), kind, ponto);
    for _ in 0..n {
        st.cloth_filter_step(&mut mesh, kind, &passo(s));
    }
    mesh.positions().to_vec()
}

fn desvio(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐ **UM FILTRO TOCA A PEÇA INTEIRA; UM CARIMBO TOCA UM DISCO.**
///
/// É a afirmação que faz do filtro uma ferramenta diferente e não um pincel
/// grande: a espec §7 diz *todas as células não mascaradas, raio infinito, sem
/// banda*, e o observável disso é a contagem de vértices que se movem.
#[test]
fn um_filtro_toca_a_peca_inteira_e_um_carimbo_toca_um_disco() {
    let mut mesh = plano();
    let n = mesh.vert_count();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(
        &mesh,
        ClothFilterProps::default(),
        ClothFilterKind::Gravity,
        [0.0; 3],
    );
    let movidos = st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo(1.0));
    println!("filtro: {movidos} de {n} vertices movidos");
    assert!(
        movidos > 0,
        "o filtro nao moveu nada -- a fixtura nao produz o fenomeno"
    );
    assert_eq!(
        movidos, n,
        "o filtro tinha de alcancar a peca INTEIRA (espec §7), e alcancou {movidos} de {n}"
    );
}

/// ⭐⭐⭐ **O FILTRO DE TECIDO ACUMULA; O DE MALHA NÃO.**
///
/// É a diferença de GESTO entre os dois, e ela é load-bearing: o
/// [`super::stroke_filter`] repõe a pose congelada a cada passo (dois passos com
/// a mesma força dão o MESMO resultado — voltar com o dedo desfaz), e a espec §7
/// manda o filtro de tecido correr **um passo de simulação por movimento do
/// rato**. ⛔ Se alguém puser um `restore_frozen_pose` aqui, o pano deixa de cair
/// e este gate reprova.
#[test]
fn o_filtro_de_tecido_acumula_em_vez_de_repor_a_pose() {
    let um = corre(ClothFilterKind::Gravity, 1.0, 1, [0.0; 3]);
    let tres = corre(ClothFilterKind::Gravity, 1.0, 3, [0.0; 3]);
    let base = plano().positions().to_vec();
    let d1 = desvio(&base, &um);
    let d3 = desvio(&base, &tres);
    println!("um passo move {d1:.6}; tres passos movem {d3:.6}");
    assert!(
        d1 > 0.0,
        "um passo tinha de mover -- a fixtura nao produz o fenomeno"
    );
    assert!(
        d3 > d1 * 1.5,
        "tres passos moveram {d3:.6} contra {d1:.6} de um -- o filtro esta' a REPOR a pose \
         em vez de simular, e um solver de tecido reposto e' um filtro de malha caro"
    );
}

/// ⭐⭐ **SÓ O APERTO LÊ O PONTO CONGELADO** — os outros quatro tipos dão o mesmo
/// bloco de vértices, ao bit, com o cursor noutro sítio.
///
/// ⚠️ **A régua é o produto, não o código:** o [`ClothFilterKind::le_o_ponto`]
/// afirma-o, e este gate mede-o correndo cada tipo com dois pontos distintos.
/// *Um censo que só lê a tabela que ele próprio afirma não é um censo.*
#[test]
fn so_o_aperto_le_o_ponto_congelado() {
    for kind in ClothFilterKind::ALL {
        let a = corre(kind, 1.0, 2, [0.0, 0.0, 0.0]);
        let b = corre(kind, 1.0, 2, [0.7, 0.4, 0.0]);
        let d = desvio(&a, &b);
        println!(
            "{:<8} com o ponto noutro sitio: desvio {d:.6}",
            kind.label()
        );
        if kind.le_o_ponto() {
            assert!(d > 0.0, "{:?} diz LER o ponto e nao mudou com ele", kind);
        } else {
            assert_eq!(
                d, 0.0,
                "{:?} diz NAO ler o ponto e mudou com ele (desvio {d:.6})",
                kind
            );
        }
    }
}

/// ⭐⭐ **CADA UM DOS CINCO MOVE A PEÇA** — o gate que impede um tipo de nascer
/// mudo, que é o defeito que esta casa varre a cada wave.
#[test]
fn os_cinco_tipos_movem_a_peca() {
    let base = plano().positions().to_vec();
    for kind in ClothFilterKind::ALL {
        let out = corre(kind, 1.0, 2, [0.0; 3]);
        let d = desvio(&base, &out);
        println!("{:<8} move {d:.6}", kind.label());
        assert!(
            d > 0.0,
            "{:?} nao moveu um vertice -- um tipo mudo e' pior que um tipo ausente",
            kind
        );
    }
}

/// ⭐ **SEM PEN-DOWN O PASSO É UM NO-OP EXACTO** — e a guarda é DERIVADA (há
/// sessão? a captura cobre a malha?), nunca um flag: dois campos a dizerem
/// *«estou em modo filtro»* podem discordar, e nesse dia o filtro correria sobre
/// o `pre` de outro gesto.
#[test]
fn sem_pen_down_o_passo_nao_toca_a_peca() {
    let mut mesh = plano();
    let antes = mesh.positions().to_vec();
    let mut st = SculptStroke::default();
    assert!(!st.cloth_filter_running());
    let movidos = st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo(1.0));
    assert_eq!(movidos, 0, "o passo correu sem pen-down");
    assert_eq!(
        desvio(&antes, mesh.positions()),
        0.0,
        "a peca mexeu-se sem pen-down"
    );
}

/// ⭐⭐ **O UNDO COBRE A PEÇA INTEIRA, e ele é o do TRAÇO.**
///
/// A espec §7 pede *um passo por uso do filtro*, e é exactamente o que o
/// `close_stroke` dá: ele grava `touched` + as posições congeladas. ⛔ Uma porta
/// de undo própria seria a segunda resposta a *«como se desfaz um punhado de
/// vértices deslocados»* — o mesmo argumento que fez o transform reusar a do
/// traço.
#[test]
fn o_pen_down_do_filtro_congela_a_peca_inteira_para_o_undo() {
    let mesh = plano();
    let n = mesh.vert_count();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(
        &mesh,
        ClothFilterProps::default(),
        ClothFilterKind::Gravity,
        [0.0; 3],
    );
    assert_eq!(
        st.touched().len(),
        n,
        "o pen-down do filtro tem de congelar a malha INTEIRA, senao o Ctrl+Z devolve metade"
    );
    assert!(st.cloth_filter_running());
}

/// ⭐ **O `s = 0` NÃO MEXE A PEÇA** — o controlo que todo gate de força precisa,
/// e a promessa que faz do arrasto uma recta: no ponto em que a mão não andou,
/// nada acontece.
///
/// ⚠️ **Ele NÃO é vácuo:** o solver corre à mesma (as restrições relaxam), e o
/// que se afirma é que sem força externa a malha de repouso é ponto fixo dele.
#[test]
fn com_arrasto_zero_a_peca_fica_parada() {
    let base = plano().positions().to_vec();
    for kind in ClothFilterKind::ALL {
        let out = corre(kind, 0.0, 3, [0.0; 3]);
        let d = desvio(&base, &out);
        println!("{:<8} com s = 0 move {d:.3e}", kind.label());
        assert_eq!(d, 0.0, "{:?} mexeu a peca com arrasto zero", kind);
    }
}

/// ⭐⭐ **O CENSO dos cinco contra os arms da lei** — quem escreve `σ` é quem a
/// tabela diz que é.
#[test]
fn so_a_escala_e_de_ancora_entre_os_cinco() {
    let de_ancora: Vec<_> = ClothFilterKind::ALL
        .into_iter()
        .filter(|k| super::stroke_cloth_filter::e_de_ancora(*k))
        .collect();
    assert_eq!(
        de_ancora,
        vec![ClothFilterKind::Scale],
        "a particao dos arms mudou -- releia a espec §7 antes de mexer no selector"
    );
}

/// ⭐⭐⭐ **AS NORMAIS SEGUEM A FORMA DEPOIS DE UM PASSO DO FILTRO.**
///
/// ⛔⛔ **Sem isto o render MENTE, e foi um report com foto que o apanhou**
/// (dono, 07/09: *«o render fica muito estranho, como se tivesse feito o bake de
/// uma textura»*). É a descrição exacta do sintoma: o passo escrevia as posições
/// novas e a malha ficava com as normais VELHAS ⇒ a janela de upload levava
/// geometria nova com sombreamento antigo, e *o relevo deixa de ser forma e passa
/// a ser um desenho colado por cima dela*.
///
/// ⚠️ **A régua é uma malha CONSTRUÍDA DE NOVO das posições deformadas** — o
/// oráculo é a geometria, não a nossa própria rotina de refresco. Compará-la com
/// ela mesma seria um espelho.
#[test]
fn depois_de_um_passo_as_normais_concordam_com_a_geometria() {
    // ⛔⛔ **A PEÇA É UMA ESFERA, e a escolha veio de DUAS mutações que
    // sobreviveram.** Numa folha plana o filtro é degenerado para esta pergunta:
    // a `Gravity` faz uma **translação rígida** (todos os vértices andam o mesmo
    // vector) e o `Expand` faz o repouso crescer **no plano** — nos dois casos a
    // superfície **não se dobra**, as normais velhas estão CERTAS, e apagar o
    // `refresh_region` deixava este gate verde. *A fixtura não produzia o
    // fenómeno* — a terceira leitura de uma mutação sobrevivente, e a mais cara.
    //
    // Numa ESFERA o `Inflate` empurra cada vértice ao longo da normal dele e as
    // restrições **resistem** ⇒ a superfície encurva de verdade.
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let repouso: Vec<[f32; 3]> = mesh.normals().to_vec();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(
        &mesh,
        ClothFilterProps::default(),
        ClothFilterKind::Inflate,
        [0.0; 3],
    );
    for _ in 0..6 {
        st.cloth_filter_step(&mut mesh, ClothFilterKind::Inflate, &passo(1.0));
    }
    // ⚠️ **A régua é uma malha CONSTRUÍDA DE NOVO das posições deformadas** — o
    // oráculo é a GEOMETRIA, não a nossa própria rotina de refresco. Compará-la
    // com ela mesma seria um espelho.
    let fresca = ph2d_mesh::Mesh::from_parts(mesh.positions().to_vec(), mesh.faces().to_vec())
        .expect("a malha deformada e' valida");
    let angulo = |a: [f32; 3], b: [f32; 3]| {
        (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    // ⭐⭐ **O CONTROLO, e ele mede a GEOMETRIA — nunca as normais guardadas.**
    // ⛔ A 1.ª redacção comparava `repouso` com `mesh.normals()`, e as duas ficam
    // paradas quando o refresco falta ⇒ *o controlo dependia da cura que o gate
    // existe para medir*, e com a mutação lia `0,04°` e reprovava pelo motivo
    // errado.
    let virou = fresca
        .normals()
        .iter()
        .zip(&repouso)
        .map(|(a, b)| angulo(*a, *b))
        .fold(0.0, f32::max);
    println!("controlo: a superficie virou ate' {virou:.2}° (medido na geometria)");
    assert!(
        virou > 5.0,
        "a fixtura nao produz o fenomeno: a superficie so' virou {virou:.2}°, e com normais \
         velhas CERTAS este gate ficaria verde sem a cura"
    );
    let pior = mesh
        .normals()
        .iter()
        .zip(fresca.normals())
        .map(|(a, b)| angulo(*a, *b))
        .fold(0.0, f32::max);
    println!("pior desvio entre a normal guardada e a da geometria: {pior:.4}°");
    // ⚠️ **A barra sai de um VALE MEDIDO:** com a cura o desvio é `0,0396°` (a
    // diferença de ordem de acumulação entre duas rotinas que somam as mesmas
    // normais de face); sem ela é a viragem inteira, `7,25°`. **`183×`** de vão.
    assert!(
        pior < 0.5,
        "as normais nao seguem a forma (pior {pior:.4}°) -- o passo do filtro esqueceu o \
         `refresh_region`, e o render vai sombrear geometria nova com normais velhas"
    );
}

/// ⭐⭐⭐ **O CENSO: TODO ESCRITOR DE POSIÇÕES REFRESCA AS NORMAIS.**
///
/// ⚠️ **É este o gate que teria apanhado o defeito, e não um teste do ficheiro
/// novo:** a propriedade não é *«o filtro de tecido refresca»* — é *«a família
/// inteira refresca»*, e o que se via ao ler o ficheiro novo era nada. **Só
/// contando a família** é que o membro em falta aparece.
///
/// ⛔ **A excepção é UMA e é NOMEADA:** o [`super::apply`] é o aplicador
/// PARTILHADO — ele escreve por conta dos outros, e quem refresca é quem o chama.
/// Um segundo nome nesta lista significa que alguém escreveu posições sem dizer
/// à malha, e o sintoma não é um teste vermelho: é o render a mentir.
///
/// ⚠️ **O que este censo NÃO vê**, escrito para não ser acreditado demais: ele lê
/// TEXTO, então um escritor que passe a malha a uma função noutro ficheiro
/// escapa-lhe. O irmão de cima é a metade que mede o comportamento.
#[test]
fn todo_escritor_de_posicoes_refresca_as_normais() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut escritores = Vec::new();
    let mut faltosos = Vec::new();
    for e in std::fs::read_dir(&dir).expect("o src da crate existe") {
        let p = e.expect("entrada").path();
        let nome = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !nome.ends_with(".rs") || nome.ends_with("_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(&p).expect("legivel");
        if !src.contains("positions_mut()") {
            continue;
        }
        escritores.push(nome.clone());
        if !src.contains("refresh_region") && nome != "stroke_apply.rs" {
            faltosos.push(nome);
        }
    }
    escritores.sort();
    println!("escritores de posicoes: {escritores:?}");
    assert!(
        escritores.len() >= 5,
        "o censo achou {} escritores -- ele deixou de encontrar a familia, e um censo que \
         varre uma populacao vazia le'-se como aprovado",
        escritores.len()
    );
    assert!(
        faltosos.is_empty(),
        "estes escrevem posicoes e NAO refrescam as normais: {faltosos:?} -- o render vai \
         sombrear geometria nova com normais velhas. A unica excepcao legitima e' o \
         aplicador PARTILHADO (`stroke_apply.rs`), porque quem o chama e' que refresca"
    );
}
