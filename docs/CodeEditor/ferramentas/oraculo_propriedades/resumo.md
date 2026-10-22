# Oráculo «propriedades por nome» — resumo (Godot 4.7.2, MIT, 22/10)

Godot CORRIDO sem janela (`probe.gd`, `--headless --script`) e o editor headless com um plugin
(`rotulos.gd`) para o rótulo do Inspector. Nunca lido. Reproduz: `PH2D_GPU=1 bash scripts/ph2d-run.sh bash
docs/CodeEditor/ferramentas/oraculo_propriedades/corre.sh` → `saida.jsonl` (2 corridas, diff vazio; ids de
instância normalizados para `#ID`). Cena: `Porta` (Sprite2D + icon.svg), `Camera` (Camera2D), `Heroi`
(Node2D + `heroi.gd`).

| # | Pergunta | Lei observada |
|---|---|---|
| a | `get_property_list()` | Sprite2D 72 entradas / 40 com EDITOR; Camera2D 90/54; Heroi 64/33. Muitas sem EDITOR (`name`, `owner`, `scene_file_path`…). Exportadas = `STORAGE+EDITOR+SCRIPT_VARIABLE`; `var` não exportada = só `SCRIPT_VARIABLE`. Hints: `velocidade` RANGE `"0.0,10.0"`; `alvo` Object/`Node2D` NODE_TYPE; `texture` Object/`Texture2D` RESOURCE_TYPE; `zoom` Vector2 LINK; `position` RANGE com `suffix:px` |
| a | Categoria / grupo | Entradas tipo `Nil` com usage CATEGORY (nome da classe: `Node`, `CanvasItem`, `Node2D`, `Sprite2D`; o script aparece como CATEGORY `heroi.gd`, hint_string `res://heroi.gd`), GROUP (`Texture`, prefixo `texture_` no hint_string) e SUBGROUP (`Thread Group`). Vêm na ordem, antes das suas propriedades |
| b | Nome × rótulo | get/set usam o nome cru (`texture`, `zoom`, `vida_max`). O Inspector mostra `capitalize()`: «Texture», «Zoom», «Vida Max», «Z Index», «Hframes». ⚠️ Dentro de um grupo com prefixo, o prefixo SAI do rótulo: `position_smoothing_speed` → «Speed» (grupo «Position Smoothing») |
| c | `texture` | `get` → `CompressedTexture2D`, `resource_path` `res://icon.svg`. `set(load(…))` funciona. `set("res://icon.svg")` (String) → vira **null** em SILÊNCIO (sem erro, sem carregar). `set(null)` → null |
| d | Propriedade inexistente | `set("nao_existe", 5)` é SILENCIOSO (nem erro, nem meta criada); `get` devolve `null` |
| e | Tipo errado | Script `int`: `"abc"` → **0**, `3.7` → **3** (trunca), `"42"` → 42, tudo calado. `@export_range(0,10)`: `50` e `-3` passam SEM clamp (o range é só do Inspector). `position=5` e `"abc"` → **(0,0)** calado. `z_index=3.9` → 3. `modulate="red"` e `cor="#ff0000"` → Color(1,0,0,1) (String→Color converte por nome/hex). `alvo` (Node2D) ← `Node` → ignorado calado (fica o anterior) |
| e | Validação do setter nativo | Só o setter fala: `z_index=99999` → `ERROR: Tried to set Z index to an invalid value… between -4096 and 4096` (mantém); `hframes=0` → `ERROR: Amount of hframes cannot be smaller than 1.` (mantém) |
| f | Sub-propriedades | `get("position:x")` → null; `get_indexed("position:x")` → 1.0; `set_indexed("position:x",10)` funciona; `set("position:x")` e `set("position.x")` → no-op calado; `get_indexed("position.x")` → null; `get_indexed("texture:resource_path")` atravessa o recurso; `position:z` → null; `set_indexed("modulate:a",0.5)` funciona |
| g | `zoom` | `get` → Vector2(1,1). `set("zoom", 2)` e `2.5` → escalar NÃO vira Vector2: chega (0,0) e o setter recusa: `ERROR: Zoom level must be different from 0 (can be negative).` — fica (1,1) |
| h | Reverter | Em nó solto/runtime `property_can_revert` = **false** e `property_get_revert` = null para tudo (vida_max, zoom, position, texture). O padrão vem de `ClassDB.class_get_property_default_value` (Camera2D.zoom (1,1), Sprite2D.position (0,0)) e `Script.get_property_default_value` (vida_max 100, velocidade 2.0, cor (0,0,0,1), alvo null) |
| i | Outro nó | `get_node("../Porta").set("modulate", Color.RED)` funciona. Caminho errado: `get_node` → `ERROR: Node not found: "../NaoExiste" (relative to "Heroi").` e null; `get_node_or_null` → null calado |
| j | `@export alvo: Node2D` | Não definido → null; depois de `set(camera)` → o Camera2D; depois de `free()` → `<Freed Object>`, `is_instance_valid` false |
| k | Só-leitura | Nenhuma propriedade com `PROPERTY_USAGE_READ_ONLY` em Sprite2D/Camera2D/Heroi/Label/CharacterBody2D/Timer/AnimationPlayer/Area2D. `var` só com `get:` → `set` ignorado calado (fica 5). `Timer.time_left` (sem setter) → ignorado calado. `name` e `scene_file_path` são escrevíveis. `global_position` escreve (sem pai transformado = position) |

## O que o PH2D deve tirar disto (leitura, não decisão)

- O Godot é **permissivo e calado** por `set()`: nome errado, tipo errado, propriedade só-leitura —
  nada avisa; só os setters nativos com validação imprimem `ERROR:`. O lado bom (avisar) é nosso.
- Rótulo = `capitalize()` do nome, menos o prefixo do grupo. O nome do código é sempre o cru.
- O range do `@export_range` não é lei do valor, é do controlo.

## Não corrido

- `property_can_revert` DENTRO do editor (com a cena aberta) — aqui só em runtime, onde responde false.
- O estilo de nome «Raw»/«Localized» do Inspector (lido só o padrão, `estilo: 1` = Capitalized).
