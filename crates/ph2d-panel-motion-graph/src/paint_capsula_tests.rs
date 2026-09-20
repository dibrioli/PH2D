//! Os gates da CÁPSULA — ordem do dono (2026-09-19). ⚠️ Um arnês de PIXEL para este painel não
//! existe, então o que se mede aqui é a GEOMETRIA (a forma que a cápsula toma, onde os pinos caem,
//! com que raio são desenhados) e a LEI (quando ela substitui o cartão), as duas por VALOR. O
//! censo de que o pintor a chama vive ao lado, por `include_str!`.

use crate::geom::{self, CARD_W, Detalhe, ROW_H, View, ZOOM_DA_CAPSULA, capsula_h};
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

/// ⭐⭐⭐ **TODAS AS CÁPSULAS TÊM A MESMA ALTURA** — *«quero cápsulas grossas grandes e robustas»*
/// e *«tamanhos irregulares»* (ordem do dono, 2026-09-19, a reprovar a 1.ª redacção).
///
/// ⛔⛔ **A 1.ª redacção fazia a altura seguir a contagem de pinos**, e daí vinham DUAS das quatro
/// queixas dele: a pastilha variava **e** o nome, que era dimensionado a partir dela. *Uma
/// grandeza a alimentar duas leituras produz duas irregularidades.*
///
/// ⚠️ **E ela é GROSSA:** `3 × ROW_H` contra os `HEADER_H` de antes — `2,54×`.
#[test]
fn todas_as_capsulas_tem_a_mesma_altura_e_ela_e_grossa() {
    let alturas = [no(1, 1), no(3, 1), no(1, 5), no(2, 4)].map(|n| capsula_h(&n));
    assert!(
        alturas.windows(2).all(|a| (a[0] - a[1]).abs() < 1e-6),
        "a altura nao pode seguir a contagem de pinos: {alturas:?}"
    );
    assert!(
        capsula_h(&no(1, 1)) > geom::HEADER_H * 2.0,
        "«grossa» e' contra o cabecalho de 26 unidades, que foi o que o dono viu"
    );
    assert!(
        capsula_h(&no(1, 1)) < geom::card_h(&no(1, 1)),
        "e ainda assim mais baixa que o cartao completo — senao nao e' uma capsula"
    );
    // E a altura que a geometria publica segue o detalhe.
    let um = no(1, 1);
    assert_eq!(geom::card_h_at(&um, &vista(1.0)), geom::card_h(&um));
    assert_eq!(geom::card_h_at(&um, &vista(0.3)), capsula_h(&um));
}

/// ⭐⭐ **E O PASSO DOS PINOS APERTA-SE NOS 5,1 % QUE NÃO CABEM** — a outra metade da
/// uniformidade: em vez de esticar a pastilha para dois nós em 136, eles encostam-se.
#[test]
fn o_passo_aperta_se_so_nos_nos_que_nao_cabem() {
    assert_eq!(geom::passo_do_pino(1), ROW_H, "um pino: o passo do cartao");
    assert_eq!(
        geom::passo_do_pino(3),
        ROW_H,
        "tres cabem com o passo inteiro"
    );
    assert!(
        geom::passo_do_pino(5) < ROW_H,
        "cinco nao cabem, logo apertam"
    );
    assert!(
        4.0 * geom::passo_do_pino(5) < geom::CAPSULA_H,
        "e apertados, eles CABEM na pastilha com folga para as bordas redondas"
    );
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
        (b - a - geom::passo_do_pino(2) * v.zoom).abs() < 1e-3,
        "o passo entre pinos e' o que a lei declara: {}",
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

/// ⭐⭐⭐ **OS PINOS SÃO GRANDES E BEM VISÍVEIS** — *«quero slots de conexão grandes e bem
/// visíveis»* (ordem do dono, 2026-09-19, depois de reprovar a 1.ª cápsula).
///
/// ⚠️ A régua é a RAZÃO contra o pino do CARTÃO no mesmo zoom, e não um número em píxeis: *um gate
/// escrito sobre `9 px` afirmaria a constante, nunca a lei.* E as duas cercas — a pastilha e o
/// vizinho — têm metade cada, porque cada uma nomeia uma coisa diferente que pode apagar o pino.
#[test]
fn os_pinos_da_capsula_sao_grandes_e_bem_visiveis() {
    const DO_CARTAO: f32 = 5.0; // `paint::SOCKET_R`
    // ⚠️ No LIMIAR — que é onde o artista o encontra ao afastar, e o único zoom em que as duas
    //    cercas ainda não mordem.
    let z = ZOOM_DA_CAPSULA;
    let um = geom::raio_do_pino_na_capsula(&vista(z), 1);
    assert!(
        um > DO_CARTAO * z * 2.0,
        "com um pino so', ele sai mais do DOBRO do que o cartao desenharia: {um}"
    );
    assert!(
        (um - geom::SOCKET_HIT_R).abs() < 1e-6,
        "e o que ele toma e' o raio do ALVO — o desenho deixa de mentir sobre onde se clica"
    );
    // ⛔ A cerca da PASTILHA: bem longe, o pino não pode ser maior que a cápsula.
    let longe = geom::raio_do_pino_na_capsula(&vista(0.15), 1);
    assert!(
        longe < geom::SOCKET_HIT_R && longe * 2.0 < geom::CAPSULA_H * 0.15,
        "a cerca da pastilha: um pino maior que metade dela deixa de ser um pino ({longe})"
    );
    // ⛔ A cerca do VIZINHO: com muitos pinos do mesmo lado eles não se tocam.
    let quatro = geom::raio_do_pino_na_capsula(&vista(z), 4);
    assert!(
        quatro * 2.0 < geom::passo_do_pino(4) * z,
        "dois pinos que se tocam leem-se como um ({quatro} contra o passo {})",
        geom::passo_do_pino(4) * z
    );
    assert!(
        quatro < um,
        "e mais pinos · pino mais pequeno, nunca o contrario"
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

/// ⭐⭐⭐ **O NOME TEM UM TAMANHO SÓ, É MAIOR QUE O TÍTULO DO CARTÃO, E NUNCA É CORTADO** — as três
/// metades do pedido do dono (*«fonts de tamanho único … e sem 3 pontos (…)»*).
///
/// ⚠️⚠️ **A terceira é a que exige um MEDIDOR REAL:** a fonte é derivada de uma ESTIMATIVA de
/// avanço médio, e a única forma de saber se ela é generosa o bastante é medir o pior nome do
/// catálogo com o mesmo medidor que o pintor usa. *Uma estimativa que ninguém confere é o
/// reticências de volta.*
#[test]
fn o_nome_tem_um_tamanho_so_e_nunca_e_cortado() {
    use crate::paint::paint_capsula::CAPSULA_FONTE;

    // ⚠️ **As duas primeiras metades vivem no COMPILADOR**, e é por isso que não estão aqui: o
    // `CAPSULA_FONTE` é uma constante (logo TAMANHO ÚNICO por construção) e o `const _: () =
    // assert!(CAPSULA_FONTE > 1.3 * TITLE_SIZE)` ao lado dela recusa um corpo que não seja «bem
    // maior». *O clippy recusa um `assert!` de teste sobre duas constantes — ele é dobrado antes
    // de correr —, e uma cerca que o compilador impõe é mais forte que um gate.*
    //
    // O que SOBRA para medir é a única metade que uma constante não decide: SEM RETICÊNCIAS — o
    // pior nome do catálogo, com o medidor que o pintor consulta.
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let disponivel = CARD_W - 2.0 * 12.0;
    for nome in [
        "Fibonacci Spiral",
        "Simulation Zone",
        "Four Point Warp",
        "Reroute (Value)",
        "MMMMMMMMMMMMMMMM",
    ] {
        let w = ts.prefix_width_weighted(nome, CAPSULA_FONTE, ph2d_text::FontWeight::SEMI_BOLD);
        eprintln!("  {nome:<18} {w:>8.1} de {disponivel:.1}");
    }
    let pior = "Fibonacci Spiral";
    let largura = ts.prefix_width_weighted(pior, CAPSULA_FONTE, ph2d_text::FontWeight::SEMI_BOLD);
    assert!(
        largura <= disponivel,
        "a estimativa de avanco nao e' generosa: «{pior}» mede {largura} e so' ha' {disponivel} \
         — re-derive o `AVANCO_POR_CHAR`"
    );
    // E a cápsula tem sempre a largura do cartão — só a ALTURA muda.
    assert!((CARD_W - 190.0).abs() < 1e-6);
}

/// ⭐⭐ **O NOME FICA NO CENTRO DA PASTILHA** — *«bem alinhadas no centro da cápsula»* (ordem do
/// dono, 2026-09-19). ⚠️ Medido no NÚMERO e não por um censo de texto: *um censo de texto
/// sobrevive a um `if false &&`, e esta linha já o pagou duas vezes.*
#[test]
fn o_nome_fica_no_centro_da_pastilha() {
    use crate::paint::paint_capsula::x_do_nome;
    let body = Rect::new(100.0, 50.0, 190.0, 55.0);
    for w in [10.0_f32, 80.0, 166.0] {
        let x = x_do_nome(w, body);
        assert!(
            ((x + w * 0.5) - (body.x + body.w * 0.5)).abs() < 1e-4,
            "o meio do texto tem de cair no meio da pastilha (largura {w})"
        );
    }
}
