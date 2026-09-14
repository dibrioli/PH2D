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

/// ⭐⭐⭐ **UMA LUZ DA CENA, como o MÓDULO a conhece** — quem ela é, onde está, e se está acesa.
///
/// # ⚠️ Porque ela não é a [`PointLamp`] do renderizador
///
/// Aquela é o que o **sombreamento** precisa: uma posição e uma radiância, e mais nada — ela não
/// sabe de entidades nem de olhos. Esta é o que o **canvas** precisa: o gizmo tem de saber **quem**
/// para poder escolher, e tem de ver a luz APAGADA para se poder voltar a acendê-la. ⇒ uma recolha,
/// dois consumidores; a do renderizador **deriva** desta ([`Self::lamps`]).
///
/// ⛔ *Uma segunda varredura do mundo para o canvas seria a segunda resposta a «que luzes há», e a
/// que diverge no quadro em que uma delas nasce.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLight {
    /// Os bits da entidade — é por eles que o clique a escolhe.
    pub bits: u64,
    /// Onde ela está, no MUNDO.
    pub world: [f32; 3],
    pub light: ph2d_field_ecs::FieldLight,
    /// O olho da Hierarquia. Uma luz apagada **desenha-se** e não acende.
    pub on: bool,
}

impl SceneLight {
    /// A cor da lâmpada, em linear — o que a marca do canvas pinta no miolo.
    #[must_use]
    pub fn color(&self) -> [f32; 3] {
        self.light.color
    }
}

/// **As lâmpadas que o sombreamento vê** — as ACESAS, na ordem da recolha.
#[must_use]
pub fn lamps_of(luzes: &[SceneLight]) -> Vec<PointLamp> {
    luzes
        .iter()
        .filter(|l| l.on)
        .map(|l| PointLamp {
            world: l.world,
            radiance_at_one: radiance_at_one(l.light),
        })
        .collect()
}

/// **As luzes do mundo.**
///
/// ⚠️ **`&mut` para uma LEITURA** porque `World::query` o exige — a mesma nota do
/// [`crate::materials::sync`].
pub fn of_the_world(world: &mut bevy_ecs::world::World) -> Vec<SceneLight> {
    let mut q = world.query::<(
        bevy_ecs::entity::Entity,
        &ph2d_field_ecs::FieldLight,
        Option<&ph2d_ecs::Visibility>,
    )>();
    let mut todas: Vec<(u64, ph2d_field_ecs::FieldLight, bool)> = q
        .iter(world)
        // ⭐ **O OLHO da Hierarquia apaga a luz**, e de graça: é o mesmo componente que esconde uma
        // forma. *Uma luz que não se pode desligar sem a apagar é uma luz que ninguém experimenta.*
        //
        // ⚠️ `Option`, e não `&Visibility`: o componente é OPCIONAL, e exigi-lo faria uma luz sem
        // ele desaparecer da lista **em silêncio** — um objecto que existe na Hierarquia e não
        // ilumina, sem nada a dizer porquê.
        //
        // ⚠️⚠️ **E ela FICA na lista, apagada** (14/09, o gizmo): quem a tira aqui tira-a também do
        // canvas, e uma luz sem marca no canvas não se pode voltar a acender senão pela Hierarquia.
        // *Quem filtra é o [`lamps_of`], que é o consumidor a quem o olho diz respeito.*
        .map(|(e, l, vis)| (e.to_bits(), *l, !vis.is_some_and(|v| v.hidden)))
        .collect();
    // ⚠️ Ver o topo: a soma de `f32` não é associativa.
    todas.sort_unstable_by_key(|(bits, _, _)| *bits);
    todas
        .into_iter()
        .map(|(bits, light, on)| SceneLight {
            bits,
            world: ph2d_field_ecs::world_xform(world, bevy_ecs::entity::Entity::from_bits(bits))
                .translation,
            light,
            on,
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

/// ⭐⭐⭐ **O RAIO DO DISCO da marca de uma luz**, em pixels de ecrã.
///
/// ⚠️ **É o punho do gizmo da casa** ([`crate::gizmo::GRIP_HALF_PX`]), e não um número próprio:
/// uma marca de luz é uma alça como as outras, e duas escalas de chrome na mesma janela leem-se
/// como duas ferramentas.
pub const MARK_HALF_PX: f32 = crate::gizmo::GRIP_HALF_PX;

/// Onde os raios da marca começam e acabam, em pixels — **derivados** do disco.
pub const RAY_IN_PX: f32 = MARK_HALF_PX * 1.4;
/// Ver [`RAY_IN_PX`].
pub const RAY_OUT_PX: f32 = MARK_HALF_PX * 2.4;

/// **O RAIO DE AGARRE da marca** — a que distância dela um clique ainda é dela.
///
/// ⭐ **Derivado, e é a mesma folga do vértice** ([`crate::gizmo::VERTEX_HALF_PX`] escreve-a por
/// extenso): *«um alvo que agarra exactamente onde pinta obriga a mão a acertar no pixel»*. ⇒ o
/// desenho mais meia folga do punho.
pub const MARK_GRAB_PX: f32 = MARK_HALF_PX + crate::gizmo::GRAB_PX * 0.5;

/// ⭐ **E a relação é afirmada em tempo de COMPILAÇÃO.**
///
/// ⚠️ Ela nasceu como um `#[test]`, e o clippy chamou-lhe *«esta asserção tem valor constante»* —
/// com razão: dois `const` não precisam de uma corrida de teste para serem comparados, e um gate que
/// só um `cargo test` corre é mais fraco do que um que o `cargo check` corre. *Quem encolher o
/// agarre para dentro do desenho não compila.*
const _: () = assert!(MARK_GRAB_PX > MARK_HALF_PX);

/// **A marca de uma luz, projectada** — o que o pintor desenha e o que o clique aponta.
#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub light: SceneLight,
    /// No referencial da ÁREA, como as alças do gizmo.
    pub px: [f32; 2],
}

/// ⭐⭐⭐ **ONDE CADA LUZ CAI NO ECRÃ.**
///
/// # ⛔⛔ Ela existe para haver UMA resposta, e não duas
///
/// O pintor e o teste de acerto fazem a **mesma** pergunta, e a casa já pagou essa lição duas vezes
/// neste módulo (*«a projeção é a da ÁREA e vem do dono dela — nunca uma segunda conta»*). Uma marca
/// desenhada num sítio e apanhada noutro é o defeito que se lê como *«o clique não pega»*, e que
/// nenhum dos dois lados consegue diagnosticar sozinho.
///
/// ⚠️ **As luzes ATRÁS da câmera saem da lista** — o `project` devolve `None`, e desenhar uma marca
/// no sítio onde uma projecção invertida a põe seria pintar um alvo onde não há luz nenhuma.
#[must_use]
pub fn marks(
    luzes: &[SceneLight],
    cam: &ph2d_field_render::Orbit,
    screen: ph2d_field_render::Screen,
) -> Vec<Mark> {
    luzes
        .iter()
        .filter_map(|l| {
            cam.project(l.world, screen)
                .map(|(px, _)| Mark { light: *l, px })
        })
        .collect()
}

/// ⭐⭐⭐ **QUE LUZ ESTÁ SOB ESTE PONTO** — a mais próxima, dentro do [`MARK_GRAB_PX`].
///
/// ⚠️ **A mais próxima no ECRÃ e não a mais próxima da câmera**: o que o artista aponta é o que ele
/// vê, e duas marcas sobrepostas são um caso em que qualquer regra é arbitrária — a do ecrã é a
/// única que ele consegue prever.
#[must_use]
pub fn under(
    luzes: &[SceneLight],
    cam: &ph2d_field_render::Orbit,
    screen: ph2d_field_render::Screen,
    px: [f32; 2],
) -> Option<u64> {
    marks(luzes, cam, screen)
        .into_iter()
        .map(|m| {
            let d = (m.px[0] - px[0]).hypot(m.px[1] - px[1]);
            (d, m.light.bits)
        })
        .filter(|(d, _)| *d <= MARK_GRAB_PX)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, bits)| bits)
}

#[cfg(test)]
#[path = "lights_tests.rs"]
mod tests;
