//! ⭐⭐⭐⭐ **OS GATES DA CACHE DO CAMPO DO CHÃO** — a cura que vale `1,25×` a `1,48×` no quadro
//! assente.
//!
//! ⚠️ **Eles saíram do [`super`] por um tecto de LOC** (2026-09-23, `723` contra `700`), e a
//! fronteira é a CURA: o irmão mede o que a **fita inerte** e a **lei do dono** compraram no
//! COMPILADOR; estes medem o que a cache do **chão** comprou no quadro.
//!
//! ⛔ As duas famílias têm a mesma FORMA de régua — *a saída é byte-idêntica por construção e a
//! economia é invisível a toda régua de valor* ⇒ cada gate afirma o PIXEL **e** a CONTA — e por
//! isso é fácil confundi-las num ficheiro só.

use super::super::*;

/// ⭐⭐⭐⭐ **O CAMPO DO CHÃO REAPROVEITADO NÃO MUDA UM BYTE — e a cache tem de ACERTAR.**
///
/// A assadura custa `+4,98 ms` por quadro assente e o campo não depende de para onde a câmera olha
/// ([`crate::gpu_frame::ChaveDoChao`]) — orbitar é o gesto que a paga. Este gate afirma as duas
/// metades: a imagem não muda, **e** a cache de facto acertou.
///
/// ⛔⛔⛔ **A LÂMPADA É FIXA, e a 1.ª redacção desta régua era um VÁCUO sem isso.** O
/// [`crate::gpu_frame::tests_lampada`] faz a luz **seguir a câmera** — logo orbitar trocava a LUZ,
/// a chave faltava de qualquer maneira, e as duas colunas comparavam **duas assaduras frescas**.
/// Ela lia `0 de 2 073 600` píxeis diferentes e não afirmava nada sobre a cache. *O furo estava
/// escrito no comentário da minha própria sonda, e eu li-o depois de a correr.*
///
/// ⭐ **E a metade do ACERTO é o que a torna load-bearing:** sem ela, uma cache apagada passa este
/// gate — *duas assaduras frescas dão a mesma imagem por construção.*
#[test]
#[ignore = "precisa de GPU"]
fn o_campo_do_chao_reaproveitado_nao_muda_um_byte() {
    use std::sync::atomic::Ordering;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(1);
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    // ⚠️ **FIXA em MUNDO**, e não a `tests_lampada`: é isso que faz orbitar não mexer na chave.
    const LUZ: [ph2d_field_render::PointLamp; 1] = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let pinta = |azimute: f32, chao_em_cache: bool| {
        let cam = ph2d_field_render::Orbit {
            rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
            ..ph2d_field_render::Orbit::default()
        };
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &LUZ,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda {
                chao_em_cache,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };

    // Enche a cache num azimute, e pinta noutro: o campo tem de ser o MESMO objecto.
    let _ = pinta(0.0, true);
    let antes = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed);
    let com = pinta(1.0, true);
    let acertos = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed) - antes;
    let sem = pinta(1.0, false);

    assert!(
        acertos > 0,
        "orbitar não acertou na cache do chão — sem isto as duas colunas abaixo são duas          assaduras frescas e a igualdade delas não afirma nada"
    );
    let do_fundo = sem
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == BG)
        .count();
    let total = sem.rgba.len() / 4;
    assert!(
        do_fundo * 10 < total * 9 && do_fundo * 10 > total,
        "CONTROLO: {do_fundo} de {total} píxeis são o fundo — uma imagem quase vazia é          trivialmente igual a outra"
    );
    assert_eq!(
        com.rgba, sem.rgba,
        "o campo do chão reaproveitado de outro azimute mudou a imagem — a chave está a excluir          um eixo que CHEGA ao campo"
    );
}

/// ⭐⭐⭐⭐ **E A ECONOMIA MEDE-SE PELA CONTA: orbitar reaproveita, trocar a LUZ reassa.**
///
/// A cache dá o MESMO campo nas duas rotas — que é o que a torna segura e o que a torna impossível
/// de medir por valor: *uma cache apagada também dá o mesmo campo*. ⇒ a régua é o
/// [`crate::gpu_frame::ACERTOS_DO_CHAO`].
///
/// ⛔ **O CONTROLO é a segunda metade**, e sem ele a cura lê-se como *«reaproveita sempre»*: trocar
/// a luz tem de FALTAR, senão orbitar com a luz noutro sítio entregaria a sombra de cor antiga.
///
/// ⚠️ **Ele lê um contador GLOBAL** — sob `cargo test` (threads num processo) um irmão a pintar no
/// meio disto entra na conta. Sob o `nextest`, que dá um processo por teste, é sólido; é a mesma
/// nota do `a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte`.
#[test]
#[ignore = "precisa de GPU"]
fn a_cache_do_chao_falta_quando_a_luz_muda() {
    use std::sync::atomic::Ordering;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(2);
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pinta_com = |azimute: f32,
                     luz: &[ph2d_field_render::PointLamp],
                     half_extent: f32,
                     donos: Option<&ph2d_field_eval::owners::Owners>| {
        let cam = ph2d_field_render::Orbit {
            rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
            half_extent,
            ..ph2d_field_render::Orbit::default()
        };
        let surfaces = ph2d_field_render::Surfaces {
            all: surfaces.all,
            owners: donos,
        };
        let _ = crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda::default(),
        )
        .expect("o pintor");
    };
    let luz_a = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let luz_b = [ph2d_field_render::PointLamp {
        world: [-1.6, 2.4, -1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let padrao = ph2d_field_render::Orbit::default().half_extent;
    let pinta = |azimute: f32, luz: &[ph2d_field_render::PointLamp]| {
        pinta_com(azimute, luz, padrao, None);
    };
    let conta = |f: &dyn Fn()| {
        let antes = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed);
        f();
        crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed) - antes
    };

    pinta(0.0, &luz_a);
    let ao_orbitar = conta(&|| pinta(1.0, &luz_a));
    let ao_trocar_a_luz = conta(&|| pinta(1.0, &luz_b));

    assert!(
        ao_orbitar > 0,
        "orbitar reassou o campo do chão — é o gesto que paga os +4,98 ms e a cache existe para ele"
    );
    assert_eq!(
        ao_trocar_a_luz, 0,
        "CONTROLO: trocar a LUZ acertou na cache — ela entregaria a sombra de cor da luz antiga, e sem esta metade a cura lê-se como «reaproveita sempre»"
    );

    // ⭐⭐⭐ **O ZOOM ABAIXO DO CLAMP tem de FALTAR.** A tolerância de acerto é
    // `min(HIT_EPS, half_extent/(2·lado_px))` e só desce com `lado_px > 2500 × half_extent` — a
    // `1080` píxeis isso é `half_extent < 0,432`. *Acima do clamp o zoom não é chave e abaixo é*, e
    // foi um CONTROLO que derrubou a redacção que dava a câmera inteira por fora (ver os gates de
    // `chao_ricochete.rs`).
    assert!(
        ph2d_field_render::Sharpness::for_frame(0.2, LH as usize).hit
            < ph2d_field_render::Sharpness::for_frame(padrao, LH as usize).hit,
        "CONTROLO DO CONTROLO: a `0,2` de enquadramento o clamp não solta a {LH} píxeis — então a metade abaixo mede o nada"
    );
    pinta(0.0, &luz_a);
    let ao_aproximar_muito = conta(&|| pinta_com(0.0, &luz_a, 0.2, None));
    assert_eq!(
        ao_aproximar_muito, 0,
        "CONTROLO: um zoom ABAIXO do clamp da tolerância acertou na cache — a tolerância move o campo até 0,91 de um byte ali, e a chave tem de a levar"
    );

    // ── E a CERCA: uma peça com LEI DO DONO não é cacheada ────────────────────────────────────
    //
    // ⚠️ A chave só conhece a fita COMBINADA, e a lei do dono é função das FOLHAS. A cerca é
    // conservadora de propósito, e sem esta metade ninguém a acorda quando alguém a apagar.
    let postas: Vec<ph2d_field::FieldDoc> = doc
        .nodes()
        .iter()
        .take(1)
        .map(|n| {
            ph2d_field::FieldDoc::new(vec![n.clone()], ph2d_field::NodeId(0)).expect("a folha")
        })
        .collect();
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(padrao, f32::from(u16::try_from(LH).unwrap_or(u16::MAX))),
    );
    pinta_com(0.0, &luz_a, padrao, Some(&donos));
    let com_dono = conta(&|| pinta_com(1.0, &luz_a, padrao, Some(&donos)));
    assert_eq!(
        com_dono, 0,
        "CONTROLO: uma peça com LEI DO DONO acertou na cache — a chave não a distingue, e a cerca do `campo_do_chao` existe para isso"
    );
}
