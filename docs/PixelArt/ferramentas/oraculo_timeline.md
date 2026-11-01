# Oráculo da timeline (onda P6h) — Aseprite Trial 1.3.18.6, caixa preta

Medido em 2026-10-30. Condutor: [`trial_timeline.py`](trial_timeline.py) (docstring = uso e arranque).
Fixtura: [`shells/pixel-scenes/tests/fixtures/oracle_trial_timeline.json`](../../../shells/pixel-scenes/tests/fixtures/oracle_trial_timeline.json).

**Entrada NOSSA** `tl.ase` (`trial_timeline.py gera`): 16×16, 6 quadros de 100…600 ms, cor do quadro
na camada A (de baixo, NÃO contínua), camada B (de cima, contínua) com cels só em 1 e 3, tag `walk` 2..4.
**Leitura**: o Trial não grava; a duração lê-se na barra de estado ao clicar o cabeçalho
(`<n> <ms>ms/<total>s`; com intervalo também `[<a>...<b>] [<soma>s]`), a ordem dos quadros pela cor do
canvas, o resto por fotografia. Cada lei abaixo tem o seu caso em `CASOS` (`trial_timeline.py caso <dir> <nome>`).

| Item | Comportamento medido | Caso |
|---|---|---|
| 1. Constant Frame Rate | Abre o diálogo Frame Properties com `Frame number: [1...6]` e a duração do **quadro 1 do sprite** (700 com q1=700, q5=50, activo 4). Aplica a **todos** os quadros mesmo com intervalo 3..5 seleccionado. 250 → seis × 250 ms. **1** Ctrl+Z desfaz tudo. | `cfr_*` |
| 2. Frame Properties com intervalo | Intervalo contíguo 4..6 → `[4...6]`, campo = duração do **1.º quadro do intervalo** (400; nem o mínimo 50 nem o activo 600). Picks {2,4} → `Multiple Frames`, campo = 200 (o 1.º). OK escreve o valor em **todos os seleccionados e só neles** (333 → q2 e q4; q3 fica 300). Durações diferentes não são assinaladas. 1 undo. | `cfr_prefill`, `fp_picks` |
| 3. Tag | Duplo clique no rótulo abre Tag Properties (Name, From, To, Animation Direction, Repeat ∞), **base 1** (`walk` = 2..4). Botão direito no rótulo: Tag Properties / Delete Tag. From 5 To 3 → **5..5** (to = from, sem troca). To 99 → **clamp ao último** (2..6; o diálogo reabre com 6). From 0 → **1**. Arrastar a ponta direita para o quadro 6 → 2..6; a esquerda para o 1 → 1..6 (soltar fixa a ponta no quadro sob o cursor). Arrastar o rótulo não faz nada. Editar a tag = 1 undo. Delete Frame que cobre a tag inteira **apaga a tag** (1 undo devolve). | `tag_*`, `del_frame` |
| 4. Camada contínua | Liga/desliga **só no 3.º ícone da linha da camada na timeline** (dois pontos = descontínua, barra = contínua; um clique alterna). **Não** está em Layer Properties (Name, Mode, Opacity) nem no menu Layer. Camada nova (Shift+N): **descontínua**. O clique no ícone **não é passo de undo**. Alt+N no quadro 1: na contínua o cel novo é **LIGADO** ao anterior; na descontínua é uma **cópia independente**. A tag a seguir desloca-se (2..4 → 3..5). 1 undo. | `cont_*` |
| 5. Selecção | Clique no cabeçalho = esse quadro (todas as camadas). **Shift+clique ACRESCENTA esse quadro sozinho** (picks): 2 + Shift 4 = {2,4}, o 3 fica de fora; depois de arrastar 2..3, Shift 5 = {2,3,5}. Ctrl+clique fora da caixa também acrescenta; Ctrl+clique num já seleccionado ou dentro da caixa **não faz nada** (não tira). Arrastar cabeçalhos = intervalo contíguo. Arrastar cels = rectângulo camadas×quadros. Clique no nome da camada = a linha inteira. **Esc limpa**; clique em área vazia da timeline **não limpa**. Caixa amarela à volta do envelope da selecção (também com picks). | `sel_*` |
| 5. Comandos sobre o intervalo | **Delete Frame (Alt+C)** apaga os quadros seleccionados (2..4 → ficam 1,5,6; picks {2,4} → ficam 1,3,5,6), 1 undo. **Reverse Frames (Alt+I)** inverte cels **e durações** (2..4 → ordem 1,4,3,2,5,6; 100,400,300,200,…); fica cinzento com um só quadro. **Link Cels** (botão direito no intervalo de cels: Cel Properties, Delete, Unlink, Link Cels, Duplicate Cel(s), Duplicate Linked Cel(s)) junta A1..A3 num cel ligado, 1 undo. **Delete** (tecla) sobre cels A2..A4 apaga os cels e mantém os quadros. **Mover**: arrastar a **borda** da caixa (arrastar um cabeçalho começa outro intervalo); 2..3 solto entre 5 e 6 → ordem 1,4,5,2,3,6, durações viajam com os quadros, o bloco movido fica seleccionado (4..5). Frame Properties: ver item 2. | `del_*`, `reverse`, `link_cels`, `clear_cels`, `mover` |

### Ronda 2 (2026-11-01, P6j) — o que a ronda 1 deixou por medir

Mesma entrada; antes de correr, a divisória da timeline arrasta-se de `y=741` para `y=778` (o mapa abaixo
vale). Gesto novo `dk:` = o arrasto `d:` com Ctrl premido só antes de largar.

| Item | Comportamento medido | Caso |
|---|---|---|
| Ctrl ao largar o bloco | 2..3 com Ctrl na fronteira 5\|6 → **8 quadros**: cópias (durações e cels, o cel de B do 3 também) entram na fronteira, os originais ficam (100,200,300,400,500,**200,300**,600); as cópias passam a ser o intervalo (6..7); a tag não mexe; 1 undo. O **Ctrl+Z devolve o intervalo aos originais** (2..3) e a mesma largada copia de novo, igual. | `mover_ctrl`, `mover_ctrl_undo_de_novo` |
| Mover só cels | A2..A3 arrastado 3 colunas → A5 = antigo A2, A6 = antigo A3 (os A5/A6 de antes perdem-se), A2/A3 vazios; quadros e durações **não mexem**; o intervalo segue (A5..A6). Com Ctrl as origens ficam (1 undo). As **filas andam também**: B1 largado na fila A, coluna 3 → A3 = antigo B1, B1 vazio. | `mover_cels`, `mover_cels_ctrl`, `mover_cels_b` |
| Shift+clique em cel depois de arrasto | A2..A3 + Shift A5 = quadros {2,3,5} (o 4 não), camada A («Multiple Frames»); Shift B5 = camadas B..A × {2,3,5}. Igual à nossa lei — sem cura. | `sel_shift_cel_depois_arrasto` |
| Esc com selecção E intervalo | **Um** Esc limpa os DOIS. E **Select All (Ctrl+A) limpa o intervalo** na hora. | `esc_sel_e_intervalo`, `esc_intervalo_e_sel` |
| Reverse com quadros soltos | Activo; inverte o **envelope** do 1.º ao último escolhido, os de permeio também: {2,4,6} → 1,6,5,4,3,2; {2,3,5} → 1,5,4,3,2,6. | `reverse_picks`, `reverse_picks_span` |
| Undo da tecla Delete | A2..A4 (e B1..A3, duas camadas): **1** Ctrl+Z devolve tudo. Igual à nossa lei. | `clear_cels_undo`, `clear_cels_duas` |
| Paleta do sprite nascido (#8) | *Paste as New Sprite* fica **cinzento** sob o Xvfb mesmo depois de Ctrl+C; o irmão *New Sprite From Selection* (Ctrl+Alt+N) num RGBA leva a paleta da ORIGEM (as 8 amostras); um File ▸ New RGBA nasce com a de omissão (32). | `paste_new_sprite_rgb` |

## Mapa da tela (janela 1600×1000, escala de UI 2, timeline em baixo com Tab)

Cabeçalhos dos quadros `y=826`, `x = 372 + 23,8·k`; linha B `y=850`, linha A `y=874`; ícone de
contínua `x=220`; nome da camada `x≈280–300`; rótulo da tag `(410,797)`, pontas da tag `y≈809`;
menu Frame `(251,10)` ▸ Constant Frame Rate `(329,282)`; Tag Properties From `(882,438)`, To `(882,476)`,
OK `(805,618)`. O «Don't Save» muda de sítio: acha-se por modelo (`trial_tpl/tl_dont_save.png`).

## ⛔ Recusas / limites

| O quê | Porquê |
|---|---|
| Ler o `.ase` de saída | O Trial não grava. Tudo por fotografia (barra de estado, cor do canvas, diálogos). |
| LibreSprite | Não foi preciso: o Trial respondeu aos cinco itens. |
| Ctrl ao largar, mover só cels, undo do Delete | Medidos na ronda 2 (acima). |
| *Paste as New Sprite* em si | Cinzento no Trial sob o Xvfb; medido no irmão *New Sprite From Selection*. |
| Cópia com Ctrl numa camada contínua (ligado ou cópia?) | Medido na «Ronda 3 (lacunas)»: LIGADAS. |
| Bloco de cels largado fora da grelha ou numa fila de grupo | Medido na «Ronda 3 (lacunas)». |
| Clique no ícone de contínua do CABEÇALHO (todas as camadas) | Só observado (mostra barra quando todas são contínuas); o clique não mudou nada visível com as duas já contínuas. |
| Shift+clique em cel | Lido só pelo realce dos cabeçalhos (1 e 3 realçados, 2 não) — mesma regra dos cabeçalhos, confiança menor. |
| P5: Jump/Loop/Preview/Grupos/Once PingPong | Medidos na continuação (`ronda3b`). |
| P5: Preview — redimensionar, 1.º botão | Não medido (redimensionar não tentado; o 1.º botão sem efeito visível e sem tooltip). |
| P5: Set Loop Section — undo, e F2 com picks soltos | Não medido. |
| P5: corrida 1, casos `s0_*` com opção | Em `s0` (sem filas de tags) o menu abre noutro sítio e o clique tocou o item de CIMA; a fixtura regista o item real (`clique_real`) e a velocidade foi refeita no `s1` (corrida 2). |
| Fechar e reabrir | ⚠️ `Ctrl+O` do mesmo ficheiro já aberto abre UMA SEGUNDA aba: as primeiras corridas mediram sobre documentos sujos. `abre()` fecha tudo (8× Ctrl+W + «Don't Save» por modelo); as leis da tabela vêm todas de corridas depois dessa cura, ou de casos cuja leitura não depende do estado de partida. Um `Cel Properties` não-modal ficou aberto (clique errado) e não interfere. |

### Ronda 3 (2026-11-01, P5) — tocar: subtags, velocidade, opções

Condutor [`trial_p5_play.py`](trial_p5_play.py) (`gera`, `todos <dir>` / `todos <dir> 2`); fixtura
[`oracle_trial_play_p5.json`](../../../crates/ph2d-pixel-core/tests/fixtures/oracle_trial_play_p5.json).
Entradas: 8 quadros de 100 ms com uma cor cada; `s1` out 1..6 Fwd ∞ ∋ in 2..4 PingPong ×2; `s2` out 1..6 Fwd ×2 ∋
in 2..4 Reverse ∞; `s3` a 1..4 ∩ b 3..6 (Fwd ∞); `s4` = `s1` com a lista trocada; `s0` sem tags. Índices **base 0**.
Leitura: vídeo 50 q/s do centro do canvas. As opções ficam no **botão direito no Play da timeline**: Play Animation
(Enter), Play Preview Animation (Shift+Enter), Playback Speed 0.25×/0.5×/1×/1.5×/2×/3×, Play Once, Play All Frames
(Ignore Tags), Play Subtags & Repetitions (**ligado por omissão**), Rewind on Stop.

| Item | Comportamento medido | Caso |
|---|---|---|
| Subtags na timeline | Tags aninhadas e sobrepostas: **uma fila de rótulos por nível**, a de dentro abaixo e recolhida (foto `s1_tab`, `m_s0_s3`). | — |
| Tag-raiz ao começar | A raiz é a tag **mais interior** que contém o quadro activo (a ordem da lista NÃO conta: `s4` = `s1`); com duas sobrepostas, a que começa primeiro (`s3` f3 → a). Fora de tags, a raiz é a timeline inteira. | `s*_on_f*` |
| Subtags ON | Dentro da raiz, cada tag interior em que se **entra** toca com a sua direcção e o seu `repeat` (repeat ∞ conta **1** passada quando não é a raiz), entrando pelo início da 1.ª passada (Reverse entra no `to`). `s1` f0: `0 1 2 3 4 3 2 5 6 7 │0…`; `s2` f0: `0 1 4 3 2 5 6 1 4 3 2 5 6 7 │0…`. Ao sair de uma tag para outra que a sobrepõe entra-se na outra pelo `from` dela: `s3` f0: `0 1 2 3 4 3 4 5 6 7 │0…`. Raiz com repeat ∞ volta a si (`s1` f1: `1 2 3 4 3 2 5 6 │1…`); raiz com repeat **finito** acaba e o tocar **continua na timeline inteira** (não na tag de fora): `s1` f3: `3 4 3 2 5 6 7 0 1 2 3 4 3 2 5 6 7 │0…`; `s2` f1: `1 4 3 2 5 6 1 4 3 2 5 6 7 0 │1…`. Começar a meio conta como 1.ª passada. | `s1..s4_on_*` |
| Subtags OFF | Só a raiz toca, **em ciclo para sempre e ignorando `repeat`**; as interiores são ignoradas. Fora de tags = timeline inteira em frente (`s1` f0: `0…7 │0`). `s1` f1: `1…6 │1`; `s1` f3: `3 4 3 2 3 4 3 2…`; `s2` f3: `3 2 4 3 2 4…`. | `s*_off_*` |
| Play Once | Ao Enter **salta logo para o início da raiz** (Reverse: o `to`), toca-a **uma vez em linha recta, sem subtags nem repetições**, e **volta ao quadro de partida** e pára. Raiz no Once = tag mais interior com repeat ∞; se essa tem repeat finito, a timeline inteira. `s0` f2: `2→0 1…7 →2`; `s1` f5: `5→1…6 →5`; `s1` f2/f3: `→0…7 →` partida; `s2` f1: `→0…7 →1`; `s2` f3: `→4 3 2 →3`; `s3` f2: `→1…4 →2`. Igual com subtags OFF (f0 → `0…7`). | `*_once_*` |
| Play All Frames | Ignora tags: da partida em frente, `0` depois do último (`s1` f3: `3 4 5 6 7 0 1…`). Com Once: `3→0…7→3`. | `s1_all_*` |
| Rewind on Stop | ON: ao parar (Enter) volta ao **quadro de partida** (`s0` f2 → 2; `s1` f3 → 3; f1 → 1). OFF: fica onde estava (`s1` f3 → 6). | `*_rewind_*`, `s1_norewind_f3` |
| Velocidade | Duração real = duração ÷ velocidade: 0.25× → 400 ms, 0.5× → 200, 1.5× → 66,7, 2× → 50, 3× → 33,3 (média dos runs de 20 ms). É **do documento aberto**, não preferência: pôr 0.5× no `s1` não muda o `s0`; fechar o `s1` sem gravar e reabrir volta a 1×. As outras quatro opções são **globais** (persistem entre documentos). | `s1_v*`, `s0_v0.5_outro_doc`, `s1_v0.5_reaberto` |
| Preview flutuante | Abre sozinha num HOME limpo: painel **dentro** da janela principal (canto inferior direito), título «Preview», três botões no título (centrar, tocar, fechar), mostra o quadro activo (foto `s1_aberto`). *Play Preview Animation* (Shift+Enter) toca-a e fica marcado no menu. | foto |
| «Grupos de tags» | Não existe. Botão direito no rótulo: *Tag Properties…* / *Delete Tag*. Frame ▸ Tags: *Tag Properties…*, *New Tag*, *Delete Tag*. Tag Properties: Name (+ botão de cor), From, To, Animation Direction, ☐ Repeat (∞) — sem grupo, pai nem fila (fotos `tag_rc`, `r3_subs`, `r3_loop_dlg`). A fila do rótulo é só desenho do aninhamento. | foto |
| Jump to First/Last Frame In Tag | Frame ▸ Jump to (sem atalho; ao lado: First/Previous/Next/Last Frame = Home/Left/Right/End, Go to Frame Alt+G). Vai ao `from`/`to` da tag **mais interior** que contém o activo; fora de tags, ao 0 / ao último. Repetir no `from` não sobe para a tag de fora. `s1`: f3,f2,f4 → 2 / 4; f1 → 1 / 6; f0, f7 → 0 / 7. | `ronda3b.jump_in_tag_s1` |
| Set Loop Section | **F2**, no botão direito no cabeçalho de quadro (não está nos menus Frame). Com intervalo de quadros: cria uma tag normal chamada **`Loop`** (Forward, repeat ∞, rótulo igual às outras) sobre o intervalo. De novo com outro intervalo: **move a mesma** `Loop` (não cria outra). Sem intervalo: não faz nada. Toca como qualquer tag: `s0` Loop 2..4, de f0: `0 1 2 3 4 2 3 4…`; de f6: `6 7 0 1 2 3 4 2 3 4…`. | `ronda3b.loop_section_s0` |
| Play Once, raiz PingPong ∞ | Salta para o `from`, faz ida E volta uma vez e regressa à partida: `s5` f3: `→2 3 4 3 2 →3`; f2: `2 3 4 3 2 →2`. Fora da tag a timeline passa em linha recta (`s5` f0: `0…7 →0`; f6: `→0…7 →6`). | `ronda3b.once_pingpong_inf` |
| Preview (detalhe) | Shift+Enter toca **só a Preview**; o editor fica parado (vídeo dos dois: editor 3 s no 0). Parada, segue o quadro activo (clique no cabeçalho 5 → os dois mostram 5). Roda do rato = **zoom próprio** (o do editor fica 100 %). Botões do título: 2.º = tocar/parar só a Preview (o ícone vira ■), 3.º = fechar, 1.º = sem efeito visível (provavelmente centrar). Arrastar o título move-a **dentro** da janela principal (não é janela do sistema). | `ronda3b.preview_s0`, fotos `r3_pv_*` |

### Ronda 3 (lacunas) (2026-11-01, Xvfb :79) — cópia na contínua, cels fora da grelha, fila de grupo

Mesmo condutor e mapa (timeline com Tab; a divisória arrasta-se `d:1000,746,1000,780` — em `y=741` o arrasto não
pegou). Entrada `tl.ase` e, para o grupo, `tl_grupo.ase` (`trial_timeline.py gera`): a `tl.ase` com o grupo **G**
por cima de B e a camada **C** (descontínua, cel em todos os quadros) dentro de G; filas G `y=850`, C `874`,
B `898`, A `922`. Fixtura: chave `round3_gaps` de `oracle_trial_timeline.json`.

| Item | Comportamento medido | Caso |
|---|---|---|
| Ctrl ao largar sobre a camada contínua | Quadros 1..3 copiados para a fronteira 5\|6 → 9 quadros, cópias em 6..8. Na contínua **B** cada cópia fica **LIGADA ao seu original**: B1↔B6 e B3↔B8 (clicar o cel mostra a barra de ligação entre os dois; B1 e B3 continuam sem ligação entre si). Na descontínua **A** as cópias são independentes (sem barra). | `r3_ctrl_continua` + fotos `z4`, `z5*` |
| Cels largados à direita do último quadro | A2..A3 largado 7 colunas à direita (o quadro 9 não existia) → o Trial **CRIA os quadros** que faltam: 6 → **10**; A9/A10 = antigos A2/A3, A2/A3 e A7/A8 vazios, B vazio nos novos. Cada quadro novo leva a duração do **último** (600 ms; total 4,50 s). O intervalo segue (9..10). | `r3_cels_fora_dir` |
| Cels largados abaixo da última camada | A2..A3 arrastado duas filas para baixo da A → **nada muda** (a fila prende-se à última camada, que é a de partida). | `r3_cels_fora_baixo` |
| Fora à direita E abaixo | Igual à direita: a fila prende-se (A), as colunas criam quadros (6 → 10, A9/A10). | `r3_cels_fora_ambos` |
| Cels largados na fila de um GRUPO | A2..A3 (ou C2..C3, de dentro do grupo) largado na fila G → as origens **ficam vazias** e o grupo não mostra cel: os cels **PERDEM-SE** (não há recusa nem aviso); o intervalo passa a G2..G3. **1** Ctrl+Z devolve tudo. Largado na fila G **e** à direita do último quadro: os cels perdem-se também e **não** nascem quadros (6 continua 6); fica uma caixa de intervalo desenhada fora da grelha. | `grupo_largar`, `grupo_largar_c`, `grupo_largar_dir` |
