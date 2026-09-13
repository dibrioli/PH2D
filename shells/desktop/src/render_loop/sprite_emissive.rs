//! **O passe da sprite que EMITE** — plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md) W8.
//!
//! > Enio, 2026-08-21: *"1) sprite como fonte de luz."*
//!
//! # A forma, e por que ela já existia
//!
//! O Motion já tinha um brilho HDR (`fx.glow`) e a forma dele é exactamente esta: **re-desenhar um
//! subconjunto das instâncias em isolamento** num alvo `Rgba16Float`, passar o bright-pass + blur, e
//! **somar** o resultado sobre o `game_rt` antes do tonemap. O emissor de sprites é a mesma
//! máquina com outra lista.
//!
//! ⚠️ **Reusa o [`MotionFx`](ph2d_render::MotionFx), e isso é uma decisão, não preguiça.** Ele é um
//! RT `Rgba16Float` do tamanho da janela mais a cadeia de mips — construir um segundo custaria a
//! mesma memória para fazer a mesma coisa. O que **não** se pode é os dois passes correrem
//! entrelaçados: cada um escreve o RT inteiro e consome-o logo a seguir, por isso eles correm em
//! **sequência** (emissor primeiro, glow do Motion depois) e o RT nunca é partilhado no tempo.
//!
//! # De onde vem a lista, e por que o extract não foi tocado
//!
//! ⚠️ O `sim_extract` está **no tecto de LOC** (770 linhas, com a nota `ph2d-loc-cap` no cabeçalho).
//! Acrescentar-lhe um segundo laço seria a forma óbvia e a errada.
//!
//! Não é preciso: cada entidade-espelho do [`PresentWorld`] carrega um
//! [`SimRef`](ph2d_ecs::SimRef) — o ponteiro de volta para a entidade do `sim`. Este módulo
//! **relê** as instâncias já montadas, pergunta ao `sim` quais delas emitem, e devolve uma cópia com
//! o `tint` multiplicado. *A instância que o emissor desenha é, por construção, a MESMA que o ecrã
//! desenha* — mesma pose, mesma UV, mesmo recorte —, e nenhuma das dezenas de decisões do extract
//! precisou de ser repetida aqui.
//!
//! # O que ele NÃO faz
//!
//! ⛔ **Uma sprite emissora não ilumina as vizinhas.** Isso é propagação de luz 2D — outro sistema,
//! e o `ph2d-light` que existe é um rig de sombreamento por normais, não isso. A distinção está
//! escrita no componente ([`ph2d_ecs::emissive`]), que é onde alguém vai lê-la.
//!
//! ⛔ **Não toca no desenho normal da sprite.** O halo é **somado** por cima; a sprite continua a
//! ser desenhada pelo caminho de sempre. Sem nenhuma sprite a emitir, a lista sai vazia, o passe não
//! corre, e o quadro é **byte-idêntico** — há gate.

use ph2d_ecs::{PresentWorld, SimWorld, SpriteEmissive};

/// Os parâmetros do halo. **Não são autorados** — o componente tem um knob só (a intensidade), e
/// estes descrevem o *carácter* da emissão, que é a mesma para todas as sprites.
///
/// ⚠️ **O `threshold` é `1.0` e isso é a definição, não uma afinação.** «Emitir» é ter cor acima do
/// branco; o bright-pass a 1.0 é exactamente a pergunta *"esta cor passa do que um ecrã consegue
/// mostrar?"*. Baixá-lo faria uma sprite normal (que vive toda em `[0, 1]`) começar a brilhar sem
/// ninguém pedir — e aí o emissor deixaria de ser um componente para ser um efeito global.
pub(crate) fn bloom_params() -> ph2d_render::BloomParams {
    ph2d_render::BloomParams {
        threshold: 1.0,
        // Um joelho estreito: a passagem de «não emite» a «emite» acompanha a intensidade que o
        // artista dialou, em vez de ser mascarada por uma rampa larga aqui.
        knee: 0.1,
        intensity: 1.0,
        radius: 1.0,
        saturation: 1.0,
        // Neutro: o halo herda a cor da arte. ⚠️ Um tint aqui seria um SEGUNDO sítio onde a cor da
        // luz se autora, e os dois divergiriam no dia em que alguém mexesse num só.
        tint: [1.0, 1.0, 1.0, 1.0],
        // O halo REDONDO (`stretch = 1` é o círculo que sempre shipou; o ângulo não tem efeito
        // nele) e SEM teto no bright-pass (`clamp = 0` é «desligado», o caminho literal). Os três
        // são a anamorfose e o clamp do `fx.glow` (doc 89 folha 11) — autoria de nó, não carácter
        // de emissor. ⚠️ Escritos por nome de propósito: um campo novo no `BloomParams` tem de
        // ser erro de compilação AQUI, não um default herdado em silêncio (foi assim que a
        // integração de 2026-08-22 apanhou estes três).
        stretch: 1.0,
        angle: 0.0,
        // E a OPERAÇÃO/FONTE do `fx.glow` (doc 89 folha 11) — ambas no neutro, pela mesma razão
        // que os três acima: elas são autoria de NÓ, não carácter de emissor.
        //
        // ⚠️ **O `source` importa aqui mais do que parece.** «Emitir» é ter cor acima do branco,
        // e é isso que o `threshold = 1.0` pergunta; ler o ALFA faria toda sprite opaca acender,
        // ou seja transformaria o componente `SpriteEmissive` num efeito global — exactamente o
        // que o comentário do `threshold` acima recusa.
        operation: 0.0,
        source: 0.0,
        clamp: 0.0,
        // E a MÁSCARA DE SUJIDADE apagada (doc 89 folha 11), pela terceira vez a mesma razão:
        // ela é autoria de NÓ. ⚠️ E aqui há uma segunda: a imagem dela escolhe-se pelo nome de
        // um objecto da cena, e um emissor de sprite não tem grafo nenhum onde esse nome pudesse
        // ser escrito — o knob sem a imagem não é meia-feature, é um número inerte.
        dirt_intensity: 0.0,
    }
}

/// **As instâncias que emitem, com o `tint` já multiplicado.** Vazio quando nenhuma sprite emite —
/// e é esse vazio que mantém o quadro byte-idêntico.
///
/// ⚠️ **Multiplica só o RGB, nunca o alfa.** O alfa é *cobertura*: escalá-lo faria uma sprite
/// meio-transparente virar opaca ao emitir, e o halo apareceria com a forma errada. O que passa do
/// branco é a cor; a forma continua a ser a que o artista desenhou.
///
/// ⚠️ **Pela porta que leva a MALHA junto** (`LiftedInstances::collect_from`, plano
/// `docs/Skeleton/03` W3): o halo de uma imagem presa ao esqueleto sai da imagem DEFORMADA — uma
/// cópia só do `RenderInstance` brilhava com o quad de repouso. O `sim` é lido DENTRO da varredura
/// do `present`: são dois mundos de tipos distintos, e o compilador recusa-o no dia em que o
/// deixarem de ser.
pub(crate) fn collect(
    sim: &SimWorld,
    present: &mut PresentWorld,
    out: &mut ph2d_render::LiftedInstances,
) {
    out.collect_from(present, |sim_entity, inst| {
        let Some(em) = sim.world().get::<SpriteEmissive>(sim_entity).copied() else {
            return false;
        };
        if !em.emits() {
            return false;
        }
        let k = em.clamped();
        inst.tint[0] *= k;
        inst.tint[1] *= k;
        inst.tint[2] *= k;
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{GlobalTransform, SimRef, Transform};
    use ph2d_render::{LiftedInstances, RenderInstance};

    /// Uma instância mínima com um `tint` reconhecível.
    ///
    /// ⚠️ Escrita por extenso porque a `RenderInstance` **não implementa `Default`** — e é
    /// deliberado: ela é `Pod` e vai para a GPU, onde um campo esquecido a zero desenha algo
    /// errado em silêncio em vez de falhar.
    fn instance(tint: [f32; 4]) -> RenderInstance {
        RenderInstance {
            world_pos: [0.0, 0.0],
            size: [1.0, 1.0],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint,
            basis: RenderInstance::IDENTITY_BASIS,
            texture_id: 0,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            z_order: 0,
            sampling: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 0,
        }
    }

    /// Monta um `sim` com uma entidade e o `present` com o espelho dela.
    fn world_pair(emissive: Option<SpriteEmissive>, tint: [f32; 4]) -> (SimWorld, PresentWorld) {
        let mut sim = SimWorld::default();
        let e = sim.world_mut().spawn((Transform::default(),)).id();
        if let Some(em) = emissive {
            sim.world_mut().entity_mut(e).insert(em);
        }
        let mut present = PresentWorld::new();
        present.world_mut().spawn((
            SimRef(e),
            GlobalTransform::from_transform(Transform::default()),
            instance(tint),
        ));
        (sim, present)
    }

    /// **Sem o componente, a lista é VAZIA** — e é isso que faz o passe não correr e o quadro ficar
    /// byte-idêntico ao de antes desta feature existir.
    #[test]
    fn a_sprite_without_the_component_does_not_emit() {
        let (sim, mut present) = world_pair(None, [1.0, 1.0, 1.0, 1.0]);
        let mut out = LiftedInstances::default();
        collect(&sim, &mut present, &mut out);
        assert!(out.is_empty(), "uma sprite sem `SpriteEmissive` nao emite");
    }

    /// ⚠️ **Zero também não emite.** O componente presente e a zero é o estado que o artista deixa
    /// ao desligar o knob sem apagar a linha — e ele tem de custar exactamente nada.
    #[test]
    fn an_intensity_of_zero_does_not_emit() {
        let (sim, mut present) = world_pair(Some(SpriteEmissive(0.0)), [1.0, 1.0, 1.0, 1.0]);
        let mut out = LiftedInstances::default();
        collect(&sim, &mut present, &mut out);
        assert!(out.is_empty());
    }

    /// **O `tint` sai multiplicado no RGB, e o alfa INTACTO.**
    #[test]
    fn the_colour_is_scaled_and_the_alpha_is_not() {
        let (sim, mut present) = world_pair(Some(SpriteEmissive(4.0)), [1.0, 0.5, 0.25, 0.5]);
        let mut out = LiftedInstances::default();
        collect(&sim, &mut present, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out.instances()[0].tint[0], 4.0);
        assert_eq!(out.instances()[0].tint[1], 2.0);
        assert_eq!(out.instances()[0].tint[2], 1.0);
        assert_eq!(
            out.instances()[0].tint[3],
            0.5,
            "o alfa e' COBERTURA — escala-lo faria o halo sair com a forma errada"
        );
    }

    /// **A intensidade é presa ao tecto da representação**, e um valor absurdo não vira infinito.
    #[test]
    fn an_absurd_intensity_is_clamped_instead_of_saturating() {
        let (sim, mut present) = world_pair(Some(SpriteEmissive(1.0e9)), [1.0, 1.0, 1.0, 1.0]);
        let mut out = LiftedInstances::default();
        collect(&sim, &mut present, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out.instances()[0].tint[0], ph2d_ecs::EMISSIVE_MAX);
        assert!(
            out.instances()[0].tint[0].is_finite(),
            "um tint infinito atravessa o blur e apaga o quadro"
        );
    }

    /// ⚠️ **O buffer é REUSADO entre quadros** (HR-3, zero-alloc), por isso `collect` limpa-o. Sem
    /// isto a lista crescia sem fim e o halo ficava mais forte a cada quadro — o género de defeito
    /// que aparece como «o brilho vai aumentando sozinho» e que ninguém liga ao buffer.
    #[test]
    fn the_buffer_is_cleared_so_frames_do_not_accumulate() {
        let (sim, mut present) = world_pair(Some(SpriteEmissive(2.0)), [1.0, 1.0, 1.0, 1.0]);
        let mut out = LiftedInstances::default();
        for _ in 0..3 {
            out.push(instance([9.0, 9.0, 9.0, 9.0]), None);
        }
        collect(&sim, &mut present, &mut out);
        assert_eq!(out.len(), 1, "o lixo do quadro anterior ficou na lista");
    }
}
