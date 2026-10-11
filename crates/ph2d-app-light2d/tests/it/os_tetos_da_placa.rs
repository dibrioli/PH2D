//! ⭐ **Os tetos do dispositivo** (W5e, `docs/Luz2D/01_plano.md` §7): a luz acima de 1080p cabe no
//! lado de textura e na memória que a placa comporta, cedendo a grade FINA (a sombra direta) e nunca
//! o nível do artista; e o governador tem degraus abaixo do Baixo.
//!
//! O vermelho que estes gates guardam (rodada `medida_luz2d_w5e`, A1): a 2160p a captura fina pedia
//! 10 240 texels contra os 8 192 do dispositivo — `Device::create_texture` dava erro de validação
//! (`luz2d/tex/cobertura_fina`) e a luz saía apagada nos três níveis, sem aviso nenhum.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::{Custo, Governador, ORCAMENTO_MS, degrau_do};
use ph2d_app_light2d::teto::{Tetos, config_do_quadro, fina_que_cabe};
use ph2d_app_light2d::{LuzDoQuadro, grade_do_quadro, smoke};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{AvisoDaLuz, NivelDaLuz as N};
use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{Config, ErroSolver, PassList, Solver};

use crate::comum::{gpu, renderer};
use crate::medida_luz2d_w5e::fabrica;

/// O lado de textura que o dispositivo da casa pede (o padrão do wgpu; o adaptador da RTX 5060 Ti
/// daria 32 768, mas subi-lo é da `ph2d-gpu` — ver o handoff W5e).
const LADO: u32 = 8192;
const UHD: (u32, u32) = (3840, 2160);

/// A configuração do degrau `n` com a fina `2^k` na vista de fábrica de `alvo`.
fn config(alvo: (u32, u32), n: N, k: u32) -> Config {
    let dg = degrau_do(n).com_fina_engrossada(k);
    let a = fabrica(alvo);
    config_do_quadro(grade_do_quadro(a, alvo, dg).expect("grade"), a, dg, true)
}

fn bytes(alvo: (u32, u32), n: N, k: u32) -> u64 {
    let c = config(alvo, n, k);
    PassList::de(
        &Config {
            grade: c.grade_efetiva(),
            ..c
        },
        false,
    )
    .bytes_alocados()
}

/// A fina cede ao LADO de textura, e só onde precisa: a 1080p, 1440p e no ultrawide do dono nada
/// muda; a 2160p o MENOR `k` que cabe (o `k − 1` não cabe). Controlo na mesma corrida: o degrau da W5d
/// (`k = 0`) a 2160p passa o lado — o vermelho da rodada.
#[test]
fn a_fina_cede_ao_lado_da_placa_e_so_onde_precisa() {
    let tetos = Tetos {
        lado: LADO,
        recusou: None,
    };
    for alvo in [(1920, 1080), (2560, 1440), (2560, 1080), UHD] {
        for n in N::ALL {
            let k = fina_que_cabe(fabrica(alvo), alvo, degrau_do(n), tetos)
                .unwrap_or_else(|| panic!("{alvo:?} {n:?}: nada cabe"));
            assert!(
                config(alvo, n, k).cabe_na_placa(LADO).is_ok(),
                "{alvo:?} {n:?}: o k {k} escolhido não cabe"
            );
            if alvo.0 <= 2560 {
                assert_eq!(k, 0, "{alvo:?} {n:?}: engrossou sem precisar");
            } else {
                assert!(k >= 1, "{alvo:?} {n:?}: a 2160p a fina da W5d cabia?");
                assert!(
                    matches!(
                        config(alvo, n, k - 1).cabe_na_placa(LADO),
                        Err(ErroSolver::GradeGrandeDemais { .. })
                    ),
                    "{alvo:?} {n:?}: o k {k} não é o menor"
                );
            }
        }
    }
}

/// A fina cede aos BYTES que a placa recusou: o menor `k` abaixo deles; abaixo de todo degrau, nada
/// cabe (`None`: a luz recusa e o aviso diz).
#[test]
fn a_fina_cede_a_memoria_recusada() {
    let alvo = (1920, 1080);
    let recusou = bytes(alvo, N::Media, 0);
    let tetos = Tetos {
        lado: LADO,
        recusou: Some(recusou),
    };
    let k = fina_que_cabe(fabrica(alvo), alvo, degrau_do(N::Media), tetos).expect("cabe");
    assert!(k >= 1, "a recusa não engrossou nada");
    assert!(bytes(alvo, N::Media, k) < recusou);
    assert!(
        bytes(alvo, N::Media, k - 1) >= recusou,
        "o k {k} não é o menor"
    );
    let minimo = (0..=degrau_do(N::Media).fina_max())
        .map(|k| bytes(alvo, N::Media, k))
        .min()
        .expect("degraus");
    let nada = Tetos {
        lado: LADO,
        recusou: Some(minimo),
    };
    assert_eq!(
        fina_que_cabe(fabrica(alvo), alvo, degrau_do(N::Media), nada),
        None,
        "abaixo de todo degrau, nada cabe"
    );
}

fn fixo(ms: f64) -> Custo {
    Custo {
        por_nivel: 0.0,
        fixo: ms,
    }
}

/// Abaixo do Baixo o governador engrossa a FINA (o custo que não segue o nível cresce com a janela),
/// e ao sobrar folga devolve-a ANTES de subir o nível (a sombra nítida primeiro). Controlo: sem faixa
/// (a W5d) o Baixo era o chão — caro para sempre.
#[test]
fn o_governador_engrossa_a_fina_abaixo_do_baixo() {
    let caro = fixo(ORCAMENTO_MS * 1.5);
    let mut sem_faixa = Governador::novo(N::Baixa);
    for _ in 0..30 {
        sem_faixa.ler(caro, N::Alta);
    }
    assert_eq!((sem_faixa.nivel, sem_faixa.fina), (N::Baixa, 0), "controlo");

    let mut g = Governador::novo(N::Baixa);
    g.faixa_da_fina(0, 3);
    for _ in 0..3 {
        g.ler(caro, N::Alta);
    }
    assert_eq!(
        (g.nivel, g.fina),
        (N::Baixa, 1),
        "três caras engrossam a fina"
    );
    for _ in 0..30 {
        g.ler(caro, N::Alta);
    }
    assert_eq!(g.fina, 3, "até o fim da faixa, e não passa");
    // Com a fina 3, a fina 2 prevista é ×4 no fixo: só volta com folga para ela.
    let barato = fixo(ORCAMENTO_MS * 0.75 / 4.0 * 0.9);
    let voltou = (0..100).any(|_| {
        g.ler(barato, N::Alta);
        g.fina == 2
    });
    assert!(voltou, "com folga a fina volta");
    assert_eq!(g.nivel, N::Baixa, "a fina volta ANTES do nível subir");
    for _ in 0..300 {
        g.ler(fixo(0.01), N::Alta);
    }
    assert_eq!((g.nivel, g.fina), (N::Alta, 0), "tudo de volta até o teto");
    g.faixa_da_fina(2, 3);
    assert_eq!(g.fina, 2, "o mínimo dos tetos prende a fina");
}

/// Abaixo do Baixo, com o custo nas CASCATAS (a 2160p: 10,0 de 12,3 ms, rodada A1), quem engrossa é
/// a CÉLULA — a fina não pagaria nada; e ela volta (antes do nível) quando a folga prevista paga.
#[test]
fn o_governador_engrossa_a_celula_quando_as_cascatas_pesam() {
    use ph2d_app_light2d::qualidade::CELULA_MAX;
    let caro = Custo {
        por_nivel: ORCAMENTO_MS * 1.6,
        fixo: ORCAMENTO_MS * 0.4,
    };
    let mut g = Governador::novo(N::Baixa);
    g.faixa_da_fina(0, 3);
    for _ in 0..3 {
        g.ler(caro, N::Alta);
    }
    assert_eq!((g.fina, g.celula), (0, 1), "as cascatas pesam: a célula");
    for _ in 0..30 {
        g.ler(caro, N::Alta);
    }
    assert_eq!(g.celula, CELULA_MAX, "até o teto da célula");
    assert!(g.fina > 0, "acabada a célula, a fina");
    for _ in 0..600 {
        g.ler(fixo(0.01), N::Alta);
    }
    assert_eq!((g.nivel, g.fina, g.celula), (N::Alta, 0, 0), "tudo volta");
    // Prevista pela lei: na célula 1 com as cascatas a 0,2 do orçamento, a célula 0 custaria 0,8 —
    // acima dos 0,75 de folga: não volta.
    let mut h = Governador::novo(N::Baixa);
    h.faixa_da_fina(0, 3);
    for _ in 0..3 {
        h.ler(caro, N::Alta);
    }
    let justo = Custo {
        por_nivel: ORCAMENTO_MS * 0.2,
        fixo: 0.0,
    };
    for _ in 0..100 {
        h.ler(justo, N::Alta);
    }
    assert_eq!(h.celula, 1, "sem folga prevista a célula não volta");
}

fn cena_3() -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    w
}

/// Um quadro da luz sob o escopo de validação do dispositivo: devolve o erro, se houve.
fn quadro(gpu: &GpuContext, luz: &mut LuzDoQuadro, w: &World, alvo: (u32, u32)) -> Option<String> {
    let r = renderer(gpu);
    let escopo = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
    assert!(luz.preparar(w, fabrica(alvo), alvo));
    luz.gravar(gpu, &mut PresentWorld::new(), &r);
    pollster::block_on(escopo.pop()).map(|e| format!("{e}"))
}

/// ⭐ A 2160p a luz ACENDE (Baixo e Médio), sem erro do dispositivo, com a sombra engrossada e o aviso
/// que o diz. Controlo na mesma corrida: o solver do degrau da W5d recusa limpo (a recusa, não o erro
/// de validação de antes).
#[test]
fn a_luz_acende_a_2160p_com_a_sombra_engrossada() {
    let Some(gpu) = gpu() else { return };
    assert_eq!(gpu.device.limits().max_texture_dimension_2d, LADO);
    let w = cena_3();
    let escopo = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let antigo = Solver::novo(&gpu, config(UHD, N::Baixa, 0));
    assert!(
        matches!(antigo, Err(ErroSolver::GradeGrandeDemais { .. })),
        "controlo: o degrau da W5d a 2160p tinha de recusar"
    );
    drop(antigo);
    assert!(
        pollster::block_on(escopo.pop()).is_none(),
        "a recusa deu erro"
    );
    for n in [N::Baixa, N::Media] {
        let mut luz = LuzDoQuadro::com_degrau(degrau_do(n));
        let erro = quadro(&gpu, &mut luz, &w, UHD);
        assert_eq!(erro, None, "{n:?}: o dispositivo deu erro");
        assert!(luz.acesa(), "{n:?}: a luz apagou: {:?}", luz.erro());
        let (k, pelos_tetos) = luz.fina_engrossada();
        assert!(k >= 1 && pelos_tetos, "{n:?}: a fina não cedeu ({k})");
        assert!(
            luz.avisos_de(Entity::PLACEHOLDER)
                .contains(&AvisoDaLuz::SombraEngrossada { vezes: 1 << k }),
            "{n:?}: o artista não é avisado"
        );
    }
}

/// ⭐ Quando a placa RECUSA a memória, a luz recua para a fina mais grossa e, se nem essa cabe, apaga
/// com o aviso — sem erro de validação e sem pânico. A recusa provoca-se de verdade: uma instância com
/// o limiar do orçamento do driver (`MemoryBudgetThresholds`) no menor valor em que o dispositivo
/// nasce e aloca 64 MB. Controlo: a mesma luz na placa da casa acende.
#[test]
fn a_luz_recua_quando_a_placa_recusa_a_memoria() {
    let Some(casa) = gpu() else { return };
    let w = cena_3();
    let alvo = (1920, 1080);
    let mut controlo = LuzDoQuadro::com_degrau(degrau_do(N::Media));
    assert_eq!(quadro(&casa, &mut controlo, &w, alvo), None);
    assert!(controlo.acesa(), "controlo: a luz na placa da casa");
    drop(casa);

    let Some(apertada) = [2u8, 3, 4, 5, 6, 8, 10, 13, 16, 20, 25, 30, 40, 50]
        .into_iter()
        .find_map(placa_com_limiar)
    else {
        eprintln!("sem VK_EXT_memory_budget (o limiar não recusa): nada a medir");
        return;
    };
    // A 2160p nem a fina mais grossa cabe na folga do limiar mínimo (< 64 MB + um passo dele): a luz
    // TEM de acabar apagada — e a 1080p ela pode acender com a fina engrossada.
    for (alvo, tem_de_apagar) in [(UHD, true), (alvo, false)] {
        recua(&apertada, &w, alvo, tem_de_apagar);
    }
}

fn recua(apertada: &GpuContext, w: &World, alvo: (u32, u32), tem_de_apagar: bool) {
    let mut luz = LuzDoQuadro::com_degrau(degrau_do(N::Media));
    let mut recusas = Vec::new();
    for _ in 0..=degrau_do(N::Media).fina_max() + 1 {
        assert_eq!(quadro(apertada, &mut luz, w, alvo), None);
        if let Some(ErroSolver::SemMemoria { bytes }) = luz.erro()
            && recusas.last() != Some(bytes)
        {
            recusas.push(*bytes);
        }
        if luz.acesa() {
            break;
        }
    }
    assert!(!recusas.is_empty(), "a placa apertada não recusou nada");
    assert!(
        recusas.windows(2).all(|p| p[1] < p[0]),
        "cada nova tentativa pede menos: {recusas:?}"
    );
    // O RECUO: depois da 1.ª recusa a luz tenta uma fina mais grossa (outra recusa, menor) ou acende.
    assert!(
        recusas.len() >= 2 || luz.acesa(),
        "a luz não recuou depois da recusa: {recusas:?}"
    );
    assert!(
        !(tem_de_apagar && luz.acesa()),
        "{alvo:?}: acendeu onde nada cabia"
    );
    if luz.acesa() {
        assert!(luz.fina_engrossada().0 >= 1, "acendeu sem engrossar");
    } else {
        assert!(
            luz.avisos_de(Entity::PLACEHOLDER)
                .iter()
                .any(|a| matches!(a, AvisoDaLuz::SemMemoria { .. })),
            "apagada sem o aviso"
        );
        // Nada cabe abaixo da recusa: os quadros seguintes NÃO voltam a alocar a grade.
        let montagens = luz.montagens();
        for _ in 0..3 {
            assert_eq!(quadro(apertada, &mut luz, w, alvo), None);
        }
        assert_eq!(
            luz.montagens(),
            montagens,
            "a luz apagada continua a montar o solver a cada quadro"
        );
    }
}

/// O dispositivo com o limiar `t` % do orçamento do driver, se ele nasce e aloca 64 MB.
fn placa_com_limiar(t: u8) -> Option<GpuContext> {
    let instancia = wgpu::Instance::new(wgpu::InstanceDescriptor {
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
            for_resource_creation: Some(t),
            for_device_loss: None,
        },
        ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
    });
    let gpu = GpuContext::new(instancia, None).ok()?;
    let escopo = gpu.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let t64 = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tetos/64MB"),
        size: wgpu::Extent3d {
            width: 4096,
            height: 2048,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let recusou = pollster::block_on(escopo.pop()).is_some();
    drop(t64);
    (!recusou).then_some(gpu)
}
