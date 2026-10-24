# 10 — O `ph2d.object` no topo, fora de `ph2d.property`, é um aviso (24/10/2026)

> Origem: §10 do [handoff D5_8B_8C_1B](handoffs/HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-23_D5_8B_8C_1B.md).
> Ordem do dono 24/10: «mesmo idioma dos avisos que o editor já dá; ver como o Godot trata o equivalente».

## 1. O defeito

No topo do script a cena ainda não existe. O `ph2d.object("X")` do topo só devolve a **tabela-marca**
(`{__kind = "object", name = "X"}`, `crates/ph2d-script/src/lane.rs` `declared_object`) — que só tem sentido
como valor de `ph2d.property`. Em qualquer outro sítio do topo (`local porta = ph2d.object("Porta")`) o script
carrega calado e só falha na linha que USA `porta` (um `ph2d.set` que recebe uma tabela em vez de entidade).

## 2. O oráculo (Godot 4.7.2, MIT — corrido, não lido)

[`ferramentas/oraculo_objeto_no_topo/`](ferramentas/oraculo_objeto_no_topo/corre.sh) corre
`godot --headless --check-only` sobre 11 casos; a saída está em [`saida.txt`](ferramentas/oraculo_objeto_no_topo/saida.txt).

| Caso | Godot |
|---|---|
| `var porta = get_node("Porta")` · `$Porta` · `%Porta` · `(get_node(…))` | **erro** `GET_NODE_DEFAULT_WITHOUT_ONREADY`: «won't return nodes in the scene tree before `_ready()`; use `@onready`» |
| `@export var porta: Node = $Porta` | o mesmo erro (o `@export` não salva o `$`) |
| `@onready var porta = $Porta` · `@export var porta: Node` · dentro do `_ready()` | nada |
| `var portas = [$Porta, $Janela]` · `var n = get_node("Porta").name` | **nada — ponto cego**: só olha para o valor INTEIRO |

## 3. A lei do PH2D

- **Aviso** (amarelo, a lista de problemas e o sublinhado da chamada inteira — o idioma do I7 e do item 8, na
  pausa da escrita) em todo `ph2d.object(…)` que corre no topo (fora do corpo de qualquer função) e **não** é o
  2.º argumento de `ph2d.property` (com ou sem parênteses à volta).
- Classe `object_at_top` (`ph2d-code-explain`), texto em `code.explain.object_at_top[.why]`, porta
  `Syntax::stray_top_objects` (`ph2d-code-syntax/src/facts.rs`), ligação em `lint.rs::top_object_lints`.
- Não precisa de cena: no topo nenhum objecto existe ainda (os avisos de nome do item 8 calam-se sem cena; este não).
- **Divergência medida do Godot:** a lista e o `.name` também avisam. No Godot o ponto cego custa pouco (o `$`
  dá `null`); aqui o valor é uma tabela que parece um objecto e falha longe — sem ponto cego.
- **Aviso, não erro:** o Godot trata-o como erro por omissão (`Warning treated as error`), mas o nosso idioma
  de lint é o aviso, e o script continua a carregar (a falha real é na linha que usa o valor).
- Fora do âmbito: `ph2d.object` dentro de uma função chamada no topo (o Godot também não segue chamadas).

## 4. Provas

- `ph2d-code-syntax` `o_objecto_no_topo_fora_da_propriedade_e_so_ele` — os casos do oráculo em Luau.
- `ph2d-app-code` `o_objecto_no_topo_fora_da_propriedade_e_um_aviso` — de ponta a ponta (o valor da propriedade e o do `init` não avisam);
  **RED medido** com a ligação no `lint_tick` comentada (`left: []`, `right: ["object_at_top"]`).
- Smoke conduzido sobre a cena 9 com o binário da linha:
  [`ferramentas/conduz_objeto_no_topo/`](ferramentas/conduz_objeto_no_topo/corre.sh) — fotos 01 (sublinhado),
  02 (a lista: «ph2d.object here runs before the scene exists · Line 7»), 03 (desfeito: o aviso some).
