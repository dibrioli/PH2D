# ARQUIVO — CLAUDE.md (história, 136 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-1859,1996-2007` do original.
>
> ⚠️ **A única alteração ao corpo:** 12 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Sprite Inspector** ([ADR-0069..0074](../../architecture/decisions)) — ⚠️ **esta linha dizia
  «fechado sem pendência» e a spec pede 12 seções: existiam 9.** ✅ **As três que faltavam nasceram
  em 2026-08-22/23** (`line/Sprite`): 9-Slice, Sockets/Âncoras (com o gizmo de canvas) e Animation.
  ⚠️ *A informação existia desde 2026-05-31 num handoff **arquivado**, e o roteador dizia o
  contrário — o roteador é o que se lê.*
  **Aberto:** ✅ **a §11 Animation NASCEU (2026-08-23) — as DOZE seções existem.**
  ⛔ **O `SpriteFrames` da spec §8.3 NÃO foi construído, e é uma recusa medida:** o pool de frames
  já existe (a **grelha** `hframes × vframes`, cujo índice `Sprite::frame` é o **único sink vivo** —
  o `SpriteSheetRef` é proveniência de autoria, não índice). Uma animação é um **intervalo nomeado
  sobre as células que a sprite já tem** — o modelo do Aseprite aplicado ao pool que existe.
  ⚠️ **O tique corre no PASSO FIXO** (`SimComponent`, o replay tem de o reproduzir) e a lei pura
  **nunca vê um float**; escreve `Sprite::frame` **só quando ele muda** (o undo regista por diff).
  ⚠️ **Tocar uma vez pára no ÚLTIMO frame** · ping-pong não repete as pontas · `repeat_delay` só
  conta se vier outro ciclo · velocidade negativa toca ao contrário, não faz o tempo recuar.
  ⚠️ **`ANIM_TAGS_MAX` é 64 e não os 256 da spec** — o motivo é o dela («típico < 50»): *um modelo
  que aceita o que o painel não mostra produz estado inalcançável*.
  ⚠️ **O TRANSPORTE foi auditado em 23/08 e tinha quatro defeitos numa família só**
  ([doc 21](../../Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md), e a wave está aplicada):
  *«pausado» e «terminado» leem-se igual no `playing == false` e não são a mesma coisa* — a
  reprodução que se **ESGOTOU** volta ao princípio quando alguém lhe toca (a caixa **ou** a lista),
  e uma pausa explícita não é tocada; **rebobinar move a IMAGEM**, não só contadores; e a caixa
  «Playing» **pergunta à cena**, nunca ao `WidgetStore` (era dupla fonte de verdade, e o motor
  escreve aquele campo sozinho). ⚠️ **A barra de frames ARRASTA** (pedido do Enio) — ela era
  desenho, hoje é um `Slider` registado que mede **posição** e não progresso, e **agarrá-la pausa**
  (o dedo e o tique escreviam o mesmo campo). ⭐ A régua vivia em **três cópias** e uma mutação
  sobreviveu a mudar só a do pintor: hoje é `scrub_position` ↔ `scrub_cell`, uma lei em dois
  sentidos com gate de ida-e-volta. ⛔ A §11 tinha 33 gates e **nenhum que carregasse num pixel** —
  hoje tem `seam_anim.rs`. ⚠️ **MEDIDO e não curado:** com a animação a tocar, um quadro com input
  regista um passo de undo (o relógio vive num `SimComponent` registado) — **família
  pré-existente**, a física faz o mesmo com o `Transform`; as três saídas estão na auditoria §4 e a
  escolha é do Enio.
  ⭐ **PINTAR UMA FOLHA DESDOBRA-A** (Enio, 2026-08-23, com foto): sob pré-visualização de
  ferramenta o extract troca a UV pelo rect INTEIRO da textura transitória, mas o quad continuava a
  ser o de UMA célula — a tira saía **esmagada 8:1**. ⚠️ **E o caminho do PONTEIRO fazia a mesma
  conta** (`sprite_image_to_screen_affine`), o que os deixava consistentes um com o outro e errados
  com o artista; por isso os dois chamam a MESMA função (`sim_extract_sheet::unfolded_quad`).
  ⚠️ **O desdobrado centra-se no PIVÔ e ignora o frame vivo** — ancorá-lo na célula viva (a 1.ª
  versão) fá-la-ia **deslizar debaixo do pincel**, porque o tique continua a andar. ⛔ A
  pré-visualização da grelha faz o **contrário** e também está certa: ali a célula viva **é** o quad
  real. ⭐ E como a folha aberta mostra tudo, o `frame` deixa de ter efeito visível — daí a
  **célula extra acima dela, a tocar a animação enquanto se pinta**, ⚠️ **mesmo com o transporte
  pausado**: ela corre sobre uma **cópia** do animador (a lei pura também desiste com
  `playing == false`), e do que a cópia produz volta só o relógio — o `playing` do documento fica
  intacto. ⚠️ **As LINHAS da grelha seguem o MODO** (`lattice(.., unfolded)`): a folha pintada
  centra-se no pivô e a pré-visualizada dispõe-se à volta da célula viva, e as duas disposições
  **nunca** coincidem (o desvio é `(lcol + ½ − hf/2)·cw`) — foi o 2.º report com foto.
  ⚠️ **E a CAIXA DO GIZMO envolve a folha aberta** (`sheet_grid_overlay::gizmo_box`) — ela ficava do
  tamanho de UMA célula no meio de oito. ⛔ A escolha vive numa função com gate, e **não no fio**: em
  `snapshots::build_view` ela não é alcançável de um teste, e a mutação que a desligava compilava e
  passava a suíte inteira.
  ⭐ **A GRELHA VÊ-SE** (Enio, 2026-08-23): a caixa **«Show sheet on canvas»** (§4 Sprite Sheet, só
  aparece com grelha) abre a folha no canvas — as outras células esmaecidas no lugar delas, com as
  linhas dos cortes e a viva contornada. ⚠️ **Fantasmas de PRESENTE, nunca documento** (o molde é o
  fan-out do 9-slice) e o interruptor é **vista**: vive só no `WidgetStore`, sem barramento, sem
  undo, sem save. ⛔ Um clique numa célula **não** escolhe o frame — pede hit-test de canvas a
  competir com a seleção; a barra de frames e o campo já o fazem.
  ✅ **IMPORTAR ASEPRITE (`.ase`) — FEITO em 2026-08-23** (crate-folha
  [`ph2d-aseprite`](../../../crates/ph2d-aseprite) + [`ase_import.rs`](../../../shells/desktop/src/ase_import.rs)):
  largar o ficheiro nativo dá **UMA** sprite com grelha + a biblioteca de animações dele.
  ⚠️ **Clean-room da spec pública** (o Aseprite é GPLv2; a especificação do formato é documentação).
  ⚠️ **O corte entre as duas portas é o que cada uma SABE**: o par `.png`+`.json` traz rectângulos
  com nome ⇒ N sprites soltas; o `.ase` traz a **autoria** ⇒ uma sprite com grelha, que é o modelo
  da §11. ⚠️ **A ordem dos quadros é o CONTRATO** — uma tag indexa **células**, então a folha é
  empacotada em linha; por colunas dá uma folha bonita e todas as animações trocadas.
  ⚠️ **UMA TIRA sempre que couber** (o `hframes` do inspector fica legível); o teto é
  `MAX_SHEET_EDGE_PX = 8192`, que é **memória de GPU** (`max_texture_dimension_2d`).
  ✅ **E a recusa da duração por-FRAME reabriu e FECHOU** (spec §8.12: *«não há quem a produza»* —
  há, é este importador, e nos ficheiros reais elas **variam**): `AnimationTag::per_frame_ms`, vazio
  = uniforme. ⭐ A lei pura não precisou de refactoração — o `step_ticks` já perguntava **por
  frame**, era só a resposta que era uniforme. ⚠️ Curto ou `0` caem no `frame_ms`, então **não há
  estado inválido** quando o intervalo muda. ⭐ **E o Inspector EDITA-O** (Enio pediu: *«se não tiver um
  parâmetro de duração para cada quadro, crie»*) — o campo mora **colado à barra de frames**, que já
  é o selector de célula: *um painel que pergunta duas vezes «qual quadro?» é um painel em que os
  dois podem discordar*. ⚠️ `0` = herda · declarar uma célula **não** escreve as outras · limpar
  **encolhe** o vetor (senão o aviso de ritmo próprio mente para sempre).
  ⚠️ O que o ficheiro traz e não honramos sai numa **nota que nomeia a camada** (tilemaps, z-index
  de cel, modo de mistura de grupo) — *um importador que ignora em silêncio é pior que um que
  recusa*. ⭐ **O smoke ESCREVE o `.ase`** (`PH2D_ASE_SMOKE=1`), então testá-lo não precisa do
  Aseprite instalado — e há gate a correr o escritor do smoke pelo leitor real.
  ⭐ **E 18 ficheiros escritos pelo Aseprite REAL lêem-se, 0 recusados** (as 12 fixturas de teste do
  repositório oficial + 2 exemplos MIT + 4 personagens): o instrumento é
  `cargo run -p ph2d-aseprite --example ase_info -- <ficheiro|pasta>`, que corre o **mesmo** parse
  do produto. ⚠️ Dois achados que só ficheiros reais dão: a duração **varia por quadro** em
  ficheiros comuns, e personagens reais chegam **sem tags** — o que torna a regra «sem tags recebe
  uma» o caminho normal, não a excepção.
  ⚠️ **E o `.ase` não aparecia no diálogo «Import…»** (Enio, no mesmo dia) — o defeito **não era o
  `.ase`**: o drop roteava por um predicado (11 extensões) e o diálogo oferecia uma lista **escrita
  à mão** com 4, então o `.gif`/`.psd`/`.ora` estavam invisíveis lá **há meses**. ⇒
  [`import_router.rs`](../../../shells/desktop/src/import_router.rs): a **lista** é a fonte
  (`ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS` + `ase_import::ASE_EXTENSIONS`), o predicado é derivado
  dela, e **as duas portas chamam a mesma função**. *Uma lista escrita à mão ao lado de um
  predicado é duas respostas à mesma pergunta, e a que o artista vê é a que envelhece.*
  ✅ **OS SINAIS (§8.10) EXISTEM** (2026-08-23) — e saem pelo **outbox** do `ph2d-runtime`, não
  pelo ActionBus que a spec desenha: ela é anterior ao ADR-0143, e um sinal no bus faria o motor de
  animação **chamar** o editor. ⚠️ **Dois nomes AUTORADOS na tag** (`signal_on_finish` /
  `signal_on_loop`), vazio = **calada** — é a lei dos contatos da física: acabar e dar a volta
  distinguem-se por serem nomes diferentes, não por um campo de fase. ⚠️ Um tique atrasado colapsa
  num sinal só, **com a contagem dentro** (`SignalOrigin::Animation::cycles`) — dez passos que
  ninguém deu é ruído. ⚠️ **A pré-visualização é MUDA**: pegar no pincel não pode tocar um som.
  ⛔ Dos quatro eventos da spec, dois ficam fora **com motivo medido**: o `FrameChanged` por
  FREQUÊNCIA (~12×/s por sprite, e quem o consome já lê o `Sprite::frame`) e o `AnimationChanged`
  por NATUREZA (é um clique no Inspector, não um facto da cena). Detalhe:
  [spec 08, secção final](../../Sprite_projeto/08_animation_inline.md) ·
  ✅ **uma âncora já MOVE coisas** (2026-08-22): `ph2d_ecs::AnchorMount` faz dela um
  **QUADRO na hierarquia** ([ADR-0072-amendment-1](../../architecture/decisions/0072-amendment-1.md)),
  autorado pela linha «Rides Parent Anchor» da §12 e demonstrado em `PH2D_MOUNT_SMOKE`.
  **Escolher uma âncora POUSA o objeto nela** (só a posição — o ângulo é do filho; e **nunca** ao
  escolher «—», porque desmontar é largar), com «Reset to Anchor» a refazê-lo; a âncora montada
  fica **visível ao mexer no filho, mesmo com a §12 fechada**; e o dono tem duas caixas —
  **«Always show anchors»** (viva) e **«Show anchors at runtime»** (⛔ grava e **não tem quem a
  leia**: não há modo de jogo, o `shells/game`/R1 está adiado).
  ⚠️ **A precedência do overlay é `Editing` > `AlwaysVisible` sobre a MESMA entidade** — o modo de
  edição é *superset* (as mesmas âncoras, mais o realce e as alças). ⛔ Ao contrário, a caixa
  **rouba o destaque à selecionada**, e o gate afirmava-o: *um gate verde pode pinar um defeito de
  produto*, e os três desta linha foram apanhados por smoke.
  ⚠️ **A lei entra nas DUAS travessias de mundo pela MESMA função** (`mount_state`) —
  `propagate_transforms` e `world_transform`: só numa, a espada **desenha** na mão e todo gesto
  agarra-a na origem do pai. ⛔ Das três superfícies do ADR-0072 §2.6 só a **Rust** tinha onde
  existir; **Luau e MCP estão BLOQUEADOS por outro subsistema** (o `ScriptHost` do desktop corre um
  script placeholder e **nunca** recebe `provide_read`; o `McpHost` é um `MemoryHost` de JSON, com
  «backends reais em S2/S3» escrito nele) — o gatilho de cada uma está na
  [spec §7.8-bis](../../Sprite_projeto/07_named_anchors.md), e construí-las hoje repetiria, um nível
  acima, o defeito que esta wave curou · o `AnchorData::user_data` não tem UI, com o `variant_editor`
  órfão a apontar-lhe · os 4 goldens seguem `unimplemented!()` (falta o arnês headless).
  ✅ A UI de Save/Open **existe** desde 2026-08-23 (ver *Editor / shell*).
  **Smokes:** `PH2D_SLICE_SMOKE=1..3` · `PH2D_SOCKET_SMOKE` · `PH2D_MOUNT_SMOKE` · `PH2D_ANIM_SMOKE` ·
  `PH2D_ASE_SMOKE` ·
  `PH2D_SHEET_SMOKE` · `PH2D_EMISSIVE_SMOKE` · `PH2D_DITHER_SMOKE`.
  **Ler:** ⚠️ [auditoria de 7 lentes](../../Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md)
  (o que estava morto/incompleto, **com o que já foi curado marcado**) ·
  ⚠️ [auditoria da §11 Animation](../../Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)
  (aplicada; 11 achados, 11 mutações, e **5 recusas medidas** — leia-as antes de propor um alcance
  de campo ou de mexer no que a lista faz ao clique) · [spec](../../Sprite_projeto/README.md) ·
  [handoffs](../../Sprite_projeto/handoffs/README.md) (⚠️ índice **à mão**: esta pasta não entra no
  `doc-index.sh` porque o `README.md` acima dela é a spec — e até 2026-08-23 os handoffs eram
  **órfãos**, citados por nada)
