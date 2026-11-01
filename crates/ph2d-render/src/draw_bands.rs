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
#[path = "draw_bands_tests.rs"]
mod tests;
