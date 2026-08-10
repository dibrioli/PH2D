//! **Os quatro "Use as …" leem a APARÊNCIA, não o pigmento.**
//!
//! Report do Enio (2026-08-09): *"Use as Brush Shape não transfere para o brush os relevos criados
//! por Impasto"*, com o pedido de conferir os outros modos "Use as …" do menu. Conferidos: os
//! quatro passavam por DUAS portas do tool (`capture_layers_as_brush_shape` para o Shape,
//! `composite_to_lum` para Grain / Paper / Granulation) e **nenhuma das duas iluminava**.
//!
//! ⚠️ **O enquadramento que decide o conserto:** o BAKE (`RasterEditTool::run_full`) responde à
//! MESMA pergunta — *com o que este documento se parece?* — e o doc-comment dele já dizia por que
//! ilumina: *"o campo de altura não sobrevive ao Apply, então a sombra tem de ser assada, senão o
//! Apply jogaria o relevo fora em silêncio e devolveria tinta chapada"*. Duas portas, uma pergunta,
//! respostas diferentes — e o sintoma era assimétrico: um sprite JÁ assado da hierarquia levava a
//! sombra do relevo (o ramo `read_sprite_source` do shell lê a textura assada) e o documento ATIVO
//! não. Medido antes do conserto: **523 de 3600 texels diferem, pior delta 68**.
//!
//! ⚠️ **E a luz não entra pela COBERTURA, o que é fato e não escolha:** a silhueta que o documento
//! ativo captura é alpha, e o shade escreve COR e nunca alpha. Por isso o relevo chega ao slot Shape
//! pela **cor capturada** — a captura ilumina os pixels de cada camada com o passe canônico
//! ([`super::shape_settings`]) e guarda a APARÊNCIA. Um desenho a tinta PRETA continua imprimindo a
//! própria cobertura, e é o relevo que sombreia o que ela imprime.
//!
//! ⚠️ **A 1ª versão levava o relevo por um GANHO escalar** (um `rgb × luminância`), e ela foi
//! substituída porque não era exata: MEDIDO contra o que o artista vê, errava até 96 níveis de 255
//! (98 com cera âmbar) — um escalar não carrega a COR do especular. Os gates deste arquivo medem a
//! PROPRIEDADE (o relevo alcança a cor e nunca a cobertura), então sobreviveram à troca de mecanismo
//! sem uma linha; os que mediam a sonda de albedo do ganho morreram com ela.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

/// `paper_alpha` = a opacidade do papel: **255** é o caso do sprite importado e **0** o da tela
/// transparente em que o artista desenha do zero. A rota de Shape lê COBERTURA, então os dois
/// respondem coisas diferentes e os gates precisam dos dois.
///
/// `impasto` liga o depósito de corpo; com ele desligado o documento é o CONTROLE — sem relevo, tudo
/// aqui tem de ser byte-idêntico ao que já shipava.
fn ridge(paper_alpha: u8, impasto: bool, color: [f32; 3]) -> PainterTool {
    let size = 60u32;
    let mut t = PainterTool::default();
    let mut px = vec![255u8; (size * size * 4) as usize];
    for p in px.chunks_exact_mut(4) {
        p[3] = paper_alpha;
    }
    t.set_source(px, size, size);
    // Falloff MACIO de propósito: um disco duro deixa um platô de paredes verticais, cujo `h` é o
    // mesmo no centro e nos dois flancos — não há gradiente para a luz ler, e o gate estaria
    // afirmando sobre nada (a armadilha que o `impasto_light_reads_as_raised_not_engraved` documenta).
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 0.0,
        falloff: Falloff::Smooth,
        color,
        space_attenuation: false,
        impasto,
        impasto_depth: 1.0,
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.paint.impasto_show = true;
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(30.0, 10.0, PointerPhase::Down));
    t.on_canvas_pointer(at(30.0, 50.0, PointerPhase::Move));
    t.on_canvas_pointer(at(30.0, 50.0, PointerPhase::Up));
    t
}

/// A MESMA fixture, para o gate irmão de `shape_alpha_tests` — um documento com relevo esculpido.
/// Emprestada em vez de re-escrita: duas fixtures do mesmo documento derivam, e a segunda passa a
/// medir um relevo que não é o que esta mede.
pub(super) fn ridge_for_alpha_gate() -> PainterTool {
    ridge_for_alpha_gate_with(true)
}

/// A mesma, com o impasto como PARÂMETRO — o gate irmão precisa do controle sem relevo, e ele tem de
/// ser o mesmo documento em tudo o mais.
pub(super) fn ridge_for_alpha_gate_with(impasto: bool) -> PainterTool {
    ridge(0, impasto, [0.1, 0.2, 0.3])
}

fn lum(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect()
}

/// `(quantos texels diferem, pior delta)` — um resumo honesto de quão longe dois campos estão.
fn spread(a: &[u8], b: &[u8]) -> (usize, i32) {
    let n = a.len().min(b.len());
    let mut diff = 0usize;
    let mut worst = 0i32;
    for i in 0..n {
        let d = i32::from(a[i]) - i32::from(b[i]);
        if d != 0 {
            diff += 1;
        }
        worst = worst.max(d.abs());
    }
    (diff, worst)
}

/// A COR que o slot Shape capturou, com o relevo LIGADO ou DESLIGADO — o par que isola o ganho.
///
/// ⚠️ O interruptor é o `impasto_show`, e a escolha é load-bearing: ele é EXIBIÇÃO (quem o lê é o
/// `impasto_visible`) e não move um byte do canvas, então a única coisa que difere entre as duas
/// capturas é o ganho. Comparar contra um gesto construído SEM impasto compararia o canvas.
fn captured_colour(show_relief: bool) -> Vec<u8> {
    let mut t = ridge(255, true, [0.1, 0.2, 0.3]);
    t.paint.impasto_show = show_relief;
    t.capture_layers_as_brush_shape();
    t.paint
        .shape_layers
        .rgb_image(0)
        .expect("a captura guarda a cor da camada")
        .rgb
        .to_vec()
}

fn silhouette(t: &mut PainterTool) -> Vec<u8> {
    t.capture_layers_as_brush_shape();
    t.brush_shape_image()
        .map(|(sil, _, _)| sil.to_vec())
        .expect("a captura produz uma silhueta")
}

/// **Uma pergunta, uma resposta.** O que o Grain / Paper / Granulation leem é, ao byte, o que o BAKE
/// produz — as duas portas para *"com o que este documento se parece?"* passam pela MESMA função.
///
/// **Mutação que deve sangrar:** remover o `apply_impasto_light` do `composite_to_lum`.
#[test]
fn the_use_as_paths_read_what_the_bake_bakes() {
    let (grain, _, _) = ridge(255, true, [0.1, 0.2, 0.3])
        .composite_to_lum()
        .expect("composite");
    let (baked, _, _) = ridge(255, true, [0.1, 0.2, 0.3]).run_full();
    let (diff, worst) = spread(&grain, &lum(&baked));
    assert_eq!(
        (diff, worst),
        (0, 0),
        "o Grain / Paper / Granulation e o BAKE respondem a mesma pergunta e discordam — e discordam \
         exatamente pela luz do relevo, que era o report do Enio"
    );
}

/// **O CONTROLE, e é ele que torna as duas metades seguras:** num documento sem relevo o passe
/// multiplica por 1 e soma 0, então nada aqui se move — nem a luminância que o Grain lê, nem a
/// silhueta que o Shape captura.
///
/// ⚠️ Sem este gate as duas mudanças seriam afirmações sobre o caso novo apenas; com ele, todo
/// documento que ninguém esculpiu está pinado.
#[test]
fn a_document_without_relief_is_untouched_to_the_byte() {
    let mut lit = ridge(255, false, [0.1, 0.2, 0.3]);
    lit.paint.impasto_show = true;
    let mut dark = ridge(255, false, [0.1, 0.2, 0.3]);
    dark.paint.impasto_show = false;

    let (a, _, _) = lit.composite_to_lum().expect("composite");
    let (b, _, _) = dark.composite_to_lum().expect("composite");
    assert_eq!(spread(&a, &b), (0, 0), "sem relevo, a luz nao muda um byte");
    assert_eq!(
        spread(&silhouette(&mut lit), &silhouette(&mut dark)),
        (0, 0),
        "sem relevo, o ganho e a identidade e a silhueta nao se move"
    );
}

/// **O relevo CHEGA ao pincel pela COR** — o report de 2026-08-09 (*"Use as Brush Shape não
/// transfere para o brush os relevos criados por Impasto"*), medido depois de a 2ª rodada do MESMO
/// dia dizer onde ele NÃO pode chegar.
///
/// ⚠️ **A 1ª versão levava o relevo à SILHUETA, e o Enio devolveu a foto:** a sombra de um relevo é
/// ganho `< 1`, então na cobertura ela vira transparência e a tela aparece através dela — um
/// documento opaco carimbava FURADO exatamente onde ele esculpiu (*"aparece um alpha, o branco no
/// lugar da sombra"*). O relevo SOMBREIA a tinta; ele não a perfura.
///
/// As duas metades, e nenhuma sozinha basta: a cor capturada tem de VARIAR com o relevo (senão ele
/// não chegou a lugar nenhum) **e** a silhueta por alpha tem de ficar CHEIA (senão ele chegou pelo
/// canal errado, que é o defeito reportado).
///
/// **Mutação que deve sangrar:** aplicar o ganho na máscara em vez de na cor.
#[test]
fn the_relief_reaches_the_brushs_colour_and_never_its_coverage() {
    // ⚠️ **O controle varia o GANHO e nada mais** — `impasto_show` é EXIBIÇÃO e não move um byte do
    // canvas. Pedir só que a cor VARIE seria verde por vácuo: a cor de um desenho varia sozinha (é a
    // tinta sobre o papel), e a mutação que tira o ganho da cor passaria. Foi o que ela fez.
    let differing = captured_colour(true)
        .iter()
        .zip(captured_colour(false).iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 100,
        "so {differing} texels da cor capturada diferem da MESMA captura com a luz do relevo \
         desligada — o relevo esculpido nao alcancou o pincel"
    );
    // A outra metade: a silhueta por ALPHA de um papel OPACO e o quadrado CHEIO. Era ela que o
    // ganho perfurava, e e a foto do report.
    let sil = silhouette(&mut ridge(255, true, [0.1, 0.2, 0.3]));
    let (smin, smax) = sil
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    assert_eq!(
        (smin, smax),
        (255, 255),
        "a silhueta de um papel opaco saiu {smin}..{smax} — o relevo voltou a entrar pela COBERTURA \
         e o carimbo pinta furado onde o artista esculpiu"
    );
}

/// ⚠️ **O PREÇO da correção acima, pinado em vez de escondido:** sem cor para sombrear, o relevo não
/// alcança o carimbo.
///
/// Com **Per-Layer Color desligado** o carimbo pinta uma cor CHAPADA através da silhueta, e a
/// silhueta é o alpha autorado — não há canal onde a sombra caiba. É o estreitamento que a 2ª rodada
/// do report impôs: a alternativa é a cobertura, que é o defeito. Quem quiser o relevo no pincel liga
/// Texture Color, que é o modo da própria foto do Enio.
///
/// Este gate não julga o desenho; ele impede que a ausência seja lida como esquecimento.
#[test]
fn without_a_colour_to_shade_the_relief_does_not_reach_the_stamp() {
    let lit = silhouette(&mut ridge(255, true, [0.1, 0.2, 0.3]));
    let flat = silhouette(&mut ridge(255, false, [0.1, 0.2, 0.3]));
    assert_eq!(
        spread(&lit, &flat),
        (0, 0),
        "a silhueta de um papel opaco mudou com o relevo — ou o ganho voltou a cobertura, ou este \
         gate parou de medir a rota de cor chapada"
    );
}

/// **A silhueta é COR-INDEPENDENTE, e é por isso que ela continua sendo cobertura.**
///
/// Trocar a captura do documento ativo pela LUMINÂNCIA — o que as outras portas do slot Shape leem
/// (o file-load e o sprite chapado) — teria "resolvido" o report e transformado todo desenho a tinta
/// PRETA sobre transparência num carimbo invisível. O ganho multiplica o que já está lá; ele não
/// decide o que está lá.
///
/// **Mutação que deve sangrar:** a silhueta passar a ser a luminância do composite.
/// **O relevo chega também ao modo PER-LAYER COLOR** — e este gate existe porque a primeira versão
/// do conserto cobria metade das rotas.
///
/// ⚠️ O caminho de imagem única passa pelo `flatten`; o **Per-Layer Color** não — ele carimba pelas
/// máscaras cruas e pela COR por camada. Com o ganho aplicado no flatten, o relevo alcançava o
/// pincel exatamente enquanto o artista não ligasse o modo que pinta com as cores da textura, que é
/// o modo que o report do Enio pedia (2026-08-09).
///
/// ⚠️ **O canal mudou na 2ª rodada do mesmo report** (era a máscara, é a cor), e o gate mudou com
/// ele: uma máscara que varia AQUI é o carimbo furado da foto.
///
/// **Mutação que deve sangrar:** aplicar o ganho na máscara em vez de na cor.
#[test]
fn the_relief_reaches_the_per_layer_colour_route_too() {
    let mut t = ridge(255, true, [0.1, 0.2, 0.3]);
    t.capture_layers_as_brush_shape();
    let masks = t.paint.shape_layers.masks();
    assert_eq!(masks.len(), 1, "a fixture tem uma camada");
    let (mmin, mmax) = masks[0]
        .lum
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    assert_eq!(
        (mmin, mmax),
        (255, 255),
        "a MASCARA que o Per-Layer Color carimba saiu {mmin}..{mmax} — o relevo esta entrando pela \
         cobertura e o carimbo pinta furado"
    );
    // A cor, contra o MESMO gesto com a luz do relevo desligada (o controle que isola o ganho).
    let differing = captured_colour(true)
        .iter()
        .zip(captured_colour(false).iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 100,
        "so {differing} texels da COR que o Per-Layer Color carimba diferem da captura sem a luz do \
         relevo — ele parou antes da rota que pinta com as cores da textura"
    );
}

/// ⚠️ **A primeira versão deste gate tomava o MÁXIMO sobre a tela e passava sob a mutação.** O papel
/// transparente tem alpha 0 e RGB 255, então a luminância dele é 255 — o máximo media o PAPEL, não a
/// pincelada. Um oráculo global sobre uma tela cujo fundo domina não fala sobre o traço; a asserção é
/// no texel onde a tinta está.
#[test]
fn a_black_drawing_still_prints() {
    let sil = silhouette(&mut ridge(0, true, [0.0, 0.0, 0.0]));
    let at_stroke = sil[30 * 60 + 30]; // o centro do traço, que desce pela coluna 30
    assert!(
        at_stroke > 200,
        "uma pincelada PRETA sobre tela transparente imprime {at_stroke} no proprio traco — a \
         silhueta virou luminancia e o carimbo ficou invisivel"
    );
}

/// **Guardar a cor não move a silhueta** — a rota do sprite plano continua entregando exatamente a
/// máscara de luminância que sempre entregou.
///
/// ⚠️ Sem este gate, "o slot agora guarda uma camada" seria uma afirmação sobre uma estrutura, e o
/// artista julgaria pelo carimbo: trocar a silhueta pela COBERTURA aqui faria todo sprite opaco virar
/// um quadrado, que é o defeito que o próprio relatório desta jornada mediu do outro lado.
///
/// **Mutação que deve sangrar:** `set_brush_shape_image_rgba` guardar o alpha em vez da luminância.
#[test]
fn installing_a_sprite_with_colour_keeps_the_silhouette_it_always_had() {
    // Um sprite com cor E alpha variando de formas DIFERENTES — senão luminância e cobertura
    // coincidem e a mutação não teria como divergir.
    let (w, h) = (8u32, 8u32);
    let n = (w * h) as usize;
    let mut px = vec![0u8; n * 4];
    for (i, p) in px.chunks_exact_mut(4).enumerate() {
        p[0] = (i * 3) as u8;
        p[1] = (i * 5) as u8;
        p[2] = (i * 7) as u8;
        p[3] = 255 - (i * 2) as u8;
    }
    let lum: Vec<u8> = px
        .chunks_exact(4)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect();

    let mut old = PainterTool::default();
    old.set_brush_shape_image(lum, w, h);
    let before = old.brush_shape_image().expect("silhueta").0.to_vec();

    let mut new = PainterTool::default();
    new.set_brush_shape_image_rgba(&px, w, h, Some(7));
    let after = new.brush_shape_image().expect("silhueta").0.to_vec();

    assert_eq!(
        spread(&before, &after),
        (0, 0),
        "guardar a cor mudou a silhueta — o carimbo do artista nao e o mesmo"
    );
}

/// **Um sprite de UMA camada pinta com as próprias cores** — o report, no mecanismo.
///
/// A capacidade sempre existiu (o modo liga um bit, e `color_on` desligado já significa *"a camada
/// pinta as cores que capturou"*); o que faltava era **ter cor guardada** nesta rota e a UI que a
/// liga. Aqui está a metade do modelo; a metade da UI é o seam do painel.
///
/// **Mutação que deve sangrar:** `set_brush_shape_image_rgba` delegar ao `set_brush_shape_image`.
#[test]
fn a_single_layer_sprite_can_paint_its_own_colours() {
    let (w, h) = (4u32, 4u32);
    let mut px = vec![255u8; (w * h) as usize * 4];
    for (i, p) in px.chunks_exact_mut(4).enumerate() {
        p[0] = (i * 11) as u8;
    }
    let mut t = PainterTool::default();
    t.set_brush_shape_image_rgba(&px, w, h, Some(7));
    assert_eq!(
        t.brush_settings().shape_layer_count,
        1,
        "o sprite tem de chegar como UMA camada, senao nao ha o que colorir"
    );
    t.toggle_brush_shape_per_layer_color();
    assert!(
        t.brush_settings().shape_per_layer_color,
        "o modo nao liga — o checkbox ficaria pintado e inerte"
    );
    assert!(
        t.paint.shape_layers.rgb_image(0).is_some(),
        "a camada nao carrega o RGB do sprite — o modo pintaria com a cor do pincel, que e o que \
         desligar o checkbox ja faz"
    );
}

/// Sonda: imprime o retrato inteiro (as quatro rotas contra o que o artista vê).
#[test]
#[ignore = "sonda de medicao; roda com -- --ignored --nocapture"]
fn measure_what_the_use_as_paths_read() {
    let mut t = ridge(255, true, [0.1, 0.2, 0.3]);
    let (seen, w, h) = t.take_preview_arc().expect("preview");
    let seen_lum = lum(&seen);
    println!("canvas {w}x{h}   relevo visivel = {}", t.impasto_visible());
    let (grain, _, _) = t.composite_to_lum().expect("composite");
    println!(
        "GRAIN/PAPER/GRANUL. vs o que se VE : {:?}",
        spread(&grain, &seen_lum)
    );
    let sil = silhouette(&mut t);
    let (min, max) = sil
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    println!("SHAPE  faixa de valores: {min}..{max}");
    let (baked, _, _) = ridge(255, true, [0.1, 0.2, 0.3]).run_full();
    println!(
        "BAKE   vs GRAIN                    : {:?}",
        spread(&lum(&baked), &grain)
    );
}
