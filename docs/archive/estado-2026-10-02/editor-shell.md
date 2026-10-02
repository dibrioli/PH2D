# ARQUIVO — CLAUDE.md (história, 52 linhas)

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
> Recorte: linhas fora de `1-1644,1697-1708` do original.
>
> ⚠️ **A única alteração ao corpo:** 8 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Editor / shell — undo, persistência, inspector** — **uma** fila de undo, snapshot-based, registrada por **DIFF num só
  ponto** (`App::post_frame_undo`), cuja unidade é `ProjectState = {WorldSnapshot + VecScene}` — a MESMA captura que a
  persistência usa (o save só anexa os pixels). Ctrl+S/Ctrl+O salvam o projeto inteiro num postcard versionado.
  ⚠️ **`canonicalize()` ordena por CONTEÚDO, nunca por `Entity::to_bits()`** (id de alocação) — foi isso que fazia todo
  frame virar um passo espúrio; e **toda raiz ganha `RootOrder` explícito**: *não se escolhe um desempate melhor, não se
  tem empate*.
  ⚠️ **Referência durável entre objetos é o NOME** (`stable_name_id`, hash do `Name`), nunca os bits — o undo respawna
  tudo com bits novos, e bits **dentro dos bytes de um componente** envenenam o próprio undo.
  ⚠️ O undo de **PAINÉIS** é sistema separado e **não existe** (decisão do Enio).
  ✅ **O UNDO SEPARA PREVIEW DE DOCUMENTO** (Enio, 2026-08-23: *«corrigir o CtrlZ para ambas»* —
  feito no mesmo dia, [`ph2d-preview-drive`](../../../crates/ph2d-preview-drive/src/lib.rs)): *o documento é o
  valor **AUTORADO**; o que um motor escreve agora é pré-visualização — vê-se, não se guarda nem se
  desfaz.* O motor continua a escrever no mundo (um só sink); a **captura** é que repõe o autorado
  durante a fotografia. ⚠️ **O ledger entra na ASSINATURA da `ProjectState::capture`** — uma
  função-irmã «com ledger» seria a segunda porta pela qual o defeito voltava. ⚠️ **A granularidade
  é o CAMPO**: repor o `SpriteAnimator` inteiro engoliria uma mexida na velocidade a meio da
  reprodução. ⚠️ **O passo nascia por CLIQUE, não por quadro** (mover o cursor não conta como
  input) — e é por isso que tirar só o relógio do componente registado **não** curava. ⭐ A `settle`
  faz a corrida virar **um** passo (*«desfaz a corrida»*), e a lei da **outra mão** impede que uma
  edição feita a meio dela fique por baixo do memo. Vale para o **save** pela mesma porta.
  ✅ **E O TERCEIRO MEMBRO — a timeline — CURADO no mesmo dia**
  ([`ph2d-timeline-preview`](../../../crates/ph2d-timeline-preview)), pelo mesmo ledger. ⚠️ **A nota
  que o deixava de fora estava ERRADA no ponto que decidia o preço:** ela dizia que o censo era
  `O(mundo)`; o `TimelineDoc` **nomeia** quem ele anima (`doc.bindings()`), então é `O(bindings)`.
  *Uma ausência afirmada sem olhar a API é um palpite com cara de medição* — a segunda no mesmo dia
  (a outra: «este app não tem diálogo de ficheiro»). ⭐ **E os QUATRO componentes que a timeline
  escreve entram**: o `Sprite` ficou de fora na 1.ª versão por uma COLISÃO de granularidade (a §11
  conduz aquele componente por CAMPO), e a cura foi **olhar o que a curva de facto escreve** —
  `tint[3]` é um número, não o `Sprite`. *Quando duas granularidades colidem, a pergunta é qual
  delas é grosseira demais para o que o motor faz*
  ([auditoria 21 §4](../../Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)) ·
  **Aberto:**
  ✅ **O FICHEIRO DO PROJETO TEM NOME** (2026-08-23, [`project_io.rs`](../../../shells/desktop/src/project_io.rs)):
  `Save` · `Save As…` · `Open Project…` com diálogo, e os três itens do menu deixaram de ser
  **mudos** (eles consumiam o clique e não faziam nada — *pior que um botão ausente: o artista
  conclui que gravou*). A sessão passa a ter um ficheiro (`App::project_path`), a env só o
  **semeia**, e a barra de título diz qual é. ⚠️ **Abrir pergunta SEMPRE** — *o gesto que destrói o
  trabalho não gravado pergunta; o que grava é que pode ser silencioso.* ⚠️ A extensão é
  **`.ph2dproj`** e **não** `.ph2d`, que já é uma **imagem** neste app (há gate a ligar as duas
  listas). ⚠️ O teclado e o menu chamam as **mesmas** funções, e o `project_save()`/`project_load()`
  sem caminho **morreram** — uma decisão de *onde* escondida dentro de quem executa não é alcançável
  nem por um gate nem por um diálogo ·
  ✅ **`SpriteSource::Individual` PERSISTE — esta nota envelheceu** e mandava reconstruir trabalho
  pago: [`project_sprite_pixels.rs`](../../../shells/desktop/src/project_sprite_pixels.rs) fecha as **oito**
  ferramentas de imagem de uma vez pelo funil `commit_edited_texture`, com a identidade a ser o
  CONTEÚDO (`AssetId` blake3) e precedência por ORDEM sobre o Painter/bake. ⚠️ O
  `CookedTexture` fica de fora **por gate explícito** (`should_collect`) — a pergunta aberta é se
  alguém o re-deriva no load, não se ele devia ser embutido · limpar o `vec_history` morto
  (subsumido pela captura).
  **Ler:** [`project.rs`](../../../shells/desktop/src/project.rs) · [`undo.rs`](../../../shells/desktop/src/undo.rs) ·
  [história](../estado-2026-08-18/editor-shell.md)

