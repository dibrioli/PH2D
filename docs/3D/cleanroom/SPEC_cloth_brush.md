# SPEC — o PINCEL DE TECIDO e o FILTRO DE TECIDO (clean-room do alvo `blender-cloth`)

```
Alvo: Blender 5.2.0 (tag v5.2.0) — Sculpt Mode, pincel de tecido + filtro de tecido + o alvo de
  deformação «simulação de pano» dos pincéis Pose/Boundary · Licença: GPL-2.0-or-later · Degrau: T2
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-cloth.md, 2026-09-05
Papel E: subagente-E da janela 1246816c-63cf-414b-842d-663a8baa86ca (2026-09-05). ⛔ Este subagente
  leu o fonte; a janela-mãe NÃO, e é ela a I.
Patente (§8.1): buscado em 2026-09-05 (termos no ledger). Nenhuma patente viva alcança o método.
  Duas CERCAS nomeadas: US 10 586 401 B2 (Pixar — pincéis por soluções ANALÍTICAS de elasticidade
  linear; nunca implementar um modo «elástico» por Kelvinlet) e US 10 713 855 B2 (Audaces, BRASIL —
  vestuário sobre MANEQUIM com escultura + pano + impedir a entrada no manequim; se a casa um dia
  fizer «vestir um corpo» com colisão contra o corpo, refazer a busca com parecer humano).
Filtragem §4.3: executada em 2026-09-05 (E) · Sweep: verde em 2026-09-05 (E), vassoura de 70 entradas
Auditoria §4.2 (R-pré): ✅ auditada contra §4.2 por R-pré em 2026-09-05 — sweep verde sobre espec +
  fixtures + INBOX + READMEs + docs/3D/cloth/02-04 (e sobre o histórico destes caminhos). UM achado de
  expressão (uma frase de comentário do fonte citada como (F) no §7) e CINCO higienes §4.3 (detalhe de
  implementação descrito como comportamento) curados no acto pelo R-pré; os nomes de fixture do §14 e a
  contagem do gate 15 alinhados aos ficheiros. Veredictos e curas, um a um: LEDGER §Papel R.
  ✅ EMENDAS Q8, Q9 e Q10 de 2026-09-06 — Q10 (§10.5 NOVA): dois traços de APERTO por passo, 12 passos,
  prova do fatiamento `0,000000`. Q9 (§4.3 · §10.4 NOVA · §14 gates 12 e 18): o centro da queda do
  Snake Hook está UM PASSO atrasado, e a força por passo das âncoras é zerada nos DOIS modos de âncora.
  Q8 (§1 fases 0/1 · §2.1 · §3.1 · §3.3 · §5.2-bis NOVA · §10.2 · §10.3 NOVA · §13 ·
  §14 gates 8/16/17): a lista de restrições do ramo *Local* vem em DUPLICADO. Escritas pelo subagente-E
  da mesma janela, com o fonte reaberto só para estas perguntas.
  **As TRÊS auditadas contra §4.2 por R-pré em 2026-09-06** (contexto novo, independente de quem as
  escreveu; leu os dois lados, o fonte por shell) — sweep verde sobre a espec emendada + a pasta inteira
  das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes caminhos.
  UM achado §4.2 (um nome interno do alvo em forma de identificador, no §10.4) e UMA insuficiência
  (o gate 16 fixava `2×`, que é só o caso de UMA passagem de simetria) **curados no acto pelo R-pré**;
  os factos das três emendas conferidos no fonte, **todos correctos**. Veredictos: LEDGER §Papel R.
  ⭐ ERRATA de 2026-09-06 (`3d621e94b` + `d5844ad5c`: §2.1 · §3.1 · §5.2 · §10 · fixtures de esfera)
  auditada contra §4.2 por R-pré em 2026-09-06 — sweep verde sobre espec + a pasta inteira das fixtures +
  INBOX + ledger + o histórico destes caminhos. TRÊS nomes internos do alvo (dois no §5.2, um no §10)
  re-expressos em vocabulário do domínio e UMA linha órfã de arnês apagada no §10, no acto; os seis
  factos da errata conferidos no fonte pelo R-pré, todos correctos. Detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q11 de 2026-09-06 — auditada contra §4.2 por R-pré em 2026-09-06** (§3.1 · §4.2 · §5.2 ·
  **§5.2-ter NOVA** · §9 nº 20 · **§10.6 NOVA** · §11 · §14 gates 19-21, + a fixture nova
  `plano_apertar_ponto_radial_local_origem_fraco`): o aperto **inverte a malha debaixo do cursor no
  PRIMEIRO passo simulado**, e a partir daí o resultado por vértice é decidido pela ORDEM de
  resolução. Escrita pelo subagente-E da mesma janela, com o fonte reaberto só para estas perguntas
  e com uma corrida NOVA do oráculo (o par de força do §5.2-ter).
  **R-pré: contexto novo, independente de quem a escreveu; leu os dois lados (o fonte por shell).**
  ⛔ **ZERO achados de §4.2** — sem trecho, sem nome interno, sem wording de comentário/manual; a
  ordem de criação do §3.1 é mecanismo (§4.1.11) escrito em vocabulário do domínio, e o nome da
  fixture nova também. Sweep **verde** sobre a espec emendada + a pasta inteira das fixtures + INBOX
  + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** da espec e das fixtures.
  ⭐⭐ **FIDELIDADE: os factos foram conferidos no fonte e os NÚMEROS reconstruídos do zero pelo
  R-pré a partir das fixtures** — a ordem interna de criação (as cinco espécies na ordem que o §3.1
  passou a dizer), o filtro de raio alcançar só duas delas, o anel percorrido face a face, a
  ausência de tecto no factor de correcção, a re-escala a comprimento 1 nos dois apertos, o instante
  em que cada modo lê as posições, e o **§9 nº 20** (a versão que gravou as fixtures MULTIPLICA —
  não há divergência a declarar). As **duas tabelas** do §5.2-ter e do §10.6 reproduzem célula a
  célula (`10 / 18 / 52` · `0,675` · `1,060` · `0,103` · `0,059` · `0,286` · `0,088` · `0,283` ·
  `2 145` · `2 029` · `0,303401` · `0,004082`), e os «9 vértices que passam o cursor» também.
  **QUATRO curas aplicadas no acto pelo R-pré, todas funcionais:** (1) o §3.1 dizia que a âncora e o
  pino nascem «para todo vértice visível», o que contradizia o §2.3 — cada um tem a SUA condição;
  (2) a régua da compressão do §5.2/§10.6 não dizia sobre que pares corria e **subestimava por
  `3,5×`** (`0,052`/`−18,1` só sobre arestas, contra `0,015`/`−64,4` sobre todos os pares da
  construção — as duas ficam agora escritas); (3) as **duas réguas** da tabela do §5.2-ter (o
  quadrilátero invertido e a assimetria de espelho) não estavam definidas em lado nenhum, logo o
  gate 19 não era edificável — ficam definidas, com a leitura ERRADA nomeada (somar metades
  triangulares dá `11 / 26 / 88`); (4) os gates 15 e 17 citavam `51` e `50` traços de memória, com
  `54` no disco. Mais a decisão do §5.2-ter posta nas duas frases que o dono precisa de ler.
  Veredictos e detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q12 de 2026-09-06 — auditada contra §4.2 por R-pré em 2026-09-06** (§4.2-bis NOVA · §4.3 · §4.4 ·
  §4.6 NOVA · §10 contagem · §10.7 NOVA · §14 gates 22-24, + as duas fixtures por passo
  `plano_empurrar_radial_local_origem` e `plano_inflar_radial_local_origem` e o gerador do índice
  delas): a **normal da área** que o Push lê (quando é reavaliada, sobre que malha, em que raio, com
  que peso e com que regra de desempate), o **factor de escala** que a multiplica, a lei do **centro
  da área** (que fecha a pergunta em aberto no fim do §4.4), e o **censo das grandezas que são
  degeneradas num plano visto de frente e vivas numa superfície curva** — entre elas que o
  deslocamento do cursor que os oito modos lêem (no guarda) e de que quatro consumidores tiram
  direcção é a **projecção** no plano do ecrã, e que só o arrasto tira a direcção da diferença dos
  dois pontos 3D. Escrita pelo subagente-E da mesma janela, com o fonte
  reaberto só para estas perguntas e com uma corrida NOVA do oráculo (as 26 corridas-prefixo do §10.7).
  **R-pré: contexto novo, independente de quem a escreveu; leu os dois lados (o fonte por shell).**
  ⛔ **ZERO achados de §4.2** — sem trecho, sem nome interno, sem wording de comentário/manual; a
  regra dos dois baldes, a lei do centro da área e o censo do §4.6 estão em vocabulário do domínio, e
  as fórmulas são matemática (§4.1.2). Sweep **verde** sobre a espec emendada + a pasta inteira das
  fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes caminhos (os
  dois hits do ledger são os pré-existentes de 2026-09-05, já registados lá para o R-pós).
  ⭐⭐ **FIDELIDADE: os factos foram conferidos no fonte e os NÚMEROS reconstruídos do zero pelo
  R-pré** — a reavaliação por passo sobre a malha **deformada** (com a lista completa do que a
  congela), o disco de **meio** raio, o peso `3p²−2p³`, os dois baldes pelo sinal contra a vista, o
  vector nulo sem `NaN`, o factor de escala de três números fixado no pen-down e aplicado componente a
  componente, a lei do centro da área, e as **duas** metades da §4.6-1 (o deslocamento des-projectado
  à profundidade do pen-down · o arrasto a ler a diferença dos dois pontos 3D). Reproduzem-se do zero:
  `0,3518`/`+0,5 %` e `1,3184`/`+7,6 %` das calotas · `0,05455` e `0,01547` · `15,83°` e `1,039×` ·
  `19,3 %` · a razão `0,06543/0,09347 = 0,7000 = 2R` · as **11** colunas dos dois rastreios do §10.7 ·
  as contagens `0,236509` · `0,463862` · `0,325769` · `0,046715` · e o `gera_indice.py`, que
  **regenera o `indice.json` byte-a-byte** (56 entradas para 56 ficheiros).
  ⚠️ **A correcção à §5.2-bis (o duplicado nas costuras do *Dynamic*) confere**, e o mecanismo também:
  o registo de pares é local a cada construção e cada cópia carrega a sua própria célula.
  **SEIS curas aplicadas no acto pelo R-pré, todas funcionais:** (1) o desempate dos baldes é *o
  primeiro que esteja não-vazio **E** cuja soma não se anule* — a redacção anterior mandava responder
  o vector nulo num caso em que o alvo lê o outro balde; (2) o guarda de «passo parado» vale para os
  **oito** modos (a célula da §4.6-1 lia-se como se o arrasto não o tivesse), e Push/Inflate/Expand
  não lêem `δ` para a direcção nenhuma — não só o arrasto; (3) ⛔ **a §4.6-4 afirmava peso de ÁREA e
  isso NÃO é demonstrável** com o que esta linha tem à mão (nem o corpus o decide: no plano todos os
  pesos coincidem, na esfera UV a simetria em longitude apaga a diferença) — a linha passa a declarar
  a FORMA e a nomear o peso como pergunta aberta, com a régua que a fecharia; (4) o §10.7 dizia que a
  frente a `1R` **ultrapassa** o pen-down do Push e os próprios números dizem o contrário
  (`0,1976 < 0,2195`); (5) o gate 22 dava a sequência de ângulos truncada em `…` — ficam os **11**
  passos e a razão de ela ser simétrica; (6) o censo da §4.6 dizia «as oito fixtures de esfera» e
  listava sete — a oitava é o **arrasto**, que é precisamente o CONTROLO do gate 22.
  ⚠️ **E DUAS no README das fixtures, ambas de contagem, ambas da família que esta linha já pagou
  duas vezes:** `ls *.porpasso.txt.gz` devolve **13** e não `9`, e os quatro que sobram são a 1.ª
  geração — **três deles NÃO passam a prova do fatiamento** (`0.330421` · `0.115064` · `0.004244`) e
  estavam debaixo da frase «o pen-down de TODOS eles está na origem»; ⭐ o quarto passa, e passa
  *porque a área dele é Global e não tem centro para ficar refém do sobrevoo* — que é o mecanismo que
  o próprio README explica. Mais o total do fim, parado em `53` com `56` no disco, e a fixture
  `_fraco` que faltava na tabela das corridas (agora `56` linhas para `56` ficheiros).
  ⭐⭐⭐ **E uma SÉTIMA cura, de SUFICIÊNCIA, que fecha a pergunta que o I acabou de fazer (Q13):** a
  §4.3 descrevia `δ` como uma des-projecção de ecrã sem nunca dizer **qual é a vista**, e sem isso ele
  não era reconstruível deste lado. As fixtures são **ortográficas** (prova nos números) ⇒
  `δ = proj_⊥v̂(c_k − c_{k−1})` sobre o `caminho` que o cabeçalho já traz, com `v̂` = **`z`** no corpus
  do plano (projecção = no-op) e **`y`** no da esfera. Está agora na §4.3 e no README das fixtures,
  com o ⛔ de que o plano do **ECRÃ** não é o plano tangente do pen-down — a rota que o I mediu a
  piorar (`0,265 → 0,605` · `0,351 → 0,663`).
  Veredictos e detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q14 de 2026-09-06** (§2.2 errata · §3.2 · §4.2 · §4.5 · §5.2 · **§5.2-quater NOVA** ·
  §10 contagem · **§10.8 NOVA** · §11 · §14 gates 25-31, + as **nove** fixtures `*_origem_1passo*` que
  isolam a rede de restrições): como a âncora, o pino e o corpo mole são resolvidos DENTRO da
  varredura (uma lista só, `Δ/2` para as quatro espécies, e ⭐ o alvo de cada uma lido no instante da
  projecção — vivo na estrutural, **móvel** no corpo mole, fixo na âncora e no pino); o **censo dos
  limites do solver**, que devolve **nenhum** e desmonta a busca por um tecto dependente do tamanho
  do passo; ⭐⭐⭐ e a correcção da premissa que gerou a pergunta — **os sete traços de um passo dos
  cinco modos que escrevem ACELERAÇÃO movem o disco do pincel e ZERO vértices fora dele, logo não
  exercitam uma única restrição de distância** (⚠️ o Expand é «modo de força» pela §4.2 e está do
  OUTRO lado desta partição, porque escreve repouso e não aceleração);
  mais o instante em que o desvio de repouso do Expand passa a valer (no
  MESMO passo, antes da 1.ª varredura) e o facto de ele entrar nas **quatro** espécies. Escrita pelo
  subagente-E da mesma janela, com o fonte reaberto só para estas perguntas e com uma corrida NOVA do
  oráculo (10 execuções: 1 de validação + as 9 gravadas).
  ✅ **AUDITADA contra §4.2 por R-pré em 2026-09-06** — contexto novo, independente do subagente que a
  escreveu; leu os dois lados (o fonte por shell).
  ⚠️ **TRÊS achados §4.2, todos da mesma espécie — comentário/doc-comment do alvo re-dito
  quase-verbatim — e todos CURADOS NO ACTO por re-expressão** (nenhum facto perdido): a justificação
  do 1.º passo não simular (§1 fase 0, **pré-existente**), a frase que descrevia como a restrição
  guarda as pontas (§3.2) e a razão de o peso por vértice ser um vector da malha inteira (§5.2 nº 5).
  ⛔ Zero trechos, zero nomes internos, zero wording de manual; os nomes de fixture, as chaves dos
  cabeçalhos e o vocabulário novo (**gaveta · memória de forma · desvio de repouso**) são do domínio,
  e `Gauss–Seidel`/`Jacobi` são literatura pública. Sweep **verde** sobre a espec emendada + a pasta
  inteira das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes
  caminhos (os dois hits do ledger são os pré-existentes de 2026-09-05, já registados lá).
  ⭐⭐ **FIDELIDADE: os 11 factos foram conferidos no fonte e TODOS os números reconstruídos do zero**
  a partir das fixtures, com script próprio fora do repo. Confirmados: a lista única e o laço `5×` na
  ordem de criação; `Δ/2` nas quatro espécies; o peso por índice com a excepção do corpo mole; `φ`
  calculado **uma vez por passo para a malha inteira**; o alvo de cada espécie lido **no instante da
  projecção** (estrutural vivo · âncora fixa · memória de forma **a andar** · pino fixo); as gavetas de
  âncora a nascerem na posição de repouso com factor `1`; `ℓ'` nas quatro espécies com o desvio
  **inteiro** nas de alvo próprio; o desvio do Expand somado na fase do gesto, que corre **inteira**
  antes do solver; o guarda de cursor parado a desistir **só** da fase do gesto, com a zeragem do
  factor por passo **depois** dele; e a errata da banda (`R(1+L)` / `R(1+L·F)`).
  ⚠️ **O censo dos limites está certo DENTRO do solver e não era exaustivo no passo:** falta-lhe o
  bloqueio de eixos e o recorte de espelho da casa, que aparam o deslocamento **na escrita** (§6.1) —
  ficam agora nomeados na tabela, com o ⛔ de não dependerem do tamanho do passo.
  ⭐ Reproduzem-se célula a célula: o censo do disco (`171`/`156` dentro e **`0`** fora nos sete traços
  de aceleração; `173` + `675`/`1151`/`1279` nos três de âncora/repouso) · a tabela *Local*×*Global* e
  as três razões (`0,752`/`0,705`/`0,797`) · as **quatro** sequências de razões do perfil (`4,52` e
  `5,11` nos degraus; `1,32`/`1,47` nas caudas) · o traço curto (`0,649·δ` contra `0,760·δ`) · a curva
  *Constant* (`0,878999 = 1,465·δ`, `107` vértices, máximo a `0,552 R`, degrau `1,071→1,205 R` com
  razão `5,32`, e `0` vértices na *Smooth*) · `verifica_traco.py` **verde sobre os 65** ·
  `gera_indice.py` a regenerar o `indice.json` **byte-a-byte** (65 entradas para 65 ficheiros) · e a
  tabela «As corridas» do README, **65 linhas, todas a casar com o índice**.
  ⛔ **TRÊS números discordaram e foram corrigidos para a leitura do R-pré:** (1) a 4.ª razão do traço
  curto é `1,2747` ⇒ **`1,27`**, não `1,28` (e são `8` razões, logo `9` células); (2) o terceiro valor
  da errata da banda — o gancho dá **`1,2234`**, e o `1,2239` é de **outras** fixtures; (3) a razão do
  par de força do Expand é **`0,24999`** nos máximos e `0,250000` na mediana por vértice (`0,2497` é o
  que sai de dividir os cabeçalhos já arredondados). ⭐⭐ **E a errata da banda ganhou a prova que lhe
  faltava:** `plano_agarrar_radial_local_preset` corre com `limite = 5,0` e alcança `2,0745` —
  `0,99 · R(1+L)` e `+19 %` para lá de `R·L`; *com um só `L` as duas fórmulas eram indistinguíveis por
  medição.*
  ⚠️ **DUAS afirmações de fidelidade estavam ERRADAS e foram curadas** (as duas com o mesmo efeito num
  port: apagar comportamento sem aviso): a supressão do segundo `Δ/2` por igualdade de índice alcança
  a âncora e o pino, **não o corpo mole** — ali a segunda metade é aplicada à memória de forma, e quem
  a encaminha é a espécie (um braço só **congela a plasticidade**); e a combinação Expand + pino /
  Expand + corpo mole **é** alcançável com o pincel de tecido sozinho (são opções independentes do
  modo), ao contrário do que a emenda declarava — só a âncora é que não.
  **CURAS de SUFICIÊNCIA nos gates 25-31, para que sejam edificáveis do lado limpo:** as réguas do
  censo (limiar, centro e tamanho do disco) e da amostragem do perfil ficam escritas; o gate 25 passa
  a nomear os **sete** traços com as contagens de cada um; o 26 ganha a taxa do corpo mole; o 27 ganha
  o **ponto de encontro `(1−ρ)A₀ + ρB₀`** (⛔ não é o ponto médio) e o invariante que o produz; o 28
  ganha as duas leituras da razão; o 29 troca o tecto de `1,4` — que **reprovava o próprio oráculo** —
  por um discriminador derivado (maior razão ÷ mediana das restantes, com o vazio de `3,6×` medido);
  o 30 ganha o limiar, a isenção do controlo de força fraca e a segunda metade com `L = 5`; e o 31
  passa a declarar-se **gate de espec**, porque nenhuma fixture traz um ponto de caminho repetido.
  ⚠️ **E o vocabulário «modos de FORÇA» tinha DUAS populações na mesma emenda** — a §4.2 chama-lhes
  seis (o Expand incluído) e o censo precisava dos **cinco que escrevem aceleração**; a §5.2-quater
  passa a dizê-lo na primeira linha, senão a conclusão lê-se ao contrário.
  ⚠️ Mais **uma cura no README das fixtures**: o parágrafo dos valores de omissão tinha **sete**
  excepções e nomeava uma — entre elas a fixture de `limite = 5,0` que a errata da banda agora usa.
  **Veredicto: ATESTADO** — a emenda Q14 pode ser lida pela janela-mãe. Detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q15 de 2026-09-06** (§2.1 errata · §3.1 · **§3.1-bis NOVA** · **§10.9 NOVA** ·
  §14 gates 32-34, + as QUATRO fixtures de topologia `plano.faces` · `esfera.faces` ·
  `plano.celulas` · `esfera.celulas`): **a ordem de criação passa a ser aplicável fora do plano.**
  As três ordens da §3.1 ficam escritas por inteiro — as **células** por índice crescente (a busca
  ORDENA o que devolve, logo a ordem não depende do cursor), dentro de cada uma os **vértices
  PRÓPRIOS** dela por índice crescente (e cada vértice é próprio de uma só célula, logo a sequência
  global é uma **permutação** agrupada por célula, ⛔ **não** a ordem crescente), e o **anel** pelas
  faces incidentes por índice crescente de FACE, cada uma a dar o canto anterior e depois o seguinte
  no sentido de percurso, com a deduplicação a guardar a primeira ocorrência. As quatro fixtures dão
  ao lado limpo a lista de faces e a partição em células das duas malhas, que ele não podia
  reconstruir (ele casa a malha dele com as fixtures **por posição**, logo tinha os índices de
  vértice e não tinha nem as faces nem as células).
  ⭐⭐ **A partição é MEDIDA, não derivada:** ela não é observável de fora, mas a aplicação de
  referência tem um gesto que reordena a malha pela **mesma** lei, e a permutação que ele devolve
  lê-se de Python e bate a partição derivada **elemento a elemento nas duas malhas** (§10.9).
  ⛔⛔ **E DUAS presunções do I foram REFUTADAS:** «no plano a ordem das células é benigna» — o plano
  tem **duas** células e a ordem de visita é a identidade **rodada** (`[2080..4224]` e depois
  `[0..2079]`), com o descenso exactamente no pen-down das fixtures `_origem`; e «no plano a lei do
  anel degenera» — ela degenera no **interior** e não no **bordo** (`128` vértices), e o corpus não a
  vê porque esses `128` estão todos **fora da banda** com `limite = 2,5`. ⭐ A fixture
  `plano_agarrar_radial_local_preset` (`limite = 5,0`) alcança `126` dos `128` e **discrimina**.
  Escrita pelo subagente-E da mesma janela, com o fonte reaberto só para estas perguntas e com uma
  corrida NOVA do harness (a geração das quatro fixtures e a validação da permutação).
  ✅ **AUDITADA contra §4.2 por R-pré em 2026-09-06** — ⚠️⚠️ **esta emenda tinha SHIPADO SEM
  ATESTAÇÃO** (o quadro não a trazia e o ledger não tinha a secção dela, e a §3.R diz que sem
  atestado a janela não implementa); a auditoria foi feita pelo R-pré da Q16, no mesmo dia, ao dar
  pela ausência. **ZERO achados de expressão**; **duas** higienes curadas no acto — duas frases que
  descreviam a FORMA do código e uma delas citava a prosa dele como autoridade, re-expressas como
  comportamento (*a ordenação é de propósito* · *a tabela é construída em paralelo e depois
  ordenada*). ⭐⭐ **FIDELIDADE: todos os números reconstruídos do zero das quatro fixtures novas** —
  plano `2` células (`2 145` + `2 080`, `2 048` faces cada, **`1`** descenso, as duas contíguas e a
  de índice `1` a ser `[2080..4224]`) · esfera `4` células (`1 569`/`1 520`/`1 504`/`1 457`, `1 536`
  faces cada, **`3`** descensos) · as somas a baterem `4 225` e `6 050` **sem um repetido** · os
  anéis divergentes `128` (`3,0 %`) e `5 959` (`98,5 %`) · o mais próximo dos `128` a `1,5` do
  pen-down com **`0`** dentro de `1,2250` e **`126`** dentro de `2,1000` · e os «dois `2 145`
  diferentes», com intersecção `1 099`. Conferidos no fonte: o tecto de `2 500` faces por folha e a
  profundidade `99`; a lista de vértices da folha **ordenada** e reclamada por índice crescente de
  folha; e o teste `centro[eixo] ≥ limiar` a mandar o lado `≥` para o **primeiro** filho, no qual a
  recursão desce primeiro. ⛔ **UMA cura de fidelidade:** a regra do **empate de eixos** era dada
  como facto de fonte e a função que a resolve **não vive na parte do fonte que esta linha tem** — a
  metade `X`↔`Y` fica provada pela partição medida do plano, a metade `Y`↔`Z` fica nomeada como a
  única cláusula sem prova deste lado (§3.1-bis).
  ✅ **EMENDA Q16 de 2026-09-06** (**§5.7-bis NOVA** · §10 contagem · **§10.10 NOVA** ·
  §11 · §14 gate 31 reescrito + gates 35-38, + as **oito** fixtures por passo que separam a fase do
  gesto da fase do solver — ⚠️ **a §5.2 é REFERIDA e não foi alterada**: a 1.ª redacção desta linha
  listava-a como tocada e o diff da emenda não lhe põe uma linha; R-pré, 2026-09-06): as três perguntas do I devolvem **NÃO** as três, e a resposta é um
  **censo exaustivo**, não uma impressão — (1) nada na projecção de uma restrição de distância
  depende de quanto o par está esticado além do factor `(1 − ℓ'/D)`: sem tecto, sem segundo passe,
  sem correcção de ordem superior, e o comprimento de repouso é lido de **um** sítio só; (2) não
  existe termo, peso ou espécie que só entre quando o deslocamento tem componente ao longo da
  normal — as quatro espécies do §3.2 são a lista inteira e **não há rigidez angular nenhuma**; (3)
  não há sub-passos: **um** passo de solver por passo de pincel, e o número de varreduras é uma
  constante que não lê o tamanho do gesto. ⇒ ⭐⭐⭐ **a resposta útil não era um facto que faltasse, era
  um INSTRUMENTO que faltava**, e ele é a 4.ª pergunta: **oito** traços novos que cortam o passo em
  duas metades — `_parado` (um impulso conhecido no passo 2 e depois **dez passos sem força nenhuma**,
  com um controlo que prova que a fase do gesto está calada), `_forca05`/`_forca025`/`_massa2` (a
  mesma cena a **um quarto, um dezasseis avos e metade** da amplitude), `_amort1` (sem memória de
  velocidade) e `_global` (metade das projecções por restrição). Escrita pelo subagente-E da mesma
  janela, com o fonte reaberto só para estas perguntas e com uma corrida NOVA do oráculo (107
  execuções: 104 que viraram fixture + 1 sonda do instrumento + 2 de controlo).
  ✅ **AUDITADA contra §4.2 por R-pré em 2026-09-06** — contexto novo, independente do subagente que
  a escreveu; leu os dois lados (o fonte por shell). Sweep **verde** sobre a espec emendada + a pasta
  inteira das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes
  caminhos (os dois hits do histórico são os pré-existentes de 2026-09-05, do LEDGER, já registados lá).
  ⛔ **ZERO achados de EXPRESSÃO** — sem trecho, sem nome interno, sem wording de comentário ou de
  manual; e ⭐ **a ordem do censo do §5.7-bis é a das PERGUNTAS do INBOX, não a de coisa nenhuma do
  alvo** (conferido linha a linha contra o Q16), que é onde um censo de ausências escorrega.
  **TRÊS higienes §4.2/§4.3 curadas no acto por re-expressão, sem perder facto:** um termo de código
  em inglês onde a palavra do domínio já dizia tudo; *«há uma lista e um laço»*, que descrevia a
  FORMA do código e passa a dizer o comportamento (*o conjunto é percorrido inteiro, `5` vezes, e
  nenhuma passagem selecciona as esticadas*); e *«uma constante do ficheiro»*, que localizava a
  constante no fonte e passa a *«um valor fixo do programa»*.
  ⭐⭐ **FIDELIDADE: as DEZ linhas do censo foram conferidas no fonte e TODOS os números do §10.10
  reconstruídos do zero a partir das fixtures**, com script próprio fora do repo. Confirmados no
  fonte: o factor único e o guarda de separação nula · a ausência de tecto · uma travessia só,
  repetida `5` vezes, sem passe selectivo · a correcção linear · o comprimento de repouso escrito
  **uma** vez na criação, com o desvio do Expand como único somando (um sítio de escrita) · nenhum
  termo que dependa da normal dentro da projecção · nenhuma restrição sobre ângulo · **quatro**
  espécies e nem uma quinta · **um** passo de solver por passo de pincel, sem laço de sub-passos ·
  o passo de tempo como valor fixo, num sítio só. Reproduzem-se célula a célula as **quatro** tabelas
  (as `2 × 11` do pen-down, as `2 × 11` a `1R`, as `4 × 11` do máximo da malha, a amplitude e a
  calibração): `0,06543` · `0,12215` · `0,15216` · `0,19698` · `0,28222` · `0,8493` · `15,1 %` ·
  `0,2500`/`0,0625`/`0,6677`/`0,3388` · `0,03272`/`0,19434`; mais a **prova do fatiamento recalculada**
  (`0,000000` nas oito, comparando o bloco `k = 12` com o `.deformado`), as oito contagens de
  `movidos` e os oito `max_deslocamento` dos cabeçalhos, o `indice.json` a regenerar-se
  **byte-a-byte** (73 entradas para 73 ficheiros) e o `verifica_traco.py` **verde sobre os 73**.
  ⭐ **E as DUAS armadilhas que o especificador reportou conferem:** o tecto da massa é mesmo `2` (a
  porta de propriedades declara a faixa `0,01..2`, e é o que a §5.4 já dizia), logo o `4` pedido foi
  coagido; e a régua das excepções do README reproduz-se **exactamente** — `23` de `73`, com a
  repartição `6/6/4/3/2/1/1/1` por grandeza e `16` de `73` fora da área *Local*.
  ⛔ **CINCO curas de fidelidade e de suficiência, aplicadas no acto:** (1) o manifesto da emenda
  dizia tocar a **§5.2** e o diff não lhe põe uma linha; (2) o máximo da malha do Push `_parado`
  vira no passo **`5`**, não no `4` — o `4` é o do Inflate `_parado`, e os três viram em passos
  diferentes; (3) ⛔ o gate 37 convertia resíduo em projecções por `x / 15,1` e **esquecia o vão da
  própria régua** (os `15,1 %` valem `5` projecções), contradizendo o *«uma projecção a mais»* da
  secção que o gera — passa a `5 · x / 15,1`; (4) ⛔ o gate 36 punha a barra em `f32` sobre razões
  lidas de um ficheiro de **seis casas**, e o oráculo lê `0,2499924`/`0,0624943` ⇒ *a barra
  reprovaria a fixture que a define* (passa a `±2·10⁻⁵`, derivada da resolução), e o «ao bit» do
  §10.10 caiu; (5) ⛔ o gate 38 mandava a sequência «descer» depois do máximo e a cauda do `_origem`
  **volta a subir** nos dois últimos passos (o vértice do máximo viaja com o cursor) ⇒ a régua é o
  **argmax**. ⭐⭐ **Mais DUAS de suficiência, que o lado limpo não podia edificar:** a linha `1R` do
  §10.10 não nomeava a sonda, e o vértice **espelhado** dá outros números (`0,00477` contra `0,00421`)
  — ficam os dois de repouso escritos por coordenada, com o achado que vem de graça: *numa cena de
  simetria perfeita a única coisa que separa os dois lados é a ORDEM (§3.1-bis), e isso é uma medição
  a favor da explicação (a) do §5.7-bis*; e o controlo do gate 37 sobe de uma célula para **a malha
  inteira** (no passo 2 as duas áreas dão `máx` da diferença por vértice `= 0,000000` sobre os `4 225`).
  **Veredicto: ATESTADO** — a emenda Q16 pode ser lida pela janela-mãe. Detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q17 de 2026-09-07** (§4.2 linha do Inflate · **§4.2-bis (2) ERRATA + (5) + (8) NOVO** ·
  **§4.2-ter NOVA** · §5.7-bis FECHO · §10 contagem · **§10.11 NOVA** · §14 gate 23 **REVOGADO E
  INVERTIDO** + gates **39-42**, + as TRÊS fixtures `esfera_empurrar_radial_local_1passo` ·
  `esfera_inflar_radial_local_1passo` · `plano_inflar_radial_local_1passo_2tracos`): as três
  perguntas do I devolvem **NÃO** as três — não há diferença no corte `d ≥ R`, nem na banda, nem no
  conjunto de células, e a prova não é um censo mas um **CONTROLO**: um arnês independente que
  implementa a espec tal como estava reproduz o **arrasto** com `err_max = 3,7·10⁻⁶` sobre a malha
  inteira e nos 12 passos, e reproduz o produto do I no **empurrar** a quatro algarismos. ⇒ o defeito
  estava na espec, e na **fase do gesto**: (1) as NORMAIS que o gesto lê — a normal da área do Push e
  a normal por vértice do Inflate — são as da superfície **que o traço encontrou**, não as da
  deformada (a §4.2-bis dizia o contrário, com um ⛔ a proibir a leitura certa); (2) por causa disso,
  o disco de amostragem da normal da área — que mede contra as posições **de agora** — fica **VAZIO**
  assim que a cova passa `R · «Normal Radius» = 0,175`, e o Push **não escreve força nenhuma** nesse
  passo (o padrão de disparo é `2 3 4 5 10` de onze no traço de referência, e **volta** a disparar
  quando o cursor avança). Com as duas, os **dez** traços de empurrar/inflar por passo descem de
  `0,2369`/`0,2446` para `err_max ≤ 5·10⁻⁶` — a resolução do ficheiro. Escrita pelo subagente-E da
  mesma janela, com o fonte reaberto só para estas perguntas e com uma corrida NOVA do oráculo
  (3 execuções que viraram fixture + 3 de sonda).
  ✅ **AUDITADA contra §4.2 por R-pré em 2026-09-07** — contexto novo, independente do subagente que
  a escreveu; leu os dois lados (o fonte por shell). Sweep **verde** sobre a espec emendada + a pasta
  inteira das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes
  caminhos (os hits do histórico são os pré-existentes de 2026-09-05 já adjudicados no ledger; o
  patch do commit da emenda passa **limpo**, sozinho).
  ⛔ **ZERO achados de EXPRESSÃO** — sem trecho, sem nome interno, sem wording de comentário ou de
  manual. ⭐ Conferido de propósito: **nenhum comentário do fonte fala do instante das normais**, logo
  a §4.2-ter é medição e não tradução; e a §5.7-bis FECHO mantém a ordem das PERGUNTAS do INBOX, que
  é onde um censo de ausências escorrega.
  ⭐⭐ **FIDELIDADE: as duas correcções são VERDADEIRAS no fonte, com o mecanismo de CADA UMA
  conferido, e o limiar dos `67` passos reconstruído do zero.** (1) As normais que o gesto lê são as
  da superfície que o traço encontrou porque **quem as refresca é o passo de preparação do objecto
  para edição, e o traço corre-o UMA vez** (a escrita de posições da escultura marca as normais da
  árvore de desenho, ⛔ **não** as da malha) — logo tanto a normal da área do Push como a normal por
  vértice do Inflate saem da mesma fotografia. (2) O disco de amostragem mede contra as posições **de
  agora** porque o ramo que leria as de partida está atrás de uma condição que **este pincel não
  activa**; quando ele fica vazio a resposta é o **vector nulo** e o deslocamento do Push é
  exactamente zero, sem direcção de reserva. ⭐⭐ **O limiar reproduz-se célula a célula**: sobre os
  **sete** traços de empurrar por passo há `67` passos com o cursor em movimento; medindo
  `min_v |p_v(bloco k−1) − c_k|`, ficam **`53` abaixo** com máximo **`0,17313`** e **`14` acima** com
  mínimo **`0,17586`**, e `R · 0,5 = 0,17500` cai no vão — **nem um passo do lado errado**, e os
  quatro padrões de disparo (`2 3 4 5 10` · `2 3 4 5 6 10` · `2..9` · os onze) saem os quatro. ⭐ Na
  esfera o vector recuperado fica a `0,03°` da normal da superfície no cursor contra `17,43°` da
  vista, e no 2.º traço da fixture de dois traços a direcção por vértice bate as normais da malha
  deixada pelo 1.º (mediana `1,7·10⁻⁵`) contra as planas (`0,299`), com a inclinação máxima em
  **`22,4°`**. Verificador **verde sobre os 76**, `indice.json` a regenerar-se **byte-a-byte**
  (76 para 76), a tabela «As corridas» com **76** linhas iguais ao disco, e o censo das excepções do
  README a dar **`24` distintas de `76`** (as linhas somam `25` porque uma fixture é excepção em duas
  grandezas) e **`16`** fora da área *Local*.
  ⛔ **NOVE curas aplicadas no acto pelo R-pré, todas funcionais** — ⚠️ **as três primeiras são a
  mesma espécie: a emenda corrigiu a lei em §4.2/§4.2-bis e deixou-a escrita ao contrário noutras
  secções, que é onde um port a vai ler.** (1) A **§4.6 linha 2** ainda dizia que a normal da área é
  a média das normais **actuais** e que «roda com a vala» — o que roda é o CONJUNTO amostrado, e a
  célula não mencionava que a grandeza **deixa de existir**; (2) a **§4.6 linha 4** dizia «sobre a
  malha ACTUAL» e a leitura do censo dizia «da malha deformada»; (3) o **§4.2-ter** afirmava que
  *nada mais no pincel lê normais* e há um **terceiro leitor** — o recorte *Front Faces Only* do §4.1,
  que obedece à mesma lei e está desligado nos presets, logo o corpus não o observa.
  ⭐⭐ (4) **E o §7 estava CERTO e passou a ser uma divergência não-nomeada: o FILTRO faz o
  contrário** — cada passo dele repete a preparação do objecto para edição, logo ali as normais **são**
  as de agora; *a mesma palavra («Inflate») nomeia duas leis neste documento*, e agora as duas
  dizem-no. (5) O **gate 23** dizia-se «REVOGADO E **INVERTIDO**» e o veredito dele **não** se
  inverte: o que se reavalia a cada passo é o conjunto amostrado, logo um port que congele o vector
  do gesto e empurre em todos os passos continua errado — ler «invertido» reconstrói o defeito que a
  Q17 curou; a razão real de ele sair é ser **ambíguo** (a mutação dele é byte-idêntica à lei correcta
  em todo passo de disco não-vazio). (6) A **régua do resíduo** não estava definida em lado nenhum e
  os gates 39/41/42 penduram-se nela — fica escrita, com o ⛔ de que a barra é o **vão de ordens de
  grandeza** e não o dígito, e com uma régua irmã que **não** depende da queda. (7) A régua do **gate
  40** não dizia sobre que vértices corria o mínimo, contra que posições, nem com que índice de
  cursor — fica escrita como foi reconstruída. (8) A **cova do 1.º traço** era `0,0999` na espec e no
  README e o ficheiro diz **`0,09917`**. (9) Dois totais do README ainda em `73` com `76` no disco.
  ⭐⭐⭐ **E uma DÉCIMA, que é um achado e não uma cura: a fixture NOVA de dois traços RESPONDE em
  parte à pergunta que a §4.6 declarava indecidível** — o PESO da soma por face. Fora do plano as três
  somas deixam de coincidir, e a mediana do desvio dá **uniforme `1,7·10⁻⁵` · ângulo `3,0·10⁻⁴` ·
  área `6,6·10⁻⁴`**. É indicação e não prova (os três passam qualquer barra), mas *quem acrescenta uma
  fixture tem de reconferir as notas que diziam «o corpus não decide isto»* (CLAUDE.md §0.0).
  **Veredicto: ATESTADO** — a emenda Q17 pode ser lida pela janela-mãe. Detalhe: LEDGER §Papel R.
  ⚠️⚠️ **E o INSTRUMENTO que confere os atestados nasceu errado DUAS vezes** (a 1.ª está no INBOX): a
  contagem case-sensitive lia `4` onde a verdade era `8`, e a redacção corrigida **contava-se a si
  própria** (a linha que dizia como contar casava com o próprio padrão). ⇒ o censo honesto não é uma
  contagem, é **por bloco** — nenhuma linha `EMENDA Q<n>` pode ficar sem atestado antes da seguinte
  (⛔ o bloco abaixo NÃO leva cercas de código: este cabeçalho já é um bloco, e uma cerca aninhada
  fecha-o mesmo indentada):
      awk '/EMENDA Q[0-9]+ de/{if(e&&!a)print "SEM ATESTADO: "substr(e,1,50); e=$0; a=0}
           tolower($0) ~ /auditadas? contra .*4\.2 por r-pr/{a=1}
           END{if(e&&!a)print "SEM ATESTADO: "substr(e,1,50)}' docs/3D/cleanroom/SPEC_cloth_brush.md
  Silêncio = tudo atestado. ⛔ Não substitua isto por um número: um número envelhece a cada emenda, e
  a linha que o guarda passa a ser mais uma coisa que alguém tem de lembrar de mexer.
  ✅ **EMENDA Q18 de 2026-09-07** (**§4.2-quater NOVA** · **§4.6 linha 4 FECHADA** · §5.2 · §5.4 ·
  **§5.4-bis NOVA** · §10 contagem · **§10.12 · §10.13 · §10.14 NOVAS** · §11 · §14 gates 43-47, + as
  **cinco** fixtures novas: os dois traços LONGOS de plano
  `plano_arrastar_radial_{local,global}_origem_36passos` — 36 passos, `.deformado` **e** por passo —
  e os **três** por passo de esfera `esfera_{agarrar,gancho,expandir}_radial_dinamica`): as quatro
  perguntas do I devolvem **SIM · SIM (com o mecanismo maior que a pergunta) · SIM (com a premissa
  refutada) · SIM (com uma advertência nova)**. (1) O par de `φ` confirma-se lado a lado no fonte —
  o factor das cinco varreduras **traz** a banda, o da integração **não**, e dentro da integração a
  banda entra **uma vez só**, depois do amortecimento e **antes** do termo de velocidade (§5.4-bis);
  as duas letras deixam de ter o mesmo nome. (2) O peso da soma por face é **uniforme** — e a razão
  é maior que a pergunta: o programa tem **DUAS** leis de normal por vértice, e a que corre no
  caminho da escultura é a soma **sem peso** das normais unitárias das faces; a outra, que só se lê
  numa malha que nunca foi esculpida, é **ponderada pelo ângulo do canto**, e é ela que a esfera do
  corpus traz (§4.2-quater). (3) O traço longo está gravado — ⛔ **e a premissa «um traço mais longo
  é mais fundo» está REFUTADA**: em área *Local* a profundidade **satura** em `0,76 R` com 36 passos
  (contra `0,94 R` com 12), e os `5,4 R` do regime que o I procura são um facto da **ÁREA**, não do
  comprimento (§10.12). (4) Os três por passo de esfera estão gravados — ⛔ **e a esfera não é
  reproduzível**: quatro corridas da MESMA configuração dão saídas que diferem entre si até `0,036`
  (`11 %`–`60 %` do sinal), a divergência nasce no 1.º passo simulado (`4,3·10⁻⁴`) e amplifica
  `~65×`; o plano dá `0,000000`. Cada bloco por passo leva por isso a **banda de realização** ao
  lado, que é a régua sem a qual uma comparação por passo na esfera não quer dizer nada (§10.13).
  Escrita pelo subagente-E da mesma janela, com o fonte reaberto só para estas perguntas e com
  corridas NOVAS do oráculo (**304** execuções: 113 que viraram fixture, 162 do censo de realização
  e da banda, 29 de sonda) mais **11** corridas de leitura de normais.
Mapa de leitura da literatura (⭐ pública e lícita a TODOS os papéis):
  · Jakobsen, "Advanced Character Physics", GDC 2001 — integração de Verlet por posições + relaxação
    de restrições de distância por projecção. É EXACTAMENTE a família do solver do alvo.
  · Müller, Heidelberger, Hennix, Ratcliff, "Position Based Dynamics", VRIPhys 2006 / JVCIR 2007 —
    a restrição de distância (§3.3), a rigidez por iteração (§3.4), o laço (§3.1).
  · Goldenthal et al., "Efficient Simulation of Inextensible Cloth", SIGGRAPH 2007 — contexto sobre
    limitação de esticão (o alvo NÃO a tem; ver §5.7).
  ⛔ Nenhum destes tem apêndice de código do alvo. ⛔ NÃO procurar «PBD cloth brush» em hospedagem de
  código: uma implementação encontrada por esse caminho é, com probabilidade alta, um fork do alvo.
Denylist de URLs (⛔ NÃO abrir): projects.blender.org · developer.blender.org · git.blender.org ·
  github.com/blender/* e qualquer espelho · code-search por «sculpt cloth», «cloth brush solver»,
  «sculpt cloth simulation» · builds/branches de terceiros do Sculpt Mode.
Denylist de CAMINHOS (⛔ os dois estão NESTE disco): ~/Referencias/** (o oráculo, os dumps crus, as
  notas, os rascunhos) · /home/enio/Documentos/Recursos/BlenderSculpt/** (o fonte). ⇒ o Passo 0 do
  BLOCO-I nega `Read` E `Bash` sobre os dois.
"Este documento descreve comportamento; não contém expressão do alvo."
```

> **Como ler.** Cada secção diz **o que o programa faz**, em vocabulário do NOSSO domínio. A
> proveniência de cada número está ao lado dele: **(F)** = lido do fonte como facto de
> comportamento · **(A)** = lido dos *presets* que o artista vê (a biblioteca de pincéis do
> binário) · **(M)** = medido no oráculo (fixture em `fixtures/cloth/`) · **(H)** = história
> (mensagem de commit / blog / issue, com a referência no §9). Um número sem letra não existe aqui.
>
> ⚠️ **As nossas próprias docs anteriores enganam em cinco pontos**, e este documento corrige-os
> no §12 — leia-o antes de reutilizar qualquer frase de [`04`](../cloth/04_espec_do_comportamento.md).

---

## §0 — A leitura em uma linha

**O pincel não é um solver de pano com um pincel em cima: é um solver de RESTRIÇÕES DE DISTÂNCIA
por relaxação (a família Jakobsen/PBD), com integração de Verlet por posições, que corre UM passo
por passo do pincel, sobre um sub-conjunto de células da estrutura espacial da escultura — e o
pincel fala com ele por TRÊS canais só: uma FORÇA por vértice (seis dos oito modos), uma ÂNCORA
por vértice com uma força de restrição própria (Grab · Snake Hook · e todo pincel alheio que
escolha «simulação de pano» como alvo), ou um DESVIO DO COMPRIMENTO DE REPOUSO por vértice
(Expand).** Tudo o resto — a área simulada, a banda graduada, o pino, a plasticidade, a colisão —
é quem decide *quais* restrições existem, *quais* estão activas e *com que peso* cada vértice as
obedece.

⇒ A [auditoria §8-ter](../cloth/03_auditoria_2026-09-05.md) tinha razão no diagnóstico e errava
no remédio: o que separa o nosso pincel do alvo **não é** «escrever alvos por vértice» (o Grab do
alvo faz exactamente isso) — é que (a) **cinco modos são forças e não alvos**, (b) a força entra
numa simulação que **guarda velocidade entre passos** e cujas restrições **estruturais são de
distância entre vizinhos e entre pares de vizinhos** (anel-1 completo), e (c) o solver **nunca
recomeça do zero dentro de um traço**: a posição, a velocidade, o desvio de repouso e as âncoras
sobrevivem de passo a passo.

---

## §1 — Arquitectura de fases (um passo do pincel, por passagem de simetria)

O traço é uma sequência de **passos** (§6.2). A cada passo, e **separadamente para cada passagem
de simetria** (espelho · radial · ladrilho), corre o seguinte, nesta ordem (F):

| # | fase | entra | sai |
|---|---|---|---|
| 0 | **Primeiro passo de uma passagem?** ⇒ com área *Local*, constrói as restrições das células da área (§2, §3) e **termina sem simular** (F) — no 1.º passo de uma passagem o deslocamento do cursor é zero, e os modos que dele tiram direcção, referencial ou alvo não têm resposta definida sem ele. ⚠️⚠️ **E esta construção NÃO marca as células como construídas** — a marca é a **ACTIVAÇÃO** (fase 3), que este passo nunca alcança (F) ⇒ **a fase 1 do passo seguinte constrói-as OUTRA VEZ, e o conjunto de restrições do *Local* fica com CADA restrição DUAS vezes** (§3.1, §5.2-bis: é a origem MEDIDA do factor `~2` de amplitude entre *Local* e *Global*). ⚠️ Com área *Dynamic*/*Global* o 1.º passo também não simula **e não constrói**: as restrições ficam para o 2.º passo e nascem **uma vez só**. | — | células com restrições, ainda **inactivas** |
| 1 | **Garantir restrições** para toda célula do conjunto afectado que ainda não as tenha (§3) | conjunto de células (§2.1) | restrições novas (só para células **ainda não activadas** — uma célula já activada nunca mais é construída; ⚠️ a fase 0 do *Local* constrói **sem** activar, logo essas células são construídas **duas** vezes) |
| 2 | **Guardar o estado**: a posição de simulação de TODOS os vértices ← posições actuais da malha | malha | `x` |
| 3 | **Activar** as células do conjunto afectado | — | células activas |
| 4 | **Aplicar o gesto** (§4): forças → aceleração; ou âncoras + força de âncora; ou desvio de repouso | cursor, delta, normal, factores | `a`, âncoras, desvios |
| 5 | **Passo de simulação** (§5): 5 varreduras de relaxação sobre as restrições das células ACTIVAS; depois, por célula activa: integrar, colidir, **escrever na malha**, e **desactivar a célula** | `x`, `x_prev`, `a` | malha deformada; `x_prev`, `x_colisão` |

⚠️ **Três consequências que decidem o desenho:**
- **A escrita na malha é por passo, não por traço** — o que se vê é sempre o estado da simulação
  (§6.1); não há «preview» separado.
- **Cada passagem de simetria vê o resultado da anterior** (a fase 2 relê a malha), e as restrições
  são partilhadas entre passagens: uma célula tocada por duas passagens tem as restrições das duas.
- **A simulação avança mesmo sem forças**: se o cursor não se mexeu no ecrã, a fase 4 não aplica
  nada, mas a fase 5 corre na mesma (velocidade + restrições) — o pano continua a assentar
  enquanto houver passos (com o método de traço *Dots* dos presets, um passo por evento de rato).

---

## §2 — A área simulada

### §2.1 — De que é feita: CÉLULAS, não vértices

A escultura vive numa árvore espacial de células-folha (a mesma que serve a pintura e o *picking*).
A área simulada é um **conjunto de células**, escolhido a cada passo (F):

| área (rótulo na UI) | conjunto de células | centro `c` da área | raio usado |
|---|---|---|---|
| **Local** *(omissão do código)* | as que intersectam a esfera de centro `c` e raio `R₀·(1+L)` | a **localização inicial** do traço (fixa, por passagem de simetria) — ⚠️ é o ponto da superfície sob o cursor **no último HOVER antes do pen-down** (escrito pelo desenho do cursor, não pelo início do traço: M — um traço scriptado sem hover fica com o centro num ponto velho) | **`R₀` = raio no 1.º passo** (o raio por pressão não muda a área) |
| **Global** | **todas** as células-folha | — | — |
| **Dynamic** *(o que os presets usam — ver §8)* | as que intersectam a esfera de centro `c` e raio `R·(1+L)` | a **localização actual** do cursor | `R` = raio actual (com pressão) |

`L` = *Simulation Limit* (omissão `2,5`, faixa `0,1..10` (F)). O teste célula-esfera é o do ponto
da caixa da célula mais próximo do centro (distância² < raio²), sobre as caixas **actuais** (F).

⭐⭐ **A célula-folha tem no máximo 2 500 FACES (F), logo numa malha pequena há POUCAS células:** a
grelha de 4 225 vértices (4 096 quads) dá **exactamente 2** células, de 2 048 faces cada; a esfera
de 6 050 vértices (6 144 faces — **5 952 quads e 192 triângulos**, os dois anéis dos pólos) dá
**exactamente 4**, de 1 536 faces cada (M, 2026-09-06 — ⛔ a redacção anterior dizia «~2» e «~3», e
o `3` estava errado: a partição inteira, com os vértices próprios de cada célula, está no §3.1-bis e
é fixture). ⇒ a
activação é **grossa** — activar uma célula que a esfera da área toca traz consigo um pedaço grande
da malha. ⚠️ **O que impede esses vértices longínquos de se moverem é o peso de banda `w` DENTRO do
factor por vértice `φ` (§5.2), não a granularidade da célula:** há DOIS portões, e são diferentes —
o **grosso** (a célula inactiva: as suas restrições são saltadas e os seus vértices não são
integrados) e o **fino** (`w = 0` além do limite ⇒ `φ = 0` ⇒ nem a correcção de restrição nem a
velocidade retida movem o vértice, mesmo que a célula dele esteja activa). No plano, o Local pára
**exactamente** no disco de `3,5 R` por causa do portão fino (M — §10).

⭐⭐ **Local contra Dynamic — o que difere no fonte, além do centro fixo e de `R₀` (F):**
- **o centro de TUDO** (a esfera de células, a banda `w`, a distância da força, a retenção de
  velocidade) é a **localização inicial fixa** no Local e a **do cursor, a cada passo** no Dynamic;
- **a criação de restrições é filtrada por raio** no Local (só vértices com `|p⁰ − c| < R₀·(1+L)`, em
  posições de repouso), e **sem filtro** (todos os vértices das células tocadas) no Dynamic/Global;
- as restrições nascem **de uma vez no 1.º passo — e OUTRA VEZ no 2.º**, logo em **DUPLICADO**, no Local (§1 fase 0, §5.2-bis); **incrementalmente e uma vez só**, à medida que
  novas células entram na área, no Dynamic;
- a banda `w` do Local é avaliada com o centro FIXO, logo o fim de um traço longo cai mais longe do
  centro da força ⇒ menos deslocamento lá; no Dynamic o centro segue o cursor e o traço inteiro
  recebe força cheia. ⇒ para o mesmo traço, o Dynamic desloca mais que o Local, e é ≈ Global. (A
  razão exacta **não** é emergente — ver a seguir.)

⭐⭐⭐ **A alavanca DOMINANTE não é o centro fixo: é a lista de restrições DUPLICADA do *Local*
(§1 fase 0, §3.1, §5.2-bis).** O centro fixo e o aro preso explicam o **BORDO** (§10.2), não o
**INTERIOR**: um port com o aro certo e a lista simples lê *Local* ≈ *Global* coluna a coluna no
interior, e no alvo o *Local* entrega no centro `0,34–0,57×` o *Global*, uniformemente ao longo do
traço (M — §10.2 e §10.3).

⚠️ **A célula é a unidade de activação, e isso tem um efeito visível:** todo vértice de uma célula
activa é integrado (fase 5), mesmo que esteja fora da esfera — o que o segura é o **peso de
banda** (§2.2), que vale `0` fora do limite. E uma restrição pertence à célula onde foi CRIADA
(a do vértice de origem), mesmo que ligue a um vizinho de outra célula: ela só é resolvida quando
**essa** célula está activa.

### §2.2 — A banda graduada (*Simulation Falloff*) — a fórmula

Peso de banda `w(p)` de um ponto `p`, para as áreas *Local* e *Dynamic* (F):

```
limite   = R·(1 + L)
início   = R·(1 + L·F)          F = Simulation Falloff, omissão 0,75, faixa 0..1
d        = |p − c|
w(p) = 1                                 se d < início
     = 0                                 se d > limite
     = 3t² − 2t³,  t = 1 − (d − início)/(limite − início)     na banda   (smoothstep)
```

Para *Global*, `w ≡ 1`. Para qualquer pincel que **não** seja o pincel de tecido mas use a
simulação como alvo (Pose/Boundary), `w ≡ 1` (F — eles não têm área; H: D8885).

⚠️ **Três leitores, DOIS pontos de avaliação diferentes — e isso é load-bearing:**

| quem lê `w` | avaliado em que POSIÇÃO | com que `R` |
|---|---|---|
| a força do gesto (§4) | posição **actual** do vértice | raio actual |
| o factor por vértice das restrições (§5.2) | posição de **REPOUSO do traço** (a de quando a simulação nasceu) | raio actual |
| a retenção de velocidade (§5.4) | posição de **repouso do traço** | raio actual |

⇒ Um vértice que a simulação arrasta para fora da banda **continua** a obedecer às restrições e a
reter velocidade com o peso do sítio de onde partiu; só a força nova o deixa de tocar.

⛔⛔ **ERRATA de 2026-09-06 (F+M) — esta linha dizia `1,875·R` a `2,5·R` e estava ERRADA por um
factor de `1,4`.** As duas fronteiras são `R·(1 + L)` e `R·(1 + L·F)`, exactamente como o bloco de
fórmula acima já diz, e com os valores de omissão (`L = 2,5`, `F = 0,75`) isso dá **início
`2,875·R`** e **limite `3,5·R`** — largura `0,625·R`, que é a única coisa que a redacção anterior
acertava. *A conta errada era `R·L` e `R·L·F`: ela esquece o `R` que abre os dois parênteses.*
⚠️ **E é MEDIDO, não derivado** (M; números reconstruídos do zero pelo R-pré em 2026-09-06, régua =
`|u| > 1e-5`, a mesma que o cabeçalho de cada fixture usa para o campo `movidos`): nas **26** fixtures
*Local* de 12 passos do plano com `limite = 2,5`, o vértice movido mais distante do pen-down cai em
**`3,492·R`–`3,497·R`** (`1,2221`–`1,2239`) em **25** delas — encostado a `3,5·R = 1,2250` e **muito
para lá** de `2,5·R = 0,8750`. ⚠️ A 26.ª é o **controlo de força fraca**
(`plano_apertar_ponto_radial_local_origem_fraco`, `3,456·R`), cujo maior deslocamento vale `0,004` e
cuja franja da banda **desaparece na resolução do ficheiro** — ⛔ ela não desmente a fronteira, mede
a resolução. As três fixtures que este bloco nomeia dão `1,2221` (`plano_expandir_radial_local`),
`1,2234` (`plano_agarrar_radial_local`) e `1,2234` (`plano_gancho_radial_local`); ⚠️ o valor `1,2239`
existe no corpus mas é de **outras** fixtures (o arrastar, os dois apertos, o empurrar, o inflar).
⭐⭐ **E há um SEGUNDO ponto da recta, que é o que torna a errata irrefutável:** a fixture
`plano_agarrar_radial_local_preset` corre com **`limite = 5,0`** e o seu vértice movido mais distante
está a **`2,0745`** — `0,99·R(1+L) = 2,10`, e **`+19 %` para lá de `R·L = 1,75`**. *Duas leituras da
mesma fórmula não podem sobreviver a dois valores de `L`: `R(1+L)` acerta nos dois (`1,00×` e
`0,99×`), `R·L` erra por factores diferentes (`1,40×` e `1,19×`).*
⇒ *um port com a leitura antiga simula um disco `40 %` mais pequeno e põe `φ = 0` num anel inteiro
onde o alvo ainda tem `φ = 1`.*

### §2.3 — O pino opcional (*Pin Simulation Boundary*)

Só existe para a área **Local** (a UI só o mostra aí; o código recusa-o em *Dynamic*) e nasce
**desligado** (F). Ligado, ao construir as restrições de uma célula, todo vértice cuja banda
`w < 1` (avaliada na localização **actual** do cursor e no raio inicial) recebe uma **restrição de
âncora à posição de repouso do traço** com força `1 − w` (§3.2). ⇒ na banda o vértice é puxado de
volta ao sítio com força crescente até `1` no limite; fora do limite (`w = 0`) o pino é total.
(H: D8435 — nasceu porque forças grandes, anchored ou com pinch/grab a força máxima, rompiam a
simulação na fronteira; é opção porque, num pincel que deforma a malha inteira, o pino adiciona
restrições indesejadas.)

### §2.4 — A fronteira TOPOLÓGICA da malha

⛔ **Não é do pincel de tecido.** O que a nota de versão de 2.83 chama «proteger as fronteiras da
malha ao esculpir tecido» é o **auto-mascaramento de arestas de fronteira** da casa (opção
genérica de todo pincel), com o seu número de *passos de propagação*: ele entra no pincel de tecido
como mais um **factor por vértice** (§4.1), em TODOS os sítios onde o mascaramento entra — forças,
restrições e integração. O pincel de tecido não tem nenhum código próprio de fronteira.
A lei dele, lida no módulo de auto-mascaramento (F): com `N` = *passos de propagação*
(omissão `1`), calcula-se para cada vértice a distância `k` em ARESTAS até ao vértice de fronteira
mais próximo (busca em largura, `N` rondas; quem fica a mais de `N` passos não é tocado), e o
multiplicador do vértice é `1 − (1 − k/N)²` — `0` na própria fronteira, `1` a `N` passos, rampa
quadrática entre os dois. Tudo o que este documento chama «auto-máscara» já o inclui, e ele entra
nos três sítios (forças, restrições, integração).

---

## §3 — As restrições

### §3.1 — Estruturais: o padrão de vizinhança

Ao construir uma célula, para cada vértice **visível** `v` da célula que esteja dentro do raio de
construção (*Local*: `|p⁰(v) − c| < R₀·(1+L)`, avaliado em posições de repouso; *Dynamic*/*Global*:
todos) (F):

1. uma restrição de distância `(v, n)` para cada vizinho topológico `n` de `v`;
2. uma restrição de distância `(a, b)` para **cada par ordenado de vizinhos distintos** `a ≠ b` de `v`.

⭐⭐ **O anel-1 é o das ARESTAS DAS FACES POLIGONAIS, não de uma triangulação (confirmado por leitura
do fonte, 2026-09-06):** o vizinho de `v` é, por cada face que o contém, os DOIS cantos adjacentes a
`v` NAQUELA face (o anterior e o seguinte), deduplicados. ⇒ **numa grelha de quads um vértice
interior tem exactamente 4 vizinhos** (N, S, E, O) — ⛔ **não** 6, e **nenhuma diagonal do quad é
vizinha**. Numa malha de triângulos regular são 6. A diagonal só aparece como restrição de PAR
(passo 2): dos 4 vizinhos de um vértice de grelha saem `4·3 = 12` pares ordenados ⇒ 6 não ordenados
= as 2 «diâmetros» N-S / E-O (comprimento `2h`, o papel de dobra) + as 4 diagonais N-E… (`√2·h`,
cisalhamento). O sistema de escultura NÃO triangula a malha para escolher vizinhos.

Cada par não ordenado entra **uma vez por CONSTRUÇÃO** — o registo de pares já criados nasce e morre **dentro de uma construção** (a fase 1 de um passo), e não é partilhado entre construções (F) ⇒ ⚠️⚠️ **duas construções sobre a mesma célula deixam DUAS cópias de cada par**, que é exactamente o que a área *Local* faz (§1 fase 0, §5.2-bis). Dentro de UMA construção não há duplicados (H:
D8007 curou a duplicação). ⭐ **O que isso produz** (grelha de quads, vértice interior de 4 vizinhos):
as **4** arestas (estrutural) + as **2** «diagonais longas» N-S / E-O (dobra, `2h`) + as **4**
diagonais N-E… (cisalhamento, `√2·h`) — o «4 + 2 + 4» do gate 8. Numa malha de triângulos (6
vizinhos): as 6 arestas + os pares do anel — os 6 pares vizinhos são arestas do anel (já estruturais
de outro vértice) e os restantes atravessam-no (3 «diâmetros» `2h` + 6 cordas `√3·h`). ⇒ **a rigidez
de dobra NÃO tem modelo próprio: é a restrição de distância ao segundo vizinho pelo anel.** Os
autores chamam-lhe, por escrito, «básico» e sabem que repete restrições (H: D6715).

- **Comprimento de repouso:** a distância entre os dois vértices nas **posições de repouso do
  traço** — ou nas posições da **base persistente** se o pincel estiver em modo *Persistent* e
  ela existir (§6.4). Força `1`.
- **Ordem:** as restrições ficam na ordem de criação e são resolvidas **nessa ordem**,
  sequencialmente (Gauss–Seidel). A ordem de criação é determinística dada a ordem das células (F).
  ⚠️⚠️ **E ela é EXACTAMENTE esta, o que só passou a ser preciso quando se soube que ela decide o
  resultado (§5.2-ter)** (F, 2026-09-06):
  1. **célula a célula**, por **ordem CRESCENTE do índice da célula** no vector de células da árvore
     (F, 2026-09-06). ⭐⭐ **A busca ORDENA o conjunto que devolve antes de o entregar** — a
     ordenação é feita de propósito, não é um efeito colateral ⇒ **a ordem das células não
     depende do cursor** (ele decide QUAIS células entram, nunca a ordem relativa delas), nem de
     paralelismo, nem da corrida. E a varredura *Global*, que leva todas as folhas, é construída
     percorrendo o vector por índice crescente, logo dá a MESMA ordem relativa. ⇒ ⛔ *«a ordem em que
     a busca as devolve» não é uma ordem opaca: é uma ordenação por inteiro, e é reprodutível.*
  2. dentro da célula, os **vértices PRÓPRIOS e VISÍVEIS** dela — ⛔ **não** «todos os vértices que
     as faces dela usam» —, por **ordem CRESCENTE de índice de vértice** (F, 2026-09-06). Os
     escondidos são **retirados** dessa lista, preservando a ordem relativa dos que ficam (⚠️ é uma
     filtragem, ⛔ nunca uma reordenação); com nada escondido, a lista é a dos próprios, tal e qual.
     ⚠️ Cada vértice é próprio de **exactamente UMA** célula (§3.1-bis) ⇒ a construção visita cada
     vértice **uma vez só**, e a sequência global de vértices é uma **PERMUTAÇÃO** de `0..N−1`
     agrupada por célula — ⛔ **não** a ordem crescente global, nem no plano (§3.1-bis);
  3. dentro de um vértice, **nesta ordem**: o **corpo mole** (se houver plasticidade) · as
     `(v, n)` para cada vizinho do anel, **na ordem do anel** · as `(a, b)` de cada par ordenado de
     vizinhos distintos, na mesma ordem do anel · a **âncora de deformação** · o **pino**.
     ⛔ O corpo mole vem ANTES das estruturais e o pino DEPOIS da âncora — a espec anterior não
     dizia a ordem interna, e ela é observável assim que duas espécies disputam um vértice.
  4. o registo de pares já criados é **partilhado pela construção inteira** — é o MESMO registo do
     parágrafo acima: **um por CONSTRUÇÃO**, comum a todas as células dela, e é por isso que duas
     construções deixam duas cópias. Logo a **primeira** ocorrência de um par fixa a posição dele na
     lista, e um par cujos dois vértices vivem em células diferentes cai na célula que lá chegou
     primeiro.
  ⚠️ **O anel de um vértice é percorrido face a face** (para cada face que contém `v`, os dois cantos
  adjacentes, deduplicados), logo a ordem do anel é a ordem das faces à volta do vértice — não uma
  ordem angular nem a dos índices de VÉRTICE (F). ⭐⭐ **E «a ordem das faces» é, por inteiro
  (F, 2026-09-06):**
  - as faces incidentes em `v` vêm por **ordem CRESCENTE de índice de FACE** — a tabela
    vértice→faces é construída **em paralelo** e depois **ordenada**, e é essa ordenação que lhe
    tira a corrida ⇒ ⛔ *não é a ordem de chegada das threads, e não é implementação-definida*;
  - de cada face saem **dois** cantos, na ordem **anterior, depois seguinte** segundo o **sentido de
    percurso (winding)** da face;
  - a deduplicação guarda a **PRIMEIRA** ocorrência (uma face posterior que reapresente um vizinho
    já visto **não** o move para o fim).
  ⇒ ⭐ **Numa grelha de quads a lei DEGENERA na ordem crescente de índice de vértice** (o vértice
  interior `2112` da grelha das fixtures tem faces `[2015, 2016, 2079, 2080]` e anel
  `[2047, 2111, 2113, 2177]`, que já é crescente) — e **na esfera não** (o vértice `3025` tem faces
  `[3059, 3061, 3118, 3120]` e anel `[3024, 2962, 3026, 3088]`, contra `[2962, 3024, 3026, 3088]`
  da ordem de índice). ⚠️ **Um pólo da esfera tem `96` faces incidentes** e a lista delas **não é
  contígua** no vector de faces (`33, 66, 189, 247, …`) ⇒ a ordem do anel de um pólo não é
  adivinhável: sai da **lista de faces**, que é agora fixture (§3.1-bis). (M, 2026-09-06)
- ⚠️ **O filtro de raio da construção só vale para as ESTRUTURAIS e para o corpo mole** (F,
  2026-09-06): a **âncora de deformação** e o **pino** ficam **FORA** dele — mas ⛔ isso **não** quer
  dizer «sem condição nenhuma»: cada uma tem a **sua**, e nenhuma delas é o raio da construção.
  A âncora radial do Grab usa o **raio do PINCEL** (`d⁰ < R₀`, filtro próprio e diferente); a do
  Grab-plano e a do Snake Hook não têm condição de distância nenhuma (todo vértice visível da
  célula); e o **pino** tem a dele, que é a **banda** (`w < 1`, §2.3) e só existe com a opção ligada
  e fora da área *Dynamic*.
  ⇒ um port que filtre as quatro espécies pelo mesmo raio cria menos âncoras e menos pinos do que o
  alvo. ⚠️ **Na área *Local* isso é observável só pela ORDEM da lista**, porque o factor por vértice
  dessas restrições extra é `0` (§5.2: `σ = 0` fora do conjunto que o gesto reescreve; `φ = 0` além
  do limite da banda) — mas a lista fica com outro comprimento e outra ordem, e isso basta (§5.2-ter).

### §3.1-bis — A PARTIÇÃO EM CÉLULAS: a lei, e as duas fixtures que a fixam (F + M, 2026-09-06)

⭐⭐⭐ **O problema que esta secção fecha:** a ordem de criação da §3.1 é *célula → vértice próprio →
anel*, e nenhuma das três se lê das posições de repouso. O lado limpo reconstrói a malha das
fixtures **por posição**, logo tem os índices de VÉRTICE do alvo e **não** tem nem a lista de faces
nem a partição em células. ⇒ a lei era **inaplicável fora do plano**, onde ela degenera por acidente.
As duas fixtures novas dão-lhe as duas coisas que faltavam.

#### A lei da partição (F)

A árvore é uma bissecção espacial das **faces**, construída uma vez por malha:

- **centro de uma face** = o **ponto médio da caixa** dos vértices dela (⛔ **não** o centróide);
- a **raiz** recebe a caixa da malha inteira (a fusão das caixas das faces); todo nó **interior**
  recalcula a sua caixa como a **caixa dos centros** das faces que lhe cabem;
- um nó é **folha** quando tem **≤ 2 500 faces** (ou quando a profundidade atinge `99`);
- caso contrário parte-se pelo **eixo de maior extensão** da caixa (empate ⇒ o eixo de índice mais
  alto: `X` só ganha se for estritamente maior que os outros dois; entre `Y` e `Z` empatados ganha
  `Z`), no **ponto médio** desse eixo, e uma face vai ao **primeiro** filho sse
  `centro[eixo] ≥ limiar`.
  ⚠️ **A regra do EMPATE tem meia prova, e a metade que falta é nomeada** (R-pré, 2026-09-06): a
  caixa do plano das fixtures empata `X` com `Y` (`3,0` e `3,0`, com `Z = 0`) e a partição medida
  parte por **`Y`** — as duas células são as duas metades em `y`, e a de índice **`1`** é a do lado
  `≥`, o que confirma **as três** cláusulas seguintes de uma vez. ⛔ **O desempate `Y` ↔ `Z` não é
  decidido por fixture nenhuma deste corpus**, e a função que o resolve **não vive na parte do fonte
  que esta linha tem** ⇒ trate-o como a única cláusula desta secção **sem prova deste lado**;
- os dois filhos são reservados **em par**, no fim do vector de nós, e o **primeiro é o lado `≥`**;
  a recursão desce **primeiro** no filho `≥` ⇒ os índices são atribuídos em **profundidade primeiro**.

⚠️ **A ordem interna das faces dentro de uma folha é implementação-definida** (a partição não é
estável) **e não é observável**: o que sai dela é sempre um CONJUNTO, e o passo seguinte ordena.

Depois, por folha, e é aqui que nasce a ordem de visita:

- o **conjunto** de vértices usados pelas faces da folha, **ordenado por índice crescente**;
- percorrendo as folhas por **índice crescente**, cada uma **reclama** os vértices que ainda não
  foram reclamados: esses são os **próprios** dela; os restantes são **partilhados** e ficam a cargo
  de outra folha.
⇒ ⭐ **os vértices próprios PARTICIONAM a malha** (a soma bate `N` exactamente, medido nas duas
fixtures), e a concatenação `folha 0 → folha 1 → …` dos próprios de cada uma é **a ordem de visita
da construção de restrições**, da integração e de toda a fase que a §1 escreve «por célula, por
vértice».
⚠️ **A partição é sobre os vértices que alguma FACE usa** — um vértice solto não pertence a célula
nenhuma, logo **nunca é simulado nem integrado**, sob área nenhuma (F). Nas duas malhas das
fixtures não há nenhum (a soma dos próprios bate `N`), mas o caso existe e é comportamento de borda.

#### O que isso dá nas duas malhas das fixtures (M)

| malha | vértices | faces | nós | folhas | faces por folha | próprios por folha | ordem de visita = crescente? |
|---|---|---|---|---|---|---|---|
| plano `64×64` | `4 225` | `4 096` | `3` | **`2`** | `2 048` · `2 048` | `2 145` · `2 080` | ⛔ **não** |
| esfera `96×64` | `6 050` | `6 144` | `7` | **`4`** | `1 536` × 4 | `1 569` · `1 520` · `1 504` · `1 457` | ⛔ **não** |

⭐⭐ **No PLANO a ordem é a identidade RODADA:** a folha de índice `1` fica com `[2080..4224]` e a de
índice `2` com `[0..2079]`, as duas **contíguas** ⇒ a sequência de visita tem **um único descenso**,
exactamente no meio da grelha — que é onde o pen-down das fixtures `_origem` está. ⇒ ⛔ *a presunção
«no plano a ordem das células é benigna porque a construção acaba na mesma lista» está **REFUTADA**:
a lista acaba com o mesmo CONJUNTO e com outra ORDEM, e a §5.2-ter já mostrou que a ordem decide o
resultado assim que a malha se inverte debaixo do cursor.*

⭐⭐ **Na ESFERA as quatro folhas são entrelaçadas** (nenhuma é um intervalo de índices: a folha `3`
vai de `0` a `6 043` e a folha `4` de `20` a `6 049`) ⇒ **três descensos**, e a ordem não é
adivinhável de nenhuma regra simples. É por isso que ela é fixture e não prosa.

⚠️ **A partição é da MALHA, não do traço** — ela é calculada uma vez, sobre as posições de
**repouso**, e não muda durante o traço (as caixas actualizam-se; a pertença não). O cursor entra só
no passo que escolhe QUAIS células ficam activas (§2.1).

#### As duas fixtures

- **`plano.faces.txt.gz` · `esfera.faces.txt.gz`** — a lista de faces na ordem de armazenamento, com
  os índices de vértice que as fixtures de repouso já usam (`f i j k …`, uma face por linha, na
  ordem de percurso da face). É o que torna a lei do anel aplicável.
- **`plano.celulas.txt.gz` · `esfera.celulas.txt.gz`** — as células por índice crescente; por
  célula, `cv <i> <n> <vértices próprios, em ordem de visita>` e `cf <i> <m> <faces da célula>`.
  A linha `cf` existe para o lado limpo poder calcular a **caixa** de cada célula e aplicar o teste
  de intersecção do §2.1 sozinho, sem ter de reconstruir a árvore.

⚠️ **As duas malhas foram conferidas contra as fixtures de repouso que já existiam, vértice a
vértice, na ordem de índice: diferença máxima `0,0` às seis casas do ficheiro** ⇒ os índices são os
mesmos, e é lícito cruzar as quatro fixtures novas com qualquer traço do §10.

#### ⛔⛔ O plano NÃO degenera: a divergência existe e está TAPADA pela banda (M, 2026-09-06)

Contando, vértice a vértice, quantos anéis a **ordem de face** entrega diferentes do que a **ordem
de índice de vértice** entregaria:

| malha | vértices | anéis divergentes | onde |
|---|---|---|---|
| plano | `4 225` | **`128`** (`3,0 %`) | **só no bordo** — `3 969` vértices interiores (4 vizinhos) saem todos crescentes, e dos `256` do bordo divergem exactamente metade |
| esfera | `6 050` | **`5 959`** (`98,5 %`) | em toda a parte; e os dois pólos têm `96` vizinhos cada |

⭐⭐ **E é por isso que o plano «não se mexeu um bit» quando a lei do anel foi corrigida — não porque
ela degenere, mas porque os `128` vértices onde ela NÃO degenera estão todos fora da banda:** o mais
próximo está a `1,5` do pen-down e a banda de `limite = 2,5` acaba a `3,5 R = 1,2250` ⇒ `0` dos
`128` são alcançados, e `φ = 0` apaga a diferença (§5.2). ⇒ ⛔ *«o corpus não discrimina» é um facto
sobre a BANDA das fixtures, não sobre a lei.*

⭐⭐⭐ **E há UMA fixture de plano que discrimina, e ela já existe:**
`plano_agarrar_radial_local_preset` corre com `limite = 5,0`, logo a banda acaba a
`R(1+L) = 2,1000` e alcança **`126` dos `128`**. *Um port com o anel por índice tem de divergir
nessa fixture e em nenhuma outra do plano.*

⚠️⚠️ **DOIS `2 145` diferentes, e confundi-los inverte uma leitura:** a célula de índice `1` do
plano tem `2 145` vértices próprios **e** a banda de `3,5 R` contém `2 145` vértices — são
**conjuntos distintos** (intersecção `1 099`), e o segundo é que é o `movidos` do índice das
fixtures. *A coincidência é do tamanho, não do conjunto.*

⚠️⚠️ **A malha das fixtures NÃO está reorganizada espacialmente.** A aplicação de referência tem um
gesto que **reordena** a malha para que os próprios de cada célula fiquem contíguos — e, nessa
malha, a ordem de visita **passa a ser** a crescente global. Esse gesto é um comando explícito do
artista (e um efeito colateral de remalhar), **não** acontece ao entrar em escultura, e o arnês não
o corre ⇒ ⛔ *um port que assuma «os vértices são visitados por índice» está a implementar o caso
reorganizado, e as fixtures são o caso normal.* ⭐ **É também o que torna esta secção MEDIDA e não
derivada: correr esse gesto sobre a mesma malha devolve uma permutação observável de Python, e ela
bate a partição derivada aqui EXACTAMENTE, nas duas malhas** (§10.9).

### §3.2 — As quatro espécies, e o que cada uma liga

Toda restrição é «distância entre o ponto A e o ponto B tem de valer `ℓ`», com A sempre um
vértice da malha (F). As quatro espécies diferem em **quem é B** e em **`ℓ`**:

| espécie | B | `ℓ` | força `s` | quem cria |
|---|---|---|---|---|
| **estrutural** | outro vértice | distância de repouso | `1` | §3.1 |
| **âncora de deformação** | um ponto por vértice que o gesto escreve (§4.3) | `0` | ver §4.3: `0,1·fade` (Grab radial) · `0,1` (Grab plano) · `0,35` (Snake Hook) · `0,01` (pincel alheio com alvo = simulação) | Grab, Snake Hook, Pose/Boundary |
| **corpo mole** (plasticidade) | um ponto por vértice, «memória de forma», que nasce na posição de repouso do traço | `0` | `1` (o efeito é repartido pela plasticidade, §5.3) | quando *Soft Body Plasticity* `> 0` |
| **pino** | a posição de repouso do traço | `0` | `1 − w` | §2.3 |

⚠️ **Uma restrição de âncora só puxa o vértice** (B não é um vértice; só A se move). E a força de
uma âncora de deformação é ainda **multiplicada, no solver, por um segundo factor por vértice**
(§5.2) que o Grab fixa a `1` (radial) ou ao seu peso de plano, e que o Snake Hook **reescreve a
cada passo** — é *isso* a «força ajustada a cada passo do pincel» da documentação.

⭐⭐⭐ **ONDE VIVE O PONTO `B`, E QUEM LHE TOCA DURANTE A VARREDURA** (F, 2026-09-06 — a resposta à
pergunta *«o alvo é recalculado dentro da varredura ou fixado uma vez por passo?»*). Nenhuma das
duas pontas é **fotografada na construção**: cada uma é lida, **no instante da projecção**, da
gaveta por-vértice a que pertence. Logo a resposta não é «fixo» nem «recalculado» — é **quem mais
escreve naquela gaveta durante as varreduras**, e são quatro respostas diferentes:

| espécie | de que gaveta é `B` | quem escreve nessa gaveta durante as 5 (ou 10) projecções |
|---|---|---|
| **estrutural** | a posição de TRABALHO do outro vértice | **a própria varredura** ⇒ `B` é **vivo** (é isto que faz o laço ser Gauss–Seidel e não Jacobi) |
| **âncora de deformação** | a gaveta de âncora daquele vértice | **ninguém** — o gesto escreve-a uma vez por passo (§1 fase 4) e o solver nunca lhe toca ⇒ **fixa durante o passo inteiro** |
| **corpo mole** | a memória de forma daquele vértice | **a própria varredura** — cada projecção puxa a memória para o vértice na fracção `1 − ρ` ⇒ **o alvo ANDA entre projecções** |
| **pino** | a posição de repouso do traço | **ninguém** ⇒ **fixa** (e é a mesma gaveta que a construção lê para os comprimentos de repouso) |

⚠️ **O mesmo vale para `A`:** ele é sempre a posição de trabalho do vértice, lida no instante da
projecção. ⇒ *uma implementação que fotografe as posições no início de uma varredura e só as
publique no fim é um Jacobi e diverge do alvo já na primeira varredura de qualquer passo em que a
malha não esteja em repouso.*
⚠️ **As duas gavetas por-vértice do modo de âncora nascem com valores que não são zero:** a gaveta
de âncora nasce igual à **posição de repouso do traço** e o factor por passo `σ` nasce a **`1`** (F)
— e só existem se o pincel for um dos dois modos de âncora, ou se um pincel alheio estiver a usar a
simulação como alvo. ⇒ num pincel de FORÇA elas não existem, e não há âncora nenhuma.
⚠️ **Esta tabela é (F) e NÃO se reconstrói de uma fixture** (R-pré, 2026-09-06) — a saída do oráculo
mostra só o resultado das quatro leis somadas. A régua do lado limpo é o **gate 27**, que corre sobre
restrições isoladas do NOSSO motor; ⭐ o corpus dá-lhe **três** apoios laterais, e vale nomeá-los
porque são os únicos traços em que estas espécies aparecem de todo: `plano_arrastar_radial_local_plast05`
e `plano_arrastar_radial_dinamica_preset` (plasticidade `0,5` ⇒ corpo mole vivo) e
`plano_arrastar_radial_local_pino` (pino ligado). *Nos outros 62 traços o corpo mole e o pino não
existem, logo nenhum deles pode acusar um port que os escreva ao contrário.*

### §3.3 — O que a construção NÃO faz

- Não cria restrições para vértices escondidos (F). Vértices **mascarados** entram nas restrições
  como vértices normais — o que os pinta como «pregados» é o factor por vértice `1 − máscara` que
  multiplica toda correcção e todo movimento deles (§5.2, §5.4): máscara `1` ⇒ imóvel.
- Não limita o número de restrições por vértice (F).
- Não separa estrutural de cisalhamento de dobra: é UM tipo com `ℓ` diferente.
- Não reconstrói uma célula **depois de ela ter sido ACTIVADA** — a partir daí ela é final (F).
  ⚠️ **Antes disso reconstrói:** a fase 0 do *Local* constrói e não activa, logo o passo seguinte
  constrói a mesma célula de novo e as restrições ficam em duplicado (§1, §5.2-bis).

---

## §4 — O gesto, modo a modo

### §4.1 — O factor por vértice `f` que multiplica QUALQUER gesto

Para cada vértice de cada célula afectada, o PRODUTO dos factores abaixo (F — só uma ordem importa:
a dureza remapeia a distância ANTES de a curva a ler; e a gravidade entra a meio, §4.2):

```
f = (1 − máscara) · (0 se escondido) · (0 se fora dos planos de recorte da vista)
  · w(posição actual)                                  ← banda (§2.2), com o raio actual
  · (0 se virado para trás e «Front Faces Only»)
  · (0 se distância ≥ R)                                ← o corte duro no raio do pincel
  · curva(distância remapeada pela dureza, R)           ← a curva de falloff do pincel
  · auto-máscara · textura do pincel
  · B                                                  ← a força do pincel (abaixo)
```

- **distância**: para todos os modos excepto Grab, do vértice **actual** ao cursor (esférica; ou no
  plano da vista se a forma do falloff for *Projected*); para o **Grab**, da posição de **repouso do
  traço** ao cursor (o Grab mede tudo na malha de partida). Com **Force Falloff = Plane** a
  distância passa a ser `|distância assinada ao plano|` (§4.4).
- **dureza** `h`: distância `< h·R` ⇒ `0`; senão remapeada linearmente para `[0, R]` (F).
- **curva**: os presets da casa (*Smooth* = `3u²−2u³` com `u = 1 − d/R`; *Sharp* = `u²`; *Root*,
  *Linear*, *Constant*, *Sphere*, *Pow4*, *InvSquare*, *Smoother*, ou a curva desenhada) (F).
- **`B`, a força do pincel** — com `α = força²` (a UI mostra `força`; o alvo eleva ao quadrado para
  dar sensibilidade aos valores baixos), `flip = ±1` (direcção *Add/Subtract* × Ctrl),
  `pressão`, `overlap` (`1` salvo atenuação de espaçamento) e `feather` (simetria) (F):

| modo | `B` |
|---|---|
| Drag · Push · Pinch Point · Pinch Perpendicular · Inflate | `10 · α · flip · pressão · overlap · feather` |
| Grab | `força · feather` (⚠️ sem quadrado, sem pressão: «o mesmo falloff de um Grab normal») |
| Snake Hook | `força · feather · pressão · overlap` |
| Expand | `0,1 · α · flip · pressão · overlap · feather` (H: «Expand é mais sensível à força porque continua a expandir ao passar sobre os mesmos vértices») |

⭐ **Consequência de escala:** a força dos cinco modos de força vale, no centro do pincel e a força
máxima, `10` por passo, e o solver converte-a em `10 · dt / massa = 0,1 / massa` unidades de
DESLOCAMENTO por passo (§5.4) — **um número absoluto, independente do raio e da malha**. O único
modo cuja força cresce com o raio é o Push (§4.2). (F; o oráculo confirma a ordem de grandeza —
§10.)

### §4.2 — Os modos de FORÇA (seis): direcção, referencial, sinal

A força de um vértice é `F = f · u`, com `u` o vector abaixo; ela entra como `a += F / massa` (F).

| modo (rótulo) | `u` — para onde | referencial | o que o *Force Falloff = Plane* muda |
|---|---|---|---|
| **Drag** | ⭐ **a direcção UNITÁRIA do movimento do cursor entre este passo e o anterior** — a MESMA para todos os vértices. ⛔ NÃO é «para o ponto do cursor» | espaço do objecto (o cursor é re-apanhado na superfície a cada passo) | só a forma de `f` (faixa em vez de disco) |
| **Push** | ⭐ **`− n̂_área · 2R · escala`** — para DENTRO, ao longo da **normal da área** do pincel, com magnitude proporcional ao raio (`escala` corrige objectos com escala não-uniforme) | normal da área (média das normais sob o pincel; congelável com *Original Normal*) | idem |
| **Pinch Point** | vector unitário do vértice **para o cursor** (actual) | objecto | ⭐ **muda o alvo**: `u` passa a ser o unitário **para o PLANO** (perpendicular ao plano, com o sinal da distância assinada) |
| **Pinch Perpendicular** | do unitário «vértice → cursor», **só as componentes** ao longo de `x̂ = n̂ × d̂` (perpendicular ao traço, no plano tangente) e de `ẑ = n̂` — a componente ao longo do traço é descartada ⇒ converge para a **LINHA** do traço e para o **plano** dela. Não re-normalizado (magnitude ≤ 1) | referencial local do traço (§4.4) | só a forma de `f` |
| **Inflate** | a **normal do vértice** — ⚠️ **da superfície que o TRAÇO ENCONTROU**, não da deformada (§4.2-ter; a redacção anterior dizia «actual» e está **refutada por medição**) | objecto | só a forma de `f` |
| **Gravidade** (todos os modos) | `− ĝ · g`, com `ĝ` a normal +Z do objecto de gravidade (ou +Z do mundo) trazida ao espaço do objecto, `g` = *Gravity* da escultura (omissão **`0`** — M, instalação limpa) | mundo → objecto | — |

⚠️ **A gravidade é aplicada ANTES do corte no raio e da curva**: o seu factor é só
`(1 − máscara) · recorte · w` ⇒ actua na **área simulada inteira**, não no disco do pincel
(H: D8406 — «para a maioria dos modos faz mais sentido aplicar gravidade a toda a simulação»).

⚠️ **Nenhuma força é aplicada num passo em que o deslocamento do cursor no ECRÃ seja zero** (F) —
o teste é sobre o delta de agarrar (§4.3), não sobre a posição 3D.
⚠️⚠️ **Mas o SOLVER corre na mesma, e esse passo NÃO é um no-op** (F, 2026-09-06): o guarda desiste
da fase do gesto, não do passo — as varreduras e a integração correm a seguir como em qualquer outro
passo. ⇒ nesse passo a relaxação dá **mais 5 (ou 10) projecções** sobre uma malha já deformada e a
integração transporta a velocidade de Verlet. ⭐ E há uma segunda consequência, porque a **zeragem do
factor por passo das âncoras acontece DEPOIS do guarda**: num passo parado o `σ` e a gaveta de âncora
ficam com os valores do passo ANTERIOR, e a âncora continua a puxar para o alvo velho. *Parar a mão
não pára o pano — é isto que faz um traço com pausas continuar a assentar.*

⭐⭐⭐ **A MAGNITUDE DOS DOIS APERTOS NÃO DECRESCE COM A PROXIMIDADE — e é isso que os separa dos
outros quatro modos de força** (F, 2026-09-06 · M — §5.2-ter). Nos dois apertos `u` é o vector
`vértice → alvo` **re-escalado a comprimento 1** (no aperto de linha, `1` antes de se descartarem as
componentes; a projecção deixa-o `≤ 1`), e o único factor que sabe da distância é a **curva de
falloff — que ali está no MÁXIMO**. ⇒ o vértice que está a meia aresta do cursor recebe o mesmo
impulso do que está a meio raio, e **ultrapassa** o cursor. Não há tecto de deslocamento, nem corte
ao ultrapassar, nem amortecimento próprio: tudo o que o aperto tem, o arrasto também tem (F).

⚠️ **Consequência com número, na malha de referência** (aresta `0,0469`, `R = 0,35`, força `1`,
massa `1`): o impulso máximo é `10·α·dt/massa = 0,1` por passo (§4.1/§5.4), **`2,1×` a aresta**.
No **primeiro** passo simulado do aperto de ponto o oráculo põe `9` vértices para lá do cursor e
devolve **`10` quadriláteros com a orientação invertida**; o arrasto, no mesmo passo e com o mesmo
impulso, devolve **zero** (M — §5.2-ter). *A inversão não é um acidente de traço longo: ela nasce no
primeiro passo, a partir do repouso, e é a LEI.*

⚠️ **A direcção nula tem tratamento próprio e é o único caso especial dos apertos:** um vértice
**exactamente** sobre o cursor (ou, no falloff de plano, exactamente SOBRE o plano) dá separação
nula, e a re-escala a comprimento 1 aplicada a essa separação devolve o **vector NULO** ⇒ **força
zero**, sem `NaN`, sem direcção de reserva e sem o vértice ser saltado. Um vértice a um epsilon dele
recebe a força **inteira** (F). ⇒ *o ponto onde o aperto é mais forte é o ponto onde a direcção dele
está pior determinada.*

### §4.2-bis — A NORMAL DA ÁREA e o factor de ESCALA do Push (F, 2026-09-06)

⭐⭐⭐ **O Push é o único modo cuja direcção não é lida da malha vértice a vértice nem do gesto: ela é
UM vector por passo, e é a lei da casa do *Sculpt Plane*.** O deslocamento que ele empilha é

```
u = − n̂_área · R · escala · 2          (R = o raio do pincel neste passo)
```

**(1) Quando `n̂_área` é reavaliada.** A cada passo do traço, na **primeira** passagem de simetria; as
outras passagens recebem-na espelhada/rodada. As únicas coisas que a congelam são as opções
*Original Normal* / *Original Plane* do pincel (ambas **desligadas** nos presets de tecido — A) e o
facto de o pincel ser o *Grab* da casa (⛔ que **não** é o modo Agarrar do tecido: a regra de congelar
nomeia o pincel, não o modo de deformação). ⇒ **nas fixtures ela muda a cada passo.**

**(2) De que malha — ⛔⛔ ERRATA DE 2026-09-07 (M): esta linha estava INVERTIDA na metade que decide,
e as DUAS grandezas não vêm da mesma superfície.**

| grandeza | de que superfície | prova |
|---|---|---|
| as **POSIÇÕES** que o disco de amostragem testa | a de **AGORA**, já deformada | é o que faz o disco poder ficar **vazio** — item (8) |
| as **NORMAIS** que entram na soma | ⭐ as da superfície **que o traço encontrou** (§4.2-ter) | a direcção recuperada do oráculo é `(0, 0, 1)` **a cinco casas** em todos os passos em que o gesto dispara, sobre uma folha que já afundou `0,2` — onde a normal *actual* sob o cursor não é vertical |

⚠️ A redacção anterior dizia «das posições e normais **ACTUAIS**» e acrescentava um ⛔ a proibir a
leitura de repouso. A metade das posições estava certa; a das normais está **refutada por medição**
(§10.11), e é ela que decide a direcção do gesto.

**(3) De que vértices, e com que peso.** De cada vértice **visível** das células que o pincel juntou
(§2.1) cuja distância ao cursor seja `d ≤ R · «Normal Radius»`, com a distância medida pela forma de
queda do pincel (3D com *Sphere*, que é a dos presets — A). Cada um contribui com a **normal do
vértice** multiplicada por `3p² − 2p³`, `p = 1 − d/(R·«Normal Radius»)`, saturado a `[0,1]`; o
resultado é a **soma normalizada**.
⚠️⚠️ **O «Normal Radius» dos presets de tecido é `0,5` (A)** ⇒ **a normal é amostrada num disco de
METADE do raio do pincel**, não no disco inteiro. *Um port que a tire do disco inteiro tem uma
direcção diferente assim que a superfície deixa de ser plana ou de estar em repouso.*

**(4) A regra de desempate, que não é uma média.** Os vértices são repartidos em **dois** baldes pelo
sinal de `n̂ · v̂` (`v̂` = a direcção da vista): `> 0` no balde da frente, `≤ 0` no de trás. A resposta é
a soma normalizada do **PRIMEIRO balde que esteja não-vazio E cuja soma tenha comprimento não-nulo,
nesta ordem fixa** — nunca a mistura dos dois, e ⛔ nunca «o balde com mais vértices». ⇒ basta **um**
vértice virado para a vista para que os virados ao contrário não contem.
⚠️ **O balde da frente ser não-vazio não basta:** se a soma dele se anular (normais opostas que se
cancelam), a resposta é a do balde de trás — o teste é *não-vazio **e** soma não-nula*, avaliado balde
a balde e nesta ordem, ⛔ **não** «escolher o balde e só depois olhar para a soma».

**(5) Quando não há resposta.** Se **nenhum** dos dois baldes passa esse teste (nenhum vértice
qualifica, ou as duas somas têm comprimento zero), `n̂_área` é o **vector NULO** — e o Push desse passo
é **força zero**, sem `NaN` e sem direcção de reserva.
⚠️⚠️ **Isto NÃO é um caso degenerado: é o regime normal do Push à força de omissão** — item (8).

**(6) A forma de queda *Projected*** projecta ainda `n̂_área` no plano do ecrã e re-normaliza (com
*Sphere*, a dos presets, não faz nada).

**(7) O `escala`.** É um vector de **três** números fixado no pen-down, e ele não depende do passo,
nem da pressão, nem da malha: `escala_eixo = max(|escala do objecto em x, y, z|) / escala do objecto
nesse eixo`. A multiplicação é **componente a componente** ⇒ num objecto de escala **uniforme** ele é
`(1, 1, 1)` e o módulo do deslocamento é exactamente `2R` (M: `plano_empurrar_radial_local_1passo`
dá `0,06942 = 2 · 0,35 · 0,0992`, §10.1); num objecto de escala **não-uniforme** ele **entorta a
direcção** além de mudar o módulo — não é um factor escalar.

**(8) ⭐⭐⭐ O PUSH CALA-SE QUANDO A FOLHA AFUNDA MAIS DO QUE O DISCO DE AMOSTRAGEM (M, 2026-09-07).**
Juntando (2) e (3): o disco tem raio `R · «Normal Radius» = 0,175` com as omissões, e mede-se contra
as posições **de agora**. Numa folha que o próprio Push já afundou, chega um momento em que **nenhum
vértice** está a menos de `0,175` do cursor (que continua no plano de partida) ⇒ `n̂_área` é o vector
nulo ⇒ **o gesto não escreve aceleração nenhuma nesse passo, e o passo é só solver.**

⚠️ **É medido, e a régua é um limiar limpo.** Recuperando do oráculo, passo a passo, o vector do
gesto (§10.11) sobre **sete** traços de empurrar do plano — `67` passos com o cursor em movimento:

| | quantos | `min_v \|v_agora − cursor\|` |
|---|---|---|
| o gesto **DISPARA** (`\|u\| = 0,70000 = 2R`) | `53` | de `0,00767` até **`0,17313`** |
| o gesto está **CALADO** (`\|u\| = 0`) | `14` | de **`0,17586`** até `0,19656` |

⇒ o vão é `0,17313 … 0,17586` e **`R · 0,5 = 0,17500` cai dentro dele**. ⛔ Não há um único passo do
lado errado, e a régua não é escolhida: o `0,5` é o *Normal Radius* dos presets, já declarado em (3).

⭐⭐ **E ele volta a disparar**, o que é o que torna o padrão irreconhecível de fora: à medida que o
cursor avança para terreno ainda pouco afundado, o disco volta a apanhar vértices. Em
`plano_empurrar_radial_local_origem` o gesto escreve nos passos `2, 3, 4, 5` e `10`, e cala-se nos
outros seis; no mesmo traço em área *Global* escreve em `2..6` e `10`; com massa `2` escreve em
`2..9`; e às forças `0,5` e `0,25` — em que a folha nunca afunda `0,175` — escreve nos **onze**.
⇒ *o mesmo caminho, o mesmo raio e a mesma curva dão quatro padrões de disparo diferentes, e o que
os separa é a profundidade da cova.*

⚠️ **Consequência para um port, e é a resposta à pergunta que gerou esta emenda:** um port que
reavalie a normal sobre a malha deformada **e** aplique força em todos os passos entrega uma frente
de ataque `16×` mais curta do que a do alvo — não porque o corte no raio, a banda ou o conjunto de
células estejam errados (não estão: §10.11), mas porque **empurra com uma direcção inclinada em
passos em que o alvo não empurra de todo.**

⚠️ **O mesmo `n̂_área` é o `ẑ` do referencial local do traço e a normal que o falloff de plano usa
(§4.4)** — as três leituras são a mesma grandeza, calculada uma vez por passo.

### §4.2-ter — ⭐⭐⭐ AS NORMAIS QUE O GESTO LÊ SÃO AS DA SUPERFÍCIE QUE O TRAÇO ENCONTROU (M, 2026-09-07)

**A lei, em uma frase: dentro de um traço, o pincel deforma a malha mas continua a ler as normais
com que o traço começou.** Só duas coisas as lêem — a **normal da área** do Push (§4.2-bis) e a
**normal por vértice** do Inflate (§4.2) — e as duas obedecem. Os oito modos não têm terceira
leitura de normal: o arrasto tira a direcção do cursor, os dois apertos do vector para o alvo, o
Expand do comprimento de repouso, e as duas âncoras do delta.
⚠️⚠️ **Mas HÁ um terceiro leitor de normais no pincel, fora dos modos, e ele obedece à MESMA lei
(R-pré, 2026-09-07):** o recorte *Front Faces Only* do factor por vértice (§4.1) compara a **normal
do vértice** com a direcção da vista, e a normal que ele compara é a mesma fotografia — a da
superfície que o traço encontrou. ⛔ Ele está **desligado** nos presets de tecido (§8.2), logo o
corpus não o observa; um port que o ligue lendo normais da malha deformada erra por dentro do
factor, e não pelo `u`. (A colisão do §5.6 também lê normais, mas as do **colisor**, que é outra
geometria — não entra nesta lei.)

⚠️ **Isto é uma propriedade do TRAÇO, não do programa:** de um traço para o seguinte as normais
**refrescam-se**, e a superfície que o traço seguinte encontra é a que o anterior deixou.
⚠️⚠️ **E é do TRAÇO, não do solver de tecido: o FILTRO de tecido (§7) faz o CONTRÁRIO** — ali as
normais são reavaliadas a cada passo, porque cada passo do filtro repete a preparação do objecto
para edição que o pincel só corre ao começar o traço. ⇒ *a mesma palavra («Inflate») nomeia duas
leis em dois sítios deste documento*, e o §7 já diz a dele. Um port que leve esta secção ao filtro
congela o que lá é vivo.

**A medição (§10.11), em três leituras independentes:**

| o que se mede | a favor de «a superfície que o traço encontrou» | a favor de «a superfície de agora» |
|---|---|---|
| **Inflate**, direcção por vértice, `11` passos × `2` fixtures | resíduo `7·10⁻⁶ … 7,4·10⁻⁴`, amplitude `1,00000` | resíduo `0,32 … 0,77` |
| **Push**, direcção por passo, `53` passos com gesto | `(0, 0, 1)` a **cinco casas** em todos | a normal actual sob o cursor está a dezenas de graus da vertical |
| **Inflate**, 2.º de dois traços sobre a mesma malha | resíduo `3,5·10⁻⁷`, amplitude `1,00000` **contra as normais do pen-down DESTE traço** | — |

⭐ A terceira é a que separa **«a superfície do início do traço»** de **«a superfície original do
objecto»**: o 1.º traço deixa uma cova de `0,0992` (normais inclinadas até `22,4°`), e o 2.º usa
**essas** normais, não as planas — resíduo `0,32` e amplitude `0,946` se se insistir nas planas.
Fixture: `plano_inflar_radial_local_1passo_2tracos`.

⭐⭐ **E que é a normal da ÁREA, e não a da vista, decide-se fora do plano** — no plano as duas valem
`(0,0,1) `e são indistinguíveis. Sobre a esfera, num passo simulado a partir do repouso (onde a
relaxação é um no-op por construção, logo o deslocamento **é** `u · f(v) · dt`), o vector recuperado
tem `|u| = 0,700000 = 2R` com resíduo `6,1·10⁻⁷`, e está a **`0,023°`** da normal da superfície no
cursor contra **`17,43°`** da normal da vista. Fixture: `esfera_empurrar_radial_local_1passo`
(controlo: `esfera_inflar_radial_local_1passo`, amplitude `1,000000`, resíduo `5,0·10⁻⁵`).

⚠️⚠️ **O que esta linha NÃO pode afirmar, e a experiência que o fecharia.** Todo o corpus foi gravado
com **um traço = uma invocação** do gesto. Dentro dessa invocação a malha é reescrita a cada passo e
o pedido de reavaliação da cena só é honrado **entre** invocações — que é exactamente o que a 3.ª
leitura mede. ⛔ **Não se pode concluir daqui que numa sessão interactiva, onde a cena é reavaliada
entre dois eventos de ponteiro, as normais não sejam as de cada passo.** ⇒ é uma **divergência
declarada**, com o preço: um traço interactivo do alvo pode diferir do que estas fixtures dizem, e
quem quiser fechá-la precisa de um traço gravado evento a evento, com uma reavaliação de cena entre
eles.
⚠️ **E ela é MAIS ESTREITA do que a frase acima deixa supor (R-pré, 2026-09-07, conferido no fonte):
«interactivo» não implica «reavaliado».** Quem refresca as normais é o passo de **preparação do
objecto para edição**, e o pincel corre-o **uma vez por invocação** — no caminho normal de desenho
da vista o traço interactivo **também não** o volta a correr, e a lei destas fixtures vale lá tal e
qual. As condições que o fazem voltar a correr no meio de um traço são **duas e nomeáveis**: (a) o
desenho da vista **não** usar o caminho rápido da escultura (modificadores activos, ou sombreado que
obrigue a reavaliar o objecto), e (b) o motor de render da vista ser **externo**. ⇒ *a divergência
existe, e o que a dispara é a configuração da VISTA, não o facto de haver uma mão.*
⭐ **A escolha que fica é a das fixtures**, porque é o corpus que gateia o port — e ela é
também a mais barata e a mais estável: uma fotografia das normais no pen-down, ao lado da fotografia
das posições que a simulação já guarda (§5.2-quater).

### §4.2-quater — ⭐⭐⭐ HÁ **DUAS** LEIS DE NORMAL POR VÉRTICE, e a do caminho da escultura é a SEM PESO (F+M, 2026-09-07)

⛔⛔ **O peso da soma por face (§4.6 linha 4) estava aberto porque a pergunta tinha duas respostas
certas.** O programa calcula a normal de um vértice de **duas maneiras**, e qual delas está no
vector quando o gesto o lê depende do **estado** da malha, não do modo do pincel:

| lei | quando corre | como a normal do vértice se forma |
|---|---|---|
| **incremental** (⭐ a do caminho da escultura) | sempre que a malha **já traz normais calculadas de antes** e só alguns pedaços dela se mexeram — isto é, **a cada refrescamento durante e depois de uma escultura** | **soma SEM PESO das normais UNITÁRIAS das faces incidentes**, normalizada no fim; se a soma tiver comprimento zero, a resposta é um **eixo fixo do objecto** |
| **reconstrução total** | quando não há normais de antes para aproveitar — na prática, a **primeira** avaliação de uma malha que ainda não foi esculpida | **soma ponderada pelo ÂNGULO DO CANTO** que a face faz naquele vértice, normalizada no fim |

⚠️ **Nos dois casos a normal da face é UNITÁRIA antes de entrar na soma** — é isso que faz a
primeira lei ser *uniforme* e não *por área*: uma soma de normais de face **não** normalizadas
traria a área embutida, e não é o que acontece.

**A medição** (M, 2026-09-07 — leituras do vector de normais do próprio programa, comparadas com as
três candidatas calculadas sobre as MESMAS posições; a grandeza é o ângulo entre direcções
unitárias, em graus, e a resolução do canal é `≈ 0,027°`):

| malha lida | uniforme | ângulo | área | veredito |
|---|---|---|---|---|
| esculpida, vértices movidos, refrescada sobre a forma de agora (4 malhas: 2 esferas agarradas/infladas + 2 muito deformadas) | **mediana `0,000`–`1,3·10⁻³` · máx `≤ 0,027`** | mediana `0,022`–`0,059` · máx `1,62` | mediana `0,031`–`0,073` · máx `2,36` | ⭐ **uniforme, à resolução do canal** |
| grelha de quadriláteros nunca esculpida, com vértices deslocados ao acaso | mediana `3,51` · máx `17,4` | **mediana `0,000` · máx `0,023`** | mediana `4,25` · máx `31,6` | ⭐ **ângulo** |
| a mesma, triangulada | mediana `6,06` · máx `72,6` | **mediana `0,000` · máx `0,027`** | mediana `9,34` · máx `94,3` | ⭐ **ângulo** |
| esfera UV nunca esculpida, com vértices deslocados ao acaso | mediana `4,30` · máx `115,8` | **mediana `2,3·10⁻³` · máx `0,027`** | mediana `5,52` · máx `83,5` | ⭐ **ângulo** |

⚠️ **A separação entre as candidatas é da mesma ordem do erro de cada uma** — isto é, a que ganha
ganha por ser exacta, não por ser a menos má; e o `máx ≤ 0,027°` da vencedora é o mesmo em todas as
linhas, porque é o chão do canal e não uma propriedade da lei.

⛔⛔ **CONSEQUÊNCIA PARA O CORPUS, e é uma que se lê ao contrário com facilidade:** as fixtures de
esfera são **primeiros traços sobre uma malha nunca esculpida** ⇒ as normais que o gesto delas leu
são as da lei **ponderada pelo ângulo**. As fixtures de plano têm todas as normais `(0,0,1)` nas
duas leis. E o 2.º traço da fixture de dois traços lê a lei **sem peso**, porque a região já tinha
sido movida. ⇒ *o corpus contém as duas leis, e não é o modo do pincel que escolhe.*

⭐ **O que um port faz, e porquê:** implementar a lei **sem peso** (é a do caminho da escultura, e é
a única que vale a partir do segundo traço). O preço de a usar também no primeiro traço está
medido e é **desprezável neste corpus**: na esfera em repouso as duas leis diferem por mediana
`0,021°` e máximo `0,029°` — sobre um deslocamento de `0,25` isso é `1,3·10⁻⁴`, duas ordens abaixo
da barra do gate 15. ⛔ **Mas isso é uma propriedade da malha, não da lei:** numa malha de
triângulos irregulares as duas separam-se por **graus** (até `116°` medido), e é por isso que a
diferença fica NOMEADA aqui em vez de arrumada como ruído.

⚠️ **Fronteira honesta:** a lei da reconstrução total foi estabelecida por **medição** (as três
linhas de malha nunca esculpida acima); ⛔ o código que a executa **não vive na parte do fonte que
esta linha tem**, ao contrário da lei incremental, que está confirmada no fonte. A lei incremental
é a que o port precisa, e essa está confirmada dos dois lados.

⚠️ **E há uma terceira população, que esta secção não cobre:** malhas de subdivisão e topologia
dinâmica têm cada uma o seu próprio fornecedor de normais (§6.7). Nada aqui se aplica a elas.

### §4.3 — Os modos de ÂNCORA (dois) — e o delta de agarrar

**O delta de agarrar `δ`** é um vector em espaço do objecto, derivado do ECRÃ (F): o ponto do
cursor é des-projectado à **profundidade da localização original de agarrar** (a do 1.º passo) e
subtraído ao ponto anterior. Para o **Grab** o delta **acumula** desde o pen-down (ponto actual −
ponto original: um vector total); para os **outros sete modos** é **incremental** (ponto actual −
ponto anterior). Com *Normal Weight* `> 0` (omissão `0`) o delta é inclinado para a normal da
área como no Grab da casa. Com falloff *Projected* é achatado no plano da vista.

⭐⭐⭐ **A consequência que só uma superfície CURVA revela: `δ` NÃO é a diferença dos dois pontos 3D do
cursor — é a PROJECÇÃO dessa diferença no plano do ecrã** (F, 2026-09-06). As duas des-projecções são
feitas à **mesma** profundidade (a do pen-down), logo numa vista **ortográfica** a componente do
deslocamento ao longo do eixo da vista é **descartada por construção**, e numa vista em perspectiva
ela é descartada *e* o resto é reescalado pela razão de profundidades. ⇒ numa folha plana vista de
frente o caminho vive nesse plano e `δ` **é** a diferença dos pontos, ao bit; numa superfície curva
os dois vectores separam-se.
⚠️ **Quem lê `δ`:** o guarda de «passo sem movimento» de TODOS os modos · a âncora do Agarrar (que o
acumula) · a âncora e o avanço do centro do Snake Hook · a **normal do plano de queda** (§4.4) · o
`x̂` do referencial local do traço (§4.4). ⛔ **O guarda de «passo sem movimento» vale para os OITO
modos, o arrasto incluído** — ele corre antes de o modo ser escolhido.
⛔ **E o arrasto é o único modo que tira a direcção do MOVIMENTO do cursor e NÃO a tira de `δ`:** ela
é a diferença **dos dois pontos 3D** do cursor, normalizada (§4.2) — e é por isso que o arrasto é o
modo que se comporta igual nas duas superfícies. ⚠️ **Push, Inflate e Expand não lêem deslocamento de
cursor nenhum para a direcção** (normal da área · normal do vértice · nenhuma), e o aperto de ponto
também não (vértice → cursor) — eles só encontram `δ` no guarda e, se o falloff for de plano, na
normal desse plano.
⚠️ **Números na esfera das fixtures** (esfera unitária, caminho de `x = −0,3` a `+0,3` em 12 passos,
vista ao longo do eixo de profundidade — M): `δ` vale `(0,05455, 0, 0)` em **todos** os passos,
enquanto a diferença dos pontos 3D chega a `(0,05455, ∓0,01547, 0)` — **`15,83°`** de diferença de
direcção no 1.º e no último passo, `0°` a meio, e até `1,039×` de módulo. No **Agarrar**, que
acumula, o desvio entre o `δ` acumulado e a diferença acumulada dos pontos chega a **`0,04569`** a
meio do caminho, que é **`19,3 %`** do maior deslocamento daquela fixture (`0,236509`).

⭐⭐⭐ **E por isso `δ` É RECONSTRUÍVEL DO NOSSO LADO, sem câmara nenhuma — falta só o EIXO DA VISTA,
que fica aqui** (R-pré, 2026-09-06, derivado das próprias fixtures e confirmado pela medição
independente do I): as fixtures foram gravadas em vista **ORTOGRÁFICA**, e nessa vista as duas
des-projecções à mesma profundidade dão exactamente *a componente do deslocamento perpendicular ao
eixo da vista*. Logo, com `c_k` os pontos do `caminho` que cada fixture já traz no cabeçalho:

```
δ_k = proj_⊥v̂ (c_k − c_{k−1})            (os sete modos incrementais)
δ_k = proj_⊥v̂ (c_k − c_0)                (o Agarrar, que acumula)
```

⚠️ **O eixo da vista `v̂` NÃO está no cabeçalho das fixtures e é diferente nos dois corpora:**
- **corpus do PLANO** — a folha vive no plano `z = 0` e a vista é ao longo de **`z`** ⇒ a projecção
  é um **no-op** e `δ` é a diferença dos pontos ao bit (é esta a degenerescência que a §4.6-1 nomeia);
- **corpus da ESFERA** — a vista é ao longo de **`y`** (o caminho pousa em `y = −√(1−x²)`), ⇒ `δ` vive
  no plano **`x–z`** e é a componente `y` que se perde.

⭐ **A prova de que é ortográfica está nos próprios números**: o passo do caminho é `0,6/11 =
0,054545…` e `δ` mede `0,05455` em **todos** os 12 passos da esfera, apesar de os pontos do caminho
estarem a profundidades diferentes — numa vista em perspectiva os dois não podiam coincidir.
⛔ **Projectar no plano perpendicular à NORMAL DO PEN-DOWN é outra coisa e está medido a piorar** (o
I mediu `0,265 → 0,605` no Agarrar e `0,351 → 0,663` no gancho): o plano é o do **ECRÃ**, e só numa
folha vista de frente é que ele coincide com o plano tangente.

⚠️ **A localização do cursor** (`c`, centro do disco de influência):
- Drag/Push/Pinch/Inflate/Expand: **re-apanhada na superfície a cada passo** (raio contra a malha).
- **Grab**: **fica no ponto do pen-down** durante todo o traço (é o que faz o Grab «pegar» num
  conjunto fixo de vértices).
- **Snake Hook**: `c ← c + δ` a cada passo — o centro **anda com o gancho no plano de profundidade
  original**, não é re-apanhado na superfície.
  ⚠️⚠️ **E o `δ` desse avanço é o do passo ANTERIOR, não o deste passo (F, 2026-09-06):** o avanço
  acontece **antes** de `δ` ser recalculado, logo quando a queda por-vértice é avaliada o centro está
  **onde o pincel estava no início do passo**, não onde o cursor chegou. Em fórmula:
  `c_k = pen-down + Σ_{i<k} δ_i` — o centro está **um passo atrasado** em relação ao cursor, e como
  a localização **nunca mais é lida do evento depois do 1.º passo**, essa soma é a definição do
  centro (não há re-projecção nem raio contra a malha que possam divergir dela).
  ⭐ **No 1.º passo simulado (o 2.º passo do traço) `δ` do passo anterior é ZERO** ⇒ o centro é
  **exactamente o ponto do pen-down**, e o vértice mais deslocado é o do pen-down — não o que está
  sob o cursor (M — §10.4).
  ⇒ *é isto que faz o pico da deformação ficar ATRÁS do cursor, e um port que centre a queda no
  cursor apanha material novo a cada passo em vez de arrastar o que já pegou.*

| modo | âncora de deformação do vértice `v` (o ponto B, §3.2) | força da restrição `s` (fixa na criação) | factor por passo `σ_v` (§5.2) |
|---|---|---|---|
| **Grab** | `p⁰(v) + δ · f_v` — a partir da posição de **repouso do traço**, deslocada pelo delta TOTAL pesado | radial: `0,1 · curva(d⁰, R₀)` e **só** para vértices com `d⁰ < R₀` (raio inicial); plano: `0,1` para todos os vértices da célula | radial: `1` · plano: `clamp(f_v, 0, 1)` reescrito a cada passo |
| **Snake Hook** | `x(v) + δ · f_v` — a partir da posição **actual**, deslocada pelo delta INCREMENTAL pesado | `0,35` para todos os vértices da célula | ⭐ **`f_v` reescrito a cada passo, e ZERO para quem não está sob o pincel** |
| pincel alheio com alvo = simulação (Pose, Boundary) | a posição que esse pincel calcularia para o vértice (ele escreve-a na âncora em vez de na malha) | `0,01` para todos | `1` |

⭐⭐ **A lei do Snake Hook, que é a que a documentação diz dar as dobras naturais:** como B é «onde
o vértice está + δ·f» e a correcção é proporcional a `s · σ_v = 0,35 · f_v` (§5.2), o puxão
efectivo por varredura é `≈ 0,3 · 0,35 · f_v² · δ` — **quadrático no falloff**, re-ancorado no
estado actual, e **nulo fora do pincel**: a simulação nunca vê um alvo velho. O Grab, ao contrário,
guarda âncoras «de partida + delta total» para um conjunto fixo, com força fixa por vértice.
(H: D8621 — «muda a força das restrições de deformação por passo para afectar o resultado da
simulação o menos possível».)

⚠️ **O Grab mede o falloff na malha de PARTIDA** (distâncias e recorte sobre as posições de repouso
do traço) — é por isso que o conjunto agarrado não muda quando a malha se mexe. ⚠️ **O Grab é o
ÚNICO assim:** os outros sete modos — o **Snake Hook incluído** — medem a distância, o recorte e a
textura sobre as **posições ACTUAIS** da malha, isto é, o estado deformado com que o passo começa
(F). ⇒ no Snake Hook o material já puxado viaja **com** o centro atrasado, e o gancho continua a
segurar o que agarrou em vez de agarrar o que está debaixo do cursor.

⚠️ **A força por passo das âncoras é ZERADA em TODO o objecto antes de ser reescrita, nos DOIS
modos de âncora** (F — a espec dizia «o Grab não», e estava errado): o passo começa com `σ ≡ 0` em
toda a malha e cada um dos dois preenche o que lhe toca — o Grab põe `1` (radial) ou `clamp(f,0,1)`
(plano) nas células afectadas, o Snake Hook põe `f`. O que os distingue **não é zerar ou não**, é o
valor com que reescrevem e o facto de o conjunto afectado do Grab ser fixo.

⚠️ **Nenhum dos dois modos de âncora aplica força.** A aceleração fica a zero; o que move é a
restrição de âncora dentro das 5 varreduras (§5.2).

### §4.4 — O plano de falloff e o referencial local do traço

Quando o modo é Pinch Perpendicular **ou** o *Force Falloff* é *Plane*, o alvo calcula, no passo,
**a normal e o centro da área** sob o pincel (a lei da casa: *Sculpt Plane* — normal da área,
vista, ou eixo; *Original Normal/Plane* congelam) e monta um referencial (F):

```
ẑ = n̂_área      x̂ = n̂_área × δ̂      ŷ = n̂_área × x̂      origem = c
```

O **plano de falloff** passa pelo **centro da área** com normal **`δ̂`** (a direcção do
movimento no ecrã, des-projectada). ⇒ A faixa de influência é perpendicular ao traço, de
meia-espessura `R`, **sem limite ao longo do plano** (só o conjunto de células a limita). O cursor
desenha-o como um segmento com setas nas duas pontas, de comprimento `2R`, ao longo de `x̂`
(para o Grab, transladado pelo delta) (F).

⚠️ Respostas a duas perguntas do [`04` §F](../cloth/04_espec_do_comportamento.md): **(3)** o plano
NÃO é o do pen-down nem o da vista — é o plano do centro da área com a normal = direcção do
movimento, **recalculado a cada passo** (salvo *Original Plane/Normal*); **(12)** a simetria é
expandida **por passagem** e cada passagem corre gesto+solver inteiros (§1).

⭐⭐⭐ **O «CENTRO DA ÁREA» não é o centroide do disco — e é isso que fecha a pergunta que ficou em
aberto no INBOX** (F, 2026-09-06). Ele sai da **mesma** varredura que a normal (§4.2-bis: mesmo
conjunto de células, mesmo disco de raio `R · «Normal Radius»` = **metade** do raio do pincel, mesmos
dois baldes com a mesma regra do primeiro balde não vazio, mesmas posições **actuais**), mas o que se
soma **não é a posição do vértice**:

```
contribuição(v) = c + (p_v − c) · (1 − a_v)      a_v = 3p² − 2p³,  p = 1 − d_v/(R·«Normal Radius»)
centro da área  = média das contribuições do balde escolhido
                = c   se nenhum vértice qualificar
```

⇒ **o peso `1 − a` vale ZERO no cursor e cresce para a borda do disco**: cada vértice é *puxado para
o cursor* antes de entrar na média, e quanto mais perto do cursor está, mais completamente é
substituído por ele. Numa folha em repouso com o cursor sobre ela o centro da área **é praticamente o
cursor**; numa folha já cavada ele fica **muito mais perto do cursor** do que o centroide das
posições.
⚠️⚠️ **É por isso que a medição do I — «o plano pelo CURSOR reproduz o alvo e o plano pelo centro da
área afasta-o» — não refuta esta secção: o que foi medido foi um CENTROIDE, e o alvo não usa um
centroide.** *Uma recusa medida responde a uma pergunta, e aquela respondeu «o centroide não serve»,
não «o centro da área não serve».* O plano pelo cursor é a **aproximação de primeira ordem** do
centro da área, e é por isso que ele passa quase.
⚠️ **A origem do referencial local NÃO é o centro da área — é o cursor `c`** (o bloco acima já o
diz): as duas grandezas têm consumidores diferentes, e só o plano de queda lê o centro da área.
⚠️ Na área *Local* o centro da área e a localização inicial da área (§2.1) são **coisas distintas**:
o centro da área é reavaliado a cada passo à volta do cursor; a localização inicial fica no pen-down
durante todo o traço.

### §4.5 — Expand — o modo que muda o REPOUSO

`Expand` não aplica força nem âncora: para cada vértice sob o pincel, um **desvio de repouso**
por vértice `τ_v` acumula `τ_v += 0,01 · f_v` a cada passo (F). O comprimento de repouso efectivo
de toda restrição `(a, b)` passa a ser `ℓ + (τ_a + τ_b)/2` (§5.2). Com `flip = −1` contrai.
- `τ` vive na simulação ⇒ **morre no fim do traço** (§6.3), mas a geometria que ele produziu fica.
- A ordem de grandeza: a força máxima, `B = 0,1` ⇒ `τ` cresce `0,001` por passo no centro — e é
  **absoluto** (não relativo à aresta): numa aresta de `0,047` (a grelha do oráculo) são `2 %` por
  passo (M, §10).

⭐⭐⭐ **QUANDO `τ` PASSA A VALER: no MESMO passo, ANTES da primeira varredura** (F+M, 2026-09-06). A
fase do gesto corre inteira antes da fase do solver (§1), e é ela que soma o incremento; quando a
1.ª varredura lê o comprimento de repouso efectivo, o incremento **deste** passo já lá está. ⇒ *o
Expand é o único modo em que a relaxação tem trabalho para fazer no PRIMEIRO passo simulado, com a
malha ainda em repouso* — nos modos de força, a relaxação desse passo corre sobre uma malha por
mexer e toda a correcção dela é identicamente zero (§5.2).

⚠️⚠️ **A prova está numa fixture que já existia e ninguém tinha lido assim:** o
`plano_expandir_radial_local_1passo` move **848** vértices num passo em que a aceleração é zero e
não há âncora nenhuma (`846` na gémea com o pen-down na origem, `plano_expandir_radial_local_origem_1passo`). *Se `τ` só valesse no passo seguinte, esse traço tinha de mover **zero**
vértices.* ⭐ E a corrida nova de 2026-09-06 fecha-a por **intervenção**: o mesmo traço com força
`0,5` — que muda `τ` e mais nada, porque `B = 0,1·α` com `α = força²` — dá `0,000478` contra
`0,001914`. ⚠️ **As duas razões, com a conta de cada uma escrita** (reconstruídas do zero pelo R-pré,
2026-09-06): a **razão dos máximos** vale **`0,24999`** (`0,00047846 / 0,00191389`, e não `0,2497`,
que é o que sai de dividir os dois números do cabeçalho já arredondados a seis casas) e a **mediana
das razões por vértice** vale **`0,250000`** sobre os `662` vértices que os dois traços movem (M:
`plano_expandir_radial_local_origem_1passo` e `…_forca05`). *O passo inteiro é linear em `τ`, o que
só é possível se `τ` já estiver na conta quando a varredura corre.*

⚠️⚠️ **E `τ` NÃO é privilégio das restrições de par: ele entra no comprimento de repouso de TODA
restrição, as quatro espécies** (F). A soma é sempre *«metade do `τ` de cada extremo»* — só que as
três espécies cujos dois extremos são **o mesmo vértice** (âncora, corpo mole, pino) recebem por isso
o **`τ` INTEIRO daquele vértice**, e não metade. ⇒ com o Expand a soprar, um pino deixa de segurar o
vértice na posição de repouso e passa a segurá-lo **a `τ` dela**, e uma âncora passa a mirar uma
casca de raio `τ` à volta do alvo em vez do alvo.
⚠️ **QUAIS destas combinações o artista alcança, uma a uma** (corrigido pelo R-pré, 2026-09-06 — a
redacção anterior dizia que nenhuma era alcançável com o pincel de tecido sozinho, e isso é verdade
só para a âncora): o **pino** e o **corpo mole** não são modos, são **opções independentes do modo**
(o pino nasce da caixa da fronteira em qualquer área que não seja a dinâmica; o corpo mole nasce de
plasticidade `> 0`) ⇒ **Expand + pino** e **Expand + corpo mole** são alcançáveis com o pincel de
tecido sozinho, e é lá que o `τ` inteiro se vê. ⛔ Só a **âncora de deformação** é inalcançável junto
com o Expand pelo pincel de tecido (a âncora só existe nos dois modos de âncora), e essa combinação
alcança-se quando um pincel alheio usa a simulação como alvo. *Um port que some `τ` só à lista das
estruturais escreve uma lei diferente, e o corpus de hoje — que corre com pino desligado e
plasticidade `0` — não a vê falhar.*

### §4.6 — O que é DEGENERADO num plano visto de frente e VIVO numa superfície curva (F+M, 2026-09-06)

⛔⛔⛔ **Leia isto antes de procurar uma lei «para superfície curva»: não existe nenhuma.** Nenhuma
decisão do alvo — nem no gesto, nem no solver, nem na construção — pergunta pela curvatura, pela
normal do ponto do cursor ou pelo tipo de malha. O que existe é uma lista **fechada** de grandezas
que, numa grelha plana vista de frente e em repouso, valem sempre a mesma coisa (ou zero), e que
numa esfera passam a variar. *Um port calibrado só no plano acerta nelas por acidente.*

| # | grandeza | no plano visto de frente | numa superfície curva | quem a lê |
|---|---|---|---|---|
| 1 | **o deslocamento do cursor `δ`** (§4.3) | **igual** à diferença dos dois pontos 3D | a componente de profundidade é **descartada**: `15,83°` de diferença de direcção nas fixtures de esfera, `1,039×` de módulo, `0,04569` de desvio acumulado (`19,3 %` do maior deslocamento do Agarrar) | Agarrar · Snake Hook · normal do plano de queda · `x̂` do referencial · e o guarda de «passo parado» dos **oito** modos (o arrasto incluído) — ⛔ o que o arrasto **não** tira de `δ` é a **direcção** |
| 2 | **a normal da área** (§4.2-bis) | exactamente a normal da folha em todos os passos em que existe — e ⚠️ **passa a NÃO EXISTIR** (vector nulo, força zero) assim que a cova afasta todo o vértice do disco de amostragem: §4.2-bis (8) | média com peso `3p²−2p³` das normais **da superfície que o traço encontrou** (§4.2-ter — ⛔ **não** as de agora; esta célula dizia «actuais» até 2026-09-07), amostradas num disco de **meio** raio cujas POSIÇÕES são as de agora: não é a normal no ponto do cursor, e o que roda com a vala é o CONJUNTO amostrado, não as normais | Push · plano de queda · `ẑ` do referencial |
| 3 | **o centro da área** (§4.4) | praticamente o cursor | mistura pesada que **puxa para o cursor**, e não o centroide do disco | plano de queda |
| 4 | **a normal do vértice** | `+ẑ` para todos, seja qual for o peso usado ao somar as faces | **soma das normais UNITÁRIAS das faces que tocam o vértice, SEM peso, normalizada no fim** (uma soma de comprimento zero cai num eixo fixo do objecto), sobre a superfície **que o traço encontrou** no pincel (§4.2-ter — ⛔ esta célula dizia «a malha ACTUAL» até 2026-09-07) e sobre a **malha de agora** no filtro (§7). ⭐ **O peso FECHOU em 2026-09-07: é uniforme** — e há uma **segunda** lei, ponderada pelo ângulo do canto, que só se lê numa malha nunca esculpida e que é a que as fixtures de ESFERA trazem: **§4.2-quater** | Inflate (pincel e filtro, com leis opostas quanto ao INSTANTE) |
| 5 | **a repartição em dois baldes** pelo sinal de `n̂ · v̂` (§4.2-bis) | um balde só: o segundo nunca é usado | o segundo balde enche-se assim que o disco alcança a silhueta, e ⛔ ele só é lido se o primeiro estiver **vazio ou somar zero** (§4.2-bis-4) | normal e centro da área |
| 6 | **a distância** ao cursor e à localização da área | a distância no plano | **corda 3D, nunca geodésica**: na esfera unitária o disco de `R = 0,35` é uma calota de arco `0,3518` (`+0,5 %`) e o limite da área de `3,5R = 1,225` é uma calota de arco `1,3184` (**`+7,6 %`**) | a curva de queda, o filtro de raio, a banda da área, o filtro de construção |

⭐⭐⭐ **A LINHA 4 FECHOU em 2026-09-07, e o censo não tem mais nenhuma metade aberta.** O caminho
até aqui vale por si, porque cada degrau foi lido ao contrário do seguinte:
- **2026-09-06** — declarada aberta: no plano os três pesos dão `+ẑ` e na esfera UV a simetria em
  longitude põe-nos a menos de ruído um do outro ⇒ *o corpus não decide*. **Certo, e ainda é.**
- **2026-09-07 (errata)** — a fixture de dois traços deu **uniforme `1,7·10⁻⁵` · ângulo `3,0·10⁻⁴`
  · área `6,6·10⁻⁴`** (mediana sobre os `171` movidos), e a nota passou a *indicação*.
  ⚠️ **A indicação estava certa e o intervalo de confiança não:** medidas as três candidatas contra
  o vector de normais do próprio programa **naquela mesma malha**, elas concordam entre si a
  `≤ 0,31°` de máximo e `0,000°` de mediana — *aquela fixture não separa nada; o que a fez
  responder foi a lei estar certa, não a régua ser boa.*
- **2026-09-07 (Q18.2)** — a resposta, com a régua que a decide: perguntar as normais **ao próprio
  programa** sobre malhas em que os três pesos discordam por graus. ⇒ **uniforme**, à resolução do
  canal — e, de brinde, uma **segunda** lei que ninguém procurava (**§4.2-quater**).

⇒ **Um port implementa a soma SEM PESO de normais de face unitárias.** ⚠️ E a régua que 2026-09-06
prescrevia — *«um traço de Inflate sobre uma malha deliberadamente irregular»* — continua a ser a
régua certa para a **outra** metade, a que separa as duas leis do §4.2-quater num traço a sério; o
que a Q18.2 fez foi responder à pergunta sem ela, medindo a grandeza em vez do efeito dela.
*Uma régua indirecta que acerta não deixa de ser indirecta: aqui ela acertou o veredito com um
intervalo de confiança `20×` mais estreito do que o que a malha permitia.*

⚠️ **A leitura das oito fixtures de esfera (M) sai desta tabela, e ela NÃO é uma família só:**
- **Agarrar · Snake Hook · Aperto de linha** — linha 1 (e a 2 no aperto de linha, que monta o
  referencial): o `δ` projectado.
- **Push** — linha 2 (e a 5): a normal da área.
- **Inflate** — linha 4: a normal por vértice da superfície **que o traço encontrou** (§4.2-ter).
  ⚠️ Numa fixture de esfera de **um** passo simulado a partir do repouso as duas leituras coincidem
  por construção, logo ela fixa a **direcção e a amplitude** e **não** separa os dois instantes —
  quem os separa são os traços de plano de 12 passos e a fixture de dois traços.
- **Aperto de ponto** — ⛔ **não é curvatura**: o maior deslocamento da fixture de esfera (`0,4639`)
  é **maior** que o do plano (`0,3258`), logo ela está no **regime que o §5.2-ter descreve**, onde a
  malha inverte e quem decide o vértice é a ordem de resolução. A régua ali é a dos **dois regimes**
  do gate 20, não uma barra por vértice.
- **Expand** — ⛔ **também não é curvatura**: ele não lê nenhuma das seis linhas (só o factor por
  vértice), e o maior deslocamento da fixture é `0,046715` sobre uma malha cuja aresta no equador
  mede `0,0491`×`0,0654` ⇒ *o denominador da razão é menor que uma aresta*, exactamente como no
  `plano_expandir_radial_local_1passo`. Ele pertence à pergunta do Expand no plano.
- **Arrasto** — é a **OITAVA** fixture de esfera e não tem linha nenhuma da tabela: a direcção dele é
  a diferença dos dois pontos 3D (linha 1, o ⛔). ⭐ É por isso que ele é o **CONTROLO** da 2.ª metade
  do gate 22: se um port projectar em toda a parte, é esta fixture que o denuncia.

⚠️ **A malha da esfera das fixtures é ANISOTRÓPICA** (96 meridianos × 64 paralelos ⇒ no equador
`0,0654` na longitude contra `0,0491` na latitude — M): os comprimentos de repouso das restrições de
par do anel (§3.1) não são todos iguais, e os pólos são vértices de valência `96`. ⛔ Nada disso é
uma regra nova — é a mesma construção do §3.1 sobre outra topologia —, mas um port que assuma uma
grelha regular ao escrever o anel mede-se bem no plano e mal aqui.

---

## §5 — O solver

### §5.1 — A família, em uma frase

Verlet por posições + relaxação sequencial de restrições de distância (Jakobsen 2001; PBD §3),
**com a relaxação ANTES da integração** dentro do mesmo passo, uma rigidez global de `0,6` por
restrição, `5` varreduras, e passo de tempo `0,01` (F). Não há massa por vértice (uma massa
global), não há limitação de esticão, não há dobra própria, não há auto-colisão.

### §5.2 — As 5 varreduras de relaxação

Antes das varreduras, um **factor por vértice** é pré-calculado para TODOS os vértices da malha,
**uma vez por passo** (F):

```
φ_relax(v) = (1 − máscara_v) · auto-máscara_v · w(p⁰_v)          ⭐ TRAZ a banda
```

(`w` avaliado na posição de **repouso** do traço; sem banda nas ferramentas sem área, e `w ≡ 1` em
*Global* — §2.2.)

⚠️⚠️ **ESTE NÃO É O FACTOR DA INTEGRAÇÃO, e a espec chamava aos dois `φ`.** A §5.4 usa um
`φ_int(v) = (1 − máscara_v) · auto-máscara_v` **sem** a banda, e a banda entra lá **uma vez só**, no
termo da velocidade. Um port que use um vector só para as duas coisas paga `banda²` na retenção de
velocidade — e ⛔ o erro é **invisível em três sítios ao mesmo tempo**. O par está lado a lado, com
o censo da invisibilidade, na **§5.4-bis**; leia-a antes de escrever qualquer um dos dois.
⛔ **Abaixo, `φ_A` e `φ_B` são sempre `φ_relax`.**

Para `k = 1..5`, para cada restrição `(A, B, ℓ, s)` **cuja célula esteja activa**, em ordem (F):

```
d  = B − A                     (B: vértice, âncora, memória de forma ou repouso — §3.2)
D  = |d|
ℓ' = ℓ + (τ_A + τ_B)/2         (Expand; τ = 0 sem Expand)
Δ  = 0,6 · d · (1 − ℓ'/D)      (se D = 0: Δ = 0,6·d = 0 — H: D7184 curou a divisão por zero)
h  = Δ/2
σ  = (σ_A + σ_B)/2 se a restrição é uma ÂNCORA DE DEFORMAÇÃO (A = B ⇒ σ = σ_A), senão 1
```

- **estrutural / âncora / pino:** `A += h · φ_A · s · σ`; e, **só se B é outro vértice**,
  `B −= h · φ_B · s · σ`.
- **corpo mole (plasticidade `ρ` = *Soft Body Plasticity* do pincel activo):**
  `A += h · φ_A · s · ρ` e **a memória de forma `B −= h · φ_A · s · (1 − ρ)`** — com `ρ = 0` a
  memória segue o vértice e nunca o puxa; com `ρ = 1` o vértice volta à memória e ela não se
  mexe; entre os dois, a forma «lembra-se» parcialmente do que foi deformado (H: D9187 — a
  1.ª versão pregava à posição ORIGINAL e rompia com gravidade e com o Grab).

⚠️ **Confirmação de fonte (2026-09-06), para o arnês de paridade:**
- **A correcção é `Δ/2`, não `Δ` inteiro**, para TODA espécie (o `h = Δ/2` do laço acima é o que cada
  extremo recebe). Numa
  restrição estrutural, cada um dos dois vértices leva `Δ/2` ⇒ juntos fecham `Δ`. **Numa âncora, B não
  é vértice e NÃO se move: só A leva `Δ/2`** ⇒ a âncora fecha só metade por varredura (é «mole» de
  propósito, e por isso precisa das 5 varreduras para chegar). *Se um port dá abaixo do oráculo com
  `Δ/2`, o défice está noutro factor (o `σ` por passo do Snake Hook, o `s`, ou a re-ancoragem por
  passo), não em trocar `Δ/2` por `Δ` — o fonte é `Δ/2`.*
- **O `σ` (o factor por passo) multiplica SÓ as âncoras de DEFORMAÇÃO** (Grab, Snake Hook, pincel
  alheio): vale `1` em toda restrição que não seja âncora de deformação, e só é `(σ_A+σ_B)/2` quando a
  restrição é de deformação. ⛔ O **pino** e o **corpo mole** NÃO o levam (o seu peso é a força `s` e,
  no corpo mole, a plasticidade `ρ`).
- **A força `s` do Grab radial é `0,1 · curva(d⁰)` com a curva do PINCEL** (o preset de falloff do
  pincel activo avaliado na distância de repouso ao centro), não uma curva fixa.

⭐⭐⭐ **A ÂNCORA NÃO TEM PASSE PRÓPRIO — as quatro espécies vivem numa lista SÓ e num laço SÓ**
(F, 2026-09-06 — a resposta directa à pergunta *«ela é percorrida na mesma ordem que as de par, ou
num passe antes/depois?»*). O laço é literalmente *«cinco vezes, a lista de fio a pavio»*, e a
espécie de cada restrição só é lida **dentro** da projecção, para escolher o factor que a multiplica
e para quem vai a metade do outro lado. Em consequência, e cada uma destas é uma coisa que um port
pode ter escrito ao contrário:
1. **A ordem é a de CRIAÇÃO** (§3.1) e nada a reordena por espécie. Por vértice a ordem de nascimento
   é **corpo mole → estruturais (anel-1 e pares do anel) → âncora → pino**, e as estruturais de um
   vértice também nascem no bloco dos **vizinhos** dele ⇒ **entre duas restrições que TOCAM o mesmo
   vértice pode estar a âncora dele**. ⛔ *Não há «primeiro as distâncias, depois as âncoras».*
   ⚠️ A frase inversa também é falsa: dentro do bloco de um vértice as estruturais dele são
   contíguas, e a âncora vem **depois** delas — o que as intercala é o bloco do vizinho seguinte.
2. **A correcção é sempre `Δ/2`** — na âncora, no pino e no corpo mole também. O que faz uma âncora
   fechar só metade do que uma estrutural fecha **não é um factor diferente**: é o outro extremo não
   ser um vértice, logo o segundo `Δ/2` não é aplicado a ninguém.
3. ⚠️ **A supressão do lado `B` é uma comparação de ÍNDICE — mas ela só alcança DUAS das três
   espécies de alvo próprio** (corrigido pelo R-pré, 2026-09-06): as três guardam o **mesmo índice**
   nos dois extremos, e é essa igualdade que suprime o segundo `Δ/2` **na âncora e no pino**. ⛔ **No
   corpo mole o segundo `Δ/2` NÃO é suprimido: ele é aplicado à memória de forma**, e quem o encaminha
   para lá é a **espécie** — o corpo mole é o único que tem braço próprio na projecção. ⇒ um port com
   um só braço, guardado pela igualdade de índices, **congela a memória de forma** e a plasticidade
   deixa de existir sem que nada avise.
4. **O peso `φ` é lido por ÍNDICE, um por extremo** (`φ_A` para A, `φ_B` para B) — ⛔ excepto no
   corpo mole, onde as **duas** metades levam o `φ` de **A**: a memória de forma não tem `φ` próprio.
5. **`φ` é calculado UMA vez por passo, antes da 1.ª varredura, e para a malha INTEIRA** — não por
   célula activa e não por varredura. ⚠️ **A razão é funcional, e um port que a ignore lê o peso
   errado:** uma restrição de par liga vértices que podem pertencer a **células diferentes**, e a
   projecção precisa do `φ` dos dois no mesmo instante ⇒ o peso tem de estar disponível para
   **qualquer** vértice da malha enquanto o laço corre, e não só para os do lote em mãos.

⚠️⚠️ **`ℓ'` vale para as QUATRO espécies, e nas três de alvo próprio o incremento entra INTEIRO**
(F, 2026-09-06): a soma é sempre *«metade do desvio de cada extremo»*, e quando os dois extremos são
o mesmo vértice as duas metades somam o desvio dele por completo (§4.5).

⚠️⚠️ **O factor `(1 − ℓ'/D)` NÃO TEM TECTO, e muda de SINAL quando o par fica COMPRIMIDO** (F,
2026-09-06 — o único guarda do fonte é o `D = 0`): com `D > ℓ'` ele vive em `[0, 1)` e a correcção
puxa; com `D < ℓ'` ele é **negativo e cresce sem limite** à medida que `D → 0`, e a correcção
**empurra**, com magnitude `0,6/2 · (ℓ'/D − 1)` **vezes a separação actual** por projecção. ⇒ *é
aqui que a relaxação deixa de ser uma contracção e passa a ser um amplificador — ver §5.2-ter.*

⚠️⚠️ **E a régua da compressão tem de dizer SOBRE QUE PARES é que corre, senão subestima por `3,5×`**
(M, conferido pelo R-pré em 2026-09-06 sobre as fixtures `*.porpasso`): o pior `D/ℓ` dos doze passos
do aperto de ponto vale **`0,052`** (factor `−18,1`) se se olharem só os pares que são **arestas da
malha**, e **`0,015`** (factor **`−64,4`**) se se olharem **todos** os pares que a construção cria —
que é a população certa, porque as restrições de PAR DO ANEL (cisalhamento e dobra, §3.1) são
restrições de distância como as outras e sofrem a mesma projecção. O arrasto, na mesma régua larga,
nunca desce abaixo de `D/ℓ = 0,49` (factor `−1,1`), e a fixture de força fraca fica em `0,830`
(factor `−0,20`). ⇒ um par empurrado a **`19×`** a separação dele numa só projecção, e a lista do
*Local* projecta-o **dez** vezes por passo. ⚠️ **Quando este documento diz «par estrutural» numa
tabela de medição, é a régua LARGA que vale** — a estreita fica aqui só porque foi a primeira a ser
escrita e a diferença entre as duas é ela própria o achado.

⛔⛔⛔ **O CENSO DOS LIMITES DO SOLVER, feito de propósito para fechar a pergunta *«há algum tecto
que só se veja quando o passo é grande?»*** (F, 2026-09-06 — leitura integral do laço e da
integração, uma linha de cada vez). **A resposta é NÃO, e a lista é exaustiva:**

| o que se procurou | existe? |
|---|---|
| tecto na magnitude de uma correcção | **não** — o único guarda do laço é a separação nula |
| tecto no deslocamento acumulado de um vértice num passo | **não** |
| desistência / condição de saída antecipada por convergência | **não** — o laço é um `5` fixo, sem medir resíduo |
| número de varreduras que dependa do passo de tempo, do tamanho do gesto ou do estado da malha | **não** — `5` é uma constante do ficheiro |
| sub-passos do solver dentro de um passo de pincel | **não** (§5.7) |
| limite ao número de restrições por vértice | **não** (§3.3) |
| tecto na velocidade de Verlet, ou corte ao ultrapassar o alvo | **não** (§5.4) |
| algo que salte uma restrição | **sim, um só**: a célula dela não estar activa |

⚠️ **A única coisa em todo o passo que APARA um deslocamento vive FORA do solver e nasce desligada**
(R-pré, 2026-09-06 — o censo acima é do laço e da integração, e esta não está em nenhum dos dois): na
**escrita na malha** (§6.1) o deslocamento passa pelo **bloqueio de eixos** e pelo **recorte do
modificador de espelho** da casa, que zeram componentes inteiras. ⛔ Não é um tecto de magnitude, não
depende do tamanho do passo e **não** está activo em nenhuma fixture do §10 — mas um port que a
esqueça diverge de todo traço feito com um eixo travado.

⇒ **nada no solver lê o tamanho do gesto.** Se um port se afasta mais quando o passo é grande, a
causa não é um limite que lhe falte: é que a projecção `(1 − ℓ'/D)` é **não-linear** no esticão e o
laço deixa de ser uma contracção — e é a §5.2-quater que mede o quanto.

⚠️ **Só a POSIÇÃO é corrigida — não há projecção de velocidade separada**: a velocidade do passo
seguinte sai da diferença de posições (§5.4), logo as correcções das restrições **entram na
velocidade**.

⭐ **E a relaxação vem ANTES da integração, no mesmo passo — logo a resposta das restrições a uma
força chega com UM PASSO de atraso** (F; M — §10, os dumps de um passo): no primeiro passo simulado
a relaxação corre sobre a malha ainda em repouso (correcção zero), depois a integração aplica a
força e escreve na malha; só no passo seguinte as restrições vêem o vértice deslocado e puxam a
vizinhança. ⇒ um dab isolado (dois passos, o 1.º nunca simula) é **força pura sem resposta
elástica**, e o «pano» só existe a partir do 3.º passo do traço.

⚠️ `0,6` era `0,5` até 2020-10-18 (H: D9202 — «reduz artefactos quando restrições de tipos
diferentes disputam o mesmo vértice»). ⚠️ As forças de âncora do Grab eram `1,0` e desciam a
`0,1` em 2020-10-15 (H: D9201 — «instabilidade na zona onde o fade dava 1»; «encontrado
empiricamente, pode precisar de afinação»); a dos pincéis alheios era mais alta e desceu a `0,01`
em 2020-09-14 (H: D8884 — «impedia as dobras de se formarem»).

### §5.2-bis — A AMPLITUDE do *Local*: a lista de restrições vem em DUPLICADO (F; M 2026-09-06)

⭐⭐⭐ **O facto.** Com área *Local*, o conjunto de restrições contém **cada restrição exactamente
duas vezes** — duas cópias idênticas, na ordem «a lista inteira, e a seguir a lista inteira outra
vez». Não é uma decisão de rigidez nem um parâmetro: é a consequência de a fase 0 construir **sem
activar** (§1, §3.1, §3.3). Com *Global* cada restrição aparece **uma** vez.

⚠️⚠️ **CORRECÇÃO de 2026-09-06 (F) — no *Dynamic* a lista NÃO é simples: ela é simples DENTRO de cada
passo e duplicada nas COSTURAS entre passos.** O registo de pares já criados vive **uma construção**
(§3.1) e no *Dynamic* há **uma construção por passo** (a de cada passo constrói as células que
acabaram de entrar no alcance e ainda não foram activadas). Um par cujos dois vértices vivem em
células construídas em passos **diferentes** é criado **duas** vezes: uma pela célula que chegou
primeiro (com o anel do vértice dela a alcançar a vizinha) e outra pela célula que chegou depois.
⇒ **a frente que varre a malha deixa atrás de si uma costura de restrições em duplicado**, uma por
fronteira entre lotes de construção. ⛔ As duas cópias carregam **células diferentes**, logo cada uma
só é projectada nos passos em que a **sua** célula está activa — e num passo em que as duas estejam
activas, o par é projectado a dobrar. *No *Local* isto não acontece* (o conjunto de células é fixo e
constrói-se duas vezes inteiro); no *Global* também não (uma construção só, a malha toda).
⚠️ Isto vale para as **10** fixtures de área *Dynamic* — as 2 do plano e as **8 da esfera** —, e é uma
grandeza que se **conta**, não se escolhe: são os pares cujos dois extremos caíram em lotes de
construção distintos.

**O que isso faz.** As `5` varreduras (§5.2) percorrem a lista sequencialmente (Gauss–Seidel), logo
no *Local* cada restrição é projectada **`10` vezes por passo** em vez de `5`, com a 2.ª projecção a
ver já o resultado da 1.ª. O material fica mais rígido, o deslocamento no INTERIOR cai, e o
**alcance** cresce (numa relaxação sequencial o alcance por passo É o número de passagens).

⚠️ **É constante desde o início do traço, e NÃO acumula:** a lista já está dobrada quando a primeira
relaxação com efeito corre. As duas cópias são iguais (mesma ordem, mesmos comprimentos de repouso,
mesmas forças), porque as duas construções partem do mesmo centro fixo, do mesmo raio inicial e das
mesmas posições de repouso (F).

⚠️ **Porque é INVISÍVEL no primeiro passo simulado dos modos de FORÇA.** A relaxação corre ANTES da
integração (§5.2): nesse passo a malha ainda está em repouso e toda correcção estrutural é zero —
percorrer duas vezes uma lista de correcções nulas não muda nada. Por isso o passo 2 do *Local* e do
*Global* é idêntico (`0,09347` sob o pen-down nas duas fixtures, §10.2) e a divergência começa no
passo 3. ⛔ **Nos modos de ÂNCORA não é invisível já no passo 2**, porque a âncora é escrita na fase
4 e a restrição de âncora **não** está satisfeita quando a relaxação corre — é isso que faz um traço
de Grab de um único passo simulado mover `~1 300` vértices em vez de `~870` (M — §10.3).

⚠️ **Duplicam-se TODAS as espécies** (estrutural, âncora de deformação, corpo mole, pino), porque o
que corre duas vezes é a construção inteira (§3.2).

⚠️ **Com simetria há MAIS cópias:** a fase 0 corre uma vez por passagem de simetria e todas elas
constroem antes de qualquer activação, logo uma célula tocada por `n` passagens fica com `n` cópias
da fase 0 **mais** uma da fase 1 do 2.º passo = `n + 1` (F). Nas fixtures o factor medido é
exactamente `2` (⇒ uma passagem efectiva) — M, §10.3.

⛔ **«Pôr 10 varreduras» NÃO é o mesmo, e a medição diz quanto:** dobrar a lista intercala as duas
projecções de cada restrição de outra maneira do que dobrar o laço exterior. Um traço de arrastar
cruza o oráculo a `10` e um de agarrar entre `9` e `10` (M — §10.3). ⇒ **para um port, a forma FIEL é
a lista dobrada no ramo *Local*, não um número de varreduras diferente por área.**

⭐ **E isto FECHA a suspeita de que a correcção de âncora seria maior do que `Δ/2`** (§5.2): não é —
`Δ/2` está certo, e o défice de um port estava no número de PASSAGENS, porque todas as fixtures de
âncora são de área *Local*.

### §5.2-ter — O APERTO INVERTE A MALHA NO 1.º PASSO, e a partir daí quem decide é a ORDEM (F; M 2026-09-06)

⛔⛔⛔ **Leia isto antes de procurar uma lei de força que falte nos apertos: não falta nenhuma.** As
três coisas que um port procura primeiro **não existem** no alvo (F, conferido linha a linha):
o vértice sobre o cursor **não** tem tratamento especial além da direcção nula (§4.2) · o factor e a
direcção do aperto são avaliados **no mesmo instante** que os dos modos de arrasto — as posições com
que o passo começa, antes da relaxação dele — e só o Grab lê outro instante (§4.3) · e **não há**
tecto de deslocamento, corte ao ultrapassar o cursor nem amortecimento próprio do aperto.
⇒ *a relaxação também não faz NADA de diferente num passo de aperto: ela não sabe qual é o modo de
deformação* (nos modos de força não há sequer âncoras, e a lista de restrições nasce das posições de
REPOUSO, iguais nos dois casos).

⭐⭐⭐ **O que o aperto faz de diferente é ANTES da relaxação: ele vira a malha do avesso debaixo do
cursor, no PRIMEIRO passo simulado.** A magnitude não decresce com a proximidade e a curva de
falloff está no máximo ali (§4.2), logo o vértice ao lado do cursor anda mais do que a distância a
que estava dele. A partir daí a relaxação recebe pares **comprimidos**, onde `(1 − ℓ/D)` inverte o
sinal e cresce sem tecto (§5.2), e **o resultado por vértice passa a ser decidido pela ORDEM em que
a lista é percorrida** — que é a coisa que um Gauss–Seidel não comuta.

⭐⭐ **A PROVA está dentro do próprio oráculo, e não precisa de nós: a SIMETRIA DE ESPELHO.** A malha
de repouso, o caminho do cursor (em `y = 0`), a lei da força e o **conjunto** de restrições são todos
simétricos em relação ao traço. A **ordem** da lista não é.

⚠️⚠️ **AS DUAS RÉGUAS DA TABELA, escritas para se poderem CONSTRUIR** — sem isto o gate 19 não é
edificável, e um número medido cuja régua não está escrita não é um número (R-pré, 2026-09-06: as
duas foram **reconstruídas do zero a partir das fixtures** e devolvem, célula a célula, os valores
da tabela; se um port as escrever de outra maneira, mede outra coisa):

- **Quadrilátero de orientação invertida** = a face cuja **normal depois do passo** aponta ao
  contrário da normal dela **em repouso**. Na grelha plana isso é o sinal da componente `z` do
  **produto vectorial das DIAGONAIS** do quadrilátero, `(p₃ − p₁) × (p₄ − p₂)` com os cantos na
  ordem do quadrilátero — equivalente, na prática, à normal de Newell do polígono. Conta-se sobre
  **todas** as faces da malha, e o valor é um **inteiro**. ⛔ Não é «somar as duas metades
  triangulares»: essa leitura conta também o quadrilátero apenas DOBRADO (que não inverteu) e
  devolve `11 / 26 / 88` onde a régua certa devolve `10 / 18 / 52`.
- **Assimetria de espelho ÷ `|u|max`** = `max_v ‖u(v) − M(u(m(v)))‖∞` a dividir por `max_v ‖u(v)‖₂`,
  com `m` o vértice reflectido no plano do traço e `M` a reflexão do próprio vector de
  deslocamento. Numerador em **norma do máximo por componente**, denominador em **norma euclidiana**
  — as duas normas são diferentes de propósito, e trocá-las muda o número.

Por passo (M — fixtures `*.porpasso`):

| traço | quadriláteros invertidos `k=2 / k=3 / k=12` | assimetria de espelho ÷ `|u|max`, `k=2 / k=3 / k=12` |
|---|---|---|
| **aperto de PONTO** (força `1`) | **`10` / `18` / `52`** | `0,000` / **`0,675`** / **`1,060`** |
| **aperto de PONTO** (força `0,2` — o controlo) | **`0` / `0` / `0`** | `0,000` / `0,103` / `0,144` |
| aperto de LINHA (força `1`) | `6` / `5` / `2` | `0,000` / `0,099` / `0,204` |
| arrastar *Local* | `0` / `0` / `0` | `0,000` / `0,059` / `0,219` |
| arrastar *Global* | `0` / `0` / `57` | `0,000` / `0,064` / `0,286` |
| Snake Hook (`_2passos_origem`) | `0` / `11` / — | `0,088` / `0,283` / — |
| Grab (`_2passos_origem`) | `0` / `0` / — | `0,095` / `0,099` / — |

⭐ **Três leituras que esta tabela fecha:**
1. **A assimetria é fabricada pela relaxação e por mais nada.** Em TODOS os modos de FORÇA ela é
   `0,000000` no 1.º passo simulado — exactamente o passo em que a relaxação corre sobre a malha em
   repouso e não tem o que corrigir (§5.2) — e nasce no passo seguinte. Nos DOIS modos de ÂNCORA ela
   já lá está no 1.º passo simulado, que é exactamente o passo em que a âncora dá trabalho à
   relaxação (§5.2-bis). *A régua concorda com o mecanismo nos dois sentidos.*
2. **O piso da ordem é `6 %` a `10 %`** do maior deslocamento (arrasto e Grab, sem uma única face
   invertida). É o preço que **qualquer** port paga por não ter a mesma ordem, e é a barra honesta do
   gate 15.
3. **A inversão multiplica esse piso por `7` a `11`** (`0,675` contra `0,059`), e é a ÚNICA coisa que
   distingue os traços: as linhas com faces invertidas são as linhas com assimetria grande, na mesma
   ordem (ponto `>` gancho `>` linha `>` arrasto), e essa é **exactamente** a ordem de erro que um
   port mede contra o oráculo.

⭐⭐⭐ **E é uma INTERVENÇÃO, não uma correlação: o par de força.** A fixture nova
`plano_apertar_ponto_radial_local_origem_fraco` é o **mesmo** traço, a mesma malha, o mesmo caminho e
os mesmos parâmetros, com **uma** coisa mudada — força `1 → 0,2`, que põe o impulso máximo em
`0,004` (`0,085×` a aresta, contra `2,1×`). Resultado: **zero** faces invertidas nos doze passos, e a
assimetria cai de `0,675` para `0,103` — o piso do arrasto. ⇒ *tira-se a inversão e a
sensibilidade à ordem desaparece; o modo, a lei e a maquinaria não mudaram.*

⛔⛔ **A consequência para um port, e ela é uma decisão de PRODUTO, não de engenharia:**
- a ordem do alvo é a do §3.1 e é **cell-major**, sobre a partição da árvore espacial DELE; na malha
  destas fixtures (4 225 vértices) essa partição tem **~2 células** (§2.1) e a fronteira entre elas
  passa pela zona do pincel;
- ⇒ **num retalho invertido, o resultado por vértice do aperto não é reproduzível por uma árvore
  espacial diferente da do alvo.** Não é lei em falta: é uma resposta que a ordem define. Um port com
  outra partição fica no piso de `6 %`–`10 %` **fora** da inversão e em `≈ 70 %` **dentro** dela.
- ⇒ a barra de paridade dos modos de aperto **não pode ser por vértice num retalho invertido**
  (gate 20); e a régua que continua a valer por vértice é a de **antes** da inversão — o 1.º passo
  simulado, onde os seis modos de força já são exactos ao bit, e a fixture de força fraca inteira.
- ⚠️ **E o alvo sabe que isto é um defeito dele:** são as duas entradas abertas do §9 nº 23 —
  *artefactos dos pincéis de tecido* e *o aperto do filtro numa superfície plana*. ⇒ **reproduzir o
  oráculo aqui é reproduzir um defeito conhecido e aberto do alvo.** A saída alternativa —
  limitar o impulso do aperto à distância que falta até ao alvo, que é a única linha que a inversão
  pede — **muda o produto e diverge do oráculo de propósito**, e por isso é decisão do dono, com o
  preço já medido nesta tabela. ⛔ Não a tome sozinho.
  ⚠️ **A decisão põe-se em duas frases, e são estas** (sem elas ninguém a pode tomar sem adivinhar):
  **(a) reproduzir** — apertar com força alta vira o retalho debaixo do cursor do avesso, as faces
  atravessam-se e a superfície fica com um nó que nada desfaz depois (§11); é o que o alvo faz hoje,
  e apertar com força baixa continua limpo. **(b) limitar** — o aperto nunca ultrapassa o ponto para
  onde puxa, o nó não aparece em força nenhuma, e a nossa saída deixa de casar com a do alvo
  exactamente nos traços fortes. ⛔ **Não há terceira**: a inversão nasce no PRIMEIRO passo, antes de
  a relaxação correr, logo nenhuma afinação do solver a evita.

### §5.2-quater — ⛔⛔⛔ O que os traços de UM PASSO dos modos que escrevem ACELERAÇÃO **não** medem (F+M, 2026-09-06)

⚠️ **A palavra «força» tem duas populações neste documento, e aqui vale a estreita** (precisado pelo
R-pré, 2026-09-06): a §4.2 chama «modos de FORÇA» aos **seis** que não são de âncora, e o Expand é
um deles — mas o Expand **não escreve aceleração nenhuma**, escreve desvio de repouso (§4.5). ⇒ nesta
secção, e nos gates que ela alimenta, o sujeito são os **CINCO modos que escrevem aceleração**
(arrastar · empurrar · inflar · apertar ponto · apertar linha). *Ler «os seis de força» aqui inverte
a conclusão, porque o Expand está do outro lado da partição.*

⛔⛔⛔ **Leia isto antes de concluir seja o que for a partir de um traço de um passo simulado.** A
relaxação corre **antes** da integração (§5.2). Num traço de um passo de um modo que escreve
aceleração, a malha com que a relaxação se encontra está **em repouso**, todo par está exactamente no
comprimento de repouso e `τ` é zero ⇒ **todas as correcções que ela calcula são identicamente zero**.
O que sobra é a força, dividida pela massa, vezes o passo de tempo. ⇒ *um traço desses mede a área, a
banda, o factor por vértice, a curva, a dureza, a direcção do modo e a integração — e **não toca numa
única restrição de distância**.*

⭐⭐ **E isso CONTA-SE, não se argumenta** (M, 2026-09-06; recontado do zero pelo R-pré). A régua, por
inteiro, para que o censo seja reconstruível: **movido** = `|u| > 1e-5` sobre as posições a seis casas
(é a mesma régua com que o cabeçalho de cada fixture enche o campo `movidos`); **disco** = os vértices
cuja posição de **repouso** dista menos de `R` da **ponta do caminho** (o cursor do passo simulado).
⚠️ **O disco tem tamanho diferente conforme onde a ponta cai na grelha** — `173` vértices nas fixtures
com o pen-down em `x = −0,3` (ponta em `+0,3`), que são as deste censo, e `177` nas de pen-down na
origem (ponta em `+0,6`). ⚠️ Nos dois modos de âncora o resultado é o mesmo com qualquer das duas
pontas do caminho, porque o disco inteiro se move de qualquer maneira.

| traço de UM passo (pen-down em `x = −0,3`) | movidos DENTRO do disco | movidos FORA do disco | total (= o `movidos` do cabeçalho) |
|---|---|---|---|
| arrastar · empurrar · inflar · apertar ponto | `171` | **`0`** | `171` |
| arrastar com massa `2` | `171` | **`0`** | `171` |
| arrastar com força `0,5` | `168` | **`0`** | `168` |
| apertar linha | `156` | **`0`** | `156` |
| **expandir** | `173` | **`675`** | `848` |
| **agarrar** | `173` | **`1151`** | `1324` |
| **gancho** | `173` | **`1279`** | `1452` |

⇒ **os SETE traços de um passo dos cinco modos que escrevem aceleração movem o disco e mais nada; os
três que falham movem entre 5 e 8,5 vezes mais vértices, e a esmagadora maioria deles é material que
força nenhuma tocou.** Aquele material só pode ter sido movido pela relaxação. *A partição «modos que
escrevem aceleração» × «modos que escrevem âncora ou repouso» é, medida, a partição «traços que não
exercitam a rede de restrições» × «traços cuja resposta É a rede de restrições».*

⭐⭐⭐ **A INTERVENÇÃO que o prova: dobrar a lista muda o resultado de um passo em 20–30 %.** A área
*Local* projecta cada restrição `10` vezes por passo e a *Global* `5` (§5.2-bis). Correndo o **mesmo
gesto, o mesmo caminho, o mesmo raio e a mesma força**, só a área a mudar (M, corrida nova de
2026-09-06, pen-down na origem, um passo simulado):

| traço de um passo | *Local* (`10` projecções) | *Global* (`5`) | razão |
|---|---|---|---|
| gancho | `0,456101` | `0,343172` | `0,752` |
| agarrar | `0,134311` | `0,094722` | `0,705` |
| expandir | `0,001914` | `0,001525` | `0,797` |

⚠️ **A leitura honesta da diferença:** a *Global* também constrói para a malha inteira e tem `w ≡ 1`
— mas na janela do perfil (`d < 1,6 R`) a banda vale `1` nas duas (§2.2: ela só começa a cair a
`2,875 R`), logo o que resta a explicar `25 %` do pico é o **número de passagens**. *Nenhum dos sete
traços de força de um passo consegue ver esta diferença: neles as duas áreas dão o mesmo, porque
percorrer duas vezes uma lista de correcções nulas não muda nada.*

⭐⭐⭐ **O «degrau» do perfil é a rede, e o seu SÍTIO move-se com o número de passagens.** Perfil ao
longo da linha do traço, um passo, `δ = 0,6`. ⚠️ **A amostragem, por inteiro** (precisada pelo R-pré,
2026-09-06, que reproduziu a tabela célula a célula): os vértices são os que estão **sobre o eixo do
traço** (`y = z = 0`) em `x = k · aresta`, `k = 0..7`, com `aresta = 0,046875` — logo `d` é a
distância de repouso ao **pen-down**, medida **no sentido do traço**, e `d/R = k · 0,13393`.
⛔ `d` não é a distância ao centro da queda: nestas fixtures o pen-down está na origem, e é dele que
a tabela parte nos dois modos.

| `d/R` | gancho *Local* | gancho *Global* | agarrar *Local* | agarrar *Global* |
|---|---|---|---|---|
| `0,000` | `0,4513` | `0,3149` | `0,1343` | `0,0900` |
| `0,134` | `0,4419` | `0,3188` | `0,1262` | `0,0843` |
| `0,268` | `0,4559` | `0,3357` | `0,1098` | `0,0714` |
| `0,402` | `0,3743` | `0,2954` | `0,0895` | `0,0553` |
| `0,536` | **`0,1947`** | **`0,0577`** | `0,0697` | `0,0399` |
| `0,670` | **`0,0431`** | `0,0351` | `0,0529` | `0,0276` |
| `0,804` | `0,0392` | `0,0327` | `0,0400` | `0,0188` |
| `0,938` | `0,0329` | `0,0233` | `0,0302` | `0,0128` |

Razão entre células consecutivas: o gancho *Local* faz `1,02 · 0,97 · 1,22 · 1,92 · **4,52** · 1,10
· 1,19` e o gancho *Global* faz `0,99 · 0,95 · 1,14 · **5,11** · 1,65 · 1,07 · 1,40`. ⇒ **é o mesmo
degrau, deslocado UMA célula para dentro quando as passagens caem de `10` para `5`.**
⭐ **E o agarrar não tem degrau nenhum** (`1,06 · 1,15 · 1,23 · 1,29 · 1,32 · 1,32 · 1,32`): ele
decai suavemente e assenta numa cauda de `1,32` por célula (*Local*) / `1,47` (*Global*). *A cauda é
a rede a transmitir; o degrau é a rede a **ceder** — e só o gancho a leva ao ponto de ceder.* ⚠️ O
que os separa **não** é a lei do alvo (as duas são quadráticas na queda: no agarrar a força da
restrição já traz a curva, no gancho é o factor por passo que a traz): é a **magnitude** — âncora de
`0,35` contra `0,1 · curva`, e um pico de `0,4561` contra `0,1343` no mesmo passo, `3,4×`.

⭐⭐⭐ **E o degrau é um fenómeno de `δ` GRANDE — desaparece a `δ` pequeno com tudo o resto igual.** O
mesmo gancho, um passo, com o percurso encurtado de `0,6` para `0,05` (≈ uma aresta) dá
`1,07 · 1,15 · 1,22 · 1,27 · 1,30 · 1,31 · 1,31 · 1,31` — ⚠️ **oito razões, logo NOVE células**
(a tabela acima pára em `0,938 R`; esta segue até `1,072 R`), e a 4.ª é `1,2747`, que arredonda a
`1,27` (o R-pré recontou-a em 2026-09-06). É **a curva do agarrar, sem degrau**, e o
pico vale `0,649 · δ` contra `0,760 · δ` no traço longo (M: `plano_gancho_radial_local_origem_1passo`
e `…_curto`). ⇒ *a resposta de um passo nem sequer é proporcional a `δ`: a projecção
`(1 − ℓ/D)` deixa de ser uma mola quando o par é esticado várias vezes o comprimento dele, e passa a
devolver uma fracção fixa da separação ACTUAL — um travão muito mais duro. É essa não-linearidade
que fabrica o degrau, e é ela que um port ligeiramente mais mole alisa.*

⭐⭐ **A prova de que a rede também ULTRAPASSA (o gémeo em TRACÇÃO do achado de compressão do
§5.2-ter):** com a curva *Constant* — que põe `f_v = 1` em todo o disco e `0` fora dele, tornando o
alvo da âncora uniforme e o perfil de fora um medidor puro da rede — o mesmo passo devolve
**`0,8790`**, que é **`1,465 × δ`**, com **`107` vértices a andarem MAIS do que o alvo da própria
âncora** e o máximo a `0,552 R` do centro, não no centro (M:
`plano_gancho_radial_local_origem_1passo_constante`; com a curva *Smooth* são `0` vértices). *Uma
âncora não pode empurrar um vértice para além do ponto para onde puxa: quem o leva lá é a cadeia de
pares esticados, a projectar em Gauss–Seidel mais vezes do que a contracção aguenta.* ⚠️ E o degrau
dessa fixture está a `1,071 R → 1,205 R` (razão `5,3`), que é **onde `f_v` cai a zero** — ⇒ *a
posição do degrau é onde a âncora deixa de alimentar e a rede assume; a forma dele é da rede.*

⚠️ **O que isto manda fazer, e por que ordem:** o traço que isola a rede com **zero** força e **zero**
âncora é o `plano_expandir_radial_local_origem_1passo` — nele a única coisa que existe é a lista de
restrições, a ordem dela e o número de passagens. ⇒ **fechá-lo primeiro**; os dois modos de âncora
somam-lhe a lei da âncora, e só depois é justo lê-los.

### §5.3 — O que «damping» é, de facto

Não é um amortecimento de Rayleigh nem uma viscosidade: é **a fracção de velocidade PERDIDA por
passo**, aplicada multiplicativamente à velocidade de Verlet (§5.4). Omissão do código `0,01`
(⇒ `99 %` de retenção); faixa `0,01..1` no pincel, `0..1` no filtro (F). E é **modulado pela
banda**: retenção efectiva `= (1 − damping) · w(p⁰_v)` (H: D9084 — «ajuda a fundir artefactos com
áreas dinâmicas, porque o amortecimento cresce quando o vértice se afasta»). ⇒ a documentação
(«quanto as forças se propagam») é a leitura invertida de «quanta velocidade sobrevive».

### §5.4 — A integração (por célula activa, por vértice, com `φ_int = (1 − máscara) · auto-máscara`, ⛔ SEM banda)

```
φ_int(v) = (1 − máscara_v) · auto-máscara_v            ⛔ NÃO traz a banda (≠ φ_relax da §5.2)

v̄        = x − x_prev                    (x já corrigido pelas 5 varreduras deste passo;
                                          x_prev = o x corrigido do passo ANTERIOR)
x_prev   ← x
x       += a · φ_int · 0,01               (a = Σ forças / massa; dt = 0,01)   ⛔ sem banda
x       += v̄ · φ_int · (1 − damping) · w(p⁰_v)                              ⭐ a banda, UMA vez
colisão (§5.6)
x_col    ← x                              (origem do raio de colisão do próximo passo)
a        ← 0
```

⚠️⚠️ **A banda aparece EXACTAMENTE uma vez neste bloco, e o `φ` daqui não é o da §5.2** — a lei
inteira, os três sítios em que a banda podia entrar e o que cada engano custa estão na **§5.4-bis**.
⚠️ O termo de aceleração **não** é multiplicado por `w` aqui (a força já o trazia, §4.1) — mas é
pelo `φ_int` de máscara. ⚠️ **A massa é um ganho inverso puro** sobre um `dt` fixo: dobrar a massa
divide exactamente por dois o deslocamento por força num passo (F; M — §10). Faixa `0,01..2`,
omissão `1`.

### §5.4-bis — ⭐⭐⭐ OS DOIS `φ` E OS TRÊS SÍTIOS DA BANDA (F, 2026-09-07 — confirmado lado a lado a pedido do I)

⛔⛔ **A espec chamava `φ` a duas grandezas diferentes, em duas secções, e a diferença entre elas é
a diferença entre `banda` e `banda²`.** Esta secção existe para que isso deixe de depender de o
leitor reparar. Confirmado no fonte, os **três** sítios em que a banda `w(p⁰_v)` poderia entrar num
passo, e o que de facto acontece em cada um:

| # | sítio | traz a banda? | o factor completo |
|---|---|---|---|
| 1 | o factor das **cinco varreduras** de relaxação (§5.2), pré-calculado uma vez por passo para a malha inteira | ⭐ **SIM** | `φ_relax = (1 − máscara) · auto-máscara · w(p⁰)` |
| 2 | o termo de **aceleração** da integração (§5.4) | ⛔ **NÃO** | `φ_int = (1 − máscara) · auto-máscara` |
| 3 | o termo de **velocidade** da integração (§5.4) | ⭐ **SIM, e é a única vez ali** | `φ_int · (1 − damping) · w(p⁰)` |

⚠️ **A ordem dentro da integração é load-bearing e mede-se:** o factor chega à integração **sem**
banda; o termo de aceleração consome-o assim; **depois** ele é escalado por `(1 − damping)`;
**depois** disso é multiplicado pela banda; e só então escala a velocidade. ⇒ um port que aplique a
banda ao factor **à entrada** da integração escreve `banda` também no termo de aceleração, e um que
use `φ_relax` nos dois escreve `banda²` na retenção de velocidade.

⚠️ **A banda dos dois é avaliada na posição de REPOUSO** (`p⁰`), não na de agora — nos dois sítios,
e é a mesma lei do §2.2. E nos dois ela só entra **quando há traço activo**: no filtro (§7), que não
tem traço, não há banda nenhuma.

⛔⛔⛔ **O CENSO DA INVISIBILIDADE — é por isto que um `φ` só sobrevive a um corpus inteiro** (M, o I
mediu-o em 2026-09-07 e a leitura confere com a lei acima): o erro `banda²` está calado em três
sítios ao mesmo tempo —

1. na área ***Global*** a banda é `1` em toda a malha ⇒ `1² = 1`, e **todos** os traços *Global* do
   corpus ficam byte-idênticos;
2. no termo de **aceleração** o factor extra vale exactamente `1` de qualquer maneira, porque a
   força já corta em `d ≥ R` e a banda só começa a descer a `R(1+L·F) = 2,875 R` — os dois suportes
   **não se tocam**;
3. no anel entre `2,875 R` e `3,5 R`, que é o único sítio onde ele morde, o deslocamento já é de
   ordem `10⁻³` ⇒ o erro fica três ordens de grandeza abaixo da barra do gate 15.

⇒ **o único instrumento do corpus que o vê é um traço de plano em área *Local*, comparado por
vértice sobre a malha inteira** (não pelo máximo, não pelo centro): ali o erro sobe a `3,9·10⁻³`
contra `< 5·10⁻⁶` com a lei certa, enquanto o mesmo traço em *Global* sai **byte-idêntico** com as
duas leis (ali `banda ≡ 1`, logo `banda² = banda`; o I mediu `2·10⁻⁵` nos dois casos) — é a razão
entre esses dois traços que denuncia o defeito, e não o valor de nenhum deles.
*Uma grandeza que só é observável num anel de `0,6 R` de espessura precisa de uma régua que olhe
para a malha inteira; um máximo nunca lá chega.*

### §5.5 — Massa, passo de tempo, iterações: os números e o RECURSO que cada um nomeia

| constante | valor | de que recurso é | proveniência |
|---|---|---|---|
| varreduras de relaxação por passo | `5` | tempo por passo × rigidez aparente (mais varreduras = pano mais inextensível) | F |
| rigidez por restrição | `0,6` | estabilidade quando espécies de restrição diferentes disputam um vértice | F · H D9202 |
| passo de tempo | `0,01` | escala do deslocamento por força (`0,1/massa` a força máxima) | F |
| força da âncora Grab | `0,1 × fade` (radial) · `0,1` (plano) | estabilidade na zona de fade = 1 | F · H D9201 |
| força da âncora Snake Hook | `0,35` (× `f_v` por passo) | — | F |
| força da âncora de pincel alheio | `0,01` | «deixar as dobras formarem-se» | F · H D8884 |
| incremento de repouso do Expand | `0,01 · f` por passo | — | F |

### §5.6 — Colisão (opcional, *Use Collisions*, nasce desligada)

Colisores = todos os objectos visíveis da cena **com modificador de colisão** e árvore de
aceleração construída, na pose do quadro (F). Por vértice, depois da integração, para cada
colisor (F):

1. raio, em espaço do MUNDO, de `x_col` (posição pós-integração do passo anterior) até `x`
   (actual), com comprimento `|x − x_col|`; o teste é raio-vs-triângulo (watertight na
   precalculação, mas o cast é feito sem a bandeira watertight) com **espessura de raio `0,3`**
   (uma constante absoluta, em unidades de mundo — F).
2. Se há impacto **dentro do comprimento do raio**: `x ← ponto de impacto + n̂_impacto · 0,005 +
   0,35 · (projecção de x no plano do impacto − ponto de impacto)` — i.e., o vértice pára na
   superfície, é afastado `0,005` pela normal, e **conserva 35 %** do deslizamento tangencial
   que pretendia (fricção `0,65`) (F).
3. Volta ao espaço do objecto.

Limitações **declaradas pelos autores** (H: blog 2020-10): um vértice que já esteja DENTRO do
colisor nunca é expulso (colisão por raio, não por campo de distância); sem auto-colisão, e a
razão nomeada é o tamanho da célula-folha da estrutura espacial. Os autores nomeiam «fricção» e
«distância à superfície» como parâmetros que ficaram por expor (H: D8019). Uma issue aberta
(#96124) reporta que o filtro ignora a espessura exterior do colisor.

### §5.7 — O que o solver NÃO tem (ausências afirmadas por leitura integral)

Sem limitação de esticão · sem massa por vértice · sem modelo de dobra · sem sub-passos (um passo
de solver por passo de pincel) · sem projecção de velocidade · sem auto-colisão · sem detecção
contínua (o raio de colisão é o único CCD, e só contra colisores) · sem re-malhagem (o pincel
recusa topologia dinâmica — F) · sem cache entre traços.

### §5.7-bis — ⛔⛔⛔ O CENSO DA RESPOSTA AO ESTICÃO: as três coisas que um port procura quando fica abaixo do alvo, e que NÃO existem (F, 2026-09-06)

⚠️ **Porque esta secção existe.** Um port fiel pode acertar o 1.º passo simulado **ao bit** e ficar
`5`–`9 %` **abaixo** do alvo a partir do 3.º — que é o primeiro em que a relaxação tem trabalho para
fazer (§5.2-quater) — e ficar assim **só** nos modos que empurram a folha para fora do plano dela,
onde os pares esticam por construção. A leitura natural é *«falta-me um termo que só acorda com o
esticão»*. **Não falta.** O fonte foi relido de propósito para esta pergunta, linha a linha, e o
censo devolve **nenhum**:

| o que se procurou | existe? |
|---|---|
| um segundo factor na projecção que dependa do esticão, além de `(1 − ℓ'/D)` | **não** — o factor é esse e mais nada; o único guarda é a separação nula (§5.2) |
| um tecto na correcção quando o par se afasta muito do repouso | **não** — e é o mesmo `não` do censo do §5.2, agora conferido a partir da pergunta oposta |
| um **segundo passe** sobre as restrições muito esticadas | **não** — o conjunto de restrições é percorrido **inteiro**, `5` vezes, e nenhuma passagem selecciona as mais esticadas |
| uma correcção de **ordem superior** (o termo é linear na separação, sempre) | **não** |
| o comprimento de repouso lido de **outro** sítio quando o par estica | **não** — `ℓ` é gravado uma vez, na criação, e o único termo que lhe soma é o desvio do Expand (§4.5) |
| uma restrição, peso ou termo que só entre com componente ao longo da **normal** | **não** — e ver o parágrafo seguinte |
| rigidez **angular** / modelo de dobra próprio (uma restrição sobre um ÂNGULO) | **não** — a §3.1 continua verdadeira: o papel de dobra é feito pela restrição de **distância ao segundo vizinho pelo anel**, e é uma restrição de distância como as outras |
| uma quinta espécie de restrição que a §3.2 não liste | **não** — são quatro, e a espécie só é lida **dentro** da projecção (§5.2) |
| **sub-passos**: um passo de pincel que se subdivide quando o deslocamento é grande | **não** — um passo de pincel corre **um** passo de solver, e o número de varreduras é um valor **fixo do programa** — não é parâmetro, e não lê o `dt`, nem o tamanho do gesto, nem o estado da malha (§5.5) |
| um `dt` variável, ou um `dt` que dependa do deslocamento | **não** — é a mesma constante em todo passo (§5.4) |

⭐⭐⭐ **O que ISSO deixa como explicação, e é a razão de o §10.10 existir:** se nenhum termo falta,
então uma diferença que **nasce com o esticão** só pode vir de (a) a **ordem** em que as projecções
correm — a relaxação é sequencial, cada uma lê a posição que a anterior deixou, e a influência da
ordem cresce com o tamanho das correcções, logo é **invisível** num gesto quase-rígido e visível
assim que os pares esticam (§3.1-bis, §5.2-ter); ou (b) a **população** de restrições — quantas
existem e quantas vezes cada uma é projectada (§5.2-bis); ou (c) um erro na **fase do gesto** que
só se manifesta depois de a malha se mexer, porque quase tudo o que a fase do gesto lê é lido na
posição **actual** (§4.1). ⇒ *as três são separáveis por medição, e são-no pelas oito fixtures do
§10.10 — nenhuma delas precisa de um facto novo do alvo.*

⚠️ **E há uma quarta que não é do solver nem do gesto: a régua.** Um port que compare `máx |u|` do
fim do traço está a ler **um** número de um regime não-linear; a comparação que localiza é a de
**cada passo** (§10.2), e a que isola é a que **corta a força fora** (§10.10).

⭐⭐⭐ **FECHO (2026-09-07, M — §10.11): era a (c), e o censo estava certo.** Nenhum facto faltava ao
solver, e as três hipóteses eram separáveis: o mesmo arnês que reproduz o **arrasto** com
`err_max = 3,7·10⁻⁶` sobre a malha inteira — usando exactamente esta ordem, esta população e esta
retenção — reproduz o **empurrar** com `0,2369`. ⇒ a ordem e a população estão **ilibadas por
resultado**, não por argumento. O que falhava eram **duas leituras da fase do gesto**, as duas sobre
a mesma grandeza: as normais são as da superfície **que o traço encontrou** (§4.2-ter) e não as de
agora, e por isso o Push **cala-se por completo** nos passos em que a cova passa o disco de
amostragem (§4.2-bis (8)). Com as duas, os dez traços descem a `≤ 5·10⁻⁶` (gate 41).
⚠️ **A leitura *«o resíduo está na resposta ao ESTICÃO»* (Q16 do INBOX) era a inferência natural e
estava errada**: o défice crescia com o esticão porque o esticão é o que afunda a cova, e a cova é o
que cala o gesto — *duas grandezas que crescem juntas, e a que se mediu não era a causa.*

---

## §6 — Como a deformação é cometida, o que sobrevive, undo, simetria

### §6.1 — A escrita na malha

A cada passo, por célula activa: `Δx_v = x_v − posição avaliada actual`; `Δx` passa pelo
**bloqueio de eixos** e pelo **recorte do modificador de espelho** da casa; depois é somado às
posições originais, às avaliadas (quando há modificadores por cima) e às *shape keys*
dependentes, e as caixas da célula são actualizadas (F). ⇒ O que a malha guarda é **sempre** o
estado da simulação — não há passo de «assar».

### §6.2 — Passos, espaçamento, âncora

- Um **passo** = uma chamada do laço do §1. Com o método de traço *Dots* (o dos 13 presets — A),
  é **um passo por evento de movimento**. Com *Space*, o espaçamento é calculado sobre um raio
  fictício de `100 px` e não sobre o raio do pincel, **para o ritmo da simulação não depender do
  raio** (F): passo a cada `2·espaçamento` px (espaçamento `10` ⇒ `20 px`; os presets de Grab usam
  `3` ⇒ `6 px`). A casa mantém o espaçamento ligado para os modos de âncora **de propósito**, para
  que se possa escolher entre «simula sempre» e «simula só quando a mão anda» (F).
- Traço **anchored** / *drag dot*: ao contrário de todos os outros pincéis, a malha **NÃO** é
  reposta a cada passo — a simulação continua do estado anterior (F; H: D8348).
- **Pressão**: muda a força (`pressão` em `B`) e, se *Size Pressure*, o raio actual `R` (que a
  área *Dynamic* usa e a *Local* ignora).

### §6.3 — O que sobrevive

| entre PASSOS do mesmo traço | entre TRAÇOS |
|---|---|
| `x` (relida da malha), `x_prev`, `x_col`, `a` (zerada), as restrições e os estados das células, as âncoras de deformação e os seus `σ`, a memória de forma (corpo mole), os desvios de repouso `τ`, a lista de colisores | ⛔ **nada da simulação** — ela nasce com o traço e morre com ele. A MALHA fica deformada, e o traço seguinte constrói restrições **da malha deformada** (deformação acumula) — salvo *Persistent* (§6.4) |

### §6.4 — Base persistente

O operador *Set Persistent Base* copia as posições (e normais) actuais para um atributo da malha
(F). Com *Persistent* ligado no pincel, **os comprimentos de repouso e o teste de raio da
construção** usam essas posições em vez das actuais ⇒ o mesmo pano de partida pode ser simulado
vezes seguidas com forças diferentes sem acumular (H: D8428). ⚠️ As âncoras do Grab, o pino e a
memória de forma continuam a partir das posições **actuais** do início do traço (F). ⚠️ Em malhas
sem atributos persistentes (multires) a base vive só na sessão de escultura (H: #133267); e
houve um período em que a opção era no-op para o pincel de tecido (H: #134781, 2025-02).

### §6.5 — Undo

Não há nada de especial: o traço inteiro é **um** passo de undo (posições das células tocadas,
registadas na primeira vez que cada célula é tocada no traço) (F). A única particularidade é a
do §6.2 — o pincel de tecido está **excluído** do «repor a malha antes do passo» dos traços
anchored. Uma issue de 2020 sobre undo (#82388) foi do sistema de undo, não do pincel (H).

### §6.6 — Simetria

Espelho, radial e ladrilho são **passagens sequenciais** do laço inteiro (§1), cada uma com
localização, delta, normal e gravidade reflectidos/rodados. Com área *Local*, no 1.º passo, **todas
as passagens constroem as restrições ANTES de qualquer activação** — senão a 2.ª passagem
encontrava a célula já activada pela 1.ª e não lhe acrescentava as suas (H: D9303, T81904).
⚠️ Uma issue **aberta** (#131122) reporta que o Grab de tecido e os pincéis de fronteira com alvo
de simulação não funcionam com simetria — o mecanismo não está descrito nela.

### §6.7 — Multires e topologia dinâmica

Multires: o mesmo laço sobre as grelhas de subdivisão (vizinhos = os da grelha, com costuras) (F).
Topologia dinâmica: o pincel **recusa** (não está na lista dos que a suportam) (F).

---

## §7 — O filtro de tecido

Mesmo solver, sem pincel (F):

| aspecto | o que o filtro faz |
|---|---|
| **área** | todas as células não totalmente mascaradas/escondidas; restrições construídas UMA vez, para TODOS os vértices (raio infinito), ao carregar; sem banda (`w ≡ 1`); sem pino |
| **um passo** | a cada movimento do rato: guardar estado (§1 fase 2) → forças → activar todas → passo de simulação |
| **força escalar `S`** | `S = força_base · (x_rato − x_pressão) · 0,001 · escala_UI` — ⭐ arrastar para a **direita** é positivo, e a magnitude é **pixels** (`0,001` por px a força base `1`); `força_base` é o parâmetro *Strength* (omissão `1`, faixa `−10..10`) |
| **factor por vértice** | `(1 − máscara) · auto-máscara · (0 se fora do face set activo, com *Use Face Sets*) · S` |
| **Gravity** | força `= M · (0, 0, −f)` — ou `(0, −f, 0)` na orientação *View* (na vista, o eixo da gravidade é o −Y do ecrã, para que a queda seja o «baixo» que o artista vê e não a profundidade — F) — com `M` a matriz da orientação (*Local* = identidade · *World* = inversa da matriz do objecto · *View* = inversa da vista × inversa do objecto) |
| **Inflate** | força = normal **actual** do vértice × f — ⚠️⚠️ **e o «actual» aqui é literal, ao contrário do pincel:** cada passo do filtro repete a preparação do objecto para edição, que **refresca as normais**, enquanto o traço do pincel a corre uma vez só e por isso lê as do início (§4.2-ter). ⇒ *a mesma palavra nomeia duas leis*; um port que partilhe o código do Inflate entre os dois tem de lhe passar QUAL fotografia usar (F, conferido 2026-09-07) |
| **Expand** | `τ_v += 0,01 · f` (§4.5) |
| **Pinch** | força = unitário do vértice **para o vértice activo no momento em que o filtro começou** (o ponto NÃO segue o rato) × f |
| **Scale** | ⭐ é o único filtro por ÂNCORA: âncora `= p⁰_v + p⁰_v · f` com as componentes dos eixos desligados anuladas (no referencial da orientação) ⇒ escala em torno da **origem do objecto**; força de âncora `0,01` |
| **gravidade da cena** | somada em TODOS os tipos: `ĝ · g · S` com `ĝ` = −Z do objecto de gravidade ou `(0,0,−1)`, `g` = *Gravity* da escultura |
| **Force Axis** | bandeiras X/Y/Z (omissão: as três) — para o Scale anulam componentes da âncora; para as forças, ⚠️ o código lido só as aplica ao Scale (a limitação de eixos das forças passa pela orientação) |
| **Orientation** | *Local* (omissão) · *World* · *View* — define `M` acima **e** a direcção da gravidade do tipo *Gravity* |
| **massa / damping** | omissão `1,0` (faixa `0..2`) / **`0,0`** (faixa `0..1`) — ⚠️ o filtro nasce **sem** perda de velocidade nenhuma |
| **colisões** | idem §5.6, opção nasce desligada |
| undo | um passo por uso do filtro (início ao carregar, fim ao largar) |
| cancelar | há uma issue aberta (#105335): o botão direito não cancela de imediato |

⚠️ O filtro **não** tem os modos Drag/Push/Grab/Snake Hook/Pinch Perpendicular (são gestos de
pincel — precisam de um cursor com direcção).

---

## §8 — Constantes e omissões, com o recurso e a proveniência

### §8.1 — Do CÓDIGO (as omissões de um pincel novo)

| controlo (rótulo) | omissão | faixa | recurso que nomeia |
|---|---|---|---|
| Deformation | Drag | 8 | — |
| Force Falloff | Radial | Radial · Plane | — |
| Simulation Area | **Local** | Local · Global · Dynamic | tempo (Global simula tudo) |
| Simulation Limit `L` | `2,5` | `0,1..10` | tempo × alcance |
| Simulation Falloff `F` | `0,75` | `0..1` | — |
| Pin Simulation Boundary | **off** | bool | estabilidade vs. liberdade (§2.3) |
| Cloth Mass | `1,0` | `0,01..2` | ganho inverso (§5.4) |
| Cloth Damping | `0,01` | `0,01..1` | retenção de velocidade (§5.3) |
| Soft Body Plasticity | `0,0` | `0..1` | — |
| Use Collisions | off | bool | tempo |
| Persistent | off | bool | — |
| Normal Weight (só Grab/Snake Hook) | `0` | `0..1` | — |
| Gravity (cena) | `0` (M) | — | — |

### §8.2 — Dos PRESETS que o artista vê (A — os 13 pincéis de tecido da biblioteca do binário)

⭐ **Os presets contradizem as omissões do código em quatro colunas, e são eles que o artista
recebe.** Nomes = os rótulos dos presets; tipo = o modo de deformação (ou o pincel alheio com alvo
= simulação):

| preset | modo | área | `L` | damping | plasticidade | força | espaçamento | curva | pressão→força | colisões |
|---|---|---|---|---|---|---|---|---|---|---|
| Drag Cloth | Drag | **Dynamic** | 2,5 | 0,01 | **0,5** | 0,6 | 10 | Smooth | sim | **on** |
| Push Cloth | Push | Dynamic | 2,5 | 0,01 | 0,3 | 0,85 | 10 | **Sharp** | sim | on |
| Pinch Point Cloth | Pinch Point | Dynamic | **3,5** | 0,01 | 0,4 | 0,3 | 10 | Smooth | sim | on |
| Pinch Folds Cloth | Pinch Perpendicular | Dynamic | 2,5 | 0,01 | 0,4 | 0,5 | 10 | Smooth | sim | on |
| Inflate Cloth | Inflate | Dynamic | 2,5 | **0,4** | 0 | 0,3 | 10 | Smooth | sim | on |
| Expand/Contract Cloth | Expand | Dynamic | 2,5 | **0,4** | 0 | 0,6 | 10 | Smooth | sim | on |
| Grab Cloth | Grab | **Local** | **5,0** | **0,6** | 0 | 1,0 | 3 | Smooth | não | on |
| Grab Random Cloth | Grab (com jitter) | Local | 5,0 | 0,6 | 0 | 1,0 | 3 | Smooth | não | on |
| Grab Planar Cloth | Grab, **Force Falloff = Plane** | Dynamic | 2,5 | 0,01 | 0 | 0,5 | 3 | Smooth | não | on |
| Stretch/Move Cloth | pincel **Pose** (escala/translação) com alvo = simulação | — | — | 0 (o pincel alheio cria a simulação com damping `0`) | 0 | 0,5 | 10 | Smooth | não | — |
| Bend/Twist Cloth | pincel **Pose** (rotação/torção) com alvo = simulação | — | — | 0 | 0 | 0,5 | 10 | Smooth | não | — |
| Bend Boundary Cloth | pincel **Boundary** (dobrar) com alvo = simulação | — | — | 0 | 0 | 0,5 | 3 | Sharp | não | — |
| Twist Boundary Cloth | pincel **Boundary** (torcer) com alvo = simulação | — | — | 0 | 0 | 0,5 | 3 | Sharp | não | — |

Todos: falloff `F = 0,75`, massa `1`, pino **off**, *Front Faces Only* off, raio em pixels `70`
(não travado à cena), método de traço **Dots**. ⚠️ **Não há preset de Snake Hook** — o modo só
existe no menu. ⚠️ Os pincéis Pose/Boundary com alvo = simulação criam a simulação com **massa 1,
damping 0, sem plasticidade, sem colisão** e força de âncora `0,01`, e o passo de simulação corre
**só na passagem principal de simetria, depois de todas as passagens terem escrito as âncoras** (F).

### §8.3 — O cursor (o que o artista vê da área)

Círculo **tracejado** em `R·(1 + L·F)` (alfa `0,5×`) e círculo **contínuo** em `R·(1+L)` (alfa
`0,7×`), sempre que a área não é *Global* (F). Durante um traço *Local*/radial, se o cursor sair da
área, os dois círculos são redesenhados **a vermelho, na localização inicial** (F). Com *Force
Falloff = Plane*, em vez dos círculos: o segmento com setas do §4.4 (F).

### §8.4 — O painel (ordem exacta, F)

Simulation Area · [se ≠ Global] Simulation Limit · Simulation Falloff · [se Local] Pin Simulation
Boundary · Deformation · Force Falloff · Cloth Mass · Cloth Damping · Soft Body Plasticity · Use
Collisions — mais os genéricos da casa (Strength, Radius, Persistent + Set Persistent Base, Front
Faces Only, Normal Weight nos modos de âncora, curva, dureza, textura, auto-mascaramento).

---

## §9 — A sabedoria dos autores, re-dita, com proveniência

⛔ **Os links abaixo são para páginas do repositório do alvo — o I NÃO os abre.** Estão aqui
como proveniência (as mensagens de commit são públicas; o texto foi re-dito). Referências por
`D<n>` (revisão), `T<n>`/`#<n>` (issue), data.

| # | o que os autores aprenderam | fonte |
|---|---|---|
| 1 | A construção das restrições é «extremamente básica», repete restrições, e podia ser mais barata e multi-thread — eles sabem, e deixaram assim porque «funciona ok». | D6715, 2020-02-28 |
| 2 | O Grab original acumulava o delta a partir da posição do passo anterior e usava força errada; foi refeito para **escrever posições** a partir das originais, com a simulação a resolver o resto. Os autores **tentaram e rejeitaram** um Grab por forças — «isto é mais controlável, e o falloff do Grab afina-se pelo falloff da simulação». | D7756, 2020-05-19 |
| 3 | Dois vértices coincidentes davam divisão por zero na correcção — daí o teste `D = 0` do §5.2. | D7184 / T74808, 2020-03 |
| 4 | Gravidade só dentro do raio do pincel «não faz sentido» — passou a actuar em toda a área simulada, sem escalar pelo raio. | D8406, 2020-07-28 |
| 5 | O «corpo mole» nasceu como pino à posição ORIGINAL com força regulável; **rompia** com gravidade e com o Grab; foi substituído pela **plasticidade** com memória de forma deformável — «muito melhor e mais previsível», e «aumenta a estabilidade». | D7845 → D9187, 2020-07/10 |
| 6 | O pino da fronteira existe porque forças grandes (anchored, pinch, grab a força total) **rompiam** a simulação na fronteira; com ele «a simulação não se parte seja qual for a força»; é opção porque estraga pincéis que deformam a malha inteira. | D8435, 2020-07-31 |
| 7 | Área *Global* nasceu porque as pessoas punham `L = 10` para simular tudo — «um hack, porque os limites escalam com o raio». | D8481, 2020-08-06 |
| 8 | A força das âncoras dos pincéis alheios era alta demais e «impedia as dobras de se formarem» — desceu a `0,01`, com a intenção (não cumprida) de a expor por pincel. | D8884, 2020-09-14 |
| 9 | Pincéis alheios com alvo = simulação **não têm área nem banda** — afectam a malha inteira. | D8885, 2020-09-17 |
| 10 | Área *Dynamic*: activa células e constrói restrições **durante** o traço; «sem restrições de comprimento do traço, área, nem número de vértices»; as células fora da área não são resolvidas nem colididas. | D8726, 2020-10-01 |
| 11 | O damping passou a ser modulado pela banda «porque ajuda a fundir artefactos com áreas dinâmicas». | D9084, 2020-10-01 |
| 12 | As âncoras do Grab a força `1` na zona de fade `= 1` davam instabilidade — força escalada por um factor **«encontrado empiricamente»**, que «pode precisar de afinação». | D9201, 2020-10-15 |
| 13 | A rigidez por restrição passou de `0,5` a `0,6`·(½) para reduzir artefactos quando várias espécies de restrição disputam um vértice. | D9202, 2020-10-18 |
| 14 | O falloff de plano existia desde o início mas dava «imensos artefactos» nos modos de âncora por falta de funcionalidades no solver; o Grab usou radial à força até o plano ser reimplementado **pelas âncoras** (com a força reescrita a cada passo). | D9320, 2020-10-22 |
| 15 | Com espelho e área *Local*, a 2.ª passagem encontrava a célula já activada e não acrescentava as suas restrições — construir tudo **antes** de activar. | D9303 / T81904, 2020-10-23 |
| 16 | Uma regressão de desempenho: o ramo *Local* corria também o código *Global* por um `if` que devia ser `else if`. | D9762 / T83201, 2020-12 |
| 17 | Snake Hook: «muda a força das restrições de deformação por passo para afectar o resultado da simulação o menos possível»; «agarra o pano sem produzir artefactos na superfície e cria dobras mais naturais do que qualquer outro modo». | D8621, 2020-08-24 |
| 18 | Colisões por raio: colidem com **qualquer** geometria (mesmo não-manifold) e o vértice pára na superfície; a desvantagem nomeada: um vértice dentro do colisor nunca sai; o plano era colisão por SDF; auto-colisão bloqueada pelo tamanho da célula-folha. | blog 2020-10-20 |
| 19 | O pincel foi posicionado, pelo autor, como «substituto de pincéis com alfas de tecido, quando só se quer FINGIR detalhe de pano em partes da malha»; «um solver de pano a sério que corre a simulação completa dá resultados mais exactos». | blog 2020-02-25 |
| 20 | Regressão de 2024 no Pinch: uma multiplicação foi trocada por uma subtracção num refactor — o comportamento «de antes» é a multiplicação (a do §4.2). ⚠️⚠️ **CORRIGIDO NA ESPEC (2026-09-06): ela foi CONSERTADA no mesmo dia em que foi relatada, DOIS ANOS antes da versão que gravou as fixtures, e o consertar foi voltar à multiplicação** — a versão do oráculo lê o aperto tal como o §4.2 o descreve (F: direcção × factor), e **não** há divergência deliberada a declarar aqui. ⇒ *uma entrada de regressão FECHADA numa tabela de história lê-se como dívida viva; esta diz agora a data do conserto.* | #127836 relatada e fechada em 2024-09-19 |
| 21 | Com falloff *Constant* o corte no raio tem de ser explícito (a curva constante não corta) — daí o corte duro do §4.1. | #139846, 2025-06-06 |
| 22 | O Grab com falloff de plano estoirava ao clicar fora da malha (o cache do traço passou a existir antes de haver superfície) — a cerca é «não desenhar/agir sem superfície sob o cursor». | #161820, 2026-07-23 |
| 23 | **Abertos**: artefactos dos pincéis de tecido (#138844) · distorção grande em malhas de densidade irregular (#131510 — ⚠️ consistente com forças ABSOLUTAS e restrições ao anel-1, §4.1/§3.1) · Grab de tecido e Boundary com alvo = simulação não funcionam com simetria (#131122) · o Pinch do filtro numa superfície plana (#132316) · o filtro ignora a espessura exterior do colisor (#96124) · o botão direito não cancela o filtro de imediato (#105335). | tracker, 2024–2026 |

---

## §10 — Vectores de teste (o oráculo)

⭐ **Os traços do binário 5.2.1 sobre malhas NOSSAS** — ⚠️ **CONTE-OS, não cite número nenhum de
memória** (`ls docs/3D/cleanroom/fixtures/cloth/*.deformado.txt.gz | wc -l`): esta linha esteve em
`51` depois de a §10.5 acrescentar dois, a §10.6 mais um, a §10.7 mais dois, a §10.8 mais **nove**,
a §10.10 mais **oito**, a §10.11 mais **três** e a §10.12 mais **dois** — e esteve parada em `76`
enquanto a §10.12 entrava.
⚠️ **A tabela abaixo NÃO é o corpus** — ela tem as `47` linhas da 1.ª geração, e os traços das
§10.2–§10.11 vivem nas secções delas. O corpus é o directório.
⚠️ **O `indice.json` é DERIVADO e regenera-se** — `python3 fixtures/cloth/gera_indice.py` (uma
entrada por `.deformado.txt.gz`); ⛔ não o edite à mão, e não confie num número escrito aqui. Malhas: grelha plana 64×64 e esfera UV
96×64; um traço por modo e por variante de solver, em `fixtures/cloth/` (proveniência e verificador
no README de lá).
Colunas: **movidos** = vértices com `|u| > 1e-5` · **máx `|u|`** em unidades de objecto · **alcance/R**
= distância máxima de um vértice movido ao caminho, sobre o raio · **fracção normal** = `Σ|u·n⁰|/Σ|u|`
(quanto levanta contra quanto desliza) · **coerência** = módulo do vector unitário médio dos
deslocamentos grandes (`1` = uma direcção só; `0` = radial) · **Δárea** = variação da área da grelha.

| fixture | modo | falloff | área | passos | movidos | máx `|u|` | alcance/R | fracção normal | coerência | Δárea |
|---|---|---|---|---|---|---|---|---|---|---|
| `plano_arrastar_plano_local` | Drag | plane | local | 12 | 2146 | `0.8996` | `3.49` | `0.00` | `0.99` | +15.41 % |
| `plano_arrastar_radial_dinamica` | Drag | radial | dynamic | 12 | 2508 | `0.6128` | `3.49` | `0.00` | `1.00` | +3.21 % |
| `plano_arrastar_radial_dinamica_preset` | Drag | radial | dynamic | 12 | 2455 | `0.3296` | `3.45` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_global` | Drag | radial | global | 12 | 4225 | `0.6446` | `5.49` | `0.00` | `1.00` | +1.17 % |
| `plano_arrastar_radial_local` | Drag | radial | local | 12 | 2144 | `0.3316` | `3.49` | `0.00` | `1.00` | -0.07 % |
| `plano_arrastar_radial_local_1passo` | Drag | radial | local | 2 | 171 | `0.0992` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_2passos` | Drag | radial | local | 3 | 1438 | `0.1359` | `3.25` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_amort1` | Drag | radial | local | 12 | 2141 | `0.2199` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_arrastar_radial_local_amort05` | Drag | radial | local | 12 | 2142 | `0.2544` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_massa2` | Drag | radial | local | 12 | 2143 | `0.1546` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_massa2_1passo` | Drag | radial | local | 2 | 171 | `0.0496` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_pino` | Drag | radial | local | 12 | 2144 | `0.3235` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_plast05` | Drag | radial | local | 12 | 2141 | `0.2343` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_arrastar_radial_local_forca05` | Drag | radial | local | 12 | 2139 | `0.0733` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_arrastar_radial_local_forca05_1passo` | Drag | radial | local | 2 | 168 | `0.0248` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_expandir_radial_local` | Expand | radial | local | 12 | 2134 | `0.0115` | `3.49` | `0.00` | `0.35` | +0.00 % |
| `plano_expandir_radial_local_1passo` | Expand | radial | local | 2 | 848 | `0.0019` | `2.94` | `0.00` | `0.10` | +0.00 % |
| `plano_agarrar_plano_local` | Grab | plane | local | 12 | 2146 | `0.3076` | `3.49` | `0.00` | `1.00` | +1.24 % |
| `plano_agarrar_radial_local` | Grab | radial | local | 12 | 2139 | `0.1699` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_agarrar_radial_local_1passo` | Grab | radial | local | 2 | 1324 | `0.1341` | `3.20` | `0.00` | `1.00` | +0.00 % |
| `plano_agarrar_radial_local_24passos` | Grab | radial | local | 24 | 2142 | `0.1585` | `3.49` | `0.00` | `1.00` | -0.05 % |
| `plano_agarrar_radial_local_2passos` | Grab | radial | local | 3 | 1872 | `0.1461` | `3.41` | `0.00` | `1.00` | -0.00 % |
| `plano_agarrar_radial_local_amort06` | Grab | radial | local | 12 | 2131 | `0.1315` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_agarrar_radial_local_preset` | Grab | radial | local | 12 | 4123 | `0.1326` | `5.49` | `0.00` | `1.00` | -0.14 % |
| `plano_inflar_radial_local` | Inflate | radial | local | 12 | 2146 | `0.3172` | `3.49` | `0.95` | `1.00` | +3.02 % |
| `plano_inflar_radial_local_1passo` | Inflate | radial | local | 2 | 171 | `0.0992` | `0.99` | `1.00` | `1.00` | +0.20 % |
| `plano_apertar_linha_radial_local` | Pinch Perpendicular | radial | local | 12 | 2135 | `0.1005` | `3.49` | `0.00` | `0.03` | +0.11 % |
| `plano_apertar_linha_radial_local_1passo` | Pinch Perpendicular | radial | local | 2 | 156 | `0.0876` | `0.99` | `0.00` | `0.00` | +0.16 % |
| `plano_apertar_ponto_plano_local` | Pinch Point | plane | local | 12 | 2146 | `0.6239` | `3.49` | `0.00` | `1.00` | +33.42 % |
| `plano_apertar_ponto_radial_local` | Pinch Point | radial | local | 12 | 2146 | `0.3258` | `3.49` | `0.00` | `0.77` | +5.90 % |
| `plano_apertar_ponto_radial_local_1passo` | Pinch Point | radial | local | 2 | 171 | `0.0992` | `0.99` | `0.00` | `0.02` | +0.19 % |
| `plano_empurrar_plano_local` | Push | plane | local | 12 | 2146 | `0.5201` | `3.49` | `0.98` | `0.99` | +8.25 % |
| `plano_empurrar_radial_local` | Push | radial | local | 12 | 2145 | `0.2590` | `3.49` | `0.94` | `1.00` | +2.08 % |
| `plano_empurrar_radial_local_1passo` | Push | radial | local | 2 | 171 | `0.0694` | `0.99` | `1.00` | `1.00` | +0.10 % |
| `plano_gancho_radial_local` | Snake Hook | radial | local | 12 | 2140 | `0.0915` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_gancho_radial_local_1passo` | Snake Hook | radial | local | 2 | 1452 | `0.4894` | `3.37` | `0.00` | `0.99` | +2.26 % |
| `plano_gancho_radial_local_24passos` | Snake Hook | radial | local | 24 | 2142 | `0.0293` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_gancho_radial_local_2passos` | Snake Hook | radial | local | 3 | 1950 | `0.3648` | `3.43` | `0.00` | `0.99` | +1.07 % |
| `plano_gancho_radial_local_amort06` | Snake Hook | radial | local | 12 | 2135 | `0.0634` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `esfera_arrastar_radial_dinamica` | Drag | radial | dynamic | 12 | 2183 | `0.5828` | `3.48` | `0.59` | `0.96` | — |
| `esfera_expandir_radial_dinamica` | Expand | radial | dynamic | 12 | 2096 | `0.0467` | `3.44` | `0.83` | `0.98` | — |
| `esfera_agarrar_radial_dinamica` | Grab | radial | dynamic | 12 | 1863 | `0.2365` | `3.49` | `0.65` | `0.99` | — |
| `esfera_inflar_radial_dinamica` | Inflate | radial | dynamic | 12 | 2181 | `0.2670` | `3.48` | `0.70` | `0.91` | — |
| `esfera_apertar_linha_radial_dinamica` | Pinch Perpendicular | radial | dynamic | 12 | 2162 | `0.2497` | `3.47` | `0.59` | `0.21` | — |
| `esfera_apertar_ponto_radial_dinamica` | Pinch Point | radial | dynamic | 12 | 2183 | `0.4639` | `3.47` | `0.61` | `0.76` | — |
| `esfera_empurrar_radial_dinamica` | Push | radial | dynamic | 12 | 2102 | `0.4794` | `3.47` | `0.93` | `1.00` | — |
| `esfera_gancho_radial_dinamica` | Snake Hook | radial | dynamic | 12 | 2234 | `0.1690` | `3.52` | `0.62` | `0.88` | — |

### §10.1 — O que os dumps confirmam (cada número casa com a secção citada)

- **A força é absoluta e a massa é ganho inverso (§4.1, §5.4):** um passo de Drag a força `1` desloca o vértice sob o cursor `0.09917` (previsto `0,1·f`, com `f = 0,99` no vértice mais próximo do cursor ⇒ `0,099`); com massa `2`: `0.04958` (razão `0.5000`, prevista `0,5`); com força `0,5`: `0.02479` (razão `0.2500`, prevista `0,25` — a força entra ao QUADRADO). Inflate e Pinch Point dão o mesmo módulo no 1.º passo (`0.09917` · `0.09917`); **Push dá `0.06942`** = `2R·0,099` com `R = 0,35` (razão Push/Drag `0.700`, prevista `2R = 0,70`).
- **A relaxação atrasa um passo (§5.2):** no dab de um passo só os `171` vértices dentro do raio se movem (`alcance/R = 0.99`) e o Expand de um passo move **`848`** vértices; com dois passos simulados o Drag já arrasta `1438` vértices até `3.25 R`.
- **Direcção (§4.2):** coerência (módulo do vector unitário médio dos deslocamentos grandes) — Drag `1.00` (uma direcção só), Pinch Point `0.02` (radial ⇒ soma nula), Pinch Perpendicular `0.00`; fracção normal Inflate `1.00` (para cima), Push `1.00` (para baixo: `u_n` mínimo `-0.0694`), Drag `0.00` (no plano).
- **Velocidade retida (§5.3):** Drag 12 passos, `máx|u|` = `0.3316` com damping `0,01`, `0.2544` com `0,5`, `0.2199` com `1,0`; com o falloff de PLANO a faixa inteira acumula velocidade e passa do percurso do cursor (`0.8996` para um percurso de `0,6`).
- **Área (§2.1):** Local — alcance `3.49 R` (a esfera de células `R₀(1+L) = 3,5 R` a partir do pen-down); Dynamic — `3.49 R` a partir do caminho, `máx` `0.6128` (segue o cursor); Global — todos os `4225` vértices.
- **Âncoras (§4.3):** Grab radial 12 passos `máx` `0.1699` e 24 passos `0.1585` (mesmo percurso: a resposta depende do percurso, não do número de passos); Grab de plano `0.3076`; Snake Hook `0.0915` (12) e `0.0293` (24 — metade do delta por passo ⇒ ~metade da resposta: lei quadrática no falloff); com damping `0,6` (o dos presets de Grab): Grab `0.1315`, e com `L = 5` `0.1326`.
- **Expand (§4.5):** 12 passos, `máx|u|` `0.0115` (`0.25` arestas) e Δárea `+0.002 %`.

⚠️⚠️ **As fixtures de ESFERA são todas de área DINÂMICA (ERRATA de 2026-09-06).** A 1.ª entrega
gravou-as como *Local*, mas um traço scriptado **não dispara o hover** que fixa o centro da área
Local — esse centro fica na ORIGEM do objecto (o valor obsoleto da **localização inicial** guardada —
a célula «centro» do Local na tabela do §2.1). Numa esfera
unitária a origem põe **toda** a malha dentro da banda (todo vértice a `1,0` < início da banda
`1,006`), então a saída lida `6 050 / 6 050` movidos, uniformes — a esfera a deslocar-se como um
CORPO, que é **artefacto do arnês, não comportamento Local do alvo**. ⭐ **O R₀ estava certo (`0,35`,
tamanho travado à cena); o defeito era o CENTRO.** A área Dinâmica, cujo centro é o cursor de cada
passo (que o traço scriptado FORNECE), funciona: as 8 fixtures de esfera param no bordo da banda
(`alcance ≈ 3,5 R`, `0` vértices além), e é sobre elas que se lê o relevo fora do plano numa
superfície curva. A área **Local** fica medida **só no plano** (onde a origem cai na superfície e o
disco de `3,5 R` é exacto). Mecanismo e a errata completa: [ledger](LEDGER_blender-cloth.md).

---

### §10.2 — O instrumento POR PASSO (2026-09-06, a pedido do I)

Quatro traços com as posições **depois de cada passo** (`fixtures/cloth/*_origem.porpasso.txt.gz`,
método e prova no README de lá): Drag *Local* e *Global* (12 passos), Snake Hook e Grab *Local*
(2 passos simulados). ⚠️ **Pen-down na origem** (determinismo do centro *Local*, ver README).
O rastreio de sete vértices por passo (`*.porpasso.rastreio.txt`) é o lado MEDIDO para a Q3 do I
(Local ≈ metade do Dinâmico/Global): ⭐ **o que os números dizem** —

| passo | Local: sob o pen-down | Global: sob o pen-down | Local: no limite 3,5R | Global: no limite 3,5R | Local: fora 4R | Global: fora 4R |
|---|---|---|---|---|---|---|
| 1 | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 2 | `0.09347` | `0.09347` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 3 | `0.16758` | `0.20530` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 4 | `0.21314` | `0.27913` | `0.00000` | `0.00005` | `0.00000` | `0.00001` |
| 5 | `0.24711` | `0.33335` | `0.00000` | `0.00028` | `0.00000` | `0.00011` |
| 6 | `0.27291` | `0.38610` | `0.00002` | `0.00103` | `0.00000` | `0.00049` |
| 7 | `0.29041` | `0.44464` | `0.00004` | `0.00273` | `0.00000` | `0.00157` |
| 8 | `0.29737` | `0.49123` | `0.00008` | `0.00584` | `0.00000` | `0.00388` |
| 9 | `0.29100` | `0.54262` | `0.00013` | `0.01074` | `0.00000` | `0.00797` |
| 10 | `0.27014` | `0.58434` | `0.00019` | `0.01766` | `0.00000` | `0.01427` |
| 11 | `0.24062` | `0.61514` | `0.00025` | `0.02659` | `0.00000` | `0.02293` |
| 12 | `0.22022` | `0.64571` | `0.00032` | `0.03738` | `0.00000` | `0.03378` |

1. **O bordo do Local é uma ÂNCORA e o do Global não existe:** no limite (`3,5R`) o Local move
   `≤ 0,0003` em 12 passos e **fora dele exactamente `0`** (sem restrições, `φ = 0`); o Global move o
   mesmo vértice `0,037` e o de fora `0,034` — a folha inteira desliza. ⇒ o disco Local é uma membrana
   **presa no aro** por `w → 0` DENTRO do conjunto restringido (o raio de construção é o LIMITE, não o
   início da banda).
2. **A consequência está no pen-down:** no Local o vértice do pen-down sobe até `0,297` (passo 8) e
   depois **RECUA** para `0,220` — o material atrás do cursor é puxado de volta para o aro preso; no
   Global sobe monotonamente até `0,646` (a folha acompanha o cursor com a velocidade acumulada).
   Sob o cursor do passo os dois são parecidos (`≈ 0,20–0,23` contra `≈ 0,22–0,31`).
3. **Até ao passo 2 os dois são IGUAIS ao bit** (`0,09347` / `0,00072` / `0`) e só divergem no
   passo 3. ⚠️ **A 1.ª leitura desta linha — «logo o mecanismo é o aro» — está REFUTADA** (§10.3):
   os DOIS mecanismos previam esta igualdade, porque no passo 2 a relaxação corre sobre uma malha em
   repouso (§5.2-bis). O aro explica as colunas `3.5R`/`4R`; o **interior** é a lista duplicada.
⇒ **Um port cujo `limite_3.5R` não seja `≈ 0` e cujo `fora_4R` não seja exactamente `0` no Local tem
o aro LIVRE**: ou cria restrições só até ao início da banda (`R(1+L·F)`) em vez de até ao limite
(`R(1+L)`), ou o `w` não chega a `0` dentro do conjunto restringido, ou `φ` não multiplica a
correcção do lado do vértice do aro. Régua directa: `limite_3.5R` tem de ler `≈ 0` e `fora_4R`
exactamente `0` no Local — e `> 0,03` nos dois no Global.
⚠️ **Mas um port com o aro CERTO e o interior igual ao Global tem outra coisa, e é a §5.2-bis:** as
colunas do aro batem com a lista simples, e o interior não. As duas leituras separam-se pela coluna,
não pelo passo.

### §10.3 — A experiência das varreduras (2026-09-06) — o lado MEDIDO da §5.2-bis

Medição nossa, com a nossa lei, variando **só** o número de varreduras de relaxação por passo (M):

| traço | varreduras que reproduzem o oráculo |
|---|---|
| arrastar radial **Global** (12 passos) | **5** — erro `≤ 4 %` em 12 passos × 5 colunas |
| arrastar radial **Local** (12 passos) | **10** — erro `≤ 3 %`, e só a `10` aparece o **pico-e-recuo** (máximo no passo 8, recuo até ao 12); a `5` a curva é monótona crescente |
| agarrar radial **Local** (2 passos) | cruza entre **9** e **10** |

⭐⭐ **O botão parte o corpus exactamente na linha *Local* / não-*Local*** (50 fixtures, `5` contra
`10` varreduras, erro relativo por traço): *Local* (38) — **27 melhoram**, 7 ficam ao bit em `0` (os
de um passo de força, §5.2-bis), 4 pioram; *Global* (2) — **os dois pioram**; *Dynamic* (10) — **9
pioram**. Ordens de grandeza, não afinação: arrastar *Local* `1,253 → 0,071`; arrastar *Global*
`0,175 → 0,565`.

⭐⭐⭐ **A prova que não é amplitude ajustada são as CONTAGENS de vértices movidos, que são inteiros:**
a `10` varreduras oito traços *Local* passam a mover **exactamente** o número do oráculo, e nos
traços de um e dois passos — onde não há acumulação possível — o alcance salta para o dele: agarrar
`869 → 1307` (oráculo `1324`), expandir `597 → 840` (`848`), arrastar `1050 → 1428` (`1438`). *Numa
relaxação sequencial o alcance por passo É o número de passagens ⇒ a contagem de movidos MEDE as
passagens, e ela diz `~2×` no ramo Local dentro de UM passo de pincel.*

⛔ **Duas coisas que esta medição NÃO explica** (e que não se devem misturar com ela): o Snake Hook
de 2 passos e o apertar-ponto *Local* pioram a `10` — no Hook o pico do port não está sob o cursor,
que é defeito de LOCALIZAÇÃO e não de amplitude; e na esfera os modos que não são arrasto erram em
*Dynamic* sem que as varreduras lhes toquem.

### §10.4 — ONDE está o pico (2026-09-06) — o lado MEDIDO da §4.3 do Snake Hook

A sonda por passo imprime a distância do vértice mais deslocado ao cursor **daquele passo**, em
raios (M):

| traço | pico do port (centro no cursor) | pico do oráculo |
|---|---|---|
| arrastar radial *Local*, passo 8 | `0,82R` | `0,82R` |
| arrastar radial *Local*, passo 11 | `1,02R` | `1,02R` |
| gancho radial *Local*, passo 2 | `0,05R` | **`0,86R`** |
| gancho radial *Local*, passo 3 | `0,24R` | **`0,91R`** |

⇒ **o arrasto está no sítio certo e o gancho não:** o pico do oráculo fica onde o pincel **estava**.
No 1.º passo simulado o vértice mais deslocado do oráculo é **o do pen-down** (`max = c0`), que é o
que a §4.3 prevê com o centro atrasado (`δ` anterior `= 0`).

⭐ **Medido pelo port com o centro atrasado (mutação de uma linha), 7 traços de gancho de 7 melhoram**
(`err_max/max_oráculo`, exemplos: `1passo` `0,999 → 0,467`; `2passos_origem` `0,700 → 0,324`;
o traço longo `0,162 → 0,129`), e a **contagem de movidos** do traço de um passo vai de `1040` para
`1434` contra `1452` do oráculo — outro inteiro a convergir.

⛔ **E o resíduo que sobra NÃO é a força da âncora** (varrida de `0,20` a `1,00` no traço de um
passo): a `0,35` — o valor que a §3.2 já dá — a amplitude bate (`0,4935` contra `0,4894`) e sobra
`0,2036` de erro; a `0,50` o erro desce e a **amplitude estoura `25 %`**. Nenhum valor torna o traço
exacto ⇒ a constante da espec está certa e o resíduo é de forma.

⚠️ **Quanto à forma que falta: no alvo NÃO existe eixo, plano nem limite de profundidade próprios do
Snake Hook** (leitura integral da fase de gesto, 2026-09-06). O que dá forma ao gancho, e que um port
pode não ter, é o par: **(a)** o centro atrasado desta secção e **(b)** a distância medida sobre as
posições **actuais** (§4.3) — juntos fazem o material já puxado viajar com o centro. O «plano de
profundidade» da §4.3 é do **delta** (a des-projecção do cursor), e vale para os oito modos; a queda
por-vértice é a distância comum ao centro, com a forma de queda que o pincel tiver — a **esférica**
por omissão, ou a *Projected* (medida no plano da vista) quando o artista a escolhe. É uma opção do
pincel, comum aos oito modos, **não** uma lei do modo.

### §10.5 — Os dois traços de APERTO por passo (2026-09-06, a pedido do I)

`plano_apertar_ponto_radial_local_origem` e `plano_apertar_linha_radial_local_origem`: 12 passos
cada, corridas-prefixo com `prova_do_fatiamento = 0,000000`, mesmo método e mesmo caminho (pen-down
na origem) da §10.2. Entregues porque os dois modos de aperto são exactos no traço de um passo e
divergem no fim do traço inteiro — só o dump por passo diz **em que** passo nasce.

⭐ **O que o rastreio já mostra, e o estado final escondia** — `|u|` sob o pen-down, passos 2..12:

| modo | `|u|` sob o pen-down, por passo |
|---|---|
| **aperto de PONTO** | `0,093 · 0,184 · 0,118 · 0,106 · 0,197 · 0,208 · 0,201 · 0,187 · 0,160 · 0,149 · 0,154` |
| **aperto de LINHA** | `0,000 · 0,001 · 0,001 · 0,002 · 0,005 · 0,006 · 0,005 · 0,004 · 0,004 · 0,003 · 0,003` |

⚠️ **O aperto de ponto NÃO é monótono** — sobe, **desce** no passo 4, volta a subir e desce de novo:
a força aponta para o **cursor**, que se afasta a cada passo, logo o vértice é puxado e depois
largado, e o pico anda com o cursor. Uma lei que integre monotonamente ultrapassa (é o sinal que o I
mede no fim do traço). ⚠️ **E o aperto de LINHA quase não move o pen-down** (`≤ 0,006`, contra
`0,10` no vizinho a `1R`): ele aperta contra a **linha** do traço, e o que está SOBRE a linha já lá
está.

---

### §10.6 — O CONTROLO do par de força do aperto (corrida NOVA do oráculo, 2026-09-06)

`plano_apertar_ponto_radial_local_origem_fraco` — **`.deformado` + `.porpasso` + `.porpasso.rastreio`**,
12 passos, `prova_do_fatiamento = 0,000000`. É o **mesmo** traço de aperto de ponto da §10.5 (mesma
malha, mesmo caminho a partir da origem, mesma área *Local*, mesmos limite/banda/massa/amortecimento),
com **UMA** coisa mudada: **força `1,0 → 0,2`**. Existe para ser a outra metade do par do §5.2-ter.

| | força `1,0` | força `0,2` |
|---|---|---|
| impulso máximo por passo (`10·α·dt/massa`) | `0,100` = **`2,1×` a aresta** | `0,004` = **`0,085×` a aresta** |
| vértices movidos no fim | `2 145` | `2 029` |
| máx `|u|` no fim | `0,303401` | `0,004082` |
| **quadriláteros invertidos**, passos 2 / 3 / 12 | **`10` / `18` / `52`** | **`0` / `0` / `0`** |
| **assimetria de espelho ÷ `|u|max`**, passos 2 / 3 / 12 | `0,000` / **`0,675`** / `1,060` | `0,000` / `0,103` / `0,144` |
| pior compressão, **só sobre pares que são ARESTAS** (`D/ℓ`, dos 12 passos) | `0,0523` ⇒ factor `−18,1` | `0,8304` ⇒ factor `−0,20` |
| pior compressão, **sobre TODOS os pares da construção** (a régua larga, §5.2) | `0,0153` ⇒ factor `−64,4` | `0,8304` ⇒ factor `−0,20` |

⚠️ **Para que serve, do lado do port:** ela é a régua por vértice que os apertos ainda admitem. Num
traço em que a malha nunca se inverte, o aperto é tão comparável quanto o arrasto; a partir do
momento em que se inverte, a barra tem de ser a do gate 20, não a por vértice. ⛔ **Não a use como
prova de que o aperto está certo** — ela prova o contrário do que parece: prova que o que separava as
duas leituras era a inversão, não uma lei.

### §10.7 — Os dois traços de FORÇA NORMAL por passo (corrida NOVA do oráculo, 2026-09-06, a pedido do I)

`plano_empurrar_radial_local_origem` e `plano_inflar_radial_local_origem` — **`.deformado` +
`.porpasso` + `.porpasso.rastreio`**, 12 passos cada, `prova_do_fatiamento = 0,000000` nos dois
(26 corridas-prefixo: `k = 1..12` de cada traço mais a corrida inteira da mesma sessão).
⚠️ **Pen-down na ORIGEM**, pela mesma razão das §10.2/§10.5: o centro da área *Local* de um traço
scriptado é a **localização inicial** guardada, e um traço cujo pen-down não é a origem depende de o
sobrevoo ter chegado (errata do §10). ⛔ Não confunda estes dois com os `plano_empurrar_radial_local`
e `plano_inflar_radial_local` do corpus (pen-down em `x = −0,3`): são o **mesmo gesto** com outro
ponto de partida, e os números abaixo são os destes.

`|u|` **depois** do passo `k`, dos vértices de repouso nomeados (deslocamentos perpendiculares ao
traço a partir do pen-down) e do vértice mais próximo do cursor do passo:

| passo | Push: pen-down | Push: 1R | Push: 3,5R | Push: sob o cursor | Inflate: pen-down | Inflate: 1R | Inflate: 3,5R | Inflate: sob o cursor |
|---|---|---|---|---|---|---|---|---|
| 2 | `0.06543` | `0.00050` | `0.00000` | `0.06990` | `0.09347` | `0.00072` | `0.00000` | `0.09986` |
| 3 | `0.17038` | `0.00423` | `0.00000` | `0.18733` | `0.22771` | `0.00857` | `0.00000` | `0.24879` |
| 4 | `0.22446` | `0.02977` | `0.00000` | `0.24728` | `0.26348` | `0.05248` | `0.00000` | `0.28459` |
| 5 | `0.23493` | `0.07046` | `0.00001` | `0.25578` | `0.26706` | `0.10238` | `0.00001` | `0.29137` |
| 6 | `0.23822` | `0.10596` | `0.00003` | `0.25839` | `0.26888` | `0.13989` | `0.00004` | `0.30880` |
| 7 | `0.23968` | `0.13452` | `0.00007` | `0.24698` | `0.27012` | `0.16933` | `0.00010` | `0.31915` |
| 8 | `0.23848` | `0.15728` | `0.00013` | `0.23497` | `0.27060` | `0.19245` | `0.00018` | `0.32474` |
| 9 | `0.23546` | `0.17465` | `0.00020` | `0.22357` | `0.27017` | `0.20985` | `0.00026` | `0.32592` |
| 10 | `0.23108` | `0.18680` | `0.00026` | `0.23925` | `0.26880` | `0.22213` | `0.00035` | `0.32387` |
| 11 | `0.22561` | `0.19418` | `0.00032` | `0.23761` | `0.26641` | `0.23007` | `0.00043` | `0.31358` |
| 12 | `0.21945` | `0.19761` | `0.00036` | `0.23225` | `0.26294` | `0.23452` | `0.00049` | `0.31013` |

⭐ **O que eles dizem, e que o traço inteiro esconde:**
1. **O passo 2 confirma a razão `2R` ao bit** — Push `0,06543` contra Inflate `0,09347` dá
   `0,7000` = `2 · 0,35` (§10.1). *No 1.º passo simulado a normal da área e a normal do vértice são a
   MESMA coisa (a folha está plana e em repouso): os dois traços só divergem a partir do passo 3.*
2. **O pen-down do Push satura e RECUA** (`0,2397` no passo 7 → `0,2195` no passo 12) e o do Inflate
   **fica** (`0,2701` → `0,2629`) — ⛔ e a diferença não é o módulo da força: é que a normal da área é
   uma média sobre meio raio (§4.2-bis), logo o Push empurra ao longo de uma direcção que **roda com
   a vala** enquanto o Inflate empurra ao longo da normal **do próprio vértice**.
3. **O aro está preso nos dois** (`3,5R` fica em `0,0004`/`0,0005` ao fim de 12 passos, `4R` em zero
   exacto) — o mesmo aro do §10.2, ⇒ estes dois traços **não** medem a fronteira, medem o interior.
4. **A frente a `1R` cresce monotonamente nos dois** e **quase alcança** o pen-down do Push no passo
   12 (`0,1976` contra `0,2195` — ⛔ não o ultrapassa em passo nenhum dos doze) — é a onda que a relaxação leva para fora, e ela é
   a coluna que mais depende do número de passagens (§5.2-bis).

---

### §10.8 — As NOVE corridas que isolam a REDE de restrições (corrida NOVA do oráculo, 2026-09-06, a pedido do I)

Todas com **pen-down na ORIGEM** e **um** passo simulado (caminho de dois pontos), na grelha de
sempre (`R = 0,35`, força `1` salvo indicação, curva *Smooth* salvo indicação, aresta `0,046875`).
⚠️ **Validação da sessão antes de as gravar:** uma corrida de controlo do gancho com o caminho
`0 → 0,3 → 0,6` devolveu `máx = 0,343869`, o **mesmo valor a seis casas** que a fixture
`plano_gancho_radial_local_2passos_origem` gravada na sessão anterior. *A sessão reproduz; o que se
segue é comparável com o corpus antigo.*

| fixture | o que a corrida muda | movidos | máx `|u|` |
|---|---|---|---|
| `plano_expandir_radial_local_origem_1passo` | (a referência da rede pura) | `846` | `0,001914` |
| `plano_expandir_radial_global_origem_1passo` | área *Global* ⇒ `5` projecções em vez de `10` | `724` | `0,001525` |
| `plano_expandir_radial_local_origem_1passo_forca05` | força `0,5` ⇒ `τ` a `0,25` e mais nada | `662` | `0,000478` |
| `plano_gancho_radial_local_origem_1passo` | (a referência do gancho) | `1451` | `0,456101` |
| `plano_gancho_radial_global_origem_1passo` | área *Global* | `1068` | `0,343172` |
| `plano_gancho_radial_local_origem_1passo_constante` | curva *Constant* ⇒ `f_v` é um degrau | `1735` | `0,878999` |
| `plano_gancho_radial_local_origem_1passo_curto` | percurso `0,05` em vez de `0,6` | `1129` | `0,032433` |
| `plano_agarrar_radial_local_origem_1passo` | (a referência do agarrar) | `1323` | `0,134311` |
| `plano_agarrar_radial_global_origem_1passo` | área *Global* | `881` | `0,094722` |

⭐ **Cada uma existe para responder a UMA pergunta, e as três colunas de intervenção são pares
A/B com uma variável só** — a área (o número de passagens), a força do Expand (`τ`) e o
comprimento do percurso (`δ`). As leituras estão na **§5.2-quater** (a rede) e na **§4.5** (`τ`).
⚠️ **A `_constante` é a única do corpus inteiro que não usa a curva *Smooth*** — o cabeçalho dela
di-lo (`curva constant`), e um port que ignore esse campo mede outro gesto.
⚠️ **As de área *Global* têm o pen-down na origem como as outras, mas a área *Global* não tem centro
nenhum** (§2.1) — o pen-down delas importa só por ser onde o pincel está.

### §10.9 — A PARTIÇÃO EM CÉLULAS, medida contra um observável (2026-09-06, a pedido do I)

⚠️⚠️ **A partição em células não é observável do lado de fora** — ela vive numa estrutura interna que
a API de scripting da aplicação de referência não expõe, e o binário instalado não se recompila (o
checkout é esparso). ⇒ escrevê-la a partir da lei do §3.1-bis seria **derivar**, e a casa não aceita
um número sem medição ao lado.

⭐⭐⭐ **Há um observável, e ele é EXACTO.** A mesma aplicação tem um gesto que **reordena a malha**
para o consumo desta árvore, e ele usa a **mesma** lei de partição, os **mesmos** `2 500` de tecto,
a **mesma** ordenação e o **mesmo** critério de posse. A malha que ele devolve tem os vértices
dispostos exactamente na **ordem de visita** desta secção — e essa ordem lê-se de Python, comparando
as posições antes e depois (nas duas malhas **todas as posições são distintas**, logo o
emparelhamento é uma bijecção, sem tolerância nenhuma).

Corrido sobre as duas malhas das fixtures:

| malha | posições duplicadas | permutação decodificada | folhas | **derivada == observada** |
|---|---|---|---|---|
| plano `64×64` | `0` | sim | `2` (índices `1`, `2`) | ✅ **igual, elemento a elemento** |
| esfera `96×64` | `0` | sim | `4` (índices `3`..`6`) | ✅ **igual, elemento a elemento** |

E, por célula, o bloco correspondente da permutação observada bate a lista de próprios derivada,
na mesma ordem: plano `2 145` + `2 080`; esfera `1 569` + `1 520` + `1 504` + `1 457`. A soma bate
`4 225` e `6 050` — **a partição é total, e nenhum vértice aparece duas vezes**.

⇒ a tabela do §3.1-bis é **(M)**, não uma derivação: as fixtures `*.celulas.txt.gz` gravam uma
permutação que a própria aplicação de referência reproduz.

⚠️ **O que este observável NÃO prova:** que a árvore usada no traço seja construída no mesmo
instante que a do gesto de reordenação. Prova que as duas leis coincidem sobre a mesma malha — o
que basta, porque a partição depende só da malha em repouso (§3.1-bis) e a malha não muda de
topologia durante um traço.

---

### §10.10 — ⭐⭐⭐ O INSTRUMENTO QUE CORTA O PASSO EM DUAS METADES (2026-09-06, a pedido do I)

⚠️ **A pergunta que ele responde não é «qual é a lei» — é «de que METADE do passo é o meu resíduo».**
O §5.7-bis fecha a busca por um termo em falta: não há. Sobra localizar, e para isso o corpus
antigo não servia: **todos** os seus traços misturam, em cada passo, a fase do gesto (que lê a
malha deformada em quase tudo — §4.1) com a fase do solver. Estes oito cortam.

#### As quatro alavancas, e o que cada uma RETIRA

| fixture (todas `plano_…_origem…`, 12 passos, pen-down na origem) | o que muda | o que a alavanca RETIRA |
|---|---|---|
| `plano_empurrar_radial_local_origem_parado` · `plano_inflar_radial_local_origem_parado` | o cursor **pára** depois do 1.º avanço: o caminho traz o mesmo ponto repetido `10` vezes | ⭐⭐⭐ **a fase do gesto inteira, do passo 3 ao 12** — um impulso conhecido no passo 2 e depois **dez passos de solver puro**: sem força, sem curva de queda, sem normal da área, sem normal de vértice. *O que sobra é retenção + relaxação, e mais nada.* |
| `plano_empurrar_radial_local_origem_forca05` · `…_forca025` · `…_massa2` · `plano_inflar_radial_local_origem_massa2` | a **amplitude**: `¼`, `1/16` e `½` do impulso, com a cena, a rede e a ordem idênticas | a **não-linearidade**: quanto mais pequeno o impulso, menos os pares esticam e menos a ordem das projecções pesa. *Um resíduo que encolhe com a amplitude é da resposta ao esticão; um que se mantém em proporção é um ganho constante.* |
| `plano_empurrar_radial_local_origem_amort1` | `damping = 1` | ⭐ **a memória de velocidade**: sem o termo de Verlet, cada passo é *força + relaxação* e nada atravessa de um passo para o outro |
| `plano_empurrar_radial_global_origem` | área **Global** | ⭐ **metade das projecções por restrição** (`5` em vez de `10`, §5.2-bis) e a banda (`w ≡ 1`), com a força idêntica — é a **calibração** de quanto vale uma projecção |

⚠️ **Os oito trazem `prova_do_fatiamento = 0,000000`** (o bloco `k = 12` é igual à corrida inteira,
a seis casas), e todos partilham a malha e o pen-down dos `_origem` que já existiam ⇒ são
comparáveis passo a passo com eles.

#### O CONTROLO do instrumento (M)

⭐⭐ **Um caminho cujos pontos são TODOS o mesmo não move um único vértice** — `12` passos, `0`
movidos, `máx |u| = 0,00000`, no Push e no Inflate. É a prova de que, num passo em que o cursor não
se desloca, a fase do gesto está calada (§4.2) **e** de que o solver sobre uma malha em repouso é o
no-op que a §5.2-quater diz. ⇒ nas duas fixtures `_parado` **tudo o que acontece a partir do passo 3
é solver**, e isso é medido, não presumido.

#### O que o instrumento MEDE (M) — `|u|` do vértice sob o pen-down, por passo

⚠️⚠️ **As duas sondas, escritas por inteiro, porque a linha `1R` NÃO é reconstruível sem elas**
(R-pré, 2026-09-06): o **pen-down** é o vértice de repouso em `(0, 0, 0)`, e o `1R` é o de repouso
em **`(0, +0,328125, 0)`** — a `0,9375 R` do pen-down, na **perpendicular** ao traço (a convenção do
§10.7), e ⛔ **o vértice espelhado, em `(0, −0,328125, 0)`, dá outros números** (`0,00477` contra
`0,00421` no passo 3 do Push `_parado`, `13 %` acima). ⭐⭐ *Numa cena com simetria de espelho
perfeita — malha, falloff e força — a única coisa que distingue os dois lados é a **ORDEM**, e ela
é toda conduzida por ÍNDICE, que não é simétrico: a partição do plano parte exactamente na fileira
do pen-down (§3.1-bis), logo um dos lados é visitado antes do outro, e o anel de cada vértice sai
das faces por índice crescente (§3.1).* ⇒ a assimetria é uma medição **a favor** da explicação (a) do §5.7-bis,
e um port que a não reproduza tem a ordem errada mesmo quando o `máx |u|` bate.

| passo | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Push `_origem` (com força todo o traço) | `0,06543` | `0,17038` | `0,22446` | `0,23493` | `0,23822` | `0,23968` | `0,23848` | `0,23546` | `0,23108` | `0,22561` | `0,21945` |
| **Push `_parado`** (força só no passo 2) | `0,06543` | `0,12140` | `0,14674` | `0,15216` | `0,15101` | `0,14754` | `0,14292` | `0,13771` | `0,13232` | `0,12707` | `0,12215` |
| … o mesmo, a `1R` | `0,00050` | `0,00421` | `0,01568` | `0,03145` | `0,04795` | `0,06373` | `0,07779` | `0,08920` | `0,09759` | `0,10313` | `0,10635` |
| Inflate `_origem` | `0,09347` | `0,22771` | `0,26348` | `0,26706` | `0,26888` | `0,27012` | `0,27060` | `0,27017` | `0,26880` | `0,26641` | `0,26294` |
| **Inflate `_parado`** | `0,09347` | `0,16443` | `0,18318` | `0,18263` | `0,17842` | `0,17300` | `0,16696` | `0,16067` | `0,15450` | `0,14869` | `0,14331` |
| … o mesmo, a `1R` | `0,00072` | `0,00854` | `0,02982` | `0,05347` | `0,07485` | `0,09331` | `0,10820` | `0,11908` | `0,12612` | `0,12997` | `0,13144` |

⭐ **A leitura da forma:** o `_parado` **cresce depois de a força acabar** (o passo 2 põe `0,0654` e o
solver sozinho leva-o a `0,1522` no passo 5), porque a retenção de velocidade é `99 %` e só a
relaxação a trava; depois **decai** monotonicamente enquanto a onda se espalha para `1R`. *É o
retrato do solver, sem uma única leitura da malha pela fase do gesto.*

⚠️ **E há uma segunda régua, que é a do gate 38 e NÃO é o vértice do pen-down: o `máx |u|` sobre a
MALHA, por passo** (o vértice que a realiza acompanha o cursor):

| passo | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Push `_origem` | `0,06990` | `0,18733` | `0,25055` | `0,27167` | `0,27794` | `0,27316` | `0,26500` | `0,25553` | `0,24762` | `0,25738` | `0,25937` |
| Push `_amort1` (`damping = 1`) | `0,06990` | `0,12540` | `0,15222` | `0,16657` | `0,17613` | `0,18272` | `0,18750` | `0,19104` | `0,19366` | `0,19558` | `0,19698` |
| Push `_parado` | `0,06990` | `0,12837` | `0,15228` | `0,15619` | `0,15395` | `0,14958` | `0,14422` | `0,13841` | `0,13254` | `0,12714` | `0,12285` |
| Inflate `_parado` | `0,09986` | `0,17289` | `0,18881` | `0,18649` | `0,18108` | `0,17472` | `0,16791` | `0,16104` | `0,15452` | `0,14912` | `0,14430` |

⭐⭐ **Só o `_amort1` é estritamente crescente** — sem memória de velocidade o traço nunca assenta,
e é **isso** que a retenção faz: o máximo dos outros três vira no passo `6` (`_origem`), no passo
`5` (Push `_parado`) e no passo `4` (Inflate `_parado`) — ⚠️ **os três são diferentes, e a 1.ª
redacção desta linha dava `4` ao Push `_parado`, que vira a `5`** (`0,15619`; R-pré, 2026-09-06).
⚠️ *No vértice do pen-down os quatro passam por um máximo* ⇒ a régua da monotonia tem de ser a da
malha, senão o discriminador some. ⛔ **E «vira» é o ARGMAX, não «e daí para baixo»:** no `_origem`
a cauda **volta a subir** nos dois últimos passos (`0,24762 → 0,25738 → 0,25937`), porque o vértice
que realiza o máximo viaja com o cursor — um port que exija decrescimento monótono depois do
máximo reprova o próprio oráculo.

#### A amplitude (M) — a mesma cena a `1`, `¼` e `1/16` do impulso

| | força `1,0` | força `0,5` | força `0,25` |
|---|---|---|---|
| impulso do passo 2 (c0) | `0,06543` | `0,01636` | `0,00409` |
| razão ao de força `1` | `1` | **`0,2500`** | **`0,0625`** |
| `|u|` no passo 12 (c0) | `0,21945` | `0,14653` | `0,07435` |
| razão ao de força `1` | `1` | `0,6677` | `0,3388` |

⭐⭐ **O impulso escala com o QUADRADO da força** (`0,2500` e `0,0625` — §4.1), e o **fim do
traço não escala com nada**: a `¼` do impulso o resultado é `0,67` e a `1/16` é `0,34`. *A cena é
não-linear em toda a faixa que o corpus cobre* ⇒ um port que compare só o fim do traço está a ler
um regime, não uma lei. `_massa2` dá o mesmo corte pelo outro lado (metade do impulso, `0,03272`,
`0,19434` no fim).
⛔ **A BARRA das duas primeiras razões é a do FICHEIRO, não a do `f32`** (R-pré, 2026-09-06): as
posições estão gravadas a seis casas, o que põe `±9·10⁻⁷` em cada `|u|` e `±2·10⁻⁵` na razão — e o
próprio oráculo lê **`0,2499924`** e **`0,0624943`**, logo *uma barra de `f32` reprovaria a fixture
que a define*. A razão da massa é a única que sai exacta (`0,5000000`).

#### A calibração (M) — quanto vale uma projecção

| | `plano_empurrar_radial_local_origem` | `plano_empurrar_radial_global_origem` |
|---|---|---|
| projecções por restrição e por passo | `10` (a lista vem em duplicado — §5.2-bis) | `5` |
| `|u|` no passo 2 (c0) | `0,06543` | `0,06543` — ⭐ e o controlo é mais forte do que uma célula: no passo 2 as **duas malhas inteiras** são idênticas, `máx` da diferença por vértice `= 0,000000` (M, R-pré 2026-09-06); a rede ainda não fez nada |
| planalto de `|u|` (c0) | `0,23968` (passo 7) | `0,28222` (passo 9) |
| razão | — | **`0,8493`** |

⭐⭐⭐ **Dobrar as projecções custa `15,1 %` do planalto.** É a régua que faltava para ler um resíduo:
`4 %` abaixo do alvo é ***um quarto*** do que vale passar de `5` para `10` projecções — i.e. da
ordem de **uma projecção a mais** por restrição e por passo, não de uma lei em falta. ⚠️ E o passo
2 ser **idêntico** nos dois é o controlo da própria régua: ali as duas configurações só diferem em
coisas que ainda não agiram.

#### ⇒ Como o instrumento decide (a árvore, escrita para quem tem o resíduo)

1. **O port bate o `_parado` passo a passo?** Se **sim**, a retenção, a relaxação, a rede e a ordem
   estão certas, e o resíduo é da **fase do gesto** — que lê quase tudo na posição **actual**
   (§4.1) e cuja única entrada que muda com a deformação é a distância ao cursor e as normais.
   Se **não**, o resíduo é do solver e o passo em que a divergência aparece diz qual metade.
2. **O resíduo encolhe em proporção quando a amplitude cai** (`_forca05`, `_forca025`)? Se encolhe
   **mais depressa** que a amplitude, é a resposta ao esticão (ordem ou população de restrições);
   se se mantém em proporção, é um **ganho** constante (força, massa, retenção).
3. **O `_amort1` bate?** Se sim, o termo de Verlet está certo e o resíduo vive na relaxação ou na
   força; se não, está na retenção.
4. **O `_global` bate?** Se sim com `5` projecções e não com `10`, o defeito é a **duplicação da
   lista** (§5.2-bis) e não a projecção.

⛔ **Nenhum destes quatro exige um facto novo do alvo** — os quatro são A/B do port contra fixtures
que já existem. *O que faltava não era conhecimento: era um corpus que separasse as metades.*

---

### §10.11 — ⭐⭐⭐ O INSTRUMENTO QUE LÊ O VECTOR DO GESTO DIRECTAMENTE DO ORÁCULO (2026-09-07)

**A ideia, e é o que torna esta secção diferente de todas as anteriores: a resposta de um passo é
AFIM no vector do gesto, logo o vector lê-se por mínimos quadrados em vez de se adivinhar.** Escrito
por inteiro, um passo é

```
p_depois  =  relaxa(p_antes)  +  u ⊗ (a)  +  (relaxa(p_antes) − prev) · φ·(1−damping)
                                  a_v = f(v) · dt / massa        ← conhecido (§4.1, §5.4)
```

e tudo menos `u` é computável a partir do **próprio oráculo**: `p_antes` é o bloco `k−1` do ficheiro
por passo, `relaxa(·)` é a §5.2, e `prev` é `relaxa(bloco k−2)`. ⇒ `u = Σ a_v·resíduo_v / Σ a_v²`,
com o **resíduo do ajuste** a dizer se o modelo é sequer da forma certa (um `u` por passo, ou uma
direcção por vértice no caso do Inflate).

⚠️ **A RÉGUA DO RESÍDUO, escrita por inteiro (R-pré, 2026-09-07 — sem ela os gates 39, 41 e 42 não
eram edificáveis do lado limpo).** *Resíduo relativo* é `‖r − a⊗u‖ / ‖r‖` sobre os vértices que se
movem, com `r` o campo observado e `a⊗u` o que o ajuste explica. Daí saem as duas leituras que este
documento cita: `1,00` = **nenhum vector explica nada** (é o valor dos `14` passos calados do Push),
e `≈ 0` = o modelo é o certo. ⚠️⚠️ **Ele mede a direcção E o `a_v` ao mesmo tempo**, logo os
dígitos citados (`6,1·10⁻⁷`, `3,5·10⁻⁷`, `7·10⁻⁶`) só se reproduzem com a queda do §4.1 já exacta —
⛔ **a barra dos gates é o VÃO de ordens de grandeza, nunca o dígito.** ⭐ E há uma régua irmã que
**não** depende de `a_v`, útil a quem ainda está a acertar a queda: a **distância entre direcções
unitárias** por vértice (`‖û_observado − n̂_candidata‖`), ou a fracção não-colinear
`‖r − (r·û)û‖ / ‖r‖` quando a direcção é uma só. Medidas com ela, para calibrar:
`esfera_empurrar_radial_local_1passo` dá **`1,5·10⁻⁵`** de fracção não-colinear, e o 2.º traço de
`plano_inflar_radial_local_1passo_2tracos` dá **mediana `1,7·10⁻⁵`** contra as normais da malha
deixada pelo 1.º traço e **mediana `0,30`** contra as planas — o mesmo vão, noutra unidade.

⚠️⚠️ **O CONTROLO vem primeiro, e é ele que dá autoridade a tudo o resto.** O mesmo arnês, sobre
`plano_arrastar_radial_local_origem`, devolve `|u| = 1,00000` e direcção `(1,0,0)` nos **onze** passos
com resíduo `≤ 3,6·10⁻⁵`, e reproduz a malha inteira ao fim de 12 passos com **`err_max = 3,7·10⁻⁶`**
— que é a resolução de seis casas do ficheiro. ⇒ *o corte no raio, a banda, o conjunto de células, a
ordem de criação, a lista em duplicado, a retenção e a integração estão TODOS exactos* — e é por isso
que o que sobra só pode estar no vector do gesto. ⛔ Sem este controlo, qualquer conclusão desta
secção seria sobre o arnês.

**Resultado 1 — o Push.** `|u| = 0,70000 = 2R` em todos os passos em que dispara (`53` de `67`),
`(0, 0, 1)` a cinco casas, e **exactamente zero** nos outros `14` (ali o ajuste não encontra vector
nenhum: resíduo relativo `1,00`). O limiar que os separa está em §4.2-bis (8).

**Resultado 2 — o Inflate.** Amplitude `1,00000` em **todos** os passos, com a direcção por vértice a
ser a **normal de repouso**: resíduo `7·10⁻⁶ … 7,4·10⁻⁴` contra `0,32 … 0,77` para a normal actual.
⇒ o Inflate **nunca** se cala; ele não lê a normal da área, logo não tem por onde ficar sem direcção.

**A verificação ponta-a-ponta da lei corrigida** (normais da superfície que o traço encontrou; Push
calado quando o disco fica vazio), `err_max` sobre a **malha inteira** ao fim dos 12 passos:

| traço | `err_max` | passos em que o gesto escreve |
|---|---|---|
| `plano_empurrar_radial_local_origem` | `0,000003` | `2 3 4 5 10` |
| `plano_empurrar_radial_global_origem` | `0,000001` | `2 3 4 5 6 10` |
| `plano_empurrar_radial_local_origem_forca05` | `0,000004` | os onze |
| `plano_empurrar_radial_local_origem_forca025` | `0,000003` | os onze |
| `plano_empurrar_radial_local_origem_massa2` | `0,000003` | `2 … 9` |
| `plano_empurrar_radial_local_origem_amort1` | `0,000005` | os onze |
| `plano_empurrar_radial_local_origem_parado` | `0,000005` | `2` (depois o cursor pára — §4.2) |
| `plano_inflar_radial_local_origem` | `0,000003` | os onze |
| `plano_inflar_radial_local_origem_massa2` | `0,000004` | os onze |
| `plano_inflar_radial_local_origem_parado` | `0,000003` | `2` |

⇒ **os dez traços de empurrar/inflar por passo ficam à resolução do ficheiro**, contra os `0,2446` e
`0,2369` que a espec anterior deixava.

⭐ **As três fixtures novas** (`esfera_empurrar_radial_local_1passo` ·
`esfera_inflar_radial_local_1passo` · `plano_inflar_radial_local_1passo_2tracos`) e o que cada uma
decide estão no [README das fixtures](fixtures/cloth/README.md); os números delas estão no §4.2-ter.
⚠️ **As duas de esfera são de área *Local*** — o que refuta a nota do README que dizia que um traço
scriptado não consegue fixar o centro da área Local numa esfera; consegue, se o hover for semeado.
⚠️ **Mas nelas a BANDA não é observável**: só se movem `120` vértices, todos onde `w = 1` sob
qualquer dos dois centros possíveis ⇒ elas fixam a direcção e a magnitude do gesto, não a área.

⏳ **O que fica por gravar, e é a extensão natural:** um traço de esfera de **doze** passos em área
*Local* (por passo) — só ele mede a lei do §4.2-ter numa superfície **curva** ao longo de um traço,
onde a normal do início e a de agora divergem por vértice e não só por passo.
⭐ **Meio pedido foi honrado em 2026-09-07 (§10.13): existem TRÊS traços de esfera por passo** —
agarrar, gancho e expandir. ⛔ **Mas são de área *Dynamic*, não *Local***, e sobretudo: a §10.13
mede que **na esfera duas corridas da mesma configuração não dão a mesma saída**, com a diferença a
crescer de `4,3·10⁻⁴` no 1.º passo simulado a `0,027`–`0,037` ao 12.º. ⇒ *um traço de esfera por
passo mede o que quer, desde que a régua traga a banda de realização ao lado* — e é por isso que os
três a trazem.

### §10.12 — ⭐⭐⭐ OS DOIS TRAÇOS LONGOS, e a premissa REFUTADA: a profundidade é da ÁREA, não do comprimento (2026-09-07, a pedido do I)

**O pedido.** O gate de artefacto do produto corre um traço de **35 eventos** cujo deslocamento
máximo vale `4,8 R`, e o traço mais fundo do corpus valia `0,94 R` em `12` passos ⇒ *não havia lado
aprovado no regime em que o dono esculpe*. Pediu-se um traço de arrasto de `≥ 30` passos sobre a
malha de plano, com as omissões e força `1`.

**As duas fixtures novas** (as duas com `.deformado` **e** por passo, pen-down na origem,
`36` passos sobre o **mesmo caminho** de `0,6` das fixtures `_origem` de `12`, tudo o resto igual):

| fixture | área | movidos | máx `\|u\|` | em raios | prova do fatiamento |
|---|---|---|---|---|---|
| `plano_arrastar_radial_local_origem_36passos` | *Local* | `2145` | `0.267205` | `0,76 R` | `0.000000` |
| `plano_arrastar_radial_global_origem_36passos` | *Global* | `4225` | `1.893192` | **`5,41 R`** | `0.000000` |

⛔⛔⛔ **E a medição refuta a premissa que gerou o pedido: em área *Local* um traço mais longo NÃO é
mais fundo — satura.** Varrida a contagem de passos sobre o mesmo caminho, com tudo o resto igual:

| passos | `12` | `24` | `36` | `48` | `36` (caminho `1,2`) | `36` (caminho `0,3`) |
|---|---|---|---|---|---|---|
| máx `\|u\|` | `0,32965` | `0,24864` | `0,26721` | `0,26124` | `0,23219` | `0,26723` |
| em raios | `0,94 R` | `0,71 R` | `0,76 R` | `0,75 R` | `0,66 R` | `0,76 R` |

⇒ a profundidade **não é monótona** na contagem de passos e **não depende** do comprimento do
caminho: ela vive numa banda de `0,66`–`0,94 R` em toda a família.
⭐⭐ **E o rastreio diz mais que o máximo: em *Local* o vértice do pen-down ASSENTA.** Ele sobe até
`0,28635` no passo `12` e depois **desce** monotonamente até `0,14411` no passo `36` — a folha
relaxa para trás enquanto o cursor se afasta. ⛔ Em *Global* o mesmo vértice **cresce nos 36 passos**
(`0,0993 → 1,0550`), sem um único passo de descida. *É esta a diferença que o máximo da malha
esconde, e é a régua mais forte das duas.* Mecanismo, e é o desenho da área
*Local*: a área é uma bola **fixa** de raio `R(1+L) = 3,5 R` centrada no pen-down (§2.2), e a banda
leva o factor a zero na borda dela ⇒ a folha só pode esticar-se até onde a rede de restrições a
prende, e mais impulso deixa de acumular. *Um traço mais longo espalha; não afunda.*

⭐⭐ **O regime de `4,8 R` existe e é um facto da ÁREA.** No mesmo caminho e nos mesmos `36` passos:

| o que muda | máx `\|u\|` | em raios |
|---|---|---|
| área *Global* | `1,89319` | **`5,41 R`** |
| queda da força *Plane* (área *Local*) | `0,69343` | `1,98 R` |
| *Local*, agarrar | `0,15596` | `0,45 R` |
| *Local*, expandir | `0,03477` | `0,10 R` |
| *Local*, gancho | `0,02043` | `0,06 R` |

⇒ **a fixture *Global* é o lado aprovado no regime do gate de artefacto** (`5,41 R` contra os
`4,8 R` dele), e o rastreio dela mostra porquê: o vértice do pen-down passa de `0,0993` no passo `2`
a `1,0550` no passo `36`, **sem assentar** — em *Global* não há banda (`w ≡ 1`), a malha inteira é
simulada e nada prende a folha.
⚠️⚠️ **E isto é um diagnóstico para o lado limpo, não só uma fixture:** se o gate de artefacto do
produto correr um traço em área ***Local*** e chegar a `4,8 R`, o motor está a deformar **`6×`** o
que o alvo deforma naquele regime, e o defeito é anterior a qualquer régua de relevo. *A pergunta
«qual é a barra?» tinha uma pergunta antes dela: «a nossa cena é sequer a cena do alvo?».*

⭐ **As duas são exactamente reproduzíveis** — três corridas de cada, dispersão `0,000000` por
vértice —, e por isso o instrumento por passo delas é um **fatiamento** legítimo (`prova = 0.000000`
nas duas), ao contrário do da esfera (§10.13).

### §10.13 — ⛔⛔⛔ OS TRÊS TRAÇOS DE ESFERA POR PASSO — e a esfera NÃO É REPRODUZÍVEL (2026-09-07, a pedido do I)

**O pedido.** Não havia um único dump por passo de esfera; para o plano eles resolveram cinco
perguntas seguidas. Pediram-se três: agarrar, gancho e expandir, em área *Dynamic*, no mesmo
formato.

⛔⛔⛔ **Estão gravados, e a primeira coisa que a gravação devolveu foi um defeito do INSTRUMENTO:
na esfera, uma corrida-prefixo de `k` elementos NÃO é o passo `k` da corrida inteira** — porque
**duas corridas da mesma configuração já não são a mesma corrida**. Medido, quatro realizações de
cada configuração, na mesma sessão:

| configuração | dispersão entre realizações | máx `\|u\|` | razão |
|---|---|---|---|
| **plano**, arrastar *Local*, 12 passos (**CONTROLO**) | **`0,000000`** | `0,32965` | `0,0 %` |
| plano, arrastar *Local*/*Global*, 36 passos (**CONTROLO**) | **`0,000000`** | `0,26721` / `1,89319` | `0,0 %` |
| esfera, gancho *Dynamic*, **1 passo simulado** | `0,000432` | `0,02672` | `1,6 %` |
| esfera, gancho *Dynamic*, 4 passos | `0,002011` | `0,07313` | `2,7 %` |
| esfera, agarrar *Dynamic*, 12 passos | `0,027055` | `0,23653` | `11,4 %` |
| esfera, gancho *Dynamic*, 12 passos | `0,036836` | `0,16906` | `21,8 %` |
| esfera, expandir *Dynamic*, 12 passos | `0,028415` | `0,04735` | **`60,0 %`** |

⚠️ **A leitura é uma só: a divergência nasce no PRIMEIRO passo simulado (`4,3·10⁻⁴`) e amplifica
`~65×` ao longo de doze passos.** Não é um erro que se acumule linearmente — é um estado inicial
minimamente diferente numa dinâmica que o amplifica.

⛔ **Quatro explicações foram medidas e REFUTADAS** (⚠️ leia-as antes de propor uma quinta):

| hipótese | experiência | resultado |
|---|---|---|
| é concorrência (a integração corre em paralelo por célula, e células vizinhas partilham vértices) | repetir com **uma** só linha de execução | dispersão `0,035897` — **igual** |
| é a semeadura do sobrevoo do cursor | repetir com o traço de semeadura de um elemento | `0,041243` — **igual** |
| idem, com a semeadura por traço anterior | repetir com o pré-traço de dois elementos | `0,038208` — **igual** |
| é o desenho da vista não ter acontecido | repetir forçando dois desenhos antes do traço | `0,033121` — **igual** |

⭐ **E a forma da diferença exclui «um punhado de vértices de fronteira»:** ao 1.º passo simulado,
`481` dos `631` vértices movidos já diferem entre duas realizações — é **difuso**, como um estado
inicial ligeiramente diferente, não como uma disputa pontual.

⇒ ⭐⭐ **os três ficheiros por passo levam a BANDA DE REALIZAÇÃO por passo** (a maior diferença por
vértice entre quatro realizações do MESMO prefixo), numa coluna do `.rastreio`. Ela é a régua sem a
qual uma comparação por passo na esfera não quer dizer nada:

| passo | `1` | `2` | `3` | `4` | `5` | `6` | `7` | `8` | `9` | `10` | `11` | `12` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| agarrar — banda | `0` | `0,00020` | `0,00047` | `0,00065` | `0,00219` | `0,00115` | `0,00505` | `0,00562` | `0,01125` | `0,01402` | `0,01166` | `0,01457` |
| agarrar — máx `\|u\|` | `0` | `0,00866` | `0,02260` | `0,04181` | `0,06648` | `0,09696` | `0,12115` | `0,14376` | `0,16644` | `0,18927` | `0,21282` | `0,23669` |
| gancho — banda | `0` | `0,00012` | `0,00077` | `0,00222` | `0,00465` | `0,00193` | `0,00992` | `0,03561` | `0,03449` | `0,02108` | `0,03997` | `0,03157` |
| gancho — máx `\|u\|` | `0` | `0,02672` | `0,04813` | `0,07312` | `0,09445` | `0,10771` | `0,11566` | `0,12469` | `0,13436` | `0,14551` | `0,15726` | `0,16862` |
| expandir — banda | `0` | `0,00004` | `0,00018` | `0,00023` | `0,00099` | `0,00316` | `0,00760` | `0,00846` | `0,00638` | `0,00991` | `0,01456` | `0,03042` |
| expandir — máx `\|u\|` | `0` | `0,00139` | `0,00289` | `0,00540` | `0,00908` | `0,01519` | `0,02299` | `0,03501` | `0,03806` | `0,04038` | `0,04200` | `0,04772` |

⭐⭐ **E a prova de que a «falha» do fatiamento é EXACTAMENTE a lotaria, e nada mais:** os três
números que a mediriam coincidem, traço a traço —

| traço | prova do fatiamento (`k=12` vs corrida inteira) | banda da corrida inteira | desvio ao `.deformado` já no repo |
|---|---|---|---|
| `esfera_agarrar_radial_dinamica` | `0,017736` | `0,019998` | `0,018878` |
| `esfera_gancho_radial_dinamica` | `0,036474` | `0,036202` | `0,034033` |
| `esfera_expandir_radial_dinamica` | `0,021864` | `0,021773` | `0,021234` |

*Se a corrida-prefixo fosse um instrumento errado, a 1.ª coluna seria maior que a 2.ª; ela é igual —
logo o prefixo é o passo `k`, a menos da realização.* Os três ficheiros trazem as três colunas no
cabeçalho, com o nome delas.

⛔⛔ **O QUE ISTO OBRIGA A RELER — e é a metade que interessa ao lado limpo:** os `.deformado` de
esfera do corpus são **uma** realização cada, e nenhum port pode ficar abaixo de `≈ 0,02`–`0,036`
neles. Sobre os quatro traços de esfera que ainda erram: `agarrar` `0,182` (banda `0,020` ⇒ `11 %`
do erro), `gancho` `0,255` (`0,036` ⇒ `14 %`), `expandir` `0,581` (`0,022` ⇒ `4 %`). ⇒ **a lotaria
NÃO explica nenhum dos três** — os erros são `5×` a `26×` a banda, e há lei em falta — **mas ela põe
um chão**, e uma barra escrita abaixo dele é uma barra que reprova o próprio oráculo. ⚠️ E é o
mesmo chão que diz onde procurar: nos passos `2` a `5` a banda é `10⁻⁴`–`10⁻³`, três ordens abaixo
dos erros finais ⇒ **a divergência real, se nasce cedo, é visível nesses passos**, e é para isso que
estes três ficheiros servem.

⚠️ **Fronteira honesta:** a causa da lotaria **não** está estabelecida. O que está medido é que ela
existe, que é da esfera e não do plano, que nasce no 1.º passo simulado, e que **não** é nenhuma
das quatro hipóteses acima. ⛔ Um port não pode reproduzi-la nem deve tentar.

### §10.14 — As onze corridas que leram o VECTOR DE NORMAIS ao próprio programa (2026-09-07)

⭐ **O instrumento que fechou a §4.6 linha 4 não mede um efeito — mede a grandeza.** Em vez de
inferir o peso da soma por face a partir da direcção de um impulso (o que a errata de 2026-09-07
fez, e ver §4.6 para o que isso custou), pediram-se as normais **ao programa** e compararam-se com
as três candidatas calculadas sobre as **mesmas** posições.

**As onze corridas** (todas sobre malhas geradas por nós): **seis** de malha nunca esculpida —
grelha de quadriláteros, a mesma triangulada e esfera UV, cada uma limpa e com os vértices
deslocados por um gerador determinístico nosso —; **duas** que repõem numa grelha nova as posições
de duas fixtures do repo, para medir o que aquela malha podia decidir; e **três** dentro de uma
sessão de escultura, que esculpem e depois pedem um refrescamento das normais sobre a forma de
agora, para ler a lei que corre no caminho da escultura. As tabelas estão no **§4.2-quater**.

⚠️ **A corrida de refrescamento usa um facto do §1** (um traço de **um** elemento nunca simula, logo
não move nada) para forçar o refrescamento sem alterar a malha — *sem isso as normais lidas são as
de um instante anterior, e a leitura sai a `84°` de qualquer candidata*, que foi a primeira coisa
que esta medição devolveu e é, de passagem, uma quarta confirmação independente do §4.2-ter.

---

## §11 — Comportamento de borda, caso a caso (F salvo indicação)

| caso | o que o alvo faz |
|---|---|
| dois vértices coincidentes numa restrição | correcção zero (sem divisão por zero) |
| par estrutural COMPRIMIDO (`D < ℓ`) | ⚠️ **sem tecto**: a correcção troca de sinal e cresce como `ℓ/D`; medido `−18,1` num aperto (§5.2) |
| vértice **exactamente** sobre o cursor, num modo de aperto | direcção nula ⇒ **força zero** (sem `NaN`, sem direcção de reserva, sem saltar o vértice); a um epsilon dali recebe a força inteira (§4.2) |
| vértice exactamente **sobre o plano** de falloff, no aperto de ponto com *Force Falloff = Plane* | o mesmo: distância assinada zero ⇒ direcção nula ⇒ força zero |
| retalho já invertido debaixo do cursor | nada o desfaz: não há detecção de inversão, nem tecto de deslocamento, nem corte ao ultrapassar o alvo (§5.2-ter) |
| **passo em que o cursor não se moveu no ecrã** | a fase do gesto desiste, **o solver corre na mesma** (mais 5/10 projecções + integração) e o factor por passo das âncoras **não é zerado** ⇒ a âncora continua a puxar para o alvo do passo anterior (§4.2) |
| **par esticado a várias vezes o comprimento de repouso** | a projecção deixa de ser uma contracção: com a curva *Constant*, `107` vértices acabam o passo MAIS longe do que o alvo da própria âncora (`1,465 × δ`) — sem tecto, sem corte (§5.2-quater) |
| Expand com pino ligado, ou com um pincel alheio a mirar a simulação | o desvio de repouso `τ` entra **também** no pino, na âncora e no corpo mole, e ali **inteiro** (os dois extremos são o mesmo vértice) ⇒ o pino segura a `τ` da posição de repouso, não nela (§4.5) |
| vértice escondido | sem restrições, factor `0` |
| vértice totalmente mascarado | tem restrições, mas `φ = 0` ⇒ nunca se move — os vizinhos vêem-no como âncora rígida |
| célula sem vértices visíveis | construída vazia; activada/desactivada sem efeito |
| cursor parado (delta de ecrã zero) | sem forças; o passo de simulação corre na mesma |
| cursor fora da malha (Drag/Push/…) | sem localização ⇒ o passo não acontece (o traço não «apanha»); para Grab/Snake Hook a localização é do pen-down e o traço continua fora da malha |
| 1.º passo de cada passagem de simetria | nunca simula |
| 2.º passo (o 1.º simulado) | força pura: as restrições ainda não respondem (§5.2); nos modos de âncora, ao contrário, a âncora É resolvida nesse passo (o alvo é escrito antes da relaxação) |
| raio por pressão a variar | *Local* ignora (usa `R₀`); *Dynamic* usa `R` a cada passo; o factor do gesto usa `R` sempre |
| *Local* e o cursor sai da área | a área NÃO segue; o cursor pinta os limites a vermelho na localização inicial |
| área *Dynamic* + pino | o pino é recusado (a opção não se aplica) |
| área *Global* + pino | sem efeito (`w ≡ 1` ⇒ ninguém está na banda) |
| *Persistent* sem base definida | comporta-se como sem *Persistent* |
| traço anchored | a malha não é reposta por passo; a simulação continua |
| passo cujo cursor não se deslocou | a fase do gesto desiste (§4.2) e o solver corre à mesma; ⭐ **MEDIDO** (§10.10): um caminho em que **todos** os pontos são o mesmo deixa a malha intacta (`0` movidos em 12 passos), e um em que o cursor pára ao 2.º ponto continua a deformar-se durante os `10` passos seguintes |
| topologia dinâmica activa | o pincel recusa |
| multires | funciona sobre a grelha do nível activo |
| vértice dentro de um colisor | nunca é expulso (H) |
| massa `0,01` (o mínimo) | deslocamento por força `100×` o de massa `1` (`10/passo` a força máxima) — a UI impede `0` |
| damping `1` | velocidade zerada a cada passo (só forças e restrições) |
| plasticidade `1` | o vértice é sempre puxado à memória de forma inicial (o pano «recupera») |
| filtro com força negativa | inverte todas as forças e o Scale encolhe |
| filtro com *Use Face Sets* | vértices fora do face set activo: factor `0` |
| **vértice numa malha que NUNCA foi esculpida** | a normal por vértice dele é a da lei ponderada pelo **ângulo do canto**; a partir do 1.º refrescamento depois de a vizinhança se mover passa a ser a soma **sem peso** (§4.2-quater) ⇒ ⚠️ uma malha meio esculpida carrega as **duas** leis ao mesmo tempo, e não é o pincel que escolhe |
| **soma de normais de face de comprimento zero** (vértice em cume perfeito, faces opostas) | a normal cai num **eixo fixo do objecto** — sem `NaN`, sem saltar o vértice (§4.2-quater) |
| **repetir o MESMO traço sobre a MESMA malha, numa superfície curva** | ⛔ **não dá a mesma saída**: quatro corridas do mesmo traço de esfera diferem entre si até `0,036` (`11 %`–`60 %` do sinal), com a divergência a nascer no 1.º passo simulado (§10.13). No plano dá `0,000000`. ⚠️ Não é concorrência nem semeadura de cursor: as quatro hipóteses estão medidas e refutadas |
| **traço de arrasto muito longo em área *Local*** | a profundidade **satura**: `0,94 R` a 12 passos, `0,71`–`0,76 R` a 24/36/48, e o comprimento do caminho não a move (§10.12). Em *Global* o mesmo traço chega a `5,41 R` e não assenta |

---

## §12 — ⛔ As cinco coisas que a nossa doc pública [`04`](../cloth/04_espec_do_comportamento.md) inferiu ao contrário

1. **«Drag puxa o pano PARA o cursor (campo radial)»** — ⛔ não. O Drag aplica a **mesma direcção
   unitária a todos os vértices**: a direcção do movimento do cursor entre passos (§4.2). Quem é
   radial é o **Pinch Point**. *A translação uniforme pesada pelo falloff era o Drag — só que como
   FORÇA numa simulação com velocidade, e não como posição.*
2. **«Push afasta o pano do cursor»** — ⛔ não. O Push empurra **para dentro, ao longo da normal da
   área**, com magnitude `2R` (§4.2). É o único modo cuja força escala com o raio.
3. **«Damping é acoplamento/propagação»** — ⛔ é **perda de velocidade por passo**, modulada pela
   banda (§5.3). A frase da documentação descreve o EFEITO visto de fora (com retenção alta, o
   movimento propaga-se mais longe).
4. **«A fronteira topológica tem rampa por passos no pincel de tecido»** — é o **auto-mascaramento
   de fronteira** da casa, genérico (§2.4); o pincel não tem código de fronteira.
5. **«O Grab de tecido é `Deformation Target = Cloth Simulation`»** — o controlo *Deformation
   Target* é de **outros** pincéis (Pose, Boundary…); o pincel de tecido não o tem. O Grab de
   tecido é um **modo** do pincel, e o mecanismo comum é a **âncora com restrição** (§4.3). A
   frase «o pincel deforma as restrições» é verdadeira para 2 dos 8 modos; os outros 6 são
   forças ou repouso.

E uma correcção à [auditoria](../cloth/03_auditoria_2026-09-05.md) §8-ter: «escrever a posição
alvo de cada vértice é o oposto do caminho da dobra natural» — o Grab do alvo escreve exactamente
isso e é o modo que os presets mais usam (3 de 13); a diferença está em **como a restrição da
âncora é resolvida** (0,3·0,1·fade por varredura, contra as restrições estruturais a 0,3), e em o
Snake Hook **re-ancorar** no estado actual com força quadrática no falloff.

---

## §13 — Notas de custo

- Por passo: `O(#restrições activas × 5)` na relaxação + `O(#vértices das células activas)` na
  integração + colisão `O(#vértices activos × #colisores × log)`. A construção de uma célula é
  `O(Σ_v grau(v)²)` (pares de vizinhos) — numa malha de triângulos regular, `~21` restrições
  candidatas por vértice, `~10` novas após dedup.
- A área *Local* com `L = 2,5` simula um disco de `3,5R`; os autores dizem que se mantém em tempo
  real «enquanto o raio não for grande demais» (H: D6715). A área *Dynamic* limita o trabalho ao
  disco actual e é o que os presets usam.
- A relaxação é sequencial (Gauss–Seidel) — o determinismo vem da ordem de células e de
  vértices.
- ⚠️ **A área *Local* paga o DOBRO da relaxação** (a lista vem em duplicado, §5.2-bis): `10`
  projecções por restrição e por passo, e o dobro da memória de restrições. Não é uma escolha de
  desempenho do alvo — é o preço do mecanismo, e um port fiel paga-o.

---

## §14 — Gates propostos (a barra é DERIVADA — CLAUDE.md §0.0)

| # | gate | barra | de onde |
|---|---|---|---|
| 1 | **Drag é uma direcção uniforme**: num retalho plano, um passo de Drag dá deslocamentos cuja direcção, nos vértices com `f > 0,5`, é a do movimento do cursor a `< 1°` de desvio ANTES das restrições | `1°` (é um `normalize` exacto; a tolerância é o `f32`) | §4.2 · fixture `plano_arrastar_radial_local_1passo` |
| 2 | **Pinch Point é radial**: idem, direcção = do vértice para o cursor | idem | §4.2 · fixture `plano_apertar_ponto_radial_local_1passo` |
| 3 | **Inflate levanta**: componente normal do deslocamento `> 0` em todo vértice com `f > 0` | exacta | §4.2 · fixture `plano_inflar_radial_local_1passo` |
| 4 | **Push entra**: componente normal `< 0`, magnitude `∝ 2R` (dois raios ⇒ razão `2 ± f32`) | derivada do `2R` | §4.2 |
| 5 | **Expand muda o repouso**: a área do retalho cresce; e a força é `0,1×` a dos outros modos | `Σ área > área₀` · razão `10 ± 1 %` | §4.5 · fixture `plano_expandir_radial_local` |
| 6 | **A força é absoluta**: o deslocamento no 1.º passo simulado é `0,1 · α / massa` no centro, independente de `R` e da aresta — massa `2` ⇒ metade exacta | `f32` (a conta é `a·dt`) | §4.1/§5.4 · fixtures `plano_arrastar_radial_local_1passo`, `plano_arrastar_radial_local_massa2_1passo` |
| 7 | **A banda é smoothstep** em `[R(1+LF), R(1+L)]`, nas três leituras, e `w ≡ 1` em *Global* | exacta (polinómio) | §2.2 |
| 8 | **O padrão de restrições** numa grelha regular: 4 + 2 + 4 por vértice interior, sem duplicados **DENTRO de uma construção** (⚠️ o conjunto do *Local* tem a lista inteira **duas** vezes — gate 16) | contagem exacta | §3.1 · §5.2-bis |
| 9 | **5 varreduras, `0,6`, meio para cada lado**: numa restrição isolada entre dois vértices livres com `φ = 1`, o erro após `k` varreduras é `(1 − 0,6)^k` do inicial | `0,4^5 = 0,01024 ± f32` | §5.2 |
| 10 | **Damping é retenção**: com damping `d`, um vértice livre em voo perde `d` da velocidade por passo; com `d = 1` pára | `f32` | §5.3 · fixture `plano_arrastar_radial_local_amort05` |
| 11 | **O 1.º passo nunca simula**; o passo seguinte sim | exacta | §1 |
| 12 | **A força por passo das âncoras é zerada em todo o objecto a cada passo e reescrita**: no Snake Hook com a queda `f` (⇒ `0` fora do pincel), no Grab com `1` (radial) nas células afectadas — ⛔ **não** «o Grab não zera» | exacta | §4.3 |
| 18 | **O centro do Snake Hook está um passo ATRASADO**: no 1.º passo simulado o vértice mais deslocado é o do **pen-down**, não o que está sob o cursor; e ao longo do traço a distância do pico ao cursor é `≈` o passo do traço, não `≈ 0` | posição do `arg max`, em raios | §4.3 · §10.4 · fixtures `plano_gancho_radial_local_1passo`, `plano_gancho_radial_local_2passos_origem` |
| 13 | **Grab mede na malha de partida**: mover o pano não muda o conjunto agarrado | exacta | §4.3 |
| 14 | **A simetria é por passagem** e a 2.ª passagem vê a 1.ª | fixture com espelho | §6.6 |
| 15 | **Paridade com o oráculo** — **todos** os traços de §10 (⛔ **conte-os**, `ls fixtures/cloth/*.deformado.txt.gz \| wc -l`; esta célula já esteve parada em `51` com `54` no disco): a barra é a **discretização** e o **`f32`**: a nossa malha é a mesma (gerada pela mesma lei), logo a comparação é por vértice; a barra por vértice é a aresta × a diferença de ordem das restrições (Gauss–Seidel não comuta) — MEDIR primeiro a dispersão entre duas ordens nossas e usar essa dispersão como barra (⛔ não um epsilon de conforto; ⛔ não bit-parity — ADR-0162) | derivada por medição | §10 |
| 16 | **A lista do *Local* vem em duplicado**: no mesmo retalho e no mesmo traço, a contagem de restrições do conjunto *Local* é **exactamente `2×`** a que uma construção só produz (⚠️ `2×` é o caso de UMA passagem de simetria, que é o das fixtures; a lei geral é `n+1` cópias para `n` passagens — §5.2-bis), e a de *Global*/*Dynamic* é `1×`; e a régua de comportamento é a **CONTAGEM DE VÉRTICES MOVIDOS** num traço de âncora de UM passo simulado (inteiro, sem acumulação possível), que tem de bater a do oráculo | contagem exacta (inteiros dos dois lados) | §5.2-bis · §10.3 · fixtures `plano_agarrar_radial_local_1passo`, `plano_expandir_radial_local_1passo`, `plano_arrastar_radial_local_2passos` |
| 19 | **O aperto INVERTE no 1.º passo simulado, e o arrasto não**: no mesmo retalho, mesmo caminho e mesma força, contar os quadriláteros de orientação invertida depois do 1.º passo simulado — aperto de ponto `> 0`, arrasto `= 0`. E a versão de força fraca do MESMO traço tem de dar `0` nos doze passos | contagem exacta (inteiros; o oráculo dá `10` / `0` / `0`) | §4.2 · §5.2-ter · fixtures `plano_apertar_ponto_radial_local_origem`(+`_fraco`), `plano_arrastar_radial_local_origem` |
| 20 | **A barra dos apertos é a da ORDEM, e mede-se, não se escolhe**: a assimetria de espelho da NOSSA saída dividida pelo maior `|u|` não pode passar a do ORÁCULO no mesmo passo (`0,675` no passo 3 do aperto de ponto); e a paridade por vértice do gate 15 **não se aplica** a um passo com faces invertidas — ali a barra é esta. ⛔ Barra derivada do oráculo, ⛔ nunca um epsilon de conforto | a do oráculo, passo a passo | §5.2-ter · §10.6 |
| 21 | **Fora da inversão o aperto é tão comparável quanto o arrasto**: sobre a fixture de força fraca (zero faces invertidas nos 12 passos) a paridade por vértice do aperto de ponto tem de ficar no mesmo patamar da do arrasto — se ficar pior, o defeito **não** é a ordem e há lei em falta | o erro relativo do arrasto no mesmo traço | §10.6 · fixture `plano_apertar_ponto_radial_local_origem_fraco` |
| 17 | **É só o ramo *Local***: a mesma experiência que melhora os traços *Local* tem de **piorar** os *Global* e os *Dynamic* — um port que dobre a relaxação em toda a parte passa o gate 16 e reprova aqui | sinal do erro relativo, em **todos** os traços de §10 (⛔ contados, nunca citados de memória) | §5.2-bis · §10.3 |
| 22 | **O deslocamento do cursor é a PROJECÇÃO, e só o arrasto não o usa**: sobre as fixtures de esfera, o `δ` que alimenta a âncora do Agarrar, a do Snake Hook, a normal do plano de queda e o `x̂` do referencial tem componente **exactamente zero** ao longo do eixo da vista em **todos** os passos; e a direcção do arrasto no mesmo traço **tem** componente de profundidade, com o ângulo entre as duas a reproduzir, nos **11** passos, `15,83° · 12,61° · 9,42° · 6,27° · 3,13° · 0°` **e o espelho** (`3,13° · 6,27° · 9,42° · 12,61° · 15,83°`) — a sequência é simétrica porque o caminho é simétrico em relação ao topo da esfera, ⛔ e não é uma tabela a copiar: sai de `atan(Δy/Δx)` sobre a esfera unitária ⇒ ⚠️ **duas metades**, e a segunda é o controlo: um port que use a diferença dos pontos 3D em toda a parte passa a 1.ª e reprova a 2.ª, e um que projecte em toda a parte faz o inverso | `0` exacto numa metade · a tabela do §4.3 na outra | §4.3 · §4.6 · fixtures de esfera |
| 23 | ⛔⛔ **REVOGADO em 2026-09-07 (M — §10.11): a LEI que ele nomeava está refutada.** Dizia *«a normal do Push é REAVALIADA sobre a malha DEFORMADA»*, e as **normais** somadas não são as da deformada: são as da superfície que o traço encontrou (§4.2-ter). Quem as reavalia erra a frente de ataque por `16×`. ⚠️⚠️ **Mas o veredito dele NÃO se inverte, e ler «invertido» aqui reconstrói o defeito que a Q17 curou** (R-pré, 2026-09-07): o que se reavalia a cada passo é o **CONJUNTO amostrado** (posições de agora), e é por isso que `n̂_área` **muda** de passo para passo — chegando ao vector **nulo**. ⇒ um port que congele o vector do gesto no valor do pen-down e empurre em **todos** os passos continua errado, e é o gate **40** que o apanha. ⚠️ O gate antigo saía de serviço por outro motivo: a mutação que ele mandava (fixar a normal na de repouso) é, num plano, **byte-idêntica** à lei correcta em todo passo em que o disco não está vazio ⇒ ele passava ou reprovava conforme a mutação levasse ou não o caso do disco vazio junto — *ambíguo, não invertido*. ⚠️ **A metade que estava certa era o CONTROLO** (no 1.º passo simulado as duas leis coincidem por construção, porque a malha ainda está em repouso) — e é precisamente por isso que ele não distinguia nada. Os gates que ocupam este lugar são o **39** (o instante das normais) e o **40** (o silêncio) | — | §4.2-bis · §4.2-ter · §10.11 |
| 39 | ⭐⭐⭐ **AS NORMAIS SÃO AS DA SUPERFÍCIE QUE O TRAÇO ENCONTROU.** ⚠️ **Três metades, e cada uma mata uma leitura diferente:** (a) **Inflate** — em `plano_inflar_radial_local_origem`, a direcção por vértice do impulso do passo `12` bate as normais de **repouso** com resíduo relativo `< 10⁻³` e as normais **actuais** com resíduo `> 0,5`; (b) **Push** — em `plano_empurrar_radial_local_origem`, o vector do gesto é `(0,0,−0,7)` a cinco casas em **todos** os passos em que dispara, sobre uma folha já afundada `0,2`; (c) **e não são as do objecto, são as do TRAÇO** — em `plano_inflar_radial_local_1passo_2tracos` a direcção do 2.º traço bate as normais da malha **no pen-down dele** (resíduo `3,5·10⁻⁷`, amplitude `1,00000`) e **não** as planas (resíduo `0,32`, amplitude `0,946`). ⛔ Um port que fotografe as normais **uma vez, no início da sessão** passa (a) e (b) e reprova (c). ⚠️ **Duas notas de edificabilidade (R-pré, 2026-09-07):** o *resíduo relativo* é o do §10.11, e a barra é o **vão de ordens de grandeza**, ⛔ nunca o dígito citado; e as «normais actuais»/«do pen-down deste traço» exigem escolher o PESO da soma por face, que o §4.6 linha 4 declara **aberto** — ⭐ o veredito não depende dele (o vão é `10⁻⁵`–`10⁻⁴` contra `10⁻¹`, medido com os **três** pesos) | resíduos do ajuste, nas três | §4.2-ter · §10.11 |
| 40 | ⭐⭐⭐ **O PUSH CALA-SE QUANDO A COVA PASSA O DISCO DE AMOSTRAGEM, E VOLTA A DISPARAR.** Régua, por inteiro (R-pré, 2026-09-07 — reconstruída do zero sobre as sete fixtures por passo): no passo `k`, `min_v \|p_v − c_k\|` com `p_v` = as posições do bloco `k−1` do ficheiro por passo (a malha **com que o passo `k` começa**), `c_k` = o `k`-ésimo ponto do `caminho`, o mínimo sobre **todos** os vértices da malha e a distância **3D** (a forma de queda dos presets), contra `R · «Normal Radius»` (`0,175` com as omissões). ⚠️ **Duas metades:** (a) o **padrão** de disparo tem de bater o do oráculo passo a passo — `2 3 4 5 10` em `plano_empurrar_radial_local_origem`, `2 3 4 5 6 10` no mesmo traço em *Global*, `2..9` com massa `2`, e **os onze** às forças `0,5` e `0,25` (onde a folha nunca chega a `0,175`); (b) o **limiar** não é escolhido: sobre os `67` passos com cursor em movimento dos sete traços de empurrar, o maior `min_v` com gesto é `0,17313` e o menor sem gesto é `0,17586`, e `R·0,5 = 0,17500` cai entre os dois — nem um passo do lado errado. ⛔ Um port que aplique força em todos os passos passa (b) por vacuidade e reprova (a); um que se cale para sempre à primeira falha reprova as duas (o oráculo volta a disparar no passo `10`) | padrão exacto (inteiros) · o vão medido | §4.2-bis (8) · §10.11 |
| 41 | ⭐⭐ **A PARIDADE DOS DEZ TRAÇOS DE EMPURRAR/INFLAR POR PASSO, À RESOLUÇÃO DO FICHEIRO.** Com os gates 39 e 40 honrados, os dez traços do §10.11 têm de reproduzir-se com `err_max ≤ 5·10⁻⁶` sobre a **malha inteira** e nos **doze** passos — ⛔ não «na barra do gate 15»: aqui a barra é a discretização de seis casas do próprio ficheiro, porque a lei é exacta. ⚠️ **Isto substitui a leitura antiga de que Push e Inflate ficavam `5`–`9 %` abaixo** (§5.7-bis): aquele défice era o vector do gesto, não a resposta ao esticão | `5·10⁻⁶` (a resolução do ficheiro) | §10.11 |
| 42 | ⭐ **A DIRECÇÃO DO PUSH É A NORMAL DA ÁREA, NÃO A DA VISTA** — e só uma superfície curva o diz. Em `esfera_empurrar_radial_local_1passo` (um passo simulado a partir do repouso, onde a relaxação é um no-op por construção ⇒ o deslocamento **é** `u·f(v)·dt`), o vector recuperado tem de estar a `< 0,1°` da normal da superfície no cursor e a `≈ 17,4°` da normal da vista, com `\|u\| = 2R ± f32`. ⚠️ **Controlo obrigatório ao lado**: `esfera_inflar_radial_local_1passo`, direcção por vértice, amplitude `1,000000` — sem ele, um port que ponha a direcção certa pela razão errada (por exemplo o vector cursor→centro do objecto, que nesta cena coincide) passa | ângulo `< 0,1°` contra `≈ 17,4°` | §4.2-bis · §4.2-ter · §10.11 |
| 25 | ⛔⛔ **UM PASSO DE ACELERAÇÃO NÃO MOVE NADA FORA DO DISCO — e é o CONTROLO de que a relaxação não corre ali**. Régua, por inteiro: **movido** = `\|u\| > 1e-5` sobre as posições a seis casas; **disco** = posição de **repouso** a menos de `R` da **ponta do caminho**, que nestas fixtures (pen-down em `x = −0,3`) tem `173` vértices. Nos **sete** traços de um passo dos cinco modos que escrevem aceleração a contagem FORA do disco tem de ser **`0`**, e a de dentro `171` (arrastar · arrastar com massa `2` · empurrar · inflar · apertar ponto), `168` (arrastar com força `0,5`) e `156` (apertar linha); nos três de âncora/repouso a de fora tem de ser `675` (expandir) · `1151` (agarrar) · `1279` (gancho) e a de dentro `173` nos três. ⚠️ **Duas metades, e a 1.ª é o controlo:** um port cuja relaxação corra DEPOIS da integração passa a 2.ª e reprova a 1.ª | contagens exactas (inteiros dos dois lados) | §5.2 · §5.2-quater · as **dez** fixtures `plano_*_1passo` de pen-down em `x = −0,3` |
| 26 | **A ÂNCORA vive na mesma lista e recebe `Δ/2`**: numa fixtura de uma restrição de âncora isolada com `φ = 1`, `s = 1` e `σ = 1`, a distância ao alvo após `k` varreduras é `(1 − 0,3)^k` da inicial — ⛔ **não** `(1 − 0,6)^k`, que é a lei das ESTRUTURAIS (gate 9). O **pino** segue a mesma lei da âncora. ⭐ E o **corpo mole** fecha a separação à MESMA taxa da âncora — `0,3ρ` do lado do vértice mais `0,3(1−ρ)` do lado da memória dá `0,3` para qualquer `ρ` —, o que o distingue não é a taxa, é **as duas pontas se moverem** | `0,7^5 = 0,16807 ± f32` nos três · e, no corpo mole, `\|Δvértice\| + \|Δmemória\| = Δ/2` exacto | §3.2 · §5.2 |
| 27 | ⭐ **O alvo de cada espécie é lido no INSTANTE da projecção, e a memória de forma ANDA**: com plasticidade `0 < ρ < 1` e uma restrição de corpo mole isolada, a memória tem de ter-se deslocado no fim do passo — e os dois têm de convergir para o ponto **`(1 − ρ)·A₀ + ρ·B₀`** (`A` = vértice, `B` = memória), que é a combinação que a projecção **conserva**: cada varredura move `A` de `+0,3ρ(B−A)` e `B` de `−0,3(1−ρ)(B−A)`, e `(1−ρ)A + ρB` fica invariante. ⛔ Não é o ponto médio: com `ρ = 0` o encontro é em `A₀` (a memória vai ter com o vértice) e com `ρ = 1` é em `B₀`. ⚠️ **Controlo:** com a âncora de deformação, o alvo tem de ficar **exactamente** onde o gesto o pôs, ao bit, no fim das 10 projecções | ponto de encontro `= (1−ρ)A₀ + ρB₀ ± f32` · posição da âncora idêntica ao bit | §3.2 · §5.2 |
| 28 | ⭐⭐ **`τ` entra ANTES da 1.ª varredura do MESMO passo, e é linear**: o traço de UM passo do Expand tem de mover `846` vértices (régua do gate 25; ⛔ um port que só aplique `τ` no passo seguinte move **zero**), e o mesmo traço a força `0,5` tem de dar `0,25 ×` o deslocamento — as duas leituras da razão são a **dos máximos** (`0,24999`) e a **mediana das razões por vértice** sobre os `662` vértices que os dois traços movem (`0,250000`); ⛔ dividir os dois `max_deslocamento` do cabeçalho, já arredondados a seis casas, dá `0,2497` e não é a régua. ⚠️ **Duas metades**, e a 2.ª mata a hipótese «move alguma coisa por outro motivo» | contagem exacta · razão `0,25 ± f32` | §4.5 · §10.8 · fixtures `plano_expandir_radial_local_origem_1passo`(+`_forca05`) |
| 29 | ⭐⭐⭐ **O DEGRAU do gancho é da REDE, e o sítio dele desloca-se com o número de passagens.** Régua do perfil: §5.2-quater (eixo do traço, `k · aresta`, `k = 0..7`). ⚠️ **«Há degrau» tem de ser um predicado DERIVADO, não um tecto escolhido** — o critério é *a maior razão entre células consecutivas dividida pela MEDIANA das restantes* (`≥ 2` ⇒ há degrau). Ele separa o corpus por um vazio de `3,6×`: gancho `3,95` (*Local*) e `4,63` (*Global*) contra agarrar `1,05`/`1,10` e `_curto` `1,05`. ⛔ **Um tecto absoluto de `1,4` na razão máxima REPROVA o próprio oráculo** (o agarrar *Global* faz `1,47` sem degrau nenhum, só por decair mais depressa). **Metades:** (a) no gancho a maior razão tem de cair entre `0,536 R` e `0,670 R` na *Local* e entre `0,402 R` e `0,536 R` na *Global* (o oráculo dá `4,52` e `5,11`); (b) o **agarrar** não pode ter degrau (discriminador `≤ 1,2`), com a cauda a assentar em `1,32` *Local* / `1,47` *Global*; (c) o `_curto` (`δ = 0,05`) também não pode ter degrau — se tiver, o defeito não é a rede, é a lei da âncora | posição do máximo da razão, em células · discriminador `≥ 2` no gancho e `≤ 1,2` nos outros dois | §5.2-quater · §10.8 · fixtures `plano_gancho_radial_{local,global}_origem_1passo`, `…_curto`, `plano_agarrar_radial_{local,global}_origem_1passo` |
| 30 | **A banda começa a `R(1+L·F)` e acaba a `R(1+L)`** — com as omissões, `2,875 R` e `3,5 R`, ⛔ **não** `1,875 R` e `2,5 R`. Régua: `movido` = `\|u\| > 1e-5`; num traço *Local* de 12 passos do plano com `limite = 2,5` o vértice movido mais distante do pen-down tem de ficar entre `3,49 R` e `3,50 R` (`25` fixtures dão `1,2221`–`1,2239` contra `3,5 R = 1,2250`; ⛔ isenta o controlo de força fraca, cuja franja cai na resolução do ficheiro). ⚠️ **Segunda metade, e é ela que refuta a leitura antiga:** com `limite = 5,0` o mesmo traço tem de chegar a `≈ 0,99 · R(1+L) = 2,08`, e `R·L` daria `1,75` — *um `L` só não distingue as duas fórmulas; dois distinguem* | `1,2221`–`1,2239` e `2,0745` (M) | §2.2 · fixtures `plano_{expandir,agarrar,gancho}_radial_local` e `plano_agarrar_radial_local_preset` |
| 31 | **Um passo sem movimento de cursor ainda SIMULA — e agora tem ORÁCULO** (a cláusula «gate de espec» de 2026-09-06 **caducou**: as fixtures `_parado` trazem `10` pontos de caminho repetidos). ⚠️ **Duas metades, e a 1.ª é o controlo:** (a) um caminho cujos pontos são **todos** o mesmo tem de deixar a malha intacta — `0` vértices movidos em `12` passos, exacto, nos dois modos (a fase do gesto desiste E o solver sobre uma malha em repouso é um no-op); (b) um caminho que avança **uma** vez e depois pára tem de continuar a deformar-se durante os `10` passos seguintes, e a barra é a paridade por vértice do gate 15 contra `plano_{empurrar,inflar}_radial_local_origem_parado` — ⛔ não «diferem». Um port que trate o passo parado como no-op reprova (b); um que aplique força num passo parado reprova (a) | (a) `0` movidos, inteiro · (b) a barra do gate 15 | §4.2 · §10.10 · §11 |
| 32 | ⭐ **A ORDEM DE CRIAÇÃO lê-se de fixture, e é um inteiro dos dois lados**: (a) o anel de **cada** vértice das duas malhas tem de bater, elemento a elemento, o que sai de `*.faces.txt.gz` pela lei do §3.1 (faces incidentes por índice crescente; de cada face, o canto **anterior** e depois o **seguinte** no sentido de percurso; deduplicação que guarda a PRIMEIRA ocorrência) — `4 225` e `6 050` anéis, igualdade exacta; (b) a sequência de vértices que a construção visita tem de bater, elemento a elemento, a concatenação de `*.celulas.txt.gz` por ordem crescente de célula. ⚠️ **Duas metades de saúde da própria fixture, que um parse errado reprova:** a partição é **total** (`Σ próprios` = `4 225` / `6 050`, sem repetidos) e a sequência tem **exactamente `1`** descenso no plano e **`3`** na esfera | igualdade de inteiros | §3.1 · §3.1-bis · fixtures `plano.faces` · `esfera.faces` · `plano.celulas` · `esfera.celulas` |
| 33 | ⭐⭐ **O anel É a ordem das FACES — e o corpus do plano só o prova numa fixture**: com o anel trocado para a ordem crescente de índice de vértice, (a) **todos** os traços de plano com `limite = 2,5` têm de ficar **byte-idênticos** (os `128` vértices onde as duas leis divergem estão no bordo, e o mais próximo está a `1,5` do pen-down contra uma banda que acaba a `1,2250` ⇒ `φ = 0`), e (b) `plano_agarrar_radial_local_preset` (`limite = 5,0`, banda até `2,1000`, alcança `126` dos `128`) e os **oito** traços de esfera (`98,5 %` dos anéis divergem) têm de **mudar**. ⚠️ **A 1.ª metade é o CONTROLO**: um port que passe só a (b) pode estar a mudar outra coisa qualquer | mutação A/B: byte-idêntico numa metade, diferente na outra | §3.1 · §3.1-bis |
| 34 | ⭐⭐ **A ordem de visita é a das CÉLULAS, não a crescente global** — nem no plano: com a visita trocada para `0..N−1`, a saída tem de mudar. ⚠️ **A barra é «diferem», não «casam com o oráculo»**: é um A/B sobre o nosso motor, porque nenhuma fixture isola a ordem. ⛔ **Se não mudar em traço nenhum, o veredito NÃO é «a ordem não importa»** — é que este corpus não a observa, e isso vai ao ledger como recusa medida, com a contagem de traços por trás. ⚠️ E o par com o gate 20: num passo com faces invertidas, a ordem **decide** o resultado por vértice (§5.2-ter), logo é aí que a diferença tem de aparecer primeiro | mutação A/B, traço a traço | §3.1 · §3.1-bis · §5.2-ter |
| 35 | ⭐⭐⭐ **O SOLVER SOZINHO, sem uma única leitura da malha pela fase do gesto**: sobre `plano_{empurrar,inflar}_radial_local_origem_parado`, a paridade por vértice do gate 15 tem de valer nos **doze** passos. ⚠️ **É o gate mais forte do corpus para a relaxação**, porque do passo 3 ao 12 não há força, nem curva de queda, nem normal: só retenção e projecções. ⚠️ **Segunda metade, e é o discriminador**: um port que passe este e falhe `plano_empurrar_radial_local_origem` tem o defeito na **fase do gesto**; um que falhe este tem-no no **solver**, e o primeiro passo a divergir diz onde (⛔ um port não pode falhar os dois e declarar «é a relaxação» sem correr este) | a barra do gate 15, passo a passo | §5.7-bis · §10.10 |
| 36 | ⭐⭐ **A resposta NÃO é proporcional ao impulso, e a não-linearidade tem número**: o impulso do passo 2 escala com o **quadrado** da força (`0,2500` e `0,0625` para força `0,5` e `0,25`), e o `\|u\|` do passo 12 escala `0,6677` e `0,3388`. ⛔ **A barra é a do FICHEIRO (`±2·10⁻⁵` na razão), NÃO a do `f32`** — o oráculo lê `0,2499924` e `0,0624943` e uma barra de `f32` reprovaria a fixture que a define; a razão da massa (`0,5000000`) é a única exacta. ⚠️ **As duas metades são a régua:** a 1.ª é aritmética da fase do gesto (§4.1) e a 2.ª é o regime do solver — ⛔ **um port que compare só o fim do traço não pode concluir nada sobre uma lei**, e um cujo erro relativo **não** varie ao longo desta faixa tem um ganho constante, não um defeito de esticão | `0,2500`/`0,0625 ± f32` numa metade · as razões do §10.10 na outra | §10.10 · fixtures `plano_empurrar_radial_local_origem_forca05`, `…_forca025`, `…_massa2`, `plano_inflar_radial_local_origem_massa2` |
| 37 | ⭐ **Uma projecção vale `15,1 %` do planalto, e é a régua de qualquer resíduo**: no mesmo traço, `Global` (`5` projecções) e `Local` (`10`) têm de dar a **MESMA MALHA** no passo 2 (⭐ não só a mesma célula: `máx` da diferença por vértice `= 0,000000` sobre os `4 225` — ali a rede ainda não agiu) e planaltos na razão `0,8493`. ⇒ ⚠️ **a conversão traz o VÃO da régua dentro**: os `15,1 %` medem **`5`** projecções a mais por restrição e por passo (`5 → 10`), logo um resíduo de `x %` no planalto vale `5 · x / 15,1` projecções — a `4 %` isso é `1,3`, que é a leitura do §10.10 (⛔ a 1.ª redacção dizia `x / 15,1` e contradizia, cinco linhas acima, o «uma projecção a mais» da própria secção). ⚠️ **A 1.ª metade é o controlo da régua**: se o passo 2 já diferir, o defeito não é a contagem de projecções | malha idêntica no passo 2 · razão `0,8493` | §5.2-bis · §10.10 · fixture `plano_empurrar_radial_global_origem` |
| 38 | **Sem memória de velocidade o traço deixa de assentar.** ⚠️ **A régua é o `máx \|u\|` sobre a MALHA, por passo** (⛔ não o vértice do pen-down: ali as duas configurações passam por um máximo, e o discriminador desaparece). Com `damping = 1` a sequência tem de ser **estritamente crescente** nos 11 passos simulados (`0,06990 → 0,19698`); com `damping = 0,01`, o mesmo traço tem de ter o **máximo dos onze no passo `6`** (`0,27794`). ⛔ **A régua da 2.ª metade é o ARGMAX, nunca «decresce depois do máximo»**: a cauda do `_origem` **volta a subir** nos dois últimos passos (`0,24762 → 0,25738 → 0,25937`), porque o vértice que realiza o máximo viaja com o cursor — exigir decrescimento reprova o oráculo. ⚠️ **Duas metades**: a monotonia é a forma, e a paridade por vértice do gate 15 é o valor — um port cuja retenção esteja errada pode acertar a forma e falhar o valor | monotonia exacta (inteiros de comparação) · a barra do gate 15 | §5.3 · §5.4 · §10.10 · fixtures `plano_empurrar_radial_local_origem_amort1` e `plano_empurrar_radial_local_origem` |
| 43 | ⭐⭐⭐ **A BANDA ENTRA UMA VEZ NA VELOCIDADE E NENHUMA NA ACELERAÇÃO — e a régua tem de olhar para a MALHA INTEIRA.** Sobre `plano_arrastar_radial_local_origem` (por passo), a paridade por vértice tem de dar `err_max < 5·10⁻⁶` nos doze passos; com a banda aplicada **duas** vezes (o factor da §5.2 usado também na integração) o mesmo traço sobe a `3,9·10⁻³`. ⚠️ **A metade que faz o gate existir é o CONTROLO, e ele é EXACTO**: o **mesmo** traço em área *Global* (`plano_arrastar_radial_global_origem`) tem de sair **byte-idêntico** com as duas leis — em *Global* a banda é `1` em toda a malha, logo `banda² = banda` **por construção**, e o erro do port não se move (o I mediu `≈ 2·10⁻⁵` nas duas). ⇒ é a **razão** entre os dois traços que denuncia o defeito, e não o valor de nenhum deles. ⛔ Um port que meça só o máximo, ou só o vértice do pen-down, ou só traços *Global*, passa com `banda²` lá dentro: o erro vive num anel de `2,875 R` a `3,5 R` onde o deslocamento já é de ordem `10⁻³` | `5·10⁻⁶` (resolução do ficheiro) contra `3,9·10⁻³`; e o *Global* invariante | §5.2 · §5.4 · §5.4-bis |
| 44 | ⭐⭐ **A NORMAL POR VÉRTICE É A SOMA SEM PESO DE NORMAIS DE FACE UNITÁRIAS** — e a régua não é um traço, é a grandeza. Sobre uma malha de triângulos **irregulares** (áreas e ângulos muito diferentes à volta do mesmo vértice), a normal que o motor usa tem de bater a soma sem peso a `< 0,03°` e afastar-se da ponderada por **área** por `> 1°` em pelo menos um vértice. ⚠️ **Duas metades**, e a 2.ª é a que impede passar por sorte: numa grelha regular ou numa esfera UV as três candidatas concordam a `< 0,04°` ⇒ *uma malha regular não testa este gate*. ⛔ E o gate **não** pode ser escrito sobre uma fixture do corpus: no plano as três dão `(0,0,1)` e na fixture de dois traços concordam a `0,31°` de máximo | `0,03°` de um lado, `1°` do outro | §4.2-quater · §4.6 linha 4 |
| 45 | ⭐ **UM TRAÇO MAIS LONGO NÃO É MAIS FUNDO EM ÁREA *LOCAL*** — e é o gate que impede calibrar uma régua de relevo num regime que o alvo não produz. **Três metades, e a do meio é o discriminador forte:** (a) sobre `plano_arrastar_radial_local_origem_36passos` o máximo da malha tem de ficar em `0,76 R` (`0.267205`), **abaixo** do mesmo caminho em `12` passos (`0,94 R`); (b) ⭐ no MESMO traço, o vértice do pen-down tem de **subir até ao passo `12` (`0,28635`) e depois DESCER até `0,14411` no `36`** — ele *assenta*, e o traço continua a acontecer à volta dele; (c) sobre `plano_arrastar_radial_global_origem_36passos` o mesmo vértice tem de crescer em **todos** os 36 passos, de `0,0993` a `1,0550`, chegando a `5,41 R` de máximo de malha (`1.893192`). ⚠️ Um motor que afunde monotonamente com o número de passos passa (c) e reprova (a) **e** (b) ⇒ ⛔ **um gate de artefacto do produto que corra um traço *Local* de ~35 eventos e chegue a `4,8 R` está a medir uma cena que o alvo não produz**, e a barra dele não é calibrável ali | os dois máximos, na barra do gate 15 · o argmax `12` e a monotonia (inteiros) | §2.2 · §10.12 |
| 46 | ⛔⛔ **NA ESFERA, A BARRA TEM UM CHÃO QUE NÃO É NOSSO — e ele mede-se, não se escolhe.** Nenhum gate de paridade sobre um traço de esfera pode ter barra abaixo da **banda de realização** daquele traço (`0,020` agarrar · `0,036` gancho · `0,022` expandir, medidas com quatro corridas da mesma configuração), sob pena de reprovar o próprio oráculo. ⚠️ **E a metade que evita a leitura preguiçosa:** os erros abertos hoje são `5×` a `26×` a banda (`0,182` · `0,255` · `0,581`) ⇒ ⛔ **a lotaria NÃO os explica** e há lei em falta; quem invocar esta linha para relaxar uma barra tem de mostrar o quociente. ⭐ **O sítio de procurar é cedo**: nos passos `2`–`5` a banda é `10⁻⁴`–`10⁻³`, três ordens abaixo do erro final, logo uma divergência que nasça aí é visível nos ficheiros por passo | a banda medida, por traço | §10.13 |
| 47 | ⭐ **O FATIAMENTO É UM INSTRUMENTO, E ELE TEM UM CONTROLO** — que é o que separa «o prefixo não é o passo `k`» de «a corrida não se repete». Em todo traço novo por passo, `prova_do_fatiamento` tem de ser `0,000000` **ou** ficar `≤` à **banda de realização da corrida inteira** do mesmo traço. ⛔ Uma prova acima da banda diz que o prefixo não reproduz o passo e o ficheiro **não é oráculo**; igual à banda diz que reproduz, a menos da realização (é o caso dos três de esfera: `0,0177` vs `0,0200` · `0,0365` vs `0,0362` · `0,0219` vs `0,0218`). ⚠️ **Sem a 2.ª coluna a 1.ª não tem leitura**, e foi assim que quatro ficheiros da 1.ª geração ficaram no directório a parecer oráculo | `0,000000`, ou `≤` a banda | §10.12 · §10.13 |
| 24 | **A razão `2R` do Push, e a igualdade Push/Inflate no 1.º passo simulado**: no passo 2 dos dois traços do §10.7 o vértice do pen-down move `0,06543` e `0,09347`, razão `0,7000 = 2·R`; e a divergência entre os dois só pode começar no passo **3** — se começar no 2, o port está a ler duas normais diferentes numa folha plana em repouso, onde elas são a mesma | razão `2R ± f32` · igualdade de direcção no passo 2 | §4.2-bis · §10.1 · §10.7 |

---
