//! **Arch-gate: o cursor é mapeado pela BANDA da cena, nunca pela janela.**
//!
//! # O defeito MEDIDO (2026-09-17, report do dono: *«o botão +10 não faz nada»*)
//!
//! Com um painel a partir o centro — a timeline, por exemplo — a cena renderiza numa BANDA
//! (`CenterSplit::scene_viewport`) e a projecção dela MUDA. Medido numa janela de `1930×1012` com
//! `t = 0,55`, logo uma banda de `1930×556`:
//!
//! | | caixa de ecrã do botão do HUD |
//! |---|---|
//! | onde ele é DESENHADO (medido na foto) | `x 879..1050 · y 402..462` |
//! | onde o dedo o ENCONTRAVA (varredura pelo `path_at` do produto) | `x 800..1128 · y 728..848` |
//! | depois da cura | `x 872..1056 · y 392..472` |
//!
//! ⇒ o alvo era clicável `~340 px` abaixo de onde aparece — **dentro do painel da timeline**, onde
//! o `on_canvas` é `false` e o ramo nem chega a correr. A aritmética fecha antes do código: `10`
//! unidades de mundo em `556 px` são `55,6 px/unidade`, e o centro do botão (`y = −2,778`) cai em
//! `432` — exactamente onde a foto o mostra.
//!
//! ⚠️ **A lei já estava escrita e a porta não a usava:** o doc do
//! `ph2d_app_motion::field_gizmo::scene_window_wh` diz *«todo mapeamento mundo↔tela do chrome da
//! cena TEM de usar isto»*.
//!
//! # Porque TEXTUAL
//!
//! As duas portas leem `self.gfx` (câmera, superfície, `HeroScreen`), que exige janela e GPU —
//! nenhum teste de unidade as alcança. É a forma dos irmãos `architecture_*` da shell.

/// ⚠️ A PROSA sai antes de a lei ser aplicada — senão este cabeçalho satisfazia o `assert`.
fn sem_comentarios(src: &str) -> String {
    src.lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

/// As duas portas do vector que convertem ECRÃ ↔ MUNDO, e o ficheiro onde cada uma vive.
const PORTAS: [(&str, &str, &str); 2] = [
    (
        "vec_world_at",
        include_str!("../../src/connector_gesture.rs"),
        "o ponto do cursor em mundo",
    ),
    (
        "vec_px_to_world",
        include_str!("../../src/vec_snap.rs"),
        "quanto vale um pixel em mundo (a tolerância de captura)",
    ),
];

#[test]
fn as_portas_de_ecra_para_mundo_passam_pela_banda_da_cena() {
    for (porta, src, o_que) in PORTAS {
        let src = sem_comentarios(src);
        assert!(
            src.contains(&format!("fn {porta}")),
            "controlo positivo: `{porta}` deixou de viver neste ficheiro — este gate passou a medir \
             o nada"
        );
        assert!(
            src.contains("scene_window()"),
            "a porta `{porta}` ({o_que}) não passa pela `App::scene_window`.\n\
             Com o centro partido, a cena desenha numa BANDA e a projecção MUDA: mapear contra a \
             janela põe o que se VÊ e o que se PEGA em espaços diferentes — medido em 17/09, o \
             botão do HUD era clicável 340 px abaixo de onde aparece, dentro do painel da timeline."
        );
    }
}

/// ⛔ **E a própria `scene_window` tem de perguntar ao SPLIT** — se ela devolvesse a janela, as duas
/// portas acima continuariam verdes sobre o defeito, que é a forma mais barata de uma cura morrer.
#[test]
fn a_banda_da_cena_e_derivada_do_split_do_centro() {
    let src = sem_comentarios(include_str!("../../src/connector_gesture.rs"));
    for agulha in ["center_split", "scene_camera_window"] {
        assert!(
            src.contains(agulha),
            "a `scene_window` não lê `{agulha}` — ela deixou de derivar a banda do split, e as \
             portas que a chamam voltaram a mapear contra a janela sem uma linha vermelha"
        );
    }
}
