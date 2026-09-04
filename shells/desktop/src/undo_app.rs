//! ⭐⭐ **O SHELL a operar a fila de undo** — irmão do [`super`] por RESPONSABILIDADE.
//!
//! ⚠️ **O corte foi obrigado pela INTEGRAÇÃO de 2026-09-04, e a causa nomeia a lei:** duas
//! linhas acrescentaram a este arquivo no mesmo dia — a `line/3DModeling` pôs a seleção 3D
//! a sobreviver ao `Ctrl+Z`, a `line/components` pôs a biblioteca a voltar — e nenhuma das
//! duas o via estourar, porque **sozinha nenhuma estourava** (`620 / 600` na árvore
//! combinada). *Um tecto de LOC é a única coisa neste repo que só a fusão pode acusar.*
//!
//! O que fica no pai é a **unidade** (o [`super::ProjectState`]) e a **fila**
//! ([`super::ProjectUndo`]); o que vem para aqui é quem as opera a partir da `App` — a
//! fotografia, a reposição, e o passo que nasce por DIFF uma vez por quadro.

use super::{ProjectState, field_selection_back, field_selection_ids, surviving_selection};
use ph2d_vec_scene::VecScene;

impl crate::App {
    /// Captura o estado do projeto AGORA. `None` se o gfx ainda não inicializou.
    ///
    /// ⚠️ **Ela fotografa o DOCUMENTO, não o instante.** Enquanto um motor conduz uma entidade —
    /// uma animação a tocar, o solver a simular —, o valor autorado é reposto no mundo durante a
    /// fotografia e o vivo é devolvido logo a seguir ([`crate::preview_drive`]). Sem isto, cada
    /// clique dado enquanto algo se move sozinho empilhava um passo de undo cujo conteúdo era só o
    /// relógio (ou só a pose), e o Ctrl+Z do artista gastava-se a desfazer nada.
    ///
    /// ⚠️ **E vale para o SAVE pela mesma porta**, que é a lei deste módulo (undo e save gravam a
    /// MESMA captura): gravar a meio de uma reprodução guarda a célula que o artista escolheu, não
    /// onde o ciclo calhou estar.
    #[must_use]
    pub(crate) fn capture_project(&mut self) -> Option<ProjectState> {
        // ⭐⭐⭐ **O MUNDO E O DOCUMENTO TÊM DE CONCORDAR ANTES DA FOTOGRAFIA** (report do Enio,
        // 2026-08-27: *«as peças apagadas voltaram sem pais e na posição (0,0) do mundo»*).
        //
        // ⛔ **O mecanismo, reproduzido por sonda:** a reconciliação `path ⟺ entidade` corre CEDO
        // no quadro (`render_loop/mod.rs`, antes do canvas) e o *Delete* da Hierarquia corre TARDE.
        // Logo o quadro em que uma peça vetorial é apagada **termina inconsistente**: a entidade já
        // morreu e o `VecPath` dela ainda está no documento. Esta captura fotografava esse
        // instante, e o Ctrl+Z repunha-o — aí a reconciliação do quadro seguinte via *«um path sem
        // entidade»* e **CUNHAVA** uma: `Transform::default()`, sem `ChildOf`, chamada `Path N`.
        // Isto é, exactamente, um objeto **sem pai na origem do mundo**.
        //
        // ```text
        // UNDO+1: Path 0<-None@Vec2(0.0, 0.0) | Path 1<-None@Vec2(0.0, 0.0)
        // ```
        //
        // ⚠️ **Aqui, e não no dreno do Delete:** todo `despawn` produz a mesma inconsistência, e o
        // Delete é só um dos produtores. E é aqui que ela deixa de existir para os DOIS
        // consumidores — *undo e save gravam a MESMA captura*, que é a lei deste módulo: um save
        // feito no mesmo quadro escrevia o fantasma no ficheiro.
        //
        // ⚠️ **Sim, esta chamada pode CRIAR entidades** (a 3.ª direcção do `sync` cunha a entidade
        // de um path novo). É o que o quadro seguinte faria de qualquer maneira, e fazê-lo antes da
        // fotografia é o que põe a entidade **dentro** do passo de undo em vez de fora dele.
        // Idempotente: com o mundo e o documento a par, é no-op.
        //
        // ⚠️ **Empréstimos DISJUNTOS de `self`** — `gfx` e `vec_entities` são campos diferentes.
        if let Some(gfx) = self.gfx.as_mut() {
            crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
        }
        // ⚠️ Empréstimos DISJUNTOS de `self` — o ledger e o `gfx` são campos diferentes, e é por
        // isso que os dois podem estar vivos ao mesmo tempo.
        let drive = &self.preview_drive;
        let baseline = self.undo_baseline.as_ref();
        let gfx = self.gfx.as_mut()?;
        // ⚠️ **A biblioteca é codificada AQUI, e a cache é que a torna barata** — ver
        // [`crate::project_library`]: sem ela isto custava até 28 % de um quadro, em todo quadro
        // com input.
        let library = gfx.library_cache.doc(&gfx.catalogs).clone();
        Some(ProjectState::capture(
            drive,
            &mut gfx.sim,
            &gfx.vec_scene,
            &gfx.flip,
            &gfx.guides,
            &gfx.ui_states,
            &library,
            &gfx.component_registry,
            &mut gfx.undo_capture_cache,
            // ⚠️ **O baseline é o passo anterior**, e é dele que a cena é reaproveitada quando o
            // documento vetorial não mudou. Empréstimos disjuntos: `undo_baseline` e `gfx` são
            // campos diferentes de `self`.
            baseline,
        ))
    }

    /// Aplica um estado restaurado ao App: restaura o mundo + a geometria, reconstrói
    /// a ponte, e limpa os **bits** de seleção (o restore dá ids de entidade NOVOS, então
    /// bits antigos ficam mortos). Re-arma o baseline para não re-detectar como mudança.
    ///
    /// ⚠️ **Os bits morrem; a SELEÇÃO não precisa morrer com eles.** Até 2026-07-18 esta função
    /// zerava também o `vec_pen`, e a consequência era um bug que o Enio reportou como *"o undo faz
    /// os pins sumirem, embora ainda funcionando"*: o envelope segue a deformar (o recook varre por
    /// QUERY) e o **overlay** — gaiola e pinos — é desenhado a partir da seleção, que já não existe.
    /// A ferramenta ficava funcionando e invisível.
    ///
    /// A seleção do pen é `VecPathId` e **viaja no snapshot** (a `VecScene` é parte do estado), então
    /// ela sobrevive: [`surviving_selection`] mantém os ids que ainda existem na cena restaurada, e o
    /// `sync_selection` do frame seguinte re-deriva os bits do mapa reconstruído. Nada de bits mortos
    /// atravessa — que era o perigo real que o zeramento defendia.
    pub(crate) fn apply_project(&mut self, state: &ProjectState) {
        // ⛔ **As lápides ANTES da guarda do `gfx`** — elas vivem num global, e deixá-las dentro
        // dela fazia um restauro sem GPU manter as da sessão anterior. Ver `project_library`.
        crate::project_library::apply_forgotten(&state.library);
        // ANTES do restore: o que o artista tinha selecionado, em ids ESTÁVEIS.
        let was_selected = self.vec_pen.selected_paths().to_vec();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐⭐⭐ **E a seleção 3D também, pela mesma razão e na mesma unidade** — ver
        // [`field_selection_ids`]. ⚠️ Tem de ser AQUI, antes do `restore`: depois dele os bits que a
        // seleção guarda já apontam para entidades que deixaram de existir.
        let was_field: Vec<ph2d_ecs::StableId> =
            gfx.hero_screen.as_ref().map_or_else(Vec::new, |h| {
                let bits: Vec<u64> = h
                    .gizmo
                    .selection
                    .iter()
                    .copied()
                    .chain(h.gizmo.extra_selection.iter().copied())
                    .collect();
                field_selection_ids(gfx.sim.world(), &bits)
            });
        let (vec, map, flip, flip_map) = state.restore(&mut gfx.sim, &gfx.component_registry);
        gfx.vec_scene = vec;
        gfx.flip = flip;
        gfx.guides = state.guides.clone();
        gfx.ui_states = state.ui_states.clone();
        // ⭐ A seleção 3D volta com os bits NOVOS, depois de o mundo ter sido reconstruído.
        let field_back = field_selection_back(gfx.sim.world_mut(), &was_field);
        // ⭐⭐⭐ **A biblioteca volta** — a taxonomia e as lápides (Enio, 2026-08-30).
        //
        // ⛔ **E a cache TEM de ser invalidada**, senão o quadro seguinte devolveria os bytes
        // antigos: a revisão é por-árvore e a restaurada nasce em `0`, então colidir com a que a
        // cache já viu é o caso NORMAL, não o raro.
        gfx.catalogs = crate::project_library::apply_catalogs(&state.library);
        gfx.library_cache.invalidate();
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.clear_all_selection();
            for bits in field_back {
                hero.gizmo.add_to_selection(bits);
            }
        }
        self.vec_entities = map;
        self.flip_entities = flip_map;
        self.vec_sel = crate::vec_selection::VecSelSync::default();
        self.vec_pen.clear();
        // E de volta, filtrada pelo que sobreviveu. O `vec_sel` ficou zerado de propósito: no frame
        // seguinte o `sync_selection` vê "o pen mudou" e republica os bits NOVOS no gizmo.
        let alive = surviving_selection(&was_selected, &gfx.vec_scene);
        if !alive.is_empty() {
            self.vec_pen.select_many(&alive);
        }
        // Live Shapes: uma SESSÃO de texto viva reescreveria o `VecShape` (e a pose) da
        // entidade com os params dela a cada frame — ou seja, desfaria o undo no frame
        // seguinte. O estado restaurado é a verdade: a sessão termina (o objeto de texto
        // continua lá, editável pela seleção).
        self.vec_text_edit = None;
        self.vec_text_last_target = None;
        // O cache de poses dos rótulos é a memória de "o que EU escrevi no frame passado". O
        // restore põe poses novas no mundo sem passar por ele — deixá-lo faria o passe ler a
        // pose restaurada como um arrasto do usuário. É inócuo (o estado restaurado é
        // auto-consistente, e re-absorver devolve o MESMO offset), mas zerar é o honesto: a
        // memória não é mais de nada. O arm de rótulo pendente morre com a sessão de texto.
        self.vec_label_poses.clear();
        self.vec_label_pending = None;
        // O memo do Offset vivo é chaveado por `VecPathId`, e o restore RECICLA os ids: um id
        // que volta descrevendo outra forma acertaria o memo e desenharia o offset da forma
        // ANTERIOR sobre a nova, sem erro nenhum. O espelho do painel morre pela mesma razão
        // (a forma que ele espelhava pode ter deixado de ter offset).
        self.offset_live.forget();
        // O Pattern vivo pela mesma razão do offset.
        self.pattern_live.forget();
        self.contour_live.forget();
        // A SIMETRIA viva pela MESMA razão (id reciclado; o memo é chaveado por `VecPathId`).
        self.symmetry_live.forget();
        self.profile_live.forget();
        // O FX raster vivo (plano 24) pela mesma razão: a cena inteira mudou debaixo do cozimento.
        self.fx_live.forget();
        self.fx_silhouette.forget();
        self.vec_offset_mirrored = None;
        // A mesma forma (mesmo id) pode voltar com OUTROS parâmetros — zerar o alvo
        // força a re-semente dos sliders, senão o painel seguiria mostrando o valor
        // que o undo acabou de desfazer.
        self.vec_shape_last_focus = None;
        // ⚠️ **E o latch de «armado» também**: o restore repõe outra selecção, e um latch de antes
        // do undo descreveria um gesto que já não aconteceu.
        self.vec_shape_armed = false;
        self.vec_shape_armed_target = None;
        // O Colorize ao vivo guarda a base congelada de um desenho que este restore acaba de
        // substituir — re-Aplicar sobre ela apagaria o estado restaurado. A sessão termina.
        self.flip_colorize.end_live();
        self.undo_baseline = Some(state.clone());
        self.title_dirty = true;
    }

    /// Desfaz (ou refaz) um passo da fila global: empurra o estado atual pro outro
    /// lado e aplica o restaurado.
    pub(crate) fn apply_undo(&mut self, redo: bool) {
        let Some(current) = self.capture_project() else {
            return;
        };
        let restored = if redo {
            self.undo.redo(current)
        } else {
            self.undo.undo(current)
        };
        if let Some(state) = restored {
            self.apply_project(&state);
            if Self::undo_log_on() {
                eprintln!("[undo] {} aplicado", if redo { "redo" } else { "undo" });
            }
        } else if Self::undo_log_on() {
            eprintln!("[undo] {} sem passo", if redo { "redo" } else { "undo" });
        }
    }

    /// **O log do undo está ligado?** Cacheado — os três sítios que o consultam correm por
    /// passo de undo, e ler o ambiente é caro o suficiente para não o fazer em cada um.
    pub(crate) fn undo_log_on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("PH2D_UNDO_LOG").is_some())
    }

    /// Chamado UMA vez por frame, depois de `render_frame` (com `self` livre e o
    /// estado já reconciliado pelo `sync`). Duas coisas, nesta ordem:
    ///
    /// 1. **Ctrl+Z/Y** pendente: aplica e re-arma o baseline (então o passo 2 vê o
    ///    estado restaurado == baseline e não registra um passo espúrio).
    /// 2. **Auto-commit**: se houve input neste frame e nenhum gesto está em curso,
    ///    compara o estado atual com o baseline; se mudou, o baseline antigo (o
    ///    estado-pré) vira um passo de undo e o novo vira o baseline. É o ponto único
    ///    que torna "tudo que o usuário fizer" desfazível, por diff — sem instrumentar
    ///    cada ação.
    pub(crate) fn post_frame_undo(&mut self) {
        // ⚠️ **PRIMEIRO, e antes de qualquer captura:** o ledger da condução esquece quem os
        // motores não declararam neste quadro. Quem parou volta a ser documento **já nesta
        // fotografia** — é isso que faz uma corrida inteira colapsar em UM passo, em vez de zero.
        self.preview_drive.settle();
        // ⭐⭐⭐ **E o que o módulo 3D autorou SEM evento** (W115) — a forma que a paleta escolheu,
        // a escultura que o diálogo carregou. Elas chegam por **pedido servido noutro quadro**, e
        // sem esta metade nasciam **sem passo próprio**, fundindo-se na acção seguinte do artista.
        //
        // ⚠️ **Tirada em TODO quadro, ao lado da outra** — deixá-la pousada faria a próxima
        // supressão legítima registar um passo que já foi registado.
        let had_input = std::mem::take(&mut self.any_input_this_frame)
            | crate::field3d_smoke::take_authored_change();
        // O clique no botão Undo/Redo da barra entra pela MESMA porta do Ctrl+Z (que pode
        // rotear para o Áudio, o Painter, o global ou o image-edit). Ele arma o
        // `undo_request` logo abaixo, e o passo é aplicado ainda neste frame.
        if let Some(redo) = self.undo_button.take() {
            self.undo_or_redo(redo);
        }
        if let Some(redo) = self.undo_request.take() {
            self.apply_undo(redo);
            return;
        }
        // Estabelece o baseline no PRIMEIRO frame (gfx pronto), antes de qualquer
        // ação. Sem isto a primeira ação não teria pré-estado e não seria desfazível.
        if self.undo_baseline.is_none() {
            self.undo_baseline = self.capture_project();
            return;
        }
        // Um gesto em andamento (botão pressionado) muta a cada Move; espera o fim
        // para não registrar um passo por frame.
        //
        // ⚠️ **Um recálculo assíncrono pendente conta como gesto em andamento** — é a MESMA
        // frase acima aplicada a um trabalho que continua depois do botão soltar. O ajuste
        // Trap/Bleed do Colorize roda fora da thread de UI (`09 §7.2`, 304 ms/tique), então
        // soltar o slider registraria um passo com o resultado ANTIGO e a chegada do worker
        // registraria um segundo: dois Ctrl+Z para um arrasto.
        // ⚠️ **Uma PREVIEW ao vivo tambem conta como gesto em andamento** — e a MESMA frase, num
        // trabalho que o produto faz sozinho: uma transição de ESTADO de UI (plano UI/UX W7) põe
        // a cena numa pose DIFERENTE a cada quadro, então sem a supressão um Show de 150 ms
        // viraria nove passos de undo. Quando ela chega, a cena está numa pose AUTORADA e o diff
        // registra um — o preço certo de *"eu mostrei o hover"*.
        // ⚠️ **O gizmo 3D de modelagem tem de dizer-se por conta própria** (ADR-0161 W6): o gancho
        // de ponteiro dele consome o `Down` e volta ANTES da linha que escreve o `held_button`, então
        // a condição acima — que está certa — não alcançava aquele gesto. Sem esta, arrastar uma
        // seta registava um passo de undo POR QUADRO.
        //
        // ⭐⭐⭐ **E o motivo tem NOME** (W114) — porque a pergunta que um report de *«o undo pula
        // etapas»* faz é exactamente *«qual destas cinco comeu o meu passo?»*.
        //
        // ⚠️ **Uma mudança suprimida NÃO se perde: ela funde-se no PRÓXIMO passo**, porque o
        // `undo_baseline` só é substituído quando um passo é registado. ⇒ duas acções viram um
        // `Ctrl+Z` só, que é o sintoma que o artista descreve. *Um passo suprimido e um passo
        // ausente leem-se iguais de fora, e as causas são opostas.*
        let motivo = if !had_input {
            Some("sem entrada neste quadro")
        } else if self.held_button.is_some() {
            Some("botao do rato em baixo")
        } else if crate::field3d_smoke::gesture_in_progress() {
            Some("arrasto do gizmo 3D em curso")
        } else if self.flip_colorize.live_busy(self.flip_style.as_ref()) {
            Some("colorize a recalcular")
        } else if self.ui_state_live {
            Some("transicao de estado de UI ao vivo")
        } else {
            None
        };
        if let Some(motivo) = motivo {
            // ⚠️ **A captura só acontece com o log LIGADO** — ela é o passo caro do quadro, e
            // pagá-la em toda supressão seria trocar um diagnóstico por uma regressão de relógio.
            if Self::undo_log_on()
                && let Some(atual) = self.capture_project()
                && self.undo_baseline.as_ref() != Some(&atual)
            {
                eprintln!(
                    "[undo] ⛔ o documento MUDOU e o passo foi SUPRIMIDO — motivo: {motivo}                      (ela vai FUNDIR-SE no proximo passo)"
                );
            }
            return;
        }
        let Some(current) = self.capture_project() else {
            return;
        };
        if self.undo_baseline.as_ref() == Some(&current) {
            return; // nada mudou desde o último passo
        }
        if Self::undo_log_on() {
            let base = self.undo_baseline.as_ref();
            // ⭐ O que a captura INCREMENTAL fez (F2): quantas linhas o pré-filtro acusou,
            // quantas de facto mudaram, e **quantos bytes** o passo custa. ⚠️ Um `sujas` muito
            // maior que `reserializadas` é o falso positivo do `DerefMut` — alguém reescreve o
            // que já lá estava, e a cura é `set_if_neq` em quem escreve.
            let cap = self
                .gfx
                .as_ref()
                .map(|g| g.undo_capture_cache.last_report())
                .unwrap_or_default();
            eprintln!(
                "[undo]   captura: sujas={} reserializadas={} nascidas={} mortas={} linhas={} delta={} B",
                cap.dirty, cap.reserialized, cap.spawned, cap.despawned, cap.rows, cap.delta_bytes
            );
            eprintln!(
                "[undo] passo registrado (fila undo={}, {} sob conducao) — diff: world={} vec={} flip={}",
                self.undo.depth() + 1,
                // ⚠️ Quantas entidades um motor está a escrever AGORA. Um passo registado com este
                // número **alto** é a pergunta certa a fazer: ou a condução não foi declarada, ou o
                // artista de facto editou (`crate::preview_drive`).
                self.preview_drive.len(),
                base.is_some_and(|b| b.world != current.world),
                base.is_some_and(|b| b.vec != current.vec),
                base.is_some_and(|b| b.flip != current.flip),
            );
            if let Some(b) = base
                && b.vec != current.vec
            {
                let ids = |v: &VecScene| v.paths().iter().map(|p| p.id).collect::<Vec<_>>();
                let (a, c) = (ids(&b.vec), ids(&current.vec));
                let mut sa = a.clone();
                let mut sc = c.clone();
                sa.sort_unstable();
                sc.sort_unstable();
                eprintln!(
                    "[undo]   vec: base={a:?} atual={c:?} · mesmos ids={} · só a ORDEM={}",
                    sa == sc,
                    sa == sc && a != c
                );
            }
        }
        if let Some(base) = self.undo_baseline.replace(current) {
            self.undo.push_undo(base);
        }
    }
}
