//! ⭐⭐⭐ **OS GATES DA CHAVE DA GRAVATA** — irmão de [`super::tests`] e de
//! [`super::tip_tests`] pelo teto de LOC da shell (HR-18, 600), cortado por
//! RESPONSABILIDADE: aquele defende as chaves de **furo** e de **peça**, o outro a de
//! **ponta**, e este a **face que se cruza sobre si própria** — mais a ORDEM das quatro e a
//! guarda que a gravata armou.
//!
//! ⛔⛔⛔ **O report que os exige** (Enio, 2026-08-30, com foto): *«destruiu completamente a
//! malha»* — `0` faces auto-intersectadas no caminho de omissão contra **`125`** no outro, e
//! nenhuma das chaves que o botão lia dizia mais do que *pior*.

use ph2d_mesh::{Face, Mesh};

use super::tests::{cubo, cubos, sem, um_quad};

/// ⭐⭐⭐ **GATE — a face em OITO perde, e a régua antiga era CEGA a ela.**
///
/// ⛔⛔⛔ **É o gate do report de 2026-08-30** (*«destruiu completamente a malha»*, com foto). A
/// régua que via aquele estrago — [`ph2d_quadfill::local_shape`] — **já existia numa crate do
/// produto** e o único leitor dela era a **sonda**. As colunas que o [`super::worse`] lia diziam
/// apenas *pior* (`χ` de `1` para `0`, bordo de `4` para `12`), e o que o dono viu foi uma peça
/// rasgada de alto a baixo: `0` faces auto-intersectadas no caminho de omissão contra **`125`**.
///
/// ⚠️ *Uma régua na prateleira não protege ninguém* — é a família do §5.0 do `CLAUDE.md`
/// (**nenhum instrumento pergunta se o valor chega a um CONSUMIDOR**), e desta vez o consumidor
/// em falta era o próprio botão.
#[test]
fn a_face_em_oito_perde_e_a_regua_antiga_nao_a_via() {
    let boa = um_quad(false);
    let torta = um_quad(true);

    // ⛔ O CONTROLE: sob as DUAS chaves anteriores elas são indistinguíveis.
    assert_eq!(super::open_edges(&boa), super::open_edges(&torta));
    assert_eq!(super::components(&boa), super::components(&torta));

    // ⭐ E a régua nova separa-as, com zero natural de um lado.
    assert_eq!(super::bowties(&boa), 0);
    assert!(
        super::bowties(&torta) > 0,
        "⛔ a fixtura tem de CONTER o fenomeno, senao este gate nao prova nada"
    );

    // ⭐⭐ A forma é dada PERFEITA na torta e PÉSSIMA na boa — o desempate que escolheria o
    // estrago se a chave nova não existisse.
    assert!(
        super::worse(&torta, 0, 0.0, sem(), &boa, 999, 89.0, sem()),
        "⛔ uma malha com faces auto-intersectadas e' PIOR que uma feia mas sa'"
    );
    assert!(
        !super::worse(&boa, 999, 89.0, sem(), &torta, 0, 0.0, sem()),
        "⛔ e a relacao tem de ser ANTI-SIMETRICA"
    );
}

/// ⭐⭐⭐ **GATE — a ORDEM das quatro chaves, lida onde ela de facto vive.**
///
/// ⚠️ **No fonte e não por fixtura, e a razão é medida:** construir um par que empate em bordo
/// **e** difira em peças **e** em gravatas exige uma malha fechada com uma face cruzada, e
/// cruzar uma face de um sólido **abre bordo** (foi o que reprovou a 1.ª fixtura deste bloco).
/// *Quando a fixtura que isolaria a chave não existe, a ordem lê-se onde ela está escrita.*
///
/// ⛔ Furos e peças decidem primeiro — *o que o artista vê antes de tudo é um buraco ou um pedaço
/// a flutuar*. Uma face em oito é estrago de **superfície**, e vem a seguir. Quem achar a chave
/// nova a mais importante e a subir desfaz as duas leis que os reports anteriores compraram.
#[test]
fn a_ordem_das_chaves_e_furos_pecas_gravatas_forma() {
    let src = include_str!("sculpt3d_retopo_rulers.rs");
    let ini = src
        .find("pub(super) fn worse(")
        .expect("a funcao mudou de nome");
    // ⚠️ **Do CORPO, não da assinatura.** A 1.ª redacção fatiava a partir do `fn` e a lista de
    // parâmetros nomeia `a_over60` **antes** de tudo ⇒ o gate reprovava sobre a ordem certa.
    // *Um gate que lê o fonte tem de saber onde acaba a declaração.*
    let corpo = &src[ini..];
    let abre = corpo
        .find(") -> bool {")
        .expect("a assinatura de worse mudou");
    let corpo = &corpo[abre..];
    let fim = corpo.find("\n}").expect("o corpo de worse nao fecha");
    let corpo = &corpo[..fim];
    let em = |agulha: &str| corpo.find(agulha).expect(agulha);
    assert!(
        em("a_holes") < em("a_parts"),
        "⛔ os furos decidem antes das pecas"
    );
    assert!(
        em("a_parts") < em("a_bow"),
        "⛔ as pecas decidem antes das gravatas"
    );
    assert!(
        em("a_bow") < em("a_over60"),
        "⛔ as gravatas decidem antes da forma -- uma face cruzada nao e' um gradiente de \
         qualidade"
    );
}

/// ⭐⭐⭐ **GATE — a face em OITO ARMA outra tentativa, e a régua antiga não a armava.**
///
/// ⛔⛔ **É a metade da cura de 30/08 que o [`super::worse`] sozinho não dá.** O `worse` só
/// ordena as candidatas que **existem**; se a primeira sair cruzada e nenhuma outra for pedida,
/// o artista recebe-a na mesma. A condição que pede mais uma tentativa é [`super::still_broken`],
/// e até este dia ela era **só** o bordo.
///
/// ⭐⭐ **E isto é estritamente melhor que uma RECUSA:** as candidatas extra passam todas pelo
/// `worse`, logo *só vencem onde são melhores* — se todas saírem cruzadas ainda se entrega a
/// menos má. *Uma recusa absoluta transformaria um defeito raro numa ferramenta inutilizável*, e
/// a prova de corpus que a justificaria **não existe — ela foi medida e diz o CONTRÁRIO**: toda
/// malha retopologizada da pasta do dono tem faces cruzadas, incluindo `Sculpt_Blender.obj`, a
/// saída que ele **aprovou** (`1` em `8 291`). *Um veto teria recusado a malha que ele elogiou.*
#[test]
fn a_face_em_oito_arma_outra_tentativa() {
    let boa = um_quad(false);
    let torta = um_quad(true);

    // ⛔ O CONTROLE: pela régua ANTIGA (só o bordo) as duas armam igual — as duas têm bordo,
    // por serem quads soltos, logo a fixtura tem de o dizer explicitamente para o gate não
    // ficar verde por acaso.
    assert_eq!(super::open_edges(&boa), super::open_edges(&torta));

    // ⭐ E uma malha FECHADA e sã não arma nada — é o que torna a condição barata.
    let fechada = cubos(1);
    assert_eq!(super::open_edges(&fechada), 0);
    assert_eq!(super::bowties(&fechada), 0);
    assert!(
        !super::still_broken(&fechada),
        "⛔ uma peca fechada e sa' nao pode pedir mais uma tentativa -- isso seria pagar sempre"
    );

    // ⭐⭐ Agora a mesma malha fechada, com UMA face cruzada: tem de armar.
    let fechada_torta = {
        let (v, f) = cubo(0.0);
        let mut faces: Vec<Face> = f
            .into_iter()
            .map(|q| Face::quad(q[0], q[1], q[2], q[3]))
            .collect();
        let c = faces[0].verts().to_vec();
        faces[0] = Face::quad(c[0], c[1], c[3], c[2]);
        Mesh::from_parts(v, faces).expect("a fixtura e' construida aqui")
    };
    assert!(
        super::bowties(&fechada_torta) > 0,
        "⛔ a fixtura tem de CONTER o fenomeno"
    );
    assert!(
        super::still_broken(&fechada_torta),
        "⛔ uma face cruzada sobre si propria tem de pedir outra tentativa"
    );
}

/// ⭐⭐⭐ **GATE — os DOIS sítios que armam tentativa extra passam pela MESMA porta.**
///
/// ⛔ O botão arma uma 3.ª e uma 4.ª candidata, e as duas perguntam a mesma coisa. ⚠️ **Enquanto
/// a pergunta era a do bordo sozinho ela estava escrita duas vezes** — e uma lei escrita em dois
/// sítios não é uma lei, é uma coincidência à espera de divergir (a 3.ª chave entrar numa e não
/// na outra teria sido exactamente isso). *Uma porta, dois chamadores.*
///
/// ⛔⛔ **E a metade que PROÍBE a forma antiga tem de DESCASCAR OS COMENTÁRIOS**, senão o
/// primeiro que **documentar** a mudança — escrevendo a forma velha para dizer que ela morreu —
/// reprova o portão. *É a armadilha de todo gate textual, e o ficheiro medido documenta
/// precisamente essa mudança, ao lado do `use` que ela esvaziou.*
#[test]
fn os_dois_sitios_que_armam_perguntam_pela_mesma_porta() {
    let src = include_str!("sculpt3d_history_retopo_extract.rs");
    assert_eq!(
        src.matches("still_broken(&out)").count(),
        2,
        "⛔ os dois sitios que armam tentativa extra tem de chamar a MESMA funcao"
    );
    let codigo: Vec<&str> = src
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect();
    let codigo = codigo.join("\n");
    assert!(
        !codigo.contains("open_edges(&out) > 0"),
        "⛔ ficou um sitio a perguntar so' pelo bordo -- a 3.a chave nao o alcanca"
    );
    // ⛔ O CONTROLE do descascador: ele tem de continuar a ver o CÓDIGO, senão a asserção de
    // cima passaria sobre um ficheiro vazio e não mediria nada.
    assert_eq!(
        codigo.matches("still_broken(&out)").count(),
        2,
        "⛔ o descascador comeu o codigo -- a assercao de cima ficaria vacua"
    );
}

/// ⭐⭐⭐ **GATE — a RAZÃO de a guarda não ser um veto continua escrita ao lado dela.**
///
/// ⛔⛔ **Sem isto, «promover a guarda a recusa» lê-se como uma melhoria óbvia** — e ela foi
/// **medida e refutada** em 2026-08-30: as três malhas retopologizadas da pasta do dono têm faces
/// cruzadas, `Sculpt_Blender.obj` (a que ele aprovou) incluída. *Um default sem razão escrita é um
/// default que o próximo inverte.*
#[test]
fn a_razao_de_nao_ser_veto_esta_ao_lado_da_guarda() {
    let src = include_str!("sculpt3d_retopo_rulers.rs");
    for agulha in ["Sculpt_Blender.obj", "sculpt_t003.obj", "8 291", "APROVOU"] {
        assert!(
            src.contains(agulha),
            "⛔ a refutacao do veto perdeu {agulha} -- alguem vai propo-lo outra vez"
        );
    }
}
