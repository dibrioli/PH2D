# 01 — O que é OBJECTO, o que é COMPONENTE (proposta, 2026-10-07)

> **Pedido do dono, 07/10:** *«Godot brilha porque tudo é objeto e o usuário sabe muito bem como criar
> e onde encontrar. Nós aqui temos um problema: não está bem definido o que é objeto, o que é
> componente. O que é a animação/Timeline? Na Godot basta acrescentar um AnimationPlayer na árvore e
> sabemos onde ficam as animações. Aqui a timeline parece estar solta. Segundo o plano seríamos mais
> próximos de Unity que de Godot — foi uma boa ideia? Defina tudo que é objeto (Hierarchy) e tudo que
> é componente (Inspector). Como ficar mais organizado e intuitivo que a própria Godot?»*
>
> Leitor: o dono (as escolhas do §9) e a janela que implementa (o resto). ⚠️ **É um PLANO**: nada
> se constrói antes do §9 respondido. Não re-litiga a [spec/06](../UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md)
> (tipos e modos, aprovada 03/10) — assenta por cima dela.

## §1 — O estado de hoje (medido no código, 07/10)

| | o que existe | onde |
|---|---|---|
| **Tipos de objecto** | 6, **derivados** do marcador (nunca um campo): Empty · Image · Vector · Flip · Painted · Skeleton | `ObjectKind`, [`ph2d-component-desc/src/lib.rs:334`](../../crates/ph2d-component-desc/src/lib.rs) |
| **Hierarchy** | só objectos, em árvore (`ChildOf`), com olho/cadeado/grupo e **um** selo de 3 letras (`PRF`, `CAM`…) | `crates/ph2d-panel-hierarchy/` |
| **Componentes** | **~107** tipos registados (vtable por `StableId` blake3), no máximo 1 de cada por objecto | [`ph2d-ecs/src/scene/registry.rs:314`](../../crates/ph2d-ecs/src/scene/registry.rs) + `ph2d-physics-ecs` |
| **Inspector** | cartões em **12 famílias** de ordem fixa (Identity · Transform · Rendering · Ordering · Geometry · Animation · Physics · Camera · Logic · Audio · HUD · Instance) + *Add Component* (paleta com busca) | `crates/ph2d-panel-inspector/src/paint_familias.rs` · `ph2d-app-components/src/component_palette.rs` |
| **Prefab** | sub-árvore marcada `MasterRoot` que vive **no mundo**; cópias = `InstanceOf` + overrides | [`asset_index_build.rs:6`](../../crates/ph2d-app-components/src/asset_index_build.rs) |

**O que está SOLTO** — dado que o artista cria e que não tem lugar na Hierarchy:

| sistema | onde vive hoje | por que confunde |
|---|---|---|
| **Timeline** | **um** `TimelineDoc` por projecto (`ProjectFile.timeline`); as pistas apontam objectos por `wire_id` ([`binding.rs:41`](../../crates/ph2d-timeline/src/binding.rs)) | a animação do Herói não está no Herói: está num painel global |
| **Motion Nodes** | **um** `MotionDoc` por projecto (`ProjectFile.motion`); publica formas por NOME | o que ele cria aparece na cena sem dono visível |
| **Mundo físico** | `PhysicsSettings` global (`ProjectFile.physics`), painel próprio | a gravidade não está em lado nenhum da árvore |
| **Mixer de áudio** | `SceneAudio` global | idem |
| **Duas animações** | `SpriteAnimations`/`SpriteAnimator` (intervalos da grelha) **e** a Timeline | duas respostas para «onde anima isto?» |

**E um nome com dois donos:** na tela o molde chama-se **Prefab** (`asset.kind.component` ⇒ `"Prefab"`),
mas o módulo, a crate e os docs chamam-lhe **Componente** (`ph2d-app-components`, `docs/Components/`,
`CLAUDE.md §5.1`). A mesma palavra nomeia o *molde* (sentido Figma) e o *cartão do Inspector* (sentido
Unity) — é a forma do defeito «chave e texto do MESMO tipo» um nível acima.

## §2 — O oráculo: o Godot CORRIDO (MIT, 4.7.2)

Fixture: [`ferramentas/godot_4.7.2_taxonomia.txt`](ferramentas/godot_4.7.2_taxonomia.txt), gerada por
[`ferramentas/godot_taxonomia.sh`](ferramentas/godot_taxonomia.sh) (`--dump-extension-api`, sem ler fonte).

- **1 036 classes: 283 nós · 416 recursos.** «Tudo é nó» é meia-verdade: **o Godot tem DUAS
  categorias**, nó (na árvore) e recurso (dentro do Inspector — `Animation`, `Shape2D`, `Texture2D`,
  `AudioStream`, `SpriteFrames`…).
- **Muitos nós são componentes disfarçados.** 17 nós não desenham nada (`AnimationPlayer`, `Timer`,
  `AudioStreamPlayer`, `NavigationAgent2D`, `HTTPRequest`…) e vários `Node2D` só modificam o pai
  (`CollisionShape2D`, `RemoteTransform2D`, `VisibleOnScreenNotifier2D`, `PathFollow2D`).
- ⇒ **O segredo do Godot não é «tudo é nó». É «tudo tem UM lugar visível»:** a animação de X está
  num nó dentro de X, e as `Animation` estão dentro desse nó.
- **O preço que ele paga** (é aqui que se ganha dele):
  1. **Árvore inchada:** uma personagem = `CharacterBody2D` + `Sprite2D` + `CollisionShape2D` +
     `AnimationPlayer` + `Timer` + `AudioStreamPlayer2D`, ou seja 6 linhas para UMA coisa que o
     artista nomeia.
  2. **Regras de pai escondidas:** o `CollisionShape2D` só funciona como filho de um corpo, e o
     erro é um triângulo amarelo. A estrutura da árvore carrega significado e parte-se ao arrastar.
  3. **O *Create New Node* é a árvore de herança:** 50 tipos sob `Node2D`, ordenados pelo nome da
     classe e não pelo propósito.
  4. **O Inspector mostra secções de herança** (`Node2D ▸ CanvasItem ▸ Node`): vocabulário de
     engenheiro.
- **E o do Unity** (sem oráculo instalado; é o desenho público): uma linha por coisa, que é bom. Mas
  a animação parte-se em **5 peças**: `Animator` (componente), `Animator Controller` (asset),
  `AnimationClip` (asset), `Timeline` (asset) e `PlayableDirector` (componente). É exactamente a
  confusão do pedido. E os componentes são **invisíveis** na Hierarchy.

## §3 — O veredito: «Unity ou Godot?»

**Por baixo, ficamos com o modelo Unity (ECS), e foi a escolha certa.** É o que desacopla as linhas
([ADR-0075](../architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md))
e o que aguenta 100 k objectos a 60 Hz. Trocar o motor por uma árvore de nós não é preciso: **o que
falta é uma REGRA de interface, não um motor.**

> **Objecto como no Unity, LUGAR como no Godot, MODO como no Blender** (spec/06).

## §4 — As leis (as definições)

1. **OBJECTO = algo que ESTÁ na cena.** Tem lugar no mundo (transform) ou agrupa coisas que têm. O
   artista dá-lhe nome. ⇒ **Hierarchy.**
2. **COMPONENTE = algo que um objecto TEM ou FAZ.** Não existe sozinho e não tem posição própria (usa
   a do objecto). ⇒ **cartão no Inspector, e ícone na linha do objecto na Hierarchy.**
   - O teste: *«isto pode existir sem um dono?»* Se não pode ⇒ componente.
3. **CONTEÚDO = o que se EDITA dentro de um objecto, num modo.** São as camadas da Imagem, os ossos
   do Esqueleto, os quadros do Flip, os clipes da Animação e os nós do Motion. ⇒ **sub-linhas
   recolhidas sob o objecto** (o Outliner do Blender). Clicar nelas entra no modo ou editor certo.
4. **LUGAR ÚNICO:** tudo o que o artista cria tem resposta para *«onde está?»*, que é *«na linha Y da
   Hierarchy»*. As duas únicas portas fora da árvore têm nome e ficam na barra de cima:
   - **Projecto ▸ Configurações** (teclas, unidades, cores, snaps);
   - **Quadros** (as abas do MiroClone, que são documentos e não cena).
5. **UMA COISA = UMA LINHA.** O Herói é **uma** linha com 5 componentes, não 6 nós. Filhos só quando
   o artista pensa em PARTES (braço, arma, roda). ⛔ Um componente nunca é filho obrigatório: zero
   regras de pai escondidas.
6. **A CENA É UM OBJECTO:** a raiz da Hierarchy, como o nó raiz do Godot. Os componentes de cena
   vivem nela:
   - **Animação da cena**: a Timeline de hoje, sem perder nada;
   - **Mundo físico**: gravidade e passo;
   - **Mixer**: o `SceneAudio`;
   - **Motion**, se a escolha 9.3 for «componente».
7. **PREFAB = um objecto-molde; INSTÂNCIA = objecto com 🔗.** A palavra «Componente» deixa de nomear
   o molde em docs e mensagens ao dono. O nome interno (`ph2d-app-components`) fica até uma linha de
   renome própria (§8 F0).

## §5 — A tabela: cada coisa da PH2D no seu lugar

| coisa | é | Hierarchy | Inspector | entra em |
|---|---|---|---|---|
| **Cena** (raiz) | objecto (novo) | 1.ª linha, sempre | Animação da cena · Mundo físico · Mixer | — |
| Vazio / Grupo | objecto | linha | Transform | — |
| Imagem | objecto | linha ▸ *Camadas* (conteúdo) | Rendering · Ordering · (pixels) | Paint · Edit |
| Vetor (forma) | objecto | linha | Geometry · Rendering | Edit |
| Flip | objecto | linha ▸ *Quadros* (conteúdo) | Rendering | Draw · Edit |
| Esqueleto | objecto | linha ▸ *Ossos* (conteúdo) | — | Edit · Pose |
| Câmara · Fonte de som · Corpo físico · HUD | **atalhos** do Add (Vazio + componente já posto), não tipos | linha com o ícone do componente | o cartão respectivo | só Object |
| Prefab / Instância | objecto com 🔗 | linha ▸ (peças) | Instance (overrides) | abrir o molde |
| Motion (gerador) | ver 9.3 | linha ▸ *Nós* · as formas que publica como filhos | parâmetros | editor de nós |
| Corpo físico · Colisor · Juntas · Raio | componente | ícone ⬢ | Physics | — |
| Vida · Jogador · Fábrica · Tempo de vida · Arma · Máquina de estados · Timers · Tweens · Acções de sinal · Script | componente | ícone ⚙ | Logic | — |
| Navegação (agente · região · custo) | componente | ícone | Logic/Physics | — |
| Câmara de jogo · Seguir · Limites · Tremor · Parallax | componente | ícone 🎥 | Camera | — |
| Fonte de som · Ouvinte | componente | ícone 🔊 | Audio | — |
| HUD (tela · rótulo · botão · contador) | componente | ícone | HUD | — |
| **Animação** (clipes da Timeline) | **componente** (ver 9.1) | ícone 🎞 ▸ *Clipes* (conteúdo) | Animation | a Timeline, sobre ESTE objecto |
| Intervalos da grelha (sprite) | **conteúdo da Imagem** (o `SpriteFrames` do Godot) | sob a Imagem | Animation | Sprite Inspector |
| Teclas (input map) · unidades · cores · snaps | projecto | — | — | Projecto ▸ Configurações |
| Quadros (MiroClone) | documento | — | — | aba de cima |
| Sinais em execução | tempo de execução (não se guarda) | — | — | monitor durante o Play |

## §6 — A Timeline (a pergunta do dono)

**Recomendação: a Animação é um COMPONENTE, e os clipes dela são CONTEÚDO visível na Hierarchy.**

```
▾ 🎬 Cena                         🎞
    ▸ 🎞 Animação da cena (2 clipes)
  ▾ 🧍 Herói                ⬢ ⚙ 🔊 🎞
      ▸ 🎞 Clipes (3)  — andar · pular · morrer
      ▸ 🖼 Camadas (4)
    ▸ 🗡 Espada
  ▸ 📷 Câmara               🎥
```

- O painel **Timeline mostra a animação do objecto seleccionado**, ou a do ancestral mais próximo
  que tenha uma. O cabeçalho diz de quem é: *«Animação de: Herói ▾»*. É o que o Godot faz ao
  seleccionar um `AnimationPlayer`, mas sem o nó a mais.
- **Zero perda:** o `TimelineDoc` global de hoje passa a ser a *Animação da cena*. Os clipes de um
  objecto podem animar os filhos dele, porque a pista já aponta por `wire_id`. O precedente de
  «documento por dono» já existe: `ProjectFile.painted` guarda um documento por `StableId`.
- **As duas animações deixam de competir.** Os intervalos da grelha são CONTEÚDO da Imagem (como o
  `SpriteFrames`), e um clipe da Timeline pode trocar o intervalo activo. É o par
  `AnimatedSprite2D` + `AnimationPlayer` do Godot, com um lugar para cada.
- **A alternativa (B, à Godot):** a Animação vira um OBJECTO-FILHO («Animador») na árvore. Fica mais
  literal, mas custa uma linha a mais por personagem e devolve a regra de pai escondida («o Animador
  anima o pai»). Recomendo A.

## §7 — Como ganhar do Godot (o que muda na tela)

1. **Ícones de componente na linha da Hierarchy.** Num relance vê-se o que cada objecto faz; o clique
   no ícone leva ao cartão. O Godot só mostra script e sinais; o Unity não mostra nada.
2. **Sub-linhas de conteúdo.** *«Onde estão as animações do Herói?»* ⇒ na linha 🎞 debaixo do Herói.
   *«Onde estão as camadas desta imagem?»* ⇒ debaixo dela.
3. **Add por PROPÓSITO, com busca por intenção.** O menu Add já está agrupado por tipo (spec/06). O
   *Add Component* passa a achar também por verbo: *«cair»* ⇒ Corpo físico, *«pular»* ⇒ Jogador
   plataforma, *«tocar som»* ⇒ Fonte de som. O Godot procura pelo nome da classe.
4. **Zero regras de pai escondidas.** O colisor é componente do corpo e nunca um filho que se arrasta
   para fora. ⇒ Não existe o triângulo amarelo.
5. **Filtro da Hierarchy por componente:** *«mostrar só o que tem Vida»*, *«só o que tem som»*.
6. **O Inspector fala de propósito, não de herança:** as 12 famílias de hoje ficam (já são isto).

## §8 — As fases (cada uma fecha com teste de costura, foto e smoke)

| fase | o quê | onde mora o código | notas |
|---|---|---|---|
| **F0** | vocabulário: «Prefab» em todos os docs e mensagens; este doc no roteador | docs | o renome da crate fica para uma linha própria |
| **F1** | **objecto Cena** na raiz; Mundo físico e Mixer passam a componentes DELA | `ph2d-panel-hierarchy`, `ph2d-app-components` | `PROJECT_SCHEMA` sobe (conta-se, nunca se escolhe); migração lê o global antigo |
| **F2** | ícones de componente na linha + clique leva ao cartão | `ph2d-panel-hierarchy` | id novo ⇒ os 7 sítios da costura (DIRETIVA §2) |
| **F3** | sub-linhas de conteúdo (Camadas · Quadros · Ossos) | hierarchy + cada família declara o seu conteúdo (ponto de extensão, como o `ObjectTypeDecl`) | a família declara, a shell não escreve a lista |
| **F4** | **Animação por objecto**: componente + Timeline segue a selecção; o global vira a Animação da Cena | `ph2d-timeline` + `ph2d-panel-timeline` | o maior; documento por `StableId` |
| **F5** | Motion como objecto-gerador (se 9.3 = sim) | `ph2d-app-motion` | as formas publicadas aparecem como filhos |
| **F6** | busca por intenção no Add Component + filtro por componente | `component_palette`, hierarchy | sinónimos por chave i18n |

⛔ Toda a lógica nova vai para crates de família ou painel; a shell é só composição
(`the_shell_only_shrinks`).

## §9 — As escolhas do DONO

1. **Animação:** componente do objecto, com os clipes visíveis debaixo dele (**A**, recomendado), ou
   objecto-filho «Animador» como no Godot (**B**)?
2. **Objecto Cena na raiz da Hierarchy**, guardando a animação da cena, a física e o som? —
   *recomendação: sim.*
3. **Motion Nodes:** vira um objecto «Motion» na árvore, com o que ele cria como filhos
   (*recomendado*), ou fica um componente da Cena?
4. **Ícones de componente na linha da Hierarchy** e **sub-linhas de conteúdo** (camadas, ossos,
   quadros, clipes)? — *recomendação: os dois.*

**Respostas do dono (07/10) — as quatro na recomendação:**

| | escolha |
|---|---|
| 1 | **A** — a Animação é componente do objecto, com os clipes visíveis debaixo dele |
| 2 | **sim** — objecto Cena na raiz (animação da cena, mundo físico, mixer) |
| 3 | **Motion vira objecto** na árvore, com o que cria como filhos; pode haver vários |
| 4 | **os dois** — ícones de componente na linha e sub-linhas de conteúdo |

⇒ As fases F0–F6 do §8 ficam todas no plano, na ordem.

## §10 — Recusas (o que NÃO fazer)

- ⛔ **Tornar tudo nó, como o Godot:** devolve a árvore inchada e as regras de pai escondidas.
- ⛔ **Componente como filho na Hierarchy:** é o mesmo defeito por outra porta. Componente é ícone e
  cartão; filho é parte.
- ⛔ **Guardar o tipo num campo:** o tipo continua derivado do marcador (spec/06 §5).
- ⛔ **Trocar o motor ECS:** o problema é de lugar na interface, não de armazenamento (§3).
- ⛔ **Uma segunda Timeline global ao lado da por-objecto:** a global é a da Cena, e só há uma porta.

## §11 — Onda 1 entregue (08/10): F0 + F1 + F2 + F3, e o que a medição corrigiu neste plano

Handoff: [`HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md`](handoffs/HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md).

| plano | o que ficou | porquê (medido) |
|---|---|---|
| §4.6 Cena = objecto com Mundo físico e **Mixer** | raiz **sintética** (`RowRole::Scene`), com **dois** componentes: Mundo físico (abre o painel `physics`) e Animação da cena (abre a `timeline`) | ⛔ **Não há Mixer**: o `SceneAudio` é o livro das vozes que tocam, não dado do artista (não é persistido). ⛔ **Entidade real recusada**: o `PhysicsSettings` fica FORA do undo de propósito (`project.rs:52`) — como componente de uma entidade, o Ctrl+Z do canvas rebobinaria a gravidade |
| §8 F1 «`PROJECT_SCHEMA` sobe» | **não sobe** | nada novo se guarda: a Cena é linha, o mundo físico continua em `ProjectFile.physics` |
| §8 F3 «ponto de extensão como o `ObjectTypeDecl`» | `ModeFamily::content` / `open_content` (default vazio) | o `ObjectTypeDecl` **não existe no código** (só na spec/06); as famílias com conteúdo já estão compostas como `ModeFamily` — a shell não ganhou lista nova |
| §5 Flip ▸ *Quadros* | Flip ▸ **Camadas** | os quadros vivem na linha do tempo; cem desenhos em linhas são a árvore inchada do §2 |
| §5 Esqueleto ▸ *Ossos* | **já estava**: os ossos são entidades filhas, linhas desde antes | um cabeçalho «Ossos» acrescentaria uma linha e um nível sem informação nova |
| §7.1 ícone por componente | ícone por **FAMÍLIA** presente (`hier_rows::component_icon`: Animation · Anchors · Physics · Logic · Audio · Camera · Scripting) | 107 tipos não cabem numa linha; as famílias que dizem o que o objecto É (Transform, Rendering, Image…) não distinguem ninguém |

**O custo que a Cena cobra:** todo objecto desce um nível (12 px de nome). É o do Godot e do Outliner
do Blender; os censos de rótulo continuam verdes porque a fixtura armada perdeu a «Scene Root» falsa
que já simulava esta raiz.

## §12 — Onda 2 entregue (08/10): F4, a Animação por objecto

Handoff: [`HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md`](handoffs/HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md).
Escolhas do dono (08/10): **a chave vai para a animação do cabeçalho**; **a cópia leva clips próprios** — no
Duplicate e, depois do smoke, também na cópia de um **Prefab** (Make, Instantiate, colocar); as cópias de
JOGO da Fábrica não (`ph2d_ecs::SubtreeCopies`).

| plano | o que ficou | porquê (medido) |
|---|---|---|
| §6 «documento por `StableId`» (precedente `painted`) | **UM `TimelineDoc`**, e cada clip tem DONO (`NamedClip.owner`: `None` = Cena) | 216 ficheiros contam com um documento só; o undo da timeline é um; a Cena arruma clips de qualquer objecto no Arrange dela. N documentos seriam a «segunda Timeline» do §10 |
| §8 F4 «`PROJECT_SCHEMA` sobe» | **não sobe** (187); sobe o `DOC_VERSION` do blob, 18 → 19 | o blob tem versão própria (`project_schema_history.rs:51`); e a v18 **migra** (`doc_v18.rs`), a 1.ª desta escada que não recusa, provada contra uma fixtura gravada antes da v19 |
| §5 «Animação» componente (cartão no Inspector) | componente **derivado** do documento (tem ⇔ é dono de ≥ 1 clip); o cartão É a Timeline; Add Component ▸ *Animation*; tirar = ▾ *Remove animation from X* | um marcador no ECS ao lado dos clips teria duas pilhas de undo a discordar; o catálogo exige tipo registado (gate de órfãos) |
| §6 «Clipes como conteúdo» via `ModeFamily::content` | a Timeline não é família de modo ⇒ `ContentOpens::{Mode, Panel}` e a Timeline declara o grupo (`ph2d-panel-timeline/src/hier.rs`); a Cena recebe conteúdo pela entrada `HIER_SCENE_ROW` | `family()` só acha uma família pelo MODO (`mode_drive.rs:155`) |
| `MAX_CLIPS` (16) do documento | **por animação** (o recurso é o seletor, que mostra uma); o documento pára no `u16` do strip | e o seletor de acções do osso inteligente passou a listar a animação onde o osso vive — o pool fixo de 16 já não cabia o documento |
| — | o dope sheet lista o que a animação em vista anima (+ bindings ainda sem chave) | a foto do smoke mostrou o Inimigo (Cena) nas linhas do Herói |

## §13 — Onda 3 (F5 + F6): a medição e as escolhas do dono (08/10)

**O que a medição de abertura derrubou no §5/§8:**

| plano | o que é | porquê (medido) |
|---|---|---|
| §5 «as formas que publica como filhos» | o resultado do Motion **não são objectos**: é um fluxo de instâncias desenhado direto (`present.rs`, ADR-0035), milhares por quadro | uma linha por cópia é a árvore inchada do §2; o que vira filho é o **molde** (a forma que ele copia) |
| «Motion é algo da cena» | hoje o resultado **só se desenha com a ferramenta Motion activa** (`present.rs:229`, `motion_active`) | é o «solto» do §1 no grau máximo: sai da tela ao trocar de ferramenta |
| N documentos (um por objecto Motion) | **UM `MotionDoc`**, cada nó com dono (molde da onda 2) | 258 ficheiros contam com o tipo, `ProjectFile.motion` é um campo, o undo é um (`MotionHistory`) |

**Escolhas do dono (08/10):**

| | escolha |
|---|---|
| F5.1 | o resultado do objecto Motion aparece **sempre** (como qualquer objecto, Play incluído); some só pelo olho |
| F5.2 | a forma copiada (a «Bola») **vira filha do objecto Motion e não se desenha sozinha** — só as cópias (como no Cavalry) |
| F6.1 | a busca por intenção do *Add Component* entende **só inglês** (a língua da interface): «fall» ⇒ Corpo físico |
| F6.2 | o filtro da Hierarquia por componente é um **botão ▾ ao lado da busca**, que lista só os componentes presentes na cena, com contagem |
