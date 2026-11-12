# Cabeçalhos de ficheiro cortados para baixo do tecto de LOC (integração 2026-11-12b)

O tecto da shell e o de `ph2d-app-components` em `architecture_the_shell_only_shrinks` somam entre linhas; a cura é corte, nunca subir o número. História verbatim dos `//!` removidos (o porquê histórico vive aqui, não no código — CLAUDE.md §2). Cada bloco está indentado (4 espaços) sob o caminho do ficheiro e a linha onde começava; o que ficou no código é o título e o 1.º parágrafo.

## `shells/desktop/src/vec_tree_settle.rs:4`

    //!
    //! A captura do undo tem de ser **PONTO FIXO dos sistemas**: fotografar, deixar o quadro seguinte
    //! correr sem entrada nenhuma, e fotografar outra vez tem de dar a MESMA foto. Quando não é, o
    //! diff por-quadro do [`crate::undo`] lê a convergência dos próprios sistemas como acção do
    //! artista — nasce um passo **fantasma**, ele limpa a pilha de redo, e o `Ctrl+Z` seguinte gasta-se
    //! a desfazer o lixo que o quadro acabou de criar. Para o dono: *«o undo só faz uma etapa»*.
    //!
    //! A F4.6c curou UM escritor — o arrasto da Hierarquia — **movendo-o** para antes da projecção de
    //! z, e o handoff de 2026-09-07 deixou a pergunta escrita: *«os outros verbos tardios — apagar,
    //! duplicar, Remove from Sheet — escrevem a árvore no MESMO sítio tardio e têm a mesma latência
    //! estrutural. NÃO foi medido se produzem o fantasma.»*
    //!
    //! **Foi medido, e produzem.** Os dois gates de [`crate::vec_zorder_late_writers_tests`] nasceram
    //! vermelhos, e o mecanismo é o do [`ph2d_vec_entities::entities::sync`], que é bidireccional:
    //!
    //! | verbo tardio | o que ele escreve | o que o `sync` do quadro SEGUINTE faz sozinho |
    //! |---|---|---|
    //! | apagar | despawna a **entidade** | tira o caminho do **documento** ⇒ passo `["vec"]` |
    //! | duplicar | põe o caminho no **documento** | cunha a **entidade** ⇒ passo `["world"]` |
    //!
    //! ⚠️ **As duas metades da mesma latência** — um gate só deixaria metade da classe por medir.
    //!
    //! # ⛔ Porque a cura NÃO foi mover estes, nem mover a leitura
    //!
    //! - **Mover os escritores** custou-nos uma linha no reparent porque o `drain_reparent` é
    //!   auto-contido; apagar e duplicar vivem dentro do [`crate::render_loop::hierarchy::dispatch`],
    //!   ~2 300 linhas de verbos com dependências no que o quadro computou entretanto. E **não fecha a
    //!   classe**: o próximo verbo tardio que alguém escrever renasce com o defeito.
    //! - **Mover a LEITURA para o fim** foi medido e recusado: entre a projecção (`reorder_to`) e o
    //!   `dispatch` correm ~40 consumidores da cena — `envelope_live`, `skeleton_live`, `pattern_live`,
    //!   `align_live`, o hit-test e o próprio desenho. A ordem das `paths` **é** a ordem de pintura;
    //!   projectar só no fim daria a todos eles um quadro de atraso.
    //!
    //! ⇒ a leitura fica onde está, **para o desenho**, e esta porta corre outra vez **para a
    //! fotografia**. São duas perguntas, não duas respostas à mesma: uma serve o que se VÊ neste
    //! quadro, a outra o que se GUARDA dele.
    //!
    //! ⚠️ **É o sítio e a razão do [`ph2d_app_components::instance_sync::App::sync_instances`]**, que corre a dois
    //! passos daqui com o mesmo doc: *depois do quadro, e antes da captura — senão a escrita do sync
    //! vira um passo de undo que ninguém deu.* A lei já estava escrita ali, para outro sistema.

## `shells/desktop/src/vec_zorder_late_writers_tests.rs:4`

    //!
    //! A F4.6c curou o *«reordenei objectos na hierarquia e não funcionou o undo»* movendo o dreno do
    //! reparent para **antes** da projecção de z, e o handoff de 2026-09-07 nomeou o que ficava:
    //!
    //! > *«Os outros verbos tardios da Hierarquia — apagar · duplicar · Remove from Sheet — escrevem a
    //! > árvore no MESMO sítio tardio e têm a mesma latência estrutural. NÃO foi medido se produzem o
    //! > fantasma.»*
    //!
    //! ⚠️ **Medir era o passo, não curar** — se o fantasma não aparecesse, a resposta seria uma recusa
    //! medida, e ela valeria tanto como uma cura. Os dois gates abaixo **nasceram vermelhos**: o
    //! fantasma existe para os dois, e a cura é o [`crate::vec_tree_settle`].
    //!
    //! # O mecanismo, medido no produto (2026-09-08)
    //!
    //! O [`ph2d_vec_entities::entities::sync`] é **bidireccional**, e é isso que fecha o ciclo:
    //!
    //! | verbo | o que o dreno escreve | o que o `sync` do quadro SEGUINTE faz sozinho |
    //! |---|---|---|
    //! | apagar | `despawn` da **entidade** (`hierarchy_delete.rs`) | tira o caminho do **documento** |
    //! | duplicar | um caminho novo no **documento** (`hierarchy_duplicate.rs`) | cunha a **entidade** |
    //!
    //! Os dois drenos vivem no `render_loop::hierarchy::dispatch`, que corre **~2 300 linhas depois**
    //! da projecção (`vec_scene.reorder_to`). ⇒ a fotografia do quadro do gesto guardava um mundo e um
    //! documento que discordavam, e o quadro seguinte reconciliava-os **sem entrada nenhuma** — que é
    //! a definição do passo fantasma.
    //!
    //! ⚠️ **Apagar tira do MUNDO e duplicar põe no DOCUMENTO**: as duas metades da mesma latência, e um
    //! gate só deixaria metade da classe por medir.
    //!
    //! # ⚠️ Os CONTROLOS destes gates mudaram com a cura, e a mudança é o achado
    //!
    //! Na 1.ª redacção eles afirmavam *«o caminho ainda está no documento neste quadro»* e *«a entidade
    //! ainda não existe»* — ou seja, descreviam o **defeito**, não a lei. Com a reconciliação a correr
    //! antes da captura, os dois passaram a ser falsos: o gesto tardio deixa o estado **já fechado** na
    //! fotografia. *Um controlo escrito contra o mundo doente reprova quando ele sara* — os de hoje
    //! afirmam que a reconciliação FEZ o trabalho, que é o que impede o ponto fixo de ser vácuo.

## `shells/desktop/src/scene_mapping.rs:4`

    //!
    //! O doc do [`ph2d_app_motion::field_gizmo::scene_window_wh`] escreve-a desde 2026-07-25:
    //! *«todo mapeamento mundo↔tela do chrome da cena TEM de usar isto»*. Sob um split do centro
    //! (a ferramenta MOTION activa — ⛔ **não** a timeline) a cena **não é a janela**: ela desenha num sub-rectângulo
    //! `[0, 0, w, h·t]`, e a projecção MUDA — não é um recorte.
    //!
    //! | quando | quem foi posto na porta | quem ficou de fora |
    //! |---|---|---|
    //! | 2026-07-25 | a grade e o gizmo | todo o resto |
    //! | 2026-08-25 | o **pan** (a cena andava `t` vezes o que o cursor andava) | todo o resto |
    //! | 2026-09-17 | o `vec_world_at` (o botão do HUD, `~340 px` ao lado) | **50 chamadas** |
    //! | **hoje** | **as 40 ligações que alimentam a câmera** | — e um CENSO impede a quinta |
    //!
    //! ⛔⛔ *Três vezes a cura foi pôr **um** consumidor na porta e deixar os outros.* É por isso que
    //! esta wave não acaba numa conversão: acaba num **censo** ([`crate::tests`] na shell), que reprova
    //! a chamada nova que não passe por aqui.
    //!
    //! # ⚠️ Os DOIS eixos, e não só o `y`
    //!
    //! O [`ph2d_render::Camera2d::screen_to_world`] deriva `aspect = w/h` e `half_w = half_h·aspect`.
    //! Sob um split HORIZONTAL o `h` encolhe ⇒ o `aspect` **cresce** ⇒ o `x` também sai errado. A
    //! leitura *«é o y que está deslocado»* descreve o sintoma mais visível, não a conta.
    //!
    //! # ⚠️ Por que a porta mora no `AppGfx` e não no `App`
    //!
    //! Havia **DUAS** respostas à mesma pergunta — `App::scene_window` (`&self`) e
    //! `field_gizmo_host::scene_window_of` (`&AppGfx`) — e a segunda existe porque a primeira pede o
    //! `App` INTEIRO emprestado, o que colide com o `&mut gfx.<campo>` do mesmo quadro (o
    //! `gizmo_drag.rs` tem isso escrito ao lado do código). ⇒ a porta pede o **mínimo** (`&AppGfx`) e
    //! as outras duas **delegam**: quem empresta menos serve os dois chamadores, e o contrário não.
    //!
    //! # ⭐ Fora do split é BYTE-IDÊNTICO
    //!
    //! `CenterSplit::None` ⇒ [`scene_camera_window`] devolve a janela inteira. É essa propriedade que
    //! torna a conversão de 40 sítios segura, e ela tem gate próprio.

## `shells/desktop/src/merge_layers.rs:6`

    //!
    //! # O que este verbo é, ao lado do vizinho
    //!
    //! | | **Merge Sprites** | **Merge to Layers** |
    //! |---|---|---|
    //! | pixels | uma imagem, achatada | uma imagem achatada **e** um documento com N camadas |
    //! | as fontes | despawnadas | despawnadas |
    //! | dá para separar depois | **não** | **sim** — abra o Painter |
    //!
    //! ⚠️ **A geometria é EXACTAMENTE a mesma**, e é de propósito: a mesma união, o mesmo warp, o mesmo
    //! «over». Duplicar aquele laço para o modo novo seria pedir que duas cópias concordassem para
    //! sempre — em vez disso o [`crate::hero_intents::sprite_merge`] ganhou um **modo** que, além de
    //! compor, guarda o que cada fonte pôs em cada pixel. *O que o Painter mostra é o que o ecrã já
    //! mostra, porque é a mesma conta.*
    //!
    //! # Por que a sprite continua a ter a textura achatada
    //!
    //! ⚠️ **A sprite tem de desenhar e de GRAVAR sem o Painter.** O documento vive na ferramenta; o
    //! ficheiro do projeto guarda os pixels da sprite. Se a única cópia fosse o documento, fechar o
    //! app antes de abrir o Painter perdia a fusão — o mesmo defeito das ilhas do BG-Removal
    //! (2026-08-20), que abriam perfeitas e gravavam vazias.
    //!
    //! Então há duas coisas, e as duas são verdade ao mesmo tempo: a **textura** (achatada, durável) e
    //! o **documento** (em camadas, na ferramenta). Abrir o Painter mostra o segundo; tudo o resto vê
    //! o primeiro.
    //!
    //! # A ordem das camadas
    //!
    //! ⚠️ **De baixo para cima, na ordem em que o «over» as compôs.** A fusão itera as fontes e põe
    //! cada uma *sobre* o acumulado, por isso a última fica em cima; o
    //! [`ph2d_tool_painter::PainterTool::add_raster_layer_with_pixels`]
    //! insere no topo, por isso adicioná-las por essa mesma ordem reproduz a pilha. Trocar a ordem aqui
    //! daria um documento que compõe diferente do que a sprite mostra — e ninguém saberia qual dos dois
    //! está certo.

## `shells/desktop/src/vec_frame_resize.rs:5`

    //!
    //! O gizmo escreve a **pose** (`Transform.scale` da entidade) e a pose de um pai é herdada por
    //! todo descendente — é isso que um grafo de cena É. Numa forma-folha isso é exatamente o certo:
    //! escalar um desenho é escalar o desenho. Numa MOLDURA é o oposto do que o artista pede: os
    //! filhos esticam junto, a tipografia achata, e a regra de âncora que ele acabou de armar nunca
    //! chega a correr, porque a moldura não mudou de **caixa** — ela mudou de **escala**.
    //!
    //! O painel já fazia o certo: `W`/`H` reescrevem a GEOMETRIA da moldura
    //! ([`crate::input_dispatch::apply_vec_transform`]) e deixam a pose em paz. Este módulo é o que
    //! leva a alça do gizmo à mesma porta, para que as duas metades da interface não respondam
    //! diferente à mesma pergunta.
    //!
    //! # Quem decide é o OBJECTO, pelo checkbox **Resize Box** (decisão do Enio, 2026-08-03)
    //!
    //! A regra vive em [`ph2d_ecs::resizes_box`]: molduras e os filhos delas nascem marcados, todo o
    //! resto nasce desmarcado, e o checkbox da seção Transform é o override.
    //!
    //! ⚠️ **Eu argumentei contra o checkbox e a decisão foi revertida — as duas metades ficam aqui**,
    //! porque uma cerca de Chesterton sem o lado que perdeu é um convite a refazer o argumento. O que
    //! eu disse: um interruptor persistente responde com estado por-OBJECTO a uma pergunta sobre a
    //! intenção do GESTO, e esquecer de o restaurar torna o *próximo* gesto silenciosamente errado.
    //!
    //! O que a decisão viu e o argumento não: **o comportamento antigo é o CERTO para um objeto de
    //! game**, e ele tem de continuar alcançável *dentro* de uma moldura — um contentor que é cenário,
    //! e não layout, quer escalar o conteúdo junto. Sem o override isso ficava inexprimível: a única
    //! saída era seleccionar o conteúdo, o que muda o que o gizmo mede.
    //!
    //! ⚠️ **E nos FILHOS o checkbox não é decoração:** o tamanho de um filho é ENTRADA da disposição, e
    //! o passe de layout mede a **caixa**. Escalar a pose de um filho num fluxo deixa a caixa onde
    //! estava, e o fluxo re-flui em volta de um número que já não descreve o que se vê.
    //!
    //! O default derivado (em vez de um bool escrito em cada entidade) é o que faz uma forma arrastada
    //! para dentro de uma moldura passar a valer a regra de dentro no mesmo frame — ver
    //! [`ph2d_ecs::resize_box_default`].

## `crates/ph2d-app-components/src/instance_variant.rs:4`

    //!
    //! Uma **variante derivada** (o modelo Unity / flecs `IsA`) é *um mestre cuja raiz é ela própria
    //! uma instância de outro mestre*: `MasterRoot` **e** [`InstanceOf`] na mesma entidade. O sync não
    //! precisou de uma linha para isso — [`crate::instance_sync`] procura *toda* entidade cujo elo
    //! aponta para um `MasterRoot` vivo, e uma variante é exactamente isso.
    //!
    //! ⭐ **Medido por sonda antes de se escrever código** (2026-08-27): editar a base leva a alteração
    //! à variante **e às instâncias da variante** num passe só (`passe = 2`, duas escritas). A ordem
    //! topológica sai de graça pela mesma coincidência que a F5.1 nomeou — a ordem de criação é a de
    //! dependência, e `live_instances` ordena por `StableId`.
    //!
    //! # ⭐⭐ O MAPA de re-key já vive no mundo: são os próprios elos
    //!
    //! A tabela do doc 04 §2.6 promete *«trocar variant↔base preserva os overrides (re-key
    //! determinístico)»*, e a pesquisa do endereçamento chama-lhe *«a operação que nenhum outro sistema
    //! consegue, porque nenhum tem chave mestre-relativa com caminho»*. Aqui ela é mais barata do que a
    //! pesquisa previa, e a razão é a tese do ADR-0164: **a instância é uma entidade real**, então as
    //! peças da variante *já dizem* de que peça da base nasceram.
    //!
    //! ```text
    //! sonda 2026-08-27:  sid peca da base = 2 | sid peca da variante = 4 | link da peca da variante = 2
    //! ```
    //!
    //! ⇒ `base 2 → variante 4` lê-se invertendo os elos da variante. **Sem nomes, sem caminhos, sem
    //! heurística** — é a diferença entre este re-key e o `ByName`/`ByHierarchy` do Unity, que o próprio
    //! Unity documenta como não-reproduzível com nomes duplicados (HR-5).
    //!
    //! # A troca é um RE-KEY, e o resto já estava construído
    //!
    //! [`swap`] muda três coisas — o elo da raiz, os elos das peças e as chaves de override — e **para
    //! aí**. Quem materializa as peças que o mestre novo tem a mais, quem apaga as que ele não tem, e
    //! quem guarda a excepção de uma peça que morre ([`ObjectInstance::orphans`]) já é o passe
    //! estrutural da F5.1 + F5.3. *A peça que falta pode já estar construída — meça a estrutura dela
    //! primeiro.*
    //!
    //! ⭐ E é isso que faz a troca ser **reversível de graça**: trocar para uma variante que não tem a
    //! peça `X` sepulta a excepção de `X` nos órfãos; trocar de volta materializa `X` e **exuma-a**.
    //!
    //! # ⛔ Mestre NÃO aparentado: `None`, e é uma decisão
    //!
    //! Sem antepassado comum não há mapa — só heurística por nome ou por caminho. Os três modos do
    //! Unity (`Nenhum` · `Por nome` · `Por hierarquia`) são um gesto próprio, com relatório, e **nunca
    //! automáticos** (doc 04 §2.6). Aqui a resposta é a recusa, e ela é honesta: a alternativa seria
    //! aplicar a excepção do artista a um objeto que calhou de ter o mesmo nome.

## `crates/ph2d-app-components/src/parallax_bridge.rs:7`

    //!
    //! # A lei, e a referência que ela usa
    //!
    //! ```text
    //! saida = autorada + centro_da_vista · (1 − k)
    //! ```
    //!
    //! Com a câmera na **origem** nada se mexe: o artista põe o fundo onde ele deve estar *quando a
    //! câmera está em zero*. ⚠️ **A alternativa era guardar a posição da câmera no instante da
    //! autoria** — e ela custa ESTADO, que teria de sobreviver ao ficheiro, ao `Ctrl+Z` e ao
    //! rebobinar. A origem do mundo não custa nada, e **é o que o alvo faz** (medido: o declive do
    //! `Parallax2D` da Godot é exactamente `1 − scroll_scale`).
    //!
    //! # ⛔ Sem câmera de jogo, NADA é conduzido — e isso é a lei, não uma guarda
    //!
    //! A vista é a da **câmera do jogo**, nunca a do editor, pela razão que o `fase_game_camera` já
    //! escreve: uma corrida que dependesse de onde o artista rolou o ecrã seria outra corrida em cada
    //! máquina. Sem câmera na cena o fundo fica **onde o artista o pôs**.
    //!
    //! ⛔⛔ **E quem o devolve é esta ponte, nunca o `settle`** (auditoria 26, §1.2). A 1.ª redacção
    //! dizia *«o `settle` devolve-lhe a pose autorada»* — e ele só ESQUECE: a pose deslocada ficava
    //! no mundo, a captura seguinte gravava-a como documento, e quando a condução voltava ela era
    //! deslocada DUAS vezes (medido: `x = 200 → 400` com `k 0,5`). Hoje todo objecto que a ponte
    //! deixa de conduzir — sem câmera, `k` neutro, a câmera a atravessá-lo, o componente retirado, um
    //! `UiCanvas` anexado — é LARGADO por `release_to_authored`, numa varredura do ledger no fim.
    //!
    //! # ⚠️ O caso que decide se esta ponte está CERTA: o artista ARRASTA o fundo
    //!
    //! O ledger tem uma lei genérica — *«se o `before` deste quadro não é o que o motor deixou no
    //! anterior, outra mão escreveu»* — e ela sozinha **não serve aqui**, porque o que a outra mão
    //! deixou no mundo é a pose **DESLOCADA**, não a autorada. Tomá-la como autorada faria o fundo
    //! saltar `centro · (1 − k)` no quadro seguinte.
    //!
    //! ⇒ a ponte **reconhece a própria escrita**: ela sabe o que devia lá estar (`autorada` deslocada)
    //! e, quando o vivo é outro, **recupera** o autorado subtraindo o mesmo deslocamento. A comparação
    //! é exacta de propósito — o valor foi escrito por esta função, bit a bit.
    //!
    //! # ⛔ Quem tem `UiCanvas` fica de fora
    //!
    //! Um HUD já é conduzido pelo [`crate::hud_bridge`], e dois motores sobre o mesmo `Transform`
    //! escreveriam um por cima do outro. *A raiz de um HUD é a paralaxe com fracção `0`, e quem a
    //! implementa é aquela ponte.*

## `shells/desktop/src/main.rs:11`

    //! Layered subsystems (each gated to keep the demo bootable even if
    //! one fails — never crash the shell over an integration-demo issue):
    //! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
    //! - **M6** AssetDb loads 16 real PNG files from `assets/sprites/` (auto-
    //!   generated on first launch) and composes them into a 256×256
    //!   RGBA8 atlas. Falls back to procedural dummy if anything fails.
    //! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
    //!   keeps the GC budget warm for future script-driven gameplay.
    //! - **M12** editor data layer: `ZenMode` (Tab toggle), `ToastQueue`
    //!   (T key adds info toast), theme switch (M key flips Dark↔Light),
    //!   and a `FloatingPanel` (Procreate-style selection demo). Visible
    //!   via window title since Vello widget paint requires sharing the
    //!   wgpu Surface with the sprite pipeline (see `integration.rs`).
    //!
    //! M8 add-on (gamepad path):
    //! - gilrs adapter (`gilrs_adapter` module) pumps gamepad events into
    //!   [`ph2d_input::InputState`] each frame, BEFORE sim tick.
    //! - Connection / button / axis events log to the terminal at the
    //!   `[Nms]` timestamp prefix used by the rest of the shell.
    //! - Axis logs filter dead-zone jitter (only `|value| > 0.25`).
    //! - Pencil events are wired through the abstraction but not produced
    //!   by this shell (iPad shell will, in M9+).
    //!
    //! Out of scope here:
    //! - **M8 → ScriptHost**: `InputState` lives on `App`, but routing into
    //!   `ph2d.input` Luau snapshot is a follow-up.
    //! - **M11** Vello text/widget overlay — needs surface-sharing pass.

## `shells/desktop/src/render_loop/fase_physics_overlay.rs`

```text
            // ⛔⛔⛔ **A JANELA DA CENA, e nunca a da janela — a QUINTA vez que esta lei é paga**
            // ([`crate::scene_mapping`], que lista as outras quatro).
            //
            // ⚠️ **MEDIDO pela foto da cena `PH2D_RAY_SMOKE=1` (19/09), e a aritmética fecha antes
            // do código:** com a ferramenta MOTION activa — que o `~/.ph2d/layout.txt` do dono
            // **reactiva no quadro 1** — a cena desenha num sub-rectângulo `[0, 0, w, h·t]` e a
            // projecção MUDA (não é um recorte). O [`ph2d_render::Camera2d::world_to_screen`]
            // deriva os **dois** eixos de `h`, logo com a janela inteira tudo sai `h/(h·t)` vezes
            // maior e o zero vertical fica em `h/2` em vez de `h·t/2`.
            //
            // A conta da foto: janela `1930×1012`, banda `≈557`, `height_world ≈ 11,14 m` ⇒ o raio
            // do olho, que nasce em `y = +0,35`, tinha de ser desenhado a `≈261 px` e apareceu a
            // **`≈474`** — `~210 px` abaixo e `1,8×` mais comprido. *Os corpos estavam no sítio
            // certo e a física que se desenha por cima deles não.*
            //
            // ⛔ **E isto NÃO é da timeline estar aberta:** o único escritor de um `CenterSplit` é
            // o ramo da ferramenta Motion — a correcção que a wave do HUD pagou um dia antes, por
            // ler *onde o defeito aterrava* como *o que o causava*.
```

## `shells/desktop/src/render_loop/fase_snapshots_publish.rs`

```text
            // ⭐⭐⭐ **A caixa de objecto só existe quando NENHUMA ferramenta autora no canvas**
            // (ADR-0112) — a vectorial fora do Select e do modo Forma (lá, só alças: F10), **a do Flip sempre, e o Pixel Sprite ▸ Edit** (o `ramo_pixel`).
            // As alças registam hit-rects, e os dois ramos de canvas (`ramo_ferramenta_vetorial` e
            // `ramo_flip_premidos`) exigem o MESMO `on_canvas`: uma caixa sobre um canvas de
            // autoria é um ladrão de cliques, seja qual for a família do objecto que a publica.
            //
            // ⚠️ **E nunca durante o modo de PREVIEW** (W7r): a caixa é derivada da pose
            // AUTORADA, então enquanto a máquina move a forma ela fica para trás e passa a
            // descrever um lugar que a forma já não ocupa — é a razão pela qual o ADR-0128
            // recusou cinco vezes um gizmo sobre geometria que se move.
```
