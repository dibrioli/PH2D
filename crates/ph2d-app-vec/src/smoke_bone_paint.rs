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
//! # ⭐⭐⭐ O QUE ELA DEMONSTRA MUDOU EM 2026-09-15, POR ORDEM DO DONO
//!
//! *«Inative a possibilidade de pintar sobre malha deformada por ossos. SE o usuário entrar no modo
//! Painter em imagem deformada por ossos a imagem deixa a deformação para ser pintada. Ao sair do
//! modo painter, ela retorna a deformação.»*
//!
//! ⇒ a cena passou a demonstrar o **ciclo** (dobrado → achata ao pegar no pincel → pinta → dobra
//! outra vez ao largá-lo, com a tinta a dobrar junto), e **não** mais as guias a seguir a curva.
//!
//! ⚠️⚠️ **E a consequência está NOMEADA e é do dono decidir:** o chrome do Painter que passou a
//! seguir a arte dobrada (a grelha, os selos, os gizmos, os contornos das formas — item 4) **deixa
//! de ter sujeito enquanto se pinta**, porque durante a pintura não há dobra nenhuma. ⛔ Ele não foi
//! apagado e não custa nada: sem malha o [`ph2d_app_painter::canvas_map::CanvasMap`] degenera no
//! afim do quad, **ao bit**. Fica **dormente** — e volta a valer no dia em que pintar sobre a dobra
//! for permitido. *Um roteiro de smoke que continuasse a mandar procurar a curva debaixo do pincel
//! seria uma cena a ensinar o contrário do que acontece.*
//!
//! # ⭐⭐⭐ E EM 2026-09-16 A ORDEM VIROU REGRA (fila do esqueleto, F6-s)
//!
//! *Toda* ferramenta que pinta, apaga ou deforma pixels endireita a imagem (a Remoção de fundo
//! incluída), **menos o Liquify**, que trabalha sobre a dobra. ⇒ o roteiro passou a ter as TRÊS
//! metades: o Painter endireita, o Liquify volta a dobrar (e o anel dele segue a dobra — o chrome
//! dormente acorda ali), a Remoção de fundo endireita outra vez. ⭐ E as de TAMANHO/MARGEM (a
//! resposta do dono no mesmo dia): o Padding endireita e avisa; o Make Square — o canvas desta cena
//! não é quadrado — solta a imagem dos ossos, e o Ctrl+Z devolve-a.
//!
//! ⚠️ **Se a linha `[bone-paint-smoke]` não aparecer, PARE:** a cena não montou.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// ⭐⭐⭐ **O NÍVEL DESTE ROTEADOR É UMA CONTAGEM: quantos canvas dobrados a cena monta.**
///
/// ⚠️⚠️ **A premissa da redacção anterior MORREU em 2026-09-17, e a morte é a wave.** Ela dizia:
/// *«este é de PRESENÇA, como o da máscara: ele lê `var_os(..).is_some()`, logo não há `match` de
/// níveis e o maior com significado é 1»*. Isso era verdade enquanto a única pergunta desta cena
/// era o **Painter sobre uma arte dobrada**. A F9 trouxe outra, e ela **não é observável com uma
/// imagem só**: o `Smooth` colapsava no `Fast` quando a soma das malhas presas passava o orçamento
/// do quadro, e uma cena com UM canvas nunca lá chega.
///
/// ⇒ `PH2D_VEC_BONE_PAINT_SMOKE=<n>` monta `n` canvas. ⭐ **`=1` continua byte-idêntico** (há gate),
/// e é isso que mantém o roteiro de 8 passos que o dono já aprovou.
///
/// ⚠️ **O tecto é a LEGIBILIDADE, e não um recurso:** acima de `6` os canvas deixam de caber no
/// ecrã com a dobra à vista, e uma cena que sai do enquadramento ensina o contrário do que diz
/// (a lição que a foto do `ParticleEmitter` pagou). Quem quiser medir mais tem a sonda
/// `o_que_um_quadro_custa_com_a_malha_assada`, que varre `1`, `4` e `8` sem ecrã nenhum.
pub const NIVEIS: u32 = 6;

/// ⭐ **Quantos canvas a cena monta** — o valor do roteador, coagido a `1..=NIVEIS`.
///
/// ⚠️ Um valor ilegível (ou a env vazia, que é como um `env VAR=` a arma) cai em `1`: *o caminho de
/// omissão é a cena que o dono já aprovou, nunca uma que ele não pediu.*
#[must_use]
pub fn quantos() -> u32 {
    quantos_de(std::env::var("PH2D_VEC_BONE_PAINT_SMOKE").ok().as_deref())
}

/// A LEI do [`quantos`], sem a env.
///
/// ⚠️ **Ela existe separada porque uma env NÃO se escreve num gate:** `std::env::set_var` é
/// `unsafe` na edição 2024 e corre numa árvore de testes com threads — *a lei é parâmetro e a
/// leitura da env é uma linha em quem chama*, que é a mesma forma que a porta da assadura já usa.
#[must_use]
pub(crate) fn quantos_de(v: Option<&str>) -> u32 {
    v.and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, NIVEIS)
}

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

/// ⛔⛔⛔ **A `FRACCAO_DO_ALCANCE` E A `forca_do_osso()` MORRERAM em 2026-09-15, e o que as matou
/// foi a LEI DOS PESOS mudar por baixo delas.**
///
/// Elas existiram por duas semanas e resolviam um problema real: com o *bump* euclidiano
/// `(1 − (d/raio)²)²` a arte tinha de viver **dentro** do alcance dos ossos e **longe da borda**
/// dele — fora, a pele saltava em salto seco para o osso mais próximo e a arte RASGAVA (1.º report
/// do dono); na borda, os pesos normalizados tendiam todos a zero ao mesmo tempo, a curvatura do
/// campo explodia e a arte FACETAVA (2.º report, *«a malha deforma a curva»*). A cura era derivar a
/// `strength` da altura da arte, e ela chegou a `2,0` nesta cena.
///
/// ⭐⭐⭐ **O padrão-ouro apaga a pergunta inteira.** Os *Bounded Biharmonic Weights* são resolvidos
/// **sobre a arte** e não sobre um raio: não há órfão por construção (todo ponto do domínio tem
/// peso) e não há borda de suporte (a curvatura é o que a energia minimiza). ⇒ **o alcance deixa de
/// decidir o que quer que seja para uma imagem**, e isto não é uma opinião — está MEDIDO:
///
/// | lei | `strength` | peças | faceta | esticão | círculo | **VAZAMENTO** |
/// |---|---:|---:|---:|---:|---:|---:|
/// | euclidiana (local) | `1,0` | `2 430` | `6,78 px` | `2,234` | `1,2923` | `0,00 px` |
/// | euclidiana (a que shipou) | `2,0` | `2 430` | `0,38 px` | `1,267` | `1,1499` | ⛔ **`26,37 px`** |
/// | **padrão-ouro** | `1,0` | `2 430` | **`1,27 px`** | `1,492` | `1,2913` | ⭐ **`0,78 px`** |
/// | **padrão-ouro** | `2,0` | `2 430` | **`1,27 px`** | `1,492` | `1,2913` | ⭐ **`0,78 px`** |
///
/// ⭐ **As duas últimas linhas são IDÊNTICAS coluna a coluna** — é essa a prova de que a `strength`
/// ficou inerte para uma imagem, e é por isso que esta cena voltou ao valor de fábrica.
///
/// ⛔⛔ **E a coluna que decide é a ÚLTIMA, que não existia quando a `strength = 2,0` shipou.**
/// *Vazamento* é: rodar só a PONTA da corrente e medir quanto a arte da RAIZ se mexe. As três
/// colunas de suavidade são ganhas **por construção** por uma mistura larga de mais — um osso cujo
/// raio cobre a arte inteira faz toda a arte responder a todos os ossos, o que é suavíssimo e **não
/// é um rig**. A `2,0` rodar a ponta arrasta a raiz `26 px`; o padrão-ouro arrasta `0,78`.
///
/// ⚠️ **O que se PERDE está dito e é real:** o círculo desenhado sai de `1,15` para `1,29` de
/// ovalização. ⛔ Isso **não** é uma regressão do motor — é a articulação a passar a existir: uma
/// circunferência desenhada por cima de uma junta que dobra `25°` **tem** de deformar, e o que a
/// mantinha redonda era o rig não estar de facto a articular.
///
/// ⛔ **Limite NOMEADO:** uma forma VECTORIAL presa ao mesmo esqueleto continua na lei euclidiana
/// (o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma), logo ali a `strength`
/// ainda manda. Um rig com as duas mídias tem hoje duas leis.
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
fn corrente_em(sim: &mut SimWorld, pixels_per_meter: f32, centro: [f64; 2]) -> Option<Vec<Entity>> {
    let mut pai: Option<Entity> = None;
    let mut ossos = Vec::new();
    for (k, (a, b)) in eixos_em(pixels_per_meter, centro).into_iter().enumerate() {
        let Some(osso) = ph2d_skeleton_live::bone::create(sim, pai, a, b) else {
            eprintln!("[bone-paint-smoke] o osso {k} nao nasceu -- PARE");
            return None;
        };
        let ent = Entity::try_from_bits(osso)?;
        // ⭐⭐⭐ **NADA é escrito no alcance: ele fica no valor de FÁBRICA.** Ver o bloco que
        // substituiu a `forca_do_osso` — com os pesos do padrão-ouro a `strength` é inerte para uma
        // imagem, e as duas linhas medidas daquela tabela são idênticas coluna a coluna.
        ossos.push(ent);
        pai = Some(ent);
    }
    Some(ossos)
}

/// A corrente, com o CENTRO escolhido — a cena lotada põe uma por canvas.
///
/// ⚠️ **Uma lei, dois consumidores.** Escrever a disposição outra vez na cena lotada faria as duas
/// divergirem no primeiro ajuste, e os gates continuariam a medir a de cima.
fn eixos_em(pixels_per_meter: f32, centro: [f64; 2]) -> Vec<([f64; 2], [f64; 2])> {
    let largura = f64::from(LARGURA_PX) / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let (x0, passo) = (centro[0] - largura / 2.0, largura / f64::from(OSSOS));
    (0..OSSOS)
        .map(|k| {
            (
                [x0 + passo * f64::from(k), centro[1]],
                [x0 + passo * f64::from(k + 1), centro[1]],
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

/// Monta a cena. Devolve `(bits do 1.º canvas, quantos canvas foram montados)` — o chamador assenta
/// a selecção no primeiro e avança as células do atlas pelo segundo.
///
/// ⚠️ **UM tempo só, ao contrário da [`crate::smoke_bone`]**: aquela prende FORMAS vectoriais e
/// precisa da entidade que o `vec_entities::sync` cria no meio do quadro. Aqui o sujeito é uma
/// IMAGEM, e ela já existe no instante em que nasce.
///
/// ⭐⭐⭐ **`n = quantos()` canvas, e é essa contagem que torna a F9 OBSERVÁVEL** — ver o doc de
/// [`NIVEIS`]. Com `n = 1` a cena é a de sempre, ao bit.
pub fn build(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, u32)> {
    let n = quantos();
    let mut primeiro = None;
    let mut montados = 0_u32;
    for k in 0..n {
        let centro = centro_do_canvas(k, n, pixels_per_meter);
        let Some(bits) = um_canvas(
            sim,
            renderer,
            asset_db,
            cell_idx + k,
            pixels_per_meter,
            atlas_asset_map,
            centro,
            k,
        ) else {
            break;
        };
        primeiro.get_or_insert(bits);
        montados += 1;
    }
    let bits = primeiro?;
    anuncia(n, montados);
    Some((bits, montados.max(1)))
}

/// ⭐ **Onde o canvas `k` de `n` fica, em metros de mundo** — uma FILEIRA centrada na origem.
///
/// ⛔⛔ **Uma COLUNA foi construída, FOTOGRAFADA e REVERTIDA, e a causa não é o espaçamento:** a
/// arte dobrada **varre para CIMA** muito além da caixa de repouso dela (com `OSSOS = 3` a ponta
/// roda `2 × DOBRA_GRAUS`), e o enquadramento automático da cena (`ViewFocusKind::All`) ajusta-se às
/// caixas das sprites, não ao que a pele desenha. ⇒ numa coluna o canvas de cima fica **sempre**
/// cortado no topo, por mais que se afastem — *nenhum valor do parâmetro livre serve, logo o que
/// está errado é a disposição.*
///
/// ⭐ **Deitada, o problema desaparece por geometria:** dobrar **encurta** a pegada horizontal (a
/// tira enrola-se), logo a largura de repouso é um tecto para o que se vê, e o enquadramento por
/// caixas acerta. ⚠️ **`1,2 ×` a largura** é o primeiro passo que separa as pontas de duas
/// vizinhas sem as encostar.
fn centro_do_canvas(k: u32, n: u32, pixels_per_meter: f32) -> [f64; 2] {
    if n <= 1 {
        return [0.0, 0.0];
    }
    let largura = f64::from(LARGURA_PX) / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let passo = largura * 1.2;
    let x0 = -passo * f64::from(n - 1) / 2.0;
    [x0 + passo * f64::from(k), 0.0]
}

/// Um canvas: sobe a tinta, monta a corrente dele, prende e dobra. `None` se alguma metade falhar.
#[expect(
    clippy::too_many_arguments,
    reason = "e' o construtor de uma cena: cada argumento e' uma porta do app (render, assets,               atlas, regua, posicao) e agrupa-los numa struct so' esconderia isso"
)]
fn um_canvas(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    centro: [f64; 2],
    k: u32,
) -> Option<u64> {
    // ⚠️ **A porta RETANGULAR, e não a do *New Image…***: a `spawn_blank_canvas` é quadrada por
    // desenho (é o caminho daquele modal), e um canvas quadrado é exactamente o que esta cena não
    // pode ter. O doc da `spawn_rgba` diz que ela foi extraída para isto — *«uma tira RETANGULAR
    // com conteúdo»*.
    let nome = if k == 0 {
        "Canvas".to_owned()
    } else {
        format!("Canvas {}", k + 1)
    };
    let (label, bits) = match ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LARGURA_PX,
        ALTURA_PX,
        branco(LARGURA_PX, ALTURA_PX),
        Vec2::new(centro[0] as f32, centro[1] as f32),
        pixels_per_meter,
        atlas_asset_map,
        &nome,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[bone-paint-smoke] o canvas {k} nao subiu: {e}");
            return None;
        }
    };
    let e = Entity::try_from_bits(bits)?;

    // ── Os três ossos, deitados ao longo do canvas ──────────────────────────────────────────────
    let Some(ossos) = corrente_em(sim, pixels_per_meter, centro) else {
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
            "[bone-paint-smoke] o canvas '{label}' NAO prendeu ao esqueleto -- PARE, a cena nao \
             montou"
        );
        return Some(bits);
    }

    // ── E só AGORA a dobra ──────────────────────────────────────────────────────────────────────
    //
    // ⚠️⚠️ **Depois de prender, nunca antes.** O repouso de uma pele é o instante do bind: dobrada
    // antes, esta pose SERIA o repouso e o canvas sairia recto — a cena montaria e não provaria nada.
    dobra(sim, &ossos);
    Some(bits)
}

/// O roteiro, e ele é OUTRO conforme a cena tem um canvas ou muitos.
///
/// ⛔⛔ **Duas cenas, dois roteiros — e não um roteiro com uma frase a mais.** A cena de UM canvas é
/// sobre o Painter a endireitar a arte; a lotada é sobre o `Smooth` a alisar **apesar** da cena
/// cheia. *Um roteiro que tenta ensinar as duas manda o dono fazer oito passos para chegar ao que
/// ele foi ver.*
fn anuncia(pedidos: u32, montados: u32) {
    if montados < pedidos {
        eprintln!(
            "[bone-paint-smoke] PARE: pedi {pedidos} canvas e montei {montados} — a cena nao esta' \
             completa"
        );
    }
    if pedidos <= 1 {
        println!(
            "[bone-paint-smoke] canvas 'Canvas' ({LARGURA_PX}x{ALTURA_PX}, branco) PRESO a {OSSOS} \
             ossos e dobrado {DOBRA_GRAUS}° por junta. NADA mais esta' armado.\n\
             [bone-paint-smoke] 1) veja o canvas DOBRADO  2) pegue a ferramenta Painter: ele tem de \
             ENDIREITAR-SE para ser pintado, e um aviso diz porque  3) pinte qualquer coisa (uma \
             forma, um traco)  4) no Painter escolha o Liquify: o canvas VOLTA a dobrar-se, o anel \
             do cursor segue a dobra e empurrar deforma a imagem dobrada  5) pegue a Remocao de \
             fundo (tecla 3): endireita-se outra vez  6) pegue noutra ferramenta (Select): o canvas \
             tem de VOLTAR a dobrar-se, e o que voce pintou tem de dobrar com ele  7) PAD (Padding): \
             endireita e o aviso diz que o Apply solta dos ossos  8) SQUAR (Make Square): o canvas \
             fica QUADRADO, RETO e SOLTO dos ossos; Ctrl+Z devolve-o dobrado."
        );
        return;
    }
    println!(
        "[bone-paint-smoke] {montados} canvas ({LARGURA_PX}x{ALTURA_PX}, brancos) PRESOS a {OSSOS} \
         ossos cada e dobrados {DOBRA_GRAUS}° por junta — a cena CHEIA, que e' onde o Smooth \
         costumava desistir. ⚠️ So' o PRIMEIRO esta' enquadrado: os outros ficam de lado de \
         proposito, para ENCHER o orcamento do quadro. E' a junta do que esta' a` sua frente que \
         voce julga.\n\
         [bone-paint-smoke] 1) abra o painel dos ossos: menu Window > Bones, e carregue no separador \
         «Bones» a` direita  2) na linha «Deform» carregue em «Fast»: a curva de cima do canvas fica \
         com ARESTAS RETAS  3) carregue em «Smooth»: ela fica CURVA  4) o contraste: feche o app e \
         volte a abri-lo com PH2D_SKIN_BAKE=0 na frente do comando; ali o «Smooth» desenha \
         exactamente o mesmo que o «Fast» — era esse o defeito."
    );
}

#[cfg(test)]
#[path = "smoke_bone_paint_tests.rs"]
mod tests;
