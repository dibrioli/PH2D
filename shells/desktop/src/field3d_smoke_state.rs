//! ⭐ **O QUE O SMOKE É** — os tipos do estado da janela 3D de modelagem, e a célula que o guarda.
//!
//! ⚠️ O corte saiu do teto de LOC do shell (HR-18), e a fronteira é a que já existia por dentro: o
//! arquivo pai responde *o que se FAZ* (armar, isolar, voar, publicar pedidos) e este responde *o que
//! existe*. Nenhuma lei se mudou de sítio — só as definições.
//!
//! ⚠️ **A célula continua a ser UMA**, e a porta dela continua a ser o `with_smoke` do pai: o estado
//! vive num `thread_local` deste módulo, e não num campo do `App`, porque o `app_state.rs` é
//! partilhado e a `line/sculpt3d` edita-o — um campo novo lá seria uma colisão por conveniência.

use std::cell::RefCell;

use std::sync::Arc;
use std::sync::mpsc::Receiver;

use ph2d_editor::zones::Rect as EditorRect;
use ph2d_field::FieldDoc;
use ph2d_field_render::Orbit;

/// ⭐ **O traçado que está em VOO** — e a bandeira que o pode abandonar (W32).
///
/// ⚠️ O `size` viaja com ele porque a decisão de cancelar precisa de saber **o que** está a correr:
/// um refinamento cede à mão, um traçado de movimento nunca (ver `field3d_preview::cancels_the_inflight`).
pub(crate) struct InFlight {
    pub(crate) rx: Receiver<Ready>,
    pub(crate) cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(crate) size: (u32, u32),
}

pub(crate) struct Smoke {
    /// A peça a traçar, **cozida da cena** (`field3d_scene`). `None` quando não há geometria
    /// nenhuma — apagar o último filho na Hierarquia é um gesto normal, e o resultado normal dele
    /// é a tela ficar vazia.
    pub(crate) doc: Option<FieldDoc>,
    /// ⭐ **A SEMENTE** — a cena inicial, que a ponte planta **uma vez** e depois deita fora.
    ///
    /// ⚠️ **Ela é separada do `doc` de propósito, e a mistura era um bug.** A ponte oferecia o `doc`
    /// **cozido** como semente, e o comentário ao lado afirmava que ele *"deixa de existir"* — o que
    /// nunca foi verdade: ele é reescrito a cada quadro. O sintoma: apagar a peça na Hierarquia
    /// **fazia-a voltar no quadro seguinte**, porque a ponte não encontrava raiz e replantava o que
    /// tinha acabado de cozer.
    ///
    /// *Uma semente usa-se uma vez.* Depois de plantada, a cena é a fonte — e apagar a peça apaga-a.
    pub(crate) seed: Option<FieldDoc>,
    pub(super) matcap: Arc<MatcapTexels>,
    pub(crate) cam: Orbit,
    /// O último quadro pronto — com o tamanho a que foi traçado, para o desenhar sem esticar
    /// enquanto o próximo (já do tamanho novo) não chega.
    pub(super) frame: Option<(Arc<Vec<u8>>, u32, u32)>,
    pub(super) inflight: Option<InFlight>,
    /// Quando o pedido em voo saiu — é o relógio que faz a rotação ser por SEGUNDO.
    pub(super) since: std::time::Instant,
    /// **O que já foi pedido**: a câmera, o tamanho **e o DOCUMENTO**.
    ///
    /// ⚠️ **O documento faz parte da chave, e não fazia** — foi um smoke reprovado (Enio,
    /// 2026-08-19: *"slider disfuncional"*). O controle mudava o raio, o documento mudava, o painel
    /// mostrava o número novo... e a peça na tela **não se mexia**, porque a pergunta *"mudou
    /// alguma coisa?"* olhava só a câmera e o tamanho.
    ///
    /// *Um cache que não conhece uma das entradas não é um cache — é um congelador.* E o sintoma
    /// culpava o controle, que estava certo.
    pub(super) requested: Option<(Orbit, u32, u32, FieldDoc)>,
    /// Quanto custou o último traçado — o número que o painel mostra, porque quem mexe num raio
    /// é quem paga o traçado seguinte.
    pub(crate) last_trace_ms: f32,
    /// ⭐ **O que o último traçado custou, COM o tamanho a que foi medido** — a entrada do laço que
    /// escolhe a resolução do preview ([`crate::field3d_preview`]).
    ///
    /// ⚠️ Separado do `last_trace_ms` de propósito: aquele é para o artista ler, este é para a
    /// máquina decidir, e um tempo sem os pixels ao lado não prevê coisa nenhuma.
    pub(crate) measured: Option<crate::field3d_preview::Measured>,
    /// Já anunciou o primeiro quadro? Ver a nota do `boot`.
    pub(super) announced: bool,
    /// **A área onde o quadro foi desenhado da última vez** — é ela que responde *"este clique é
    /// meu?"*. Sem isto a cena engoliria gestos de qualquer canto da janela.
    pub(crate) area: Option<EditorRect>,
    /// O arrasto em curso, se houver ([`crate::field3d_input`]).
    pub(crate) drag: Option<Drag>,
    /// A última posição do ponteiro, para o delta do arrasto.
    pub(crate) last_pointer: (f32, f32),
    /// ⭐ **O prato para de girar assim que o artista toca nele.**
    ///
    /// A lei da casa é *feature nova = auto-play*: a peça gira sozinha para provar que existe. Mas
    /// continuar a girar **depois** de alguém a ter posto num ângulo é desfazer o gesto dele a cada
    /// quadro — a auto-demonstração deixa de ser um convite e passa a ser uma disputa.
    pub(crate) manual: bool,
    /// ⭐ **Onde o gizmo está**, publicado pela ponte com a cena — que é quem tem o mundo. `None`
    /// quando nada de modelagem está selecionado.
    ///
    /// ⚠️ A âncora atravessa o quadro em vez de ser recalculada aqui de propósito: o `draw` não tem
    /// mundo nenhum, e dar-lhe um significaria o traçado passar a depender do ECS.
    pub(crate) gizmo: Option<crate::field3d_gizmo::Anchor>,
    /// A alça sob o cursor. Só realce — quem manda no arrasto é o `drag`.
    pub(crate) gizmo_hot: Option<crate::field3d_gizmo::Handle>,
    /// O que o arrasto pediu e a ponte ainda não aplicou: `(entidade, pedido)`.
    ///
    /// ⚠️ **Acumula**, e não substitui: entre dois quadros chegam vários eventos de ponteiro, e
    /// guardar só o último faria a peça andar menos do que a mão — devagar, e só quando o rato vai
    /// depressa, que é o defeito mais difícil de acreditar. Cada verbo acumula à maneira dele
    /// (`Motion::merge`).
    pub(crate) pending_move: Option<(u64, crate::field3d_gizmo::Motion)>,
    /// ⭐ **O arrasto do gizmo em curso**, congelado no instante da pegada.
    ///
    /// ⚠️ **A âncora é congelada de propósito.** Ela é republicada a cada quadro a partir da pose do
    /// objeto — que o próprio arrasto está a mudar. Medir o total contra uma âncora que se move
    /// seria medir contra o resultado, e o gesto perseguiria a própria cauda.
    ///
    /// ⭐ E é por medir o **total desde a pegada**, e não incrementos, que prender à grelha funciona:
    /// um total preso é exato, enquanto uma soma de incrementos presos acumula o erro de cada um.
    pub(crate) drag_grip: Option<Grip>,
    /// ⭐ **O gesto está preso à grelha?** — o `Ctrl`, lido no início de cada movimento.
    ///
    /// ⚠️ Lido a cada movimento, e não guardado na pegada: o Blender deixa entrar e sair da grelha a
    /// meio do arrasto, e é o que se quer — mira-se à mão até perto e prende-se no fim. Congelar o
    /// modificador na pegada obrigaria a soltar e repetir o gesto.
    pub(crate) snapping: bool,
    /// ⭐ **O número que está a ser DIGITADO no meio do gesto** (W26, [`crate::field3d_typed`]).
    ///
    /// ⚠️ **Enquanto ele existe, o rato deixa de mandar** — e é isso que faz digitar funcionar: sem
    /// essa cedência, o movimento seguinte do ponteiro sobrescreveria o número no quadro a seguir a
    /// alguém o escrever, e o defeito leria como *"digitar não faz nada"*.
    ///
    /// `None` = não há entrada; o gesto é do ponteiro, como sempre foi.
    pub(crate) typed: Option<String>,
    /// Onde o botão desceu — é o que distingue um **clique** (selecionar) de um **arrasto**
    /// (orbitar). ⚠️ Sem ele, todo clique na peça seria também um giro de zero graus, e a única
    /// forma de selecionar seria a Hierarquia.
    pub(crate) press_at: Option<(f32, f32)>,
    /// Um clique que ainda não foi resolvido: o pixel, no referencial da área desenhada.
    ///
    /// ⚠️ Resolver aqui é impossível — a pergunta *"de quem é este ponto?"* precisa do MUNDO, e o
    /// ponteiro corre fora do quadro. Ele viaja para a ponte pelo mesmo cano dos intents do painel.
    pub(crate) pending_pick: Option<[f32; 2]>,
    /// ⭐ **Que verbo o gizmo está a oferecer** — mover, rodar ou escalar.
    ///
    /// ⚠️ É estado de **vista**, e não do documento: por isso vive aqui e não num componente. O
    /// painel mostra-o e as teclas `G`/`R`/`S` trocam-no.
    pub(crate) gizmo_mode: crate::field3d_gizmo::Mode,
    /// ⭐ **Em que referencial os eixos do gizmo apontam** — do mundo, ou do próprio objeto.
    /// Estado de **vista**, como o verbo.
    pub(crate) gizmo_frame: crate::field3d_gizmo::Frame,
    /// ⭐ **O nó ISOLADO** — mostrar só ele, ou `None` para a peça inteira (W38).
    ///
    /// ⚠️ **Estado de VISTA, e a lei é a do módulo irmão, lida e não re-decidida**
    /// (`sculpt3d_objects::toggle_isolate`): é um **toggle** — *"um «sair do isolamento» separado
    /// seria uma segunda porta para o mesmo fato, e a que o artista não acha quando a cena some"* —
    /// e **nada aqui entra na história**: isolar não move um vértice, então não é um passo de
    /// desfazer nem viaja no arquivo. Por isso vive aqui, ao lado do verbo do gizmo, e **não** num
    /// componente do mundo.
    ///
    /// ⚠️ Guarda os **bits** da entidade, que morrem num undo — e é por isso que quem o lê confirma
    /// que o nó ainda existe antes de o usar (ver `field3d_scene::cook_root`). Um isolamento
    /// pendurado numa entidade morta apagaria a peça da tela sem nada a explicar.
    pub(crate) isolated: Option<u64>,
    /// ⭐⭐ **A VIAGEM entre vistas em curso** (W51) — `None` quando a câmera está parada.
    ///
    /// ⚠️ **Cache do gesto:** um voo a atravessar um fecho de painel reabriria o módulo a meio de um
    /// movimento que ninguém pediu. Ver [`crate::field3d_flight`].
    pub(crate) flight: Option<crate::field3d_flight::Flight>,
    /// Quantas viagens já houve — é ela que dá um **id novo** a cada uma, para a mola da casa
    /// começar do zero em vez de continuar a anterior.
    pub(crate) flight_gen: u32,
    /// A viagem acabou de partir? O shell precisa de saber para **semear** a track em `0`.
    pub(crate) flight_fresh: bool,
    /// ⭐ **A parte da área que a moldura do app NÃO tapa** (W50), publicada pelo shell todo quadro.
    ///
    /// ⚠️ **Cache do quadro, como a `area`:** ela depende de que painéis estão abertos e de onde a
    /// faixa do topo foi pintada NESTE quadro. `None` até o shell a publicar, e aí a área inteira é
    /// a resposta — que é o que era antes desta wave.
    pub(crate) safe: Option<EditorRect>,
    /// ⭐ **A bola do gizmo de navegação sob o cursor** (W49) — só realce e a decisão do clique.
    ///
    /// ⚠️ **Cache do gesto, não vista:** ela é reposta a cada movimento do ponteiro, e um valor
    /// atravessando um fecho de painel reacenderia uma bola que ninguém está a apontar.
    pub(crate) nav_hot: Option<crate::field3d_views::Standard>,
    /// ⭐ **A bola em que o botão DESCEU**, se desceu numa. É ela que faz o `Up` sem movimento ser
    /// uma escolha de vista em vez de uma órbita de zero graus.
    pub(crate) nav_press: Option<crate::field3d_views::Standard>,
    /// ⭐ **Há um contorno FECHADO escolhido no editor vetorial?** (W53) — publicado pelo shell,
    /// como o irmão abaixo e pela mesma razão: quem tem a cena vetorial é o `AppGfx`.
    ///
    /// É ele que faz os botões `+ Extrude` / `+ Revolve` aparecerem só quando há o que extrudar.
    /// ⭐⭐ **QUAL contorno fechado está escolhido** (W57), não só se há um.
    ///
    /// ⚠️ **A identidade era deitada fora e voltou a ser precisa.** Até a W56 o shell publicava um
    /// `bool` — bastava para os dois botões `+ Extrude`/`+ Revolve` aparecerem. O gesto de
    /// **religar** uma forma a outro desenho precisa do **id**, e ele não é redescobrível do lado
    /// de lá: quem drena as intenções recebe o mundo, nunca a cena vetorial. *Publicar um `bool`
    /// onde a fonte tinha um id é deitar fora a metade que a próxima feature ia pedir.*
    pub(crate) profile_pick: Option<u64>,
    /// ⭐ **Há uma escultura VIVA na cena?** — publicado pelo shell, que é quem tem o `AppGfx`.
    ///
    /// ⚠️ Atravessa o quadro em vez de ser perguntado aqui, pela razão do `gizmo`: este arquivo não
    /// tem `AppGfx` nenhum, e dar-lhe um faria o traçado passar a depender do módulo de escultura.
    /// Sem a feature `sculpt3d` ele fica **sempre falso**, e o botão nunca é oferecido.
    pub(crate) has_live_sculpt: bool,
}

/// **O que um arrasto de gizmo guarda** desde a pegada até soltar.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Grip {
    /// A âncora **no instante da pegada** — ver a nota de [`Smoke::drag_grip`].
    pub(crate) anchor: crate::field3d_gizmo::Anchor,
    /// O pixel da pegada, no referencial da área desenhada.
    pub(crate) from: [f32; 2],
    /// O que o mundo **já recebeu** deste gesto. O que falta aplicar é `total.since(applied)`.
    pub(crate) applied: crate::field3d_gizmo::Motion,
}

/// O gesto de navegação em curso.
///
/// ⚠️ Os botões são os **mesmos** do módulo de escultura (`sculpt3d_input.rs`): esquerdo e direito
/// orbitam, o do meio faz pan. Não é herança por analogia — são duas janelas 3D no mesmo app, e uma
/// mão que aprendeu a girar numa tem de girar na outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Drag {
    Orbit,
    Pan,
    /// ⭐ Uma alça do gizmo 3D está agarrada. ⚠️ **Ela ganha do botão esquerdo**, e é o único sítio
    /// onde a navegação cede: uma seta que orbitasse a câmera em vez de mover a peça seria uma alça
    /// pintada e morta.
    Gizmo(crate::field3d_gizmo::Handle),
}

/// O que uma requisição de traçado devolve.
pub(crate) struct Ready {
    pub(super) rgba: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) hits: usize,
    pub(super) edges: usize,
    pub(super) millis: f64,
}

pub(super) struct MatcapTexels {
    pub(super) side: u32,
    pub(super) rgb: Vec<f32>,
}

thread_local! {
    pub(super) static STATE: RefCell<Option<Option<Smoke>>> = const { RefCell::new(None) };
}
