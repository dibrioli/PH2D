//! **W4 — o artista** (`docs/Luz2D/01_plano.md` §4): a lâmpada de fábrica (a semântica, não um
//! palpite), o menu Add, o dreno das edições do Inspector e os AVISOS que chegam à UI.

use bevy_ecs::world::World;
use ph2d_app_light2d::inspector::{apply, build_info, tag_do_tipo, tipo_da_tag};
use ph2d_app_light2d::object_add::{ENTRIES, LUZ, LUZ_AMBIENTE, add, luz_de};
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, grade_do_quadro};
use ph2d_ecs::{GlobalTransform, Name, PresentWorld, SimWorld, Transform};
use ph2d_editor_core::luz_edits::{AvisoDaLuz, LuzFieldEdit as E, TIPOS};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{Luz, TipoDeLuz};
use ph2d_render::Camera2d;

use crate::comum::{H, W, afim, gpu, renderer};

fn com_luz(luz: Luz) -> World {
    let mut w = World::new();
    let t = Transform::IDENTITY;
    w.spawn((luz, t, GlobalTransform::from_transform(t)));
    w
}

/// ⭐⭐ **A lâmpada de fábrica deixa um sprite branco BRANCO a 1 m** (plano §2.3: `luz_em = I/2πr`
/// no ponto; o disco de raio `a` dá `asin(a)/a` — G1). Pelo GABARITO, a partir do `World`.
#[test]
fn a_lampada_de_fabrica_deixa_branco_a_um_metro() {
    let cena = cena_de_luz(&com_luz(Luz::default()));
    let a = f64::from(Luz::TAMANHO_DE_FABRICA_M);
    let esperado = a.asin() / a;
    for p in [[1.0, 0.0], [0.0, -1.0], [0.6, 0.8]] {
        let e = ph2d_light2d_ref::luz_em(&cena, p);
        for c in e {
            assert!((c - esperado).abs() < 1e-3, "em {p:?}: {c} ≠ {esperado}");
        }
    }
    assert!((esperado - 1.0).abs() < 0.011, "branco a 1 m (±1,1 %)");
}

/// ⭐⭐ **A ambiente de fábrica é o FORNO BRANCO** — a cena exatamente como sem luz.
#[test]
fn a_ambiente_de_fabrica_e_o_forno_branco() {
    let cena = cena_de_luz(&com_luz(Luz::ambiente()));
    for p in [[0.0, 0.0], [12.0, -7.0]] {
        for c in ph2d_light2d_ref::luz_em(&cena, p) {
            assert!((c - 1.0).abs() < 1e-6, "em {p:?}: {c}");
        }
    }
}

/// A cena com a lâmpada de fábrica e, se `fortes`, as [`LAMPADAS_DIRETAS_MAX`] lâmpadas MAIS fortes
/// longe dela (que ocupam o caminho direto). Devolve o índice da de fábrica em `emissores`.
fn de_fabrica_com(fortes: bool) -> (ph2d_light2d_ecs::CenaDeLuz, usize) {
    let mut w = World::new();
    let t = Transform::IDENTITY;
    let e = w
        .spawn((Luz::default(), t, GlobalTransform::from_transform(t)))
        .id();
    if fortes {
        for k in 0..ph2d_light2d_gpu::LAMPADAS_DIRETAS_MAX {
            let mut t = Transform::IDENTITY;
            t.translation.x = 50.0 + k as f32;
            w.spawn((
                Luz {
                    intensidade: 1000.0,
                    ..Luz::default()
                },
                t,
                GlobalTransform::from_transform(t),
            ));
        }
    }
    let cena = cena_de_luz(&w);
    let i = cena
        .luzes_emissoras
        .iter()
        .find(|(q, _)| *q == e)
        .map(|&(_, i)| i)
        .expect("a de fábrica emite");
    (cena, i)
}

/// O primeiro zoom (÷2^k da vista de fábrica, 1920×1080) em que a lâmpada `i` sai da grade.
fn sai_da_grade(cena: &ph2d_light2d_ecs::CenaDeLuz, i: usize) -> Option<i32> {
    let janela = WindowSize::new(1920, 1080);
    (0..10).find(|&k| {
        let cam = Camera2d::new([0.0, 0.0], Camera2d::default().height_world * 2f32.powi(k));
        let g = grade_do_quadro(
            cam.world_to_screen_affine(janela),
            (1920, 1080),
            DEGRAU_PADRAO,
        )
        .expect("grade");
        ph2d_light2d_gpu::o_que_fica_fora(cena, &g)
            .fora
            .iter()
            .any(|f| f.quem == ph2d_light2d_gpu::Quem::Emissor(i))
    })
}

/// ⭐ **A lâmpada NÃO some ao afastar o zoom** (smoke do dono, 09–10/10: *«o tamanho afeta a
/// qualidade»*): pela luz direta (C3) ela não depende da grade — em 10 oitavas nunca sai. Controlo:
/// com o caminho direto ocupado por 16 lâmpadas mais fortes, ela fica nas cascatas e sai a ÷8 (o
/// número medido antes da C3).
#[test]
fn a_lampada_de_fabrica_nao_some_ao_afastar_o_zoom() {
    let (cena, i) = de_fabrica_com(false);
    assert_eq!(
        sai_da_grade(&cena, i),
        None,
        "a lâmpada direta saiu da grade"
    );
    let (cena, i) = de_fabrica_com(true);
    assert_eq!(
        sai_da_grade(&cena, i),
        Some(3),
        "controlo: nas cascatas ela sai a ÷8"
    );
}

/// ⭐⭐ **O menu Add cria a luz com o componente** — a de fábrica, a ambiente, nome único, raiz com
/// identidade; uma entrada alheia não é desta família.
#[test]
fn o_menu_add_cria_a_luz_com_o_componente() {
    let mut sim = SimWorld::new();
    let a = add(LUZ, &mut sim).expect("é desta família");
    let b = add(LUZ, &mut sim).expect("é desta família");
    let c = add(LUZ_AMBIENTE, &mut sim).expect("é desta família");
    let w = sim.world();
    let e = |bits| ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(w.get::<Luz>(e(a)), Some(&Luz::default()));
    assert_eq!(
        w.get::<Luz>(e(c)).map(|l| l.tipo),
        Some(TipoDeLuz::Ambiente)
    );
    assert!(w.get::<Transform>(e(a)).is_some());
    assert!(
        w.get::<ph2d_ecs::StableId>(e(a)).is_some(),
        "sem identidade o undo a perde"
    );
    assert_ne!(
        w.get::<Name>(e(a)).map(|n| n.0.clone()),
        w.get::<Name>(e(b)).map(|n| n.0.clone()),
        "dois cliques, dois nomes"
    );
    assert_eq!(add(ph2d_editor_core::object_add::EMPTY, &mut sim), None);
    assert!(
        ENTRIES.iter().all(|&en| luz_de(en).is_some()),
        "entrada sem luz"
    );
}

/// ⭐⭐⭐ **Cada edição do Inspector muda o SEU campo**, e o que não é uma luz é recusado (sem
/// escrever).
///
/// **Mutações que devem sangrar:** trocar dois braços do `aplicar` · aceitar `NaN`/negativo · a cor
/// sem o sRGB → linear.
#[test]
fn cada_edicao_muda_o_seu_campo() {
    let mut sim = SimWorld::new();
    let bits = add(LUZ, &mut sim).unwrap();
    let luz = |sim: &SimWorld| {
        *sim.world()
            .get::<Luz>(ph2d_ecs::Entity::from_bits(bits))
            .unwrap()
    };
    assert!(apply(&mut sim, bits, &E::Intensidade(3.0)));
    assert!(apply(&mut sim, bits, &E::Tamanho(0.5)));
    assert!(apply(&mut sim, bits, &E::Alcance(7.0)));
    assert!(apply(&mut sim, bits, &E::Tipo(1)));
    assert!(apply(&mut sim, bits, &E::Cor([255, 188, 0])));
    let l = luz(&sim);
    assert_eq!(
        (l.intensidade, l.tamanho_m, l.alcance_m, l.tipo),
        (3.0, 0.5, 7.0, TipoDeLuz::Ambiente)
    );
    // sRGB 188 → linear 0,5029 (IEC 61966); 255 → 1; 0 → 0.
    assert!((l.cor[0] - 1.0).abs() < 1e-6 && l.cor[2] == 0.0);
    assert!(
        (l.cor[1] - 0.502_886).abs() < 1e-4,
        "verde linear {}",
        l.cor[1]
    );
    assert!(
        apply(&mut sim, bits, &E::Alcance(f32::INFINITY)),
        "∞ é legal"
    );
    for recusada in [
        E::Intensidade(f32::NAN),
        E::Intensidade(-1.0),
        E::Tamanho(-0.1),
        E::Alcance(0.0),
        E::Tipo(2),
    ] {
        let antes = luz(&sim);
        assert!(!apply(&mut sim, bits, &recusada), "{recusada:?} aceite");
        assert_eq!(luz(&sim), antes, "{recusada:?} escreveu");
    }
    assert!(
        !apply(&mut sim, u64::MAX >> 1, &E::Intensidade(1.0)),
        "entidade que não existe"
    );
}

/// ⭐ A tag do TIPO faz ida e volta e cobre os chips do painel (`luz_edits::TIPOS`).
#[test]
fn a_tag_do_tipo_faz_ida_e_volta() {
    for t in [TipoDeLuz::Ponto, TipoDeLuz::Ambiente] {
        assert_eq!(tipo_da_tag(tag_do_tipo(t)), Some(t));
    }
    assert_eq!(
        usize::from(tag_do_tipo(TipoDeLuz::Ambiente)) + 1,
        TIPOS.len()
    );
}

/// ⭐ O instantâneo: nada sem `Luz` (ADR-0166); a cor volta ao sRGB que o seletor fala; e o
/// «branco a 1 m» da de fábrica é 1.
#[test]
fn o_instantaneo_da_seccao() {
    let mut sim = SimWorld::new();
    let bits = add(LUZ, &mut sim).unwrap();
    let vazio = ph2d_app_components_free_empty(&mut sim);
    let luz = LuzDoQuadro::new();
    assert!(build_info(sim.world(), Some(vazio), 1, &luz).is_none());
    assert!(build_info(sim.world(), None, 0, &luz).is_none());
    let i = build_info(sim.world(), Some(bits), 2, &luz).expect("tem luz");
    assert_eq!(i.cor_srgb, [255; 3]);
    assert!((i.luz_a_1m - 1.0).abs() < 1e-6);
    assert_eq!(i.selected_count, 2);
}

fn ph2d_app_components_free_empty(sim: &mut SimWorld) -> u64 {
    sim.world_mut().spawn(Transform::IDENTITY).id().to_bits()
}

/// ⭐⭐⭐ **Os AVISOS chegam à UI** — o que só o terminal via (W3 «ABERTO» 5): a lâmpada sem tamanho,
/// a menor que o texel, e a RECUSA do solver (com o aviso que sobe UMA vez, na transição).
#[test]
fn os_avisos_chegam_a_ui() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let r = renderer(&gpu);
    let mut w = World::new();
    let t = Transform::IDENTITY;
    let mut lampada = |l: Luz| w.spawn((l, t, GlobalTransform::from_transform(t))).id();
    let ponto = lampada(Luz {
        tamanho_m: 0.0,
        ..Luz::default()
    });
    let mini = lampada(Luz {
        tamanho_m: 0.01,
        ..Luz::default()
    });
    let boa = lampada(Luz::default());
    let mut luz = LuzDoQuadro::new();
    assert!(luz.preparar(&w, afim(), (W, H)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    // ⭐ Pela luz direta (C3) o ponto e a mini acendem: nada a avisar.
    for e in [ponto, mini, boa] {
        assert_eq!(
            luz.avisos_de(e),
            vec![],
            "a lâmpada direta não tem do que avisar"
        );
    }
    // Além do teto (16 mais fortes no caminho direto), as duas voltam às cascatas — e ao aviso.
    for k in 0..ph2d_light2d_gpu::LAMPADAS_DIRETAS_MAX {
        let mut t = Transform::IDENTITY;
        t.translation.x = -1.5 + 0.1 * k as f32;
        t.translation.y = 0.8;
        w.spawn((
            Luz {
                intensidade: 1000.0,
                tamanho_m: 0.3,
                ..Luz::default()
            },
            t,
            GlobalTransform::from_transform(t),
        ));
    }
    assert!(luz.preparar(&w, afim(), (W, H)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert_eq!(luz.avisos_de(ponto), vec![AvisoDaLuz::PontoSemTamanho]);
    assert_eq!(
        luz.avisos_de(mini),
        vec![AvisoDaLuz::PequenaDemaisParaOZoom]
    );
    assert_eq!(luz.recusa_nova(), None);

    // A RECUSA: uma janela que a placa não comporta (o recurso é o `max_texture_dimension_2d`).
    let largo = gpu.device.limits().max_texture_dimension_2d * 8;
    assert!(luz.preparar(&w, afim(), (largo, H)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert!(
        matches!(
            luz.avisos_de(boa).first(),
            Some(AvisoDaLuz::GradeGrandeDemais { .. })
        ),
        "a recusa do solver não chegou ao aviso: {:?}",
        luz.avisos_de(boa)
    );
    assert!(luz.recusa_nova().is_some(), "a recusa nova sobe como aviso");
    assert_eq!(luz.recusa_nova(), None, "e só UMA vez");
    assert!(luz.preparar(&w, afim(), (W, H)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert_eq!(luz.recusa_nova(), None);
    assert!(luz.preparar(&w, afim(), (largo, H)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert!(luz.recusa_nova().is_some(), "uma recusa que volta é NOVA");
}

/// ⭐⭐⭐ **G8, «no MESMO quadro»**: a shell aplica as edições do Inspector DEPOIS da codificação
/// (onde a cena foi coletada); a re-coleta antes do solver leva-as ao quadro em curso. E a última
/// `Luz` apagada deixa o quadro NEUTRO, nunca preto.
///
/// Controlo: sem `recolher`, o solver ainda acende a lâmpada que o artista acabou de apagar.
#[test]
fn a_edicao_depois_da_codificacao_chega_ao_mesmo_quadro() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let r = renderer(&gpu);
    let maximo = |luz: &LuzDoQuadro| {
        crate::comum::fluencia_lida(&gpu, luz.solver().expect("solver"))
            .iter()
            .flatten()
            .fold(0.0_f32, |m, &v| m.max(v))
    };
    for recolhe in [false, true] {
        let mut sim = SimWorld::new();
        let bits = add(LUZ, &mut sim).unwrap();
        let e = ph2d_ecs::Entity::from_bits(bits);
        sim.world_mut()
            .entity_mut(e)
            .insert(GlobalTransform::from_transform(Transform::IDENTITY));
        let mut luz = LuzDoQuadro::new();
        assert!(luz.preparar(sim.world(), afim(), (W, H)), "a codificação");
        assert!(
            apply(&mut sim, bits, &E::Intensidade(0.0)),
            "o Inspector, depois"
        );
        if recolhe {
            luz.recolher(sim.world());
        }
        luz.gravar(&gpu, &mut PresentWorld::new(), &r);
        let m = maximo(&luz);
        eprintln!("recolher = {recolhe}: fluência máxima {m}");
        if recolhe {
            assert!(m < 1e-6, "a lâmpada apagada no Inspector ainda acende: {m}");
        } else {
            assert!(
                m > 0.1,
                "controlo: sem re-coleta a edição chega atrasada ({m})"
            );
        }
    }
    // A última `Luz` apagada depois da codificação: o quadro fica neutro (receptores sem luz).
    let mut sim = SimWorld::new();
    let bits = add(LUZ, &mut sim).unwrap();
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert(GlobalTransform::from_transform(Transform::IDENTITY));
    let mut luz = LuzDoQuadro::new();
    assert!(luz.preparar(sim.world(), afim(), (W, H)));
    sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(bits));
    luz.recolher(sim.world());
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert!(
        !luz.acesa() && luz.recebida().is_none(),
        "sem Luz o quadro é o de sempre"
    );
}
