//! **Arch-gate: quem pede fantasmas ao onion entrega o RELÓGIO DO CLIP** (report do dono,
//! 2026-09-14: *«só aparece a silhueta do futuro»*).
//!
//! # O defeito, medido
//!
//! Um fantasma do onion é a pose de [`ph2d_timeline::pose_at`], que fala o tempo do **clip activo**
//! — é o espelho exacto do `apply_active_clip`, e há gate de equivalência a pinar isso. Mas o
//! relógio que a VISTA dirige não é sempre o mesmo objecto: a aba **Keys** (a de omissão) move o
//! `clip_playhead`, um contêiner aberto move o `container_playhead`, e só o Arrange move o
//! `playhead` da cena. O shell escolhe entre os três em **dois** sítios (o dreno da timeline e o
//! passe de AutoKey) — e a chamada do onion passava um **quarto** palpite escrito à mão,
//! `self.playhead.time()`.
//!
//! Consequência, na configuração de fábrica: arrastar o cursor na aba Keys **não move** esse
//! número. Ele fica em `0`, não existe keyframe ANTES dele, e o passado nunca tem o que mostrar —
//! o futuro tem. *Um recurso meio mudo lê-se como um recurso partido.*
//!
//! ⇒ a única resposta aceitável é a que o quadro já publica UMA vez, do relógio activo:
//! `TimelineViewSnapshot::clip_time` (`None` quando o clip não toca aqui, ou toca duas vezes numa
//! pilha — e aí não há vizinho de que o passado e o futuro sejam vizinhos).
//!
//! # Porque um gate TEXTUAL, e porque VIVE AQUI
//!
//! A decisão é pura e testada do lado do onion (`timeline_onion_tests::
//! without_a_clip_instant_the_onion_publishes_nothing`, com o controlo). O que **nenhum teste de
//! unidade alcança** é o fio: a chamada vive numa fase do `render_frame`, que exige janela, GPU e
//! superfície. É a mesma forma — e o mesmo remédio — do irmão
//! `the_motion_path_is_offered_only_on_the_keys_tab`, que vive em `shells/desktop/tests/it/`.
//!
//! ⚠️ **Este mora na `ph2d-editor-core` por uma razão MEDIDA, não por arrumação:** em 2026-09-14 a
//! shell tinha `90` linhas de folga contra o tecto do `the_shell_only_shrinks` (vizinho deste
//! ficheiro, e que varre o mesmo caminho). Um gate de 90 linhas lá dentro estouraria a catraca que
//! existe para a shell só encolher. ⛔ O preço está nomeado: um gate que lê a shell **pelo
//! caminho** escapa à linha que muda o código de sítio — se o ficheiro da fase mudar de nome, a
//! varredura continua a achá-lo (ela é do `src/` inteiro), mas se a shell inteira mudar de sítio é
//! o controlo positivo que reprova.

use std::path::{Path, PathBuf};

/// A ÚNICA resposta aceitável para *«em que instante do clip está esta vista?»* — o espelho que o
/// `fase_timeline_drain` publica do relógio ACTIVO, uma vez por quadro. Um `self.playhead.time()`,
/// um `clip_playhead.time()` lido à mão ou qualquer outro derivado seriam a segunda porta para a
/// mesma pergunta, e é exactamente essa que envelhece.
const A_PORTA: &str = "self.timeline_view.clip_time";

/// Os relógios CRUS. Nenhum deles pode aparecer dentro da chamada: os três existem, e escolher
/// entre eles aqui é re-derivar o que a vista já respondeu.
const RELOGIOS_CRUS: &[&str] = &[
    "self.playhead",
    "self.clip_playhead",
    "self.container_playhead",
];

/// O módulo do onion e os testes dele: ali dentro os chamadores são internos (já sob a decisão) e as
/// fixtures passam literais de propósito — é o que um teste de unidade É.
fn e_o_proprio_onion(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("timeline_onion"))
}

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src")
}

fn ficheiros(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            ficheiros(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") && !e_o_proprio_onion(&p) {
            out.push(p);
        }
    }
}

/// ⚠️ **A PROSA sai antes de a lei ser aplicada** — e esta linha nasceu de o gate ter reprovado
/// sobre o **comentário que explica a cura**, que nomeia o relógio errado para dizer que ele saiu.
/// *Um censo textual que não separa prosa de código mente nos DOIS sentidos*: aqui acusaria o
/// caminho certo, e um `// self.timeline_view.clip_time` num sítio errado absolveria-o.
fn sem_comentarios(args: &str) -> String {
    args.lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A lista de argumentos que começa no `(` em `open`, até o parêntese que a fecha.
fn argumentos(src: &str, open: usize) -> &str {
    let bytes = src.as_bytes();
    let mut nivel = 0i32;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'(' => nivel += 1,
            b')' => {
                nivel -= 1;
                if nivel == 0 {
                    return &src[open + 1..i];
                }
            }
            _ => {}
        }
    }
    &src[open + 1..]
}

#[test]
fn the_onion_speaks_the_clip_clock() {
    let mut files = Vec::new();
    ficheiros(&shell_src(), &mut files);
    assert!(
        files.len() > 20,
        "controlo positivo: a varredura não achou o `src/` da shell ({} ficheiros)",
        files.len()
    );

    const PORTA: &str = "collect_onion_ghosts(";
    let mut vistas = 0_usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut from = 0;
        while let Some(off) = src[from..].find(PORTA) {
            let at = from + off;
            let open = at + PORTA.len() - 1;
            let args = sem_comentarios(argumentos(&src, open));
            let args = args.as_str();
            assert!(
                args.contains(A_PORTA),
                "{} chama `{PORTA}` sem `{A_PORTA}`. O fantasma é a pose do CLIP ACTIVO \
                 (`pose_at`), e o relógio que a vista dirige é o do clip na aba Keys, o do \
                 contêiner dentro de um, e o da cena só no Arrange. Um palpite aqui deixa todo \
                 gate do onion verde com o passado mudo na tela do artista.\nargumentos: {args}",
                path.display()
            );
            for cru in RELOGIOS_CRUS {
                assert!(
                    !args.contains(cru),
                    "{} passa o relógio CRU `{cru}` ao `{PORTA}` — escolher entre os três aqui é a \
                     quarta resposta a uma pergunta que o quadro já publica em `{A_PORTA}`.",
                    path.display()
                );
            }
            vistas += 1;
            from = at + PORTA.len();
        }
    }
    // Controlo positivo: sem isto, apagar a chamada (ou renomear a porta) faria este gate passar
    // afirmando exactamente nada.
    assert_eq!(
        vistas, 1,
        "controlo positivo: esperava EXACTAMENTE uma chamada do shell ao onion; achei {vistas} — \
         se o cozimento dos fantasmas ganhou um segundo sítio, ele entra nesta lei"
    );
}
