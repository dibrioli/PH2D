//! **Arch-gates da topologia dinâmica** (ADR-0150, W9.1).
//!
//! ⚠️ Eles leem a FONTE porque o que afirmam é *quem pergunta a quem, e em que
//! ordem* — e porque uma [`Sculpt3dScene`] não nasce sem `wgpu::Device`. A
//! metade que é geometria vive nos gates de unidade: o motor em
//! `ph2d-mesh::dyntopo_tests` e a lei do traço em
//! `ph2d-sculpt3d::stroke_tests::growing_the_stroke_keeps_the_frozen_base`.

use crate::sculpt_source;
use sculpt_source::{braced_block, branch_containing, function_body, sculpt_src, squeezed};

/// **REFINA E DEPOIS CARIMBA.**
///
/// ⚠️ A ordem inversa não falha — ela desenha: o barro cai na malha grossa e o
/// adensamento chega depois, então o traço fica com a silhueta do que a malha
/// **era**. Um dab de atraso é invisível num teste de contagem e óbvio na tela,
/// que é a pior combinação.
#[test]
fn the_refinement_runs_before_the_dab_lands() {
    let body = function_body(&sculpt_src(), "sculpt_at");
    let refine = body
        .find("refine_for_dab")
        .expect("o dab passa pela porta do refino");
    let stamp = body.find("self.stroke.dab(").expect("e carimba");
    assert!(
        refine < stamp,
        "refinar tem de vir ANTES de carimbar — invertido, o detalhe nasce um dab atrasado"
    );
}

/// **DESARMADO, o caminho do dab é o de sempre.**
///
/// ⚠️ A guarda é a PRIMEIRA linha da porta, e não um `if` no chamador: o dia em
/// que houver um segundo sítio de dab (um filtro, um script), quem esquecer a
/// pergunta herda a resposta certa.
#[test]
fn the_refinement_is_off_by_default_and_the_guard_is_the_first_question() {
    let src = sculpt_src();
    let body = function_body(&src, "refine_for_dab");
    let armed = body
        .find("self.dyntopo.armed")
        .expect("ela pergunta pelo arm");
    let engine = body
        .find("refine_in_sphere")
        .expect("e só então chama o motor");
    assert!(armed < engine, "o arm é perguntado ANTES do motor");

    // E o default é DESLIGADO — o aviso é dos autores do Blender, e está no
    // cabeçalho do módulo. ⚠️ A âncora é o `impl`, e não `fn default`: o
    // cluster tem vários `default() -> Self` e o primeiro é de outro tipo —
    // exatamente a doença que o `branch_containing` nasceu para curar.
    let dflt = braced_block(&src, "impl Default for Dyntopo");
    assert!(
        dflt.contains("armed: false"),
        "a topologia dinâmica nasce desarmada"
    );
}

/// **O TRAÇO SOBREVIVE AO REFINO — sem `begin`.**
///
/// ⚠️ Este é o gate que impede a regressão mais cara desta wave: chamar
/// `begin` depois de refinar parece a coisa óbvia (a malha mudou!) e é a doença
/// do produto-por-dab, que **não quebra nada visivelmente** — o traço só fica
/// mais forte quanto mais o refino disparar.
#[test]
fn refining_grows_the_stroke_instead_of_restarting_it() {
    let body = function_body(&sculpt_src(), "refine_for_dab");
    assert!(
        body.contains("self.stroke.grow_with("),
        "o traço é RE-DIMENSIONADO, que é o que preserva o `pre`"
    );
    // ⚠️ **E a PARENTELA atravessa junto.** Crescer sem ela deixaria cada
    // vértice novo entrar como nunca-visto, capturando como `pre` uma posição
    // que já contém o deslocamento deste traço — o dab soma outra vez e sai uma
    // agulha da altura do gesto (medido: 0,720 da aresta contra 0,053).
    assert!(
        body.contains("&mut births") && body.contains("&births"),
        "os nascimentos que o refino declara são os mesmos que o traço herda"
    );
    assert!(
        !body.contains("self.stroke.begin("),
        "e nunca RE-COMEÇADO: `begin` joga fora o `pre` e o traço passa a compor"
    );
    assert!(
        body.contains("self.mesh_rebuilt()"),
        "e a GPU precisa da malha inteira: há faces novas, que um upload \
         incremental por-vértice não descreve"
    );
}

/// **UM TRAÇO QUE MUDOU A TOPOLOGIA DESFAZ PELA MALHA INTEIRA.**
///
/// ⚠️ E a pergunta é sobre a CONTAGEM, não sobre o modo: armado e sem nada a
/// refinar (a malha já tem a densidade pedida ali), o traço é um traço comum, e
/// gastar uma cópia de documento nele seria pagar pelo modo em vez de pelo que
/// ele fez.
#[test]
fn a_stroke_that_changed_the_topology_undoes_by_the_whole_mesh() {
    // ⚠️ Comprimido: este gate JÁ falhou sobre produto correto quando um lint
    // trocou um `if` de duas condições por um `.filter(…)` e o `rustfmt` o
    // quebrou em quatro linhas. Ver `squeezed`.
    let body = squeezed(&function_body(&sculpt_src(), "close_stroke"));
    assert!(
        body.contains("self.dyn_before.take()"),
        "o fecho consome a foto do pen-down"
    );
    assert!(
        body.contains("vert_count()!=self.mesh().vert_count()"),
        "e decide pela CONTAGEM, não pelo arm"
    );
    assert!(
        body.contains("StrokeUndo::Remeshed"),
        "a entrada é a troca simétrica que o remesh já usa"
    );
    // A foto é tirada no pen-down, DEPOIS do `aim`: antes dele ela seria da
    // peça anterior.
    let down = sculpt_src();
    let aim = down.find("scene.aim(pos.0, pos.1);").expect("o aim");
    let open = down.find("scene.open_dyntopo_stroke();").expect("a foto");
    assert!(aim < open, "a foto é da peça que ESTE traço vai esculpir");
}

/// **AS DUAS TECLAS EXISTEM E DIZEM O QUE FIZERAM.**
///
/// ⚠️ O log não é enfeite aqui: ligar TRIANGULA a malha, e uma mudança calada é
/// a que o artista descobre no save. O mesmo vale para a recusa com a pilha de
/// multires montada — o remesh já tem essa lei, e silêncio a tornaria
/// indistinguível de uma tecla morta.
#[test]
fn arming_says_what_it_did_and_the_refusal_is_named() {
    let src = sculpt_src();
    let arm = branch_containing(&src, "scene.toggle_dyntopo()");
    assert!(arm.contains("K::KeyP"), "a topologia dinâmica tem tecla");
    assert!(
        arm.contains("trianguladas"),
        "e o log diz quantas faces a triangulação criou"
    );
    assert!(
        arm.contains("multires") && arm.contains("RECUSA"),
        "a recusa com a pilha montada é NOMEADA, não silenciosa"
    );
    let detail = branch_containing(&src, "scene.cycle_detail()");
    assert!(detail.contains("K::KeyU"), "o detalhe tem tecla própria");
}

/// ⭐⭐⭐ **O ALVO SAI DA PEÇA E NUNCA DO PINCEL, e a conta é feita UMA vez.**
///
/// ⛔⛔ **Este gate dizia o CONTRÁRIO até 2026-09-14** — chamava-se *«o alvo é
/// uma FRAÇÃO DO PINCEL»* e afirmava `edge_target(radius, …)`. Quem o desmentiu
/// foi o dono: *«a densidade da malha deve ser independente do zoom»*. E o
/// mecanismo é directo: o `Brush::radius` é **derivado do raio em PIXELS através
/// da câmera**, a cada dab, logo o zoom entrava no alvo. Medido na cena `=14`,
/// mesmo pincel e mesmo slider: alvo `0,0415` · `0,0953` · `0,2029` às
/// distâncias `1,5` · `3` · `6` — **`4,9×` só por aproximar ou afastar**.
///
/// ⭐ A cura é ancorar na **ÁREA DA SUPERFÍCIE**, que é a mesma que o botão de
/// retopologia já tinha escrita para o mesmo defeito (*o `Quad Size` absoluto,
/// refutado com foto*). ⇒ *o pincel diz ONDE, o slider diz QUÃO FINO.*
///
/// ⚠️ **A conta continua a ser UMA**: duas divergem no dia em que uma ganhar um
/// caso especial, e a forma como isso aparece é um log que diz um número e uma
/// geometria que usa outro.
#[test]
fn the_edge_target_comes_from_the_piece_never_from_the_brush() {
    let src = sculpt_src();
    let body = function_body(&src, "refine_for_dab");
    assert!(
        body.contains("edge_target_for_mesh("),
        "o alvo não sai da porta ancorada na peça — sem ela o zoom volta a \
         decidir a densidade"
    );
    assert!(
        !body.contains("edge_target(radius"),
        "o alvo voltou a sair do RAIO do pincel, que é derivado da câmera"
    );
    assert_eq!(
        src.matches("edge_target_for_mesh(").count(),
        1,
        "e há UMA chamada no cluster: uma segunda seria a segunda resposta"
    );
    // ⭐⭐ **E a ESCOLHA entre os dois sliders é feita numa porta só.** Ordem do
    // dono (14/09): o `Detail` da secção *Topology* governa a topologia dinâmica
    // e o das propriedades do pincel governa a densidade. ⚠️ **Dois sítios a
    // escolher entre dois sliders é como a tecla `U` passa a mexer no errado** —
    // e ela é o segundo consumidor desta porta.
    let escolha = function_body(&src, "detalhe_do_gesto");
    assert!(
        escolha.contains("brush.offers_density_controls()"),
        "a escolha entre os dois sliders não pergunta ao PINCEL — é a mesma \
         porta que o painel consulta para oferecer a pista, e duas respostas \
         dariam um slider visível a governar outra coisa"
    );
    // ⚠️ **E o PASSE não lê nenhum dos dois directamente:** ele pergunta à
    // porta. Um `self.dyntopo.detail` aqui dentro seria o pincel de densidade a
    // seguir o slider da topologia dinâmica — exactamente a partilha que a ordem
    // do dono desfez. ⛔ A régua é o CORPO da função e não o cluster: o ficheiro
    // inteiro tem leituras legítimas (a tecla `U` escreve, o retrato publica).
    assert!(
        !body.contains("self.dyntopo.detail") && !body.contains("brush.density_detail"),
        "o passe lê um dos dois sliders directamente — a escolha entre eles tem \
         de viver na porta, senão a tecla `U` e o passe podem discordar"
    );
}

/// **TODA CENA QUE EXISTE ARMA O MÓDULO.**
///
/// ⚠️ **Este gate nasceu de um canvas em BRANCO no smoke.** A `=14` tinha
/// predicado próprio, malha própria e roteiro próprio — e o `smoke_armed` era
/// uma ENUMERAÇÃO de `"1"…"13"`, então o módulo nunca armava. Cada peça estava
/// certa e o app abria preto, que é a forma mais cara de errar: nada aponta para
/// a lista que ficou para trás.
///
/// O gate lê os níveis que os PREDICADOS declaram (`Some("<n>")` no arquivo das
/// cenas) e exige que o `smoke_armed` responda `true` a cada um — sem tabela
/// própria, porque uma tabela aqui seria a segunda lista a apodrecer.
#[test]
fn every_scene_level_that_exists_arms_the_module() {
    let src = sculpt_source::family("scenes.rs");
    let levels: Vec<u32> = src
        .match_indices("Some(\"")
        .filter_map(|(at, _)| {
            let rest = &src[at + 6..];
            let end = rest.find('"')?;
            rest[..end].parse::<u32>().ok()
        })
        .collect();
    assert!(
        levels.len() >= 10,
        "o scanner tem de achar as cenas; achou {levels:?}"
    );
    for n in levels {
        // ⭐ A pergunta do PRODUTO, e não uma cópia dela: `arms` é pura e pública desde a auditoria
        // de arquitectura A2 (2026-09-12). A cópia existia porque `smoke_armed` lia o ambiente e era
        // `pub(crate)` — e escrever no ambiente é `unsafe` na edição 2024.
        assert!(
            ph2d_app_sculpt3d::scenes::arms(Some(&n.to_string())),
            "a cena =`{n}` existe e o módulo NÃO arma nela — o canvas abre em branco"
        );
    }
}

/// **E a lei que o gate acima interroga é um PARSE, nunca uma lista.**
///
/// ⚠️ O gate acima chama o produto (`arms`), então uma enumeração que cobrisse os níveis de HOJE
/// passava nele — e apodrecia na cena N+1. Esta metade proíbe a forma, não o resultado.
#[test]
fn the_arming_question_is_a_parse_and_not_a_list() {
    let body = squeezed(&function_body(&sculpt_src(), "arms"));
    assert!(
        body.contains("parse::<u32>()"),
        "armar é PERGUNTAR se o artista pediu uma cena"
    );
    assert!(
        !body.contains(r#"Some("1"|"#),
        "e nunca uma lista de níveis: ela apodrece no dia em que a cena N+1 nascer"
    );
}

/// ⭐⭐⭐ **A PORTA PERGUNTA AO VERBO, e não ao caminho que o gesto tomou.**
///
/// Até 2026-09-14 o refino tinha **um** chamador de produto — o braço do
/// carimbo —, logo a pergunta que o produto respondia era *«este gesto passou
/// pelo caminho do carimbo?»* e não *«este verbo cria superfície nova?»*. ⚠️ É a
/// mesma família de defeito que este módulo já pagou três vezes ao contrário:
/// *inferir uma propriedade do VERBO a partir do CAMINHO do gesto.*
///
/// ⛔ **Report do dono:** *«algumas tools que não deveriam fazer a subdivisão …
/// estão fazendo (como smooth) enquanto algumas que deveriam não estão»*. A
/// tabela inteira é pergunta de oráculo (`docs/3D/22`); a célula da **MÁSCARA**
/// não é, e é a que a cura fecha — medido, ela levava a peça de `830` para
/// `1 331` vértices num gesto que não move um único vértice.
///
/// As três metades:
/// 1. a porta **recebe** o verbo;
/// 2. ela lê as **duas** colunas (refino e colapso são leis independentes);
/// 3. o chamador passa o verbo do pincel **armado**, nunca um literal.
#[test]
fn the_dyntopo_door_asks_the_verb() {
    let src = sculpt_src();
    let body = function_body(&src, "refine_for_dab");
    assert!(
        body.contains("verbo.refina_no_dyntopo()") && body.contains("verbo.colapsa_no_dyntopo()"),
        "a porta do dyntopo não consulta as DUAS colunas do verbo — sem isso ela \
         responde «este gesto passou pelo carimbo?», que é a pergunta errada"
    );
    // ⚠️ **O chamador passa o verbo E o ajuste do pincel ARMADO** (`brush.*`), e
    // não os campos de `self.brush`: entre os dois está o `armed_brush`, que é
    // quem resolve os modificadores do gesto. *Duas respostas para «o que está
    // na mão» divergem no dia do primeiro modificador que troca de verbo.*
    // ⚠️ **Desde 14/09 ele passa o PINCEL inteiro**, e não o verbo solto: a porta
    // precisa de escolher entre os DOIS sliders de `Detail` (o da cena e o do
    // pincel de densidade), e um verbo solto não sabe responder a isso.
    assert!(
        src.contains("self.refine_for_dab(&brush, hit.point)"),
        "o chamador não passa o pincel armado à porta do dyntopo"
    );
}

/// ⭐⭐⭐ **AS TRÊS RAZÕES DO SILÊNCIO SÃO TRÊS, E CADA UMA É DITA.**
///
/// ⛔ **Report do dono, 2026-09-14: *«não vejo efeito com density»*.** O pincel
/// estava certo; o que faltava era ele DIZER porque não fez nada. Um verbo cujo
/// efeito inteiro é sobre a topologia **parece partido** sempre que o passe não
/// corre, e as três razões — o modo desligado · a pilha de multiresolução
/// montada · não haver aresta fora da faixa — **o artista vê iguais**: nada
/// acontece.
///
/// ⚠️ **A régua é a CONTAGEM de chamadas, e não «a função existe»:** apagar uma
/// das três deixaria as outras duas a funcionar e a terceira muda, que é
/// exactamente a forma que este report tem.
#[test]
fn the_dyntopo_pass_names_every_reason_it_did_nothing() {
    let src = sculpt_src();
    let body = function_body(&src, "refine_for_dab");
    let queixas = body.matches("self.queixa_do_passe(").count();
    assert_eq!(
        queixas, 2,
        "a porta do dyntopo tem {queixas} queixas e as razões do silêncio são \
         DUAS (pilha montada · a malha já está no ponto pedido) — a que faltar é \
         um pincel que parece partido e não diz porquê"
    );
    // ⚠️⚠️ **Eram TRÊS e passaram a DUAS em 2026-09-14, e a que saiu não foi
    // apagada: ela ficou INALCANÇÁVEL.** Por ordem do dono o pincel de densidade
    // corre sem o interruptor, e a queixa só falava por quem não tem lei
    // por-vértice — ou seja, exactamente por quem já não passa naquele ramo.
    // *Uma queixa que ninguém pode disparar faz o censo dizer três onde a
    // verdade é duas.*
    assert!(
        !body.contains("DESLIGADA"),
        "a queixa do modo desligado voltou — ela é inalcançável desde que a \
         densidade corre sem o interruptor"
    );
    // ⚠️ E ela é **por traço e não por dab** — um dab corre por movimento do
    // ponteiro, e uma queixa por dab é um log que ninguém lê.
    assert!(
        src.contains("dyn_queixa_dita"),
        "a queixa não tem a trava de uma-por-traço"
    );
}
