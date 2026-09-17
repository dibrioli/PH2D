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

/// ⭐⭐⭐ **E os SCRIPTS do artista falam na mesma janela** (TOP-20 #16): um `ph2d.emit` tem de chegar
/// à tabela de acções no MESMO quadro.
///
/// **Mutação que deve sangrar:** mover o bloco dos scripts para depois da tabela.
#[test]
fn o_script_emite_antes_de_a_tabela_de_accoes_ler() {
    ordem_dos_motores("scripts(");
}

/// ⭐⭐⭐ **A ordem de UM motor, em DUAS metades — e elas reprovam por motivos DIFERENTES.**
///
/// ⚠️⚠️ **A partição foi IMPOSTA quando as três chamadas se agruparam numa porta** (o tecto de 200
/// LOC do `fase_signal_outbox`): o texto emendado do quadro colhe só as `fase_*`, logo o corpo do
/// `motores_do_quadro` deixou de estar nele.
///
/// ⛔ **Emendar os dois ficheiros para medir a ordem seria FRAUDE** — tudo o que está no irmão viria
/// depois de tudo o que está na fase, e a asserção passaria **por construção**. É a lei que o corte
/// do teclado da escultura já pagou (13/09).
///
/// ⇒ (a) no quadro, a PORTA corre antes de a tabela ler · (b) no ficheiro do motor, a porta chama
/// aquele motor. Apagar qualquer uma delas reprova, e nenhuma implica a outra.
fn ordem_dos_motores(motor: &str) {
    let src = crate::frame_text::render_frame();
    let porta = src
        .find("motores_do_quadro::correm(")
        .expect("os motores nao correm no quadro — os componentes seriam inertes");
    let tabela = src
        .find(".read(&mut self.signal_readers.action)")
        .expect("a tabela de accoes mudou de forma — este gate mede a ordem contra ela");
    assert!(
        porta < tabela,
        "a tabela de accoes le' ANTES de os motores falarem: um `ph2d.emit`, um `finished` ou uma \
         travessia de contador chegariam um quadro atrasado"
    );
    // (b) E a porta de facto chama ESTE motor. ⚠️ `include_str!` falha a COMPILAR se alguém mover
    // o ficheiro — o modo de falha alto que o HOWTO pede.
    let irmao = include_str!("../../src/render_loop/motores_do_quadro.rs");
    let corpo = &irmao[irmao
        .find("pub(super) fn correm(")
        .expect("a porta dos motores mudou de nome")..];
    let fim = corpo.find("\n}\n").expect("a porta acaba");
    assert!(
        corpo[..fim].contains(motor),
        "a porta dos motores nao chama `{motor}` — ele seria inerte com o quadro inteiro verde"
    );
}

/// ⭐⭐⭐ **E os EMISSORES DE PARTÍCULAS gritam na mesma janela** (TOP-20 #18): o `finished` de uma
/// rajada tem de chegar à tabela de acções no MESMO quadro em que a última partícula morreu.
///
/// **Mutação que deve sangrar:** mover o bloco das partículas para depois da tabela.
#[test]
fn as_particulas_gritam_antes_de_a_tabela_de_accoes_ler() {
    ordem_dos_motores("particulas(");
}

/// ⭐⭐⭐ **E a VIGIA DO CONTADOR fala na mesma janela** — uma travessia de limiar chega à tabela de
/// acções no MESMO quadro.
///
/// **Mutação que deve sangrar:** tirar o `vigias(…)` da porta dos motores.
#[test]
fn a_vigia_fala_antes_de_a_tabela_de_accoes_ler() {
    ordem_dos_motores("vigias(");
}

/// ⚠️ **Cada motor recebe o cursor PRÓPRIO** — partilhar o da tabela deixaria uma das duas sem
/// sinais. ⛔ A vigia **não aparece aqui de propósito**: ela não OUVE, só fala, e é a única fonte
/// desta janela cuja entrada é o estado do MUNDO.
#[test]
fn cada_motor_que_ouve_tem_cursor_proprio() {
    let src = include_str!("../../src/render_loop/motores_do_quadro.rs");
    for (motor, cursor) in [
        ("fn scripts(", "&mut leitores.script"),
        ("fn particulas(", "&mut leitores.particles"),
    ] {
        assert!(
            src.contains(motor),
            "o motor `{motor}` saiu do ficheiro — este gate mede a fiacao contra ele"
        );
        assert!(
            src.contains(cursor),
            "`{motor}` nao recebe o cursor proprio (`{cursor}`)"
        );
    }
}

/// ⭐⭐ **As partículas desenham-se com QUALQUER ferramenta** — ao contrário do stream do Motion,
/// que só existe com a ferramenta MOTION na mão.
///
/// **Mutação que deve sangrar:** pôr o slice das partículas dentro do `if motion_active`.
#[test]
fn as_particulas_desenham_se_com_qualquer_ferramenta() {
    // ⚠️ **A lente aqui é o FICHEIRO do presente, e não o texto emendado do quadro:** o
    // `run_present_phase` não é uma `fase_*`, logo não é emendado. O `include_str!` falha a
    // COMPILAR se alguém mover o ficheiro — o modo de falha alto que o HOWTO pede.
    let src = include_str!("../../src/render_loop/present.rs");
    let slice = src
        .find("let particulas: &[ph2d_render::RenderInstance] = &particles.instances;")
        .expect("as particulas nao entram no passe de sprites — o componente seria invisivel");
    let motion_gate = src
        .find("let motion_slice: &[ph2d_render::RenderInstance] = if motion_active")
        .expect("o slice do Motion mudou de forma — este gate mede a diferenca contra ele");
    // As duas linhas existem, e a das partículas NÃO é a do Motion: ela não pergunta pela
    // ferramenta. (A ordem entre elas não importa; o que importa é a ausência da guarda.)
    assert_ne!(slice, motion_gate);
    let depois = &src[slice..];
    let fim = depois.find(';').expect("uma linha acaba");
    assert!(
        !depois[..fim].contains("motion_active"),
        "o slice das particulas passou a depender da ferramenta MOTION"
    );
}

/// ⭐⭐ **E cada motor OUVE antes de ANDAR** — a metade que saiu do texto do quadro quando os dois
/// motores foram para o irmão `motores_do_quadro`.
///
/// ⚠️ A lente é o FICHEIRO do motor, por `include_str!` (falha a COMPILAR se alguém o mover).
///
/// **Mutação que deve sangrar:** ler o cursor depois de chamar a ponte.
#[test]
fn cada_motor_ouve_antes_de_andar() {
    let src = include_str!("../../src/render_loop/motores_do_quadro.rs");
    for (motor, ponte) in [
        ("fn scripts(", "script_bridge::frame("),
        ("fn particulas(", "particles.frame("),
    ] {
        let corpo = &src[src.find(motor).unwrap_or_else(|| panic!("falta `{motor}`"))..];
        let leitura = corpo.find("signals.read(reader)").unwrap_or_else(|| {
            panic!("`{motor}` nao le' o cursor — os sinais deste quadro nao chegam")
        });
        let anda = corpo
            .find(ponte)
            .unwrap_or_else(|| panic!("`{motor}` nao chama a ponte"));
        assert!(
            leitura < anda,
            "`{motor}` anda ANTES de ouvir: um sinal deste quadro so' chegaria no seguinte"
        );
    }
}
