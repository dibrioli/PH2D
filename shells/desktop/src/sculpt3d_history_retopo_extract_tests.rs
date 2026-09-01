//! ⭐ **OS GATES do caminho da EXTRACÇÃO** — o irmão de
//! [`sculpt3d_history_retopo_extract`].
//!
//! ⚠️ **Ele existe por causa do tecto de LOC do shell (HR-18)**, e o corte é o idioma da
//! casa: o `mod tests` inline vai para o irmão **do assunto**. *O ficheiro do produto
//! guarda o que o botão faz; este guarda o que se prova sobre ele.*
//!
//! ⚠️ **Sem `use super::*`, de propósito:** todo gate aqui chama o irmão pelo prefixo
//! (`super::extract_from`, `super::boundary_edges`, `super::worse`), então o glob era
//! morto — e um `unused_imports` é **erro** sob o `-D warnings` do `ship.sh`, não aviso.

/// ⭐⭐⭐ **GATE 11 — o caminho antigo continua byte-idêntico enquanto o
/// interruptor estiver desligado.**
///
/// ⚠️ **A decisão é pura de propósito.** O gesto em si precisa de GPU (a cena
/// segura buffers de device), então um gate sobre ele é `skip` gracioso na
/// máquina sem adapter — e *skip gracioso não é verde*. O que se pina aqui é a
/// **UMA MEDIÇÃO DE PONTA VAZIA** — `tips = 0`, que a chave da amputação lê como *«não
/// medido»* e não como *«perfeito»*, logo não decide. ⚠️ É o valor com que os gates das
/// OUTRAS chaves entram: uma fixtura que trouxesse uma contagem passaria a testar a chave
/// nova por acidente.
fn sem() -> ph2d_quadfill::TipDeviation {
    ph2d_quadfill::TipDeviation::default()
}

/// **UMA MEDIÇÃO DE DENSIDADE DA PONTA VAZIA** — a irmã da [`sem`] para a 5.ª chave
/// (2026-09-01). ⚠️ Pela mesma razão: `tips = 0` é *«não medido»*, logo a chave da grade na
/// ponta não decide, e os gates das outras chaves continuam a medir o que sempre mediram.
pub(super) fn sem_den() -> ph2d_quadfill::TipDensity {
    ph2d_quadfill::TipDensity::default()
}

/// ⭐⭐⭐ **GATE 11 — o caminho antigo continua byte-idêntico enquanto o
/// interruptor estiver desligado.**
///
/// ⚠️ **A decisão é pura de propósito.** O gesto em si precisa de GPU (a cena
/// segura buffers de device), então um gate sobre ele é `skip` gracioso na
/// máquina sem adapter — e *skip gracioso não é verde*. O que se pina aqui é a
/// **decisão**, que é a única coisa que a env acrescenta ao caminho de sempre.
#[test]
fn o_caminho_novo_e_o_de_omissao_e_so_o_zero_o_desliga() {
    for (value, want) in [
        // ⭐⭐ O caso por omissão VIROU em 2026-08-25 (ordem do dono do produto): é o
        // caminho NOVO que o Enio recebe sem configurar nada. *A lei «shipa
        // desligado» valeu enquanto ele não fechava a casca; ele fecha.*
        (None, true),
        // ⚠️ E o `"0"` é a ÚNICA palavra que desliga — quem quer o de sempre tem de
        // o pedir por este nome exacto.
        (Some("0"), false),
        (Some("1"), true),
        (Some("sim"), true),
        (Some(""), true),
    ] {
        assert_eq!(
            super::extract_from(value),
            want,
            "PH2D_RETOPO_EXTRACT={value:?} tinha de dar {want}"
        );
    }
}

/// ⭐⭐⭐ **A ORDEM DO CRITÉRIO: furos primeiro, e ela é a decisão de produto.**
///
/// ⛔⛔ Uma ordem que pusesse o enviesamento à frente escolheria *a peça mais bonita com
/// um buraco na ponta* — e «furos nas pontas» foi a queixa do artista **três vezes
/// seguidas**. ⚠️ *Nada no tipo impede trocar a ordem: são três números da mesma peça.*
#[test]
fn a_escolha_poe_os_furos_a_frente_do_enviesamento() {
    // Uma peça FECHADA e uma com bordo — o cubo de quads da casa, e um quad solto.
    let fechada = ph2d_mesh::shapes::cube(1.0);
    let furada = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");
    assert_eq!(
        super::boundary_edges(&fechada),
        0,
        "⛔ a fixtura fechada tem de FECHAR, senao o gate compara duas peças furadas"
    );
    assert_eq!(
        super::boundary_edges(&furada),
        4,
        "⛔ a fixtura furada tem de CONTER o fenomeno"
    );

    // A furada e' PIOR mesmo com enviesamento perfeito contra uma fechada horrivel.
    assert!(
        super::worse(
            &furada,
            0,
            0.0,
            sem(),
            sem_den(),
            &fechada,
            999,
            89.0,
            sem(),
            sem_den()
        ),
        "⛔ os FUROS tem de vir antes do enviesamento"
    );
    // Empatados nos furos, decide a contagem de faces >60.
    assert!(
        super::worse(
            &fechada,
            10,
            0.0,
            sem(),
            sem_den(),
            &fechada,
            2,
            89.0,
            sem(),
            sem_den()
        ),
        "⛔ empatados nos furos, decide o >60"
    );
    // Empatados nos dois, decide a mediana.
    assert!(
        super::worse(
            &fechada,
            3,
            9.0,
            sem(),
            sem_den(),
            &fechada,
            3,
            8.0,
            sem(),
            sem_den()
        ),
        "⛔ empatados nos dois, decide a mediana"
    );
    assert!(
        !super::worse(
            &fechada,
            3,
            8.0,
            sem(),
            sem_den(),
            &fechada,
            3,
            8.0,
            sem(),
            sem_den()
        ),
        "⛔ iguais nao podem ser PIORES -- a comparacao tem de ser estrita"
    );
}

/// ⭐⭐⭐ **O CAMINHO DA EXTRACÇÃO TEM ACABAMENTO — e ele pousa na ESCULTURA.**
///
/// ⛔⛔ **As duas metades são precisas, e a segunda defende o defeito que já custou o
/// produto inteiro.** Em 2026-08-21 a porta do shell passou ao `fill` a malha original
/// onde ele esperava a **indexada**, e os quatro números do relatório saíram
/// **bit-a-bit iguais** aos da corrida correta — o dano era só geométrico. Aqui a
/// direcção é a oposta e o erro seria o mesmo: alisar contra a `work` (a remalhada)
/// somaria os dois erros e apagaria o relevo que o F1 já arredondou.
///
/// ⚠️ **O gate LÊ O FONTE** pela mesma razão que o irmão dele abaixo: um alisamento que
/// desapareça, ou que troque de superfície, compila e passa a suíte inteira.
#[test]
fn a_extraccao_alisa_contra_a_escultura_e_nao_contra_a_remalhada() {
    // ⚠️ **O ficheiro medido mudou em 2026-09-01** — a tentativa passou para o irmão
    // [`super::one`] quando o tecto de LOC estourou (`694`). *Um gate que lê o fonte tem de
    // seguir o fonte, e é por isso que ele reprovou nesse corte em vez de ficar mudo.*
    let src = include_str!("sculpt3d_retopo_one.rs");
    // ⚠️ **O token vem partido de propósito:** este gate lê o ficheiro em que ele
    // próprio vive, e um literal inteiro contar-se-ia a si mesmo. *Um gate que se conta
    // nunca mede o produto.*
    let call = concat!("ph2d_quadfill::", "finish_extracted(");
    let n = src.matches(call).count();
    assert_eq!(
        n, 1,
        "o caminho da extraccao chama o acabamento {n} vezes; tem de ser UMA -- ver o \
         doc do `ph2d_quadfill::fill` e o defeito de 2026-08-21"
    );
    let full = concat!(
        "ph2d_quadfill::",
        "finish_extracted(&mut out, cx.reference)"
    );
    assert!(
        src.contains(full),
        "⛔⛔ o acabamento tem de pousar na `reference` (a ESCULTURA) e nao na `work` \
         (a remalhada)"
    );
    // ⛔⛔ **E o alisamento CRU não pode voltar por uma segunda porta.** Em 2026-08-28 o
    // Laplaciano passou a ser a *ronda zero* de `finish_extracted`; uma chamada solta aqui
    // seria um segundo acabamento a correr por cima do primeiro, e as duas passariam neste
    // ficheiro sem se verem.
    // ⛔ **Os DOIS ficheiros**, desde o corte: o alisamento cru não pode voltar por nenhum.
    for (nome, texto) in [
        ("one", src),
        (
            "extract",
            include_str!("sculpt3d_history_retopo_extract.rs") as &str,
        ),
    ] {
        assert_eq!(
            texto.matches(concat!("ph2d_quadfill::", "smooth(")).count(),
            0,
            "⛔ o alisamento cru voltou ao `{nome}` -- ele vive dentro de `finish_extracted`"
        );
    }
}

/// ⭐⭐ **E A BIFURCAÇÃO É UMA SÓ** — o que faz o «byte-idêntico» ser
/// verificável em vez de prometido.
///
/// ⚠️ **O gate LÊ O FONTE**, e é de propósito: um segundo sítio a chamar
/// [`super::extract_requested`] compilaria, passaria a suíte, e partiria a
/// afirmação de que o caminho antigo está intocado. *Uma promessa sobre o
/// código não é uma propriedade do código até alguém a contar.*
#[test]
fn a_bifurcacao_para_o_caminho_novo_e_uma_so() {
    let src = include_str!("sculpt3d_history_retopo_global.rs");
    let n = src.matches("extract_requested()").count();
    assert_eq!(
        n, 1,
        "a cadeia global chama `extract_requested()` {n} vezes; tem de ser UMA, \
         na primeira linha da porta"
    );
    assert_eq!(
        src.matches("quad_remesh_extract(").count(),
        1,
        "e chama o caminho novo uma vez so'"
    );
}

/// ⭐⭐⭐ **GATE — «furo» conta as DUAS formas de a casca não fechar.**
///
/// ⛔⛔ Até 2026-08-28 a chave da frente de [`super::worse`] contava só as arestas de
/// **bordo**. Uma aresta **não-manifold** — três faces a tocá-la — passava invisível, e
/// o artista vê o mesmo entalhe escuro nos dois casos: o ficheiro que ele exportou nesse
/// dia tinha `19 786` quads impecáveis e **`2` arestas não-manifold** num ponto só.
///
/// ⚠️ **As duas fixturas têm o MESMO número de arestas de bordo**, e é isso que faz este
/// gate discriminar: sob a lei antiga elas **empatam** na chave da frente e a escolha cai
/// para o enviesamento — que é dado aqui **perfeito na peça não-manifold e péssimo na
/// limpa**, exactamente o desempate que produzia a peça furada. *Um gate cujas fixturas
/// diferem em duas coisas não prova nada sobre nenhuma delas.*
#[test]
fn a_escolha_ve_a_aresta_nao_manifold_e_nao_so_o_bordo() {
    // ⭐ TRÊS quads a partilhar a aresta `(0,1)` — 9 arestas de bordo e 1 não-manifold.
    let tripla = ph2d_mesh::Mesh::from_parts(
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
            ph2d_mesh::Face::quad(0, 1, 2, 3),
            ph2d_mesh::Face::quad(1, 0, 5, 4),
            ph2d_mesh::Face::quad(0, 1, 6, 7),
        ],
    )
    .expect("a fixtura e' construida aqui");
    // ⭐ TRÊS triângulos soltos — 9 arestas de bordo e ZERO não-manifold.
    let soltos = ph2d_mesh::Mesh::from_parts(
        (0..9)
            .map(|i| [i as f32, (i % 3) as f32, 0.0])
            .collect::<Vec<_>>(),
        vec![
            ph2d_mesh::Face::tri(0, 1, 2),
            ph2d_mesh::Face::tri(3, 4, 5),
            ph2d_mesh::Face::tri(6, 7, 8),
        ],
    )
    .expect("a fixtura e' construida aqui");

    // ⛔ O CONTROLE: sob a régua ANTIGA as duas são indistinguíveis.
    assert_eq!(
        super::boundary_edges(&tripla),
        super::boundary_edges(&soltos),
        "⛔ as duas fixturas tem de EMPATAR no bordo, senao este gate nao isola a \
         aresta nao-manifold"
    );
    assert_eq!(
        super::rulers::open_edges(&tripla),
        super::boundary_edges(&tripla) + 1,
        "⛔ a fixtura tripla tem de CONTER exactamente uma aresta nao-manifold"
    );
    assert_eq!(
        super::rulers::open_edges(&soltos),
        super::boundary_edges(&soltos),
        "⛔ a fixtura de triangulos soltos nao pode ter nenhuma"
    );

    // ⭐⭐ E a escolha: a não-manifold perde **mesmo com a forma perfeita** contra uma
    // limpa horrível. Sob a lei antiga esta asserção é FALSA — o empate no bordo levava
    // a decisão para o `>60`, e ali a não-manifold ganhava.
    assert!(
        super::worse(
            &tripla,
            0,
            0.0,
            sem(),
            sem_den(),
            &soltos,
            999,
            89.0,
            sem(),
            sem_den()
        ),
        "⛔ uma aresta NAO-MANIFOLD tem de contar como furo -- o artista ve o mesmo \
         entalhe escuro que uma aresta de bordo lhe da'"
    );
}

/// ⭐⭐⭐ **O CAMPO ADAPTATIVO TEM DE PODER PERDER** — e o gate lê o FONTE.
///
/// ⛔⛔⛔ **Report do artista, 2026-08-30, com foto: «praticamente uma regressão».** No
/// `Detail` de FÁBRICA (`0,50`) o `Follow Curvature = 1` levava a peça dele de
/// `χ = 2 · 0 bordo` para `χ = 1 · 4 bordo` — **furos onde não havia**.
///
/// ⚠️⚠️ **A wave que o introduziu mediu a `Detail 0,85`, onde fica limpo.** *Afinar e
/// validar num ponto do slider que não é o de fábrica é medir a configuração que
/// ninguém usa.*
///
/// ⚠️ **Por que o gate lê o fonte:** a guarda é uma **ausência de dano** ao fim de uma
/// cadeia que precisa da malha inteira; um gate que a medisse teria de correr o botão
/// duas vezes por fixtura. E o modo de falha é apagar a recaída — que compila, passa a
/// suíte inteira, e devolve os furos.
///
/// ⚠️ **A 2.ª asserção é a que a 1.ª versão da cura falhou:** ela pedia **uma** candidata
/// sem campo, e a peça continuou com `4` bordo. *A linha de base não é uma corrida, são
/// duas* — a alinhada e a suave — e é o `worse` entre elas que dá a malha limpa.
///
/// ⚠️ **A condição ALARGOU em 2026-08-30 e este gate seguiu-a:** ela era `open_edges(&out) > 0`
/// e passou a [`super::still_broken`] (bordo **ou** face cruzada sobre si própria). *Um gate que
/// nomeia a guarda pelo TEXTO dela reprova quando o texto muda — e é para isso que ele serve:
/// obrigar quem mexe na guarda a vir aqui dizer que sabia.*
#[test]
fn o_campo_adaptativo_recua_quando_abre_a_malha() {
    let src = include_str!("sculpt3d_history_retopo_extract.rs");
    assert!(
        src.contains("if adaptive > 0.0") && src.contains("still_broken(&out, dev, den)"),
        "⛔ a recaida do campo adaptativo desapareceu: sem ela o `Follow Curvature` volta a \
         poder abrir furos que o caminho de omissao nao tem"
    );
    let (_, recaida) = src
        .split_once("let uniforme = if adaptive > 0.0")
        .expect("a recaida tem de existir e chamar-se assim");
    // ⚠️ **A janela subiu de `1200` para `1600` em 2026-09-01, e não é afrouxar:** as quatro
    // chamadas de `guarded` desta recaída ganharam o argumento da **cerca de viagem**
    // (`ph2d_quadfill::EXTRACT_TRAVEL`), o que alonga o bloco em ~`160` bytes sem lhe mudar a
    // forma. ⛔ A janela tem de conter as DUAS corridas dos DOIS ramos — se ela as cortasse, as
    // contagens abaixo liam `1` e `1` e o gate reprovava sobre código correcto.
    let recaida = &recaida[..recaida.len().min(1600)];
    // ⚠️ **CONTAGEM e não `contains`** — a 1.ª versão deste gate perguntava se as duas
    // candidatas *apareciam*, e a mutação que apagava metade da corrida **sobreviveu**:
    // o ramo SERIAL (`PH2D_RETOPO_SERIAL=1`) tem as mesmas duas linhas, então a string
    // continuava lá. *Um `contains` sobre um fonte com dois ramos mede o ramo que sobrou.*
    let alinhadas = recaida.matches("ALIGN_WEIGHT").count();
    let suaves = recaida.matches("guarded(0.0, false, 0.0,").count();
    assert!(
        alinhadas >= 2 && suaves >= 2,
        "⛔ a recaida tem de correr a CORRIDA INTEIRA (a alinhada E a suave) nos DOIS ramos \
         -- achei {alinhadas} alinhada(s) e {suaves} suave(s). A 1.a versao pediu so' uma \
         candidata e a peca do artista ficou com 4 bordo"
    );
    assert!(
        recaida.contains("worse("),
        "⛔ a recaida tem de decidir pelo mesmo `worse`: uma candidata que entra sem passar \
         por ele pode PIORAR a escolha, e a garantia inteira era que ela so' pode nao vencer"
    );
    // ⛔⛔ **E o SENTIDO do `worse`**, que a mutação `!worse(...)` sobreviveu a expor: um
    // `contains("worse(")` casa igualmente bem com a decisão **invertida**, que trocaria a
    // malha limpa pela esburacada em todas as peças. *Um gate textual sobre uma condição
    // não vê a negação dela.*
    // ⚠️ **A varredura é sobre o FICHEIRO INTEIRO e não sobre a fatia**: a 1.ª versão
    // olhava `recaida[..1200]`, que **não alcançava** a linha da decisão, e a mutação
    // sobreviveu. ⛔ **E ela NÃO descasca comentários** — se alguém documentar a cura no
    // ficheiro do produto escrevendo esse token, este gate fica vermelho sobre código
    // correcto ([[feedback-a-textual-gate-must-strip-comments-or-documenting-the-cure-fails-it]]).
    // *A cura, nesse dia, é descascar — não afrouxar.*
    assert!(
        !src.contains("!worse("),
        "⛔ a recaida esta' INVERTIDA: com `!worse` ela adopta a candidata uniforme \
         exactamente quando a adaptativa era melhor"
    );
}

/// ⭐⭐⭐ **O BOTÃO DIZ QUANDO AMPUTOU** — e esta é a única coluna do relatório que o
/// artista não consegue derivar de nenhuma outra.
///
/// ⛔⛔ **Foto do Enio, 2026-08-30: uma seta VERDE e uma VERMELHA na mesma peça.** A
/// amputação sai com casca fechada, `χ = 2`, `100 %` de quads e quads bonitos; o
/// alcance global é um **extremo único**, e na peça dele ele lia `−16,2 %` enquanto
/// **dez** das doze pontas estavam a `−0,1 %` e **duas** tinham perdido `20 %`.
/// *Ele descobria-a fotografando o ecrã — três vezes.*
///
/// ⚠️ **E a frase tem de dizer O QUE FAZER:** a causa é resolução (a célula ficou mais
/// grossa que a ponta), e o `Detail` que a cura não se anuncia sozinho.
#[test]
fn a_linha_do_artista_nomeia_as_pontas_amputadas() {
    let base = crate::sculpt3d::history::remesh::QuadRemeshReport {
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
        shape: ph2d_quadfill::QuadShape {
            aspect_p50: 1.0,
            skew_p50: 1.0,
            ..ph2d_quadfill::QuadShape::default()
        },
        aligned: true,
        measured: true,
        mirrored: 0,
        doublets: 0,
        folded: 0,
        tips_cut: 0,
        tips_total: 12,
        tips_worst_pct: -0.4,
        coverage_shell_p50: 0.0,
        coverage_shell_worst: 0.0,
        coverage_samples: 500,
    };
    let calada = crate::sculpt3d::history::retopo_global::retopo_line(&base);
    assert!(
        !calada.to_lowercase().contains("amputad"),
        "⛔ sem ponta cortada a linha tem de ser CALADA, senao a palavra perde o peso: {calada}"
    );

    let acusa = crate::sculpt3d::history::retopo_global::retopo_line(
        &crate::sculpt3d::history::remesh::QuadRemeshReport {
            tips_cut: 2,
            // ⚠️ Valor SEM ambiguidade de arredondamento: a linha imprime com `{:.0}`, e
            // `−21,5` sai `−22` — a 1.ª versão deste gate procurava `−21` e reprovou.
            tips_worst_pct: -21.0,
            ..base
        },
    );
    assert!(
        acusa.contains("AMPUTADA"),
        "⛔ com 2 pontas cortadas a linha tem de o DIZER: {acusa}"
    );
    // ⚠️ **O DENOMINADOR e a PIOR PERDA vão junto** — «2 amputadas» significa coisas
    // opostas se a peça tem 2 pontas ou 12, e `−3 %` não é o mesmo report que `−21 %`.
    assert!(
        acusa.contains("de 12") && acusa.contains("-21"),
        "⛔ a frase tem de trazer o denominador e a pior perda: {acusa}"
    );
    // ⭐ **E tem de dizer O QUE FAZER.** A causa é resolução, e sem isto o artista
    // recebe um diagnóstico sem acção.
    assert!(
        acusa.contains("Detail") && acusa.contains("Follow Curvature"),
        "⛔ a frase tem de nomear as duas alavancas: {acusa}"
    );
}
