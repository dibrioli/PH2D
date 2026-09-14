//! ⭐⭐⭐ **AS LUZES DA CENA** — a lâmpada como OBJECTO 3D (ordem do dono, 2026-09-14: *«a luz deve
//! virar objeto 3d como nos app 3d»*).
//!
//! # A lei, numa frase
//!
//! Uma luz é uma **entidade** com um [`ph2d_field_ecs::FieldLight`] e uma pose. ⇒ ela aparece na
//! Hierarquia, escolhe-se, renomeia-se, apaga-se, esconde-se e **move-se com o mesmo gizmo** que
//! move uma forma — tudo isso sem uma linha de código próprio, porque é o que ser uma entidade
//! desta cena já significa (`docs/3DModeling` W5).
//!
//! # ⛔ O que este ficheiro NÃO faz, e porquê
//!
//! Ele não é um rig. A [`ph2d_light::LightRig`] é o estúdio **ancorado no ecrã** da tinta e da
//! escultura, e o modo Render do modelador deixou de a usar: as luzes desta cena são as luzes desta
//! cena. ⚠️ **O CÉU continua ancorado no ecrã** (`docs/Render3d/05` §16), e essa é uma incoerência
//! declarada: a sala não roda com a câmera e as lâmpadas rodam. *O céu de MUNDO é outra decisão, e
//! ela chega com o resto dos modos de arte* — o dono declarou esta fatia **provisória**.
//!
//! # ⚠️ A ordem é a da varredura, e ela tem de ser ESTÁVEL
//!
//! O sombreamento soma as luzes, e a soma de `f32` **não é associativa**: duas ordens dão dois
//! últimos bits. ⇒ a lista sai ordenada pelos bits da entidade, que é a mesma disciplina do
//! `canonicalize` do editor. *Sem isso o mesmo documento dava duas imagens conforme a ordem em que
//! o ECS calhou de arrumar os arquétipos.*

use ph2d_field_render::PointLamp;

/// ⭐⭐⭐ **ONDE A PRIMEIRA LUZ NASCE** — em mundo, derivado da câmera e do rig.
///
/// # ⚠️ Ela é DERIVADA do rig que ela substitui, e não escolhida
///
/// O [`ph2d_light::Light::KEY`] é *«superior-esquerda a `30°`»* — `230°` de azimute e `30°` de
/// elevação —, e esse número foi **afinado pelo Enio em 2026-07-12**. ⇒ a primeira luz desta cena
/// nasce **nessa direcção**, a uma distância de onde ela ilumina como a lâmpada do rig iluminava.
///
/// ⚠️ **A DISTÂNCIA sai da intensidade**: a unidade da força é *«a intensidade do rig medida a uma
/// unidade»* (ver `ph2d_field_ecs::FieldLight`), logo a `1` unidade a peça recebe o que recebia. A
/// peça de omissão mede `~0,6` de raio, então `1` põe a lâmpada **fora** dela e perto o suficiente
/// para a queda `1/r²` ser um gesto que se vê.
///
/// ⛔ **O eixo é o do CANVAS, virado uma vez.** O rig é autorado em espaço de tela (`y` para baixo)
/// e o mundo tem `y` para cima — a mesma negação que a [`crate::render_light::lamps`] faz, e que a
/// casa já pagou uma vez (*«sem esta negação a mesma lâmpada acende a pintura por cima e a escultura
/// por baixo»*).
/// ⚠️⚠️ **E é uma FUNÇÃO da câmera, não uma constante.** A direcção do rig é de **ECRÃ**; a luz é um
/// objecto de **MUNDO**. Onde «superior-esquerda» cai no mundo depende de para onde a câmera olha —
/// e a primeira redacção disto era um literal em mundo, o que punha a lâmpada num sítio arbitrário
/// em toda câmera que não fosse a de abertura. *Uma direcção de tela escrita como um ponto de mundo
/// é a mesma classe de erro que o sinal de `y` desta casa já pagou.*
#[must_use]
pub fn opening_place(cam: &ph2d_field_render::Orbit) -> [f32; 3] {
    // A lâmpada do rig, em espaço de VISTA — pela porta, nunca por um literal.
    let ecra = crate::render_light::lamps(&ph2d_light::LightRig::default())
        .first()
        .map_or([0.0, 0.0, 1.0], |l| l.to_light);
    let (right, up, toward_eye) = cam.basis();
    let r = opening_distance(cam);
    [0, 1, 2].map(|i| {
        cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
    })
}

/// **A que distância a primeira luz nasce** — em unidades de mundo.
///
/// ⭐ **Derivada da CÂMERA, e não escolhida:** o `half_extent` é a meia-largura do que ela enquadra,
/// logo `2×` põe a lâmpada **fora da peça** e a **um passo de zoom** de estar à vista. *Uma lâmpada
/// que nasce dentro da peça está por dentro do sólido; uma que nasce a dez unidades é uma lâmpada
/// que o artista tem de ir procurar.*
#[must_use]
pub fn opening_distance(cam: &ph2d_field_render::Orbit) -> f32 {
    2.0 * cam.half_extent
}

/// ⭐⭐⭐ **A PRIMEIRA LUZ, inteira** — onde ela está E que força tem.
///
/// # ⚠️ A força é `r²`, e isso é uma DERIVAÇÃO e não um gosto
///
/// A unidade da intensidade é *«a do rig, medida a uma unidade»* e a queda é `1/r²` ⇒ uma luz de
/// força `r²` a `r` unidades entrega, **no centro da peça, exactamente o que a lâmpada do rig
/// entregava**. *A cena abre com a luz que ela tinha; o que muda é que agora ela tem um sítio.*
///
/// ⚠️⚠️ **E ela NÃO reproduz a imagem antiga, nem pode** — medido na esfera do `docs/Render3d/05`
/// §6, a peça move-se `~21` bytes de média. Uma luz direccional entrega a mesma radiância em todo o
/// lado; um ponto entrega `1/r²`, e o lado de lá da peça está mais longe. *A troca é o preço de a
/// luz passar a estar em algum sítio, e é o que o dono pediu.*
#[must_use]
pub fn opening_light(cam: &ph2d_field_render::Orbit) -> ([f32; 3], ph2d_field_ecs::FieldLight) {
    let r = opening_distance(cam);
    (
        opening_place(cam),
        ph2d_field_ecs::FieldLight {
            intensity: r * r,
            ..ph2d_field_ecs::FieldLight::default()
        },
    )
}

/// **As luzes do mundo**, prontas a sombrear.
///
/// ⚠️ **`&mut` para uma LEITURA** porque `World::query` o exige — a mesma nota do
/// [`crate::materials::sync`].
pub fn of_the_world(world: &mut bevy_ecs::world::World) -> Vec<PointLamp> {
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &ph2d_field_ecs::FieldLight,
        Option<&ph2d_ecs::Visibility>,
    )>();
    let mut vivas: Vec<(u64, ph2d_field_ecs::FieldLight)> = q
        .iter(world)
        // ⭐ **O OLHO da Hierarquia apaga a luz**, e de graça: é o mesmo componente que esconde uma
        // forma. *Uma luz que não se pode desligar sem a apagar é uma luz que ninguém experimenta.*
        //
        // ⚠️ `Option`, e não `&Visibility`: o componente é OPCIONAL, e exigi-lo faria uma luz sem
        // ele desaparecer da lista **em silêncio** — um objecto que existe na Hierarquia e não
        // ilumina, sem nada a dizer porquê.
        .filter(|(_, _, vis)| !vis.is_some_and(|v| v.hidden))
        .map(|(e, l, _)| (e.to_bits(), *l))
        .collect();
    // ⚠️ Ver o topo: a soma de `f32` não é associativa.
    vivas.sort_unstable_by_key(|(bits, _)| *bits);
    vivas
        .into_iter()
        .map(|(bits, l)| {
            let e = bevy_ecs::entity::Entity::from_bits(bits);
            PointLamp {
                world: ph2d_field_ecs::world_xform(world, e).translation,
                radiance_at_one: radiance_at_one(l),
            }
        })
        .collect()
}

/// **A radiância que uma luz entrega a UMA unidade de distância.**
///
/// ⚠️ **O `π` é o mesmo da [`crate::render_light::lamps`]**, e por isso mora ao lado dela em espírito:
/// o rig da casa promete que *«uma superfície plana de frente para uma luz de intensidade `1`
/// devolve `1`»*, e o MaterialX recebe a **radiância que chega**, que para uma difusa branca é
/// `L/π`. *Uma segunda tradução do `π` aqui seria a segunda resposta à mesma pergunta.*
#[must_use]
pub fn radiance_at_one(l: ph2d_field_ecs::FieldLight) -> [f32; 3] {
    l.color
        .map(|c| c.max(0.0) * l.intensity.max(0.0) * core::f32::consts::PI)
}

/// ⭐⭐⭐ **AS LUZES SEGUEM A CENA** — chamada uma vez por quadro, ao lado da tabela de materiais.
///
/// ⚠️ **Ela refaz SEMPRE**, e não só quando o documento muda: uma luz não é geometria, logo o
/// `doc_mudou` que governa a tabela de materiais nada diz sobre ela — arrastar a luz não recoze o
/// documento. *Um `dirty` próprio seria uma terceira fonte de verdade sobre um passeio de
/// microssegundos por uma lista de comprimento `1`.*
///
/// ⚠️ **E ela larga o pedido guardado dos viewports quando a lista MUDA**, como o `set_look`: sem
/// isso o laço do preview responde *«nada mudou»* — a câmera, o tamanho e o documento são os mesmos
/// — e o quadro fica com a luz antiga até alguém tocar na peça. *É o congelador que o doc do
/// `Viewport::requested` já descreve.*
pub(crate) fn sync(sim: &mut ph2d_ecs::SimWorld) {
    let agora = of_the_world(sim.world_mut());
    crate::smoke::with_smoke(|s| {
        if *s.lights != agora {
            s.lights = std::sync::Arc::new(agora);
            s.forget_requests();
        }
    });
}

#[cfg(test)]
#[path = "lights_tests.rs"]
mod tests;
