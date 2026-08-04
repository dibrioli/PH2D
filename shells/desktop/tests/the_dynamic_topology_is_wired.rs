//! **Arch-gates da topologia dinâmica** (ADR-0150, W9.1).
//!
//! ⚠️ Eles leem a FONTE porque o que afirmam é *quem pergunta a quem, e em que
//! ordem* — e porque uma [`Sculpt3dScene`] não nasce sem `wgpu::Device`. A
//! metade que é geometria vive nos gates de unidade: o motor em
//! `ph2d-mesh::dyntopo_tests` e a lei do traço em
//! `ph2d-sculpt3d::stroke_tests::growing_the_stroke_keeps_the_frozen_base`.

mod sculpt_source;
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
        body.contains("self.stroke.grow_to("),
        "o traço é RE-DIMENSIONADO, que é o que preserva o `pre`"
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

/// **O ALVO É UMA FRAÇÃO DO PINCEL, e a conta é feita UMA vez.**
///
/// ⚠️ Duas contas divergem no dia em que uma delas ganhar um caso especial, e a
/// forma como isso aparece é um log que diz um número e uma geometria que usa
/// outro.
#[test]
fn the_edge_target_is_derived_once_from_the_brush_radius() {
    let src = sculpt_src();
    let body = function_body(&src, "refine_for_dab");
    assert!(
        body.contains("edge_target(radius, self.dyntopo.detail)"),
        "o alvo sai da porta única, contra o raio do dab"
    );
    assert_eq!(
        src.matches("edge_target(").count(),
        1,
        "e há UMA chamada no cluster: uma segunda seria a segunda resposta"
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
    let src = sculpt_source::source("sculpt3d_scenes.rs");
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
        // SAFETY-free: o teste roda numa thread só e a var é lida logo abaixo.
        unsafe { std::env::set_var("PH2D_SCULPT3D_SMOKE", n.to_string()) };
        assert!(
            ph2d_host_desktop_smoke_armed(),
            "a cena =`{n}` existe e o módulo NÃO arma nela — o canvas abre em branco"
        );
    }
    unsafe { std::env::remove_var("PH2D_SCULPT3D_SMOKE") };
}

/// A cópia da pergunta que o produto faz. ⚠️ Ela é uma CÓPIA porque
/// `smoke_armed` é `pub(crate)` e um teste de integração não a alcança — e é
/// por isso que o gate irmão abaixo afirma que as duas dizem a mesma coisa.
fn ph2d_host_desktop_smoke_armed() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .is_some_and(|n| n >= 1)
}

/// **E a cópia acima não pode divergir do produto.**
///
/// ⚠️ Sem esta metade o gate anterior mede a si mesmo: ele ficaria verde com o
/// produto de volta na enumeração, porque a cópia responderia `true` sozinha.
#[test]
fn the_arming_question_is_a_parse_and_not_a_list() {
    let body = squeezed(&function_body(&sculpt_src(), "smoke_armed"));
    assert!(
        body.contains("parse::<u32>()"),
        "armar é PERGUNTAR se o artista pediu uma cena"
    );
    assert!(
        !body.contains(r#"Some("1"|"#),
        "e nunca uma lista de níveis: ela apodrece no dia em que a cena N+1 nascer"
    );
}
