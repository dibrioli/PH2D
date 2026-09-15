//! ⭐⭐⭐ **UM CANVAS DO PAINTER PRESO A OSSOS, E DOBRADO** — `PH2D_VEC_BONE_PAINT_SMOKE=1`.
//!
//! # Por que esta cena existe
//!
//! ⛔⛔ **A cura das guias chatas (item 4 do dono, 2026-09-15) não tinha CENA.** A grelha, os selos de
//! operação e a caixa do gizmo passaram a seguir a arte dobrada, está tudo gateado — e **nenhuma cena
//! deste app punha o Painter a pintar por cima de arte dobrada por ossos**, logo o dono não tinha como
//! olhar para a correcção. *Uma cura que ninguém pode ver é uma cura que ninguém julga.*
//!
//! ⚠️ **Ela é da família do VECTOR e não da do Painter**, e não é arrumação: a `ph2d-app-painter` não
//! conhece o esqueleto — nem deve. Quem tem as duas metades é esta família, que já possui a cena dos
//! ossos ([`crate::smoke_bone`]).
//!
//! # O que ela monta, e por que cada peça está lá
//!
//! | peça | o que ela serve |
//! |---|---|
//! | um **canvas branco opaco** de `1024²` | é o sujeito do Painter, e o branco é o fundo em que as guias se leem |
//! | **três ossos** ao meio dele, em corrente | três porque com dois a dobra tem um vinco só, e o que o olho julga é a CURVA |
//! | o canvas **PRESO** a eles | é isto que faz a sprite ser desenhada como MALHA, e sem malha não há o que corrigir |
//! | os dois últimos ossos **rodados** | ⚠️ a dobra é feita DEPOIS de prender: o repouso é o instante do bind, e dobrar antes não dobraria nada |
//!
//! ⛔ **Nada mais é armado** — nem a ferramenta, nem o pincel, nem a grelha. É a lei que a cena da
//! máscara já escreveu: *uma cena que arma estado por baixo da mesa salta exactamente a costura que
//! ela devia provar, e esconde um default mau.*
//!
//! ⚠️ **Se a linha `[bone-paint-smoke]` não aparecer, PARE:** a cena não montou.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// **O maior nível a que este roteador de facto responde.**
///
/// ⚠️⚠️ **CONTADO no roteador, nunca escrito de memória** (CLAUDE.md §5.0) — e este é de
/// **PRESENÇA**, como o da máscara: ele lê `var_os(..).is_some()`, logo não há `match` de níveis e
/// o maior com significado é `1`. ⛔ Declarar mais seria prometer uma cena que ninguém escreveu.
pub const NIVEIS: u32 = 1;

/// Ver o cabeçalho do módulo.
#[must_use]
pub fn armed() -> bool {
    std::env::var_os("PH2D_VEC_BONE_PAINT_SMOKE").is_some()
}

/// O lado do canvas, em pixels — e o número é MEDIDO contra o orçamento de peças do quadro.
///
/// ⚠️⚠️ **Um canvas OPACO é coberto de malha de ponta a ponta**, então o tamanho dele É a densidade
/// da pele. Medido nesta cena com os defaults do produto intactos (`PH2D_BONE_LOG=1`):
///
/// | lado | peças guardadas | orçamento do quadro |
/// |---:|---:|---:|
/// | 1024 | **3 690** | 1 543 ⇒ **acima** |
/// | 640 | **1 620** | 1 543 ⇒ **acima** |
/// | **512** | **1 144** | 1 543 ⇒ dentro |
///
/// ⛔ **A alavanca é o TAMANHO e não a grelha**, de propósito: baixar `GridOptions` seria armar a
/// cena com números que o produto não usa, e a lei da cena da máscara é explícita — *uma cena que
/// arma estado por baixo da mesa esconde um default mau*. ⚠️ E uma cena acima do orçamento ensinaria
/// ao dono que o app engasga, sobre uma fixtura que eu escolhi.
const LADO_PX: u32 = 512;

/// Quanto cada junta dobra, em graus.
///
/// ⚠️ **Escolhido para a dobra ser VISÍVEL sem inverter a malha**: a `25°` por junta a ponta faz
/// `50°` com a raiz, que é bem mais do que a barra de meio pixel que a subdivisão persegue e bem
/// menos do que o ângulo em que o mapa começa a dobrar sobre si mesmo (a régua da dobra vive na
/// `ph2d_skeleton::fold`).
const DOBRA_GRAUS: f32 = 25.0;

/// Monta a cena. Devolve os bits do canvas, para o chamador assentar a selecção nele.
///
/// ⚠️ **UM tempo só, ao contrário da [`crate::smoke_bone`]**: aquela prende FORMAS vectoriais e
/// precisa da entidade que o `vec_entities::sync` cria no meio do quadro. Aqui o sujeito é uma
/// IMAGEM, e ela já existe no instante em que nasce.
pub fn build(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    let (label, bits) = match ph2d_image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LADO_PX,
        2, // branco opaco — é sobre ele que as guias se leem, e é ele que dá tinta à malha
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[bone-paint-smoke] o canvas nao subiu: {e}");
            return None;
        }
    };
    let e = ph2d_ecs::Entity::try_from_bits(bits)?;

    // ── Os três ossos, ao meio do canvas ────────────────────────────────────────────────────────
    let largura = f64::from(LADO_PX) / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let (x0, passo) = (-largura / 2.0, largura / 3.0);
    let mut pai: Option<ph2d_ecs::Entity> = None;
    let mut ossos = Vec::new();
    for k in 0..3 {
        let (a, b) = (
            [x0 + passo * f64::from(k), 0.0],
            [x0 + passo * f64::from(k + 1), 0.0],
        );
        let Some(osso) = ph2d_skeleton_live::bone::create(sim, pai, a, b) else {
            eprintln!("[bone-paint-smoke] o osso {k} nao nasceu -- PARE");
            return Some(bits);
        };
        let ent = ph2d_ecs::Entity::try_from_bits(osso)?;
        ossos.push(ent);
        pai = Some(ent);
    }

    // ── Prender o canvas a eles ─────────────────────────────────────────────────────────────────
    //
    // ⚠️⚠️ **Os pixels vêm do MAPA DA CÉLULA, não do componente** — e isto custou uma corrida. Uma
    // sprite que vive numa célula do **atlas partilhado** não carrega `SpritePixels`: esse carimbo é
    // do caminho `Individual` (uma textura própria), e é por lá que a cena dos ossos lê a imagem
    // dela. A `spawn_rgba` põe os bytes no `AssetDb` e o vínculo `célula → AssetId` no
    // `atlas_asset_map` — é ali que eles estão. *Copiar a leitura da cena irmã lia o componente
    // errado e devolvia `None` em silêncio.*
    let arte = atlas_asset_map
        .get(&cell_idx)
        .and_then(|id| asset_db.get(id));
    let preso = arte
        .as_ref()
        .and_then(|a| a.image_rgba8())
        .is_some_and(|(w, h, cow)| {
            ph2d_skeleton_live::skin_live::bind_image(
                sim,
                e,
                &cow,
                [w, h],
                pixels_per_meter,
                ph2d_poly2d::GridOptions::default(),
                ossos.first().copied(),
            )
        });
    if !preso {
        eprintln!(
            "[bone-paint-smoke] o canvas NAO prendeu ao esqueleto -- PARE, a cena nao montou"
        );
        return Some(bits);
    }

    // ── E só AGORA a dobra ──────────────────────────────────────────────────────────────────────
    //
    // ⚠️⚠️ **Depois de prender, nunca antes.** O repouso de uma pele é o instante do bind: dobrada
    // antes, esta pose SERIA o repouso e o canvas sairia recto — a cena montaria e não provaria nada.
    for osso in ossos.iter().skip(1) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(*osso) {
            t.rotation += DOBRA_GRAUS.to_radians();
        }
    }

    println!(
        "[bone-paint-smoke] canvas '{label}' ({LADO_PX}x{LADO_PX}, branco) PRESO a 3 ossos e dobrado \
         {DOBRA_GRAUS}° por junta. NADA mais esta' armado.\n\
         [bone-paint-smoke] 1) pegue a ferramenta Painter  2) escolha uma forma (Rectangle/Ellipse) e \
         arraste uma sobre o canvas: o CONTORNO e a CAIXA dela tem de seguir a curva, e a caixa tem de \
         FECHAR pela curva (os quatro lados, nao tres)  3) ligue a grelha (Grid): as linhas dela tem de \
         acompanhar a dobra em vez de a atravessarem a direito  4) o que estiver desenhado tem de poder \
         ser AGARRADO onde ele aparece, nao na reta entre as pontas."
    );
    Some(bits)
}

#[cfg(test)]
mod tests {
    /// ⭐⭐⭐ **A CENA PRENDE ANTES DE DOBRAR, E A ORDEM É LOAD-BEARING.**
    ///
    /// ⛔⛔ O repouso de uma pele é **o instante do bind**. Dobrada primeiro, esta pose SERIA o
    /// repouso e o canvas sairia RECTO — a cena montaria, imprimiria a linha de sucesso, e não
    /// provaria nada. *Um smoke que monta e não demonstra é pior que um ausente: ele é acreditado.*
    ///
    /// ⚠️ A régua é a POSIÇÃO no ficheiro, que é o que uma leitura rápida do diff inverte sem dar
    /// por isso — mover o laço da rotação para cima de `bind_image` é uma edição de duas linhas.
    #[test]
    fn the_scene_binds_before_it_bends() {
        let fonte = include_str!("smoke_bone_paint.rs");
        let bind = fonte.find("bind_image(").expect("a cena prende a imagem");
        let dobra = fonte
            .find("t.rotation += DOBRA_GRAUS")
            .expect("a cena dobra os ossos");
        assert!(
            bind < dobra,
            "a cena dobra ANTES de prender: o repouso passa a ser a pose dobrada e o canvas sai \
             recto — ela montaria e nao provaria nada"
        );
    }

    /// ⭐ **O nível declarado é o que existe** — ver a nota do [`super::NIVEIS`].
    #[test]
    fn the_router_declares_only_the_scene_that_exists() {
        assert_eq!(super::NIVEIS, 1);
    }
}
