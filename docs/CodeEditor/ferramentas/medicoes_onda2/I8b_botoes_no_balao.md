# I8b — as correcções no balão e a 1.ª acção pré-seleccionada: o que o VS Code e o Godot FAZEM

- **Data:** 2026-10-18 · `loadavg` na corrida exploratória `19.19 15.41 10.93`; nas corridas do script `2.30 6.22 8.51` e `7.34 6.57 8.26` (a cor de um píxel não depende da carga; a temporização dos balões sim, e cada espera é ≥ 2× o atraso do produto).
- **Versões:** `code --version` = `1.142.0 6d62b1d5dace93a33b3a6d3eca1317bd5e15bcb1 x64` (pacote `visual-studio-code-bin 1.142.0-1`, binário Microsoft, núcleo Code-OSS MIT; só CORRIDO) · `godot --version` = `4.7.2.stable.arch_linux.ed1daf0bf` (`pacman -Qi godot`: 4.7.2-2.1, **MIT**).
- **Ecrã:** `Xvfb :117 -screen 0 1600x1000x24` privado; janela em (0,0) a 1600×1000; fotos `import -window root`; rato e teclas `xdotool` (o XTest funciona no Xvfb — na Xwayland de um `kwin --virtual` não). Nada correu no ecrã do dono; nenhum `spectacle`.
- **Isolamento do VS Code:** `env -i`, `HOME`, `--user-data-dir`, `--extensions-dir` e `XDG_RUNTIME_DIR` próprios; nenhuma extensão; `chat.disableAIFeatures: true` (sem isto o menu de acções ganha entradas da IA, que não são o objecto da medição); tema de fábrica do perfil novo (nenhum `workbench.colorTheme`). Godot: `HOME`/`XDG_*` próprios, `--rendering-driver opengl3` (llvmpipe), projecto temporário.
- **Re-correr:** `bash docs/CodeEditor/ferramentas/oraculo_i8b/corre.sh [saída]` — grava as fotos e [`I8b/amostras.txt`](I8b/amostras.txt) (cada cor abaixo vem de lá, com a coordenada no ecrã 1600×1000).
- **Entrada NOSSA:** `typo.ts` = `const amplitude = 1;\nconsole.log(amplitud);` (uma correcção) · `varios.ts` = `class Ponto { x = 0; }\nconst p = new Ponto();\np.y = 1;` (várias) · `err.gd` = `extends Node / func _ready(): / pirnt(1) / var amplitude = 1 / print(amplitud)`.

## 1. VS Code — o balão sobre a palavra sublinhada

Foto: [`I8b/v1_balao.png`](I8b/v1_balao.png) (rato em (557,117) sobre `amplitud`, 2 s).

- O balão tem **quatro faixas**: (1) a mensagem `Cannot find name 'amplitud'. Did you mean 'amplitude'?` seguida da fonte `ts(2552)` em cinza mais apagado; (2) a informação relacionada `typo.ts(1, 7): 'amplitude' is declared here.` (a parte `typo.ts(1, 7):` é ligação, azul); (3) separador e o tipo `any` (o balão de tipo do TS, empilhado no mesmo balão); (4) **uma faixa de rodapé** com duas ligações: `View Problem (Alt+F8)` e `Quick Fix... (Ctrl+.)`.
- ⛔ **As correcções NÃO estão no balão.** Nem com uma única correcção disponível (`typo.ts`, onde o menu só tem `Change spelling to 'amplitude'`) nem com várias (`varios.ts`, [`I8b/v8_balao_varios.png`](I8b/v8_balao_varios.png): `Property 'y' does not exist on type 'Ponto'. ts(2339)` + `any` + o MESMO rodapé). O «Did you mean» vive só no texto da mensagem; a acção é uma ligação genérica `Quick Fix...`.
- Cores: fundo do balão `#202122` (1000,140) · rodapé com fundo próprio `#262829` (1000,218) · separador `#252627` (1000,180) · borda 1 px `#272829` (524,190) · fundo do editor `#121314` (800,400) · texto da mensagem `#BFBFBF` · fonte `ts(2552)` `#7E7F7F` · ligações do rodapé ≈ `#48A0C7` (pico do histograma; o texto é anti-serrilhado).
- Sobre a ligação `Quick Fix...` ([`I8b/v4_sobre_link.png`](I8b/v4_sobre_link.png)): aparece uma dica `Quick Fix... (Ctrl+.)` por baixo; a ligação não muda de cor visivelmente.
- **Sarjeta** ([`I8b/v2_sarjeta.png`](I8b/v2_sarjeta.png)): o rato na sarjeta da linha 2 mostra só o ponto vermelho apagado de «pôr breakpoint»; nenhum balão.
- **Lâmpada** ([`I8b/v3_lampada.png`](I8b/v3_lampada.png)): só aparece com o CURSOR na linha do erro (não com o rato); como a linha começa na coluna 0, ela é desenhada uma linha abaixo, no início do texto, com o ícone de «correcção preferida» (lâmpada azul com estrela). Rato sobre ela, 3,5 s: dica de texto `Show Code Actions. Preferred Quick Fix Available (Shift+Alt+.)` (fundo `#000000`, borda e texto `#FFFFFF`). Também sem lista de correcções.

## 2. VS Code — clicar `Quick Fix...` dentro do balão

Foto: [`I8b/v5_menu_do_balao.png`](I8b/v5_menu_do_balao.png) (1,5 s depois do clique).

- **Não aplica directamente:** abre o MENU de acções (o mesmo do Ctrl+.), ancorado logo abaixo do balão: cabeçalho `Quick Fix` e a linha `💡 Change spelling to 'amplitude'`.
- ⛔ **O balão NÃO fecha** ao clicar: `#262829` continua no rodapé (1000,218) com o menu aberto, e assim ficou 3,5 s depois e com o rato já sobre a linha do menu (dica `Enter to Apply, Ctrl+Enter to Preview`).
- `Esc` 1× fecha o menu e o balão FICA (1000,218 = `#262829`); `Esc` 2× fecha o balão (`#121314`).
- **Aplicar** (clique na linha do menu): o texto passa a `amplitude`, e o menu E o balão fecham juntos ([`I8b/v7_aplicado.png`](I8b/v7_aplicado.png); (1000,218) = `#121314`).
- A 1.ª linha do menu, **aberto pelo RATO**: enchimento `#313233` sobre o fundo do menu `#202122`, cantos arredondados, **sem anel** (a borda interna (683,262) é `#313233`, igual ao enchimento).

## 3. VS Code — Ctrl+. pelo TECLADO e o rato sobre outra linha

Fotos: [`I8b/v9_ctrl_ponto.png`](I8b/v9_ctrl_ponto.png) · [`I8b/v10_rato_na_2a.png`](I8b/v10_rato_na_2a.png) · [`I8b/v11_seta_abaixo.png`](I8b/v11_seta_abaixo.png).

- Menu de `varios.ts`: `Quick Fix` → `Declare property 'y'`, `Add index signature for property 'y'`; `Move` → `Move to file`, `Move to a new file`; rodapé `Learn more about JS/TS refactorings` (ligação). Cada correcção leva a lâmpada amarela à esquerda.
- **1.ª linha pré-seleccionada (aberto pelo teclado) = TINTA + ANEL:** enchimento `#313233` (650,187), linha de 24 px (y 175–198), x 445–714; **anel de 1 px `#3994BC`** desenhado 1 px PARA DENTRO do enchimento (topo y 176 e base y 197 em x 650; lados x 446 e 713 em y 187). Fundo do menu `#202122`; borda do menu `#2A2B2C`. **O texto não muda de cor** (`#BFBFBF` na seleccionada e nas outras). Não é um bloco sólido de cor de destaque: o azul é só a linha de 1 px.
- **Rato sobre a 2.ª linha:** a selecção MUDA para ela — a 2.ª fica com a tinta `#313233` e **sem anel** (borda (446,211) = `#313233`); a 1.ª perde tudo (`#202122`). Há um só estado «seleccionado»; o anel diz que ele chegou pelo teclado.
- **Rato sai do menu:** a 2.ª continua seleccionada (tinta, sem anel).
- **`↓`:** a selecção salta para `Move to file` com tinta + anel `#3994BC` de novo (650,260 = `#313233`; 446,260 = `#3994BC`).

## 4. Godot 4.7.2 — o erro no editor de scripts

Fotos: [`I8b/g2_palavra_pirnt.png`](I8b/g2_palavra_pirnt.png) · [`I8b/g7_lista_erros.png`](I8b/g7_lista_erros.png) · [`I8b/g8_menu_direito.png`](I8b/g8_menu_direito.png) · controlo [`I8b/g1_balao_print.png`](I8b/g1_balao_print.png).

- O erro **não sublinha a palavra**: pinta a LINHA inteira — erro `#5F3632` (linhas 4 e 6), aviso `#36342E` (linha 5, `amplitude` não usada) sobre o fundo `#1A1A1A`. A sarjeta mostra o marcador de execução de linha (`»|`), igual em todas.
- **Rato sobre a palavra `pirnt`, sobre `amplitud`, sobre o marcador da sarjeta e sobre o número da linha (3 s cada): NADA aparece.** Controlo (o rato chega ao Godot): sobre `print`, o mesmo rato abre o balão de DOCUMENTAÇÃO do símbolo (`Method @GlobalScope.print(...)` + descrição + exemplo), e sobre `Node` o da classe. ⚠️ Os balões só aparecem depois de o editor ter foco (um clique no texto); antes do clique nenhum aparecia.
- A mensagem vive na barra de baixo do editor: `Error at (4, 5): Function "pirnt()" not found in base self.` (vermelho) e os contadores `⊗ 2  ⚠ 1`. Rato sobre o `2`: dica `Errors`. Clique: abre a lista `Line 4: Function "pirnt()" not found in base self.` / `Line 6: Identifier "amplitud" not declared in the current scope.` — `Line N:` é ligação (vai à linha); **nenhum botão de correcção, nenhum «did you mean»**.
- Menu do botão direito sobre `pirnt`: `Undo · Redo · Cut · Copy · Paste · Select All · Indent · Unindent · Toggle Comment · Toggle Bookmark` — nenhuma acção de correcção. **Não observámos acções de código (code actions / quick fix) em parte nenhuma do Godot 4.7.2.**

## 5. Godot — a lista de autocompletar

Fotos: [`I8b/g9_completar.png`](I8b/g9_completar.png) · [`I8b/g10_rato_na_3a.png`](I8b/g10_rato_na_3a.png) · [`I8b/g11_seta_abaixo.png`](I8b/g11_seta_abaixo.png) (`pri` + Ctrl+Espaço).

- **1.ª linha pré-seleccionada = TINTA SÓLIDA SUBTIL, sem anel:** `#353535` (880,307) sobre o fundo do popup `#252525`, linha de 25 px (y 295–319) a toda a largura (x 636–1061), sem borda nem anel (o topo (880,300) e a borda esquerda (636,312) são `#353535`). Texto `#CCCCCC` na seleccionada, `#C9C9C9` nas outras (mesma cor na prática). O popup não tem borda; fundo do editor `#1A1A1A`.
- **Rato sobre a 3.ª linha: nada muda** — a diferença de píxeis da lista antes/depois é **0** (AE=0; o cursor de texto, que pisca, fica fora da comparação). A 1.ª continua `#353535`. O Godot não desenha estado de rato-por-cima.
- **`↓`:** a tinta `#353535` passa para a 2.ª linha; a 1.ª volta a `#252525`.

## O que isto decide para o I8b

| Decisão | VS Code 1.142 | Godot 4.7.2 | Consequência para nós |
|---|---|---|---|
| **(a)** Cada correcção como controlo PRÓPRIO dentro do balão? | **Não.** O balão tem uma ligação genérica `Quick Fix... (Ctrl+.)` (e `View Problem (Alt+F8)`) num rodapé com fundo próprio `#262829`; mesmo com uma só correcção, o nome dela não aparece no balão. | **Não.** O erro não tem balão (linha pintada; mensagem na barra de baixo e na lista de erros, sem correcções). | Nenhum dos dois oráculos faz o que o dono pediu — botões por correcção no balão são NOSSOS. O que se aproveita do VS Code: o rodapé com fundo distinto do corpo do balão como lugar das acções, e o atalho escrito no rótulo (`(Ctrl+.)`). |
| **(b)** Clicar fecha o balão? | Clicar `Quick Fix...` **não fecha**: abre o menu por baixo e o balão fica; 1.º `Esc` fecha o menu, 2.º `Esc` o balão. **Aplicar** uma correcção (no menu) fecha os dois. | — (não há balão de erro) | Botão que APLICA fecha o balão (é o fim do gesto no VS Code: aplicar ⇒ menu e balão fecham). Não há oráculo para «botão no balão que aplica directo» — o VS Code nunca aplica a partir do balão. |
| **(c)** Como se desenha a 1.ª acção pré-seleccionada; rato × teclado | Tinta `#313233` sobre `#202122` (+17 níveis, cinza neutro). **Teclado:** + anel 1 px `#3994BC` 1 px para dentro. **Rato:** a selecção segue o rato, só tinta, sem anel. Menu aberto pelo rato: 1.ª já seleccionada só com tinta. Texto não muda de cor. | Tinta `#353535` sobre `#252525` (+16 níveis), sem anel, a toda a largura; o rato não desenha nada; só `↓`/`↑` mudam a selecção. | **Nenhum dos dois usa bloco sólido de cor de destaque** — o defeito que o dono viu («parece já premido») é nosso. Os dois usam uma tinta NEUTRA ~+16 níveis acima do fundo da lista e mantêm a cor do texto; a cor de destaque, quando existe (VS Code), é só um anel de 1 px que marca a origem TECLADO. |

## O que NÃO foi medido

- O VS Code com as funções de IA ligadas (o menu ganha entradas «Fix with …»): desligadas de propósito.
- O tema claro de qualquer dos dois; as cores acima são as do tema escuro de fábrica.
- ⚠️ Na 1.ª tentativa (corrida exploratória) o VS Code isolado arrancou ~10 s com o `HOME` do dono e abriu `~/.vscode-shared/sharedStorage/state.vscdb` (modificado às 08:53:54); a partir daí o `HOME` também é isolado e o `corre.sh` já nasce assim.
