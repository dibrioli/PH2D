//! **Os gates da COSTURA do `.ase`** ([`super`]).
//!
//! ⚠️ **O que está aqui é o que é alcançável sem GPU.** O `import_ase` pede um `SpriteRenderer`
//! vivo, então a função inteira não é testável — foi por isso que as três decisões dela saíram
//! para funções **puras** (`grid_for`, `pack`, `library`), que é onde um erro faria a animação sair
//! trocada. ⚠️ **O resíduo por gatear está declarado**: a subida da textura, o `hframes`/`vframes`
//! escritos na sprite, e o `size` dividido pela grelha — três linhas, conferidas de olho e cobertas
//! pelo smoke.

use super::*;

fn frame(ms: u16, w: u16, h: u16, fill: u8) -> AseFrame {
    AseFrame {
        duration_ms: ms,
        rgba: vec![fill; usize::from(w) * usize::from(h) * 4],
    }
}

fn ase_tag(name: &str, from: u16, to: u16, direction: u8, repeat: u16) -> AseTag {
    AseTag {
        name: name.to_owned(),
        from,
        to,
        direction,
        repeat,
    }
}

/// **UMA TIRA, sempre que ela couber** — é o que o artista espera, é o que o Aseprite exporta por
/// omissão, e faz o `hframes` do inspector ser legível.
///
/// **Mutação que deve sangrar:** trocar o atalho da tira pela quase-quadrada sempre.
#[test]
fn frames_that_fit_become_one_strip() {
    assert_eq!(grid_for(8, 32, 32), (8, 1));
    assert_eq!(grid_for(1, 16, 16), (1, 1));
    assert_eq!(
        grid_for(256, 32, 32),
        (256, 1),
        "256 x 32 = 8192, cabe justo"
    );
}

/// **E quando ela NÃO cabe, a folha vira quase-quadrada** — e o limite é de memória de GPU, com o
/// número ao lado (`MAX_SHEET_EDGE_PX`).
///
/// ⚠️ As duas metades juntas: um `grid_for` que devolvesse sempre uma tira passaria o gate acima.
#[test]
fn frames_that_do_not_fit_wrap_into_rows() {
    let (cols, rows) = grid_for(300, 128, 128);
    assert!(cols < 300, "300 x 128 = 38400 px nao cabe numa tira");
    assert!(
        cols * 128 <= 8192,
        "a folha ficou com {} px de largura",
        cols * 128
    );
    assert!(cols * rows >= 300, "a grelha tem de caber os 300 quadros");
    // E ela é quase-quadrada, não uma coluna: 18x17 para 300.
    assert!(rows <= cols + 1, "{cols}x{rows} nao e' quase-quadrada");
}

/// **A ORDEM É O CONTRATO**: em linha, da esquerda para a direita e de cima para baixo — porque uma
/// tag é um intervalo sobre as CÉLULAS, e a célula 5 tem de ser o quadro 5.
///
/// ⚠️ Empacotar por colunas daria uma folha bonita e todas as animações trocadas, e nenhum outro
/// gate veria isso.
///
/// **Mutação que deve sangrar:** trocar `(i % cw, i / cw)` por `(i / ch, i % ch)`.
#[test]
fn the_frames_are_packed_row_major_because_a_tag_indexes_cells() {
    // Quatro quadros de 1x1, cada um com um valor diferente, numa grelha 2x2.
    let frames: Vec<AseFrame> = (0..4).map(|i| frame(100, 1, 1, 10 + i as u8)).collect();
    let sheet = pack(&frames, 1, 1, 2, 2);
    // Linha 0: quadros 0 e 1. Linha 1: quadros 2 e 3.
    assert_eq!(sheet[0], 10, "celula (0,0) = quadro 0");
    assert_eq!(sheet[4], 11, "celula (1,0) = quadro 1");
    assert_eq!(sheet[8], 12, "celula (0,1) = quadro 2");
    assert_eq!(sheet[12], 13, "celula (1,1) = quadro 3");
}

/// **As células que sobram na última linha ficam transparentes** — a grelha é rectangular e o
/// número de quadros raramente é.
#[test]
fn the_leftover_cells_are_transparent() {
    let frames: Vec<AseFrame> = (0..3).map(|_| frame(100, 1, 1, 255)).collect();
    let sheet = pack(&frames, 1, 1, 2, 2);
    assert_eq!(
        &sheet[12..16],
        &[0, 0, 0, 0],
        "a 4a celula tinha de ficar vazia"
    );
}

/// **Uma tag do Aseprite vira uma da §11 inteira** — nome, intervalo, direcção e repetições.
///
/// ⚠️ `repeat: 0` do ficheiro é `None` na §11 (as duas dizem *para sempre* com valores diferentes),
/// e trocá-los faria toda animação importada tocar **uma vez** e parar.
#[test]
fn a_tag_crosses_over_whole() {
    let frames: Vec<AseFrame> = (0..8).map(|_| frame(70, 1, 1, 0)).collect();
    let (tag, note) = tag_from_ase(&ase_tag("walk", 0, 7, 0, 0), &frames);
    assert_eq!(tag.name, "walk");
    assert_eq!((tag.from, tag.to), (0, 7));
    assert_eq!(tag.frame_ms, 70);
    assert_eq!(tag.direction, AnimDirection::Forward);
    assert_eq!(tag.repeat, None, "zero do ficheiro = para sempre");
    assert!(note.is_none(), "durou tudo igual: nada a avisar");

    let (pp, _) = tag_from_ase(&ase_tag("hit", 2, 5, 2, 3), &frames);
    assert_eq!(pp.direction, AnimDirection::PingPong);
    assert_eq!(pp.repeat, Some(3));

    // As quatro direcções do ficheiro dão as quatro da §11, e um número fora da lista não inventa.
    let dirs: Vec<_> = (0..4)
        .map(|d| tag_from_ase(&ase_tag("d", 0, 1, d, 0), &frames).0.direction)
        .collect();
    assert_eq!(dirs, AnimDirection::ALL.to_vec());
    assert_eq!(
        tag_from_ase(&ase_tag("d", 0, 1, 99, 0), &frames)
            .0
            .direction,
        AnimDirection::Forward
    );
}

/// **A DURAÇÃO POR-QUADRO É APROXIMADA — E DITA.** ⚠️ É a recusa medida que este ficheiro reabre
/// (spec §8.12): a §11 guarda um `frame_ms` por tag. Aproximar em silêncio faria o *hold* de
/// antecipação do artista desaparecer sem uma linha a explicar.
///
/// **Mutação que deve sangrar:** devolver `None` em vez da nota.
#[test]
fn a_tag_with_per_frame_durations_keeps_them_all() {
    let frames = vec![
        frame(50, 1, 1, 0),
        frame(50, 1, 1, 0),
        frame(400, 1, 1, 0), // o hold
        frame(50, 1, 1, 0),
    ];
    let (tag, note) = tag_from_ase(&ase_tag("idle", 0, 3, 0, 0), &frames);
    assert_eq!(
        tag.per_frame_ms,
        vec![50, 50, 400, 50],
        "o ritmo proprio tem de passar INTEIRO — era isto que a recusa impedia"
    );
    assert!(tag.has_per_frame_timing());
    assert_eq!(tag.frame_ms, 50, "e o campo do painel mostra o mais comum");
    let note = note.expect("tinha de avisar que ha' ritmo proprio");
    assert!(note.contains("idle"), "a nota tem de NOMEAR a tag: {note}");
    assert!(note.contains("400"), "e dizer o intervalo real: {note}");
}

/// **UMA TAG DE RITMO UNIFORME NÃO CARREGA VETOR NENHUM** — é o caso comum, e a metade que impede
/// a feature de encher todo projeto de dados que dizem o que o `frame_ms` já diz.
///
/// **Mutação que deve sangrar:** preencher o `per_frame_ms` sempre.
#[test]
fn a_uniform_tag_carries_no_vector() {
    let frames: Vec<AseFrame> = (0..4).map(|_| frame(80, 1, 1, 0)).collect();
    let (tag, note) = tag_from_ase(&ase_tag("walk", 0, 3, 0, 0), &frames);
    assert!(tag.per_frame_ms.is_empty(), "uniforme nao guarda vetor");
    assert!(!tag.has_per_frame_timing());
    assert_eq!(tag.frame_ms, 80);
    assert!(note.is_none(), "e nao ha' nada a avisar");
}

/// **O vetor é do INTERVALO da tag, não do ficheiro inteiro** — uma tag `4..6` num ficheiro de dez
/// quadros guarda três valores, e o primeiro é o do quadro 4.
///
/// **Mutação que deve sangrar:** copiar `frames` inteiro em vez da fatia.
#[test]
fn the_vector_covers_the_tags_own_range() {
    let mut frames: Vec<AseFrame> = (0..10).map(|_| frame(60, 1, 1, 0)).collect();
    frames[4] = frame(300, 1, 1, 0);
    let (tag, _) = tag_from_ase(&ase_tag("attack", 4, 6, 0, 0), &frames);
    assert_eq!(tag.per_frame_ms, vec![300, 60, 60]);
}

/// **UM FICHEIRO SEM TAGS RECEBE UMA**, com o nome do ficheiro, a cobrir todos os quadros.
///
/// ⛔ Sem ela a sprite nasce com uma grelha e nada para tocar, e a §11 fica muda sobre um ficheiro
/// que É uma animação. É o que o próprio Aseprite faz ao exportar um ficheiro sem tags — e a nota
/// diz que a animação foi inventada por nós.
#[test]
fn a_file_without_tags_still_gets_something_to_play() {
    let doc = AseDoc {
        width: 1,
        height: 1,
        frames: (0..5).map(|_| frame(120, 1, 1, 0)).collect(),
        tags: Vec::new(),
        notes: Vec::new(),
    };
    let (lib, notes) = library(&doc, "hero");
    let tags: Vec<_> = lib.iter().collect();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "hero", "o nome do ficheiro");
    assert_eq!((tags[0].from, tags[0].to), (0, 4), "cobre TODOS os quadros");
    assert_eq!(tags[0].frame_ms, 120);
    assert_eq!(notes.len(), 1, "e diz que a animacao foi inventada");
}

/// **Uma tag que a §11 recusa sai NOMEADA, com o motivo.** ⛔ «Alguma coisa não entrou» manda o
/// artista adivinhar entre um nome repetido, um nome vazio e uma lista cheia — três consertos
/// diferentes.
#[test]
fn a_rejected_tag_names_itself_and_the_reason() {
    let doc = AseDoc {
        width: 1,
        height: 1,
        frames: (0..4).map(|_| frame(100, 1, 1, 0)).collect(),
        tags: vec![
            ase_tag("walk", 0, 1, 0, 0),
            ase_tag("walk", 2, 3, 0, 0), // o mesmo nome
            ase_tag("", 0, 1, 0, 0),     // vazio
        ],
        notes: Vec::new(),
    };
    let (lib, notes) = library(&doc, "hero");
    assert_eq!(lib.iter().count(), 1, "so' a primeira `walk` entra");
    assert_eq!(notes.len(), 2, "duas recusas, duas linhas: {notes:?}");
    assert!(notes[0].contains("walk") && notes[0].contains("Duplicate"));
    assert!(notes[1].contains("Empty"));
}

/// **As duas extensões que o Aseprite escreve, sem olhar a maiúsculas** — a extensão vem do sistema
/// de ficheiros do utilizador, e no Windows ela pode chegar em maiúsculas.
#[test]
fn both_aseprite_extensions_are_recognised() {
    use std::path::Path;
    assert!(is_ase_file(Path::new("/a/hero.ase")));
    assert!(is_ase_file(Path::new("/a/hero.aseprite")));
    assert!(is_ase_file(Path::new("/a/hero.ASE")));
    assert!(!is_ase_file(Path::new("/a/hero.png")));
    assert!(!is_ase_file(Path::new("/a/hero")));
}
