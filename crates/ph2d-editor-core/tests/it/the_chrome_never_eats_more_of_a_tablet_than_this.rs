//! ⭐⭐⭐ **O ORÇAMENTO DE ECRÃ É UM NÚMERO QUE REPROVA** — não uma boa intenção.
//!
//! > Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir
//! > perdendo espaço.»*
//!
//! # ⛔⛔ Por que uma nota não chegava
//!
//! A `D2` tem escrito *«⚠️ o preço que o Enio aceitou: a barra global come uma faixa de altura
//! permanente»* — e uma segunda faixa (o cabeçalho da área) foi construída na entrega 30 **e
//! revertida no mesmo dia**, porque a nota não trava nada. *Uma restrição sem instrumento é uma
//! nota que envelhece* (`CLAUDE.md` §2).
//!
//! ⇒ este gate mede a **área de desenho** como percentagem da janela, nos três tablets reais, e
//! reprova quando ela desce. Uma faixa nova passa a ter de dizer o que devolve.
//!
//! # ⚠️ Ele mede o PRODUTO, e por isso faz as DUAS passagens
//!
//! A altura da fila de ferramentas depende da largura da área (ela quebra em linhas), e a largura
//! da área não depende da altura da fila. É a mesma aritmética do `hero::frame_layout`: uma
//! passagem com a faixa a zero para ler a largura, e a definitiva a seguir. ⛔ Medir com a faixa
//! fixa daria a resposta de uma janela que não existe.
//!
//! # ⚠️ E a catraca tem CENSO, senão vira licença
//!
//! Toda linha traz um piso **e** um tecto: o piso reprova quando se perde área, e o tecto reprova
//! quando se GANHA muito — porque nesse dia a barra ficou obsoleta e tem de descer. *Uma catraca
//! sem censo de obsolescência não desce: ela vira licença* (`CLAUDE.md` §5.0).

use ph2d_editor_core::screens::hero::{HeroScreen, menu_bar, tool_bar};
use ph2d_editor_core::screens::layout::{
    CenterSplit, ChromeBands, DockSide, DockSides, HeroLayout,
};
use ph2d_editor_core::widget::RailButtonSize;
use ph2d_editor_core::zones::Rect;

/// Os três tablets, em pontos lógicos. ⚠️ **O menor manda** — é nele que a aritmética do chrome
/// morde primeiro, e é ele que o `spec/01 §6` nomeia como a restrição que não escala.
const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

/// ⭐ **A catraca**, com as duas colunas ABERTAS — que é o estado de trabalho, não o melhor caso.
///
/// | alvo | sem pincel | com pincel |
/// |---|---:|---:|
/// | iPad 12.9 | ~~50,8 %~~ → **50,6 %** | idem |
/// | iPad 11 | ~~44,0 %~~ → **49,6 %** | idem |
/// | iPad mini | ~~40,9 %~~ → **48,9 %** | idem |
///
/// ⭐⭐⭐ **O PISO SUBIU em 2026-09-20, e quem o mandou subir foi a metade da OBSOLESCÊNCIA.** A
/// largura de fábrica de uma coluna deixou de ser um absoluto e passou a ser uma **fracção da
/// janela** ([`ChromeBands::default_dock_w`]): `+5,6` pontos no iPad 11 e **`+8,0`** no mini.
///
/// ⚠️⚠️ **E o `50,6` do 12,9" NÃO é desta wave: a TABELA é que estava para trás.** O piso desceu
/// `0,2` pontos em 2026-09-07 (a divisória de `4 px` por fronteira, a pedido do dono) e o `FLOOR`
/// foi actualizado — *esta tabela não*. ⇒ ela dizia `50,8` sobre um produto que media `50,6` havia
/// duas semanas. *Quando um ficheiro imprime duas medidas da mesma grandeza e elas discordam, isso
/// É o achado.*
///
/// ⚠️ **E o iPad 12,9" não se mexeu um pixel com esta wave** — ele É a referência, e a lei é inerte acima dela
/// por construção. *Uma cura que melhorasse os três teria escalado também o ecrã grande, e ali
/// escalar faz o chrome CRESCER* (na janela de `1 930 px` do dono seriam `870 px` de colunas
/// contra `612`).
///
/// ⛔⛔ **E este gate era CEGO ao defeito até esse dia:** ele construía as bandas a partir da const
/// `ChromeBands::DEFAULT`, logo media `612 px` em qualquer janela e ficava verde sobre a linha que
/// o `medicoes/06 §1` já escrevia havia três semanas. *Um gate que reconstrói a banda em vez de
/// ler a LEI mede a fórmula e não o produto* — a mesma armadilha do `tool_bar_lines`, no mesmo
/// ficheiro, pela segunda vez.
///
/// ⭐⭐ **A coluna «com pincel» DEIXOU de ser pior** (entrega 32). Ela media `40,8` e `37,6` porque
/// a fila de ferramentas quebrava em duas linhas (`54 → 108 px`) nos dois tablets menores no
/// instante em que o Painter entrava em mãos. Hoje a faixa é **sempre uma linha** e o que não cabe
/// vive atrás do `⋯` (`tool_bar::bar_split`) — `+3,2` pontos no iPad 11 e `+3,3` no mini.
///
/// ⚠️ **E foi o TECTO deste gate que obrigou a actualizar estes números** em vez de os deixar como
/// folga silenciosa: `40,8 → 44,0` disparou a metade de obsolescência. *É para isto que ela existe.*
/// ⛔⛔ **O piso DESCEU 0,2 pontos em 2026-09-07, e a descida tem dono, número e comparação.**
///
/// Enio, com a captura do Godot ao lado: *«entre os painéis e o Canvas e entre os painéis e a
/// timeline há espaços (provavelmente 3 ou 4 px)»*. A divisória de **4 px** por fronteira custa
/// **0,2 pontos** de área em cada célula — medido, não estimado.
///
/// ⚠️ **A comparação é o que torna isto uma decisão e não uma cedência:** o cabeçalho por área,
/// recusado em 2026-08-31, custava **1,5 pontos** — *sete vezes e meia mais* — e não devolvia nada
/// além de casa para dois interruptores. Esta devolve o que o dono foi buscar à referência: a
/// leitura de que as áreas são superfícies distintas.
///
/// ⛔ **Um piso que desce por pedido do dono continua a ser uma catraca** — o que ele proíbe é
/// descer em SILÊNCIO, por uma faixa que ninguém pediu. *A barra move-se com a assinatura de quem
/// a moveu.*
const FLOOR: [(&str, f32); 3] = [("iPad 12.9", 50.5), ("iPad 11", 49.3), ("iPad mini", 48.6)];

/// Quanto acima do piso é «ganhou-se área e a barra ficou obsoleta».
const STALE_ABOVE: f32 = 2.0;

/// A área de desenho, em % da janela, com as duas colunas abertas.
fn drawing_area_pct(w: f32, h: f32) -> f32 {
    let vp = Rect::new(0.0, 0.0, w, h);
    let mut bands = ChromeBands {
        rail_w: 0.0,
        top_bar_h: menu_bar::MENU_BAR_H,
        tool_bar_h: 0.0,
        // ⭐⭐⭐ **As colunas vêm da LEI e não do `DEFAULT` cru** (2026-09-20). ⛔ Enquanto este
        // gate lia a const, ele era **cego à largura da janela** — e foi por isso que ele ficou
        // verde durante três semanas sobre o defeito que o `medicoes/06 §1` já nomeava: *«a mesma
        // decisão custa 20 % mais no aparelho mais pequeno»*. ⚠️ Um gate que reconstrói a banda em
        // vez de ler a lei mede a fórmula, não o produto — é a mesma armadilha que a nota do
        // `tool_bar_lines` aqui em baixo regista, e ela mordeu duas vezes no mesmo ficheiro.
        left_dock_w: ChromeBands::default_dock_w(DockSide::Left, w),
        right_dock_w: ChromeBands::default_dock_w(DockSide::Right, w),
        ..ChromeBands::DEFAULT
    };
    // ⭐ **UMA linha, como o `hero::frame_layout` faz desde 2026-08-31** — o que não cabe vai para
    // o `⋯` (`tool_bar::bar_split`). ⚠️ A 1.ª redacção deste gate chamava o `tool_bar_lines` e
    // continuou a medir `2` linhas depois de o produto deixar de as usar: *um gate que reproduz a
    // fórmula em vez de perguntar ao produto pina a fórmula, não o produto.*
    //
    // ⚠️ **E o `painter` SAIU da assinatura**, porque ele deixou de mexer na altura. A propriedade
    // que ele comprava mede-se agora onde ela vive — ver
    // `the_painter_costs_no_screen_because_the_overflow_takes_it`. Um parâmetro que já não muda a
    // resposta faria este gate medir a mesma célula duas vezes.
    bands.tool_bar_h = tool_bar::tool_bar_h(RailButtonSize::Small, 1);
    let l = HeroLayout::for_viewport_bands(vp, false, bands, CenterSplit::None, DockSides::BOTH);
    100.0 * l.draw_area.w * l.draw_area.h / (w * h)
}

/// ⭐⭐⭐ **O chrome não come mais do tablet do que já comia.**
#[test]
fn the_chrome_never_eats_more_of_a_tablet_than_this() {
    let mut measured = 0usize;
    let mut stale = Vec::new();
    for (name, floor) in FLOOR {
        let (_, w, ht) = TABLETS
            .iter()
            .find(|(n, _, _)| *n == name)
            .copied()
            .unwrap_or_else(|| panic!("controlo: `{name}` saiu da lista de tablets"));
        let pct = drawing_area_pct(w, ht);
        println!("{name:11} area de desenho = {pct:5.1} %  (piso {floor:.1} %)");
        assert!(
            pct >= floor - 0.05,
            "{name}: a area de desenho caiu para {pct:.1} % (piso {floor:.1} %) — uma faixa nova \
             comeu ecra de um tablet"
        );
        if pct > floor + STALE_ABOVE {
            stale.push(format!("{name}: {pct:.1} % contra piso {floor:.1} %"));
        }
        measured += 1;
    }
    assert_eq!(measured, FLOOR.len());
    assert!(
        stale.is_empty(),
        "a catraca ficou OBSOLETA — ganhou-se area e o piso nao desceu, logo ele deixou de \
         defender o que mede:\n  {}",
        stale.join("\n  ")
    );
}

/// ⛔⛔ **E RECOLHER as colunas devolve o ecrã** — é a maior alavanca que existe, e ela é medida.
///
/// ⚠️ Ela está aqui para que a diferença entre os dois estados **não encolha em silêncio**: no dia
/// em que uma faixa permanente nascer, é este número que deixa de valer a pena.
#[test]
fn collapsing_both_columns_still_gives_the_tablet_back() {
    let h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for (name, w, ht) in TABLETS {
        let vp = Rect::new(0.0, 0.0, w, ht);
        let bands = ChromeBands {
            rail_w: 0.0,
            top_bar_h: menu_bar::MENU_BAR_H,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
            ..ChromeBands::DEFAULT
        };
        let closed = HeroLayout::for_viewport_bands(
            vp,
            false,
            bands,
            CenterSplit::None,
            DockSides {
                left: false,
                right: false,
            },
        );
        let pct = 100.0 * closed.draw_area.w * closed.draw_area.h / (w * ht);
        println!("{name:11} colunas fechadas = {pct:5.1} %");
        // ⚠️ `88,9` e não `89,0`: o menor dos três mede `88,97 %`, e uma barra escrita a partir do
        // número IMPRESSO (arredondado) reprova sobre produto correcto — foi o que aconteceu na
        // 1.ª corrida deste gate.
        //
        // ⛔ **`88,7` desde 2026-09-07**: a divisória de 4 px entre áreas (pedido do dono, com a
        // captura do Godot) leva o menor dos três a `88,8` impresso. ⚠️ E a armadilha de cima
        // mordeu-me na mesma corrida: escrevi `43,8` para o iPad 11 a partir do `43,7` impresso, a
        // presumir que o custo era `−0,2` uniforme — ele é `−0,3` ali. *Um custo medido numa célula
        // não se estende às outras por subtracção.*
        assert!(
            pct >= 88.7,
            "{name}: fechar as duas colunas devolve so' {pct:.1} % — o chrome permanente cresceu"
        );
    }
    let _ = h;
}

/// ⭐⭐⭐ **O PINCEL não custa ecrã — e não custa porque o `⋯` leva o excesso.**
///
/// ⛔⛔ Ele custava: a fila ia de **10** para **18** entradas e resolvia isso **crescendo**
/// (`54 → 108 px`), o que valia `−3,2` pontos no iPad 11 e `−3,3` no mini
/// (`docs/UI_New_and_Simple/medicoes/06_o_orcamento_de_ecra_em_tablet.md`).
///
/// ⚠️ **A igualdade sozinha seria VAZIA** — ela também seria verdade se o Painter não trouxesse
/// ferramenta nenhuma. Por isso o gate exige as duas metades: a faixa fica em **uma linha** nos
/// dois estados, e nos dois tablets menores há de facto **transbordo**, que é quem paga a
/// igualdade.
#[test]
fn the_painter_costs_no_screen_because_the_overflow_takes_it() {
    let h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for (name, w, ht) in TABLETS {
        let vp = Rect::new(0.0, 0.0, w, ht);
        let bands = ChromeBands {
            rail_w: 0.0,
            top_bar_h: menu_bar::MENU_BAR_H,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
            ..ChromeBands::DEFAULT
        };
        let area_w =
            HeroLayout::for_viewport_bands(vp, false, bands, CenterSplit::None, DockSides::BOTH)
                .draw_area
                .w;
        let (idle, idle_over) = tool_bar::bar_split(&h.store, false, false, area_w);
        let (paint, paint_over) = tool_bar::bar_split(&h.store, true, false, area_w);
        println!(
            "{name:11} repouso: {:2} na fila + {:2} atras do dots  |  pincel: {:2} + {:2}",
            idle.entries.len(),
            idle_over.len(),
            paint.entries.len(),
            paint_over.len()
        );
        // Controlo: o Painter tem de trazer mais ferramentas, senão não há nada a medir.
        assert!(
            paint.entries.len() + paint_over.len() > idle.entries.len() + idle_over.len(),
            "controlo: {name} — o Painter nao acrescenta ferramenta nenhuma a' fila"
        );
        for (estado, rail) in [("repouso", &idle), ("pincel", &paint)] {
            let lines = ph2d_editor_core::widget::horizontal_lines(
                rail,
                area_w - 16.0,
                RailButtonSize::Small,
            );
            assert_eq!(
                lines, 1,
                "{name} {estado}: a fila precisa de {lines} linhas — ela voltou a crescer"
            );
        }
        if w < 1300.0 {
            assert!(
                !paint_over.is_empty(),
                "{name}: com o pincel nada transbordou — ou a fila encolheu, ou a igualdade acima \
                 e' vazia"
            );
        }
    }
}
