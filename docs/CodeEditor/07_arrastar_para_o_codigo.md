# CodeEditor — item 8: arrastar objetos para o código

> Origem: decisão do dono nº 4 ([00_decisoes_do_dono.md](00_decisoes_do_dono.md), 15/10), escolhida a 21/10 a seguir
> ao smoke OK do item 7: *«script aponta outro objeto pelo nome (arrastar da Hierarchy, como no Godot) — e tudo o que
> torne mais fácil, rápido e intuitivo»*. O porquê de poder apontar já está escrito no
> [ADR-0188 §5](../architecture/decisions/0188-o-editor-de-scripts-mora-no-ph2d-e-o-script-mora-no-projeto.md)
> (revoga «scripts só falam por sinais»); este doc é o DESENHO que ele adiava.

## 1. O que existe (lido no código, 21/10)

- **Arrasto da Hierarchy:** gesto interno ao painel (`HierarchyDragState { dragged: NodeId, … }`,
  `ph2d-editor-core/src/interaction/drag.rs`; início em `dispatch/pointer_down.rs`, fim em `pointer_up.rs` →
  `find_hierarchy_drop` só olha linhas da Hierarchy → `HierReparent`). Não sai do painel.
- **Arrasto de assets:** `DragPayload { Prefab { stable_id }, Image { asset } }` + `InFlightDrag` (veredito, fantasma
  `screens/hero/asset_drag_ghost.rs`), alvos em `asset_drop::resolve` (`ph2d-app-components`), fio na shell
  `asset_drag_wire.rs` (`drop_target_at`). É a porta para um ALVO novo (a aba de código).
- **Aba de código:** `DocSurface` (`ph2d-app-code/src/surface.rs`: `pointer`, `pointer_move`); `view.rs::hit` dá o
  byte sob o rato; `edit.rs::paste` insere num passo de undo.
- **API:** `Entity = number`; `ph2d.find_by_name(name) -> Entity?` (o primeiro na ordem da lista; `lane.rs::find`);
  `ph2d.get/set(entity, PoseField)`; `ph2d.spawn(master: string?)`; `ph2d.attach_script(entity, script: string)`.
  Nenhuma função de assets. Propriedades de script: `ph2d.property(nome, default)` com `ScriptValue` = número, sim/não,
  texto, posição, cor (`ph2d-script/src/props.rs`) — **nenhuma de objeto**.
- **Nomes:** `Name` não é único no ECS; o editor impõe unicidade ao renomear (`ph2d_unique_name`). O commit do nome
  (`RenameCommit`, `hierarchy_rename.rs`) não guarda o nome antigo. Os scripts do projeto todos: `ScriptLibrary`
  (`ph2d-script-doc`, `iter`/`set_text`).
- **Problemas nossos:** `lint.rs::field_lints` (`self.<campo>` desconhecido, com «quis dizer»); nenhum facto de cena
  chega ao editor além de `link.rs::scene_facts` (contagem de `LuauScript`).

## 2. Oráculo — o EDITOR do Godot 4.7.2 corrido (MIT; [`ferramentas/oraculo_arrastar/`](ferramentas/oraculo_arrastar/saida.txt))

Numa Xvfb; cada caso é um arrasto REAL do rato (xdotool) da árvore da cena, do FileSystem ou do Inspector para o
script aberto, com Ctrl/Shift/Alt na soltura; uma sonda (`sonda.gd`, EditorPlugin) só diz onde estão as coisas e lê o
texto. **Duas corridas, mesmos resultados** (21/10, 22:07Z e 22:13Z).

| Arrasto | Sem modificador (Shift = igual) | Ctrl | Alt |
|---|---|---|---|
| um nó (script do nó raiz) | `$Porta` no cursor da soltura | `@onready var porta: Sprite2D = $Porta` + quebra de linha, no cursor | `@export var porta: Sprite2D` + quebra |
| nome com espaço | `$"Caixa Grande"` | `@onready var caixa_grande: Node2D = $"Caixa Grande"` | idem `@export` |
| nó com nome único | `%Placar` | `@onready var placar: Label = %Placar` | idem |
| neto | `$Jogador/Arma` | `@onready var arma: Node2D = $Jogador/Arma` | idem |
| dois nós (clique + Ctrl+clique) | `$Porta, %Placar` | uma linha `@onready` por nó | — |
| script de um nó filho (`jogador.gd`) | caminho RELATIVO: `$Arma`, `$"../Porta"`, `$".."` | idem, relativo | — |
| script de nenhum nó (`solto.gd`) | como o da raiz (`$Porta`), sem aviso | idem | — |
| ficheiro (`icon.svg`, `.tscn`, `.tres`, `.gd`) | `"res://icon.svg"` | `const ICON = preload("uid://…")` (uid quando o ficheiro tem; senão o caminho) | (= sem) |
| propriedade do Inspector (`texture`, `text`) | `"texture"` — o NOME entre aspas | igual | igual |

- **A meio do arrasto** ([foto](ferramentas/oraculo_arrastar/foto_01_a_meio_do_arrasto.png)): um cursor fino onde vai
  cair, o ícone + nome do nó ao lado do rato, e um balão no canto: *«Drop node path. Hold Ctrl: Add @onready var
  pointing to the node path. Hold Alt: Add @export var pointing to the node.»*
- **Undo:** um Ctrl+Z desfaz a soltura inteira (volta ao texto byte a byte).
- **O cursor** fica no fim do que foi escrito (com Ctrl: no início da linha seguinte).
- **Escrever `$`** abre a lista com todos os caminhos (`.`, `"Caixa Grande"`, `%Placar`, `Jogador`, `Jogador/Arma`,
  `Placar`, `Porta`); `get_node("` idem entre aspas; `%` sozinho: nada.
- **Renomear o nó depois** (Porta→Portao, Placar→Pontos): **o texto NÃO muda** e o editor **não avisa**
  ([foto](ferramentas/oraculo_arrastar/foto_02_depois_de_renomear.png)) — o script parte em silêncio ao correr.
- **Ctrl+clique sobre `$Porta`:** nada (nem selecciona o nó, nem abre).

## 3. A lei (nossa) — o Godot nos gestos, a decisão do dono onde ele parte

Godot onde ele acerta (gesto, texto no cursor, Ctrl = declarar, vários = lista, um undo, cursor fino + nome + balão);
**divergências declaradas** onde o dono decidiu mais (renomear e verificar) ou onde o Luau difere:

1. **A referência escrita é `ph2d.object("Porta")`** — API nova: `(name: string) -> Entity`, o objecto com esse nome;
   **erro** (que pára o objecto, como os outros erros da API) se não existir. É o `$` do Godot: tipo não-opcional,
   para que `ph2d.get(ph2d.object("Porta"), "x")` passe no `--!strict` (com o `find_by_name`, `Entity?`, não passa). O
   `find_by_name` fica como está (a pergunta «existe?», `nil` incluído).
   - *Pelo nome e não por caminho:* o nosso `find` é global por nome e o editor impõe nomes únicos; o caminho relativo
     do Godot (`$"../Porta"`) parte ao reparentar, o nome não.
2. **Ctrl** = `local porta = ph2d.object("Porta")` + quebra de linha, no cursor da soltura, **com o recuo da linha**
   (o Godot não recua — num `local` do Luau o recuo é o bloco). Sem anotação de tipo: o Luau infere `Entity` (o Godot
   anota porque o GDScript inferiria `Variant`). Nome da variável = o do Godot (`snake_case` do nome), feito
   identificador Luau (acentos tirados, dígito inicial com `_`, palavra reservada com `_` no fim).
3. **Vários objectos:** `ph2d.object("Porta"), ph2d.object("Placar")`; com Ctrl, um `local` por linha.
4. **Alt** = o `@export` do Godot: `local porta = ph2d.property("porta", ph2d.object("Porta"))` declara uma
   **propriedade de objecto** — um campo no Inspector do objecto que tem o script, que guarda o **`StableId`** do objecto
   escolhido (aponta-se arrastando da Hierarchy para o campo, ou pela lista). `ScriptValue::Object(u64)` no FIM do
   enum (o fio é posicional). É a referência mais durável de todas: sobrevive a renomear sem reescrever nada.
5. **Renomear reescreve** (divergência: o Godot parte em silêncio). Renomear um objecto troca o nome em **todos** os
   scripts do projeto, nas chamadas `ph2d.object("…")`, `ph2d.find_by_name("…")` e no 1.º argumento de
   `ph2d.spawn("…")` (o mestre) — só em literais de texto, nunca num nome montado — **no mesmo passo de undo** que o
   renomear (o marcador do undo global do I10).
6. **O editor verifica** (divergência: o Godot não avisa). `ph2d.object("X")` sem objecto `X` na cena = um problema na
   lista de problemas (Aviso, porque um `spawn` pode criá-lo ao correr), com «quis dizer» pelos nomes da cena; dois
   objectos com o mesmo nome = Aviso («vale o primeiro da lista»).
7. **Escrever `ph2d.object("`** abre a lista com os nomes da cena (o `$` do Godot), pelo mesmo `Completer` do I4.
8. **Ctrl+clique** sobre o nome em `ph2d.object("Porta")` selecciona o objecto na Hierarchy (divergência para mais: o
   Godot não faz nada; a mão do Ctrl do Acabamento passa a cobrir também estes nomes).
9. **Assets:** um prefab (mestre) → `"Inimigo"` (o nome, como o caminho entre aspas do Godot; é o que o
   `ph2d.spawn` recebe); Ctrl → `local INIMIGO = "Inimigo"` (o `const … = preload` do Godot; o Luau não tem
   `const`). Um script da lista de scripts → `"bob.luau"` (o que o `attach_script` recebe).
10. **Propriedade do Inspector:** o NOME entre aspas, como no Godot (`"x"`, `"rotation"` — o que o `ph2d.get/set`
    recebe); uma propriedade de script do próprio objecto → `self.velocidade`.
11. **A meio do arrasto:** o cursor fino na posição de soltura (pela mesma `hit` do clique), o fantasma com o nome, e
    um balão de ajuda com as teclas (texto pela `ph2d-i18n`). Soltar fora do texto (calha, barra) não escreve nada.
12. **Undo:** a soltura é UM passo (`paste`). O cursor fica no fim do que foi escrito.

## 4. Onde

- **Lei pura (o texto da soltura, o nome da variável, a reescrita do renomear, o problema do nome):**
  `ph2d-app-code` (módulo novo `drop`), testada sem janela.
- **`ph2d.object` + `ScriptValue::Object`:** `ph2d-script` (`api_table`, `api_help`, `lane`, `props`) e o
  `ph2d.d.luau` regenerado (`ph2d-bindgen`).
- **O arrasto que sai da Hierarchy e o alvo novo:** `ph2d-editor-core` (o `DragPayload` ganha a variante dos objectos;
  a Hierarchy semeia-a ao sair do próprio painel) + `asset_drop` (alvo «aba de código») + o fio da shell, **sem
  crescer a shell** (catraca `the_shell_only_shrinks`: 33 linhas de folga — a lei mora nas crates).
- **Nomes da cena para o editor:** um facto novo ao lado do `link.rs::scene_facts` (os nomes e quantos de cada).

## 5. Fases (cada uma com portões e mutação; smoke no fim)

Ordem: **8a → 8d → 8b → 8c** (o 8b escreve código da API que o 8d cria).

- **8a** Hierarchy → código: §3.1–3, 5–8, 11–12.
- **8d** Os scripts tocam componentes e imagens (decisão do dono nº 8, 21/10): ler e escrever cada propriedade que o
  Inspector mostra, pelo mesmo nome. Hoje não há tabela de reflexão — cada cartão é à mão (33 `populate_*`), e só a
  pose chega ao Luau; o ponto de partida é o catálogo `ph2d-component-desc` (107 `ComponentDesc`, `FieldDesc` com id e
  `FieldKind`), que hoje não toca valores. Plano + oráculo próprios antes de codar.
- **8b** Assets, lista de scripts e propriedades do Inspector: §3.9–10, com a API do 8d.
- **8c** Alt → propriedade de objecto: §3.4.
- **Smoke = cena 7** (`PH2D_CODE_SMOKE=7`): um jogo pequeno com objectos para arrastar; conduzida e fotografada antes.

## 6. Decisões do dono

- **Nº 8 (21/10):** o que os scripts ainda não sabem tocar (componentes, imagens, propriedades fora da pose) —
  **ensinar os scripts já**, antes de fechar o item 8 (fase 8d).

## 7. Estado (21/10)

8a FEITA na linha (`89433a58a4`..`652730fb20`: oráculo+plano, `ph2d.object`, lei `dropping.rs`, fio, nomes da cena, cena 7, corrida conduzida, portão movido) — [handoff](handoffs/HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-21_ITEM8A_ARRASTAR.md). Próximo: 8d (plano + oráculo Godot primeiro), depois 8b, 8c, fecho da linha.
