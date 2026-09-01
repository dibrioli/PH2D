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
/// **decisão**, que é a única coisa que a env acrescenta ao caminho de sempre.
/// **UMA MEDIÇÃO DE PONTA VAZIA** — `tips = 0`, que a chave da amputação lê como *«não
/// medido»* e não como *«perfeito»*, logo não decide. ⚠️ É o valor com que os gates das
/// OUTRAS chaves entram: uma fixtura que trouxesse uma contagem passaria a testar a chave
/// nova por acidente.
fn sem() -> ph2d_quadfill::TipDeviation {
    ph2d_quadfill::TipDeviation::default()
}

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
        super::worse(&furada, 0, 0.0, sem(), &fechada, 999, 89.0, sem()),
        "⛔ os FUROS tem de vir antes do enviesamento"
    );
    // Empatados nos furos, decide a contagem de faces >60.
    assert!(
        super::worse(&fechada, 10, 0.0, sem(), &fechada, 2, 89.0, sem()),
        "⛔ empatados nos furos, decide o >60"
    );
    // Empatados nos dois, decide a mediana.
    assert!(
        super::worse(&fechada, 3, 9.0, sem(), &fechada, 3, 8.0, sem()),
        "⛔ empatados nos dois, decide a mediana"
    );
    assert!(
        !super::worse(&fechada, 3, 8.0, sem(), &fechada, 3, 8.0, sem()),
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
    let src = include_str!("sculpt3d_history_retopo_extract.rs");
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
    let full = concat!("ph2d_quadfill::", "finish_extracted(&mut out, &reference)");
    assert!(
        src.contains(full),
        "⛔⛔ o acabamento tem de pousar na `reference` (a ESCULTURA) e nao na `work` \
         (a remalhada)"
    );
    // ⛔⛔ **E o alisamento CRU não pode voltar por uma segunda porta.** Em 2026-08-28 o
    // Laplaciano passou a ser a *ronda zero* de `finish_extracted`; uma chamada solta aqui
    // seria um segundo acabamento a correr por cima do primeiro, e as duas passariam neste
    // ficheiro sem se verem.
    assert_eq!(
        src.matches(concat!("ph2d_quadfill::", "smooth(")).count(),
        0,
        "⛔ o alisamento cru voltou a este caminho -- ele vive dentro de `finish_extracted`"
    );
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
        super::worse(&tripla, 0, 0.0, sem(), &soltos, 999, 89.0, sem()),
        "⛔ uma aresta NAO-MANIFOLD tem de contar como furo -- o artista ve o mesmo \
         entalhe escuro que uma aresta de bordo lhe da'"
    );
}

/// ⭐⭐⭐ **GATE — a densidade SEGUE A FORMA, e a contagem não se mexe.**
///
/// ⛔ Report do artista (2026-08-28): *«as pontas finas, que deveriam ser relativamente mais
/// densas que as áreas lisas, têm menos densidade de faces e perdem detalhes»*. A régua é o
/// campo de passo que [`super::sizing_field`] entrega.
///
/// ⚠️ **As três metades do contrato, e a do meio é a que se esquece:** (1) com o knob a zero
/// o campo é **vazio** — a saída é a de sempre, e não «quase»; (2) com o knob a um o passo é
/// **menor onde a curvatura é maior**; (3) a **contagem prevista não muda**, senão o slider
/// que passou a pedir uma contagem volta a mentir.
#[test]
fn a_densidade_segue_a_curvatura_sem_mudar_a_contagem() {
    // ⚠️ Um toro: o tubo aperta e o buraco interior é chato. ⛔ Uma esfera tem curvatura
    // CONSTANTE — a fixtura não conteria o fenómeno.
    let work = ph2d_mesh::shapes::torus(64, 24, 1.0, 0.22);
    let target = ph2d_quadflow::edge_for_detail_by_count(&work, 0.5);

    // (1) O knob a zero é a AUSÊNCIA do campo, não um campo constante.
    assert!(
        super::sizing_field(&work, target, 0.0).is_empty(),
        "⛔ com `Follow Curvature` a zero o campo tem de ser VAZIO -- e' isso que faz o \
         passo ser o escalar de sempre"
    );

    let field = super::sizing_field(&work, target, 1.0);
    assert_eq!(field.len(), work.vert_count());

    // (2) Menor onde aperta: correlaciona o passo com a curvatura, por bandas.
    let curv = work.curvatures();
    let mut rows: Vec<(f32, f32)> = (0..work.vert_count())
        .map(|v| (curv[v].abs(), field[v]))
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let n = rows.len();
    let flat: f32 = rows[..n / 4].iter().map(|r| r.1).sum::<f32>() / (n / 4) as f32;
    let tight: f32 = rows[3 * n / 4..].iter().map(|r| r.1).sum::<f32>() / (n - 3 * n / 4) as f32;
    eprintln!(
        "[retopo] passo medio: chapado {flat:.5} · apertado {tight:.5} ({:.2}x) | alvo {target:.5}",
        flat / tight
    );
    assert!(
        tight < flat,
        "⛔ o passo tem de ser MENOR onde a curvatura e' maior (apertado {tight:.5}, \
         chapado {flat:.5})"
    );

    // (3) A contagem prevista não se mexe — a adaptação MOVE os quads, não os cria.
    let count = |h: &dyn Fn(usize) -> f32| -> f64 {
        let pos = work.positions();
        let mut acc = 0.0f64;
        for f in work.faces() {
            let v = f.verts();
            for k in 1..v.len() - 1 {
                let (a, b, c) = (
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                let (u, w) = (
                    [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                    [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
                );
                let nn = [
                    u[1].mul_add(w[2], -(u[2] * w[1])),
                    u[2].mul_add(w[0], -(u[0] * w[2])),
                    u[0].mul_add(w[1], -(u[1] * w[0])),
                ];
                let tri = f64::from(
                    nn[0]
                        .mul_add(nn[0], nn[1].mul_add(nn[1], nn[2] * nn[2]))
                        .sqrt(),
                ) * 0.5;
                let hh =
                    f64::from((h(v[0] as usize) + h(v[k] as usize) + h(v[k + 1] as usize)) / 3.0);
                acc += tri / (hh * hh);
            }
        }
        acc
    };
    let uniform = count(&|_| target);
    let adapted = count(&|v| field[v]);
    eprintln!("[retopo] contagem prevista: uniforme {uniform:.0} · adaptada {adapted:.0}");
    assert!(
        (adapted / uniform - 1.0).abs() <= 0.02,
        "⛔ a adaptacao mudou a CONTAGEM em {:.1} % (uniforme {uniform:.0}, adaptada \
         {adapted:.0}) -- ela move os quads, nao os cria, senao o slider volta a mentir",
        100.0 * (adapted / uniform - 1.0)
    );
}

/// ⭐⭐⭐ **O ALISAMENTO DO PEDIDO É EM LOG, e o gate é a MÉDIA GEOMÉTRICA.**
///
/// ⛔⛔ **É a única asserção que distingue as duas leis.** Um campo de duas metades
/// `{1, 4}` difundido até assentar converge para `2` se a média for **geométrica**
/// (log) e para `2,5` se for **aritmética** (linear) — e as duas passam em qualquer
/// gate que só olhe «o campo ficou mais uniforme».
///
/// ⚠️ **Por que log é a lei certa:** a grandeza que a cadeia consome é uma *razão*
/// de tamanhos (*«a ponta é METADE do corpo»*), não uma diferença. Alisar em linear
/// enviesa para o maior — a ponta subiria mais depressa do que o corpo desce, que é
/// o contrário do que o report do artista pede.
#[test]
fn o_alisamento_do_pedido_e_geometrico_e_nao_aritmetico() {
    // Um quadrado de 4 vértices: dois valem `1` e dois valem `4`.
    let mesh = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");

    let mut h = vec![1.0f32, 1.0, 4.0, 4.0];
    super::target::smooth_in_log(&mesh, &mut h, 200);

    let geo = 2.0f32; // √(1×4)
    let arit = 2.5f32; // (1+4)/2
    for v in &h {
        assert!(
            (v - geo).abs() < 0.05,
            "o campo assentou em {v:.3}; a media GEOMETRICA e' {geo} e a ARITMETICA {arit} \
             -- se ele assentar na aritmetica, o alisamento deixou de ser em log"
        );
    }
    assert!(
        (h[0] - arit).abs() > 0.4,
        "⛔ o CONTROLO: o valor tem de ficar LONGE da media aritmetica, senao este gate \
         nao distingue as duas leis"
    );
}

/// ⚠️ **Zero rondas é um no-op BYTE-IDÊNTICO** — a metade que mantém
/// `PH2D_SIZING_SMOOTH=0` a ser uma bissecção honesta.
#[test]
fn zero_rondas_de_alisamento_nao_mexe_no_pedido() {
    let mesh = ph2d_mesh::shapes::uv_sphere(12, 8, 1.0);
    let antes: Vec<f32> = (0..mesh.vert_count())
        .map(|i| 0.01f32.mul_add(i as f32, 0.05))
        .collect();
    let mut depois = antes.clone();
    super::target::smooth_in_log(&mesh, &mut depois, 0);
    assert_eq!(
        antes.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        depois.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "zero rondas tem de devolver os MESMOS BITS"
    );
}

/// ⭐⭐⭐ **O MEIO-PASSO É O QUE FAZ O ALISAMENTO CONVERGIR** — e este gate nasceu de
/// uma mutação que sobreviveu.
///
/// ⛔⛔ **A fixtura tem de ALTERNAR com a bipartição do grafo.** O anel de vértices
/// de um quad é um ciclo de comprimento `4`, que é **bipartido**: `{0, 2}` de um
/// lado, `{1, 3}` do outro. Um passo INTEIRO de Jacobi (`v ← média dos vizinhos`)
/// troca os dois lados a cada ronda e **oscila para sempre**; o meio-passo
/// (`v ← v + ½(média − v)`) contrai.
///
/// ⚠️ **O gate irmão, com `{1, 1, 4, 4}`, NÃO distingue os dois** — ali a partição
/// dos valores não coincide com a do grafo e o passo inteiro também converge. *Foi
/// exactamente essa mutação que sobreviveu, e a cura é a fixtura, não a lei.*
#[test]
fn o_alisamento_converge_mesmo_quando_o_pedido_alterna() {
    let mesh = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");

    // ⚠️ Os valores alternam AO LONGO DO ANEL — é isso que arma a oscilação.
    let mut h = vec![1.0f32, 4.0, 1.0, 4.0];
    super::target::smooth_in_log(&mesh, &mut h, 200);

    let span =
        h.iter().copied().fold(f32::MIN, f32::max) / h.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        span < 1.01,
        "⛔ o campo NAO assentou: ele ainda varia {span:.3}x depois de 200 rondas ({h:?}) \
         -- um passo INTEIRO sobre um grafo bipartido troca os dois lados para sempre"
    );
    for v in &h {
        assert!(
            (v - 2.0).abs() < 0.05,
            "assentou em {v:.3} e a media geometrica de 1 e 4 e' 2,0"
        );
    }
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
        src.contains("if adaptive > 0.0") && src.contains("still_broken(&out, dev)"),
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
