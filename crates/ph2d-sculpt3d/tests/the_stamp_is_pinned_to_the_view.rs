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

/// A vista OLHANDO DE FRENTE (a tela é o plano XY), a uma régua dada.
fn front(view_units: f32) -> AlphaStencil {
    AlphaStencil {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        view_units,
    }
}

/// A MESMA vista depois de um quarto de volta da câmera em torno de `+Y`: o que
/// era `+X` da tela agora é `−Z` do objeto.
fn turned(view_units: f32) -> AlphaStencil {
    AlphaStencil {
        right: [0.0, 0.0, -1.0],
        up: [0.0, 1.0, 0.0],
        view_units,
    }
}

fn brush_with(stencil: Option<AlphaStencil>, alpha: Alpha, stamp_scale: f32) -> Brush {
    Brush {
        alpha: Some(alpha),
        alpha_stencil: stencil,
        alpha_stencil_scale: stamp_scale,
        ..Brush::default()
    }
}

/// Os pesos que uma GRADE DE TELA recebe — o que o artista de fato vê.
///
/// ⚠️ **A amostragem é em coordenadas de TELA e depois levada ao objeto pela
/// própria base do estêncil**, e é isso que torna o oráculo sobre *o que aparece*
/// em vez de sobre *que números o kernel produz*: girar a câmera gira a grade
/// junto, então um padrão colado ao barro muda o que se lê e um preso à tela não.
fn screen_field(b: &Brush, s: &AlphaStencil) -> Vec<f32> {
    let f = b.alpha_frame();
    // ⚠️ **A grade é FINA de propósito.** Com 24 amostras ela cai no limite de
    // Nyquist das bandas do carimbo, e o gate de densidade mediu 2× onde a
    // geometria dá 4 — um oráculo sub-amostrado reporta o próprio limite, não o
    // produto.
    let n = 96usize;
    let mut out = Vec::with_capacity(n * n);
    for row in 0..n {
        for col in 0..n {
            // Meia tela para cada lado, na régua da vista.
            let u = (col as f32 / n as f32 - 0.5) * s.view_units;
            let v = (row as f32 / n as f32 - 0.5) * s.view_units;
            let p = [
                s.right[0] * u + s.up[0] * v,
                s.right[1] * u + s.up[1] * v,
                s.right[2] * u + s.up[2] * v,
            ];
            out.push(b.alpha_weight(p, &f));
        }
    }
    out
}

/// **GIRAR O OBJETO NÃO MOVE O CARIMBO** — a primeira metade do pedido.
///
/// ⚠️ **O oráculo é a IMAGEM NA TELA, e não os pesos de vértices fixos.** Um
/// padrão colado ao barro dá pesos *iguais* nos mesmos vértices quando a câmera
/// gira — é a tela que muda. Amostrando a tela, um estêncil sai IDÊNTICO e um
/// campo de objeto sai outro; a comparação certa é essa.
#[test]
fn turning_the_object_does_not_move_the_stamp() {
    let (front_v, turned_v) = (front(2.0), turned(2.0));
    let a = brush_with(Some(front_v), stamp(), 0.5);
    let b = brush_with(Some(turned_v), stamp(), 0.5);

    let before = screen_field(&a, &front_v);
    let after = screen_field(&b, &turned_v);
    assert_eq!(
        before, after,
        "a câmera girou e o carimbo foi junto — ele não está preso ao viewport"
    );

    // ⚠️ **O CONTROLE, sem o qual o gate acima é verde por vácuo:** um campo de
    // OBJETO tem de mudar sob a mesma órbita. Sem esta metade, um `screen_field`
    // que devolvesse uma constante passaria pela primeira.
    let ground = brush_with(None, Alpha::Strata, 0.5);
    let g0 = screen_field(&ground, &front_v);
    let g1 = screen_field(&ground, &turned_v);
    assert_ne!(
        g0, g1,
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
    let (far, near) = (front(4.0), front(1.0));
    let a = brush_with(Some(far), stamp(), 0.5);
    let b = brush_with(Some(near), stamp(), 0.5);
    assert_eq!(
        screen_field(&a, &far),
        screen_field(&b, &near),
        "o carimbo mudou de tamanho com o zoom — a régua dele não é a da vista"
    );

    // O controle: um padrão de OBJETO fica maior na tela quando a câmera
    // aproxima, que é o comportamento que este modo existe para NÃO ter.
    let ground = brush_with(None, Alpha::Strata, 0.5);
    assert_ne!(
        screen_field(&ground, &far),
        screen_field(&ground, &near),
        "um padrão de objeto não respondeu ao zoom: o oráculo está cego"
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
    let v = front(2.0);
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
    let v = front(2.0);
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
