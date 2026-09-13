# LEDGER de proveniência — clean-room do pincel POSE (alvo `blender-pose`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-pose.md` (append cego).

⚠️ **Aberto em 2026-09-13, ANTES da primeira leitura do fonte** (§6). As secções *Alvo*,
*Triagem* e *Patente* foram escritas com o fonte FECHADO: a única leitura do checkout até aqui
foi o ficheiro de licença na raiz (`COPYING`) e os metadados de git (`git log -1`,
`git sparse-checkout list`), que não são expressão do pincel.

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — pincel **Pose** do Sculpt Mode (nome público do enum `sculpt_brush_type`: `POSE`) + o que ele invoca que decide comportamento (máscara, simetria, conectividade, pressão, inversão, undo, custo) |
| Versão / commit do fonte | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` (2026-07-13), checkout **esparso e grafted** (sem história local; a história vem da web) — o MESMO checkout que a obra `blender-cloth` leu |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D. Zona de notas/rascunhos/oráculo desta obra: `~/Referencias/blender-pose/` |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença | **GPL-2.0-or-later** — `COPYING` diz que o Blender usa a GNU GPL, remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991) e declara que não está disponível sob outra licença (lido 2026-09-13); o cabeçalho SPDX dos ficheiros do pincel é conferido na travessia |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS** (build 2026-09-01) — ⚠️ patch-release acima do fonte lido (5.2.0); a diferença fica registada e **o oráculo é o binário** |
| Não é AGPL | ⇒ oráculo local por escolha, não por obrigação |

### A concessão relevante (GPLv2 §0 e §2)

> *"The act of running the Program is not restricted, and the output from the Program is
> covered only if its contents constitute a work based on the Program (independent of having
> been made by running the Program)."* (§0) · *"You may modify your copy or copies of the
> Program or any portion of it…"* (§2)

⇒ Ler, correr, modificar e instrumentar **em privado** é licenciado; a **saída** (posições de
vértices de malhas NOSSAS) é dado (SKILL §1.1/§5). Nenhum acto deste ledger distribui o alvo.

---

## §2 — Triagem: a escada de portas (2026-09-13, fonte fechado)

| degrau | veredito | o que foi buscado |
|---|---|---|
| T0 | ⛔ não para o pincel | o pincel só existe neste alvo, sob GPL |
| T0 (só o **solver de cadeia**) | ✅ **existe porta permissiva para a FAMÍLIA do solver**: o *Coding Challenge #64* da Coding Train (cadeia de segmentos que seguem um alvo, com base fixa), repositório `CodingTrain/website` sob **MIT** (LICENSE lido por `curl` em 2026-09-13, «Copyright (c) 2019 Coding Train»). A página pública do alvo (BlenderNation, 2019-12-12) diz que o solver do pincel remete a esse tutorial | ⚠️ vale para o **solver** genérico «segue-o-líder» e não para a semântica do pincel (achar a origem, os segmentos, os pesos, as deformações), que é o que esta espec fecha. Não se porta nada dele por esta obra — fica registado como literatura livre |
| T1 (a) código dos autores | ⛔ não há paper nem código de referência separado do pincel | busca «pose brush» + autor |
| T1 (b) versão antiga sob licença branda | ⛔ o Blender é GPL desde 2002; o pincel nasceu em 2019 (2.81) | — |
| T1 (c) reimplementação permissiva independente | ⛔ nenhuma achada: **SculptGL** (MIT) não tem pincel de pose; **VSculpt** (`khanhha/VSculpt`) lista Grab/Snake Hook/Rotate e nenhum Pose; **ClaySpaceDesktop** (`CyberdyneCorp`, criado 2026-08-10) tem `license: null` na API do GitHub ⇒ **todos os direitos reservados**, e não é porta; **BrokkrSculpt** é AGPL-3.0; `freestyle-sculpt` (Rust) é dyntopo; **Nomad Sculpt** é proprietário (e o fórum dele tem o pedido de *pose brush* como *feature request*) | WebSearch 2026-09-13: «open source sculpting "pose brush" implementation MIT» · «github sculpt "pose brush" javascript OR rust OR c++ license» |
| T1 (d) e-mail ao autor | não aplicável — o alvo é projecto institucional com cessão à Fundação; não há dual-license possível por um autor | — |
| **T2** | ✅ **degrau desta obra** — copyleft com fonte | o dono ordenou a obra (MISSÃO-E da janela I) |

---

## Patente (§8.1) — checkpoint incondicional (2026-09-13)

- **Termos:** `sculpting brush pose deformation inverse kinematics chain mesh topology flood fill` ·
  `site:patents.google.com sculpting mesh brush inverse kinematics segments pose deformation user stroke` ·
  `patent digital sculpting posing tool rotate region of mesh around pivot falloff weights` ·
  `"sculpting" "posing" mesh "geodesic" OR "topological" mask rotation joint Autodesk OR Maxon OR Pixologic OR Adobe` ·
  `Pixologic ZBrush Transpose patent mask topology posing "action line"`.
- **Resultado:** ⭐ **nenhuma patente viva alcança o método** (pincel que, a partir do cursor, acha
  por distância ao longo da malha uma cadeia de segmentos, resolve uma cadeia de IK que segue o
  arrasto e deforma cada região por rotação/escala/translação ponderada por um peso suavizado).

| patente | dono | estado | lê sobre nós? |
|---|---|---|---|
| **US 9 460 556 B2** — *interactive masking and modifying of 3D objects* (a máscara topológica + a «action line» do Transpose) | Pixologic | ⭐ **EXPIRADA** (falta de pagamento de anuidade; o Google Patents dá *lapsed* 2024) — prioridade 2007-02-20 | ⭐ é a patente mais próxima **e está expirada** ⇒ literatura livre (§8.1: «melhor documento de espec do mundo, de graça»). A reivindicação 2 é máscara por distância topológica entre vértices; a 1, rodar/escalar/mover a região não mascarada em relação a uma linha desenhada |
| US 8 704 828 B1 — *inverse kinematic melting for posing models* | Pixar | **VIVA até 2031-09-27** | ⛔ não — a reivindicação 1 exige atributos de pose de **graus de liberdade de um modelo rigado**, uma «pose profunda» de repouso e uma pose atractora que se modifica por comparação; o pincel não tem rig, nem pose profunda, nem atractor. ⚠️ **cerca nomeada**: nunca implementar um «derreter de volta à pose de repouso» por comparação com uma pose atractora |
| **US 10 586 401 B2** — pincéis por soluções regularizadas de elasticidade (Kelvinlets) | Pixar | VIVA até 2038 | ⛔ não — a reivindicação exige soluções analíticas fechadas da elasticidade linear. ⚠️ **cerca herdada da obra `blender-cloth`**: nunca implementar um modo «elástico» por Kelvinlet dentro do Pose |
| US 2016/0092033 A1 (US 9 710 910?) — escultura e deformação em dispositivo móvel | Disney | concedida, viva até 2034 | ⛔ não — reivindica deformar por toque + uma «máquina de esmagar»; nada de rotação por pivô nem cadeia de IK |
| US 2024/0169635 A1 — animação facial anatómica | Unity | **abandonada** | ⛔ não — músculos, tensões e rede neural |
| US 2008/0309664 A1 — *Mesh Puppetry* | (Microsoft Research) | pedido; não avaliado a fundo | ⛔ não — optimização variacional de esqueleto + pesos; o pincel não optimiza |

⇒ Veredito: **prosseguir**. Sem «PATENTE VIVA» a reportar.

---

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E (alvo `blender-pose`)** despachado pela janela-mãe — a janela I da `line/sculpt3d` |
| session-id da janela-mãe | `9f820704-0d7e-4d96-847e-9cd720cbf178` (elo **I-3**, o mesmo do INC-4 da obra `blender-cloth`) |
| transcript do subagente (⛔ **zona contaminada** — I nunca lê) | ficheiros de subagente sob `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178*` |
| aberto em | 2026-09-13 |
| paralelismo | ⚠️ três outros subagentes-E trabalham **ao mesmo tempo** na mesma worktree (`blender-pull` · `blender-boundary` · `blender-unblocked`); este toca só os seus caminhos |
| lê | o fonte do pincel e o que ele invoca que decide comportamento, por shell (`cat`/`sed`/`rg`), porque o `deny` da linha nega `Read` |

### INBOX transcrito (append cego da janela I)

| data | conteúdo |
|---|---|
| 2026-09-13 | «I session: `9f820704-0d7e-4d96-847e-9cd720cbf178` 2026-09-13 — janela I da line/sculpt3d (a mesma do INC-4 classificado RELANCE); obra: pincel Pose» |

---

## Cobertura da travessia (§3.E)

⚠️ **Duas corridas.** A 1.ª (2026-09-13, ~19:29–19:56) montou triagem, patente, vassoura, oráculo e
o modelo de referência, e **morreu por limite de uso** antes de escrever a espec — **sem nada
comitado**. A 2.ª (mesma data, este agente) **refez a travessia do zero** (contexto novo: a
cobertura de uma corrida perdida não é cobertura), aproveitando os artefactos em disco (dumps,
notas, harness) como **dados**.

### Fonte lido, ficheiro a ficheiro

| área | ficheiro(s) | o que foi lido | data |
|---|---|---|---|
| **núcleo do pincel** | `…/sculpt_paint/mesh/sculpt_pose.cc` (2 263 L) | **INTEIRO** — solver da cadeia, os 4 escritores de cadeia, os 3 aplicadores por tipo de malha, o crescimento dos pesos (3 variantes), a origem+pesos do 1.º segmento (3 variantes), o desvio, a construção da cadeia por topologia, as 3 deformações, o referencial de pivô, o despacho | 2026-09-13 |
| idem, **cabeçalho** | `…/mesh/sculpt_pose.hh` (73 L) | INTEIRO — a forma do segmento e da cadeia | 2026-09-13 |
| idem, **fora de escopo** | as 6 rotinas de origem por conjuntos de faces (L 1034–1929) | **fronteiras e assinaturas lidas**; corpo **não** especificado (exclusão C). ⭐ Achado que importa ao escopo: o termo de correcção do deslocamento do arrasto é escrito **só** por essas rotinas ⇒ é **zero** no modo de topologia | 2026-09-13 |
| **simetria** | `…/mesh/sculpt.cc` L 665–690, 2 960–3 050; `…/paint_intern.hh` L 630–662 | teste de lado, área de simetria do vértice, espelhar ponto e rotação por área, validade da iteração, espelhar | 2026-09-13 |
| **travessia em largura** | `…/mesh/sculpt_flood_fill.hh` (inteiro) + `…/mesh/sculpt_flood_fill.cc` L 1–150 | fila FIFO, sementes, visitados, ligação artificial na vizinhança | 2026-09-13 |
| **vizinhança de vértice** | `…/mesh/sculpt.cc` L 554–583, 8 022–8 040 | construção por faces incidentes, os dois adjacentes por face, deduplicação | 2026-09-13 |
| **sementes espelhadas** | `…/mesh/sculpt_expand.cc` L 574–615 | busca do parceiro espelhado, limite de distância, **ordenação** da lista | 2026-09-13 |
| **suavização dos pesos** | `…/mesh/sculpt_smooth.cc` L 30–90, 745–800 | média de vizinhos (self excluído), a variante com guarda que **não** é a usada, e o laço por partição **em paralelo sobre o mesmo arranjo** | 2026-09-13 |
| **ligações entre peças** | `…/mesh/sculpt.cc` L 6 244–6 300, 6 356–6 420, 6 500–6 530 | ilhas, busca do par, emparelhamento mútuo, cache por distância, custo `O(n²)` | 2026-09-13 |
| **força / pressão / inversão** | `…/mesh/sculpt.cc` L 1 373–1 412, 2 316–2 475, 5 120 | pluma de simetria, factor de direcção, a linha do pincel de pose, origem do estado de inversão | 2026-09-13 |
| **deslocamento do arrasto** | `…/mesh/sculpt.cc` L 4 180–4 350 | âncora, projecção no plano de profundidade, **acumulação**, fixação do ponto de aplicação | 2026-09-13 |
| **raio em espaço de objecto** | `…/mesh/sculpt.cc` L 120–131, 5 860–5 905 | projecção do raio em pixels, fixação no 1.º evento | 2026-09-13 |
| **undo / rebase** | `…/mesh/sculpt.cc` L 5 169–5 215, 7 645–7 656, 7 737–7 790 | por que este pincel **não** repõe a malha; o rebase; travas de eixo e recorte de espelho | 2026-09-13 |
| **máscara / ocultação** | `…/mesh/sculpt.cc` L 7 202–7 240 | factor por vértice | 2026-09-13 |
| **âmbito de nós** | `…/mesh/sculpt.cc` L 920–940, 3 223–3 280 | o pincel declara que precisa da malha inteira, e usa posições originais | 2026-09-13 |
| **curva de atenuação** | `…/blenkernel/intern/brush.cc` L 1 608–1 675 | os 9 presets, e o argumento ser o **índice do segmento** | 2026-09-13 |
| **omissões e faixas** | `…/makesdna/DNA_brush_types.h` L 356–380; `…/makesrna/intern/rna_brush.cc` L 2 574–2 590, 3 334–3 404, 3 638–3 650 | omissões cruas e as faixas **públicas** dos 7 controlos | 2026-09-13 |

### História e literatura (§3.E, §4.1.12)

| fonte | o que foi percorrido |
|---|---|
| **commits** | log completo dos ficheiros do pincel (`notes/commits_*.txt`, 37 entradas + os dois ficheiros históricos) — incluindo o commit de origem do modo por conjuntos de faces, cuja mensagem dá a **motivação de produto** citada no §15 da espec |
| **issues** | **22** issues do rastreador público lidas (títulos + corpo + comentários, `notes/issue_*.json`); **8** entraram na espec §15 com link |
| **patente** | tabela no §Patente acima — a de Pixologic, **expirada**, é literatura livre |
| ⛔ **não lido de propósito** | revisões diferenciais (`D####`) — são **diffs**; ficam na denylist da espec |

---

## Oráculo (§5)

| | |
|---|---|
| **Binário** | `/usr/bin/blender` 5.2.1 LTS (2026-09-01). ⚠️ **patch-release acima do fonte lido** (5.2.0) — registado; o oráculo é o binário |
| **Harness** | `~/Referencias/blender-pose/oracle/harness.py` — API **pública** (`bpy`), malhas geradas por nós, traço entregue como lista de eventos, sessão gráfica **virtual** (`kwin_wayland --virtual`) para não tomar o ecrã do dono. ⛔ **Não** é uma build instrumentada: nada do alvo foi recompilado |
| **Por que uma janela real** | o pincel corre dentro do operador de traço, que precisa de uma região 3D viva; o modo sem interface não a tem |
| **Dumps** | `~/Referencias/blender-pose/oracle/out/` — **262** ficheiros `.npz`: 69 finais + 11 séries **por evento** (adendo D da missão) + corridas de custo e de repetibilidade |
| **Publicado** | `docs/3D/cleanroom/fixtures/pose/` — 69 + 11 + 3 malhas, **com as chaves renomeadas para vocabulário do domínio** e README de proveniência. `analise.json` é derivado |
| **Modelo de referência** | `~/Referencias/blender-pose/oracle/modelo.py` (+ `modelo_f32.py`, a variante que decide a região em `f32`) — **bancada, não produto** (§3.E permite). Fecha **54 de 69** fixtures a `≤ 1e-5`, a maioria em `~1e-7`; as 15 restantes têm mecanismo nomeado na espec §12.3 |
| **Bit-parity** | ⛔ **não prometida** — a cerca do [ADR-0162](../../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) fica de pé; a meta é a barra derivada da espec §12.3 |

### Experiências decisivas (o que tornou a espec confiável)

| pergunta | experiência | veredito |
|---|---|---|
| por que fixtures com o **mesmo** traço divergem conforme o raio? | contar os vértices cuja resposta a `distância < raio` **muda entre `f32` e `f64`**, e re-correr o modelo com o predicado em `f32` | ⭐ **1 vértice** basta: `5,65e-3 → 1,08e-7` e `1,52e-2 → 8,37e-8`, sem mexer em mais nada, e as fixtures de 0 flips ficam iguais. Espec §12.1 |
| a divergência é fragilidade ou desacordo? | perturbar o raio em `1e-6` | fixtures com erro `4e-7` saltam `1,5e-1` ⇒ **estar perto da descontinuidade não obriga a discordar** |
| o zero-deslocamento de uma fixture é bug do oráculo? | medir o comprimento do 1.º segmento | `4,8e-8` ⇒ **degenerescência real**, e a âncora cancela a translação. Espec §11.1 |
| a pose depende da taxa de eventos? | mesmo arrasto em 4 / 12 / 36 eventos | ⭐⭐ **1 segmento: idêntico ao bit; 3 segmentos: `5,3e-2`** ⇒ o solver **carrega estado**. O modelo reproduz `5,321e-2` **exactamente**. Espec §5.1-bis — ⚠️ **isto refutou uma frase que eu já tinha escrito na espec** («o gate de não depender da taxa de eventos») |
| a pressão entra na força? | comparar as duas fixtures | **idêntico ao bit** ⇒ não entra. Espec §1.3 |
| empate no vértice sob o cursor explica algum resíduo? | medir a folga ao 2.º vértice mais próximo | ⛔ **refutado** — o cursor cai **exactamente** sobre um vértice nas fixtures (`d₁ = 0`), não há empate |

---

## Corrente I

| janela | session-id | data | motivo | declaração |
|---|---|---|---|---|
| I-3 | `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-13 | despachou este E para a obra do Pose | ⏳ a janela declara pelo **inbox** |

---

## Papel R

*(vazio — R-pré ainda não despachado)*

---

## Espec

| versão | caminho | rascunho | filtragem §4.3 | sweep | data |
|---|---|---|---|---|---|
| 1 | `docs/3D/cleanroom/SPEC_pose_brush.md` | `~/Referencias/blender-pose/draft/SPEC_pose_brush.md` (regra do arquivo fechado, §4.1.11) | executada — cada número com proveniência (fórmula · dump · decisão nossa); nomes internos renomeados para vocabulário do domínio; zero trecho; as citações de prosa dos autores foram **re-ditas**, não transcritas | verde | 2026-09-13 |

⚠️ **Commit ÚNICO, pós-filtragem** (§3.E): a espec **não** entrou por rascunhos incrementais.

---

## Incidentes

*(vazio)*

---

## Fechamento R

*(vazio)*
