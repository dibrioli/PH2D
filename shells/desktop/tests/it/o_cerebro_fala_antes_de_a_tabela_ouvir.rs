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
    let src = crate::frame_text::render_frame();
    let script = src
        .find("motores_do_quadro::scripts(")
        .expect("os scripts nao correm no quadro — o componente seria inerte");
    let tabela = src
        .find(".read(&mut self.signal_readers.action)")
        .expect("a tabela de accoes mudou de forma");
    assert!(
        script < tabela,
        "a tabela de accoes le' ANTES de os scripts emitirem: um `ph2d.emit` chegaria um quadro \
         atrasado"
    );
    // ⚠️ **O cursor é um ARGUMENTO da chamada** desde que os dois motores saíram para o irmão
    // `motores_do_quadro` (tecto de LOC): a leitura deixou de estar no texto do quadro, e o que
    // este gate mede aqui é que o motor recebe o cursor PRÓPRIO — partilhar o da tabela deixaria
    // uma das duas sem sinais. *Quem lê ANTES de correr é medido no ficheiro do motor, abaixo.*
    let chamada = &src[script..];
    let fim = chamada.find(");").expect("a chamada acaba");
    assert!(
        chamada[..fim].contains("&mut self.signal_readers.script"),
        "os scripts nao tem cursor proprio"
    );
}

/// ⭐⭐⭐ **E os EMISSORES DE PARTÍCULAS gritam na mesma janela** (TOP-20 #18): o `finished` de uma
/// rajada tem de chegar à tabela de acções no MESMO quadro em que a última partícula morreu.
///
/// **Mutação que deve sangrar:** mover o bloco das partículas para depois da tabela.
#[test]
fn as_particulas_gritam_antes_de_a_tabela_de_accoes_ler() {
    let src = crate::frame_text::render_frame();
    let particulas = src
        .find("motores_do_quadro::particulas(")
        .expect("os emissores nao correm no quadro — o componente seria inerte");
    let tabela = src
        .find(".read(&mut self.signal_readers.action)")
        .expect("a tabela de accoes mudou de forma");
    assert!(
        particulas < tabela,
        "a tabela le' ANTES de as particulas gritarem: um `finished` chegaria um quadro atrasado"
    );
    // E o cursor é PRÓPRIO — partilhar o da tabela deixaria uma das duas sem sinais.
    let chamada = &src[particulas..];
    let fim = chamada.find(");").expect("a chamada acaba");
    assert!(
        chamada[..fim].contains("&mut self.signal_readers.particles"),
        "os emissores nao tem cursor proprio"
    );
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
        ("pub(super) fn scripts(", "script_bridge::frame("),
        ("pub(super) fn particulas(", "particles.frame("),
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
