# HANDOFF DE INTEGRAÇÃO — `line/Painter`, 2026-08-09

**Status:** FECHADO 2026-08-09 · no `main` em `0e93f9e4b` (o commit que trouxe este arquivo).

> **O CARIMBO EM GRADE, A PILHA QUE VOLTOU A PINTAR, E A SHAPE QUE SAI COZIDA**
>
> | | |
> |---|---|
> | Branch | `line/Painter` |
> | Worktree | `Worktrees/line-Painter` |
> | Base (`merge-base` com `main`) | `17a0f6d6d` |
> | Tip | `c071b1201` |
> | Commits | **26** |
> | Diff cumulativo | **77 arquivos, +6.362 / −339** |
> | Smokes | **APROVADOS pelo Enio** (2026-08-09) |

---

## §1 — O que a jornada entrega

Cinco blocos, todos nascidos de report do Enio com foto.

### (A) A regressão do Composite Brush — a pilha voltou a pintar (7 commits)

*"não consegue pintar mais que uma mancha"*, com listras retangulares. **Ablacionada
antes de qualquer hipótese**: 141 colunas entintadas sem composite (controle) · 108 com
Brush+Smear · **141 com Brush+Blur** ⇒ o Blur inocente, o Smear reproduzindo sozinho; e
removendo o *restore* da sessão de smear o traço volta a 141.

⚠️ **A causa são DOIS tempos de vida que estão certos sozinhos.** Desde a wave do campo,
uma esfregada acumula um mapa de deslocamento e resolve **uma vez** a partir dos pixels
congelados no pen-down — a lei que matou o filamento. O composite promete o oposto (*cada
operação processa o canvas como a de baixo o deixou*), e isso é **por BATCH**. O render de
smear do batch seguinte reescrevia a região a partir de uma base que nunca tinha visto o
Brush.

A cura é a que o próprio card promete — *pinta, depois esfrega o que pintou* — e mantém a
lei do campo intacta. ⚠️ **E ela passa pela PORTA de quem deposita**, não por uma
reconstrução de fora: três dobras foram construídas e medidas antes disso, cada uma com o
seu número (copiar a REGIÃO ⇒ 141 mas com escada + estrias · SOMAR o delta ⇒ 131, perde
tinta · recuperar a alfa de `after = before(1−a) + C·a` ⇒ **108, zero em toda parte** — a
cor e o espaço com que o depósito compõe não são `brush.color` em sRGB de 8 bits). *Só quem
deposita sabe `(C, a)` por texel.*

Três defeitos de display fecharam em seguida, cada um com o mecanismo nomeado pelo próprio
report:

- **a escada axis-aligned** — com a fonte mutável, um texel com `disp ≠ 0` **fora** do
  batch atual seguia mostrando o render de uma fonte que não existe mais; a sessão passou a
  carregar `touched_all`;
- **a borda inferior** (*"por que só na borda inferior?"*) — o render passou a cobrir `all`
  e o **dirty** continuou marcando só `rect`. A assimetria cai fora por geometria: `all` só
  excede `rect` do lado para onde o traço já andou;
- **a sessão de smear da pilha NUNCA era encerrada** (BUGS #22) — `end_smear_session`
  perguntava `paint_mode.smears()` (Smear ou Knife) e a camada Smear do Composite roda em
  `PaintMode::Paint`. ⚠️ O doc do próprio `smears()` avisa que **enumerar os sítios é o que
  apodrece quando entra um membro novo na família** — a pilha é o terceiro.

⚠️ **E duas sondas deram NEGATIVO, o que é achado e está registrado:** a costura vertical
**não reproduz** na fixture (a foto mostra uma coluna de altura inteira, não a bbox de um
dab ⇒ o suspeito é o pipeline de DISPLAY, não a operação), e **a borda dura NÃO é da
pilha** — o brush digital sozinho endurece igual (rampa 90→10% no ombro: **7 texels numa
passada · 3 em cinco · 2 em quinze**, idêntico com e sem Smear/Blur). É o defeito ABERTO
que a §13.10 já mede.

### (B) Os quatro "Use as ..." leem a APARÊNCIA (2 commits)

*"Use as Brush Shape não transfere para o brush os relevos criados por Impasto"* — e os
quatro (Shape / Grain / Paper / Granulation) passavam por **duas** portas do tool e
**nenhuma** iluminava. O enquadramento que decide: o **BAKE responde a MESMA pergunta** — *com
o que este documento se parece?* — e o doc-comment dele já dizia por que ilumina.

⚠️ **A segunda metade cobria só metade das rotas:** o modo **Per-Layer Color** não passa
pelo flatten (carimba pelas máscaras CRUAS), então o relevo alcançava o pincel exatamente
enquanto o artista não ligasse *o modo que pinta com as cores da textura* — que é o modo que
o report pedia.

### (C) O GRID STAMP (8 commits) — a feature nova da jornada

O carimbo preso a uma **grade própria**: motor + entrada no menu de método · os controles ·
as rows do painel · **botão direito apaga a célula** · a grade desenhada na tela · o
encaixe/fit e a escolha da silhueta.

⚠️ **O `+1` que atravessou três rodadas de report** foi medido pela porta do artista antes
de qualquer código: numa célula de 40 px o carimbo pintava **41 colunas, todas cheias, com
a sobra inteira à direita e abaixo** — e o mesmo `+1` numa célula de 41 e numa de 32.
**A assimetria é o diagnóstico:** um raio errado erraria dos DOIS lados. E a cura final é a
lição desta jornada inteira:

> o motor tem **CINCO amostradores** do mesmo stamp cacheado, cada um com a aritmética
> repetida inline. O corte *"fora do quadrado unitário não há dab"* foi escrito em **UM**.
> Os outros quatro continuaram.

E a imagem **parava de extrapolar a célula** porque o raio compensava a cauda do falloff
(a tinta macia acaba em `t ≈ 0,61`) e compensava **também** uma silhueta de Shape, que acaba
no aro geométrico: a imagem saía **1,64× a célula**. O `grid_stamp_frame` passou a perguntar
a MESMA porta do aro do bow wave (`height_push::rim_t0`), que já respondia *onde a tinta
deste dab acaba*.

### (D) A Shape sai COZIDA, sem relevo real (3 commits — o segundo REVERTE o primeiro)

⚠️ **Construído, smokado, REPROVADO e retirado dentro da própria linha.** A Shape passou a
carregar o RELEVO (o carimbo reagindo à luz do documento); o Enio reprovou — *"não ficou
bom, embora funcione"* — e pediu o oposto: **uma versão COZIDA, bake perfeito, sem relevo
real**. O relevo vivo saiu inteiro (**843 linhas**, incluindo `impasto_gain.rs`).

⚠️ **E "cozido" virou um número, não um adjetivo.** A versão anterior guardava a cor **crua**
e um ganho de **luminância**; medido contra o que a tela mostra, o erro era de **96 níveis
de 255** com material neutro e **98** com cera âmbar — um escalar não tem onde guardar a
*cor* do especular nem a da cera. Agora a captura roda o **passe canônico da luz** (o mesmo
da tela) sobre os pixels de cada camada.

⚠️ **A aproximação fica nomeada:** a luz é a do documento (a pilha DOBRADA). Para a camada
ativa isso é exatamente a tela; para uma de baixo é a sombra que se vê *através* da pilha —
a mesma escolha que o ganho já fazia.

Mais dois fechamentos do mesmo dia: o **carimbo fica como o artista o largou** (o
*settle* do pen-up é pulado no Digital : Stroke : Grid Stamp — medido: canvas byte-idêntico,
relevo movendo 1264 texels, e `smoothing = 0` tornando-o byte-idêntico ⇒ o assentamento era
a diferença inteira) e **o preview do painel volta a mostrar a escultura** em vez da imagem
chapada.

### (E) Os pendentes do `--ignored` (1 commit)

Ver **§5** — a metade que mais importa para quem integra.

---

## §2 — Superfície de colisão: **VAZIA**

Conferido por `git diff`, não por auto-relato.

| Item | Estado |
|---|---|
| `PROJECT_SCHEMA` (`shells/desktop/src/project.rs`) | **diff VAZIO** |
| Contrato congelado (`ph2d-nodegraph`, `ph2d-core/src/tool.rs`) | **diff VAZIO** |
| Gates de contrato | `architecture_contract_surface` **3/3** · `architecture_tool_contract_surface` **4/4** |
| ADR novo | **nenhum** |
| `*/Cargo.toml` · `Cargo.lock` | **nenhum** — zero crate nova, zero dep nova |
| ids numéricos (scrollbar, gizmo, …) | **nenhum** — os ids novos são **hash de string** (`hash_node_id`), logo fora de qualquer contador |
| i18n · z-order · scroll map | **não tocados** |
| Registro do `ph2d-ecs` | **não tocado** |
| Docs | só `docs/Painter/BUGS_painter.md` (+101, a entrada #22) |

⇒ **Esta linha fica FORA de toda disputa de número da janela.**

---

## §3 — Os pontos sensíveis de MERGE

Três arquivos compartilhados. Nenhum é conflito garantido; os três são onde um **merge
textual limpo pode estar semanticamente quebrado**.

1. ⚠️ **`crates/ph2d-panel-painter-layers/src/populate.rs` (+167 / −133) — é uma REMOÇÃO.**
   Catorze linhas de botões de rampa foram hoistadas para um helper (`register_ramp_buttons`)
   porque o `Alpha From Image` levou o pai a **201 do teto de 200**, e uma chamada nova
   (`register_grid_stamp`) entrou. **Se outra linha acrescentou rows àquela lista, um merge
   limpo pode perdê-las** — é exatamente a classe que a wave dos fades da `line/anim`
   documentou. Quem pega: **`architecture_panel_wiring_parity`** (verde aqui) e os seams do
   painel. **Rode os dois na árvore combinada.**

2. `shells/desktop/src/render_loop/mod.rs` (+15 −…) — arquivo de **8.967 linhas** que toda
   linha toca. O toque daqui é a fiação do bridge da grade (`painter_bridge_grid`) e do
   `paint_perf`.

3. `shells/desktop/src/input_dispatch.rs` (+14) — a rota do **botão direito apaga a célula**
   (`painter_grid_erase`). ⚠️ Se o `main` tiver PARTIDO este arquivo pelo teto de LOC (o
   precedente do `keyboard.rs` → `keyboard_escapes.rs` em 08/08), o bloco funde limpo **para
   o lado errado do corte**. Confira que ele segue **antes** do picking/gizmo genérico.

⚠️ **LOC sem folga:** `crates/ph2d-painter-brush/src/stroke.rs` está em **697 / 700**. A
próxima adição ali tem de orçar o split.

---

## §4 — Gates rodados (números, não adjetivos)

| Suíte | Resultado |
|---|---|
| `ph2d-tool-painter --lib --release` | **1160 passam, 0 falham** |
| `ph2d-tool-painter --lib` (**DEBUG**) | **1158 passam, 0 falham** |
| `ph2d-painter-brush` | **325 + 1** |
| `ph2d-panel-painter-layers` | **43 + 22 + 15 + 14 + 9 + 7 + 6 + 4 + 4 + 3 + 2 + 2 + 1 + 1 + 1** |
| `ph2d-host-desktop --release` | **134 alvos, todos `ok`** |
| `architecture_workspace_file_loc_cap` | **2/2** |
| `node_id_collisions` | **7/7** |
| `architecture_panel_wiring_parity` | **1/1** |
| `cargo fmt --all -- --check` | limpo |
| `clippy --all-targets --release` (4 crates tocadas) | **0 warnings, 0 errors** |

⚠️ **Rode a suíte do Painter em DEBUG também** — precedente registrado nesta linha (o
`ph2d-flip-colorize` panicava só em debug).

### ⚠️ E a armadilha que vai custar meia hora a quem não ler isto

**Os `--ignored` desta crate TÊM de rodar `--test-threads=1`, com a máquina calma.**

Rodados em paralelo (o default), **cinco** kills de relógio dão vermelho e **nenhum é
código**. Medido nesta sessão, mesmo binário:

| | paralelo (`load 41`) | serial (`load 0,6`) |
|---|---|---|
| `smear_perf_kill_criterion` @2048²/@4096² | **11,36 / 20,04 ms/move** | **5,50 / 5,60** |
| `sculpt_perf_kill_criterion` | FAIL | ok |
| `deform_transform_perf_move…` | FAIL | **0,746 ms/frame** |
| `deform_warp_perf_drag…` | FAIL | **0,818 ms/frame** |
| `warp_perf_kill_criterion` | FAIL | ok (SMEAR 6,00 · DEFORM 0,32) |

Serial e calmo: **212 passam, 2 falham** — e os 2 são os `#[ignore]` **declarados RED**
(§5). *Nenhum número de relógio desta máquina significa nada com o `load` acima de ~5.*

---

## §5 — Os dois pendentes que a última sessão atacou

**(1) `measure_how_the_tick_scales_with_the_brush_radius` — FECHADO.** Panicava com a frase
que o próprio `EngineSlot` escreve: *"chame `sess.bring_home()` na porta que abriu este
caminho"*. Era isso — uma porta que alcança a sessão e não traz o motor de volta; o
`on_tick` do fim do laço o entrega ao worker, e da segunda volta em diante o `DerefMut`
panica nomeando o sítio. Uma linha, **antes do relógio** (esperar o worker não é custo do
passo). O irmão levou a mesma linha: hoje é no-op, mas **a regra é da PORTA, não do laço**.

**(2) `watercolor_app_params_incremental_matches_full_{diluted,mixer_on}` — SEGUEM RED, com
diagnóstico novo.** O texto do take 7 (*"ou depende dos params exatos do Enio, ou não está no
canvas CPU"*) foi **superseded** e reescrito no doc-comment do gate. Quatro sondas
`#[ignore]` novas medem:

- **não é ruído numérico** — MESMO estado, união × retângulo de 64×40 dentro dela: **Δ0 em
  2560 px**. O composite é função pura do estado sobre a região dele;
- **não é a soma-prefixo do `box_blur`** — ela *é* dependente da janela (**1,98e-4** num
  sinal fracionário), mas torná-la exata em `f64` deixa os dois gates em Δ2. Hipótese
  construída, medida e **REFUTADA**;
- **não é o `settled`** — duas ablações deixam **139 dos 152 px** de pé;
- **é RAIO DE INVALIDAÇÃO**: `pad +0 → 152 px · +64 → 38 · +128 → 1 · +2·raio → 0`. O
  resíduo escala com o pincel (**12 px a r=20 · 152 a r=80 · 361 a r=160**), vive no **aro**,
  e o termo de borda o amplifica **17×** (`edge_gain = 0` ⇒ 9 px).

⛔ **E o `pad += 2·raio` NÃO é a cura — isso é medição, não gosto:** a janela é
`dirty ⊕ 4·pad` por eixo, então num pincel de 80 px ela vira o **canvas inteiro todo
quadro** — exatamente o custo que o caminho incremental existe para evitar, e que o
`measure_the_area_a_watercolor_frame_walks` vigia. Falta a grandeza **NOMEADA** de alcance
`2·raio`; até ela aparecer o gate fica `#[ignore]`.

⚠️ **Os dois commits de sonda mexem SÓ em código de teste** — a cadeia é
`paint.rs:262 #[cfg(test)] mod tests;` → `tests.rs` → `measure_wetpaint_cost` →
`measure_wetpaint_tick` → `measure_wetpaint_probes`. As três fontes do composite
(`watercolor_render`, `_render/window`, `_field`) estão com **diff vazio**: toda ablação foi
revertida por `cp` de backup, nunca por `git checkout`.

---

## §6 — Como re-smokar (já aprovado, mas é o roteiro)

Todos `--release`.

1. **Grid Stamp** — Digital : Stroke : **Grid Stamp**. A imagem tem de **caber e centrar** na
   célula (nada de invadir a vizinha à direita/abaixo com corte à esquerda/acima), o **botão
   direito apaga** a célula, e a **grade aparece** sobre qualquer fundo.
   ⚠️ **Carimbe e SOLTE:** o desenho no *mouse up* tem de ficar **igual ao do mouse down**.
2. **A Shape cozida** — ponha uma imagem com impasto em Shape: ela sai **cozida** (a luz
   assada) e **sem relevo real** (carimbar com ela não deposita relevo).
   ⚠️ **O CONTROLE:** um documento **sem** escultura tem de sair **idêntico** ao de antes.
3. **A pilha** — Composite Brush com Smear: pintar tem de continuar pintando (não uma mancha),
   sem escada axis-aligned, sem entalhe na borda inferior, e **três traços seguidos** não
   podem escorregar (a sessão encerra no pen-up).
4. **Use as ...** — os quatro (Shape / Grain / Paper / Granulation) transferem o **relevo**,
   inclusive com **Per-Layer Color** ligado.
5. Regressão: `PH2D_IMPASTO_SMOKE=1` e `=2` · `PH2D_WETPAINT_SMOKE=1` · `PH2D_MASK_SMOKE=1`.

---

## §7 — Aberto, com o preço ao lado

- **O resíduo Δ2 do wash incremental** (§5) — falta a grandeza nomeada de alcance `2·raio`.
- **A costura vertical** do report do smear **não reproduz** na fixture; o próximo passo
  **não é código**, é a armadilha que o BUGS #11 já deixou armada (`PH2D_PREVIEW_DIAG` /
  `PH2D_PREVIEW_DUMP`), que separa composite de upload de overlay em UMA corrida.
- **O endurecimento da borda** do brush digital segue aberto e **não é desta wave** — medido,
  o controle endurece igual (7 → 3 → 2 texels em 1/5/15 passadas). As duas leis de acúmulo
  possíveis já foram tentadas e cada uma tem artefato (produto endurece · envelope dá
  contas), então a próxima hipótese **não pode ser uma terceira lei**.
- **A dobra do Brush na fonte do smear é aproximação NOMEADA:** ela dobra na posição
  **pintada**, não na pré-imagem, então uma esfregada longa arrasta tinta fresca um pouco
  mais longe do que um revezamento físico faria. Dobrar em `p − disp(p)` seria exato e é um
  **scatter**, que pode deixar buracos na fonte — o doc-comment diz onde esse trade começa.
- **Meio gate do smear na pilha NÃO existe, por fixture e não por preguiça:** *"o Smear da
  pilha ainda esfrega"* resistiu a três oráculos, cada um saturado por um mecanismo
  diferente (fora do eixo ⇒ a esfregada desloca AO LONGO do traço · alcance = a pegada do
  próprio pincel · marca pré-pintada ⇒ o Brush opaco a cobre). O risco está escrito no
  teste.
- `stroke.rs` em **697 / 700**.

---

## §8 — Ordem de integração

Nada aqui força ordem: **zero schema, zero ADR, zero `Cargo.toml`, zero id numérico**. O
único cuidado é o **§3.1** (a remoção no `populate.rs` compartilhado), que pede
`architecture_panel_wiring_parity` + os seams do painel **na árvore combinada**.
