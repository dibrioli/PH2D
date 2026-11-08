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
    // Dono 08/11: «Rename layers…» e «Open in tab…» entre a Layer e a fila.
    let botao = achado(&r, INSP_PHYS_EDIT_MATRIX).expect("Open in tab…");
    let rename = achado(&r, INSP_PHYS_LAYER_RENAME).expect("Rename layers…");
    assert!(botao.y > lista.y + lista.h, "os botões abaixo da Layer");
    let primeiro = caixas
        .iter()
        .chain([&mais])
        .map(|c| c.y)
        .fold(f32::MAX, f32::min);
    for b in [botao, rename] {
        assert!(b.y + b.h < primeiro, "os botões antes da fila: {b:?}");
    }
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

/// ⭐ Onda 9f (dono 07/11: «não gostei do sublinhado. vamos tentar uma outline colorida na moldura»): cada quadro
/// leva, na MOLDURA, o contorno da cor da SUA camada, dentro da caixa (a medida não muda); o desligado, a mesma cor
/// mais apagada. O seletor Layer e os campos do Rename, o da camada de cada um.
#[test]
fn cada_quadro_tem_a_moldura_na_cor_da_sua_camada() {
    use ph2d_editor_core::paint::{
        espessura_do_contorno, forget_layer_outlines, layer_outline_of, resolve,
    };
    use ph2d_editor_core::panel::PanelHost;
    use ph2d_tokens::{ColorToken, LAYER_OUTLINE_OFF_ALPHA};
    mundo();
    forget_layer_outlines();
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let tema = host.theme();
    let e = espessura_do_contorno();
    let mut estados = Vec::new();
    for j in [0, 1, 3] {
        let quadro = achado(&r, INSP_PHYS_COLLIDES[j]).expect("o quadro está pintado");
        let s = layer_outline_of(INSP_PHYS_COLLIDES[j]).expect("o quadro tem contorno");
        assert_eq!(s.camada, j, "o quadro {j} contorna na camada {}", s.camada);
        assert_eq!(s.rect, quadro, "o contorno mudou a caixa do quadro {j}");
        let t = s.traco;
        assert!(
            (t.x - quadro.x - e * 0.5).abs() < 1e-3
                && (t.x + t.w + e * 0.5 - quadro.x - quadro.w).abs() < 1e-3
                && (t.y - quadro.y - e * 0.5).abs() < 1e-3
                && (t.y + t.h + e * 0.5 - quadro.y - quadro.h).abs() < 1e-3,
            "o traço do quadro {j} não está dentro da caixa: {t:?} em {quadro:?}"
        );
        let base = resolve(ColorToken::layer(j), tema);
        let cor = if s.ligado {
            base
        } else {
            base.multiply_alpha(LAYER_OUTLINE_OFF_ALPHA)
        };
        assert_eq!(s.cor, cor.components, "a cor da camada {j}");
        estados.push(s.ligado);
    }
    assert!(
        estados.contains(&true) && estados.contains(&false),
        "a cena tem de mostrar os dois estados: {estados:?}"
    );
    let pick = layer_outline_of(INSP_PHYS_LAYER_PICK).expect("o seletor Layer tem contorno");
    assert_eq!(pick.camada, 3, "o seletor contorna na camada do objecto");
    assert_eq!(
        Some(pick.rect),
        achado(&r, INSP_PHYS_LAYER_PICK),
        "o contorno do seletor mudou a caixa dele"
    );
    fim();
}

/// ⭐ Onda 9e: um nome comprido corta-se no título dobrado, e a dica dele diz o resumo com o nome INTEIRO.
#[test]
fn o_titulo_dobrado_diz_o_nome_inteiro_na_dica() {
    use ph2d_panel_inspector::ids::CARD_PHYS_LAYERS;
    let longo = "Flying Enemies That Only Hit The Players";
    let mut nomes: [String; 32] = Default::default();
    nomes[0] = "Ground".into();
    nomes[3] = longo.into();
    let mut linhas = [0u32; 32];
    linhas[3] = 1;
    linhas[0] = 1 << 3;
    set_current(nomes, linhas, 1 | 1 << 3);
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let dica = host
        .store()
        .tooltip_for(CARD_PHYS_LAYERS)
        .map(str::to_owned);
    assert!(
        dica.as_deref().is_some_and(|d| d.contains(longo)),
        "a dica do título não tem o nome inteiro: {dica:?}"
    );
    fim();
}

/// O nome comprido, cortado com «…» na frase e no título dobrado, volta INTEIRO no balão: a mão sobre a frase
/// «Applies to everything on …» acende o id dela, e cada um diz o nome todo (collider e Ray Sensor). O nome curto,
/// que cabe, não ganha balão nenhum (dono 08/11: o balão que repete o que se lê é ruído); as opções da lista
/// mostram o nome inteiro, então nunca o repetem.
///
/// Mutações: o balão sempre aceso (o nome curto fica vermelho); sem a dica da frase; sem o rect dela.
#[test]
fn o_nome_comprido_volta_inteiro_ao_passar_a_mao() {
    use ph2d_editor_core::panel::PanelHostInternal as _;
    use ph2d_host::{PointerButton, PointerEvent, PointerKind};
    let dica = |host: &ph2d_ui_testkit::MockPanelHost, id| {
        host.store().tooltip_for(id).unwrap_or_default().to_owned()
    };
    // O nome curto cabe: nada cortou, nada a dizer.
    let mut nomes: [String; 32] = Default::default();
    nomes[3] = "Foes".into();
    set_current(nomes, [u32::MAX; 32], 1 | 1 << 3);
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_COLLIDES_APPLIES).is_some(),
        "a frase tem rect"
    );
    let _ = pinta(&mut host, &mut st);
    assert_eq!(
        dica(&host, INSP_PHYS_COLLIDES_APPLIES),
        "",
        "o nome curto não se repete"
    );
    assert_eq!(
        dica(&host, CARD_PHYS_LAYERS),
        "",
        "o título inteiro não se repete"
    );
    assert_eq!(
        dica(&host, INSP_PHYS_LAYER[3]),
        "",
        "a opção mostra o nome inteiro"
    );
    fim();
    // Nomes curtos, mas a fila não cabe no título dobrado: ele mostra só a camada, e o balão diz a fila toda.
    let mut nomes: [String; 32] = Default::default();
    let curtos = [
        "Ground", "Player", "Props", "Zones", "Enemies", "Pickups", "Water", "Lava",
    ];
    for (i, n) in curtos.iter().enumerate() {
        nomes[i] = (*n).into();
    }
    set_current(nomes, [u32::MAX; 32], 0xFF);
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let _ = pinta(&mut host, &mut st);
    assert!(
        dica(&host, CARD_PHYS_LAYERS).contains("Lava"),
        "a fila trocada pela camada"
    );
    assert_eq!(dica(&host, INSP_PHYS_COLLIDES_APPLIES), "", "«Zones» cabe");
    fim();
    let longo = "Flying Enemies Of The North";
    let mut nomes: [String; 32] = Default::default();
    nomes[3] = longo.into();
    set_current(nomes, [u32::MAX; 32], 1 | 1 << 3);
    let (mut host, mut st) = cena(na_camada_3(with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let frase = achado(&r, INSP_PHYS_COLLIDES_APPLIES).expect("a frase tem rect");
    let _ = host.dispatch_pointer_event(PointerEvent {
        x: frase.x + frase.w * 0.5,
        y: frase.y + frase.h * 0.5,
        pressure: 1.0,
        kind: PointerKind::Move,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    });
    assert_eq!(host.store().hot_id(), Some(INSP_PHYS_COLLIDES_APPLIES));
    let esperada = format!("Applies to everything on {longo}");
    assert_eq!(dica(&host, INSP_PHYS_COLLIDES_APPLIES), esperada);
    assert_eq!(dica(&host, INSP_RAY_SEES_APPLIES), "", "sem raio, sem dica");
    assert!(dica(&host, CARD_PHYS_LAYERS).contains(longo));
    // A lista aberta pinta a opção com o nome inteiro: sem balão.
    let chip = achado(&r, INSP_PHYS_LAYER_PICK).expect("o seletor");
    let _ = clica(&mut host, &mut st, chip);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, INSP_PHYS_LAYER[3]).is_some(), "a opção da 3");
    assert_eq!(dica(&host, INSP_PHYS_LAYER[3]), "");
    // O Ray Sensor, os mesmos campos: a mesma dica nos ids dele.
    let olho = ph2d_editor_core::ray_edits::InspectorRayInfo {
        layer: 3,
        ..crate::a_seccao_ray_sensor_esta_viva::olho()
    };
    ph2d_panel_inspector::set_current_inspector_ray(Some(olho));
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    let _ = pinta(&mut host, &mut st);
    ph2d_panel_inspector::set_current_inspector_ray(None);
    assert_eq!(dica(&host, INSP_RAY_SEES_APPLIES), esperada);
    fim();
}
