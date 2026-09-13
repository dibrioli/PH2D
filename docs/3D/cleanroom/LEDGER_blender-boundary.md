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

## Corrente I

| janela | session-id | data | motivo | declaração |
|---|---|---|---|---|
| I (janela-mãe) | `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-13 | despachou este E (e três irmãos) | pelo **inbox** (entrada de 2026-09-13, transcrita acima) |

## Papel R

| papel | id | data |
|---|---|---|
| R-pré | ⏳ | — |
| R-pós | ⏳ | — |

## Espec

| versão | caminho | commit |
|---|---|---|
| 1 | `docs/3D/cleanroom/SPEC_boundary_brush.md` | commit único, docs-only, `line/sculpt3d`, 2026-09-13 |

**Filtragem §4.3 executada** em 2026-09-13, secção a secção: cada frase responde *o que o programa
FAZ*; cada número traz `F` (fórmula) · `M` (medido, com a fixture) · `D` (documentado pelos autores,
com o endereço) · `N` (decisão nossa). **Sweep verde** sobre a espec, as `77` fixtures, o README
delas e o texto do report final, com a vassoura de `163` entradas.

⚠️ **Citações verbatim na espec: ZERO.** Os factos que vêm do manual e das mensagens de commit
públicas estão **re-ditos em palavras nossas** com o endereço ao lado (o direito de citação estava
disponível e não foi preciso usá-lo). ⛔ Nenhum trecho de código, nenhum comentário do alvo, nenhum
nome interno. Os únicos nomes do alvo que a espec conserva são **públicos** (valores de enum da API
e nomes de propriedade), declarados como tal no §0 dela, ao abrigo da SKILL §4.1.13.

## Incidentes

(vazio)
