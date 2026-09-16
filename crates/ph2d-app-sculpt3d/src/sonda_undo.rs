//! ⭐⭐⭐ **A SONDA DO UNDO DA ESCULTURA** (`PH2D_SCULPT3D_UNDO_PROBE=1`, com a cena
//! `PH2D_SCULPT3D_SMOKE=47`) — o ROTEIRO e a LEITURA do aparelho do report *«smoke ok. Mas sem
//! undo/redo.»* (Enio, 2026-09-16, sobre o pincel de plano).
//!
//! # Por que ela existe
//!
//! O traço desse pincel **desfaz e refaz** em dois gates da família (`undo_plano_tests`) — um pelas
//! funções de dentro da cena, outro pelas MESMAS portas de ponteiro e de teclado que a shell chama
//! —, e mesmo assim o dono não teve undo. ⇒ o defeito vivia **entre** o evento do `winit` e a porta
//! da família, onde nenhum gate de estado chega: *entre o gesto e o passo há uma máquina, e só a
//! app real a atravessa*.
//!
//! # ⚠️ Por que ela está partida em duas
//!
//! O **executor** vive na shell (`shells/desktop/src/sculpt3d_undo_probe.rs`), porque só ela
//! conduz o teclado e o ponteiro reais. O resto — *que* gesto, *em que* quadro, e *o que* se lê
//! depois — não precisa da `App`, e mora aqui pela catraca `the_shell_only_shrinks`: a sonda
//! inteira na shell custava 187 linhas à unidade de compilação mais lenta da árvore.
//!
//! O roteiro repete o que o smoke da `=47` manda fazer, pela ordem: abrir o painel, escolher
//! `Plane`, esfregar, **arrastar a pista `Height`**, **digitar num campo com Enter**, esfregar de
//! novo — e só depois `Ctrl+Z` duas vezes e `Ctrl+Shift+Z` duas.
//!
//! # Como se lê
//!
//! ```text
//! [probe-sculpt-undo] f=NN undo=<n> redo=<n> verbo=<v> malha=<soma> foco=<b> teclas=<porque> mao=<id> held=<b>
//! ```
//!
//! - `undo` sobe **uma** vez por traço. Um esfregão que não o faz subir é a gravação a falhar.
//! - cada `Ctrl+Z` desce `undo`, sobe `redo` e muda `malha`. Um `Ctrl+Z` que não mexe em nada disto
//!   é a tecla a morrer antes da cena — e `teclas` diz **porquê**.
//!
//! # ⭐ O que ela mediu (2026-09-16)
//!
//! | quadro | `mao` | `teclas` | `Ctrl+Z` |
//! |---|---|---|---|
//! | antes da cura | `vector` do quadro 1 em diante | **mortas** (*«Motion/Vector em mãos»*) | `undo=2` fica `2`, a malha não mexe |
//! | depois da cura | `vector` até ao 1.º esfregão, `move` a seguir | vivas | `2→1→0`, a malha volta ao valor **original ao dígito**; o refazer devolve os dois |
//!
//! ⚠️ **O `vector` do quadro 1 vem do `~/.ph2d/layout.txt`** (`active=vector` da última sessão do
//! dono), e é essa a configuração que reproduz o report — a sonda corre de propósito contra o
//! ficheiro REAL. ⚠️ **Os passos do PAINEL imprimem «roteiro morto» nessa arrumação**, e isso é
//! honesto e não invalida a medição: com `active=vector` o painel da escultura não é pintado numa
//! janela de `1024×768`, e o defeito não depende dele (o traço usa o verbo que estiver na mão).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_editor_core::NodeId;
use winit::keyboard::KeyCode;

use crate::Sculpt3dScene;

/// O último quadro do roteiro que se lê (o 2.º `Ctrl+Shift+Z` sobe no 119).
const FIM: u32 = 124;

/// **Um gesto do roteiro**, já no vocabulário das portas de fumo da shell e com o widget RESOLVIDO.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Passo {
    Nada,
    /// Carregar e largar uma tecla, com o texto que ela escreve.
    Tecla(KeyCode, Option<&'static str>),
    /// `Enter` — o que faz um campo de uma linha gravar.
    Enter,
    /// `Ctrl+Z` (`refazer = false`) ou `Ctrl+Shift+Z`, meia tecla de cada vez: o release é um
    /// evento como qualquer outro, e o undo varre o diff em todo quadro com input.
    CtrlZ {
        refazer: bool,
        desce: bool,
    },
    Desce(f32, f32),
    Move(f32, f32),
    Solta,
}

/// O quadro do roteiro, ou `None` sem a env. **Avança um por chamada** — a shell chama-a uma vez
/// por quadro, e o relógio mora aqui porque o executor não pode acrescentar um campo à `App`.
#[must_use]
pub fn quadro() -> Option<u32> {
    static ON: OnceLock<bool> = OnceLock::new();
    static FRAME: AtomicU32 = AtomicU32::new(0);
    ON.get_or_init(|| std::env::var_os("PH2D_SCULPT3D_UNDO_PROBE").is_some())
        .then(|| FRAME.fetch_add(1, Ordering::Relaxed))
}

/// O gesto do quadro `f` numa janela `w × h`. `widget` é o índice de acerto da shell — o mesmo
/// caminho do dedo —, e um widget que não está lá é **roteiro morto**, dito em voz alta.
///
/// ⚠️ **Os ids vêm do painel por esta família e não pela shell**: o painel é dependência só de TESTE
/// dela (o produto chega-lhe pelo registo), e uma sonda não justifica mudar o que o binário leva.
pub fn passo(f: u32, (w, h): (u32, u32), widget: impl Fn(NodeId) -> Option<(f32, f32)>) -> Passo {
    use ph2d_panel_sculpt3d::ids::{
        SCULPT3D_PLANO_ALTURA, SCULPT3D_PLANO_PROFUNDIDADE_NUM, SCULPT3D_VERB,
    };
    #[allow(clippy::cast_precision_loss)]
    let (cx, cy, dx) = (w as f32 * 0.42, h as f32 * 0.52, f as f32 * 6.0);
    let plano = ph2d_sculpt3d::Verb::ALL
        .iter()
        .position(|v| *v == ph2d_sculpt3d::Verb::Plane);
    let chip = SCULPT3D_VERB[plano.expect("o pincel de plano esta' no `Verb::ALL`")];
    let aperta = |id, nome: &str, desloca: f32, desce: bool| match widget(id) {
        Some((x, y)) if desce => Passo::Desce(x, y),
        Some((x, y)) => Passo::Move(x + desloca, y),
        None => {
            eprintln!(
                "[probe-sculpt-undo] f={f} ⛔ o {nome} NAO esta' no indice de acerto — roteiro morto"
            );
            Passo::Nada
        }
    };
    let passo = match f {
        // ⭐ **A CRASE, e não uma escrita na visibilidade** — é o gesto do dono (passo 1 do roteiro
        // da `=47`), e uma sonda que abrisse o painel por baixo mediria outro programa.
        8 => Passo::Tecla(KeyCode::Backquote, None),
        30 => aperta(chip, "chip Plane", 0.0, true),
        // A — esfregar.
        40 => Passo::Desce(cx, cy),
        41..=52 => Passo::Move(cx + dx - 240.0, cy),
        // B — arrastar a pista Height.
        60 => aperta(SCULPT3D_PLANO_ALTURA, "pista Height", 0.0, true),
        61..=65 => aperta(SCULPT3D_PLANO_ALTURA, "pista Height", -6.0, false),
        // C — digitar 1 no campo Depth e confirmar com Enter.
        70 => aperta(SCULPT3D_PLANO_PROFUNDIDADE_NUM, "campo Depth", 0.0, true),
        72 => Passo::Tecla(KeyCode::Digit1, Some("1")),
        73 => Passo::Enter,
        // D — esfregar outra vez.
        80 => Passo::Desce(cx, cy + 24.0),
        81..=92 => Passo::Move(cx + dx - 480.0, cy + 24.0),
        31 | 53 | 66 | 71 | 93 => Passo::Solta,
        // Dois Ctrl+Z e dois Ctrl+Shift+Z, pelo teclado real.
        100 | 106 => Passo::CtrlZ {
            refazer: false,
            desce: true,
        },
        101 | 107 => Passo::CtrlZ {
            refazer: false,
            desce: false,
        },
        112 | 118 => Passo::CtrlZ {
            refazer: true,
            desce: true,
        },
        113 | 119 => Passo::CtrlZ {
            refazer: true,
            desce: false,
        },
        _ => Passo::Nada,
    };
    if passo != Passo::Nada {
        eprintln!("[probe-sculpt-undo] f={f} {passo:?}");
    }
    passo
}

/// **A leitura** depois do gesto do quadro `f`, até ao fim do roteiro: os passos para desfazer e
/// refazer, o verbo em mãos e a soma das coordenadas da malha (que um desfazer tem de devolver ao
/// dígito). A shell acrescenta o que só ela sabe — o foco, a razão das teclas, a mão e o botão.
#[must_use]
pub fn leitura(f: u32, cena: Option<&Sculpt3dScene>) -> Option<String> {
    (f <= FIM).then(|| {
        let Some(s) = cena else {
            return format!("[probe-sculpt-undo] f={f} SEM CENA");
        };
        let malha: f64 = s
            .mesh()
            .positions()
            .iter()
            .map(|p| f64::from(p[0].abs() + p[1].abs() + p[2].abs()))
            .sum();
        format!(
            "[probe-sculpt-undo] f={f} undo={} redo={} verbo={} malha={malha:.6}",
            s.undo.len(),
            s.redo.len(),
            s.brush.verb.label()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{Passo, passo};

    /// ⭐ **GATE — o roteiro nunca deixa o botão PRESO.** Um `Desce` sem `Solta` antes do próximo
    /// aperto faria o gesto seguinte ser um arrasto do anterior, e a sonda mediria outro programa
    /// sem o dizer. ⚠️ Com todos os widgets presentes — a metade que o roteiro morto não exercita.
    #[test]
    fn o_roteiro_larga_tudo_o_que_aperta() {
        let (mut preso, mut apertos, mut z) = (false, 0, 0i32);
        for f in 0..=super::FIM {
            match passo(f, (1024, 768), |_| Some((10.0, 10.0))) {
                Passo::Desce(..) => {
                    assert!(!preso, "f={f}: um aperto com o botao ainda em baixo");
                    (preso, apertos) = (true, apertos + 1);
                }
                Passo::Move(..) => assert!(preso, "f={f}: um arrasto sem botao"),
                Passo::Solta => preso = false,
                Passo::CtrlZ { desce, .. } => {
                    assert!(!preso, "f={f}: um Ctrl+Z a meio de um gesto");
                    z += if desce { 1 } else { -1 };
                    assert!((0..=1).contains(&z), "f={f}: meia tecla desemparelhada");
                }
                _ => {}
            }
        }
        assert!(
            !preso && z == 0,
            "o roteiro acabou com o botao ou a tecla em baixo"
        );
        assert_eq!(apertos, 5, "chip + dois esfregoes + pista + campo");
    }
}
