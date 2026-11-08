//! ⭐ **A fila «Collides with» do cartão Collision Layers** (ficha f1, ondas 6 e 7): o clique real no quadro pede o par da
//! camada do objecto; só as visíveis; a forma solta não tem fila. Que a edição é a da célula da matriz e
//! chega ao motor está em `ph2d-app-physics` (`physics_tests_quadros`) e, com o desfazer, na shell.

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::{as_part, with_body};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_panel_inspector::camadas_fisica::{proxima_livre, set_current};
use ph2d_panel_inspector::ids::*;

/// Camadas 0, 1 e 3 à vista, com nomes; a 3 toca a 1 e não a 0. A linha 0 toca todas: a armadilha de quem lê
/// a linha errada.
fn mundo() {
    let mut nomes: [String; 32] = Default::default();
    nomes[0] = "Ground".into();
    nomes[1] = "Props".into();
    nomes[3] = "Enemy".into();
    let mut linhas = [0u32; 32];
    linhas[0] = u32::MAX;
    linhas[1] = 1 | 1 << 3;
    linhas[3] = 1 << 1;
    set_current(nomes, linhas, 1 | 1 << 1 | 1 << 3);
}

fn na_camada_3(mut info: InspectorPhysicsInfo) -> InspectorPhysicsInfo {
    info.layer = 3;
    info
}

fn fim() {
    set_current(Default::default(), [u32::MAX; 32], 0xFF);
    sai();
}

#[test]
fn o_clique_real_no_quadro_pede_o_par_com_a_camada_do_objecto() {
    mundo();
    for info in [with_body(), as_part()] {
        let (mut host, mut st) = cena(na_camada_3(info), [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        for (quadro, b) in [(INSP_PHYS_COLLIDES[1], 1), (INSP_PHYS_COLLIDES[3], 3)] {
            let caixa = achado(&r, quadro).expect("o quadro está pintado");
            assert_eq!(
                clica(&mut host, &mut st, caixa),
                [EditorAction::LayerPairToggle { a: 3, b }],
                "um clique, uma edição"
            );
        }
    }
    fim();
}

/// Só as visíveis, e a própria aparece; a do objecto entra mesmo escondida.
#[test]
fn a_fila_usa_so_as_visiveis_e_a_do_objecto() {
    mundo();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let vistas: Vec<usize> = (0..32)
        .filter(|&j| achado(&r, INSP_PHYS_COLLIDES[j]).is_some())
        .collect();
    assert_eq!(vistas, [0, 1, 3]);
    let mut info = with_body();
    info.layer = 20;
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_COLLIDES[20]).is_some(),
        "a do objecto nunca falta"
    );
    fim();
}

/// A dica diz o estado do par certo: a linha 3 toca a 1 e não a 0.
#[test]
fn o_quadro_le_a_linha_da_camada_do_objecto() {
    mundo();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let dica = |host: &ph2d_ui_testkit::MockPanelHost, j: usize| {
        host.store()
            .tooltip_for(INSP_PHYS_COLLIDES[j])
            .unwrap_or_default()
            .to_owned()
    };
    assert_eq!(
        dica(&host, 1),
        "Things on Enemy bump into things on Props. Click to let them pass through."
    );
    assert_eq!(
        dica(&host, 0),
        "Things on Enemy pass through things on Ground. Click to make them bump."
    );
    assert_eq!(
        dica(&host, 3),
        "Things on Enemy pass through each other. Click to make them bump."
    );
    fim();
}

/// Os quadros cabem na largura, quebram de linha pelo nome, e nenhum toca o outro nem o controlo seguinte.
#[test]
fn os_quadros_quebram_pelo_nome_sem_se_tocar() {
    let mut nomes: [String; 32] = Default::default();
    for (i, nome) in nomes.iter_mut().enumerate().take(8) {
        *nome = format!("A long layer name {i}");
    }
    set_current(nomes, [u32::MAX; 32], 0xFF);
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let caixas: Vec<_> = (0..8)
        .map(|j| achado(&r, INSP_PHYS_COLLIDES[j]).expect("pintado"))
        .collect();
    let lista = achado(&r, INSP_PHYS_LAYER_PICK).expect("a lista");
    let mais = achado(&r, INSP_PHYS_COLLIDES_ADD).expect("o +");
    assert!(
        caixas.iter().any(|c| c.y > caixas[0].y),
        "os nomes compridos quebram de linha"
    );
    for (n, a) in caixas.iter().chain([&mais]).enumerate() {
        assert!(a.y > lista.y + lista.h, "abaixo da Layer");
        assert!(
            a.x >= lista.x - 0.5 && a.x + a.w <= lista.x + lista.w + 0.5,
            "{n} sai da coluna: {a:?} contra {lista:?}"
        );
        for b in caixas.iter().skip(n + 1) {
            let separados =
                a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
            assert!(separados, "{a:?} pisa {b:?}");
        }
    }
    let botao = achado(&r, INSP_PHYS_EDIT_MATRIX).expect("Edit matrix…");
    let ultimo = caixas
        .iter()
        .chain([&mais])
        .map(|c| c.y + c.h)
        .fold(0.0, f32::max);
    assert!(botao.y > ultimo, "a frase e o botão vêm depois da fila");
    fim();
}

#[test]
fn a_forma_solta_nao_tem_quadros_nem_edit_matrix() {
    mundo();
    let solta = InspectorPhysicsInfo {
        part_owner: String::new(),
        ..as_part()
    };
    let (mut host, mut st) = cena(solta, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    for id in INSP_PHYS_COLLIDES
        .iter()
        .copied()
        .chain([INSP_PHYS_COLLIDES_ADD, INSP_PHYS_EDIT_MATRIX])
    {
        assert!(achado(&r, id).is_none(), "{id:?} pintado na solta");
    }
    assert!(achado(&r, INSP_PHYS_LAYER_PICK).is_some(), "a Layer fica");
    assert!(
        achado(&r, INSP_PHYS_LAYER_RENAME).is_some(),
        "o Rename fica"
    );
    fim();
}

#[test]
fn edit_matrix_e_o_mais_pedem_as_portas_certas() {
    mundo();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let botao = achado(&r, INSP_PHYS_EDIT_MATRIX).expect("Edit matrix…");
    assert_eq!(
        clica(&mut host, &mut st, botao),
        [EditorAction::LayerMatrixOpen { layer: 3 }]
    );
    let mais = achado(&r, INSP_PHYS_COLLIDES_ADD).expect("o +");
    assert_eq!(
        clica(&mut host, &mut st, mais),
        [EditorAction::LayerShow { layer: 2 }],
        "a próxima livre, a mesma porta do «+ Layer» da matriz"
    );
    set_current(Default::default(), [u32::MAX; 32], u32::MAX);
    assert_eq!(proxima_livre(), None);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_COLLIDES_ADD).is_none(),
        "nas 32 o + sai"
    );
    fim();
}

/// A mudança da matriz, do nome e da camada aparecem no quadro seguinte: o cartão não guarda cópia.
#[test]
fn os_quadros_acompanham_a_matriz_e_o_nome() {
    mundo();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let mut nomes: [String; 32] = Default::default();
    nomes[1] = "Crates".into();
    let mut linhas = [u32::MAX; 32];
    linhas[3] = 1;
    set_current(nomes, linhas, 1 | 1 << 1 | 1 << 3);
    let _ = pinta(&mut host, &mut st);
    assert_eq!(
        host.store().tooltip_for(INSP_PHYS_COLLIDES[1]),
        Some("Things on Layer 3 pass through things on Crates. Click to make them bump."),
    );
    fim();
}

/// ⭐ Onda 9e (dono 07/11: «melhor um sublinhado colorido que o pequeno quadrado»): cada quadro leva, debaixo
/// do NOME, o sublinhado da cor da SUA camada; o seletor Layer, o da camada do objecto, debaixo do rótulo.
#[test]
fn cada_quadro_sublinha_o_nome_na_cor_da_sua_camada() {
    use ph2d_editor_core::paint::{forget_layer_underlines, layer_underline_of, resolve};
    use ph2d_editor_core::panel::PanelHost;
    use ph2d_tokens::ColorToken;
    mundo();
    forget_layer_underlines();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let tema = host.theme();
    for j in [0, 1, 3] {
        let quadro = achado(&r, INSP_PHYS_COLLIDES[j]).expect("o quadro está pintado");
        let s = layer_underline_of(INSP_PHYS_COLLIDES[j]).expect("o quadro tem sublinhado");
        assert_eq!(s.camada, j, "o quadro {j} sublinha a camada {}", s.camada);
        assert_eq!(
            s.cor,
            resolve(ColorToken::layer(j), tema).components,
            "a cor da camada {j}"
        );
        // Debaixo do nome: depois da caixinha, na metade de baixo do quadro, sem sair dele.
        assert!(
            s.rect.x > quadro.x + quadro.h * 0.5,
            "o traço começa no nome, não na caixinha"
        );
        assert!(
            s.rect.x + s.rect.w <= quadro.x + quadro.w,
            "não passa do quadro"
        );
        assert!(s.rect.y > quadro.y + quadro.h * 0.5, "na base do nome");
        assert!(
            s.rect.y + s.rect.h <= quadro.y + quadro.h,
            "dentro do quadro"
        );
        assert!(
            s.rect.w > s.rect.h * 4.0,
            "a largura do texto, não um ponto"
        );
    }
    let pick = layer_underline_of(INSP_PHYS_LAYER_PICK).expect("o seletor Layer tem sublinhado");
    assert_eq!(pick.camada, 3, "o seletor sublinha a camada do objecto");
    fim();
}
