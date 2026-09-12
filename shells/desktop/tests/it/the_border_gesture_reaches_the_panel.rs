//! ⛔⛔ **O gesto da borda tem de CHEGAR ao painel — e os gates de geometria não o vêem.**
//!
//! A lei de fechar por arrasto vive no `ph2d-editor-core` e é medida lá: a área devolvida, a
//! exclusão entre a costura e a alça, o degrau uma linha abaixo do mínimo. ⚠️ **Nada disso afirma
//! que a shell os CHAMA.** Um gesto ligado a nada passa em todos aqueles testes — é a espécie de
//! defeito que este repo varre a cada wave (*«um controlo pintado que não alcança consumidor»*).
//!
//! ⚠️ **Este gate lê o FONTE, e é o idioma da casa para esta costura:** o irmão
//! `the_arrangement_is_read_at_boot_and_written_on_change` já o faz, com o motivo escrito — o
//! `dock_seam_move`/`_up` **faz `return` no `input_dispatch`** e não passa pelos hooks que um
//! teste de ponteiro conseguiria conduzir.
//!
//! # ⛔⛔⛔ O que este ficheiro deixou de afirmar, e porquê (2026-09-07)
//!
//! A versão anterior exigia, entre outras strings, `panel_visibility.insert(id, true)` — **a linha
//! do defeito** que o dono reportou como *«o inspector explodiu, soltou vários painéis no meio do
//! canvas»*. *Ele leu-a e chamou-lhe correcta.*
//!
//! ⚠️ **A lição não é «gates textuais são maus»** — é que um gate textual só pode afirmar sobre
//! **quem chama quem**, nunca sobre *o que a chamada faz*. A conduta mudou de sítio
//! ([`ph2d_editor_core::screens::hero::dock_columns`]) precisamente para poder ter régua a sério, e ela
//! está em `crates/ph2d-panel-registry-init/tests/it/a_column_gives_back_exactly_what_it_took.rs`,
//! que constrói um `HeroScreen` com o registo real, pinta o quadro e **conta os painéis**.
//!
//! ⇒ o que fica aqui é só a costura: *a shell pergunta pela alça, chama a porta, e não tem uma
//! segunda cópia da lei.*

use std::fs;
use std::path::PathBuf;

fn src(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⛔⛔ **O CÓDIGO sem a prosa** — e esta linha já pagou o mesmo defeito na wave 27.
///
/// Uma asserção de **AUSÊNCIA** sobre o fonte acusa o doc-comment que EXPLICA a ausência. Aqui o
/// `dock_seam_move` traz escrito *«fecha pela mesma porta que o menu usa (`panel_visibility`)»* —
/// uma frase que descreve correctamente uma chamada que já não vive neste ficheiro — e o
/// `contains` cru leu-a como reincidência.
///
/// ⚠️ *Um censo que lê o fonte tem de saber todas as formas do que lê*, e uma delas é o comentário
/// que fala sobre o código em vez de o ser.
///
/// ⚠️ **Corta em `//` e não trata literais de string**: não há nenhum neste ficheiro (verificado), e
/// um parser a sério aqui seria mais código do que o que ele guarda.
fn code_only(s: &str) -> String {
    s.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⛔⛔⛔ **O ARRASTO NÃO FECHA COLUNA NENHUMA** — ordem do dono, 2026-09-09.
///
/// > *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra
/// > superior. quando colapsei arrastando e abri no menu da barra superior travou por um
/// > minuto.»* — Enio.
///
/// ⚠️ **Este gate está INVERTIDO.** Ele exigia o contrário — que a shell consultasse o degrau
/// (`DOCK_W_COLLAPSE`) e chamasse o `dock_columns::close` — e aquilo shipou **um dia**. O gesto
/// existia por uma medição boa (fechar as duas colunas devolve 89–92 % do ecrã num tablet); o
/// dono usou-o e retirou-o. *Uma medição justifica um desenho; ela não o aprova.*
///
/// ⇒ a asserção é uma **AUSÊNCIA**, que é a forma com dentes aqui: no dia em que alguém voltar a
/// ligar o fecho ao arrasto — por hábito, ou a reconstruir a partir do doc do degrau, que **fica
/// escrito de propósito** — este gate reprova antes de o dono voltar a encontrá-lo.
#[test]
fn the_border_drag_never_closes_a_column() {
    let s = code_only(&src("src/dock_resize.rs"));
    for proibido in ["DOCK_W_COLLAPSE", "dock_columns::close", "may_close"] {
        assert!(
            !s.contains(proibido),
            "o arrasto da borda voltou a poder FECHAR a coluna (`{proibido}`) — o dono retirou \
             esse gesto em 2026-09-09; fechar mora no menu da barra superior"
        );
    }
    // ⛔ **O controlo do INSTRUMENTO:** um ficheiro renomeado, ou um `code_only` partido, deixaria
    //    as três ausências verdes sobre uma string vazia.
    assert!(
        s.contains("dock_seam_move") && s.contains("set_dock_width"),
        "o `dock_resize.rs` deixou de conter o gesto de redimensionar — esta varredura perdeu o \
         alvo e as ausências acima não medem nada"
    );
}

/// ⛔⛔⛔ **A shell NÃO tem uma segunda cópia da lei.**
///
/// Esta é a asserção com dentes deste ficheiro, e é uma **ausência**: no dia em que alguém voltar a
/// escrever aqui *quais* painéis a coluna leva, volta a existir uma resposta que nenhum gate mede —
/// porque estas funções são privadas de um `impl App` do binário e **nenhum teste as alcança**.
/// Foi exactamente assim que o defeito de 2026-09-07 viveu duas waves com a suíte verde.
#[test]
fn the_shell_does_not_keep_a_second_copy_of_which_panels_a_column_takes() {
    // ⚠️ Sem a prosa: ver [`code_only`]. O doc do gesto EXPLICA a porta pelo nome, e uma asserção
    //    de ausência sobre o texto cru acusa a explicação.
    let s = code_only(&src("src/dock_resize.rs"));
    assert!(
        !s.contains("panel_visibility"),
        "a shell voltou a decidir quais paineis uma coluna leva. Essa lei vive na \
         `ph2d_editor_core::screens::hero::dock_columns`, onde ela TEM regua (o gate \
         `a_column_gives_back_exactly_what_it_took` conta os paineis sobre o registo real); aqui \
         ela seria inalcancavel de qualquer teste"
    );
    assert!(
        !s.contains("with_registry"),
        "a shell voltou a varrer o registo de paineis por conta propria — e uma varredura sobre o \
         registo e' um SUPERCONJUNTO por construcao: a informacao que decide («o dono tinha isto \
         aberto?») nao esta' la'"
    );
}

/// ⭐⭐ **E a ALÇA reabre** — sem isto, uma coluna esvaziada pelo menu não teria caminho de volta
/// num ecrã de toque, onde não há cursor para descobrir a costura.
///
/// ⚠️ Ela chamava-se `the_handle_reopens_the_column_it_closed` — *«it closed»* era o arrasto, que
/// deixou de fechar em 2026-09-09. Quem esvazia a coluna hoje é o menu; a alça continua a ser
/// quem a traz de volta.
#[test]
fn the_handle_reopens_a_column_the_menu_emptied() {
    let s = code_only(&src("src/dock_resize.rs"));
    assert!(
        s.contains("dock_reopen_at"),
        "a shell nao pergunta pela alca: com a coluna fechada a borda fica MUDA, e num ecra~ de \
         toque nao ha' cursor que denuncie uma costura invisivel"
    );
}

/// ⛔ **E ela é PINTADA** — a costura vive do cursor, que num tablet não existe.
#[test]
fn the_handle_is_painted_every_frame() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ph2d-editor-core/src/screens/hero/paint.rs");
    let s = code_only(&fs::read_to_string(&root).expect("hero/paint.rs"));
    assert!(
        s.contains("paint_dock_reopen"),
        "ninguem pinta a alca: ela existiria como geometria e como alvo de toque, e invisivel — \
         que e' exactamente «chrome vivo e invisivel», a especie que a costura foi desenhada para \
         evitar"
    );
}
