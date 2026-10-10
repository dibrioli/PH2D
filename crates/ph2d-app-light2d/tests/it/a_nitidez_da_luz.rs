//! ⭐ **A nitidez da luz** (pedido do dono, 2026-10-09: *«escolha no editor, automática em runtime»*):
//! Baixa · Média · Alta = células de 4 · 3 · 2 px; o «Automático» desce quando a placa não aguenta e
//! nunca passa do nível escolhido.

use ph2d_app_light2d::LuzDoQuadro;
use ph2d_app_light2d::qualidade::{Custo, Governador, ORCAMENTO_MS, degrau_do};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{NivelDaLuz as N, PreferenciaDaLuz};

use crate::comum::{H, W, afim, gpu, renderer};

#[test]
fn cada_nivel_tem_a_sua_celula() {
    assert_eq!(
        N::ALL.map(|n| degrau_do(n).c0_px),
        [4.0, 2.0, 1.0],
        "Baixa · Média · Alta"
    );
}

/// O governador desce depois de leituras caras SEGUIDAS, sobe só com folga prevista pela lei
/// `1/célula²`, e nunca passa do teto.
/// Um custo sem a grade fina (tudo segue o nível).
fn so(ms: f64) -> Custo {
    Custo {
        por_nivel: ms,
        fixo: 0.0,
    }
}

#[test]
fn o_governador_desce_sobe_e_respeita_o_teto() {
    let caro = ORCAMENTO_MS * 1.5;
    let mut g = Governador::novo(N::Alta);
    assert_eq!(
        g.ler(so(caro), N::Alta),
        N::Alta,
        "um pico isolado não muda"
    );
    g.ler(so(caro), N::Alta);
    assert_eq!(
        g.ler(so(caro), N::Alta),
        N::Media,
        "três seguidas descem um nível"
    );
    // Na Média, a Alta prevista é ×(2/1)² = 4: só sobe se 4·ms < 0,75·orçamento.
    let barato = ORCAMENTO_MS * 0.75 / 4.0 * 0.9;
    let quase = ORCAMENTO_MS * 0.75 / 4.0 * 1.1;
    for _ in 0..100 {
        assert_eq!(
            g.ler(so(quase), N::Alta),
            N::Media,
            "sem folga prevista não sobe"
        );
    }
    let subiu = (0..100).any(|_| g.ler(so(barato), N::Alta) == N::Alta);
    assert!(subiu, "com folga prevista, sobe");
    for _ in 0..100 {
        assert_eq!(g.ler(so(0.01), N::Alta), N::Alta, "nunca passa do teto");
    }
    assert_eq!(
        g.ler(so(0.01), N::Baixa),
        N::Baixa,
        "baixar o teto obedece na hora"
    );
    // Subir com o teto na MÉDIA: chega lá e não passa.
    let mut g = Governador::novo(N::Baixa);
    let niveis: Vec<N> = (0..200).map(|_| g.ler(so(0.01), N::Media)).collect();
    assert!(niveis.contains(&N::Media), "com folga sobe até o teto");
    assert!(!niveis.contains(&N::Alta), "e nunca acima dele");
}

/// ⭐ A grade FINA da luz direta (W4b) tem o mesmo tamanho em todo nível: a subida prevê só a
/// parte do nível ×4 e soma a fina UMA vez. Controlo: com o custo inteiro ×4 (a lei antiga), metade
/// do orçamento na fina já impedia toda subida.
#[test]
fn o_governador_preve_a_subida_com_a_grade_fina_fixa() {
    let fixo = ORCAMENTO_MS * 0.5;
    let por_nivel = (ORCAMENTO_MS * 0.75 - fixo) / 4.0 * 0.9;
    assert!(
        4.0 * (por_nivel + fixo) > ORCAMENTO_MS * 0.75,
        "controlo: pela lei antiga não subiria"
    );
    let mut g = Governador::novo(N::Media);
    let subiu = (0..100).any(|_| g.ler(Custo { por_nivel, fixo }, N::Alta) == N::Alta);
    assert!(subiu, "a fina não se multiplica pelo nível");
}

/// Na PLACA: com o «Automático», o custo chega ao governador SEM ESPERAR (em poucos quadros), e o
/// manual fixa o nível escolhido.
#[test]
fn o_automatico_le_o_custo_da_placa_sem_esperar() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
    {
        eprintln!("sem TIMESTAMP_QUERY — o automático fica no teto; gate saltado");
        return;
    }
    let r = renderer(&gpu);
    let mut w = bevy_ecs::world::World::new();
    let t = ph2d_ecs::Transform::IDENTITY;
    w.spawn((ph2d_light2d_ecs::Luz::default(), t));
    let mut luz = LuzDoQuadro::new();
    luz.preferir(PreferenciaDaLuz {
        nivel: N::Media,
        automatica: false,
    });
    assert_eq!(luz.qualidade(), Some((N::Media, false)));
    luz.preferir(PreferenciaDaLuz::default());
    assert_eq!(
        luz.qualidade(),
        Some((N::Alta, true)),
        "o automático começa no teto"
    );
    for _ in 0..60 {
        luz.preferir(PreferenciaDaLuz::default());
        assert!(luz.preparar(&w, afim(), (W, H)));
        luz.gravar(&gpu, &mut PresentWorld::new(), &r);
        if luz.leituras_de_custo() >= 3 {
            break;
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    let ms = luz
        .custo_ms()
        .expect("em 60 quadros o automático nunca leu o custo");
    eprintln!("custo lido sem esperar: {ms:.3} ms");
    assert!(
        luz.leituras_de_custo() >= 3,
        "o automático leu {} vez(es) e parou — o pedido não foi libertado",
        luz.leituras_de_custo()
    );
    assert!(ms > 0.0 && ms < 1000.0);
}

/// ⭐ **Os níveis são DISTINTOS em qualquer zoom** (smoke do dono, 09/10: *«mesmo com alta qualidade
/// …»* — com 4 · 3 · 2 px a Média e a Alta caíam na MESMA célula, porque a grade arredonda para a
/// potência de 2 acima). Varre 64 zooms de 3 oitavas: cada nível tem metade da célula do anterior.
#[test]
fn cada_nivel_tem_metade_da_celula_do_anterior_em_todo_zoom() {
    let janela = ph2d_host::WindowSize::new(1920, 1080);
    for k in 0..64 {
        let altura = 2.5 * 2f32.powf(k as f32 / 21.0);
        let afim = ph2d_render::Camera2d::new([0.0, 0.0], altura).world_to_screen_affine(janela);
        let t = N::ALL.map(|n| {
            ph2d_app_light2d::grade_do_quadro(afim, (1920, 1080), degrau_do(n))
                .expect("grade")
                .texel_m
        });
        assert!(
            (t[0] - 2.0 * t[1]).abs() < 1e-12 && (t[1] - 2.0 * t[2]).abs() < 1e-12,
            "a {altura:.2} m de altura as células são {t:?}"
        );
    }
}

/// ⭐⭐ **Com o Automático DESLIGADO, o custo no Inspector é o do nível escolhido** (smoke do dono,
/// 10/10: *«no Inspector o custo é igual para qualquer qualidade»* — o custo só se lia para o
/// governador, e com o Automático desligado ficava congelado no último nível automático).
#[test]
fn com_o_automatico_desligado_o_custo_e_o_do_nivel_escolhido() {
    let Some(gpu) = gpu() else { return };
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
    {
        eprintln!("sem TIMESTAMP_QUERY — gate saltado");
        return;
    }
    let fhd = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let r = renderer(&gpu);
    let mut w = bevy_ecs::world::World::new();
    w.spawn((
        ph2d_light2d_ecs::Luz::default(),
        ph2d_ecs::Transform::IDENTITY,
    ));
    let mut luz = LuzDoQuadro::new();
    let mut custo = |nivel: N| {
        let p = PreferenciaDaLuz {
            nivel,
            automatica: false,
        };
        let mut ms = None;
        for _ in 0..60 {
            luz.preferir(p);
            assert!(luz.preparar(&w, fhd, (1920, 1080)));
            luz.gravar(&gpu, &mut PresentWorld::new(), &r);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            ms = luz.custo_ms();
            if ms.is_some() && luz.qualidade() == Some((nivel, false)) {
                // Três quadros a mais: a leitura sem espera é de há poucos quadros.
                for _ in 0..3 {
                    luz.preferir(p);
                    assert!(luz.preparar(&w, fhd, (1920, 1080)));
                    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
                    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                }
                ms = luz.custo_ms();
                break;
            }
        }
        ms.unwrap_or_else(|| panic!("{nivel:?} sem o Automático: o custo nunca foi lido"))
    };
    let baixa = custo(N::Baixa);
    drop(custo);
    // O 1.º quadro do nível novo recria a grade: o custo lido era o da ANTERIOR e não se mostra.
    let p = PreferenciaDaLuz {
        nivel: N::Alta,
        automatica: false,
    };
    luz.preferir(p);
    assert!(luz.preparar(&w, fhd, (1920, 1080)));
    luz.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert_eq!(
        luz.custo_ms(),
        None,
        "trocado o nível, o Inspector ainda mostra o custo do anterior"
    );
    let mut custo = |nivel: N| {
        let p = PreferenciaDaLuz {
            nivel,
            automatica: false,
        };
        for _ in 0..60 {
            luz.preferir(p);
            assert!(luz.preparar(&w, fhd, (1920, 1080)));
            luz.gravar(&gpu, &mut PresentWorld::new(), &r);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        luz.custo_ms()
            .unwrap_or_else(|| panic!("{nivel:?} sem o Automático: o custo nunca foi lido"))
    };
    let alta = custo(N::Alta);
    eprintln!("custo sem o Automático: Low {baixa:.2} ms · High {alta:.2} ms");
    // A lei 1/célula² dá ×16 entre os dois (medido na W5a a 1080p: 1,57 × 19,5 ms).
    assert!(
        alta > 4.0 * baixa,
        "o custo não seguiu o nível: Low {baixa:.2} · High {alta:.2} ms"
    );
}
