//! ⭐⭐ **COM QUE TINTA O TRAÇO DESENHA** — a porta única da fileira *Type* da secção *Stroke*
//! (plano 35, wave D).
//!
//! # Por que é um módulo próprio, e irmão do [`crate::vec_stroke_present`]
//!
//! Aquele responde *"esta forma TEM traço?"*; este, *"com que tinta ele desenha?"*. São duas
//! perguntas sobre o mesmo objecto, e a resposta de uma **pressupõe** a da outra — sem traço não há
//! tinta de traço. Mantê-las juntas num ficheiro faria a segunda herdar o `Option` da primeira por
//! acidente; separadas, cada uma declara o próprio `None`.
//!
//! # ⛔ Por que a lista tem DUAS variantes e não as cinco do preenchimento
//!
//! O renderer de traço não desenha gradiente. Um chip que produzisse um `StrokePaint::Linear`
//! gravaria estado que **nada pinta** — o documento leria de volta uma tinta inalcançável, e o
//! sintoma seria uma forma que abre diferente de como fechou. A recusa está escrita no
//! [plano 35 §2.1](../../docs/Vector%20Module/35_plano_padrao_no_traco.md); quando um gradiente no
//! traço for pedido, o `StrokePaint` ganha uma variante e esta lista ganha um chip.

use ph2d_panel_vector::StrokePaintKind;
use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{PatternFill, PatternSource, StrokePaint, VecScene};

/// O chip de tinta de traço que este `NodeId` nomeia (`None` se não é um deles).
///
/// ⚠️ **Uma porta, e não dois `if` no despacho** — a mesma forma do `vec_fill_kind_for_id`: quem
/// acrescentar uma variante ao `StrokePaint` acrescenta-a aqui e o despacho não muda uma linha.
#[must_use]
pub(crate) fn kind_for_id(id: ph2d_editor::NodeId) -> Option<StrokePaintKind> {
    if id == ph2d_editor::ids::VECTOR_STROKE_KIND_SOLID {
        Some(StrokePaintKind::Solid)
    } else if id == ph2d_editor::ids::VECTOR_STROKE_KIND_PATTERN {
        Some(StrokePaintKind::Pattern)
    } else if id == ph2d_editor::ids::VECTOR_STROKE_KIND_BRUSH {
        // ⭐⭐⭐ **O terceiro chip** (plano 36, W4). ⚠️ **Faltar AQUI é pior que faltar no
        // `set_kind`:** sem este braço o clique nem chega à porta do documento — ele cai no
        // despacho e desaparece, e o único sintoma é um chip que não acende. *Um controlo nunca
        // pintado e um morto sob o dedo dão o MESMO report.*
        Some(StrokePaintKind::Brush)
    } else {
        None
    }
}

/// **A tinta do traço da forma selecionada.** `None` quando não há uma resposta — nada selecionado,
/// selecção múltipla, ou **a forma não tem traço**.
///
/// ⚠️ O terceiro caso é o que distingue esta porta da irmã: uma forma sem traço tem uma resposta
/// para *"tem traço?"* (`Some(false)`) e **nenhuma** para *"que tinta?"*. É por isso que a caixa é
/// pintada e a fileira de tipo não.
#[must_use]
pub(crate) fn selected_stroke_paint_kind(
    scene: &VecScene,
    pen: &PenTool,
) -> Option<StrokePaintKind> {
    let [id] = pen.selected_paths() else {
        return None;
    };
    let s = scene.path(*id)?.stroke.as_ref()?;
    Some(match s.paint {
        StrokePaint::Solid(_) => StrokePaintKind::Solid,
        StrokePaint::Pattern(_) => StrokePaintKind::Pattern,
        StrokePaint::Brush(_) => StrokePaintKind::Brush,
    })
}

/// **Troca a tinta do traço da forma selecionada.** `true` se o documento mudou (um passo de undo).
///
/// - `Solid` -> a cor de recurso do padrão (`StrokeSpec::color()`), que é a que a linha já pintava
///   enquanto o ladrilho não resolvia. ⭐ **Ir e voltar não pisca para uma cor arbitrária.**
/// - `Pattern` **já sendo padrão** -> não mexe: a arte, o reticulado e a colocação sobrevivem a
///   trocar de chip e voltar, exactamente como no preenchimento.
/// - `Pattern` **sem ser** -> precisa de `pattern`, que vem de fora resolvido.
///
/// ⚠️⚠️ **`pattern == None` com `Pattern` é DESISTÊNCIA e não muda nada** — o artista fechou o
/// diálogo da arte, e apagar-lhe a cor do traço por isso seria o pior dos dois mundos. É a mesma lei
/// do `apply_vec_set_fill_kind`, e por isso está escrita do mesmo jeito.
///
/// ⚠️ **A fonte vem RESOLVIDA de fora** porque escolhê-la pode abrir um diálogo de ficheiro, que
/// congela o laço — isso é da shell (`crate::modal`), nunca desta função de documento.
pub(crate) fn set_kind(
    scene: &mut VecScene,
    history: &mut History,
    pen: &PenTool,
    kind: StrokePaintKind,
    pattern: Option<(PatternSource, [f64; 2], [f64; 2])>,
) -> bool {
    let [id] = pen.selected_paths() else {
        return false;
    };
    let id = *id;
    let Some(cur) = scene.path(id).and_then(|p| p.stroke.as_ref()) else {
        return false;
    };
    let novo = match (kind, &cur.paint) {
        // Já é o que se pediu: nada a fazer, e a lei inteira do padrão sobrevive.
        (StrokePaintKind::Solid, StrokePaint::Solid(_))
        | (StrokePaintKind::Pattern, StrokePaint::Pattern(_))
        | (StrokePaintKind::Brush, StrokePaint::Brush(_)) => return false,
        // ⭐⭐⭐ **O PINCEL NASCE AQUI** (plano 36, W4) — e a recusa que estava nesta linha era o
        // que tornava a secção *Brush* inteira inalcançável: ela só é pintada quando o traço JÁ é
        // um pincel, e a **única** porta para o ser é este chip. *Um defeito circular: o painel
        // esperava pelo estado que só o painel podia criar.*
        //
        // ⚠️ **`art: None` é legítimo por TIPO** — um pincel sem arte escolhida desenha a
        // `fallback`, e a arte entra depois pelo gesto de duas mãos (`set_art`). Exigi-la aqui
        // faria o chip abrir um diálogo de ficheiro, que é precisamente o que o plano 36 recusa.
        //
        // ⚠️⚠️ **A `fallback` carrega a cor ACTUAL, pela MESMA lei que o braço `Pattern` escreve
        // abaixo** — o default de `BrushStroke` é preto opaco, então sem esta linha a linha do
        // artista **salta para preto** no clique, e voltar a `Solid` devolveria preto em vez da cor
        // dele. *Ir e voltar não pode piscar para uma cor arbitrária.*
        (StrokePaintKind::Brush, _) => StrokePaint::Brush(Box::new(ph2d_vec_scene::BrushStroke {
            fallback: cur.color(),
            ..ph2d_vec_scene::BrushStroke::default()
        })),
        // ⚠️ A cor que fica é a `fallback` do padrão — a que a linha já mostrava.
        (StrokePaintKind::Solid, _) => StrokePaint::Solid(cur.color()),
        (StrokePaintKind::Pattern, _) => {
            let Some((source, size, origin)) = pattern else {
                return false;
            };
            let mut f = PatternFill::new(source, size, cur.color());
            // ⚠️ **A OPACIDADE atravessa a troca de tinta.** Um traço a 50% que vira padrão nasceria
            // com `alpha = 1,0` (o default do construtor) e **saltaria para opaco** no clique; e a
            // primeira mexida no painel puxá-lo-ia de volta a 50%, porque é ali que a opacidade do
            // traço mora (`StrokeStyle::onto`). *Uma opacidade, uma casa — inclusive no nascimento.*
            f.alpha = f32::from(cur.color().a) / 255.0;
            // ⛔ O canto é o da FORMA, não a origem do mundo — a lei que o `Clamp` do preenchimento
            // pagou com um report (`texture_pattern_pick::default_placement`).
            f.origin = origin;
            StrokePaint::Pattern(Box::new(f))
        }
    };
    let pre = scene.clone();
    let Some(path) = scene.path_mut(id) else {
        return false;
    };
    let Some(s) = path.stroke.as_mut() else {
        return false;
    };
    s.paint = novo;
    history.push_undo(pre);
    true
}

/// ⭐⭐⭐ **Põe a ARTE de um pincel** (plano 36, W4) — a porta do gesto de duas mãos.
///
/// ⚠️ Resolve por **ID** e não pela selecção, pela mesma razão que o picker do padrão: o alvo é
/// capturado no *arm*, e o clique seguinte cai noutra forma, que passa a ser a selecionada. Ler a
/// selecção aqui apontaria o pincel para a forma errada.
///
/// ⛔ **Uma forma não pode ser o próprio pincel** — a recusa é a primeira linha, e há uma segunda,
/// PURA, no `brush_live`. *Duas metades porque as duas portas existem: esta autora, aquela resolve.*
///
/// `true` se o documento mudou (um passo de undo).
pub(crate) fn set_art(
    scene: &mut VecScene,
    history: &mut History,
    host: ph2d_vec_scene::VecPathId,
    art: ph2d_vec_scene::VecPathId,
) -> bool {
    if art == host {
        return false;
    }
    let Some(cur) = scene
        .path(host)
        .and_then(|p| p.stroke.as_ref())
        .and_then(ph2d_vec_scene::StrokeSpec::brush)
    else {
        return false;
    };
    if cur.art == Some(art) {
        return false;
    }
    let mut next = cur.clone();
    next.art = Some(art);
    let pre = scene.clone();
    let Some(s) = scene.path_mut(host).and_then(|p| p.stroke.as_mut()) else {
        return false;
    };
    s.paint = StrokePaint::Brush(Box::new(next));
    history.push_undo(pre);
    true
}

/// **O que a secção *Brush* pede ao documento** (plano 36, W4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BrushCmd {
    /// Multiplica a altura derivada da largura do traço.
    Scale(f64),
    /// Multiplica a largura do motivo para dar o avanço.
    Spacing(f64),
    /// Desvio ao longo da normal, em unidades de mundo.
    Offset(f64),
    /// Orientação do motivo sobre a curva, em GRAUS.
    Rotation(f64),
    /// A arte do outro lado da curva.
    Flip,
}

/// Aplica `cmd` ao pincel da forma selecionada. No-op silencioso quando não há forma, quando o
/// traço não é um pincel, ou quando o valor já era esse.
///
/// ⚠️ **O `if` de igualdade no fim é o que impede um passo espúrio** quando o slider re-publica o
/// valor que já lá estava — a mesma disciplina da porta do padrão.
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut History,
    pen: &PenTool,
    cmd: BrushCmd,
) -> bool {
    let Some(sel) = pen.selected() else {
        return false;
    };
    let Some(cur) = scene
        .path(sel)
        .and_then(|p| p.stroke.as_ref())
        .and_then(ph2d_vec_scene::StrokeSpec::brush)
    else {
        return false;
    };
    let mut next = cur.clone();
    match cmd {
        BrushCmd::Scale(v) => next.scale = v,
        BrushCmd::Spacing(v) => next.spacing = v,
        BrushCmd::Offset(v) => next.offset = v,
        BrushCmd::Rotation(v) => next.rotation_deg = v,
        BrushCmd::Flip => next.flip = !next.flip,
    }
    if &next == cur {
        return false;
    }
    let pre = scene.clone();
    let Some(s) = scene.path_mut(sel).and_then(|p| p.stroke.as_mut()) else {
        return false;
    };
    s.paint = StrokePaint::Brush(Box::new(next));
    history.push_undo(pre);
    true
}

/// O comando que este `NodeId` nomeia (`None` se não é um clique da secção *Brush*).
#[must_use]
pub(crate) fn cmd_for_id(id: ph2d_editor::NodeId) -> Option<BrushCmd> {
    (id == ph2d_editor::ids::VECTOR_BRUSH_FLIP).then_some(BrushCmd::Flip)
}

/// O comando de um SLIDER da secção *Brush* (`None` se não é dela). ⚠️ O `event.rs` do painel já
/// converteu o track para o domínio do documento — aqui `v` é valor.
#[must_use]
pub(crate) fn slider_cmd_for_id(id: ph2d_editor::NodeId, v: f64) -> Option<BrushCmd> {
    use ph2d_editor::ids as i;
    if id == i::VECTOR_BRUSH_SCALE {
        Some(BrushCmd::Scale(v))
    } else if id == i::VECTOR_BRUSH_SPACING {
        Some(BrushCmd::Spacing(v))
    } else if id == i::VECTOR_BRUSH_OFFSET {
        Some(BrushCmd::Offset(v))
    } else if id == i::VECTOR_BRUSH_ROTATION {
        Some(BrushCmd::Rotation(v))
    } else {
        None
    }
}

/// ⭐⭐⭐ **O CHIP `Brush` DEIXA DE SER MORTO — e o que se mede é o VALOR A CHEGAR AO CONSUMIDOR.**
///
/// # O defeito, e por que ele era circular
///
/// Faltavam **dois** braços, não um: o `kind_for_id` não mapeava o `VECTOR_STROKE_KIND_BRUSH` (o
/// clique nem chegava à porta do documento) e o `set_kind` recusava-o (`=> return false`). A secção
/// *Brush* do painel só é pintada quando o traço **já é** um pincel — e a única porta para o ser é
/// este chip. ⇒ *o painel esperava pelo estado que só ele podia criar*, e o resto da UI (o botão da
/// arte, os quatro sliders, o Flip) estava construído, registado e inalcançável.
///
/// # Por que estes gates e não «`set_kind` devolveu `true`»
///
/// ⚠️ Um `true` diz que a porta correu, não o que ela escreveu. O que os dois consumidores a
/// jusante de facto perguntam é: `StrokeSpec::brush()` (é sobre esse `Option` que o shell mapeia o
/// `BrushRow` — `None` **esconde a secção**) e [`selected_stroke_paint_kind`] (que acende o chip).
/// São essas duas perguntas que se fazem aqui, mais a **cor**, que é onde o defeito silencioso
/// morava.
#[cfg(test)]
mod brush_kind_gates {
    use super::*;
    use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex};

    /// Uma forma com traço **sólido** numa cor que não é nenhum default (⚠️ nem preto, nem opaca:
    /// é contra o preto opaco do `BrushStroke::default()` que a cura se mede).
    fn forma_com_traco_solido(cor: Rgba8) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
        let mut scene = VecScene::default();
        let id = scene.push_path(VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            stroke: Some(StrokeSpec::new(cor, 0.5)),
            ..VecPath::default()
        });
        let mut pen = ph2d_vec_edit::PenTool::default();
        pen.select_many(&[id]);
        (scene, pen, id)
    }

    /// **O clique no chip chega à porta do documento.**
    ///
    /// ⚠️ Este é o braço cuja falta é INVISÍVEL a todo gate sobre o `set_kind`: sem ele o despacho
    /// simplesmente não reconhece o id, e a porta nunca é chamada.
    #[test]
    fn the_brush_chip_is_reachable_by_its_node_id() {
        assert_eq!(
            kind_for_id(ph2d_editor::ids::VECTOR_STROKE_KIND_BRUSH),
            Some(StrokePaintKind::Brush),
            "o chip Brush nao e' mapeado — o clique cai no despacho e desaparece, e o unico \
             sintoma e' um chip que nao acende"
        );
        // ⛔ E nenhum vizinho o reclama: seria a fileira do traço a comer o clique do outro.
        assert_eq!(
            kind_for_id(ph2d_editor::ids::VECTOR_FILL_KIND_PATTERN),
            None
        );
        assert_eq!(kind_for_id(ph2d_editor::ids::VECTOR_BRUSH_FLIP), None);
    }

    /// ⭐⭐ **O VALOR QUE CHEGA AO CONSUMIDOR: um pincel com a cor que a linha já tinha.**
    ///
    /// ⚠️ **A cor é a metade que falharia em silêncio.** `BrushStroke::default().fallback` é
    /// **preto opaco**; sem carregar a `cur.color()` a linha do artista salta para preto no clique,
    /// e voltar a `Solid` devolveria preto — porque `StrokeSpec::color()` lê a `fallback`. É a
    /// mesma lei que o braço `Pattern` já escrevia ao lado.
    #[test]
    fn clicking_the_brush_chip_leaves_a_brush_paint_carrying_the_colour_it_had() {
        let cor = Rgba8::new(11, 22, 33, 128);
        let (mut scene, pen, id) = forma_com_traco_solido(cor);
        let mut h = History::default();
        let kind = kind_for_id(ph2d_editor::ids::VECTOR_STROKE_KIND_BRUSH)
            .expect("o chip tem de mapear — vide o gate irmao");
        assert!(
            set_kind(&mut scene, &mut h, &pen, kind, None),
            "o chip Brush nao muda o documento — a seccao Brush fica inalcancavel para sempre"
        );
        let s = scene
            .path(id)
            .and_then(|p| p.stroke.as_ref())
            .expect("a forma tem traco");
        let b = s.brush().expect(
            "o traco nao e' um pincel — e' este `Option` que o shell mapeia para o `BrushRow`, \
             entao `None` aqui ESCONDE a seccao Brush inteira",
        );
        assert_eq!(
            b.fallback, cor,
            "o pincel nasceu com a cor de recurso ERRADA (o default e' preto opaco): a linha \
             salta de cor no clique, e voltar a Solid devolve preto"
        );
        assert_eq!(
            b.art, None,
            "a arte nasce por escolher — o chip ARMA o gesto de duas maos, nao abre um dialogo"
        );
        assert_eq!(h.undo_len(), 1, "trocar a tinta e' UM passo de undo");
        // ⚠️ **Ir e VOLTAR não pisca**: a cor do artista sobrevive à ida ao pincel.
        assert!(set_kind(
            &mut scene,
            &mut h,
            &pen,
            StrokePaintKind::Solid,
            None
        ));
        assert_eq!(
            scene
                .path(id)
                .and_then(|p| p.stroke.as_ref())
                .map(StrokeSpec::color),
            Some(cor),
            "a volta ao solido nao devolveu a cor — o pincel comeu-a"
        );
    }

    /// ⭐⭐⭐ **A SECÇÃO BRUSH PASSA A SER ALCANÇÁVEL** — as duas perguntas que a fazem subir.
    ///
    /// ⚠️ As duas são as **mesmas expressões** que o shell corre por quadro
    /// (`render_loop`: `…stroke.as_ref().and_then(StrokeSpec::brush).map(|b| BrushRow {…})` e
    /// `set_stroke_paint_kind(selected_stroke_paint_kind(…))`). Perguntá-las aqui é medir a
    /// costura, não uma paráfrase dela.
    ///
    /// ⚠️ **O CONTROLO é a primeira metade**: antes do clique as duas respondem que a secção não
    /// sobe. Sem ele o gate ficaria verde num produto em que ela está sempre visível.
    #[test]
    fn the_brush_section_becomes_reachable_only_after_the_chip_is_clicked() {
        let (mut scene, pen, id) = forma_com_traco_solido(Rgba8::new(200, 40, 40, 255));
        // CONTROLO — o estado de partida: chip aceso é o `Solid`, e a secção NÃO sobe.
        assert_eq!(
            selected_stroke_paint_kind(&scene, &pen),
            Some(StrokePaintKind::Solid)
        );
        assert!(
            scene
                .path(id)
                .and_then(|p| p.stroke.as_ref())
                .and_then(StrokeSpec::brush)
                .is_none(),
            "a seccao Brush ja' subia num traco solido"
        );
        let mut h = History::default();
        assert!(set_kind(
            &mut scene,
            &mut h,
            &pen,
            kind_for_id(ph2d_editor::ids::VECTOR_STROKE_KIND_BRUSH).expect("o chip mapeia"),
            None,
        ));
        // ⇒ o chip acende no Brush …
        assert_eq!(
            selected_stroke_paint_kind(&scene, &pen),
            Some(StrokePaintKind::Brush),
            "o chip nao acende no Brush depois do clique — o artista clica e nada muda na tela"
        );
        // … e a secção sobe (é este `Some` que vira `BrushRow`).
        assert!(
            scene
                .path(id)
                .and_then(|p| p.stroke.as_ref())
                .and_then(StrokeSpec::brush)
                .is_some(),
            "a seccao Brush continua escondida — o botao da arte, os 4 sliders e o Flip ficam \
             construidos e inalcancaveis, que e' o defeito CIRCULAR desta wave"
        );
        // ⚠️ E os knobs dela já escrevem: a secção não nasce decorativa.
        assert!(apply(&mut scene, &mut h, &pen, BrushCmd::Scale(3.0)));
    }

    /// ⚠️ **Pedir o pincel que já lá está é um no-op** — senão cada clique reconstruiria o pincel e
    /// a arte, os quatro knobs e o Flip saltariam para o default.
    #[test]
    fn asking_for_the_brush_it_already_has_preserves_the_whole_law() {
        let (mut scene, pen, id) = forma_com_traco_solido(Rgba8::new(7, 8, 9, 255));
        let mut h = History::default();
        assert!(set_kind(
            &mut scene,
            &mut h,
            &pen,
            StrokePaintKind::Brush,
            None
        ));
        assert!(apply(&mut scene, &mut h, &pen, BrushCmd::Rotation(45.0)));
        let antes = scene.path(id).and_then(|p| p.stroke.as_ref()).cloned();
        let passos = h.undo_len();
        assert!(
            !set_kind(&mut scene, &mut h, &pen, StrokePaintKind::Brush, None),
            "pedir o pincel que ja' la' esta' gravou um passo"
        );
        assert_eq!(
            scene.path(id).and_then(|p| p.stroke.as_ref()).cloned(),
            antes,
            "o pincel foi reconstruido — os knobs saltam para o default a cada clique no chip"
        );
        assert_eq!(h.undo_len(), passos);
    }
}

#[cfg(test)]
#[path = "vec_stroke_paint_tests.rs"]
mod tests;
