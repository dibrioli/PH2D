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
| Ctrl ao largar o intervalo movido (copiar), mover intervalo só de cels | Não medido (fora do tempo). |
| Undo do Delete (tecla) sobre cels | Não medido. |
| Clique no ícone de contínua do CABEÇALHO (todas as camadas) | Só observado (mostra barra quando todas são contínuas); o clique não mudou nada visível com as duas já contínuas. |
| Shift+clique em cel | Lido só pelo realce dos cabeçalhos (1 e 3 realçados, 2 não) — mesma regra dos cabeçalhos, confiança menor. |
| Fechar e reabrir | ⚠️ `Ctrl+O` do mesmo ficheiro já aberto abre UMA SEGUNDA aba: as primeiras corridas mediram sobre documentos sujos. `abre()` fecha tudo (8× Ctrl+W + «Don't Save» por modelo); as leis da tabela vêm todas de corridas depois dessa cura, ou de casos cuja leitura não depende do estado de partida. Um `Cel Properties` não-modal ficou aberto (clique errado) e não interfere. |
