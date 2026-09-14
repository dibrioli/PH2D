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

| papel | id | data |
|---|---|---|
| R-pré | subagente R-pré despachado pela janela-mãe `9f820704-0d7e-4d96-847e-9cd720cbf178` (elo **I-3**) — **contexto novo**, ⛔ não é o subagente-E que escreveu a espec; **viu os dois lados** (fonte por shell, `Read` deny-listed para a sessão; transcript = zona contaminada) | 2026-09-13 — ⛔ **NÃO ATESTADA: 6 achados, 4 substanciais.** Veredictos abaixo |

### Auditoria R-pré — 2026-09-13 (espec v1)

**Sweep (§7.1):** `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-pose.txt
docs/3D/cleanroom/SPEC_pose_brush.md docs/3D/cleanroom/fixtures/pose` ⇒ **`✓ limpo`, 112 entradas,
exit 0.** ⚠️⚠️ **E é exactamente por isso que a auditoria humana existe: o sweep é cego a TRADUÇÃO.**
Cinco das 27 entradas de prosa da vassoura são frases de comentário do fonte que a espec carrega
**em português**, palavra a palavra — o grep nunca as veria. *Uma vassoura em inglês não varre uma
espec em português; ela só prova que ninguém colou.*

⚠️⚠️ **E o sweep sobre ESTE ledger dá `exit 1`, por CONSTRUÇÃO do protocolo — não é achado, é uma
tensão entre duas regras, e o R-PÓS tem de a saber:** o §6 **manda** o ledger registar os ficheiros
do fonte percorridos, e o §7.1 **manda** a vassoura conter os nomes de ficheiro do alvo ⇒ a secção
*Cobertura da travessia* casa consigo mesma. Conferido: o [`LEDGER_blender-cloth.md`](LEDGER_blender-cloth.md)
tem a **mesma** propriedade contra a vassoura dele. ⇒ a barra do §7.2 (*«zero hits sobre a árvore
inteira»*) **não é satisfazível** enquanto os ledgers viverem na árvore rastreada: ou o sweep de
fechamento exclui `docs/**/cleanroom/LEDGER_*.md` por escrito, ou a cobertura passa a ser registada
sem nomear ficheiro. ⛔ **Decisão de protocolo, não desta auditoria** — fica nomeada. A **espec
sozinha** é `exit 0`, que é a condição de entrega do §4.3.

**Conferido e limpo:** nomes de ficheiro das 85 fixtures · chaves do cabeçalho das fixtures (todas
em vocabulário nosso: `modo`/`origem`/`raio`/`segmentos`/`desvio_da_origem`/… — ⛔ nenhuma chave de
dump do alvo) · renomeações de domínio que estão CERTAS (o factor de pose → **peso**; os vizinhos
artificiais; a **franja**; a **semente**) · a decomposição em fases A–F (forçada pelo fluxo de
dados, não arbitrária) · §2.1 passo 3 (**a ordenação das sementes é REAL** — conferida no fonte,
`std::ranges::sort` no produtor da lista; a espec não a inventou) · §2.3 (a origem de recurso e a
inicialização dela em `C` estão exactas) · §3.1–§3.2 (re-descrição legítima: as duas regras de
paragem estão ditas por grandeza, não por prosa do alvo) · as **12 issues públicas** do §15
(§4.1.12 — é o canal permitido, e estão re-ditas).

#### Os 4 SUBSTANCIAIS (a espec carrega expressão do alvo)

| # | sítio | o que é | vassoura |
|---|---|---|---|
| **A1** | **§3**, 1.º parágrafo (a frase que explica por que a cadeia é forçada a um segmento) | **tradução frásica de um comentário do fonte, com a oração causal incluída.** O FACTO é livre; a frase é a expressão mais protegida do ficheiro (§4.2) | entrada **91** |
| **A2** | **§8**, abertura (a frase da passagem única de simetria + o «as outras devolvem imediatamente») | **tradução frásica de um comentário do fonte**, e a segunda metade descreve o **retorno antecipado** do código em vez do comportamento | entradas **95** e **96** |
| **A3** | **§5.1** (os 5 passos numerados) | **transcrição instrução-a-instrução do corpo de UMA função do alvo**: os mesmos intermediários, na mesma ordem, em notação de atribuição — §4.2 *«pseudo-código que espelha o original linha a linha é tradução, não descrição»*. O parêntese do passo 5 é, além disso, **tradução do comentário** que marca esse passo. ⚠️ O mesmo padrão, mais fraco, em **§2.2** (corpo do *callback* da varredura) e **§6** (corpo do laço das matrizes) | entrada **94** |
| **A4** | **§2.4** (*«os próprios autores o escrevem no código»*) e **§5.2** (*«confirmada pelo comentário dos autores»*) | a espec **atribui dois factos do produto a comentários do fonte**. Importa o conteúdo do comentário por referência **e** viola o §4.3 (a proveniência de um número é fórmula · medição · decisão nossa). Os dois factos têm proveniência lícita à mão: a constante é **medível** das fixtures (elas carregam `pixels_por_unidade`) e o custo quadrático é **afirmado pelas issues públicas** citadas no mesmo parágrafo | entradas **86** e **98** |

#### Os 2 menores (§4.2, higiene de vocabulário)

| # | sítio | o que é |
|---|---|---|
| **A5** | **§5.2** (tabela de presets) e **§1.2** (alvo da deformação) | os presets são nomeados pelas **grafias de identificador do enum do alvo** — duas delas (`LIN`, `POW4`) **não são palavras nem rótulos**: na interface pública o alvo escreve *Linear* e *Sharper*. ⚠️ E a casa **já decidiu este vocabulário** em [`falloff.rs`](../../../crates/ph2d-sculpt3d/src/falloff.rs), com a tabela das nove leis e os nomes de domínio; a espec importa a grafia interna e contradiz o catálogo que o produto já tem. Idem `«áreas de simetria»` no §6 (termo interno traduzido — as aspas na própria espec denunciam-no) |
| **A6** | **§7.3** (*«não está na lista dos que repõem a malha…»*) e **§13** (*«o pincel declara que precisa da malha toda»*) | dois requisitos enunciados pela **estrutura interna do alvo** (uma lista de despacho e um predicado que só existem no código dele), em vez de pelo comportamento. §4.2, organização |

#### Achados FUNCIONAIS (fora do §4.2 — devolvidos ao E na mesma passagem)

- **F1 — a espec descreve UMA semente; o alvo tem DUAS grandezas distintas.** A varredura da fase A
  é semeada pelo vértice que a **eleição do cursor** produziu (mais os espelhos dele dentro de `R`,
  e a lista é ordenada), enquanto o vértice achado por uma **consulta de mais-próximo global sobre
  as posições do início do traço** recebe peso `1` **antes** de a varredura começar. Nas fixturas
  comuns os dois coincidem e a diferença é invisível; ⚠️ eles **não são a mesma coisa por
  construção**. ⇒ a espec tem de dizer qual grandeza semeia e qual recebe o peso, e o E deve
  **medir** se alguma fixtura os separa — ⭐ é candidato nomeado para o resíduo `2,7e-3` que a §12.3
  deixou **«por explicar, item aberto para o R-pré»** (a fixtura é de UM segmento ancorado, onde só
  o pivô e os pesos decidem, e o desvio é **sistemático**, não descontínuo).
- **F2 — a tabela de presets do §5.2 tem NOVE linhas e o alvo oferece DEZ.** Falta o preset
  quadrático inverso, que o artista pode escolher e que só o modo de torção lê. ⛔ Nove das dez já
  estão escritas e medidas no nosso [`falloff.rs`](../../../crates/ph2d-sculpt3d/src/falloff.rs),
  a décima incluída — *a espec manda reconstruir uma tabela que o produto já tem, e com um buraco*.

#### O que a reescrita NÃO pode perder

⚠️ **Todos os FACTOS dos seis sítios são legítimos e caros** — a ordem de percurso da cadeia
(perto→longe), o que se mede contra o estado **inicial** e o que se mede contra o estado do evento
**anterior** (§5.1-bis, o achado mais valioso da espec), o epílogo da âncora, a lei de composição
das matrizes e o teto de um segmento nos dois modos de escala. A cura é **re-exprimir**, nunca
apagar: enunciar cada fase como **requisito fechado** (em que configuração cada segmento tem de
acabar, e qual é o alvo do seguinte) em vez de como sequência de atribuições, e dar a cada número
uma proveniência de **medição**. A paridade com as fixturas não muda uma casa decimal por isto.

---

### Auditoria R-pré — 2026-09-13 (espec v2, **2.ª passagem**)

| papel | id | data |
|---|---|---|
| R-pré (2.ª passagem) | subagente R-pré **novo**, despachado pela janela-mãe `9f820704-0d7e-4d96-847e-9cd720cbf178` (elo **I-3**) — ⛔ **não** é o subagente-E que escreveu a v1 **nem** o R-pré que a reprovou; contexto novo; viu os dois lados (fonte por shell, `Read` deny-listed) | 2026-09-13 — ⛔ **NÃO ATESTA: 5 achados, ZERO substanciais** |

**Adendo A conferido por shell:** o fonte vive fora de qualquer árvore do PH2D, tag `v5.2.0` /
`fbe6228777e7d9afefcd61a413844e790ae75db7`; oráculo `/usr/bin/blender` = **5.2.1 LTS** (build
2026-09-01). Confere com o cabeçalho da espec.

#### ⭐ O CONTROLO POSITIVO DO SWEEP FOI CORRIDO POR TERCEIRO — e ele de facto acusa

⚠️ *Um instrumento cujo controlo positivo foi corrido só pelo autor dele continua por provar.* Corrido
aqui, **sem escrever a v1 em ficheiro nenhum** (⛔ nada do alvo toca `/tmp`): a vassoura de **137**
entradas contra o **histórico** do ficheiro da espec
(`cleanroom-sweep.sh <vassoura> --git-history docs/3D/cleanroom/SPEC_pose_brush.md`) ⇒ **`exit 1`,
`5` linhas distintas acusadas**, cada uma num dos cinco sítios que a 1.ª passagem nomeou (§2.4 · §3 ·
§5.1 · §5.2 · §8), impressas nos dois sentidos do diff (`-` da emenda e `+` da v1). A mesma vassoura
sobre o **HEAD** (espec + as 85 fixturas) ⇒ **`exit 0`, limpo**. ⇒ o alargamento `112 → 137`
**discrimina**, e o verde da v2 vale mais do que o verde da v1 valia.
⚠️ **O que o controlo NÃO prova:** nenhum dos 5 achados desta passagem é apanhado pela vassoura — os
quatro do §4.2 são *descrição de programa* e *tradução de uma oração curta*, formas que nenhum grep
tem como ver. *O sweep verde continua a ser condição necessária e nunca suficiente.*

#### As 6 curas da v1 — **confirmadas uma a uma contra o fonte**

| # (v1) | sítio | veredicto |
|---|---|---|
| **A1** | §3 | ✅ a oração causal desapareceu; ficou o **requisito** com proveniência de **fixtura** (duas que pedem `3` segmentos e entregam `1`) |
| **A2** | §8 | ✅ enunciado como **requisito observável** + o efeito («as passagens seguintes não produzem deslocamento»); a descrição do retorno antecipado **saiu** |
| **A3** | §5.1 | ✅ **e a cura é substantiva, não cosmética:** a sequência de atribuições virou **tabela de estado exigido por segmento**, e a origem está em **forma fechada** — uma linha de álgebra onde o alvo encadeia duas operações. ⭐ A re-expressão **produziu um facto novo e correcto** que a lista de passos escondia (cabeça e origem guardadas não distam o comprimento). Mesmo tratamento conferido em **§2.2** (conjuntos) e **§6**, cuja **lei algébrica eu re-derivei e bate** com a composição que o alvo monta |
| **A4** | §2.4 · §5.2 | ✅ custo re-atribuído às **issues públicas**; a constante angular passou a ter proveniência de **medição da saída do oráculo** (`46` vértices, duas fixturas a duas forças, dispersão `1,8e-6`) |
| **A5** | §5.2 · §1.2 · §6 | ✅ vocabulário da casa; conferido contra [`falloff.rs`](../../../crates/ph2d-sculpt3d/src/falloff.rs) — as **dez** leis, com as fórmulas certas (F2 fechado) |
| **A6** | §7.3 · §13 | ✅ os dois re-enunciados pelo observável |
| **F1** | §2.1 | ✅ **e verifiquei as DUAS grandezas no fonte**: elas são mesmo distintas (uma semeia a varredura; a outra, achada por busca **sem limite de distância**, recebe peso `1` **antes** de a varredura começar) e a espec descreve-as certo, com a medição de inércia (`69/69`) e a fixtura em falta **nomeada** |

#### Os 5 achados desta passagem — **nenhum substancial**

⚠️ **Os quatro primeiros são a MESMA ESPÉCIE que a 1.ª passagem curou (A6 e A1/A2), sobreviventes em
secções que a emenda NÃO tocou** — §4, §5.4 e §7.1 não têm um único *hunk* no diff `v1 → v2`.
*Uma cura que varre os sítios NOMEADOS deixa a espécie viva nos sítios que ninguém nomeou.*

| # | sítio | espécie | o que é |
|---|---|---|---|
| **B1** | **§4**, o parêntesis sobre a variante com guarda | §4.2 organização (= A6) | afirma o **inventário de código** do alvo (que existe uma rotina irmã e que este caminho não a chama). **Zero** conteúdo observável; apagá-lo não perde nada — o comportamento já está dito («média de um conjunto vazio») e a nossa divergência já está prescrita no §11.4 |
| **B2** | **§4**, o parágrafo do determinismo | §4.2 organização (= A6) **+ §4.3** | descreve a **estrutura de execução** do alvo (partição, paralelismo, escrita no arranjo que lê) em vez do observável, e é a única afirmação de mecanismo da espec **sem proveniência** — vem de leitura de fonte, não de medição. A metade medida (as 69 fixturas cabem; a paridade fecha a `~1e-7`) é legítima e fica |
| **B3** | **§7.1** | §4.2 organização **+ defeito de legibilidade** | a soma é escrita com dois símbolos **que a espec nunca define em lado nenhum**, e que são exactamente a decomposição de **armazenamento** do alvo (duas matrizes por segmento-e-octante, mais uma inversa) — a mesma que o §6 já tinha dissolvido numa lei. Um implementador não consegue resolver a notação, e ao tentar reconstrói a escolha de armazenamento do alvo |
| **B4** | **§5.4**, a oração final do passo 1 | §4.2 *wording* (= A1/A2) | **tradução quase 1:1 da oração de finalidade** de um comentário do alvo. É curta e o resto da frase é nosso, logo o peso é pequeno — nomeio-a porque é **precisamente a espécie que esta emenda existiu para curar**, e porque há forma alternativa melhor à mão: o observável (com a trava desligada o gesto **roda e escala**; com ela ligada, escala **sem rodar**), que é o que o controlo do §1.1 já implica |
| **B5** | **§2.2** / **§2.5** | ⚠️ **funcional**, não §4.2 | o teste de lado é um predicado de **dois** argumentos e a espec só **liga** o segundo num dos dois sítios que o usam: o §3.1 diz «contra o alvo corrente» (✅ correcto, conferido), o §2.2 não diz nada. **Conferido no fonte: na acumulação da franja o segundo argumento é o ponto do cursor `C`.** ⛔ Não é cosmético — a franja decide o pivô e o pivô decide a deformação inteira (§12.1), e o §11.3 **só é verdade** com essa ligação, que ele pressupõe sem a enunciar |

#### O que a 3.ª passagem custa

⭐ **Quatro edições de uma linha e uma cláusula acrescentada.** Nenhum achado pede medição nova,
re-derivação, nem toca numa fixtura; a paridade não muda uma casa decimal. ⚠️ E **nenhum** deles move
o sweep (ver acima) ⇒ o verde do sweep não é prova de que foram curados: a 3.ª passagem confere-os
**a olho**, como esta.

#### Interacção nomeada (⛔ não é achado desta espec)

A espec manda o implementador buscar o vocabulário das curvas ao
[`falloff.rs`](../../../crates/ph2d-sculpt3d/src/falloff.rs) — o que está **certo** (§4.1: é o
catálogo que o produto já tem). ⚠️ Só que esse ficheiro carrega, numa tabela de doc-comment, as
**grafias internas de identificador** do alvo. É a dívida que o `CLAUDE.md` §5 já declara **ABERTA e
nomeada** («os nomes de SÍMBOLO internos são §4.2 pela mesma linha da SKILL e o gate não os mede»),
e é da linha dona daquele ficheiro — não desta obra. Fica registada porque **esta espec é o que põe
um implementador a abrir aquele ficheiro**.

---

### Auditoria R-pré — 2026-09-13 (espec v3, **3.ª passagem**)

| papel | id | data |
|---|---|---|
| R-pré (3.ª passagem) | subagente R-pré **novo**, despachado pela janela-mãe `9f820704-0d7e-4d96-847e-9cd720cbf178` (elo **I-3**) — ⛔ não é o E nem nenhum dos dois R-pré anteriores; contexto novo; viu os dois lados (fonte por shell, `Read` deny-listed) | 2026-09-13 — ⛔ **NÃO ATESTA: 7 achados, 1 SUBSTANCIAL** |

**Sweep (§7.1):** `✓ limpo`, **137** entradas, `exit 0` sobre a espec + as 85 fixturas.
⚠️ **Pela terceira vez, nenhum dos achados move a vassoura** — e desta vez nem por tradução: os dois
que importam são *factos correctos mal enquadrados*. O sweep continua necessário e nunca suficiente.

#### ⛔ SUBSTANCIAL — a §5.1 manda preservar uma grandeza SEM CONSUMIDOR, e a consequência que ela afirma é FALSA

A álgebra está **certa** (a separação entre cabeça e origem vale `|2·comprimento − ‖T − O⁻‖|`,
re-derivada aqui). O que está errado é a frase seguinte, que a 2.ª passagem elogiou como *«um facto
novo e correcto»*: **não é verdade que a inconsistência chegue a pixel num dos modos.** Conferido no
fonte, dos dois lados:

- o **único** modo que lê a cabeça (o referencial do espremer/esticar, §6) **não resolve a cadeia** —
  ele só toma emprestado o quociente de escala; ali cabeça e origem são as **iniciais**, sempre;
- os modos que **resolvem** a cadeia (girar; escalar com a trava de rotação desligada) **nunca leem**
  a cabeça — o referencial deles é a identidade.

⇒ a cabeça, como a §5.1 a escreve, é **só-escrita**: nenhuma das 69 fixturas pode distinguir uma
implementação que a "corrija", e a espec não oferece nenhuma. A frase *«diverge nesse modo e em mais
nenhum»* prescreve reproduzir um artefacto interno do alvo por uma razão que não existe — que é a
espécie deste documento (descrever o programa) vestida de facto.
⚠️ **E há um caminho que a torna verdadeira, e é o caminho ERRADO:** a §5.5 diz apenas que o
quociente é *«o mesmo de §5.4»*, e o §5.4 abre com o passo de resolver a cadeia. Quem importar esse
passo passa a escrever a cabeça **e** a ler — e aí diverge do alvo. A cerca está hoje num parêntesis
da §5.1-bis, três secções à frente.
⭐ **Lição de método:** *verificar a ÁLGEBRA de um facto não é verificar que ele tem CONSUMIDOR.*

#### ⛔ A espécie ainda vive — §7.2 é a decomposição de ARMAZENAMENTO do alvo, com um símbolo que a espec nunca define

O símbolo da 1.ª coluna da tabela do §7.2 aparece **uma só vez no documento inteiro** (conferido por
grep): no cabeçalho dessa tabela. O que as duas colunas repartem é exactamente o par de matrizes por
segmento-e-octante que o alvo guarda — ⇒ é o **achado B3 da 2.ª passagem, vivo uma secção adiante**,
numa secção que **nenhuma das duas emendas tocou** (zero *hunks* em `v1 → HEAD`). Agrava:

- **contradiz a frase que a emenda 2 acabou de acrescentar ao §7.1** (*a espec não prescreve como o
  mapa é armazenado nem factorizado*);
- é **redundante**: a tabela de duas linhas do §6 já diz a mesma coisa como lei.

Endereços secundários da mesma espécie: §10 (*«reconstroem-se as 8×n matrizes»* — contagem de
armazenamento onde o §6 dá um mapa), §13 (*«o alvo já o faz»*), §7.3 no **título** (o requisito
enunciado por um passo de código **ausente**, com as aspas que a A5 já tinha apanhado como sinal — o
corpo dessa secção **foi** curado, o título não), e a cauda do 1.º ponto do §1.3 (o inventário dos
factores que *outros* pincéis do alvo aplicam; o observável é a 1.ª metade da frase).

#### Proveniência (§4.3 · §4.1.12) — 4 sítios, todos de uma linha

| # | sítio | o que falta |
|---|---|---|
| **C1** | **§1.3**, título | promete *«(medido — a ausência é o facto)»* para **cinco** ausências e o corpus mede **uma** (a pressão, com fixtura e número). Dureza · direcção *Add/Subtract* · forma de atenuação **não têm eixo nenhum** na tabela de cobertura do §14. Ou se medem (o arnês existe) ou se diz quais são medidas e quais são afirmadas — o §17 só gateia a medida, que é a metade honesta |
| **C2** | **§15**, parágrafo final | atribui uma motivação de produto **aos autores** sem nomear o canal, enquanto **todas** as outras linhas da secção trazem o link. O canal é lícito (§4.1.12) — falta dizê-lo |
| **C3** | **§16**, 1.º ponto | *«(verificado)»* não é fórmula, nem medição, nem decisão nossa. O facto é **verdadeiro** (conferido), e o observável já existe: o corpus fecha a `~1e-7` contra uma lei que não tem esse termo |
| **C4** | **§4**, determinismo | a metade *abaixo* do limiar está citada; a metade *acima* (o alvo não é reprodutível) não cita as corridas de **repetibilidade** que o oráculo produziu e que este ledger regista |

#### Funcional — 2 referências penduradas

⚠️ **§2.3** fala da *«média do passo 4»* e **§12.1** do *«teste do §2.2 passo 3»* — e o §2.2 **não tem
passos numerados** desde a emenda 1, que os substituiu pelos três conjuntos precisamente para matar a
forma espelhada. As duas referências são **irresolúveis** para quem implementa e são o **fóssil da
decomposição que foi removida**. Uma palavra cada.

#### ⛔ Conferido e LIMPO — não reabrir

- **§2.2 / §2.5, o 2.º argumento do teste de lado** (adendo C da missão): **conferido no fonte — é o
  ponto de aplicação**, e entrou **legitimamente**: é o algoritmo (§4.1.11), e a consequência dele
  tem fixtura com número (§11.3). ✅
- **§3.1** em forma de atribuições: conferido contra o fonte — é a **matemática** de uma propagação de
  máximo por Jacobi com o teste de lado sobre os recém-alcançados, não um espelho com intermediários
  removíveis; não há forma fechada a preferir. Deixar como está.
- **§6**: a regra do octante e a regra das **duas** inversões re-derivadas e **exactas**.
- **§2.3** (origem de recurso), **§3.2** (as duas regras de paragem, o retrocesso dos pesos e o
  *«`A` da última varredura, pesos da anterior»*), **§5.4** (o plano pela cabeça inicial), **§5.5**
  (a guarda), **§3** (o tecto de um segmento), **§5.2** (a constante, medida): **exactos**.

#### A leitura de método desta passagem

> ⛔⛔ **A varredura da espécie acerta onde a espécie tem NOME e falha onde ela veste a roupa de um
> FACTO.** A emenda 2 fez o que a 2.ª passagem mandou — varreu o documento inteiro e apanhou mais
> seis por conta própria — e mesmo assim ficaram as **duas** formas que não se parecem com
> «descrição de programa»: uma **tabela de factos correctos cuja ORGANIZAÇÃO é a do armazenamento do
> alvo**, e uma **álgebra correcta com uma consequência inventada**.
> ⇒ o teste que falta não é *«isto descreve o programa?»* — é **«que fixtura reprova se eu escrever
> o CONTRÁRIO disto?»**. As duas sobreviventes não têm nenhuma, e por isso passaram três filtragens.

---

### Auditoria R-pré — 2026-09-13 (espec v4, **4.ª passagem**)

| papel | id | data |
|---|---|---|
| R-pré (4.ª passagem) | subagente R-pré **novo**, despachado pela janela-mãe `9f820704-0d7e-4d96-847e-9cd720cbf178` (elo **I-3**) — ⛔ não é o E nem nenhum dos três R-pré anteriores; contexto novo; viu os dois lados (fonte por shell, `Read` deny-listed) | 2026-09-13 — ⛔ **NÃO ATESTA: 5 achados, ZERO substanciais** |

**Sweep (§7.1):** `✓ limpo`, **137** entradas, `exit 0` sobre a espec + as 85 fixturas.
⭐ **Controlo positivo re-corrido por este R, não herdado:** a mesma vassoura sobre a **v1** da espec
(`git show <v1>:…`) acusa **5** achados de conteúdo. ⇒ o verde do HEAD é um verde **medido**, não a
ausência de instrumento. ⚠️ Pela **quarta** vez, nenhum achado desta passagem move a vassoura: a
parede está intacta e **o instrumento não é o que falta**.

#### ⭐ O que está CONFERIDO e LIMPO — não reabrir

- **As 7 curas da emenda 3, uma a uma contra o diff** (`git show` do commit da emenda): a afirmação e
  a prescrição da cabeça **apagadas** e substituídas pela medição de inércia · a §5.5 com a cláusula
  nova (não resolve a cadeia; a trava de rotação não tem papel) · o §6 a nomear a direcção **inicial**
  · a secção de decomposição de armazenamento **apagada** com os quatro endereços secundários
  reescritos e a renumeração **completa** (⭐ conferido mecanicamente: **todas** as `§N.M` citadas no
  documento resolvem para um título existente — zero referências penduradas) · o §1.3 em duas colunas
  · o §15 e o §16 com a proveniência trocada pelo observável · o §4 a citar as corridas de
  repetibilidade. **7 de 7.** ✅
- **§4.2 — zero violações.** Varri o documento por identificador de código, grafia interna, prosa do
  alvo e pseudo-código espelhado: o vocabulário é de domínio de ponta a ponta, e as únicas cadeias
  ASCII são as **nossas** (símbolos de álgebra, nomes de fixtura, `f32`/`f64`/`NaN`). ✅
- **A tabela das nove leis de atenuação** é a **nossa** (confirmado contra o catálogo do produto: o
  `ALL` da casa tem `12` e a conta `9 + 1 + 2` fecha), e a identidade algébrica que a espec declara
  entre a forma dela e a da casa **verifica-se**. ✅
- **Proveniência das fixturas (§5):** malhas de entrada **nossas**, geradas pelo harness; nenhum asset
  do alvo; estatuto legal citado ao texto da licença; vocabulário renomeado; o sweep corre sobre a
  pasta. ✅
- **O corpus bate o que o §14 promete** — medido, não lido: `69` finais + `11` por evento em disco, e
  **todos os seis modos** têm fixtura. ⭐ **A minha 1.ª suspeita caiu na medição:** por nome só há UMA
  fixtura de suavização, mas o censo dos cabeçalhos dá **três** valores distintos do controlo de
  suavizações (`0`, `4`, `10`) — as fixturas variam-no **sem o dizer no nome**. *Inferir cobertura do
  nome do ficheiro é a armadilha; o instrumento é o cabeçalho.*
- **A §12.4 é HONESTA em todas as linhas que o instrumento alcança** (adendo B): as três ausências que
  ela declara **sem fixtura** não têm sequer chave no cabeçalho de nenhuma das 69 ⇒ o corpus de facto
  nunca as varia; as duas **medidas como inertes** têm o número; a linha das travas de eixo e do
  recorte de espelho idem. ✅

#### ⛔ A ESPÉCIE JÁ CURADA ESTÁ VIVA EM DOIS TÍTULOS QUE NENHUMA EMENDA TOCOU

A emenda 3 curou o título do §1.3 (*prometia «medido» para cinco ausências e o corpus media uma*).
**A mesma redacção sobrevive, verbatim desde a v1, em dois outros sítios** — conferido com
`git show <v1>` (as duas linhas são **idênticas** na v1 e no HEAD):

| # | sítio | o que a medição diz |
|---|---|---|
| **A1** | **§8**, a linha que abre as consequências da simetria | promete que **as três** são medidas. A terceira é a simetria radial ser ignorada — e ela é **sourced a uma issue**, declarada fronteira que **não copiamos**. ⛔ **Zero** fixturas radiais, e **nenhuma** das 69 tem sequer uma chave de simetria radial no cabeçalho; o §14 não tem esse eixo |
| **A2** | **§11**, o **título** da secção | promete *«todas medidas»*. A §11.4 **não tem fixtura** — e ⛔ **a própria §12.4, três secções adiante, diz isso por escrito**: o documento contradiz-se a si mesmo entre um título e o censo que a emenda 3 acabou de escrever. A §11.5 é igualmente sourced a uma issue |

⚠️⚠️ **A leitura de método desta passagem, e é a terceira vez seguida:** *a emenda cura o ENDEREÇO
que o R nomeou e não varre a REDACÇÃO.* A emenda 1 curou instâncias e a 2.ª passagem achou a espécie
viva noutro sítio; a emenda 2 varreu e curou mais seis, e a 3.ª passagem achou-a **uma secção
adiante**; a emenda 3 curou um título e a redacção **idêntica** ficou em dois títulos vizinhos.
⇒ a cura desta vez é um `grep` pela **forma da promessa** (*«medido/medidas/observável»* sem fixtura
ao lado), não por endereço.

#### ⛔ Proveniência CIRCULAR numa afirmação que decide um default de produto

**§1.4** justifica *«o artista depara-se com o controlo de ancoragem LIGADO»* — e portanto a
recomendação de **nascer ancorado** — com *«observado no dump de configuração de todas as 69
fixturas»*. Duas coisas, as duas medidas:

1. ⛔ **O número está errado:** o controlo de ancoragem vale ligado em **`64` de `69`** (cinco estão
   desligadas) e o da trava de rotação vale desligado em **`68` de `69`**. A espec **nomeia ela
   própria**, duas linhas abaixo, uma das cinco excepções como *«o lado desligado»*.
2. ⛔⛔ **E o instrumento é circular:** o README das fixturas diz, por escrito, que o harness
   **reescreve todos os parâmetros** para os valores do cabeçalho de cada fixtura ⇒ **o cabeçalho é a
   ENTRADA do harness, não uma observação do que o alvo traz de origem**. O que a espec quer afirmar é
   sobre a biblioteca de pincéis distribuída com o alvo, e o ledger não regista nenhum dump dessa
   biblioteca — só os dumps de saída.
⇒ a afirmação pode muito bem ser **verdadeira**; o que ela não tem é instrumento. Ou se nomeia o
canal real, ou se diz que o valor é o default do **harness** e que a recomendação é nossa.
⚠️ **Não bloqueia** porque a decisão está declarada como sendo de produto e o corpus tem fixtura dos
**dois** lados — seja qual for a escolha, ela é gateável.

#### ⛔ A §12.4 tem um BURACO, e ele é da espécie que a emenda 3 acabou de nomear

**§7.2** enuncia como *«requisito … enunciado pelo que se observa»* um facto sobre o **programa** do
alvo (a ausência de um passo de reposição da malha entre eventos) e rotula de ***Observável*** uma
proposição que **não o discrimina**: o observável citado refuta *«o traço acumula»*, que é outra
afirmação. As duas maneiras de lá chegar — subtrair a deriva, ou repor e aplicar — dão, **em
aritmética exacta, o mesmo ponto**; o §17 transforma na mesma o mecanismo num item de lista com a
forma de uma escolha exclusiva.
⭐ **E há uma razão legítima para a preferência, que a espec não dá:** as duas formas diferem no
**arredondamento de `f32`**, e a própria barra do documento vive a `~1e-7` com tecto a `1e-6` — ou
seja, o mecanismo *pode* valer alguns ULP de paridade. ⇒ a cura não é apagar a preferência: é
**enunciar a lei** (a pose final é a posição de início mais o deslocamento; o traço não acumula),
**declarar o mecanismo livre até ao arredondamento**, e dar-lhe a razão certa se a razão for a
paridade. Mais **uma linha na §12.4**, que hoje não o lista.

#### ⛔ O determinismo do alvo continua escrito como FACTO em dois sítios (adendo C)

A emenda 3 acrescentou o parágrafo que diz — correctamente — que o regime de não-reprodutibilidade
**nunca foi observado** (`4×3` corridas, até `66 049` vértices, idênticas ao bit) e que a frase é um
**risco nomeado pelo mecanismo**. ⛔ **Mas as frases à volta não foram reconciliadas:** a que abre a
§4 continua a chamar-lhe *«o achado»* e a afirmá-lo a negrito **antes** da correcção; a que fecha a
mesma secção fala do *«regime em que o alvo não o é»*; e a linha de abertura da §12 afirma-o **sem
nenhuma reserva**, que é onde um implementador o vai ler isolado.
⇒ três sítios dizem facto, um diz risco. *Uma cláusula corrigida pelo parágrafo seguinte é
exactamente a forma que este repo já pagou noutra secção.* Cura: a reserva viaja com a afirmação,
nos três sítios.

#### Higiene (trivial, uma linha)

- **§17**: os itens estão numerados `10 · 10a · 10c · 10b · 11` — o `10c` foi inserido **antes** do
  `10b`. Uma lista que se percorre a marcar caixas não deve ter a ordem trocada.

#### ⚠️ Para o R-PÓS, achado de PROTOCOLO (não é achado da espec)

O comando da missão varre a **espec + as fixturas** e fecha **verde**. Corri-o também sobre **este
ledger** e ele acusa **três** linhas — **todas pré-existentes, nenhuma do texto desta passagem**
(conferido: o sweep sobre o meu diff isolado dá `exit 0`). Duas delas são a **tabela de cobertura da
travessia**, que o §6 da SKILL **obriga** o ledger a ter, e a terceira é um achado de R que cita a
redacção antiga da espec.
⇒ ⛔ **O §7.2 da SKILL põe a barra do fechamento em *«zero hits sobre a árvore inteira»*, e o ledger
vive na árvore rastreada.** As duas exigências não podem ser satisfeitas ao mesmo tempo enquanto a
cobertura for registada por nome de ficheiro do alvo. *Isto não bloqueia esta espec e não é defeito
dela* — é uma decisão que o **R-pós** tem de tomar e declarar: ou a cobertura passa a ser registada
sem os nomes (por área, ou por `sha256`, como o §6 já manda para trechos), ou o fechamento declara o
ledger fora do censo **por escrito e com o motivo**. ⚠️ Uma barra que ninguém pode cumprir é uma
barra que se afrouxa em silêncio no dia do fechamento.

#### O que esta passagem NÃO pede

⭐ **Nenhum dos cinco pede medição nova** — quatro são uma edição de uma linha e o do §7.2 são duas
linhas mais uma linha de tabela. **Nenhum toca uma fixtura.** E ⛔ **nenhum é §4.2**: a parede está
intacta, o sweep tem controlo positivo, e a espec **não carrega expressão do alvo**. O que estas
cinco medem é a distância entre *«a espec diz»* e *«o corpus prova»* — que é exactamente o eixo que a
emenda 3 instrumentou e ainda não varreu até ao fim.

---

## Espec

| versão | caminho | rascunho | filtragem §4.3 | sweep | data |
|---|---|---|---|---|---|
| 1 | `docs/3D/cleanroom/SPEC_pose_brush.md` | `~/Referencias/blender-pose/draft/SPEC_pose_brush.md` (regra do arquivo fechado, §4.1.11) | executada — cada número com proveniência; nomes internos renomeados | verde (⚠️ **mas ver abaixo: o sweep era cego**) | 2026-09-13 |
| **2 — EMENDA** | idem | idem | **reescrita pela regra do arquivo fechado**, com o fonte fechado; o fonte só foi reaberto para confirmar **dois factos** (as duas sementes do §2.1 · as fixturas que exercitam o tecto de um segmento) | verde, **com controlo positivo** | 2026-09-13 |
| **3 — EMENDA 2** | idem | idem | os 5 achados de higiene aplicados **e a espécie varrida em todo o documento**, não só nos cinco endereços | verde, com controlo positivo | 2026-09-13 |
| **4 — EMENDA 3** | idem | idem | os 7 achados aplicados + a pergunta *«que fixtura reprova o CONTRÁRIO disto?»* corrida sobre o documento inteiro ⇒ nasce o censo **§12.4** (`6` linhas) | verde, com controlo positivo | 2026-09-13 |
| **5 — EMENDA 4** | idem | idem | os 5 achados aplicados + varredura pela **FORMA DA PROMESSA** (`medido/medidas/observável`) em todo o documento ⇒ o censo **§12.4** passa a `10` linhas, uma delas achada pela própria varredura | verde, com controlo positivo | 2026-09-13 |

⚠️ **Commit ÚNICO, pós-filtragem** (§3.E): a espec **não** entrou por rascunhos incrementais.

### ⛔⛔ A emenda: o R-pré NÃO atestou a versão 1

Seis achados, **quatro substanciais** — a espec carregava expressão do alvo. Curados assim:

| # | achado | cura |
|---|---|---|
| 1 | §3 — a frase que justificava o tecto de um segmento era tradução frásica de prosa do alvo | enunciado só o **requisito**, com a proveniência a ser duas fixturas que pedem `3` segmentos e entregam `1` |
| 2 | §8 — tradução de prosa, e a 2.ª metade descrevia o **retorno antecipado** do código | enunciado o **efeito observável** |
| 3 | §5.1 — os cinco passos numerados eram transcrição instrução-a-instrução (mesmos intermediários, mesma ordem, notação de atribuição) | reescrito como **requisito fechado por segmento** (tabela grandeza → valor exigido → o que é), preservando o §5.1-bis e o epílogo da âncora. ⭐ A reescrita **expôs um facto que a lista de passos escondia**: cabeça e origem guardadas não ficam à distância do comprimento (são medidas de origens diferentes), e isso só é observável no modo espremer/esticar. Mesmo tratamento em **§2.2** (conjuntos, não passos) e **§6** (**lei algébrica** da composição, não sequência de construção) |
| 4 | §2.4 e §5.2 — dois factos atribuídos a **comentários do fonte** | re-atribuídos: o custo quadrático às issues públicas já citadas; ⭐ e a constante angular **medida da saída do oráculo** (ver abaixo) |
| 5 | §5.2/§1.2/§6 — grafias de identificador de enum e um termo entre aspas | vocabulário da casa (`falloff.rs`); os oito sectores passam a **octantes de espelho** |
| 6 | §7.3/§13 — dois requisitos enunciados pela estrutura interna do alvo | re-enunciados pelo que se observa |

### ⛔⛔⛔ ACHADO DE INSTRUMENTO: o sweep estava a provar ausência de COLAGEM, não FILTRAGEM

A versão 1 passou o sweep **verde** carregando quatro traduções — porque a vassoura estava em
**inglês** e a espec é escrita em **português**. *Uma vassoura que só apanha colagem não prova
filtragem nenhuma.*

⇒ A `VASSOURA_blender-pose.txt` passou a cobrir **as duas línguas**: `112 → 137` entradas, as `25`
novas sendo a prosa do alvo **na forma que uma tradução produziria**, com acentuação correcta.

⚠️⚠️ **E a 1.ª tentativa de a alargar foi ela própria um instrumento cego:** `29` entradas escritas
**sem acentos** e em frases longas deram **`0` achados** sobre a espec defeituosa — *um filtro que
casa zero lê-se exactamente como um filtro que aprova*. Duas causas, as duas medidas: sem acentos
nada casa, e o sweep casa **por linha** enquanto a espec é *hard-wrapped*, logo toda frase que
atravessa uma quebra é invisível. ⇒ as entradas boas são **fragmentos curtos, acentuados, que cabem
numa linha**.

⭐ **Controlo positivo, e é ele que torna isto um gate:** a vassoura alargada, corrida sobre a
**versão 1** da espec (recuperada de `git show`), acusa **5 achados — exactamente os 5 sítios que o
R-pré nomeou** (§2.4, §3, §5.1, §5.2, §8); corrida sobre a versão 2, sai **limpa**.
*Sem esse controlo, o verde da versão 2 não valeria mais do que o verde da versão 1.*

### ⛔⛔ EMENDA 2 — a 2.ª passagem do R-pré também NÃO atestou (5 achados, ZERO substanciais)

As **seis** curas da emenda 1 foram conferidas uma a uma contra o fonte e **confirmadas**. O que
sobrou foi higiene do §4.2 — quatro edições de uma linha e uma cláusula — mas ⚠️⚠️ **a leitura que
importa é sobre o MÉTODO:**

> ⛔⛔ **Uma cura que varre os SÍTIOS NOMEADOS deixa a ESPÉCIE viva nos que ninguém nomeou.**
> Os quatro achados de §4.2 da 2.ª passagem eram **a mesma espécie** que a emenda 1 curou —
> *descrever o PROGRAMA em vez do COMPORTAMENTO* — vivos em **três secções que a emenda não tocou**
> (zero hunks no diff dela). ⇒ ao aplicar um lote de achados, **varra o documento inteiro à procura
> da espécie**, não só dos endereços.

⭐ **E a varredura da espécie pagou:** além dos 5 endereços do R, ela apanhou **mais 6** por conta
própria (o modo sem ramo de inversão · o valor «cru» da estrutura de dados · a rotina irmã citada
pelo grafo de chamadas · «cabeça e origem **guardadas**» · o vértice solto «sem guarda» · o
«retorno silencioso»), e **duas** referências a *ramo* que eram do nosso próprio texto e liam-se
como código do alvo.

| # | achado do R | cura |
|---|---|---|
| 1 | §4 — parêntesis a afirmar o inventário de código (existe uma variante alternativa; este caminho não a usa) | **apagado** — não tem conteúdo observável; a degenerescência já está na frase anterior e a nossa divergência no §11.4 |
| 2 | §4 — o parágrafo do determinismo descrevia a **estrutura de execução** | reescrito como observável (*«a saída não é reprodutível acima de uma partição ⇒ a paridade não é asserível nesse regime»*) + o Jacobi limpo declarado **DECISÃO NOSSA**. A metade medida fica |
| 3 | §7.1 — a soma usava dois símbolos nunca definidos, que eram a decomposição de **armazenamento** do alvo | um símbolo só, o mapa que o §6 define; a factorização fica **ao implementador**, e a espec di-lo |
| 4 | §5.4 — oração de **finalidade** no passo 1 | trocada pelo observável (*«desligada roda e escala; ligada, escala sem rodar»*) |
| 5 | **funcional** — o teste de lado é um predicado de **dois** argumentos e a espec só ligava o segundo num dos dois sítios | §2.5 ganha a **tabela dos dois usos** (`C` no §2.2 · alvo corrente no §3.1) e o §2.2 nomeia-o. ⚠️ Não é cosmético: a franja decide o pivô, o pivô decide a deformação (§12.1), e o §11.3 **pressupunha** essa ligação sem a enunciar |

⚠️⚠️ **E a lição de instrumento desta obra, que fica escrita:**

> ⭐ **SWEEP VERDE É NECESSÁRIO E NUNCA SUFICIENTE.**
> Os `5` achados da 2.ª passagem **não movem o sweep** — a vassoura de `137` entradas não apanha
> nenhum deles, e nem podia: eles não são tradução de prosa do alvo, são *a nossa própria voz a
> descrever o programa*. A parede tem **duas** metades e só uma é automatizável: o sweep prova que
> não houve **colagem nem tradução**; que o documento descreve **comportamento** é juízo, e custa
> uma leitura humana (ou de um R independente) **a cada versão**.

### ⛔⛔⛔ EMENDA 3 — a 3.ª passagem: 7 achados, **1 SUBSTANCIAL**, e a pergunta que faltava

As **11** curas anteriores (as 5 nomeadas + as 6 varridas) foram confirmadas. O substancial é de
espécie NOVA, e a lição é a mais forte desta obra:

> ⭐⭐⭐ **VERIFICAR A ÁLGEBRA DE UM FACTO NÃO É VERIFICAR QUE ELE TEM CONSUMIDOR.**
> A §5.1 afirmava — e a 2.ª passagem **elogiou** — que a inconsistência entre cabeça e origem
> «chega a pixel» num modo. A álgebra estava **certa** (o R re-derivou-a). A consequência era
> **falsa**: o único modo que lê a cabeça **não resolve a cadeia**, e os que a resolvem não a lêem.
> ⇒ a espec mandava reproduzir um artefacto interno por uma razão que não existe.
> **O teste que faltava é *«que fixtura reprova se eu escrever o CONTRÁRIO disto?»***

⭐ **Medido, e é definitivo:** trocar a cabeça guardada pela «corrigida» no modelo de referência muda
a saída de **`0` das `69`** fixturas — `max|dif| = 0,000e+00`. A cabeça é **só-escrita**.

| # | achado | cura |
|---|---|---|
| 1 | **SUBSTANCIAL** — §5.1 afirma consequência falsa; §5.5 sem a cerca | a afirmação e a prescrição **apagadas**; a cabeça declarada não-observável **com a medição**; §5.5 ganha a cláusula *«este modo NÃO resolve a cadeia, e a trava de rotação não tem papel aqui»*; §6 diz que ali o referencial nasce da direcção **inicial** |
| 2 | §7.2 era a decomposição de **armazenamento** (símbolo usado **uma vez** no documento inteiro) | **secção apagada**; §7.3→§7.2, §7.4→§7.3, referências repontadas. Mesma espécie em §10 (contagem de matrizes), §13 («o alvo já o faz»), o **título** da §7.3 e a cauda do §1.3 — as quatro reescritas |
| 3 | §1.3 promete «(medido)» para **cinco** ausências e o corpus mede **uma** | tabela de **duas colunas**: `MEDIDO` vs `AFIRMADO, sem fixtura`. ⭐ E uma segunda passou a medida: `figura_esticar_dedo_invertido` é idêntica ao bit ⇒ `2` medidas, `3` afirmadas |
| 4 | §15 atribui motivação sem nomear o canal | canal nomeado (registo público de commits) **sem link** — a denylist barra URLs de commit ao I, e *um link que ele não pode abrir é pior que nenhum* |
| 5 | §16 «(verificado)» não é proveniência | trocado pelo observável: o modelo **não tem** esse termo e fecha o corpus a `~1e-7` |
| 6 | §4 não cita as corridas de repetibilidade | citadas — e ⭐ **elas mudam a afirmação**: `4×3` corridas em 2 sessões, até `66 049` vértices, **todas idênticas ao bit** ⇒ o limiar **não foi observado**, e a frase passa a **risco nomeado pelo mecanismo** |
| 7 | duas referências penduradas a passos numerados que a emenda 1 apagou | §2.3 → «a média da franja»; §12.1 → «o predicado `dentro`» |

⭐⭐ **E a pergunta virou INSTRUMENTO, não uma correcção pontual:** a espec ganhou a **§12.4**, um
censo de **seis** afirmações sem fixtura que as refute — duas delas **medidas como inertes**, uma
**não observada**, três **afirmadas**. *Ela existe para que a próxima leitura não confunda «a espec
diz» com «o corpus prova», e para que quem acrescentar uma fixtura saiba qual linha ela apaga.*

### Medições novas desta emenda

| pergunta | experiência | veredito |
|---|---|---|
| a constante angular da torção tem proveniência lícita? | **recuperar o ângulo da malha deformada**: num vértice de peso `1` a deformação é rotação pura em torno do eixo do segmento | ⭐ `k = 0,020000 rad/px`, sobre `46` vértices, dispersão `1,8e-6`. Duas fixturas a forças `1,0` e `0,5` dão o **mesmo** `k` ⇒ medem a constante **e** confirmam que a força entra linearmente. Nenhuma leitura de prosa |
| a confusão das duas sementes (§2.1) explica o resíduo aberto? | separar as duas no modelo e apagar o pré-peso | ⛔ **refutado**: em `69` de `69` fixturas a varredura alcança o vértice pré-pesado, e apagá-lo muda a saída em `0,000e+00`. O pré-peso é **inerte neste corpus**; as duas só se separam numa fixtura que o corpus não tem (nomeada na espec §2.1) |

---

### ⛔⛔ EMENDA 4 — a 4.ª passagem: 5 achados, ZERO substanciais, ZERO §4.2 — e o PADRÃO a quebrar

O R-pré confirmou as **7** curas da emenda 3 uma a uma **contra o diff**, deu o §4.2 por **limpo**,
verificou que a renumeração fechou (toda `§N.M` citada resolve) e re-correu o **controlo positivo**
do sweep (`5` achados na v1, `0` no HEAD) ⇒ **a parede está intacta**. Não atestou por cinco achados,
todos da distância entre *«a espec diz»* e *«o corpus prova»*.

| # | achado | cura aplicada (v5) |
|---|---|---|
| 1 | a **forma de promessa** viva em dois **TÍTULOS** desde a v1 — §8 *«as três medidas»* (a 3.ª tem zero fixturas radiais) e §11 *«(todas medidas)»* (a §11.4 não tem) —, um deles a **contradizer o §12.4** que a emenda 3 acabou de escrever | os dois títulos reescritos pela forma honesta; a 3.ª consequência do §8 ganha *«⛔ SEM FIXTURA, e não é descuido»* com o número (`0` de `69` cabeçalhos trazem a chave) e o §11 diz quais três têm fixtura e qual não |
| 2 | o default de **ancorado/trava** (§1.4) justificado por leitura **circular** — e com os números errados | a §1.4 passa a dizer as **três** coisas separadas: o neutro da estrutura é `false`; o harness correu `ancorado` em **`64` de `69`** e trava desligada em **`68` de `69`**, que é **escolha nossa** (o README das fixturas declara que o harness reescreve todos os parâmetros); e ⛔ *«qual dos dois o artista encontra: não medido»*. A recomendação fica rotulada **RECOMENDAÇÃO NOSSA** |
| 3 | a §7.2 rotulava de **observável** uma proposição sobre a ausência de um passo interno do alvo, que **não discrimina** | a §7.2 passa a afirmar a **LEI** (*depois de `N` eventos a posição é a do início do traço mais o `G` acumulado, nunca uma soma de passos*), declara ⛔ **o MECANISMO é livre**, dá a razão de paridade com número (as duas formas diferem só no arredondamento de `f32`, e a barra vive a `~1e-7`) e entra no censo |
| 4 | o **determinismo do alvo** escrito como **facto** em três sítios e como risco num só | a reserva viaja com a afirmação nos **três**: §4 abertura (*«um RISCO nomeado pelo mecanismo, NÃO um facto observado»*), §4 fecho (*«o regime em que o alvo **poderia** não o ser»* + a reserva explícita) e §12 abertura (*«risco do mecanismo, não observado neste corpus»*) |
| 5 | a lista de verificação com um item **fora de ordem** (`10`, `10a`, `10c`, `10b`, …) | renumerada `1..19`, com o solver incremental antes do espremer/esticar (a ordem que o texto já pedia) |

⚠️⚠️ **A LIÇÃO, e é a que vale para a próxima obra: é a TERCEIRA passagem seguida em que a emenda
cura o endereço nomeado e não varre a redacção.** A espécie da emenda 1 (expressão do alvo) morreu
varrendo os cinco endereços e ficou viva em três secções que ninguém nomeou; a da emenda 3
(afirmação sem consumidor) morreu nos sete endereços e ficou viva em **dois títulos**, verbatim
desde a v1. ⇒ **cura-se pela FORMA DA PROMESSA, não pelo endereço**: varrer o documento inteiro por
*«medido / medidas / observável»* e perguntar a cada ocorrência *«que fixtura reprova se eu escrever
o CONTRÁRIO disto?»*.

⭐ **E a varredura por forma produziu um achado que o R não tinha:** a §5.1-bis afirmava que os
outros modos (torção, translação, espremer/esticar) **não** têm memória de evento — e a fixtura que
a refutaria é *o mesmo arrasto a duas taxas de evento nesses modos*, que o corpus **não tem** (as
três taxas só existem para a rotação). Declarada e censada. ⇒ o censo do §12.4 passou de **6** para
**10** linhas.

### Medições desta emenda

| pergunta | experiência | veredito |
|---|---|---|
| o default de ancorado/trava lê-se dos cabeçalhos? | contar a chave nas 69 fixturas **e** ler o README do harness | ⛔ **proveniência circular**: `ancorado` `True` em **`64` de `69`**, trava `False` em **`68` de `69`** — e o harness **reescreve todos os parâmetros** antes de cada traço ⇒ o cabeçalho é ENTRADA, não observação |
| os controlos que a espec declara sem efeito variam no corpus? | censo dos cabeçalhos das 69 | ⛔ `hardness`, `direction`, `falloff_shape`, `use_frontface` e `normal_weight` são **CONSTANTES** nas 69 ⇒ *o corpus nunca os varia*, logo não pode medir ausência de efeito. É por isso que a tabela do §1.3 os separa das medidas |
| há traços com simetria radial? | censo dos cabeçalhos | ⛔ **`0` de `69`**, e **nenhuma** traz sequer a chave ⇒ a linha do §8 é **da issue**, não nossa |

⚠️ **Duas notas do R que ficam por escrito:**

1. o censo do §12.4 está **honesto** — as três ausências que ele declara não têm chave no cabeçalho
   de nenhuma fixtura, logo não são descuido de medição;
2. uma suspeita do R caiu na medição: **por NOME** só uma fixtura parecia cobrir a suavização, e o
   censo dos **cabeçalhos** mostra as três valores presentes. ⇒ ***Inferir cobertura do nome do
   ficheiro é a armadilha; o instrumento é o cabeçalho.***

---

## Incidentes

*(vazio)*

---

## Fechamento R

*(vazio)*
