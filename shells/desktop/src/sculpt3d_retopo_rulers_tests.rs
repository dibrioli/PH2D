//! ⭐⭐⭐ **OS GATES DA RÉGUA QUE CONTA AS PEÇAS** — irmão de
//! [`super`] pela mesma lei que partiu o botão: *o gate mora onde mora a REGRA, não onde
//! mora o ficheiro que a chama.*
//!
//! ⛔⛔⛔ **A foto de 2026-08-30** (*«péssimo»*, um quad a flutuar solto ao lado de uma
//! ponta) atravessou **todas** as réguas desta linha: `χ` conta os dois lados e dá um
//! número plausível, o bordo é `0`, o não-manifold é `0`, a forma dos quads é boa, e a
//! contagem de quads até **sobe**. *Um pedaço que se solta sai FECHADO.*
//!
//! ⚠️ **Cada gate aqui carrega o CONTROLE da régua antiga ao lado**, porque uma chave nova
//! que se limita a concordar com a que já existia não compra nada: se a fixtura não
//! **empatar** em `open_edges`, o gate não isola a contagem de peças de coisa nenhuma.

use ph2d_mesh::{Face, Mesh};

/// Um cubo fechado de quads, deslocado em `x` — `12` arestas, cada uma com exactamente
/// duas faces.
fn cubo(dx: f32) -> (Vec<[f32; 3]>, Vec<[u32; 4]>) {
    let v = vec![
        [dx, 0.0, 0.0],
        [dx + 1.0, 0.0, 0.0],
        [dx + 1.0, 1.0, 0.0],
        [dx, 1.0, 0.0],
        [dx, 0.0, 1.0],
        [dx + 1.0, 0.0, 1.0],
        [dx + 1.0, 1.0, 1.0],
        [dx, 1.0, 1.0],
    ];
    let f = vec![
        [0, 1, 2, 3],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ];
    (v, f)
}

/// `n` cubos fechados, soltos uns dos outros.
fn cubos(n: usize) -> Mesh {
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for k in 0..n {
        #[expect(
            clippy::cast_precision_loss,
            reason = "k <= 3 nesta fixtura; o valor e' so' um deslocamento de posicao"
        )]
        let (v, f) = cubo(k as f32 * 4.0);
        let base = u32::try_from(verts.len()).expect("a fixtura e' pequena");
        verts.extend(v);
        faces.extend(
            f.into_iter()
                .map(|q| Face::quad(q[0] + base, q[1] + base, q[2] + base, q[3] + base)),
        );
    }
    Mesh::from_parts(verts, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — DUAS superfícies fechadas não são UMA, e a régua antiga não sabe disso.**
///
/// ⛔ **Este é o gate que a foto exigia.** As duas metades da chave da frente ([`super::open_edges`]
/// = bordo **+** não-manifold) dão `0` numa peça inteira **e** numa peça partida em duas: um pedaço
/// que se desprende leva as próprias arestas com ele, cada uma com as suas duas faces. *Contar
/// arestas nunca revela quantas peças elas formam.*
#[test]
fn duas_superficies_fechadas_dao_zero_no_bordo_e_duas_na_contagem() {
    let inteira = cubos(1);
    let partida = cubos(2);

    // ⛔ O CONTROLE: sob a régua ANTIGA as duas são indistinguíveis, e nas DUAS metades dela.
    assert_eq!(
        super::open_edges(&inteira),
        0,
        "⛔ a fixtura de um cubo tem de ser fechada, senao o controle nao vale"
    );
    assert_eq!(
        super::open_edges(&partida),
        super::open_edges(&inteira),
        "⛔ as duas fixturas TEM de empatar na chave da frente -- e' isso que faz este gate \
         isolar a contagem de pecas"
    );
    assert_eq!(
        super::boundary_edges(&partida),
        0,
        "⛔ e nao pode ser so' o nao-manifold a empatar: o bordo tambem e' zero nas duas"
    );

    // ⭐⭐ E a régua nova separa-as.
    assert_eq!(super::components(&inteira), 1);
    assert_eq!(super::components(&partida), 2);
    assert_eq!(
        super::components(&cubos(3)),
        3,
        "⛔ e conta, nao satura em 2"
    );
}

/// ⭐⭐⭐ **GATE — a peça PARTIDA perde, mesmo sendo a mais bonita das duas.**
///
/// ⚠️ **A forma é dada perfeita na partida e péssima na inteira** — exactamente o desempate que
/// escolheria o estilhaço. Sob a lei ANTERIOR (sem [`super::components`]) esta asserção é
/// **falsa**: as duas empatam em `open_edges` e a decisão cai para o `>60°`, onde a partida ganha.
#[test]
fn a_peca_partida_perde_para_a_inteira_mesmo_com_a_forma_perfeita() {
    let inteira = cubos(1);
    let partida = cubos(2);
    assert!(
        super::worse(&partida, 0, 0.0, &inteira, 999, 89.0),
        "⛔ uma peca em dois pedacos e' PIOR que uma inteira feia -- o artista ve' o pedaco a \
         flutuar"
    );
    assert!(
        !super::worse(&inteira, 999, 89.0, &partida, 0, 0.0),
        "⛔ e a relacao tem de ser ANTI-SIMETRICA: se a partida e' pior, a inteira nao e'"
    );
}

/// ⭐⭐⭐ **GATE — a contagem de peças NÃO passa por cima dos furos.**
///
/// ⚠️ **A ordem é uma decisão** (`furos → peças → >60° → enviesamento`) e não um acaso do código:
/// os furos ficam à frente porque *foi isso que se mediu*. Sem este gate, alguém que ache a chave
/// nova mais importante põe-na à frente e desfaz a lei que a queixa do artista comprou três vezes.
#[test]
fn os_furos_continuam_a_decidir_antes_das_pecas() {
    let inteira_furada = {
        let (v, f) = cubo(0.0);
        let faces = f
            .into_iter()
            .take(5) // ⭐ tira uma face: 1 peça, 4 arestas de bordo.
            .map(|q| Face::quad(q[0], q[1], q[2], q[3]))
            .collect::<Vec<_>>();
        Mesh::from_parts(v, faces).expect("a fixtura e' construida aqui")
    };
    let partida_fechada = cubos(2);

    assert_eq!(super::components(&inteira_furada), 1);
    assert_eq!(super::open_edges(&inteira_furada), 4);
    assert_eq!(super::components(&partida_fechada), 2);
    assert_eq!(super::open_edges(&partida_fechada), 0);

    assert!(
        super::worse(&inteira_furada, 0, 0.0, &partida_fechada, 999, 89.0),
        "⛔ o FURO decide antes da contagem de pecas -- pos a chave nova a' frente?"
    );
}

/// ⭐⭐⭐ **GATE — a união é por ARESTA, e um toque num vértice só não cola nada.**
///
/// ⚠️ *Dois sacos que se tocam num ponto são, para quem olha, duas peças.* Uma união por vértice
/// daria `1` e aprovaria a peça partida — e é o erro fácil de escrever, porque o mapa de vértices
/// é mais curto de montar que o de arestas.
#[test]
fn um_toque_num_vertice_so_nao_cola_duas_pecas() {
    // Dois quads que partilham EXACTAMENTE o vértice 2, e nenhuma aresta.
    let m = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0], // ⭐ o vértice partilhado
            [0.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
        ],
        vec![Face::quad(0, 1, 2, 3), Face::quad(2, 4, 5, 6)],
    )
    .expect("a fixtura e' construida aqui");
    assert_eq!(
        super::components(&m),
        2,
        "⛔ a uniao e' por ARESTA -- um vertice partilhado nao faz uma peca so'"
    );
}

/// ⭐⭐⭐ **GATE — uma aresta NÃO-MANIFOLD não parte a peça, e as duas réguas medem coisas
/// independentes.**
///
/// ⚠️ Três faces a partilhar uma aresta é um defeito de *casca*, não de *contagem*: se
/// [`super::components`] as separasse, ela passaria a duplicar o [`super::open_edges`] e o veto
/// recusaria peças que a escada já sabe ordenar.
#[test]
fn uma_aresta_nao_manifold_nao_parte_a_peca() {
    let tripla = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        vec![
            Face::quad(0, 1, 2, 3),
            Face::quad(1, 0, 5, 4),
            Face::quad(0, 1, 6, 7),
        ],
    )
    .expect("a fixtura e' construida aqui");
    assert!(
        super::open_edges(&tripla) > super::boundary_edges(&tripla),
        "⛔ a fixtura tem de CONTER a aresta nao-manifold, senao este gate nao diz nada"
    );
    assert_eq!(
        super::components(&tripla),
        1,
        "⛔ as tres faces partilham uma aresta -- sao UMA peca"
    );
}

/// ⭐⭐⭐ **GATE — o veto é RELATIVO à entrada, nunca absoluto.**
///
/// ⚠️ Uma cena que já traz dois objectos soltos tem todo o direito de sair com dois. *Um veto
/// absoluto (`saiu > 1`) recusaria toda peça legítima de mais de um corpo* — e ninguém o notaria
/// enquanto o corpus de teste fosse só de peças únicas.
#[test]
fn o_veto_compara_com_o_que_entrou_e_nao_com_um() {
    let uma = cubos(1);
    let duas = cubos(2);
    let tres = cubos(3);

    assert_eq!(
        super::shattered(&duas, &uma),
        Some((2, 1)),
        "⛔ partir uma peca em duas TEM de vetar, com a contagem dos dois lados"
    );
    assert!(
        super::shattered(&duas, &duas).is_none(),
        "⛔ duas pecas a entrar e duas a sair nao e' estilhaco nenhum"
    );
    assert!(
        super::shattered(&uma, &duas).is_none(),
        "⛔ juntar tambem nao e' estilhaco -- o veto olha uma direccao so'"
    );
    assert_eq!(super::shattered(&tres, &duas), Some((3, 2)));
}

/// ⭐⭐⭐ **GATE — o BOTÃO chama o veto, e chama-o DEPOIS da escada.**
///
/// ⚠️ **Uma régua que ninguém invoca é uma nota**, e as sete asserções acima ficariam todas verdes
/// com o botão a entregar a peça partida na mesma. ⛔ **E a POSIÇÃO é metade da lei:** posto antes
/// da 3.ª/4.ª tentativas, o veto recusaria peças que a escada ainda ia consertar.
///
/// ⚠️ Este gate mora aqui — ao lado da regra, e não ao lado do ficheiro que a chama — pela mesma lei
/// que já custou quatro vermelhos de árvore a esta linha.
#[test]
fn o_botao_chama_o_veto_e_depois_da_escada() {
    let src = include_str!("sculpt3d_history_retopo_extract.rs");
    let veto = src
        .find("rulers::shattered(&out, &reference)")
        .expect("⛔ o botao TEM de chamar o veto -- sem isto a peca partida shipa");
    assert!(
        src[veto..veto + 220].contains("RemeshRefusal::Shattered"),
        "⛔ o veto tem de RECUSAR, nao so' de medir"
    );
    let quarta = src
        .find("let uniforme = if adaptive > 0.0")
        .expect("⛔ a 4.a tentativa mudou de forma -- reconfira a ordem do veto");
    assert!(
        quarta < veto,
        "⛔ o veto tem de vir DEPOIS da ultima tentativa: antes dela recusaria pecas que a \
         escada ainda ia consertar"
    );
    let relatorio = src
        .find("let tips = ph2d_quadfill::tip_survival")
        .expect("⛔ a medicao das pontas mudou de nome");
    assert!(
        veto < relatorio,
        "⛔ e ANTES do relatorio: medir as pontas de uma peca que vai ser recusada e' trabalho \
         que ninguem le'"
    );
}

/// ⭐⭐⭐ **GATE — a segunda régua fala SÓ onde a primeira está muda.**
///
/// ⛔ **Duas frases para o mesmo defeito é ruído**, e na peça do artista a contagem de pontas e a
/// cobertura disparam juntas. O que a cobertura acrescenta é o caso que a outra **não pode** ver:
/// a `tip_survival` tem de ACHAR um ápice primeiro, então uma perda numa crista ou numa saliência
/// larga sai com `tips_cut == 0`.
///
/// ⚠️ **E o `0` de «não medido» não pode acusar:** a cadeia local não mede cobertura nenhuma, e
/// sem a guarda da contagem ela leria `0,0 %` — que passa a barra por baixo e fica calada por
/// acidente, não por lei.
#[test]
fn a_cobertura_so_fala_quando_a_contagem_de_pontas_esta_muda() {
    use crate::sculpt3d::history::remesh::QuadRemeshReport;
    use crate::sculpt3d::history::retopo_global::retopo_line;
    let base = QuadRemeshReport {
        verts: 100,
        quads: 100,
        non_quads: 0,
        edge: 0.1,
        ms: 1.0,
        holes: 0,
        irregular: 0,
        edge_median_ratio: 1.0,
        edge_max_ratio: 1.0,
        edge_max_span: 0.0,
        shape: ph2d_quadfill::QuadShape::default(),
        aligned: true,
        measured: true,
        mirrored: 0,
        doublets: 0,
        folded: 0,
        tips_cut: 0,
        tips_total: 12,
        tips_worst_pct: 0.0,
        coverage_shell_p50: 0.0,
        coverage_shell_worst: 0.0,
        coverage_samples: 0,
    };

    // ⛔ CONTROLE 1: sem medição, a cobertura é MUDA mesmo com o valor a zero.
    let muda = retopo_line(&base);
    assert!(
        !muda.contains("POR COBRIR"),
        "⛔ `coverage_samples == 0` e' NAO MEDIDO -- nao pode ler-se como aprovado nem acusar: \
         {muda}"
    );

    // ⭐ Medida e fora da barra, com a contagem de pontas calada: ela fala.
    let fala = retopo_line(&QuadRemeshReport {
        coverage_samples: 500,
        coverage_shell_p50: 0.06,
        coverage_shell_worst: 0.09,
        ..base
    });
    assert!(
        fala.contains("POR COBRIR") && fala.contains("6.0"),
        "⛔ com a casca a 6 % e zero pontas acusadas, a linha tem de o DIZER: {fala}"
    );

    // ⛔ CONTROLE 2: com a contagem de pontas a acusar, a cobertura CALA-SE.
    let uma_so = retopo_line(&QuadRemeshReport {
        tips_cut: 2,
        tips_worst_pct: -21.0,
        coverage_samples: 500,
        coverage_shell_p50: 0.06,
        coverage_shell_worst: 0.09,
        ..base
    });
    assert!(
        uma_so.contains("AMPUTADA") && !uma_so.contains("POR COBRIR"),
        "⛔ duas frases para o mesmo defeito e' ruido -- a especifica ganha: {uma_so}"
    );

    // ⭐ E dentro da barra fica calada, medida ou não.
    let limpa = retopo_line(&QuadRemeshReport {
        coverage_samples: 500,
        coverage_shell_p50: 0.003,
        ..base
    });
    assert!(!limpa.contains("POR COBRIR"), "⛔ dentro da barra: {limpa}");
}

/// ⭐⭐⭐ **GATE — a recusa NOMEIA a contagem e o conserto.**
///
/// ⚠️ **O conserto vem antes do `Detail`**, e a ordem é medida: na peça do artista a re-entrada
/// parte a peça em **qualquer** ponto do slider, então mandá-lo mexer no slider primeiro seria
/// mandá-lo ao sítio errado. *Uma recusa que não diz o que fazer a seguir é uma recusa muda.*
#[test]
fn a_recusa_do_estilhaco_diz_a_contagem_e_manda_desfazer() {
    let frase = super::super::RemeshRefusal::Shattered { pieces: 2, was: 1 }.explain();
    assert!(
        frase.contains('2') && frase.contains('1'),
        "⛔ a frase tem de trazer os DOIS numeros -- «partiu» sem denominador nao diz nada: {frase}"
    );
    assert!(
        frase.contains("Ctrl+Z"),
        "⛔ e tem de nomear o conserto que de facto resolve: {frase}"
    );
    assert!(
        frase.contains("escultura fica como esta'"),
        "⛔ e tem de dizer que a peca dele NAO se perdeu: {frase}"
    );
}

/// **UM QUAD SÓ**, plano — em ordem (`false`) ou trocado em OITO (`true`).
///
/// ⚠️ **Um quad solto e não um cubo, e a 1.ª redacção pagou por isso:** permutar os cantos de
/// uma face **muda as arestas que ela contribui**, e num cubo isso abre `4` arestas de bordo ⇒ o
/// controlo *«a topologia fica intacta»* reprovava `0` contra `4`. Solto, as duas versões têm as
/// mesmas `4` arestas de bordo e a mesma peça única — *a única coisa que muda é a face cruzar-se
/// a si própria*, que é exactamente o que este gate quer isolar.
fn um_quad(gravata: bool) -> Mesh {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let f = if gravata {
        Face::quad(0, 1, 3, 2)
    } else {
        Face::quad(0, 1, 2, 3)
    };
    Mesh::from_parts(verts, vec![f]).expect("a fixtura e' construida aqui")
}

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
        super::worse(&torta, 0, 0.0, &boa, 999, 89.0),
        "⛔ uma malha com faces auto-intersectadas e' PIOR que uma feia mas sa'"
    );
    assert!(
        !super::worse(&boa, 999, 89.0, &torta, 0, 0.0),
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
