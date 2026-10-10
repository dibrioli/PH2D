//! ⭐ **A nitidez da luz** (pedido do dono, 2026-10-09: *«escolha no editor, automática em runtime»*):
//! Baixa · Média · Alta = células de 4 · 3 · 2 px; o «Automático» desce quando a placa não aguenta e
//! nunca passa do nível escolhido.

use ph2d_app_light2d::LuzDoQuadro;
use ph2d_app_light2d::qualidade::{Governador, ORCAMENTO_MS, degrau_do};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{NivelDaLuz as N, PreferenciaDaLuz};

use crate::comum::{H, W, afim, gpu, renderer};

#[test]
fn cada_nivel_tem_a_sua_celula() {
    assert_eq!(
        N::ALL.map(|n| degrau_do(n).c0_px),
        [4.0, 3.0, 2.0],
        "Baixa · Média · Alta"
    );
}

/// O governador desce depois de leituras caras SEGUIDAS, sobe só com folga prevista pela lei
/// `1/célula²`, e nunca passa do teto.
#[test]
fn o_governador_desce_sobe_e_respeita_o_teto() {
    let caro = ORCAMENTO_MS * 1.5;
    let mut g = Governador::novo(N::Alta);
    assert_eq!(g.ler(caro, N::Alta), N::Alta, "um pico isolado não muda");
    g.ler(caro, N::Alta);
    assert_eq!(
        g.ler(caro, N::Alta),
        N::Media,
        "três seguidas descem um nível"
    );
    // Na Média, a Alta prevista é ×(3/2)² = 2,25: só sobe se 2,25·ms < 0,75·orçamento.
    let barato = ORCAMENTO_MS * 0.75 / 2.25 * 0.9;
    let quase = ORCAMENTO_MS * 0.75 / 2.25 * 1.1;
    for _ in 0..100 {
        assert_eq!(
            g.ler(quase, N::Alta),
            N::Media,
            "sem folga prevista não sobe"
        );
    }
    let subiu = (0..100).any(|_| g.ler(barato, N::Alta) == N::Alta);
    assert!(subiu, "com folga prevista, sobe");
    for _ in 0..100 {
        assert_eq!(g.ler(0.01, N::Alta), N::Alta, "nunca passa do teto");
    }
    assert_eq!(
        g.ler(0.01, N::Baixa),
        N::Baixa,
        "baixar o teto obedece na hora"
    );
    // Subir com o teto na MÉDIA: chega lá e não passa.
    let mut g = Governador::novo(N::Baixa);
    let niveis: Vec<N> = (0..200).map(|_| g.ler(0.01, N::Media)).collect();
    assert!(niveis.contains(&N::Media), "com folga sobe até o teto");
    assert!(!niveis.contains(&N::Alta), "e nunca acima dele");
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
