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

    // (1) IDA — o retrato publica o que a cena tem.
    s.dyntopo.detail = 0.15;
    assert!(
        (s.panel_snapshot(false).ui.dyn_detail - 0.15).abs() < 1e-6,
        "o retrato não publica o detalhe da cena — a pista nasceria a mostrar \
         outro número que o do motor"
    );

    // (2) VOLTA — a cena escreve o que o painel mandou.
    //
    // ⚠️ **Pela PORTA DO PRODUTO** (`apply_panel_intent`), e não pelo `apply_ui`
    // privado: é este o caminho que um arrasto de pista toma, e um gate que
    // chamasse o ajudante interno afirmaria sobre código que o painel não usa —
    // a mesma armadilha que este repo já registou como *nomear a VISIBILIDADE
    // em vez da lei*.
    let mut ui = s.panel_snapshot(false).ui;
    ui.dyn_detail = 0.9;
    s.apply_panel_intent(ph2d_panel_sculpt3d::Sculpt3dIntent::SetUi(ui));
    assert!(
        (s.dyntopo.detail - 0.9).abs() < 1e-6,
        "a pista não chega ao campo da cena: {} — um slider morto",
        s.dyntopo.detail
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
