//! **Expand** na shell — os cliques de *Offset Path* e *Outline Stroke*.
//!
//! O motor é `ph2d_vec_boolean::expand`; aqui mora só o que é de DOCUMENTO: quais paths, em
//! que z, com que pose, e um passo de undo para o gesto inteiro.
//!
//! **Por-path, não N-ário** (≠ booleana): offsetar três formas selecionadas offseta as três,
//! cada uma na sua. Uma operação N-ária aqui seria "funda tudo e offsete o resultado", que
//! é outra coisa e que o artista consegue pedindo Union antes.

use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{
    LineJoin, OffsetSide, VecPathId, VecScene, VecXforms, WidthProfile, bake_xform, xform_of,
};

/// Qual comando o clique pediu.
// Sem `Eq`: o perfil carrega `f64`. `PartialEq` basta — ninguém usa isto como chave.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Expand {
    /// A borda anda `d` (negativo encolhe), com quinas em `join`, no(s) contorno(s) `side`.
    Offset { join: LineJoin, side: OffsetSide },
    /// O traço vira forma preenchida.
    OutlineStroke,
    /// O traço vira forma preenchida com a largura VARIANDO pelo `profile` — o Power Stroke.
    PowerStroke { profile: WidthProfile },
}

/// O id do painel → o comando, ou `None` se o id não é nosso. Porta única: o dreno pergunta
/// para saber se ATENDE, e a mesma resposta diz o QUE fazer.
///
/// A junção vem do PAINEL (`ph2d_panel_vector::expand_join`) e não de uma cópia daqui: uma
/// segunda tabela divergiria no dia em que aparecesse um 4º estilo de quina.
pub(crate) fn expand_for_id(id: ph2d_editor::NodeId) -> Option<Expand> {
    if id == ph2d_editor::ids::VECTOR_EXPAND_OFFSET_PATH {
        Some(Expand::Offset {
            join: offset_join(),
            side: offset_side(),
        })
    } else if id == ph2d_editor::ids::VECTOR_EXPAND_OUTLINE_STROKE {
        Some(Expand::OutlineStroke)
    } else if id == ph2d_editor::ids::VECTOR_EXPAND_POWER_STROKE {
        // O perfil vem dos sliders, e quem os lê é a `render_loop` (é ela que tem o store).
        // Aqui só se diz QUAL comando é; o `profile` é preenchido lá, como o `d` do offset.
        Some(Expand::PowerStroke {
            profile: WidthProfile::UNIFORM,
        })
    } else {
        None
    }
}

/// A junção do Offset, lida do PAINEL (`ph2d_panel_vector::expand_join`). Porta única: o
/// clique e o drag ao vivo perguntam à mesma, senão divergiriam num 4º estilo de quina.
pub(crate) fn offset_join() -> LineJoin {
    match ph2d_panel_vector::expand_join() {
        1 => LineJoin::Round,
        2 => LineJoin::Bevel,
        _ => LineJoin::Miter,
    }
}

/// Qual contorno o Offset move, lido do PAINEL. Porta única, como a junção.
pub(crate) fn offset_side() -> OffsetSide {
    match ph2d_panel_vector::expand_side() {
        0 => OffsetSide::Outer,
        1 => OffsetSide::Inner,
        _ => OffsetSide::Both,
    }
}

/// Aplica `cmd` a cada path SELECIONADO. `d` é a distância do offset (ignorada pelo
/// Outline Stroke). Um passo de undo para o gesto inteiro; re-seleciona o que saiu.
///
/// ⚠️ **Um passo de undo, não um por forma** — desfazer "o Expand" tem de custar um Ctrl+Z,
/// não tantos quantos objetos estavam selecionados (a lição que o bake da física pagou).
pub(crate) fn apply_vec_expand(
    scene: &mut VecScene,
    history: &mut History,
    pen: &mut PenTool,
    xforms: &VecXforms,
    cmd: Expand,
    d: f64,
) {
    let pre = scene.clone(); // UM passo de undo para o gesto inteiro
    if expand_selection(scene, pen, xforms, cmd, d) {
        history.push_undo(pre);
        eprintln!("[ph2d-vec] expand {cmd:?}: ok");
    } else {
        eprintln!("[ph2d-vec] expand {cmd:?}: nada a converter na seleção");
    }
}

/// O NÚCLEO — offseta/converte cada path selecionado no lugar (remove+insere) e re-seleciona
/// o que saiu. Devolve `true` se mudou algo. **Sem undo**: quem chama decide (o clique/release
/// empurra um passo; o preview ao vivo confia no diff do undo global, suprimido por
/// `held_button` durante o arrasto). É a MESMA porta que o clique e o preview usam, senão o que
/// o artista vê arrastando divergiria do que o botão assa.
pub(crate) fn expand_selection(
    scene: &mut VecScene,
    pen: &mut PenTool,
    xforms: &VecXforms,
    cmd: Expand,
    d: f64,
) -> bool {
    // Os z's dos selecionados, de trás para a frente — cada path é substituído no lugar (o
    // total pode crescer), então se percorre da frente para trás.
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    if zs.is_empty() {
        return false;
    }

    let mut produced: Vec<VecPathId> = Vec::new();
    let mut touched = false;
    for &z in zs.iter().rev() {
        let Some(src) = scene.paths().get(z).cloned() else {
            continue;
        };
        // ADR-0111: a pose vive no `Transform` da entidade e o resultado nasce world-space,
        // então o operando é assado no MUNDO antes de entrar no motor — como na booleana.
        let mut world = src.clone();
        bake_xform(&mut world, &xform_of(xforms, src.id));

        let results = match cmd {
            Expand::Offset { join, side } => ph2d_vec_boolean::offset_path(&world, d, join, side),
            Expand::OutlineStroke => ph2d_vec_boolean::outline_stroke(&world),
            Expand::PowerStroke { profile } => ph2d_vec_boolean::power_stroke(&world, &profile),
        };
        if results.is_empty() {
            continue; // nada a fazer nesta forma (sem traço, offset que a some)
        }
        touched = true;

        // O Outline Stroke converte o TRAÇO. Se a forma também tinha PREENCHIMENTO, essa
        // região continua existindo e fica no lugar dela, agora sem traço — é o grupo de
        // dois objetos que o Illustrator produz. Sem fill não sobra nada da original.
        // Os dois comandos que assam TINTA convertem o traço e deixam o miolo em paz.
        let bakes_ink = matches!(cmd, Expand::OutlineStroke | Expand::PowerStroke { .. });
        let keep_fill = bakes_ink && src.fill.is_some();
        scene.remove_path(src.id);
        let mut at = z;
        if keep_fill {
            let mut base = world.clone();
            base.stroke = None;
            // `insert_path` cunha id novo (o antigo saiu com o `remove_path`) — guarda o
            // que ELE devolveu, senão a re-seleção aponta para um path que não existe.
            produced.push(scene.insert_path(at, base));
            at += 1;
        }
        for r in results {
            produced.push(scene.insert_path(at, r));
            at += 1;
        }
    }
    if touched {
        pen.select_many(&produced);
    }
    touched
}

/// A **sessão VIVA** do Offset Path — o preview enquanto o slider de Offset é arrastado.
///
/// O offset é DESTRUTIVO (consome a forma), então não dá para offsetar sobre o resultado do
/// frame anterior — seria cumulativo. A sessão guarda a cena de ANTES do arrasto e, a cada
/// frame, a RESTAURA e re-offseta com o `d` atual. O `held_button` suprime o undo global
/// durante o arrasto; ao soltar, o diff registra UM passo (a lição do painter, `2026-07-16`).
pub(crate) struct OffsetSession {
    /// A cena inteira no instante do grab — a fonte que cada frame re-offseta.
    pre: VecScene,
    /// Os paths selecionados no grab (a re-seleção após restaurar).
    sources: Vec<VecPathId>,
    /// ⚠️ **As poses CONGELADAS no grab.** O preview churna a cena (remove a fonte, insere o
    /// resultado) a cada frame, então o `vec_entities::sync` do frame seguinte DESPAWNA a
    /// entidade da fonte — e com ela some a pose dela do mapa vivo. Reconstruir os xforms do
    /// mapa churnado devolveria `Xform::IDENTITY` para a fonte, e o offset nasceria na origem:
    /// a forma **pulava de lugar**. Congelar a pose no grab a mantém estável o arrasto inteiro.
    xforms: VecXforms,
    /// O que o ÚLTIMO preview deixou na cena — os paths VIVOS do arrasto. O `settle_origins`
    /// tem de pulá-los como pula a caneta (são geometria de MUNDO reescrita a cada frame).
    ///
    /// ⚠️ Sem isto a forma **pulava para o canto direito**: o `clone_from(&pre)` restaura
    /// também o `next_id` da cena, então o resultado renasce com o MESMO id todo frame; o
    /// `sync` mantém a entidade do frame 1 — que o `settle` JÁ assentou (`Transform` =
    /// centro) — e a geometria nova, de MUNDO, nunca é re-assentada (o settle só toca
    /// entidade na identidade). Mundo × centro = translação DOBRADA, e cada re-grab
    /// compunha mais um centro. Tratar o preview como GESTO mantém o invariante do arrasto
    /// inteiro (geometria mundo + entidade na identidade); o assentamento acontece UMA vez,
    /// no frame do release, quando a sessão já morreu.
    live: Vec<VecPathId>,
    /// O arrasto já churnou a cena? Enquanto NÃO (o grab, antes de mover), `|d| ~ 0` é
    /// restauração pura — cena intocada, entidades vivas, release sem custo. Depois do 1º
    /// `d` real as entidades das fontes morreram, e `|d| ~ 0` passa a mostrar cópias de
    /// mundo (ver [`Self::preview`]).
    churned: bool,
}

impl OffsetSession {
    /// Abre a sessão no grab do slider — `None` se não há forma selecionada (nada a prever).
    /// `xforms` são as poses no instante do grab, congeladas (ver o campo).
    pub(crate) fn begin(scene: &VecScene, pen: &PenTool, xforms: &VecXforms) -> Option<Self> {
        let sources: Vec<VecPathId> = pen.selected_paths().to_vec();
        sources
            .iter()
            .any(|id| scene.paths().iter().any(|p| p.id == *id))
            .then(|| Self {
                pre: scene.clone(),
                sources,
                xforms: xforms.clone(),
                live: Vec::new(),
                churned: false,
            })
    }

    /// Os paths que o último [`Self::preview`] deixou na cena — o `settle_origins` os pula
    /// como pula a caneta (ver o campo `live`).
    pub(crate) fn live_paths(&self) -> &[VecPathId] {
        &self.live
    }

    /// Restaura a cena do grab e re-offseta ao `d` atual (junção/lado lidos do painel), com as
    /// poses CONGELADAS. Sem undo — o diff global fecha o passo ao soltar.
    ///
    /// Uma fonte que o `expand_selection` NÃO consumiu (`results` vazio) tem três destinos, e
    /// errar qualquer um foi bug visível:
    /// - **zona morta antes de qualquer churn** (`|d| < MIN_OFFSET`, nada aconteceu ainda):
    ///   cena byte-idêntica à do grab, entidades das fontes VIVAS — agarrar o slider e soltar
    ///   sem mover não muda nem um id (e não custa um passo de undo);
    /// - **`|d| ~ 0` DEPOIS de churnar** (o arrasto voltou ao centro): a entidade da fonte já
    ///   morreu, então restaurá-la crua ganharia entidade nova na IDENTIDADE e a forma pularia
    ///   para a ORIGEM (a geometria é LOCAL). Ela vira uma **cópia assada em MUNDO** — a pose
    ///   viaja na geometria, como em todo resultado do preview;
    /// - **aniquilada de verdade** (`|d|` grande que a some, e ela tinha área): some DESTE
    ///   frame — o próximo `d` a restaura do `pre` como sempre. Fonte SEM área (uma linha, que
    ///   o motor não sabe offsetar) nunca é aniquilada: vira cópia de mundo e fica visível.
    pub(crate) fn preview(&mut self, scene: &mut VecScene, pen: &mut PenTool, d: f64) {
        scene.clone_from(&self.pre);
        pen.select_many(&self.sources);
        if !self.churned && d.abs() < ph2d_vec_boolean::MIN_OFFSET {
            self.live.clear();
            return;
        }
        self.churned = true;
        let cmd = Expand::Offset {
            join: offset_join(),
            side: offset_side(),
        };
        expand_selection(scene, pen, &self.xforms, cmd, d);
        let mut live: Vec<VecPathId> = pen
            .selected_paths()
            .iter()
            .copied()
            .filter(|id| !self.sources.contains(id))
            .collect();
        for &id in &self.sources {
            let Some(z) = scene.paths().iter().position(|p| p.id == id) else {
                continue; // consumida pelo expand — o resultado já está em `live`
            };
            let Some(src) = scene.paths().get(z).cloned() else {
                continue;
            };
            // `> 0.0` é fraco de propósito, e o lado fraco é o SEGURO: uma forma de área
            // quase-zero que "deveria" sumir vira cópia visível — nada se perde; o inverso
            // (apagar uma linha porque o motor não a offseta) perderia arte.
            let annihilated =
                d.abs() >= ph2d_vec_boolean::MIN_OFFSET && ph2d_vec_boolean::area(&src).abs() > 0.0;
            scene.remove_path(id);
            if !annihilated {
                let mut world = src;
                bake_xform(&mut world, &xform_of(&self.xforms, id));
                live.push(scene.insert_path(z, world));
            }
        }
        pen.select_many(&live);
        self.live = live;
    }
}

/// Os dois knobs do Offset como o PAINEL os guarda (`join`, `side`) — a chave de mudança
/// que o retune observa. Porta única: o retune e o release leem a MESMA.
#[must_use]
pub(crate) fn expand_knobs() -> (u8, u8) {
    (
        ph2d_panel_vector::expand_join(),
        ph2d_panel_vector::expand_side(),
    )
}

/// O que o frame deve fazer com a janela de retune — decidido por [`OffsetRetune::step`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RetuneStep {
    /// Nada mudou — a janela segue armada.
    Keep,
    /// Um knob mudou e a janela é válida — re-offseta ([`OffsetRetune::apply`]).
    Retune,
    /// Outra edição aconteceu (o undo andou) — a janela morre: retunar por cima
    /// restauraria a cena do grab e ENGOLIRIA a edição do artista.
    Dead,
}

/// A **janela de RETUNE** do Offset — o "diálogo ainda aberto" do Illustrator.
///
/// Soltar o slider COMITA o offset, mas a escolha de quina vem depois de VER o resultado:
/// clicar Miter/Round/Bevel (ou trocar o Side) logo após o release re-offseta NA HORA, ao
/// MESMO `d`. A janela guarda a sessão viva (cena do grab + poses congeladas + o `d`
/// comitado) e morre na primeira edição alheia — o oráculo é a PROFUNDIDADE do undo:
/// qualquer passo que não seja nosso (mover, apagar, Ctrl+Z) a diverge, e retunar por
/// cima dessa edição a engoliria em silêncio.
pub(crate) struct OffsetRetune {
    sess: OffsetSession,
    /// O `d` comitado no release — todo retune re-offseta a este valor.
    d: f64,
    /// Os knobs `(join, side)` vistos por último — diferença = retune.
    knobs: (u8, u8),
    /// A profundidade do undo com o NOSSO passo já registrado. `None` = a aprender no
    /// próximo frame (o diff registra no FIM do frame, depois deste bloco) — e é por isso
    /// que todo retune a re-arma para `None`: o passo dele ainda não landou.
    depth: Option<usize>,
}

impl OffsetRetune {
    /// Abre a janela no release — `None` se o arrasto nunca churnou (não há o que retunar,
    /// e uma janela sobre nada retunaria a cena inteira num clique de chip perdido).
    pub(crate) fn after_release(sess: OffsetSession, d: f64) -> Option<Self> {
        sess.churned.then(|| Self {
            sess,
            d,
            knobs: expand_knobs(),
            depth: None,
        })
    }

    /// A máquina de UM frame: aprende a profundidade do undo, morre quando ela diverge,
    /// retuna quando um knob mudou. Pura — o efeito fica em [`Self::apply`].
    pub(crate) fn step(&mut self, depth_now: usize, knobs_now: (u8, u8)) -> RetuneStep {
        match self.depth {
            None => {
                self.depth = Some(depth_now);
                RetuneStep::Keep
            }
            Some(d0) if d0 != depth_now => RetuneStep::Dead,
            Some(_) if knobs_now != self.knobs => RetuneStep::Retune,
            Some(_) => RetuneStep::Keep,
        }
    }

    /// Executa o retune: re-offseta ao `d` comitado com os knobs novos (o preview lê o
    /// painel — a mesma porta de sempre).
    ///
    /// ⚠️ **As entidades do resultado voltam à IDENTIDADE antes do preview.** O release as
    /// deixou ASSENTADAS (`Transform` = centro), e o preview re-insere geometria de MUNDO
    /// sob os MESMOS ids (o `clone_from(&pre)` restaura o contador) — sem o reset, mundo ×
    /// centro dobraria a pose: a lição exata do "pula pro canto direito" (`9c0446df`). O
    /// `settle` re-assenta NESTE frame (entidade na identidade de novo) e o diff do undo
    /// registra o passo no fim dele.
    pub(crate) fn apply(
        &mut self,
        scene: &mut VecScene,
        pen: &mut PenTool,
        sim: &mut ph2d_ecs::SimWorld,
        map: &crate::vec_entities::VecEntityMap,
        knobs_now: (u8, u8),
    ) {
        for id in &self.sess.live {
            if let Some(&bits) = map.get(id)
                && let Some(mut t) = sim
                    .world_mut()
                    .get_mut::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(bits))
            {
                *t = ph2d_ecs::Transform::IDENTITY;
            }
        }
        self.sess.preview(scene, pen, self.d);
        self.knobs = knobs_now;
        self.depth = None;
    }
}

#[cfg(test)]
#[path = "vec_expand_tests.rs"]
mod tests;
