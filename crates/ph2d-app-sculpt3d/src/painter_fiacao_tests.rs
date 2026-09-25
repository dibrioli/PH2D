//! ⭐⭐⭐ **O CENSO DA FIAÇÃO DO PAINTER NA PEÇA** — os elos que a costura
//! [`crate::painter_na_malha`] precisa de ter LIGADOS, na família e na shell.
//!
//! ⛔⛔ **Porque é um censo de TEXTO e não um gate de comportamento:** a prova de
//! COMPORTAMENTO existe e vive no `tinta_no_produto_painter.rs` — um traço do
//! Painter aterra na peça e o `Ctrl+Z` devolve-a ao bit —, mas aqueles gates são
//! `#[ignore]` + placa (uma cena pede um `wgpu::Device`), e **nem a suíte da
//! família nem o CI correm um `#[ignore]`**. E os três elos da shell nem esses
//! gates os alcançam: eles chamam [`crate::painter_na_malha::entrega`] e
//! [`crate::painter_na_malha::quadro`] directamente, logo uma shell que deixasse
//! de os chamar ficava verde em todo o lado — *a lei certa numa porta que ninguém
//! chama lê-se exactamente como a lei ausente*, a forma que o censo irmão
//! (`tinta_fiacao_tests.rs`) já pagou cinco vezes.
//!
//! ⚠️ **Cada agulha leva a CONTAGEM que o código tem, e não «está presente»:** a
//! soltura da tela é chamada DUAS vezes no [`crate::painter_na_malha::quadro`] (sem
//! barro e sem cena), e uma mutação que apagasse uma delas deixaria a outra a
//! satisfazer uma agulha de presença.
//!
//! ⛔ **A prosa é CORTADA antes de se medir:** um doc-comment que EXPLICA a
//! costura contém os nomes dela, e o [`so_a_prosa`] é o CONTROLO de que nenhuma
//! agulha se satisfaz a partir de um comentário.

fn sem_prosa(f: &str) -> String {
    f.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn so_a_prosa(f: &str) -> String {
    f.lines()
        .filter(|l| l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

const COSTURA: &str = include_str!("painter_na_malha.rs");
const CURSOR: &str = include_str!("cursor.rs");
/// ⚠️ **Caminho relativo para FORA da crate, e é de propósito:** os três elos da
/// shell são o que nenhum gate de produto alcança. Um `git mv` de qualquer um
/// destes ficheiros faz isto **falhar a COMPILAR**, que é a metade barata da
/// família (HOWTO §2).
const HOST: &str = include_str!("../../../shells/desktop/src/sculpt3d_host.rs");
const ENTREGA: &str =
    include_str!("../../../shells/desktop/src/input_dispatch/painter_canvas_input.rs");
const QUADRO: &str =
    include_str!("../../../shells/desktop/src/render_loop/fase_painter_dispatch.rs");
/// As quatro portas que mexem na peça e têm de fechar a pincelada que escorre
/// (etapa 3) — cada uma é um ficheiro da família.
const UNDO: &str = include_str!("undo.rs");
const KEYS: &str = include_str!("keys.rs");
const PANEL: &str = include_str!("panel.rs");
const INPUT_DOWN: &str = include_str!("input_down.rs");

/// Cada elo: o ficheiro, a agulha, quantas vezes o código a tem, e o que parte
/// quando ela some.
fn elos() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    usize,
    &'static str,
)> {
    vec![
        (
            "sculpt3d_host.rs",
            HOST,
            ".is_some_and(|p| p.on_screen_canvas())",
            1,
            "P1 a escultura rouba o botão ESQUERDO ao Painter",
        ),
        (
            "painter_canvas_input.rs",
            ENTREGA,
            "ph2d_app_sculpt3d::painter_na_malha::entrega(",
            1,
            "P2 o ponteiro do Painter nunca chega à peça",
        ),
        (
            "fase_painter_dispatch.rs",
            QUADRO,
            "ph2d_app_sculpt3d::painter_na_malha::quadro(",
            1,
            "P3 a tela da vista nunca é presa",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "s.painter_pousa(&f)",
            1,
            "P4 o traço só aparece na peça quando o dedo se levanta",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "scene.painter_fecha();",
            1,
            "P5 o traço nunca fecha: sem passo de desfazer",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "painter.clear_screen_canvas();",
            2,
            "P6 a tela não se limpa: o traço seguinte recompõe o anterior",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "painter.release_screen_canvas()",
            2,
            "P7 a tela fica presa sem barro: a ponte da sprite fica cega",
        ),
        (
            "cursor.rs",
            CURSOR,
            "self.painter_raio_px",
            1,
            "P8 o anel deitado na peça mente sobre o pincel do Painter",
        ),
        // ── ETAPA 2: a tela semeada ──
        (
            "painter_na_malha.rs",
            COSTURA,
            "painter.screen_canvas_reads_the_piece()",
            1,
            "P9 os modos que lêem a cor debaixo do pincel borram o VAZIO",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "sessao.com_semente(retrato.clone());",
            1,
            "P10 a tela semeada é pousada como «over»: a peça inteira vira o retrato",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "let _ = painter.take_screen_canvas();",
            2,
            "P11 o retrato (ou a limpeza de uma tela molhada) é pousado como mudança: \
             o quadro seguinte varre a peça inteira",
        ),
        (
            "painter_canvas_input.rs",
            ENTREGA,
            "super::painter_canvas_mods::forward(painter, shift, ctrl, alt);",
            2,
            "P12 Shift/Ctrl/Alt não chegam ao Painter sobre a peça",
        ),
        // ── o anel do Liquify e a aquarela molhada (report de 24/09) ──
        (
            "painter_na_malha.rs",
            COSTURA,
            "s.painter_raio_px = Some(painter.screen_canvas_ring_px());",
            1,
            "P13 o anel do Liquify fica no tamanho do pincel de pintura",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "painter.screen_canvas_is_wet()",
            2,
            "P14 a aquarela seca a cada traço: limpar ou semear a tela seca o papel",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "scene.painter_guarda(vista, retrato);",
            1,
            "P15 a tela molhada nunca é guardada: o traço seguinte nasce em papel seco",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "self.painter_ultima = Some(Arc::clone(&f.rgba));",
            1,
            "P16 a semente reaproveitada não é o que a peça recebeu: o traço anterior soma duas vezes",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "s.painter_molhada = None;",
            2,
            "P17 uma tela renascida ou solta é tomada pela molhada guardada",
        ),
        // ── ETAPA 3: a tinta molhada que escorre depois de largar ──
        (
            "painter_na_malha.rs",
            COSTURA,
            "painter.screen_canvas_is_flowing()",
            2,
            "P18 o traço fecha no pen-up com a água a correr (ou nunca fecha quando ela pára)",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "s.painter_escorre_se_a_peca_mudou();",
            1,
            "P19 outra mão mexe na peça e a água continua a escrever por cima",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "self.painter_escorre = Some(self.edits);",
            1,
            "P20 o pouso da própria água lê-se como outra mão: o traço fecha no 1.º pouso",
        ),
        (
            "undo.rs",
            UNDO,
            "self.painter_fecha_o_que_escorre();",
            1,
            "P21 o Ctrl+Z desfaz o passo ANTERIOR com a pincelada ainda aberta",
        ),
        (
            "keys.rs",
            KEYS,
            "scene.painter_fecha_o_que_escorre();",
            1,
            "P22 uma tecla da escultura mexe na peça com a pincelada ainda aberta",
        ),
        (
            "panel.rs",
            PANEL,
            "self.painter_fecha_o_que_escorre();",
            1,
            "P23 o painel muda a peça (e a topologia) com o plano ainda emprestado",
        ),
        (
            "input_down.rs",
            INPUT_DOWN,
            "scene.painter_fecha_o_que_escorre();",
            1,
            "P24 um clique da escultura abre um traço por cima da pincelada aberta",
        ),
        (
            "panel.rs",
            PANEL,
            "crate::painter_na_malha::o_painel_mexe_na_peca(&intent)",
            1,
            "P25 um AJUSTE do painel (a cor do pincel) fecha a pincelada que escorre",
        ),
        (
            "painter_na_malha.rs",
            COSTURA,
            "if scene.painter_escorre.is_some() {",
            1,
            "P26 o 2.º traço fecha a pincelada que escorre SEM guardar a tela: a água morre",
        ),
    ]
}

/// ⭐⭐⭐ **GATE — cada elo está no CÓDIGO, com a contagem que ele tem.**
#[test]
fn a_costura_do_painter_esta_ligada_nas_duas_pontas() {
    let elos = elos();
    assert!(elos.len() >= 26, "o piso de população: {} elos", elos.len());
    for (ficheiro, texto, agulha, esperado, parte) in elos {
        let n = sem_prosa(texto).matches(agulha).count();
        assert_eq!(
            n, esperado,
            "{ficheiro}: `{agulha}` aparece {n} vez(es) no código e devia aparecer \
             {esperado} — {parte}"
        );
    }
}

/// **O CONTROLO** — nenhuma agulha vive na prosa, senão a metade de cima
/// poderia satisfazer-se a partir de um comentário que explica a costura.
#[test]
fn nenhuma_agulha_da_costura_vive_na_prosa() {
    for (ficheiro, texto, agulha, _, _) in elos() {
        assert!(
            !so_a_prosa(texto).contains(agulha),
            "{ficheiro}: `{agulha}` aparece num comentário — o censo leria a prosa"
        );
    }
}

/// ⭐⭐ **GATE — um AJUSTE do painel não fecha a pincelada que escorre; um GESTO
/// fecha.** Report do dono (24/09): *«a simulação seca (para) ao trocar a cor do
/// pincel»* — a caixa de cor do painel é um `SetUi`. ⚠️ Corre sem adaptador, ao
/// contrário do gate de produto irmão (`trocar_a_cor_com_a_agua…`, `#[ignore]`),
/// logo é ESTE que a suíte e o CI vêem.
#[test]
fn um_ajuste_do_painel_nao_fecha_a_pincelada_e_um_gesto_fecha() {
    use crate::painter_na_malha::o_painel_mexe_na_peca;
    use ph2d_panel_sculpt3d::{Sculpt3dIntent, Sculpt3dUi};
    assert!(
        !o_painel_mexe_na_peca(&Sculpt3dIntent::SetUi(Sculpt3dUi::default())),
        "trocar a cor do pincel fecha a pincelada (o report do dono)"
    );
    for gesto in [
        Sculpt3dIntent::MaskClear,
        Sculpt3dIntent::Remesh,
        Sculpt3dIntent::Subdivide,
        Sculpt3dIntent::ToggleDyntopo,
    ] {
        assert!(
            o_painel_mexe_na_peca(&gesto),
            "o CONTROLO: um gesto do painel deixou de fechar ({gesto:?})"
        );
    }
}
