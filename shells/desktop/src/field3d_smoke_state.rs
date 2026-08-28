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
/// ⚠️ A **espécie** viaja com ele porque a decisão de cancelar precisa de saber *o que* está a
/// correr: um refinamento cede à mão, um traçado de movimento nunca
/// (ver `field3d_preview::cancels_the_inflight`).
///
/// ⚠️⚠️ **Era o `size` que viajava, e o comentário já dizia «espécie»** — o tamanho respondia por
/// ela até a W73 pôr um refinamento no tamanho grosso, e a partir daí o degrau do meio do assentar
/// **nunca era abandonado** (W89).
pub(crate) struct InFlight {
    pub(crate) rx: Receiver<Ready>,
    pub(crate) cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// ⭐ `true` quando este traçado é um **refinamento** (contorno fino + anti-serrilhado), os dois
    /// degraus do assentar; `false` para o quadro de movimento.
    pub(crate) refinement: bool,
}

/// ⭐⭐⭐ **UM VIEWPORT** — a câmera, o que ela já traçou, e o laço que a serve (W90).
///
/// # Porque isto saiu do [`Smoke`]
///
/// O módulo nasceu com **uma** vista, e estes nove campos viviam soltos ao lado do documento, do
/// gizmo e dos gestos. O canvas de primeira classe que o plano pede
/// (`docs/3DModeling/03_plano_implicito.md`: *"modo de viewport próprio, com cabeçalho e divisão"*)
/// precisa de **N** câmeras a olhar a MESMA peça — e a fronteira entre o que é de uma vista e o que
/// é do módulo já estava escrita, campo a campo, nos doc-comments deles.
///
/// ⚠️ **O que fica no [`Smoke`] é o que é do DOCUMENTO ou do GESTO**: a peça, o gizmo, o arrasto em
/// curso, o laço, a isolação, o verbo. Um gesto acontece **num** viewport de cada vez (o
/// [`Smoke::active`]), e duplicá-lo por vista daria dois arrastos a discordar sobre a mesma peça.
pub(crate) struct Viewport {
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
    /// ⚠️ **O `bool` é *«este traçado é de MOVIMENTO»*** (W73) — e ele é o que torna o assentar uma
    /// escada: sem essa memória, o degrau que alisa e o que aumenta são indistinguíveis, porque os
    /// dois pedem o mesmo `(câmera, tamanho, documento)`.
    pub(super) requested: Option<(Orbit, u32, u32, FieldDoc, bool)>,
    /// Quanto custou o último traçado — o número que o painel mostra, porque quem mexe num raio
    /// é quem paga o traçado seguinte.
    pub(crate) last_trace_ms: f32,
    /// ⭐ **O que o último traçado custou, COM o tamanho a que foi medido** — a entrada do laço que
    /// escolhe a resolução do preview ([`crate::field3d_preview`]).
    ///
    /// ⚠️ Separado do `last_trace_ms` de propósito: aquele é para o artista ler, este é para a
    /// máquina decidir, e um tempo sem os pixels ao lado não prevê coisa nenhuma.
    pub(crate) measured: Option<crate::field3d_preview::Measured>,
    /// **A área onde o quadro foi desenhado da última vez** — é ela que responde *"este clique é
    /// meu?"*. Sem isto a cena engoliria gestos de qualquer canto da janela.
    pub(crate) area: Option<EditorRect>,
    /// ⭐ **O prato para de girar assim que o artista toca nele.**
    ///
    /// A lei da casa é *feature nova = auto-play*: a peça gira sozinha para provar que existe. Mas
    /// continuar a girar **depois** de alguém a ter posto num ângulo é desfazer o gesto dele a cada
    /// quadro — a auto-demonstração deixa de ser um convite e passa a ser uma disputa.
    pub(crate) manual: bool,
    /// ⭐⭐⭐ **A CACHE DE FITAS É DO VIEWPORT** (W90) — e não do módulo.
    ///
    /// ⚠️ **Ela era do [`Smoke`], e com a divisão isso vira um defeito medido:** o tecto da cache é
    /// **derivado do que um quadro pede** (W89, `TapeCache::begin`), e com quatro viewports a
    /// chamá-lo o último a passar dimensionava a cache para **um quarto** da tela. Era a debulha
    /// que a W89 acabou de curar, reaberta pela porta do lado.
    ///
    /// ⭐ E o total não sobe: quatro viewports de um quarto da área pedem, somados, as regiões de
    /// uma tela — *o que muda é quem contabiliza.*
    pub(crate) tapes: Arc<ph2d_field_render::TapeCache>,
}

impl Viewport {
    /// ⚠️ **Só para o gate**: em que estado esta vista PAROU.
    ///
    /// `None` enquanto ela ainda não tem quadro pronto ou tem traçado em voo; `Some(true)` quando o
    /// último traçado que ela pediu foi um quadro de **movimento** (grosso, sem anti-serrilhado).
    ///
    /// *A pergunta vive onde os dados vivem* — abrir os campos ao resto do shell só para um teste
    /// os poder ler seria pagar em encapsulamento o que uma função de três linhas resolve.
    /// ⚠️ **Só para o gate**: volta ao estado de *«ainda não tracei nada»*, para que a ORDEM em que
    /// as vistas recebem a primeira imagem possa ser observada do frio.
    #[cfg(test)]
    #[doc(hidden)]
    pub(crate) fn probe_forget_frame(&mut self) {
        self.frame = None;
        self.requested = None;
        self.inflight = None;
    }

    /// ⚠️ **Só para o gate**: esta vista já tem uma imagem na tela?
    #[cfg(test)]
    #[doc(hidden)]
    pub(crate) fn probe_has_frame(&self) -> bool {
        self.frame.is_some()
    }

    // ⚠️ `cfg(test)` porque ela é uma sonda **de teste** numa crate binária: sem isto o build do
    // produto carrega um método que ninguém chama, e o aviso está certo.
    #[cfg(test)]
    #[doc(hidden)]
    pub(crate) fn probe_resting_state(&self) -> Option<bool> {
        if self.frame.is_none() || self.inflight.is_some() {
            return None;
        }
        Some(self.requested.as_ref().is_some_and(|(_, _, _, _, k)| *k))
    }

    /// **Um viewport novo com esta câmera** — o resto nasce vazio, que é o estado de *«ainda não
    /// tracei nada»*.
    pub(crate) fn new(cam: Orbit, manual: bool) -> Self {
        Self {
            cam,
            frame: None,
            inflight: None,
            since: std::time::Instant::now(),
            requested: None,
            last_trace_ms: 0.0,
            measured: None,
            area: None,
            manual,
            tapes: Arc::new(ph2d_field_render::TapeCache::new()),
        }
    }
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
    /// ⭐⭐⭐ **OS VIEWPORTS** — nunca vazio, e o [`Smoke::active`] é o que o gesto comanda.
    ///
    /// ⚠️ A invariante *«nunca vazio»* é o que torna [`Smoke::vp`] infalível, e há gate. Sem ela,
    /// todo acesso à câmera passaria a ser um `Option` e os ~30 sítios que perguntam por ela teriam
    /// de responder *«e se não houver vista nenhuma?»* — uma pergunta que o produto não tem.
    pub(crate) vps: Vec<Viewport>,
    /// Qual viewport o gesto comanda. Ver [`Smoke::vp`].
    pub(crate) active: usize,
    /// ⭐⭐⭐ **Como o canvas está dividido** — ver [`crate::field3d_layout::Split`].
    ///
    /// ⚠️ **Estado de VISTA**, como a câmera: ele não toca a peça, não entra no undo e não viaja no
    /// arquivo. ⛔ E **não** viaja no [`crate::field3d_view::View`] que sobrevive a fechar o painel:
    /// restaurar a divisão obrigaria a restaurar as **quatro** câmeras, e o que a W43 promete é
    /// *«a peça certa vista do sítio onde a deixei»* — uma promessa sobre UMA vista.
    pub(crate) split: crate::field3d_layout::Split,
    /// Já anunciou o primeiro quadro? Ver a nota do `boot`.
    pub(super) announced: bool,
    /// O arrasto em curso, se houver ([`crate::field3d_input`]).
    pub(crate) drag: Option<Drag>,
    /// A última posição do ponteiro, para o delta do arrasto.
    pub(crate) last_pointer: (f32, f32),
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
    /// ⚠️ **O `bool` é «aditivo?»** (W58): um clique com `Shift`/`Ctrl` **alterna** o objeto na
    /// seleção em vez de a substituir — o mesmo verbo do canvas 2D.
    pub(crate) pending_pick: Option<([f32; 2], bool)>,
    /// ⭐⭐ **O laço em curso**, em pixels LOCAIS da área 3D — `(canto de partida, canto de agora)`.
    /// É ele que a moldura pinta enquanto o dedo está em baixo.
    pub(crate) lasso: Option<([f32; 2], [f32; 2])>,
    /// O laço **terminado**, à espera de virar seleção. ⚠️ Separado do de cima porque um deles é
    /// *desenho* e o outro é *pedido*: pintar a partir do pedido faria a moldura sobreviver ao
    /// dedo por um quadro.
    pub(crate) pending_lasso: Option<([f32; 2], [f32; 2])>,
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

impl Smoke {
    /// ⭐ **O viewport que o gesto comanda.**
    ///
    /// ⚠️ **Infalível por invariante, não por sorte:** [`Smoke::vps`] nunca é vazio, e o índice é
    /// preso ao alcance **aqui** em vez de em cada chamador. *Um `Option` nesta porta obrigaria os
    /// ~30 sítios que perguntam pela câmera a responder «e se não houver vista nenhuma?».*
    pub(crate) fn vp(&self) -> &Viewport {
        &self.vps[self.active.min(self.vps.len() - 1)]
    }

    /// Ver [`Smoke::vp`].
    pub(crate) fn vp_mut(&mut self) -> &mut Viewport {
        let i = self.active.min(self.vps.len() - 1);
        &mut self.vps[i]
    }
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
    /// ⭐⭐ **O LAÇO** (W58) — um rectângulo que escolhe tudo o que se vê dentro dele.
    ///
    /// ⚠️ **Ele nasce do MODIFICADOR, nunca de arrastar em espaço vazio.** Arrastar orbita, e é o
    /// gesto principal do módulo (a pesquisa do navball mede os utilizadores *quase 2× mais rápidos*
    /// a arrastar do que a clicar). Roubá-lo em espaço vazio faria o mesmo botão fazer duas coisas
    /// conforme o que estivesse por baixo — e o artista descobriria a diferença ao girar a peça e
    /// ver um rectângulo.
    ///
    /// ⭐ **`Shift`/`Ctrl` é a MESMA tecla que já significa «estou a falar da seleção»** neste app
    /// (o canvas 2D usa-a para alternar um objeto no clique). Segurada com um **clique** ela
    /// alterna um; segurada com um **arrasto**, alterna um rectângulo. *Um vocabulário, não dois.*
    Lasso,
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
