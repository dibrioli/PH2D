//! **UM CARIMBO É UMA FOLHA NA FRENTE DA TELA** — os gates do modo de projeção.
//!
//! Pedido do Enio (2026-08-09): *"é melhor que a projeção da imagem externa (e
//! apenas dela) seja screen — projeção frontal que não muda com a rotação do
//! objeto. Se der zoom no objeto, a textura permanece fixa como no início; ou
//! seja, relativamente muda ao esculpir"*.
//!
//! É o estêncil do ZBrush e o *View Plane* do Blender: o padrão fica preso ao
//! viewport, e o barro passa por baixo dele.
//!
//! ⚠️ **Os gates medem o CAMPO, não a câmera.** Uma órbita é, para o motor,
//! outra base `right`/`up` — e um zoom é outra régua de vista. Quem monta essas
//! duas coisas a partir de uma `Camera3d` é a cena, e o gate dela mora no shell;
//! aqui se pergunta o que o padrão faz quando elas mudam, que é a parte que
//! decide o que o artista vê.

use ph2d_sculpt3d::{Alpha, AlphaImage, AlphaStencil, Brush};

/// Bandas diagonais — um campo estruturado nos DOIS eixos, senão metade de um
/// deslocamento concorda consigo mesma.
fn stamp() -> Alpha {
    let n = 32u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let v = u8::from((x + y) % 8 < 4) * 255;
            let i = ((y * n + x) * 4) as usize;
            rgba[i..i + 3].fill(v);
            rgba[i + 3] = 255;
        }
    }
    Alpha::Image(std::sync::Arc::new(
        AlphaImage::from_rgba(n, n, &rgba).expect("imagem válida"),
    ))
}

/// A razão do frustum de uma câmera de 45° — a mesma da cena do smoke.
const HPD: f32 = 0.828_427;

/// Uma vista montada como a cena a monta: o olho a `dist` do alvo, ao longo do
/// eixo que a própria base da tela define.
///
/// ⚠️ **O olho sai de `right × up`, e não de um terceiro vetor passado ao lado** —
/// pela mesma razão que o `AlphaFrame::stencil` deriva o eixo da base: um olho
/// escrito à mão poderia DISCORDAR da tela que ele diz observar, e o gate ficaria
/// verde sobre um frustum torto.
fn view(right: [f32; 3], up: [f32; 3], dist: f32) -> AlphaStencil {
    let n = [
        right[1] * up[2] - right[2] * up[1],
        right[2] * up[0] - right[0] * up[2],
        right[0] * up[1] - right[1] * up[0],
    ];
    AlphaStencil {
        right,
        up,
        eye: [n[0] * dist, n[1] * dist, n[2] * dist],
        height_per_depth: HPD,
    }
}

/// A vista OLHANDO DE FRENTE (a tela é o plano XY), com o olho a `dist`.
fn front(dist: f32) -> AlphaStencil {
    view([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], dist)
}

/// A MESMA vista depois de um quarto de volta da câmera em torno de `+Y`: o que
/// era `+X` da tela agora é `−Z` do objeto.
fn turned(dist: f32) -> AlphaStencil {
    view([0.0, 0.0, -1.0], [0.0, 1.0, 0.0], dist)
}

fn brush_with(stencil: Option<AlphaStencil>, alpha: Alpha, stamp_scale: f32) -> Brush {
    Brush {
        alpha: Some(alpha),
        alpha_stencil: stencil,
        alpha_stencil_scale: stamp_scale,
        ..Brush::default()
    }
}

/// Os pesos que uma GRADE DE TELA recebe, numa profundidade dada — o que o
/// artista de fato vê.
///
/// ⚠️ **A amostragem é em coordenadas de TELA e depois levada ao objeto pelo
/// próprio frustum do estêncil**, e é isso que torna o oráculo sobre *o que
/// aparece* em vez de sobre *que números o kernel produz*: girar a câmera gira a
/// grade junto, então um padrão colado ao barro muda o que se lê e um preso à
/// tela não.
///
/// ⚠️ **E a PROFUNDIDADE é um parâmetro, e não uma constante escondida** — é ela
/// que distingue uma folha presa à tela de um plano colado ao barro na
/// profundidade em que alguém escolheu medir.
fn screen_field_at(b: &Brush, s: &AlphaStencil, depth: f32) -> Vec<f32> {
    let f = b.alpha_frame();
    // O eixo da vista: o olho está a `|eye|` dele, e a cena fica do lado oposto.
    let n = [
        s.right[1] * s.up[2] - s.right[2] * s.up[1],
        s.right[2] * s.up[0] - s.right[0] * s.up[2],
        s.right[0] * s.up[1] - s.right[1] * s.up[0],
    ];
    // ⚠️ **A grade é FINA de propósito.** Com 24 amostras ela cai no limite de
    // Nyquist das bandas do carimbo, e o gate de densidade mediu 2× onde a
    // geometria dá 4 — um oráculo sub-amostrado reporta o próprio limite, não o
    // produto.
    let g = 96usize;
    // Quanto mundo a tela abrange NESTA profundidade — a régua que o frustum dá.
    let span = depth * s.height_per_depth;
    let mut out = Vec::with_capacity(g * g);
    for row in 0..g {
        for col in 0..g {
            // Meia tela para cada lado, na régua desta profundidade.
            let u = (col as f32 / g as f32 - 0.5) * span;
            let v = (row as f32 / g as f32 - 0.5) * span;
            let p = [
                s.eye[0] - n[0] * depth + s.right[0] * u + s.up[0] * v,
                s.eye[1] - n[1] * depth + s.right[1] * u + s.up[1] * v,
                s.eye[2] - n[2] * depth + s.right[2] * u + s.up[2] * v,
            ];
            out.push(b.alpha_weight(p, &f));
        }
    }
    out
}

/// A grade no plano do ALVO — a profundidade em que o modelo está enquadrado.
fn screen_field(b: &Brush, s: &AlphaStencil) -> Vec<f32> {
    let dist = (s.eye[0] * s.eye[0] + s.eye[1] * s.eye[1] + s.eye[2] * s.eye[2]).sqrt();
    screen_field_at(b, s, dist)
}

/// O pior desvio entre dois campos — o oráculo de *"é a mesma imagem?"*.
fn max_delta(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()))
}

/// **A barra que separa ARREDONDAMENTO de um carimbo que NADA**, e ela é MEDIDA.
///
/// Uma mesma fração de tela alcançada por caminhos aritméticos diferentes (uma
/// profundidade multiplica, a divisão de perspectiva devolve) não volta bit a
/// bit: sobra ruído na última casa da coordenada, que o ladrilho amplia pelo
/// número de texels. Medido entre as profundidades 4 e 6: **9,85e-6**.
///
/// ⚠️ **E a barra só significa alguma coisa por causa da distância até o
/// defeito**, que também é medida: com a régua presa a UMA âncora (o modelo
/// anterior, reinstalado como mutação) o mesmo par de profundidades desvia
/// **1,0** — o campo inteiro, de preto a branco, e a mesma mutação derruba
/// junto o gate do zoom. São **cinco ordens de grandeza** entre o ruído e o
/// defeito, então nenhuma escolha razoável de barra confunde uma com a outra.
const SCREEN_EPS: f32 = 1e-3;

/// **GIRAR O OBJETO NÃO MOVE O CARIMBO** — a primeira metade do pedido.
///
/// ⚠️ **O oráculo é a IMAGEM NA TELA, e não os pesos de vértices fixos.** Um
/// padrão colado ao barro dá pesos *iguais* nos mesmos vértices quando a câmera
/// gira — é a tela que muda. Amostrando a tela, um estêncil sai IDÊNTICO e um
/// campo de objeto sai outro; a comparação certa é essa.
#[test]
fn turning_the_object_does_not_move_the_stamp() {
    let (front_v, turned_v) = (front(5.0), turned(5.0));
    let a = brush_with(Some(front_v), stamp(), 0.5);
    let b = brush_with(Some(turned_v), stamp(), 0.5);

    let before = screen_field(&a, &front_v);
    let after = screen_field(&b, &turned_v);
    let d = max_delta(&before, &after);
    assert!(
        d <= SCREEN_EPS,
        "a câmera girou e o carimbo foi junto ({d:.4} de desvio) — ele não está \
         preso ao viewport"
    );

    // ⚠️ **O CONTROLE, sem o qual o gate acima é verde por vácuo:** um campo de
    // OBJETO tem de mudar sob a mesma órbita. Sem esta metade, um `screen_field`
    // que devolvesse uma constante passaria pela primeira.
    let ground = brush_with(None, Alpha::Strata, 0.5);
    let g0 = screen_field(&ground, &front_v);
    let g1 = screen_field(&ground, &turned_v);
    assert!(
        max_delta(&g0, &g1) > SCREEN_EPS,
        "um padrão de OBJETO não mudou com a órbita: o oráculo não está olhando \
         para a tela"
    );
}

/// **O ZOOM NÃO REDIMENSIONA O CARIMBO** — a segunda metade do pedido.
///
/// Aproximar reduz quantas unidades de objeto cabem na tela; um carimbo medido
/// em fração da tela encolhe na mesma proporção em unidades de objeto e fica do
/// mesmo tamanho para quem olha. *Relativamente ao modelo ele muda* — que é o
/// que o pedido diz, e o que um estêncil é.
#[test]
fn zooming_does_not_resize_the_stamp() {
    let (far, near) = (front(8.0), front(2.0));
    let a = brush_with(Some(far), stamp(), 0.5);
    let b = brush_with(Some(near), stamp(), 0.5);
    let d = max_delta(&screen_field(&a, &far), &screen_field(&b, &near));
    assert!(
        d <= SCREEN_EPS,
        "o carimbo mudou de tamanho com o zoom ({d:.4} de desvio) — a régua dele \
         não é a da vista"
    );

    // O controle: um padrão de OBJETO fica maior na tela quando a câmera
    // aproxima, que é o comportamento que este modo existe para NÃO ter.
    let ground = brush_with(None, Alpha::Strata, 0.5);
    assert!(
        max_delta(&screen_field(&ground, &far), &screen_field(&ground, &near)) > SCREEN_EPS,
        "um padrão de objeto não respondeu ao zoom: o oráculo está cego"
    );
}

/// **O CARIMBO NÃO NADA COM A PROFUNDIDADE** — a propriedade que faz da âncora
/// uma pergunta sem sentido, e o gate que fechou o report de 2026-08-09.
///
/// O defeito: a régua da vista era medida em UM ponto, e os dois consumidores
/// escolhiam pontos diferentes — o dab no acerto, o preview no centro da peça.
/// Medido na cena do smoke (`measure_the_view_ruler_at_two_anchors`), a régua
/// vale **3,35 na frente do modelo e 4,17 no centro**: o carimbo desenhado no
/// barro saía **24,8% maior** que o depositado, e mudava de erro conforme o
/// artista andava pela peça.
///
/// ⚠️ **A cura não foi escolher a âncora certa — foi tirar a pergunta.** Um
/// carimbo é uma folha na frente da TELA, então dois pontos que caem no mesmo
/// pixel têm de receber a mesma coordenada de carimbo, estejam eles no nariz ou
/// na nuca do modelo. Nenhuma âncora consegue isso: ela lineariza a perspectiva
/// numa profundidade e erra em todas as outras.
///
/// ⚠️ **O CONTROLE está na segunda metade** — um padrão de OBJETO *tem* de mudar
/// entre as duas profundidades, senão um `screen_field_at` que ignorasse o
/// parâmetro passaria pela primeira sem olhar para nada.
#[test]
fn the_stamp_does_not_swim_with_depth() {
    let v = front(5.0);
    let b = brush_with(Some(v), stamp(), 0.25);

    // As duas pontas do que uma peça enquadrada ocupa: o nariz e a nuca.
    let near = screen_field_at(&b, &v, 4.0);
    let far = screen_field_at(&b, &v, 6.0);
    let d = max_delta(&near, &far);
    assert!(
        d <= SCREEN_EPS,
        "o mesmo pixel mostrou carimbos diferentes conforme a profundidade do \
         barro sob ele ({d:.4} de desvio) — o carimbo não está preso à tela, \
         está colado num plano"
    );

    let ground = brush_with(None, Alpha::Strata, 0.25);
    assert!(
        max_delta(
            &screen_field_at(&ground, &v, 4.0),
            &screen_field_at(&ground, &v, 6.0)
        ) > SCREEN_EPS,
        "um padrão de OBJETO não mudou entre duas profundidades: o oráculo não \
         está amostrando o que diz amostrar"
    );
}

/// **UM DESLOCAMENTO DE UM CARIMBO ANDA EXATAMENTE UM LADRILHO** — a régua do
/// `Stamp Offset`, afirmada onde a razão do frustum NÃO é 1.
///
/// ⚠️ **Este gate existe porque as outras fixtures de deslocamento vivem no
/// retrato do painel, onde a razão vale exatamente `1,0`** — e ali toda conversão
/// espúria (multiplicar por uma régua, por uma profundidade, pela razão) é a
/// identidade. Uma "conversão" reintroduzida passaria despercebida em todas
/// elas e pousaria o carimbo no lugar errado só no barro, que é onde o artista
/// olha.
///
/// O oráculo é a PERIODICIDADE: o padrão ladrilha com período 1, então deslocar
/// de um ladrilho inteiro tem de devolver **o mesmo campo**. É um oráculo que
/// não conhece a fórmula — só o que o artista vê.
#[test]
fn an_offset_of_one_stamp_walks_exactly_one_tile() {
    let v = front(5.0);
    let scale = 0.25;
    let still = brush_with(Some(v), stamp(), scale);
    let walked = Brush {
        // Um ladrilho inteiro, na régua em que o deslocamento é autorado.
        alpha_offset: [scale, 0.0],
        ..brush_with(Some(v), stamp(), scale)
    };
    let d = max_delta(&screen_field(&still, &v), &screen_field(&walked, &v));
    assert!(
        d <= SCREEN_EPS,
        "deslocar de um carimbo inteiro não devolveu o mesmo campo ({d:.4} de \
         desvio) — o deslocamento não está na régua da vista"
    );

    // ⚠️ **O CONTROLE anda UM OITAVO de ladrilho, e o número não é gosto:** as
    // bandas deste carimbo repetem a cada 8 dos 32 texels, ou seja **um quarto
    // de ladrilho** — então meio ladrilho são dois períodos e devolve o mesmo
    // campo. O primeiro corte deste gate usava meio ladrilho e reprovou produto
    // CORRETO por isso: *um controle tem de andar menos que o período do que ele
    // controla*.
    let nudged = Brush {
        alpha_offset: [scale * 0.125, 0.0],
        ..brush_with(Some(v), stamp(), scale)
    };
    assert!(
        max_delta(&screen_field(&still, &v), &screen_field(&nudged, &v)) > SCREEN_EPS,
        "um oitavo de carimbo de deslocamento não moveu nada: o controle não \
         está exercitando o deslocamento"
    );
}

/// **E O TAMANHO AINDA É UM CONTROLE** — o carimbo é imune ao zoom, não ao
/// artista.
///
/// ⚠️ Ele existe porque a cura óbvia para os dois gates acima — ignorar a escala
/// — passaria nos dois. E ele mede DENSIDADE (transições ao longo de uma linha),
/// não bytes: *"mudou"* é o que uma imagem faz quando qualquer coisa muda.
#[test]
fn the_stamp_size_still_governs_how_big_it_lands() {
    let v = front(5.0);
    let crossings = |scale: f32| {
        let f = screen_field(&brush_with(Some(v), stamp(), scale), &v);
        f.windows(2)
            .filter(|w| (w[0] < 0.5) != (w[1] < 0.5))
            .count()
    };
    let (big, small) = (crossings(1.0), crossings(0.25));
    assert!(
        small > big * 2,
        "um carimbo 4× menor desenhou {small} transições contra {big} — a pista \
         de tamanho não governa o estêncil"
    );
}

/// **OS NOVE PROCEDURAIS NÃO SÃO TOCADOS, AO BIT** — *"e apenas dela"*, no
/// pedido do Enio, é uma afirmação verificável.
///
/// ⚠️ **O gate passa a vista aos dois lados**, e é isso que o torna forte: não
/// basta que um procedural sem estêncil não mude — ele tem de ignorar um
/// estêncil que ESTÁ ali. É a mesma neutralidade por construção que o
/// deslocamento já tinha, agora estendida ao modo inteiro.
#[test]
fn the_nine_procedurals_never_see_the_view() {
    let v = front(5.0);
    for a in Alpha::ALL.iter() {
        let quiet = brush_with(None, a.clone(), 0.5);
        let with_view = brush_with(Some(v), a.clone(), 0.5);
        assert_eq!(
            with_view.alpha_scale_resolved(),
            quiet.alpha_scale_resolved(),
            "{}: a régua da vista alcançou um padrão de objeto",
            a.label()
        );
        assert_eq!(
            with_view.alpha_frame(),
            quiet.alpha_frame(),
            "{}: o frame de um padrão de objeto veio da tela",
            a.label()
        );
    }
}

/// **SEM VISTA, UMA IMAGEM É O QUE ERA** — o caminho de compatibilidade dito em
/// vez de assumido.
///
/// Nem todo consumidor tem câmera (o retrato do painel arma a vista CANÔNICA;
/// um gate de kernel não arma nenhuma), e `None` tem de devolver exatamente o
/// frame autorado — senão esta wave teria mudado o significado de todo teste que
/// já existia.
#[test]
fn without_a_view_an_image_reads_as_it_always_did() {
    let b = Brush {
        alpha_az_deg: 37,
        alpha_elev_deg: 11,
        ..brush_with(None, stamp(), 0.5)
    };
    let authored = Brush {
        alpha: None,
        ..b.clone()
    };
    assert_eq!(
        b.alpha_frame(),
        authored.alpha_frame(),
        "sem vista o frame de uma imagem deixou de ser o autorado"
    );
    assert!(
        (b.alpha_scale_resolved() - b.alpha_scale).abs() < f32::EPSILON,
        "sem vista a escala de uma imagem deixou de ser o campo cru"
    );
}
