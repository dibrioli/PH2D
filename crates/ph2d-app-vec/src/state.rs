//! ⭐⭐ **O estado da família `vec`, com um dono só** — W2/L4 A2 (os primeiros 19) + A9 (os outros
//! 50), ADR-0075.
//!
//! # ⭐ A9 (2026-09-12): a `App` já não tem campo `vec_*` solto
//!
//! O campo da `App` chama-se `vec` (era `vec_state`), e cada campo daqui perdeu o prefixo:
//! `app.vec_pen` passou a `app.vec.pen`. Os 50 que faltavam entraram pela régua da auditoria de
//! arquitectura, que pergunta pelo **assunto** de cada campo e não por quem o lê:
//!
//! - **morto** ⇒ apagado antes de agrupar (o `vec_history` era escrito e nunca lido);
//! - **assunto da família + tipo de crate** ⇒ aqui;
//! - **assunto da família + tipo da shell** ⇒ o TIPO desceu primeiro, depois o campo. Eram oito
//!   (`VecTextEdit`, `ConnectorDrag`, `HandleDrag`, `TrimHit`, o `Grab` da largura, `VecSelSync`,
//!   `LabelPoses`, `BuildSession`); os outros doze «tipos da shell» eram FACHADAS de módulos desta
//!   crate — medido, não suposto.
//!
//! ⚠️ **A régua de baixo dizia que `vec_entities` e `vec_pen` não eram estado da família** («são o
//! documento e a árvore, que a shell inteira conduz»). A A9 mediu o preço que essa frase temia: com
//! os campos numa struct só, o que parte é emprestar o `VecState` **inteiro** ao lado de um campo
//! dele — e isso acontecia num sítio só, o `smoke_bone::bind`, curado lendo o mapa de dentro do
//! `st`. Os empréstimos por campo (`&mut app.vec.pen` ao lado de `&app.vec.entities`) são
//! disjuntos como sempre foram. *Muitos leitores não mudam o assunto de um campo.*
//!
//! Dois grupos têm tipo próprio porque nascem com valores que não são os do `Default` dos tipos:
//! [`TextKnobs`] (o estilo de texto corrente) e [`ExpandKnobs`] (o par que o frame viu no quadro
//! anterior, que tem de nascer igual ao do painel).
//!
//! # A régua da W2/L4, que escolheu os primeiros 19 (história)
//!
//! `App` tem ~401 campos públicos e **75** deles chamam-se `vec_*`. Quantos podem sair não é uma
//! escolha — mede-se por duas perguntas, nesta ordem:
//!
//! 1. **quem os lê?** `app_state.rs` (a declaração) e `main.rs` (o construtor) não contam. Restam
//!    **30** campos cujos únicos consumidores de fora da família são os dois roteadores da shell
//!    (`render_loop/mod.rs`, `input_dispatch.rs`) — ou ninguém. Os outros 45 estão espalhados, e
//!    três deles são o substrato do módulo: `vec_entities` (189 ficheiros), `vec_scene` (131),
//!    `vec_pen` (36). ⛔ Esses não são «estado da família»: são o documento e a árvore, que a shell
//!    inteira conduz.
//! 2. **o TIPO deles vem de uma crate?** Dos 30, **19** sim (11 de `ph2d_vec_*`, 8 primitivos) e
//!    **11 não** — o tipo mora na shell (`crate::blend_live::BlendSpines`,
//!    `crate::vec_bucket::BucketCache`, `crate::morph_live::MorphPlans`, …). Um campo cujo tipo
//!    está na shell **não pode** viver aqui, e agrupá-lo num `VecState` da shell seria trocar 11
//!    campos por 1 sem mover uma linha de lugar.
//!
//! ⇒ estes 19. Os 11 bloqueados estão nomeados no handoff de integração: cada um espera que o
//! TIPO dele saia, não que alguém o agrupe.
//!
//! # ⚠️ Porque não é um recurso do ECS
//!
//! O ADR-0075 manda desacoplar por ECS, e a primeira leitura disto é «ponha num componente». ⛔
//! Errado aqui, e o mecanismo está escrito no [`undo.rs`](../../../shells/desktop/src/undo.rs): o
//! `ProjectState` que o undo fotografa é `{WorldSnapshot + VecScene + FlipDoc + …}`, e o
//! `WorldSnapshot` **cobre toda entidade com componente registado**. Um `ShapeTool` ou um `Pencil`
//! como componente entraria na fotografia ⇒ **cada nudge do lápis viraria um passo de undo**.
//!
//! Estado de FERRAMENTA não é estado de documento. O precedente da casa é o mesmo: `prop_state`,
//! `walk_state`, `z_walk_state` e `input` já são sub-structs num campo de `App`.
//!
//! # ⚠️ `Default` é derivado, e isso foi VERIFICADO campo a campo
//!
//! Os 19 inicializadores do `main.rs` eram `false` / `0` / `None` / `Vec::new()` /
//! `Pencil::default()` / **`ShapeTool::new()`** — e esse último é literalmente `Self::default()`
//! ([`shape.rs`](../../ph2d-vec-edit/src/shape.rs)). *Um `derive(Default)` sobre um campo cujo
//! `new()` faz mais que o `Default` muda o arranque do app em silêncio* — aqui não faz, e é por
//! isso que se conferiu antes de derivar.
//!
//! Na A9 a mesma conferência correu sobre os 50: `PenTool::new()`, `MorphPlans::new()`, os
//! `BTreeMap::new()` (`SideCache`, `BlendSpines`, `LabelPoses`, `LiveGeometry`) e o
//! `PencilHand::default()` são o `Default` deles. Os únicos que não eram — os oito números do
//! texto e o par do Expand — ganharam `Default` manual nos grupos acima, e o `VecState` continua a
//! derivar o seu.

/// Os campos da família `vec` que saíram de [`App`](../../../shells/desktop/src/app_state.rs).
///
/// O nome de cada um perdeu o prefixo `vec_`, que agora é o do campo em `App` (`vec`).
#[derive(Default)]
pub struct VecState {
    /// A cena da APARÊNCIA do objecto (`PH2D_VEC_APPEARANCE_SMOKE`) já montou.
    pub appearance_smoke_done: bool,

    /// O **overlay ordenado** de TODOS os blends (passos + fontes reempilhadas, em z), cozido em
    /// MUNDO a cada frame pelo `blend_live::recook` e desenhado por
    /// `ph2d_vec_render::draw_blend_overlay`. Não está na cena (não é pickável) — é o que torna o
    /// blend UM objeto, e não N. Runtime-only.
    pub blend_overlay: Vec<ph2d_vec_scene::VecPath>,

    /// **Pick Shapes** (ADR-0128 C2b): as formas fechadas que o artista clicou **na ordem**, no
    /// modo `DrawMode::PickBlend`. O botão Blend as liga nessa sequência (em vez da ordem de z), e a
    /// prévia do spine as costura no canvas. Esvaziado ao criar o blend ou ao sair do modo.
    /// Runtime-only.
    pub blend_picks: Vec<ph2d_vec_scene::VecPathId>,

    /// Em que TEMPO está a cena dos ossos (`PH2D_VEC_BONE_SMOKE`): `0` monta, `1` deixa o `sync`
    /// dar entidade às formas, `2` prende, `3` acabou. ⚠️ Ela precisa de dois quadros porque
    /// prender exige a ENTIDADE da forma, e quem a cria corre depois do prólogo.
    pub bone_smoke_step: u8,
    /// ⭐ Latch one-shot da [`crate::smoke_bone_paint`] — o canvas do Painter preso a ossos.
    ///
    /// ⚠️ **Ele mora AQUI e não na `App`**, e não por arrumação: as duas catracas da shell
    /// (`the_app_only_sheds_fields` e `fn_loc_caps`) disseram-no pelo nome no mesmo minuto —
    /// *«um campo novo tem DONO: ponha-o no estado da família do assunto dele»*.
    pub bone_paint_smoke_done: bool,

    /// ⭐ **A IMAGEM da cena de osso** — os bits da sprite e a raiz do esqueleto dela.
    ///
    /// ⚠️ **Slot próprio, e não uma 4.ª entrada no `_pend`:** aquele guarda `VecPathId`, e uma
    /// imagem não é um caminho. Enfiá-la lá pediria um id inventado, e o passo que espera as
    /// entidades (`vec_entities::contains_key`) procuraria por ele para sempre.
    pub bone_smoke_img: Option<(u64, Option<ph2d_ecs::Entity>)>,
    /// As três peças da cena entre os dois tempos: `(forma, raiz do esqueleto dela)`.
    pub bone_smoke_pend: Option<[(ph2d_vec_scene::VecPathId, Option<ph2d_ecs::Entity>); 3]>,

    /// In-app path clipboard for Vector Ctrl+C/X/V — a clone of the copied path
    /// (geometry + style, id-less). `None` until the first copy/cut.
    pub clipboard: Option<ph2d_vec_scene::VecClip>,

    /// A forma cujo CONTOUR o painel está espelhando (os três sliders + os dois trios). Mesmo
    /// papel do [`Self::offset_mirrored`] e pela mesma razão: o `paint` lê o STORE primeiro
    /// (senão o número saltaria durante o arrasto), então sem uma borda que reescreva o store na
    /// troca de seleção, escolher outra forma mostraria os valores da anterior. Runtime-only.
    pub contour_mirrored: Option<ph2d_vec_scene::VecPathId>,

    /// A **LINHA DE CORTE recém-começada** (o press da caneta em modo `Cut`), esperando a
    /// entidade dela nascer no `vec_entities::sync` para receber o `VecCutPath`. Espelho exato do
    /// `vec_connect_pending` e do `vec_blend_pending`, e pela mesma razão: sem esta fila de um
    /// item, a lâmina ficaria na cena como um caminho comum — desenhada como arte, exportada
    /// como arte, e fora do alcance do botão que existe para a descartar.
    pub cut_pending: Option<ph2d_vec_scene::VecPathId>,

    /// A cena do **fade vetorial** já montou? (`PH2D_VEC_FADE_SMOKE`, uma vez por sessão.)
    pub fade_smoke_done: bool,

    /// Gradient group: the gradient handle currently being DRAGGED on-canvas —
    /// a multi-point point OR a linear/radial endpoint (`None` = not dragging).
    pub grad_drag: Option<ph2d_vec_render::GradHandle>,

    /// The selected gradient handle (drives the overlay highlight + the Remove-
    /// point / Influence / Jitter targets, via [`GradHandle::point`]). `None` = none.
    pub grad_selected: Option<ph2d_vec_render::GradHandle>,

    /// **O LÁPIS** — a mão livre (W1 do plano 25). Irmão do [`Self::pen`] e do [`Self::shape`]: a shell
    /// converte tela→mundo e ele acumula as amostras, decima e ajusta a spline AO VIVO. O path
    /// vivo mora na cena desde o press (o padrão da `ShapeTool`), então preview, undo de um passo
    /// e seleção no release saem do caminho normal.
    pub pencil: ph2d_vec_edit::Pencil,

    /// "Set Center" armado: a próxima pressão no canvas põe a ORIGEM da forma
    /// selecionada ali (ADR-0112). Desarma no press.
    pub pivot_edit: bool,

    /// A forma cujo **perfil de largura** os quatro knobs `W *` estão espelhando (ADR-0148).
    /// Mesmo papel do [`Self::offset_mirrored`] e pela mesma razão: a borda é a SELEÇÃO, e
    /// sem ela escolher uma forma com perfil vivo mostraria os knobs globais do painel.
    pub profile_mirrored: Option<ph2d_vec_scene::VecPathId>,

    /// ADR-0108 Fase 1: drag-to-size shape drawing (Rectangle / Ellipse /
    /// Polygon). Sibling of `vec_pen`; the shell routes canvas input to one or
    /// the other by `vec_draw_mode` (mirrored from the tool by `vector_bridge`).
    pub shape: ph2d_vec_edit::ShapeTool,

    /// A cena da PILHA DE APARÊNCIA (`PH2D_VEC_STACK_SMOKE`) já montou.
    pub stack_smoke_done: bool,

    /// **O eixo de SESSÃO da simetria**, em MUNDO — a linha que aparece no instante em que o botão
    /// liga, com a cena vazia e nada seleccionado.
    ///
    /// ⚠️ Mora aqui e não na ferramenta porque nasce do centro do **ecrã** (*"a tela é a referência
    /// para a posição inicial da linha"*), e só a shell tem câmera. `None` = por semear: a
    /// semeadura acontece na aresta desligado→ligado, e é isso que faz a linha ficar no lugar ao
    /// longo de quantos desenhos o artista quiser.
    pub symmetry_origin: Option<[f64; 2]>,

    /// A alça do TEXTO EM CAMINHO está sob arrasto? (W5) Armada no press de Node, limpa no
    /// release. Runtime-only: um arrasto vivo não é documento — o resultado (`start_offset`) é,
    /// e vive no `VecTextPath`. Um booleano e não um alvo porque há UMA alça (o `start_offset` do
    /// texto selecionado); qual texto vem da seleção, como no painel. Ver [`crate::text_ride`].
    pub textpath_handle_drag: bool,

    /// A geometria do realce, em MUNDO — derivada do [`Self::trim_hit`] no mesmo quadro.
    pub trim_piece: Vec<ph2d_vec_scene::VecPath>,

    /// A sessão de **Blend** aberta (as duas fontes + o escape + os passos produzidos).
    /// É ela que faz o *Rotate Match* re-rodar na hora, em vez de o artista ter de desfazer e
    /// adivinhar (`crate::blend`).
    pub blend: Option<crate::blend::BlendSession>,

    /// A sequência de z que o Blend pediu (**fundo → topo**), à espera das ENTIDADES.
    ///
    /// O blend cria os passos no documento, mas quem manda no z é a **árvore** (ADR-0110) — e a
    /// entidade de um path novo só nasce no `vec_entities::sync`, mais adiante no mesmo frame.
    /// Então o pedido espera aqui e é aplicado logo depois do sync (`vec_entities::restack`).
    ///
    /// É uma LISTA de sequências, uma por blend: o Expand (ADR-0128 Fase D) age sobre todos os
    /// blends que a seleção toca, e cada um pede a sua própria fatia contígua de z. Guardar só uma
    /// seria um corte silencioso — o 2º blend sairia com a pilha errada e ninguém saberia.
    pub restack: Vec<Vec<ph2d_vec_scene::VecPathId>>,

    /// ADR-0108 cutover: the Pen of the Vector drawing tool. Operates on the
    /// document scene `AppGfx.vec_scene` (document ≠ tool); driven by the shell
    /// input hooks while the `vector` tool is active, and styled each frame from
    /// the tool's palette via `render_loop::vector_bridge`.
    pub pen: ph2d_vec_edit::PenTool,

    /// The tool's current draw-mode + shape parameters, mirrored each frame from
    /// the `VectorTool` (the input dispatch can't downcast — that lives in the
    /// allowlisted bridge). Decides pen vs shape routing + sizes the shapes.
    pub draw_config: ph2d_tool_vector::VectorDrawConfig,

    /// **A mão FILTRADA do lápis** (o estabilizador). Semeada no press, avançada em cada move —
    /// a shell é quem a avança, porque é ela que possui a entrada; e o filtro (`lazy_mouse_step`)
    /// mora numa crate que o `Pencil` não pode ver (`pencil_input`).
    pub pencil_hand: crate::pencil_input::PencilHand,

    /// ADR-0108 Fase 1: o gesto de REGIÃO do modo Node (px de tela, o mesmo `(f32, f32)` do
    /// `last_pointer`) — arrastar do vazio com a ferramenta Vector activa. `None` = parado; no
    /// release dirige `box_select_with` ou `lasso_select_with`, conforme a forma que ele congelou.
    pub marquee: Option<crate::marquee::VecMarquee>,

    /// **O conector em construção** (modo `DrawMode::Connect`, Down..Up). O path já está na
    /// cena desde o Down e o componente já está na entidade: o "preview" do arrasto É o
    /// conector de verdade, re-cozido pela MESMA `route` a cada frame — o que se vê é o que
    /// se obtém, e não há um segundo caminho de desenho para divergir. `None` = sem gesto.
    pub connect: Option<crate::connector_drag::ConnectorDrag>,

    /// O arrasto de uma **alça de ponta** de conector (os dois círculos que reposicionam onde a
    /// linha encosta na forma). `None` fora do gesto.
    pub conn_handle: Option<crate::connector_drag::HandleDrag>,

    /// O conector recém-FECHADO (Up), esperando a entidade dele nascer no `vec_entities::sync`
    /// para receber o componente. Um clique rápido (Down e Up no mesmo frame) fecha o gesto
    /// antes de qualquer `sync` — sem esta fila de um item, a linha ficaria na cena sem
    /// `VecConnector`: um traço inerte que não segue ninguém.
    pub connect_pending: Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecConnector)>,

    /// O lado por onde cada ponta de cada conector saiu no frame anterior — a memória da
    /// histerese de `side_towards` (sem ela a saída pisca na diagonal). Runtime-only.
    pub connect_sides: crate::connector_live::SideCache,

    /// O **Blend Object recém-criado** (ADR-0128), esperando a entidade dele nascer no
    /// `vec_entities::sync` para receber o `VecBlend`. Espelho do `vec_connect_pending`: o spine
    /// já está na cena, e sem esta fila de um item a linha ficaria sem componente — um path
    /// invisível que não interpola ninguém.
    pub blend_pending: Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecBlend)>,

    /// O morph recém-criado, à espera de a entidade dele nascer no `sync` (espelho do blend).
    pub morph_pending: Option<(ph2d_vec_scene::VecPathId, ph2d_ecs::VecMorph)>,

    /// ⭐⭐ **O CONJUNTO de estados à espera da entidade dele nascer** (plano 32 W8).
    ///
    /// ⚠️ **Slot próprio e não um campo a mais no irmão acima:** o payload é outro — aquele leva um
    /// componente, este leva também a **lista de quem vai ser reparentado e escondido**. Fundi-los
    /// obrigaria o `morph_live::upkeep` a saber de reparentar, que não é o assunto dele.
    pub morph_set_pending: Option<ph2d_vec_entities::morph_set::MorphSetPending>,

    /// O canto da gaiola sob arrasto agora (`(bits do CONTAINER do envelope, índice 0..4)`), ou
    /// `None` — o gesto de Fatia 1 (ADR-0129), armado no press de Node e limpo no release. O alvo é a
    /// ENTIDADE (o container não tem path — Fatia 3). Runtime-only: um arrasto vivo não é documento (o
    /// resultado, os `corners`, é; e vive no `VecEnvelope`). Ver o `envelope_gesture` da shell.
    pub envelope_drag: Option<(u64, usize)>,

    /// Qual das duas alças do PATTERN (Start/End, plano 23 W4) está sob arrasto, se alguma. Armada
    /// no press de Select, limpa no release. Runtime-only: o arrasto não é documento — o resultado
    /// (`start_offset`/`end_offset`) é, e vive no `VecPatternPath`. Um `Option` e não um booleano
    /// porque há DUAS alças. Ver [`crate::pattern_live::handle`].
    pub patternpath_handle: Option<crate::pattern_live::PatternHandle>,

    /// **O Picker de caminho-guia armado** (Enio 2026-07-23): `Some` enquanto se espera o clique que
    /// escolhe o guia de um motivo (Pattern) ou de um texto (Text on Path). A fonte foi capturada no
    /// arm; o clique seguinte no canvas prende, um clique no vazio desiste. Sair do modo Select (ou
    /// da tool) o limpa. Runtime-only: o pick não é documento — o vínculo que ele cria é.
    pub path_pick: Option<crate::pick::PathPick>,

    /// **O que já foi propagado entre o painel autorado e o mundo** (plano UI/UX W8b.4).
    ///
    /// ⚠️ O memo do reconcile de duas direções: sem ele o ponteiro e o mundo sobrescreviam-se em
    /// frames alternados e o slider tremeria. Ver `widget_value::reconcile`.
    pub widget_applied: crate::widget_value::Applied,

    /// O `Plan` de cada morph enquanto a relação não muda. Runtime-only: derivável das fontes,
    /// fora do save e do undo.
    pub morph_plans: crate::morph_live::MorphPlans,

    /// O spine AUTOMÁTICO que o `blend_live::recook` escreveu por último, por blend — a memória que
    /// detecta a edição do spine (modo Node) para marcar `spine_authored` (ADR-0128). Runtime-only.
    pub blend_spines: crate::blend_live::BlendSpines,

    /// O **hospedeiro** do rótulo que o duplo-clique acabou de abrir, esperando a 1ª letra
    /// materializar o objeto de texto (e com ele a entidade) para receber o `VecLabel`. Um
    /// rótulo nasce VAZIO — sem geometria não há path, sem path não há entidade, e sem entidade
    /// não há onde pendurar o vínculo. Espelho do `vec_connect_pending`.
    pub label_pending: Option<ph2d_vec_scene::VecPathId>,

    /// A pose que o passe dos rótulos escreveu no frame anterior, por rótulo. É o que distingue
    /// "o hospedeiro se moveu" (dirigir) de "o USUÁRIO arrastou o rótulo" (absorver no offset) —
    /// um `Transform` diferente do que gravamos só pode ter vindo do gizmo. Runtime-only.
    pub label_poses: crate::state::LabelPoses,

    /// **O MAPA QUE FOI DESENHADO** — a fusão dos nove produtores de `LiveGeometry`, guardada tal
    /// como o `ph2d_vec_render::dispatch` a consumiu.
    ///
    /// ⚠️ Ela existe porque o `vec_gizmo_pick` declara, no próprio doc, que a pergunta *"o que
    /// está desenhado aqui?"* é feita ao **MESMO mapa** que o `dispatch` recebe — e a fiação
    /// contradizia-o: os seis sítios de pick passavam só o `offset_live`, então tudo o que os
    /// outros oito produtores desenham (uma simetria, uma largura viva, um contorno, um padrão,
    /// uma instância, uma booleana, um alinhamento) era **visível e não-clicável**.
    ///
    /// ⚠️ **Ela é do frame ANTERIOR, e isso é a semântica CERTA, não uma concessão:** o artista
    /// clica no que VÊ, e o que ele vê é o último frame desenhado. O input corre antes do frame,
    /// então um mapa "deste frame" não existe quando o clique chega.
    ///
    /// ⚠️ E ela não introduz classe nova de obsolescência: o `offset_live` que o pick lia até aqui
    /// é escrito pelo **mesmo bloco por-frame** que monta esta fusão, logo os dois têm exactamente
    /// a mesma frescura — o que muda é o número de produtores, de um para nove.
    pub live_drawn: ph2d_vec_render::LiveGeometry,

    /// **Os fatos DERIVADOS por frame sobre os caminhos** — os intervalos das molduras e as poses
    /// que o auto layout deu —, publicados pelo passe de DESENHO para quem vier depois.
    ///
    /// ⚠️ **Existe porque quem APONTA não pode reconstruí-los.** O hit-test monta o `VecViewState`
    /// dele do zero a cada evento (`vec_entities::view_state`), e aquela porta só sabe o que a
    /// ÁRVORE diz — escondido e travado. Os intervalos e as poses são resultado do passe de
    /// layout, que roda no desenho; sem os republicar, todo consumidor de ponteiro os vê VAZIOS e
    /// decide como se nenhuma moldura existisse.
    ///
    /// Um frame de atraso é a semântica CERTA, não uma concessão: o artista clica no que está na
    /// tela, e o que está na tela é o último frame desenhado.
    pub view_derived: ph2d_vec_scene::VecViewState,

    /// Os knobs `(Corner, Side)` do painel no frame ANTERIOR. É o que distingue *"o artista
    /// clicou um chip"* (retunar os offsets vivos da seleção) de *"o painel está no valor de
    /// sempre"* — sem a borda, todo frame reescreveria o componente de toda forma selecionada.
    pub expand_knobs: ExpandKnobs,

    /// A forma cujo offset vivo o painel está ESPELHANDO (slider + chips). Trocar a seleção
    /// republica; sem isto, escolher uma forma offsetada mostraria os knobs globais do painel e
    /// o chip mentiria sobre o que está na tela. Runtime-only.
    pub offset_mirrored: Option<ph2d_vec_scene::VecPathId>,

    /// A alça de LARGURA agarrada agora (plano 25 §5). Runtime-only: o que o documento guarda é o
    /// `VecStrokeProfile`, e isto é só qual parada o dedo está a mover.
    pub width_grab: Option<crate::width_grab::Grab>,

    /// O caminho de REFERÊNCIA da cena de smoke do Width Tool, à espera de ganhar o perfil no
    /// frame seguinte (o componente precisa de uma entidade, e ela nasce no `sync`).
    pub width_ref: Option<ph2d_vec_scene::VecPathId>,

    /// **O Shape Builder em curso** (modo `DrawMode::Build`). Guarda o arranjo das formas
    /// selecionadas + as faces já pintadas. `None` fora do modo, ou com menos de 2 formas
    /// fechadas selecionadas (aí não há região para pintar).
    ///
    /// Vive entre frames de propósito: o arranjo MEMOIZA a geometria de cada região
    /// visitada, e reabri-lo por frame faria todo hover pagar a booleana de novo.
    pub build: Option<crate::shape_build::BuildSession>,

    /// Snap + grid settings of the Vector tool (edited by the panel's Snap section).
    pub snap: crate::snap::VecSnapSettings,

    /// Snap targets of the CURRENT gesture — collected once at Down (the scene's
    /// shape doesn't change mid-drag; only the dragged thing, which is excluded).
    pub snap_targets: ph2d_vec_edit::SnapTargets,

    /// Smart guides to draw this frame (cleared at Up / when nothing snapped).
    pub snap_guides: Vec<ph2d_vec_render::Guide>,

    /// Edição de texto em curso (modo `DrawMode::Text`): o ponto de inserção, o
    /// conteúdo e os glyphs já na cena. `None` = sem cursor de texto ativo.
    pub text_edit: Option<crate::text_edit::VecTextEdit>,

    /// ⭐ **O ESTILO de texto corrente** — o que o painel edita e a PRÓXIMA sessão herda.
    /// Um grupo e não oito campos soltos porque o assunto é um só, e porque os valores com
    /// que nasce não são os do `Default` dos tipos (ver [`TextKnobs`]).
    pub text: TextKnobs,

    /// Último clique primário no canvas (instante + posição de tela) — só para detectar
    /// o DUPLO-clique que reabre um texto no modo Select. O canvas não emite
    /// `DoubleClick` (o evento do chrome é por-widget), então o par é rastreado aqui.
    pub last_canvas_click: Option<(std::time::Instant, (f32, f32))>,

    /// O último ALVO das configs de texto do painel (sessão ou objeto selecionado).
    /// Quando muda, a shell publica a semente dos sliders — uma vez (senão o seed
    /// brigaria com o arrasto).
    pub text_last_target: Option<ph2d_vec_scene::VecPathId>,

    /// O último ALVO das seções de FORMA do painel (a forma viva paramétrica
    /// selecionada). Mesma regra do texto: quando muda, a shell semeia os sliders e a
    /// tool adota os params — uma vez. Zerado no restore (undo/load) porque a forma
    /// pode ter voltado com outros params sob o MESMO id: sem re-semear, o painel
    /// mostraria o valor desfeito.
    /// ⚠️ O PAR `(alvo, tipo)`, nunca só o alvo: sem o tipo, *"nada selecionado, catálogo em
    /// Star"* e *"nada selecionado, catálogo em Polygon"* comparam iguais e os campos ficam com
    /// os números da forma anterior (report do Enio, 2026-08-01).
    pub shape_last_focus: Option<(Option<ph2d_vec_scene::VecPathId>, ph2d_vec_scene::ShapeKind)>,

    /// ⭐⭐ **O artista está ARMADO para desenhar** — carregou numa forma do catálogo e ainda não
    /// tocou noutro objeto. Enquanto isto vale (e só no modo `Shape`), os campos de parâmetro do
    /// painel são os da forma ARMADA, e a caixa numérica NÃO alcança a forma selecionada
    /// (`vec_shape_params::shape_field_target`).
    ///
    /// ⚠️ **O modo sozinho não responde**, e foi a 1.ª redacção desta cura: *"desenhei uma estrela,
    /// deixa-me ajustar as pontas dela"* e *"armei o Polígono, mostra-me o Polígono"* são os dois o
    /// modo `Shape` com uma forma viva selecionada — o que os separa é **qual gesto veio por
    /// último**. A tool publica o clique ([`ph2d_tool_vector::VectorTool::take_shape_armed`]) e
    /// [`Self::shape_armed_target`] apaga o latch quando a selecção muda (desenhar selecciona a
    /// forma nova, então o ciclo Live Shape volta sozinho).
    pub shape_armed: bool,

    /// ⭐⭐⭐ **O PEDAÇO que o Trim vai apagar** (plano 38) — o que o cursor aponta neste quadro, e
    /// `None` quando ele não aponta nada. O realce desenha-o e o clique apaga-o, **pela mesma
    /// resposta**: numa ferramenta destrutiva, acender uma coisa e apagar outra é o pior defeito
    /// possível.
    pub trim_hit: Option<crate::trim::TrimHit>,

    /// ⭐⭐⭐ **A FACE que o Balde vai preencher** (plano 40) — a região sob o cursor neste quadro,
    /// em MUNDO, e `None` quando ele não aponta região nenhuma. O realce desenha-a e o clique
    /// deposita-a, **pela mesma resposta**.
    pub bucket_face: Option<crate::bucket::BucketHit>,

    /// A rede de arcos guardada, com a chave do documento que a produziu.
    ///
    /// ⚠️ **Guardada porque montá-la custa `3,8 ms` a 20 traços e `188 ms` a 80** (medido), contra
    /// `0,08–0,35 ms` para achar a face nela. Montar por quadro está refutado; achar a face por
    /// quadro é de graça.
    pub bucket_cache: Option<crate::bucket::BucketCache>,

    /// Os preenchimentos que nasceram neste quadro e ainda esperam a ENTIDADE: `(caminho,
    /// semente)`. Drenados logo depois do `vec_entities::sync`, que é quem a cria.
    pub bucket_new: Vec<(u64, [f32; 2], Vec<ph2d_ecs::FillAnchor>)>,

    /// O alvo vivo do frame anterior — só existe para detectar a MUDANÇA que desarma o
    /// [`Self::shape_armed`].
    pub shape_armed_target: Option<ph2d_vec_scene::VecPathId>,

    /// `VecPathId` → entidade ECS que o representa na Hierarquia (ADR-0110). O
    /// invariante "um path ⟺ uma entidade" é mantido por `vec_entities::sync`.
    pub entities: ph2d_vec_entities::entities::VecEntityMap,

    /// Espelho da última sincronia de seleção canvas ↔ Hierarquia (ADR-0110): diz
    /// **quem** mudou neste frame, e por isso quem manda. Ver `sync_selection`.
    pub sel: crate::selection_sync::VecSelSync,
}

/// **O estilo de texto corrente** (A9, 2026-09-12): os oito números que o painel de texto edita e
/// que a PRÓXIMA sessão de texto herda. Moravam soltos na `App` como `vec_text_*` (e o `text_wrap`
/// já aqui, com o doc a chamar-lhes «irmãos»).
///
/// ⚠️ **`Default` manual, e é o motivo de o grupo existir:** os valores com que o app nasce são os
/// `DEFAULT_TEXT_*` da ferramenta e os eixos da fonte embutida — não os zeros do `derive`. Eles
/// eram escritos no construtor da `App`; aqui ficam ao lado dos campos que inicializam, e o
/// `VecState` continua a derivar o seu.
pub struct TextKnobs {
    /// Tamanho (world) que a próxima sessão de texto começa e que o slider Size do
    /// painel edita. Persiste entre sessões (é o default corrente do usuário).
    pub size: f64,
    /// Peso (`wght`) que a próxima sessão de texto começa e que o slider Weight do
    /// painel edita. Persiste entre sessões.
    pub weight: f32,
    /// Entrelinha (múltiplo do tamanho) corrente do texto — slider Line-height do
    /// painel; persiste entre sessões.
    pub line_height: f64,
    /// Tracking (fração do tamanho, em) corrente do texto — slider Tracking do painel;
    /// persiste entre sessões.
    pub tracking: f64,
    /// Alinhamento horizontal corrente do texto (L/C/R) — botões do painel; persiste
    /// entre sessões.
    pub align: ph2d_vec_text::TextAlign,
    /// Valores correntes dos eixos de variação da fonte além do peso (opsz/wdth/…), na
    /// ordem de `vec_font::variation_axes`. Reseedado quando a família muda; a seção
    /// Axes do painel os edita e uma nova sessão herda estes defaults.
    pub extra_axes: Vec<(ph2d_vector_font::AxisTag, f32)>,
    /// Família de fonte corrente do texto (`None` = InterVariable embutida). Os botões
    /// `<`/`>` do painel ciclam; persiste entre sessões.
    pub family: Option<String>,
    /// A largura de refluxo corrente do texto (`None` = Auto). Default da PRÓXIMA sessão e
    /// espelho da que está viva — o mesmo papel dos irmãos deste grupo.
    pub wrap: Option<f64>,
}

impl Default for TextKnobs {
    fn default() -> Self {
        Self {
            size: ph2d_tool_vector::params::DEFAULT_TEXT_SIZE,
            weight: ph2d_tool_vector::params::DEFAULT_TEXT_WEIGHT as f32,
            line_height: ph2d_tool_vector::params::DEFAULT_TEXT_LINE_HEIGHT,
            tracking: ph2d_tool_vector::params::DEFAULT_TEXT_TRACKING,
            align: ph2d_vec_text::TextAlign::Left,
            extra_axes: ph2d_system_fonts::library::seed_extra_axes(None),
            family: None,
            // ⚠️ **Auto é o default**, e é decisão de produto: um texto criado com a ferramenta
            // cresce com o que se digita (é o que todo editor faz num clique-e-digite). Uma
            // caixa nasce quando o artista a pede — e o gesto de ARRASTAR uma caixa ainda não
            // existe, então pedi-la é escolher `Fixed`.
            wrap: None,
        }
    }
}

/// Os knobs `(Corner, Side)` do Expand como o frame os viu no quadro ANTERIOR — ver o doc do campo
/// [`VecState::expand_knobs`].
///
/// ⚠️ **Um tipo e não a tupla crua por causa do valor com que nasce:** ele tem de ser o mesmo par
/// com que o PAINEL nasce ([`ph2d_panel_vector::EXPAND_KNOBS_AT_BIRTH`]), senão o 1.º quadro leria
/// um clique que ninguém deu e retunaria os offsets vivos da seleção. O par era escrito à mão em
/// dois sítios (o `thread_local` do painel e o construtor da `App`); agora sai de um.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandKnobs(pub (u8, u8));

impl Default for ExpandKnobs {
    fn default() -> Self {
        Self(ph2d_panel_vector::EXPAND_KNOBS_AT_BIRTH)
    }
}

/// A pose que ESTE passe escreveu no frame anterior, por rótulo. Runtime-only (não vai para o
/// save nem para o undo): o pior que um cache perdido causa é um frame de absorção idempotente
/// — o estado restaurado é auto-consistente (`centro = âncora + offset`), então re-absorver
/// devolve o MESMO offset.
///
/// ⚠️ Desceu da shell (`label_live.rs`) em 2026-09-12 (`line/render-loop`, A9 da auditoria de
/// arquitectura): a instância dela é o [`VecState::label_poses`], e o passe dos
/// rótulos, que continua na shell, recebe-a por referência.
pub type LabelPoses = std::collections::BTreeMap<ph2d_vec_scene::VecPathId, ph2d_ecs::Transform>;
