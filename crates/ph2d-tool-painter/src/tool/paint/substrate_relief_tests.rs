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
