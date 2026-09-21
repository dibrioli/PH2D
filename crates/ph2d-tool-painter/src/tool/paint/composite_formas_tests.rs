//! **A pilha do Composite contra os métodos de RE-CARIMBO** — os gates do report do dono de
//! 2026-09-21 (*"os Stroke:Method vivos (booleanos) — Ellipse, Polygon, Line — não funcionam
//! corretamente com o composite, mudam de aparência e o Boolean não funciona"* + *"undo/redo …
//! podem deixar resíduos"*).
//!
//! ⚠️ **Os dois reports são UM defeito**, e a lei está no doc de
//! [`super::composite_acumulado::PainterTool::restamp_reset_pilha`]: *um re-carimbo descasca a
//! TELA, logo ele tem de descascar também os ACUMULADORES da pilha*. Aqui medem-se as duas
//! consequências que o dono vê — o arco que o boolean tinha de apagar, e o rasto de uma figura
//! arrastada.

use super::composite::CompositeOp;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const S: u32 = 256;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma tela branca com o método **Ellipse** na mão e a pilha pedida, **de cima para baixo**.
///
/// ⚠️ A ordem é ingrediente: [`PainterTool::acrescenta_camada`] põe a camada nova no FUNDO, logo a
/// PRIMEIRA criada é a de CIMA — uma pilha `Brush` + `Blur` nesta ordem tem o borrão por baixo da
/// tinta, onde ele borra a tela em branco e não faz nada.
fn cena(camadas: &[(CompositeOp, f32)]) -> PainterTool {
    cena_na_rota(camadas, false)
}

/// A mesma cena, com a ROTA escolhida: `replay = true` é a de bissecção
/// (`PH2D_COMPOSITE_REPLAY=1`), que guarda a história em `lotes` e tem a **mesma doença**.
/// ⚠️ A escolha é um CAMPO e não a variável de ambiente — *um gate que lê o ambiente mede a
/// máquina*.
fn cena_na_rota(camadas: &[(CompositeOp, f32)], replay: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    t.paint.brush.radius_px = 3.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.paint.composite_enabled = !camadas.is_empty();
    t.paint.pilha_por_replay = replay;
    for (op, s) in camadas {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, *s);
    }
    t
}

/// Desenha uma elipse centrada em `c` de raio `r` pelo caminho do PRODUTO (um pen-down no centro,
/// arrasto até ao raio, pen-up). ⚠️ Sob a mão a figura é um RASCUNHO — ela só é carimbada no
/// pen-up —, logo o descascar que esta wave cura é o do ramo do rascunho.
fn elipse(t: &mut PainterTool, c: [f32; 2], r: f32) {
    t.on_canvas_pointer(cp(c, PointerPhase::Down));
    for i in 1..=4 {
        let u = i as f32 / 4.0;
        t.on_canvas_pointer(cp([c[0] + r * u, c[1]], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([c[0] + r, c[1]], PointerPhase::Up));
}

fn tinta(t: &PainterTool) -> usize {
    (0..(S * S) as usize)
        .filter(|&i| t.canvas_rgba[i * 4] < 250)
        .count()
}

/// O texel mais escuro numa janela 5×5 à volta de `p`.
fn mais_escuro(t: &PainterTool, p: [f32; 2]) -> u8 {
    let mut pior = 255u8;
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let x = (p[0] as i32 + dx).clamp(0, S as i32 - 1) as u32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let y = (p[1] as i32 + dy).clamp(0, S as i32 - 1) as u32;
            pior = pior.min(t.canvas_rgba[((y * S + x) * 4) as usize]);
        }
    }
    pior
}

/// As pilhas que este ficheiro mede — **`Brush` sozinho fica de fora de propósito**: com menos de
/// duas camadas activas o carimbo nem abre a pilha (`camadas_activas() < 2`), logo ele não pode
/// reproduzir o defeito e um gate que o incluísse mediria o caminho de sempre.
fn pilhas() -> [(&'static str, Vec<(CompositeOp, f32)>); 4] {
    use CompositeOp::{Blur, Brush, Erase, Smear};
    [
        ("Blur/Brush", vec![(Blur, 1.0), (Brush, 1.0)]),
        ("Smear/Brush", vec![(Smear, 1.0), (Brush, 1.0)]),
        ("Brush/Brush", vec![(Brush, 0.5), (Brush, 0.5)]),
        ("Erase/Brush", vec![(Erase, 0.4), (Brush, 1.0)]),
    ]
}

/// ⭐ **O BOOLEAN continua a apagar o arco interior com a pilha montada.**
///
/// Duas circunferências `Add` que se cruzam: o contorno pintado é o da UNIÃO, logo o pedaço do
/// contorno de uma que cai **dentro** da outra tem de ficar branco. Com os acumuladores por
/// descascar, o carimbo da 1.ª figura sobrevivia nos planos e a composição repunha-o.
///
/// **Medido antes da cura** (o branco é `255`): `Blur/Brush` **`26`** · `Smear/Brush` **`0`** ·
/// `Brush/Brush` **`63`** · `Erase/Brush` **`0`**; sem pilha, `255`.
///
/// ⚠️ **O CONTROLO é a metade que torna isto uma afirmação:** com as mesmas duas figuras em
/// `Overlay` (sem boolean) aquele ponto é **PINTADO** — sem ele, um gate que só exigisse branco
/// passaria numa fixtura em que as figuras nem se cruzam.
///
/// **Mutação que sangra:** tirar o `restamp_reset_pilha()` do `peel_drag_preview`.
#[test]
fn o_boolean_apaga_o_arco_interior_com_a_pilha_montada() {
    let (c1, c2, r) = ([100.0f32, 128.0], [156.0f32, 128.0], 45.0f32);
    let ang = 20.0f32.to_radians();
    let interior = [c1[0] + r * ang.cos(), c1[1] + r * ang.sin()];

    // CONTROLO: sem boolean (Overlay) o contorno inteiro é pintado, logo o ponto é escuro.
    let mut ctl = cena(&[]);
    elipse(&mut ctl, c1, r);
    elipse(&mut ctl, c2, r);
    assert!(
        mais_escuro(&ctl, interior) < 128,
        "a fixtura não contém o fenómeno: em Overlay o arco interior tinha de estar pintado \
         (mais escuro = {})",
        mais_escuro(&ctl, interior)
    );

    // A referência: o mesmo boolean SEM pilha nenhuma.
    let mut base = cena(&[]);
    base.set_stroke_op_mode(1); // Add
    elipse(&mut base, c1, r);
    elipse(&mut base, c2, r);
    let tinta_base = tinta(&base);
    assert_eq!(
        mais_escuro(&base, interior),
        255,
        "sem pilha o boolean já não apaga o arco — o defeito não é da pilha"
    );

    for (nome, camadas) in pilhas() {
        let mut t = cena(&camadas);
        t.set_stroke_op_mode(1);
        elipse(&mut t, c1, r);
        elipse(&mut t, c2, r);
        assert_eq!(
            mais_escuro(&t, interior),
            255,
            "{nome}: o arco interior voltou — os planos da pilha guardaram o carimbo da 1.ª figura"
        );
        // …e a pilha não pode inventar nem perder geometria: quem só DEPOSITA pinta o mesmo
        // contorno. (O Smear ARRASTA tinta por construção, logo ele fica fora desta metade.)
        if nome != "Smear/Brush" {
            assert_eq!(
                tinta(&t),
                tinta_base,
                "{nome}: o contorno da união deixou de ser o mesmo que sem pilha"
            );
        }
    }
}

/// ⭐ **Arrastar UMA figura não deixa rasto.**
///
/// A tela mostra UMA figura, no destino — o carimbo da posição anterior tinha de ser descascado
/// com a tela. **Medido antes da cura:** `+22 %` de tinta a mais nas pilhas de depósito e
/// **`+67 %`** com uma camada `Smear`, contra `+0 %` sem pilha.
///
/// ⚠️ A camada `Smear` é o membro que exige a **segunda** metade da cura (o campo de deslocamento
/// dela é por TRAÇO e vive fora dos planos), e por isso ela está nesta lista.
///
/// **Mutação que sangra:** tirar o `restamp_reset_pilha()` do `peel_drag_preview` · tirar o
/// `end_smear_session()` de dentro dele (só a linha do `Smear`).
#[test]
fn arrastar_uma_figura_nao_deixa_rasto() {
    for (nome, camadas) in pilhas() {
        // A referência: a MESMA figura desenhada já no destino.
        let mut destino = cena(&camadas);
        elipse(&mut destino, [160.0, 128.0], 40.0);
        let alvo = tinta(&destino);

        let mut t = cena(&camadas);
        elipse(&mut t, [90.0, 128.0], 40.0);
        t.on_canvas_pointer(cp([90.0, 128.0], PointerPhase::Down));
        for i in 1..=7u8 {
            t.on_canvas_pointer(cp([90.0 + f32::from(i) * 10.0, 128.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([160.0, 128.0], PointerPhase::Up));
        assert_eq!(
            tinta(&t),
            alvo,
            "{nome}: a figura arrastada deixou rasto — a tela tem de mostrar UMA figura, \
             a do destino"
        );
    }
}

/// ⭐ **Uma camada MAIOR que o pincel sobrevive ao re-carimbo.**
///
/// Uma camada com `size > 1` subamostra a lista de dabs por ARCO, e o acumulador dela
/// (`composite_arco`) é por TRAÇO. Num re-carimbo o `arc_len` dos dabs recomeça do zero e o
/// acumulador do carimbo anterior **recusa a lista inteira** — a camada desaparece.
///
/// **Medido antes da cura:** a figura re-carimbada dois píxeis ao lado pinta **`740`** texels
/// contra os `5 641` de uma figura acabada de desenhar no mesmo sítio.
///
/// ⚠️ **`size = 3` é ingrediente da fixtura, não decoração:** com `size = 1` a camada não
/// subamostra nada (`camada_dabs` devolve `None`) e o acumulador nunca é escrito — *uma fixtura no
/// ponto neutro do knob não testa o knob*.
///
/// **Mutação que sangra:** tirar o `composite_arco = [NEG_INFINITY; …]` do `restamp_reset_pilha`.
#[test]
fn uma_camada_maior_sobrevive_ao_recarimbo() {
    use CompositeOp::Brush;
    let camadas = [(Brush, 1.0), (Brush, 1.0)];
    let grande = |t: &mut PainterTool| {
        for pos in 0..t.composite_len() {
            t.set_composite_layer_size(pos, 3.0);
        }
    };
    let mut t = cena(&camadas);
    grande(&mut t);
    elipse(&mut t, [128.0, 128.0], 50.0);
    let uma_vez = tinta(&t);
    assert!(
        uma_vez > 3_000,
        "a fixtura não contém o fenómeno: uma camada de `size = 3` tinha de pintar largo \
         ({uma_vez})"
    );

    // Um agarrar que não move: o gesto re-carimba a figura na mesma geometria.
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Up));
    let depois = tinta(&t);
    // ⚠️ **A barra sai de um VALE MEDIDO e não é uma folga escolhida:** o re-carimbo pinta
    // `5 627` dos `5 658` (`99,5 %` — os `31` que faltam são orla, e a igualdade EXACTA não vale
    // porque a lista de dabs de uma figura é re-derivada a cada carimbo), e com o acumulador por
    // repor ele pinta **`740`** (`13 %`). O vale entre os dois lados é de **`7,6×`**.
    let barra = uma_vez * 9 / 10;
    assert!(
        depois >= barra,
        "a camada maior desapareceu no re-carimbo ({depois} contra {uma_vez}, barra {barra}) — \
         o acumulador de arco do carimbo anterior recusou a lista inteira"
    );
}

/// ⭐ **A rota de BISSECÇÃO tem a mesma cura.**
///
/// O `PH2D_COMPOSITE_REPLAY=1` guarda a história em `lotes` em vez de planos, e ela tem
/// exactamente a mesma doença: um re-carimbo re-emite o traço inteiro e a história do carimbo
/// anterior continua lá. *Uma cura que só tratasse a rota de omissão deixaria a bissecção a medir
/// outro programa* — e uma bissecção que mede outro programa não bissecta nada.
///
/// **Mutação que sangra:** tirar o `pilha.lotes.clear()` do `restamp_reset_pilha`.
#[test]
fn a_rota_de_bisseccao_tambem_descasca_a_historia() {
    use CompositeOp::{Blur, Brush};
    let (c1, c2, r) = ([100.0f32, 128.0], [156.0f32, 128.0], 45.0f32);
    let ang = 20.0f32.to_radians();
    let interior = [c1[0] + r * ang.cos(), c1[1] + r * ang.sin()];
    let camadas = [(Blur, 1.0), (Brush, 1.0)];

    let mut t = cena_na_rota(&camadas, true);
    t.set_stroke_op_mode(1); // Add
    elipse(&mut t, c1, r);
    elipse(&mut t, c2, r);
    assert_eq!(
        mais_escuro(&t, interior),
        255,
        "na rota de replay o arco interior voltou — a história do 1.º carimbo ficou nos `lotes`"
    );

    // …e arrastar uma figura não deixa rasto, pela mesma razão.
    let mut destino = cena_na_rota(&camadas, true);
    elipse(&mut destino, [160.0, 128.0], 40.0);
    let alvo = tinta(&destino);
    let mut d = cena_na_rota(&camadas, true);
    elipse(&mut d, [90.0, 128.0], 40.0);
    d.on_canvas_pointer(cp([90.0, 128.0], PointerPhase::Down));
    for i in 1..=7u8 {
        d.on_canvas_pointer(cp([90.0 + f32::from(i) * 10.0, 128.0], PointerPhase::Move));
    }
    d.on_canvas_pointer(cp([160.0, 128.0], PointerPhase::Up));
    assert_eq!(
        tinta(&d),
        alvo,
        "na rota de replay a figura arrastada deixou rasto"
    );
}

/// A mesma cena, mas com a tela **VAZIA** (tudo zero: RGB preto com alfa `0`).
///
/// ⚠️ **A opacidade da tela é ingrediente e não decoração.** Numa tela branca opaca — a que a
/// [`cena`] deste ficheiro monta — o esfregão move branco para dentro de branco e o borrão não
/// tem vizinho vazio de onde puxar: *a fixtura deixa de conter o fenómeno*. Quem mede um
/// acumulador que ATRAVESSA a tela mede-o aqui, e a régua é o **alfa**.
fn cena_vazia(camadas: &[(CompositeOp, f32)]) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (S * S * 4) as usize], S, S);
    t.paint.brush.radius_px = 6.0;
    t.paint.brush.color = [0.75, 0.12, 0.12];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.paint.composite_enabled = !camadas.is_empty();
    for (op, s) in camadas {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, *s);
    }
    t
}

/// Os bytes crus do quadrado `[0, lado) ²` — a janela que contém SÓ a primeira figura.
fn janela_bytes(t: &PainterTool, lado: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((lado * lado * 4) as usize);
    for y in 0..lado {
        let i = ((y * S) * 4) as usize;
        v.extend_from_slice(&t.canvas_rgba[i..i + (lado * 4) as usize]);
    }
    v
}

/// Quantos texels com alfa acima do ruído há no quadrado `[lo, hi) ²`.
fn alfa_na_caixa(t: &PainterTool, lo: u32, hi: u32) -> usize {
    let mut n = 0;
    for y in lo..hi {
        for x in lo..hi {
            if t.canvas_rgba[((y * S + x) * 4 + 3) as usize] > 4 {
                n += 1;
            }
        }
    }
    n
}

/// ⭐⭐ **A FIGURA QUE JÁ ESTÁ NA TELA NÃO MUDA QUANDO NASCE A SEGUINTE** — o report do dono de
/// 2026-09-21: *«2 círculos com o mesmo pincel e um está diferente do outro»*.
///
/// Uma sessão de figuras é **UM traço** (o pen-up não a fecha — ela fica editável até ao Apply), e
/// um lote do re-carimbo é a **CONCATENAÇÃO** das listas de dabs de todas as figuras vivas. Todo
/// acumulador POR-TRAÇO da pilha atravessa essa fronteira se ninguém a partir, e o do esfregão
/// atravessava: *o último dab de um círculo levantava tinta para o primeiro dab do círculo
/// seguinte, através da tela*.
///
/// **Medido na janela que contém só a 1.ª figura (soma do alfa), ANTES da cura:**
/// `Smear/Brush` **`522 145 → 448 091`** (`−14,2 %`; `n` `2 859 → 2 568`) ·
/// `Blur/Smear/Brush` **`422 835 → 361 950`** (`−14,4 %`).
///
/// ⚠️ **As outras quatro pilhas leem IGUAL dos dois lados e entram aqui como POPULAÇÃO**, porque a
/// lei é de todo acumulador e não do esfregão: *hoje só ele atravessa, e o gate reprova no dia em
/// que outro o faça.*
///
/// ⚠️ **A fronteira é DERIVADA** (`arc_len` recomeça em zero a cada `fill_*_preview`), e não um
/// campo novo — ⛔ um limiar sobre o comprimento do salto seria um número escolhido, e um traço à
/// mão livre rápido produz saltos legítimos do mesmo tamanho.
///
/// **Mutação que sangra:** `let fonte = from;` em [`super::smear_warp`] — as duas pilhas de
/// esfregão divergem, as outras quatro continuam iguais.
#[test]
fn a_primeira_figura_nao_muda_quando_nasce_a_segunda() {
    use CompositeOp::{Blur, Brush, Erase, Smear};
    let (a, b, r) = ([70.0f32, 70.0], [190.0f32, 190.0], 40.0f32);
    const JANELA: u32 = 130;

    // CONTROLO GEOMÉTRICO: as duas figuras não se tocam, senão «idêntico» seria a expectativa
    // errada e o gate estaria a exigir o impossível.
    let folga = (b[0] - a[0]).hypot(b[1] - a[1]) - 2.0 * r;
    assert!(
        folga > 2.0 * 6.0,
        "as duas figuras têm de ficar separadas por mais do que um diâmetro de pincel: folga \
         {folga:.1}"
    );
    assert!(
        a[0] + r + 6.0 < JANELA as f32 && b[0] - r - 6.0 > JANELA as f32,
        "a janela tem de conter a 1.ª figura inteira e nenhum pedaço da 2.ª"
    );

    let todas: [(&str, Vec<(CompositeOp, f32)>); 6] = [
        ("Brush", vec![(Brush, 1.0)]),
        ("Blur/Brush", vec![(Blur, 1.0), (Brush, 1.0)]),
        ("Smear/Brush", vec![(Smear, 1.0), (Brush, 1.0)]),
        ("Brush/Brush", vec![(Brush, 1.0), (Brush, 1.0)]),
        ("Erase/Brush", vec![(Erase, 0.4), (Brush, 1.0)]),
        (
            "Blur/Smear/Brush",
            vec![(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)],
        ),
    ];

    for (nome, camadas) in &todas {
        let mut so_a = cena_vazia(camadas);
        elipse(&mut so_a, a, r);
        let antes = janela_bytes(&so_a, JANELA);

        let mut as_duas = cena_vazia(camadas);
        elipse(&mut as_duas, a, r);
        elipse(&mut as_duas, b, r);
        let depois = janela_bytes(&as_duas, JANELA);

        // CONTROLO POSITIVO 1: a 1.ª figura pintou alguma coisa — senão comparar dois vazios é
        // uma tautologia.
        let n1 = alfa_na_caixa(&so_a, 0, JANELA);
        assert!(n1 > 500, "{nome}: a 1.ª figura mal pintou ({n1} texels)");
        // CONTROLO POSITIVO 2: a 2.ª figura de facto nasceu — senão o gate mede uma cena onde
        // nada aconteceu e passa por vácuo.
        let n2 = alfa_na_caixa(&as_duas, JANELA, S);
        assert!(n2 > 500, "{nome}: a 2.ª figura não nasceu ({n2} texels)");

        let difs = antes.iter().zip(&depois).filter(|(x, y)| x != y).count();
        assert_eq!(
            difs, 0,
            "{nome}: nascer a 2.ª figura mexeu em {difs} bytes da 1.ª"
        );
    }
}

/// ⭐⭐ **A SEGUNDA FIGURA NÃO PAGA O ARCO DA PRIMEIRA** — a outra metade do report do dono de
/// 2026-09-21 (*«se dois círculos cada um tem um aspecto»*), e a que só arma quando uma camada é
/// **MAIOR que o pincel**. O dono disse qual era a pilha dele: *«todas as camadas ativas, com
/// tamanhos diferentes»*.
///
/// Uma camada com `size > 1` subamostra a lista por ARCO e o acumulador dela (`composite_arco`) é
/// por TRAÇO. Um lote do re-carimbo é a **CONCATENAÇÃO** das figuras vivas, logo o arco do lote
/// **anda para trás** na fronteira entre elas — e a subamostragem lê isso como *«este dab está
/// perto demais do último que guardei»* e **recusa a lista inteira da segunda figura**.
///
/// ⚠️ **É a MESMA fronteira que a corrente do esfregão parte**
/// ([`super::arco_subfigura`]), por isso as duas a perguntam à mesma porta.
///
/// ⚠️ **`size = 3` é ingrediente e não decoração:** com `size = 1` o `camada_dabs` devolve `None`,
/// o acumulador nunca é escrito e o fenómeno **não existe** — é por isso que o CONTROLO desta
/// fixtura é a mesma cena com o knob no ponto neutro.
#[test]
fn a_segunda_figura_nao_paga_o_arco_da_primeira() {
    use CompositeOp::Brush;
    let (a, b, r) = ([70.0f32, 70.0], [190.0f32, 190.0], 40.0f32);
    const JANELA: u32 = 130;

    let duas_figuras = |size: f32| -> (usize, usize) {
        let mut t = cena_vazia(&[(Brush, 1.0)]);
        for pos in 0..t.composite_len() {
            t.set_composite_layer_size(pos, size);
        }
        elipse(&mut t, a, r);
        elipse(&mut t, b, r);
        (alfa_na_caixa(&t, 0, JANELA), alfa_na_caixa(&t, JANELA, S))
    };

    // CONTROLO: no ponto neutro do knob as duas figuras pintam o mesmo — é isso que prova que a
    // fixtura mede a SUBAMOSTRAGEM e não a geometria das duas posições.
    let (n1_neutro, n2_neutro) = duas_figuras(1.0);
    assert!(
        n1_neutro > 500 && n2_neutro > 500,
        "controlo: com `size = 1` as duas figuras têm de pintar ({n1_neutro} / {n2_neutro})"
    );

    let (n1, n2) = duas_figuras(3.0);
    assert!(
        n1 > 500,
        "a fixtura não contém o fenómeno: a 1.ª figura mal pintou ({n1} texels)"
    );
    // As duas figuras são CONGRUENTES (mesmo raio, mesmo pincel, mesma pilha), logo a segunda tem
    // de pintar o que a primeira pinta. A folga cobre a orla — a lista de dabs é re-derivada a
    // cada carimbo — e nada mais.
    let razao = n2 as f32 / n1 as f32;
    assert!(
        razao > 0.9,
        "a 2.ª figura pagou o arco da 1.ª: {n2} texels contra {n1} ({:.0} %)",
        razao * 100.0
    );
}

/// ⭐⭐⭐ **A CORRENTE PARTE NA FRONTEIRA MESMO QUANDO A CAMADA SUBAMOSTRA** — os dois
/// acumuladores deste report **compõem**, e é essa composição que decide que a cura é uma PORTA.
///
/// O esfregão parte a corrente dele quando o arco anda para trás, mas ele lê a lista **DEPOIS** da
/// subamostragem da camada. Se o filtro comesse o primeiro dab da 2.ª figura, o esfregão nunca
/// veria o arco andar para trás e a corrente atravessaria a tela — *a cura de uma metade seria
/// desfeita pela outra, em silêncio*.
///
/// O que torna isto verdade não é uma cerca a lembrar: é a porta responder à mesma pergunta nos
/// dois sítios, logo **o dab da fronteira sobrevive ao filtro por construção**.
///
/// ⚠️ **`size = 3` é o ingrediente** — no ponto neutro o filtro nem corre, e este gate seria o
/// irmão dele outra vez.
#[test]
fn a_corrente_parte_na_fronteira_mesmo_numa_camada_subamostrada() {
    use CompositeOp::{Brush, Smear};
    let (a, b, r) = ([70.0f32, 70.0], [190.0f32, 190.0], 40.0f32);
    const JANELA: u32 = 130;
    let camadas = [(Smear, 1.0), (Brush, 1.0)];

    let cena_grande = || {
        let mut t = cena_vazia(&camadas);
        for pos in 0..t.composite_len() {
            t.set_composite_layer_size(pos, 3.0);
        }
        t
    };

    let mut so_a = cena_grande();
    elipse(&mut so_a, a, r);
    let antes = janela_bytes(&so_a, JANELA);

    let mut as_duas = cena_grande();
    elipse(&mut as_duas, a, r);
    elipse(&mut as_duas, b, r);
    let depois = janela_bytes(&as_duas, JANELA);

    // CONTROLO POSITIVO: as duas figuras existem na cena subamostrada — sem isto o gate compara
    // dois vazios e passa por vácuo, que é exactamente o estado em que o defeito a curar punha a
    // segunda figura.
    let n1 = alfa_na_caixa(&so_a, 0, JANELA);
    let n2 = alfa_na_caixa(&as_duas, JANELA, S);
    assert!(n1 > 500, "a 1.ª figura mal pintou ({n1} texels)");
    assert!(n2 > 500, "a 2.ª figura não nasceu ({n2} texels)");

    let difs = antes.iter().zip(&depois).filter(|(x, y)| x != y).count();
    assert_eq!(
        difs, 0,
        "com a camada a subamostrar, nascer a 2.ª figura mexeu em {difs} bytes da 1.ª"
    );
}
