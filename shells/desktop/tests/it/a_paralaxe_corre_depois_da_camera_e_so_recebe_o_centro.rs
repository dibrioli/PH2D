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
//! # ⭐⭐ Metade 2 — só o CENTRO atravessa, e é ISSO a invariância ao zoom
//!
//! A pesquisa mediu no alvo que o deslocamento de um `Parallax2D` **não depende do zoom nem da
//! rotação da câmera** — a paralaxe nasce da TRANSLAÇÃO, e uma câmera que só roda não produz
//! nenhuma ([`docs/Components/23_pesquisa_paralaxe.md`](../../../../docs/Components/23_pesquisa_paralaxe.md)
//! §4.4). ⛔ **Do lado da lei isso é estrutural e não uma guarda:** a `desloca` recebe o centro e
//! mais nada, logo não HÁ zoom para ler. O que pode envelhecer é esta composição — alguém a passar
//! a meia-janela «porque está à mão» —, e é exactamente essa recaída que este gate reprova.
//!
//! ⚠️ **A lente é o TEXTO EMENDADO do quadro** (`frame_text::render_frame`), nunca um ficheiro: a
//! fase que corre primeiro pode morar no ficheiro que vem depois.

/// **Mutação que deve sangrar:** mover a chamada `self.fase_paralaxe(camera_rect)` para antes do
/// `self.fase_game_camera(...)`, ou passar `camera_rect` inteiro em vez do centro.
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

    // E a chamada à lei recebe o CENTRO, nunca o rectângulo.
    let chamada = src
        .find("parallax_bridge::drive_parallax(")
        .expect("a fase deixou de chamar a ponte: a lei existe e o quadro nao a corre");
    let corte = &src[chamada..];
    let fim = corte.find(");").expect("a chamada da ponte nao fecha");
    let args = &corte[..fim];
    assert!(
        args.contains("centro"),
        "a ponte deixou de receber o centro da vista:\n{args}"
    );
    assert!(
        !args.contains("camera_rect"),
        "a meia-janela atravessa para a lei: a paralaxe e um DESLOCAMENTO e o tamanho da vista nao \
         entra nela — passar o rectangulo inteiro oferece a ponte um dado que ela nao pode usar, e \
         e por onde a invariancia ao zoom se perde em silencio:\n{args}"
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
