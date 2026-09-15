//! ⭐⭐⭐ **A UV DE ORIGEM debaixo do ponteiro, para as ferramentas que APONTAM** — hoje as três
//! entradas de canvas da Remoção de fundo (o conta-gotas, o pincel de protecção e o *Add area*).
//!
//! ⚠️ **Ela vive na FOLHA e não na shell** (2026-09-15): é a pergunta INVERSA do
//! [`crate::sprite_image_to_screen_affine`] — mesmo assunto, mesma casa —, é PURA (entram os dois
//! mundos, a câmera e um ponto; sai uma UV) e a `shells/desktop` só encolhe.
//!
//! ⛔⛔ **As três faziam a MESMA conta errada, e por escrito:** cada uma montava uma CAIXA ALINHADA
//! AOS EIXOS a partir de `translation ± size/2` e dividia o ecrã por ela. Isso ignora **três** coisas
//! que o desenho honra — a **rotação** (uma sprite rodada amostrava na diagonal errada), a **pose do
//! PAI** (a caixa saía da pose LOCAL, então numa sprite filha o ponteiro caía fora da pegada: o
//! mesmo defeito que o Painter pagou em 2026-08-19) e a **MALHA** (numa arte presa ao esqueleto e
//! dobrada, o texel debaixo do dedo não é o que a caixa diz). *Três cópias da mesma conta, e a
//! terceira coisa ninguém tinha sequer perguntado.*
//!
//! ⇒ uma porta, dois estados, e cada um é a lei que já existe noutro sítio:
//! - a sprite é desenhada como **malha** ⇒ a [`ph2d_render::mesh_uv`], a mesma que o pincel usa;
//! - é desenhada como **quad** ⇒ o afim de `ph2d_sprite_screen`, o mesmo que a prévia desenha —
//!   ⚠️ e é ele, não a lei do quad do renderer, porque **numa folha ele desdobra a grelha** e a
//!   prévia da Remoção de fundo é a imagem INTEIRA.
//!
//! ⚠️ **As dimensões em pixels CANCELAM-SE** e por isso não entram na assinatura: o afim leva
//! `0..iw` sobre o quad, e a fracção é `img / iw`. Passa-se `1 × 1`, que é a unidade em que a
//! resposta é dada — *um número inventado aqui seria lido como «a resolução importa»*.

/// A resposta da porta — **três** estados porque o chamador tem três coisas diferentes a fazer, e
/// nenhuma delas é «um `Option` com um `if` ao lado» (é a mesma razão da [`ph2d_render::MeshUv`]).
pub enum UvSobOPonteiro {
    /// A UV de origem, **não cortada** — quem quer saber se caiu dentro pergunta `(0.0..1.0)`,
    /// exactamente como as três entradas já faziam com a caixa.
    Uv(f32, f32),
    /// Há sprite, e ela **não se desenha ali**: a arte está dobrada e este ponto está fora dela.
    /// O gesto consome o clique e não faz nada — que é o que a caixa já fazia com um ponto de fora.
    ForaDaArte,
    /// A selecção **não é uma sprite desenhada**. ⚠️ Estado próprio de propósito: as três entradas
    /// recusam o clique aqui (ele cai para o pick / o arrasto da cena) e **consomem-no** nos outros
    /// dois — juntar isto ao `ForaDaArte` trocaria, em silêncio, quem fica com o botão.
    SemSujeito,
}

/// Ver o cabeçalho do módulo e a [`UvSobOPonteiro`].
pub fn uv_sob_o_ponteiro(
    sim: &ph2d_ecs::SimWorld,
    present: &mut ph2d_ecs::World,
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    bits: u64,
    px: f32,
    py: f32,
) -> UvSobOPonteiro {
    let entity = ph2d_ecs::Entity::from_bits(bits);
    // ⚠️ Pose de MUNDO, nunca a local — vide o doc do `sprite_image_to_screen_affine`.
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return UvSobOPonteiro::SemSujeito;
    };
    let grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    // ⚠️ **`starting = true` e pegada `[0, 0]`:** isto é uma porta de APONTAR — a pegada só tem
    // sentido para quem vai pousar um disco de tinta, e um gesto que aponta começa a cada evento.
    // ⇒ fora da arte desenhada a porta RECUSA, e aqui isso é `None`.
    match ph2d_render::mesh_uv(
        present,
        bits,
        camera.screen_to_world((px, py), window),
        true,
        [0.0, 0.0],
    ) {
        ph2d_render::MeshUv::Use { u, v, .. } => UvSobOPonteiro::Uv(u, v),
        ph2d_render::MeshUv::Refuse => UvSobOPonteiro::ForaDaArte,
        ph2d_render::MeshUv::Quad => {
            // A UNIDADE: ver o cabeçalho — `img / 1` já É a fracção.
            let affine =
                crate::sprite_image_to_screen_affine(1, 1, tr, sprite, grid, camera, window);
            let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
            UvSobOPonteiro::Uv(img.x as f32, img.y as f32)
        }
    }
}
