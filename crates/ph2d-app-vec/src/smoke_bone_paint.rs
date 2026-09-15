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

/// ⭐⭐⭐ **A altura do canvas é DERIVADA do alcance de um osso — nunca escrita à mão.**
///
/// ⛔⛔ É esta derivação que impede a banda órfã que reprovou a 1.ª redacção (ver o cabeçalho). O
/// raio de um osso é `comprimento × strength`, o comprimento é `LARGURA_PX / OSSOS`, e a arte sobe
/// `ALTURA_PX / 2` acima do eixo ⇒ **a meia-altura tem de ficar ABAIXO do comprimento de um osso**.
///
/// ⚠️ **A folga é `15/16`, e ela não é decoração:** a medição põe a ravina exactamente onde a
/// aritmética a põe (meia-altura `168` limpa, `176` já com `4,12 %` de órfãs sobre um alcance de
/// `170,67`), e uma fixtura encostada à ravina é uma fixtura que a próxima edição empurra lá para
/// dentro. ⛔ Escrever `320` como literal ao lado de `512` seriam **dois números independentes**, e
/// mexer num deles levaria a arte para fora do alcance **em silêncio** — que é o defeito que esta
/// cena pagou uma vez.
const ALTURA_PX: u32 = 2 * LARGURA_PX * FOLGA_16 / (OSSOS * 16);

/// A folga entre a meia-altura da arte e o alcance de um osso, em dezasseis avos. Ver [`ALTURA_PX`].
const FOLGA_16: u32 = 15;

/// Quanto cada junta dobra, em graus.
///
/// ⚠️ **Escolhido para a dobra ser VISÍVEL sem inverter a malha**: medido nesta geometria, `25°` por
/// junta dá `det_min 0,2497` com `0,00 %` da arte do avesso e `0,00 %` órfã (tabela no cabeçalho).
/// ⛔ **A régua que decide é a da ÓRFÃ, não a da inversão** — foi trocar as duas que reprovou a 1.ª
/// redacção desta cena.
const DOBRA_GRAUS: f32 = 25.0;

/// **Os eixos dos ossos, em metros de mundo** — a corrente deitada ao longo da largura do canvas.
///
/// ⚠️ **Uma porta só, porque o gate monta a MESMA corrente.** O gate não pode chamar a [`build`]
/// (ela precisa do renderer e do atlas), então o que ele reusa é isto: escrever a disposição duas
/// vezes faria o gate medir uma cena que o dono não vê. *Duas cópias de uma geometria convergem
/// enquanto ninguém mexe numa delas.*
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
    let mut pai: Option<Entity> = None;
    let mut ossos = Vec::new();
    for (k, (a, b)) in eixos(pixels_per_meter).into_iter().enumerate() {
        let Some(osso) = ph2d_skeleton_live::bone::create(sim, pai, a, b) else {
            eprintln!("[bone-paint-smoke] o osso {k} nao nasceu -- PARE");
            return Some(bits);
        };
        let ent = Entity::try_from_bits(osso)?;
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

    /// ⭐⭐⭐ **A PELE QUE A CENA PRODUZ, medida pela PORTA DO PRODUTO.**
    ///
    /// ⚠️ Ela monta a corrente com a [`super::eixos`] (a mesma que a [`super::build`] usa), prende
    /// pelo [`ph2d_skeleton_live::skin_live::bind_image`] e resolve pelo `skin_of` — ⛔ **nenhuma
    /// cinemática escrita aqui**. Uma segunda cadeia de transformações neste ficheiro seria uma
    /// segunda resposta à mesma pergunta, e mediria uma cena que o dono não vê.
    ///
    /// A `altura_px` é parâmetro para o gate do CONTROLO poder pedir o quadrado que foi reprovado.
    fn dobra_da_cena(altura_px: u32) -> ph2d_skeleton::fold::FoldReport {
        let mut sim = SimWorld::default();
        let mut pai: Option<Entity> = None;
        let mut ossos = Vec::new();
        for (a, b) in super::eixos(PPM) {
            let osso = ph2d_skeleton_live::bone::create(&mut sim, pai, a, b).expect("osso");
            let ent = Entity::from_bits(osso);
            ossos.push(ent);
            pai = Some(ent);
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
        let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("a pele resolve");
        let (mx, my) = (f64::from(tamanho[0]) / 2.0, f64::from(tamanho[1]) / 2.0);
        ph2d_skeleton::fold::measure(&pele, [-mx, -my, mx, my], 65)
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
        let r = dobra_da_cena(super::ALTURA_PX);
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

    /// ⭐⭐⭐ **O CONTROLO: a régua VÊ o defeito que reprovou a 1.ª redacção.**
    ///
    /// ⛔⛔ Sem esta metade o gate acima é **vácuo** — uma régua que responde `0,00 %` a tudo
    /// aprovaria qualquer canvas. Ela pede exactamente o canvas QUADRADO que o dono reprovou e
    /// exige que a coluna da órfã o acuse.
    ///
    /// ⚠️ E ela é também a prova da aritmética do cabeçalho: com `n` ossos ao longo de `W`, o
    /// alcance é `W/n` e a arte sobe `W/2` — *o quadrado tem banda órfã por construção*.
    #[test]
    fn o_canvas_quadrado_que_o_dono_reprovou_e_acusado_por_esta_mesma_regua() {
        let r = dobra_da_cena(super::LARGURA_PX);
        assert!(
            r.orphan > 0.30,
            "a regua leu so' {:.2}% de orfas no canvas QUADRADO que foi reprovado: ela deixou de \
             ver o defeito, e o gate irmao passou a nao afirmar nada",
            r.orphan * 100.0
        );
    }

    /// ⭐⭐ **A ALTURA É DERIVADA, e a lei está escrita no tipo: meia-altura < alcance de um osso.**
    ///
    /// ⚠️ Ela lê a `strength` do **produto** ([`ph2d_skeleton_ecs::Bone::default`]) e não um `1.0`
    /// escrito aqui: se o default do alcance mudar, é este gate que fica vermelho — e não o smoke do
    /// dono. ⛔ Uma asserção sobre um literal meu não teria essa propriedade.
    #[test]
    fn a_meia_altura_cabe_no_alcance_de_um_osso() {
        let strength = ph2d_skeleton_ecs::Bone::default().strength;
        let comprimento = f64::from(super::LARGURA_PX) / f64::from(super::OSSOS);
        let alcance = comprimento * strength;
        let meia_altura = f64::from(super::ALTURA_PX) / 2.0;
        assert!(
            meia_altura < alcance,
            "meia-altura {meia_altura} nao cabe no alcance {alcance} — a arte nasce com banda orfa"
        );
        // ⚠️ E a folga é a MEDIDA, não «alguma»: encostada à ravina, a próxima edição cai lá dentro.
        assert!(
            meia_altura / alcance <= 0.95,
            "a folga ficou em {:.4} do alcance — a fixtura esta' encostada a' ravina",
            meia_altura / alcance
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
