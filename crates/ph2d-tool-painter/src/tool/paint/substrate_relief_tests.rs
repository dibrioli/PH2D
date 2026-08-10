//! Gates + sonda do **SUBSTRATO** ([`super::substrate_relief`]).
//!
//! A lei em uma frase: o dente do papel é uma superfície, a normal dele soma à da tinta, e o
//! [`super::impasto_shade::Rig`] — que é RELATIVO — a sombreia. As perguntas que decidem se isso está
//! certo são quatro, e cada uma tem gate próprio porque cada uma pode falhar sozinha.
use super::*;
use crate::Region;
use ph2d_editor_core::tool::RasterEditTool;

const N: u32 = 96;

/// Uma tela BRANCA e chapada, sem relevo de tinta nenhum — o documento do Digital.
fn blank() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t
}

/// A tela depois do passe de luz.
fn lit(t: &PainterTool) -> Vec<u8> {
    let mut rgba = vec![255u8; (N * N * 4) as usize];
    t.apply_impasto_light(
        &mut rgba,
        Region {
            x: 0,
            y: 0,
            w: N,
            h: N,
        },
    );
    rgba
}

/// Excursão de luminância em NÍVEIS (`max − min`) — o número que diz se o dente se vê.
fn excursion(px: &[u8]) -> u32 {
    let l: Vec<u32> = px.chunks_exact(4).map(|c| u32::from(c[0])).collect();
    l.iter().max().copied().unwrap_or(0) - l.iter().min().copied().unwrap_or(0)
}

/// ⚠️ **O NEUTRO É BYTE-IDÊNTICO, e é o que torna esta wave segura de shipar.**
///
/// `depth = 0` é o default, então toda arte já feita e todo documento que ninguém tocou têm de sair
/// exatamente como saíam. Não "quase": ao BYTE.
#[test]
fn the_substrate_is_off_by_default_and_off_is_byte_identical() {
    let t = blank();
    assert_eq!(t.substrate_depth(), 0.0, "o default tem de ser DESLIGADO");
    let before = lit(&t);
    assert_eq!(excursion(&before), 0, "sem substrato a tela sai chapada");

    let mut t2 = blank();
    t2.set_substrate_depth(0.0); // o gesto explícito de desligar tem de ser igualmente inerte
    assert_eq!(lit(&t2), before, "depth 0 nao pode mover um byte");
}

/// ⚠️ **O PAPEL ACENDE SEM TINTA — a razão de esta wave existir.**
///
/// O Digital não tem `covers`, `heights` nem `mats`: os três planos são do impasto e nascem vazios. Se
/// a regra *"relevo sob cobertura zero não acende"* valesse aqui, o dente seria invisível exatamente no
/// meio para o qual ele foi pedido, e todos os outros gates passariam.
#[test]
fn the_paper_lights_on_a_canvas_with_no_paint_at_all() {
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let e = excursion(&lit(&t));
    assert!(
        e >= 6,
        "o dente do papel tem de se ver numa tela NUA; excursao medida {e} niveis"
    );
}

/// ⚠️ **A UNIDADE atravessa a conversão do consumidor.**
///
/// O `shade_over` multiplica a inclinação por `DEPTH_UNIT_PX` porque o buffer de altura da TINTA é
/// medido em cargas; o dente já é medido em pixels. Entregar a inclinação crua inclinaria o papel 16×
/// demais — e o modo de falha não é sutil, é chapa ondulada em vez de papel.
#[test]
fn the_tooth_crosses_the_lights_depth_unit_on_the_way_in() {
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let e = excursion(&lit(&t));
    assert!(
        e <= 120,
        "o dente esta inclinado demais para ser papel ({e} niveis) — a conversao de unidade caiu"
    );
}

/// ⚠️ **A ROUGHNESS TEM TRABALHO** — o pedido explícito do Enio, e a espécie de controle que esta casa
/// extermina quando não move nada.
///
/// ⚠️ **Ela é a ÍNGREMEZA do dente, não a largura de um realce — e foi a MEDIÇÃO que decidiu isso.** A
/// primeira leitura (o expoente especular, a Roughness da TINTA) fez este gate nascer com **0 texels
/// movidos**, porque o realce plano é subtraído e clampado e num dente de ~1 px ele é nulo em qualquer
/// expoente (o ⛔ em [`super::substrate_relief`]). A leitura que shipa é a das referências — o
/// *Contrast* do Corel ("*steepness of the paper grain*") e o *Roughness* do ArtRage.
#[test]
fn the_paper_roughness_changes_the_picture() {
    let mut tight = blank();
    tight.set_substrate_depth(1.0);
    tight.set_substrate_roughness(0.0);
    let mut broad = blank();
    broad.set_substrate_depth(1.0);
    broad.set_substrate_roughness(1.0);
    let (a, b) = (lit(&tight), lit(&broad));
    let moved = a
        .chunks_exact(4)
        .zip(b.chunks_exact(4))
        .filter(|(x, y)| x[0] != y[0])
        .count();
    assert!(
        moved > 200,
        "a Roughness do papel tem de mover o realce; texels diferentes: {moved}"
    );
}

/// ⚠️ **O DEPTH é MONOTÔNICO** — um slider que não ordena não é um slider.
#[test]
fn a_deeper_tooth_reads_deeper() {
    let mut prev = 0;
    for step in [1u32, 2, 4] {
        let mut t = blank();
        t.set_substrate_depth(step as f32 / 4.0);
        let e = excursion(&lit(&t));
        assert!(
            e >= prev,
            "depth {step}/4 leu MENOS que o anterior ({e} contra {prev})"
        );
        prev = e;
    }
    assert!(prev > 0, "no Depth maximo o dente tem de existir");
}

/// ⚠️ **Ligar o relevo sem papel escolhido ARMA um papel.** Sem isto o interruptor liga e não mostra
/// nada — o controle morto que a casa recusa, na forma mais fácil de shipar sem ver.
#[test]
fn arming_the_relief_without_a_paper_picks_one() {
    let mut t = blank();
    assert!(
        !t.paint.brush.paper.is_active(),
        "a fixture tem de comecar SEM papel, senao este gate nao testa nada"
    );
    t.set_substrate_depth(1.0);
    assert!(
        t.paint.brush.paper.is_active(),
        "ligar tem de armar um papel"
    );
    assert!(
        excursion(&lit(&t)) > 0,
        "e o papel armado tem de ser VISIVEL"
    );
}

/// ⚠️ **O papel é do CANVAS; o slot é do PINCEL.** O fan-out é o que impede trocar de modo de pintura
/// de trocar o papel debaixo da obra.
#[test]
fn the_paper_survives_a_change_of_paint_mode() {
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let kind = t.paint.brush.paper.kind;
    for b in t.paint.brush_by_mode.iter() {
        assert_eq!(
            b.paper.kind, kind,
            "um slot de pincel ficou com outro papel — o fan-out nao alcancou todos"
        );
    }
}

/// ⚠️ **E O PAPEL SOBREVIVE A PEGAR OUTRA FERRAMENTA — MEDIDO, e a lei já estava escrita.**
///
/// *"O papel é do CANVAS; o slot é do PINCEL"* é o doc do `set_paper_field`, e o `set_substrate_depth`
/// sempre o honrou. Os **sete** setters do slot Paper não: eles escreviam só o pincel vivo. Isso era
/// invisível enquanto o dente só temperava o pigmento MOLHADO — desde que ele virou uma superfície do
/// canvas, deixou de ser. Medido antes da cura: Paper Size em 5, pegar a **Faca** (slot próprio) o
/// devolvia a 1 e a excursão caía de **166 para 75 níveis**, com o papel mudando debaixo da obra
/// porque o artista pegou uma ferramenta.
///
/// **Mutação que tem de sangrar:** tirar o `sync_paper_across_slots()` de qualquer um dos sete.
#[test]
fn every_paper_knob_reaches_every_brush_slot() {
    let knobs: [PaperKnob; 8] = [
        ("Paper kind", |t| {
            t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperRough.to_u8());
        }),
        ("Paper Size", |t| t.set_brush_paper_size(0, 5.0)),
        ("Paper Angle", |t| t.set_brush_paper_angle(37.0)),
        ("Paper Mapping", |t| t.set_brush_paper_mapping(0)),
        ("Paper Offset", |t| t.set_brush_paper_offset(0, 0.25)),
        ("Paper Contrast", |t| t.set_brush_paper_param(0, 0.9)),
        ("Paper reset", |t| t.reset_brush_paper()),
        ("Preset", |t| t.apply_brush_preset(1)),
    ];
    for (name, turn) in knobs {
        let mut t = blank();
        t.set_substrate_depth(1.0);
        // ⚠️ **A fixture parte de um papel DISTINTO do default, e é o que dá dentes ao gate.** A row do
        // Preset nasceu VAZIA: o papel que o preset semeia é byte-idêntico ao que o arm põe (medido —
        // `PaperCold`, size `[1,1]`, angle 0), então live e slots concordavam com ou sem a porta, e a
        // mutação passava. Partindo de um ângulo que ninguém mais escreve, toda porta que NÃO propaga
        // deixa os slots com ele e o gate morde.
        t.set_brush_paper_angle(41.0);
        let before = t.paint.brush.paper;
        turn(&mut t);
        let live = t.paint.brush.paper;
        assert_ne!(
            live, before,
            "{name}: o gesto nao mudou o papel VIVO — esta row nao testa porta nenhuma"
        );
        for (i, b) in t.paint.brush_by_mode.iter().enumerate() {
            assert_eq!(
                b.paper, live,
                "{name}: o slot {i} ficou com outro papel — pegar a ferramenta dele troca o papel \
                 debaixo da obra"
            );
        }
    }
}

/// ⚠️ **O DENTE CHEGA AO PRODUTOR DA GPU — o gate que faltava, e a razão de o Enio ver o papel morto.**
///
/// Este app tem **DOIS produtores** para a mesma tela: o `apply_impasto_light` da CPU (que todo gate
/// acima dirige) e o `ImpastoLightPass` da GPU, que recebe os planos já dobrados por
/// [`super::impasto_gpu::…::impasto_gpu_planes_in`] e refaz **só a óptica** no device. Um documento
/// pintado num canvas que a GPU compõe — o caso normal — nunca passa pelo laço da CPU.
///
/// A primeira versão desta wave somava a inclinação do dente **dentro do laço da CPU**, então os sete
/// gates ficavam verdes sobre um produto em que o papel não acendia. Reportado: *"Paper parece não
/// funcionar para Digital"*.
///
/// O oráculo é o PLANO que sobe ao device, e não um pixel: o shader deriva a normal por diferença
/// central sobre ele, então um plano chato é uma tela chata, sem exceção.
///
/// **Mutação que tem de sangrar:** tirar o dente do `ReliefFields::height_at` e devolvê-lo ao laço da
/// CPU — o gate volta a `0,000000` de excursão, que é exatamente o que o artista viu.
#[test]
fn the_tooth_reaches_the_gpu_producer_too() {
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let planes = t
        .impasto_gpu_planes_in((0, 0, N, N))
        .expect("com substrato ligado o passe de luz tem de produzir planos");
    let (lo, hi) = planes
        .relief
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    assert!(
        hi - lo > 1e-3,
        "o plano de relevo que sobe para a GPU é CHATO ({lo:.6}..{hi:.6}) — o dente do papel não \
         atravessa o fold, então na tela que o device compõe o papel não existe"
    );
}

/// **E o CORPO do papel também chega** — a outra metade, e ela falha sozinha.
///
/// A luz pesa por presença (`body`), e a lei do substrato é a exceção honesta *"a cobertura de um papel
/// é 1"*. Na CPU isso é um `max` no laço; do lado da GPU tem de viajar no uniform, senão o plano de
/// relevo sobe cheio de dente e o shader o multiplica por zero — **um plano certo, apagado no device**.
///
/// **Mutação que tem de sangrar:** devolver `paper_body` a `0` no uniforme.
#[test]
fn the_papers_presence_reaches_the_gpu_producer_too() {
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let planes = t
        .impasto_gpu_planes_in((0, 0, N, N))
        .expect("planos com substrato ligado");
    assert!(
        planes.paper_body > 0.5,
        "o uniform não diz ao shader que há papel — o dente sobe e é multiplicado por cobertura zero"
    );
    let off = blank()
        .impasto_gpu_planes_in((0, 0, N, N))
        .map_or(0.0, |p| p.paper_body);
    assert!(
        off < 0.5,
        "sem substrato o uniform ainda declara papel — a presença deixaria de ser uma exceção e viraria \
         a regra, acendendo relevo de tinta sobre cobertura zero"
    );
}

/// Uma tela já drenada uma vez, com um retângulo pequeno sujo — o estado em que o artista está
/// **enquanto pinta**: existe composição em cache e existe um confinamento válido.
fn mid_stroke() -> PainterTool {
    let mut t = blank();
    t.set_substrate_depth(0.5);
    let _ = t.take_preview_dirty(); // a primeira drenagem registra a chave (não invalida nada)
    t.mark_dirty(Region {
        x: 8,
        y: 8,
        w: 12,
        h: 12,
    });
    t
}

/// ⚠️ **OS RETÂNGULOS — report do Enio, 2026-08-10 (com foto):** *"aquelas áreas retangulares além do
/// ponto do pincel … elas marcam os retângulos por onde passa o pincel"*.
///
/// A causa não é do desenho do dente: é do **confinamento**. O produto declara a lei em dois lugares,
/// e os dois estavam certos — o doc do `preview_gpu_region` (*"`None` significa que a mudança não foi
/// confinada"*) e o do fold parcial de planos (*"correto exatamente quando o relevo composto FORA da
/// região não mudou"*). Um Relief novo muda o relevo composto em **todo** texel, e nenhum dos knobs
/// que o produzem derrubava o rect: o device então recebia o dente novo só dentro do rastro do pincel
/// e ficava com o dente velho em volta. A fronteira entre as duas eras É o retângulo da foto.
///
/// **Mutação que tem de sangrar:** tirar o `reconcile_substrate()` do topo do `take_preview_dirty`.
#[test]
fn a_substrate_change_is_never_reported_as_a_confined_rect() {
    let mut t = mid_stroke();
    t.set_substrate_depth(0.9);
    assert!(t.take_preview_dirty(), "a mudança tem de sujar a preview");
    assert_eq!(
        t.preview_gpu_region(),
        None,
        "o substrato mudou na tela INTEIRA e a pista GPU declarou a mudança CONFINADA ao rastro do \
         pincel — o fold parcial então dobra o dente novo só ali, e a fronteira com o dente velho é o \
         retângulo que o artista fotografou"
    );
}

/// ⚠️ **O PAPEL NÃO ATUALIZAVA EM TEMPO REAL — o segundo report, e é o MESMO mecanismo.** *"Ao
/// aumentar o valor de Relief o papel não é atualizado em tempo real, mas só ao usar o pincel."*
///
/// Um knob de substrato não levanta o `preview_dirty` sozinho, então a tela ficava exatamente como
/// estava até a pincelada seguinte — que é quando o artista finalmente via o papel, **e em retângulos**.
/// O oráculo aqui é o do ARTISTA: a drenagem tem de produzir uma tela, e uma tela DIFERENTE.
///
/// **Mutação que tem de sangrar:** tirar o `reconcile_substrate()` do topo do `take_preview_arc`.
#[test]
fn raising_the_relief_repaints_without_a_stroke() {
    let mut t = blank();
    t.set_substrate_depth(0.2);
    let (before, _, _) = t
        .take_preview_arc()
        .expect("a primeira drenagem publica a tela");
    let before = before.to_vec();

    t.set_substrate_depth(1.0);
    let (after, _, _) = t
        .take_preview_arc()
        .expect("subir o Relief tem de repintar SEM uma pincelada — foi o report do Enio");
    assert_ne!(
        after.to_vec(),
        before,
        "a tela saiu byte-idêntica depois de o Relief mudar: o papel só apareceria na próxima pincelada"
    );
}

/// ⚠️ **A TESTEMUNHA existe porque a LISTA apodrece — e este gate é a lista.**
///
/// Os knobs que alimentam o dente são nove, e **sete deles moram num arquivo sobre a aquarela**
/// (`watercolor_settings`), escrito quando o papel só temperava o pigmento molhado e não tinha como
/// mudar um pixel fora da pincelada. Uma linha de `invalidate_composite` por setter é a regra que o
/// décimo nasce sem — e o passo 4 do roteiro do smoke manda o artista mexer justamente nestes.
///
/// **Mutação que tem de sangrar:** chavear a testemunha só em `depth`/`rough` (a chave "óbvia"),
/// deixando o slot Paper de fora — sete das oito linhas abaixo ficam vermelhas.
/// Um knob do papel: o nome que o gate imprime e o gesto que o move.
type PaperKnob = (&'static str, fn(&mut PainterTool));

#[test]
fn every_paper_knob_the_tooth_reads_drops_the_confinement() {
    let knobs: [PaperKnob; 8] = [
        ("Relief", |t| t.set_substrate_depth(0.9)),
        ("Roughness", |t| t.set_substrate_roughness(0.9)),
        ("Paper kind", |t| {
            t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperRough.to_u8());
        }),
        ("Paper Size", |t| t.set_brush_paper_size(0, 3.0)),
        ("Paper Angle", |t| t.set_brush_paper_angle(37.0)),
        ("Paper Offset", |t| t.set_brush_paper_offset(0, 0.25)),
        ("Paper Contrast", |t| t.set_brush_paper_param(0, 0.9)),
        ("Paper reset", |t| t.reset_brush_paper()),
    ];
    for (name, turn) in knobs {
        let mut t = mid_stroke();
        turn(&mut t);
        assert!(
            t.take_preview_dirty(),
            "{name}: a mudança tem de sujar a preview"
        );
        assert_eq!(
            t.preview_gpu_region(),
            None,
            "{name} muda o dente na tela inteira e saiu declarado como confinado ao rastro do pincel"
        );
    }
}

/// ⚠️ **E TROCAR A IMAGEM tem de contar também — o buraco que a MUTAÇÃO achou.**
///
/// Re-escolher outra camada como papel deixa o slot exatamente igual (o `kind` já é `Image`, a Size já
/// é a que estava): o único fato que muda são os PIXELS. Uma chave que só olha para o
/// `TextureSettings` acha que nada mudou — e a mutação *"tirar a versão da imagem da chave"*
/// sobrevivia à varredura inteira dos oito knobs, porque todos eles mexem no `TextureSettings`.
///
/// A versão é MONOTÔNICA e nunca o endereço do buffer, o cuidado que o ADR-0124 pagou: um `Arc` novo
/// pode nascer no endereço de um morto, e aí a comparação diria *"a mesma imagem"* sobre outra.
#[test]
fn re_picking_the_paper_image_drops_the_confinement_too() {
    let (iw, ih) = (16u32, 16u32);
    let a: Vec<u8> = (0..iw * ih)
        .map(|i| if (i % 4) < 2 { 0 } else { 255 })
        .collect();
    let b: Vec<u8> = (0..iw * ih)
        .map(|i| if (i % 8) < 4 { 0 } else { 255 })
        .collect();

    let mut t = blank();
    t.use_layers_as_watercolor_paper(a, iw, ih);
    t.set_substrate_depth(0.5);
    let _ = t.take_preview_dirty(); // registra a chave
    t.mark_dirty(Region {
        x: 8,
        y: 8,
        w: 12,
        h: 12,
    });

    t.use_layers_as_watercolor_paper(b, iw, ih); // o slot fica IGUAL; só os pixels mudam
    assert!(
        t.take_preview_dirty(),
        "trocar a imagem tem de sujar a preview"
    );
    assert_eq!(
        t.preview_gpu_region(),
        None,
        "o papel virou outro e a mudança saiu declarada como confinada ao rastro do pincel"
    );
}

/// ⚠️ **UM CONTROLE MORTO QUE EU SHIPEI: o papel do artista não chegava ao dente.**
///
/// O gesto *"Use as Brush Paper"* carrega uma imagem em `paint.paper_image`, e os outros dois
/// consumidores do slot (o `watercolor_render` e o `dab_route` do Wet Paint) sempre a passaram ao
/// `texture::sample`. O substrato passava `None` — e o `patterns::sample_kind` responde a isso com
/// `1.0` (*"kind is Image but no pixels supplied → inert"*), um dente **CONSTANTE**. Gradiente zero é
/// tela chata: o artista escolhia o próprio papel digitalizado e o slider Relief não movia um pixel.
///
/// ⚠️ A porta é a do menu da Hierarquia, que ainda se chama *"Use as Watercolor **Paper**"* e ARMA a
/// aquarela — herança de quando o papel só temperava o pigmento molhado. Isso é adjacência de PRODUTO
/// (o slot virou medium-agnóstico nesta wave, o rótulo não), reportada e não contrabandeada aqui.
///
/// **Mutação que tem de sangrar:** devolver o `None` ao `texture::sample` do `tooth_px`.
#[test]
fn a_paper_image_is_the_paper_the_artist_chose() {
    let mut t = blank();
    // Um papel de listras FINAS — e o período é parte da fixture, não decoração. O slot é `Tiled`
    // (`u = px / 256`), então numa tela de 96 px em Size 1 só **37,5%** da tile é visitada: a primeira
    // versão deste gate desenhou uma listra do tamanho da imagem e mediu `0` níveis sobre um produto
    // JÁ CORRIGIDO — a janela inteira caía dentro da metade escura. Uma fixture tem de conter o
    // fenômeno, e aqui isso quer dizer *mais de um período dentro da janela amostrada*.
    let (iw, ih) = (16u32, 16u32);
    let lum: Vec<u8> = (0..iw * ih)
        .map(|i| if (i % 4) < 2 { 0 } else { 255 })
        .collect();
    t.use_layers_as_watercolor_paper(lum, iw, ih);
    t.set_brush_paper_size(0, 4.0);
    t.set_brush_paper_size(1, 4.0);
    t.set_substrate_depth(1.0);
    let e = excursion(&lit(&t));
    assert!(
        e >= 6,
        "o papel que o artista carregou não alcança o dente (excursão {e} níveis) — o slot é uma \
         imagem e o substrato amostra sem ela, então o dente sai constante e a luz não desenha nada"
    );
}

/// SONDA — a **calibração** de [`super::substrate_relief::MAX_TOOTH_PX`] e a leitura contra o alvo.
///
/// Rodar: `cargo test -p ph2d-tool-painter probe_substrate_depth_ladder -- --ignored --nocapture`
#[test]
#[ignore = "sonda de calibracao: imprime a escada, nao afirma um bar"]
fn probe_substrate_depth_ladder() {
    println!("\n=== excursao de luminancia do dente, por Depth e Roughness ===");
    for d in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let mut row = format!("depth {d:.2}: ");
        for r in [0.0f32, 0.5, 1.0] {
            let mut t = blank();
            t.set_substrate_depth(d);
            t.set_substrate_roughness(r);
            row.push_str(&format!(
                "rough {r:.1} -> {:>3} niveis   ",
                excursion(&lit(&t))
            ));
        }
        println!("{row}");
    }
    println!();
}

/// ⚠️ **O Ctrl+Z devolve a TINTA com o papel ligado** — a metade do report #3 que É deste módulo.
///
/// O substrato liga o passe de luz num documento que antes não o tinha (`impasto_visible` passa a ser
/// verdadeiro sem uma pincelada), e o passe de luz é o que decide qual PRODUTOR compõe a tela. Um undo
/// que voltasse por outro produtor deixaria resíduo — então o oráculo é o byte, não a impressão.
///
/// ⚠️ **A outra metade do report NÃO é deste módulo e não foi construída:** o Relief e a Roughness são
/// knobs de FERRAMENTA, e nenhum knob de ferramenta deste app entra na fila de undo — medido, o card
/// **Lighting** do impasto (que shipou e foi smokado) se comporta exatamente igual. Ver a
/// [`probe_substrate_undo`], que põe os dois lado a lado.
#[test]
fn painting_over_the_paper_still_undoes_to_the_byte() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let clean: Vec<u8> = (*t.canvas_rgba).clone();
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([20.0, 48.0], PointerPhase::Down));
    for k in 1..=5u8 {
        t.on_canvas_pointer(cp([20.0 + f32::from(k) * 10.0, 48.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([70.0, 48.0], PointerPhase::Up));
    let painted: Vec<u8> = (*t.canvas_rgba).clone();
    assert!(
        clean.iter().zip(&painted).any(|(a, b)| a != b),
        "a fixture nao pintou nada — o gate nao testaria o undo de coisa nenhuma"
    );

    assert!(t.undo_last(), "o traco tem de estar na pilha de undo");
    assert_eq!(
        *t.canvas_rgba, clean,
        "com o papel ligado o Ctrl+Z deixou residuo na tela"
    );
}

/// SONDA — **o que o Ctrl+Z faz com o papel ligado?** (report #3 do Enio, 2026-08-10)
///
/// Rodar: `cargo test -p ph2d-tool-painter probe_substrate_undo -- --ignored --nocapture`
#[test]
#[ignore = "sonda de reproducao do report de undo"]
fn probe_substrate_undo() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }
    fn stroke(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([20.0, 48.0], PointerPhase::Down));
        for k in 1..=5u8 {
            t.on_canvas_pointer(cp([20.0 + f32::from(k) * 10.0, 48.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([70.0, 48.0], PointerPhase::Up));
    }

    // (a) a TINTA volta com o papel ligado?
    let mut t = blank();
    t.set_substrate_depth(1.0);
    let clean: Vec<u8> = (*t.canvas_rgba).clone();
    stroke(&mut t);
    let painted: Vec<u8> = (*t.canvas_rgba).clone();
    let moved = clean.iter().zip(&painted).filter(|(a, b)| a != b).count();
    let undone = t.undo_last();
    let after: Vec<u8> = (*t.canvas_rgba).clone();
    let residual = clean.iter().zip(&after).filter(|(a, b)| a != b).count();
    println!(
        "\n(a) TINTA com papel ligado: {moved} bytes pintados, undo_last={undone}, residual apos undo={residual}"
    );

    // (b) o RELIEF em si e desfazivel?
    let mut t2 = blank();
    stroke(&mut t2);
    t2.set_substrate_depth(1.0);
    let ok = t2.undo_last();
    println!(
        "(b) RELIEF: subi para 1.00; undo_last={ok} deixou o Relief em {:.2}",
        t2.substrate_depth()
    );

    // (c) e o irmao mais velho — a LUZ do impasto — e desfazivel?
    let mut t3 = blank();
    stroke(&mut t3);
    t3.set_impasto_light_angle(123.0);
    let ok3 = t3.undo_last();
    println!(
        "(c) LIGHT ANGLE (o precedente): girei para 123; undo_last={ok3} deixou o angulo em {}",
        t3.paint.impasto_rig.current().angle_deg
    );
    println!();
}
