//! ⭐⭐⭐ **A ORDEM do cérebro dentro do quadro** (TOP-20 #15).
//!
//! A máquina de estados **anuncia** e a tabela de acções (#5) **age**. Se a tabela lesse primeiro, o
//! anúncio chegaria um quadro atrasado — *invisível num toast e visível no dia em que o consumidor
//! for SOM*, que é a frase que esta shell já paga em quatro sítios.
//!
//! ⚠️ **A lente é o TEXTO EMENDADO do quadro** (`frame_text::render_frame`), nunca um ficheiro: a
//! fase que corre primeiro pode morar no ficheiro que vem depois, e um gate que lesse um `mod.rs`
//! mediria a ordem dos FICHEIROS.

/// **Mutação que deve sangrar:** trocar os dois blocos de sítio no `fase_signal_outbox`.
#[test]
fn o_cerebro_anuncia_antes_de_a_tabela_de_accoes_ler() {
    // ⚠️ **A agulha é o `.read(&mut …)` e NÃO a expressão inteira**: o `rustfmt` parte
    // `self.signals.read(…)` em três linhas, e a 1.ª redacção deste gate nasceu vermelha por
    // procurar um literal que o formatador tinha desfeito. *Uma agulha tem de sobreviver ao `fmt`.*
    let src = crate::frame_text::render_frame();

    let cerebro = src
        .find("state_machine_tick::advance_machines(")
        .expect("os cerebros nao sao avancados no quadro — o componente seria inerte");
    let tabela = src
        .find(".read(&mut self.signal_readers.action)")
        .expect("a tabela de accoes mudou de forma — este gate mede a ordem contra ela");
    assert!(
        cerebro < tabela,
        "a tabela de accoes le' ANTES de o cerebro anunciar: uma porta so' abriria no quadro \
         seguinte ao toque do botao"
    );

    // E a leitura do cérebro vem **antes** de ele avançar — é isso que faz todas as máquinas
    // partirem da mesma fotografia e fecha a classe dos laços sem um `if`.
    let leitura = src
        .find(".read(&mut self.signal_readers.machine)")
        .expect("o cerebro nao tem cursor proprio");
    assert!(
        leitura < cerebro,
        "a fotografia dos sinais tem de ser tirada ANTES de qualquer maquina avancar"
    );

    // ⚠️ E o cursor é PRÓPRIO: partilhar o da tabela faria uma das duas ficar sem sinais.
    assert_ne!(
        leitura, tabela,
        "o cerebro e a tabela nao podem partilhar cursor"
    );
}
