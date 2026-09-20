//! **OS GATES DOS CANAIS** — a máscara e a cor, cada uma a desfazer-se por si.
//!
//! ⚠️ **Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO mais o tecto de
//! LOC** (`711` contra `700`, quando o terceiro canal chegou): lá moram os
//! gates da GEOMETRIA e das operações de malha inteira, aqui os dos dois canais
//! que um traço escreve **sem mover um vértice**.
//!
//! ⛔ Uma entrada no `FILE_OVERAGE_OK` não era saída (CLAUDE.md §5.0).

use ph2d_sculpt3d::Verb;

use super::tests::{one_dab, scene};

/// Abre a GPU, ou diz que não há nada a afirmar. (Cópia local dos irmãos: um
/// macro exportado entre módulos de teste seria acoplamento por conveniência —
/// a mesma razão escrita no [`super::tests`], de onde esta cópia veio quando o
/// tecto de LOC partiu aquele ficheiro.)
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

/// ⭐⭐ **UM TRAÇO DE PINTURA DESFAZ, E AVISA A TELA** — o gémeo do irmão de
/// cima, e ele nasceu com a chegada do terceiro canal (2026-09-19).
///
/// ⚠️ **A pergunta que o registo faz é sobre a JANELA e não sobre o VERBO**
/// ([`super::super::history`]`::color_window_changed`), pela lição que a máscara
/// já pagou: com o `Paint` na mão um gesto de FILTRO escreve POSIÇÕES, e uma
/// entrada gravada como se fosse cor trocaria o canal errado no `Ctrl+Z`.
///
/// ⚠️ **O pincel pinta PRETO de propósito:** o `DEFAULT_COLOR` é **branco**, e
/// um pincel branco sobre barro branco escreve um no-op perfeito — *um corpus no
/// ponto neutro de um canal não testa esse canal*.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_paint_stroke_undoes_and_tells_the_screen() {
    let gpu = gpu_or_skip!();
    let mut s = scene(&gpu.device, Verb::Paint);
    s.brush.color = [0.0, 0.0, 0.0];
    assert!(
        s.mesh().colors().is_none(),
        "premissa: a malha nasce sem plano de cor"
    );

    one_dab(&mut s);
    let pintados = s
        .mesh()
        .colors()
        .expect("o traço de pintura não criou o plano")
        .iter()
        .filter(|c| **c != ph2d_mesh::DEFAULT_COLOR)
        .count();
    assert!(
        pintados > 0,
        "premissa: o dab não escreveu cor nenhuma, e sem isso o desfazer não tem o que desfazer"
    );

    s.objects[s.active].uploaded = true;
    let edits = s.edits;

    assert!(
        s.undo_stroke(),
        "o traço de pintura não gravou passo de undo"
    );
    let ainda = s
        .mesh()
        .colors()
        .expect("o plano não devia ter sido removido")
        .iter()
        .filter(|c| **c != ph2d_mesh::DEFAULT_COLOR)
        .count();
    assert_eq!(
        ainda, 0,
        "o Ctrl+Z deixou {ainda} vertices pintados: a janela do traco nao cobria o que ele pintou"
    );
    assert!(
        !s.objects[s.active].uploaded,
        "o desfazer nao invalidou o upload: a cor voltou na memoria e a tela mostra a de antes"
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
