# HANDOFF (continuação, janela nova) — `line/Vector`: OS ABERTOS, um a um (2026-10-04)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). Substitui o
> [handoff de 03/10](HANDOFF_line_Vector_CONTINUACAO_A_FRENTE_PINTA_POR_CIMA_2026-10-03.md) (o §3
> dele — a frente pinta por cima — vive aqui como o aberto **A2**).

## 0. Onde está

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector`, base `main` `1ad60a1ce`, HEAD `348d86d58`, `49` commits à frente, nenhum integrado |
| smoke | binário `--profile smoke` compilado na worktree |
| o dono | aprovou o smoke da F51 (*«smoke OK»*, 2026-10-04), mandou **resolver os abertos** e **não fechar** a linha |

O que a janela de 03–04/10 entregou está na [fila §F51](../01_a_fila.md): **o Bind COZE os efeitos e
uma forma presa não recebe efeitos** (ordem do dono — o botão «antes/depois dos ossos» foi construído,
medido e RETIRADO na mesma tarde; as medidas estão na fila para ninguém o reconstruir às cegas);
o pânico do `linesweeper` curado na porta única; a forma cozida guarda o contacto da F50 (só a
união); quatro ficheiros cortados abaixo do tecto de LOC.

## 1. ⭐⭐⭐ A LISTA VIVA DOS ABERTOS — toda janela a lê ao começar e a ACTUALIZA ao acabar

> ⛔ **Regra desta lista:** ao fim da janela, cada item fica com o estado novo (`✅ FEITO <commit>` /
> `⏳ parcial: …` / `⛔ recusado pelo dono: …`), e um aberto NOVO que a janela descobrir entra com o
> próximo número livre. O próximo handoff de continuação COPIA esta secção actualizada. *Uma lista
> que só a janela que a escreveu conhece morre com a janela.*

### A1 — ❓ PERGUNTA AO DONO (fazer PRIMEIRO): num desenho muito torcido, cada pedaço segue que osso?

- **Hoje:** os efeitos cozem no Bind, e o campo de pesos é resolvido sobre o desenho COZIDO
  (`skin_live_prender::bind_com` → `campo_do_caminho(src cozido)`) ⇒ cada pedaço segue o osso de que
  está mais PERTO no desenho (lei espacial, à *Puppet* do After Effects). Medido na F50-k (barra da
  fixtura, nós dominados por osso): *Twist* `60°` `[55, 22, 55]` · `120°` `[33, 66, 33]` · `150°`
  `[40, 53, 39]` — a ponta enrolada perto do osso do meio passa a ser dele.
- **A escolha anterior do dono** (feita ANTES de existir o Bind que coze): *«o osso de onde VEIO»* (lei
  material, à Blender). Pergunte de novo, em linguagem dele, com as duas opções lado a lado (o que
  muda só se vê num efeito forte). Use `AskUserQuestion`.
- **Se ele escolher «de onde veio»** — desenho proposto (NÃO medido): no Bind, para os efeitos que
  MOVEM pontos (*Twist*, *Warp*, *Bloat*, *Zig Zag*, *Falloff* sobre eles), resolver o campo sobre a
  forma SEM efeito e passar os VÉRTICES da malha do campo pelo efeito (com o `FxCtx` da forma) — os
  pesos ficam, a malha deforma-se ⇒ um campo material sobre o cozido, sem solver extra. ⚠️ Medir:
  dobras da malha onde o efeito sobrepõe a forma a si mesma (a leitura baricêntrica acha o 1.º
  triângulo); os geradores (*Hatch*, *Repeat*, *Sketch*, *Knot*, *Trim*) não são mapas de pontos —
  ficam com o campo do cozido; e a pilha mista. Gate sugerido: nós dominados por osso no cozido = os
  da fonte, em toda a varredura do *Twist* `0°`…`360°`.
- **Se ele escolher «o mais perto»:** feche a F50-k/§2-b na fila como decidido e siga.

### A2 — ⭐ Numa dobra muito forte, a parte de TRÁS pinta por cima da da FRENTE (o próximo trabalho)

- **O que o artista vê** (FOTOGRAFADO a `100°`–`110°` na `PH2D_VEC_BONE_SMOKE=5`): o contorno já se
  une, mas o que é ABERTO ou fica DENTRO — as riscas do *Hatch*, os dentes do *Zig Zag* — da parte de
  trás aparece por cima da parte da frente onde os membros se sobrepõem. A imagem presa resolve isto
  pela ORDEM DAS FACES (F48-c, `ordena_pelo_osso`, chave `Σwⱼ·j/Σwⱼ`).
- ⚠️ **Mudou desde 03/10:** os efeitos agora são COZIDOS no Bind ⇒ as riscas do *Hatch* são
  subcontornos ABERTOS da própria fonte (não uma pilha viva). A lei tem de servir a forma cozida
  (`SkinnedPath::efeitos_cozidos`) e, idealmente, qualquer forma com subcontornos abertos.
- **Antes de construir (CLAUDE.md §5.0):** meça se a COMPOSIÇÃO já o exprime — talvez baste recortar
  os caminhos ABERTOS da parte de trás pela região da parte da frente deformada
  (`ph2d-vec-boolean::cut`), sem partir o preenchimento.
- Desenho mais pesado (se a composição não chegar): partir a forma em pedaços por osso (cada
  triângulo da malha do campo → o osso da chave dele), desenhar por chave crescente. Riscos medidos
  noutras mídias: a COSTURA entre pedaços (F48/F49), o PREÇO (régua `diag_o_preco_do_efeito_por_quadro`,
  hoje `0,9`–`2,7 ms`), o recorte de abertos.

### A3 — ⏳ Prender uma forma com um *Repeater* de muitas cópias é LENTO

- Medido em 03/10: um *Repeater* `39 × 39` que gira (`~1 000` contornos) — em DEBUG o Bind não acabou
  em 10 min (o solver `campo_do_caminho` sobre o cozido inteiro, síncrono no Bind). O pânico do
  `linesweeper` que ele também causava está CURADO (gate `a_dense_spinning_repeater_union_answers_instead_of_panicking`).
- **Fazer:** medir em `--release` (o Bind e o 1.º quadro), com `loadavg` ao lado. Se for inaceitável
  (CLAUDE.md §0.0: o tecto diz de que RECURSO é), decidir pela medição: solver numa thread (a F50-j já
  o faz para a pilha viva), malha do campo mais grossa acima de N contornos, ou um tecto de cópias
  com a razão no painel. Nenhum número sem a tabela ao lado.

### A4 — ⏳ Projectos ANTIGOS: forma presa com efeitos vivos

- Uma forma presa ANTES de 03/10 que tenha efeitos continua a desenhá-los (a lei F50, agora em
  `skin_desenho_efeitos.rs`), mas o painel de uma forma presa só mostra a frase — os efeitos ficam
  invisíveis e não se editam (solta-se a forma para mexer). ⚠️ Viola a lei do dono «presa não tem
  efeitos» por omissão.
- **Fazer (decisão técnica, padrão-ouro):** a cura natural é COZER no carregamento — ao ler um
  projecto, uma forma presa com efeitos activos é cozida como o Bind faria (a fonte guardada é
  refeita com a geometria cozida, pesos e campo resolvidos no repouso dos tendões guardados,
  `efeitos_cozidos = true`), e a pilha esvazia. Meça que não move um pixel (gate com controlo). ⚠️ Se
  isto tocar no formato do ficheiro, conte o degrau com `python3 scripts/schema-recount.py`.

### A5 — Dois detalhes pequenos (do dono — só se ele reparar; registados para não se perderem)

- A **cúspide da ARTE** de uma imagem presa junto a uma tampa redonda (limite da F49): pede o fecho
  sobre o contorno da ARTE (pixels, na `attach_skin_meshes`). Ver fila §F49 «LIMITE CONHECIDO».
- No *Zig Zag* muito dobrado (`~110°`) os dentes de dentro encavalam-se e fecham buraquinhos REAIS (a
  imagem também os mostra).

## 2. Lições da janela de 03–04/10 (morderam)

- ⛔ **Uma régua da câmera que não é a da FOTO aprova uma cena que não cabe:** o gate da `=5` usava
  `±4,5 m` e a vista real é `±4,0` (a barra de ferramentas tapa `~0,95 m`). Fotografe SEMPRE.
- ⛔ **Simplificar uma lei apaga o que a lei antiga SABIA:** cozer os efeitos fez a barra parecer «do
  artista» e o contacto foi à bola, que comia os dentes do *Zig Zag* — só a FOTO o viu. O facto passou
  a viajar na fonte (`SkinnedPath::efeitos_cozidos`).
- ⛔ **Gates que leem um ficheiro pelo CAMINHO** (`include_str!`) partem-se em espécies mudas quando o
  código muda de ficheiro: o controlo positivo do `skin_bake_tests` apanhou o `bind` mudado para
  `skin_live_prender.rs`; os gates de ausência podem passar em vazio — leia-os depois de um corte.
- ⛔ **`pkill -f <padrão>` mata o próprio shell** (o padrão está na linha de comando dele): mate por PID.
- ⛔ **Mensagem de commit com apóstrofos dentro de `bash -c '…'`** parte o comando: use `-F <ficheiro>`.
- ⭐ **Os gates de ARQUITECTURA (`ph2d-editor-core --test it architecture`) não corriam na linha** —
  apanharam 4 ficheiros acima de 700 linhas e um leitor fora do censo da malha posada. Corra-os ao
  fim de cada onda, não só no fecho.

## 3. Ao fechar a linha (SÓ quando o dono mandar)

Gate batched 1× (`BASE=1ad60a1ce bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh`) — ⚠️ os
testes da SHELL correram só em parte (o `project_schema`); `ph2d-vec-skin`, `ph2d-vec-boolean`,
`ph2d-vec-scene` e `ph2d-skeleton-live` mudaram e a shell consome-os. Clippy `--all-targets`;
`CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`; `cargo machete`;
`bash scripts/agent-loop-profile.sh`; `rm -rf target/*/incremental`; handoff de INTEGRAÇÃO (F48
retirada + F49 + ordem das faces + F50 + F51 — o `bind` agora recebe `&mut VecScene`, os chamadores de
outras linhas que prendam formas têm de acompanhar); trocar o link da linha Vector no `CLAUDE.md`
§5.1; e por ÚLTIMO o binário de smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2×, a 2.ª
saída colada no handoff).
