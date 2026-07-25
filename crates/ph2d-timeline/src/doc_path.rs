//! **A porta única do motion path**, no nível do DOCUMENTO ([ADR-0141] §2).
//!
//! [`crate::MotionPath::edit`] mantém a geometria e a tabela de arco em acordo. Mas a
//! distância de cada âncora está guardada num **segundo lugar** — o *valor* da key
//! correspondente na track —, e mover uma âncora move a distância de todas as
//! seguintes. Fechar só o primeiro lugar deixa o sistema **estável e errado**: a curva
//! nova na tela, e o objeto ainda percorrendo os números da curva velha.
//!
//! Então quem move uma âncora move as duas coisas, aqui, numa chamada. Duas portas
//! divergem — é a lição que este módulo já pagou três vezes noutro eixo
//! ([[feedback_derived_coordinate_seed_must_match_sample]]).
//!
//! Split de `doc.rs` sob o teto de 700 LOC, e uma unidade por direito próprio: é o
//! único lugar do documento que sabe que uma track pode ter geometria do lado.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::{AnimTarget, AnimValue, RationalTime};

use crate::doc::TimelineDoc;
use crate::path::{PathAnchor, TangentKind};
use crate::prop::PropKind;

/// O estado RESOLVIDO do auto-orient de uma entidade — o que o apply honra e o que o
/// painel mostra, da mesma função.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoOrient {
    /// Não pedido (ou não há trajetória).
    Off,
    /// Pedido e honrado: o objeto gira para a tangente do caminho.
    Active,
    /// **Pedido e RECUSADO** porque a entidade tem uma track de Rotation. A recusa é
    /// nomeada de propósito: um toggle que fica ligado sem efeito é pior que um
    /// desligado, e o artista precisa saber o que apagar para o ter.
    BlockedByRotationTrack,
}

impl TimelineDoc {
    /// **Move (ou remodela) a âncora `i` da trajetória de `target`, e reescreve as
    /// distâncias que as keys guardam** — as duas metades, numa operação.
    ///
    /// Devolve `false` se `target` não nomeia um binding Position com caminho, ou se
    /// não há âncora `i`; nesse caso nada é tocado.
    ///
    /// ⚠️ **Os TEMPOS das keys não se movem.** Arrastar uma âncora é uma edição
    /// ESPACIAL: o objeto passa a fazer um caminho diferente *no mesmo tempo*, que é o
    /// que se espera de puxar uma curva na tela. Quem quer mudar o *quando* arrasta a
    /// key no dope-sheet, e esse é o outro gesto.
    ///
    /// ⚠️ **E o pareamento é por ÍNDICE.** Âncora `i` é key `i`, na ordem em que a
    /// track as guarda (ordenada por tempo). Uma track com menos keys do que o caminho
    /// tem âncoras é escrita até onde alcança — o resto do caminho fica lá, e a
    /// autoria (Fatia 4) é quem mantém os dois do mesmo tamanho.
    ///
    /// ⚠️ **Re-suaviza as âncoras `auto` depois de mover** (a queixa do smoke: *"se
    /// tentar arrastar qualquer ponto a curva quebra"*). Uma alça Auto Bezier é função
    /// dos VIZINHOS, então mover uma âncora invalida as alças dela (se ainda `auto`) e
    /// as dos vizinhos `auto`. Sem re-derivar, as alças ficam apontando para onde a
    /// âncora ESTAVA e a curva entorta — é o mesmo que o AE faz ao arrastar um keyframe
    /// Auto Bezier. Uma âncora que o artista MOLDOU (`auto = false`) tem as alças
    /// **transladadas** com ela e é preservada ([`MotionPath::resmooth_auto`] a pula).
    pub fn move_path_anchor(&mut self, target: AnimTarget, i: usize, to: PathAnchor) -> bool {
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        let Some(path) = b.path.as_mut() else {
            return false;
        };
        if !path.set_anchor(i, to) {
            return false;
        }
        path.resmooth_auto();
        // As distâncias novas, pela porta compartilhada — LIDAS do caminho que acabou de
        // ser reconstruído, nunca recalculadas aqui. Uma segunda aritmética é uma
        // segunda resposta, e o sintoma é o objeto a andar os números de outra curva.
        self.rewrite_path_key_values(target)
    }

    /// **Move a ponta de uma alça de tangente da âncora `i`** — o gesto que MOLDA a
    /// curva à mão (o *"ainda não temos alças de manipulação"* do smoke). `out` escolhe
    /// a alça de saída (verdadeiro) ou a de entrada; `tip` é a nova ponta, em MUNDO.
    ///
    /// ⚠️ **Arrastar uma alça torna a âncora MOLDADA** (`auto = false`), senão o próximo
    /// re-suavizar de um vizinho jogaria o trabalho fora. Como a alça OPOSTA acompanha
    /// depende do [`TangentKind`] da âncora (o menu de botão direito o escolhe):
    ///
    /// - `Smooth` (o default, *Continuous Bezier* do AE): a oposta **espelha** a direção
    ///   mantendo o **próprio comprimento** — mantém a passagem pela âncora lisa sem criar
    ///   uma quina a cada toque. Numa ponta do caminho a alça oposta tem comprimento zero
    ///   e continua zero, o espelho não a acorda;
    /// - `Symmetric`: a oposta **espelha por completo** (`-h`), mesma direção e mesmo
    ///   comprimento que a arrastada;
    /// - `Corner`: a oposta fica **exatamente onde estava** — a curva pode dobrar aqui.
    ///
    /// Reescreve as distâncias das keys: mexer numa alça muda o comprimento de arco do
    /// segmento, então o número que cada key seguinte guarda muda com ela.
    pub fn move_path_tangent(
        &mut self,
        target: AnimTarget,
        i: usize,
        out: bool,
        tip: [f32; 2],
    ) -> bool {
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        let Some(path) = b.path.as_mut() else {
            return false;
        };
        let Some(mut a) = path.anchors().get(i).copied() else {
            return false;
        };
        let h = [tip[0] - a.anchor[0], tip[1] - a.anchor[1]];
        let n = (h[0] * h[0] + h[1] * h[1]).sqrt();
        let unit = if n > f32::EPSILON {
            [h[0] / n, h[1] / n]
        } else {
            [0.0, 0.0]
        };
        // Como a alça OPOSTA segue a arrastada, pelo tipo da âncora. `kind` é copiado para
        // fora de `a` antes do closure, senão ele emprestaria `a` e as escritas abaixo não
        // compilariam.
        let kind = a.kind;
        let opposite = |oppo: [f32; 2]| -> [f32; 2] {
            match kind {
                TangentKind::Corner => oppo,
                TangentKind::Symmetric => [-h[0], -h[1]],
                TangentKind::Smooth => {
                    let len = (oppo[0] * oppo[0] + oppo[1] * oppo[1]).sqrt();
                    [-unit[0] * len, -unit[1] * len]
                }
            }
        };
        if out {
            a.in_handle = opposite(a.in_handle);
            a.out_handle = h;
        } else {
            a.out_handle = opposite(a.out_handle);
            a.in_handle = h;
        }
        a.auto = false;
        path.set_anchor(i, a);
        self.rewrite_path_key_values(target)
    }

    /// **Change the [`TangentKind`] of anchor `i` on `target`'s trajectory** — the
    /// convert the right-click menu (*Corner / Smooth / Symmetric*) drives, the sibling
    /// of the vector module's per-vertex retype.
    ///
    /// Goes through [`crate::MotionPath::retype_anchor`] (Corner keeps the handles put,
    /// Smooth and Symmetric rebuild them colinear), then rewrites the key distances:
    /// `Symmetric` averages the two handle lengths, which changes the segment's arc
    /// length, so the number every following key stores changes with it. Refuses a
    /// non-Position target or a missing anchor, touching nothing.
    pub fn set_path_tangent_kind(
        &mut self,
        target: AnimTarget,
        i: usize,
        kind: TangentKind,
    ) -> bool {
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        let Some(path) = b.path.as_mut() else {
            return false;
        };
        if !path.retype_anchor(i, kind) {
            return false;
        }
        self.rewrite_path_key_values(target)
    }

    /// **O K do modo Path: acrescentar uma âncora onde o objeto está** ([ADR-0141] §3).
    ///
    /// Isto é o que "capturar a pose" significa numa trajetória, e é por isso que o
    /// `sample_prop_value` do shell **recusa** Position: capturar não é ler um escalar,
    /// é uma edição da GEOMETRIA — a âncora nova muda o percurso, e o percurso é o que
    /// as outras keys medem. Uma única porta faz as quatro coisas:
    ///
    /// 1. a âncora entra na posição que o TEMPO dela manda (âncora `i` é key `i`);
    /// 2. os vizinhos ainda `auto` re-suavizam (uma alça Auto Bezier é função dos
    ///    vizinhos, e o vizinho acabou de mudar);
    /// 3. as distâncias são relidas do caminho reconstruído;
    /// 4. **TODAS as keys** recebem a distância nova — inclusive as anteriores, se o
    ///    re-suavizar mexeu na curva antes do ponto novo.
    ///
    /// Re-keyar um instante que já tem key **move** aquela âncora em vez de empilhar
    /// uma segunda (o contrato do [`crate::TimelineDoc::upsert_key`]). Devolve `false`
    /// se `target` não é um binding Position.
    ///
    /// ⚠️ **A primeira âncora nasce sem percurso**, e isso é correto: um caminho de um
    /// ponto tem comprimento zero, e a track fica com uma key de valor `0`. O objeto
    /// não se move até haver a segunda — que é exatamente o que uma animação de uma
    /// key faz em qualquer canal.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    pub fn add_path_key(&mut self, target: AnimTarget, t: RationalTime, at: [f32; 2]) -> bool {
        let Some(b) = self.bindings().iter().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        // Onde a âncora entra: a contagem de keys ESTRITAMENTE antes de `t`. Num
        // instante já keyado isso dá o índice daquela key, e o passo abaixo a
        // sobrescreve em vez de duplicar — a mesma semântica do `upsert_key`.
        let (i, existing) = match self.active_clip().track(target) {
            Some(tr) => {
                let ks = tr.keys();
                (
                    ks.iter().filter(|k| k.t < t).count(),
                    ks.iter().any(|k| k.t == t),
                )
            }
            None => (0, false),
        };
        // A key primeiro (o valor certo é escrito no fim, quando a geometria existe).
        self.upsert_key(
            b.entity,
            PropKind::Position,
            t,
            AnimValue::Float(0.0),
            crate::Interp::Linear,
        );
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        let path = b
            .path
            .get_or_insert_with(|| crate::MotionPath::new(Vec::new()));
        let fresh = crate::MotionPath::auto_smooth(None, at, None);
        if existing {
            path.set_anchor(i, fresh);
        } else {
            path.insert_anchor(i, fresh);
        }
        // Os vizinhos `auto` (e a nova) ganham as alças que a vizinhança de AGORA pede.
        path.resmooth_auto();
        self.rewrite_path_key_values(target)
    }

    /// Reescreve o valor de TODA key com a distância que o caminho diz hoje.
    ///
    /// Extraída porque há dois autores de geometria ([`Self::move_path_anchor`] e
    /// [`Self::add_path_key`]) e **uma** resposta para "que número a key guarda". Duas
    /// cópias divergem, e o sintoma é o objeto a andar os números de uma curva que já
    /// não existe.
    fn rewrite_path_key_values(&mut self, target: AnimTarget) -> bool {
        let Some(b) = self.bindings().iter().find(|b| b.target == target) else {
            return false;
        };
        let Some(path) = b.path.as_ref() else {
            return false;
        };
        let lens: Vec<f64> = (0..path.len()).filter_map(|k| path.arclen_at(k)).collect();
        let Some(track) = self.active_clip_mut().track_mut(target) else {
            return true;
        };
        // ⚠️ **As âncoras SÃO as keys** (path.rs) — a contagem TEM de bater. O `zip`
        // abaixo escreve só até a mais curta, então uma track com MENOS keys do que o
        // caminho tem âncoras deixa a key de CHEGADA com a distância de uma âncora do
        // MEIO em vez do `total`, e o percurso do objeto encolhe: era o *"se tentar
        // arrastar qualquer ponto a curva quebra"* do smoke, causado por uma autoria que
        // NÃO passou pela porta única (`add_path_key`). Aqui é um tripwire de dev — a
        // autoria correta (K e Separate->Path) mantém os dois em passo por construção.
        debug_assert_eq!(
            track.len(),
            lens.len(),
            "a track tem {} keys para {} âncoras — a autoria não passou pela porta única",
            track.len(),
            lens.len()
        );
        for (id, s) in track.ids().to_vec().into_iter().zip(lens) {
            track.set_value(id, AnimValue::Float(s as f32));
        }
        track.resolve_roving();
        true
    }

    /// [`Self::add_path_key`] por ENTIDADE, criando o binding Position se ainda não
    /// houver — a forma que o intent do K usa, e a única que o shell precisa conhecer.
    pub fn key_the_path(&mut self, entity: u64, t: RationalTime, at: [f32; 2]) -> bool {
        let target = self.bind(entity, PropKind::Position);
        self.add_path_key(target, t, at)
    }

    /// **Reconcilia TODA trajetória Position com a track do clip ativo** — o invariante
    /// *"as âncoras SÃO as keys (1:1)"* mantido quando as keys são editadas pela TIMELINE,
    /// não pelo canvas ([ADR-0141]).
    ///
    /// As portas do caminho (`add_path_key`/`move_path_anchor`/`move_path_tangent`) mantêm
    /// âncoras e keys em passo. Mas **mover / fundir / apagar / duplicar uma key de Position
    /// no dope-sheet edita a TRACK sozinha** — e aí a contagem de âncoras descola da de keys,
    /// que é o *"mais pontos na curva do que keys na timeline → a curva quebra ao arrastar"*
    /// que o smoke reportou (o caso real: separar X/Y, deslocar uma, juntar de volta — o
    /// `merge_moved_over_stationary` da track absorve uma key, e a âncora órfã fica). Chamado
    /// UMA vez pelo `settle` do [`crate::intent_apply`], junto do re-derivar dos roving e do
    /// re-sort dos strips — o mesmo choke point que restaura todo invariante que uma edição
    /// pode quebrar.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    pub(crate) fn reconcile_position_paths(&mut self) {
        let targets: Vec<AnimTarget> = self
            .bindings()
            .iter()
            .filter(|b| b.prop == PropKind::Position && b.path.is_some())
            .map(|b| b.target)
            .collect();
        for target in targets {
            self.reconcile_one_position_path(target);
        }
    }

    /// Reconcilia UMA trajetória com a track do clip ativo.
    ///
    /// Âncoras e keys vivem no MESMO eixo — distância de arco —, então cada key é casada com
    /// a âncora cujo `arclen` bate com a distância que ela guarda (a mais próxima, gulosa,
    /// dentro de uma tolerância relativa): uma key que manteve a distância mantém a âncora (e
    /// a forma dela); uma que sumiu deixa a âncora órfã cair; e uma que apareceu
    /// (duplicar/colar) faz nascer uma na curva, na distância dela. **Idempotente quando já
    /// está em passo** — toda key casa a própria âncora, então um caminho intocado não é
    /// perturbado (nenhum passo de undo espúrio).
    fn reconcile_one_position_path(&mut self, target: AnimTarget) {
        // As distâncias que as keys da track guardam, em ordem de TEMPO.
        let Some(track) = self.active_clip().track(target) else {
            return;
        };
        let key_ds: Vec<f64> = track
            .keys()
            .iter()
            .map(|k| match k.value {
                AnimValue::Float(v) => f64::from(v),
                _ => 0.0,
            })
            .collect();

        {
            let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
                return;
            };
            let Some(path) = b.path.as_mut() else {
                return;
            };
            let old = path.anchors().to_vec();
            let arclen: Vec<f64> = (0..old.len())
                .map(|i| path.arclen_at(i).unwrap_or(0.0))
                .collect();

            // Tolerância de casamento: a key guarda `arclen` em `f32` e a âncora o mede em
            // `f64`, então o casamento EXATO tem um resíduo de arredondamento — relativo ao
            // tamanho da distância. Larga o bastante para aceitá-lo, apertada o bastante para
            // uma DUPLICATA (mesma distância, âncora já consumida) não roubar a âncora do
            // vizinho e sim nascer coincidente.
            let tol = |a: f64| 1e-4 * (a.abs() + 1.0);

            // Já 1:1 e na mesma ordem → nada a fazer (o caminho rápido, byte-idempotente).
            // Conferido por VALOR e não só por contagem: um reorder mantém a contagem mas
            // ainda tem de re-costurar.
            let in_sync = key_ds.len() == old.len()
                && key_ds
                    .iter()
                    .zip(&arclen)
                    .all(|(d, a)| (d - a).abs() <= tol(*a));
            if in_sync {
                return;
            }

            // Casamento guloso pela distância mais próxima, no eixo compartilhado.
            let mut used = vec![false; old.len()];
            let mut new_anchors: Vec<PathAnchor> = Vec::with_capacity(key_ds.len());
            for &d in &key_ds {
                let pick = (0..old.len()).filter(|&j| !used[j]).min_by(|&a, &c| {
                    (arclen[a] - d)
                        .abs()
                        .partial_cmp(&(arclen[c] - d).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                match pick {
                    Some(j) if (arclen[j] - d).abs() <= tol(arclen[j]) => {
                        used[j] = true;
                        new_anchors.push(old[j]); // preserva geometria + forma + kind
                    }
                    _ => {
                        // Uma key sem âncora (duplicar/colar, ou fora de qualquer âncora):
                        // nasce na curva, na distância que ela guarda; as alças as re-suaviza
                        // o passo abaixo.
                        let pos = path.at(d).map(|s| s.point).unwrap_or_else(|| {
                            old.last().map(|a| a.anchor).unwrap_or([0.0, 0.0])
                        });
                        new_anchors.push(crate::MotionPath::auto_smooth(None, pos, None));
                    }
                }
            }

            *path = crate::MotionPath::new(new_anchors);
            path.resmooth_auto();
        }
        // As distâncias seguem a nova lista de âncoras — pela porta única.
        self.rewrite_path_key_values(target);
    }

    /// **O auto-orient desta entidade vale, e se não vale, por quê** ([ADR-0141] §6).
    ///
    /// UMA porta com DOIS consumidores: o apply pergunta para saber se escreve o
    /// ângulo, e o painel pergunta para saber o que MOSTRAR. Duas respostas divergem, e
    /// aí o toggle diz "ligado" enquanto nada gira — o modo de falha que faz o artista
    /// desconfiar da ferramenta inteira.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    #[must_use]
    pub fn auto_orient(&self, entity: u64) -> AutoOrient {
        let Some(b) = self.binding_for(entity, PropKind::Position) else {
            return AutoOrient::Off;
        };
        if !b.auto_orient {
            return AutoOrient::Off;
        }
        // ⚠️ A RECUSA. Girar para a tangente escreve `Transform.rotation`, que é
        // exatamente o que uma track de Rotation escreve — dois autores do mesmo campo,
        // e o de trás vence em silêncio. O apply resolveria isso por ORDEM, que é a
        // pior maneira: funcionaria, ninguém saberia por quê, e inverter a ordem um dia
        // mudaria a animação de alguém sem nada no diff.
        if self.binding_for(entity, PropKind::Rotation).is_some() {
            return AutoOrient::BlockedByRotationTrack;
        }
        AutoOrient::Active
    }

    /// Liga/desliga o auto-orient do binding Position de `entity`. Devolve o estado
    /// RESOLVIDO (que pode ser a recusa), nunca só o que foi escrito.
    pub fn set_auto_orient(&mut self, entity: u64, on: bool) -> AutoOrient {
        if let Some(b) = self
            .bindings_mut()
            .iter_mut()
            .find(|b| b.entity == entity && b.prop == PropKind::Position)
        {
            b.auto_orient = on;
        }
        self.auto_orient(entity)
    }

    /// A âncora `i` da trajetória de `target`, se houver.
    #[must_use]
    pub fn path_anchor(&self, target: AnimTarget, i: usize) -> Option<PathAnchor> {
        self.binding(target)?
            .path
            .as_ref()?
            .anchors()
            .get(i)
            .copied()
    }
}

#[cfg(test)]
#[path = "doc_path_tests.rs"]
mod tests;
