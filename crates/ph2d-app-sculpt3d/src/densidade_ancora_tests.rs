//! **DE ONDE SAI O ALVO DO PASSE DE TOPOLOGIA** — e as duas coisas de que ele
//! NÃO pode depender.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de responsabilidade: lá *o que o
//! pincel de densidade faz*, aqui *de onde vem o número que ele persegue*.
//!
//! ⛔⛔ **Este módulo existe por uma ordem do dono** (2026-09-14): *«a densidade
//! da malha deve ser independente do zoom»*. O alvo saía do `Brush::radius`, que
//! é derivado do raio em PIXELS **através da câmera** — medido, `4,9×` de alvo
//! só por aproximar ou afastar. Hoje ele sai da **ÁREA DA SUPERFÍCIE**, que é
//! propriedade da forma e não da vista, e daí vêm de graça as **duas**
//! invariâncias que este ficheiro gateia.
//!
//! ⚠️ O arnês vive dois níveis acima e chega por `use super::*`.

use super::*;

/// ⭐⭐⭐ **A PISTA DO DETALHE CHEGA AO MOTOR — e a régua é a MALHA, nunca o
/// campo.**
///
/// ⚠️⚠️ **É o ponto cego que o §5.0 do roteador nomeia:** *nenhum instrumento
/// deste repo pergunta se o VALOR chega a um consumidor*. Um slider pode estar
/// pintado, registado, vivo sob o ponteiro e varrido pela costura — e o número
/// dele nunca sair do painel. As três metades aqui são:
///
/// 1. o painel **publica** o que a cena tem (ida);
/// 2. a cena **escreve** o que o painel mandou (volta);
/// 3. ⭐ **duas posições da pista dão malhas DIFERENTES** no mesmo gesto — a
///    única das três que prova que o número atravessou até ao motor.
///
/// ⛔ Sem a terceira, cravar o `detail` numa constante dentro do passe deixaria
/// as duas primeiras verdes.
#[test]
#[ignore]
fn a_pista_do_detalhe_chega_ao_motor() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(10, 14, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));

    // (1) IDA — o retrato publica o que a cena tem. ⚠️ **Os DOIS**: o da cena
    // (a topologia dinâmica) e o do PINCEL de densidade, que desde a ordem do
    // dono de 14/09 são campos diferentes.
    s.dyntopo.detail = 0.15;
    s.brush.density_detail = 0.35;
    let retrato = s.panel_snapshot(false, None).ui;
    assert!(
        (retrato.dyn_detail - 0.15).abs() < 1e-6,
        "o retrato não publica o detalhe da CENA — a pista nasceria a mostrar \
         outro número que o do motor"
    );
    assert!(
        (retrato.brush.density_detail - 0.35).abs() < 1e-6,
        "o retrato não publica o detalhe do PINCEL"
    );

    // (2) VOLTA — a cena escreve o que o painel mandou.
    //
    // ⚠️ **Pela PORTA DO PRODUTO** (`apply_panel_intent`), e não pelo `apply_ui`
    // privado: é este o caminho que um arrasto de pista toma, e um gate que
    // chamasse o ajudante interno afirmaria sobre código que o painel não usa —
    // a mesma armadilha que este repo já registou como *nomear a VISIBILIDADE
    // em vez da lei*.
    let mut ui = s.panel_snapshot(false, None).ui;
    ui.dyn_detail = 0.9;
    ui.brush.density_detail = 0.8;
    s.apply_panel_intent(ph2d_panel_sculpt3d::Sculpt3dIntent::SetUi(ui));
    assert!(
        (s.dyntopo.detail - 0.9).abs() < 1e-6,
        "a pista da CENA não chega ao campo dela: {} — um slider morto",
        s.dyntopo.detail
    );
    assert!(
        (s.brush.density_detail - 0.8).abs() < 1e-6,
        "a pista do PINCEL não chega ao campo dele: {} — um slider morto",
        s.brush.density_detail
    );

    // (3) ⭐ **E O NÚMERO CHEGA AO MOTOR:** o mesmo gesto, na mesma malha, com
    // duas posições da pista, tem de dar contagens DIFERENTES. É esta metade
    // que uma constante cravada no passe faria sangrar.
    let conta_com = |detalhe: f32| {
        let mut s = cena_com(
            &gpu.device,
            Verb::Density,
            ph2d_mesh::shapes::uv_sphere(10, 14, 1.0),
            detalhe,
            160.0,
        );
        um_dab(&mut s);
        vertices(&s)
    };
    let grosso = conta_com(0.0);
    let fino = conta_com(1.0);
    assert!(
        fino > grosso,
        "as duas pontas da pista dão a mesma malha ({grosso} e {fino}) — o \
         número não atravessa até ao motor"
    );
}

/// ⭐⭐⭐ **CADA GESTO LÊ O SEU PRÓPRIO SLIDER** — ordem do dono, e o gate que a
/// torna observável.
///
/// *«Deixe o slider Detail para o dynamic Retopology e coloque outro slider
/// Detail exclusivo para o pincel, nas propriedades do pincel.»* (14/09)
///
/// São **dois campos** — um na CENA (`Dyntopo::detail`, que governa o traço dos
/// outros pincéis) e um no PINCEL (`Brush::density_detail`) —, e a porta que
/// escolhe entre eles é a `Brush::offers_density_controls`, a mesma que o painel
/// consulta para oferecer a pista.
///
/// ⚠️⚠️ **A régua põe os dois em valores OPOSTOS e mede a MALHA**, que é a única
/// forma de apanhar a troca: com os dois iguais — que é como todo o resto do
/// arnês os deixa — um `if` invertido daria exactamente o mesmo resultado. *Dois
/// números iguais não distinguem duas leis.*
///
/// ⛔ E ele varre os **dois sentidos**: um gate que só medisse a densidade
/// ficaria verde se ela lesse o seu e o `Draw` lesse o dela também.
#[test]
#[ignore]
fn cada_gesto_le_o_seu_proprio_slider() {
    let gpu = gpu_or_skip!();

    // A DENSIDADE segue o slider DELA, e ignora o da cena.
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        0.5,
        160.0,
    );
    // ⚠️ O da cena vai para o EXTREMO OPOSTO: se o passe o lesse, a malha iria
    // para o outro lado e a asserção de baixo cairia.
    s.dyntopo.detail = 0.0;
    s.brush.density_detail = 1.0;
    let fino = ph2d_mesh::edge_target_for_mesh(s.mesh(), 1.0);
    assert!(
        (s.detalhe_do_gesto(&s.brush.clone()) - 1.0).abs() < 1e-6,
        "a densidade leu o slider da CENA — ela tem o próprio"
    );
    for _ in 0..8 {
        um_dab(&mut s);
    }
    let raio = s.armed_brush([0.0, 1.0, 0.0]).radius;
    let centro = s
        .pick_active(CENTRE.0, CENTRE.1)
        .map_or([0.0, 1.0, 0.0], |h| h.point);
    let m = aresta_mediana_na_esfera(&s, centro, raio);
    assert!(
        m > 0.0 && m < fino * 2.0,
        "a densidade não perseguiu o alvo do slider DELA (mediana {m:.4} contra \
         alvo {fino:.4}) — ela está a ler o número da cena"
    );

    // ⭐ **O CONTROLO, no sentido oposto:** um verbo de traço segue o slider da
    // CENA e ignora o do pincel de densidade.
    let mut s = cena_com(
        &gpu.device,
        Verb::Draw,
        ph2d_mesh::shapes::uv_sphere(10, 14, 1.0),
        0.5,
        160.0,
    );
    s.dyntopo.detail = 1.0;
    s.brush.density_detail = 0.0;
    assert!(
        (s.detalhe_do_gesto(&s.brush.clone()) - 1.0).abs() < 1e-6,
        "um verbo de traço leu o slider do pincel de densidade"
    );
    let antes = vertices(&s);
    um_dab(&mut s);
    assert!(
        vertices(&s) > antes,
        "o desenho não adensou ({antes} -> {}) — com o slider da CENA no fino \
         ele tem de partir; se ele lesse o `0,0` do outro, não partiria nada",
        vertices(&s)
    );
}

/// ⭐⭐ **E A TECLA `U` CICLA O SLIDER DO GESTO EM MÃOS — nunca o outro.**
///
/// ⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE:** cravar o `cycle_detail`
/// a escrever sempre em `self.dyntopo.detail` passava a suíte inteira. O atalho
/// mexeria num slider e o artista veria **o outro** parado — e o gesto dele não
/// mudaria de densidade nenhuma. *Um atalho que escreve no controlo errado é
/// indistinguível de um atalho morto, e nenhum gate de fiação o vê: ele está
/// ligado.*
///
/// ⚠️ **As duas metades, e nenhuma basta:** a que escreve e a que **NÃO** toca
/// no vizinho. Sem a segunda, um `cycle_detail` que escrevesse nos DOIS ficaria
/// verde — e aí mexer no atalho com um pincel na mão estragaria o ajuste do
/// outro.
#[test]
#[ignore]
fn a_tecla_do_detalhe_cicla_o_slider_do_gesto_em_maos() {
    let gpu = gpu_or_skip!();

    // Com a DENSIDADE na mão: escreve no slider dela, e o da cena fica quieto.
    let mut s = cena_com(
        &gpu.device,
        Verb::Density,
        ph2d_mesh::shapes::uv_sphere(10, 14, 1.0),
        0.5,
        160.0,
    );
    s.dyntopo.detail = 0.15;
    s.brush.density_detail = 0.5;
    s.cycle_detail();
    assert!(
        (s.brush.density_detail - 0.5).abs() > 1e-6,
        "a tecla não mexeu no slider do PINCEL com ele na mão"
    );
    assert!(
        (s.dyntopo.detail - 0.15).abs() < 1e-6,
        "a tecla mexeu no slider da CENA com a densidade na mão — ela estragou \
         o ajuste do vizinho: {}",
        s.dyntopo.detail
    );

    // ⭐ **O CONTROLO, no sentido oposto.**
    let mut s = cena_com(
        &gpu.device,
        Verb::Draw,
        ph2d_mesh::shapes::uv_sphere(10, 14, 1.0),
        0.5,
        160.0,
    );
    s.dyntopo.detail = 0.5;
    s.brush.density_detail = 0.15;
    s.cycle_detail();
    assert!(
        (s.dyntopo.detail - 0.5).abs() > 1e-6,
        "a tecla não mexeu no slider da CENA com um verbo de traço na mão"
    );
    assert!(
        (s.brush.density_detail - 0.15).abs() < 1e-6,
        "a tecla mexeu no slider do PINCEL de densidade com o Draw na mão: {}",
        s.brush.density_detail
    );
}

/// **A ARESTA MEDIANA das faces DENTRO da esfera do pincel** — a densidade que
/// o artista de facto vê onde ele passou.
///
/// ⚠️ **É a régua certa para a invariância ao zoom, e a contagem GLOBAL não
/// é:** um pincel de `160 px` cobre menos peça quando a câmera se aproxima,
/// logo a contagem final tem de variar — o que **não** pode variar é a
/// densidade *onde ele tocou*.
fn aresta_mediana_na_esfera(s: &Sculpt3dScene, centro: [f32; 3], raio: f32) -> f32 {
    let mesh = s.mesh();
    let p = mesh.positions();
    let mut arestas = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            let meio = [
                (a[0] + b[0]) * 0.5,
                (a[1] + b[1]) * 0.5,
                (a[2] + b[2]) * 0.5,
            ];
            let d2 = (meio[0] - centro[0]).powi(2)
                + (meio[1] - centro[1]).powi(2)
                + (meio[2] - centro[2]).powi(2);
            if d2 <= raio * raio {
                arestas.push(
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
                );
            }
        }
    }
    if arestas.is_empty() {
        return 0.0;
    }
    arestas.sort_by(f32::total_cmp);
    arestas[arestas.len() / 2]
}

/// **SONDA:** a densidade que sai contra o ZOOM — o report do dono de 14/09
/// (*«a densidade da malha deve ser independente do zoom»*) e a prova da cura.
#[test]
#[ignore]
fn diag_a_densidade_contra_o_zoom() {
    let gpu = gpu_or_skip!();
    for distancia in [1.5f32, 3.0, 6.0] {
        let mut s = cena_com(
            &gpu.device,
            Verb::Density,
            // ⚠️ **A malha é a FINA de propósito:** com a grossa da cena o
            // pincel a `distância 1,5` fica **menor que um triângulo** (raio de
            // mundo `0,12` contra arestas de `~0,3`) e não há o que pegar — o
            // arranjo não conteria o fenómeno em duas das três células, e o
            // gate mediria o nada. *Uma fixtura tem de conter o fenómeno em
            // TODAS as células que compara.*
            ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
            0.5,
            160.0,
        );
        s.camera.distance = distancia;
        // O raio do pincel EM MUNDO, que é por onde o zoom entrava no ALVO.
        let raio = s.armed_brush([0.0, 1.0, 0.0]).radius;
        let alvo_velho = ph2d_mesh::edge_target(raio, 0.5);
        let alvo_novo = ph2d_mesh::edge_target_for_mesh(s.mesh(), 0.5);
        for _ in 0..8 {
            um_dab(&mut s);
        }
        // O centro do dab: a MESMA porta que o `sculpt_at` usa.
        let centro = s
            .pick_active(CENTRE.0, CENTRE.1)
            .map_or([0.0, 1.0, 0.0], |h| h.point);
        let mediana = aresta_mediana_na_esfera(&s, centro, raio);
        println!(
            "distancia {distancia:>4}: raio_mundo {raio:.4} · alvo VELHO {alvo_velho:.4} · \
             alvo NOVO {alvo_novo:.4} · mediana na esfera {mediana:.4} · {} verts",
            vertices(&s)
        );
    }
}

/// ⭐⭐⭐ **A DENSIDADE É INDEPENDENTE DO ZOOM — ordem do dono, com o número.**
///
/// *«A densidade da malha deve ser independente do zoom.»* (14/09)
///
/// ⛔⛔ **O defeito era real e está medido.** O alvo de aresta saía do
/// `Brush::radius`, que é **derivado do raio em PIXELS através da câmera** a
/// cada dab. Mesma peça, mesmo pincel (`160 px`), mesmo slider (`0,5`):
///
/// | distância | raio em mundo | alvo VELHO | alvo NOVO | mediana na esfera |
/// |---|---|---|---|---|
/// | 1,5 | 0,1198 | **0,0415** | `0,0804` | `0,0601` |
/// | 3,0 | 0,2752 | 0,0953 | `0,0804` | `0,0539` |
/// | 6,0 | 0,5858 | **0,2029** | `0,0804` | `0,0543` |
///
/// ⇒ o alvo passa de **`4,9×` de dispersão a ZERO**, e a densidade **alcançada**
/// fica em **`±11 %`** — o resíduo é a discretização (uma aresta parte-se ao
/// meio ou não se parte).
///
/// ⚠️⚠️ **A régua é a densidade ONDE O PINCEL TOCOU, e a contagem GLOBAL não
/// serve:** um pincel de `160 px` cobre menos peça quando a câmera se aproxima,
/// logo a contagem final **tem** de variar — isso é o pincel a dizer ONDE, que é
/// exactamente a separação que esta cura compra. *Uma régua global mediria as
/// duas perguntas somadas e não saberia qual delas se mexeu.*
///
/// ⚠️ **A malha da fixtura é a FINA de propósito:** com a grossa da cena, a
/// `distância 1,5` o pincel fica **menor que um triângulo** e não há o que
/// pegar — o arranjo não conteria o fenómeno em todas as células.
#[test]
#[ignore]
fn a_densidade_nao_depende_do_zoom() {
    let gpu = gpu_or_skip!();
    let mut alvos = Vec::new();
    let mut medianas = Vec::new();
    for distancia in [1.5f32, 3.0, 6.0] {
        let mut s = cena_com(
            &gpu.device,
            Verb::Density,
            ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
            0.5,
            160.0,
        );
        s.camera.distance = distancia;
        alvos.push(ph2d_mesh::edge_target_for_mesh(s.mesh(), 0.5));
        let raio = s.armed_brush([0.0, 1.0, 0.0]).radius;
        for _ in 0..8 {
            um_dab(&mut s);
        }
        let centro = s
            .pick_active(CENTRE.0, CENTRE.1)
            .map_or([0.0, 1.0, 0.0], |h| h.point);
        let m = aresta_mediana_na_esfera(&s, centro, raio);
        assert!(
            m > 0.0,
            "a {distancia}: nenhuma aresta dentro da esfera — a fixtura não \
             contém o fenómeno nesta célula"
        );
        medianas.push(m);
    }

    // (1) O ALVO é o mesmo, e aqui a barra é a IGUALDADE: ele é função da área
    // e do slider, e nenhum dos dois se mexeu.
    let (lo, hi) = (
        alvos.iter().copied().fold(f32::MAX, f32::min),
        alvos.iter().copied().fold(0.0f32, f32::max),
    );
    assert_eq!(
        lo, hi,
        "o alvo variou com o zoom ({alvos:?}) — ele voltou a sair do raio do \
         pincel, que a câmera decide"
    );

    // (2) E a densidade ALCANÇADA fica na banda medida. ⚠️ A barra é `1,25` e
    // não `1,0`: o resíduo é a discretização (uma aresta parte-se ao meio ou
    // não se parte), e medido ele é `1,115`.
    let (lo, hi) = (
        medianas.iter().copied().fold(f32::MAX, f32::min),
        medianas.iter().copied().fold(0.0f32, f32::max),
    );
    assert!(
        hi / lo <= 1.25,
        "a densidade alcançada dispersou {:.3}× com o zoom ({medianas:?}) — a \
         banda medida é 1,115 e a barra 1,25",
        hi / lo
    );
}

/// ⭐⭐ **E A DENSIDADE TAMBÉM NÃO DEPENDE DO TAMANHO DA PEÇA.**
///
/// ⚠️ **É a outra metade que ancorar na ÁREA compra**, e ela é a razão de o
/// número da referência se chamar `1/(resolução · escala_do_objecto)`: o slider
/// pede uma **contagem**, logo uma peça grande e uma pequena com a mesma forma
/// saem com o mesmo número de triângulos — e não com o mesmo comprimento de
/// aresta, que faria a pequena sair grossa e a grande irreconhecível.
///
/// ⛔ **Sem este gate, cravar a área numa constante passaria despercebido:** a
/// invariância ao ZOOM continuaria verde (uma constante não depende da câmera),
/// e só a peça de outro tamanho o acusa. *Duas invariâncias parecidas, e só uma
/// delas é medida pela outra.*
#[test]
#[ignore]
fn a_densidade_nao_depende_do_tamanho_da_peca() {
    let gpu = gpu_or_skip!();
    let mut contas = Vec::new();
    for raio in [0.5f32, 1.0, 2.0] {
        let mut s = cena_com(
            &gpu.device,
            Verb::Density,
            ph2d_mesh::shapes::uv_sphere(24, 36, raio),
            0.5,
            160.0,
        );
        // ⚠️ **A câmera acompanha a peça**, senão a variável a mudar seriam
        // DUAS (o tamanho e o enquadramento) e o gate não saberia qual mediu.
        s.camera.distance = 3.0 * raio;
        let alvo = ph2d_mesh::edge_target_for_mesh(s.mesh(), 0.5);
        // O alvo é um COMPRIMENTO, logo tem de escalar com a peça: o que fica
        // invariante é a contagem, `área / alvo²`.
        contas.push(s.mesh().surface_area() / (alvo * alvo));
        for _ in 0..6 {
            um_dab(&mut s);
        }
    }
    let (lo, hi) = (
        contas.iter().copied().fold(f32::MAX, f32::min),
        contas.iter().copied().fold(0.0f32, f32::max),
    );
    assert!(
        hi / lo < 1.001,
        "a contagem pedida variou {:.4}× com o TAMANHO da peça ({contas:?}) — o \
         alvo deixou de sair da área",
        hi / lo
    );
}
