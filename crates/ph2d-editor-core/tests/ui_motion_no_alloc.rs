//! **O relógio da UI viva não aloca por quadro.**
//!
//! ## Por que este gate existe, e por que ele não é de identidade
//!
//! O [`UiMotion::advance`] corre **uma vez por quadro, para sempre**, sobre a população inteira da
//! UI (medido: **4 491 tracks** com todos os painéis populados — ver a sonda
//! `ui_motion_population_census` da `ph2d-panel-registry-init`). Ele consultava a lei através de um
//! `&self`, o que o impedia de pedir `&mut self.tracks` ao mesmo tempo, e contornava isso
//! **colectando um `Vec` do tamanho do mapa** e voltando a descer a árvore por entrada.
//!
//! ⚠️ **Nenhum gate de identidade consegue ver essa diferença.** A rota antiga e a nova computam a
//! MESMA lei sobre os MESMOS campos, então todo teste de comportamento fica verde nas duas — o que
//! muda é uma alocação de ~72 kB e uma travessia inteira, sessenta vezes por segundo. Um defeito
//! que só um relógio ou um contador enxerga precisa de um gate que conte.
//!
//! ⚠️ **CONTADOR e não relógio, de propósito:** um kill de wall-clock nesta workstation mede o
//! `load average` tanto quanto o código (a política que o repositório já escreveu três vezes). Uma
//! contagem de blocos é determinística e não flaka.
//!
//! ## A fixture TEM de conter o fenómeno
//!
//! ⚠️ **Com o mapa VAZIO este gate seria verde sob a rota antiga**: `collect` sobre um iterador
//! vazio devolve um `Vec` de capacidade zero, que não chama o alocador. O que faz a mutação
//! sangrar é a POPULAÇÃO — por isso a fixture instala tracks antes de a janela de medição abrir, e
//! o número é deliberadamente da ordem do que o app real carrega.
//!
//! *Mutação: devolver o `advance` a `let laws: Vec<_> = self.tracks.iter().map(…).collect()` ⇒ uma
//! alocação por quadro ⇒ sangra.*

use dhat::{HeapStats, Profiler};
use ph2d_a11y::NodeId;
use ph2d_editor_core::motion::{Role, UiCharacter, UiMotion};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const DT: f64 = 1.0 / 60.0;
/// Da ordem da população real (4 491 medidos); o que importa é ser MUITO maior que zero.
const TRACKS: u64 = 512;
const FRAMES: usize = 10;

#[test]
fn the_ui_clock_does_not_allocate_per_frame() {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);

    // SETUP, fora da janela: instala a população. A primeira vista de um id CHEGA ao alvo, então
    // uma só chamada por id basta para o mapa existir — e nenhuma delas fica EM VOO, que é
    // exactamente o regime do app parado, o mais comum e o que este gate protege.
    for i in 0..TRACKS {
        m.animate(NodeId(i + 1), 0.0, Role::Fade);
    }
    assert_eq!(
        m.remembered(),
        TRACKS as usize,
        "a fixture tem de LEMBRAR a populacao: com o mapa vazio a rota antiga tambem nao alocaria"
    );
    assert_eq!(m.in_flight(), 0, "o regime medido e' o app PARADO");

    let profiler = Profiler::builder().testing().build();
    let before = HeapStats::get();

    for _ in 0..FRAMES {
        m.advance(DT);
        // Re-alveja no MESMO valor: é o que o tique do produto faz a cada quadro, e mantém as
        // tracks vivas contra a poda sem pôr nada em voo.
        for i in 0..TRACKS {
            m.animate(NodeId(i + 1), 0.0, Role::Fade);
        }
    }

    let after = HeapStats::get();
    drop(profiler);

    let blocks = after.total_blocks - before.total_blocks;
    let bytes = after.total_bytes - before.total_bytes;
    assert_eq!(
        blocks, 0,
        "o relogio da UI alocou {blocks} blocos ({bytes} bytes) em {FRAMES} quadros sobre \
         {TRACKS} tracks -- o `advance` corre por quadro para sempre, e a populacao dele e' a UI \
         inteira. Procure um `collect`/`Vec` novo no laco."
    );
}
