//! Gates do [`super`] — irmão por `#[path]`, e portanto FILHO: `use super::*` alcança os
//! privados, e o `vec_filter.rs` volta para debaixo do teto de 700 LOC.
use super::*;
// O vocabulário dos MODOS mora no catálogo, que é quem o consome (a `SPECS`).
use crate::vec_filter_kinds::FALLOFF_MODES;

fn stack(kinds: &[u8]) -> VecFilter {
    VecFilter {
        ops: kinds.iter().map(|k| FxOp::new(*k)).collect(),
    }
}

/// **Reordenar troca DOIS vizinhos, e as pontas são no-ops** — subir na primeira linha e descer
/// na última não fazem nada, e o painel nem desenha essas setas. Aqui prova-se que, mesmo se
/// alguém as despachasse, a pilha não se deforma (um `swap` fora de faixa entraria em pânico).
#[test]
fn reordering_swaps_neighbours_and_the_ends_are_no_ops() {
    let mut f = stack(&[FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW]);
    assert!(f.move_down(0));
    assert_eq!(
        f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
        vec![FxOp::GLOW, FxOp::BLUR, FxOp::DROP_SHADOW]
    );
    assert!(f.move_up(2));
    assert_eq!(
        f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
        vec![FxOp::GLOW, FxOp::DROP_SHADOW, FxOp::BLUR]
    );
    let before = f.clone();
    assert!(!f.move_up(0), "subir na primeira linha não faz nada");
    assert!(!f.move_down(2), "descer na última não faz nada");
    assert!(!f.move_down(9), "nem uma linha que não existe");
    assert_eq!(f, before, "e nenhuma delas pode deformar a pilha");
}

/// **Uma pilha só desenha se algum degrau estiver LIGADO** — a porta única que o produtor e a
/// remoção do componente perguntam. Vazia e toda-desligada são o mesmo fato para quem desenha.
#[test]
fn a_stack_is_active_only_while_some_op_is_enabled() {
    assert!(!VecFilter::default().is_active(), "vazia não desenha nada");
    let mut f = stack(&[FxOp::BLUR, FxOp::GLOW]);
    assert!(f.is_active());
    f.ops[0].enabled = false;
    assert!(f.is_active(), "um degrau ligado basta");
    f.ops[1].enabled = false;
    assert!(!f.is_active(), "toda desligada é o mesmo que vazia");
}

/// **Os defaults de cada tipo são VISÍVEIS** — armar no neutro seria um clique que não muda um
/// pixel, e o artista concluiria que o botão está quebrado.
///
/// ⚠️ O laço varre **todos** os tipos e pergunta o que exigir à [`FxOp::SPECS`]: um tipo novo
/// entra neste gate sem que ninguém o acrescente aqui, que é o oposto da lista escrita à mão
/// que a W2 tinha (e que teria ficado verde sobre os quatro tipos desta wave).
#[test]
fn a_new_op_is_born_visible() {
    for kind in 0..FxOp::KINDS as u8 {
        let o = FxOp::new(kind);
        let s = FxOp::spec(kind);
        assert_eq!(
            o.kind, kind,
            "o Add do tipo {kind} tem de criar o tipo {kind}"
        );
        assert!(o.enabled, "nasce ligado ({})", s.name);
        assert!(
            o.opacity > 0.0,
            "opacidade zero seria invisível ({})",
            s.name
        );
        if s.radius_label.is_some() {
            assert!(o.radius > 0.0, "raio zero não desenharia nada ({})", s.name);
        }
        if s.offset_labels.is_some() {
            assert!(
                o.offset != [0.0, 0.0],
                "uma sombra sem deslocamento é um glow — o default tem de a mostrar ({})",
                s.name
            );
        } else {
            assert!(
                o.offset == [0.0, 0.0],
                "só quem tem offset nasce deslocado ({})",
                s.name
            );
        }
    }
}

/// **Quem tem modos nasce no default declarado; quem não tem nasce em ZERO.** Um número
/// guardado que não seleciona nada é a semente de "este campo quer dizer o quê aqui?".
#[test]
fn only_the_kinds_with_modes_carry_one() {
    for kind in 0..FxOp::KINDS as u8 {
        let o = FxOp::new(kind);
        let s = FxOp::spec(kind);
        if s.modes.is_empty() {
            assert_eq!(o.mode, 0, "{} não tem modos e nasceu em {}", s.name, o.mode);
        } else {
            assert!(
                (o.mode as usize) < s.modes.len(),
                "{} nasceu num modo que a tabela não oferece ({})",
                s.name,
                o.mode
            );
        }
    }
    // Os dois de dentro nascem em CONTOUR — a banda que segue o contorno é o que "sombra
    // interna" desenha para quem olha a forma; a proximidade fica como a outra opção.
    for kind in [FxOp::INNER_SHADOW, FxOp::INNER_GLOW] {
        assert_eq!(FxOp::new(kind).mode, FxOp::MODE_CONTOUR);
        assert_eq!(FxOp::spec(kind).modes.len(), 2);
    }
}

/// **A tabela é indexada pelo CÓDIGO, e cada tipo tem nome próprio.** Um `SPECS` fora de ordem
/// daria ao painel o nome de um tipo e os controles de outro — e nada pareceria quebrado até
/// alguém procurar o Offset da Drop Shadow no card do Glow.
#[test]
fn the_spec_table_is_indexed_by_the_kind_code() {
    assert_eq!(FxOp::SPECS.len(), FxOp::KINDS);
    for (name, kind) in [
        ("Blur", FxOp::BLUR),
        ("Glow", FxOp::GLOW),
        ("Drop Shadow", FxOp::DROP_SHADOW),
        ("Inner Shadow", FxOp::INNER_SHADOW),
        ("Inner Glow", FxOp::INNER_GLOW),
        ("Outline", FxOp::OUTLINE),
        ("Feather", FxOp::FEATHER),
        ("Bevel", FxOp::BEVEL),
        ("Color Overlay", FxOp::COLOR_OVERLAY),
    ] {
        assert_eq!(FxOp::kind_name(kind), name, "o código {kind} é o {name}");
    }
    let mut names: Vec<&str> = FxOp::SPECS.iter().map(|s| s.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), FxOp::KINDS, "dois tipos com o mesmo nome");
    // Um código de uma versão futura cai no tipo mais inerte, nunca em pânico.
    assert_eq!(FxOp::kind_name(200), "Blur");
}

/// **QUEM oferece a escolha de queda, e o que cada um arma ao nascer.**
///
/// ⚠️ Este gate existe porque o seam do painel é dirigido pela TABELA: apagar os modos do Glow
/// deixa-o verde (ele passa a esperar zero chips, coerentemente), então ele não pode testemunhar
/// que a capacidade EXISTE. O fato mora aqui, e é aqui que se pina.
#[test]
fn the_falloff_choice_is_offered_where_it_means_something() {
    for kind in [FxOp::GLOW, FxOp::INNER_SHADOW, FxOp::INNER_GLOW] {
        assert_eq!(
            FxOp::spec(kind).modes,
            &FALLOFF_MODES,
            "{} tinha de oferecer Proximity/Contour",
            FxOp::kind_name(kind)
        );
    }
    // ⚠️ **O Glow nasce em Proximity, e os de DENTRO em Contour.** O Glow SEMPRE foi a silhueta
    // borrada — ganhar uma opção não pode repintar o que "Add Glow" quer dizer para quem já o
    // usa —, e um Glow salvo antes desta wave carrega `mode = 0`, que é exatamente este.
    assert_eq!(FxOp::new(FxOp::GLOW).mode, FxOp::MODE_PROXIMITY);
    assert_eq!(FxOp::new(FxOp::INNER_SHADOW).mode, FxOp::MODE_CONTOUR);
    assert_eq!(FxOp::new(FxOp::INNER_GLOW).mode, FxOp::MODE_CONTOUR);
    // E quem não oferece escolha guarda ZERO — um número que não seleciona nada é a semente do
    // "este campo quer dizer o quê aqui?".
    for kind in 0..FxOp::KINDS as u8 {
        if FxOp::spec(kind).modes.is_empty() {
            assert_eq!(FxOp::new(kind).mode, 0, "{}", FxOp::kind_name(kind));
        }
    }
}

/// **`tints`/`displaces` são VISTAS da tabela, não uma segunda opinião.** Foi a divergência
/// entre elas e o `paint` que a tabela veio matar.
#[test]
fn the_predicates_are_views_of_the_table() {
    for kind in 0..FxOp::KINDS as u8 {
        let o = FxOp::new(kind);
        let s = FxOp::spec(kind);
        assert_eq!(o.tints(), s.color_label.is_some(), "tints() do {}", s.name);
        assert_eq!(
            o.displaces(),
            s.offset_labels.is_some(),
            "displaces() do {}",
            s.name
        );
    }
    // E as duas metades que decidem a MARGEM da textura: quem mora dentro não cresce nada.
    for kind in [
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::BEVEL,
        FxOp::COLOR_OVERLAY,
    ] {
        assert!(
            !FxOp::spec(kind).grows,
            "{} desenha só dentro do que recebeu — margem seria textura paga a troco de nada",
            FxOp::kind_name(kind)
        );
    }
    for kind in [
        FxOp::BLUR,
        FxOp::GLOW,
        FxOp::DROP_SHADOW,
        FxOp::OUTLINE,
        FxOp::FEATHER,
    ] {
        assert!(FxOp::spec(kind).grows, "{} espalha", FxOp::kind_name(kind));
    }
    assert!(
        FxOp::spec(FxOp::COLOR_OVERLAY).radius_label.is_none(),
        "o Color Overlay é PONTUAL — um raio nele seria knob morto (e um dispatch a mais)"
    );
}

/// **QUEM toma a lei de mistura, e por quê os outros não.**
///
/// ⚠️ Este gate existe porque a lista é uma DECISÃO, não uma dedução — e a dedução tentadora
/// (`!grows`) dá hoje exactamente o mesmo conjunto. Se alguém a "simplificar" assim, um tipo
/// futuro que espalhe para fora E tinja por dentro nasce sem o controle, em silêncio. O
/// número que justifica a lista é medido na `ph2d-render`
/// (`the_blend_of_an_outer_halo_only_reaches_the_antialiased_fringe`).
#[test]
fn the_blend_is_offered_where_a_colour_lands_on_something() {
    for kind in [
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::BEVEL,
        FxOp::COLOR_OVERLAY,
    ] {
        assert!(
            FxOp::spec(kind).takes_blend,
            "{} tinge o que já está lá — tem de tomar a lei",
            FxOp::kind_name(kind)
        );
    }
    for kind in [
        FxOp::BLUR,
        FxOp::GLOW,
        FxOp::DROP_SHADOW,
        FxOp::OUTLINE,
        FxOp::FEATHER,
    ] {
        assert!(
            !FxOp::spec(kind).takes_blend,
            "{} entra por baixo (ou não tem cor própria) — a lei seria uma orla de 1 px",
            FxOp::kind_name(kind)
        );
    }
    // ⚠️ **A coincidência com `!grows` ACABOU, e o assert que a pinava caiu — como ele próprio
    // mandava.** Ele existia para provar que as duas listas eram iguais *por acidente*; o
    // **Color Adjust** é o primeiro tipo que não cresce E não toma a lei (ele não tem cor
    // PRÓPRIA: a saída dele é a entrada ajustada, o argumento do Blur e do Feather). As duas
    // perguntas — *preciso de margem na textura?* e *a minha cor encosta na de baixo?* — passaram
    // a ter respostas diferentes, que é exactamente o que o campo próprio existia para permitir.
    assert!(
        !FxOp::spec(FxOp::COLOR_ADJUST).grows && !FxOp::spec(FxOp::COLOR_ADJUST).takes_blend,
        "o Color Adjust é o tipo que quebrou a coincidência — se ele voltar a coincidir, esta \
         nota mente"
    );
}

/// **Um degrau nasce em Normal, e Normal é o neutro.** Uma lei exótica por default repintaria
/// toda arte já autorada no primeiro load.
#[test]
fn a_new_op_is_born_in_the_neutral_law() {
    for kind in 0..FxOp::KINDS as u8 {
        assert_eq!(
            FxOp::new(kind).blend,
            FxOp::BLEND_NORMAL,
            "{} nasceu com lei de mistura",
            FxOp::kind_name(kind)
        );
    }
    assert_eq!(
        FxOp::BLEND_NORMAL,
        0,
        "o neutro é o código 0 (`BlendMode::Normal`)"
    );
}

/// **`blend_code` é a metade de HONRAR da porta única.** Um degrau que carrega uma lei num tipo
/// que não a toma — um arquivo antigo, uma mudança de tipo, um teste — manda **Normal** ao
/// dispositivo. Sem isto o produtor desenharia uma mistura que a UI não mostra.
#[test]
fn a_law_on_a_kind_that_does_not_take_one_never_reaches_the_device() {
    for kind in 0..FxOp::KINDS as u8 {
        let mut op = FxOp::new(kind);
        op.blend = 6; // Screen — bem longe do neutro.
        let want = if FxOp::spec(kind).takes_blend {
            6
        } else {
            FxOp::BLEND_NORMAL
        };
        assert_eq!(
            op.blend_code(),
            want,
            "{}: takes_blend={} mas blend_code deu {}",
            FxOp::kind_name(kind),
            FxOp::spec(kind).takes_blend,
            op.blend_code()
        );
        assert_eq!(op.takes_blend(), FxOp::spec(kind).takes_blend);
    }
}

/// O teto é respondido pela pilha, não contado no chamador.
#[test]
fn the_ceiling_is_the_stacks_own_answer() {
    let mut f = VecFilter::default();
    while f.has_room() {
        f.ops.push(FxOp::new(FxOp::BLUR));
    }
    assert_eq!(f.ops.len(), VecFilter::MAX_OPS);
}
