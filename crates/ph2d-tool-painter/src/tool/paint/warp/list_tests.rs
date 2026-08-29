//! **A LISTA é o estado; o mapa denso é o cache** — os gates da segunda metade do [ADR-0157].
//!
//! O [`super::compose_tests`] julga a LEI (como a lista é dobrada). Este julga o **ESTADO**: que a lista
//! existe, que ela registra o que o artista fez, que ela **basta** para reconstruir a deformação, que ela
//! viaja no undo em lock-step com o mapa, e que os dois escritores que ela **não** explica dizem isso em
//! voz alta em vez de deixar a promessa virar mentira silenciosa.
//!
//! [ADR-0157]: ../../../../../../docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::field::{DabField, DeformMode, compose_at};
use ph2d_editor_core::tool::RasterEditTool as _;
use std::sync::Arc;

const SIDE: u32 = 128;
const C: [f32; 2] = [64.0, 64.0];
const R: f32 = 60.0;

/// Uma tela com listras — uma figura que uma deformação MEXE de forma medível (uma tela chapada seria
/// invariante sob qualquer warp, e todo gate aqui ficaria verde por vácuo).
fn striped_tool() -> crate::tool::PainterTool {
    let mut px = vec![255u8; (SIDE * SIDE) as usize * 4];
    for y in 0..SIDE {
        for x in 0..SIDE {
            if (x / 6) % 2 == 0 {
                let b = ((y * SIDE + x) * 4) as usize;
                px[b] = 0;
                px[b + 1] = 0;
                px[b + 2] = 0;
            }
        }
    }
    let mut t = crate::tool::PainterTool::default();
    t.set_source(px, SIDE, SIDE);
    assert!(t.ensure_warp_session(), "a sessão de warp tem de abrir");
    t
}

fn twist_dabs(n: u16) -> Vec<DabField> {
    (0..n)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                C,
                R,
                [0.0, 0.0],
                [0.0, 0.0],
                1.0,
                0.8,
                0.0,
                0.0,
                u64::from(k) + 1,
            )
        })
        .collect()
}

/// Quantos texels diferem entre duas telas, o pior delta de canal, e a MÉDIA por texel.
///
/// ⚠️ A média é o oráculo, e a contagem é só diagnóstico: numa fixture de alto contraste um deslocamento
/// **sub-pixel** troca a cor inteira de um texel de borda, então CONTAR texels pune a deriva de 0,13 px
/// como se ela fosse uma deformação diferente. A média mede *quão diferente*, que é a pergunta.
fn diff(a: &[u8], b: &[u8]) -> (usize, u8, f64) {
    let mut n = 0usize;
    let mut worst = 0u8;
    let mut sum = 0u64;
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0.iter()) {
        let d = (0..4)
            .map(|i| pa[i].abs_diff(pb[i]))
            .max()
            .unwrap_or_default();
        if d > 0 {
            n += 1;
        }
        sum += u64::from(d);
        worst = worst.max(d);
    }
    #[allow(clippy::cast_precision_loss)]
    let mean = sum as f64 / (a.len() / 4) as f64;
    (n, worst, mean)
}

/// **A lista registra o que o artista fez** — o mínimo, e o que torna tudo abaixo possível.
#[test]
fn the_authored_list_records_every_dab_in_order() {
    let mut t = striped_tool();
    let dabs = twist_dabs(7);
    for f in &dabs {
        t.warp_apply_dab(f, C, R);
    }
    assert_eq!(
        t.paint.warp.dabs.len(),
        7,
        "um dab carimbado é um dab na lista"
    );
    assert!(t.paint.warp.derived, "só houve dabs de Reshape");
    // A ORDEM importa (a composição não é comutativa), e o `seed` por dab é o que a torna verificável.
    for (i, f) in t.paint.warp.dabs.iter().enumerate() {
        assert_eq!(
            f.at([C[0] + 20.0, C[1]]),
            dabs[i].at([C[0] + 20.0, C[1]]),
            "o dab {i} da lista não é o {i}-ésimo carimbado"
        );
    }
}

/// **O GATE CENTRAL DO ADR: jogue o cache fora e a lista o reconstrói.**
///
/// Joga o mapa denso inteiro no lixo, re-cozinha da lista com a lei EXATA e renderiza pela porta do
/// produto. A imagem volta.
///
/// ⚠️ **A comparação NÃO é bit-a-bit, e o motivo é o achado do gate irmão:** o mapa do produto foi avançado
/// incrementalmente e carrega a deriva de reamostragem; o re-cook é exato. Eles descrevem a mesma
/// deformação por dois caminhos, e o que os separa é justamente o número que
/// `the_incremental_cache_drifts_from_the_exact_walk_and_this_is_the_number` mede. Aqui a barra é de
/// APARÊNCIA (quantos texels e quão longe), e o **anti-vácuo** é a outra metade: as duas telas deformadas
/// têm de estar muito mais perto uma da outra do que da tela NÃO deformada — senão um re-cook que não
/// fizesse nada passaria.
#[test]
fn the_lattice_is_a_cache_you_can_throw_away() {
    let mut t = striped_tool();
    let pristine = t.paint.warp.pre.as_ref().clone();
    let dabs = twist_dabs(20);
    for f in &dabs {
        t.warp_apply_dab(f, C, R);
    }
    let by_cache = t.canvas_rgba.as_ref().clone();

    // Joga o cache fora.
    let n = (SIDE * SIDE) as usize;
    t.paint.warp.disp = Arc::new(vec![[0.0, 0.0]; n]);
    // Re-cozinha da LISTA — a lei exata, um nó de cada vez, sem olhar o mapa antigo (ele não existe mais).
    {
        let list = Arc::clone(&t.paint.warp.dabs);
        let disp = Arc::make_mut(&mut t.paint.warp.disp);
        for y in 0..SIDE {
            for x in 0..SIDE {
                disp[(y * SIDE + x) as usize] =
                    compose_at(&list, [f32::from(x as u16), f32::from(y as u16)]);
            }
        }
    }
    t.warp_render_from_session(super::Region {
        x: 0,
        y: 0,
        w: SIDE,
        h: SIDE,
    });
    let by_list = t.canvas_rgba.as_ref().clone();

    let (n_deform, w_deform, m_deform) = diff(&pristine, &by_cache);
    let (n_between, w_between, m_between) = diff(&by_cache, &by_list);
    println!(
        "re-cook da lista: media {m_between:.2} ({n_between} texels, pior {w_between}) | a deformação em \
         si: media {m_deform:.2} ({n_deform} texels, pior {w_deform})"
    );
    assert!(
        n_deform > 2000,
        "a fixture mal deforma ({n_deform} texels) — o gate ficaria verde por vácuo"
    );
    // ANTI-VÁCUO: a lista reconstrói a MESMA deformação, não uma tela em branco nem a pristina.
    assert!(
        m_between * 4.0 < m_deform,
        "o re-cook da lista (media {m_between:.2}) não reproduziu a deformação (media {m_deform:.2})"
    );
}

/// **Os dois escritores que a lista NÃO explica dizem isso.**
///
/// ⚠️ Sem esta metade, *"jogue o cache fora e re-cozinhe"* seria uma promessa que quebra em silêncio no
/// primeiro Reconstruct — e o modo de falha seria a deformação do artista **desaparecer**.
#[test]
fn reconstruct_says_the_list_no_longer_explains_the_map() {
    let mut t = striped_tool();
    for f in &twist_dabs(5) {
        t.warp_apply_dab(f, C, R);
    }
    assert!(t.paint.warp.derived, "cinco dabs, nada mais");
    t.warp_reconstruct_dab(C, R, 0.8);
    assert!(
        !t.paint.warp.derived,
        "o Reconstruct edita o MAPA; a lista deixou de bastar e a sessão tem de dizê-lo"
    );
}

/// **A lista viaja no undo em LOCK-STEP com o mapa.**
///
/// ⚠️ Este é o gate que o `mats` desta linha custou: um plano que fica de fora do snapshot **se esconde**
/// até alguém contradizê-lo. Aqui a contradição é barata — desfazer para um instante com menos dabs e
/// perguntar se as duas metades concordam.
#[test]
fn undo_carries_the_list_in_lock_step_with_the_map() {
    let mut t = striped_tool();
    for f in &twist_dabs(3) {
        t.warp_apply_dab(f, C, R);
    }
    let snap = t.warp_for_snapshot();
    assert_eq!(snap.dabs.len(), 3, "o snapshot leva a lista");
    for f in &twist_dabs(9) {
        t.warp_apply_dab(f, C, R);
    }
    assert_eq!(t.paint.warp.dabs.len(), 12);
    let live_map = t.paint.warp.disp.as_ref().clone();

    t.restore_warp(snap);
    assert_eq!(
        t.paint.warp.dabs.len(),
        3,
        "o restore devolveu o mapa de 3 dabs e tem de devolver a LISTA de 3"
    );
    assert!(
        t.paint.warp.disp.as_ref() != &live_map,
        "o mapa restaurado tem de ser o de antes dos 9 dabs (senão o gate não distingue nada)"
    );
}
