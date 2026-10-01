//! Gates do plano das secções — ver o cabeçalho de [`super`].

use super::*;

/// ⭐⭐ **Nenhum pintor de secção pergunta o tema ao HOST** — a pergunta é a [`tema`], que devolve o
/// da secção em curso. Um `ctx.host.theme()` num pintor de secção pinta os controlos no tema do
/// PAINEL por cima de um cartão no tema da SECÇÃO (o `retheme` só recolore o cartão).
///
/// ⚠️ Os DOIS que ficam no `paint.rs` são o CHROME (o título, o fecho, a superfície e o vazio), que é
/// do painel por construção — e a contagem é exacta, para um terceiro que lá entre ser acusado.
/// *Mutação: o `header` de volta a `ctx.host.theme()` ⇒ o `widgets.rs` acusa.*
#[test]
fn nenhum_pintor_de_seccao_pergunta_o_tema_ao_host() {
    // A agulha monta-se em runtime: escrita como literal aqui, este gate leria-se a si próprio.
    let agulha = ["host", ".theme()"].concat();
    let pintores: [(&str, &str); 9] = [
        ("widgets.rs", include_str!("widgets.rs")),
        ("body.rs", include_str!("body.rs")),
        ("body_peca.rs", include_str!("body_peca.rs")),
        ("brush.rs", include_str!("brush.rs")),
        ("brush_cor.rs", include_str!("brush_cor.rs")),
        ("brush_fileiras.rs", include_str!("brush_fileiras.rs")),
        ("mask_tools.rs", include_str!("mask_tools.rs")),
        ("tool.rs", include_str!("tool.rs")),
        ("../preview.rs", include_str!("../preview.rs")),
    ];
    for (nome, texto) in pintores {
        assert_eq!(
            texto.matches(agulha.as_str()).count(),
            0,
            "o pintor `{nome}` pergunta o tema ao host — pergunte à `plano::tema`, senão a \
             secção pinta-se no tema do painel por cima do cartão no tema dela"
        );
    }
    assert_eq!(
        include_str!("../paint.rs").matches(agulha.as_str()).count(),
        2,
        "o `paint.rs` só pergunta o tema ao host para o CHROME (o vazio e o painel) — a terceira \
         pergunta é um pintor de secção no tema errado"
    );
}

/// ⭐ **O escopo do tema abre na tarefa e FECHA ao sair** — fora de uma secção, o tema é o do
/// painel outra vez. *Mutação: o `Escopo` sem `Drop` ⇒ o tema da última secção vaza.*
#[test]
fn o_escopo_do_tema_fecha_ao_sair() {
    assert_eq!(
        TEMA.with(Cell::get),
        None,
        "fixtura: começa fora de uma secção"
    );
    {
        let _e = Escopo(TEMA.with(|t| t.replace(Some(Theme::Candy))));
        assert_eq!(TEMA.with(Cell::get), Some(Theme::Candy));
        {
            let _f = Escopo(TEMA.with(|t| t.replace(Some(Theme::Light))));
            assert_eq!(TEMA.with(Cell::get), Some(Theme::Light));
        }
        assert_eq!(
            TEMA.with(Cell::get),
            Some(Theme::Candy),
            "o escopo interior não devolveu o exterior"
        );
    }
    assert_eq!(
        TEMA.with(Cell::get),
        None,
        "o escopo vazou para fora da secção"
    );
}

/// ⭐ **As listas e o painel falam das MESMAS secções** — toda secção que o painel declara como
/// cabeçalho dobrável está numa das duas listas do plano, e nenhuma está nas duas.
#[test]
fn as_listas_do_plano_cobrem_os_cabecalhos_do_painel() {
    let cabecalhos: Vec<NodeId> = crate::rows::section_headers().collect();
    assert_eq!(
        cabecalhos.len(),
        SECCOES_MOVEIS.len() + 1,
        "piso de população"
    );
    for id in &cabecalhos {
        let movel = SECCOES_MOVEIS.contains(id);
        let fixa = *id == SECCAO_FIXA;
        assert!(
            movel ^ fixa,
            "o cabeçalho {id:?} não tem lugar no plano (ou tem dois)"
        );
    }
}
