//! **O BAKE DA FOLHA** — as peças deixam de ser N imagens e passam a ser N janelas para UMA.
//!
//! Plano [`docs/Sprite_projeto/17`] §7.3, W5.2. O que ele faz, em uma frase: lê os pixels de cada
//! filho, redimensiona-os para o tamanho que eles ocupam **na folha**, compõe tudo numa imagem, e
//! reata cada filho a uma **região** dela.
//!
//! ## Duas saídas, o mesmo bake
//!
//! 1. **Reatar** — cada peça vira `SpriteSheetRef` + uma janela na textura partilhada: N sprites,
//!    uma textura, um draw call (plano §6).
//! 2. **Exportar** — a mesma [`AuthoredSheet`] escreve `.png` + `.json` ([`crate::sheet_export`]),
//!    e é isso que torna a ferramenta reversível: o que sai daqui re-importa por esta mesma porta.
//!
//! ## O que ele NÃO faz, e porquê
//!
//! ⚠️ **Não re-arranja.** As peças estão onde o **artista** as arrastou, e assar é honrar isso —
//! por isso o núcleo chamado é o [`ph2d_sprite_sheet::compose`] (compõe em rects DADOS) e não o
//! `pack` (que escolhe os rects). *Quem escolhe e quem honra são duas perguntas, e só a primeira
//! tem opinião.* Quem quer o arranjo automático tem o item "Auto-Arrange Pieces", que é outro
//! gesto, com outro nome, no mesmo menu.
//!
//! ⚠️ **Recusa quando a folha está doente.** Sobreposição faz uma peça conter os pixels da vizinha;
//! transbordo faz a região declarada apontar para fora da imagem. Nos dois casos o `.png` e o
//! `.json` sairiam a discordar um do outro, e esse defeito **só aparece no consumidor**, meses
//! depois, noutro programa. A moldura já acende vermelha (`crate::sheet_bounds::health`) e o
//! toast nomeia o remédio — *recusar apontando para a cura é melhor do que assar uma folha
//! partida.*

use std::collections::BTreeMap;

use ph2d_asset::AssetDb;
use ph2d_ecs::{Entity, Name, SimWorld, SpriteSheetFrame, SpriteSheetRef};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};
use ph2d_sprite_sheet::{AuthoredSheet, PackInput};

use crate::hero_intents::texture_edit;

/// O canto superior-esquerdo de uma peça **em pixels da folha**, a partir da pose dela.
///
/// ⚠️ **É a inversa EXATA do `sheet_frame::place`** (que escreve a pose a partir do retângulo), e
/// tem de ser: uma peça que o auto-arranjo pôs em `(64, 32)` tem de assar em `(64, 32)`, não em
/// `(63, 32)`. As duas cruzam a mesma inversão de eixo — a folha conta pixels com `(0,0)` no canto
/// superior-esquerdo, o mundo tem `+y` para cima — e escrevê-la em dois sítios é como os dois
/// passam a discordar num pixel, que é o suficiente para uma franja aparecer no jogo.
///
/// Devolve `i64` de propósito: uma peça arrastada para fora dá negativo, e um `u32` daria a volta
/// e produziria um canto absurdo que "cabe". Quem recusa é o chamador, com o nome da peça.
pub(crate) fn corner_px(
    center: [f32; 2],
    half: [f32; 2],
    sheet_half: [f32; 2],
    pixels_per_meter: f32,
) -> [i64; 2] {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let x = ((center[0] - half[0] + sheet_half[0]) * ppm).round();
    // O `y` inverte: o topo da folha (`+sheet_half`) é a linha 0.
    let y = ((sheet_half[1] - center[1] - half[1]) * ppm).round();
    [x as i64, y as i64]
}

/// Uma peça pronta a compor: a entidade, o nome da região, os pixels já no tamanho da folha, e o
/// canto onde ela assenta.
struct Baked {
    entity: Entity,
    name: String,
    input: PackInput,
    at: [u32; 2],
}

/// **Compõe** a folha: lê os filhos e devolve a imagem + as regiões. **Não escreve no documento.**
///
/// ⚠️ Está separada do [`bake`] porque os dois consumidores querem metades diferentes: *assar*
/// reata as peças à textura partilhada (muda o documento, tem undo); *exportar* só quer os bytes
/// para os gravar em disco. Uma exportação que reatasse em silêncio faria um pedido de ficheiro
/// mudar a cena — e o artista descobriria pelo undo, que é o pior sítio para descobrir.
///
/// Devolve também os pares `(entidade, nome de região)`, que é o que o [`bake`] precisa para
/// reatar. `None` quando recusou — e recusar **sempre toasta a razão**, porque um botão que não
/// faz nada e não diz nada é indistinguível de um botão partido.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compose_sheet(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, ph2d_asset::AssetId>,
    next_sheet_id: &mut u32,
    sheet_bits: u64,
    toasts: &mut ToastQueue,
) -> Option<(AuthoredSheet, Vec<(Entity, String)>)> {
    let sheet = Entity::from_bits(sheet_bits);
    let Some(cfg) = sim.world().get::<SpriteSheetFrame>(sheet).copied() else {
        toasts.push(Toast::warning("Bake Sheet: select a sheet"));
        return None;
    };
    // ⚠️ A recusa vem ANTES de qualquer leitura de GPU: ler N texturas para depois descobrir que a
    // folha está doente paga o custo caro do caminho que ia ser deitado fora.
    let health = crate::sheet_bounds::health(sim, sheet);
    if !health.is_ok() {
        let what = match (health.overlap, health.overflow) {
            (true, true) => "pieces overlap and some fall outside",
            (true, false) => "pieces overlap",
            _ => "some pieces fall outside the sheet",
        };
        toasts.push(Toast::error(format!(
            "Bake Sheet: {what} - fix it first (Auto-Arrange Pieces)"
        )));
        return None;
    }
    let Some(sheet_half) = crate::sheet_bounds::sheet_half_local(sim, sheet) else {
        toasts.push(Toast::warning("Bake Sheet: select a sheet"));
        return None;
    };
    let size_px = cfg.pixels_for(sheet_half[0] * 2.0).max(1);

    let baked = read_pieces(
        sim,
        renderer,
        asset_db,
        atlas_asset_map,
        sheet,
        &cfg,
        sheet_half,
    );
    if baked.is_empty() {
        toasts.push(Toast::warning("Bake Sheet: this sheet has no pieces"));
        return None;
    }
    let name = sim
        .world()
        .get::<Name>(sheet)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "sheet".to_string());
    // ⚠️ **Reusa o id da folha quando ela JÁ foi assada**, e só então gasta um novo. Assar duas
    // vezes com ids diferentes deixaria a primeira folha no `sheets` sem ninguém a nomear — lixo
    // que o save arrasta para sempre (e o `collect_sprite_pixels` só descarta o que *nenhum*
    // sprite usa, o que é tarde demais se um sprite antigo ainda a nomear).
    let sheet_id = existing_sheet_id(sim, sheet).unwrap_or_else(|| {
        let id = *next_sheet_id;
        *next_sheet_id = next_sheet_id.saturating_add(1);
        id
    });

    let at: Vec<[u32; 2]> = baked.iter().map(|b| b.at).collect();
    let entities: Vec<(Entity, String)> =
        baked.iter().map(|b| (b.entity, b.name.clone())).collect();
    let inputs: Vec<PackInput> = baked.into_iter().map(|b| b.input).collect();
    let authored = match ph2d_sprite_sheet::compose(sheet_id, name, size_px, inputs, &at) {
        Ok(s) => s,
        Err(e) => {
            toasts.push(Toast::error(format!("Bake Sheet: {e}")));
            return None;
        }
    };
    Some((authored, entities))
}

/// **Assa a folha**: compõe, sobe uma textura, e reata cada filho a uma região dela.
///
/// Devolve a folha assada. `None` quando o [`compose_sheet`] recusou (ele já toastou) ou quando a
/// GPU recusou a textura.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bake(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, ph2d_asset::AssetId>,
    sheets: &mut BTreeMap<u32, AuthoredSheet>,
    sheet_textures: &mut BTreeMap<u32, u32>,
    next_sheet_id: &mut u32,
    sheet_bits: u64,
    toasts: &mut ToastQueue,
) -> Option<AuthoredSheet> {
    let (authored, entities) = compose_sheet(
        sim,
        renderer,
        asset_db,
        atlas_asset_map,
        next_sheet_id,
        sheet_bits,
        toasts,
    )?;
    let sheet_id = authored.id;
    let texture_id =
        match renderer.acquire_individual(authored.width, authored.height, &authored.rgba) {
            Ok(id) => id,
            Err(e) => {
                toasts.push(Toast::error(format!("Bake Sheet: GPU upload: {e}")));
                return None;
            }
        };
    rebind(sim, &authored, &entities, texture_id, sheet_id);
    sheet_textures.insert(sheet_id, texture_id);
    sheets.insert(sheet_id, authored.clone());
    toasts.push(Toast::success(format!(
        "Sheet baked: {} pieces share one {}\u{00d7}{} texture",
        authored.regions.len(),
        authored.width,
        authored.height
    )));
    Some(authored)
}

/// O id da folha que estes filhos já nomeiam, se todos nomearem o mesmo.
fn existing_sheet_id(sim: &SimWorld, sheet: Entity) -> Option<u32> {
    let children = sim.world().get::<bevy_ecs::hierarchy::Children>(sheet)?;
    let mut found: Option<u32> = None;
    for c in children.iter() {
        let id = sim.world().get::<SpriteSheetRef>(*c)?.sheet;
        match found {
            None => found = Some(id),
            Some(f) if f == id => {}
            // Filhos de folhas diferentes: não há um id a reusar, e escolher um deles seria
            // reescrever a folha de outra pessoa.
            Some(_) => return None,
        }
    }
    found
}

/// Lê cada filho, redimensiona-o para o tamanho que ele ocupa na folha, e calcula o canto.
fn read_pieces(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, ph2d_asset::AssetId>,
    sheet: Entity,
    cfg: &SpriteSheetFrame,
    sheet_half: [f32; 2],
) -> Vec<Baked> {
    let children: Vec<Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(sheet)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    let mut out: Vec<Baked> = Vec::with_capacity(children.len());
    // ⚠️ **Os nomes das regiões têm de ser ÚNICOS.** O índice de região é a referência durável que
    // o `Sprite` guarda, e ele é a posição na lista **ordenada por nome** — dois filhos com o mesmo
    // nome dariam duas regiões indistinguíveis, e reatá-los pelo nome apontaria os dois para a
    // mesma. A hierarquia não impede nomes repetidos (o `unique_name` só age na criação), então a
    // unicidade garante-se aqui, onde ela é load-bearing.
    let mut seen: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for child in children {
        let Some(bx) = crate::sheet_bounds::piece_box_of(sim, child) else {
            continue;
        };
        let Some(src) =
            texture_edit::read_sprite_source(child, sim, renderer, asset_db, atlas_asset_map)
        else {
            continue;
        };
        // Alfa RETO: uma folha é um PNG, e o `bind_sheet_region` marca os sprites como retos. É a
        // mesma porta que toda ferramenta de imagem usa antes de reamostrar.
        let straight = src.image.into_straight();
        let (sw, sh) = (straight.width, straight.height);
        // Quantos pixels da folha esta peça ocupa — a caixa dela, à densidade da folha.
        let want_w = (cfg.pixels_for(bx.half[0] * 2.0)).max(1);
        let want_h = (cfg.pixels_for(bx.half[1] * 2.0)).max(1);
        let (sx, sy) = (
            want_w as f32 / sw.max(1) as f32,
            want_h as f32 / sh.max(1) as f32,
        );
        // ⚠️ O MESMO reamostrador da ferramenta Rasterize (Mitchell-Netravali) — um só no projeto.
        // A rotação **não** entra aqui: ela já está na caixa (`piece_box`), e passá-la de novo
        // rodaria os pixels uma segunda vez.
        let r = ph2d_tool_rasterize::rasterize(&straight.pixels, sw, sh, sx, sy, 0.0);
        let [x, y] = corner_px(bx.center, bx.half, sheet_half, cfg.pixels_per_meter);
        // Negativo não acontece com a folha sã (a `health` já recusou), mas um `as u32` sobre um
        // negativo daria a volta em silêncio — este `max(0)` é o cinto do suspensório.
        let at = [x.max(0) as u32, y.max(0) as u32];
        let base = sim
            .world()
            .get::<Name>(child)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("piece_{}", child.to_bits()));
        let name = match seen.get_mut(&base) {
            Some(n) => {
                *n += 1;
                format!("{base}_{n}")
            }
            None => {
                seen.insert(base.clone(), 0);
                base
            }
        };
        out.push(Baked {
            entity: child,
            name: name.clone(),
            input: PackInput {
                name,
                width: r.width,
                height: r.height,
                rgba: r.pixels,
            },
            at,
        });
    }
    out
}

/// Reata cada filho a' sua região da folha assada.
fn rebind(
    sim: &mut SimWorld,
    authored: &AuthoredSheet,
    entities: &[(Entity, String)],
    texture_id: u32,
    sheet_id: u32,
) {
    for (entity, name) in entities {
        // ⚠️ O índice vem da lista ORDENADA POR NOME, não da ordem em que se leu os filhos: é essa
        // ordenação que faz o índice sobreviver a um save/load (o construtor de `AuthoredSheet`
        // ordena de propósito, e o import faz o mesmo).
        let Some(region) = authored.regions.iter().position(|r| &r.name == name) else {
            continue;
        };
        let rect = authored.regions[region].rect;
        if let Ok(mut e) = sim.world_mut().get_entity_mut(*entity) {
            if let Some(mut sprite) = e.get_mut::<Sprite>() {
                crate::project_sprite_pixels::bind_sheet_region(&mut sprite, texture_id, rect);
            }
            e.insert(SpriteSheetRef {
                sheet: sheet_id,
                region: region as u32,
            });
            // ⚠️ **Os pixels PRÓPRIOS deixam de ser a verdade desta peça**, e o componente que os
            // nomeia tem de sair com eles. Deixá-lo faria o save gravar duas vezes a mesma arte —
            // uma na folha e outra por-peça —, e o `should_collect` não o apanharia (ele pergunta
            // pela origem, que agora é `Individual` a apontar para a textura DA FOLHA). O undo
            // devolve-o, porque a captura é por diff do mundo.
            e.remove::<ph2d_ecs::SpritePixels>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Uma peça que já tem o tamanho certo NÃO é reamostrada.**
    ///
    /// ⚠️ Nasceu como sonda, a perseguir o relato *"o bake às vezes piora a borda transparente e
    /// às vezes não afeta"*: a hipótese era que o arredondamento de `pixels_for` deslocasse o alvo
    /// em um pixel e disparasse um passe de Mitchell — que tem lobos negativos e deixa halo numa
    /// borda de alfa — por nada. **A medição REFUTOU a hipótese** (8 densidades × 22 tamanhos, zero
    /// casos), e a causa verdadeira estava noutro sítio: o recorte de região que faltava no
    /// `texture_edit`.
    ///
    /// O gate fica. Refutada ou não, a invariante é a que se quer — reamostrar de graça custa
    /// nitidez —, e no dia em que o `pixels_for` mudar de arredondamento ele di-lo antes de o
    /// artista o ver na borda. *Uma sonda que confirma uma invariante vale mais viva do que
    /// apagada.*
    #[test]
    fn a_piece_that_already_fits_is_not_resampled() {
        let mut bad = Vec::new();
        for ppm in [16.0f32, 32.0, 50.0, 100.0, 96.0, 256.0, 1024.0, 37.5] {
            let cfg = ph2d_ecs::SpriteSheetFrame::at_density(ppm);
            for src in [
                7u32, 15, 16, 17, 31, 32, 33, 48, 64, 96, 100, 127, 128, 129, 160, 200, 255, 256,
                333, 512, 1000, 1024,
            ] {
                // O que o import escreve: `sprite.size = src / ppm` em f32. Se o `pixels_for` o
                // devolver ao inteiro original, `sx == 1.0` exatamente e o resampler sai cedo.
                let size_m = src as f32 / ppm;
                let want = cfg.pixels_for(size_m);
                if want != src {
                    bad.push(format!("ppm={ppm} src={src} -> want={want}"));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "{} casos em que o bake reamostraria sem precisar:\n{}",
            bad.len(),
            bad.join("\n")
        );
    }

    /// ⚠️ **A inversa exata do `place`.** Uma peça que o auto-arranjo pôs no canto `(0,0)` da folha
    /// tem o centro em `(-half_folha + half_peça, +half_folha - half_peça)`; assá-la tem de
    /// devolver `(0,0)` — não `(0,1)`.
    #[test]
    fn a_piece_in_the_top_left_bakes_at_the_origin() {
        // Folha de 8 m (800 px a 100 px/m), peça de 2 m (200 px).
        let sheet_half = [4.0, 4.0];
        let half = [1.0, 1.0];
        let center = [-4.0 + 1.0, 4.0 - 1.0];
        assert_eq!(corner_px(center, half, sheet_half, 100.0), [0, 0]);
    }

    /// O eixo Y INVERTE: descer no mundo é aumentar a linha da folha.
    #[test]
    fn going_down_in_the_world_increases_the_row() {
        let sheet_half = [4.0, 4.0];
        let half = [1.0, 1.0];
        let top = corner_px([-3.0, 3.0], half, sheet_half, 100.0);
        let lower = corner_px([-3.0, 1.0], half, sheet_half, 100.0);
        assert_eq!(top, [0, 0]);
        assert_eq!(lower, [0, 200], "2 m abaixo = 200 px mais abaixo");
    }

    /// A densidade escala tudo — a mesma pose a 200 px/m dá o dobro dos pixels.
    #[test]
    fn density_scales_the_corner() {
        let sheet_half = [4.0, 4.0];
        assert_eq!(
            corner_px([-3.0, 3.0], [1.0, 1.0], sheet_half, 200.0),
            [0, 0]
        );
        assert_eq!(
            corner_px([0.0, 0.0], [1.0, 1.0], sheet_half, 200.0),
            [600, 600]
        );
    }

    /// ⚠️ Uma peça arrastada para fora dá canto NEGATIVO, e o tipo devolvido tem de o conseguir
    /// dizer: em `u32` daria a volta para perto de 4 mil milhões, e o retângulo absurdo passaria
    /// pelo teste de «cabe» de quem o recebesse.
    #[test]
    fn a_piece_pushed_out_reports_a_negative_corner() {
        let out = corner_px([-9.0, 0.0], [1.0, 1.0], [4.0, 4.0], 100.0);
        assert!(out[0] < 0, "canto = {out:?}");
    }
}
