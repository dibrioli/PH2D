use super::*;

const BRANCO: [f32; 4] = [1.0; 4];

/// ⭐ **Quem mostra a folha ao vivo** (B10.3): a sprite pintada SEM cartão Animation não está à vista nem selecionada; com o
/// cartão, à vista pela seleção ou por um cartão solto preso; a aba do editor à frente mostra-a sempre; outro objeto não.
/// *Mutações: ignorar o cartão ⇒ a sprite simples selecionada «à vista» (a leitura de 98 ms volta); ignorar a aba ⇒ o
/// editor nunca se atualiza.*
#[test]
fn a_folha_ao_vivo_so_esta_a_vista_com_o_cartao_ou_a_aba() {
    let mut w = World::new();
    let simples = w
        .spawn((ph2d_render::Sprite::individual(3, [1.0; 2], BRANCO),))
        .id()
        .to_bits();
    let animada = w
        .spawn((
            ph2d_render::Sprite::individual(4, [1.0; 2], BRANCO),
            ph2d_ecs::SpriteAnimations::default(),
        ))
        .id()
        .to_bits();
    let sel = |b: u64| std::iter::once(b);
    assert!(!shown(&w, sel(simples), None, simples), "sem cartão");
    assert!(shown(&w, sel(animada), None, animada), "cartão da seleção");
    assert!(
        shown(&w, [simples, animada].into_iter(), None, animada),
        "cartão solto preso"
    );
    assert!(!shown(&w, sel(simples), None, animada), "outro selecionado");
    assert!(
        shown(&w, std::iter::empty(), Some(simples), simples),
        "a aba"
    );
    assert!(
        !shown(&w, std::iter::empty(), Some(animada), simples),
        "outra aba"
    );
}
