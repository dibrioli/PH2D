# 13 — A Bancada: o editor de scripts que sabe em quem roda (plano, 08/11/2026)

> Origem: pedido do dono 08/11 — *«estude a crítica (`GameEngineSamples/critica_ph2d.html#editor_scripts`),
> trabalhe a ideia, melhore; funcional como o Godot, mas belo e original como o editor de Collision
> Layers da linha GameEngineFuncional02. Planeje.»* Linha: `line/ScriptEditor` (continua o CodeEditor).
> Estado: **PLANO APROVADO 08/11** (as quatro decisões do §6, todas na recomendação). Próximo: Onda A.

## 1. A tese

A crítica fecha na maquete **B** («o molde do Godot, ligado à cena»): o editor já é forte; ele só **não sabe em quem o
script roda**. Nenhuma das 9 engines estudadas mostra, dentro do código, o valor que cada objecto deu a uma
propriedade. É esse o lugar original — e o editor de Collision Layers da GEF02 já ensinou ao dono o idioma que o
resolve: **uma grelha «quem × o quê» com nomes nas duas bordas, a faixa acesa sob o rato, a cor própria de cada
coisa, a lista à esquerda com contagem, e a FRASE fixa no pé que diz o que o próximo clique faz.**

A Bancada = o molde B + esse idioma, aplicado ao script:

```
┌ Scripts ──────┬ player.luau ● ─────────────────────────────────── 2 objects ┐
│ ■ player   2  │  1 ◆ ph2d.property("speed", 220, {min=0,max=600})  ◉◉       │
│ ■ coin     5  │  2 ◆ ph2d.property("lives", 3)                     ◉        │
│ ■ door  unused│  4   function init(self) …                                  │
│ + New script  │  8   function update(self, dt) …                            │
├ In this script│ 15 ⚡ function on_signal(self, signal, from)  ← coin         │
│ Properties    ├ Who has what · 2 objects × 2 properties ─────────────────────┤
│  speed  ln 1  │            speed     lives                                  │
│  lives  ln 2  │  Hero      220       3                                      │
│ Functions     │  Hero 2   [180]      3        ← célula «changed» acesa      │
│  init / update├──────────────────────────────────────────────────────────────┤
│ Hears  coin   │ Hero 2 · speed is 180 here (220 in the script). Click to edit.│  ← frase fixa
└───────────────┴ Line 11 · sped is not a property — on Hero, Hero 2 · ×2/frame ┘
```

Três peças são **novas no mundo** (nenhuma das 9 as tem, nas fontes lidas): a grelha **Who has what**
(objectos × propriedades, editável no lugar), as **marcas da margem** que respondem «em quem?» com os valores, e o
**problema único** (texto + Play) com os nomes dos objectos e «×N». O resto é o Godot bem feito.

## 2. A linguagem visual (herdada da GEF02, não reinventada)

| Regra da aba Collision Layers (GEF02) | Na Bancada |
|---|---|
| Cor fluorescente única por camada (`ColorToken::layer(n)`, 32) | Cor única por **objecto** que usa o script — a mesma no chip da grelha, na marca da margem, no prefixo da linha do Console, e no contorno do objecto no canvas ao pairar. ⚠️ ver §6, decisão D2 |
| Grelha com faixas acesas e nomes nas duas bordas | **Who has what**: linhas = objectos, colunas = propriedades; a coluna da propriedade sob o cursor do texto acende |
| Lista «Layers» com contagem e «+ Add» | Lista **Scripts** com «2» objectos / «unused», e «+ New script» |
| Frase fixa no pé em negrito | A **frase do pé**: diz o que o rato está a apontar e o que o clique faz; e, se há problema, o problema em frase |
| Dicas que usam os nomes actuais | Idem: «Hero 2 · speed is 180 here…» |
| Sem duplo clique; botões explícitos (dono 08/11) | Idem: «Select Hero 2», «Edit», «Make it a property» — nada escondido em duplo clique nem em Ctrl/Alt mudos |

⚠️ Os tokens das camadas e o `paint_layer_outline` vivem **só** na `line/GameEngineFuncional02` (130 commits à
frente do `main` em 08/11). A Onda B começa **depois** de a GEF02 integrar (memória: «GEF02 integra à noite»), por
`git rebase main` — nunca copiando o código dela (seriam duas cópias do mesmo token).

## 3. O que já existe (mapa medido, 08/11)

| Peça | Estado | Onde |
|---|---|---|
| Quantos objectos usam um script | existe, não pintado | `ph2d-script/src/host.rs:355` `objects_using` |
| Propriedades / símbolos / referências do script | existe | `ph2d-code-syntax/src/facts.rs:73,82,105,266` |
| Linha do Console com o DONO (`entity`) | existe; sem hora | `ph2d-script/src/console.rs:71,116,207-245` |
| Problemas do texto (`problems` + `lint`) | existe, separado do Play | `ph2d-app-code/src/doc.rs:254-285`; `ph2d-panel-code/src/status.rs:216`, `problem_list.rs` |
| Comandos (tabela única → menus do app + Ctrl+K) | existe | `ph2d-app-code/src/commands.rs` |
| Dobra (pílulas na margem) | existe, sem atalho | `ph2d-panel-code/src/paint.rs:401`, `folds.rs` |
| Soltar da Hierarchy (nome / Ctrl variável / Alt campo) | existe, teclas mudas | `ph2d-app-code/src/drop_doc.rs`, `ph2d-code-syntax/src/dropping.rs` |
| Strict / Format on save / Type hints | existem como comandos-frase | `ph2d-app-code/src/strict.rs`, `format.rs`, `intel.rs` |
| `luau-lsp` | achado ao lado do exe ou no cache; **não vem no pacote** | `ph2d-code-lsp/src/resolve.rs:76-100`, `scripts/luau-lsp-fetch.sh` |
| Comentar linha (Ctrl+/), outline, popover «em quem», grelha, menu da aba, botão direito, «número → propriedade», depurador | **faltam** | — |
| Gancho de depuração no Luau | **falta**; `mlua` 0.12 `luau` sem `set_hook` | `ph2d-script/src/host.rs:53` |

## 4. As ondas

Cada onda fecha com smoke do dono (`/pd-onda-fechar`), cena nova no roteador (`ph2d-app-code/src/smoke.rs`,
`CENAS` 10 → +1 por onda) e fotos lado a lado com a maquete (memória 07/11).

### Onda A — o que é barato e já dói (não depende da GEF02)
1. **Um problema, um lugar.** O erro do Play e o aviso do texto que falam da MESMA linha fundem-se numa linha:
   `Line 11 · sped is not a property of this script. Did you mean speed? — on Hero, Hero 2 · ×2 a frame`. O
   erro do Play agrupa-se por (script, linha, mensagem) com o conjunto de entidades e a contagem; a frase do Luau
   («attempt to perform arithmetic on nil») passa pelo `ph2d-code-explain` como a do texto já passa. **Um
   contador só** na linha de estado (texto + Play). Gate: dirigir 3 entidades com o mesmo erro → 1 linha, 3 nomes.
2. **Print com o objecto e a hora do jogo** (`coins: 3   Hero · 0:12`): `ConsoleLine` ganha o tempo de jogo.
3. **Ctrl+/** comenta/descomenta a linha ou a selecção (um passo de undo); **Ctrl+Shift+[ / ]** dobra/desdobra.
4. **Lista de scripts com contagem** («2», «unused»), pelo `objects_using`.
5. **O assistente vem junto**: `luau-lsp` (e o StyLua, decisão 6) no pacote do app, ao lado do executável — o
   `resolve.rs` já o procura lá primeiro. Sem ele: a linha de estado diz o que se perde e tem o botão «Install».
   (Decisões 2 e 6 do dono.)

### Onda B — a Bancada (depois da GEF02 integrar)
6. **In this script** (abaixo da lista): Properties · Functions · Hears (sinais que o `on_signal` compara), com
   `ln N`; clique salta. Fonte: `facts.rs`.
7. **Marcas na margem**: ◆ na linha de cada `ph2d.property`, ⚡ na função de sinal; pontos com a cor de cada objecto
   que dá um valor diferente do padrão. Pairar/clicar abre o **popover «em quem»**: `Hero 220 · default`,
   `Hero 2 180 · changed`, botão «Select Hero 2».
8. **Who has what** — a grelha objectos × propriedades no pé do código (dobrável, como o cartão «Player · 1
   object» da aba de camadas): célula alterada acesa na cor do objecto; clicar edita o valor (o mesmo comando do
   cartão Script do Inspector, logo o mesmo undo); a coluna da propriedade sob o cursor acende.
9. **A frase do pé**, fixa, em negrito: descreve o que está sob o rato e o que o clique faz.
10. **Pairar um objecto** (na grelha, no popover ou no Console) acende o contorno dele no canvas.

### Onda C — comandos e ensino
11. **Menu da própria aba**: File · Edit · Find · Go To · Run (~24 entradas, contra 58 do Godot); os comandos do
    código saem do File/Edit geral do app. «Restart Code Assistant» no fim, com explicação. ⚠️ D3.
12. **Script options** — cartão com alfinete, três caixas (Strict types · Format on save · Type hints) e o estado do
    assistente; os comandos-frase que trocam de texto morrem.
13. **Ctrl+K acha funções do script** e mostra o atalho de cada comando; em cima do código, o **caminho até o
    cursor** clicável (`player.luau › update › line 11`).
14. **Botão direito** no texto: os consertos da linha primeiro, depois «Help», «Find References», «Comment Line».
15. **Soltar com escolha à vista**: ao soltar da Hierarchy aparece «Name · Variable · Inspector field»; Ctrl/Alt
    continuam a saltar a pergunta.
16. **«Make 220 a property»** — conserto novo no Alt+Enter sobre um número: escreve `ph2d.property("speed", 220)`
    no topo e `self.speed` no lugar; o campo aparece no Inspector.

### Onda D — depurar dentro (só com medição antes)
17. **Ponto de parada** (F9 na margem), **Stopped at line 17** com as variáveis, Continue / Next line;
    **Pause on Error**.
18. **Valor no fim da linha** durante o Play (`x = 412`, `printed 3×`).
**Kill-criterion, escrito antes de construir:** o gancho de linha do Luau custa ≤ 2 % do quadro de 60 Hz numa
cena de 50 objectos com script, medido com o gancho armado e desarmado no MESMO processo, intercalado
(MEDIR_VELOCIDADE.md). Acima disso a D17 fica só no «Pause on Error» (gancho apenas no erro) e a D18 não existe
nesta forma. 1.º passo da onda: confirmar que o `mlua`/Luau expõe o gancho (o Luau tem `interrupt` e `debugstep`,
não o `lua_sethook` do Lua 5).

## 5. Não fazer (decidido aqui)
- Minimapa: nenhuma das 9 o tem como peça de engine que importe ao iniciante; custa largura da aba.
- Janela separada do editor: a aba já é central; reabrir isto é outra conversa.
- Copiar o código da aba de camadas para cá: reusa-se por `main` depois da GEF02 (§2).

## 6. Decisões do dono (08/11 — respostas: D1 **Bancada** · D2 **cor por objecto** · D3 **menu da própria aba** · D5 **sim, sob o kill-criterion**)
- **D1** — A Bancada (B + grelha «Who has what» + frase do pé) é o rumo?
- **D2** — Cor por **objecto** (cada objecto que usa o script tem a sua cor, como cada camada tem a sua) ou cor
  por **script**?
- **D3** — Menu **da própria aba** (Godot) ou continuar nos menus gerais do app + Ctrl+K?
- ~~D4~~ — o assistente dentro do app **já foi decidido** (decisões 2 e 6, 15–16/10: «Sim»); a Onda A só cumpre.
- **D5** — A Onda D (ponto de parada) entra no plano, sob o kill-criterion do §4?
