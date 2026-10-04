# HANDOFF (continuação, janela nova) — `line/Vector`: A2–A4 FEITOS, o smoke do A2 e os abertos A5–A7 (2026-10-04, 2.ª onda)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). Substitui o
> [handoff de 04/10 (1.ª onda)](HANDOFF_line_Vector_CONTINUACAO_OS_ABERTOS_2026-10-04.md).

## 0. Onde está

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector`, base `main` `1ad60a1ce`, HEAD `42ca2e29f` (ou depois), `70` commits à frente, nenhum integrado |
| smoke | o binário `--profile smoke` da worktree foi construído pelo `fotografa_cena.sh` a `c5458de61`; reconstrua antes do smoke |
| o dono | escolheu no A1 «o osso mais PERTO»; aprovou o smoke do A2 e da ordem da imagem (*«smoke ok»*, 2026-10-04); mandou **não fechar** a linha |

O que a 2.ª onda de 04/10 entregou está na fila: [§F52](../01_a_fila.md) (A2 — a frente tapa as riscas
de trás; a chave de osso pela PROFUNDIDADE, que curou também a ordem das faces da imagem), §F53 (A3 — o
índice dos anéis, Bind `343 → 64 ms`), §F54 (A4 — a forma presa antiga coze os efeitos, ao bit).
Bateria verde no fim: `ph2d-skeleton-live` 276 · `ph2d-vec-skin` 48 · `ph2d-app-vec smoke_bone` 59 ·
`architecture` 102 · shell `skeleton` 19 · clippy `-D warnings` nas quatro crates.

## 1. ⭐⭐⭐ A LISTA VIVA DOS ABERTOS — toda janela a lê ao começar e a ACTUALIZA ao acabar

> ⛔ **Regra desta lista:** ao fim da janela, cada item fica com o estado novo (`✅ FEITO <commit>` /
> `⏳ parcial: …` / `⛔ recusado pelo dono: …`), e um aberto NOVO que a janela descobrir entra com o
> próximo número livre. O próximo handoff de continuação COPIA esta secção actualizada. *Uma lista
> que só a janela que a escreveu conhece morre com a janela.*

### A1 — ✅ DECIDIDO pelo dono (2026-10-04): **o osso mais PERTO** (como está hoje) — nada a construir

> Perguntado com as duas opções lado a lado (*«o osso de onde o pedaço veio»* × *«o osso mais perto»*),
> o dono escolheu a lei ESPACIAL. A lei material abaixo fica como registo; NÃO se constrói.


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

### A2 — ✅ FEITO (F52, `3caa88daa` + docs; smoke do dono APROVADO 2026-10-04: «smoke ok»): a parte da FRENTE tapa o que é aberto na de trás

> As riscas abertas cortam-se no repouso onde a malha posada as tapa (chave de osso pela
> PROFUNDIDADE na hierarquia, triângulos do avesso, pedaços mais curtos que o traço saem); mutação
> 12/12. ⭐ Achado no caminho: a ordem das faces da IMAGEM (F48-c) punha a RAIZ por cima (a coluna
> vem por `to_bits`, decrescente no `bevy_ecs` 0.19) — curada pela mesma chave. Os dentes do *Zig
> Zag* que se encavalam são o A5 (buraquinhos reais). Detalhe: fila §F52. O texto abaixo é o
> registo de antes.


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

### A3 — ✅ FEITO (F53, `8d1a3def8`): prender um *Repeater* denso deixa de parar a tela

> Medido em release: o custo era a cerca de cobertura da grelha (varria todos os anéis por
> pergunta), não o solver — malha `743 → 21 ms`, Bind `343 → 64 ms` a `39²`. Índice dos anéis por
> faixa/célula, mesma resposta ao bit (gate contra a varredura); nenhum tecto. Detalhe: fila §F53.


- Medido em 03/10: um *Repeater* `39 × 39` que gira (`~1 000` contornos) — em DEBUG o Bind não acabou
  em 10 min (o solver `campo_do_caminho` sobre o cozido inteiro, síncrono no Bind). O pânico do
  `linesweeper` que ele também causava está CURADO (gate `a_dense_spinning_repeater_union_answers_instead_of_panicking`).
- **Fazer:** medir em `--release` (o Bind e o 1.º quadro), com `loadavg` ao lado. Se for inaceitável
  (CLAUDE.md §0.0: o tecto diz de que RECURSO é), decidir pela medição: solver numa thread (a F50-j já
  o faz para a pilha viva), malha do campo mais grossa acima de N contornos, ou um tecto de cópias
  com a razão no painel. Nenhum número sem a tabela ao lado.

### A4 — ✅ FEITO (F54, `a6c3dae04`): a forma presa de um projecto antigo coze os efeitos vivos

> No quadro, antes da pele: a fonte que o Bind de hoje faria (o mesmo cozido da F50, campo do
> contorno cozido nos eixos dos tendões guardados) — desvio `0` ao bit, controlo `3,83`/`0,48`.
> Sem degrau de schema. Detalhe: fila §F54.


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

### A6 — ⏳ (novo, 04/10) Contornos FECHADOS da parte de trás quando a união NÃO corre

- A F52 corta só os ABERTOS: o contorno fechado é da união do contacto. Mas a união só corre quando
  é neutra em repouso (`Preparado::uniao_neutra`); numa forma cujos contornos se sobrepõem já em
  repouso (as cópias de um *Repeater*, a agulha de um *Bloat* forte) o traço de um contorno fechado
  de trás continua a pintar por cima da frente numa dobra forte. Não fotografado. Desenho provável:
  a mesma `Posada` com a lei aplicada ao TRAÇO dos fechados (o preenchimento não se corta), ou
  partir a forma em camadas por chave. Medir antes (CLAUDE.md §5.0).

### A7 — (novo, 04/10, pequeno) O recorte da F52 paga `0,25 ms` por forma com riscas mesmo SEM dobra

- `so_o_que_se_ve` posa a malha e amostra as riscas em toda pose (`diag_o_preco_do_recorte_por_quadro`,
  release). Uma saída rápida (nenhum triângulo virado e nenhuma caixa de triângulos de chave maior a
  sobrepor-se a outra não vizinha) pouparia o caso comum. Só se o preço aparecer numa cena cheia.

## 2. Lições da 2.ª onda de 04/10 (morderam)

- ⛔⛔ **A COLUNA de pesos não é a profundidade do osso** — os tendões vêm por `to_bits` e o `bevy_ecs`
  0.19 aloca índices DECRESCENTES (MEDIDO: `0x…fe`, `…fd`, `…fc` por ordem de criação). A F48-c
  supunha «raiz → ponta» e o gate dela media com a MESMA chave ⇒ verde sobre a ordem invertida. Quem
  precisa de «quem está por cima» lê `esqueletos::profundidades`.
- ⛔ **Três réguas mentiram** antes da lei (F52): amostrar a curva desenhada por PONTO (uma cúbica de
  alça só de um lado deixa vãos de `0,39`); «trás à vista» que ignorava a zona da junta; e a
  compressão do próprio membro de trás a `150°`. Meça ao troço e diga o que a régua não julga.
- ⛔ **Uma mutação que sobrevive pode acusar a LEI, não o gate**: a guarda «não coze uma fonte já
  cozida» (A4) estava errada — tirá-la tornou a lei mais geral e o gate novo apanha-a reposta.
- ⛔ A mutação `>` → `>=` num predicado que a RÉGUA partilha sobrevive sempre ao gate de igualdade:
  fixe a convenção da fronteira num gate próprio.
- ⚠️ O `smoke` HERDA o `release`: os «10 min em debug» do A3 eram da crate a `opt-level 0` nos testes;
  o que o dono sentiria eram `0,3`–`0,8 s` — meça em release antes de chamar algo de lento.

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

⚠️ **O que a 2.ª onda acrescenta ao handoff de integração:** `skin_desenho::quadro` ganhou `ordem: &[f64]`
(a profundidade dos tendões) e `Leis` ganhou `frente` (literais completos de `Leis` noutras linhas
precisam do campo); `skin_image_fecho::ordena_pelo_osso` ganhou `prof` — a ordem das faces da IMAGEM
mudou (agora o último osso por cima, como o dono pediu) e isso é visível nas cenas `=3`/`=4`; a shell
chama `skeleton_live::coze_os_efeitos_presos` na `fase_vector_view_and_drives` (+2 linhas; a catraca
`the_shell_only_shrinks` tinha `378` de folga).
