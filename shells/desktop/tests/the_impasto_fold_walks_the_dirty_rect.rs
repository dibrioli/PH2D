//! **Arch-gate: o fold do impasto anda o RETÂNGULO SUJO, não a tela.**
//!
//! ## O defeito (medido, 2026-07-25)
//!
//! `impasto_gpu_planes` materializa o relevo composto para o shader ler, e o produtor GPU o chamava por
//! **frame sujo** — que durante um traço é **por movimento**. Medido em release, na RTX:
//!
//! | canvas | fold/frame | do que é feito |
//! |---|---|---|
//! | 2048² | 45,5 ms | 1,5 ms de alocação, o resto é walk por-texel |
//! | 4096² | **202,4 ms** | 0,15 ms de alocação, **180 ms de walk** |
//!
//! E o mesmo walk sobre uma janela de 512² custa **2,82 ms nas DUAS telas** — o custo é a contagem de
//! texels e nada mais. Head-to-head no device, traço esculpido, por movimento:
//!
//! | canvas | pista GPU (antes) | pista CPU | razão |
//! |---|---|---|---|
//! | 2048² | 57,1 ms | 2,10 ms | 27× |
//! | 4096² | **225,6 ms** | 2,12 ms | **106×** |
//!
//! Ou seja: a pista construída para acelerar o impasto era **106× mais lenta** que a que ela substituiu,
//! no jeito mais comum de usar impasto — e o roteador mandava o documento para lá. A wave de 18/07 provou
//! que as duas pistas desenham a MESMA imagem; nunca perguntou qual era a mais rápida.
//!
//! Com a janela: **1,98 ms a 2048² e 2,62 ms a 4096²** (86× a 4K), em paridade com a CPU.
//!
//! ## Por que um gate de TEXTO, e por que ele é necessário
//!
//! O ganho inteiro repousa num `Some`: se `preview_gpu_region` parasse de chegar, ou se alguém trocasse
//! a porta de volta pela de tela cheia, **o produto continuaria CORRETO** — apenas 86× mais lento — e
//! todo gate de aparência seguiria verde, incluindo a paridade e2e entre os dois produtores. É a forma
//! exata de regressão que este módulo já sofreu uma vez, calada.
//!
//! O gate de COMPORTAMENTO do outro lado da costura existe e roda headless
//! (`ph2d_tool_painter`: `the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` prova que a
//! porta regional é limitada pela janela; `a_window_folds_exactly_what_the_whole_canvas_folded_there`
//! prova que ela diz a mesma coisa). Nenhum dos dois vê a SHELL: ela poderia pedir a tela inteira toda
//! vez e os dois seguiriam verdes. Este aqui prova que ela **pede a janela** — e a decisão mora dentro
//! de `compose_light_premul`, que exige device e sessão, então nenhum teste de unidade a alcança.
//!
//! ⚠️ Ele afirma uma **relação**, nunca uma distância em bytes: um gate ancorado em "a menos de N bytes
//! de" expira no dia em que alguém insere uma linha no meio, que foi como dois arch-gates da
//! `line/Vector` chegaram vermelhos ao `main` em 2026-07-23.

const SRC: &str = include_str!("../src/render_loop/painter_gpu_preview.rs");

/// Controle positivo: o arquivo foi mesmo lido, e é o que este gate pensa que é.
///
/// Sem isto, um `include_str!` apontando para o lugar errado (ou um arquivo esvaziado) deixaria todo
/// `assert!(!SRC.contains(...))` abaixo passar por vacuidade — um gate que não pode falhar pelo motivo
/// que alega ([[reference_topic_gate_discipline]]).
#[test]
fn the_gate_reads_the_producer_it_claims_to_read() {
    assert!(
        SRC.len() > 8_000,
        "o produtor GPU encolheu para {} bytes — este gate provavelmente está lendo o arquivo errado",
        SRC.len()
    );
    assert!(
        SRC.contains("fn compose_light_premul("),
        "a função que decide a janela sumiu do arquivo; se foi movida, mova este gate junto"
    );
}

/// **A shell pede uma JANELA, e a janela vem do retângulo sujo do tool.**
///
/// Três afirmações, porque são três fatos independentes e cada um sozinho pode apodrecer:
///
/// 1. a porta chamada é a regional (`impasto_gpu_planes_in`);
/// 2. o que ela recebe nasce de `preview_gpu_region()` — o retângulo que o tool só oferece quando a
///    mudança FOI confinada a ele (uma edição estrutural passa por `invalidate_composite`, que o
///    derruba), e é isso que torna o upload parcial correto em vez de meramente rápido;
/// 3. a shell **pergunta ao passe** se os planos já foram semeados, porque quem é dono da textura é
///    quem sabe se ela já teve a pintura inteira — um resize a reconstrói e a resposta volta a `false`.
///
/// **Mutações que devem sangrar:** trocar `impasto_gpu_planes_in(plane_win)` por
/// `impasto_gpu_planes()` (mata 1) · cravar `plane_win` em `(0, 0, width, height)` (mata 2) · tirar o
/// `planes_seeded` do filtro (mata 3).
#[test]
fn the_shell_folds_the_window_the_tool_reports_dirty() {
    assert!(
        SRC.contains("impasto_gpu_planes_in("),
        "a shell precisa chamar a porta REGIONAL do fold — a de tela cheia custa 202 ms/frame a 4096²"
    );
    assert!(
        SRC.contains("preview_gpu_region()"),
        "a janela tem de vir do retângulo sujo do tool; qualquer outra origem é um segundo dono do \
         fato \"o que mudou neste frame\""
    );
    assert!(
        SRC.contains("planes_seeded("),
        "a shell tem de perguntar ao PASSE se os planos já foram semeados — um upload parcial sobre \
         uma textura recém-criada ilumina o resto da pintura como se fosse chapada"
    );
}

/// **A porta de tela cheia não é mais chamada aqui.**
///
/// Separado do gate acima de propósito: aquele diz *"a regional é chamada"*, e as duas frases não são a
/// mesma — um refactor pode chamar as DUAS (a regional no caminho novo, a cheia num ramo esquecido) e
/// deixar aquele verde enquanto o caminho caro volta a existir. É a lição de camadas do próprio módulo:
/// duas defesas, dois gates ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// A busca é pela chamada `impasto_gpu_planes()` com parênteses vazios, que é o que a porta cheia é;
/// `impasto_gpu_planes_in(` não casa com ela.
#[test]
fn the_full_canvas_door_is_not_the_one_the_producer_takes() {
    assert!(
        !SRC.contains("impasto_gpu_planes()"),
        "alguém trouxe a porta de tela cheia de volta ao produtor GPU — ela custa 202 ms por movimento \
         a 4096², e o produto continua CORRETO com ela, que é por que isto precisa de um gate"
    );
}
