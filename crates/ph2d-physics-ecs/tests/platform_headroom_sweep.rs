//! **O SENSOR DO AGACHAR VARRE O CORPO** (`W-ShapeCast`) — os gates de produto.
//!
//! ⚠️ **O primeiro nasceu VERMELHO**, e o número dele é o do defeito medido: com
//! os três raios o personagem levantava-se **para dentro** de um pilar de 8 cm
//! posto no vão entre duas amostras — cabeça a **1,267** contra uma face de
//! pedra em **1,25**, com o corpo ainda debaixo dela.
//!
//! ⚠️ **E o CONTROLE não é cerimónia:** sem ele, *"a pedra recusa o levantar"* é
//! satisfeito por um personagem que nunca se levanta — que é exactamente o que
//! um sensor cravado em *bloqueado* produziria.

#[path = "platform_pillar_rig.rs"]
mod pillar_fixture;

use pillar_fixture::{CROUCH_HEIGHT, FLOAT_HEIGHT, RADIUS, rig};

/// Onde a cabeça fica AGACHADO e DE PÉ — os dois pólos que todo gate deste
/// arquivo compara.
const CROUCHED_TOP: f32 = CROUCH_HEIGHT + pillar_fixture::BODY_HALF;
const STANDING_TOP: f32 = FLOAT_HEIGHT + pillar_fixture::BODY_HALF;

/// **UMA PEDRA ENTRE OS RAIOS ANTIGOS RECUSA O LEVANTAR** — o gate que nasceu
/// vermelho.
///
/// O pilar mede 8 cm e está em `+0,10`: nenhum dos três raios que o sensor
/// lançava (`−0,20 · 0,00 · +0,20`) o tocava, e o corpo — cuja cápsula em
/// `x = 0,10` alcança `0,3 + sqrt(0,2² − 0,1²)` — toca-o com folga.
///
/// ⚠️ **A MUTAÇÃO que o mata:** devolver o `probe_headroom` aos três raios. Com
/// ela a cabeça chega a **1,267** contra pedra em **1,25**, e este gate sangra.
#[test]
fn a_stone_between_the_old_rays_still_refuses_the_stand() {
    const BOTTOM: f32 = 1.25;
    let (top, x) = rig(Some((0.10, 0.04, BOTTOM))).crouch_then_release(60, 120);
    assert!(
        top < BOTTOM,
        "a cabeca nao pode subir ATRAVES da pedra: topo {top:.3} contra face em {BOTTOM:.2}"
    );
    assert!(
        (top - CROUCHED_TOP).abs() < 0.05,
        "ele fica agachado, na altura de agachado: {top:.3} (esperado ~{CROUCHED_TOP:.2})"
    );
    // ⚠️ E não é por ter fugido: o solver expulsa um corpo de sob uma pedra
    // estreita demais, e *ficar baixo* e *escapar de lado* são vereditos
    // diferentes sobre o mesmo número de altura.
    assert!(
        x.abs() < 0.05,
        "ele fica ONDE estava, nao escorrega de lado: x = {x:.3}"
    );
}

/// **O CONTROLE: com o céu limpo ele levanta-se.**
///
/// Sem este gate, o de cima é satisfeito por um sensor que responde *bloqueado*
/// a tudo — e a wave inteira teria sido trocar um defeito visível por um mudo.
#[test]
fn with_a_clear_sky_he_stands_back_up() {
    let (top, _) = rig(None).crouch_then_release(60, 120);
    assert!(
        (top - STANDING_TOP).abs() < 0.05,
        "sem teto nenhum ele levanta-se inteiro: {top:.3} (esperado ~{STANDING_TOP:.2})"
    );
}

/// **E uma laje LARGA continua a recusar** — a regressão do comportamento que já
/// shipava, e que nenhuma parte desta wave podia mexer.
#[test]
fn a_wide_slab_still_refuses_the_stand() {
    const BOTTOM: f32 = 1.25;
    let (top, _) = rig(Some((0.0, 6.0, BOTTOM))).crouch_then_release(60, 120);
    assert!(
        top < BOTTOM,
        "sob uma laje larga ele fica agachado, como sempre: {top:.3}"
    );
}

/// **A QUINA DA CAIXA DEIXOU DE SER UM TETO** — a segunda limitação que a
/// varredura apaga, e a única mudança de comportamento *permissiva* desta wave.
///
/// A pedra começa em `x = +0,20`, exactamente onde o raio da borda nascia. A
/// cápsula ali alcança só `centro + 0,30`, então de pé (centro 1,10) ela chega a
/// **1,40** — e um teto entre 1,40 e 1,60 era espaço vazio que o raio da borda
/// via como pedra.
///
/// ⚠️ **Isto NÃO é uma tolerância afrouxada:** a pergunta passou a ser feita
/// sobre o corpo em vez de sobre a caixa dele. O par de asserções abaixo é o que
/// separa as duas coisas — a pedra que a cápsula de facto toca continua a
/// recusar.
#[test]
fn the_capsule_corner_is_no_longer_a_ceiling() {
    // A pedra ocupa tudo à direita de x = +0,20.
    let stone = |bottom: f32| Some((10.0 + RADIUS, 10.0, bottom));

    let (high, _) = rig(stone(1.45)).crouch_then_release(60, 120);
    assert!(
        (high - STANDING_TOP).abs() < 0.05,
        "a 1,45 a capsula passa por baixo e ele levanta-se: {high:.3}"
    );

    let (low, _) = rig(stone(1.35)).crouch_then_release(60, 120);
    assert!(
        low < 1.35,
        "a 1,35 a capsula ENCOSTA (ela alcanca 1,40 ali) e o levantar e' recusado: {low:.3}"
    );
}
