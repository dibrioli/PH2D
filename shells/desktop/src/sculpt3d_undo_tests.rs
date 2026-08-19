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
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_quad_retopology_changes_the_mesh_and_undoes_whole() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Draw);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    let before_faces = s.mesh().faces().len();

    let (verts, quads, non_quads, ms) = s
        .quad_remesh(0.18, 0.0)
        .expect("a retopologia correu sobre uma peca de um nivel");
    eprintln!(
        "[sculpt3d] retopologia: {verts} vertices, {quads} quads, {non_quads} nao-quads, {ms:.0} ms"
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
        s.quad_remesh(0.18, 0.0).is_err(),
        "a retopologia correu com a pilha montada: os indices da saida nao nomeiam o nivel de baixo"
    );
}
