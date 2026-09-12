//! **O afim imagem→ecrã é construído sobre a pose de MUNDO.**
//!
//! Enio, 2026-08-19: *"se a sprite é filha de outra, não consigo pintá-la"*.
//!
//! ## O mecanismo
//!
//! [`ph2d_sprite_screen::sprite_image_to_screen_affine`] compõe `imagem → local → mundo → ecrã`, e
//! o comentário dentro dela sempre prometeu *"sprite-local meters → world"*. Mas o `Transform` que
//! ela recebia era a pose **LOCAL** da entidade. Numa sprite de **raiz** local e mundo são a mesma
//! coisa — e foi por isso que a promessa sobreviveu a **21 chamadores** sem ninguém reparar.
//!
//! Numa sprite **filha**, falta a cadeia do pai. O afim mapeia o ponteiro para outro sítio, e a
//! guarda de pegada do Painter — que usa este mesmo afim para decidir se o clique caiu sobre o
//! sprite — recusa **toda** pincelada. O sintoma não é pintar torto: é não pintar.
//!
//! ## ⚠️ Ele MUDOU DE CASA com a lei, e ficou mais forte (W2 Fase D)
//!
//! Este gate vivia em `shells/desktop/tests/it/the_painter_affine_takes_the_world_pose.rs` e lia
//! `render_loop/bgremoval_preview.rs` por `read_to_string` de **caminho fixo**. Quando o afim saiu
//! para esta folha, ele reprovou — alto, que é a metade boa (`HOWTO` §2.9: *a agulha nomeia um
//! endereço*).
//!
//! ⭐ A cura não foi reapontar o caminho: foi trocar o `read_to_string` pelo **`include_str!`**,
//! que falha em tempo de **COMPILAÇÃO** se o ficheiro voltar a mudar de sítio. O `HOWTO` §2.6
//! chama-lhe *«boa propriedade»* — remove a classe de falha inteira em vez de corrigir uma
//! instância dela. *O gémeo em runtime só falha se o teste correr; um `#[ignore]` ou um filtro e
//! ele nunca falha.*
//!
//! ⚠️ A defesa principal **não é este gate**: é o TIPO. O parâmetro passou de `&Transform` para
//! `Transform` por valor, e por isso todo chamador antigo deixou de compilar até resolver a pose.
//! *Uma convenção nova sobre a mesma assinatura teria sido esquecida no 22º sítio.* Este gate
//! guarda o resto: que ninguém desfaça a assinatura.

/// O parâmetro é por VALOR e chama-se `world_tr` — as duas metades da defesa de tipo.
///
/// ⚠️ **O NOME não mudou ao mudar de casa, de propósito.** A prova desta wave é o
/// `nextest-list-diff` com `ONLY-A = 0`, e ele indexa por NOME: renomear um teste que se move
/// lê-se exactamente como um teste PERDIDO. *Um nome melhor não vale uma prova mais fraca.*
#[test]
fn the_affine_takes_a_world_transform_by_value() {
    const FONTE: &str = include_str!("../src/lib.rs");
    assert!(
        FONTE.contains("world_tr: ph2d_ecs::Transform,"),
        "o parametro de pose do `sprite_image_to_screen_affine` deixou de ser `world_tr: \
         ph2d_ecs::Transform` (por VALOR).\n\
         Voltar a `&Transform` faz todo chamador antigo compilar outra vez — e um deles passa a \
         pose LOCAL, que numa sprite FILHA recusa toda pincelada."
    );
}
