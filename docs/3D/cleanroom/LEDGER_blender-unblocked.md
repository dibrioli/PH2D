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
