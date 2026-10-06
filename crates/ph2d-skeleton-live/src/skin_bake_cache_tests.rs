//! Os gates do memo da assadura — ver o cabeçalho do [`super`].
//!
//! ⚠️ **Eles medem o MEMO e nunca a assadura**: a lei entra por parâmetro, logo cada gate traz a
//! própria e conta quantas vezes ela correu. *Um memo prova-se pelo número de vezes que o trabalho
//! caro NÃO aconteceu; medir a saída dele não distingue um memo de uma função pura.*

use super::*;
use ph2d_poly2d::Mesh2d;

/// Uma malha mínima com `n` triângulos e uma tabela de UM osso — o memo não olha para o conteúdo,
/// só o carrega.
fn malha(n: u32) -> SkinnedMesh {
    let mut mesh = Mesh2d {
        rest: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        tris: Vec::new(),
        size: [4, 4],
    };
    for _ in 0..n {
        mesh.tris.push([0, 1, 2]);
    }
    SkinnedMesh {
        pesos: vec![1.0; mesh.rest.len()],
        mesh,
    }
}

/// Uma lei que conta quantas vezes correu e devolve uma malha com `saida` triângulos.
fn lei(
    corridas: &std::cell::Cell<usize>,
    saida: Option<u32>,
) -> impl FnOnce(&SkinnedMesh) -> Option<SkinnedMesh> + '_ {
    move |_m| {
        corridas.set(corridas.get() + 1);
        saida.map(malha)
    }
}

/// ⭐⭐⭐ **A SEGUNDA CONSULTA NÃO VOLTA A ASSAR — e devolve a MESMA malha, não uma igual.**
///
/// ⚠️ **A prova é `Rc::ptr_eq` e não `==`:** uma lei determinística devolve uma malha igual às duas
/// vezes, logo uma igualdade de valor ficaria verde sobre um memo que não memoiza nada.
#[test]
fn a_segunda_consulta_nao_volta_a_assar() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    let a = assada_do_bind(7, b"fonte-a", &crua, lei(&corridas, Some(9))).expect("assou");
    let b = assada_do_bind(7, b"fonte-a", &crua, lei(&corridas, Some(9))).expect("do memo");
    assert_eq!(corridas.get(), 1, "a lei correu {} vezes", corridas.get());
    assert!(Rc::ptr_eq(&a, &b), "o memo devolveu outra malha");
    assert_eq!(a.mesh.tris.len(), 9);
}

/// ⛔⛔ **MUDAR A FONTE MANDA ASSAR OUTRA VEZ, com os MESMOS bits** — é isto que faz o endereço ser
/// só um endereço.
///
/// Sem esta metade, re-prender a arte (ou desfazer para um bind anterior) entregaria a malha assada
/// do bind ANTERIOR, com o número certo de vértices e a geometria errada — o modo de falha mudo que
/// esta família chama *«o controlo desenhado por um mapa e agarrado por outro»*.
#[test]
fn mudar_a_fonte_manda_assar_outra_vez() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    let a = assada_do_bind(7, b"fonte-a", &crua, lei(&corridas, Some(9))).expect("assou");
    let b = assada_do_bind(7, b"fonte-B", &crua, lei(&corridas, Some(11))).expect("assou de novo");
    assert_eq!(corridas.get(), 2, "a lei correu {} vezes", corridas.get());
    assert_eq!(a.mesh.tris.len(), 9);
    assert_eq!(
        b.mesh.tris.len(),
        11,
        "a 2.ª consulta devolveu a malha da 1.ª fonte"
    );
}

/// ⭐⭐ **O `None` TAMBÉM É GUARDADO.**
///
/// ⚠️ É o caso mais comum de todos — a porta fechada responde `None` em toda a arte —, e sem o
/// guardar o produto pagaria uma tentativa de assadura por imagem por QUADRO para chegar sempre à
/// mesma resposta. *Um memo que só guarda os sucessos é mais caro que não ter memo nenhum no
/// caminho de omissão.*
#[test]
fn o_none_tambem_e_guardado() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    assert!(assada_do_bind(3, b"f", &crua, lei(&corridas, None)).is_none());
    assert!(assada_do_bind(3, b"f", &crua, lei(&corridas, None)).is_none());
    assert_eq!(corridas.get(), 1, "a lei correu {} vezes", corridas.get());
    assert_eq!(gavetas(), 1, "o `None` nao ocupou gaveta nenhuma");
}

/// ⛔ **A GAVETA MENOS USADA SAI — e a que acabou de ser assada FICA.**
///
/// ⚠️ **A 2.ª metade é a que importa:** despejar antes de inserir faria uma cena com mais de
/// [`ENTRADAS_MAX`] imagens deitar fora exactamente a malha que acabou de custar uma assadura, e o
/// quadro seguinte voltaria a assá-la — *um memo que expulsa o recém-chegado é um contador de
/// trabalho repetido com o nome de cache*.
#[test]
fn a_gaveta_menos_usada_sai_e_a_recem_assada_fica() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    for bits in 0..ENTRADAS_MAX as u64 {
        let _ = assada_do_bind(bits, b"f", &crua, lei(&corridas, Some(1)));
    }
    assert_eq!(gavetas(), ENTRADAS_MAX);
    // A gaveta `0` é a mais antiga; tocar na `1` não a salva, e a `ENTRADAS_MAX` é a nova.
    let nova = ENTRADAS_MAX as u64;
    let n = assada_do_bind(nova, b"f", &crua, lei(&corridas, Some(5))).expect("assou");
    assert_eq!(gavetas(), ENTRADAS_MAX, "o memo passou do tecto");
    assert_eq!(n.mesh.tris.len(), 5);
    // Ela ainda lá está: uma 2.ª consulta não volta a assar.
    let antes = corridas.get();
    let _ = assada_do_bind(nova, b"f", &crua, lei(&corridas, Some(5)));
    assert_eq!(
        corridas.get(),
        antes,
        "a malha recem-assada foi despejada pelo proprio despejo"
    );
}

/// ⭐⭐ **TOCAR NUMA GAVETA SALVA-A DO DESPEJO** — a metade que faz o memo ser LRU e não FIFO.
///
/// ⚠️ **Sem ela nenhum gate media o `visto = agora` do ACERTO**, e apagá-lo é invisível na cena
/// pequena: numa cena com mais de [`ENTRADAS_MAX`] imagens presas, a que o artista está a ver a
/// cada quadro seria despejada pela que ninguém olha, e voltaria a ser assada a seguir. *Um memo
/// que não refresca quem acerta é um FIFO com o nome de cache.*
#[test]
fn tocar_numa_gaveta_salva_a_do_despejo() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    for bits in 0..ENTRADAS_MAX as u64 {
        let _ = assada_do_bind(bits, b"f", &crua, lei(&corridas, Some(1)));
    }
    // A `0` é a mais antiga — tocá-la põe-na no fim da fila e manda a `1` para a frente dela.
    let _ = assada_do_bind(0, b"f", &crua, lei(&corridas, Some(1)));
    let _ = assada_do_bind(ENTRADAS_MAX as u64, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(gavetas(), ENTRADAS_MAX);

    let antes = corridas.get();
    let _ = assada_do_bind(0, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(
        corridas.get(),
        antes,
        "a gaveta TOCADA foi despejada: o memo esta' a ordenar por idade de ENTRADA, nao de USO"
    );
    // E o CONTROLO: a `1`, que ninguém tocou, foi mesmo a despejada.
    let _ = assada_do_bind(1, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(
        corridas.get(),
        antes + 1,
        "ninguem foi despejado — o tecto nao esta' a funcionar e o gate nao afirma nada"
    );
}

/// ⛔⛔ **O DESPEJO É PELA IDADE E NUNCA PELO ENDEREÇO** — e foi uma mutação SOBREVIVENTE que
/// mostrou que nenhum dos outros gates o dizia.
///
/// ⚠️ Enquanto uma cena não passa do tecto os dois critérios coincidem, e mesmo acima dele a
/// diferença só aparece quando a ordem de ENTRADA discorda da ordem dos BITS — que é o caso normal,
/// porque `Entity::to_bits()` é um id de alocação e não uma data. ⇒ esta cena enche o memo por
/// ordem **decrescente** de endereço, e é isso que separa as duas leis.
///
/// *Um memo que despeja pelo id despeja a arte que o artista abriu primeiro por ela ter calhado num
/// slot alto do ECS — e isso lê-se como «a assadura é lenta», nunca como «o memo escolheu mal».*
#[test]
fn o_despejo_e_pela_idade_e_nao_pelo_endereco() {
    esquece_tudo();
    let corridas = std::cell::Cell::new(0);
    let crua = malha(2);
    // A mais ANTIGA é a de MAIOR endereço.
    for bits in (1..=ENTRADAS_MAX as u64).rev() {
        let _ = assada_do_bind(bits, b"f", &crua, lei(&corridas, Some(1)));
    }
    let _ = assada_do_bind(0, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(gavetas(), ENTRADAS_MAX);

    let antes = corridas.get();
    let _ = assada_do_bind(1, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(
        corridas.get(),
        antes,
        "a gaveta de MENOR endereco foi despejada: o memo esta' a ordenar por bits"
    );
    let _ = assada_do_bind(ENTRADAS_MAX as u64, b"f", &crua, lei(&corridas, Some(1)));
    assert_eq!(
        corridas.get(),
        antes + 1,
        "a gaveta mais ANTIGA sobreviveu — o despejo nao foi pela idade"
    );
}
