//! O gate da pilha do Composite no estado inicial da macro (W6c): o censo dos controlos não tem
//! controlo da pilha, por isso a ida e volta prova-se aqui, campo a campo.

use super::*;

/// ⭐ Cada camada da pilha — operação, força, escopo da borracha, cor, dureza e tamanho —, o ligado, o
/// `n` e a operação do `+` chegam ao tool que nasce do estado inicial.
#[test]
fn o_estado_inicial_leva_a_pilha_do_composite_camada_a_camada() {
    let fabrica = PainterTool::default();
    let base = CompositeLayer::default();
    let ops = [CompositeOp::Smear, CompositeOp::Blur, CompositeOp::Erase];
    let mut t = PainterTool::default();
    let p = &mut t.paint;
    p.composite_enabled = true;
    p.composite_len = 3;
    p.composite_add_op = CompositeOp::Blur;
    for (i, c) in p.composite.iter_mut().enumerate() {
        let x = i as f32;
        *c = CompositeLayer {
            op: ops[i % ops.len()],
            strength: 0.25 + 0.1 * x,
            erase_scope: EscopoDaBorracha::Traco,
            color: Some([0.05 + 0.1 * x, 0.5, 0.9]),
            hardness: Some(0.3 + 0.05 * x),
            size: 1.5 + 0.25 * x,
        };
    }
    // A fixtura sai da fábrica em CADA campo: um campo deixado no de omissão não acusa a sua perda.
    let f = &fabrica.paint;
    assert_ne!(p.composite_enabled, f.composite_enabled);
    assert_ne!(p.composite_len, f.composite_len);
    assert_ne!(p.composite_add_op, f.composite_add_op);
    assert_eq!(
        f.composite,
        [base; super::super::super::composite::N_CAMADAS]
    );
    for c in &p.composite {
        assert_ne!(c.op, base.op);
        assert_ne!(c.strength, base.strength);
        assert_ne!(c.erase_scope, base.erase_scope);
        assert_ne!(c.color, base.color);
        assert_ne!(c.hardness, base.hardness);
        assert_ne!(c.size, base.size);
    }

    let (cfg, texturas) = t.macro_estado();
    let mut u = PainterTool::default();
    u.macro_estado_aplica(&cfg, texturas).expect("aplica");
    let (a, b) = (&t.paint, &u.paint);
    assert_eq!(b.composite, a.composite, "as camadas");
    assert_eq!(
        (b.composite_enabled, b.composite_len, b.composite_add_op),
        (a.composite_enabled, a.composite_len, a.composite_add_op)
    );
}

/// B41.5: a dureza AUSENTE de uma camada mantém a da camada (como todo campo do estado); `null` é «sem
/// dureza própria». ⛔ Antes, ausente virava `None` e a camada perdia a dureza em silêncio.
#[test]
fn a_dureza_ausente_da_camada_mantem_e_o_null_tira() {
    let base = CompositeLayer {
        hardness: Some(0.4),
        ..CompositeLayer::default()
    };
    assert_eq!(
        camada_de_json(&json!({}), base).hardness,
        Some(0.4),
        "ausente"
    );
    assert_eq!(
        camada_de_json(&json!({ "hardness": null }), base).hardness,
        None,
        "null"
    );
    assert_eq!(
        camada_de_json(&json!({ "hardness": 0.75 }), base).hardness,
        Some(0.75),
        "número"
    );
}

/// B41.5: um estado cujo trilho não é desta tela (a máscara com outro tamanho) é erro e deixa a ferramenta
/// INTACTA. ⛔ Antes, o erro do trilho subia depois de o preset, o meio, os pincéis e a pilha já estarem
/// escritos: a ferramenta ficava meio-aplicada.
#[test]
fn um_estado_que_falha_no_trilho_deixa_a_ferramenta_intacta() {
    let mut a = PainterTool::default();
    a.paint.composite_enabled = true;
    a.paint.composite_len = 3;
    let (cfg, mut texturas) = a.macro_estado();
    let mut controlo = PainterTool::default();
    let antes = controlo.macro_estado();
    controlo
        .macro_estado_aplica(&cfg, texturas.clone())
        .expect("controlo: o estado aplica-se");
    assert_ne!(
        controlo.macro_estado(),
        antes,
        "controlo: o estado muda a ferramenta"
    );

    texturas.insert("trilho/mascara".to_owned(), vec![0; 5]);
    let mut u = PainterTool::default();
    assert!(
        u.macro_estado_aplica(&cfg, texturas).is_err(),
        "a máscara de outra tela é erro"
    );
    assert_eq!(u.macro_estado(), antes, "a ferramenta ficou meio-aplicada");
}
