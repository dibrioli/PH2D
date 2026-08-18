//! **A cena do espaçamento ARMA o que a mensagem dela promete.**
//!
//! O gate de cook irmão (`ph2d-node-registry-init/tests/spacing_scene.rs`) prova que o NÓ, dadas
//! as quatro trilhas e os quatro conjuntos de params, devolve `9 · 9 · 8 · 16`. Ele monta o grafo
//! ele próprio — logo é **CEGO** à cena: se o `name_and_wire_spacing` esquecer o `mode`, as quatro
//! fileiras caem na contagem por número, o smoke desenha quatro linhas parecidas, e os três gates
//! de lei seguem verdes.
//!
//! Esta é a metade que só o FONTE da cena pode responder, e ela vive aqui porque a cena precisa de
//! uma janela (`self.gfx`) — nenhum teste de unidade a alcança.
//!
//! ⚠️ **O oráculo é a PROPRIEDADE, nunca a distância entre linhas:** *as duas primeiras fileiras
//! armam uma CONTAGEM · as duas últimas armam o MODO e o ESPAÇAMENTO*, com controle positivo para
//! uma varredura vazia não passar por verde.

const SCENE: &str = include_str!("../src/motion_node_path_smoke.rs");

/// A metade do arquivo que monta a cena `=2` — o resto (a cena `=1`, o roteiro de frames) não é
/// assunto deste gate, e lê-lo inteiro faria a `count` da cena original contar como prova.
fn spacing_scene_body() -> &'static str {
    let start = SCENE
        .find("fn name_and_wire_spacing")
        .expect("a cena do espaçamento tem de existir; se ela se mudou, este gate se muda com ela");
    let rest = &SCENE[start..];
    let end = rest
        .find("\n/// O frame corrente do roteiro")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn the_spacing_scene_arms_the_mode_and_the_spacing_it_prints() {
    let body = spacing_scene_body();

    // Controle positivo: a varredura enxerga a cena. Sem ele, um `find` que devolvesse um trecho
    // vazio deixaria todo `assert!(contains)` abaixo trivialmente satisfeito ao contrário — e o
    // primeiro a falhar seria este, com o diagnóstico certo.
    assert!(
        body.contains("motion.path"),
        "a varredura não achou o corpo da cena; o gate estaria medindo uma string vazia"
    );

    assert!(
        body.contains(r#"set_param(path, "mode", 1.0)"#),
        "as fileiras de baixo TÊM de armar o modo Spacing — sem ele as quatro contam por número \
         e a cena desenha quatro linhas que afirmam demonstrar uma diferença"
    );
    assert!(
        body.contains(r#"set_param(path, "spacing", 0.5)"#),
        "o espaçamento de 0,5 é o número que a mensagem imprime como vão das duas de baixo"
    );
    assert!(
        body.contains(r#"set_param(path, "count", 9.0)"#),
        "as duas de cima são o CONTROLE, e contam por número — 9, o valor impresso"
    );
}

/// **A mensagem e a cena não podem divergir.**
///
/// Os quatro números vivem em dois lugares por natureza (o que a cena ARMA e o que ela DIZ), e é
/// exactamente o par que apodrece quando alguém afina um e esquece o outro. Este gate os prende:
/// os vãos e as contagens que o `eprintln!` promete têm de estar escritos ali.
#[test]
fn the_printed_table_says_the_numbers_the_scene_produces() {
    let body = spacing_scene_body();
    for claim in ["9 pecas,  vao 0,444", "9 pecas,  vao 0,889", "8 pecas,  vao 0,500", "16 pecas, vao 0,500"] {
        assert!(
            body.contains(claim),
            "a tabela impressa perdeu a linha {claim:?} — os números dela são gateados em \
             ph2d-node-registry-init/tests/spacing_scene.rs"
        );
    }
}
