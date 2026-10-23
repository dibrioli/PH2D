//! ⭐ W6g r9 — **o teto dos cantos chega à tela** (handoff de integração W6b–W6g §9.4): o contador da placa
//! (`TemposMs::cantos_recusados`, gateado por `os_tetos_da_placa::o_teto_dos_cantos_recusa_e_diz`) → a
//! `LuzDoQuadro` → `AvisoDaLuz::CantosArredondados` na ficha que a shell entrega ao Inspector (a mesma
//! `publicar` da `fase_snapshots_publish`) → a frase `panel.inspector.luz.corners_rounded_n` pintada no
//! cartão LUZ (o censo dos avisos). O controlo: o teto do padrão, com os tempos lidos, não diz nada.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::{AvisoDaLuz, InspectorLuzInfo, NivelDaLuz, PreferenciaDaLuz};
use ph2d_editor_core::zones::Rect;
use ph2d_light2d_ecs::Luz;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, censo_dos_avisos};
use ph2d_ui_testkit::MockPanelHost;

use crate::comum::{gpu, renderer};
use crate::medida_luz2d_w5d::{FHD, vista};

/// O máximo de quadros conduzidos: os tempos leem-se SEM ESPERAR, de há poucos quadros.
const QUADROS: usize = 60;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};
const BAIXA: PreferenciaDaLuz = PreferenciaDaLuz {
    nivel: NivelDaLuz::Baixa,
    automatica: false,
};

/// A cena 5 conduzida como a shell a conduz (`preparar` · `gravar` · `publicar`), com a lâmpada escolhida.
fn ficha(gpu: &ph2d_gpu::GpuContext, teto: u32) -> (InspectorLuzInfo, LuzDoQuadro) {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let lampada = w
        .query::<(Entity, &Luz)>()
        .iter(&w)
        .next()
        .expect("a cena 5 tem uma lâmpada")
        .0;
    let r = renderer(gpu);
    let mut luz = LuzDoQuadro::new().com_teto_dos_cantos(teto);
    let mut info = None;
    for _ in 0..QUADROS {
        assert!(luz.preparar(&w, vista(1.0), FHD), "a cena 5 tem luz");
        luz.gravar(gpu, &mut PresentWorld::new(), &r);
        assert!(luz.acesa(), "o solver recusou: {:?}", luz.erro());
        info = luz.publicar(BAIXA, &w, (Some(lampada.to_bits()), 1)).0;
        if luz.leituras_de_custo() >= 2 {
            break;
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    assert!(
        luz.leituras_de_custo() >= 2,
        "os tempos foram lidos {} vez(es) em {QUADROS} quadros: a régua é cega",
        luz.leituras_de_custo()
    );
    (info.expect("a lâmpada tem a ficha da secção LUZ"), luz)
}

/// As frases que o cartão LUZ pintou com esta ficha.
fn pintadas(info: InspectorLuzInfo) -> Vec<String> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    ph2d_panel_inspector::set_current_inspector_luz(Some(info));
    let (_, avisos) =
        censo_dos_avisos::medindo(|| host.paint::<InspectorPanel>(&mut state, VIEWPORT));
    ph2d_panel_inspector::set_current_inspector_luz(None);
    avisos.into_iter().map(|a| a.texto).collect()
}

/// **Mutações que devem sangrar:** a cópia dos tempos na `LuzDoQuadro` (`quadro.rs`) · o `push` do aviso
/// (`quadro_avisos.rs`) · a frase do aviso no cartão (`sections/luz.rs`).
#[test]
fn o_teto_dos_cantos_chega_a_frase_do_cartao_luz() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let (info, luz) = ficha(&gpu, u32::MAX);
    let placa: u32 = luz
        .solver()
        .and_then(|s| s.cantos_recusados(&gpu))
        .expect("os cantos")
        .iter()
        .sum();
    assert!(placa > 0, "o teto mínimo não recusou nada na placa");
    assert!(
        info.avisos
            .contains(&AvisoDaLuz::CantosArredondados { blocos: placa }),
        "a placa recusou {placa} blocos e a ficha diz {:?}",
        info.avisos
    );
    let frase = ph2d_i18n::tr_with("panel.inspector.luz.corners_rounded_n", &[("n", &placa)]);
    let frases = pintadas(info);
    assert!(
        frases.contains(&frase),
        "o cartão LUZ não pintou «{frase}»: {frases:?}"
    );

    let (padrao, _) = ficha(&gpu, ph2d_light2d_gpu::CANTOS_POR_BLOCOS_PADRAO);
    assert!(
        !padrao
            .avisos
            .iter()
            .any(|a| matches!(a, AvisoDaLuz::CantosArredondados { .. })),
        "controlo: o teto do padrão recusou {:?}",
        padrao.avisos
    );
}
