//! **O tecto da emissão é UM número, escrito em dois sítios** — plano
//! [`docs/Sprite_projeto/18`](../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md) W8.
//!
//! # Por que há duas cópias
//!
//! `ph2d_ecs::EMISSIVE_MAX` é a lei: o tecto da **representação** (o meio-float do `GameRt`).
//! `ph2d_editor::EMISSIVE_MAX_UI` é a cópia que o slider do Inspector usa para mapear o seu
//! curso `0..1` na intensidade real.
//!
//! ⚠️ **A cópia não é descuido — é a seta de dependência.** O painel é *chrome* e **não** depende do
//! `ph2d-ecs`: essa seta está ausente de propósito (o painel desenha; o ECS é o modelo). Sem
//! dependência não há como importar a constante, e o que sobra é copiá-la **com alguém a medir que
//! as duas concordam**. É a mesma resposta que este projeto já deu para um shader que não pode
//! importar Rust.
//!
//! ⚠️ **Este teste vive no SHELL** porque o shell é o único sítio que vê as duas crates. Pô-lo em
//! qualquer uma delas seria impossível — cada uma só conhece metade da afirmação.
//!
//! # O que se parte sem ele
//!
//! Nada compila mal e nada fica vermelho. O slider passa a mapear para outra escala: o artista
//! arrasta até ao fim, lê `64` na chip, e a sprite emite `32` — ou emite `128` e satura. *Um número
//! duplicado não diverge com um erro; diverge com uma discordância silenciosa.*

/// **O tecto é o mesmo dos dois lados.**
#[test]
fn the_emissive_ceiling_is_one_law() {
    assert_eq!(
        ph2d_ecs::EMISSIVE_MAX,
        ph2d_editor::EMISSIVE_MAX_UI,
        "o teto da emissao divergiu entre o MODELO (`ph2d_ecs::EMISSIVE_MAX`) e a UI \
         (`ph2d_editor::EMISSIVE_MAX_UI`).\n\n\
         O slider do Inspector guarda `0..1` e multiplica por este numero para chegar a \
         intensidade real. Com os dois em desacordo, o artista arrasta ate' ao fim, le' um numero \
         na chip, e a sprite emite outro -- sem erro nenhum.\n\n\
         O numero VERDADEIRO e' o do `ph2d-ecs`: ele descreve o que o meio-float do `GameRt` \
         aguenta, e esta' documentado ao lado da constante. Corrija a COPIA."
    );
}

/// ⚠️ **Controle: o tecto não é zero nem infinito.** Sem isto, `0.0` dos dois lados passaria — e um
/// tecto de zero faz o slider inteiro mapear para «não emite», que é uma feature morta que nenhum
/// outro gate apanha.
#[test]
fn the_ceiling_is_a_usable_number() {
    assert!(
        ph2d_ecs::EMISSIVE_MAX > 1.0 && ph2d_ecs::EMISSIVE_MAX.is_finite(),
        "o teto da emissao ({}) tem de ser finito e maior que 1.0 — abaixo de 1.0 o multiplicador \
         nunca empurra a cor acima do branco, e o bright-pass do bloom nunca a encontra: o slider \
         inteiro passaria a nao fazer nada",
        ph2d_ecs::EMISSIVE_MAX
    );
}
