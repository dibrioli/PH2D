//! **Arch-gate: o extract de sprites pergunta à PORTA, e a porta é a que tem os gates.**
//!
//! ⛔⛔ **A caça aos controlos mortos de 2026-08-30 achou, na §8 Visibility do Inspector, dois
//! campos cuja escrita não decidia coisa nenhuma:**
//!
//! * o `cull_mask` da câmara **nunca é atribuído** (só há o literal `u32::MAX` em todo o repo),
//!   logo 31 dos 32 bits da grade de camadas são inertes;
//! * o [`ph2d_ecs::OnScreenEnabler::contains`] tinha **zero chamadores de produção** — a caixa
//!   e os quatro números do rect gravavam no `.ph2dproj` e nada os lia.
//!
//! A cura pôs as três razões de *«esta entidade desenha?»* numa porta com nome
//! (`render_loop::off_canvas::draws_this_frame`), onde cada uma tem gate de unidade e prova de
//! mutação. **Este ficheiro guarda a outra metade: que o extract de facto a CHAMA.**
//!
//! # Porque é arch-gate, e não um teste de unidade
//!
//! O `sim_extract::run` pede um `&SpriteRenderer` — um objecto de `wgpu` com device. A decisão que
//! ele toma por entidade vive **dentro de um closure** passado ao `propagate_transforms`, e é
//! exactamente a razão que o doc do `off_canvas` já dava para a primeira razão existir separada:
//! *«uma mutação que a desligasse compilaria e passaria a suíte inteira»*.
//!
//! ⚠️ **E há precedente medido nesta pasta:** `cargo test --bins` corre os 4 300 testes da shell e
//! **não toca em `shells/desktop/tests/`** — quatro vermelhos já se esconderam aí. Um gate de fiação
//! só vale se o portão de fecho o alcançar.
//!
//! [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O extract chama a porta, e passa-lhe as DUAS entradas que ela precisa.**
///
/// A máscara da câmara e a pose de mundo do quadro. Passar-lhe `u32::MAX` cravado, ou uma pose
/// local, seria o mesmo defeito noutro sítio.
#[test]
fn the_sprite_extract_routes_every_skip_through_the_named_door() {
    let s = src("render_loop/sim_extract.rs");
    let at = s.find("off_canvas::draws_this_frame(").unwrap_or_else(|| {
        panic!(
            "o extract deixou de perguntar a' porta `off_canvas::draws_this_frame`. As tres \
                 razoes de «esta entidade desenha?» — o olho/receita, a mascara de camadas e o \
                 rect do OnScreenEnabler — voltaram a estar soltas no fio, onde nenhuma mutacao \
                 as mata de forma observavel."
        )
    });
    let call = &s[at..];
    let end = call.find(");").expect("chamada sem fecho");
    let call = &call[..end];
    assert!(
        call.contains("cull_mask"),
        "a porta esta' a ser chamada sem a mascara da camara — com um literal no lugar dela, as 32 \
         caixas da §8 voltam a ser inertes:\n{call}"
    );
    assert!(
        !call.contains("u32::MAX"),
        "a porta esta' a receber `u32::MAX` cravado. Era exactamente esse literal — o unico valor \
         que o `Camera2d::cull_mask` alguma vez toma — que tornava 31 dos 32 bits inertes:\n{call}"
    );

    // ⚠️ **A metade que prova que a chamada MANDA.** Um `draws_this_frame` calculado e ignorado
    // seria verde na asserção acima; o que decide é o `if`.
    assert!(
        s.contains("if drawn\n") || s.contains("if drawn "),
        "o resultado da porta e' calculado e nao e' o que decide a emissao da instancia"
    );
}

/// **E o tique da animação pergunta a metade de CORRER.**
///
/// O `HideVisible` deixa a entidade sem desenho; os dois modos de pausa param o relógio. Se só o
/// primeiro estivesse ligado, dois dos três valores do enum seriam controlos mortos — que é a
/// família de defeito que esta wave curou.
#[test]
fn the_animation_tick_asks_whether_processing_is_paused() {
    let s = src("render_loop/sprite_anim_tick.rs");
    assert!(
        s.contains("on_screen_gate::processing_paused("),
        "o tique da animacao deixou de perguntar pelo `OnScreenEnabler`: os modos `InheritPause` e \
         `PauseProcessing` voltam a ser duas opcoes de enum sem efeito nenhum"
    );
    assert!(
        s.contains("paused.contains(&entity)"),
        "a lista de pausados e' calculada e nao e' consultada no laco — o custo fica e o efeito nao"
    );
}

/// ⛔⛔⛔ **O BLOQUEADOR DO OUTRO LADO, escrito como uma medição e não como uma nota.**
///
/// A grade de 32 caixas da §8 escreve `ph2d_ecs::VisibilityLayer`, e o consumidor dela está
/// completo e gateado. **O que falta é o AUTOR:** nada no produto escreve
/// `Camera2d::cull_mask` — ele só existe como o literal `u32::MAX` — e a superfície que o
/// autoraria (um filtro de camadas do viewport, no menu *View*) mora em `ph2d-editor-core`
/// (`src/ids/menus.rs` + `screens/hero/menu_rows.rs` + `screens/hero/chrome/view_toggles.rs`),
/// fora do alcance da linha que curou este lado.
///
/// ⚠️ **A pergunta é feita ao CÓDIGO, não a uma nota.** No dia em que alguém autorar a máscara,
/// este gate fica vermelho e obriga quem o fez a vir aqui apagar o bloqueador — em vez de deixar
/// uma nota a dizer «31 bits inertes» a envelhecer sozinha.
#[test]
fn nothing_authors_a_camera_cull_mask_yet_and_that_is_the_named_blocker() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let camera = fs::read_to_string(repo.join("crates/ph2d-render/src/camera.rs"))
        .expect("o `Camera2d` mudou de casa — reancore este gate");
    // A metade justa: a sonda está mesmo a ler o ficheiro certo.
    assert!(
        camera.contains("pub cull_mask: u32"),
        "a sonda nao encontrou o campo que ela julga — sem esta metade ela responderia «ninguem o \
         escreve» para sempre, sobre qualquer ficheiro"
    );
    let authored: Vec<&str> = camera
        .lines()
        .filter(|l| l.contains("cull_mask") && l.contains('='))
        .filter(|l| !l.contains("u32::MAX"))
        .filter(|l| !l.trim_start().starts_with("//") && !l.trim_start().starts_with("///"))
        .collect();
    assert!(
        authored.is_empty(),
        "alguem passou a autorar o `cull_mask` da camara. ⇒ As 32 caixas da §8 Visibility deixaram \
         de ser um interruptor «esconder» e passaram a ser camadas a se'rio: reveja o doc de \
         `render_loop/off_canvas::layer_visible`, que declara este bloqueador, e apague-o.\n  {}",
        authored.join("\n  ")
    );
}
