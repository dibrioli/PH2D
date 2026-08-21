//! **OS DOIS CANAIS DE UMA ENTRADA DE TRAÇO** — o braço `StrokeUndo::Stroke`,
//! medido no produto.
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! `sculpt3d_filter_tests`: um gesto exige uma cena e uma cena exige um device,
//! então estes gates são `#[ignore]` + `gpu_or_skip!`.
//!
//! ⚠️ **A família inteira do undo tinha UM teste** antes desta wave — o
//! `swap_window` solto, no `mod tests` do pai (`sculpt3d_history.rs`), que é uma
//! unidade pura. Os catorze braços que a [`super::Sculpt3dScene`] aplica só eram
//! tocados pelo gate do filtro, e é por isso que este arquivo nasce: o canal de
//! MÁSCARA de um traço não tinha um único chamador a prová-lo, e ele estava
//! errado em duas metades ao mesmo tempo (o registo perguntava ao verbo, e o
//! desfazer não avisava a tela).
//!
//! ⚠️ **O caminho de módulo é `sculpt3d::history::undo`, e não `sculpt3d::undo`**
//! — o [`super`] é filho do `sculpt3d_history.rs`. Um filtro com o nome errado
//! devolve `0 passed`, que **não é verde: é nada rodou**.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::history::undo::tests -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::Verb;

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar. (Cópia local dos irmãos: um
/// macro exportado entre módulos de teste seria acoplamento por conveniência.)
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// O meio do viewport — onde a esfera enquadrada está, e por onde o dedo entra.
const CENTRE: (f32, f32) = (450.0, 350.0);

/// Uma cena com uma esfera e o verbo pedido em mãos.
fn scene(device: &wgpu::Device, verb: Verb) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, uv_sphere(24, 36, 1.0), 1.0);
    s.viewport = (900, 700);
    s.brush.verb = verb;
    s
}

/// Um traço de UM dab no meio da peça — a sequência que o braço de escultura do
/// `sculpt3d_pointer_down` percorre (`aim` → `stroke.begin` → `sculpt_at`,
/// `sculpt3d_input.rs:172-202`), sem o `App` que ela precisaria para existir.
fn one_dab(s: &mut Sculpt3dScene) {
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peca enquadrada");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    assert!(
        s.sculpt_at(CENTRE.0, CENTRE.1),
        "o dab nao pegou a malha: a fixture nao contem o fenomeno"
    );
    s.close_stroke();
}

/// **UM TRAÇO DE MÁSCARA DESFAZ, E AVISA A TELA.**
///
/// ⚠️ **As duas metades falhavam por motivos diferentes**, e é por isso que uma
/// asserção só não bastaria:
///
/// - o canal voltava (o braço trocava as máscaras), mas
/// - o braço **não chamava `mesh_rebuilt()` nem tocava o `uploaded`** — então a
///   máscara voltava na memória e a GPU continuava a mostrar a de antes. O
///   irmão [`super::super::StrokeUndo::Mask`] (a operação de plano inteiro) faz
///   exactamente esse par um degrau acima; o traço não fazia.
///
/// ⚠️ **O que volta é o VALOR neutro, não a AUSÊNCIA do plano.** `masks_mut()`
/// cria o plano ao ser tocado, e desfazer um traço nunca o remove — quem remove
/// é o `StrokeUndo::Mask { before: None }`, que é outra entrada e outro gesto.
/// Afirmar `masks().is_none()` aqui seria pinar a lei do vizinho.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_mask_stroke_undoes_and_tells_the_screen() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Mask);
    assert!(
        s.mesh().masks().is_none(),
        "premissa: a malha nasce sem plano de mascara"
    );

    one_dab(&mut s);
    let painted = s
        .mesh()
        .masks()
        .expect("o traco de mascara nao criou o plano")
        .iter()
        .filter(|m| **m != ph2d_mesh::DEFAULT_MASK)
        .count();
    assert!(
        painted > 0,
        "premissa: o dab nao escreveu mascara nenhuma, e sem isso o desfazer nao tem o que desfazer"
    );

    s.objects[s.active].uploaded = true;
    let edits = s.edits;

    assert!(
        s.undo_stroke(),
        "o traco de mascara nao gravou passo de undo"
    );
    let still = s
        .mesh()
        .masks()
        .expect("o plano nao devia ter sido removido")
        .iter()
        .filter(|m| **m != ph2d_mesh::DEFAULT_MASK)
        .count();
    assert_eq!(
        still, 0,
        "o Ctrl+Z deixou {still} vertices mascarados: a janela do traco nao cobria o que ele pintou"
    );
    assert!(
        !s.objects[s.active].uploaded,
        "o desfazer nao invalidou o upload: a mascara voltou na memoria e a tela mostra a de antes"
    );
    assert_ne!(
        s.edits, edits,
        "o desfazer nao contou como edicao: a doacao ao Painter serve um carimbo velho"
    );
}

/// **UM TRAÇO DE GEOMETRIA NÃO INVENTA UM PLANO DE MÁSCARA.**
///
/// ⚠️ **É o CONTROLE do irmão acima, e ele mede a metade que o report escondia:**
/// enquanto o registo perguntava *"o verbo pinta máscara?"*, um gesto de FILTRO
/// com o pincel `Mask` em mãos gravava máscaras que ninguém pintou — e desfazer
/// **criava** o plano (`masks_mut()` o instala com [`ph2d_mesh::DEFAULT_MASK`])
/// numa malha que nunca teve um. Aqui o verbo é o `Draw`: a pergunta certa é
/// sobre a JANELA, e a resposta dela não pode mudar por causa do pincel.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_geometry_stroke_leaves_the_mask_channel_alone() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Draw);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    one_dab(&mut s);
    assert_ne!(
        s.mesh().positions(),
        &before[..],
        "premissa: o dab nao moveu a malha"
    );

    assert!(s.undo_stroke(), "o traco nao gravou passo de undo");
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "o Ctrl+Z nao devolveu a geometria"
    );
    assert!(
        s.mesh().masks().is_none(),
        "o desfazer INVENTOU um plano de mascara numa malha que nunca teve um"
    );
}

/// ⭐ **A RETOPOLOGIA MUDA A MALHA, DÁ QUADS, E DESFAZ INTEIRA.**
///
/// O gesto da Q5 do [ADR-0160], pela porta que o botão do painel chama.
///
/// ⚠️ **As três metades juntas de propósito.** Um remesh que rodasse e não
/// mudasse nada passaria por *"não deu erro"*; um que desse triângulos passaria
/// por *"mudou"*; e um que não gravasse undo só se descobre quando o artista
/// perde o trabalho. A terceira é a que este arquivo existe para guardar: a
/// retopologia entra na história pela MESMA entrada do voxel remesh
/// (`StrokeUndo::Remeshed`, que carrega a malha inteira de antes), porque um
/// remesh não partilha estrutura nenhuma com o que estava lá.
/// ⭐⭐ **O BOTÃO ENTREGA A PROMESSA DA CADEIA GLOBAL** — 100 % de quads, casca
/// fechada, e a contagem de irregulares perto do chão topológico.
///
/// ⚠️ **É o gate da PORTA e não do algoritmo.** A cadeia já é gateada, sem GPU, na
/// `ph2d-quadfill`; o que só se pode afirmar aqui é que **o gesto do artista**
/// chega àquela cadeia — que o `detail` do painel atravessa, que a peça é
/// substituída e que o Ctrl+Z a devolve.
///
/// ⚠️ **A barra dos quads é `100 %` e não *"a maioria"***, ao contrário do irmão
/// local: é a promessa inteira desta família de algoritmos, e afrouxá-la seria
/// deixar de medir a única coisa que o pivô comprou.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_button_delivers_the_global_chain() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(
        &gpu.device,
        crate::sculpt3d::fixtures::wrinkled_sphere(),
        1.0,
    );
    s.viewport = (900, 700);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    let r = s
        .quad_remesh_global(0.5)
        .expect("a cadeia global correu sobre uma peca de um nivel");
    eprintln!(
        "[sculpt3d] GLOBAL: {} vertices, {} quads, {} nao-quads, {} irregulares, bordo {}, \
         quad de {:.4}, {:.0} ms",
        r.verts, r.quads, r.non_quads, r.irregular, r.holes, r.edge, r.ms
    );

    assert_eq!(
        r.non_quads, 0,
        "sairam {} faces que nao sao quads",
        r.non_quads
    );
    assert!(r.quads > 0, "a cadeia devolveu uma malha vazia");
    // ⚠️ Uma aresta com UMA face só é a assinatura da casca rasgada, e nenhum
    // render a mostra.
    assert_eq!(
        r.holes, 0,
        "a casca saiu ABERTA: {} arestas de bordo",
        r.holes
    );
    // ⭐ O chão topológico de uma esfera é 8. A barra é `3×` — a mesma da
    // `ph2d-quadfill`, e pela mesma razão: é a CONTAGEM que é estrutural, e
    // medi-la em percentagem faz o número melhorar sozinho com a densidade.
    assert!(
        (8..=24).contains(&r.irregular),
        "{} vertices irregulares: o chao de uma esfera e' 8 e a barra e' 24",
        r.irregular
    );
    assert_ne!(
        s.mesh().positions(),
        &before[..],
        "a retopologia global nao mudou a malha"
    );
    assert!(
        s.undo_stroke(),
        "a cadeia global nao gravou passo de undo: o artista perde a peca"
    );
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "o Ctrl+Z nao devolveu a malha de antes"
    );
}

#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_quad_retopology_changes_the_mesh_and_undoes_whole() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Draw);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    let before_faces = s.mesh().faces().len();

    let r = s
        .quad_remesh(0.5, 0.0)
        .expect("a retopologia correu sobre uma peca de um nivel");
    let (verts, quads, non_quads) = (r.verts, r.quads, r.non_quads);
    eprintln!(
        "[sculpt3d] retopologia: {verts} vertices, {quads} quads, {non_quads} nao-quads, quad de \
         {:.4}, {:.0} ms",
        r.edge, r.ms
    );

    assert!(verts > 0, "a retopologia devolveu uma malha vazia");
    assert!(
        quads > non_quads,
        "sairam {quads} quads contra {non_quads} nao-quads: a maioria das faces tinha de ter \
         QUATRO lados, senao isto e' uma triangulacao com outro nome"
    );
    assert_ne!(
        s.mesh().positions(),
        &before[..],
        "a retopologia nao mudou a malha"
    );

    assert!(
        s.undo_stroke(),
        "a retopologia nao gravou passo de undo: o artista perde a peca"
    );
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "o Ctrl+Z nao devolveu a malha de antes"
    );
    assert_eq!(
        s.mesh().faces().len(),
        before_faces,
        "o Ctrl+Z devolveu outra contagem de faces"
    );
}

/// **COM A PILHA MONTADA ELA RECUSA, e diz o que fazer.**
///
/// ⚠️ **A recusa é a mesma do voxel remesh e pelo mesmo motivo:** a saída tem
/// outra contagem de vértices, e um nível de multires é uma subdivisão da base —
/// as duas coisas não coexistem. Um gesto que não faz nada e não diz nada é
/// indistinguível de um botão que não chegou.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_quad_retopology_refuses_a_multires_stack() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Draw);
    assert!(s.subdivide(), "premissa: a peca subdividiu");
    assert!(
        s.quad_remesh(0.5, 0.0).is_err(),
        "a retopologia correu com a pilha montada: os indices da saida nao nomeiam o nivel de baixo"
    );
}

/// ⭐ **O CURSO INTEIRO DO `Detail` DEVOLVE UMA PEÇA** — o gate que a foto pediu.
///
/// ⚠️ **É a versão executável do passo (5) do smoke.** O defeito de 2026-08-19
/// não morava numa posição do slider: morava em o slider oferecer posições que a
/// malha não tem como responder. Um gate que testasse um valor só continuaria
/// verde sobre exatamente o mesmo defeito — foi o que aconteceu, com `0,18`.
///
/// Cinco pontos do curso, e em cada um as três propriedades que o artista de
/// facto vê: a peça **existe**, ela é **fechada** (nenhuma aresta com uma face
/// só), e ela aponta **para fora** (volume com sinal positivo).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn every_point_of_the_detail_slider_returns_a_piece() {
    use std::collections::BTreeMap;
    let gpu = gpu_or_skip!();
    let mut counts: Vec<usize> = Vec::new();
    // ⚠️ **A GRELHA DOS DOIS KNOBS, e não a varredura de UM.** O botão passa
    // dois (`sculpt3d_panel.rs`: `quad_remesh(quad_detail, quad_adapt)`), e os
    // três testes que chamavam `quad_remesh` pinavam o segundo em `0.0` — que é
    // exatamente o único valor em que `ScaleField::adaptive` sai
    // antecipadamente e o eixo não é exercitado. Medido pela auditoria de
    // 2026-08-19: **9 das 25 células** desta grelha devolviam a peça
    // esburacada, e o gate de um eixo ficava verde sobre todas elas.
    //
    // ⚠️ E o roteiro do smoke manda visitar o pior canto: o passo (5) põe o
    // `Detail` na ponta e o passo (6) manda pôr o `Follow Curvature` em 1,0.
    for step in 0..=24u32 {
        let detail = (step / 5) as f32 / 4.0;
        let adapt = (step % 5) as f32 / 4.0;
        // ⚠️ **A MALHA DA CENA `=35`, AMASSADA — e não a esfera lisa.** Duas
        // correções aqui, e a segunda foi paga em foto:
        //
        // 1. a `uv_sphere(24,36)` das irmãs tem 830 vértices, e nela o piso já
        //    passa o teto: a faixa legal colapsa num ponto e o slider inteiro
        //    devolve a mesma malha (medido: `0,3701` nos cinco pontos);
        // 2. ⚠️ **a esfera LISA não contém o fenômeno.** A cena abre amassada, e
        //    foi na amassada que o smoke devolveu a peça esburacada
        //    (Enio, 2026-08-19). Um gate sobre a lisa fica verde sobre
        //    exatamente o defeito que o artista vê. *A fixtura só prova o que
        //    ela contém.*
        let mut s = Sculpt3dScene::new(
            &gpu.device,
            crate::sculpt3d::fixtures::wrinkled_sphere(),
            1.0,
        );
        s.viewport = (900, 700);
        let r = s.quad_remesh(detail, adapt).unwrap_or_else(|e| {
            panic!("detail={detail:.2} adapt={adapt:.2}: a retopologia recusou ({e:?})")
        });

        let mesh = s.mesh();
        assert!(
            r.verts > 0 && !mesh.faces().is_empty(),
            "detail={detail:.2} adapt={adapt:.2}: a peca sumiu -- a extracao devolveu malha vazia"
        );

        let mut undirected: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        let mut volume = 0.0f64;
        let pos = mesh.positions();
        for f in mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                *undirected
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_insert(0) += 1;
            }
            for k in 1..v.len() - 1 {
                let (a, b, c) = (
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                volume += f64::from(a[0].mul_add(
                    b[1].mul_add(c[2], -(b[2] * c[1])),
                    a[1].mul_add(
                        b[2].mul_add(c[0], -(b[0] * c[2])),
                        a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                    ),
                )) / 6.0;
            }
        }
        let open = undirected.values().filter(|c| **c == 1).count();
        eprintln!(
            "[sculpt3d] detail={detail:.2} adapt={adapt:.2}: quad {:.4}, {} vertices, {:.1}% quads, volume {volume:+.3}, borda {open}",
            r.edge,
            r.verts,
            100.0 * r.quads as f64 / (r.quads + r.non_quads).max(1) as f64
        );
        assert_eq!(
            open, 0,
            "detail={detail:.2} adapt={adapt:.2}: {open} arestas com UMA face -- a peca saiu \
             esburacada, e e' isto que o artista fotografa"
        );
        assert!(
            volume > 0.0,
            "detail={detail:.2} adapt={adapt:.2}: volume com sinal {volume:+.3} -- do avesso"
        );
        // ⚠️ **UMA componente, e a asserção é nova.** Uma peça pode ser fechada e
        // orientada em CADA pedaço e ainda assim ter-se partido em quinze: foi o
        // que a grelha mediu no canto `1,00`/`1,00`. Nenhuma das outras três
        // asserções vê isso.
        let mut seen = vec![false; mesh.vert_count()];
        let mut stack = vec![0u32];
        let mut reached = 0usize;
        if !mesh.faces().is_empty() {
            seen[0] = true;
            while let Some(v) = stack.pop() {
                reached += 1;
                for (a, b) in undirected.keys() {
                    let w = if *a == v {
                        *b
                    } else if *b == v {
                        *a
                    } else {
                        continue;
                    };
                    if !seen[w as usize] {
                        seen[w as usize] = true;
                        stack.push(w);
                    }
                }
            }
        }
        assert_eq!(
            reached,
            mesh.vert_count(),
            "detail={detail:.2} adapt={adapt:.2}: a peca partiu-se -- so' {reached} dos {} \
             vertices estao ligados ao primeiro",
            mesh.vert_count()
        );
        if adapt == 0.0 {
            counts.push(r.verts);
        }
    }
    // ⚠️ **E O KNOB TEM DE FAZER ALGUMA COISA.** Um slider que devolve a mesma
    // malha nos cinco pontos passa em todas as asserções acima — a peça existe,
    // é fechada e aponta para fora — e ainda assim é um controle morto. Foi
    // exatamente o que a primeira corrida deste gate mostrou.
    assert!(
        counts[4] > counts[0] * 2,
        "o curso do Detail nao muda a densidade: {} vertices no grosso contra {} no fino -- o \
         knob esta' MORTO (faixa legal colapsada?)",
        counts[0],
        counts[4]
    );
}

/// ⭐ **DOIS CLIQUES SEM DESFAZER AINDA DEVOLVEM UMA PEÇA** — porque é isso que o
/// roteiro do smoke manda fazer.
///
/// ⚠️ **Nenhum dos três testes chamava `quad_remesh` duas vezes**, e o roteiro da
/// cena `=35` pede exatamente isso: o passo (5) manda arrastar o `Detail` e
/// clicar, e o passo (6) manda mudar o `Follow Curvature` e comparar — sem
/// `Ctrl+Z` no meio.
///
/// O que a composição faz: o lado do quad sai da malha de **ENTRADA**
/// (`edge_for_detail`), e a saída **substitui** a entrada. Então o segundo clique
/// lê a malha do primeiro, cuja aresta média é muito maior — e o piso
/// (`3 × aresta`) sobe atrás dela. A peça tem de continuar a fechar em cada
/// degrau, e é só isso que este gate afirma: *cada clique devolve uma casca*.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn two_clicks_without_undo_still_return_a_piece() {
    use std::collections::BTreeMap;
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(
        &gpu.device,
        crate::sculpt3d::fixtures::wrinkled_sphere(),
        1.0,
    );
    s.viewport = (900, 700);

    for click in 1..=3u32 {
        let r = s
            .quad_remesh(1.0, 1.0)
            .unwrap_or_else(|e| panic!("clique {click}: a retopologia recusou ({e:?})"));
        let mesh = s.mesh();
        let mut undirected: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for f in mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                *undirected
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_insert(0) += 1;
            }
        }
        let open = undirected.values().filter(|c| **c == 1).count();
        eprintln!(
            "[sculpt3d] clique {click}: quad {:.4}, {} vertices, {} buracos reportados, borda {open}",
            r.edge, r.verts, r.holes
        );
        assert!(r.verts > 0, "clique {click}: a peca sumiu");
        assert_eq!(
            open, 0,
            "clique {click}: {open} arestas com UMA face -- o segundo remesh sobre a saida do \
             primeiro esburacou a peca"
        );
        // ⚠️ **O relatório tem de CONCORDAR com a malha.** Se o número de buracos
        // que o log imprime divergir do que a malha tem, o log é pior que
        // nenhum: ele afirma uma casca fechada sobre uma peça furada.
        assert_eq!(
            r.holes, 0,
            "clique {click}: o relatorio diz {} buracos e a malha tem {open} arestas de borda",
            r.holes
        );
    }
}
