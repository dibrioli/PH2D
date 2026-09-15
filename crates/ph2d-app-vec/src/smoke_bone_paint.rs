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
//! # ⛔⛔ A 1.ª REDACÇÃO DESTA CENA FOI REPROVADA PELO DONO («ruim», 2026-09-15), E A CAUSA É ESTA
//!
//! Ela montava um canvas **QUADRADO** de `512²` com os três ossos deitados ao meio dele, e a arte
//! saía **RASGADA**: lascas brancas e pedaços do traço pintado deslocados do resto.
//!
//! ⚠️⚠️ **Não era dobra a mais — era arte FORA DO ALCANCE.** O raio de um osso é o **comprimento
//! dele** vezes a `strength` ([`ph2d_skeleton::SkinBone::new`]), e um ponto fora do raio de TODO osso
//! é **órfão**: ele salta, em salto seco, para o osso mais próximo ([`ph2d_skeleton::Skin::weights_at`],
//! o *point binding* do Moho). Um salto seco num mapa contínuo é **um rasgo**, e é exactamente o que a
//! foto mostrava.
//!
//! ⛔⛔⛔ **A ARITMÉTICA PROÍBE O QUADRADO, e é por isso que afinar o ângulo nunca ia curar:** com `n`
//! ossos deitados ao longo da largura `W`, cada osso mede `W/n` e portanto **alcança `W/n`**; a arte
//! sobe `W/2` acima do eixo. `W/2 < W/n` só é verdade com `n < 2`. ⇒ *um quadrado com uma corrente de
//! três ossos tem uma banda órfã por construção, em qualquer ângulo e em qualquer tamanho.*
//!
//! ⚠️⚠️ **E eu citei a RÉGUA ERRADA ao escolher os `25°`.** O doc dizia que o ângulo estava «bem abaixo
//! do ponto em que o mapa dobra sobre si mesmo», e isso era **verdade e irrelevante**: a
//! [`ph2d_skeleton::fold`] mede **INVERSÃO**, e a coluna `inverted` lia `0,00 %` sobre a foto do rasgo.
//! Quem gritava era a coluna `orphan`, a **`33,85 %`** — a coluna que o doc daquela régua chama de
//! *anti-vacuidade* e que eu não olhei. *Uma régua com duas colunas responde a duas perguntas, e ler a
//! errada é ler zero sobre o defeito que está no ecrã.*
//!
//! ## A medição que decidiu a forma (3 ossos, `strength` no default do produto, dobra `25°`)
//!
//! | canvas | meia-altura | alcance do osso | `det_min` | invertida | **ÓRFÃ** |
//! |---|---:|---:|---:|---:|---:|
//! | `512×512` (a 1.ª redacção) | 256 | 170,67 | 0,2622 | 0,00 % | **33,85 %** |
//! | `512×384` | 192 | 170,67 | 0,2573 | 0,00 % | **12,31 %** |
//! | `512×336` | 168 | 170,67 | 0,2114 | 0,00 % | 0,00 % |
//! | **`512×320`** (esta cena) | **160** | **170,67** | **0,2497** | **0,00 %** | **0,00 %** |
//! | `512×288` | 144 | 170,67 | 0,2510 | 0,00 % | 0,00 % |
//! | `512×352` | 176 | 170,67 | 0,1241 | 0,00 % | 4,12 % |
//!
//! ⛔⛔ **E a cura «óbvia» — alargar o alcance — foi MEDIDA e é PIOR pelo meio:** com o quadrado de
//! `512²`, a `strength 1.0` dá `det_min 0,2622` com `33,85 %` de órfãs, a `1.5` dá **`det_min −1,2829`
//! com `0,32 %` da arte DO AVESSO**, e só a `2.0` volta a limpar (`0,4750`). *Um alcance que cobre
//! metade da banda órfã mistura um osso que chega com um vizinho que salta, e a mistura inverte-se —
//! chegar a meio é pior que não chegar.* ⇒ a alavanca desta cena é a **FORMA da arte**, não um knob.
//!
//! # O que ela monta, e por que cada peça está lá
//!
//! | peça | o que ela serve |
//! |---|---|
//! | um **canvas branco opaco**, largo e baixo | é o sujeito do Painter; o branco é o fundo em que as guias se leem, e é ele que dá tinta à malha |
//! | **três ossos** deitados ao longo dele | três porque com dois a dobra tem um vinco só, e o que o olho julga é a CURVA |
//! | a **altura DERIVADA** do alcance de um osso | ver [`ALTURA_PX`] — é isto que impede a banda órfã, e não um número escolhido |
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
use ph2d_ecs::{Entity, SimWorld, Transform};
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

/// A largura do canvas, em pixels — e o número é MEDIDO contra o orçamento de peças do quadro.
///
/// ⚠️⚠️ **Um canvas OPACO é coberto de malha de ponta a ponta**, então a ÁREA dele é a densidade da
/// pele. Medido nesta cena com os defaults do produto intactos (`PH2D_BONE_LOG=1`), a tabela está no
/// §9 do handoff; o orçamento de um quadro é `1 543` peças.
///
/// ⛔ **A alavanca é o TAMANHO e não a grelha**, de propósito: baixar `GridOptions` seria armar a
/// cena com números que o produto não usa, e a lei da cena da máscara é explícita — *uma cena que
/// arma estado por baixo da mesa esconde um default mau*. ⚠️ E uma cena acima do orçamento ensinaria
/// ao dono que o app engasga, sobre uma fixtura que eu escolhi.
const LARGURA_PX: u32 = 512;

/// Quantos ossos tem a corrente.
///
/// ⚠️ **Três, e não dois:** com dois há UM vinco, e o que o olho julga (e o que as guias têm de
/// seguir) é uma **curva**. ⚠️ Ele entra na conta da [`ALTURA_PX`] porque é ele que fixa o
/// comprimento — e portanto o ALCANCE — de cada osso.
const OSSOS: u32 = 3;

/// A altura do canvas, em pixels — uma TIRA, porque é numa tira que uma curva se lê.
///
/// ⚠️ `5/8` da largura é uma escolha de FORMA (a arte tem de ser pintável e a dobra tem de se ver);
/// o que **não** é escolha é o alcance dos ossos que a carregam — ver [`forca_do_osso`].
const ALTURA_PX: u32 = LARGURA_PX * 5 / 8;

/// ⭐⭐⭐ **QUE FRACÇÃO DO ALCANCE DE UM OSSO A ARTE USA — e o número é MEDIDO, não escolhido.**
///
/// ⛔⛔⛔ **HÁ DOIS PRECIPÍCIOS, e a 2.ª redacção desta cena caiu no segundo** (report do dono,
/// 2026-09-15: *«a malha deforma a curva»*, com a seta na borda de cima).
///
/// O primeiro é o **ÓRFÃO**: fora do raio de todo osso a pele salta em salto seco para o mais
/// próximo, e a arte RASGA. A 1.ª redacção caiu nesse, e a cura foi pôr a arte dentro do alcance.
///
/// ⚠️⚠️ **O segundo mora LOGO DENTRO do primeiro, e nenhuma régua deste repo o via:** o peso de um
/// osso é o *bump* `(1 − x²)²` com `x = d/raio`, e os pesos são **NORMALIZADOS**. Junto da borda do
/// suporte todos os pesos tendem a zero, e a razão entre dois números que tendem a zero varia
/// depressa ⇒ **a curvatura do campo explode**. O mapa continua contínuo e injectivo (`0,00 %` de
/// órfãs, `0,00 %` do avesso) e mesmo assim a malha, que pinta **um afim por triângulo**, deixa de
/// o conseguir seguir: a arte sai FACETADA e uma circunferência desenhada por cima sai com quinas.
///
/// ## A medição (512×320, dobra `25°`, a grelha de fábrica, `780` triângulos nas cinco linhas)
///
/// | `strength` | meia-altura / raio | órfã | **desvio da faceta** | `det_min` |
/// |---:|---:|---:|---:|---:|
/// | `1,0` — a redacção reprovada | `0,938` | `0,00 %` | **`14,24 px`** | `0,2518` |
/// | `1,5` | `0,625` | `0,00 %` | `1,89 px` | `0,5846` |
/// | **`2,0`** — esta cena | **`0,469`** | `0,00 %` | **`0,92 px`** | `0,7475` |
/// | `3,0` | `0,312` | `0,00 %` | `0,59 px` | `0,8279` |
///
/// ⭐ **`15×` melhor sem um triângulo a mais** — a mesma arte, a mesma dobra, a mesma malha.
///
/// ⚠️ **O `0,47` é o joelho:** abaixo dele a curva achata (`0,92 → 0,59 px` custa `1,5×` de alcance)
/// e acima dele ela dispara (`0,625` já dá o dobro). ⛔ E não é um tecto de recurso: é o sítio onde
/// a arte deixa de viver na borda do suporte dos pesos.
const FRACCAO_DO_ALCANCE: f64 = 0.47;

/// ⭐⭐⭐ **A FORÇA DE CADA OSSO, DERIVADA DA ARTE** — nunca um literal ao lado dos outros.
///
/// O raio de um osso é `comprimento × strength` ([`ph2d_skeleton::SkinBone::new`]) e o comprimento
/// aqui é `LARGURA_PX / OSSOS`. Queremos que a meia-altura da arte seja a [`FRACCAO_DO_ALCANCE`]
/// desse raio ⇒ `strength = (ALTURA_PX / 2) / (LARGURA_PX / OSSOS) / FRACCAO`.
///
/// ⚠️ **É a derivação que corre no sentido certo:** a arte é o sujeito e o rig serve-a. Escrever
/// `2.0` ao lado de `512` e `320` seriam três números independentes, e mexer em qualquer um levaria
/// a arte de volta à borda do alcance **em silêncio** — que é exactamente como esta cena foi
/// reprovada duas vezes.
///
/// ⛔ **Isto NÃO é armar a cena por baixo da mesa.** A `strength` é uma propriedade AUTORADA do
/// osso, no painel do esqueleto, e o que ela significa é *«até onde este osso alcança»*: um rig cuja
/// arte vive a `94 %` do alcance é um rig mal autorado, e é isso que a 2.ª redacção tinha. ⚠️ Que o
/// app não guie o artista para longe da borda é um item ABERTO, com o número ao lado (o handoff §13).
fn forca_do_osso() -> f64 {
    let raio_base = f64::from(LARGURA_PX) / f64::from(OSSOS);
    (f64::from(ALTURA_PX) / 2.0) / raio_base / FRACCAO_DO_ALCANCE
}

/// Quanto cada junta dobra, em graus.
///
/// ⚠️ **Escolhido para a dobra ser VISÍVEL sem maltratar a arte**: medido nesta geometria, `25°` por
/// junta dá `0,00 %` órfã, `0,00 %` do avesso, `det_min 0,7475` e `0,92 px` de faceta.
/// ⛔⛔ **São TRÊS réguas e não uma**, e cada redacção desta cena caiu na que eu não tinha corrido:
/// a inversão (que eu citei), a **órfã** (que reprovou a 1.ª) e a **faceta** (que reprovou a 2.ª).
const DOBRA_GRAUS: f32 = 25.0;

/// **Os eixos dos ossos, em metros de mundo** — a corrente deitada ao longo da largura do canvas.
///
/// ⚠️ **Uma porta só, porque o gate monta a MESMA corrente.** O gate não pode chamar a [`build`]
/// (ela precisa do renderer e do atlas), então o que ele reusa é isto: escrever a disposição duas
/// vezes faria o gate medir uma cena que o dono não vê. *Duas cópias de uma geometria convergem
/// enquanto ninguém mexe numa delas.*
/// ⭐⭐⭐ **A CORRENTE, criada e com o ALCANCE já escrito** — a porta ÚNICA que a monta.
///
/// ⛔⛔ **Ela existe por uma mutação que SOBREVIVEU** (2026-09-15): os gates montavam a corrente
/// eles próprios e escreviam a força eles próprios, logo mediam uma **reconstrução** da cena. Pôr
/// `strength = 1.0` na [`build`] deixava os seis verdes — *a lei escrita em dois sítios prova-se num
/// sítio e ship-a no outro*. Com esta porta há UM lugar onde a força é escrita, e os gates passam
/// por ele.
///
/// `None` quando um osso não nasce — e aí quem chama PARA, porque uma cena com meia corrente monta
/// e não demonstra nada.
fn corrente(sim: &mut SimWorld, pixels_per_meter: f32) -> Option<Vec<Entity>> {
    let mut pai: Option<Entity> = None;
    let mut ossos = Vec::new();
    for (k, (a, b)) in eixos(pixels_per_meter).into_iter().enumerate() {
        let Some(osso) = ph2d_skeleton_live::bone::create(sim, pai, a, b) else {
            eprintln!("[bone-paint-smoke] o osso {k} nao nasceu -- PARE");
            return None;
        };
        let ent = Entity::try_from_bits(osso)?;
        // ⭐ O ALCANCE, derivado da arte — ver [`forca_do_osso`]. Sem isto a arte vive na borda do
        // suporte dos pesos e sai FACETADA (`14,24 px` contra `0,92`), que foi o 2.º report do dono.
        if let Some(mut bone) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ent) {
            bone.strength = forca_do_osso();
        }
        ossos.push(ent);
        pai = Some(ent);
    }
    Some(ossos)
}

fn eixos(pixels_per_meter: f32) -> Vec<([f64; 2], [f64; 2])> {
    let largura = f64::from(LARGURA_PX) / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let (x0, passo) = (-largura / 2.0, largura / f64::from(OSSOS));
    (0..OSSOS)
        .map(|k| {
            (
                [x0 + passo * f64::from(k), 0.0],
                [x0 + passo * f64::from(k + 1), 0.0],
            )
        })
        .collect()
}

/// **Dobra a corrente** — todas as juntas menos a raiz. Ver a nota de ordem na [`build`].
fn dobra(sim: &mut SimWorld, ossos: &[Entity]) {
    for osso in ossos.iter().skip(1) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(*osso) {
            t.rotation += DOBRA_GRAUS.to_radians();
        }
    }
}

/// **A tinta do canvas: branco OPACO.**
///
/// ⚠️ Opaco de propósito, e em duas contas: é o fundo em que as guias se leem, e **a malha é traçada
/// da tinta** — um canvas transparente não teria malha nenhuma, logo não haveria deformação a julgar.
fn branco(w: u32, h: u32) -> Vec<u8> {
    vec![255u8; (w as usize) * (h as usize) * 4]
}

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
    // ⚠️ **A porta RETANGULAR, e não a do *New Image…***: a `spawn_blank_canvas` é quadrada por
    // desenho (é o caminho daquele modal), e um canvas quadrado é exactamente o que esta cena não
    // pode ter. O doc da `spawn_rgba` diz que ela foi extraída para isto — *«uma tira RETANGULAR
    // com conteúdo»*.
    let (label, bits) = match ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LARGURA_PX,
        ALTURA_PX,
        branco(LARGURA_PX, ALTURA_PX),
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
        "Canvas",
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[bone-paint-smoke] o canvas nao subiu: {e}");
            return None;
        }
    };
    let e = Entity::try_from_bits(bits)?;

    // ── Os três ossos, deitados ao longo do canvas ──────────────────────────────────────────────
    let Some(ossos) = corrente(sim, pixels_per_meter) else {
        return Some(bits);
    };

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
    dobra(sim, &ossos);

    println!(
        "[bone-paint-smoke] canvas '{label}' ({LARGURA_PX}x{ALTURA_PX}, branco) PRESO a {OSSOS} ossos \
         e dobrado {DOBRA_GRAUS}° por junta. NADA mais esta' armado.\n\
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
    use ph2d_ecs::{Entity, SimWorld, Transform};
    use ph2d_render::Sprite;

    /// O `pixels_per_meter` de omissão do projecto — o mesmo que a cena recebe.
    const PPM: f32 = 100.0;

    /// A escala da câmera do smoke, em pixels de ECRÃ por metro de mundo — medida no ecrã em
    /// 2026-09-15. É ela que converte as réguas de metros para o que o olho vê.
    const PX_POR_METRO: f64 = 152.0;

    /// **A CENA MONTADA, pelas portas do PRODUTO** — devolve `(mundo, entidade da arte)`.
    ///
    /// ⚠️ Ela monta a corrente com a [`super::eixos`] (a mesma que a [`super::build`] usa), escreve
    /// a força pela [`super::forca_do_osso`], prende pelo `bind_image` e dobra pela [`super::dobra`]
    /// — ⛔ **nenhuma cinemática escrita aqui**. Uma segunda cadeia de transformações neste ficheiro
    /// seria uma segunda resposta à mesma pergunta, e mediria uma cena que o dono não vê.
    ///
    /// `altura_px` e `forca` são parâmetros para os gates de CONTROLO poderem pedir exactamente as
    /// duas redacções que o dono reprovou.
    fn cena(altura_px: u32, forca: Option<f64>) -> (SimWorld, Entity) {
        let mut sim = SimWorld::default();
        // ⭐⭐ **A PORTA DO PRODUTO**, e é ela que escreve o alcance: um `None` aqui mede a cena tal
        // como o dono a vê. Ver o doc da [`super::corrente`] — a mutação que a criou.
        let ossos = super::corrente(&mut sim, PPM).expect("a corrente monta");
        if let Some(f) = forca {
            for &ent in &ossos {
                if let Some(mut bone) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ent) {
                    bone.strength = f;
                }
            }
        }
        // A sprite que a `spawn_rgba` cria: o tamanho em metros é `pixels / ppm`, âncora ao centro.
        let tamanho = [super::LARGURA_PX as f32 / PPM, altura_px as f32 / PPM];
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Sprite::atlas(0, tamanho, [1.0, 1.0, 1.0, 1.0]),
            ))
            .id();
        assert!(
            ph2d_skeleton_live::skin_live::bind_image(
                &mut sim,
                e,
                &super::branco(super::LARGURA_PX, altura_px),
                [super::LARGURA_PX, altura_px],
                PPM,
                ph2d_poly2d::GridOptions::default(),
                ossos.first().copied(),
            ),
            "o bind tinha de acontecer: ha' osso, ha' tinta e a pose nao e' singular"
        );
        super::dobra(&mut sim, &ossos);
        (sim, e)
    }

    /// A dobra da pele — as quatro colunas da [`ph2d_skeleton::fold`].
    fn dobra_da_cena(altura_px: u32, forca: Option<f64>) -> ph2d_skeleton::fold::FoldReport {
        let (sim, e) = cena(altura_px, forca);
        let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("a pele resolve");
        let (mx, my) = (
            f64::from(super::LARGURA_PX) / f64::from(PPM) / 2.0,
            f64::from(altura_px) / f64::from(PPM) / 2.0,
        );
        ph2d_skeleton::fold::measure(&pele, [-mx, -my, mx, my], 65)
    }

    /// ⭐⭐⭐ **O DESVIO DA FACETA, em pixels de ECRÃ** — quanto o afim de cada triângulo erra o campo.
    ///
    /// ⚠️ É a régua do PRODUTO (`ph2d_poly2d::deviation`, a mesma que o `Smooth` consulta), sobre a
    /// malha que a cena de facto guarda e o campo que ela de facto aplica.
    fn faceta_da_cena(altura_px: u32, forca: Option<f64>) -> f64 {
        let (sim, e) = cena(altura_px, forca);
        let malha = ph2d_skeleton_live::skin_image::mesh_of(&sim, e).expect("malha");
        let (p2l, pele) =
            ph2d_skeleton_live::skin_image::deform_field(&sim, e, malha.size, PPM).expect("campo");
        let mut w = pele.scratch();
        let posadas: Vec<[f64; 2]> = malha
            .rest
            .iter()
            .map(|&q| pele.point(p2l.apply(q), &mut w))
            .collect();
        let d = ph2d_poly2d::deviation(&malha, &posadas, &mut |q| {
            let mut w2 = pele.scratch();
            pele.point(p2l.apply(q), &mut w2)
        });
        d * PX_POR_METRO
    }

    /// ⭐ **QUANTO UMA CIRCUNFERÊNCIA DESENHADA AO CENTRO SAI FORA DE REDONDO** — `r_max / r_min`.
    ///
    /// ⚠️ É o que o dono de facto olha: ele arrasta uma *Ellipse* e julga a forma dela. O raio é
    /// `40 %` da altura, que é o tamanho da circunferência da foto de 2026-09-15.
    fn circulo_fora_de_redondo(altura_px: u32, forca: Option<f64>) -> f64 {
        let (sim, e) = cena(altura_px, forca);
        let malha = ph2d_skeleton_live::skin_image::mesh_of(&sim, e).expect("malha");
        let (p2l, pele) =
            ph2d_skeleton_live::skin_image::deform_field(&sim, e, malha.size, PPM).expect("campo");
        let mut w = pele.scratch();
        let centro = [
            f64::from(super::LARGURA_PX) / 2.0,
            f64::from(altura_px) / 2.0,
        ];
        let r = f64::from(altura_px) * 0.4;
        let n = 720;
        let pts: Vec<[f64; 2]> = (0..n)
            .map(|i| {
                let t = std::f64::consts::TAU * f64::from(i) / f64::from(n);
                pele.point(
                    p2l.apply([centro[0] + r * t.cos(), centro[1] + r * t.sin()]),
                    &mut w,
                )
            })
            .collect();
        let c = pts
            .iter()
            .fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]]);
        let c = [c[0] / f64::from(n), c[1] / f64::from(n)];
        let (mut rmin, mut rmax) = (f64::INFINITY, 0.0_f64);
        for p in &pts {
            let d = (p[0] - c[0]).hypot(p[1] - c[1]);
            rmin = rmin.min(d);
            rmax = rmax.max(d);
        }
        rmax / rmin
    }

    /// ⭐⭐⭐ **NENHUM PEDAÇO DA ARTE DESTA CENA ESTÁ FORA DO ALCANCE DOS OSSOS.**
    ///
    /// ⛔⛔ **Este é o gate que não existia, e a ausência dele custou um smoke reprovado.** Um ponto
    /// órfão salta em salto seco para o osso mais próximo, e um salto seco num mapa contínuo
    /// **rasga a arte** — foi isso que o dono fotografou em 2026-09-15.
    ///
    /// ⚠️ **As duas colunas, e a que decide é a primeira:** a `inverted` lia `0,00 %` sobre a foto
    /// do rasgo, porque inverter e rasgar são defeitos diferentes. Medir só a inversão é o que me
    /// deixou escrever «bem abaixo do ângulo em que o mapa dobra» sobre uma cena partida.
    #[test]
    fn nenhum_pedaco_da_arte_fica_fora_do_alcance_dos_ossos() {
        let r = dobra_da_cena(super::ALTURA_PX, None);
        // ⚠️ Piso de população: uma caixa fora da pele devolveria zero amostras e um relatório
        // limpo que não mediu nada — a própria régua avisa disso por escrito.
        assert!(
            r.samples > 3_000,
            "a regua mediu {} amostras: ela nao esta' sobre a arte",
            r.samples
        );
        assert_eq!(
            r.orphan,
            0.0,
            "{:.2}% da arte esta' ORFA (fora do raio de todo osso) — ela vai saltar em salto seco \
             para o osso mais perto, e o dono ve' a arte RASGADA",
            r.orphan * 100.0
        );
        assert_eq!(
            r.inverted,
            0.0,
            "{:.2}% da arte sai DO AVESSO (det_min {:.4})",
            r.inverted * 100.0,
            r.det_min
        );
    }

    /// ⭐⭐⭐ **A ARTE NÃO SAI FACETADA — e este é o gate do SEGUNDO report do dono.**
    ///
    /// ⛔⛔⛔ *«A malha deforma a curva»* (2026-09-15, com a seta na borda de cima). O mapa estava
    /// **contínuo e injectivo** (`0,00 %` órfã, `0,00 %` do avesso) e mesmo assim a arte saía com
    /// quinas: junto da borda do suporte dos pesos a curvatura do campo explode, e a malha pinta
    /// **um afim por triângulo**. *Um mapa pode estar perfeito e a malha que o amostra não o seguir.*
    ///
    /// ⚠️ **A barra é `1,5 px` de ecrã**, e o número tem os dois lados medidos: a cena entrega
    /// `0,92 px` e a redacção reprovada entregava `14,24 px`. ⛔ Ela **não** é a tolerância do
    /// `Smooth` (`0,5 px`): esse refinamento está estruturalmente desligado nesta densidade (o
    /// handoff §13 tem a aritmética), e uma barra que o produto não consegue honrar seria um número
    /// que mente.
    #[test]
    fn a_arte_nao_sai_facetada() {
        let px = faceta_da_cena(super::ALTURA_PX, None);
        assert!(
            px <= 1.5,
            "a malha erra o campo em {px:.2} px de ecra: a arte sai com quinas e uma circunferencia \
             desenhada por cima dela tambem — e' o report «a malha deforma a curva»"
        );
    }

    /// ⭐⭐⭐ **OS DOIS CONTROLOS: as duas redacções que o dono reprovou continuam a ser acusadas.**
    ///
    /// ⛔⛔ Sem eles os dois gates acima são **vácuos** — uma régua que responde «limpo» a tudo
    /// aprovaria qualquer cena. Cada um pede exactamente a configuração da foto.
    #[test]
    fn as_duas_redaccoes_reprovadas_continuam_a_ser_acusadas() {
        // 1.ª — o canvas QUADRADO com os ossos no alcance de fábrica: a arte RASGA.
        let quadrado = dobra_da_cena(super::LARGURA_PX, Some(1.0));
        assert!(
            quadrado.orphan > 0.30,
            "a regua leu so' {:.2}% de orfas no canvas QUADRADO que foi reprovado: ela deixou de \
             ver o defeito, e o gate irmao passou a nao afirmar nada",
            quadrado.orphan * 100.0
        );
        // 2.ª — a tira com a arte na BORDA do alcance: o mapa fica limpo e a malha FACETA.
        let na_borda = dobra_da_cena(super::ALTURA_PX, Some(1.0));
        assert_eq!(
            (na_borda.orphan, na_borda.inverted),
            (0.0, 0.0),
            "o controlo da faceta deixou de ser o caso SUBTIL: com orfas ou inversao ele passa a \
             medir o defeito do irmao, e a licao — *um mapa limpo pode facetar* — evapora"
        );
        let px = faceta_da_cena(super::ALTURA_PX, Some(1.0));
        assert!(
            px > 10.0,
            "a regua da faceta leu so' {px:.2} px na redaccao reprovada: ela deixou de ver o \
             defeito que o dono fotografou"
        );
    }

    /// ⭐⭐ **A FORÇA É DERIVADA, e a derivação aterra onde foi MEDIDA.**
    ///
    /// ⚠️ O gate não afirma um literal: ele refaz a conta da [`super::forca_do_osso`] a partir da
    /// arte e exige que a fracção do alcance seja a que a tabela mediu. Se alguém mexer na largura,
    /// na altura ou no número de ossos, é **este** gate que fica vermelho — e não o smoke do dono.
    #[test]
    fn a_arte_vive_longe_da_borda_do_alcance() {
        let raio = f64::from(super::LARGURA_PX) / f64::from(super::OSSOS) * super::forca_do_osso();
        let fraccao = (f64::from(super::ALTURA_PX) / 2.0) / raio;
        assert!(
            (fraccao - super::FRACCAO_DO_ALCANCE).abs() < 1e-9,
            "a arte usa {fraccao:.4} do alcance e a tabela mediu {:.4}: a derivacao deixou de \
             aterrar onde foi medida",
            super::FRACCAO_DO_ALCANCE
        );
    }

    /// ⭐⭐⭐ **A CENA DE FACTO DEFORMA A ARTE — e por uma quantidade MEDIDA.**
    ///
    /// ⛔⛔ **É o gate que impede a cura de virar disfarce.** Depois do 3.º report do dono
    /// (*«bem melhor. deformou um pouco»*) a tentação é baixar a dobra até a deformação sumir — e
    /// isso seria uma **cena que ensina o contrário do que acontece** (CLAUDE.md §5.0): o que ela
    /// existe para mostrar é justamente que a arte dobra e que as guias a seguem.
    ///
    /// ⚠️ **A deformação que sobra NÃO é defeito: é o *linear blend skinning* a fazer o que faz.**
    /// Medido, ela é quase LINEAR no ângulo, e a faceta (o defeito que foi curado) acompanha:
    ///
    /// | graus por junta | faceta | círculo fora de redondo |
    /// |---:|---:|---:|
    /// | **`25`** — esta cena | **`0,92 px`** | **`14,9 %`** |
    /// | `15` | `0,57 px` | `8,5 %` |
    /// | `10` | `0,39 px` | `5,5 %` |
    /// | `6` | `0,23 px` | `3,3 %` |
    ///
    /// ⛔ **A alternativa foi medida e recusada pelo próprio repo**: o movimento rígido (o *dual
    /// quaternion* do 2D) foi construído e **PIORA** a dobra — ver o §5 do `CLAUDE.md`.
    ///
    /// ⚠️ **As DUAS metades:** o piso diz *«a cena ainda demonstra»* e o tecto diz *«a arte não está
    /// a ser maltratada»*. Só o tecto seria um gate que uma cena plana passaria.
    #[test]
    fn a_cena_deforma_a_arte_o_bastante_para_demonstrar_e_nao_mais() {
        let fora = circulo_fora_de_redondo(super::ALTURA_PX, None);
        assert!(
            fora > 1.10,
            "a circunferencia sai {:.1}% fora de redondo: a dobra deixou de se VER, e a cena passa \
             a ensinar que prender arte a ossos nao faz nada",
            (fora - 1.0) * 100.0
        );
        assert!(
            fora < 1.20,
            "a circunferencia sai {:.1}% fora de redondo: a arte esta' a ser maltratada, e o dono \
             ve' isso antes de ver a guia que a cena existe para mostrar",
            (fora - 1.0) * 100.0
        );
    }

    /// ⭐⭐⭐ **A CENA PRENDE ANTES DE DOBRAR, E A ORDEM É LOAD-BEARING.**
    ///
    /// ⛔⛔ O repouso de uma pele é **o instante do bind**. Dobrada primeiro, esta pose SERIA o
    /// repouso e o canvas sairia RECTO — a cena montaria, imprimiria a linha de sucesso, e não
    /// provaria nada. *Um smoke que monta e não demonstra é pior que um ausente: ele é acreditado.*
    ///
    /// ⚠️ A régua é a POSIÇÃO no ficheiro, que é o que uma leitura rápida do diff inverte sem dar
    /// por isso — mover a chamada da `dobra` para cima de `bind_image` é uma edição de duas linhas.
    #[test]
    fn the_scene_binds_before_it_bends() {
        let fonte = include_str!("smoke_bone_paint.rs");
        let bind = fonte.find("bind_image(\n").expect("a cena prende a imagem");
        let dobra = fonte
            .find("dobra(sim, &ossos);")
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
