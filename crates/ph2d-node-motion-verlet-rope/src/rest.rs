//! **O PERFIL DE REPOUSO ao longo da corda** (doc 89 folha 03 — Houdini Vellum *Rest Length
//! Scale*; *Path Deform ▸ Scale Ramp*).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é por
//! RESPONSABILIDADE: o `lib.rs` responde *como a corda relaxa* e este responde *quanto cada
//! segmento quer medir*.

/// O multiplicador do repouso na CABEÇA da corda.
pub(super) const REST_START: &str = "rest_start";
/// …e na CAUDA. Ver [`Profile`].
pub(super) const REST_END: &str = "rest_end";
/// A forma entre os dois — a mesma família de quatro que todo `field.*` desta casa oferece.
pub(super) const REST_PROFILE: &str = "rest_profile";
/// As palavras da família, na ordem dos números.
pub(super) const REST_PROFILE_LABELS: &[&str] = &["Linear", "Quad", "Smooth", "Smoother"];

/// A curva de aresta sobre um `s ∈ [0,1]` já clampado — **a MESMA família dos `field.*`**
/// (HR-5, espelho verbatim da do `motion.twist`). Monótona, exacta nos extremos.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,
        2 => s * s * (3.0 - 2.0 * s),
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0),
        _ => s,
    }
}

/// **O perfil de repouso, resolvido** — ver o cabeçalho.
///
/// ⚠️ **A corda deixa de ser UNIFORME, e é isso que a célula pedia:** o repouso era
/// `length / (count − 1)` para todo segmento, então uma corda só podia ser igualmente
/// esticada do princípio ao fim. Com o perfil, a mesma corda pode nascer tensa na cabeça e
/// frouxa na cauda — o cabo que cede na ponta, o chicote que afina.
///
/// ⚠️ **O COMPRIMENTO TOTAL é preservado**, e não é cosmética: os multiplicadores são
/// **normalizados pela média**, então mudar o perfil redistribui o repouso ao longo da corda
/// sem a encompridar nem a encurtar. Sem isso, arrastar o `Rest End` de `1` para `0,5` faria a
/// corda ENCOLHER — e o artista leria o perfil como *"um segundo controle de comprimento"*,
/// com dois knobs a discordar sobre o mesmo número.
///
/// ⚠️ **Plano ⇒ `None`, e o `None` é a lei**: com `1, 1` nenhum sítio do solver vê um `Vec`, e
/// os três leitores (a semente, a restrição de distância, a de flexão) usam o **mesmo `f32`**
/// que sempre usaram. Byte-idêntico por ESTRUTURA, não por `x · 1.0`.
pub(super) struct Profile;

impl Profile {
    /// `None` quando as duas pontas valem `1` — ver o doc acima.
    ///
    /// `segments` é `count − 1`; devolve um multiplicador por SEGMENTO.
    pub(super) fn resolve(
        start: f32,
        end: f32,
        kind: i32,
        segments: usize,
        seg_rest: f32,
    ) -> Option<Vec<f32>> {
        if start == 1.0 && end == 1.0 {
            return None;
        }
        if segments == 0 {
            return None;
        }
        // ⚠️ **O `t` corre por INTERVALOS, não por pontos**: com `n` segmentos o último tem
        // `t = 1` só se o divisor for `n − 1`. Um segmento só ⇒ `t = 0` (a cabeça manda), e
        // não uma divisão por zero.
        #[expect(clippy::cast_precision_loss, reason = "contagem de segmentos, ≤ 2^24")]
        let last = (segments.saturating_sub(1)) as f32;
        let raw: Vec<f32> = (0..segments)
            .map(|i| {
                #[expect(clippy::cast_precision_loss, reason = "índice de segmento, ≤ 2^24")]
                let t = if last > 0.0 { i as f32 / last } else { 0.0 };
                start + (end - start) * curve(kind, t.clamp(0.0, 1.0))
            })
            .collect();
        // A normalização pela MÉDIA — ver o doc. Uma média nula ou negativa não tem como
        // preservar comprimento nenhum; aí o perfil desiste e a corda é a de sempre.
        #[expect(clippy::cast_precision_loss, reason = "contagem de segmentos, ≤ 2^24")]
        let mean = raw.iter().sum::<f32>() / segments as f32;
        // ⚠️ `partial_cmp` e não `!(mean > eps)`: o clippy tem razão em que a negação de uma
        // comparação parcial esconde o caso incomparável, e aqui ele existe — um `NaN` na
        // média (uma ponta em `inf`) tem de cair no MESMO lado que a média nula.
        if !matches!(mean.partial_cmp(&1e-6), Some(std::cmp::Ordering::Greater)) {
            return None;
        }
        Some(raw.iter().map(|k| seg_rest * (k / mean)).collect())
    }
}

use super::Params;

/// **O repouso do segmento `i`** — `seg_rest` quando o perfil é plano, e é o MESMO `f32`, não
/// um igual. Ver [`rest::Profile`].
pub(super) fn seg_rest_at(p: &Params, i: usize) -> f32 {
    match &p.rest {
        Some(v) => v.get(i).copied().unwrap_or(p.seg_rest),
        None => p.seg_rest,
    }
}

/// Seed a straight, horizontal strand of `count` points from `anchor`, pinned at
/// index 0 (previous == current → at rest). The first gravity step then swings it.
pub(super) fn seed(anchor: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    // ⚠️ Com perfil a pose de repouso é a soma CUMULATIVA dos segmentos, não `i · seg_rest`:
    // uma corda que nasce afunilada tem de nascer com os nós onde o solver os quer.
    let pos: Vec<[f32; 2]> = match &p.rest {
        None => (0..p.count)
            .map(|i| [anchor[0] + i as f32 * p.seg_rest, anchor[1]])
            .collect(),
        Some(rest) => {
            let mut x = anchor[0];
            (0..p.count)
                .map(|i| {
                    if i > 0 {
                        x += rest.get(i - 1).copied().unwrap_or(p.seg_rest);
                    }
                    [x, anchor[1]]
                })
                .collect()
        }
    };
    let prev = pos.clone();
    (pos, prev)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **As duas pontas em `1` ⇒ `None`, seja qual for o perfil** — o caminho por que passa
    /// toda corda já autorada.
    #[test]
    fn a_flat_profile_resolves_to_nothing_at_all() {
        for kind in 0..4 {
            assert!(
                Profile::resolve(1.0, 1.0, kind, 12, 0.25).is_none(),
                "perfil {kind}"
            );
        }
    }

    /// ⭐ **O COMPRIMENTO TOTAL é o mesmo** — o perfil redistribui, não encompridar.
    #[test]
    fn the_profile_redistributes_the_rest_without_changing_the_total() {
        let (segments, seg) = (16_usize, 0.25_f32);
        let total = seg * segments as f32;
        for (a, b, kind) in [(2.0, 0.4, 0), (0.3, 1.8, 2), (1.0, 3.0, 3)] {
            let v = Profile::resolve(a, b, kind, segments, seg).expect("nao e' plano");
            assert_eq!(v.len(), segments, "um repouso por segmento");
            let sum: f32 = v.iter().sum();
            assert!(
                (sum - total).abs() < total * 1e-4,
                "({a}, {b}, perfil {kind}): o total mudou de {total} para {sum}"
            );
        }
    }

    /// **E ele de facto AFUNILA** — a cabeça e a cauda medem coisas diferentes.
    #[test]
    fn the_head_and_the_tail_want_different_lengths() {
        let v = Profile::resolve(2.0, 0.5, 0, 12, 0.25).expect("nao e' plano");
        assert!(
            v[0] > v[v.len() - 1] * 2.0,
            "a cabeca tinha de querer bem mais que a cauda: {:.4} contra {:.4}",
            v[0],
            v[v.len() - 1]
        );
        // E é monótono: um afunilamento não pode engrossar no caminho.
        for w in v.windows(2) {
            assert!(w[1] <= w[0] + 1e-6, "subiu: {:?}", w);
        }
    }

    /// ⚠️ **Um perfil DEGENERADO desiste em vez de rebentar** — média nula (ou negativa) não
    /// tem como preservar comprimento nenhum, e uma corda de repouso zero colapsa num ponto.
    #[test]
    fn a_degenerate_profile_gives_up_instead_of_collapsing_the_rope() {
        assert!(Profile::resolve(0.0, 0.0, 0, 12, 0.25).is_none(), "media 0");
        assert!(
            Profile::resolve(-1.0, 1.0, 0, 12, 0.25).is_none(),
            "media 0 por simetria de sinal"
        );
        assert!(
            Profile::resolve(2.0, 0.5, 0, 0, 0.25).is_none(),
            "sem segmentos"
        );
    }

    /// **Um segmento só não divide por zero** — a cabeça manda, e é a única resposta possível.
    #[test]
    fn a_single_segment_takes_the_head_and_does_not_divide_by_zero() {
        let v = Profile::resolve(3.0, 0.1, 0, 1, 0.25).expect("nao e' plano");
        assert_eq!(v.len(), 1);
        assert!(
            (v[0] - 0.25).abs() < 1e-6,
            "com um segmento so', a normalizacao devolve o repouso de sempre: {:?}",
            v[0]
        );
    }
}

#[cfg(test)]
mod seam {
    //! **A COSTURA** — o perfil pelo `eval`, não pela função pura.
    //!
    //! ⚠️ Os gates acima provam a aritmética; estes provam que ela CHEGA ao solver. Uma lei
    //! certa com um `eval` que não a lê é a causa nº 1 da semana perdida no Painter.

    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    const COUNT: f32 = 17.0;
    const LENGTH: f32 = 4.0;

    /// ⚠️ **Só ESTE nó** — o `ph2d-node-registry-init` é quem regista todos, e depender dele
    /// aqui seria um ciclo (ele depende desta crate).
    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        super::super::register(&mut reg).expect("o no' regista");
        reg
    }

    /// Uma corda PENDURADA e parada, cozida num tique só: a pose de repouso é o que o perfil
    /// desenha, e sem gravidade a mexer ela lê-se directamente.
    fn hung(setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
        let reg = registry();
        let mut g = Graph::new();
        let rope = g.add_node("motion.verlet_rope");
        g.set_param(rope, "count", COUNT);
        g.set_param(rope, "length", LENGTH);
        g.set_param(rope, "gravity", 0.0);
        setup(&mut g, rope);
        g.connect(Edge {
            from: (rope, 0),
            to: (rope, 2),
            delayed: true,
        })
        .expect("o laco de estado");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, rope, 0.0).expect("coze");
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    /// Os comprimentos dos segmentos da pose.
    fn segments(p: &[[f32; 2]]) -> Vec<f32> {
        p.windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .collect()
    }

    /// ⭐ **A corda deixa de ser UNIFORME** — e o comprimento total não se mexe.
    #[test]
    fn the_profile_reaches_the_solver_and_the_rope_stops_being_uniform() {
        let flat = segments(&hung(|_, _| {}));
        assert!(!flat.is_empty(), "a corda coze");
        let spread = flat.iter().fold(0.0_f32, |a, s| a.max((s - flat[0]).abs()));
        assert!(
            spread < 1e-4,
            "CONTROLE: sem perfil os segmentos sao todos iguais ({spread:.6})"
        );

        let tapered = segments(&hung(|g, n| {
            g.set_param(n, super::REST_START, 2.0);
            g.set_param(n, super::REST_END, 0.5);
        }));
        assert_eq!(tapered.len(), flat.len(), "a contagem nao muda");
        assert!(
            tapered[0] > tapered[tapered.len() - 1] * 2.0,
            "a cabeca tinha de ficar mais longa que a cauda: {:.4} contra {:.4}",
            tapered[0],
            tapered[tapered.len() - 1]
        );
        // E o COMPRIMENTO TOTAL sobrevive — o perfil redistribui, não encompridar.
        let (t0, t1): (f32, f32) = (flat.iter().sum(), tapered.iter().sum());
        assert!(
            (t1 - t0).abs() < t0 * 0.02,
            "o total mudou de {t0:.4} para {t1:.4} -- o perfil devia so' redistribuir"
        );
    }

    /// **O default é a corda de sempre, AO BIT** — o caminho por que passa todo grafo autorado.
    #[test]
    fn the_flat_default_is_the_rope_that_shipped_bit_for_bit() {
        let implicit = hung(|_, _| {});
        let explicit = hung(|g, n| {
            g.set_param(n, super::REST_START, 1.0);
            g.set_param(n, super::REST_END, 1.0);
            g.set_param(n, super::REST_PROFILE, 3.0);
        });
        for (i, (a, b)) in implicit.iter().zip(&explicit).enumerate() {
            assert_eq!(
                (a[0].to_bits(), a[1].to_bits()),
                (b[0].to_bits(), b[1].to_bits()),
                "no' {i}: {a:?} contra {b:?} -- com as pontas em 1 o perfil nao pode existir"
            );
        }
    }
}
