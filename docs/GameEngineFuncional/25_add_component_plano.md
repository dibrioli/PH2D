# 25 — Add Component: a versão B da Crítica + o recibo da A, no padrão do Add Object (plano, 08/11)

Fonte: Crítica do dono, `GameEngineSamples/critica_ph2d.html#addcomponent` (maquetes A, B, C; §7
recomenda «B e leve o recibo da A»). Decisões do dono, 08/11
([memória](../../project-memory/project_add_component_owner_decisions_2026_11_08.md)):

1. **Versão B + recibo** — a mesma janela, **11 assuntos** (nenhum grupo de uma entrada só), **dica para
   as 64 entradas**, e depois de adicionar uma linha que diz o que veio junto e porquê, com **Undo**.
2. **Ao abrir, o 1.º item do 1.º grupo vem destacado** (a faixa nunca nasce vazia).
3. **«brings …» vira «+N»**, a lista inteira vai na dica. **Show all fica** como hoje.

Base: o Add Object (plano [24](24_add_object_plano.md), handoff
[continuação 07/11](handoffs/HANDOFF_CONTINUACAO_2026-11-07_ADD_OBJECT.md)) — **não se reconstrói**; o Add
Component LIGA o que lá já existe.

## 1. O que o código diz hoje (conferido 08/11)

| Afirmação da Crítica | Código | Veredito |
|---|---|---|
| 64 entradas = 63 componentes + Animation | 63 `Attach::Authored` em `ph2d-component-desc/src/catalog/*.rs` + `ANIMATION_ITEM` (`component_palette.rs:263-301`) | ✓ |
| 14 grupos, 7 de uma só | `build` itera `ComponentCategory::ALL` (16 variantes, `ph2d-component-desc/src/lib.rs:429-498`), grupo vazio sai | ✓ (16 no tipo, 14 com gente) |
| Sem dica | `component_palette.rs:246` `tips: Vec::new()`, `:247` `select: None` | ✓ |
| «brings» cortado | `make_item` `:157-179` cola `brings_along` (fecho transitivo, `:131-145`) no rótulo | ✓ |
| Fecha ao adicionar? | `chrome/command_palette.rs:86-92` pick ⇒ fecha; só Show all mantém aberta (`:70-74`) | fecha |
| Um Ctrl+Z desfaz tudo | `component_attach.rs:189-240` anexa cascata + alvo no MESMO quadro; o undo é o diff do quadro (`post_frame_undo`) | provável — **a provar por teste** (§4) |
| Sem aviso depois | `attach_picked` já recebe `toasts`, mas `Toast` (`editor-core/src/toast.rs:45-90`) não tem botão | ✓ — falta a superfície |
| Busca boa | `ph2d-i18n/src/component_keywords.rs`, gate `every_offered_component_has_intent_words` | ✓, não muda |

⚠️ **O look compacto liga-se a `select.is_some()`** (`widget/command_palette/look.rs:105-111`). Como o
dono quer o MESMO gesto, o Add Component passa a ter `select: Some` e herda o look sem flag novo.
Ctrl+K e a biblioteca do Motion continuam com `select: None`, sem mudança nenhuma.

⚠️ **Tectos de LOC** (medidos 08/11): `ph2d-app-components` tem 32 linhas de folga, `ph2d-app-game` 32, a
shell 1 793. Os widgets têm um tecto de 500 por ficheiro. ⇒ **Dados (grupos, ordem, dicas, desenhos,
porquês, ensino) vão para a folha `ph2d-game-dicas`** (novo `src/componente.rs` + desenhos em ficheiro
irmão). O `build` só lê essa tabela, e o saldo da `ph2d-app-components` fecha em ≤ 0: o `for c in C::ALL`
e o `cat_title`/`cat_token` saem, entra a tabela. Se não fechar, `brings_along` e `make_item` mudam-se
também para a folha. A cura é mudar de casa, nunca subir o número.

## 2. O desenho (um bloco, §0.10)

### 2.1 Os 11 assuntos — só APRESENTAÇÃO

O catálogo (`ComponentCategory`) **não muda**. O que muda é a tabela `componente::GRUPO`, que dá para cada
nome canónico o grupo e a posição. Grupos (ordem da maquete B), com o conteúdo do catálogo inteiro:

| # | Grupo | Entradas (ordem) | n |
|---|---|---|---|
| 1 | Move and collide | Physics Body · Top-Down Player · Projectile Motion · Path Follow · Motion Input · Ray Sensor | 6 |
| 2 | Find a path | Nav Agent · Nav Region · Nav Cost Area · Nav Link | 4 |
| 3 | Game rules | Signal Actions · Trigger · Counter · Counter Watch · Timers · Lifetime · Destroy Outside · Factory · Weapon · Damage · Health | 11 |
| 4 | On screen | HUD Canvas · Label · Button · Health Bar · Anchors · Anchor Mount · Clip Children | 7 |
| 5 | Camera | Camera Follow · Camera Limits · Camera Shake · Shake Emitter | 4 |
| 6 | Background | Parallax · Parallax Limits · Parallax Drift · Parallax Repeat | 4 |
| 7 | Change over time | Animation · Animations · Animator · State Machine · Tween · Sequence Player · Script | 7 |
| 8 | Look | Blend Mode · Mask · Mask Interaction · 9-Slice · Texture Filter · Texture Repeat · UV Transform · Particles · Light Occluder · Unlit | 10 |
| 9 | Sound | Audio Source 2D · Audio Listener 2D | 2 |
| 10 | Bones | Bone · Bone Angle Limit · Smart Bone | 3 |
| 11 | Organize | Transform · Tags · Constraints · Visibility Layer · On-Screen Enabler | 5 |
| | | | **63** |

⭐ **Dono 08/11 — a Game Camera sai do Add Component**: nasce só por Add Object ▸ Camera
(`Attach::Intrinsic`, como a `Luz`; o cartão fica). `ObjectKind::Camera` (marcador `GameCamera`, o
último do `ALL`) e Follow/Limits/Shake com `O::CAMERA` — os sistemas (`camera-frame`, `camera_2d`,
`shake_bridge`) só as lêem na câmera; no *Show all* ficam esmaecidas com a razão genérica, e a dica
diz «Put it on a Camera.». Sem ensino na busca: «camera» já acha Camera Follow. Gate
`a_game_camera_so_nasce_pelo_objeto_e_as_pecas_de_camera_so_numa_camera`.

Cor por grupo: um `NodeCat*` cada, como o `AddGroup`. Título: chave `component_add.group.*`.

### 2.2 O gesto e o destaque

`select: Some(PaletteSelect { initial, teach })` com `initial` = **o 1.º item visível** do modelo construído
(depende do objeto: na moeda é Top-Down Player). Ao reabrir por Show all, o destaque **fica** se o item
continua lá. Se não, volta ao 1.º. O resto (1 clique destaca, 2 cliques ou Enter adicionam e fecham, ↑↓,
faixa à altura da dica mais alta) é o do Add Object, sem código novo no widget.

### 2.3 «+N» e a dica

- Rótulo = nome + ` +N` (N = `brings_along().len()`, transitivo), nada mais. O inaplicável continua com o
  sufixo da razão (Show all).
- Dica (`PaletteTip`) = título + corpo em três partes, como na maquete B 2:
  **frase** (Plain English, uma ou duas linhas, chave `component_add.<nome>.tip`) · `Words: …` (as palavras
  de busca que já existem, para ensinar a busca) · `Also adds: X, Y.` (só quando N > 0, gerada).
- **Desenho** para cada uma das 64 (`&[Mark]`, moldura 2:1, cores do tema), em `ph2d-game-dicas`.

### 2.4 O recibo com Undo (o da A)

- Só quando a cascata não está vazia: `Physics Body added. Also added: Collision Shape, because a body needs
  a shape.` + botão **Undo** (desfaz o passo inteiro: alvo e trazidos).
- O «porquê» é por ARESTA `requires` (tabela `componente::PORQUE`, chave `component_add.why.<a>.<b>`). Um
  gate garante que toda aresta tem o seu porquê.
- Superfície: `Toast` ganha uma ação opcional (`with_action(label, NodeId)`) em `editor-core/src/toast.rs`. O
  clique na ação regista um alvo, e a shell trata-o como o Ctrl+Z. Fica no ar o tempo de leitura (ttl que já
  existe) e some no próximo gesto de undo/redo.

### 2.5 O ensino da busca (o mesmo mecanismo do Add Object)

Só onde a busca hoje não acha, e só com peças que existem:
- `platform player, platformer player` → «To make it walk and jump, add Physics Body, then press Make
  Platform Player on its card.» · botão Start with the Physics Body.
- `collision shape, collider, hitbox` → «A shape comes with Physics Body. Add the body, or use Add Object ▸
  Collision Shape.»

O conjunto final é medido: correr as palavras contra o modelo, e só fica o caso que dá zero resultados.

### 2.6 Fotos sem clique (a tela virtual não recebe cliques)

`PH2D_OBJECT_ADD_PICK=object_add.game.sprite PH2D_COMPONENT_ADD_SMOKE=1`: o Add Object cria a Sprite, o
smoke põe-lhe a moeda da Crítica (Physics Body, Audio Source 2D, Signal Actions, Animations, Animator) e abre a
janela (`shells/desktop/src/render_loop/fase_component_add_smoke.rs`). `PH2D_OBJECT_ADD_HOVER=<nome canónico>` e
`PH2D_OBJECT_ADD_QUERY=<texto>` são os da paleta, partilhados; `PH2D_COMPONENT_ADD_PICK=<nome canónico>` adiciona
e fecha (a foto do recibo).

## 3. Fora deste item (e porquê)

- Sugestões por tipo de objeto (A/C) e os verbos da C: o dono escolheu B.
- Aviso ao remover o que outro precisa (Unity): a Crítica o põe no lote 3, que olha componente a componente.
- Cortar componentes: lotes 3 a 10.
- **Ao dono, na foto lado a lado** (abertos do Add Object): o peso das letras (`CrispHeavy` engrossa ≤ 12 px no
  app inteiro) e o destaque cheio × só contorno. Ambos valem para as duas janelas.

## 4. Provas (por nome; mutação em cada lei)

| Teste (crate) | Lei | Mutação que tem de sangrar |
|---|---|---|
| `todo_componente_oferecido_tem_grupo_posicao_e_dica_com_desenho` (game-dicas, dev-dep component-desc) | as 64 cobertas, nenhuma linha órfã | apagar uma linha · nome inexistente |
| `nenhum_assunto_tem_uma_entrada_so` (game-dicas) | os 11 ≥ 2 no catálogo inteiro | mudar Sound para 1 |
| `toda_dependencia_tem_o_seu_porque` (game-dicas) | `PORQUE` cobre cada aresta `requires` | apagar um porquê |
| `cada_ensino_so_cita_o_que_existe_e_so_aparece_sem_resultado` (game-dicas) | o ensino não mente | palavra que já acha |
| `toda_dica_e_traduzida_e_em_ingles` (i18n) | chave resolve | chave crua |
| `os_grupos_seguem_os_onze_assuntos_na_ordem` (app-components) | ordem e títulos | trocar dois grupos |
| `a_cascata_e_um_mais_n_no_rotulo_e_a_lista_na_dica` (app-components; substitui `the_cascade_is_shown_in_the_label_before_it_is_applied`) | N transitivo, lista na dica | N não transitivo |
| `ao_abrir_o_primeiro_da_lista_vem_destacado` (app-components) | destaque inicial | `initial` = último |
| `show_all_mantem_o_destaque_se_o_item_continua` (app-components) | não salta | sempre o 1.º |
| `um_clique_destaca_e_dois_adicionam` (desktop, clique real até ao dreno) | o gesto | um clique adiciona |
| `o_recibo_diz_o_que_veio_e_o_undo_desfaz_tudo_ate_ao_bit` (desktop) | recibo + um passo de undo | cascata em dois quadros |
| `um_aviso_com_acao_regista_o_alvo` (editor-core toast) | o botão existe e clica | não registar |

Continuam verdes por nome: os de `component_palette_tests.rs`, `component_palette_intent_tests.rs`,
`todo_componente_que_a_paleta_oferece_tem_cartao`, `selecao_tests`, `look_tests`, `fase_object_add_tests`,
`architecture_widget_loc_cap`, `the_shell_only_shrinks`.
**Mesma rodada de mutação (§0.10):** as do 1.º commit do Add Object, ainda devidas (destaque inicial,
duplo clique, opt-in, altura reservada, filtro de palavras, corte de 3 do Recent). Correm todas juntas.

## 5. Ordem

1. Dados na folha: grupos, posições, porquês, ensino, as 64 frases, e os 64 desenhos (os desenhos em paralelo
   por grupo, num agente cada).
2. `build` lê a tabela: `select`, `tips`, «+N». O saldo de LOC da app-components ≤ 0.
3. `Toast` com ação + o recibo no `attach_picked` + o Undo na shell.
4. Variáveis de foto. Fotos: 1 · ao abrir na moeda; 2 · dica do Nav Agent; 3 · busca `damage`; 4 · o recibo
   do Physics Body. Cada uma ao lado da maquete B 1–3 / A 4, com o acabamento comparado.
5. Testes por nome + a rodada de mutação. Depois, o smoke do dono em passos numerados.
