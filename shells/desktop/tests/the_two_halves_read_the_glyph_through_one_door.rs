//! **ARCH-GATE — o glifo de um `IconButton` sai de UMA porta** (plano UI/UX W8b).
//!
//! # O buraco que só isto fecha
//!
//! As duas metades da fatia têm gates de comportamento próprios: o do canvas prova que a pele
//! muda com o desenho, e o do spec prova que o `RowSpec` carrega o glifo que a porta produz. Nenhum
//! dos dois vê a falha que importa — **uma segunda normalização no lado do canvas**. Ela seria
//! *consistente consigo mesma*, então o gate do canvas continuaria verde; o do spec compara com a
//! porta e também continuaria verde; e o produto mostraria **um ícone no canvas e outro no painel**.
//!
//! Isso é a divergência que só uma screenshot revela — o modo de falha mais caro que este repo
//! conhece, e a razão inteira de a normalização ser uma função e não duas.
//!
//! ⚠️ **E o gate é de FONTE porque a falha é de FIAÇÃO.** Um teste de unidade que chamasse
//! `icon_face` provaria que a porta funciona; o que está em causa é se os dois chamadores passam
//! por ela.

/// Os dois construtores de parâmetro de pele, e o que cada um faz com o glifo.
const HALVES: [(&str, &str); 2] = [
    ("../src/widget_live.rs", "a ponte do canvas"),
    ("../src/ui_panel_spec.rs", "o plano do painel gerado"),
];

#[test]
fn the_two_halves_read_the_glyph_through_one_door() {
    for (path, who) in HALVES {
        let src = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join(path),
        )
        .unwrap_or_else(|e| panic!("{who} ({path}) nao foi lido: {e}"));

        // ⚠️ A âncora é o NOME, sem o parêntese: uma porta pode ser CHAMADA (`icon_face(p)`) ou
        // PASSADA (`.and_then(icon_face)`), e as duas a percorrem igual. Ancorar na sintaxe da
        // chamada fez este gate ficar vermelho sobre um produto correto no dia em que a metade do
        // spec virou uma referência de função — o gate estava a descrever a forma, não a lei.
        assert!(
            src.contains("icon_face"),
            "{who} nao pergunta o glifo a' porta unica (`widget_icon::icon_face`)"
        );
        // ⚠️ `build_bezpath` é a porta CRUA da geometria — legítima para quem desenha a forma,
        // e o começo de uma segunda normalização para quem monta um glifo.
        assert!(
            !src.contains("build_bezpath"),
            "{who} constroi geometria por conta propria — uma segunda normalizacao aqui daria \
             um icone no canvas e outro no painel, com as duas suites verdes"
        );
    }
}

/// **Controle positivo:** a porta que os dois citam existe, e é ela que normaliza.
///
/// ⚠️ Sem isto, renomear `icon_face` deixaria as duas asserções acima a falhar por um motivo
/// (*"não chama a porta"*) e a verdadeira causa (*"a porta mudou de nome"*) escondida; e apagar o
/// módulo faria o `read_to_string` explodir, que é a falha alta que se quer.
#[test]
fn the_one_door_is_where_it_says_it_is() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("widget_icon.rs"),
    )
    .expect("a porta unica do glifo sumiu");
    assert!(
        src.contains("pub(crate) fn icon_face("),
        "a porta mudou de nome — os gates irmaos falhariam pela razao errada"
    );
}

/// **A FILA DE INTENTS DO PAINEL AUTORADO TEM UM DRENO** — o gate que o vazamento merecia.
///
/// ⚠️ **É arch-gate porque a falha é de FIAÇÃO.** Um teste de unidade prova que `drain_intents`
/// funciona; o que estava errado é que **ninguém a chamava** — o painel autorado era o único do
/// app sem ponte, e a fila crescia sem teto com ele aberto. Nenhum gate de comportamento pode ver
/// isso, porque o defeito é a ausência de uma chamada.
///
/// ⚠️ E a segunda metade é a ORDEM: publicar DEPOIS do dreno de sinais entregaria o aperto um
/// quadro atrasado — invisível num toast e visível no dia em que o consumidor for som. O gate
/// afirma que o dreno do painel precede o dos sinais.
#[test]
fn the_authored_intent_queue_has_a_drain_and_it_runs_before_the_signal_drain() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render_loop")
            .join("mod.rs"),
    )
    .expect("o render_loop sumiu");

    let drain = src
        .find("ph2d_panel_authored::drain_intents()")
        .expect("a fila de intents do painel autorado nao tem dreno — ela cresce sem teto");
    let signals = src
        .find("self.signals.read(&mut self.signal_toast_reader)")
        .expect("o dreno de sinais mudou de forma — este gate mede a ordem contra ele");
    assert!(
        drain < signals,
        "o painel autorado publica DEPOIS do dreno de sinais: o aperto do botao chegaria um \
         quadro atrasado"
    );

    let turn = src
        .find("self.signals.advance_frame()")
        .expect("o quadro de sinais nao vira");
    assert!(
        turn < drain,
        "o painel publica ANTES de o quadro virar — o sinal dele seria aposentado na hora"
    );
}
