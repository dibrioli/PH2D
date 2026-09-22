//! ⭐⭐⭐ **O LOD DA FORMA — a tile entra quando a forma fica pequena no ecrã** (report do Enio,
//! 2026-09-21: *«ao dar o zoom … logo que muitas estrelas apareçam o app trava. Não seria
//! interessante criar um LOD para shapes? De modo que haja uma forte otimização da forma quando
//! ela fica menor na tela?»*).
//!
//! ## ⛔⛔ Isto NÃO é um motor novo — são DUAS peças que já existiam e não se conheciam
//!
//! O irmão [`apply_object_lod`](crate::motion_bridge_objects::apply_object_lod) troca a forma pela
//! TILE desde 2026-08-05, e o assador de tiles de forma **paramétrica**
//! ([`crate::motion_shape_bake`]) existe desde 2026-08-20. O que faltava era o elo: aquele LOD
//! pergunta ao [`ObjectBake`](crate::motion_object_bake), que só cobre `VecPathId` do documento, e
//! uma estrela de `source.shape` não é um — logo `tile_texture_for_gid` devolvia `None` e a cerca
//! *«sem tile fica crisp»* mandava as `90 000` cópias ao caminho caro, **em silêncio**.
//!
//! *Medido antes da primeira linha: a cena `=126` está `5,6×` acima do joelho daquele LOD e ele
//! move ZERO cópias.*
//!
//! ## A régua, e por que ela é o TAMANHO e não a contagem
//!
//! Ao afastar, o custo **satura**: `0,500` / `0,250` / `0,125` de zoom leem `4,58` / `4,55` /
//! `4,58 ms` e `44,3 MB` para a placa — enquanto a estrela encolhe de `3,00` para `0,75 px`.
//! *O número de segmentos por forma não depende de quantos píxeis ela ocupa*, e é exactamente essa
//! a frase que o dono escreveu por outras palavras.
//!
//! ## ⛔⛔⛔ E SIMPLIFICAR A SILHUETA é a cura ERRADA — foi a 1.ª hipótese e a medição refutou-a
//!
//! | lado no ecrã | TILE reamostrada | largar o arredondamento |
//! |---|---|---|
//! | `32 px` | `64,0` | `255,0` |
//! | `8 px` | `9,9` | `93,4` |
//! | **`4 px`** | **`0,4`** | `34,9` |
//! | **`2 px`** | **`0,6`** | `10,5` |
//! | **`1 px`** | **`0,2`** | `7,0` |
//!
//! (níveis de `255` de diferença máxima por pixel, contra a forma exacta — sonda
//! `audit_how_much_a_tile_would_err`.)
//!
//! Largar o arredondamento **nunca** fica invisível; a tile fica, e a partir de `4 px`. ⚠️ Uma
//! régua GEOMÉTRICA dizia o contrário (*«invisível abaixo de `4,4 px`»*) e estava a medir a
//! grandeza errada: o desvio da SILHUETA, não o que sobrevive à grelha do ecrã.
//!
//! ## ⚠️ O que esta troca CUSTA, e é declarado
//!
//! Uma tile é um quad do passe de sprites, e *«todo vector fica por cima de todo sprite»*
//! ([`VectorInstance::texture_id`](ph2d_eval_motion::VectorInstance)) ⇒ **uma forma que vira tile
//! desce no z**, para trás de qualquer sprite da cena. É a MESMA troca que o irmão já faz desde
//! que nasceu, e pela mesma razão (o passe instanciado é o que escala a milhões). ⛔ A alternativa
//! — um quad na própria cena vectorial — mantinha o z e pagava **uma imagem do Vello por cópia**,
//! que é o custo que esta wave existe para não pagar.

use crate::motion_shape_bake::{ShapeBake, tile_quad};
use crate::motion_shape_gen::VecPathStore;
use ph2d_eval_motion::VectorInstance;
use ph2d_render::RenderInstance;
use ph2d_vector::Affine;

/// **O lado máximo, em PÍXEIS DE ECRÃ, a que uma tile ainda é indistinguível da forma** — a barra
/// do LOD, e ela é MEDIDA (a tabela está no cabeçalho deste módulo).
///
/// ⚠️⚠️ **Ela NÃO é o lado da tile.** O assador dá a esta estrela uma tile de `28 px`
/// ([`BAKE_DPI`](crate::motion_object_bake::BAKE_DPI) `= 256 px/unidade`), e a tentação é dizer
/// *«serve até `28`»*. Medido, a `28 px` ela erra `17,9` níveis e a `8 px` ainda erra `9,9`: o que
/// fixa a barra é a **REAMOSTRAGEM** — acima dela a tile perde as pontas finas que a cobertura
/// analítica do rasterizador ainda resolve. O recurso tem nome e é esse.
///
/// ⭐ A medição vale porque o [`IndividualTextureStore`](ph2d_render) **tem a cadeia de mips
/// inteira e amostra trilinear** (`mipgen::mip_levels` + `MipGenerator`; o doc do
/// `create_entry_empty` di-lo por escrito) — um mipmap é uma pirâmide de filtros de caixa, que é
/// o modelo com que a sonda mediu. Sem mipmap a minificação `28 → 3 px` leria **um** texel de
/// `784` e esta barra não descreveria nada.
pub const LADO_MAXIMO_PX: f64 = 4.0;

/// ⚠️ **A barra é ERRO DE COMPILAÇÃO e não um gate**, e a razão é medida: um `assert!` de teste
/// sobre duas constantes é **dobrado pelo compilador** antes de correr (o clippy di-lo em voz
/// alta), logo ele afirmava nada. Isto ancora o número MEDIDO ao documento que o justifica — a
/// sonda deu `0,4` níveis a `4 px` e `9,9` a `8 px` —, e a segunda metade diz o que ele NÃO é.
const _: () = assert!(
    LADO_MAXIMO_PX == 4.0,
    "a sonda `audit_how_much_a_tile_would_err` deu 0,4 níveis a 4 px e 9,9 a 8 px"
);
const _: () = assert!(
    LADO_MAXIMO_PX < 28.0,
    "⛔ o lado da TILE (28 px) não é a barra — quem a fixa é a REAMOSTRAGEM"
);

/// A caixa de uma geometria em unidades LOCAIS — medida pela mesma porta que o assador usa
/// ([`ph2d_vec_render::standalone_path_screen_bounds`]), porque *duas respostas à mesma pergunta
/// divergem no dia em que uma delas mudar*.
///
/// ⚠️ **Sob a IDENTIDADE, e não sob a câmara.** A pose de uma cópia é `cam · T · base · escala`, e
/// duas matrizes não comutam: medir sob a câmara e multiplicar depois pela instância mediria outra
/// coisa. A caixa local é a única que se mede uma vez e vale para todas as cópias.
fn caixa_local(store: &VecPathStore, gid: u32) -> Option<(f64, f64)> {
    let path = store.get(gid)?;
    let (x0, y0, x1, y1) = ph2d_vec_render::standalone_path_screen_bounds(path, Affine::IDENTITY)?;
    Some((x1 - x0, y1 - y0))
}

/// O lado que uma caixa `w × h` ocupa no ecrã sob o afim `a` — a extensão da caixa ALINHADA AOS
/// EIXOS, que é o que o desenho de facto cobre.
///
/// ⚠️⚠️ **As duas entradas de cada LINHA somam-se; elas não se maximizam.** Uma redacção que
/// tomasse `max(|a|·w, |c|·h)` lê um quadrado de `4 px` rodado a `45°` como tendo `2,83 px`
/// quando ele desenha `5,66` — a diagonal dele —, e mandava-o à tile. *Foi uma mutação sobrevivente
/// que nomeou esse buraco, e o CONTROLO (a mesma forma sem rotação) que apanhou a redacção que o
/// curava a dobrar todo tamanho.*
fn lado_no_ecra(a: Affine, w: f64, h: f64) -> f64 {
    let [m00, m10, m01, m11, _, _] = a.as_coeffs();
    (m00.abs() * w + m01.abs() * h).max(m10.abs() * w + m11.abs() * h)
}

/// **Quais geometrias o LOD quer como tile neste quadro** — as que são carimbadas mais de
/// `joelho` vezes **e** cuja MAIOR cópia cabe em [`LADO_MAXIMO_PX`].
///
/// ⚠️⚠️ **O tamanho que decide é o da MAIOR cópia, nunca o da primeira nem o médio.** Uma
/// geometria com `89 999` cópias minúsculas e UMA grande tem de ficar crisp: a decisão é por
/// geometria (como a do irmão), logo a cópia grande levaria a tile com ela e o artista veria
/// exactamente aquela borrada. *Um máximo é o único agregado que não esconde o caso que dói.*
///
/// ⚠️ **A caixa é medida UMA vez por geometria e a cópia é aritmética.** Chamar a porta das
/// fronteiras por cópia seria `O(N)` travessias de geometria num quadro que já é o gargalo; a
/// extensão de uma caixa `w × h` sob um afim é `(|a00|·w + |a01|·h, |a10|·w + |a11|·h)`, exacta
/// para a caixa alinhada aos eixos que o desenho de facto ocupa.
///
/// ⚠️ **O `joelho` é PARÂMETRO e não uma leitura de constante**: a lei desta casa é que um gate
/// que leia o que o produto lê mede o produto e não a lei. O valor do produto é o
/// [`LOD_COUNT`](crate::motion_bridge_objects::LOD_COUNT) — o joelho que a casa já mediu para
/// exactamente esta pergunta (*quantas cópias crisp antes de doer*). ⏳ Baixá-lo pede medir o
/// custo do ASSADO, que esta wave não mediu: até lá, uma cena com poucos milhares de formas
/// minúsculas continua crisp, e isso é uma fronteira nomeada, não um esquecimento.
pub fn geometrias_para_lod(
    insts: &[VectorInstance],
    store: &VecPathStore,
    cam: Affine,
    joelho: usize,
) -> std::collections::BTreeSet<u32> {
    geometrias_para_lod_com(insts, store, cam, joelho, lod_ligado())
}

/// **A porta de bissecção** — `PH2D_LOD_DA_FORMA=0` devolve o de antes (toda forma desenhada
/// crisp, a qualquer tamanho).
///
/// ⚠️ **Lida UMA vez, e só aqui** — a mesma lei das irmãs [`recorte_ligado`](ph2d_vec_render) e
/// `carimbo_preparado`: *um gate que lê o ambiente mede a máquina*, logo os gates entram pelo
/// [`geometrias_para_lod_com`] e escolhem a rota por parâmetro.
fn lod_ligado() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| lod_por(std::env::var("PH2D_LOD_DA_FORMA").ok().as_deref()))
}

/// A LEI da porta acima, **pura** — o que a variável significa, sem a ler.
pub fn lod_por(valor: Option<&str>) -> bool {
    !matches!(valor.map(str::trim), Some("0"))
}

/// O CORPO, com a rota como **parâmetro**. Ver o doc da porta acima.
pub fn geometrias_para_lod_com(
    insts: &[VectorInstance],
    store: &VecPathStore,
    cam: Affine,
    joelho: usize,
    ligado: bool,
) -> std::collections::BTreeSet<u32> {
    if !ligado {
        return std::collections::BTreeSet::new();
    }
    // 1.ª passagem: conta as cópias. A caixa de cada geometria mede-se DEPOIS, uma vez.
    let mut contagem: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for vi in insts {
        if vi.geometry_id == 0 {
            continue; // já é um quad texturado — não é uma forma
        }
        *contagem.entry(vi.geometry_id).or_insert(0) += 1;
    }
    let caixas: std::collections::BTreeMap<u32, (f64, f64)> = contagem
        .iter()
        .filter(|(_, n)| **n > joelho)
        .filter_map(|(&gid, _)| Some((gid, caixa_local(store, gid)?)))
        .collect();
    if caixas.is_empty() {
        return std::collections::BTreeSet::new();
    }
    // 2.ª passagem: basta **UMA** cópia acima da barra para a geometria ficar crisp.
    //
    // ⭐⭐ **É um `any`, não um `max`, e a diferença é MEDIDA.** As duas respondem o mesmo (*«o
    // máximo cabe»* ⟺ *«nenhuma excede»*), e a primeira pode parar na cópia que decide. No zoom
    // de nascimento da cena `=126` a PRIMEIRA já excede ⇒ a decisão passa de `O(N)` a `O(1)`
    // exactamente no quadro em que ela é trabalho para o lixo. *Uma cura cujo teste custa o que
    // ela poupa não é uma cura* — a lei que o recorte por câmara desta mesma linha já pagou.
    let mut fora: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for vi in insts {
        if fora.contains(&vi.geometry_id) {
            continue;
        }
        let Some(&(w, h)) = caixas.get(&vi.geometry_id) else {
            continue;
        };
        if lado_no_ecra(crate::motion_shape_gen::instance_pose(vi, cam), w, h) > LADO_MAXIMO_PX {
            fora.insert(vi.geometry_id);
            if fora.len() == caixas.len() {
                break; // já não há candidata nenhuma — nada mais a medir
            }
        }
    }
    caixas
        .into_keys()
        .filter(|gid| !fora.contains(gid))
        .collect()
}

/// **A partição** — move para `instances`, como quads de tile, toda instância cuja geometria o LOD
/// quer **e que tenha tile assada**. Devolve quantas moveu.
///
/// ⚠️ **A cerca `sem tile fica crisp` é a mesma do irmão, e pela mesma razão**: o assado corre na
/// fase de fx e só termina no quadro seguinte, logo no primeiro quadro em que o LOD quer uma
/// geometria a tile ainda não existe. *Uma tile em falta não pode apagar a forma.*
///
/// ⭐⭐⭐ **E A COR SAI CERTA POR CONSTRUÇÃO, que é o que torna a troca fiel.** Um primitivo do
/// `source.shape` **não tem tinta própria** (`path.fill` é `None` — é o que faz o
/// `prepare_primitive` recusá-lo ao caminho do documento): a cor dele é o `tint` da INSTÂNCIA. E o
/// assador desenha a forma com [`ph2d_vec_render::draw_path_standalone`], que passa
/// `[1,0, 1,0, 1,0, 1,0]` ⇒ **a tile é a silhueta em BRANCO**, e o quad multiplica-a pelo `tint`
/// que o [`tile_quad`] preserva. ⚠️ Sem essa composição o LOD trocaria a cor de toda cópia, e o
/// sintoma seria «as estrelas ficaram brancas ao afastar».
///
/// ⚠️ **O quad sai do [`tile_quad`] e não do `vector_instance_as_tile`**, e essa é a diferença que
/// separa isto de um halo torto: a caixa de uma forma paramétrica **não** é centrada na origem
/// local dela, e o `tile_quad` desloca o centro passando pela BASE da instância. O irmão pode usar
/// o ingénuo porque lá o publicador escreve o tamanho de mundo assado no `size`.
pub fn aplica_lod_de_forma(
    instances: &mut Vec<RenderInstance>,
    vector_instances: &mut Vec<VectorInstance>,
    shape_bake: &ShapeBake,
    quer: &std::collections::BTreeSet<u32>,
) -> usize {
    if quer.is_empty() {
        return 0;
    }
    let antes = instances.len();
    vector_instances.retain(|vi| {
        if !quer.contains(&vi.geometry_id) {
            return true;
        }
        match shape_bake.tile_for_gid(vi.geometry_id) {
            Some(tile) => {
                instances.push(tile_quad(vi, tile));
                false
            }
            None => true, // ainda sem tile — fica crisp, e o assador pede-a
        }
    });
    instances.len() - antes
}

#[cfg(test)]
#[path = "motion_shape_lod_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O CONJUNTO VIVO, depois de o LOD ter levado cópias** — o que o despejo de tiles
/// ([`ShapeBake::evict_outside`]) tem de ver.
///
/// ⛔⛔ **Sem isto o despejo larga a tile que os quads estão a usar, e o modo de falha é o pior
/// que este assador já teve.** O `live` de sempre são os `geometry_id` das `vector_instances`; uma
/// geometria que o LOD moveu **já não está lá** — ela está em `instances`, como quad. E com a cena
/// PARADA o cozimento devolve cedo (`if !self.dirty && self.last_cooked_tick == Some(tick)`,
/// *«paused + unchanged → reuse the buffer»*), logo as duas listas persistem e o `live` ficaria
/// errado **para sempre**: a tile era largada, o `texture_id` do quad passava a apontar para um
/// slot livre, e o artista via a forma desaparecer ou piscar.
///
/// ⚠️ **A ligação é o `texture_id`, e é a única que sobrevive à conversão**: um quad não carrega
/// `geometry_id` nenhum ([`RenderInstance`] não tem esse campo), logo a pergunta *«esta tile ainda
/// é desenhada?»* só se responde pelo lado da textura.
///
/// ⚠️ E ela é a **UNIÃO**, nunca a substituição: uma geometria pode ter cópias dos dois lados (o
/// LOD decide por geometria, mas uma cena com duas grelhas da mesma forma a zooms diferentes é
/// exprimível), e despejar por um dos lados largaria uma tile ainda em cena.
pub fn vivas_com_o_lod(
    vector_instances: &[VectorInstance],
    instances: &[RenderInstance],
    shape_bake: &ShapeBake,
) -> std::collections::BTreeSet<u32> {
    let mut vivas: std::collections::BTreeSet<u32> =
        vector_instances.iter().map(|vi| vi.geometry_id).collect();
    // As texturas que os quads deste quadro amostram — o outro lado da conversão.
    let em_uso: std::collections::BTreeSet<u32> = instances.iter().map(|i| i.texture_id).collect();
    if em_uso.is_empty() {
        return vivas;
    }
    for gid in shape_bake.gids() {
        if shape_bake
            .tile_for_gid(gid)
            .is_some_and(|t| em_uso.contains(&t.texture_id))
        {
            vivas.insert(gid);
        }
    }
    vivas
}
