//! ⭐⭐⭐ **O OFFSET DE CAD DE UMA CAMADA** — a silhueta dela cresce ou encolhe (v22).
//!
//! Pedido do Enio, 2026-09-05: *"o offset do cad, contraindo e dilatando"*. É o *Offset Path* do
//! Illustrator aplicado a UM atributo da pilha de aparência, e é o que faz um adesivo, um rótulo
//! contornado ou um selo **numa forma só** — sem duplicar o objecto, que é a doença que a pilha
//! existe para curar.
//!
//! # Por que este módulo existe (e não é feito no renderer)
//!
//! O renderer é **sem estado** e o `path_tess` corre por forma e por quadro. Medido:
//!
//! | direcção | motor | custo por camada |
//! |---|---|---|
//! | **crescer**, contorno único | [`ph2d_vec_boolean::offset_ring`] (Minkowski, sem booleana) | **~0,5 µs** |
//! | **encolher**, ou compound | [`ph2d_vec_boolean::offset_path`] (o sweep booleano) | **85–440 µs** |
//!
//! Tesselar uma forma custa **~0,13 µs** (`encode_cost_by_n`: 10 k formas em 1,323 ms). ⇒ crescer é
//! da mesma ordem do desenho e **encolher é ~1 000× isso** — é o encolher que obriga ao memo, e é
//! por isso que ele mora aqui, do lado que tem estado entre quadros.
//!
//! # ⚠️ A chave é LOCAL, e é isso que faz o memo sobreviver ao arrasto
//!
//! O [`crate::contour_live`] coze em MUNDO e por isso **recoze a cada quadro** enquanto a forma é
//! arrastada (o próprio ficheiro dele declara sofrer disso). Aqui a distância é local
//! ([`ph2d_vec_scene::PaintEntry::dilate`]), então a chave é a geometria COZIDA + a distância + a
//! quina — nenhuma delas muda ao mover a forma pelo canvas.
//!
//! # ⛔ O cozimento passa pela porta do Contour, não por uma segunda
//!
//! [`crate::contour_live::cook_piece`] é quem escolhe entre o anel e a booleana, com o domínio de
//! cada um medido e escrito. Uma segunda escolha aqui divergiria da dele no dia em que o domínio
//! do anel crescesse.

use std::collections::BTreeMap;

use ph2d_vec_render::DilatedPaints;
use ph2d_vec_scene::{PaintKind, VecPath, VecPathId, VecScene};

/// A geometria de UMA camada dilatada.
struct Memo {
    /// A forma COZIDA que gerou isto — a chave que invalida.
    cooked: VecPath,
    dilate: f64,
    join: u8,
    /// O caminho dilatado. ⚠️ **Vazio de vértices = ANIQUILAÇÃO** (encolher comeu a forma), e
    /// desenhar nada é a resposta certa.
    out: VecPath,
}

/// O cozimento vivo de todas as camadas dilatadas da cena.
#[derive(Default)]
pub(crate) struct PaintDilateLive {
    out: DilatedPaints,
    memo: BTreeMap<(VecPathId, usize), Memo>,
}

impl PaintDilateLive {
    /// A geometria derivada deste quadro.
    pub(crate) fn out(&self) -> &DilatedPaints {
        &self.out
    }

    /// Re-coza o que mudou. Chamado uma vez por quadro, antes do desenho.
    ///
    /// ⚠️ **Sai cedo e sem alocar quando nada na cena tem offset** — que é o caminho comum, e é o
    /// que mantém byte-idêntico o desenho de um documento que nunca lhe toca.
    pub(crate) fn recook(&mut self, scene: &VecScene) {
        self.out.clear();
        for path in scene.paths() {
            if !path.paints.iter().any(|e| e.is_active() && e.is_dilated()) {
                continue;
            }
            // UM cozimento da forma por path, partilhado por todas as camadas dela.
            let cooked = path.cooked().into_owned();
            for (i, e) in path.paints.iter().enumerate() {
                if !(e.is_active() && e.is_dilated()) {
                    continue;
                }
                let chave = (path.id, i);
                let fresco = !self.memo.get(&chave).is_some_and(|m| {
                    m.dilate == e.dilate && m.join == e.dilate_join && m.cooked == cooked
                });
                if fresco {
                    let join = crate::vec_expand::join_of_code(e.dilate_join);
                    // ⛔ `None` = a booleana FALHOU (pânico do sweep isolado). A entrada não entra,
                    // e o renderer desenha a camada na silhueta de BASE. *Uma camada que volta à
                    // forma lê-se como «o offset não pegou»; uma que desaparece lê-se como
                    // «apaguei a camada».*
                    let Some(pecas) = crate::contour_live::cook_piece(&cooked, e.dilate, join)
                    else {
                        continue;
                    };
                    // ⚠️ **A tinta da camada tem de viajar no caminho dilatado**: o `path_tess` do
                    // renderer decide o que construir por `fill.is_some()`/`stroke.is_some()`, então
                    // um caminho sem elas seria tesselado a VAZIO e a camada não desenharia nada.
                    let mut out = merge(&pecas).unwrap_or_else(|| {
                        let mut vazio = cooked.clone();
                        vazio.verts.clear();
                        vazio.subpaths.clear();
                        vazio
                    });
                    out.fill = None;
                    out.stroke = None;
                    match &e.kind {
                        PaintKind::Fill(p) => out.fill = Some(p.clone()),
                        PaintKind::Stroke(sp) => out.stroke = Some(sp.clone()),
                    }
                    self.memo.insert(
                        chave,
                        Memo {
                            cooked: cooked.clone(),
                            dilate: e.dilate,
                            join: e.dilate_join,
                            out,
                        },
                    );
                }
                if let Some(m) = self.memo.get(&chave) {
                    self.out.insert(chave, m.out.clone());
                }
            }
        }
        // O memo não pode sobreviver à camada: apagar uma camada ou zerar o offset dela tem de a
        // devolver à silhueta da forma, e um memo órfão a manteria dilatada.
        self.memo.retain(|k, _| self.out.contains_key(k));
    }
}

/// **Funde as peças num composto `EvenOdd`** — a tradução que a sonda `probe_offset_as_effect`
/// validou.
///
/// ⚠️ **Um offset pode PARTIR a forma**: encolher um haltere para além do pescoço devolve duas
/// ilhas, e a sonda mediu **8** no pior caso do corpus. Tudo o que sai do motor está regularizado,
/// então um ponto está no conjunto sse um número ÍMPAR de contornos o cerca — é o que torna o
/// `EvenOdd` legítimo, qualquer que seja o aninhamento.
fn merge(pecas: &[VecPath]) -> Option<VecPath> {
    let mut it = pecas.iter();
    let mut out = it.next()?.clone();
    out.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    for p in it {
        out.subpaths.push(ph2d_vec_scene::Contour {
            verts: p.verts.clone(),
            closed: p.closed,
        });
        out.subpaths.extend(p.subpaths.iter().cloned());
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, VecVertex};

    fn quadrado(scene: &mut VecScene, r: f64) -> VecPathId {
        scene.push_path(VecPath {
            verts: [[-r, -r], [r, -r], [r, r], [-r, r]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
            ..VecPath::default()
        })
    }

    fn caixa(p: &VecPath) -> (f64, f64) {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for v in p.verts_all() {
            lo = lo.min(v.anchor[0]);
            hi = hi.max(v.anchor[0]);
        }
        (lo, hi)
    }

    /// ⭐⭐⭐ **CRESCER ALARGA A SILHUETA E ENCOLHER APERTA-A** — as duas direcções que o Enio pediu.
    ///
    /// ⚠️ **As duas metades num gate só, e elas passam por MOTORES DIFERENTES:** crescer sai pelo
    /// anel directo (`offset_ring`, ~0,5 µs) e encolher pelo sweep booleano (85–440 µs). Um gate
    /// que só medisse uma ficaria verde sobre a outra desligada — e é o encolher que o `offset_ring`
    /// recusa por construção.
    #[test]
    fn growing_widens_the_silhouette_and_shrinking_tightens_it() {
        for (d, esperado) in [(2.0_f64, 12.0_f64), (-2.0, 8.0)] {
            let mut scene = VecScene::default();
            let id = quadrado(&mut scene, 10.0);
            let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 255, 255)));
            e.dilate = d;
            scene.path_mut(id).expect("a forma").paints = vec![e];

            let mut live = PaintDilateLive::default();
            live.recook(&scene);
            let out = live.out().get(&(id, 0)).expect("a camada foi cozida");
            let (lo, hi) = caixa(out);
            assert!(
                (hi - esperado).abs() < 0.6 && (lo + esperado).abs() < 0.6,
                "dilate={d}: a silhueta devia medir +-{esperado}, mediu [{lo}, {hi}]"
            );
        }
    }

    /// ⭐⭐ **A TINTA DA CAMADA VIAJA NO CAMINHO DILATADO.**
    ///
    /// ⛔ Sem isto o `path_tess` do renderer tessela a VAZIO (ele decide o que construir por
    /// `fill.is_some()`/`stroke.is_some()`) e a camada **não desenha nada** — que o artista lê como
    /// «apaguei a camada», e não como «o offset não pegou».
    #[test]
    fn the_layers_paint_travels_on_the_dilated_path() {
        let mut scene = VecScene::default();
        let id = quadrado(&mut scene, 10.0);
        let mut f = PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 255, 255)));
        f.dilate = 2.0;
        let mut s = PaintEntry::stroke(ph2d_vec_scene::StrokeSpec::new(
            Rgba8::new(9, 9, 9, 255),
            1.0,
        ));
        s.dilate = 2.0;
        scene.path_mut(id).expect("a forma").paints = vec![f, s];

        let mut live = PaintDilateLive::default();
        live.recook(&scene);
        let tinta = live.out().get(&(id, 0)).expect("a camada 0");
        assert!(tinta.fill.is_some() && tinta.stroke.is_none(), "a de FILL");
        let traco = live.out().get(&(id, 1)).expect("a camada 1");
        assert!(
            traco.stroke.is_some() && traco.fill.is_none(),
            "a de STROKE"
        );
    }

    /// ⛔ **UMA CAMADA NO NEUTRO NÃO É COZIDA** — e uma DESARMADA também não.
    ///
    /// ⚠️ É o que mantém byte-idêntico o desenho de todo documento que não usa a feature, e é o que
    /// impede o sweep booleano de correr por uma camada que o olho não vê.
    #[test]
    fn a_neutral_or_disabled_layer_is_never_cooked() {
        let mut scene = VecScene::default();
        let id = quadrado(&mut scene, 10.0);
        let neutra = PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 255, 255)));
        let mut desarmada = PaintEntry::fill(Paint::Solid(Rgba8::new(0, 255, 0, 255)));
        desarmada.dilate = 2.0;
        desarmada.enabled = false;
        scene.path_mut(id).expect("a forma").paints = vec![neutra, desarmada];

        let mut live = PaintDilateLive::default();
        live.recook(&scene);
        assert!(live.out().is_empty(), "nada devia ter sido cozido");
    }

    /// ⭐⭐ **O MEMO SOBREVIVE, E MORRE COM A CAMADA.**
    ///
    /// ⚠️ A 2.ª metade é a que evita o defeito silencioso: zerar o offset tem de devolver a camada à
    /// silhueta da forma, e um memo órfão a manteria dilatada para sempre.
    #[test]
    fn the_memo_survives_a_recook_and_dies_with_the_layer() {
        let mut scene = VecScene::default();
        let id = quadrado(&mut scene, 10.0);
        let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 255, 255)));
        e.dilate = 2.0;
        scene.path_mut(id).expect("a forma").paints = vec![e];

        let mut live = PaintDilateLive::default();
        live.recook(&scene);
        assert_eq!(live.out().len(), 1);
        live.recook(&scene);
        assert_eq!(live.out().len(), 1, "recozer nao duplica");

        scene.path_mut(id).expect("a forma").paints[0].dilate = 0.0;
        live.recook(&scene);
        assert!(
            live.out().is_empty(),
            "zerar o offset tem de devolver a camada a' silhueta da forma"
        );
    }
}
