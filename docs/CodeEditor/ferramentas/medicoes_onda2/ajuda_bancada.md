# A página de ajuda na bancada (20/10/2026)

> Os números que o [plano da página de ajuda §7](../../04_pagina_de_ajuda.md#7-cena-de-smoke-5-ph2d_code_smoke5-ajudaluau)
> pede antes da mensagem ao dono. Sonda: `crates/ph2d-app-code/tests/it/bancada.rs::ajuda_bancada` (`#[ignore]`).
> Variantes no mesmo processo, intercaladas em blocos rodados; resumo = mínimo e mediana
> (`docs/DevOps/MEDIR_VELOCIDADE.md`). Perfil `smoke`.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it ajuda_bancada -- --ignored --nocapture
```

**loadavg:** `1.61 3.17 3.15` no início · `1.56 3.14 3.14` no fim · placa NVIDIA GeForce RTX 5060 Ti · área da aba
1600 × 862 px (a vista de 43 linhas do editor, a mesma das outras células da bancada).

## 1. Do Ctrl+clique à página pintada no membro

Quadros a 60 Hz (cada quadro: `frame` da família + pintura do script e da aba activa, depois espera até 16,7 ms).
Conta-se do clique até ao quadro em que a página é a aba activa E pintou o membro no topo. Quatro alvos
alternados (`math.sin`, `ph2d.set`, `m.cos` (o nome alternativo), `string.format`). Entre amostras: Ctrl+W
de volta ao script e a revisão do texto avança (uma letra e o Backspace), para que nenhuma resposta antiga
sirva. «Depois do Ctrl+rato» = o caminho da cena 5: a mão já sabe para onde o nome leva antes do clique.

| Ctrl+clique → página pintada (60 Hz) | N | quadros mín | quadros mediana | quadros máx | ms mín | ms mediana |
|---|---|---|---|---|---|---|
| servidor falso · clique a frio | 32 | 2 | 2 | 2 | 20,6 | 28,5 |
| servidor falso · depois do Ctrl+rato | 32 | 1 | 1 | 1 | 3,6 | 11,4 |
| `luau-lsp` 1.70.1 fixado · clique a frio | 32 | 2 | 2 | 2 | 20,5 | 28,2 |
| `luau-lsp` 1.70.1 fixado · depois do Ctrl+rato | 32 | 1 | 1 | 1 | 4,0 | 11,4 |

⇒ A frio são **2 quadros**: a definição e o balão vão e voltam dentro de um quadro, e a página abre e pinta-se
no seguinte. Com a mão do Ctrl+rato o clique já sabe o destino: **1 quadro**. O servidor real não acrescenta
nenhum quadro ao falso.

## 2. A disposição da maior página

Maior página do catálogo pela altura disposta: **`math`**, com 14 611 px de altura e 21 231 bytes de texto, a
1 600 px de largura.

| disposição da maior página (µs) | N | mín | mediana |
|---|---|---|---|
| vista nova (fria): medida + disposição + pintura | 160 | 2 620 | 2 674 |
| outra largura, vista quente: disposição + pintura | 160 | 2 183 | 2 228 |
| a mesma largura (em cache): só a pintura | 160 | 100 | 101 |
| ≈ a disposição sozinha (quente − cache) | — | 2 083 | 2 127 |

⇒ A disposição custa **~2,1 ms** e só acontece quando a página abre ou a largura muda (a chave é página +
largura). Com ela guardada, um quadro da maior página pinta-se em **~0,1 ms**. ⚠️ Na vista fria, o
`TextSystem` é partilhado e já está quente: o «frio» é o da vista (medidas e cache de linhas), não o das fontes.

## 3. Um quadro: a página aberta contra o editor a 7 500 linhas

A mesma área. A célula do editor é a `Lista::Fechada` de 7 500 linhas, a mesma do I15. CPU = `paint` da aba
numa `VectorScene`; placa = `VelloPass::render_to_intermediate` + espera do dispositivo.

| um quadro (µs) | N | CPU `paint` mín | CPU mediana | placa (render + espera) mín | placa mediana |
|---|---|---|---|---|---|
| página `ph2d` aberta (perto do `set`) | 160 | 149 | 155 | 528 | 543 |
| editor a 7 500 linhas | 160 | 401 | 412 | 546 | 561 |

⇒ Com a página aberta, um quadro custa **menos** que o editor a 7 500 linhas: ~2,7× menos no CPU e o mesmo
na placa.

⚠️ **Achado da sonda:** a página abriu no `set` à largura da mesa (1 000 px). Pintada a 1 600 px, a rolagem
**em píxeis** fica e o membro do topo muda (o `set` passa a ser o 2.º: `top_member` 8 → 9). Ao mudar de
largura, a página não mantém o membro em que estava. Está por decidir se isto é defeito: o Godot não foi
medido nisto.
