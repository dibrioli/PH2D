//! As variantes do C&F do caminho do dono (W11), em Digital: o roteiro e as cores.

use super::{Gesto, Variante, pincelada};

/// O vermelho que a `cor` escolhe; o azul que o `colordrop` derrama.
pub const COR_NOVA: [u8; 3] = [200, 30, 30];
pub const COR_DO_DROP: [u8; 3] = [40, 90, 210];

/// O C&F premido, arrastado por `pontos` (fracções da imagem) e solto, a partir de `t0` (ms).
fn cef(r: &mut Vec<(u64, Gesto)>, t0: u64, pontos: &[[f32; 2]]) -> u64 {
    r.push((t0, Gesto::CeFDesce));
    let mut t = t0;
    for &p in pontos {
        t += 50;
        r.push((t, Gesto::CeFMove(p)));
    }
    r.push((t + 100, Gesto::CeFSobe));
    t + 100
}

/// As variantes do C&F (W11), em Digital.
pub(super) fn roteiro_do_cef(v: Variante) -> Vec<(u64, Gesto)> {
    let mut r = vec![(1_000, Gesto::Grava)];
    pincelada(&mut r, 2_000, 0.30, 600);
    // Um pouco de arrasto que fica à ESQUERDA da tela: só o seletor abre.
    let fora = [[-0.30, 0.40], [-0.25, 0.42], [-0.20, 0.45]];
    let cor = if v == Variante::Cor {
        COR_NOVA
    } else {
        COR_DO_DROP
    };
    let t = cef(&mut r, 3_000, &fora);
    r.extend([
        (t + 250, Gesto::EscolheCor(cor)),
        (t + 550, Gesto::FechaSeletor),
    ]);
    let t = if v == Variante::ColorDrop {
        // De fora para dentro da tela, e solto lá: o preenchimento; depois o Done do modal, e o
        // vermelho no C&F para a pincelada se ver sobre o azul.
        let t = cef(
            &mut r,
            t + 900,
            &[[-0.20, 0.75], [0.50, 0.75], [0.55, 0.78]],
        );
        r.push((t + 450, Gesto::FillPronto));
        let t = cef(&mut r, t + 750, &[]);
        r.extend([
            (t + 250, Gesto::EscolheCor(COR_NOVA)),
            (t + 550, Gesto::FechaSeletor),
        ]);
        t + 900
    } else {
        t + 900
    };
    pincelada(&mut r, t, 0.50, 600);
    let para = t + 1_600;
    r.extend([
        (para, Gesto::Para),
        (para + 500, Gesto::Rewind),
        (para + 1_000, Gesto::Play),
    ]);
    r
}
