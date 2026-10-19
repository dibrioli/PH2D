//! ⭐ **I13 — formatar ao salvar** (decisão 6 do dono): o Ctrl+S aplica e dá o selo no MESMO quadro;
//! o formatador corre fora; a resposta entra como UM passo de desfazer só se o texto não mudou entretanto,
//! e a aba salva-se outra vez. Os falsos provam a ordem; o StyLua fixado prova o texto, o cursor e as
//! recusas (`bash scripts/stylua-fetch.sh`).

use crate::{Mesa, ctrl};
use ph2d_app_code::FormatServer;
use ph2d_app_code::commands::CMD_FORMAT_ON_SAVE;
use ph2d_code_format::Refusal;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_panel_code::Badge;
use ph2d_script::ScriptId;
use std::sync::{Arc, Mutex, mpsc};

const TORTO: &str = "local x=1\nfunction update(self, dt)\nend\n";
const DIREITO: &str = "local x = 1\nfunction update(self, dt)\nend\n";
const BOB: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");
const BOB_FORMATADO: &str =
    include_str!("../../../ph2d-code-format/tests/it/fixtures/bob.formatado.luau");
const BOB_SEM_END: &str =
    include_str!("../../../ph2d-code-format/tests/it/fixtures/bob.sem_end.luau");

fn espacos() -> FormatServer {
    FormatServer::Custom(Arc::new(|t: &str| Ok(t.replace("x=1", "x = 1"))))
}

/// Um formatador que só responde quando o teste solta (`tx.send(())`).
fn preso() -> (FormatServer, mpsc::Sender<()>) {
    let (tx, rx) = mpsc::channel::<()>();
    let rx = Mutex::new(rx);
    let f = FormatServer::Custom(Arc::new(move |t: &str| {
        let _ = rx.lock().unwrap().recv();
        Ok(t.replace("x=1", "x = 1"))
    }));
    (f, tx)
}

fn salva(m: &mut Mesa, id: ScriptId) {
    m.tecla(id, Key::Char('s'), ctrl(), Some("s"));
    m.quadro();
}

fn desfaz(m: &mut Mesa, id: ScriptId) {
    m.tecla(id, Key::Char('z'), ctrl(), Some("z"));
    m.quadro();
}

fn refaz(m: &mut Mesa, id: ScriptId) {
    let shift = Modifiers {
        shift: true,
        ..ctrl()
    };
    m.tecla(id, Key::Char('z'), shift, Some("Z"));
    m.quadro();
}

/// A resposta chega e o quadro seguinte aplica-a (o caminho do produto).
fn chega(m: &mut Mesa) {
    m.docs().wait_format();
    m.quadro();
}

fn texto(m: &mut Mesa, id: ScriptId) -> String {
    m.docs().text_of(id).expect("aberto")
}

/// O cursor no byte `b` pelo teclado: Ctrl+G na linha, depois → até à coluna.
fn poe(m: &mut Mesa, id: ScriptId, b: usize) {
    let t = texto(m, id);
    let linha = t[..b].matches('\n').count();
    let inicio = t[..b].rfind('\n').map_or(0, |i| i + 1);
    m.pinta(id);
    m.tecla(id, Key::Char('g'), ctrl(), None);
    m.escreve(id, &format!("{}\n", linha + 1));
    m.tecla(id, Key::Home, Modifiers::default(), None);
    if m.docs().selection_of(id) != [inicio..inicio] {
        m.tecla(id, Key::Home, Modifiers::default(), None); // o Home inteligente: 1.º ao recuo
    }
    for _ in t[inicio..b].chars() {
        m.tecla(id, Key::Right, Modifiers::default(), None);
    }
    assert_eq!(m.docs().selection_of(id), [b..b], "o cursor no byte {b}");
}

#[test]
fn o_ctrl_s_aplica_no_quadro_e_a_formatacao_entra_num_passo_de_desfazer() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(espacos());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    m.escreve(id, "-- a\n");
    salva(&mut m, id);
    let antes = format!("-- a\n{TORTO}");
    assert_eq!(
        m.texto_salvo(id),
        antes,
        "o Ctrl+S aplica o texto como está, no MESMO quadro"
    );
    assert_eq!(m.docs().badge_of(id), Some(Badge::Saved), "e o selo também");
    assert!(!m.docs().is_dirty(id));
    assert_eq!(texto(&mut m, id), antes, "a formatação ainda não chegou");

    chega(&mut m);
    let depois = format!("-- a\n{DIREITO}");
    assert_eq!(texto(&mut m, id), depois);
    assert_eq!(
        m.texto_salvo(id),
        depois,
        "o formatado também fica salvo (salvar = guardado)"
    );
    assert!(!m.docs().is_dirty(id), "sem ● depois da formatação");
    assert_eq!(
        m.docs().selection_of(id),
        [5..5],
        "o cursor não sai do sítio"
    );

    desfaz(&mut m, id);
    assert_eq!(texto(&mut m, id), antes, "UM Ctrl+Z tira só a formatação");
    assert!(m.docs().is_dirty(id), "o salvo é o formatado");
    refaz(&mut m, id);
    assert_eq!(texto(&mut m, id), depois);
    assert!(!m.docs().is_dirty(id));
}

#[test]
fn o_que_se_escreve_antes_da_resposta_ganha_a_formatacao() {
    let (f, solta) = preso();
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(f);
    let id = ids[0];
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    m.escreve(id, "y");
    solta.send(()).unwrap();
    chega(&mut m);
    assert_eq!(
        texto(&mut m, id),
        format!("y{TORTO}"),
        "o texto andou: a resposta larga-se"
    );
    assert_eq!(m.texto_salvo(id), TORTO);
    assert!(m.docs().is_dirty(id));

    // Dois Ctrl+S seguidos: só a resposta do último entra, num passo só.
    desfaz(&mut m, id);
    salva(&mut m, id);
    salva(&mut m, id);
    solta.send(()).unwrap();
    solta.send(()).unwrap();
    chega(&mut m);
    assert_eq!(m.docs().format_in_flight(), 0);
    assert_eq!(texto(&mut m, id), DIREITO);
    desfaz(&mut m, id);
    assert_eq!(
        texto(&mut m, id),
        TORTO,
        "um Ctrl+Z tira a formatação inteira"
    );
}

/// Ctrl+S, escrever, Ctrl+S: a 1.ª resposta foi feita sobre o texto VELHO — aplicá-la ao novo
/// partia-o. Só a do último pedido entra.
#[test]
fn a_resposta_de_um_pedido_velho_nunca_entra_no_texto_novo() {
    // Cada pedido espera a SUA chave (o texto diz qual): a ordem das respostas é a do teste.
    let (velho_tx, velho_rx) = mpsc::channel::<()>();
    let (novo_tx, novo_rx) = mpsc::channel::<()>();
    let (velho_rx, novo_rx) = (Mutex::new(velho_rx), Mutex::new(novo_rx));
    let f = FormatServer::Custom(Arc::new(move |t: &str| {
        let rx = if t.starts_with("-- c") { &novo_rx } else { &velho_rx };
        let _ = rx.lock().unwrap().recv();
        Ok(t.replace("x=1", "x = 1"))
    }));
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(f);
    let id = ids[0];
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    // No INÍCIO: desloca tudo, e as edições da 1.ª resposta (bytes do texto velho) cairiam ao lado.
    m.tecla(id, Key::Home, ctrl(), None);
    m.escreve(id, "-- c\n");
    salva(&mut m, id);
    assert_eq!(m.docs().format_in_flight(), 2);
    // A resposta VELHA chega primeiro, com o pedido novo ainda à espera.
    velho_tx.send(()).unwrap();
    let ate = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while m.docs().format_in_flight() == 2 && std::time::Instant::now() < ate {
        m.quadro();
        std::thread::yield_now();
    }
    assert_eq!(m.docs().format_in_flight(), 1, "a velha chegou");
    assert_eq!(texto(&mut m, id), format!("-- c\n{TORTO}"), "e não entrou");
    novo_tx.send(()).unwrap();
    chega(&mut m);
    assert_eq!(texto(&mut m, id), format!("-- c\n{DIREITO}"));
    assert!(!m.docs().is_dirty(id));
}

/// A formatação nunca se cola à escrita de antes: uma letra escrita mesmo antes do Ctrl+S, com o
/// cursor no mesmo sítio, continua no seu próprio passo de desfazer.
#[test]
fn a_formatacao_nunca_se_junta_a_escrita_de_antes() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(espacos());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    m.tecla(id, Key::End, ctrl(), None);
    m.escreve(id, "z");
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), format!("{DIREITO}z"));
    desfaz(&mut m, id);
    assert_eq!(texto(&mut m, id), format!("{TORTO}z"), "só a formatação sai");
    desfaz(&mut m, id);
    assert_eq!(texto(&mut m, id), TORTO);
}

#[test]
fn o_comando_liga_e_desliga_em_todas_as_abas() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(espacos());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    let rotulo = |m: &mut Mesa| {
        m.docs()
            .commands(id.0)
            .into_iter()
            .find(|c| c.id == CMD_FORMAT_ON_SAVE)
            .expect("o comando está na aba")
            .label
    };
    assert_eq!(
        rotulo(&mut m),
        "Turn Off Format on Save",
        "ligado por omissão"
    );
    assert!(m.docs().command(id.0, CMD_FORMAT_ON_SAVE));
    assert_eq!(rotulo(&mut m), "Turn On Format on Save");
    salva(&mut m, id);
    assert_eq!(m.docs().format_in_flight(), 0, "desligado: nada se pede");
    chega(&mut m);
    assert_eq!(texto(&mut m, id), TORTO);
    assert!(m.docs().command(id.0, CMD_FORMAT_ON_SAVE));
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), DIREITO);
}

#[test]
fn sem_o_programa_salva_e_avisa_uma_vez() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(FormatServer::Binary("/nao/existe/stylua".into()));
    let id = ids[0];
    m.pede(id);
    m.quadro();
    let antes = m.toasts.len();
    salva(&mut m, id);
    assert_eq!(m.texto_salvo(id), TORTO, "salvou na mesma");
    assert_eq!(m.toasts.len(), antes + 1, "o aviso do ecrã");
    salva(&mut m, id);
    assert_eq!(m.toasts.len(), antes + 1, "uma vez por sessão");
}

#[test]
fn a_recusa_do_formatador_vai_ao_console_e_o_texto_fica() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let recusa = FormatServer::Custom(Arc::new(|_: &str| Err(Refusal::WouldChangeCode)));
    let mut m = m.com_formatador(recusa);
    let id = ids[0];
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), TORTO);
    assert!(!m.docs().is_dirty(id));
    let console = m.docs().console_texts(id);
    assert_eq!(
        console,
        [ph2d_i18n::tr("code.format.would_change").to_owned()],
        "a nota diz porquê"
    );
}

#[test]
fn com_ficheiro_o_projecto_grava_se_outra_vez_formatado() {
    let (m, ids) = Mesa::new(&[("bob.luau", TORTO)]);
    let mut m = m.com_formatador(espacos());
    let id = ids[0];
    let p = std::env::temp_dir().join(format!("ph2d-i13-{}.ph2dproj", std::process::id()));
    m.ficheiro = Some(p.clone());
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    assert!(m.hero.file_menu.save);
    m.shell_drena();
    chega(&mut m);
    assert!(
        m.hero.file_menu.save,
        "a formatação que entrou grava o projecto outra vez"
    );
    m.shell_drena();
    let gravado = std::fs::read(&p).expect("gravado");
    let _ = std::fs::remove_file(&p);
    let lib = ph2d_script::ScriptLibrary::from_bytes(&gravado).expect("lê");
    assert_eq!(lib.get(id).map(|s| s.text.as_str()), Some(DIREITO));
}

fn fixado() -> FormatServer {
    FormatServer::Pinned
}

/// ⭐ Com o StyLua FIXADO pelo caminho do produto: o `bob.luau` sai byte a byte como o programa na
/// mão (a fixtura do I0d), e o cursor antes do `return` que o StyLua parte para a linha de baixo
/// continua antes do `return`.
#[test]
fn com_o_stylua_fixado_o_bob_sai_como_o_programa_e_o_cursor_fica_no_codigo() {
    let (m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let mut m = m.com_formatador(fixado());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    let ret = BOB.find("then return end").expect("o if de uma linha") + "then ".len();
    poe(&mut m, id, ret);
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), BOB_FORMATADO);
    assert_eq!(m.texto_salvo(id), BOB_FORMATADO);
    let novo = BOB_FORMATADO.find("then\n    return").expect("partido") + "then\n    ".len();
    assert_eq!(
        m.docs().selection_of(id),
        [novo..novo],
        "antes do `return`, na linha nova"
    );
    desfaz(&mut m, id);
    assert_eq!(texto(&mut m, id), BOB);
    assert_eq!(
        m.docs().selection_of(id),
        [ret..ret],
        "o desfazer devolve o cursor"
    );
}

/// ⛔ O que o produto TRAZ não muda ao primeiro Ctrl+S: o modelo do «New script» já sai do StyLua
/// fixado tal como entra (a 2.5.2 dobra uma função vazia numa linha só). O par na cena:
/// `ph2d-app-components/tests/it/os_scripts_da_cena_saem_limpos_do_formatador.rs`.
#[test]
fn o_modelo_do_script_novo_sai_limpo_do_stylua_fixado() {
    let p = ph2d_code_format::Program::pinned().expect("`bash scripts/stylua-fetch.sh`");
    let t = ph2d_app_code::list::NEW_SCRIPT_TEXT;
    assert_eq!(p.format(t).as_deref(), Ok(t));
    let vazio = "function init(self)\nend\n";
    assert_ne!(
        p.format(vazio).as_deref(),
        Ok(vazio),
        "controlo: a função vazia dobra-se"
    );
}

/// Erro de sintaxe: salva sem formatar, sem nota (o selo do Luau já diz a linha).
#[test]
fn com_o_stylua_fixado_um_texto_partido_salva_sem_formatar() {
    let (m, ids) = Mesa::new(&[("bob.luau", BOB_SEM_END)]);
    let mut m = m.com_formatador(fixado());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), BOB_SEM_END);
    assert_eq!(m.texto_salvo(id), BOB_SEM_END);
    assert!(!m.docs().is_dirty(id));
    assert!(m.docs().console_texts(id).is_empty());
}

/// ⛔ A 2.5.2 mudaria `{ read number }`: o `--verify` recusa e o console diz porquê.
#[test]
fn com_o_stylua_fixado_o_que_mudaria_o_codigo_nao_se_formata() {
    let t = "type A = { read number }\nlocal x=1\n";
    let (m, ids) = Mesa::new(&[("bob.luau", t)]);
    let mut m = m.com_formatador(fixado());
    let id = ids[0];
    m.pede(id);
    m.quadro();
    salva(&mut m, id);
    chega(&mut m);
    assert_eq!(texto(&mut m, id), t);
    assert_eq!(
        m.docs().console_texts(id),
        [ph2d_i18n::tr("code.format.would_change").to_owned()]
    );
}

/// A medição do quadro em que a formatação ENTRA (o fio da UI: as edições no buffer, quem as segue,
/// e salvar outra vez) a 51 e a 7 500 linhas, e no pior caso (toda linha re-recuada). Só imprime: o
/// registo vai para a medição do I13.
#[test]
fn medicao_do_quadro_em_que_a_formatacao_entra() {
    let bob = |n: usize| -> String {
        let b: Vec<&str> = BOB.lines().skip(2).collect();
        (0..n)
            .map(|i| {
                let l = b[i % b.len()];
                if i >= b.len() && l.starts_with("ph2d.property(") {
                    format!("-- {l}\n")
                } else {
                    format!("{l}\n")
                }
            })
            .collect()
    };
    let torto = |t: &str| -> String { t.lines().map(|l| format!("    {l}\n")).collect() };
    println!(
        "loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (nome, t) in [
        ("51", bob(51)),
        ("7500", bob(7500)),
        ("7500 re-recuadas", torto(&bob(7500))),
    ] {
        let mut melhor = f64::INFINITY;
        for _ in 0..5 {
            let (m, ids) = Mesa::new(&[("bob.luau", t.as_str())]);
            let mut m = m.com_formatador(fixado());
            let id = ids[0];
            m.pede(id);
            m.quadro();
            m.pinta(id);
            salva(&mut m, id);
            m.docs().wait_format();
            let a = std::time::Instant::now();
            m.quadro();
            melhor = melhor.min(a.elapsed().as_secs_f64() * 1e3);
            assert_ne!(texto(&mut m, id), t, "{nome}: formatou");
            assert!(!m.docs().is_dirty(id));
        }
        println!("{nome} linhas: o quadro em que entra, min {melhor:.2} ms");
    }
}
