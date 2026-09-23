//! ⭐⭐⭐ **A ORDEM da PARALAXE no quadro, e o que ATRAVESSA para a lei** (plano 24, W1).
//!
//! A lei é `saída = autorada + centro_da_vista · (1 − k)`, e ela vive na
//! `ph2d_app_components::parallax_bridge`. O que esta shell decide é **quando** ela corre e **com
//! que dado** — e as duas metades são gateadas aqui, porque nenhuma é visível de dentro da crate.
//!
//! # ⚠️ Metade 1 — a vista é a DESTE quadro
//!
//! A `fase_game_camera` calcula o rectângulo da câmera do jogo; um passe de paralaxe ANTES dela
//! deslocaria contra o enquadramento do quadro ANTERIOR, e num jogo em que a câmera segue o herói
//! o fundo lia-se como *«atrasado um quadro»* — a mesma classe de defeito que a ordem do HUD paga,
//! e que esta shell já pagou quatro vezes noutros sítios.
//!
//! # ⭐⭐ Metade 2 — a invariância ao zoom, e a PREMISSA que MORREU aqui
//!
//! A pesquisa mediu no alvo que o deslocamento de um `Parallax2D` **não depende do zoom nem da
//! rotação da câmera** — a paralaxe nasce da TRANSLAÇÃO, e uma câmera que só roda não produz
//! nenhuma ([`docs/Components/23_pesquisa_paralaxe.md`](../../../../docs/Components/23_pesquisa_paralaxe.md)
//! §4.4).
//!
//! ⛔⛔ **A 1.ª redacção deste gate exigia que o rectângulo da câmera NÃO atravessasse a fronteira**,
//! e ela estava certa enquanto a paralaxe fosse só um deslocamento. A **W3** (o confinamento) tem o
//! joelho em `(região − ecrã)/2` — *ele precisa de saber quanto a vista mede* — e a premissa morreu
//! no dia seguinte ao de ter sido escrita.
//!
//! ⇒ **as duas perguntas separaram-se, e a lei continua gateada pela mais forte das duas réguas:**
//!
//! 1. *o zoom entra no DESLOCAMENTO?* — **não**, e quem o afirma é a **ASSINATURA**:
//!    `ScrollFactor::deslocamento(centro)` recebe um centro e mais nada, logo não HÁ zoom para ler
//!    e nenhum gate é preciso. *Uma lei que o compilador proíbe não precisa de uma régua.*
//! 2. *o zoom entra no CONFINAMENTO?* — **sim**, e é a lei medida; ele tem gate próprio na crate da
//!    lei (`o_joelho_esta_onde_a_borda_da_vista_alcanca_a_regiao`).
//!
//! O que fica aqui é o que só a shell pode afirmar: **a fase entrega o rectângulo INTEIRO**, porque
//! um centro sozinho tornaria o confinamento inexprimível — o defeito oposto ao que a 1.ª redacção
//! guardava.
//!
//! ⚠️ **A lente é o TEXTO EMENDADO do quadro** (`frame_text::render_frame`), nunca um ficheiro: a
//! fase que corre primeiro pode morar no ficheiro que vem depois.

/// **Mutação que deve sangrar:** mover a chamada `self.fase_paralaxe(camera_rect)` para antes do
/// `self.fase_game_camera(...)`, ou entregar à ponte só o centro.
#[test]
fn a_paralaxe_corre_depois_da_camera_e_so_recebe_o_centro() {
    let src = crate::frame_text::render_frame();

    let camera = src
        .find("⟦fase fase_game_camera⟧")
        .expect("a `fase_game_camera` deixou de correr no quadro");
    let paralaxe = src
        .find("⟦fase fase_paralaxe⟧")
        .expect("a `fase_paralaxe` nao corre no quadro: a paralaxe existe e nada a chama");
    assert!(
        camera < paralaxe,
        "a paralaxe corre ANTES da camera: o fundo desloca-se contra o enquadramento do quadro \
         anterior, e num jogo que segue o heroi le-se como «atrasado um quadro»"
    );

    // E a chamada à lei recebe o RECTÂNGULO INTEIRO — ver o ⛔⛔ do cabeçalho sobre a premissa que
    // morreu aqui: sem a meia-janela o CONFINAMENTO da W3 é inexprimível.
    let chamada = src
        .find("parallax_bridge::drive_parallax(")
        .expect("a fase deixou de chamar a ponte: a lei existe e o quadro nao a corre");
    let corte = &src[chamada..];
    let fim = corte.find(");").expect("a chamada da ponte nao fecha");
    let args = &corte[..fim];
    assert!(
        args.contains("camera_rect"),
        "a ponte deixou de receber o rectangulo da camera: com so' o centro o joelho do \
         confinamento — `(regiao − ecra)/2` — deixa de ser calculavel, e o fundo passa a mostrar a \
         borda:\n{args}"
    );
    // ⚠️⚠️ **E SEM FILTRO, o que é uma asserção separada porque uma MUTAÇÃO o exigiu:** a régua de
    // cima procura a PALAVRA `camera_rect`, e um `camera_rect.map(|(c, _h)| (c, [0.0, 0.0]))` — que
    // esvazia a meia-janela e mata o joelho — mantém-na lá. *Uma régua que procura um nome não vê
    // o que foi feito ao valor com esse nome.*
    assert!(
        !args.contains(".map("),
        "a fase FILTRA o rectangulo antes de o entregar: a lei tem de receber o que a camera \
         devolveu, e o que se perde num filtro aqui e' a meia-janela — o joelho do confinamento \
         passa a estar sempre na borda da regiao:\n{args}"
    );

    // ⚠️ O CONTROLO da segunda metade: a shell de facto TEM o rectângulo em mãos neste ponto — sem
    // esta linha o gate acima passaria numa shell onde `camera_rect` nem existe, medindo nada.
    let fase = crate::frame_text::phases()
        .get("fase_paralaxe")
        .cloned()
        .expect("a `fn fase_paralaxe` deixou de existir");
    assert!(
        fase.contains("camera_rect"),
        "a fase nao recebe o rectangulo da camera: o controlo do gate acima e vacuo"
    );
}
