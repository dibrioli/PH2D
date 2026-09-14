# LEDGER de proveniência — clean-room do pincel BOUNDARY (alvo `blender-boundary`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-boundary.md` (append cego).

⏱️ **Aberto em 2026-09-13, ANTES da primeira leitura do fonte do pincel.** Antes desta abertura o E
fez apenas: (a) uma **listagem de NOMES** de ficheiros do directório do modo escultura no checkout
(para localizar a obra — nenhum conteúdo aberto); (b) a leitura de artefactos **NOSSOS** do repo
(o harness `docs/3D/ferramentas/blender_sculpt_oracle.py`, o `cleanroom-sweep.sh`, a SKILL, o
precedente `LEDGER_blender-cloth.md` e o README das fixtures do tecido, `shapes_open.rs`).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — pincel **Boundary** do Sculpt Mode (valor público `BOUNDARY` do enum `sculpt_brush_type`) + o que ele invoca que decide comportamento (detecção de contorno, propagação, queda, simetria, máscara, traço de «agarrar», undo, cursor) |
| Versão / commit do fonte lido | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` (2026-07-13), checkout **esparso e grafted** — o MESMO checkout que a obra `blender-cloth` leu |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D |
| Zona fora da árvore (notas, rascunhos, oráculo) | `~/Referencias/blender-boundary/` (`notes/` · `draft/` · `oracle/`) |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença | **GPL-2.0-or-later** — `COPYING` remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991) e diz que o Blender não está disponível sob outras licenças; os três ficheiros do pincel levam cabeçalho SPDX `GPL-2.0-or-later` (lido em 2026-09-13) |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS** (build 2026-09-01) — ⚠️ patch-release acima do fonte lido (5.2.0); a lista de commits do ficheiro do pincel mostra UM commit posterior à tag (2026-09-10, armazenamento local de nó — sem mudança de comportamento declarada); o oráculo é o binário |

### A concessão relevante (GPLv2 §0 e §2), transcrita do ficheiro do checkout

> *"Activities other than copying, distribution and modification are not covered by this
> License; they are outside its scope. The act of running the Program is not restricted,
> and the output from the Program is covered only if its contents constitute a work based
> on the Program (independent of having been made by running the Program)."* (§0)
>
> *"You may modify your copy or copies of the Program or any portion of it, thus forming a
> work based on the Program…"* (§2)

⇒ Ler, correr e instrumentar **em privado** é licenciado. A **saída** do programa (posições de
vértices de malhas NOSSAS) é dado. Nenhum acto deste ledger envolve distribuição. Não é AGPL.

---

## §2 — Triagem: a escada de portas (2026-09-13)

| degrau | veredito | por quê |
|---|---|---|
| T0 | ⛔ | o pincel só existe neste alvo, GPL |
| T0½ | ⛔ | nenhum ficheiro do pincel sob licença por-ficheiro ou LGPL |
| T1 (a) autores | ⛔ | o autor original (2020) publicou-o directamente no alvo; não há paper nem código de referência paralelo |
| T1 (b) versões antigas | ⛔ | o pincel nasceu em 2020 já sob GPL |
| T1 (c) reimplementações | ⛔ | busca web «boundary brush sculpting open source MIT» e afins (2026-09-13): o único outro projecto de escultura aberto achado (`khanhha/digital_sculpting`) **não declara licença** e **não tem** pincel de contorno; SculptGL (MIT) não o tem; Nomad/ZBrush são proprietários |
| T1 (d) e-mail | não tentado — sem candidato a dual-license (o pincel é parte do núcleo do alvo) |
| **T2** | ✅ **é o degrau desta obra** | copyleft com fonte |

---

## Patente (§8.1) — checkpoint incondicional (2026-09-13)

- **Termos:** `boundary brush sculpting open source` · `patent sculpting brush deformation propagated
  from mesh boundary edges falloff distance` · `patent interactive mesh deformation tool open boundary
  loop bend twist inflate brush 3D sculpting` · `patent "boundary" brush digital sculpting deform edge
  loop` · `patent brush deforms polygon mesh open border edge loop rotation pivot bend fold` ·
  `Pixologic OR Maxon OR Autodesk patent sculpting "topological distance" brush falloff mesh boundary`.
- **Resultado:** ⭐ **nenhuma patente viva alcança o método** (detectar o laço de arestas abertas mais
  próximo, propagar por passos topológicos, e aplicar rotação/translação/escala parametrizadas por um
  pivô no fim da propagação). Achados, com veredito:

| patente | dono | estado | lê sobre nós? |
|---|---|---|---|
| US 9 830 743 B2 — «Volume-preserving smoothing brush» | Autodesk | **VIVA até 2034-03-24** | ⛔ não — a reivindicação 1 exige suavização laplaciana + estimativa de volume pela média dos vectores laplacianos + **inflação** para repor o volume; o modo *Smooth* deste pincel é uma média de vizinhos do mesmo passo, **sem** estimativa nem reposição de volume. ⚠️ **cerca nomeada**: nunca acrescentar ao modo de suavizar uma «reposição de volume por inflação» |
| US 7 589 720 B2 — edição de malha por campo de gradiente / fusão por curva de fronteira | Microsoft | **EXPIRADA** (2026-06-20, fee related) | não lê (fusão de dois objectos por correspondência de fronteiras) e está expirada ⇒ literatura livre |
| US 10 586 401 B2 — Kelvinlets | Pixar | viva até 2038 | não lê (soluções analíticas de elasticidade); já cercada pela obra do tecido |

⇒ Veredito: **prosseguir**. Sem «PATENTE VIVA» a reportar.

---

## Papel E — Especificador

⚠️⚠️ **DUAS corridas de E, e a segunda NÃO herdou a leitura da primeira.** A 1.ª (2026-09-13, manhã)
morreu por limite de uso da conta depois de abrir este ledger, fazer a triagem, a busca de patente,
a vassoura e **toda** a matriz do oráculo; ela **não** escreveu espec e **não** comitou nada.
A 2.ª corrida (2026-09-13, tarde — a que assina a espec) **refez a travessia do fonte por inteiro**,
porque o contexto da primeira não existe e *uma espec só pode ser escrita por quem leu*. O que ela
herdou foram **artefactos em disco** (ledger, vassoura, notas da história, harness e dumps), nunca
conclusões. A cobertura abaixo é a da **2.ª** corrida.

| campo | valor |
|---|---|
| quem | **subagente-E (alvo `blender-boundary`)** despachado pela janela I da `line/sculpt3d` — 2 corridas, ver acima |
| session-id da janela-mãe | `9f820704-0d7e-4d96-847e-9cd720cbf178` |
| paralelismo | um de **quatro** subagentes-E simultâneos na mesma worktree (pull · pose · boundary · unblocked); este tocou SÓ `*blender-boundary*`, `SPEC_boundary_brush.md` e `fixtures/boundary/` |
| transcript (⛔ zona contaminada — I nunca lê) | ficheiros de subagente sob `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178*` |
| aberto em | 2026-09-13 |
| lê | o fonte do pincel e o que ele invoca, por shell (`sed`/`awk`/`grep`), porque o `deny` da linha nega `Read` |
| escreve | `SPEC_boundary_brush.md` (commit único, pós-filtragem) · este ledger · `VASSOURA_blender-boundary.txt` · `fixtures/boundary/` |
| ⛔ nunca | código de produto; `git push`; `git add -A`; `git stash`; `doc-index.sh` (é da janela) |

### INBOX transcrito (2026-09-13)

> `I session: 9f820704-0d7e-4d96-847e-9cd720cbf178 2026-09-13 — janela I da line/sculpt3d (a mesma do
> INC-4 classificado RELANCE); obra: pincel Boundary`

⇒ A corrente I desta obra começa na janela `9f820704-…` (elo **I-3** da linha, segundo o ledger do
tecido); o INC-4 (RELANCE) foi classificado noutro alvo e não queima esta janela.

### Cobertura da travessia (§3.E) — 2.ª corrida, 2026-09-13, por shell, fonte v5.2.0

⭐ **O ficheiro do pincel (`3 702` linhas) foi lido INTEIRO**, do primeiro ao último byte, mais o
cabeçalho dele (`167`) e os testes públicos dele (`83`). ⚠️ **O ficheiro é ~3× redundante por
construção** (a mesma lei escrita uma vez por cada uma das três representações de malha do alvo);
depois de reconhecer o padrão nas duas primeiras famílias, as terceiras cópias foram lidas por
varredura dirigida em vez de linha a linha — **e isso está declarado aqui de propósito**, porque a
espec só fala da representação base (§1 dela) e as outras duas ficam nomeadas como fora.

A tabela abaixo (herdada da 1.ª corrida e **reconferida** pela 2.ª) diz as regiões percorridas.

| área | ficheiros (relativos a `source/blender/` ou `scripts/`) | linhas | lido |
|---|---|---|---|
| **o pincel** | `editors/sculpt_paint/mesh/sculpt_boundary.cc` · `.hh` · `sculpt_boundary_tests.cc` | 3 702 + 167 + 83 | ⭐ **INTEIROS**, do 1.º ao último byte (8 blocos + o header + os testes) |
| o que decide comportamento, em `sculpt.cc` | `editors/sculpt_paint/mesh/sculpt.cc` | 8 4xx | as regiões: vizinhos de vértice (520–583) · cache de contorno e o teste de vértice de contorno (585–650) · primeiro passo da passagem, pivô de simetria, alinhamento à normal, vértice mais próximo (655–770) · pincéis que usam posições originais (915–945) · sobreposição/esbatimento de simetria (1352–1411) · força por tipo de pincel (2329–2477) · undo por nó (3170–3195) · nós que um pincel toca (3225–3254) · despacho (3575–3600) · alvo de pano e acumulação (3635–3674) · dados simétricos do traço (3676–3700) · laço de simetria/radial/ladrilho (3780–3881) · delta de agarrar (4170–4345) · verificação de modificadores (4420–4441) · actualização da geometria sob o cursor (4751–4860) · restauro por undo (5169–5210) · início do traço (5595–5621, 5680–5705, 5760–5827) · actualização do traço (5830–5925) · passo (5935–5970) · deformação de posições (7848–7898) · recorte/trava de eixos, filtros, factores (7730–7990) · filtro por área de simetria (8280–8330) |
| cabeçalho interno | `editors/sculpt_paint/mesh/sculpt_intern.hh` | — | 200–310, 405–430, 835–870 |
| contratos partilhados | `editors/sculpt_paint/mesh/mesh_brush_common.hh` | — | 440–490 + declarações dos filtros |
| enchimento por vizinhança | `editors/sculpt_paint/mesh/sculpt_flood_fill.cc` · `.hh` | 200 + 89 | ⭐ **INTEIROS** |
| esconder | `editors/sculpt_paint/mesh/sculpt_hide.cc` | — | 88–130 |
| traço (espaçamento, localização, agarrar) | `editors/sculpt_paint/paint_stroke.cc` · `paint_intern.hh` | — | 145–225, 975–1100 · 634–661 |
| cursor e pré-visualização | `editors/sculpt_paint/mesh/paint_cursor.cc` | — | 495–530, 615–670 |
| auto-máscara (o que toca o contorno) | `editors/sculpt_paint/mesh/sculpt_automasking.cc` · `sculpt_smooth.cc` | — | só as ocorrências de contorno (grep) — ⚠️ a auto-máscara por FACE SET fica fora (exclusão C) |
| kernel do pincel | `blenkernel/intern/brush.cc` | — | 1600–1680 (curvas de queda) · 1770–1830 · 1980–2018 (capacidades) |
| defaults/faixas/enums | `makesdna/DNA_brush_enums.h` (233–246, 468) · `DNA_brush_types.h` (380–386) · `makesrna/intern/rna_brush.cc` (2514–2545, 2615–2621, 2805–2832, 3349–3355) | — | ✔ |
| painel | `scripts/startup/bl_ui/properties_paint_common.py` (1025–1050) | — | ✔ |
| ⛔ NÃO lido, de propósito | o solver de pano (`sculpt_cloth.cc` — já especificado pela obra `blender-cloth`) · multires (`subdiv_ccg*`) e dyntopo (`bmesh`) além do que o pincel ramifica · face sets (exclusão C) · pincéis de cor (exclusão C) | — | fora do alcance da espec |

**História (web, porque o checkout é grafted):**
- **169** commits que tocaram o ficheiro do pincel desde 2020-08-10 (API Gitea, 3 caminhos históricos),
  com o corpo integral dos **71** que não são limpeza guardado em
  `~/Referencias/blender-boundary/notes/commits_boundary_behavior.txt`.
- **100** issues do tracker por «boundary brush» (lista) e o corpo + comentários de **21** delas
  (`notes/issues/`).
- a página do manual 5.2 do pincel (`notes/manual_boundary_latest.txt`) e as notas de lançamento 2.91
  (`notes/web_*release_notes_2_91*`).
- ⚠️ As páginas de revisão (D8356, D8526, D9204) devolvem uma página de verificação anti-bot a fetch
  automático; o conteúdo delas está nas mensagens de commit correspondentes.

**Presets do binário 5.2.1:** os **3** pincéis de contorno da biblioteca *Essentials* (lidos por `bpy`,
nunca copiados como ficheiro — `oracle/assets_boundary.json`). ⛔ Os `.blend` são assets (§8.3); só os
NÚMEROS entram na espec, como facto observado.

### Achados de PAREDE para o R (registados pelo E em 2026-09-13)

1. ⚠️ **Uma linha de CÓDIGO do alvo já vive num doc nosso, anterior a esta obra:**
   `docs/3D/20_divergencias_tools.md:596` cita, entre crases, uma condição de saída antecipada de outro
   pincel do alvo com o nome interno do delta simétrico de agarrar. É uma citação de **expressão** (uma
   linha de código), da família do INC-4. A vassoura desta obra inclui esse nome interno, logo o sweep
   de ÁRVORE do R vai acusá-la: **veredito do R**, não do E (o E não edita docs de outras obras).
2. O ledger do tecido (`LEDGER_blender-cloth.md:108`) nomeia o ficheiro do pincel na tabela de
   cobertura dele — é um LEDGER (autorizado a carregar rastros); o sweep de árvore também o acusa.
3. ⚠️ **Defeito do harness, medido e curado:** a primeira matriz (180 corridas) devolveu `NaN` em
   ~20 % das corridas dos modos que leem o ponto de sobrevoo (dobrar · expandir · inflar · torcer ·
   o alvo de pano), **alternando entre realizações da mesma corrida**. Mecanismo: o ponto de superfície
   sob o cursor é escrito SÓ pelo desenho do cursor com normal amostrada; num objecto recém-criado ele
   não existe até esse desenho correr, e um traço scriptado que chegue antes lê lixo. Cura: movimento
   de rato SIMULADO (`--enable-event-simulate`) para o pixel do pen-down + um desenho SÍNCRONO da janela
   antes do traço, e a corrida com `NaN` REFEITA em cena nova (a contagem vai no cabeçalho de cada
   fixture). *O mesmo defeito que o E do tecido pagou (achado 4 dele), com outro sintoma: lá o centro
   ficava refém do sobrevoo, aqui o próprio sinal do deslocamento.* ⛔ **Não é comportamento do
   produto**: um artista passa sempre o cursor sobre a malha antes de carregar.
4. ⚠️ **Segundo defeito do harness:** nas grelhas densas o pixel inteiro mais próximo do contacto
   pedido (a `0,2` célula da borda) caía FORA da malha, o traço não começava e a corrida gravava
   «0 movidos» em 0,2 ms. Cura: contacto a `0,005` da borda. *Um «nada se moveu» rápido é a assinatura
   de um traço que nunca começou.*
5. O oráculo corre numa **tela virtual** (`kwin_wayland --virtual --xwayland`), nunca no ecrã do dono.

---

## Oráculo (§5)

| campo | valor |
|---|---|
| binário | `/usr/bin/blender` 5.2.1 LTS |
| harness | `~/Referencias/blender-boundary/oracle/harness.py` (v2) + `malhas.py` (as malhas) + `gera_cfg.py` (a matriz) + `corre.sh` (a sessão) + `monta_fixtures.py` (o montador) — corre **com janela** numa tela virtual e sai sozinho |
| entrada | ⭐ **NOSSA**: grelhas planas, tubo aberto, cúpula aberta, esfera fechada, faixa de uma fileira, grelha triangulada, grelha deslocada meia célula, grelha com a borda de baixo perturbada — todas geradas por `malhas.py`. O pincel é o preset *Boundary* do binário **só para existir um pincel de contorno activo**, com TODOS os parâmetros reescritos |
| saída | `oracle/out/*.npz` (repouso, deformado, caminho, meta) |
| o que vira fixture | ver `docs/3D/cleanroom/fixtures/boundary/README.md` |

---

### O que a 2.ª corrida acrescentou ao oráculo (2026-09-13)

1. ⭐ **Seis corridas NOVAS, `sinal_*`** — o controlo do **sinal do avanço** nos dois sentidos de
   arrasto (`agarrar`/`expandir`/`inflar`/`dobrar`, para dentro e para fora). Sem elas o sinal da lei
   ficava por decidir entre duas leituras igualmente consistentes com o corpus antigo; com elas ele
   está fechado e cruzado em quatro modos. `cfg_sinal.json`.
2. ⭐⭐ **As `32` corridas com `NaN` foram REFEITAS e o corpus está limpo** (`0` de `186`). Eram os
   dois arrastos para dentro, os dois do alvo de pano e três séries de fatiamento de `8`. ⚠️ A
   causa é do **harness** (o ponto de sobrevoo, achado 3 abaixo) e não do produto — mas uma fixture
   com `NaN` é pior que fixture nenhuma: ela manda o Implementador caçar um fantasma. `cfg_nan.json`.
3. **As fixtures foram montadas** (`monta_fixtures.py`, que a 1.ª corrida escreveu e nunca correu):
   `61` traços + `7` séries por-passo + `9` malhas de repouso em `fixtures/boundary/`.
4. **A vassoura foi de `87` para `163` entradas** — a 1.ª corrida não podia cobrir o que só a
   travessia da 2.ª leu (nomes internos de contratos partilhados, frases de comentário e de
   mensagem de commit). ⚠️ Controlo positivo corrido: o sweep **acusa** uma frase plantada.
5. **A régua das fixtures** (`confere_cabecalhos.py`) foi escrita e tem controlo negativo.

### Achados de PAREDE acrescentados pela 2.ª corrida

6. ⚠️ **`dispersao_entre_realizacoes = 0` em todas as fixtures com duas realizações** — o pincel é
   determinístico ao bit. Isto **fecha** a pergunta sobre a barra de paridade precisar de tolerância
   para ruído do alvo: não precisa.
7. ⛔ **Um facto que quase entrou errado na espec:** a leitura directa do fonte dava ao avanço o
   sinal **oposto** ao medido. As três leis que o consomem (`EXPAND`, `INFLATE`, `BEND`) só são
   mutuamente consistentes com o sinal **medido**, e a fixture do arrasto tangente (`0` movidos)
   fecha a projecção. *A espec escreve o que a MEDIÇÃO deu, e diz que o faz* (§9.1 dela).

## Auditoria R-pré — 2026-09-13: REPROVADA, e a reescrita (versão 2)

O R-pré **não atestou** a versão 1: **4 achados**, o primeiro classificado SUBSTANCIAL. A reescrita
entregue no mesmo dia é a versão 2, e ela **aguarda um R-pré NOVO** — ⛔ a janela não implementa
antes disso.

⚠️⚠️ **O INSTRUMENTO estava cego, e essa é a lição maior desta auditoria.** O sweep de entrega da
versão 1 fechou **verde sobre quatro traduções frásicas de comentários do fonte**: das 163 entradas
da vassoura, ~100 eram prosa **toda em inglês**, e esta espec escreve-se em **português**. *Uma
vassoura na língua do alvo não varre um documento na nossa.* O R alargou-a para **199** (36 entradas
de prosa em PT), com a regra de desenho que fica: **só entram formas PT de prosa de comentário/TODO
do fonte** — ⛔ **não** entram formas PT de manual, mensagem de commit ou rastreador, que o §4.1.12
permite citar; pô-las ali faria o instrumento acusar um acto **permitido**.
⚠️ E o mesmo defeito apareceu noutras duas especs desta obra no mesmo dia ⇒ **é do método, não da
espec**.

### O que cada achado virou

| # | achado do R | o que ficou na versão 2 |
|---|---|---|
| 1 | **§5.2, coluna «porquê (D)»** — as duas justificações eram tradução de prosa que o alvo guarda DENTRO do fonte, e o selo `(D)` era **falso**: o R conferiu o corpus público inteiro e nenhuma das duas frases existe no manual, em commit ou em issue | coluna substituída por **razão derivada da GEOMETRIA**, marcada `(N)`: grau `≤ 2` = dois troços de borda encontram-se e não há regra que escolha entre eles · `> 2` vizinhos de borda = a borda **ramifica** e cadeia única deixa de estar definida. ⭐ **As condições ficam** — são medidas, e a §5.3 tem as fixtures |
| 2 | **§5.2 · §13.6 · §14.4 (+§16.1)** — três sítios citavam o que o alvo diz **sobre si mesmo** dentro do fonte. O mais caro era o §14.4: a frase seguinte, apresentada como ideia nossa, **era** o conteúdo da nota de trabalho do alvo | §5.2 passou a apoiar-se no **comportamento medido** (a fixture da quina: `0` movidos com âncoras sãs a uma célula) · §13.6 ficou só com a nossa regra `(N)`, já que a espec declara as outras representações fora de alcance · §14.4/§16.1 ficam com o **custo medido** e a melhoria como decisão nossa `(N)`, **sem** «o alvo declara» e sem o `(D)`. ⭐ O §16.1 deixou de ser um ponteiro e passou a dizer a lei observável: *a região tocada não está contida na esfera do raio* |
| 3 | **§14.4 — «três arrays do tamanho do número de vértices»** é o LAYOUT INTERNO do alvo | passou a **classe de custo**: o pen-down preenche dados por-vértice proporcionais à malha, em tempo e memória, e ⛔ *quantas estruturas e com que forma é decisão do Implementador* |
| 4 | (menor) **§7.4 — a linha branca**: conteúdo legítimo e observável; o que atravessava era a **FORMA**, decalque de uma frase inteira do manual | re-dito em estrutura nossa (dois papéis do ponto-origem: referência da deformação · o que o artista precisa de ver), com a citação do manual **endereçada** e uma nota a dizer que o texto é nosso |

### Higiene corrigida no mesmo passe

As entradas `(D)` que citavam **só a data** passaram a trazer o **endereço público** que o §4.1.12
pede: commits `d1bbef936` (2025-11-19) · `653b273d2` (2024-08-01) · `ca827e36a` (2020-08-11) ·
`bbbfd7130` (2020-09-06) · `74d1fba1d` (2020-10-18).

### O que o R conferiu e está LIMPO (⛔ não refazer)

Os nomes conservados são **todos** identificadores públicos da API, verificados um a um · zero nome
interno · a matemática das §8/§9/§10 **não** espelha corpo de função · a ordem das fases A→H é
forçada por **dependência de dados**, não pela organização do alvo · as fixtures e as chaves em
vocabulário do domínio · toda a prosa pública citada existe no corpus e está re-dita ou citada curto
com atribuição.

### Prova de que o verde da versão 2 vale (⭐ o controlo que o verde anterior não teve)

- **Controlo positivo:** as **6** linhas que a versão 1 tinha nesses sítios, extraídas do commit
  anterior, são **acusadas** pela vassoura de 199 (`exit 1`, as seis nomeadas).
- **Negativo:** a versão 2 e as 78 fixtures fecham `exit 0` sobre a mesma vassoura.
- ⚠️ **Limite NOMEADO do instrumento:** ele casa **string literal**. Uma paráfrase — ou a mesma frase
  sem acentos — **escapa** (medido: três controlos de-acentuados passaram limpos). ⇒ o sweep é a
  rede, **nunca** o juízo; quem julga é o R.

## Corrente I

| janela | session-id | data | motivo | declaração |
|---|---|---|---|---|
| I (janela-mãe) | `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-13 | despachou este E (e três irmãos) | pelo **inbox** (entrada de 2026-09-13, transcrita acima) |

## Papel R

| papel | id | data |
|---|---|---|
| R-pré (1.ª passagem) | subagente R-pré despachado pela janela `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-13 — ⛔ **REPROVADA, 4 achados** (versão 1) |
| R-pré (2.ª passagem) | subagente R-pré **novo**, mesma janela | 2026-09-13 — ⛔ **REPROVADA, 4 achados** (versão 2) |
| R-pré (3.ª passagem) | ⏳ **por despachar** — subagente NOVO, sobre a versão 3 | — |
| R-pós | ⏳ | — |

### Auditoria R-pré — 2.ª passagem (2026-09-13) — ⛔ REPROVADA (versão 2), 4 achados

**As 4 curas da 1.ª passagem foram conferidas UMA A UMA e estão FEITAS** (⛔ não refazer): a coluna
das duas recusas é hoje argumento de **geometria** marcado `N`, e é argumento **diferente** do que o
alvo guarda no fonte (conferi os dois lados) · os três sítios que citavam o que o alvo diz de si
mesmo apoiam-se agora em medição ou em decisão nossa · o layout virou **classe de custo** e recusa
por escrito prescrever o arranjo — e as três grandezas por-vértice que ele nomeia **já são o
vocabulário da própria espec** (§2/§7/§8), logo não acrescentam nada que um leitor dela não tivesse ·
a linha do sobrevoo está re-dita em estrutura nossa com o manual endereçado.
⭐ **A proposta do §14.4 marcada `N` NÃO é para reabrir:** ela é derivável da própria espec (as fases
fixadas no pen-down, §3 + a sublinearidade medida na tabela do §14.4), e a cadeia está registada
acima. *Uma recusa medida responde uma pergunta; esta já respondeu.*

**Verificação independente de TODO selo `D` (⛔ não refazer):** os **5** hashes de commit citados
existem e o assunto de cada um bate com a afirmação; os **6** números de defeito existem e o conteúdo
bate; as **2** citações curtas em itálico saem ambas de **mensagem de commit pública**; as **4**
afirmações atribuídas ao manual estão na página pública. **Nenhum selo `D` vem do fonte — excepto os
dois do achado 1.** Também limpo por inspecção: zero nome interno, zero texto de código, zero
pseudo-código espelhado nas seis leis, no produto do peso, no predicado de região e na ordem das
fases; e o conjunto de sinais do modo de laço invertido é **matemática + medição** (a tabela medida
cobre um período inteiro, logo o resto sai por periodicidade) — ⛔ **não o "corrija"**, apesar de um
comentário do fonte enumerar os mesmos inteiros.

#### Os 4 achados da 2.ª passagem

| # | onde | o que é | por que não pode |
|---|---|---|---|
| 1 ⛔ **BLOQUEIA** | §4.1, a tabela do censo de bordas | as **três** linhas são os **três casos do conjunto de testes unitários do alvo, na ordem dele**, com os valores esperados dele — e a linha do meio **não tem fixture nenhuma** e está selada `D` | o `D` do §0 é *manual / mensagem de commit / rastreador*: um ficheiro de teste **dentro da árvore do alvo** não é nenhum dos três (mesma espécie do achado 1 da 1.ª passagem). §4.1.6 só abona vector de teste **gerado a correr o oráculo**. ⚠️ Agravante: **o rótulo da linha não produz os números dela** — como está escrito conta quadrados, e os valores são os do reticulado de vértices um degrau menor ⇒ a linha não é medida nem derivável como está redigida |
| 2 | §4.5 (o gatilho que invalida o cache do censo) · §17 (recalcular a cada quadro de sobrevoo · o alvo não ter cache de sobrevoo) | três frases que afirmam **o que o CÓDIGO do alvo tem**, não o que o programa FAZ, e **nenhuma é medida** — o próprio §20 lista os dois custos como abertos | §4.3 pergunta 1. O cache em si é inferência justa do relógio medido; o **gatilho**, o **por-quadro** e a **ausência** de cache não saem de fixture nenhuma do corpus. Espécie do achado 3 da 1.ª passagem, um nível acima |
| 3 | §8.4 e §10.6.3 | dois selos `D` **sem endereço** — a 1.ª passagem prescreveu endereço a cada `D` restante e **cinco** receberam, **dois** não | §4.1.12 pede a fonte ao lado. ⚠️ Conferi: os dois estão **de facto** abonados por mensagem de commit pública, logo falta só o endereço. Dois menores da mesma família: um defeito citado com o prefixo do rastreador **antigo** enquanto os outros usam a forma actual, e um commit endereçado por **data** onde os outros cinco levam hash |
| 4 | §4.5 → §15 · §17 → «§13.1#4» | duas referências cruzadas apontam ao sítio errado (o relógio vive no §14.4, não na tabela de opções; e o caso 4 é **linha** da tabela do §13, não subsecção) | não é §4.2 — mas o artefacto atravessa a parede e manda o Implementador ao sítio errado |

#### O que cada achado pede (instrução FUNCIONAL)

1. Apagar as **duas** apelações ao conjunto de testes do alvo (a do texto de entrada e o selo da
   linha). A linha sem fixture ou (a) **passa a ser medida** numa corrida do nosso oráculo sobre uma
   malha nossa, com o nome da fixture e o selo `M`, ou (b) **passa a `F`**, dita como aritmética
   sobre um reticulado cuja contagem de vértices seja nomeada. Corrigir o rótulo para que os números
   **decorram dele**. E **reconstruir a lista de casos a partir do NOSSO corpus**, para que a
   escolha e a ordem das linhas venham das nossas fixtures. ⭐ Acrescentar **uma cláusula ao §0**:
   o selo `D` nunca cobre nada lido **dentro da árvore do alvo**, ficheiro de teste incluído — sem
   ela o mesmo selo fica disponível para a próxima secção.
2. Guardar só o que uma fixture mostra ou o que **nós exigimos**, com a marca. A instrução de
   produto (*medir antes de copiar; a chave óbvia do primeiro cache é o vértice apontado mais o
   raio*) **sustenta-se sozinha** como `N`. ⛔ Cair fora toda cláusula que descreva o que o código do
   alvo guarda ou deixa de guardar.
3. Pôr o endereço ao lado dos dois `D`; uniformizar a forma de citar defeito e commit.
4. Corrigir as duas referências.

#### O instrumento, medido por mim (⚠️ o limite é MAIOR do que este ledger registava)

- **Sweep da versão 2** sobre a espec **e** as fixtures: `exit 0`, vassoura de `199`.
- **Controlo positivo corrido por mim** sobre a versão reprovada, extraída do commit anterior:
  `exit 1`, **6** linhas — o verde da versão 2 é, portanto, controlado.
- ⛔⛔ **Mas de-acentuar a MESMA prosa mata 4 dos 6 apanhados** (`exit 1` sobrevive em 2 linhas, e
  essas duas sobrevivem por acidente — o trecho que casa não tem acento nenhum). Este ledger dizia
  *«três controlos de-acentuados passaram limpos»*; o número real é **dois terços do instrumento**.
  Medido também: só **30** das `199` entradas levam acento, e **113** de `199` têm forma de
  identificador e não de prosa. Contra os três ficheiros do alvo a vassoura casa `104`/`24`/`5`
  entradas ⇒ a cobertura de identificador é real e a metade de PROSA é fina. *O sweep é a rede;
  quem julga é o R.*
- ⚠️ **O próprio LEDGER é apanhado pela vassoura** (a tabela de cobertura nomeia os caminhos de
  ficheiro do alvo). É **por desenho** (§6: o ledger carrega rastros, e o I nunca o abre) — mas
  significa que a barra do R-pós *«zero hits sobre a árvore inteira»* precisa de **excluir este
  ficheiro por escrito**, ou de trocar os caminhos por uma descrição. ⛔ Não "curar" o ledger sem
  essa decisão.

### Versão 3 (2026-09-13) — as 4 curas da 2.ª passagem, e a cura do INSTRUMENTO

⛔ **Escrita pelo E; ⏳ NÃO atestada — aguarda um R-pré NOVO.** A regra do arquivo fechado foi
honrada: a reescrita saiu da compreensão, e o fonte só foi reaberto para confirmar **factos**
(dois endereços de commit e a data/número de um defeito). Nenhuma linha nova cita o fonte.

| # | o que o R pediu | o que a versão 3 tem |
|---|---|---|
| 1 ⛔ | apagar as **duas** apelações ao conjunto de testes do alvo; a linha sem fixture passa a `M` ou a `F` com o reticulado nomeado; o rótulo tem de **produzir** os números; e a lista de casos reconstruída a partir do NOSSO corpus | a tabela do §4.1 é hoje o **censo das nove malhas de repouso do nosso corpus**, na ordem delas — `esfera` · `grade` · `grade_pequena` · `grade_triangulada` · `grade_deslocada`/`grade_ruidosa` · `tubo` · `cupula` · `faixa`. As duas colunas de borda são **contadas** sobre as faces de cada `*.repouso.txt.gz` (⇒ `F` sobre `M`, recalculável por qualquer um do ficheiro), e a coluna `reticulado` traz o `L × C` de VÉRTICES de que a fórmula do perímetro (`2(L+C) − 4` · `2(L−1) + 2(C−1)`) **produz** os números. ⭐ Coluna nova, **o corpus confirma**: em cada peça os vértices que o traço de facto move são um múltiplo exacto da cadeia prevista (`165 = 33 × 5` · `15 = 5 × 3` · `128 = 32 × 4` · `96 = 32 × 3` · os dois `0` das recusas). ⚠️⚠️ **E a linha apagada não era só mal selada: estava ERRADA contra a nossa própria fixture** — ela dizia `6` vértices e `6` de `7` arestas ao lado do nome `faixa.repouso`, que tem `66`, `97` e `66`. *Uma linha copiada de outro corpus não descreve o nosso nem por acaso.* ⭐ E o §0 ganhou a cláusula: **o selo `D` nunca cobre nada lido dentro da árvore do alvo, ficheiro de teste incluído** |
| 2 | guardar só o que uma fixture mostra ou o que NÓS exigimos; cair fora toda cláusula sobre o que o código do alvo guarda | §4.5: fica `O(arestas)` (`F`) + o relógio medido, e o cache passa a **exigência nossa** (`N`) com a frase *«esta espec não afirma o que o alvo guarda nem quando o deita fora»*. §17: o que fica é **observável** — o desenho acompanha o cursor, logo mostra A–D do vértice apontado agora —, e o custo virou *«A–D são `O(malha)`, logo o sobrevoo custa o pen-down»* com a instrução de produto marcada `N` (medir antes de copiar; chave `(vértice apontado, raio)`); ⛔ caiu *«recalcula a cada quadro»* e *«o alvo não tem cache»*. O §20.4 passou a dizer que nenhuma fixture toca o sobrevoo |
| 3 | endereço nos dois `D` órfãos; uniformizar defeito e commit | §8.4 → commit público `c77bf9522` (2020-08-11, o que introduziu o controlo) · §10.6.3 → `2b2f3da72` (2020-10-18, o que introduziu o modo) · §8.2 `T84896` → **`#84896`** (o defeito existe no rastreador actual com esse número) · §11.2 passou de uma **data solta** a `b042b750d`, 2026-03-04, *que fecha* o defeito público #154678 ⇒ contados no ficheiro: **oito** commits citados, **oito** por hash de `9`, e **sete** defeitos, os sete na forma `#NNNNN` do rastreador actual |
| 4 | as duas referências cruzadas | §4.5 `(§15)` → **`(§14.4)`** (o relógio) · §17 `§13.1#4` → **«o caso `4` da tabela do §13»** |

#### ⭐⭐ A cura do INSTRUMENTO (o achado mais caro da 2.ª passagem), medida

A vassoura casa **string literal**, e de-acentuar a mesma prosa PT escapava-lhe. ⇒ **toda entrada de
prosa em português passou a ter também a forma SEM ACENTOS** (base64, como as outras):
**`199` → `229`** entradas — as `30` acentuadas do bloco PT ganharam gémeo (as outras `6` do bloco
não têm acento nenhum e já casavam). ⛔ Nenhuma entrada foi retirada e a regra de desenho do R fica
intacta: só prosa de **comentário/TODO do fonte**, ⛔ nunca de manual, commit ou rastreador.

**Controlo positivo corrido nos DOIS sentidos** sobre a **versão 1** (a reprovada, `23bb85670`) e
sobre uma cópia dela **de-acentuada** (gerada fora da árvore, em `~/Referencias/…/draft/controlo/`):

| vassoura | v1 acentuada | v1 de-acentuada |
|---|---|---|
| `199` (antes) | `exit 1`, **`6`** linhas | `exit 1`, **`2`** linhas — ⛔ `4` de `6` perdidas |
| **`229`** (agora) | `exit 1`, **`6`** linhas | `exit 1`, **`6`** linhas ✅ |

**Negativo:** a versão 3 + o directório inteiro das fixtures (`79` ficheiros) fecham `exit 0` sobre
a vassoura de `229`.
⚠️ **O limite NOMEADO continua a valer**: uma **paráfrase** escapa na mesma, e `113` das entradas
têm forma de identificador (essas nunca dependeram de acento). *O sweep é a rede; quem julga é o R.*

#### Histórico (`--git-history`), medido — para a decisão da integração

`exit 1`. **36** commits produzem achados (~`136` linhas de patch + `9` de mensagem).

| origem | commits | achados |
|---|---|---|
| **desta linha** (`main..HEAD`, 27 commits) | **5** | **27** |
| — os **três** commits de docs do contorno (v1 · v2 · o report da 1.ª passagem) | 3 | **14** |
| — os **dois** commits da reescrita do INC-4 (obra diferente, mesma linha) | 2 | 13 |
| **já no `main`** (obra mais velha do sculpt3d/tecido) | 31 | 109 |

⇒ **O squash dos três commits de docs do contorno vale a pena** (é barato e tira a redacção
reprovada de um patch) e remove **14 de 136**: ⛔ **não torna o sweep do histórico verde** — o
histórico já está sujo **a montante**, no `main`, e isso não é desta linha. ⛔ E **não reescrever os
commits do INC-4**: um commit que **remove** texto do alvo carrega-o no patch por construção.

⛔⛔ **NADA DE HISTÓRICO FOI TOCADO, e a decisão é do DONO** (ordem à emenda da versão 3,
2026-09-13): o squash acima **não foi feito** — ele fica como recomendação medida, para o handoff
da linha. A emenda da versão 3 acrescenta **um** commit docs-only ao contador desta linha.

### Auditoria R-pré — 3.ª passagem (2026-09-13) — ✅ VERDE, **ATESTADA** (versão 3)

**Quem:** subagente R-pré **novo** (contexto independente do E e das duas passagens anteriores),
despachado pela janela I `9f820704-…`. **Os dois lados lidos por shell** (`sed`/`grep`/`python3`);
⛔ nada escrito em `crates/`, nada de `project-memory/`, nada de histórico reescrito.

**Veredito:** a espec **passa** o §4.2. O atestado está no cabeçalho dela, numa linha greppável
(*«auditada contra §4.2 por R-pré em 2026-09-13»*). **⛔ Não refazer nada do que está abaixo.**

#### O que eu conferi (com o número ao lado)

| # | o quê | resultado |
|---|---|---|
| 1 | **A tabela do §4.1, RE-CONTADA** (achado bloqueante da 2.ª passagem) — as faces de cada um dos `9` `*.repouso.txt.gz` lidas e as arestas de borda contadas do zero por mim | ⭐ **as 6 colunas batem nas 9 linhas** (`738/1 504/0/0` · `1 089/2 112/128/128` ×4 · `25/40/16/16` · `1 089/3 136/128/128` · `800/1 568/64/64` · `385/768/32/32` · `66/97/66/66`); a fórmula do perímetro **produz** cada número a partir do `L × C` nomeado; e a coluna *«o corpus confirma»* bate com o `movidos` do cabeçalho das fixtures (`165` · `15` · `128` · `96` · os dois `0`) |
| 2 | **A linha apagada** (a que a 2.ª passagem acusou) | confirmada contra o alvo: era o conjunto de testes dele, `3` casos na ordem dele, com os valores esperados dele — e o rótulo de facto não produzia os números. **Não há resíduo dela na versão 3** |
| 3 | **As 4 curas da 1.ª passagem**, uma a uma, contra o fonte | ✅ as duas justificações das recusas (§5.2) são hoje argumento de geometria **diferente** do que o alvo guarda; ✅ caíram as três citações do que o alvo diz de si mesmo; ✅ o layout virou classe de custo; ✅ a linha do sobrevoo está re-dita |
| 4 | **As 4 curas da 2.ª passagem** | ✅ §4.5 e §17 já só dizem o que **nós** exigimos (`N`), com a frase que recusa afirmar o que o alvo guarda; ✅ os `8` `D` levam endereço; ✅ as duas referências cruzadas apontam ao sítio certo |
| 5 | **TODO comentário dos três ficheiros do alvo extraído e comparado, um a um, com a espec** | ⭐ **zero prosa de comentário atravessa**. As quatro traduções da 1.ª passagem não têm resíduo; nenhum comentário novo entrou |
| 6 | **Os `8` hashes de commit citados**, contra o corpus público capturado em `notes/` | ⭐ **os 8 existem e o assunto de cada um bate com a afirmação da espec** — incluindo o do §4.4, cujo assunto público **é** a própria afirmação (⇒ a frase *«os autores curaram-no exactamente assim»* é `D` legítimo, **não** leitura de código: eu levantei-a como suspeita e ela caiu com o assunto do commit na mão) |
| 7 | **Pseudo-código espelhado** — comparei a escolha da âncora, as duas recusas, o modo de alisar e a função de queda no contorno com o que o alvo faz | ⭐ nenhum espelha: sem intermediários nomeados, sem a ordem de escrita dele, e a espec reparte as fases de outra maneira. É o algoritmo (§4.1.11), que **pode** ir a qualquer profundidade |
| 8 | **Nomes** | `grep` de todo identificador em crase: só nomes das **nossas** fixtures (em português) e os `4` nomes públicos declarados no §0. ⭐ As curvas de queda (`SMOOTH`/`SHARP`/…) já são **vocabulário nosso** desde antes desta obra — `ph2d-sculpt3d/src/falloff.rs` —, logo não são «nomes conservados» por conservar |
| 9 | **~25 outros números** da espec (§8.3, §8.4, §9.3, §10.1, §10.5, §12.3, §12.4, §13.1, §13.2, §13.3, §14.2) contra o cabeçalho das fixtures | todos batem |
| 10 | **A régua das fixtures** (`confere_cabecalhos.py --check 59`) | `exit 0`, `68` fixtures, `59` excepções |

⭐ **O conjunto de sinais do laço invertido (§8.3) foi RE-EXAMINADO e a ordem da 2.ª passagem
(«⛔ não o corrijas») CONFIRMA-SE**: a tabela medida cobre um período inteiro (`0..4`), logo o
resto do conjunto sai por periodicidade e é **derivado**, não copiado. ⚠️ Eu levantei-o como
achado antes de ler o ledger — *e o ledger já tinha a resposta, com o mecanismo*. Fica aqui a
prova de que a nota serviu: **ela impediu uma correcção que teria desfeito trabalho já pago.**

#### As DUAS notas que **não bloqueiam** (nenhuma é §4.2; nenhuma expõe o alvo)

1. **O documento usa um QUINTO selo de proveniência que a legenda do §0 não declara.** O §0 declara
   `F`/`M`/`D`/`N`; o corpo usa **`leitura`** em três sítios (cabeçalho do §4.2, cabeçalho do §5.1 e
   a linha `4` da tabela do §13). O acto que ele nomeia é **legítimo** (SKILL §4.1.11: reconstruir
   da compreensão e reabrir só para confirmar factos) e é **honesto** — é exactamente o selo que
   avisa o Implementador de que ali **não há fixture nem fonte pública** —, mas a legenda não o
   explica. ⚠️ Ele sobreviveu às duas emendas porque as duas mexeram no **corpo** das secções e não
   nos **cabeçalhos** delas. **Cura funcional (uma linha, sem mudar conteúdo nenhum):** acrescentar
   ao §0 a entrada do selo, dizendo que é reconstrução do E a partir da leitura, sem fixture e sem
   fonte pública, e que **quem implementar reconfere contra o oráculo**. ⛔ Não precisa de R-pré
   novo: não move uma afirmação do documento.
2. **O sweep é CEGO À CAIXA, e aqui morde.** Medido por mim: das `229` entradas, **`56`** levam
   pelo menos uma maiúscula (`32` só na inicial). Escrevendo as mesmas `56` com a **inicial em
   minúscula** — que é o que acontece a uma frase citada a meio de outra — o sweep acusa **`3` de
   `56`**: ⇒ **`53` entradas evadem**. ⚠️ **O controlo positivo actual não expõe isto**: a versão 1
   lê `6` linhas acentuada, `6` de-acentuada **e `6` em minúsculas** (as que casam não dependem de
   caixa). **Cura funcional:** ou o gémeo de caixa para cada entrada de prosa (como se fez com os
   acentos: `229` → ~`285`), ou `-i` no `grep` **só** para as entradas de prosa (⛔ não para as de
   identificador: `113` das entradas têm forma de identificador e `-i` global compra falsos
   positivos). ⚠️ É **defeito do instrumento partilhado**, não desta espec — a mesma cegueira está
   nomeada e por curar numa espec irmã desta jornada, e a decisão é de quem possui o script.

#### O instrumento, re-corrido por mim (números, não promessas)

| corrida | resultado |
|---|---|
| sweep sobre a espec v3 **+** o directório inteiro das fixtures (`79` ficheiros), vassoura `229` | ✅ `exit 0` |
| controlo positivo: versão **1** (`23bb85670`) | `exit 1`, **`6`** linhas |
| a mesma versão 1 **de-acentuada** | `exit 1`, **`6`** linhas ⇒ ⭐ a cura do E confirma-se (era `2`) |
| a mesma versão 1 **em minúsculas** | `exit 1`, `6` linhas (⚠️ ver nota 2: este controlo **não** mede a cegueira de caixa) |
| ⛔⛔ sweep sobre a versão **2** (a que a 2.ª passagem REPROVOU) | ✅ **`exit 0`** — *o instrumento nunca poderia ter apanhado nenhum dos 4 achados dela.* **É a lição a levar para a SKILL: o sweep mede uma classe (string idiossincrática do alvo) e a parede tem outras três** (tabela copiada, selo mal aplicado, afirmação sobre o código do alvo) — **quem julga é o R** |

### Auditoria R-pré (2026-09-13) — ⛔ REPROVADA

**Alvo confirmado por shell:** binário `/usr/bin/blender` = **5.2.1 LTS** (build 2026-09-01); fonte
em `/home/enio/Documentos/Recursos/BlenderSculpt` na tag **v5.2.0** (`fbe62287`), **fora** de toda
árvore do PH2D. Os dois lados lidos por shell (o `deny` da linha bloqueia `Read`).

#### ⛔ O instrumento estava cego: a vassoura media só a língua do alvo

O sweep de entrega do E fechou **verde** com `163` entradas — e a espec contém **quatro** traduções
frásicas de **comentários do fonte**. Das `163`, ~`100` são frases de prosa e **todas em inglês**,
enquanto a espec é escrita em português: *a vassoura não podia, por construção, ver esta classe*.
(Mesmo achado da obra irmã `blender-pose`/`blender-unblocked` desta janela; lá a vassoura foi de
`112` para `137`.)

⇒ **Vassoura alargada de `163` para `199`** (36 entradas de prosa em **português**, base64).
⚠️ **Regra de desenho, deliberada:** só entram formas PT de prosa de **comentário/TODO do fonte**,
que não tem direito de citação nenhum (§4.2: *«comentários do original — a expressão mais protegida
do arquivo»*). ⛔ **Não** entram formas PT de prosa do **manual, mensagem de commit ou tracker**:
essa o §4.1.12 permite citar curto e com fonte, e pô-la na vassoura faria o instrumento acusar um
acto permitido.
⭐ **Controlo positivo e negativo corridos:** com a vassoura nova o sweep acusa **exactamente** os 4
sítios achados à mão (`exit 1`) e **nada mais** — zero falsos positivos sobre a espec, sobre as `78`
fixtures e sobre a árvore rastreada inteira (`git grep -F` de cada entrada nova: só a própria espec).

#### Os 4 achados (o mecanismo é UM: prosa do FONTE a atravessar a parede)

| # | onde | o que é | por que não pode |
|---|---|---|---|
| 1 | §5.2, coluna *«porquê (D)»*, as 2 linhas | as duas justificações são a tradução PT dos **dois comentários** que estão por cima dos dois testes de recusa em `sculpt_boundary.cc` | §4.2 (comentário do original) **+** proveniência falsa: nenhuma das duas frases existe no manual, em commit ou em issue — conferi o corpus público inteiro que o próprio E capturou em `notes/`. O `(D)` do §0 da espec é *manual / mensagem de commit / rastreador* |
| 2 | §5.2 (l. 173) · §13.6 (l. 711) · §14.4 (l. 768–769, e o §16.1 que lhe aponta) | a espec cita **o que o alvo diz dentro do próprio fonte** — dois `TODO` e um comentário —, e em §14.4 com o selo `(D)` | mesma §4.2 + proveniência falsa. ⚠️ Em §14.4 a frase seguinte (*«a nossa implementação pode fazer melhor: depois da fase D o conjunto alcançado é conhecido»*) **é o conteúdo do `TODO` do alvo** apresentado como ideia nossa — o caso mais caro dos quatro |
| 3 | §14.4 (l. 765) | *«três arrays do tamanho do número de vértices»* | é o **layout interno** do alvo (a struct de trabalho tem exactamente três arrays paralelos, e eu confirmei-o no header e nos `BLI_assert`) — §4.2 «organização/structs». O facto funcional é a **classe de custo**, não a contagem |
| 4 (menor) | §7.4 | *«desenha como uma linha branca … para o artista ver até onde a deformação chega (D, manual)»* | é a tradução de **uma frase inteira** do manual. O conteúdo é legítimo e **observável**; o que não pode é a forma. §4.1.12 pedia aspas + fonte, ou re-dizer |

#### O que foi CONFERIDO e está LIMPO (não refazer)

- ⭐ **Nomes conservados:** `boundary_deform_type` · `boundary_falloff_type` · `boundary_offset` ·
  `deform_target` e os valores de enum são todos identificadores **RNA públicos** (Python API),
  verificados um a um em `rna_brush.cc`. A declaração do §0 da espec está **correcta** (§4.1.13).
- ⭐ **Zero nome interno** na espec (sweep das 163 entradas de identificador: limpo).
- ⭐ **Matemática:** as seis leis do §10, a queda do §8.1, o par `LOOP`/`LOOP_INVERT` do §8.3 (o sinal
  bate com a aritmética do alvo **e** com a tabela medida) e o encaixe de ângulo do §9.3 são
  matemática, e a espec traz a medição ao lado (§4.1.2 / §4.1.11). ⛔ **Não** espelham corpo de
  função: sem intermediários nomeados, sem a sequência de recolha/filtro/dispersão do alvo.
- ⭐ **Decomposição em fases:** a ordem A→H é **forçada por dependência de dados** (âncora → cadeia →
  propagação → peso → lei → aplicar), logo não é a «organização arbitrária» do §4.2; e a espec
  reparte diferente do alvo (funde a inicialização por-modo no §10 e separa avanço/lei/aplicar).
- ⭐ **Prosa pública bem tratada:** o eixo Y/Z do manual (§10.1, §10.5), os triângulos «imprevisíveis»
  (§13.2), a geometria escondida (§4.2 da espec), os vizinhos por **aresta** de borda (§4.4), a
  auto-máscara (§8.2), o laço removido (§6.3), a primeira passagem que as seguintes desfazem (§11.2)
  e a região de simetria (§12.2) — **todas** conferidas contra o corpus público capturado: existem no
  manual / em commit / em issue, e estão **re-ditas** ou citadas curto com atribuição. Legítimas.
- ⭐ **Fixtures:** os `78` ficheiros e as chaves de cabeçalho estão em vocabulário **do domínio** e em
  português (`superficie`, `modo`, `queda_no_contorno`, `arrasto`, `movidos`…) — ⛔ nenhuma chave,
  nenhum nome de ficheiro do alvo (§4.2 cobre os dois). Limpo.

#### Nota de higiene (não bloqueia)

Várias entradas `(D)` citam só uma **data** (§4.2 da espec «2020-09-06», §4.4 «2025-11-19», §10.1
«2024-08-01») onde o §4.1.12 pede o **endereço**. As que citam número de defeito estão bem. Ao
reescrever, pôr o endereço ao lado de cada `(D)` restante.

## Espec

| versão | caminho | commit | estado |
|---|---|---|---|
| 1 | `docs/3D/cleanroom/SPEC_boundary_brush.md` | `23bb85670`, docs-only, `line/sculpt3d`, 2026-09-13 | ⛔ **REPROVADA** pelo R-pré, 1.ª passagem (4 achados) |
| 2 | o mesmo caminho | `881e96d9e`, docs-only, 2026-09-13 | ⛔ **REPROVADA** pelo R-pré, 2.ª passagem (4 achados, **1 bloqueante**) |
| 3 | o mesmo caminho | commit docs-only de 2026-09-13 (esta emenda) | ✅ **ATESTADA** pelo R-pré, 3.ª passagem, 2026-09-13 (VERDE; duas notas que não bloqueiam) — a janela PODE implementar |

**Filtragem §4.3 executada** em 2026-09-13, secção a secção: cada frase responde *o que o programa
FAZ*; cada número traz `F` (fórmula) · `M` (medido, com a fixture) · `D` (documentado pelos autores,
com o endereço) · `N` (decisão nossa). **Sweep verde** sobre a espec, as fixtures, o README delas e
o texto do report final — ⚠️ **e a vassoura que o mediu mudou duas vezes**: `163` (v1, **cega** à
nossa língua) → `199` (v2, cega ao texto **sem acentos**) → **`229`** (v3). *Um «sweep verde» sem o
número da vassoura ao lado não diz nada* — o verde da v1 e o da v3 são medições diferentes.
⚠️ **E `D` deixou de poder vir do fonte, por cláusula escrita no §0 da espec** (versão 3): manual,
commit e rastreador; ⛔ nunca um ficheiro lido dentro da árvore do alvo.

⚠️ **Citações verbatim na espec: ZERO.** Os factos que vêm do manual e das mensagens de commit
públicas estão **re-ditos em palavras nossas** com o endereço ao lado (o direito de citação estava
disponível e não foi preciso usá-lo). ⛔ Nenhum trecho de código, nenhum comentário do alvo, nenhum
nome interno. Os únicos nomes do alvo que a espec conserva são **públicos** (valores de enum da API
e nomes de propriedade), declarados como tal no §0 dela, ao abrigo da SKILL §4.1.13.

## Incidentes

(vazio)
