//! `PH2D_EMISSIVE_SMOKE` — **a sprite como fonte de luz**, ao lado da gémea apagada.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_EMISSIVE_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! Duas sprites iguais, lado a lado. A da **esquerda** não emite; a da **direita** carrega
//! [`ph2d_ecs::SpriteEmissive`] e **sangra luz** para o fundo à volta dela.
//!
//! ⚠️ **A da esquerda existe para o «antes» estar no ecrã.** Um halo sozinho parece só uma sprite
//! clara; ao lado da gémea apagada ele é inconfundível. É a mesma lei que a cena do dither aprendeu
//! à força: *o olho compara, não mede*.
//!
//! # A fixture, e por que ela é um disco com borda macia
//!
//! ⚠️ **O halo herda a FORMA e a COR da arte.** Um quadrado emitiria um halo quadrado — legível, mas
//! ninguém acredita nele como luz. Um disco com a borda a esbater dá o que o olho lê como uma
//! lâmpada. E a cor é **quente**: um halo branco lê-se como sobre-exposição; um âmbar lê-se como
//! *aceso*.
//!
//! # O que este smoke NÃO mostra
//!
//! ⛔ **A sprite da direita não ilumina a da esquerda.** Emitir e iluminar são sistemas diferentes
//! (ver [`ph2d_ecs::emissive`]): aqui a luz **sangra** sobre o fundo, ela não incide sobre os
//! vizinhos com atenuação e sombra. Se as duas parecerem interagir, é o halo a passar por cima — e é
//! honesto dizê-lo antes de alguém concluir o contrário.

use ph2d_asset::AssetDb;
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;

/// Lado do disco, em pixels.
const SIDE: u32 = 256;
/// A intensidade da gémea acesa.
///
/// ⚠️ **`6.0` é o que a MEDIÇÃO da cena pede, não um número bonito.** O bright-pass corta a `1.0`
/// (é a definição de «passa do branco»); a cor mais clara do disco é `1.0` no centro, por isso
/// qualquer coisa acima de `1` já emite. Abaixo de ~`3` o halo existe mas é tímido num ecrã claro;
/// a partir de ~`32` ele leva o disco todo e vira borrão. `6` mostra a coisa sem a caricaturar.
const INTENSITY: f32 = 6.0;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_EMISSIVE_SMOKE").is_some()
}

/// Um disco âmbar com a borda a esbater, em RGBA8 reto.
fn lamp_pixels() -> Vec<u8> {
    let r = SIDE as f32 * 0.5;
    let mut out = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let dx = x as f32 + 0.5 - r;
            let dy = y as f32 + 0.5 - r;
            let d = (dx * dx + dy * dy).sqrt() / r;
            // Borda macia no último quinto do raio: o halo do bloom segue a cobertura, e uma borda
            // dura daria um anel visível à volta do disco em vez de uma luz.
            let cover = ((1.0 - d) / 0.2).clamp(0.0, 1.0);
            // Âmbar quente. LITERAL-COLOR-OK: fixture de cena de smoke, não chrome de UI.
            let (rr, gg, bb) = (255.0_f32, 190.0_f32, 90.0_f32);
            let a = (cover * 255.0).round() as u8;
            out.extend_from_slice(&[rr as u8, gg as u8, bb as u8, a]);
        }
    }
    out
}

/// Sobe o disco como sprite de textura própria.
fn spawn_lamp(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    centre: Vec2,
    world: [f32; 2],
    label: &str,
) -> Option<u64> {
    let pixels = lamp_pixels();
    let texture_id = match renderer.acquire_individual(SIDE, SIDE, &pixels) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[emissive-smoke] `{label}` nao subiu para a GPU: {e}");
            return None;
        }
    };
    let pixels_id = asset_db.insert_image_rgba8(SIDE, SIDE, pixels);
    let (_, bits) = crate::image_import::spawn_sprite(
        sim,
        crate::image_import::PackedSource::Individual {
            texture_id,
            pixels_id,
        },
        centre,
        world,
        label,
    );
    Some(bits)
}

/// Monta a cena. Devolve os bits da sprite **acesa** — é a que responde à pergunta.
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    pixels_per_meter: f32,
) -> Option<u64> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let world = [SIDE as f32 / ppm, SIDE as f32 / ppm];
    // Um vão largo entre as duas: o halo precisa de fundo para sangrar, e encostadas uma
    // esconderia o efeito da outra.
    let gap = world[0] * 0.6;
    spawn_lamp(
        sim,
        renderer,
        asset_db,
        Vec2::new(-(world[0] + gap) * 0.5, 0.0),
        world,
        "Lamp · off",
    );
    let lit = spawn_lamp(
        sim,
        renderer,
        asset_db,
        Vec2::new((world[0] + gap) * 0.5, 0.0),
        world,
        "Lamp · emissive",
    )?;
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(lit))
        .insert(ph2d_ecs::SpriteEmissive(INTENSITY));
    Some(lit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **A fixture tem de ter uma borda MACIA.** Um disco de borda dura daria um anel em vez de
    /// uma luz, e a cena passaria a ensinar a coisa errada — o modo de falha mais provável aqui é
    /// alguém «limpar» a rampa de cobertura.
    #[test]
    fn the_lamp_has_a_soft_edge() {
        let px = lamp_pixels();
        let alpha_at = |x: u32, y: u32| px[((y * SIDE + x) * 4 + 3) as usize];
        let mid = SIDE / 2;
        assert_eq!(alpha_at(mid, mid), 255, "o centro tem de ser opaco");
        assert_eq!(alpha_at(0, 0), 0, "o canto tem de ser transparente");
        // Entre o centro e a borda existem valores INTERMÉDIOS — é isso que faz a borda macia.
        let ramp = (0..SIDE)
            .map(|x| alpha_at(x, mid))
            .filter(|&a| a > 0 && a < 255)
            .count();
        assert!(
            ramp >= 8,
            "so' {ramp} pixels de rampa na linha do meio — a borda ficou dura e o halo vai ler \
             como um anel em vez de uma luz"
        );
    }

    /// **A intensidade tem de passar do branco.** Abaixo de `1.0` o bright-pass nunca a encontra e
    /// a cena mostra duas sprites iguais.
    #[test]
    fn the_intensity_actually_crosses_the_bright_pass() {
        // ⚠️ **Contra o limiar REAL do passe, não contra o literal `1.0`.** Comparar dois literais
        // seria uma tautologia (o clippy diz isso, e tem razão) e não mediria nada: o dia em que
        // alguém mexer no `bloom_params` do passe é exactamente o dia em que esta cena tem de
        // reprovar, e só ler o número de lá o consegue.
        let threshold = crate::render_loop::sprite_emissive::bloom_params().threshold;
        assert!(
            INTENSITY > threshold,
            "a intensidade da cena ({INTENSITY}) nao passa do limiar do bright-pass ({threshold}): \
             o halo nao existiria e as duas sprites sairiam iguais"
        );
        // E que o componente a aceita **sem a cortar** — o clamp é a autoridade, não este ficheiro.
        assert_eq!(
            ph2d_ecs::SpriteEmissive(INTENSITY).clamped(),
            INTENSITY,
            "a cena pede uma intensidade que o componente corta — o halo sairia diferente do que \
             este ficheiro diz"
        );
    }
}
