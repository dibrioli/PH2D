//! **G10 (RC)** — cena estática: dois quadros seguidos dão a fluência BIT a BIT igual, e um
//! segundo solver com a mesma configuração também.

use ph2d_light2d_gpu::{Config, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro};
use crate::comum::{gpu, grade, ler_bytes};

#[test]
fn g10_dois_quadros_da_mesma_cena_sao_bit_identicos() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let cena = ceu_e_vidro();
    let mut s = Solver::novo(&gpu, Config::padrao(grade(lado, texel))).expect("solver");
    s.quadro(&gpu, &cena);
    let a = ler_bytes(&gpu, s.fluencia());
    s.quadro(&gpu, &cena);
    let b = ler_bytes(&gpu, s.fluencia());
    let mut outro = Solver::novo(&gpu, Config::padrao(grade(lado, texel))).expect("solver");
    outro.quadro(&gpu, &cena);
    let c = ler_bytes(&gpu, outro.fluencia());
    assert!(
        a.iter().any(|&x| x != 0),
        "a fluência não pode ser toda zero"
    );
    assert!(a == b, "dois quadros da mesma cena diferem");
    assert!(a == c, "dois solvers da mesma configuração diferem");
}
