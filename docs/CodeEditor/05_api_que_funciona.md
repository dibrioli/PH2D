# 05 — A API que FUNCIONA (Onda 2, item 6 · decisão do dono 20/10: «vou querer consertar as funções»)

> Leitor: a LLM da linha. Fila: [03_onda2.md](03_onda2.md) item 6 · achados de origem:
> [04_pagina_de_ajuda.md §9](04_pagina_de_ajuda.md#9-achados-do-runtime-ao-escrever-o-texto-20-10-medidos-com-a-bancada-censocorre).
> Regra da onda: cada cura muda o texto da página (`api_help.rs`, `ph2d-i18n/src/code_help/`) e o `doc` da
> `api_table` **no mesmo commit** — `toda_entrada_da_api_tem_ajuda` e o `censo` seguram-nos.

## 1. Oráculos (corridos)

**Godot 4.7.2** (MIT), motor sem janela a 60 quadros fixos sobre casos nossos —
[`ferramentas/oraculo_api/`](ferramentas/oraculo_api/) (`api.gd`, `corre.sh`, fixtura `saida.txt`):

| Caso | O que acontece (medido) |
|---|---|
| `await` dentro do gancho de cada quadro | funciona; o gancho **continua a ser chamado em todo quadro** enquanto aquela chamada espera (chamada 8 no quadro 8) — cada espera é uma corrotina independente |
| quando retoma | no quadro em que o tempo acumulado chega ao pedido, **depois** dos ganchos desse quadro; a contagem inclui o quadro em que a espera começou (`1/60` a 60 Hz retoma no próprio quadro; `0` também) |
| resíduo do `float` | `0.05` e `0.1` levaram **um quadro a mais** (o `time_left -= delta` em `double` fica positivo por 1e-17) — artefacto, não lei |
| objecto apagado a meio da espera | a continuação **nunca corre** |
| `find_child(nome, recursivo)` | o **primeiro em pré-ordem** (o `Pasta/Alvo` fundo antes do `Alvo` raso, que vem depois na árvore); inexistente → `null` |
| nomes repetidos | o Godot não os deixa entre irmãos (renomeia `@Node2D@3` / `Alvo2`) |
| criar (`instantiate` + `add_child`) | **imediato**: `_ready` corre dentro da chamada, achável por nome na linha seguinte; o `_process` dele começa no quadro SEGUINTE |
| apagar (`queue_free`) | adiado ao **fim do quadro**: no resto do quadro continua válido e achável (e ainda processa); no seguinte já não existe |
| ler outro objecto | qualquer objecto, com ou sem script |
| tecla | `just_pressed` verdade em **exactamente um** quadro, `just_released` idem; um **toque** (carregar e largar no mesmo quadro) dá `just_pressed` E `just_released` no quadro seguinte — nunca se perde |
| rato | botões: estado segurado; posição sem janela não é medível (headless) — a lei «a posição de MUNDO passa pela câmara» vem do produto (`get_global_mouse_position`) |
| `set_meta(k, null)` | **apaga** a chave (`has_meta` falso, sai da lista); listas voltam listas (`Array`, `[0]=10`), dicionários voltam dicionários; `remove_meta` existe |

**Lua 5.1** (`/usr/bin/lua5.1`, referência da linguagem): `loadstring` compila no ambiente global (`getfenv(f) == _G`);
`string.rep("x", 1e10)` dá erro de memória (ou o texto gigante) — **nunca vazio**.

**Roblox** não corre nesta máquina ⇒ só a documentação, declarado como LIDO: `task.wait(t)` devolve o tempo que passou;
`Instance.new`/`:Clone()` são imediatos; `:Destroy()` imediato; `FindFirstChild(nome, true)`; `UserInputService:IsKeyDown`.

## 2. O nosso, medido (`crates/ph2d-script/src/scene_api_tests.rs`, perfil `smoke`)

| Medida | Resultado |
|---|---|
| `loadstring("vazou = 42")()` no script A; B lê `vazou` | **42** — o ambiente é PARTILHADO entre scripts; e o código carregado NÃO vê os globais do próprio script (`-1`) |
| `#string.rep('x', 1e10)` · `2^31` · `2^32+3` | **0 · 0 · 0** (mentira) |
| `string.sub('abcdef', 2, 1e10)` | **`""`** (devia `bcdef`) |
| `string.byte('abc', 1, 1e10)` · `table.unpack({1,2,3}, 1, 2^31)` | **`nil`** · **`nil`** (mentiras) |
| `table.create` / `buffer.create` / `utf8.char` / `select` / `string.char` fora do alcance | erro — honesto |
| `bit32.band(2^32+5)` | `5` — certo (módulo 2³² é a lei do `bit32`) |
| custo por gancho (mínimo de 5 × 200 000) | `call` **49 ns** · corrotina nova **198 ns** · corrotina reaproveitada (`Thread::reset`) **107 ns** |

Causa da família do inteiro: o Luau converte o número para `int` por *cast* (`luaL_checkinteger`), e fora de ±2³¹ dá
`INT_MIN` ⇒ índice negativo ⇒ vazio. A lei certa é **saturar**: um índice ou contagem além de 2³¹ comporta-se como
`2^31-1` (o fim do texto, ou o erro «too large» do próprio Luau). Nenhuma constante nossa: o limite é o do `int` do Luau.

## 3. O desenho — uma lei por item, cada uma com o seu portão (vermelho primeiro)

### 3.0 A PISTA DO MUNDO (a porta que (b) (c) (d) usam)

Hoje o script só vê uma **fotografia** (poses dos objectos COM script) e escreve numa **fila**. A cura não larga o
HR-8 (todos leem o início do tique; as escritas aplicam-se no fim) — muda só QUEM o garante: durante os ganchos de um
tique, o `SceneScripts::tick`/`hear` empresta o `&mut World` à VM por `Lua::scope` (o `mlua` 0.12 di-lo seguro: o que
o `scope` cria morre com ele). As funções `ph2d.*` estáticas chamam a função da pista guardada no registo da VM; fora
de um tique a pista não existe e a função diz **«só funciona com a cena a correr (dentro de init/update/on_signal)»**.
Como as escritas continuam na fila até ao fim do tique, ler o mundo durante o tique **é** a fotografia — para TODO
objecto, sem copiar a cena por tique.

### 3.1 (a) `ph2d.wait(s)` — a espera

- **Cada gancho corre numa corrotina** (reaproveitada, `Thread::reset`: +58 ns por chamada medidos). Se ela cede, fica
  num `Scheduler` com o objecto dono; o gancho do quadro seguinte é chamado na mesma (lei do Godot).
- **Relógio em microssegundos inteiros**, como o `Timer` da casa: `alvo = round(s·1e6)`, cada tique soma
  `round(dt·1e6)`. Retoma **no fim do tique** em que o acumulado chega ao alvo (a contagem inclui o tique em que a espera
  começou — Godot), depois dos ganchos e antes de aplicar as escritas. `wait(0)` = no fim deste tique. O resíduo do
  `double` do Godot (um quadro a mais) fica de fora: a casa conta inteiros.
- Devolve o **tempo que passou** (Roblox). Objecto apagado, script trocado, objecto parado por erro ou rebobinar ⇒ a
  espera é **esquecida** (Godot). Pausa ⇒ não há tiques ⇒ a espera pára com o jogo.
- Fora de um gancho (no topo do script): erro claro, nunca o «attempt to yield across…».
- Portões: `uma_espera_retoma_no_tique_em_que_o_tempo_chega` (1/60, 0,05, 0,1 a 60 Hz: 1, 3, 6 tiques) ·
  `o_gancho_continua_a_ser_chamado_enquanto_espera` · `apagar_o_objecto_esquece_a_espera` ·
  `rebobinar_esquece_as_esperas` · `wait_no_topo_diz_porque`.

### 3.2 (b) `ph2d.find_by_name(nome)` — achar por nome

- Pela pista: o **primeiro em pré-ordem da hierarquia** (a mesma ordem da lista de objectos:
  `build_hierarchy_snapshot`, raízes por `RootOrder`, filhos por `sibling_key`), `nil` se não houver. Um objecto criado
  neste tique já se acha; um pedido de apagar só tira no fim do quadro (Godot). Os MESTRES (receitas) não contam como
  objectos da cena — `ph2d.spawn` é que os usa.
- Morrem `NameSnapshot`, `provide_name`, `clear_names` (controlo morto sem leitor).
- Portões: `achar_por_nome_devolve_o_primeiro_da_lista` · `um_objecto_criado_ja_se_acha` · `nome_inexistente_e_nil`.

### 3.3 (c) criar, apagar, pôr script

- **`ph2d.spawn(mestre, x?, y?) -> id`**: cria uma cópia do mestre com esse nome (a MESMA porta da `Factory`:
  `instantiate_master_many` + `Spawned`, logo **transiente**: não vai para o documento nem para o `Ctrl+Z`, e o
  rebobinar varre-a — `sweep_spawned`). **Imediato** (Godot): o id volta na linha, `ph2d.set(id, …)` no mesmo gancho
  pousa no fim do tique, o `init` do script dele corre no tique SEGUINTE. Mestre inexistente ⇒ erro com o nome.
  `ph2d.spawn()` sem mestre = objecto vazio transiente. A porta vive em `ph2d-app-components` e entra no `tick` como
  `&mut dyn SceneDoors` (o `ph2d-script` não conhece mestres).
- **`ph2d.despawn(id) -> boolean`**: pedido de **morte** (`Death`, causa `Killed`) que o dreno único da fase da fábrica
  aplica no fim do quadro (Godot: `queue_free`). Objecto do DOCUMENTO ⇒ **recusado em voz** (linha de aviso no console,
  o objecto continua a correr; devolve `false`) — a lei do verbo `Destroy`: *«o que acontece numa corrida não é
  documento»*. Apagar limpa também o estado lateral e as esperas dele.
- **`ph2d.attach_script(id, "nome.luau")`**: põe o `LuauScript` da biblioteca num objecto **transiente**; num objecto do
  documento, recusado em voz (pôr um componente é editar o documento). Script inexistente ⇒ erro.
- Saem `ph2d.spawn_named`, `SpawnQueue`, `SpawnCommand` e as quatro linhas do `UNCONSUMED_PENDING`
  (`the_intent_drain_reaches_every_variant.rs`): sem fila, não há dreno a provar.
- Portões: `spawn_devolve_um_id_que_ja_se_le_e_escreve` · `a_copia_nao_vai_para_o_documento_e_o_rebobinar_a_varre` ·
  `despawn_tira_no_fim_do_quadro` · `despawn_de_um_objecto_do_documento_e_recusado_em_voz` ·
  `attach_script_poe_o_script_e_o_init_corre_no_tique_seguinte`.

### 3.4 (d) `ph2d.get` de qualquer objecto — e o `ph2d.set` que fugia ao rebobinar

- `get` lê pela pista: **todo objecto com pose**, com ou sem script.
- ⛔ Achado ao ler (20/10): `ph2d.set` num objecto **sem** script escreve, mas a ponte só declara ao ledger quem tem
  `LuauScript` ⇒ a pose da corrida vira DOCUMENTO e o rebobinar não a desfaz. Cura: o `SceneReport` leva o «antes» de
  cada objecto escrito e o `declare` cobre-os todos; o rebobinar larga `drive.driven_by(Driver::ScriptPose)`.
- Portões: `get_le_um_objecto_sem_script` · `set_num_objecto_sem_script_volta_ao_rebobinar`.

### 3.5 (e) teclado e rato

- **`ph2d.key_down(t)` · `ph2d.key_pressed(t)` · `ph2d.key_released(t)`** (Godot/Roblox), nomes em minúsculas
  (`"a"`…`"z"`, `"0"`…`"9"`, `"space"`, `"enter"`, `"left"`, `"f1"`…), tabela única em `ph2d-input` (`Key::script_name`
  / `from_script_name`) com gate de ida-e-volta sobre tudo o que o normalizador da shell produz. Nome desconhecido ⇒
  **erro** (uma gralha não pode ler-se como «tecla largada»). O tipo `KeyName` no `ph2d.d.luau` dá a lista ao assistente.
- **`ph2d.mouse_position() -> (x, y)`** em coordenadas de MUNDO, pela porta que o Motion já usa
  (`scene_camera_window` + `Camera2d::screen_to_world`); **`ph2d.mouse_down(b)` / `mouse_pressed(b)` /
  `mouse_released(b)`** com `"left" | "right" | "middle"`. Um clique só conta se COMEÇOU na cena (um clique no Play não
  dispara um tiro); o largar passa sempre (a assimetria do teclado).
- **A borda vive no TIQUE, não no quadro**: o que aconteceu desde o último tique é visto pelo PRIMEIRO tique a seguir,
  e só por ele (dois tiques num quadro ⇒ um só «pressed»; um quadro sem tique ⇒ a borda espera). Um toque dentro de um
  quadro dá `pressed` e `released` (Godot).
- A posse do teclado é a do controlo de jogador que já existe (acorde e aba de código não chegam ao jogo).
- O empurrão dos dispositivos sai da shell para `ph2d-app-components` (`script_input`), a shell só o chama.
- `ph2d.input` fica para o comando de jogo, como está.
- Portões: `uma_tecla_e_pressed_num_so_tique` · `um_toque_no_mesmo_quadro_nao_se_perde` ·
  `dois_tiques_num_quadro_veem_uma_borda` · `um_nome_de_tecla_errado_e_erro` · `todo_codigo_da_shell_tem_nome` ·
  `o_rato_e_lido_no_mundo` · `um_clique_fora_da_cena_nao_conta`.

### 3.6 (f) o estado lateral

- `state_set(e, k, nil)` **apaga** a chave (Godot/Roblox).
- As chaves de uma tabela guardam o TIPO (`PodKey`: número ou texto) ⇒ uma lista volta lista (`#t` certo).
- **Rebobinar limpa** o estado (é estado da corrida); apagar um objecto limpa o dele. Recarregar o texto mantém-no.
- Portões: `nil_apaga_a_chave` · `uma_lista_volta_lista` · `rebobinar_limpa_o_estado`.

### 3.7 (g) `loadstring` e a família do inteiro

- `loadstring` compila no **ambiente do script que o chama** (cada ambiente recebe o seu `loadstring`): vê e escreve
  os globais do próprio script, nunca os de outro. Portão: `loadstring_nao_passa_globais_entre_scripts`.
- A família do inteiro: as funções da biblioteca que leem índices/contagens por `luaL_checkinteger` saturam o argumento
  a ±(2³¹−1) antes de o Luau o ver. Portão: `um_indice_alem_de_2_31_satura` — para cada (função, argumento) da lista,
  `2^31`, `1e10` e `-1e10` dão o mesmo que `2^31-1` / `-(2^31-1)`; e um censo que obriga toda função das bibliotecas
  `string`/`table`/`utf8`/`buffer` a estar na lista ou na lista «medida e honesta».

## 4. Fora do âmbito (declarado)

- Teclado com o editor a ouvir a mesma tecla: como o controlo de jogador de hoje (não há modo de jogo separado).
- Roda do rato; texto escrito (IME) para o jogo.
- Os nomes iguais continuam permitidos na cena (o `find_by_name` devolve o primeiro).

## 5. Ordem (um bloco por lei; série só onde uma muda a seguinte)

1. `ph2d-input`: nomes de tecla, bordas por tique, rato. 2. `ph2d-script`: a pista do mundo + (a)(b)(d)(f)(g) + os
dispositivos no host + texto da ajuda. 3. `ph2d-app-components`: as portas (`SceneDoors`, mortes, `script_input`, o
ledger). 4. A shell compõe. 5. O gate do dreno e o assistente (`ph2d.d.luau`, G3). 6. Cena de smoke 6 (um jogo pequeno:
nave às setas, tiro com espaço que nasce de um mestre e some ao fim de `wait`, clique cria no rato).

## 6. Feito (20/10) — o que mudou no caminho e o que a FOTO achou

Commits `cad77678e0` (a API) e `27f74b40fc` (cena 6 + override). Tudo do §3 entrou como escrito, com três
correcções de rota:

- **O `_G` não vazava.** Suspeitei dele e medi: a tabela partilhada já era TRANCADA (escrever é erro), e
  isso estava documentado na página `_G`. Só o `loadstring` vazava. A troca do `_G` por script foi desfeita;
  o gate `loadstring_nao_passa_globais_entre_scripts` confirma que ele continua trancado.
- **O portão da família do inteiro corre cada caso como SCRIPT.** No ambiente global o compilador usa a
  versão interna da função (`FASTCALL`) e salta a cura (`math.ldexp(1, 1e10)` deu `0`); no ambiente de um
  script, onde o código do artista corre, a cura vale. Os nomes do `bit32` no Luau são `lrotate`/`rrotate`.
  `utf8.len`/`utf8.codepoint` com um fim além do texto passaram a erro honesto.
- **A tick recebe o `SimWorld`** (não o `World`): a instanciação de mestres (`instantiate_master_many`) é
  sobre ele. `scene_tick`/`scene_hear` (sem portas, os testes) e `scene_tick_with`/`scene_hear_with` (o
  produto, com `ScriptDoors`).

**Achado pela foto da cena 6 (nenhum teste via):** a 1.ª cópia atirada perdia o script no quadro seguinte.
O sync mestre → instância (`instance_sync.rs`) propaga a AUSÊNCIA de um componente, e *«sem eco do mestre,
o mestre ganha»* — a 1.ª instância de uma receita ainda não tinha eco. Cura: o script que um script põe
numa cópia é um **override** dela (`SceneDoors::attached` → `ObjectInstance.overrides`). O gate da cena 6
passou a correr o registo do produto, o `sync_instances` e o `vec sync` em cada quadro (vermelho sem a cura).

**A família do inteiro, classificada** (Luau 736, `luaL_checkinteger`/`optinteger` lidos no fonte vendido):
saturadas — `string.sub/byte/rep/find/match/gsub/unpack`, `table.unpack/concat/move/remove`,
`utf8.len/codepoint/offset`, `math.ldexp`, `bit32.lrotate/rrotate` (módulo 32) — o `table.remove` além da lista dá
`nil` e não tira nada, igual dentro e fora do alcance (medido); o LUGAR do `table.insert` não satura (pôr em 2³¹−1 o
que foi pedido em 2³²+1 seria outra mentira — medido: aceitava e escrevia noutro sítio) ⇒ erro; honestas como estão
(erro fora do alcance, MEDIDO no gate) — `table.create`, `buffer.create`, `select`, `tonumber(base)`, `utf8.char`,
`string.char`, `table.find`, `bit32.extract/replace`; `math.random(1, 2^40)` dá erro com mensagem enganadora («interval is
empty») e `math.randomseed` satura em silêncio (só a semente) — **declarados**.

**Diferenças declaradas:** a roda do rato e o texto escrito (IME) não chegam ao jogo; o teclado que chega ao
jogo também chega aos atalhos do editor (como o controlo de jogador de hoje); nomes iguais na cena são
permitidos (`find_by_name` dá o primeiro); `ph2d.wait` dentro de uma corrotina do próprio artista pausa essa
corrotina.

## 7. O fecho (20/10) — o que as auditorias e os portões acharam, e a cura

- ⛔⛔ **As bordas do teclado e do rato nunca chegavam ao script no APP** (lente de costura): o
  `pump_gamepad` chamava `InputState::begin_frame` no início do quadro, DEPOIS de os eventos do winit
  terem chegado e ANTES de a porta dos scripts os ler. Os gates injectavam as bordas à mão e o piloto das
  fotos passava ao lado do `InputState`. Cura: o comando limpa só as dele; as do teclado e do rato limpa-as
  `script_input::frame` DEPOIS de as entregar (e sem script também). O piloto passou a pôr EVENTOS no
  `InputState` real e só arma na cena 6. Gate `a_borda_que_chega_entre_quadros_chega_ao_script_uma_vez`.
- **Corrotinas deixaram de se reaproveitar** (lente de correcção): um `coroutine.running()` guardado
  alcançava o gancho de OUTRO objecto que herdasse a corrotina. Custo medido no §2: 198 ns por gancho
  (nova) contra 49 (`call`). Uma espera cuja corrotina foi retomada por fora já não é retomada.
- **`loadstring` e bytecode — o furo apontado pela auditoria NÃO existia** (medido): o modo seguro do
  mlua (`Lua::new`) já recusa pedaços binários; um `set_mode(Text)` acrescentado sobreviveu à mutação e
  saiu. O gate `o_loadstring_de_um_script_recusa_bytecode` (bytecode REAL, do `Compiler`) fica.
- Trocar de script a meio do quadro (`attach_script` num objecto que já corre outro) renasce já; o
  `init` de quem pediu para morrer não corre; os filhos de uma cópia contam como da corrida.
- O protótipo `tests/spike` C3 passou a medir as ESPERAS DA CENA: 1000 esperas em [0,500; 0,510) s a
  60 Hz ⇒ p50 11,7 ms · p99 16,6 ms · máx 16,7 ms (tecto 1 quadro = 16,67 ms), 0 vazadas.
- Declarados (aberto): as mensagens de erro de argumento que passam pelas funções embrulhadas levam um
  prefixo `ph2d:NN:` no texto do console (a linha do script continua certa); `ph2d.wait` dentro de uma
  corrotina do artista devolve a marca a quem a retomou; uma tecla com dois lados (`shift`) dá
  `key_pressed` quando o 2.º lado desce com o 1.º em baixo; o `on_signal` não vê bordas num quadro com
  tique; chaves inteiras do estado acima de 2^53 perdem precisão; um clique na cena que o editor também usa
  (a alça da âncora, a onda do áudio) chega ao jogo — o editor não tem modo de jogo separado.

