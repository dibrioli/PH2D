//! Os gates da CÁPSULA — ordem do dono (2026-09-19). ⚠️ Um arnês de PIXEL para este painel não
//! existe, então o que se mede aqui é a GEOMETRIA (a forma que a cápsula toma, onde os pinos caem,
//! com que raio são desenhados) e a LEI (quando ela substitui o cartão), as duas por VALOR. O
//! censo de que o pintor a chama vive ao lado, por `include_str!`.

use crate::geom::{
    self, CARD_W, Detalhe, HEADER_H, ROW_H, View, ZOOM_DA_CAPSULA, capsula_h, raio_do_pino,
};
use crate::snapshot::{GraphNodeView, NodeViewKind, PortView};
use crate::state::ViewState;
use ph2d_editor_core::zones::Rect;
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const RECT: Rect = Rect::new(0.0, 0.0, 800.0, 600.0);

fn vista(zoom: f32) -> View {
    View::new(
        RECT,
        ViewState {
            zoom,
            ..ViewState::default()
        },
    )
}

fn porta() -> PortView {
    PortView {
        name: "p",
        domain: Domain::Instances,
        dim: Dim::Vec2,
        clock: Clock::Frame,
    }
}

fn no(entradas: usize, saidas: usize) -> GraphNodeView {
    GraphNodeView {
        kind: NodeViewKind::Node,
        id: 1,
        display_name: "Duplicator".into(),
        category: ph2d_node_registry::NodeUiCategory::Source,
        silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        x: 0.0,
        y: 0.0,
        inputs: (0..entradas).map(|_| porta()).collect(),
        outputs: (0..saidas).map(|_| porta()).collect(),
        primary_input: 0,
        readout: Some("42".into()),
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: Vec::new(),
        sections: Vec::new(),
    }
}

/// ⭐⭐⭐ **A CÁPSULA SUBSTITUI O CARTÃO ABAIXO DO LIMIAR DO TEXTO — e não antes.**
///
/// ⚠️ **O limiar é o MESMO do texto de propósito:** dois limiares diferentes dariam uma faixa de
/// zoom em que o cartão é «completo» e não tem nada legível dentro — o *«tudo em branco»* que o
/// smoke de 2026-09-05 devolveu.
#[test]
fn a_capsula_substitui_o_cartao_abaixo_do_limiar_do_texto() {
    assert_eq!(geom::detalhe(&vista(1.0)), Detalhe::Completo);
    assert_eq!(
        geom::detalhe(&vista(ZOOM_DA_CAPSULA * 1.001)),
        Detalhe::Completo,
        "no limiar o texto ainda se le', logo o cartao ainda e' o cartao"
    );
    assert_eq!(
        geom::detalhe(&vista(ZOOM_DA_CAPSULA * 0.999)),
        Detalhe::Capsula,
        "um fio de zoom abaixo, ele vira pastilha"
    );
    assert_eq!(geom::detalhe(&vista(0.3)), Detalhe::Capsula);
}

/// ⭐⭐ **A CÁPSULA É MAIS BAIXA QUE O CARTÃO, E TEM A ALTURA DOS PINOS DELA.**
///
/// ⚠️ **Ela não é uma constante, e a razão é geométrica:** com três entradas, uma altura fixa
/// amontoaria os pinos — e a `SOCKET_HIT_R` é fixa em píxeis de ECRÃ, logo três alvos de `9 px` a
/// `4 px` de distância são um alvo só. Mantendo o passo de `ROW_H`, o espaçamento é **o mesmo** do
/// cartão completo.
#[test]
fn a_capsula_tem_a_altura_dos_pinos_dela() {
    let um = no(1, 1);
    assert_eq!(
        capsula_h(&um),
        HEADER_H,
        "o caso esmagadoramente comum e' a pastilha do cabecalho"
    );
    assert!(
        capsula_h(&um) < geom::card_h(&um),
        "e ela e' mais baixa que o cartao completo — senao nao e' uma capsula"
    );
    assert_eq!(capsula_h(&no(3, 1)), 3.0 * ROW_H, "tres pinos, tres passos");
    assert_eq!(
        capsula_h(&no(1, 4)),
        4.0 * ROW_H,
        "e o lado que manda e' o que tem MAIS pinos"
    );

    // E a altura que a geometria publica segue o detalhe.
    assert_eq!(geom::card_h_at(&um, &vista(1.0)), geom::card_h(&um));
    assert_eq!(geom::card_h_at(&um, &vista(0.3)), capsula_h(&um));
}

/// ⭐⭐ **OS PINOS CENTRAM-SE NA CÁPSULA E MANTÊM O PASSO.**
///
/// ⛔ **A metade que mata a cura barata:** sem o passo de `ROW_H` preservado, dois pinos caberiam
/// «centrados» a meio píxel um do outro e o gate da altura acima passaria na mesma.
#[test]
fn os_pinos_centram_se_na_capsula_e_mantem_o_passo() {
    let n = no(2, 1);
    let v = vista(0.4);
    let y = |output: bool, i: usize| geom::socket_center(&n, &v, output, i).1;

    let (a, b) = (y(false, 0), y(false, 1));
    assert!(
        (b - a - ROW_H * v.zoom).abs() < 1e-3,
        "o passo entre pinos e' o mesmo `ROW_H` do cartao: {}",
        (b - a) / v.zoom
    );
    let meio = v.pt(0.0, capsula_h(&n) * 0.5).1;
    assert!(
        ((a + b) * 0.5 - meio).abs() < 1e-3,
        "e o conjunto fica CENTRADO na capsula"
    );
    // A saída única cai no meio.
    assert!((y(true, 0) - meio).abs() < 1e-3);
    // E o x é a borda certa de cada lado.
    assert!(geom::socket_center(&n, &v, true, 0).0 > geom::socket_center(&n, &v, false, 0).0);
}

/// ⭐⭐⭐ **OS PINOS PARAM DE ENCOLHER** — *«os slots de conexão ficam maiores»*.
///
/// ⚠️ A régua é a RAZÃO contra o que eles sairiam sem a lei, e não um número: *um gate escrito
/// sobre `3,3 px` afirmaria o raio base e o zoom, nunca a lei.*
#[test]
fn os_pinos_param_de_encolher_na_capsula() {
    const BASE: f32 = 5.0;
    let sem_lei = |z: f32| BASE * z;
    assert!(
        (raio_do_pino(&vista(1.0), BASE) - sem_lei(1.0)).abs() < 1e-6,
        "acima do limiar nada muda — o cartao de perto e' byte a byte o de sempre"
    );
    let z = 0.3;
    let r = raio_do_pino(&vista(z), BASE);
    assert!(
        r > sem_lei(z) * 2.0,
        "a `zoom {z}` o pino sai mais do DOBRO do que sairia: {r} contra {}",
        sem_lei(z)
    );
    assert!(
        (r - BASE * ZOOM_DA_CAPSULA).abs() < 1e-6,
        "e o que ele congela e' o raio do LIMIAR, nao um multiplicador"
    );
}

/// ⛔⛔ **NUMA CÁPSULA NÃO HÁ TOGGLE, BADGE NEM MOLDURA** — *«os parâmetros de ajustes são
/// escondidos»*, e com eles tudo o que não é o nome e os pinos.
///
/// ⚠️ **A guarda vive na GEOMETRIA e não no pintor**, e é isso que o gate afirma: estas três
/// funções são a porta que o desenho **e** o hit-test partilham — *um alvo registado onde nada é
/// desenhado é um clique que morre num controlo invisível*.
#[test]
fn numa_capsula_nao_ha_toggle_nem_badge_nem_moldura() {
    let mut n = no(1, 1);
    n.preview = Some(vec![[0.0, 0.0]]);
    n.inert = true;
    let perto = vista(1.0);
    let longe = vista(0.3);
    assert!(
        geom::preview_toggle_rect(&n, &perto).is_some()
            && geom::inert_badge_rect(&n, &perto).is_some(),
        "o CONTROLO: de perto os tres existem — senao o `None` de longe nao diz nada"
    );
    assert!(geom::preview_toggle_rect(&n, &longe).is_none());
    assert!(geom::inert_badge_rect(&n, &longe).is_none());
    assert!(
        geom::preview_frame_rect(&n, &longe, crate::state::PreviewPos::Below).is_none(),
        "e a moldura do carimbo tambem nao"
    );
}

/// ⛔ **O PINTOR DO CARTÃO DESVIA PARA A CÁPSULA** — a metade que só texto responde, irmã do
/// `os_pintores_chamam_a_lei_do_realce`: *uma lei que ninguém chama não desenha.*
#[test]
fn o_pintor_do_cartao_desvia_para_a_capsula() {
    let texto = include_str!("paint_card.rs");
    assert!(
        texto.contains("paint_capsula::draw_capsula("),
        "o `paint_card` deixou de desviar para a capsula"
    );
    // ⚠️ E o desvio tem de vir ANTES do resto: a saída cedo é o que garante que as nove peças que
    // a cápsula esconde não são desenhadas fora da pastilha.
    let i_desvio = texto
        .find("paint_capsula::draw_capsula(")
        .expect("o desvio");
    let i_corpo = texto
        .find("fill_rounded_rect(ctx.scene, body, r,")
        .expect("o corpo");
    assert!(i_desvio < i_corpo, "o desvio tem de ser a SAIDA CEDO");
}

/// ⚠️ **O nome da cápsula é MAIOR do que o título do cartão seria** — *«os nomes dos nós ficam bem
/// maiores nas cápsulas»*. A régua compara as duas leis no MESMO zoom, que é a única comparação
/// que a ordem admite.
#[test]
fn o_nome_da_capsula_e_maior_que_o_titulo_seria() {
    let capsula = include_str!("paint_capsula.rs");
    // A fracção que a cápsula usa, lida do ficheiro — derivada, não escrita duas vezes.
    const TITULO_DO_CARTAO: f32 = 13.0;
    let n = no(1, 1);
    // No limiar: `HEADER_H × 0,62` contra `13`, os dois vezes o mesmo zoom ⇒ a razão é a dos
    // coeficientes.
    let da_capsula = capsula_h(&n) * 0.62;
    assert!(
        da_capsula > TITULO_DO_CARTAO,
        "o nome da capsula ({da_capsula}) tem de ser maior que o titulo do cartao \
         ({TITULO_DO_CARTAO}) — a ordem do dono diz «bem maiores»"
    );
    assert!(
        capsula.contains("NOME_DA_CAPSULA: f32 = 0.62"),
        "a fraccao mudou: re-meca a razao acima em vez de a deixar a mentir"
    );
    // E a cápsula tem sempre a largura do cartão — só a ALTURA muda.
    assert!((CARD_W - 190.0).abs() < 1e-6);
}
