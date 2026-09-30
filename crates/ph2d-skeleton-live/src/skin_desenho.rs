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
//! [`o_estilo_serve`] e [`os_nos_servem`]: uma forma com **quinas vivas**, **efeitos** ou **offset de CAD numa camada**
//! continua a desenhar-se como ontem. As quinas vivem nos NÓS (o bake re-escreve os nós e mataria o
//! raio), os efeitos correm sobre a contagem de nós (um *Zig Zag* sobre `45` nós é outro desenho) e
//! o offset de camada é indexado pelo id da fonte numa rota que a geometria viva não leva. *Uma
//! forma que ficou como era é melhor do que uma que mudou de natureza sem aviso.*

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ph2d_skeleton::Skin;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::{VecPath, VecPathId, VecXforms};
use ph2d_vec_skin::curva::{Bake, CampoIndexado};
use ph2d_vec_skin::pesos::IndiceDoCampo;

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

/// ⭐⭐ **O ESTILO desta forma deixa-a ser desenhada pelo bake?** — ver o cabeçalho, secção
/// *«O que NÃO é assado»*. Pergunta-se ao caminho VIVO da cena: os efeitos e as camadas moram
/// nele, e o recook não os reescreve.
#[must_use]
pub fn o_estilo_serve(viva: &VecPath) -> bool {
    viva.effects.is_empty() && viva.paints.iter().all(|e| e.dilate == 0.0)
}

/// ⭐⭐ **Os NÓS desta forma deixam-na ser desenhada pelo bake?** — as quinas vivas moram nos nós
/// da FONTE, e o bake re-escreve os nós. A outra metade é a [`o_estilo_serve`].
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
}

/// O que um quadro produziu para uma forma: o caminho CRU e, quando serve, o DESENHADO.
#[derive(Clone)]
pub struct Quadro {
    /// A lei nos nós do artista — vai para o caminho vivo da cena.
    pub cru: VecPath,
    /// O bake — vai para a geometria viva, quando a forma o pode ter.
    pub desenhado: Option<VecPath>,
}

struct Ultimo {
    pele: Skin,
    leis: Leis,
    estilo_serve: bool,
    quadro: Quadro,
}

struct Gaveta {
    bind: SkinBind,
    preparado: Option<Rc<Preparado>>,
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
/// `bits` é o endereço (a entidade da forma); `skin`, `pele` e `leis` são a prova. `estilo_serve`
/// é a [`o_estilo_serve`] do caminho VIVO, que só quem chama tem na mão.
///
/// `None` quando a fonte não se lê — o chamador salta a forma, como sempre saltou.
#[must_use]
pub fn quadro(
    bits: u64,
    skin: &SkinBind,
    pele: &Skin,
    leis: Leis,
    estilo_serve: bool,
) -> Option<Quadro> {
    com_a_gaveta(bits, skin, |g| {
        if let Some(u) = &g.ultimo
            && u.pele == *pele
            && u.leis == leis
            && u.estilo_serve == estilo_serve
        {
            return Some(u.quadro.clone());
        }
        let prep = Rc::clone(g.preparado.as_ref()?);
        let q = calcula(&prep, skin, pele, leis, estilo_serve);
        g.ultimo = Some(Ultimo {
            pele: pele.clone(),
            leis,
            estilo_serve,
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
    Some(Preparado {
        guardado,
        indice,
        cozido,
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
/// ⇒ `None`, e essa forma desenha-se como antes. Os efeitos da fotografia NÃO correm aqui: quem
/// decide é o estilo vivo ([`o_estilo_serve`]), e um efeito corre sobre a contagem de nós.
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

/// A lei sobre a fonte preparada — o corpo que o [`crate::skin_live`] corria por forma.
fn calcula(
    prep: &Preparado,
    skin: &SkinBind,
    pele: &Skin,
    leis: Leis,
    estilo_serve: bool,
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
    // ⭐⭐ O que o bake percorre: a fonte, ou — com quinas vivas — a fonte já arredondada, com a
    // tabela dela a passar pela MESMA porta da escolha do artista (`pesos_do_quadro`).
    let percurso: Option<(&VecPath, &[f64])> = if os_nos_servem(&guardado.path) {
        Some((&guardado.path, pesos))
    } else {
        prep.cozido
            .as_ref()
            .map(|(c, t)| (c, skin.pesos_do_quadro(t)))
    };
    let desenhado = percurso
        .filter(|_| leis.desenho && estilo_serve)
        .map(|(fonte, tabela)| {
            ph2d_vec_skin::curva::assa_a_pele(
                pele,
                fonte,
                tabela,
                &correcoes,
                leis.rigido,
                lido,
                Bake {
                    amostras: amostras_por_segmento(segmentos(fonte)),
                    tolerancia: TOLERANCIA_DA_DIAGONAL * diagonal(fonte),
                },
            )
        })
        // ⭐⭐⭐ **O CONTACTO.** Numa dobra forte a face de DENTRO de dois membros passa uma por cima
        // da outra — é geometria de dois pedaços rígidos que rodam em torno de uma junta, não erro
        // da lei — e o TRAÇO desenharia o «olho» da sobreposição por dentro. A silhueta é a
        // fronteira da UNIÃO dos membros, que é o que o estado da arte põe no contacto (Implicit
        // Skinning, Vaillant 2013) e o que a IMAGEM presa já mostra de graça (um membro por cima
        // do outro, canto em «V»). ⚠️ **Só corre quando o contorno se CRUZA**: fora do contacto a
        // forma sai byte-idêntica, e o custo é uma varredura de segmentos. ⛔ Só no DESENHADO — o
        // `cru` são os nós que o artista edita, e trocá-los pela silhueta mudar-lhe-ia a malha.
        .map(|d| {
            if leis.contacto {
                ph2d_vec_boolean::resolve_overlap(&d).unwrap_or(d)
            } else {
                d
            }
        });
    Quadro { cru, desenhado }
}

/// Quantos segmentos a forma tem, somados os contornos.
fn segmentos(p: &VecPath) -> usize {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .map(|(v, fechado)| {
            if fechado {
                v.len()
            } else {
                v.len().saturating_sub(1)
            }
        })
        .sum()
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
