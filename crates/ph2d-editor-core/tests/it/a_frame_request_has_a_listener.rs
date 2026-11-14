//! **O pedido de quadro das abas** (`Documents::wants_frame`: o voo do editor de animações, o Kanban, o SFX Lab) não
//! tem leitor na shell, e hoje não precisa: o laço redesenha sempre (`ControlFlow::Poll`). Decisão de 08/11
//! (`docs/GameEngineFuncional/27_ferramentas_na_folha_visivel.md` §4): o contrato fica, e este portão prende o par —
//! quem tirar o laço contínuo tem de ler o pedido, senão uma animação de aba congela até o rato mexer.
//! Lê o fonte da shell, como a catraca `the_shell_only_shrinks` (veio da shell, 10/11: a shell só encolhe).

use std::fs;
use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src")
}

/// *Mutação: trocar `ControlFlow::Poll` por `Wait` no `main.rs` ⇒ vermelho (ninguém lê o pedido).*
#[test]
fn sem_laco_continuo_alguem_le_o_pedido_de_quadro() {
    let src = shell_src();
    let main = fs::read_to_string(src.join("main.rs")).unwrap();
    if main.contains("set_control_flow(ControlFlow::Poll)") {
        return;
    }
    let mut leitores = Vec::new();
    let mut pilha = vec![src];
    while let Some(d) = pilha.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && fs::read_to_string(&p).unwrap().contains(".wants_frame()")
            {
                leitores.push(p);
            }
        }
    }
    assert!(
        !leitores.is_empty(),
        "o laço deixou de ser contínuo e nenhum sítio da shell lê `Documents::wants_frame`"
    );
}

/// B23-cont: o voo de uma máquina de Morph anda por quadro (`MorphMachine::is_flying`); sem leitor no
/// `pede_o_quadro`, o laço a pedido deixa-o a 4 q/s (o despertador) depois do prazo da tecla.
#[test]
fn o_voo_do_morph_pede_quadro() {
    assert!(
        corpo_do_leitor().contains(".is_flying()"),
        "o `pede_o_quadro` não lê o voo do Morph: a transição congela entre os despertadores"
    );
}

/// B23-cont: a prévia do editor de áudio a tocar (o cursor da onda) e os medidores andam por quadro.
#[test]
fn o_som_a_sair_pede_quadro() {
    assert!(
        corpo_do_leitor().contains(".audivel()"),
        "o `pede_o_quadro` não lê o som: o cursor da onda e os medidores andam a 4 q/s"
    );
}

/// B23-cont: a folha ao vivo conta `QUADROS_PARADOS` antes de ler a tinta e o check vive pelo relógio — a 4 q/s
/// a leitura chegava ~4 s depois do traço.
#[test]
fn a_folha_devida_pede_quadro() {
    assert!(
        corpo_do_leitor().contains("SheetCache::quer_quadro"),
        "o `pede_o_quadro` não lê a folha devida: a prévia da folha pintada demora segundos"
    );
}

/// B23-cont3: o Motion parado anima o eco de uma largada e recolhe a leitura dos cartões no quadro seguinte.
#[test]
fn a_sim_do_motion_parada_pede_quadro() {
    assert!(
        corpo_do_leitor().contains("motion_quadro::quer_quadro"),
        "o `pede_o_quadro` não lê o Motion parado: o eco e os cartões andam a 4 q/s"
    );
}

/// B23-cont3: desfocada ou oculta a sessão do Quadro dorme; o que chega da rede acorda o laço pelo
/// `EventLoopProxy` (o `user_event` pede o quadro), não pelo despertador de 250 ms.
#[test]
fn a_rede_acorda_o_laco_pelo_proxy() {
    let main = fs::read_to_string(shell_src().join("main.rs")).unwrap();
    assert!(
        main.contains("instala_o_despertador") && main.contains("fn user_event"),
        "a shell não instala o despertador do laço: a rede espera o prazo de 250 ms"
    );
}

/// B23.1 (mutação): o pedido passa pela lei ANTES do redraw; um `pede_o_quadro` incondicional (sem o
/// `acorda` e o `WaitUntil`) é o laço contínuo de antes com outro nome.
#[test]
fn o_pedido_de_quadro_passa_pela_lei() {
    let c = corpo_do_leitor();
    let lei = c
        .find(".acorda(")
        .expect("o `pede_o_quadro` não consulta a lei (`Laco::acorda`)");
    let dorme = c.find("WaitUntil").expect("o `pede_o_quadro` nunca dorme");
    let desenha = c
        .find("request_redraw")
        .expect("o `pede_o_quadro` nunca desenha");
    assert!(
        lei < dorme && dorme < desenha,
        "o redraw vem antes da lei: o laço é contínuo"
    );
}

fn corpo_do_leitor() -> String {
    let src = fs::read_to_string(shell_src().join("janela_do_jogo.rs")).unwrap();
    let corpo = src.split("fn pede_o_quadro").nth(1).expect("o leitor");
    corpo.split("\n    }\n").next().unwrap().to_owned()
}

/// B23.6: no Wayland o winit 0.30.13 não dá `Occluded` (minimizada, o `Laco` nunca sabe que está oculta e a
/// janela apresenta). O winit vendorizado (`crates/ph2d-app-host/vendor/winit/VENDOR.md`) converte o
/// `suspended` do xdg_toplevel em `WindowEvent::Occluded`, e a shell entrega-o ao `Laco::oculta`.
/// *Vermelho com o `event_loop/mod.rs` do upstream (o `occluded` não vira evento) ou sem o `[patch]`.*
#[test]
fn o_wayland_suspenso_chega_a_shell_como_occluded() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ler = |p: &str| fs::read_to_string(raiz.join(p)).unwrap_or_default();
    let wl = "crates/ph2d-app-host/vendor/winit/src/platform_impl/linux/wayland";
    let cargo = ler("Cargo.toml");
    let patch = &cargo[cargo.find("[patch.crates-io]").unwrap_or(cargo.len())..];
    assert!(
        patch.contains("winit = { path = \"crates/ph2d-app-host/vendor/winit\" }"),
        "o `[patch.crates-io]` não aponta o winit para o vendorizado"
    );
    assert!(
        ler(&format!("{wl}/window/state.rs")).contains("contains(XdgWindowState::SUSPENDED))"),
        "o winit vendorizado não lê o `suspended` do xdg_toplevel"
    );
    assert!(
        ler(&format!("{wl}/state.rs")).contains(".occluded = Some(suspended)"),
        "o `configure` do winit vendorizado não guarda a mudança do `suspended`"
    );
    assert!(
        ler(&format!("{wl}/event_loop/mod.rs")).contains("event: WindowEvent::Occluded(occluded)"),
        "o laço Wayland do winit vendorizado não entrega o `Occluded`"
    );
    assert!(
        ler("shells/desktop/src/main.rs").contains("WindowEvent::Occluded(o) => l.oculta(o)"),
        "a shell não entrega o `Occluded` ao `Laco::oculta`"
    );
}

/// B23.1: cada `Fase` do rastreio (ph2d-app-host) tem a sua `marca` no quadro da shell, uma vez.
/// *Mutação: tirar a `marca(super::Fase::Acquire)` do `present.rs` ⇒ vermelho.*
#[test]
fn cada_fase_do_quadro_marca_as_faltas() {
    let rastreio = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../ph2d-app-host/src/quadro_rastreio.rs"),
    )
    .unwrap();
    let fases: Vec<&str> = rastreio
        .split("(Fase::")
        .skip(1)
        .filter_map(|r| r.split_once(", \"").map(|(f, _)| f))
        .collect();
    assert_eq!(fases.len(), 7, "controlo positivo: as fases de `FASES`");
    let rl = shell_src().join("render_loop");
    let quadro: String = ["mod.rs", "fase_input_and_drops.rs", "present.rs"]
        .iter()
        .map(|f| fs::read_to_string(rl.join(f)).unwrap())
        .collect();
    for f in fases {
        let n = quadro.matches(&format!("Fase::{f})")).count();
        assert_eq!(
            n, 1,
            "a fase `{f}` tem {n} marcas no quadro da shell (quer 1)"
        );
    }
}

/// B23.2 (12/11): as molas do chrome (hover, cascata, voo da vista), o piscar do `Ctrl+K`/cartão solto e os
/// indicadores dos pontos andam pelo relógio de parede; sem leitor, a 4 q/s a mola salta e pára a meio.
#[test]
fn o_chrome_que_anima_sozinho_pede_quadro() {
    let corpo = corpo_do_leitor();
    for p in [".a_ui_mexe()", "anima_pontos::a_mexer"] {
        assert!(
            corpo.contains(p),
            "o `pede_o_quadro` não lê `{p}`: o chrome anda a 4 q/s"
        );
    }
}

/// A lei por trás do leitor: a mola em voo e o piscar pedem quadro; assentados, deixam o laço dormir.
#[test]
fn as_molas_e_o_piscar_do_chrome_pedem_quadro_ate_assentar() {
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::motion::Role;
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(NodeId(1));
    assert!(!hero.a_ui_mexe(), "parado de fábrica não pede quadro");
    hero.motion.animate(NodeId(7), 0.0, Role::Surface);
    hero.motion.animate(NodeId(7), 1.0, Role::Surface);
    assert!(
        hero.a_ui_mexe(),
        "a mola em voo tem de pedir o quadro seguinte"
    );
    for _ in 0..600 {
        hero.motion.advance(1.0 / 60.0);
    }
    assert!(!hero.a_ui_mexe(), "assentada, a mola larga o laço");
    hero.store.note_clock(1);
    hero.store.flash(NodeId(9));
    assert!(hero.a_ui_mexe(), "o piscar do comando tem de pedir quadro");
    hero.store.note_clock(10_000_000_000);
    assert!(
        !hero.a_ui_mexe(),
        "o piscar acabou (3 pulsos): o laço dorme"
    );
}
