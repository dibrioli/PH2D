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
    // assert!(CAPSULA_FONTE > 2.0 * TITLE_SIZE)` ao lado dela recusa um corpo que não seja o que
    // o dono pediu. *O clippy recusa um `assert!` de teste sobre duas constantes — ele é dobrado
    // antes de correr —, e uma cerca que o compilador impõe é mais forte que um gate.*
    //
    // ⛔⛔ **E a metade que este gate media MORREU em 2026-09-20**, com a ordem *«aumenta a
    // largura do retângulo conforme o tamanho do nome»*: ele exigia que o pior nome coubesse na
    // largura do CARTÃO e que o corpo fosse MÁXIMO para ela — *as duas perguntas eram sobre uma
    // largura fixa, e a largura deixou de ser fixa*. O que sobra a medir é a **estimativa**, que
    // é o único caminho por onde um nome ainda pode sair cortado (quando ninguém mediu).
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for nome in [
        "Fibonacci Spiral",
        "Simulation Zone",
        "ADSR",
        "Reroute (Value)",
        "MMMMMMMMMMMMMMMM",
    ] {
        let w = ts.prefix_width_weighted(nome, CAPSULA_FONTE, ph2d_text::FontWeight::SEMI_BOLD);
        let est = geom::estimativa_da_largura(nome);
        eprintln!("  {nome:<18} texto {w:>7.1}  estimativa {est:>7.1}");
        assert!(
            w <= est,
            "a estimativa de «{nome}» ({est:.1}) e' mais ESTREITA que o texto ({w:.1}) — num \
             quadro sem medida o nome sai CORTADO"
        );
    }
}

/// ⭐⭐⭐ **O CANTO É O DO CABEÇALHO — um RECTÂNGULO, não uma pastilha.**
///
/// Ordem do dono (2026-09-20): *«no lugar das cápsulas os retângulos como nos headers dos nós, só
/// que grandes»*. ⚠️ **As duas metades reprovam por motivos diferentes:** a primeira é a
/// IDENTIDADE (o mesmo raio do cabeçalho, em qualquer zoom — um número igual escrito à mão
/// passaria hoje e divergiria no dia em que o cartão mudasse de raio); a segunda é a FORMA (um
/// raio de meia altura é a definição de pastilha, e é o que estava lá).
#[test]
fn o_canto_da_capsula_e_o_do_cabecalho_e_nao_o_de_uma_pastilha() {
    use crate::paint::CARD_RADIUS;
    use crate::paint::paint_capsula::raio_do_canto;

    for zoom in [ZOOM_DA_CAPSULA, 0.5, 0.2] {
        let v = vista(zoom);
        let r = raio_do_canto(&v);
        assert!(
            (r - CARD_RADIUS * zoom).abs() < 1e-6,
            "o canto tem de ser o MESMO do cabecalho a zoom {zoom}: {r} contra {}",
            CARD_RADIUS * zoom
        );
        // A metade da FORMA: uma pastilha teria `altura / 2`.
        let pastilha = capsula_h(&no(1, 1)) * zoom * 0.5;
        assert!(
            r < pastilha * 0.5,
            "a zoom {zoom} o canto {r} esta' na ordem da meia-altura {pastilha} — isto voltou a \
             ser uma CAPSULA"
        );
    }
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

/// ⭐⭐⭐ **A LARGURA SEGUE O NOME — e um nome curto NÃO encolhe a pastilha.**
///
/// Ordem do dono (2026-09-20): *«aumenta a largura do retângulo conforme o tamanho do nome»*.
/// ⚠️ **As três metades reprovam por motivos diferentes:** o PISO (um nome curto fica na largura
/// do cartão, senão voltam os *«tamanhos irregulares»* de 19/09), o CRESCIMENTO (um nome comprido
/// alarga, e o texto cabe com a margem dos dois lados) e a INÉRCIA no regime do cartão (de perto
/// nada disto acontece).
#[test]
fn a_largura_da_capsula_segue_o_nome_com_piso_no_cartao() {
    crate::capsula_larguras::esquece();
    let mut curto = no(1, 1);
    curto.display_name = "FK".into();
    let mut longo = no(1, 1);
    longo.display_name = "Simulation Zone".into();
    // As medidas que o produto teria — pregadas, porque um gate não tem medidor de texto.
    crate::capsula_larguras::prega("FK", 40.0);
    crate::capsula_larguras::prega("Simulation Zone", 208.5);

    let v = vista(0.5);
    assert_eq!(geom::detalhe(&v), Detalhe::Capsula);

    // (1) O PISO: um nome curto não encolhe nada.
    let (_, w_curto) = geom::card_x_w_at(&curto, &v);
    assert!(
        (w_curto - CARD_W).abs() < 1e-6,
        "um nome curto tem de ficar na largura do cartao: {w_curto} contra {CARD_W}"
    );

    // (2) O CRESCIMENTO: o texto cabe, com a margem dos dois lados.
    let (_, w_longo) = geom::card_x_w_at(&longo, &v);
    assert!(
        w_longo >= 208.5 + 2.0 * geom::MARGEM_X_DA_CAPSULA,
        "a pastilha ({w_longo}) tem de caber o texto (208,5) mais as duas margens"
    );
    assert!(
        w_longo > CARD_W,
        "e tem de CRESCER: {w_longo} contra {CARD_W}"
    );

    // (3) A INÉRCIA: de perto, o cartão é o cartão.
    let perto = vista(1.0);
    assert_eq!(geom::detalhe(&perto), Detalhe::Completo);
    for n in [&curto, &longo] {
        let (x, w) = geom::card_x_w_at(n, &perto);
        assert!(
            (w - CARD_W).abs() < 1e-6 && (x - n.x).abs() < 1e-6,
            "no regime do cartao nada se mexe: ({x}, {w})"
        );
    }
    crate::capsula_larguras::esquece();
}

/// ⭐⭐ **A PASTILHA CRESCE PARA OS DOIS LADOS** — o centro do cartão é preservado.
///
/// ⚠️ **Medido, não escolhido:** com `DX = 220` e o cartão a `190` sobram `30` unidades entre dois
/// nós encadeados; crescer só para a direita põe as `42` da pior pastilha num vão só (`−12`, e
/// ela cobre o vizinho), centrada cada lado leva `21`. FALSIFICADO por devolver `n.x` no braço
/// da cápsula.
#[test]
fn a_capsula_larga_cresce_para_os_dois_lados() {
    crate::capsula_larguras::esquece();
    let mut longo = no(1, 1);
    longo.display_name = "Simulation Zone".into();
    crate::capsula_larguras::prega("Simulation Zone", 208.5);

    let v = vista(0.5);
    let (x, w) = geom::card_x_w_at(&longo, &v);
    let centro_do_cartao = longo.x + CARD_W * 0.5;
    assert!(
        ((x + w * 0.5) - centro_do_cartao).abs() < 1e-4,
        "o centro tem de ficar onde o do cartao esta': {} contra {centro_do_cartao}",
        x + w * 0.5
    );
    assert!(x < longo.x, "e ela tem de comecar a' ESQUERDA do cartao");
    crate::capsula_larguras::esquece();
}

/// ⭐⭐⭐ **O PINO ATERRA NA BORDA DA FORMA, não na do cartão.**
///
/// ⛔ Sem isto o pino de saída de uma pastilha larga fica **dentro** dela e o fio aterra no meio
/// do nome — e nenhum dos gates de geometria que já existiam o via, porque todos mediam o `y`.
/// FALSIFICADO por voltar a `n.x + CARD_W` no `socket_center`.
#[test]
fn o_pino_aterra_na_borda_da_capsula_larga() {
    crate::capsula_larguras::esquece();
    let mut longo = no(1, 1);
    longo.display_name = "Simulation Zone".into();
    crate::capsula_larguras::prega("Simulation Zone", 208.5);

    let v = vista(0.5);
    let corpo = geom::card_rect(&longo, &v);
    let (sx, _) = geom::socket_center(&longo, &v, true, 0);
    let (ex, _) = geom::socket_center(&longo, &v, false, 0);
    assert!(
        (sx - (corpo.x + corpo.w)).abs() < 1e-3,
        "a saida tem de cair na borda DIREITA da pastilha: {sx} contra {}",
        corpo.x + corpo.w
    );
    assert!(
        (ex - corpo.x).abs() < 1e-3,
        "a entrada tem de cair na borda ESQUERDA: {ex} contra {}",
        corpo.x
    );
    // O CONTROLO: numa pastilha que não cresceu, as bordas são as do cartão de sempre.
    let mut curto = no(1, 1);
    curto.display_name = "FK".into();
    crate::capsula_larguras::prega("FK", 40.0);
    let (cx, _) = geom::socket_center(&curto, &v, true, 0);
    let (esperado, _) = v.pt(curto.x + CARD_W, 0.0);
    assert!((cx - esperado).abs() < 1e-3, "{cx} contra {esperado}");
    crate::capsula_larguras::esquece();
}
