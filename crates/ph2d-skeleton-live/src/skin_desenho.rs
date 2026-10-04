//! ⭐⭐⭐ **O DESENHO DA FORMA PRESA — os pontos do artista ficam, e o que se VÊ é a curva fiel.**
//!
//! Ordem do dono (2026-09-29): *«Nossa deformação de imagens não vetoriais está muito boa. Cuidado
//! para não estragar. Hoje nosso problema é o uso de osso com desenho vetorial. Siga como achar
//! melhor, buscando o padrão ouro.»*
//!
//! # O que estava errado, medido
//!
//! Desde que o `Bind` deixou de acrescentar pontos (ordem do dono de 2026-09-20), a forma presa
//! chega ao quadro com os nós que o artista desenhou — `8` na barra da cena. Uma cúbica por
//! segmento não segue uma dobra forte, e com `8` nós **`97,8 %`** do desvio é o MODELO e não o
//! procedimento (`docs/Skeleton/04_pesquisa_ossos_sobre_desenho_vetorial.md` §1.2): nenhum ajuste
//! de alças o cura. Barra da cena a `90°` em S, contra o padrão-ouro (a lei aplicada ponto a ponto,
//! que é o que a IMAGEM presa faz):
//!
//! | desenho | nós | desvio p90 | desvio máx |
//! |---|---:|---:|---:|
//! | o de ontem (a lei nos `8` nós) | `8` | `0,22281` | `0,45972` (`6,5 %` da diagonal) |
//! | **este** (o bake sobre os `8` nós) | `~45` | **`0,00165`** | **`0,00456`** |
//!
//! # A lei desta folha
//!
//! ⭐⭐⭐ ***O documento guarda os pontos do artista; o desenho mostra a curva fiel.*** Duas saídas
//! por forma e por quadro, e nenhuma substitui a outra:
//!
//! - **a CRUA** — a lei nos nós do artista (a de sempre), escrita no caminho VIVO da cena. É ali
//!   que o modo Node edita, que o ponto novo é inserido ([`crate::ponto_novo`]) e que o `Release`
//!   devolve. ⛔ Pôr ali os `~45` nós do bake entregaria ao artista nós que ele não desenhou.
//! - **a DESENHADA** — o [`ph2d_vec_skin::curva::assa_a_pele`] sobre os mesmos nós, entregue à
//!   geometria viva do quadro (`LiveGeometry`), que o desenho e o PICK lêem no z da forma. É a
//!   porta por onde o Offset vivo, a simetria e a largura viva já mostram o que não guardam.
//!
//! # A cache, e por que ela é a metade barata
//!
//! O que se deriva do BIND (os bytes lidos da fonte e o índice da malha do campo, `44 µs`) era
//! refeito por quadro por forma. E quando nada se mexeu — nem a pose, nem o bind, nem as leis —, o
//! quadro inteiro é o do anterior. ⇒ **uma gaveta por forma**, com a mesma disciplina do memo da
//! imagem ([`crate::skin_bake_cache`]): a ENTIDADE é o endereço e o CONTEÚDO é a prova — um bind,
//! uma pele (as poses) e as leis iguais aos da gaveta devolvem o que ela guardou; qualquer
//! diferença refaz. *Bits de entidade reciclados dão uma comparação falhada, nunca o desenho de
//! outra forma.*
//!
//! # ⛔ O que NÃO é assado, e porquê
//!
//! [`estilo_de`]: uma forma com **offset de CAD numa camada** continua a desenhar-se como ontem — o
//! offset de camada é indexado pelo id da fonte numa rota que a geometria viva não leva. *Uma
//! forma que ficou como era é melhor do que uma que mudou de natureza sem aviso.*
//!
//! # ⭐⭐⭐ As QUINAS VIVAS e os EFEITOS correm no REPOUSO, e o desenho deles dobra
//!
//! O bake re-escreve os nós, e uma quina viva ou um efeito correriam sobre os `~45` do assado (um
//! *Zig Zag* sobre `45` nós é outro desenho). ⇒ os dois são COZIDOS sobre a fonte EM REPOUSO, a
//! tabela de pesos dos nós novos sai do campo do domínio, e o assado percorre o cozido
//! ([`cozido_para_o_bake`], [`efeitos::cozido_com_efeitos`]) — é o que a IMAGEM presa faz com a arte dela.
//!
//! ⛔ **A lei antiga** (a pele nos `8` nós e o efeito DEPOIS, sobre a forma dobrada) media o efeito
//! na CAIXA da pose: nove dos dez efeitos lêem o `FxCtx` (caixa, centro, `ref_size`), e um *Warp*,
//! um *Twist* ou o espaçamento de um *Hatch* mudavam de tamanho quando o braço dobrava. Oráculo
//! CORRIDO (Blender 5.2, `GREASE_PENCIL_ARMATURE` com `parent_set(ARMATURE_AUTO)`): um modificador
//! que já existe quando se prende fica ANTES do `Armature` na pilha — corre em repouso e dobra.
//! `PH2D_SKIN_EFEITOS=0` volta à lei antiga, e é por onde se bissecta um report.
//!
//! ⭐⭐⭐ **E o DOMÍNIO do campo é o contorno COZIDO** (F50-d, report do dono de 2026-10-03): a cauda
//! de um *Twist* sai da barra, e fora do domínio da fonte cada ponto herdava o vértice mais próximo
//! — o ideal esticava `17`–`191×` entre amostras vizinhas (um rasgo). Sobre o cozido, `≤ 3,6×`. É o
//! *Puppet* do After Effects: a malha tira-se do que a camada DESENHA. Preço: um solver por pilha
//! nova (`20`–`100 ms` medidos a `load 53`), nunca por pose.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ph2d_skeleton::Skin;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_skin_weights::Handle;
use ph2d_vec_scene::effect::FxEntry;
use ph2d_vec_scene::{VecPath, VecPathId, VecXforms};
use ph2d_vec_skin::curva::{Bake, CampoIndexado};
use ph2d_vec_skin::pesos::{CampoDoDominio, IndiceDoCampo};

use crate::skinned_mesh::SkinnedPath;

/// O que um quadro de pele entrega à GEOMETRIA VIVA: o desenho fiel de cada forma presa que o
/// pode ter, em coordenadas LOCAIS do caminho (quem o põe no mundo é a [`funde`]).
pub type SkinDesenhado = BTreeMap<VecPathId, VecPath>;

/// ⭐⭐⭐ **Quantas amostras a forma INTEIRA recebe** — o orçamento que a amostragem reparte pelos
/// segmentos.
///
/// ⚠️ **Não é por segmento, e está medido:** a amostragem óptima sobre os `8` nós da barra é `64`
/// por segmento (`512` no total), e sobre os `54` nós de um ficheiro gravado entre 19 e 20 de
/// Setembro é `16` por segmento — `64` ali custaria `4×` sem ganho. O que as duas têm em comum é o
/// TOTAL, porque o que se amostra é a mesma curva. ⚠️ E amostrar MAIS piora (`128` por segmento
/// sobre `8` nós: máx `0,00456 → 0,02191`) — a amostragem passa a resolver os bicos que a malha
/// linear do campo deixa em cada aresta, em vez de os alisar. *Existe uma amostragem óptima.*
pub const AMOSTRAS_POR_FORMA: usize = 512;

/// O piso e o tecto da repartição — os extremos medidos da tabela (`16`/`64` por segmento).
pub const AMOSTRAS_POR_SEGMENTO: (usize, usize) = (16, 64);

/// ⭐ **A tolerância do ajuste, em fracção da DIAGONAL da forma** — `0,03 %`, a coluna da tabela
/// que passa por baixo do chão do modelo. ⚠️ Fracção e não comprimento: a mesma forma a outra
/// escala é o mesmo desenho, e um número absoluto mediria o tamanho.
pub const TOLERANCIA_DA_DIAGONAL: f64 = 3e-4;

/// As amostras de cada segmento para um contorno de `segs` segmentos — ver [`AMOSTRAS_POR_FORMA`].
#[must_use]
pub fn amostras_por_segmento(segs: usize) -> usize {
    let (piso, tecto) = AMOSTRAS_POR_SEGMENTO;
    AMOSTRAS_POR_FORMA.div_ceil(segs.max(1)).clamp(piso, tecto)
}

/// ⭐⭐ **O que o ESTILO do caminho VIVO diz ao bake** — os efeitos e as camadas moram nele, e o
/// recook não os reescreve.
#[derive(Clone, Debug, PartialEq)]
pub enum Estilo {
    /// Sem efeito activo nem offset de camada: o bake percorre a fonte.
    Serve,
    /// A pilha de efeitos ACTIVOS — o bake percorre a fonte cozida com eles em repouso.
    Efeitos(Vec<FxEntry>),
    /// Offset de CAD numa camada: desenha-se como ontem (ver o cabeçalho).
    NaoServe,
}

/// O [`Estilo`] do caminho VIVO.
#[must_use]
pub fn estilo_de(viva: &VecPath) -> Estilo {
    if viva.paints.iter().any(|e| e.dilate != 0.0) {
        return Estilo::NaoServe;
    }
    let activos: Vec<FxEntry> = viva
        .effects
        .iter()
        .filter(|e| e.is_active())
        .cloned()
        .collect();
    if activos.is_empty() {
        Estilo::Serve
    } else {
        Estilo::Efeitos(activos)
    }
}

/// ⭐⭐ **Os NÓS desta forma deixam-na ser desenhada pelo bake?** — as quinas vivas moram nos nós
/// da FONTE, e o bake re-escreve os nós. A outra metade é o [`estilo_de`].
#[must_use]
pub fn os_nos_servem(fonte: &VecPath) -> bool {
    !fonte.has_live_corner()
}

/// As leis que um quadro de pele lê — parte da CHAVE da gaveta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leis {
    /// A lei da curva (o ajuste das alças) — `PH2D_SKIN_CURVE=0` desliga.
    pub curva: bool,
    /// A mistura rígida (o produto) ou linear (o controlo dos gates).
    pub rigido: bool,
    /// O campo do domínio — `PH2D_SKIN_CAMPO=0` desliga.
    pub campo: bool,
    /// A leitura `C¹` do campo — `PH2D_SKIN_C1=1` liga.
    pub c1: bool,
    /// ⭐ O desenho fiel — `PH2D_SKIN_DESENHO=0` volta ao desenho nos nós do artista, e é por onde
    /// se bissecta um report.
    pub desenho: bool,
    /// ⭐⭐⭐ **O CONTACTO de uma dobra forte** — o desenho fiel que se sobrepõe a si mesmo sai como a
    /// SILHUETA dos membros ([`ph2d_vec_boolean::resolve_overlap`]). `PH2D_SKIN_CONTACTO=0` volta ao
    /// contorno com o «olho» por dentro, e é por onde se bissecta um report.
    pub contacto: bool,
    /// ⭐⭐⭐ **Os EFEITOS correm no repouso e o desenho deles dobra** — ver o cabeçalho.
    /// `PH2D_SKIN_EFEITOS=0` volta à lei antiga (a pele nos nós e o efeito sobre a forma dobrada).
    pub efeitos: bool,
}

impl Leis {
    /// As leis do produto, lidas do ambiente — **uma vez por quadro**, por quem chama.
    #[must_use]
    pub fn do_ambiente() -> Self {
        Self {
            curva: ph2d_vec_skin::curva::lei_da_curva_activa(),
            rigido: true,
            campo: ph2d_vec_skin::curva::lei_do_campo_activa(),
            c1: ph2d_vec_skin::curva::lei_c1_activa(),
            desenho: lei_do_desenho_activa(),
            contacto: lei_do_contacto_activa(),
            efeitos: std::env::var("PH2D_SKIN_EFEITOS").as_deref() != Ok("0"),
        }
    }
}

/// `PH2D_SKIN_DESENHO=0` desliga o desenho fiel — ver [`Leis::desenho`].
#[must_use]
pub fn lei_do_desenho_activa() -> bool {
    std::env::var("PH2D_SKIN_DESENHO").as_deref() != Ok("0")
}

/// `PH2D_SKIN_CONTACTO=0` desliga a silhueta do contacto — ver [`Leis::contacto`].
#[must_use]
pub fn lei_do_contacto_activa() -> bool {
    contacto_de(std::env::var("PH2D_SKIN_CONTACTO").ok().as_deref())
}

/// A leitura da porta, PURA — ligada salvo `"0"`. ⚠️ É ela que o gate mede: um gate que lesse o
/// ambiente mediria a máquina onde corre.
#[must_use]
pub fn contacto_de(valor: Option<&str>) -> bool {
    valor != Some("0")
}

/// O que se deriva do BIND e não do quadro: a fonte lida e o índice da malha do campo dela.
pub struct Preparado {
    /// A fonte, lida dos bytes do bind.
    pub guardado: SkinnedPath,
    /// O índice da malha do campo — `None` sem campo.
    pub indice: Option<IndiceDoCampo>,
    /// ⭐⭐ **A fonte com as QUINAS VIVAS já arredondadas, e a tabela de pesos dela** — o que o bake
    /// percorre quando a fonte tem raio de quina. `None` sem quinas, ou sem campo para amostrar a
    /// tabela (ver [`cozido_para_o_bake`]).
    pub cozido: Option<(VecPath, Vec<f64>)>,
    /// ⭐⭐ **Numa fonte de EFEITOS COZIDOS no Bind: a união do contacto é neutra em repouso?**
    /// (`SkinnedPath::efeitos_cozidos`) — `None` numa fonte do artista, que é da bola.
    pub uniao_neutra: Option<bool>,
}

/// O que um quadro produziu para uma forma: o caminho CRU e, quando serve, o DESENHADO.
#[derive(Clone)]
pub struct Quadro {
    /// A lei nos nós do artista — vai para o caminho vivo da cena.
    pub cru: VecPath,
    /// O bake — vai para a geometria viva, quando a forma o pode ter.
    pub desenhado: Option<VecPath>,
}

/// ⭐⭐⭐ **A fonte cozida com os EFEITOS em repouso, e a tabela de pesos dela** — derivada da
/// PILHA viva (o artista muda um efeito depois do `Bind`), e guardada na gaveta enquanto a pilha
/// for a mesma. Ver [`efeitos::cozido_com_efeitos`].
pub struct CozidoFx {
    pilha: Vec<FxEntry>,
    caminho: VecPath,
    tabela: Vec<f64>,
    /// ⭐⭐⭐ **O campo do domínio resolvido sobre o CONTORNO COZIDO** (e o índice dele) — o que se
    /// vê, pontas do efeito incluídas, é o domínio, como o *Puppet* do After Effects tira a malha
    /// do que a camada desenha. `None` quando o solver não responde (um contorno que se cruza, o
    /// *Repeat* sobreposto): aí vale o campo da fonte.
    campo: Option<Rc<CampoFx>>,
    /// ⭐⭐ **A união do contacto é NEUTRA no repouso deste cozido?** Um efeito pode desenhar
    /// contornos que se cruzam DE PROPÓSITO (as cópias de um *Repeat*, a agulha de um *Bloat*
    /// forte), e a união reescrevê-los-ia já em repouso. ⛔ *Uma lei de contacto que muda o
    /// repouso não é de contacto* ⇒ ali o desenho sai sem ela.
    contacto: bool,
}

/// O campo do contorno cozido e o índice dele.
pub type CampoFx = (CampoDoDominio, Option<IndiceDoCampo>);

/// O último campo RESOLVIDO para esta forma: a pilha para que foi pedido e a resposta do solver
/// (`None` quando ele não respondeu — vale o campo da fonte).
type Resolvido = (Vec<FxEntry>, Option<Rc<CampoFx>>);

struct Ultimo {
    pele: Skin,
    leis: Leis,
    estilo: Estilo,
    /// O cozido com que este quadro foi calculado — a chegada de um campo novo muda-o sem mudar
    /// pose, leis nem estilo.
    fx: Option<Rc<CozidoFx>>,
    quadro: Quadro,
}

struct Gaveta {
    bind: SkinBind,
    preparado: Option<Rc<Preparado>>,
    efeitos: Option<Rc<CozidoFx>>,
    resolvido: Option<Resolvido>,
    a_caminho: Option<(Vec<FxEntry>, std::sync::mpsc::Receiver<Option<CampoFx>>)>,
    ultimo: Option<Ultimo>,
    visto: u64,
}

/// ⚠️ **Quantas formas presas o memo guarda** — o recurso é MEMÓRIA: uma gaveta leva os bytes do
/// bind (a fonte e o campo, `~16 KB` na barra) mais dois caminhos. `256` gavetas são `~5 MiB` no
/// pior caso. ⛔ Não é um tecto de quantas formas o produto prende: passar dele custa refazer a
/// menos usada, nunca um desenho errado.
const GAVETAS_MAX: usize = 256;

thread_local! {
    static MEMO: RefCell<BTreeMap<u64, Gaveta>> = const { RefCell::new(BTreeMap::new()) };
    static RELOGIO: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// ⚠️ Contadores de TESTE — quantas vezes o índice e o quadro foram DERIVADOS. Só assim um gate
    /// vê a cache: ela não muda a resposta, só quantas vezes ela é calculada.
    #[cfg(test)]
    static DERIVADOS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// `(índices construídos, quadros calculados)` nesta thread — ver `DERIVADOS`.
#[cfg(test)]
pub(crate) fn derivados() -> (usize, usize) {
    DERIVADOS.with(std::cell::Cell::get)
}

/// ⭐⭐⭐ **O QUADRO DE UMA FORMA PRESA** — da gaveta quando nada mudou, calculado quando mudou.
///
/// `bits` é o endereço (a entidade da forma); `skin`, `pele` e `leis` são a prova. `estilo` é o
/// [`estilo_de`] do caminho VIVO, que só quem chama tem na mão; `eixos` são os dos ossos no
/// repouso do bind ([`crate::skin_live::eixos_do_bind`]) — só lidos quando a pilha muda.
///
/// `None` quando a fonte não se lê — o chamador salta a forma, como sempre saltou.
#[must_use]
pub fn quadro(
    bits: u64,
    skin: &SkinBind,
    pele: &Skin,
    leis: Leis,
    estilo: &Estilo,
    eixos: &dyn Fn() -> Vec<Handle>,
) -> Option<Quadro> {
    com_a_gaveta(bits, skin, |g| {
        let prep = Rc::clone(g.preparado.as_ref()?);
        let fx = match estilo {
            Estilo::Efeitos(pilha) if leis.efeitos && leis.desenho => {
                efeitos_da_gaveta(g, &prep.guardado, pilha, eixos)
            }
            _ => None,
        };
        if let Some(u) = &g.ultimo
            && u.pele == *pele
            && u.leis == leis
            && u.estilo == *estilo
            && match (&u.fx, &fx) {
                (None, None) => true,
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                _ => false,
            }
        {
            return Some(u.quadro.clone());
        }
        // ⚠️ Uma forma com efeito e sem cozido (sem campo, ou a lei desligada) desenha-se como
        // ontem: o efeito sobre os nós deformados.
        let serve = match estilo {
            Estilo::Serve => true,
            Estilo::Efeitos(_) => fx.is_some(),
            Estilo::NaoServe => false,
        };
        let q = calcula(&prep, skin, pele, leis, serve, fx.as_deref());
        g.ultimo = Some(Ultimo {
            pele: pele.clone(),
            leis,
            estilo: estilo.clone(),
            fx,
            quadro: q.clone(),
        });
        Some(q)
    })
}

/// ⭐⭐ **A FONTE LIDA desta forma presa, da gaveta** — para quem lê a fonte e não o quadro (o
/// indicador de pesos lia-a QUATRO vezes por forma por quadro, cada uma a descodificar os bytes
/// do bind inteiros).
///
/// `None` quando a fonte não é um caminho (uma imagem presa guarda outro formato) — a mesma
/// resposta que o [`crate::skinned_mesh::le`] dá, e por isso uma troca directa por ele.
#[must_use]
pub fn lida(bits: u64, skin: &SkinBind) -> Option<Rc<Preparado>> {
    com_a_gaveta(bits, skin, |g| g.preparado.clone())
}

/// A gaveta desta forma, fresca contra o `skin` — a porta única das duas leituras acima.
fn com_a_gaveta<R>(
    bits: u64,
    skin: &SkinBind,
    f: impl FnOnce(&mut Gaveta) -> Option<R>,
) -> Option<R> {
    let agora = RELOGIO.with(|r| {
        let t = r.get() + 1;
        r.set(t);
        t
    });
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        // ⚠️ **Duas provas e não uma:** o que se PREPARA depende só dos bytes da fonte, e o que se
        // CALCULA depende do bind inteiro (as correcções à mão, a `SkinLaw`). Pintar uma correcção
        // muda o segundo e não o primeiro — e refazer o índice por cada pincelada seria pagar o
        // `IndiceDoCampo::novo` que esta folha existe para não pagar.
        let fresca = match m.get_mut(&bits) {
            Some(g) if g.bind.source == skin.source => {
                if g.bind != *skin {
                    g.bind = skin.clone();
                    g.ultimo = None;
                }
                true
            }
            _ => false,
        };
        if !fresca {
            if m.len() >= GAVETAS_MAX && !m.contains_key(&bits) {
                let velha = m.iter().min_by_key(|(_, g)| g.visto).map(|(k, _)| *k);
                if let Some(k) = velha {
                    m.remove(&k);
                }
            }
            let preparado = prepara(&skin.source).map(Rc::new);
            m.insert(
                bits,
                Gaveta {
                    bind: skin.clone(),
                    preparado,
                    efeitos: None,
                    resolvido: None,
                    a_caminho: None,
                    ultimo: None,
                    visto: agora,
                },
            );
        }
        let g = m.get_mut(&bits)?;
        g.visto = agora;
        f(g)
    })
}

/// Lê a fonte e indexa o campo dela — o que só muda num novo `Bind`.
fn prepara(fonte: &[u8]) -> Option<Preparado> {
    let guardado = crate::skinned_mesh::le(fonte)?;
    #[cfg(test)]
    DERIVADOS.with(|d| {
        let (i, q) = d.get();
        d.set((i + 1, q));
    });
    let indice = guardado
        .campo
        .as_ref()
        .and_then(|c| IndiceDoCampo::novo(&c.malha));
    let cozido = cozido_para_o_bake(&guardado);
    let uniao_neutra = guardado
        .efeitos_cozidos
        .then(|| !ph2d_vec_boolean::overlaps_itself(&so_os_fechados(&guardado.path)));
    Some(Preparado {
        guardado,
        indice,
        cozido,
        uniao_neutra,
    })
}

/// ⭐⭐⭐ **A fonte com as QUINAS VIVAS arredondadas no REPOUSO, e a tabela de pesos dela.**
///
/// ⚠️ **Existe porque a barra da cena do dono TEM quinas vivas** (raio `0,5`) — a 1.ª redacção
/// desta folha deixava toda forma com raio fora do bake, e o artista não veria diferença nenhuma
/// exactamente na peça que usa. O bake re-escreve os nós (e a quina é estado do nó), logo o que
/// se percorre é a forma JÁ arredondada — que é o que a IMAGEM presa faz: a quina é parte do
/// desenho em repouso, e dobra com ele. (Sem o bake a quina é arredondada DEPOIS da deformação,
/// sobre os nós deformados.)
///
/// ⭐ **A tabela sai da mesma porta que o `Bind` usa** ([`ph2d_vec_skin::pesos::pesos_dos_pontos`]),
/// e o domínio do campo é construído sobre os contornos COZIDOS — logo o contorno arredondado cai
/// sobre a fronteira da malha, e a leitura do campo é a do padrão-ouro. Corre UMA vez por fonte.
///
/// ⛔ Sem campo (um bind anterior a 2026-09-20) não há de onde amostrar a tabela dos pontos novos
/// ⇒ `None`, e essa forma desenha-se como antes. Os efeitos da fotografia NÃO correm aqui: a pilha
/// que vale é a VIVA ([`estilo_de`]), e é a [`efeitos::cozido_com_efeitos`] que a coze.
fn cozido_para_o_bake(g: &SkinnedPath) -> Option<(VecPath, Vec<f64>)> {
    if !g.path.has_live_corner() {
        return None;
    }
    let campo = g.campo.as_ref()?;
    let mut so_quinas = g.path.clone();
    so_quinas.effects.clear();
    let cozido = so_quinas.cooked().into_owned();
    let tabela = ph2d_vec_skin::pesos::pesos_dos_pontos(&cozido, campo);
    Some((cozido, tabela))
}

/// ⭐⭐⭐ **Os EFEITOS de uma forma presa**, num irmão pelo tecto de LOC.
#[path = "skin_desenho_efeitos.rs"]
mod efeitos;
#[cfg(test)]
pub(crate) use efeitos::solver_em_fundo_no_teste;
use efeitos::{efeitos_da_gaveta, so_os_fechados, uniao_dos_fechados};


/// A lei sobre a fonte preparada — o corpo que o [`crate::skin_live`] corria por forma.
fn calcula(
    prep: &Preparado,
    skin: &SkinBind,
    pele: &Skin,
    leis: Leis,
    estilo_serve: bool,
    fx: Option<&CozidoFx>,
) -> Quadro {
    #[cfg(test)]
    DERIVADOS.with(|d| {
        let (i, q) = d.get();
        d.set((i, q + 1));
    });
    let guardado = &prep.guardado;
    // ⛔ Uma tabela que não fecha com o caminho cai na lei derivada em vez de ser lida
    // deslocada — pesos plausíveis sobre os pontos errados dão arte errada sem um erro. ⭐ E a
    // ESCOLHA DO ARTISTA (`SkinLaw`) passa pela mesma porta: ver [`SkinBind::pesos_do_quadro`].
    let fecha: &[f64] = if guardado.valida() {
        &guardado.pesos
    } else {
        &[]
    };
    let pesos = skin.pesos_do_quadro(fecha);
    let correcoes = skin.correcoes_resolvidas();
    // ⭐⭐⭐ O campo do domínio, quando o bind o guardou; `None` num bind anterior a 2026-09-20.
    let campo = leis.campo.then_some(guardado.campo.as_ref()).flatten();
    // ⚠️ A leitura `C¹` é derivada POR QUADRO e não guardada: ela empresta o campo, e a porta dela
    // está fechada por omissão (`PH2D_SKIN_C1=1` abre). *Guardar uma porta fechada não se paga.*
    let suave = campo
        .filter(|_| leis.c1)
        .and_then(ph2d_vec_skin::pesos_suave::CampoSuave::novo);
    let lido = CampoIndexado {
        campo,
        indice: campo.and(prep.indice.as_ref()),
        suave: suave.as_ref(),
    };
    // ⭐⭐⭐ O bake de uma forma com efeito lê o campo do CONTORNO COZIDO ([`CozidoFx::campo`]).
    let campo_fx = fx.and_then(|c| c.campo.as_deref()).filter(|_| leis.campo);
    let suave_fx = campo_fx
        .filter(|_| leis.c1)
        .and_then(|(c, _)| ph2d_vec_skin::pesos_suave::CampoSuave::novo(c));
    let lido_do_bake = match campo_fx {
        Some((c, i)) => CampoIndexado {
            campo: Some(c),
            indice: i.as_ref(),
            suave: suave_fx.as_ref(),
        },
        None => lido,
    };
    let mut cru = guardado.path.clone();
    if leis.curva {
        ph2d_vec_skin::curva::aplica_pela_curva_indexada(
            pele,
            &mut cru,
            pesos,
            &correcoes,
            leis.rigido,
            lido,
        );
    } else {
        ph2d_vec_skin::aplica_corrigido_com(pele, &mut cru, pesos, &correcoes, leis.rigido);
    }
    // ⭐⭐ O que o bake percorre: a fonte, ou — com efeitos ou quinas vivas — a fonte já cozida em
    // repouso, com a tabela dela a passar pela MESMA porta da escolha do artista
    // (`pesos_do_quadro`).
    let percurso: Option<(&VecPath, &[f64])> = if let Some(c) = fx {
        Some((&c.caminho, skin.pesos_do_quadro(&c.tabela)))
    } else if os_nos_servem(&guardado.path) {
        Some((&guardado.path, pesos))
    } else {
        prep.cozido
            .as_ref()
            .map(|(c, t)| (c, skin.pesos_do_quadro(t)))
    };
    let desenhado = percurso
        .filter(|_| leis.desenho && estilo_serve)
        .map(|(fonte, tabela)| {
            let (d, nos) = ph2d_vec_skin::curva::assa_a_pele_com_nos(
                pele,
                fonte,
                tabela,
                &correcoes,
                leis.rigido,
                lido_do_bake,
                Bake {
                    amostras: amostras_por_segmento(segmentos(fonte)),
                    tolerancia: TOLERANCIA_DA_DIAGONAL * diagonal(fonte),
                },
            );
            (d, quinas_do_artista(fonte, nos))
        })
        // ⭐⭐⭐ **O CONTACTO.** Numa dobra forte a face de DENTRO de dois membros passa uma por cima
        // da outra — é geometria de dois pedaços rígidos que rodam em torno de uma junta, não erro
        // da lei — e o TRAÇO desenharia o «olho» da sobreposição por dentro. A silhueta é a
        // fronteira da UNIÃO dos membros, que é o que o estado da arte põe no contacto (Implicit
        // Skinning, Vaillant 2013) e o que a IMAGEM presa já mostra de graça (um membro por cima
        // do outro, canto em «V»). ⚠️ **Desde a F41 a porta corre SEMPRE**: a união só quando o
        // contorno se CRUZA, e a bola que arredonda o vinco em todo ângulo, antes e depois do
        // encosto; sem vinco apertado a forma sai ao bit. ⛔ Só no DESENHADO — o
        // `cru` são os nós que o artista edita, e trocá-los pela silhueta mudar-lhe-ia a malha.
        .map(|(d, quinas)| {
            // ⭐⭐ **Numa forma com EFEITO, só a UNIÃO** — e só quando ela é neutra no repouso. A
            // bola arredonda o vinco do contorno do ARTISTA, e as cristas e pontas de um efeito
            // não o são (`3,5`–`7 ms` no *Zig Zag*). ⚠️ A F50-d tirou-a de todo (report do dono
            // de 2026-10-03: pedaços de traço soltos, serrilha): a serrilha era a laçada do ajuste
            // (F50-e) e os riscos soltos as LASCAS da união (F50-f) — curadas as duas, a união
            // volta, e o traço deixa de se cruzar por dentro de uma dobra forte.
            // ⭐⭐ E o mesmo numa fonte de efeitos COZIDOS no Bind (2026-10-03): a bola comia os
            // dentes de um *Zig Zag* do lado de dentro da junta (FOTOGRAFADO a `60°`).
            let neutra = fx.map(|c| c.contacto).or(prep.uniao_neutra);
            if !leis.contacto {
                d
            } else if let Some(neutra) = neutra {
                if neutra {
                    uniao_dos_fechados(&d).unwrap_or(d)
                } else {
                    d
                }
            } else {
                ph2d_vec_boolean::silhueta_da_pele(&d, &quinas).unwrap_or(d)
            }
        });
    Quadro { cru, desenhado }
}

/// ⭐⭐ **As QUINAS DO ARTISTA do assado** — cada nó da `fonte` onde o assado o pousou, com a viragem
/// que ele tem EM REPOUSO (F42, report do dono de 2026-09-30).
///
/// ⛔ A viragem lida no desenho DEFORMADO não serve: na dobra do mapa um nó do assado vira `180°` sem
/// que ninguém tenha desenhado quina nenhuma, a bola protegia-o como parede, e o traço sobre a
/// meia-volta abria fatias de cinzento e laranja no vinco. Um nó que a fonte não tem (os do ajuste)
/// fica fora da lista e é da bola.
fn quinas_do_artista(fonte: &VecPath, nos: Vec<[f64; 2]>) -> Vec<([f64; 2], f64)> {
    let repouso = ph2d_vec_boolean::quinas_de(fonte);
    debug_assert_eq!(repouso.len(), nos.len(), "um nó assado por nó da fonte");
    nos.into_iter()
        .zip(repouso)
        .map(|(no, (_, vira))| (no, vira))
        .collect()
}

/// Quantos segmentos a forma tem, somados os contornos.
///
/// ⭐⭐ **Só os contornos FECHADOS contam, quando os há** (F50-i, foto do dono de 2026-10-03:
/// *«Hatch linhas saindo da forma»*). O orçamento [`AMOSTRAS_POR_FORMA`] é o de UMA curva — o
/// contorno —, e as `58` riscas abertas de um *Hatch* repartiam-no: o contorno ficava com `16`
/// amostras por segmento e cortava caminho na dobra (`0,034`/`0,067`/`0,12` do ideal a
/// `60°`/`90°`/`120°`), com as pontas das riscas, exactas, a sobrar por fora dele.
fn segmentos(p: &VecPath) -> usize {
    let conta = |so_fechados: bool| -> usize {
        (0..p.contour_count())
            .filter_map(|c| p.contour(c))
            .filter(|(_, fechado)| *fechado || !so_fechados)
            .map(|(v, fechado)| {
                if fechado {
                    v.len()
                } else {
                    v.len().saturating_sub(1)
                }
            })
            .sum()
    };
    match conta(true) {
        0 => conta(false),
        n => n,
    }
}

/// A diagonal da caixa das âncoras e alças — a régua da tolerância.
fn diagonal(p: &VecPath) -> f64 {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for v in p.verts_all() {
        for q in [v.anchor, v.in_handle, v.out_handle] {
            x0 = x0.min(q[0]);
            y0 = y0.min(q[1]);
            x1 = x1.max(q[0]);
            y1 = y1.max(q[1]);
        }
    }
    if x1 < x0 {
        return 0.0;
    }
    (x1 - x0).hypot(y1 - y0)
}

/// ⭐⭐ **O desenho fiel no MUNDO, dentro da geometria viva do quadro.**
///
/// `vivo` é o mapa que o desenho e o PICK lêem (`ph2d_vec_render::LiveGeometry`, o mesmo tipo
/// escrito por extenso para esta folha não depender da crate de desenho).
///
/// ⚠️ **Não ESCREVE por cima de outro produtor** — uma forma presa com Offset vivo, largura viva,
/// simetria, padrão ou contorno continua a mostrar o que aquele produtor cozeu dela (sobre os nós
/// do artista, como ontem). ⛔ Os dois juntos seriam uma escolha sem dono; o primeiro a escrever
/// ganha, e os outros produtores escrevem ANTES desta chamada.
pub fn funde(
    desenho: &SkinDesenhado,
    xforms: &VecXforms,
    vivo: &mut BTreeMap<VecPathId, Vec<VecPath>>,
) {
    for (id, p) in desenho {
        vivo.entry(*id).or_insert_with(|| {
            let mut mundo = p.clone();
            ph2d_vec_scene::bake_xform(&mut mundo, &ph2d_vec_scene::xform_of(xforms, *id));
            vec![mundo]
        });
    }
}

#[cfg(test)]
#[path = "skin_desenho_tests.rs"]
mod tests;
