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
pub(crate) enum Family {
    /// Rasterizado pelo `SpriteRenderer` no `game_rt` (HDR) e tonemapeado.
    Sprite,
    /// Rasterizado pelo Vello, no espaço do desenhista.
    Vector,
}

/// Uma corrida contígua de ranks da mesma família — **uma passagem de desenho**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Band {
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
pub(crate) const MAX_BANDS: usize = 16;

/// Parte a ordem total em faixas.
///
/// `families[rank]` é a família do objecto naquele rank (0 = mais atrás).
///
/// ⚠️ **Uma faixa VAZIA nunca sai daqui** — uma passagem que não desenha nada ainda paga a colagem
/// de tela cheia, e um `Vec` com buracos faria o laço do presente ter de os saltar (que é a linha
/// que alguém esquece).
pub(crate) fn bands(families: &[Family]) -> Vec<Band> {
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
pub(crate) fn collapsed(families: &[Family]) -> Vec<Band> {
    let n = families.len() as u32;
    let mut out = Vec::new();
    if families.contains(&Family::Sprite) {
        out.push(Band {
            family: Family::Sprite,
            lo: 0,
            hi: n,
        });
    }
    if families.contains(&Family::Vector) {
        out.push(Band {
            family: Family::Vector,
            lo: 0,
            hi: n,
        });
    }
    out
}

/// A lei inteira: as faixas desta ordem, já com o teto aplicado.
///
/// Devolve `(faixas, degradou?)` — o segundo é o que o log imprime.
pub(crate) fn plan(families: &[Family]) -> (Vec<Band>, bool) {
    let b = bands(families);
    if b.len() > MAX_BANDS {
        (collapsed(families), true)
    } else {
        (b, false)
    }
}

/// ⭐⭐ **A ordem TOTAL de um quadro** — as duas famílias numa lista só, indexada por rank.
///
/// Preenchida pelo `sim_extract` logo a seguir ao [`ph2d_ecs::sort_key::compute_sort_ranks_into`],
/// que é o **único** ordenador do app. ⛔ Nada aqui reordena coisa nenhuma: isto é a leitura do que
/// ele decidiu, na forma de que o presente precisa.
#[derive(Clone, Debug, Default)]
pub(crate) struct FrameOrder {
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
}

impl FrameOrder {
    /// Esvazia mantendo a capacidade (HR-3).
    pub fn clear(&mut self) {
        self.families.clear();
        self.vector_ranks.clear();
    }

    /// Regista o rank de um objecto. `path` é `Some(id)` para uma forma vetorial.
    ///
    /// ⚠️ **Os ranks chegam FORA DE ORDEM** (o ordenador devolve por entidade, não por rank), então
    /// o vector de famílias cresce por índice e não por `push`.
    pub fn record(&mut self, rank: u32, path: Option<u64>) {
        let i = rank as usize;
        if self.families.len() <= i {
            self.families.resize(i + 1, None);
        }
        self.families[i] = Some(if path.is_some() {
            Family::Vector
        } else {
            Family::Sprite
        });
        if let Some(id) = path {
            self.vector_ranks.push((id, rank));
        }
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
            Some(fams) => plan(&fams),
            // ⛔ Buraco: a ordem não é confiável ⇒ a ordem da Fase 1, inteira, e `degradou = true`.
            None => {
                let n = self.families.len() as u32;
                let mut out = Vec::new();
                if self.families.iter().any(|f| *f != Some(Family::Vector)) {
                    out.push(Band {
                        family: Family::Sprite,
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
pub(crate) fn doc_bands_of(order: &FrameOrder) -> Vec<Band> {
    let (bands, _degraded) = order.plan();
    if !needs_banding(&bands) {
        return Vec::new();
    }
    bands
        .into_iter()
        .filter(|b| b.family == Family::Vector)
        .collect()
}

/// O plano precisa do laço de faixas, ou o pipeline de sempre já o desenha?
pub(crate) fn needs_banding(bands: &[Band]) -> bool {
    match bands {
        [] => false,
        [one] => {
            // ⚠️ Uma faixa só de VETORES **também** é o caminho de sempre: o compositor põe o
            // vetor por cima de um `game_rt` que não tem sprite nenhum.
            let _ = one;
            false
        }
        [a, b] => !(a.family == Family::Sprite && b.family == Family::Vector),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Family::{Sprite as S, Vector as V};

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
        assert!(doc_bands_of(&o).is_empty(), "[S, V] e' o caminho de sempre");

        let mut o = FrameOrder::default();
        o.record(0, Some(5));
        o.record(1, None);
        let b = doc_bands_of(&o);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].family, V);
        assert_eq!(o.vector_ids_in(b[0]), vec![5]);
    }
}
