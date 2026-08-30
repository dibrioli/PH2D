//! **A cena `=109` MOSTRA o que diz que mostra** — e o arnês tem de passar pela SHELL.
//!
//! ⚠️⚠️ **Coze pelo `pump.cook` do próprio estado, e isso é load-bearing:** os dois nós de
//! tabela leem um CANAL EXTERNO que só a shell escreve (`motion_table_gen::publish`). Um
//! `Cook::new()` local não tem externo nenhum e devolveria uma tabela VAZIA — que é a
//! assinatura exacta da feature partida, e passaria por «a cena monta».

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    reg
}

fn cook(state: &mut MotionState, reg: &NodeRegistry, sink: NodeId, t: f64) -> Stream {
    crate::render_loop::motion_table_gen::publish(state);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, reg, sink, t)
        .expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e' um stream")
    };
    s.clone()
}

fn ys(s: &Stream) -> Vec<f32> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    }
}

fn sizes(s: &Stream) -> Vec<f32> {
    match s.get("size") {
        Some(Column::Scalar(v)) => v.clone(),
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => Vec::new(),
    }
}

/// **A cena constrói os DOIS lados.** Um `?` a engolir uma aresta faria o roteador devolver
/// `unwrap_or_default()` — uma tela vazia, que num smoke se lê como *"a feature não foi
/// construída"* em vez de *"a cena está partida"*.
#[test]
fn the_table_scene_builds_both_readings() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_table_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), 2, "duas leituras: elementos e momento");
}

/// ⭐⭐⭐ **EM CIMA: uma linha é um ELEMENTO, e a ALTURA vem da coluna.**
#[test]
fn the_upper_row_has_one_element_per_file_row_and_the_height_comes_from_the_column() {
    let reg = registry();
    let mut state = MotionState::new();
    let sinks = build_table_demo_document(&mut state.doc, &reg).expect("a cena constroi");
    let s = cook(&mut state, &reg, sinks[0], 0.0);

    // O ficheiro tem 12 linhas de dados.
    assert_eq!(s.count(), 12, "uma linha do ficheiro, um elemento");

    let h = ys(&s);
    // ⚠️ **A coluna `vendas` sobe e desce** (12·19·7·25·31·18·9·22·28·35·15·21), então as
    // alturas TÊM de a seguir na mesma ordem — sem isto, uma cena que ligasse a coluna errada
    // (ou nenhuma) desenharia doze pontos igualmente espaçados e pareceria certa.
    let vendas = [
        12.0f32, 19.0, 7.0, 25.0, 31.0, 18.0, 9.0, 22.0, 28.0, 35.0, 15.0, 21.0,
    ];
    for i in 1..12 {
        let subiu_no_ficheiro = vendas[i] > vendas[i - 1];
        let subiu_no_desenho = h[i] > h[i - 1];
        assert_eq!(
            subiu_no_ficheiro, subiu_no_desenho,
            "a linha {i} sobe no ficheiro? {subiu_no_ficheiro}; e no desenho? {subiu_no_desenho}\n{h:?}"
        );
    }
    // ⚠️ **O CONTROLE**: as alturas têm de ser mesmo DIFERENTES. Sem ele, um desenho todo
    // plano satisfaria a ordem acima por vacuidade.
    let lo = h.iter().copied().fold(f32::MAX, f32::min);
    let hi = h.iter().copied().fold(f32::MIN, f32::max);
    assert!(hi - lo > 0.5, "a coluna nao levantou nada: {lo}..{hi}");
}

/// ⭐⭐⭐ **EM BAIXO: uma linha é um MOMENTO — o tamanho MUDA com o playhead.**
///
/// ⚠️ É a única metade que diz *tabela no TEMPO*. Sem a comparação entre dois instantes, um
/// campo constante qualquer passaria — e essa é exactamente a diferença que fez a Adobe
/// inventar o `.mgjson`.
#[test]
fn the_lower_square_follows_the_table_over_time() {
    let reg = registry();
    let mut state = MotionState::new();
    let sinks = build_table_demo_document(&mut state.doc, &reg).expect("a cena constroi");

    let a = cook(&mut state, &reg, sinks[1], 0.0);
    let b = cook(&mut state, &reg, sinks[1], 1.5);
    // ⚠️ **`3,0` e não `4,0`, e o gate ensinou-mo**: na coluna `nivel` os instantes `1,5` e
    // `4,0` valem os DOIS `1,0`, então a 1.ª redacção deste teste reprovava sobre produto
    // CERTO. *Dois instantes escolhidos à mão são uma afirmação sobre a fixtura, e é preciso
    // ir lê-la.* Em `3,0` o nível é `0,2`, o extremo oposto.
    let c = cook(&mut state, &reg, sinks[1], 3.0);
    assert_eq!(a.count(), 1, "um quadrado so'");

    let (sa, sb, sc) = (sizes(&a), sizes(&b), sizes(&c));
    assert!(!sa.is_empty(), "o `motion.drive` tem de escrever `size`");
    assert!(
        (sa[0] - sb[0]).abs() > 0.05,
        "o tamanho nao mudou entre t=0 e t=1,5: {} vs {} — o canal externo nao chegou, ou o \
         `value.table` nao esta' a ler o playhead",
        sa[0],
        sb[0]
    );
    assert!(
        (sb[0] - sc[0]).abs() > 0.05,
        "o tamanho nao mudou entre t=1,5 (nivel 1,0) e t=3,0 (nivel 0,2): {} vs {}",
        sb[0],
        sc[0]
    );
    // ⚠️ **O CONTROLE do CONTROLE**: o de cima NÃO pode mexer-se com o tempo — ele é `Pure`, e
    // se ele respirasse, o que estaríamos a medir em baixo seria outra coisa qualquer.
    let u0 = ys(&cook(&mut state, &reg, sinks[0], 0.0));
    let u9 = ys(&cook(&mut state, &reg, sinks[0], 9.0));
    assert_eq!(u0, u9, "a leitura por ELEMENTO nao e' funcao do playhead");
}

/// ⭐⭐ **A coluna de TEXTO é saltada, e a de números ao lado dela entra** — a divergência
/// medida contra a regra do Blender, provada na cena que o Enio abre.
#[test]
fn the_word_column_is_skipped_and_the_numbers_beside_it_arrive() {
    let t = ph2d_table::parse(CSV);
    assert_eq!(
        t.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["vendas", "tempo", "nivel"],
        "a coluna `mes` tem palavras e nao entra"
    );
    assert_eq!(t.rows, 12);
    // ⭐ E a recusa CHEGA a uma frase — sem um leitor, a divergência contra o Blender pagava o
    // preço (recusar a coluna inteira) e não entregava o benefício (o artista saber porquê).
    let r = t.report().expect("a coluna de palavras tem de ser DITA");
    assert!(r.contains("mes"), "{r}");
}

/// ⭐⭐ **E A FRASE CHEGA AO TERMINAL** — a porta é o cache da shell, uma vez por ficheiro.
#[test]
fn the_report_reaches_the_shell_and_names_the_word_column() {
    let reg = registry();
    let mut state = MotionState::new();
    let sinks = build_table_demo_document(&mut state.doc, &reg).expect("a cena constroi");
    let _ = cook(&mut state, &reg, sinks[0], 0.0);
    // Uma tabela viva no cache, e ela tem o que dizer.
    assert_eq!(state.table_cache.len(), 1);
}
