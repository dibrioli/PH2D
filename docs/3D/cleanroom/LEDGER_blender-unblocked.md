# LEDGER de proveniência — clean-room dos quatro pincéis desbloqueados (alvo `blender-unblocked`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-unblocked.md` (append cego).

**Aberto em 2026-09-13, ANTES da primeira leitura do fonte do alvo** (inclusive antes da licença),
pelo **subagente-E** despachado pela janela `9f820704-0d7e-4d96-847e-9cd720cbf178` (a janela I-3 da
`line/sculpt3d`), com o alvo `blender-unblocked`. Obra: os pincéis públicos SIMPLIFY ·
DISPLACEMENT_ERASER · DISPLACEMENT_SMEAR · SCENE_PROJECT (+ a opção pública
`project_ray_direction_type`), que o plano `docs/3D/21_plano_modos_e_ferramentas.md` §5.2 deixou fora
por bloqueio de substrato.

⚠️ **Paralelismo declarado:** outros três subagentes-E trabalham na mesma worktree ao mesmo tempo
(alvos `blender-pull`, `blender-pose`, `blender-boundary`). Este E toca SÓ: este ledger,
`VASSOURA_blender-unblocked.txt`, `SPEC_unblocked_brushes.md`, `fixtures/unblocked/` e
`~/Referencias/blender-unblocked/`.

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — os pincéis de escultura de nome público `SIMPLIFY`, `DISPLACEMENT_ERASER` («Multires Displacement Eraser»), `DISPLACEMENT_SMEAR` («Multires Displacement Smear») e `SCENE_PROJECT` («Scene Project», com a opção pública `project_ray_direction_type`) + o que cada um invoca que decide comportamento |
| Versão / commit do fonte | tag **v5.2.0**, commit `fbe62287` («Release: Bump to 5.2.0 release»), checkout **esparso** (o mesmo que a obra `blender-cloth` leu; sem história local) |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D. Zona deste E: `~/Referencias/blender-unblocked/` (`draft/`, `notes/`, `oracle/`, `out/`) |
| Repo de origem | `https://projects.blender.org/blender/blender` (⛔ na denylist do I) |
| Licença | **GPL-2.0-or-later** — `COPYING` (lido 2026-09-13) remete a `doc/license/GPL-license.txt` e diz que o Blender não está disponível sob outra licença |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS** (hash `9e2066aef7ef`, build 2026-09-01) — ⚠️ patch-release acima do fonte lido (5.2.0); o oráculo é o binário |

### A concessão relevante (GPLv2 §0 e §2)

A mesma transcrita no `LEDGER_blender-cloth.md` §Alvo (mesmo ficheiro de licença do mesmo checkout): o
acto de CORRER não é restrito, a saída só é coberta se o conteúdo for obra baseada no programa, e a
modificação privada é licenciada. ⇒ ler, correr, instrumentar em privado é licenciado; posições de
vértices de malhas NOSSAS são dado. Nenhum acto deste ledger distribui. Não é AGPL.

## §2 — Triagem (registada ANTES da leitura do fonte, 2026-09-13)

| degrau | veredito por pincel | por quê |
|---|---|---|
| T0 | ✅ **SIMPLIFY — o MECANISMO** (colapso de arestas curtas numa esfera) já está no repo como porte MIT do SculptGL (`crates/ph2d-mesh/src/collapse.rs`, cabeçalho de atribuição); ⛔ para os outros três não há porta permissiva conhecida | o SculptGL (MIT) expõe a decimação dentro da topologia dinâmica; o que ele NÃO tem é o pincel «só colapsa» nem as regras de detalhe do alvo — isso é facto de comportamento, observável por oráculo + manual |
| T1 | ⛔ nenhum irmão permissivo para DISPLACEMENT_ERASER / DISPLACEMENT_SMEAR / SCENE_PROJECT: Nomad Sculpt e ZBrush (que tem um pincel de projecção entre sub-ferramentas) são proprietários; SculptGL não tem apagador/esfregão de deslocamento nem projecção na cena; Mudbox proprietário | caçado por busca web e pelo repo (2026-09-13) |
| **T2** | ✅ degrau desta obra para os três sem porta, e para a SEMÂNTICA do Simplify (o que conta como «curta», de onde vem o detalhe) | copyleft com fonte; o dono pediu a travessia |

## Patente (§8.1) — checkpoint incondicional, 2026-09-13, antes da leitura do fonte

- **Termos:** `sculpting brush project vertices onto other scene objects raycast view direction` ·
  `multiresolution sculpting erase displacement smear displacement brush` ·
  `dynamic topology sculpting brush edge collapse decimation simplify local brush region` ·
  `shrinkwrap brush conform mesh vertices onto target surface ray projection interactive sculpting tool` ·
  `Pixologic ZBrush patent project brush subtool` ·
  `brush multiresolution mesh displacement reset subdivision surface sculpting Autodesk Mudbox`.
- **Resultado:** ⭐ **nenhuma patente viva alcança os métodos.** Achados e veredito:

| documento | estado | lê sobre nós? |
|---|---|---|
| US 10 586 401 B2 (Pixar, Kelvinlets) | viva até 2038 | ⛔ não — nenhum dos quatro pincéis é uma solução analítica de elasticidade (cerca já nomeada no ledger do tecido) |
| US 11 257 290 — decimar uma malha por auto-parametrização sucessiva | viva | ⛔ não — decimação OFFLINE com mapa bijectivo mantido entre níveis; o Simplify é colapso local de arestas sob um pincel |
| US 8 830 235 — relaxação não-uniforme para processamento multirresolução | não avaliada a fundo | ⛔ não — processamento de malha, sem pincel de apagar/esfregar deslocamento |
| US 6 751 599 — sistema de inferência difusa para simplificar malha | expirada (anterior a 2006) | não lê; e expirada |

  Arte anterior pública que tornaria inválida qualquer reivindicação posterior: a decimação na
  escultura dinâmica (Sculptris ~2009; Blender 2.66, 2013), o pincel de projecção entre sub-ferramentas do
  ZBrush (anos 2010), o apagador e o esfregão de deslocamento do Blender (2.91/2.92, 2020, publicados GPL).
  ⇒ Veredito: **prosseguir**. Sem «PATENTE VIVA» a reportar.

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E** despachado pela janela `9f820704-0d7e-4d96-847e-9cd720cbf178` (I-3 da `line/sculpt3d`), alvo `blender-unblocked` |
| transcript do subagente (⛔ **zona contaminada**) | ficheiros de subagente sob `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178*` |
| aberto em | 2026-09-13 |
| lê | o fonte dos quatro pincéis e o que eles invocam, defaults/faixas (RNA medida pelo binário), o painel (Python), a história (API do Gitea), o manual público — por shell, porque o `deny` da linha nega `Read` |
| escreve | `SPEC_unblocked_brushes.md` (commit único, pós-filtragem) · este ledger · `VASSOURA_blender-unblocked.txt` · `fixtures/unblocked/` |
| ⛔ nunca | código de produto; `git push`; `git add -A`; `git stash`; `project-memory/`; o `blender_sculpt_oracle.py` partilhado; `doc-index.sh` |

### INBOX transcrito (append cego da janela, lido 2026-09-13)

> `I session: 9f820704-0d7e-4d96-847e-9cd720cbf178 2026-09-13 — janela I da line/sculpt3d (a mesma do INC-4 classificado RELANCE); obra: os quatro pinceis cujo bloqueio caiu (Simplify, Displacement Eraser, Displacement Smear, Scene Project)`

## Corrente I

| janela | session-id | data | motivo | declaração |
|---|---|---|---|---|
| I-3 | `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-13 | abriu esta obra (via INBOX acima) e despachou este E | ⏳ a declaração é da janela, pelo inbox |

---

## Cobertura da travessia (§3.E) — 2026-09-13

⚠️ **Duas sessões**: a primeira (19:30–20:05) morreu por limite de uso da conta depois do oráculo e
antes da espec; a segunda (20:15–) retomou daqui e fez a travessia de fonte. Nada foi comitado pela
primeira. O que a primeira deixou em disco (ledger, vassoura, harness, 129 dumps, notas da história)
foi **conferido e reutilizado**; o que ela não fez (a travessia, a espec, as fixtures) está abaixo.

### Fonte percorrido, arquivo a arquivo

| área | arquivos | o que se tirou |
|---|---|---|
| Os três pincéis que deformam | os três arquivos de pincel em `…/sculpt_paint/mesh/brushes/` (apagador · esfregão · projecção) | **INTEIROS, linha a linha** — as três leis por vértice, os três modos do esfregão, os alvos e o raycast da projecção |
| O pincel de topologia | não tem arquivo próprio — ele é **quatro sítios** no despacho principal do modo de escultura (a força que devolve zero · a cláusula que liga o colapso · o rótulo · o braço vazio do despacho de deformação) | ⭐ o achado central do §3.1 da espec: **ele não tem lei por vértice** |
| A cadeia de peso comum | o cabeçalho de declarações do módulo de pincéis de malha + as três implementações no despacho principal | a ordem dos 9 passos, a dureza exacta, e ⭐ que a curva **não** traz a força |
| A lei da força | a função de força por tipo de pincel no despacho principal | ⭐⭐ **a força ao quadrado**, com o motivo escrito pelos autores |
| O passe de topologia | o módulo de topologia dinâmica do kernel (2 414 linhas): as duas filas, as prioridades, os predicados de bordo, o filtro de máscara, o colapso de aresta inteiro, o ponto de entrada | §3.3–§3.7 da espec |
| Os tamanhos de detalhe | o módulo de detalhe do modo de escultura (as 5 conversões) + as duas constantes no cabeçalho de topologia dinâmica | §3.4 da espec, as três fórmulas exactas + `0,4` e `0,4` |
| A superfície de referência | a função de avaliação de limite no módulo de grelhas subdivididas do kernel | ⭐⭐ §2: é o **limite** paramétrico, não `k` passos |
| RNA / DNA | as definições das quatro propriedades públicas + os dois enums + o valor do tipo de pincel | nomes públicos, defaults, faixas (`minimum_distance`: default `0`, faixa dura `0..10`, macia `0..1`) |
| UI (Python) | o módulo comum de propriedades de pintura | quais controlos aparecem por pincel |
| Gating de dyntopo | o módulo de topologia dinâmica do editor (313 linhas) | nenhum dos quatro está numa lista de exclusão |

### História percorrida (via API do Gitea, sem abrir o front-end)

- **9 mensagens de commit** integrais: a que introduziu a projecção · a que introduziu o apagador ·
  a que introduziu o esfregão · a correcção do NaN do esfregão · os dois refactors orientados a dados
  · a que introduziu o pincel de topologia · a que introduziu a topologia dinâmica (2013) · a
  movimentação do esfregão para arquivo próprio.
- **1 pedido público** integral (o da projecção) + 115 KB de comentários dele.
- **10 relatórios de defeito** integrais + comentários: estoiro do esfregão com topologia dinâmica ·
  estoiro do pincel de topologia · **a referência errada em subdivisão simples** · áreas a
  desaparecer no esfregão · estoiro do filtro-irmão sem multirresolução · o aviso de console do
  pincel de topologia · buracos no esfregão · **a tarefa aberta de mover os ajustes de topologia para
  o pincel** · **a projecção a ler geometria base em vez de avaliada** · ferramentas inertes com
  tablete.
- **5 listagens de pesquisa** de defeitos (uma por pincel) para varrer o que não tinha sido nomeado.
- O **manual público** dos quatro pincéis + a página de topologia dinâmica.

### Oráculo

- Harness próprio (`~/Referencias/blender-unblocked/oracle/harness.py`) sobre a API pública do
  binário 5.2.1 LTS, com janela real. **129 dumps** em `~/Referencias/blender-unblocked/out/`.
- Re-derivação **independente** dos números que a espec afirma:
  `~/Referencias/blender-unblocked/oracle/verifica.py` — ⚠️ escrita porque as duas análises da 1.ª
  sessão **discordavam no SINAL** de doze corridas de projecção (a `analise_1.txt` está errada; a
  `analise_2.txt` está certa). *Quando duas páginas imprimem a mesma grandeza e discordam, isso é o
  achado.*
- Fixtures publicadas: `~/Referencias/blender-unblocked/oracle/fixtures.py` ⇒ **60** ficheiros em
  `docs/3D/cleanroom/fixtures/unblocked/` (4,5 MiB), com README de proveniência e tabela de
  excepções **derivada**.

### Vassoura

`80 → 175` entradas: mais 95, todas de identificadores internos e frases de comentário/prosa vistos
NESTA travessia. ⚠️ **A espec foi escrita sem UMA citação verbatim, de propósito** — várias das
frases que valeria a pena citar sob o direito de citação estão na vassoura, e uma espec que só passa
o portão porque o autor se lembrou de escolher outra frase não é auditável.

### Entrega

| artefacto | caminho | sweep |
|---|---|---|
| Espec | `docs/3D/cleanroom/SPEC_unblocked_brushes.md` | ✓ verde |
| Fixtures + README | `docs/3D/cleanroom/fixtures/unblocked/` | ✓ verde |
| Este ledger | `docs/3D/cleanroom/LEDGER_blender-unblocked.md` | (não vai à janela) |

**Incidentes:** nenhum.

---

## Papel R — Revisor, modo PRÉ (§3.R)

| campo | valor |
|---|---|
| quem | **subagente R-pré** despachado pela janela `9f820704-0d7e-4d96-847e-9cd720cbf178` (I-3 da `line/sculpt3d`) — contexto novo, **distinto** do subagente-E que escreveu a espec |
| data | 2026-09-13 |
| leu | os dois lados: a espec + os três ficheiros de pincel do alvo, o despacho do modo de escultura (a lei da força, a cláusula que liga o colapso), o módulo de topologia dinâmica do kernel (prioridades, recursão, filtro de máscara, colapso de aresta), o módulo de detalhe e as definições RNA — tudo **por shell**, fora da árvore |
| escreveu | este bloco · o cabeçalho da espec · `VASSOURA_blender-unblocked.txt` (175 → **181**) |
| veredito | ⛔ **REPROVADA — 3 achados.** Sem atestado; a janela não implementa |

### O achado de INSTRUMENTO (e é o que explica os outros)

⚠️⚠️ **O sweep de E fechou VERDE sobre uma espec que carrega quatro traduções de prosa do alvo.**
A vassoura está na **língua do alvo** e a espec escreve-se **na nossa** ⇒ toda frase de comentário
vertida para português passa por baixo dela, por construção.

⭐ **A prova é decisiva e sai da própria vassoura de E:** o inglês de **cada uma** das frases que
o R-pré apanhou **já lá estava** — o E julgou-as idiossincráticas ao ponto de as varrer, e depois
escreveu-as traduzidas na espec. *O instrumento estava certo e cego ao mesmo tempo; o que faltava
era a outra língua.*

⇒ juntaram-se **6** marcas de prosa **em português** (base64, como as outras). O sweep passa a
**VERMELHO nos 6 sítios da espec** e fica **limpo nas 60 fixtures** — o vermelho é o controlo
positivo do instrumento e **a condição de re-entrada é ele voltar a verde sobre as 181**.

### Os 3 achados (mecanismo; o texto funcional foi devolvido à janela pelo contrato de retorno)

| # | onde | §4.2 violado | mecanismo |
|---|---|---|---|
| 1 | espec §3.2, o bloco de pseudo-código | *pseudo-código que espelha o original* | transliteração do fluxo de controlo do alvo passo a passo (inicializar vazio · guardar no modo manual · dois acumuladores de bandeira por «ou-bit», na ordem dele). ⭐ **A prosa imediatamente acima e abaixo já diz tudo o que ele diz** ⇒ o bloco só acrescenta **forma** |
| 2 | espec §5.2, o bloco de pseudo-código | idem | espelha o laço interior do alvo declaração a declaração, com os **mesmos intermediários** e na mesma ordem. ⛔ **A assinatura que o denuncia:** ele preserva um desvio que é **matematicamente inerte** (onde o desvio salta, o peso já valeria zero e os dois acumuladores ficariam intactos) — *preservar um ramo sem força funcional é a marca de transcrição, não de descrição* |
| 3 | espec §3.6 · §3.7 · §6.3 (4 sítios) | *comentários do original* | prosa de comentário do alvo vertida quase palavra a palavra, **com a oração causal incluída**, apresentada sem aspas e sem fonte. ⚠️ §4.1.12 admite a sabedoria dos autores **re-dita** ou citada **curta e entre aspas**; isto não é nenhuma das duas |

### O que foi conferido e está LIMPO (para não ser re-auditado)

- **Nomes:** zero identificador interno. Tudo o que a espec e os cabeçalhos das fixtures nomeiam
  é **API pública** (tipos de pincel, os dois enums de direcção e de deformação, as propriedades de
  folga e de duplo sentido, rótulos de UI) — conferido **contra as definições RNA**, §4.1.13.
- **Tabelas:** ⭐ **nenhuma LUT.** A peça de substrato em falta é dada por **fórmula publicada**, e a
  espec **proíbe por escrito** a tabela por valência — exemplar.
- **Constantes:** as três conversões de detalhe, os dois multiplicadores de prioridade, as duas
  constantes da recursão e a lei da dureza entram como **matemática/facto com proveniência** (§4.1.2–3),
  em forma de fórmula, não de código. ⭐ E a espec **acrescenta** um facto que o alvo não diz — que dois
  dos números iguais são constantes **diferentes**.
- **Organização:** descrita por fases funcionais; a cadeia de peso é uma **ordem observável** (trocá-la
  muda o resultado), logo é §4.1.1 e não decomposição arbitrária.
- **Cobertura:** a §6 **não** tem buraco de controlo — a triagem de RNA confirma que o pincel de
  projecção expõe exactamente os controlos que a espec descreve.
- **Fixtures:** 60 ficheiros, nomes e chaves de cabeçalho em vocabulário nosso, entradas **nossas**,
  sweep limpo sobre as 181 entradas.

---

## Emenda §3.E — a reescrita depois do R-pré (2026-09-13)

O **R-pré NÃO atestou** a 1.ª redacção: 3 achados, o 1.º substancial. O que ele apanhou, o que foi
feito, e o instrumento que fica:

| # | achado | cura |
|---|---|---|
| 1 | **§3.2** — bloco de pseudo-código a transliterar o fluxo de controlo do alvo (inicializar vazio · guardar num modo · dois acumuladores por «ou-bit», na ordem dele). *A prosa em volta já dizia tudo; o bloco acrescentava FORMA.* | **APAGADO.** Fica a prosa, com uma frase nova a nomear o que ele é funcionalmente: uma disjunção acrescentada à condição que já decidia o colapso |
| 2 | **§5.2** — bloco a espelhar o laço interior declaração a declaração. ⭐ **A assinatura que o denunciou:** ele preservava um desvio **matematicamente inerte** — no ramo saltado o peso já valeria zero e as duas somas ficariam intactas. *Preservar um ramo sem força funcional é marca de transcrição, não de descrição* | **REESCRITO como FÓRMULA**: média ponderada do campo sobre a vizinhança, peso próprio `1`, peso do vizinho = parte negativa do cosseno. O desvio foi **demovido a nota de custo** (§4.1.8), dizendo-se optimização |
| 3 | **§3.6 · §3.7 (×2) · §6.3** — prosa de comentário vertida quase palavra a palavra, com a oração causal incluída, sem aspas e sem fonte | **RE-DITO como REQUISITO** em cada um: a terminação da recursão e o piso relativo (§3.6) · a invariante de saída sobre faces repetidas (§3.7) · a lei que fica sobre a máscara saturada + de onde vem a protecção da região atenuada (§3.7) · a **derivação** da invariância paramétrica e o requisito de transformar direcção como direcção (§6.3) |

⚠️⚠️ **A lição do instrumento, e ela atravessa as quatro obras de hoje:** a vassoura estava só na
**língua do alvo** e a espec escreve-se **na nossa** — o inglês de cada frase apanhada **já lá
estava** e não casava com a tradução. *Uma vassoura monolingue não vigia uma espec traduzida, e o
verde dela lê-se exactamente como o verde de uma que vigia.*

**Vassoura: `175 → 181` (R-pré, prosa em português) `→ 196`** — mais **15 sentinelas** do texto
EXACTO que foi removido nos dois blocos, para que uma **recaída** reprove o sweep em vez de esperar
por outra auditoria humana.

⭐ **Controlo positivo** (senão são 15 entradas a medir nada): a mesma vassoura sobre a redacção
anterior (`de143530f`) dá **19 hits** — `5` do 1.º bloco, `8` do 2.º, `6` da prosa —; sobre a
redacção nova dá **0**. Cross-sweep contra as quatro vassouras irmãs: limpo nas quatro.

⏳ **PENDENTE: R-pré NOVO sobre esta redacção.** ⛔ Quem reescreveu não atesta — *autofiltragem não
é auditoria*, e foi precisamente a autofiltragem verde da 1.ª entrega que deixou passar os três.

## Incidentes

### INC-U1 — a 1.ª redacção FICOU NO HISTÓRICO (registado 2026-09-13, por quem reescreveu)

**O que é.** A cura dos 3 achados do R-pré limpou a **árvore**, e não o **histórico**. O commit
`de143530f` acrescentou a 1.ª redacção, e `git log -p` retém o patch dela para sempre. Medido com a
vassoura de 196 em modo `--git-history` sobre o caminho da espec: **38 linhas** de achado, todas no
patch daquele commit — os dois blocos de pseudo-código e os 4 sítios de prosa. A árvore em `HEAD`
está **limpa** (`0` hits sobre a espec e sobre as 60 fixtures, nas cinco vassouras).

⛔ **Não reproduzido aqui, por regra (§6.1).** O endereço é o sha; o conteúdo é o que a §4.2 nomeia.

**A regra que isto viola, e ela estava escrita.** A SKILL §3.E manda a espec entrar num **commit
ÚNICO, pós-filtragem (§4.3)**, *«nunca rascunhos incrementais: `git log -p` retém para sempre o que
um rascunho contaminado carregou»*. O commit **foi** único — e foi **pré-auditoria**. ⚠️ **A lei
tem uma metade que eu li como cumprida e não estava:** «pós-filtragem» não quer dizer *depois da
MINHA filtragem*; a §4.3 acaba em «autofiltragem não é auditoria», logo o commit único só é lícito
**depois do R-pré**. *Um commit único antes da auditoria é exactamente o rascunho que a regra proíbe,
com outro nome.*

**O que NÃO foi feito, e porquê.** ⛔ Nenhuma reescrita de histórico. Esta worktree tem **outros
três subagentes a commitar no mesmo ramo agora** (`blender-pull`, `blender-pose`,
`blender-boundary`) e a janela-mãe a escrever código em `crates/`; um rebase aqui destrói trabalho
alheio em curso. ⇒ **decisão do Enio**, com as três saídas e o preço de cada uma:

| saída | preço |
|---|---|
| **Deixar** e registar (o estado actual) | o histórico do ramo retém 38 linhas de forma derivada do alvo até ao fim da vida do repo. ⚠️ O R-pós corre `--git-history` no fechamento (§7.2b) e **vai** reprovar — com este registo a explicar porquê |
| **Squash da linha na integração** | o integrador funde `de143530f`+`b1026c9e8`+`9efef37ba` num commit só, com a redacção final. Barato **se** a integração já for por squash; ⛔ perde a rastreabilidade dos 3 achados, que este ledger passa a ser o único sítio a guardar |
| **Reescrever o histórico do ramo** | ⛔ **fora de questão hoje** — três linhas vivas no mesmo ramo |

**A lei que fica, e ela é para as quatro obras de hoje:** ⇒ *a espec commita-se **depois** do
R-pré, não antes.* O ciclo certo é **draft fora da árvore → R-pré → cura → commit ÚNICO**; o que
correu aqui foi *commit → R-pré → cura → 2.º commit*, e é a diferença entre um histórico limpo e um
histórico com registo de incidente. ⚠️ Esta obra **não** pode voltar atrás; as irmãs ainda podem.

#### INC-U1 — a MEDIÇÃO em TODA a obra da linha (R-pré 2.ª passagem, 2026-09-13)

⛔⛔ **O `38` acima é o tamanho do alvo `unblocked` visto de dentro do alvo `unblocked`, e as duas
saídas que o quadro de cima orça (deixar · squash da linha) foram orçadas contra ele.** Corrido o
sweep em `--git-history` **por alvo**, no range `main..HEAD` (as 25 commits desta linha):

| alvo | linhas de patch com achado | `−` remoção | `+` adição | quem as carrega |
|---|---|---|---|---|
| `blender-cloth` | **201** | 200 | 1 | `ec47dc846` **187** · `cb279c0cb` 13 · cleanroom 1 |
| `blender-pull` | **90** | 74 | 13 | `ec47dc846` **73** · cleanroom 14 · `cb279c0cb` 3 |
| `blender-unblocked` | **81** | 56 | 24 | `ec47dc846` **36** · `de143530f` 19 · `9efef37ba` 19 · outros 7 |
| `blender-boundary` | **27** | 17 | 8 | `ec47dc846` 12 · `23bb85670` 7 · `881e96d9e` 6 · outros 2 |
| `blender-pose` | **16** | 7 | 8 | `3ea4f7576` 7 · `ef681891f` 5 · `ec47dc846` 2 · outros 2 |
| `quadwild` | **0** | 0 | 0 | — |
| **total** | **415** | **354** | **54** | `crates/**` **328** · `docs/3D/cleanroom/**` **87** |

⭐⭐ **Três leituras, e cada uma muda a decisão:**

1. **O problema é `11×` maior que o medido, e mora noutro sítio.** `328` das `415` linhas vivem em
   patches de `crates/**`, e **`327` delas são as linhas `−` da cura do INC-4** (`ec47dc846`, os
   173 blocos de comentário reescritos, mais o resíduo `cb279c0cb`). ⇒ *a cura que limpou a árvore
   pôs no histórico, de uma vez, tudo o que ela apagou.* ⛔ **O squash dos commits de docs não lhe
   toca**: ele alcança no máximo as `87` de `docs/3D/cleanroom/**`, e `de143530f`+`9efef37ba` são
   `38` dessas.
2. ⛔⛔ **E o texto acusado JÁ ESTÁ no histórico do `main`, como ADIÇÃO.** Mesmo sweep, range `main`
   (sem esta linha): `cloth` **409** `+` · `pull` **865** `+` · `unblocked` **201** `+` ·
   `boundary` **65** `+` — **1 540 adições**, escritas ao longo de meses pelas linhas que
   redigiram aqueles comentários. ⇒ **reescrever o histórico DESTA linha não cura nada**: ela só
   contém as **remoções**; as adições são anteriores a ela e estão em `main` desde sempre.
   *Uma reescrita de ramo que deixa a origem intacta compra a aparência da cura e nenhum byte dela.*
3. **A saída «squash da linha» fica com o preço invertido.** Ela custa a rastreabilidade dos
   achados (como o quadro de cima já dizia) e compra `≤ 87` de `415` linhas desta linha e `0` das
   `1 540` do `main`. ⇒ ⏳ **decisão do Enio, com a pergunta agora certa:** *o que se faz com o
   histórico do REPO*, não com o desta linha — e a resposta honesta pode ser **declarar**, não
   reescrever (o registo é defesa; o escondido é a acusação pronta — §6).

⚠️ **Duas cegueiras do instrumento, medidas ao correr isto:** a coluna `+` de `main` inclui
**falsos positivos do nosso próprio vocabulário** — entradas de vassoura que casam com símbolos
públicos nossos (um factor de sobreposição de espaçamento, uma chave de i18n, um nome de directório
de painel, um identificador de raio de fronteira num teste de campo). *Uma vassoura com entrada
genérica reprova a nossa própria árvore e enterra o achado verdadeiro no meio dela.* E o sweep de
árvore com as vassouras irmãs sobre `crates docs shells scripts` está **vermelho** (`41`/`73`/`6`/
`370`/`83`/`6` linhas), com a maioria em `docs/**` — que o §5 do `CLAUDE.md` declara **fora do
censo por construção** — e um resíduo em doc-comments de `crates/**` que é a dívida de **nomes de
SÍMBOLO** já nomeada como ABERTA no fecho de 13/09.

---

## §3.R-pré (2.ª passagem) — 2026-09-13 · ⛔ **NÃO ATESTADA**

Subagente R-pré novo, contexto independente do que escreveu a espec **e** do que escreveu a
reescrita. Leu os dois lados. Veredito contra o §4.2: **4 achados, 1 SUBSTANCIAL.**

**O que foi CONFIRMADO (cura a cura, com o instrumento corrido por terceiro):**

| cura | veredito |
|---|---|
| Achado 1 — o bloco da §3.2 | ✅ **APAGADO**, conferido contra `b1026c9e8` |
| Achado 2 — o bloco da §5.2 | ✅ **REESCRITO em forma fechada**; o desvio inerte está na nota de custo e declara-se optimização |
| Achado 3 — os 4 sítios de prosa | ✅ **RE-DITOS como REQUISITO** nos quatro (§3.6 as duas constantes · §3.7 a invariante de saída · §3.7 a lei da máscara saturada · §6.3 a derivação + o requisito de transformar direcção como direcção). ⭐ Os quatro ficaram **mais fortes** do que eram: três deles passaram a dar o MECANISMO em vez do argumento dos autores |
| Controlo positivo | ✅ **CORRIDO PELO R**, não aceite do autor: a vassoura de 196 sobre `b1026c9e8` dá **19** linhas (5 + 8 + 6, na repartição declarada); sobre `HEAD` dá **0**. Sweep de `HEAD` sobre a espec + as 60 fixtures: **exit 0** |

⛔ **O que a emenda NÃO alcançou — e é a mesma espécie, em secções que ela não tocou** (a lição que
a obra irmã `blender-pose` já pagou na EMENDA 2): a acusação era *«prosa de comentário traduzida»*,
e a cura foi aplicada **aos quatro endereços acusados**. A espécie maior — **descrever o PROGRAMA
em vez do comportamento** — continua viva em dois sítios que ninguém nomeou, e a §3.1 é
**byte-idêntica** à redacção reprovada. Os achados estão no report ao I, em termos funcionais.

⛔⛔ **E o 4.º achado é do QUADRO DE ATESTADOS, não do corpo:** a política de nomes desta obra
(*zero identificador interno; tudo o que a espec nomeia é API pública, conferida contra as
definições que o binário publica, §4.1.13*) está registada **neste ledger** — e a §6 proíbe a
janela de o abrir, enquanto a §4 define o cabeçalho da espec como *«o quadro de atestados que I
confere (I não lê o ledger)»*. ⇒ **o Implementador não tem como distinguir um identificador público
lícito de um nome interno proibido**, e o incidente desta mesma linha em 13/09 (INC-4) foi
exactamente nomes internos a alojarem-se nos nossos comentários. *Um atestado guardado no único
ficheiro que o destinatário não pode ler não atesta nada para ele.*

⚠️ **Esta espec fica BLOQUEADA para implementação** até a 3.ª redacção e um R-pré novo sobre ela.

---

## Emenda 2 §3.E — a 2.ª passagem do R-pré (2026-09-13)

Ele **confirmou as três curas da 1.ª emenda uma a uma** (e registou que três delas ficaram *mais
fortes* por darem o nosso mecanismo em vez do argumento dos autores), correu o **controlo positivo
dele próprio** — `19` linhas na redacção anterior, `0` na de então — e **não atestou**: mais `4`
achados, um substancial.

| # | achado | cura |
|---|---|---|
| 1 | ⛔ **SUBSTANCIAL. §3.1, byte-idêntica à redacção já reprovada — a 1.ª emenda não lhe tocou.** Três orações sustentavam um facto de comportamento descrevendo o **INVENTÁRIO e a FORMA** do código do alvo | **APAGADAS**, e o facto ficou **MEDIDO**: `densidade_detalhe_manual` dá `max │Δposição│ = 0` **exactamente** e `0` de `1 681` vértices movidos ⇒ *o pincel é inteiramente inerte com a metade que ele arma fora de jogo*. ⭐ **Ele não tinha fixture nenhuma, e a atribuição ao fonte estava a fazer o trabalho de uma** |
| 2 | §1.1 — o motivo ergonómico do expoente da força atribuído a texto escrito no fonte | a oração saiu; a §1.1 aponta à **§9.1**, que já declara a proveniência lícita (mensagens de commit · o pedido público · o relatório de defeitos · o manual) |
| 3 | §3.2 — a frase que substituiu o bloco caracterizava o pincel pela **FORMA DA CONDIÇÃO** no código | **TABELA-VERDADE** (4 linhas), que as 4 fixtures da própria secção provam: o colapso corre se a preferência o pede **ou** se este pincel está em mãos · o partir só se a preferência o pede · o detalhe Manual desarma os dois |
| 4 | Cabeçalho — a **política de nomes** vivia só neste ledger, e o protocolo veda o ledger à janela | escrita no cabeçalho: zero identificador interno · o que a espec nomeia é **interface pública**, obtida **correndo** o alvo e despejando o que ele publica (nunca lendo) · o resto é vocabulário do domínio · e **no corpo o RÓTULO lidera** (títulos e tabelas dizem o que o artista vê; o identificador fica onde regenerar uma fixture o exige) |

**Menores, tratados:** as constantes do passe de topologia ganharam **linha de proveniência no
cabeçalho** — entram como *facto LIDO*, com o caminho para as medir nomeado e declarado por correr ·
duas orações trocaram verbo de código por facto de **estado** (o campo de deslocamento *tem valor
inicial zero*, em vez de *é alocado a zeros*) · e a narrativa da subtracção redundante foi trocada
pelo que importa: **o zero inicial é a cura publicada de uma regressão** (vértices sem valor
definido propagavam NaN), e o que a cura **não** fez foi alargar a actualização à vizinhança.

⚠️⚠️ **A LEI DAS DUAS RONDAS, e é a mais cara desta obra:** *uma emenda cura o que o auditor
NOMEOU.* A §3.1 sobreviveu à 1.ª passagem inteira — com um achado substancial dentro — **por
ninguém lhe ter apontado o dedo**, e eu reescrevi à volta dela sem a ler pela pergunta da §4.3.1.
⇒ a regra que fica, escrita no cabeçalho da espec: **quem reescreve varre o documento INTEIRO** por
*«isto descreve o que o programa FAZ, ou como ele está ESCRITO?»*, nunca só os parágrafos citados.
⭐ Aplicada agora ao documento todo por varredura textual (`o fonte` · `a função` · `o comentário` ·
`despacho` · `devolve` · `invoca` · `o módulo` · `o ficheiro` · …): **zero** ocorrências a descrever
a estrutura do alvo; os únicos hits são sobre o NOSSO lado e sobre o processo.

**Vassoura: `196 → 206`** — mais `10` sentinelas do texto exacto removido nesta ronda.
⭐ **Controlo positivo:** a mesma vassoura sobre a redacção que o R-pré reprovou (`9efef37ba`) acusa
**as quatro** linhas — §1.1, §3.1, §3.2 e §5.4 —; sobre esta acusa `0`. Cross-sweep contra as quatro
vassouras irmãs: limpo nas quatro.

⏳ **PENDENTE: 3.ª passagem do R-pré.** ⛔ A janela não implementa nem lê a espec até a linha
«auditada contra §4.2 por R-pré em `<data>`» estar escrita, e **não é quem reescreveu que a escreve**.

---

## §3.R-pré (3.ª passagem) — 2026-09-13 · ✅ **ATESTADA**

Subagente R-pré **novo** (contexto independente; não é o autor de nenhuma das três redacções, nem o
subagente-E). Redacção auditada: `a9655a9ed` + as duas curas de higiene desta mesma passagem.

**Veredito: ZERO achados de §4.2. A espec está `auditada contra §4.2 por R-pré em 2026-09-13`**, e a
linha está escrita no cabeçalho dela.

### O que foi corrido (e não só o que a passagem anterior nomeou)

A 2.ª passagem deixou a lei escrita: *uma emenda cura o que o auditor NOMEOU*. Esta passagem
confirmou as **sete** curas uma a uma **por diff** contra as redacções reprovadas e depois varreu o
documento inteiro, de novo, pela pergunta da §4.3.1 — porque foi exactamente assim que a 2.ª achou o
que a 1.ª não viu.

| # | régua | resultado |
|---|---|---|
| 1 | **Censo de identificadores do corpo** (todo token entre crases, menos os nossos e a notação matemática) | **zero** fora da lista pública declarada; tudo o resto é `ph2d_*`/`crates/…` nosso ou nome de fixture nosso |
| 2 | **Varredura mecânica por linguagem de FORMA**, com termos que as passagens anteriores **não** usaram (`assinatura` · `parâmetro` · `argumento` · `struct` · `classe` · `aloca` · `buffer` · `ponteiro` · `array` · `bitflag` · `decompo…` · `iteração` · `percorre` · `linha a linha` · …) | **zero** a descrever a estrutura do alvo. Os `17` hits são: o cabeçalho a falar do processo (7), o NOSSO lado (3), matemática e notas de custo (5), o guarda-costas do §5.2 (1) e `ramo` no sentido de **ramo de git** na §9.3 (1) |
| 3 | **Os dois blocos de pseudo-código reprovados na 1.ª ronda** | um **ausente**; o outro é média ponderada em **forma fechada**, com o ramo matematicamente inerte demovido a nota de custo que se **auto-declara** optimização |
| 4 | **A §3.1 — o achado SUBSTANCIAL da 2.ª ronda** | as três orações de inventário/forma **não existem**; o facto é medido — e ⭐ **o R-pré re-verificou a medição contra a própria fixture**, ver abaixo |
| 5 | **Citação verbatim** | zero, como o documento declara de si mesmo |
| 6 | **LUT afinada à mão** (§4.2, último ponto) | zero — a §2.3 dá a **fórmula geradora** e **proíbe por escrito** a tabela de pesos por valência, que é o caminho (a) do §4.2 |
| 7 | **As afirmações sobre o NOSSO código** (não é §4.2, mas um Implementador age sobre elas) | **todas verdadeiras**: `HYSTERESIS = 2.05` e o joelho `1,8–2,0` estão no cabeçalho de `collapse.rs` · a meia-extensão `0,4198` do limite do cubo está em `shapes.rs` · as cinco funções e os cinco ficheiros citados existem, nos ficheiros que a espec nomeia |

### ⭐ A medição da §3.1, re-verificada pelo auditor (e não aceite por declaração)

A 2.ª passagem exigiu que o facto deixasse de se apoiar no fonte e passasse a apoiar-se numa
fixture. Ele passou — e **uma fixture citada não é uma fixture conferida**, por isso o R-pré abriu-a
e recontou:

| grandeza | a espec afirma | o R-pré mediu na fixture |
|---|---|---|
| `max │saída − repouso│` | `0` exactamente | **`0.0`** exactamente |
| vértices movidos | `0` de `1 681` | **`0` de `1 681`** |
| contagem de faces | inalterada | **`3 200 → 3 200`** |
| pontos do cursor | traço de `6` | **`6`** |
| o desarme | *Detailing* **Manual** | cabeçalho: tipo de detalhe **MANUAL**, refino *Subdivide Collapse* |
| o load-bearing | auto-alisamento a `0` | cabeçalho: auto-alisamento **`0.0`** |

⇒ a atribuição ao fonte foi substituída por uma medição que **é verdadeira ao bit**, e o facto ficou
mais forte do que a versão reprovada.

### Controlo positivo — corrido pelo auditor sobre as duas redacções reprovadas

⛔ *Um sweep verde cujo controlo positivo ninguém correu não prova filtragem; prova que o
instrumento não foi apontado a nada.*

| redacção | veredito na altura | hits |
|---|---|---|
| `de143530f` (1.ª) | reprovada, 3 achados | **22** |
| `9efef37ba` (1.ª emenda) | reprovada, 4 achados | **4** — *uma por achado nomeado* |
| esta | atestada | **0** |

### ⚠️ O INSTRUMENTO tinha uma cegueira, medida e curada: a DE-ACENTUAÇÃO

A obra irmã (`blender-boundary`) mediu no mesmo dia que de-acentuar a prosa portuguesa mata parte
dos achados. Medido **aqui**: o sweep casa por texto **exacto** (`grep -F`), e **`21` das `80`
entradas de prosa (`26,2 %`) carregam marca diacrítica** — a mesma frase escrita sem acentos passava
por baixo de todas elas. ⭐ *É o buraco da vassoura monolingue um nível abaixo: lá era a **língua**,
aqui é a **ortografia**.*

⇒ **vassoura `206 → 227`**: as `21` variantes sem acento, geradas por NFD (nenhuma já lá estava).
⚠️ **Estritamente aditivas, e provado**: os três controlos dão **o mesmo número** com `206` e com
`227` (`22` · `4` · `0`), logo nenhuma delas fabrica um falso positivo sobre a redacção curada.
⚠️ **Fica NOMEADA a cegueira gémea que NÃO foi curada:** `60` das `227` entradas têm maiúscula e o
`grep -F` é sensível à caixa. Não foi tratada porque uma tradução muda acentos e **não** muda caixa
— mas quem escrever uma vassoura nova herda a pergunta.

### As DUAS notas de higiene da parede — não bloqueantes, e curadas por esta passagem

Nenhuma é violação do §4.2 (nada do alvo vazou); as duas são **bagagem da parede**, e o R-pré é a
parte certa para as curar porque não tocam em afirmação nenhuma de comportamento.

1. **O quadro de nomes declarava-se fechado e não o era.** Ele lista os identificadores do **corpo**
   — e o censo confirma que ali não há outro —, mas a §7 manda o Implementador ler o **cabeçalho de
   cada fixture**, onde aparecem mais valores públicos (método de refino, tipo de detalhe, método do
   traço, forma da pegada, predefinição de curva, sentido do pincel). São lícitos pela mesma razão e
   pelo mesmo modo de obtenção — o arnês **escreveu-os** pela API pública para a corrida ser
   regenerável (§4.1.13) — e o `README` das fixtures declara a cadeia. ⇒ o quadro passa a dizer o seu
   **âmbito** e a apontar o README. ⛔ *Uma lista que se lê como fechada e não cobre os artefactos
   para que a própria página aponta ensina o Implementador a parar de a consultar* — e ela é a única
   defesa dele, porque este ledger lhe é vedado.
2. **A §2 trazia uma LIGAÇÃO VIVA para este ledger**, que o cabeçalho da espec declara vedado ao
   Implementador, na mesma frase que dizia que ele não precisa dela. ⇒ **ligação removida**, facto
   mantido inteiro. ⚠️ O risco medido é **baixo e não nulo**: a regra `Read(docs/**/cleanroom/LEDGER_*)`
   **existe** e o Read falha fechado, e este ledger passa o sweep a **zero** — mas um `Read` negado
   convida ao `cat`, que **não** está negado, e uma página que liga ao que ela própria declara vedado
   ensina que a cerca é conselho. *A cerca é o mecanismo; o documento não deve trabalhar contra ela.*

### O que ficou conferido e NÃO precisa de ser re-auditado (salvo reescrita)

§1 (a cadeia de peso é ordem de fases — §4.1.1) · §1.1–§1.3 · §2.1–§2.4 · §2.3 (fórmulas dos papers
públicos, com a proibição da LUT escrita) · §3.2 (tabela-verdade; **e o «modo com duas bandeiras» é
`o que entra` de uma fase, §4.1.1 — além de ser vocabulário que o NOSSO `dyntopo.rs` já usa, com
`Refine` e o par `refine_in_sphere`/`collapse_in_sphere`**) · §3.3–§3.9 · §4 · §5 · §6 · §7 (chaves
e nomes de ficheiro das fixtures são **nossos**, em português) · §8 · §9 (§4.1.12, re-dito, endereços
só aqui) · §10 · a tabela de recusas medidas.

⛔ **Uma observação de QUALIDADE, sem consequência de parede** (fica registada para quem implementar,
não bloqueia): a forma prática do ponto-limite de Loop na §2.3 está escrita com uma elipse (`…`) e a
primeira das duas igualdades não fecha. O caminho lícito está lá e é o certo — **derivar** do
auto-vector à esquerda dominante, com a tese pública no mapa de leitura — mas quem implementar deve
contar com derivar, não com copiar aquela linha.
