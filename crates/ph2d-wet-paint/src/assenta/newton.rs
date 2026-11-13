//! O Newton do [`super::assenta`] e as suas voltas em execução (N.º 7 rodada 4).
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

/// **N.º 7 rodada 4 (13/11, dono «tente»):** as voltas do Newton do [`resolve`] — `4` = a conta exacta de antes,
/// `0` = só o palpite físico (−1,5 ms da média a 6 traços; pior 20 níveis no ecrã). `255` = ainda por ler:
/// `PH2D_ASSENTA_PALPITE=1` dá `0` (o olho do dono), senão [`VOLTAS_DE_FABRICA`].
/// Medido (`N7_ITEM1_sim6.md` rodada 4): sonda rotas `wN`.
#[doc(hidden)]
pub static NEWTON_VOLTAS: AtomicU8 = AtomicU8::new(255);
/// As voltas por omissão: a conta exacta até o dono aprovar o palpite a olho.
pub const VOLTAS_DE_FABRICA: u8 = 4;
/// As voltas da conta exacta.
pub const VOLTAS_EXATAS: u8 = 4;

#[inline]
pub(super) fn newton_voltas() -> u8 {
    let v = NEWTON_VOLTAS.load(Relaxed);
    if v != 255 {
        return v;
    }
    let palpite = std::env::var("PH2D_ASSENTA_PALPITE").is_ok_and(|v| v == "1");
    let v = if palpite { 0 } else { VOLTAS_DE_FABRICA };
    NEWTON_VOLTAS.store(v, Relaxed);
    v
}

/// **O palpite está ligado?** (n.º 8, o checkbox «Performance» do dono 13/11: opção a mais, desligada de fábrica.)
#[must_use]
pub fn palpite_ligado() -> bool {
    newton_voltas() == 0
}

/// Liga ou desliga o palpite: as voltas e o memo da `assenta` (que guarda contas feitas com as voltas de antes).
pub fn liga_o_palpite(sim: bool) {
    NEWTON_VOLTAS.store(if sim { 0 } else { VOLTAS_DE_FABRICA }, Relaxed);
    super::memo::memo_esquece();
}

/// Newton por canal sobre a cor que muda (`x`), a partir do palpite físico, até o ecrã voltar a `alvo`
/// (a velatura do filme não compõe em passos: o palpite deixa um resto). A derivada é numérica — com o
/// filme grosso a velatura pesa mais que a cobertura (medido: assumir `1 − a` dava passos 4× grandes e o
/// miolo ia a preto); o passo tem tecto.
///
/// ⛔ Os canais NÃO são independentes: o tapar extra pesa a diferença de pigmento das TRÊS cores, e mexer
/// num canal muda o ecrã dos outros. Cada canal lê o ecrã DEPOIS do anterior (Bug #49: com o ecrã do
/// início da volta, a derivada do verde media a mudança do vermelho e o assentado ia a verde-água).
///
/// `cego`: a sensibilidade abaixo da qual o canal fica no palpite ([`CEGO`]; [`CEGO_COM_GLAZE`] sob o Glaze).
pub(super) fn resolve(
    voltas: u8,
    mut x: [f64; 3],
    alvo: [f64; 3],
    cego: f64,
    ecra: impl Fn([f64; 3]) -> [f64; 3],
    canal: impl Fn([f64; 3], usize) -> f64,
) -> [f64; 3] {
    const H: f64 = 0.5;
    if voltas == 0 {
        return x;
    }
    let mut y = ecra(x);
    for _ in 0..voltas {
        if (0..3).all(|c| (y[c] - alvo[c]).abs() < 0.01) {
            break;
        }
        let mut mexeu = false;
        for c in 0..3 {
            let mut xh = x;
            xh[c] = if x[c] + H <= 255.0 {
                x[c] + H
            } else {
                x[c] - H
            };
            // A derivada lê só o canal `c`: `canal(x, c) == ecra(x)[c]` ao bit (31/10, `assenta_bit_tests`).
            let d = (canal(xh, c) - y[c]) / (xh[c] - x[c]);
            // Onde o ecrã quase não vê esta cor (o filme grosso esconde o assentado) o palpite físico
            // fica: corrigi-lo ali escrevia uma cor ESCONDIDA que aparecia quando o filme afinava
            // (medido: o miolo esverdeava +24 ao secar).
            if d.abs() < cego {
                continue;
            }
            let passo = ((y[c] - alvo[c]) / d).clamp(-48.0, 48.0);
            x[c] = (x[c] - passo).clamp(0.0, 255.0);
            y = ecra(x);
            mexeu = true;
        }
        // SEM (31/10, ao bit): uma volta com todo canal cego não mexe `x` nem `y` — a seguinte seria a mesma conta.
        if !mexeu {
            break;
        }
    }
    x
}
