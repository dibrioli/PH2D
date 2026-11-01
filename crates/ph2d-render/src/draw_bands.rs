//! ⭐⭐⭐ **AS FAIXAS DE DESENHO** — a lei que põe vetor e sprite na MESMA ordem.
//!
//! Report do Enio, 2026-08-30: *«desenhei um vector, depois uma imagem; no hierarchy ficou correto,
//! mas no canvas o vector ficou acima da IMG»*.
//!
//! # O mecanismo que ele encontrou
//!
//! As duas famílias são rasterizadas por motores diferentes, em texturas diferentes, e coladas por
//! um `over` **fixo**: `vello` sobre `game`. ⇒ o vetor ficava por cima **sempre**, e nenhum valor
//! de Z podia mudá-lo, porque os dois Z nunca eram comparados um com o outro. Isto estava
//! **declarado** no [ADR-0154](../../docs/architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md)
//! §Fase 1 (*«vetor desenha SOBRE os sprites … z-interleave por-instância é Fase 2 — nomeado, não
//! escondido»*). Isto é a Fase 2.
//!
//! # ⭐ O que NÃO foi preciso construir
//!
//! Uma segunda lei de ordenação. O [`ph2d_ecs::sort_key::compute_sort_ranks_into`] **nunca exigiu
//! um `Sprite`** — ele lê `ChildOf`, `ShowBehindParent`, `SortingLayer`, `OrderInLayer`,
//! `ZIndexOverride`, `YSort`, todos genéricos. Ele era sprite-only *porque só sprites lhe eram
//! entregues*. ⇒ as formas vetoriais entram na MESMA chamada, e daí saem numa ordem total única —
//! com camadas de ordenação, Y-sort, grupos e `ShowBehindParent` a valerem para elas **de graça**.
//!
//! # O que este módulo é
//!
//! A ordem total é uma sequência de ranks; cada rank pertence a uma família. Este módulo parte essa
//! sequência em **corridas** (runs) — as faixas —, e o presente desenha faixa a faixa, alternando
//! de motor. ⛔ Ele não sabe desenhar nada: é a lei, e é pura.

/// A que motor um rank pertence.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Family {
    /// Rasterizado pelo `SpriteRenderer` no `game_rt` (HDR) e tonemapeado.
    Sprite,
    /// Rasterizado pelo Vello, no espaço do desenhista.
    Vector,
    /// Um Pixel Sprite (GEF 19 E2): Vello como o vetor, mas desenhado pela família Pixel — faixa
    /// própria, para intercalar com as formas por rank dentro do que seria uma faixa de vetor.
    Pixel,
    /// Um objecto Motion (GEF 19 E3): as formas dele vão à PLACA, colada no acumulador — não é
    /// cena Vello nem sprite. A placa cola-se por faixa; sem placa (a rota do Vello) o plano põe-no
    /// por cima de tudo ([`motion_por_cima`]).
    Motion,
    /// Um objecto Flip (GEF 19 E4): o passe do Flip no `game_rt` (HDR → tonemap), como os sprites —
    /// a faixa logo acima da última de sprites vai no MESMO `game_rt` que ela (o quadro de sempre).
    Flip,
    /// O palco de uma macro em cena (GEF 19, resto da E3): a camada dele pousa no acumulador no rank
    /// da entidade Macro. Só com palco a tocar — sem ele o rank é do vizinho ([`FrameOrder::adormece_palcos`]).
    Palco,
}

impl Family {
    /// A faixa é uma cena Vello (`band_doc_scenes`) — tudo menos o sprite.
    #[must_use]
    pub fn is_scene(self) -> bool {
        matches!(self, Family::Vector | Family::Pixel)
    }

    /// A posição da família na ordem de sempre (degradada): sprites, Flip (no mesmo `game_rt`), vetor,
    /// pixel, Motion.
    fn old_stack(self) -> u8 {
        match self {
            Family::Sprite => 0,
            Family::Flip => 1,
            Family::Vector => 2,
            Family::Pixel => 3,
            Family::Motion => 4,
            Family::Palco => 5,
        }
    }
}

/// Uma corrida contígua de ranks da mesma família — **uma passagem de desenho**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Band {
    pub family: Family,
    /// Primeiro rank da faixa.
    pub lo: u32,
    /// Um a seguir ao último — a faixa é `[lo, hi)`.
    pub hi: u32,
}

/// ⚠️ **O TETO DE FAIXAS, e ele diz de que recurso é.**
///
/// Cada faixa custa **uma passagem de rasterização + uma colagem** de tela cheia. O custo não é
/// função de quantos objetos a cena tem — é de **quantas vezes ela alterna** entre as duas
/// famílias, e é o artista quem o controla, intercalando formas e imagens na hierarquia.
///
/// ⚠️ **O número é MEDIDO**, não escolhido: acima dele a lei degrada para o agrupamento por família
/// (o de sempre) **e o log diz**, em vez de o quadro cair em silêncio.
/// ⛔ Um teto sem degradação nomeada seria um congelamento a partir de uma forma a mais.
pub const MAX_BANDS: usize = 16;

/// Parte a ordem total em faixas.
///
/// `families[rank]` é a família do objecto naquele rank (0 = mais atrás).
///
/// ⚠️ **Uma faixa VAZIA nunca sai daqui** — uma passagem que não desenha nada ainda paga a colagem
/// de tela cheia, e um `Vec` com buracos faria o laço do presente ter de os saltar (que é a linha
/// que alguém esquece).
pub fn bands(families: &[Family]) -> Vec<Band> {
    let mut out: Vec<Band> = Vec::new();
    for (rank, &fam) in families.iter().enumerate() {
        let rank = rank as u32;
        match out.last_mut() {
            Some(b) if b.family == fam => b.hi = rank + 1,
            _ => out.push(Band {
                family: fam,
                lo: rank,
                hi: rank + 1,
            }),
        }
    }
    out
}

/// ⛔ **A DEGRADAÇÃO NOMEADA** — o que fazer quando a cena alterna mais que o teto.
///
/// Devolve **duas** faixas: tudo o que é sprite, e depois tudo o que é vetor — que é exactamente o
/// comportamento da Fase 1, o que shipava até hoje. ⚠️ Ela **não** é «meio certa»: ela é a ordem
/// antiga, inteira, e o log diz que caiu nela.
///
/// ⚠️ E ela devolve as faixas **por rank**, não por família: o consumidor continua a desenhar
/// intervalos de rank, então o caminho de degradação usa o MESMO laço. *Um caminho de fallback com
/// um laço próprio é o que diverge.*
pub fn collapsed(families: &[Family]) -> Vec<Band> {
    let n = families.len() as u32;
    [
        Family::Sprite,
        Family::Flip,
        Family::Vector,
        Family::Pixel,
        Family::Motion,
        Family::Palco,
    ]
    .into_iter()
    .filter(|f| families.contains(f))
    .map(|family| Band {
        family,
        lo: 0,
        hi: n,
    })
    .collect()
}

/// A lei inteira: as faixas desta ordem, já com o teto aplicado.
///
/// Devolve `(faixas, degradou?)` — o segundo é o que o log imprime.
pub fn plan(families: &[Family]) -> (Vec<Band>, bool) {
    teto(bands(families), families)
}

/// O teto de faixas sobre `b` (as faixas de `families`).
fn teto(b: Vec<Band>, families: &[Family]) -> (Vec<Band>, bool) {
    if b.len() > MAX_BANDS {
        (collapsed(families), true)
    } else {
        (b, false)
    }
}

/// ⚠️ GEF 19 E3: **sem a placa** (a rota do Vello, `PH2D_FORMAS_NA_PLACA=0`, a de bissecção) as
/// formas do Motion desenham-se por cima do mundo — as faixas Motion saem do meio e voltam numa só
/// no topo, e as vizinhas que elas partiam juntam-se outra vez (o plano de antes, byte a byte).
pub fn motion_por_cima(b: Vec<Band>, n: u32) -> Vec<Band> {
    if !b.iter().any(|b| b.family == Family::Motion) {
        return b;
    }
    let mut out: Vec<Band> = Vec::with_capacity(b.len());
    for band in b.into_iter().filter(|b| b.family != Family::Motion) {
        match out.last_mut() {
            Some(last) if last.family == band.family => last.hi = band.hi,
            _ => out.push(band),
        }
    }
    out.push(Band {
        family: Family::Motion,
        lo: 0,
        hi: n,
    });
    out
}

/// ⭐⭐ **A ordem TOTAL de um quadro** — as duas famílias numa lista só, indexada por rank.
///
/// Preenchida pelo `sim_extract` logo a seguir ao [`ph2d_ecs::sort_key::compute_sort_ranks_into`],
/// que é o **único** ordenador do app. ⛔ Nada aqui reordena coisa nenhuma: isto é a leitura do que
/// ele decidiu, na forma de que o presente precisa.
#[derive(Clone, Debug, Default)]
pub struct FrameOrder {
    /// `families[rank]` — a família do objecto naquele rank (0 = mais atrás).
    ///
    /// ⛔⛔ **`Option`, e o `None` é a razão de este campo não ser um `Vec<Family>`.** A primeira
    /// versão enchia os buracos com `Family::Sprite` por omissão, e **uma prova de mutação
    /// SOBREVIVEU**: saltar o registo de um rank de sprite era indistinguível de o registar, porque
    /// o preenchimento acertava por acidente. *Um zero de «não medido» e um de «sprite» eram o
    /// mesmo byte* — o balde vazio a ler-se como cheio. Com `Option`, um buraco tem nome, e a
    /// [`FrameOrder::plan`] degrada para a ordem da Fase 1 em vez de desenhar uma forma como se
    /// fosse um sprite (isto é: de a não desenhar).
    pub families: Vec<Option<Family>>,
    /// `(VecPathRef.0, rank)` para cada forma vetorial que entrou na ordem.
    ///
    /// ⚠️ Um `Vec` de pares e não um mapa: ele é percorrido inteiro uma vez por faixa de vetor, e
    /// a contagem de formas de um documento é de dezenas. *Um `BTreeMap` aqui seria uma alocação
    /// por quadro para uma busca que a varredura linear ganha.*
    pub vector_ranks: Vec<(u64, u32)>,
    /// `(bits da entidade, rank)` de cada Pixel Sprite que entrou na ordem (GEF 19 E2).
    pub pixel_ranks: Vec<(u64, u32)>,
    /// `(bits da entidade, rank)` de cada objecto Motion que entrou na ordem (GEF 19 E3).
    pub motion_ranks: Vec<(u64, u32)>,
    /// `(FlipObjectRef.0, rank)` de cada objecto Flip que entrou na ordem (GEF 19 E4) — o id do
    /// documento, que é o que o passe do Flip filtra.
    pub flip_ranks: Vec<(u64, u32)>,
    /// `(bits da entidade Macro, rank)` de cada palco que toca neste quadro.
    pub palco_ranks: Vec<(u64, u32)>,
    /// A placa do Motion cola-se neste quadro (escrito depois da decisão dela) — `false` = as formas
    /// por cima de tudo ([`motion_por_cima`]).
    pub motion_na_placa: bool,
    /// `(bits da entidade, rank)` de todo objecto que entrou na ordem — o que o CLIQUE pergunta
    /// ([`Self::stack_key`]).
    pub entity_ranks: Vec<(u64, u32)>,
}

impl FrameOrder {
    /// Esvazia mantendo a capacidade (HR-3).
    pub fn clear(&mut self) {
        self.families.clear();
        self.vector_ranks.clear();
        self.pixel_ranks.clear();
        self.motion_ranks.clear();
        self.flip_ranks.clear();
        self.palco_ranks.clear();
        self.motion_na_placa = false;
        self.entity_ranks.clear();
    }

    /// Regista que a entidade `bits` ocupa o `rank`.
    pub fn enrol(&mut self, bits: u64, rank: u32) {
        self.entity_ranks.push((bits, rank));
    }

    /// ⭐⭐ **A altura de um objecto na pilha que o presente DESENHA** — `(faixa, rank)`, maior = mais à
    /// frente; `None` = não entrou na ordem. `bands` é o plano deste quadro ([`Self::plan`]): no modo
    /// degradado a faixa manda (todo vetor por cima de todo sprite), como no desenho.
    pub fn stack_key(&self, bands: &[Band], bits: u64) -> Option<(usize, u32)> {
        let &(_, r) = self.entity_ranks.iter().find(|(b, _)| *b == bits)?;
        let fam = (*self.families.get(r as usize)?)?;
        let i = bands
            .iter()
            .position(|b| b.family == fam && b.lo <= r && r < b.hi)?;
        Some((i, r))
    }

    /// Regista o rank de um objecto. `path` é `Some(id)` para uma forma vetorial.
    ///
    /// ⚠️ **Os ranks chegam FORA DE ORDEM** (o ordenador devolve por entidade, não por rank), então
    /// o vector de famílias cresce por índice e não por `push`.
    pub fn record(&mut self, rank: u32, path: Option<u64>) {
        let fam = if path.is_some() {
            Family::Vector
        } else {
            Family::Sprite
        };
        self.put(rank, fam);
        if let Some(id) = path {
            self.vector_ranks.push((id, rank));
        }
    }

    /// Regista que o Pixel Sprite `bits` ocupa o `rank`.
    pub fn record_pixel(&mut self, rank: u32, bits: u64) {
        self.put(rank, Family::Pixel);
        self.pixel_ranks.push((bits, rank));
    }

    /// Regista que o objecto Motion `bits` ocupa o `rank`.
    pub fn record_motion(&mut self, rank: u32, bits: u64) {
        self.put(rank, Family::Motion);
        self.motion_ranks.push((bits, rank));
    }

    /// Regista que o objecto Flip `id` (`FlipObjectRef.0`) ocupa o `rank`.
    pub fn record_flip(&mut self, rank: u32, id: u64) {
        self.put(rank, Family::Flip);
        self.flip_ranks.push((id, rank));
    }

    /// Regista que a entidade Macro `bits` ocupa o `rank`.
    pub fn record_palco(&mut self, rank: u32, bits: u64) {
        self.put(rank, Family::Palco);
        self.palco_ranks.push((bits, rank));
    }

    /// ⭐ Escrito depois de saber quem toca: o rank de uma Macro SEM palco a tocar passa à família do
    /// vizinho de baixo (sem vizinho, o de cima; sozinho, sprite) — nada a desenhar nele, e uma faixa
    /// vazia partiria as vizinhas (cada corte custa uma colagem e o tonemap à parte).
    pub fn adormece_palcos(&mut self, tocam: impl Fn(u64) -> bool) {
        let mortos: Vec<u32> = self
            .palco_ranks
            .iter()
            .filter(|(b, _)| !tocam(*b))
            .map(|(_, r)| *r)
            .collect();
        self.palco_ranks.retain(|(b, _)| tocam(*b));
        for r in mortos {
            let i = r as usize;
            let vivo = |f: &&Option<Family>| f.is_some_and(|f| f != Family::Palco);
            let viz = self.families[..i]
                .iter()
                .rev()
                .find(vivo)
                .or_else(|| self.families[i + 1..].iter().find(vivo))
                .copied()
                .flatten();
            self.families[i] = Some(viz.unwrap_or(Family::Sprite));
        }
    }

    fn put(&mut self, rank: u32, fam: Family) {
        let i = rank as usize;
        if self.families.len() <= i {
            self.families.resize(i + 1, None);
        }
        self.families[i] = Some(fam);
    }

    /// A ordem sem buracos, ou `None` se algum rank não foi registado.
    ///
    /// ⚠️ Um buraco **não é possível** quando a conversão está certa (ela percorre exactamente os
    /// `inputs` que o ordenador rankeou). Ele existe como estado NOMEÁVEL para que a mutação que o
    /// cria sangre — e para que, se acontecer em produção, o quadro degrade em vez de perder um
    /// objecto.
    pub fn complete(&self) -> Option<Vec<Family>> {
        self.families.iter().copied().collect()
    }

    /// As faixas desta ordem, já com o teto aplicado — `(faixas, degradou?)`.
    pub fn plan(&self) -> (Vec<Band>, bool) {
        match self.complete() {
            Some(fams) if self.motion_na_placa => plan(&fams),
            Some(fams) => teto(motion_por_cima(bands(&fams), fams.len() as u32), &fams),
            // ⛔ Buraco: a ordem não é confiável ⇒ a ordem da Fase 1, inteira, e `degradou = true`.
            None => {
                let n = self.families.len() as u32;
                let mut out = Vec::new();
                if self
                    .families
                    .iter()
                    .any(|f| f.is_none_or(|f| f == Family::Sprite))
                {
                    out.push(Band {
                        family: Family::Sprite,
                        lo: 0,
                        hi: n,
                    });
                }
                if !self.flip_ranks.is_empty() {
                    out.push(Band {
                        family: Family::Flip,
                        lo: 0,
                        hi: n,
                    });
                }
                if self.has_vectors() {
                    out.push(Band {
                        family: Family::Vector,
                        lo: 0,
                        hi: n,
                    });
                }
                if !self.pixel_ranks.is_empty() {
                    out.push(Band {
                        family: Family::Pixel,
                        lo: 0,
                        hi: n,
                    });
                }
                if !self.motion_ranks.is_empty() {
                    out.push(Band {
                        family: Family::Motion,
                        lo: 0,
                        hi: n,
                    });
                }
                if !self.palco_ranks.is_empty() {
                    out.push(Band {
                        family: Family::Palco,
                        lo: 0,
                        hi: n,
                    });
                }
                (out, true)
            }
        }
    }

    /// Os ids das formas vetoriais que caem nesta faixa.
    pub fn vector_ids_in(&self, band: Band) -> Vec<u64> {
        self.vector_ranks
            .iter()
            .filter(|(_, r)| *r >= band.lo && *r < band.hi)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Os Pixel Sprites desta faixa, do de trás para o da frente (rank crescente).
    pub fn pixel_ids_in(&self, band: Band) -> Vec<u64> {
        let mut v: Vec<(u32, u64)> = self
            .pixel_ranks
            .iter()
            .filter(|(_, r)| *r >= band.lo && *r < band.hi)
            .map(|(b, r)| (*r, *b))
            .collect();
        v.sort_unstable();
        v.into_iter().map(|(_, b)| b).collect()
    }

    /// Os objectos Motion desta faixa, do de trás para o da frente (rank crescente).
    pub fn motion_ids_in(&self, band: Band) -> Vec<u64> {
        let mut v: Vec<(u32, u64)> = self
            .motion_ranks
            .iter()
            .filter(|(_, r)| *r >= band.lo && *r < band.hi)
            .map(|(b, r)| (*r, *b))
            .collect();
        v.sort_unstable();
        v.into_iter().map(|(_, b)| b).collect()
    }

    /// Os objectos Flip desta faixa (ids do documento), do de trás para o da frente.
    pub fn flip_ids_in(&self, band: Band) -> Vec<u64> {
        let mut v: Vec<(u32, u64)> = self
            .flip_ranks
            .iter()
            .filter(|(_, r)| *r >= band.lo && *r < band.hi)
            .map(|(id, r)| (*r, *id))
            .collect();
        v.sort_unstable();
        v.into_iter().map(|(_, id)| id).collect()
    }

    /// As entidades Macro com palco nesta faixa.
    pub fn palco_ids_in(&self, band: Band) -> Vec<u64> {
        self.palco_ranks
            .iter()
            .filter(|(_, r)| *r >= band.lo && *r < band.hi)
            .map(|(b, _)| *b)
            .collect()
    }

    /// Todos os Pixel Sprites da ordem, do de trás para o da frente.
    pub fn pixel_ids(&self) -> Vec<u64> {
        self.pixel_ids_in(Band {
            family: Family::Pixel,
            lo: 0,
            hi: u32::MAX,
        })
    }

    /// A cena tem alguma forma vetorial? `false` ⇒ o presente toma o caminho de sempre.
    pub fn has_vectors(&self) -> bool {
        !self.vector_ranks.is_empty()
    }
}

/// As faixas de VETOR de um quadro — vazio quando a cena **não** precisa de intercalar.
///
/// ⭐ **É esta função que decide se o quadro é banded**, e o critério é *«o plano difere da ordem
/// da Fase 1?»*: nenhuma faixa (cena vazia), só sprites, só vetores, ou sprites-e-depois-vetores
/// são exactamente o que o pipeline de sempre desenha — e nesses casos o quadro tem de ficar
/// **byte-idêntico**, porque toda a arte já feita e todo o golden dependem dele.
///
/// ⭐ `forcar` = as formas do Motion vão à PLACA neste quadro (doc 121, W2): a camada delas é
/// colada no acumulador por cima do documento, logo o documento TEM de ir às faixas.
pub fn doc_bands_of(order: &FrameOrder, forcar: bool) -> Vec<Band> {
    let (bands, _degraded) = order.plan();
    if !forcar && !needs_banding(&bands) {
        return Vec::new();
    }
    bands.into_iter().filter(|b| b.family.is_scene()).collect()
}

/// O plano precisa do laço de faixas, ou o pipeline de sempre já o desenha?
pub fn needs_banding(bands: &[Band]) -> bool {
    match bands {
        [] => false,
        [one] => {
            // ⚠️ Uma faixa só de VETORES **também** é o caminho de sempre: o compositor põe o
            // vetor por cima de um `game_rt` que não tem sprite nenhum.
            let _ = one;
            false
        }
        // ⭐ A ordem de sempre (sprites, vetor, pixel — subconjuntos incluídos) não precisa.
        _ => !bands
            .windows(2)
            .all(|w| w[0].family.old_stack() < w[1].family.old_stack()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Family::{Flip as F, Motion as M, Palco as K, Pixel as P, Sprite as S, Vector as V};

    /// ⭐ **O caso do report**: uma forma desenhada primeiro, uma imagem importada a seguir. Na
    /// hierarquia a imagem fica ABAIXO ⇒ ela desenha DEPOIS ⇒ por cima.
    #[test]
    fn the_report_case_puts_the_image_after_the_shape() {
        let (b, degraded) = plan(&[V, S]);
        assert!(!degraded);
        assert_eq!(
            b,
            vec![
                Band {
                    family: V,
                    lo: 0,
                    hi: 1
                },
                Band {
                    family: S,
                    lo: 1,
                    hi: 2
                },
            ],
            "o vetor tem de sair na PRIMEIRA faixa — e ate' hoje ele saia sempre na ultima"
        );
    }

    /// A ordem de sempre continua a ser exprimível: importar a imagem antes de desenhar.
    #[test]
    fn the_old_order_is_still_expressible() {
        let (b, _) = plan(&[S, V]);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].family, S);
        assert_eq!(b[1].family, V);
    }

    /// Ranks contíguos da mesma família colapsam **numa** passagem.
    #[test]
    fn a_run_of_the_same_family_is_one_pass() {
        let (b, _) = plan(&[S, S, S, V, V, S]);
        assert_eq!(b.len(), 3);
        assert_eq!((b[0].lo, b[0].hi), (0, 3));
        assert_eq!((b[1].lo, b[1].hi), (3, 5));
        assert_eq!((b[2].lo, b[2].hi), (5, 6));
    }

    /// ⚠️ **As faixas cobrem a ordem inteira, sem buraco e sem sobreposição** — uma passagem que
    /// salta um rank perde um objecto, e nenhuma régua de contagem o veria.
    #[test]
    fn the_bands_tile_the_whole_order() {
        let fams = [S, V, S, S, V, S, V, V, S];
        let (b, _) = plan(&fams);
        let mut next = 0u32;
        for band in &b {
            assert_eq!(band.lo, next, "buraco ou sobreposicao nas faixas: {b:?}");
            assert!(band.hi > band.lo, "faixa vazia: {band:?}");
            next = band.hi;
        }
        assert_eq!(next as usize, fams.len(), "as faixas nao cobrem o fim");
    }

    /// Uma cena sem vetor nenhum é **uma** faixa — o caminho de sempre, sem custo novo.
    #[test]
    fn a_scene_with_no_vectors_is_a_single_pass() {
        let (b, degraded) = plan(&[S, S, S, S]);
        assert!(!degraded);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].family, S);
    }

    /// E uma cena vazia não pede passagem nenhuma.
    #[test]
    fn an_empty_order_asks_for_no_pass() {
        let (b, degraded) = plan(&[]);
        assert!(b.is_empty());
        assert!(!degraded);
    }

    /// ⛔ **Acima do teto a lei DEGRADA para a ordem de sempre**, e diz que degradou.
    #[test]
    fn beyond_the_ceiling_it_falls_back_to_the_phase_one_order_and_says_so() {
        let fams: Vec<Family> = (0..MAX_BANDS + 2)
            .map(|i| if i % 2 == 0 { S } else { V })
            .collect();
        assert!(
            bands(&fams).len() > MAX_BANDS,
            "a fixtura nao estoura o teto"
        );
        let (b, degraded) = plan(&fams);
        assert!(degraded, "estourou o teto e nao disse");
        assert_eq!(b.len(), 2, "a degradacao e' sprite-tudo, depois vetor-tudo");
        assert_eq!(b[0].family, S);
        assert_eq!(b[1].family, V);
        // ⚠️ E as duas faixas cobrem a ordem INTEIRA — o consumidor filtra por família.
        assert_eq!((b[0].lo, b[0].hi), (0, fams.len() as u32));
        assert_eq!((b[1].lo, b[1].hi), (0, fams.len() as u32));
    }

    /// Exactamente no teto **não** degrada — a cerca fica no lado perigoso, não nos dois.
    #[test]
    fn exactly_at_the_ceiling_it_does_not_degrade() {
        let fams: Vec<Family> = (0..MAX_BANDS)
            .map(|i| if i % 2 == 0 { S } else { V })
            .collect();
        assert_eq!(bands(&fams).len(), MAX_BANDS);
        assert!(!plan(&fams).1);
    }

    /// A degradação de uma cena só-de-sprites continua a ser **uma** faixa.
    #[test]
    fn the_fallback_of_a_sprite_only_scene_is_one_band() {
        assert_eq!(collapsed(&[S, S]).len(), 1);
        assert_eq!(collapsed(&[V, V]).len(), 1);
        assert!(collapsed(&[]).is_empty());
    }

    /// ⭐ **A ordem chega FORA DE ORDEM e o registo tem de a colocar por índice.** O ordenador
    /// devolve `(entidade, rank)`, e a travessia que os regista é a do mundo, não a do rank.
    #[test]
    fn recording_out_of_order_still_lands_each_family_at_its_rank() {
        let mut o = FrameOrder::default();
        o.record(2, None);
        o.record(0, Some(7));
        o.record(1, None);
        assert_eq!(o.complete(), Some(vec![V, S, S]));
        assert_eq!(
            o.vector_ids_in(Band {
                family: V,
                lo: 0,
                hi: 1
            }),
            vec![7]
        );
        assert!(
            o.vector_ids_in(Band {
                family: V,
                lo: 1,
                hi: 3
            })
            .is_empty()
        );
    }

    /// Uma cena sem forma nenhuma diz que não tem — é o que manda o presente pelo caminho de
    /// sempre, sem pagar faixa nenhuma.
    #[test]
    fn a_scene_with_no_shapes_says_so() {
        let mut o = FrameOrder::default();
        o.record(0, None);
        assert!(!o.has_vectors());
        o.record(1, Some(3));
        assert!(o.has_vectors());
    }

    /// `clear` esvazia as DUAS metades — deixar `vector_ranks` para trás faria uma forma apagada
    /// continuar a pedir uma faixa, e a faixa desenharia o nada por cima de um sprite.
    #[test]
    fn clear_empties_both_halves() {
        let mut o = FrameOrder::default();
        o.record(0, Some(1));
        o.record(1, None);
        o.clear();
        assert!(o.families.is_empty());
        assert!(!o.has_vectors());
    }

    /// ⛔⛔ **UM BURACO TEM NOME.** Este é o gate que a primeira versão do modelo não conseguia
    /// ter: com o preenchimento por omissão a `Sprite`, saltar um rank de sprite era invisível.
    ///
    /// **Mutação que deve sangrar:** no `build_frame_order`, saltar o `record` quando o objecto
    /// não é uma forma.
    #[test]
    fn a_rank_that_was_never_recorded_is_a_hole_and_the_plan_degrades() {
        let mut o = FrameOrder::default();
        // O rank 0 (um sprite) NÃO é registado; só o 1.
        o.record(1, Some(9));
        assert_eq!(o.complete(), None, "o buraco no rank 0 nao foi visto");
        let (b, degraded) = o.plan();
        assert!(
            degraded,
            "uma ordem com buraco tem de degradar, nao de adivinhar"
        );
        assert_eq!(b.len(), 2, "a degradacao e' a ordem da Fase 1");
        assert_eq!(b[0].family, S);
        assert_eq!(b[1].family, V);
    }

    /// E uma ordem sem buracos **não** degrada.
    #[test]
    fn a_complete_order_does_not_degrade() {
        let mut o = FrameOrder::default();
        o.record(0, None);
        o.record(1, Some(9));
        assert_eq!(o.complete(), Some(vec![S, V]));
        assert!(!o.plan().1);
    }

    /// ⭐ **O caminho de sempre NÃO é banded** — e isto é o que mantém todo o golden byte-idêntico.
    #[test]
    fn the_phase_one_orders_do_not_need_banding() {
        for fams in [
            vec![],
            vec![S],
            vec![V],
            vec![S, S],
            vec![S, V],
            vec![S, S, V, V],
        ] {
            let (b, _) = plan(&fams);
            assert!(!needs_banding(&b), "{fams:?} nao devia precisar de faixas");
        }
    }

    /// ⭐⭐ **O caso do report PRECISA** — e é a única diferença entre os dois conjuntos.
    #[test]
    fn the_report_case_needs_banding() {
        for fams in [vec![V, S], vec![S, V, S], vec![V, S, V]] {
            let (b, _) = plan(&fams);
            assert!(needs_banding(&b), "{fams:?} tinha de precisar de faixas");
        }
    }

    /// `doc_bands_of` devolve **só** as faixas de vetor, e vazio no caminho de sempre.
    #[test]
    fn doc_bands_are_the_vector_runs_and_empty_on_the_old_path() {
        let mut o = FrameOrder::default();
        o.record(0, None);
        o.record(1, Some(5));
        assert!(
            doc_bands_of(&o, false).is_empty(),
            "[S, V] e' o caminho de sempre"
        );
        // ⭐ Com as formas na placa o mesmo quadro manda o documento às faixas.
        assert_eq!(doc_bands_of(&o, true).len(), 1);

        let mut o = FrameOrder::default();
        o.record(0, Some(5));
        o.record(1, None);
        let b = doc_bands_of(&o, false);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].family, V);
        assert_eq!(o.vector_ids_in(b[0]), vec![5]);
    }

    /// ⭐⭐ **O clique lê a pilha do DESENHO** (dono 01/11): intercalado, a altura é o rank; degradado,
    /// todo vetor fica por cima de todo sprite — como o presente o desenha.
    #[test]
    fn the_stack_key_is_the_drawn_stack_in_both_regimes() {
        let mut o = FrameOrder::default();
        // rank 0 vetor (bits 10) · rank 1 sprite (bits 11) · rank 2 vetor (bits 12)
        for (r, bits, path) in [(0u32, 10u64, Some(1u64)), (1, 11, None), (2, 12, Some(2))] {
            o.record(r, path);
            o.enrol(bits, r);
        }
        let (b, degraded) = o.plan();
        assert!(!degraded);
        let k = |bits| o.stack_key(&b, bits).expect("entrou na ordem");
        assert!(
            k(12) > k(11) && k(11) > k(10),
            "intercalado: a pilha e' o rank"
        );

        let b = collapsed(&[V, S, V]);
        let k = |bits| o.stack_key(&b, bits).expect("entrou na ordem");
        assert!(
            k(10) > k(11),
            "degradado: o vetor desenha-se por cima de todo sprite"
        );
        assert_eq!(o.stack_key(&b, 99), None, "fora da ordem nao tem altura");
    }

    /// ⭐ GEF 19 E2: três famílias — as faixas cobrem a ordem, o pixel intercala com o vetor.
    #[test]
    fn three_family_bands_tile_the_order() {
        let fams = [P, V, S, P, P, V];
        let (b, degraded) = plan(&fams);
        assert!(!degraded);
        let got: Vec<_> = b.iter().map(|b| (b.family, b.lo, b.hi)).collect();
        assert_eq!(
            got,
            vec![(P, 0, 1), (V, 1, 2), (S, 2, 3), (P, 3, 5), (V, 5, 6)]
        );
        assert!(needs_banding(&b));
    }

    /// A degradação (teto ou buraco) põe o pixel POR ÚLTIMO — o quadro de antes da E2.
    #[test]
    fn the_degraded_order_puts_pixels_last() {
        let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [P, V, S][i % 3]).collect();
        let (b, degraded) = plan(&fams);
        assert!(degraded);
        assert_eq!(
            b.iter().map(|b| b.family).collect::<Vec<_>>(),
            vec![S, V, P]
        );
        let mut o = FrameOrder::default();
        o.record_pixel(0, 40);
        o.record(2, Some(9));
        let (b, degraded) = o.plan();
        assert!(degraded, "rank 1 e' buraco");
        assert_eq!(
            b.iter().map(|b| b.family).collect::<Vec<_>>(),
            vec![S, V, P]
        );
    }

    /// A ordem de sempre com pixel por cima não pede faixas (byte-idêntico); fora dela, pede.
    #[test]
    fn the_old_stack_with_pixels_on_top_needs_no_banding() {
        for fams in [
            vec![P],
            vec![S, P],
            vec![V, P],
            vec![S, V, P],
            vec![S, S, V, P, P],
        ] {
            assert!(!needs_banding(&plan(&fams).0), "{fams:?}");
        }
        for fams in [vec![P, S], vec![P, V], vec![S, P, V], vec![V, P, S]] {
            assert!(needs_banding(&plan(&fams).0), "{fams:?}");
        }
    }

    /// `doc_bands_of` devolve as faixas de vetor E de pixel; `pixel_ids_in` vai por rank.
    #[test]
    fn doc_bands_carry_the_pixel_runs() {
        let mut o = FrameOrder::default();
        o.record_pixel(1, 21);
        o.record_pixel(0, 20);
        o.record(2, None);
        o.record(3, Some(5));
        let b = doc_bands_of(&o, false);
        assert_eq!(b.iter().map(|b| b.family).collect::<Vec<_>>(), vec![P, V]);
        assert_eq!(o.pixel_ids_in(b[0]), vec![20, 21]);
        assert!(o.vector_ids_in(b[0]).is_empty());
        assert_eq!(o.pixel_ids(), vec![20, 21]);
        o.clear();
        assert!(o.pixel_ids().is_empty());
    }

    /// O clique (`stack_key`) lê o pixel na pilha desenhada, nos dois regimes.
    #[test]
    fn the_stack_key_works_for_pixel_sprites() {
        let mut o = FrameOrder::default();
        o.record_pixel(0, 10);
        o.enrol(10, 0);
        o.record(1, Some(1));
        o.enrol(11, 1);
        let (b, _) = o.plan();
        let k = |b: &[Band], bits| o.stack_key(b, bits).expect("na ordem");
        assert!(
            k(&b, 11) > k(&b, 10),
            "intercalado: o pixel de rank 0 fica atras"
        );
        let c = collapsed(&[P, V]);
        assert!(k(&c, 10) > k(&c, 11), "degradado: o pixel por cima de tudo");
    }

    /// ⭐ GEF 19 E3: a quarta família — as corridas cruas cobrem a ordem e o Motion tem rank próprio.
    #[test]
    fn four_family_bands_tile_the_order() {
        let fams = [M, S, V, M, P];
        let got: Vec<_> = bands(&fams)
            .iter()
            .map(|b| (b.family, b.lo, b.hi))
            .collect();
        assert_eq!(
            got,
            vec![(M, 0, 1), (S, 1, 2), (V, 2, 3), (M, 3, 4), (P, 4, 5)]
        );
        assert!(!M.is_scene(), "o Motion e' placa, nao cena Vello");
    }

    /// Sem a placa o plano põe o Motion por cima de tudo e cola as vizinhas que ele partia — o
    /// resto é o plano de antes (byte-idêntico); com a placa, a faixa fica no rank dela.
    #[test]
    fn the_plan_keeps_motion_on_top_and_the_rest_unchanged() {
        let mut o = FrameOrder::default();
        o.record(0, None);
        o.record_motion(1, 40);
        o.record(2, None);
        o.record(3, Some(9));
        let (b, degraded) = o.plan();
        assert!(!degraded);
        let got: Vec<_> = b.iter().map(|b| (b.family, b.lo, b.hi)).collect();
        assert_eq!(got, vec![(S, 0, 3), (V, 3, 4), (M, 0, 4)]);
        assert!(
            !needs_banding(&b),
            "[S, V] + Motion no topo = o caminho de sempre"
        );
        o.motion_na_placa = true;
        let got: Vec<_> = o.plan().0.iter().map(|b| (b.family, b.lo, b.hi)).collect();
        assert_eq!(got, vec![(S, 0, 1), (M, 1, 2), (S, 2, 3), (V, 3, 4)]);
        assert!(
            !needs_banding(&plan(&[S, V, M]).0),
            "Motion por ultimo = o de sempre"
        );
        assert!(needs_banding(&plan(&[M, S]).0));
        let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [M, V, S][i % 3]).collect();
        let (b, _) = plan(&fams);
        assert_eq!(b.last().map(|b| b.family), Some(M));
        let mut o = FrameOrder::default();
        o.record_motion(0, 40);
        o.record(2, Some(9));
        o.motion_na_placa = true;
        let (b, degraded) = o.plan();
        assert!(degraded, "rank 1 e' buraco");
        assert_eq!(
            b.iter().map(|b| b.family).collect::<Vec<_>>(),
            vec![S, V, M]
        );
    }

    /// O clique (`stack_key`) acha o objecto Motion — no rank dele com a placa, por cima sem ela.
    #[test]
    fn the_stack_key_works_for_motion_objects() {
        let mut o = FrameOrder::default();
        o.record_motion(0, 10);
        o.enrol(10, 0);
        o.record(1, None);
        o.enrol(11, 1);
        let (b, _) = o.plan();
        let k = |bits| o.stack_key(&b, bits).expect("na ordem");
        assert!(k(10) > k(11), "sem placa: por cima do mundo");
        assert_eq!(o.motion_ids_in(b[b.len() - 1]), vec![10]);
        o.motion_na_placa = true;
        let (b, _) = o.plan();
        let k = |bits| o.stack_key(&b, bits).expect("na ordem");
        assert!(k(10) < k(11), "com placa: o rank da Hierarchy");
        assert!(doc_bands_of(&o, true).iter().all(|b| b.family != M));
    }

    /// ⭐ GEF 19 E4: o Flip logo acima dos sprites é o `game_rt` de sempre (sem faixas); abaixo deles,
    /// ou acima do vetor, intercala. A degradação e o buraco põem-no logo a seguir aos sprites.
    #[test]
    fn the_flip_sits_on_the_sprites_in_the_old_stack_and_bands_elsewhere() {
        assert!(
            !needs_banding(&plan(&[S, F, V, P, M]).0),
            "a ordem de sempre"
        );
        assert!(!needs_banding(&plan(&[F, V]).0));
        assert!(
            needs_banding(&plan(&[F, S]).0),
            "Flip por baixo das imagens"
        );
        assert!(needs_banding(&plan(&[S, V, F]).0), "Flip por cima do vetor");
        assert!(!F.is_scene(), "o Flip e' game_rt, nao cena Vello");
        let fams: Vec<Family> = (0..MAX_BANDS + 3).map(|i| [F, V, S][i % 3]).collect();
        let (b, degraded) = plan(&fams);
        assert!(degraded);
        assert_eq!(
            b.iter().map(|b| b.family).collect::<Vec<_>>(),
            vec![S, F, V]
        );
        let mut o = FrameOrder::default();
        o.record_flip(0, 77);
        o.record(2, None);
        let (b, degraded) = o.plan();
        assert!(degraded, "rank 1 e' buraco");
        assert_eq!(b.iter().map(|b| b.family).collect::<Vec<_>>(), vec![S, F]);
    }

    /// Os objectos Flip de uma faixa saem do de trás para o da frente; o clique acha-os pelo rank.
    #[test]
    fn the_flip_ids_of_a_band_come_back_to_front_and_the_click_finds_them() {
        let mut o = FrameOrder::default();
        o.record_flip(2, 70);
        o.enrol(1070, 2);
        o.record(1, None);
        o.enrol(11, 1);
        o.record_flip(0, 60);
        o.enrol(1060, 0);
        let (b, _) = o.plan();
        assert_eq!(
            b.iter().map(|b| (b.family, b.lo, b.hi)).collect::<Vec<_>>(),
            vec![(F, 0, 1), (S, 1, 2), (F, 2, 3)]
        );
        assert_eq!(o.flip_ids_in(b[0]), vec![60]);
        assert_eq!(o.flip_ids_in(b[2]), vec![70]);
        let all = Band {
            family: F,
            lo: 0,
            hi: 3,
        };
        assert_eq!(o.flip_ids_in(all), vec![60, 70], "rank crescente");
        let k = |bits| o.stack_key(&b, bits).expect("na ordem");
        assert!(k(1060) < k(11) && k(11) < k(1070));
        o.clear();
        assert!(o.flip_ranks.is_empty());
    }

    /// ⭐ Uma Macro sem palco a tocar não parte a ordem: o rank dela passa ao vizinho de baixo (ou de
    /// cima, ou sprite sozinha); com palco, faixa própria no rank dela.
    #[test]
    fn a_macro_without_a_playing_stage_gives_its_rank_to_a_neighbour() {
        let ordem = || {
            let mut o = FrameOrder::default();
            o.record(0, None);
            o.record_palco(1, 50);
            o.record(2, None);
            o.record_palco(3, 51);
            o.record(4, Some(9));
            o
        };
        let mut o = ordem();
        o.adormece_palcos(|_| false);
        assert_eq!(o.complete(), Some(vec![S, S, S, S, V]));
        assert!(o.palco_ranks.is_empty());
        assert!(!needs_banding(&o.plan().0), "o quadro de sempre");
        let mut o = ordem();
        o.adormece_palcos(|m| m == 50);
        let (b, _) = o.plan();
        assert_eq!(
            b.iter().map(|b| (b.family, b.lo, b.hi)).collect::<Vec<_>>(),
            vec![(S, 0, 1), (K, 1, 2), (S, 2, 4), (V, 4, 5)]
        );
        assert_eq!(o.palco_ids_in(b[1]), vec![50]);
        let mut o = FrameOrder::default();
        o.record_palco(0, 50);
        o.record(1, Some(9));
        o.adormece_palcos(|_| false);
        assert_eq!(
            o.complete(),
            Some(vec![V, V]),
            "sem vizinho de baixo: o de cima"
        );
        let mut o = FrameOrder::default();
        o.record_palco(0, 50);
        o.adormece_palcos(|_| false);
        assert_eq!(o.complete(), Some(vec![S]));
        assert!(
            !needs_banding(&plan(&[S, V, M, K]).0),
            "palco por ultimo = o de sempre"
        );
        assert!(needs_banding(&plan(&[K, S]).0));
    }
}
