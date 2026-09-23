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

    // ⭐⭐⭐ **Metade 3 — o RELÓGIO chega à lei** (W4). A deriva é `velocidade × playhead`, e é por
    // ele ser LIDO a cada quadro (nunca acumulado) que uma nuvem sobrevive a um scrub. ⛔ Cravá-lo
    // congela a deriva **em silêncio**: nada na tela diz porquê, e a lei continua verde nos gates
    // dela — eles recebem o instante como argumento.
    assert!(
        fase.contains("self.playhead.time()"),
        "a fase deixou de ler o relogio: a deriva da W4 congela, e os gates da lei nao o veem \
         porque recebem o instante como argumento"
    );
}

/// ⭐⭐⭐ **O prólogo da cena da paralaxe faz as TRÊS coisas que a cena não pode fazer** (W7).
///
/// ⚠️ **Cada uma, esquecida, dá uma cena que ENSINA O CONTRÁRIO** e nenhum gate da família a vê,
/// porque a família não alcança o `HeroScreen` nem o playhead:
/// - sem **tomar a vista da câmera do jogo**, o ecrã mostra a câmera do editor e o fundo anda contra
///   uma coisa que o dono não vê — *ele parado e o céu a deslizar*;
/// - sem **fechar a régua do transporte**, a meia-vista da câmera é da JANELA e a banda do canvas
///   fica com metade: o céu e o chão saem do ecrã (a armadilha que três waves desta linha pagaram);
/// - sem **o relógio a andar**, as nuvens não derivam e o herói não anda.
///
/// ⚠️ É um gate de TEXTO porque o prólogo pede `HeroScreen` + `GpuContext` e não é alcançável de um
/// teste — a mesma razão do gate da fiação do `dispatch` (§5 do Motion).
#[test]
fn o_prologo_da_cena_da_paralaxe_toma_a_vista_fecha_a_regua_e_poe_o_relogio_a_andar() {
    let fonte = include_str!("../../src/components_scenes_suplentes.rs");
    let ini = fonte
        .find("fn parallax_smoke(")
        .expect("o prologo da cena da paralaxe deixou de existir");
    let resto = &fonte[ini..];
    let corpo = &resto[..resto[1..]
        .find("\n    pub(crate) fn ")
        .map_or(resto.len(), |i| i + 1)];
    for (agulha, porque) in [
        (
            "self.game_camera_preview = true;",
            "sem tomar a vista, o fundo anda contra a camera do EDITOR",
        ),
        (
            "panel_visibility.insert(\"timeline\", false)",
            "com a regua aberta o ceu e o chao saem do ecra",
        ),
        (
            "self.playhead.play();",
            "sem o relogio a andar as nuvens nao derivam e o heroi nao anda",
        ),
        (
            "hero.gizmo.selection = Some(montada.escolhido);",
            "o roteiro nomeia uma seccao do Inspector",
        ),
    ] {
        assert!(
            corpo.contains(agulha),
            "{porque} — falta `{agulha}` no prologo"
        );
    }
    // ⛔ E a metade NEGATIVA: abrir a régua aqui desfaz a segunda linha, com as duas presentes.
    assert!(
        !corpo.contains("abre_a_regua_da_corrida"),
        "o prologo abre a regua do transporte, e a cena foi arrumada para o ecra SEM ela"
    );
}
