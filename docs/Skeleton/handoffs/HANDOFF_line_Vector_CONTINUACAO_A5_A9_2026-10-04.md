# HANDOFF (continuação, janela nova) — `line/Vector`: fechar A5, A7, A8 e A9 num turno só (2026-10-04, 3.ª onda)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). Substitui o
> [handoff da 2.ª onda](HANDOFF_line_Vector_CONTINUACAO_A2_A4_FEITOS_2026-10-04.md).

## 0. Onde está

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector`, base `main` `1ad60a1ce`, HEAD `5d3375f04` (ou depois), nenhum commit integrado |
| o dono | aprovou os smokes do A2 e do A6 (`=5` e `=6`, *«smoke ok»*); mandou corrigir **A5, A7, A8 e A9 JUNTOS, num turno só**; **não fechar** a linha |

Entregue até aqui (fila `docs/Skeleton/01_a_fila.md`): §F52 (A2 — a frente tapa as riscas; chave de
osso pela PROFUNDIDADE), §F53 (A3 — índice dos anéis), §F54 (A4 — forma presa antiga coze os efeitos),
§F55/F55-b (A6 — sem união, o traço dos fechados sai numa camada cortada do PRÓPRIO assado; o domínio
do campo segue a regra de preenchimento; cena `PH2D_VEC_BONE_SMOKE=6`).

## Mapa do código (para não reler à procura)

- `crates/ph2d-skeleton-live/src/skin_desenho.rs` (`calcula`, ⚠️ `695` linhas — tecto `700`: cure por CORTE)
- `skin_desenho_frente.rs` — `Posada` (malha posada, chave por triângulo, `virado`, `Arte`), `a_vista`
  (`AMOSTRAS = 32` + `BISSECCOES = 12`), `recorta`, `so_o_que_se_ve` (riscas), `cortes_dos_fechados`
- `skin_desenho_camadas.rs` — `Desenhado { forma, traco }`, `traco_sobre_o_assado`, `funde`
- gates: `skin_desenho_frente_tests.rs`, `skin_desenho_camadas_tests.rs` (réguas SEM a chave da lei)
- imagem presa: `skin_image_fecho.rs` (`costura`, `ordena_pelo_osso`), fila §F49 «LIMITE CONHECIDO»
- cenas: `crates/ph2d-app-vec/src/smoke_bone_*.rs` (`=4` imagem, `=5` efeitos, `=6` cópias)
- fotos sem placa: SVG do `Desenhado` + `magick` (a placa é disputada por outras linhas)

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

### A5 — (b) ⛔ recusado pelo dono no smoke (F59; feito em `f1481cd7d`, REVERTIDO em `501daabf4`: «o modo anterior era melhor») · (a) ⏳ TENTADO E REVERTIDO (F59, `ac8246764` → `9e39c48a5`): a cúspide da imagem

> ⛔ **05/10 — a hipótese seguinte (o anel por *marching squares*) foi medida e REFUTADA** (fila §F59, tabela das três leis): pior que a marcha na cúspide, pior que a lei de hoje a −149,5°, e ainda cose sobre a tinta de outro membro. Quatro desenhos ⇒ (a) fica ABERTO sem lei candidata; o que falta perceber é o critério de LADO (porque remenda sobre outro membro), não a forma do anel.

> O dono escolheu (04/10) «fechar os buracos tão pequenos que a linha os cobre» e «corrigir» a cúspide.
> (b) feito: 100° `[0,13 0,54 0,12]→[0,54]`, 110° `[1,24 0,25 1,25 0,42]→[1,24 1,25]`, mutação 5/5. (a) a tentativa
> melhorou a pose do relatório (104→48) mas regrediu −149,5° e −160° nos gates da F49 ⇒ revertida após 3
> reconstruções; próximo passo = o contorno da arte por marching squares (fila F59). Registo de antes:

- A **cúspide da ARTE** de uma imagem presa junto a uma tampa redonda (limite da F49): pede o fecho
  sobre o contorno da ARTE (pixels, na `attach_skin_meshes`). Ver fila §F49 «LIMITE CONHECIDO».
- No *Zig Zag* muito dobrado (`~110°`) os dentes de dentro encavalam-se e fecham buraquinhos REAIS (a
  imagem também os mostra).

### A6 — ✅ FEITO (F55 + F55-b `9dc1a0434`…; pedido pelo dono 04/10; o 1.º smoke do dono achou o traço DESCOLADO numa dobra agressiva — curado na F55-b; 2.º smoke APROVADO 2026-10-04: «smoke ok»): o traço dos fechados de trás não pinta por cima da frente

> Sem união, a forma sai em duas camadas (preenchimento + traço à vista). Causa de fundo curada: o
> domínio do campo segue a regra de preenchimento (a sobreposição `NonZero` era furo). Só a ARTE
> tapa; o avesso não tapa o traço de um fechado. Cena `=6`. Detalhe: fila §F55. Registo de antes:


- A F52 corta só os ABERTOS: o contorno fechado é da união do contacto. Mas a união só corre quando
  é neutra em repouso (`Preparado::uniao_neutra`); numa forma cujos contornos se sobrepõem já em
  repouso (as cópias de um *Repeater*, a agulha de um *Bloat* forte) o traço de um contorno fechado
  de trás continua a pintar por cima da frente numa dobra forte. Não fotografado. Desenho provável:
  a mesma `Posada` com a lei aplicada ao TRAÇO dos fechados (o preenchimento não se corta), ou
  partir a forma em camadas por chave. Medir antes (CLAUDE.md §5.0).

### A7 — ✅ FEITO (F56, `c9786bbd3` + `a93bfdcf0`): a saída rápida do recorte

> Sem par sobreposto nem virado o recorte nem amostra: forma com 36 riscas sem dobra 174–216 → 77 µs,
> igual ao bit (gate `a_saida_rapida_nao_muda_o_recorte_ao_bit`), mutação feita. Fila §F56. Registo de antes:

- `so_o_que_se_ve` posa a malha e amostra as riscas em toda pose (`diag_o_preco_do_recorte_por_quadro`,
  release). Uma saída rápida (nenhum triângulo virado e nenhuma caixa de triângulos de chave maior a
  sobrepor-se a outra não vizinha) pouparia o caso comum. Só se o preço aparecer numa cena cheia.

### A8 — ✅ FECHADO sem cura (F58): as janelas perdidas existem mas ficam abaixo da largura do traço (máx 0,45)

> Medido a 2 048 amostras/segmento (60°…170°): a maior janela perdida é 0,45 da largura do traço ⇒
> borrão pela régua da F52. Gate `nenhuma_janela_perdida_chega_a_largura_do_traco`. Fila §F58. Registo de antes:

- O recorte (F52/F55) amostra `32` pontos por segmento e bissecta onde o estado muda. Numa aresta
  recta muito longa uma janela curta pode cair entre duas amostras. A amostragem pelo comprimento
  (passo ¼ da aresta da malha) foi construída e RETIRADA na F55: nenhuma fixtura a exprimiu (a barra
  `400 × 2` tem a malha grossa demais), e a hipótese que a trouxe estava errada. Só com um caso
  FOTOGRAFADO.

### A9 — ✅ FEITO (F57, `4384189db` + `a3dc835f8` + `a93bfdcf0`): a ponta do corte acerta no cruzamento desenhado

> Cada ponta de trecho vai ao cruzamento mais perto ao longo do contorno fechado, até 1 largura:
> 0,13…0,98 → 0,00…0,02 na `=6`. Gate `nenhuma_ponta_de_corte_fica_a_um_tique_do_cruzamento`. Fila §F57. Registo de antes:

- Em SVG a `170°/−110°` (cena `=6` com o osso do meio quase dobrado sobre si) sobram dois ou três
  tiques de `~0,1` nas pontas de cortes do traço, junto ao vinco. Não medido nem fotografado no app.

### A10 — ✅ FEITO (F60, 05/10): a ponta do vinco converge para a pele exacta (`2,11 → ≤ 0,019` larg. a 160–175°) — fila §F60. Registo de antes:

- O traço de um fechado contorna a dobra do papel (o grampo) e volta um pedaço antes de acabar
  (FOTOGRAFADO em SVG na `=6` a `170°/−110°`, barra sem riscas). Não há cruzamento ali ⇒ o encaixe da F57
  não o alcança (de propósito: o gate prova que pontas a `> 1` largura não se mexem). Causa provável: a
  mesma (a malha recta decide, a pele exacta desenha). Cura de fundo: decidir «tapado» pela pele EXACTA
  do cobridor (inverter o mapa no triângulo por Newton, ou triângulos quadráticos com o ponto médio de
  cada aresta posado) — medir o preço antes (a dobra já custa `0,3 ms` no recorte).

### A11 — ✅ FEITO (`eaa53edb1`, 04/10; conferido 05/10: `cargo fmt --check` limpo nas crates da linha). Registo de antes:

- `cargo fmt --check`: `ph2d-app-vec` 9 diffs, `ph2d-vec-skin` 14, `ph2d-vec-boolean` 2 (a
  `ph2d-skeleton-live` ficou formatada nesta janela, `4d91679e9`); o ship corre `cargo fmt --all -- --check`.

### A12 — ✅ FEITO (F61, 05/10, «sim» do dono): as passagens NOVAS mais estreitas que o traço (fendas e pontas) cortam-se pela corda; os buracos engolidos ficam — fila §F61. Registo de antes:

- No Zig Zag a 100° (FOTOGRAFADO em SVG, `=5`) duas marquinhas escuras onde o contorno entra numa reentrância mais estreita que o traço; não são buracos (a lei da F59-b não as toca). Medir (largura da reentrância vs largura do traço) e perguntar ao dono se as quer fechadas como os buracos.

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
- ⛔ O memo do quadro (`skin_desenho::MEMO`) é por thread e guarda o desenho por forma: um gate que
  compara a lei ligada e desligada na MESMA thread lê o 1.º desenho dos dois lados — desenhe cada lado
  numa thread nova (o gate do A9 foi verde-vazio assim até se ver).
- ⛔ Um nó de canto (alças sobre as âncoras) não tem o parâmetro linear no comprimento: um gate que
  espera `u = 0,52` para `x = 5,2` reprova uma lei certa — confira o PONTO.
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

⚠️ **E o A6 (F55) acrescenta:** `SkinDesenhado` é `BTreeMap<_, Desenhado>` (`Desenhado { forma,
traco }`, com `Deref` para a forma) e `Quadro` ganhou `traco`; o domínio do campo de pesos
(`ph2d-vec-skin::pesos`) segue agora a regra de preenchimento da forma — o campo de binds NOVOS de formas
com contornos sobrepostos `NonZero` muda; `smoke_bone::NIVEIS` passou a `6` (cena `=6`, `smoke_bone_copias`).
- ⛔ `git revert -q` não existe (o revert falha e um `--amend` a seguir renomeia o commit ERRADO) — confira `git log` antes de emendar.
- ⛔ `Write` num nome que já existe escreve por cima sem aviso — `ls` antes de criar um ficheiro novo (apanhou o `skin_image_tinta.rs` do 9-slice; reposto do git).
