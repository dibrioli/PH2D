# 24 — Add Object: a versão B da Crítica, com a dica fixa embaixo (plano, 07/11)

Fonte: Crítica do dono, `GameEngineSamples/critica_ph2d.html#addobject` (maquetes A, B, C). Decisões do
dono, 07/11 ([memória](../../project-memory/project_add_object_owner_decisions_2026_11_07.md)):

1. **Versão B** — mas ao abrir a **Sprite já vem destacada** e a dica dela fica **embaixo, na janela**.
   A dica **não é mais no hover**: **1 clique destaca** e mostra a dica; **2 cliques rápidos** criam e
   **fecham**.
2. «**Paint a new image…**» fica (a Crítica viu o antigo «Image…»; já trocado em main e na linha). Ctrl+N fica.
3. **Physics Body e Ambient Light saem** (Ambient = Light + `Type: Ambient`; o seletor já existe em
   `luz_edits::TIPOS`).

## 1. O que o código diz hoje (conferido 07/11)

| Afirmação da Crítica | Código | Veredito |
|---|---|---|
| 21 entradas, 3 grupos (2D 7 · Game 12 · Empty 2) | `ph2d-editor-core/src/object_add.rs:19-50` `AddGroup::ALL` | ✓ |
| Busca só pelo nome | `object_add.rs:140,145` `keywords: ""`; a busca lê `keywords` em `widget/command_palette/busca.rs:19-65` | ✓ — basta preencher |
| 10 de 21 com dica | `ph2d-i18n/src/object_add.rs` `*.tip` | ✓ |
| Tudo nasce na origem | `ph2d-app-components/src/object_add.rs:49` `Transform::IDENTITY` | ✓ (não muda neste plano: é da C) |
| Sem recentes | nenhum store | ✓ |
| «Image…» | o texto já é «Paint a new image…» (`i18n/object_add.rs:56`, também o botão de `cards_sprite.rs:335`) | ✗ obsoleto |
| A janela fecha ao criar? | `chrome/command_palette.rs:53-85` `Click` ⇒ `set_command_pick` + `close_command_palette` | ✓ fecha com 1 clique |
| O smoke de plataforma usa o menu | `ph2d-app-game/src/smoke.rs` NÃO usa o Add; `ordem_smoke.rs:95-96` e `smoke_tipos.rs:36-54` usam | corrigido |

A dica é hoje flutuante sobre o item sob o rato (`widget/command_palette/tip.rs:26-37`, `hot_id`). O
`PaletteModel` é **partilhado** com Add Component (`component_palette.rs`), a paleta global Ctrl+K
(`global_palette.rs`) e a biblioteca de nós do Motion (`motion_bridge_library.rs`): ⇒ o novo gesto é
**opt-in por modelo**, nunca mudança da paleta genérica.

## 2. O desenho (um bloco, §0.10)

### 2.1 Gesto «destaca, depois escolhe» — `PaletteModel.pick: PickGesture`

- `PickGesture::Click` (omissão: os outros 3 consumidores ficam byte-idênticos) · `PickGesture::Select
  { initial: NodeId }` (só o Add Object, com `initial` = Sprite).
- Em `Select`: `Click(item)` grava o **destaque** (estado do store, não `hot_id`); `DoubleClick(item)` —
  já emitido pelo dispatch (`interaction/event.rs:18`, janela 350 ms; `dispatch/focus.rs:75-80`) — faz o
  pick e fecha. **Enter** escolhe o destacado; **↑ ↓** movem o destaque pelos itens visíveis (o caminho
  do teclado, Shift+A → Enter cria uma Sprite). Item bloqueado: destaca e mostra a razão, não escolhe.
- A busca: quando o destacado sai do filtro, o destaque passa ao `top_match` (`busca.rs:49`); limpar a
  busca não perde o destaque escolhido se ele voltou a estar visível.

### 2.2 A dica embaixo — `tip::paint_docked`

- Uma faixa fixa no fundo da janela: desenho à esquerda, nome em negrito + texto à direita. **Altura
  reservada = a da dica mais alta, medida** (a janela não salta ao trocar de destaque). Sem destaque
  visível (busca vazia) a faixa mostra o ensino do §2.5.
- O desenho é o `Mark` que já existe (`AddTip.picture`); o hover deixa de pintar dica neste modo.

### 2.3 Os grupos e as 19 entradas

| Grupo | Entradas (nesta ordem) |
|---|---|
| (Recent) | até 3, as últimas criadas, sem repetir |
| **Pictures** | Paint a new image… · Sprite · Animated Sprite · Pixel Sprite · Background Layer |
| **Drawing** | Vector Drawing · Text · Flip Drawing · Skeleton · Motion |
| **Game** | Character · Static Body · Collision Shape · Camera · Sound · HUD · Light |
| **Empty** | Empty · Macro |

`AddGroup` passa a `{Pictures, Drawing, Game, Empty}`; cada família re-etiqueta as suas entradas; a
ordem dentro do grupo é declarada (`AddEntry.order`), não a ordem de compilação das famílias. Saem
`object_add.game.body` (Physics Body) e `object_add.light.ambient` com as suas chaves i18n e testes.

### 2.4 Recent — por utilizador, nas preferências da janela

Até **3** tipos, o mais novo primeiro, na linha `add_recent=` de `~/.ph2d/sections.txt` (a mesma lei do
alfinete e das secções: estado do EDITOR, não autoria do jogo — o Godot também o guarda fora da cena).
⛔ **Não** vai no `.ph2dproj`: seria autoria de mentira e subiria o `PROJECT_SCHEMA` por uma preferência.
O Recent é o 1.º grupo; a mesma entrada aparece também no seu grupo (como na maquete B). Recent vazio = o
grupo não existe.

### 2.5 Palavras de busca + o ensino do que não existe

- Uma lista por entrada, `object_add.<…>.keywords`, inglês, vírgula, na lei de
  `ph2d-i18n/src/component_keywords.rs` (o mesmo `tr`, o mesmo gate de chave órfã). Mínimo medido:
  «player» → Character, «wall»/«floor»/«platform» → Static Body, «music» → Sound, «score» → HUD.
- **Ensino:** tabela curta `object_add.teach.<palavra>` (coin, enemy, door, key, spike, checkpoint,
  health…) — UMA frase que diz de que peças se monta, conferida contra o catálogo de componentes que
  existe (nenhuma frase cita componente que não está no Add Component), e um botão «Start with a
  Sprite» que destaca a Sprite. Palavra sem ensino nem resultado: «Nothing is called <x>.»

### 2.6 As dicas que faltam (10)

Paint a new image… · Vector Drawing · Text · Flip Drawing · Skeleton · Motion · Pixel Sprite · Light ·
Empty · Macro. Texto em Plain English (o que é, para que serve, uma frase de como se usa) + desenho em
`Mark`. A dica da Collision Shape passa a dizer que só faz algo dentro de um corpo (já diz).

## 3. Provas (por nome)

- Gesto: `um_clique_destaca_e_nao_cria`, `dois_cliques_criam_e_fecham`, `ao_abrir_a_sprite_vem_destacada`,
  `enter_cria_o_destacado`, `setas_movem_o_destaque`, `os_outros_tres_paletas_continuam_a_escolher_num_clique`.
- Dica: `toda_entrada_tem_dica_com_desenho` (19/19), `a_faixa_da_dica_nao_muda_de_altura`, `o_hover_nao_pinta_dica_no_add`.
- Busca: `player_acha_character`, `wall_acha_static_body`, tabela de palavra → entrada; `coin_ensina_e_nao_cria`;
  `cada_ensino_so_cita_componentes_que_existem`.
- Grupos: `os_quatro_grupos_na_ordem_da_maquete`, `physics_body_e_ambient_light_nao_estao_no_add`.
- Recent: `o_recent_guarda_tres_sem_repetir_e_volta_do_ficheiro`.
- Mutações (antes do fecho): o destaque inicial, o duplo clique, o opt-in (`Click` em todos), a altura
  reservada, o filtro de palavras, o corte de 3 no Recent.

## 4. Smoke e foto

`PH2D_OBJECT_ADD_SMOKE` (já existe, `object_add.rs:198-233`): fotos com `fotografa_cena.sh` de (1) abrir,
(2) um clique em Character, (3) busca «wall», (4) busca «coin», (5) Recent depois de criar 2 objetos —
cada uma **lado a lado com a maquete B** antes do smoke do dono.

## 5. Fora deste plano

O painel de arrastar e o «nasce onde largou» (versão C) · favoritos.
