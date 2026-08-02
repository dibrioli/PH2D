//! Os gates da reversão **do lado da pilha** — ver o cabeçalho do
//! `multires_reverse.rs`.

use super::*;
use crate::shapes;
use crate::subdivide::subdivide;

fn stack_of(mesh: Mesh) -> Multires {
    Multires::new(mesh)
}

/// A malha densa das fixtures: uma subdivisão **com os vértices embaralhados**.
///
/// ⚠️ **O embaralhamento não é tempero, e a falta dele foi medida.** A
/// `subdivide` numera os originais primeiro, então a reversão de uma subdivisão
/// crua devolve a permutação IDENTIDADE — e a renumeração inteira, inclusive a
/// cascata, fica invisível. Com `subdivide(cube)` cru, a mutação que troca a
/// cascata pela identidade **sobrevivia a todos os gates deste arquivo**. E é a
/// fixture HONESTA: reverter serve a um arquivo de terceiro, que não ordena
/// vértice nenhum.
fn imported(levels: usize) -> Mesh {
    let mut m = shapes::cube(1.0);
    for _ in 0..levels {
        m = subdivide(&m);
    }
    shapes::shuffled(&m, 0x5eed)
}

/// ⚠️ **O invariante que a pilha inteira consome, afirmado direto:** o nível `k`
/// é numerado por `subdivide` do `k − 1`. É ele que a reversão poderia quebrar
/// em silêncio — uma malha grossa correta com a numeração errada sobe e desce
/// embaralhada, e nenhum outro gate a distingue de uma certa.
fn assert_stack_is_numbered_by_subdivision(stack: &Multires) {
    for k in 1..stack.level_count() {
        let predicted = subdivide(&stack.levels[k - 1]);
        assert_eq!(
            predicted.vert_count(),
            stack.levels[k].vert_count(),
            "o nível {k} tem a contagem que a subdivisão do de baixo impõe"
        );
        assert_eq!(
            predicted.faces(),
            stack.levels[k].faces(),
            "o nível {k} tem as FACES que a subdivisão do de baixo impõe"
        );
    }
}

#[test]
fn reversing_inserts_a_level_below_and_keeps_the_artist_looking_at_the_same_mesh() {
    let fine = imported(1);
    let before = fine.positions().to_vec();
    let mut stack = stack_of(fine);
    assert!(stack.reverse().is_some());
    assert_eq!(stack.level_count(), 2);
    assert_eq!(stack.level(), 1, "o artista continua na malha que tinha");
    assert_eq!(stack.mesh().vert_count(), before.len());
    assert_stack_is_numbered_by_subdivision(&stack);
}

/// ⚠️ **Reverter MOVE vértices de lugar; não altera nenhum.** A renumeração é
/// uma permutação, então a nuvem de pontos tem de sair byte-idêntica — e é isto
/// que separa *"a malha mudou de ordem"* de *"a malha mudou"*.
#[test]
fn reversing_renumbers_the_fine_mesh_without_touching_a_single_position() {
    let fine = imported(1);
    let before = fine.positions().to_vec();
    let mut stack = stack_of(fine);
    stack.reverse().expect("reverte");
    let mut a = before;
    let mut b = stack.mesh().positions().to_vec();
    let key = |p: &[f32; 3]| (p[0].to_bits(), p[1].to_bits(), p[2].to_bits());
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "renumerar move vértices de lugar, não os altera");
}

/// ⚠️ **O gate que decide a wave.** A malha grossa é uma DECIMAÇÃO (as posições
/// são cópias, não a inversa aritmética), então quem devolve a forma é o
/// detalhe. Descer e subir tem de reproduzir a malha fina — senão reverter
/// custaria ao artista a escultura que ele já tinha.
///
/// ⚠️ **A barra NÃO é o bit, e ela não foi escolhida: é o piso que o módulo já
/// chama de *não se moveu*.** A ida e volta do `encode`/`synthesize` é exata
/// quando o detalhe é ZERO (o caso do `add_level`, e é por isso que o gate irmão
/// dele pode pedir igualdade); aqui o detalhe é grande por construção, e o frame
/// local devolve o deslocamento com um resíduo de **5,96e-8 medido** — que é
/// exatamente o épsilon de `f32`. Pedir o bit seria um gate que falha sobre
/// produto correto; usar o [`super::super::STAMP_FLOOR`] é afirmar a coisa certa,
/// e é *por* o resíduo ficar sob ele que a descida seguinte não carimba nada.
#[test]
fn walking_down_and_up_after_a_reversal_reproduces_the_fine_mesh() {
    let fine = imported(1);
    let mut stack = stack_of(fine);
    stack.reverse().expect("reverte");
    let permuted = stack.mesh().positions().to_vec();
    let bar = super::super::STAMP_FLOOR * stack.mesh().bounds().longest_edge();
    assert!(stack.lower().is_some());
    assert!(stack.higher());
    let worst = worst_gap(stack.mesh().positions(), &permuted);
    assert!(
        worst < bar,
        "a viagem devolve a malha fina (pior desvio {worst:e}, piso {bar:e})"
    );
}

fn worst_gap(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0, f32::max)
}

#[test]
fn sculpting_on_the_coarse_level_after_a_reversal_moves_the_fine_one() {
    let mut stack = stack_of(imported(1));
    stack.reverse().expect("reverte");
    assert!(stack.lower().is_some());
    let fine_before = {
        stack.higher();
        let p = stack.mesh().positions().to_vec();
        stack.lower();
        p
    };
    stack.mesh_mut().positions_mut()[0][0] += 0.5;
    stack.mesh_mut().rebuild();
    assert!(stack.higher());
    assert_ne!(
        stack.mesh().positions(),
        &fine_before[..],
        "mover a forma grande tem de mover a pele"
    );
}

#[test]
fn undoing_a_reversal_gives_the_stack_back_to_the_bit() {
    let fine = imported(1);
    let before = fine.positions().to_vec();
    let faces_before = fine.faces().to_vec();
    let mut stack = stack_of(fine);
    let rev = stack.reverse().expect("reverte");
    assert!(stack.unreverse(&rev));
    assert_eq!(stack.level_count(), 1);
    assert_eq!(stack.level(), 0);
    assert_eq!(stack.mesh().positions(), &before[..]);
    assert_eq!(stack.mesh().faces(), &faces_before[..]);
}

/// ⚠️ **O refazer não guarda nada, e é isto que o autoriza:** a reversão é
/// função pura da malha e desfazê-la devolve a malha ao bit, então chamá-la de
/// novo dá o MESMO resultado. Se um dia deixar de dar, esta é a linha que
/// sangra — e a cura seria carregar o estado, como o `drop_top` carrega.
#[test]
fn redoing_a_reversal_is_calling_it_again_and_it_lands_on_the_same_stack() {
    let mut stack = stack_of(imported(1));
    let rev = stack.reverse().expect("reverte");
    let first = stack.levels[0].positions().to_vec();
    let first_fine = stack.levels[1].positions().to_vec();
    assert!(stack.unreverse(&rev));
    let again = stack.reverse().expect("reverte de novo");
    assert_eq!(stack.levels[0].positions(), &first[..]);
    assert_eq!(stack.levels[1].positions(), &first_fine[..]);
    assert_eq!(again.perms.len(), rev.perms.len());
}

#[test]
fn reversing_from_a_level_that_is_not_the_base_is_refused() {
    let mut stack = stack_of(shapes::cube(1.0));
    assert!(stack.add_level());
    assert_eq!(stack.level(), 1);
    assert!(stack.reverse().is_none(), "só a partir do nível 0");
}

/// ⚠️ Uma recusa que já tivesse mutado metade da pilha seria pior que nenhuma:
/// a malha ficaria renumerada sem um nível embaixo que explicasse a numeração.
#[test]
fn a_refused_reversal_leaves_the_stack_exactly_as_it_was() {
    let cube = shapes::cube(1.0);
    let before = cube.positions().to_vec();
    let faces = cube.faces().to_vec();
    let mut stack = stack_of(cube);
    assert!(stack.reverse().is_none(), "um cubo não é uma subdivisão");
    assert_eq!(stack.level_count(), 1);
    assert_eq!(stack.mesh().positions(), &before[..]);
    assert_eq!(stack.mesh().faces(), &faces[..]);
}

/// ⚠️ **A CASCATA, e a fixture que a contém.** Com um nível só a permutação do
/// topo nunca é computada, então uma cascata quebrada passaria em todos os
/// gates acima. Aqui a pilha tem dois níveis ANTES da reversão, e o de cima
/// carrega escultura — que é o que distingue *renumerou certo* de *renumerou*.
#[test]
fn reversing_under_a_pyramid_renumbers_every_level_above() {
    let mut stack = stack_of(imported(1));
    assert!(stack.add_level());
    stack.mesh_mut().positions_mut()[3][1] += 0.25;
    stack.mesh_mut().rebuild();
    let sculpted = stack.mesh().positions().to_vec();
    assert!(stack.lower().is_some());
    assert!(stack.reverse().is_some(), "reverte sob a pirâmide");
    assert_eq!(stack.level_count(), 3);
    assert_stack_is_numbered_by_subdivision(&stack);
    // ⚠️ A escultura do topo é lida do NÍVEL, não de uma subida: `higher`
    // sintetiza, e a síntese carrega o resíduo de um ulp que este gate não é
    // sobre. O que ele afirma é que a renumeração MOVEU a escultura, e para isso
    // a nuvem tem de sair byte-idêntica.
    let key = |p: &[f32; 3]| (p[0].to_bits(), p[1].to_bits(), p[2].to_bits());
    let mut a = sculpted;
    let mut b = stack.levels[2].positions().to_vec();
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "a escultura viaja com a renumeração");
}

#[test]
fn undoing_a_reversal_under_a_pyramid_gives_every_level_back() {
    let mut stack = stack_of(imported(1));
    assert!(stack.add_level());
    stack.mesh_mut().positions_mut()[3][1] += 0.25;
    stack.mesh_mut().rebuild();
    assert!(stack.lower().is_some());
    // ⚠️ Os dois níveis são capturados DEPOIS da descida, que é o estado que a
    // reversão vai receber. Capturar o topo antes mediria o carimbo do `lower`
    // junto, e o gate falharia sobre uma reversão perfeita.
    let base = stack.levels[0].positions().to_vec();
    let top = stack.levels[1].positions().to_vec();
    let top_faces = stack.levels[1].faces().to_vec();
    let rev = stack.reverse().expect("reverte");
    assert!(stack.unreverse(&rev));
    assert_eq!(stack.level_count(), 2);
    assert_eq!(stack.levels[0].positions(), &base[..]);
    assert_eq!(stack.levels[1].positions(), &top[..]);
    assert_eq!(stack.levels[1].faces(), &top_faces[..]);
}

#[test]
fn unreversing_a_stack_that_is_not_where_the_reversal_left_it_is_refused() {
    let mut stack = stack_of(imported(1));
    let rev = stack.reverse().expect("reverte");
    assert!(stack.lower().is_some(), "desce para o nível novo");
    assert!(!stack.unreverse(&rev), "a pilha não está onde ela a deixou");
    assert_eq!(stack.level_count(), 2, "e nada foi mexido");
}

/// A permutação custa um `u32` por vértice por nível — o que a fila de desfazer
/// paga, e um terço de um plano de posições.
#[test]
fn the_undo_payload_is_four_bytes_per_vertex_per_level() {
    let fine = imported(1);
    let n = fine.vert_count();
    let mut stack = stack_of(fine);
    let rev = stack.reverse().expect("reverte");
    assert_eq!(rev.bytes(), n * 4);
}

/// ⚠️ **A fixture CONTÉM o fenômeno, e este gate é quem diz isso.** Se a
/// permutação sair identidade, todo gate de renumeração deste arquivo passa a
/// afirmar nada — e foi exatamente o que aconteceu enquanto a fixture era
/// `subdivide(cube)` cru. Ele mede a fixture, não o produto: é a linha que
/// impede a próxima pessoa de "simplificá-la" de volta.
#[test]
fn the_fixture_is_scattered_enough_for_the_renumbering_to_be_observable() {
    let fine = imported(1);
    let rev = crate::reversion::reverse_subdivision(&fine).expect("reverte");
    let moved = rev
        .renumber()
        .iter()
        .enumerate()
        .filter(|&(j, &o)| j != o as usize)
        .count();
    assert!(
        moved > rev.renumber().len() / 2,
        "a maioria dos vértices tem de MUDAR de índice (mudaram {moved})"
    );
}

/// ⚠️ **Reverter não pode custar ao artista a escultura dele, e o gate que diz
/// isso é o CARIMBO.** Descer logo depois de reverter tem de carimbar
/// EXATAMENTE nada: a malha grossa é uma decimação e o detalhe carrega o resto,
/// então não há nada de novo a levar para baixo. Sem o detalhe computado, o
/// `lower` acha uma diferença enorme e a carimba na base — a base passa a ser a
/// forma FINA, e o nível grosso nasce com a pele dentro. O round-trip continua
/// verde nesse mundo (a forma volta pelo outro caminho), e é por isso que ele
/// sozinho não bastava.
#[test]
fn descending_right_after_a_reversal_stamps_nothing() {
    let mut stack = stack_of(imported(1));
    stack.reverse().expect("reverte");
    let stamped = stack.lower().expect("desce");
    assert!(
        stamped.is_noop(),
        "reverter não é uma escultura: não há o que carimbar embaixo"
    );
}

/// O mesmo, com a pilha inteira: nem a descida do topo nem a da base carimbam.
#[test]
fn descending_after_a_reversal_under_a_pyramid_stamps_nothing_either() {
    let mut stack = stack_of(imported(1));
    assert!(stack.add_level());
    assert!(stack.lower().is_some());
    stack.reverse().expect("reverte");
    assert!(stack.higher(), "sobe ao topo");
    assert!(stack.lower().expect("desce do topo").is_noop());
    assert!(stack.lower().expect("desce à base").is_noop());
}

/// ⚠️ **O DETALHE viaja com a renumeração — e ver isso exige um detalhe
/// NÃO-ZERO e um oráculo GEOMÉTRICO.** Duas armadilhas de fixture moram aqui, e
/// as duas deixaram a mutação passar antes: um nível recém-subdividido tem
/// detalhe **zero**, e permutar zeros é um no-op; e o `higher` **re-encoda** o
/// detalhe no fim, então uma pilha que subiu errado fica *auto-consistente* — a
/// descida seguinte não carimba nada e o gate do carimbo fica verde sobre uma
/// forma destruída.
///
/// O que resta observável é ONDE a escultura está. O ponto é comparado por
/// POSIÇÃO e não por índice, porque índice é justamente o que a renumeração
/// muda.
#[test]
fn the_detail_of_a_sculpted_level_survives_the_renumbering() {
    let mut stack = stack_of(imported(1));
    assert!(stack.add_level());
    stack.mesh_mut().positions_mut()[3][1] += 0.25;
    stack.mesh_mut().rebuild();
    let spike = stack.mesh().positions()[3];
    assert!(stack.lower().is_some());
    stack.reverse().expect("reverte");
    assert!(stack.higher(), "sobe ao topo renumerado");
    let near = stack
        .mesh()
        .positions()
        .iter()
        .filter(|p| worst_gap(std::slice::from_ref(*p), std::slice::from_ref(&spike)) < 1e-4)
        .count();
    assert_eq!(
        near, 1,
        "a escultura continua exatamente onde o artista a deixou"
    );
}
