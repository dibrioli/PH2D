# LEDGER de proveniência — clean-room do pincel de tecido (alvo `blender-cloth`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-cloth.md` (append cego).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — *Cloth Brush* e *Cloth Filter* do Sculpt Mode (+ o que os dois invocam que decide comportamento) |
| Versão / commit | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` (2026-07-13), checkout **esparso e grafted** (profundidade 1 — ⚠️ **sem história local**; a história vem da web) |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D. Zona de notas/rascunhos/oráculo: `~/Referencias/blender-cloth/` |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença | **GPL-2.0-or-later** — `COPYING` remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991) e os ficheiros do pincel levam cabeçalho SPDX `GPL-2.0-or-later` (lido em 2026-09-05) |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS** (build 2026-09-01), Python 3.14.7 — ⚠️ **patch-release acima do fonte lido** (5.2.0); a diferença é registada e o oráculo é o binário |
| Precedente da casa | [ADR-0162](../../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) — oráculo fora da árvore |

### A concessão relevante (GPLv2 §0 e §2), transcrita do ficheiro do checkout

> *"Activities other than copying, distribution and modification are not covered by this
> License; they are outside its scope. The act of running the Program is not restricted,
> and the output from the Program is covered only if its contents constitute a work based
> on the Program (independent of having been made by running the Program)."* (§0)
>
> *"You may modify your copy or copies of the Program or any portion of it, thus forming a
> work based on the Program…"* (§2)

⇒ Ler, correr, modificar e instrumentar **em privado** é licenciado. A **saída** do
programa (posições de vértices exportadas, dumps) **não** é obra baseada no programa ⇒ é
dado (SKILL §1.1/§5). Nenhum acto deste ledger envolve distribuição. Não é AGPL.

---

## §2 — Triagem: a escada de portas

| degrau | veredito | por quê |
|---|---|---|
| T0 | ⛔ não para o pincel | o pincel só existe neste alvo, GPL |
| T0 (solver) | ✅ **existe uma porta permissiva para o SOLVER** — `newton-physics/newton`, Apache-2.0 (triagem do dia, feita pela janela-mãe, 2026-09-05) | vale para a família do solver (restrições por projecção); ⛔ **não** para a semântica do pincel, que é o que esta espec fecha |
| T1 | ⛔ nenhum irmão permissivo do *pincel* (a espec pública `docs/3D/cloth/04` §E.1 já mediu: Nomad não tem, SculptGL não documenta) | — |
| **T2** | ✅ **é o degrau desta obra** — copyleft com fonte | o dono pediu explicitamente o estudo do fonte |

Registado **antes** de qualquer leitura do fonte, em 2026-09-05.

---

## Patente (§8.1) — checkpoint incondicional

- **Buscado em:** 2026-09-05
- **Termos:** `sculpting brush cloth simulation` · `cloth brush constraints local simulation area` ·
  `position based dynamics sculpting brush cloth wrinkles` · `ZBrush cloth dynamics brush patent` ·
  `site:patents.google.com cloth simulation sculpting brush region constraints`, cruzados com
  Pixar, Pixologic/Maxon, Autodesk, Adobe, Sony e os autores do alvo.
- **Resultado:** ⭐ **nenhuma patente viva alcança o método** (pincel que escreve forças e alvos de
  restrição numa simulação de pano por relaxação de restrições, confinada a uma área à volta do
  pincel). Quatro achados, com veredito:

| patente | dono | estado | lê sobre nós? |
|---|---|---|---|
| **US 10 586 401 B2** — pincéis por soluções regularizadas de elasticidade linear (Kelvinlets) | Pixar (de Goes & James) | **VIVA até 2038-05-02** | ⛔ não — a reivindicação exige soluções **analíticas fechadas** da elasticidade linear (com razão de Poisson), «livres de discretização geométrica»; o nosso método é uma simulação **discreta por restrições**. ⚠️ **cerca nomeada**: nunca implementar um modo «elástico» por Kelvinlet |
| **US 10 713 855 B2** — criar/editar superfícies que representam **vestuário sobre um manequim** por escultura + simulação de pano | Audaces (⚠️ **empresa BRASILEIRA** — jurisdição-alvo) | **VIVA até 2036-09-22** | ⛔ não — a reivindicação 1 exige (a) um **manequim vestido** com uma superfície de peça inteira, (b) escultura que **acrescenta/remove triângulos**, (c) simulação de pano, (d) em qualquer ordem, (e) **impedir sempre** a malha de entrar no interior do manequim. O nosso pincel deforma uma escultura arbitrária, sem manequim, sem garment, sem o passo (e). ⚠️ **cerca nomeada e a mais próxima**: se algum dia a casa fizer uma ferramenta de «vestir um corpo» com colisão contra o corpo, **refazer esta busca com parecer humano** (§8.5) |
| US 7 830 375 B2 — esquemas de restrição (vértices *skinned* como restrição) | Sony Interactive | **EXPIRADA** (2025-08-02) | não lê; e expirada ⇒ literatura livre |
| US 8 140 304 B2 — simulação de pano com modelo linear de esticão/cisalhamento | — | não avaliada a fundo: modelo de material de simulação offline, sem pincel | não lê sobre um pincel |

⇒ Veredito: **prosseguir**. Sem «PATENTE VIVA» a reportar.

---

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E** despachado pela janela-mãe (a janela I da `line/sculpt3d`) |
| session-id da janela-mãe | `1246816c-63cf-414b-842d-663a8baa86ca` |
| transcript do subagente (⛔ **zona contaminada** — I nunca lê) | dentro de `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/1246816c-63cf-414b-842d-663a8baa86ca*` (ficheiros de subagente) |
| aberto em | 2026-09-05 |
| lê | o fonte do pincel e do filtro de tecido, o que eles invocam que decide comportamento, defaults/faixas (DNA/RNA), o painel (Python), o cursor, o undo — por shell (`cat`/`rg`/`sed`), porque o `deny` da linha nega `Read` |
| escreve | `SPEC_cloth_brush.md` (commit único, pós-filtragem) · este ledger · `VASSOURA_blender-cloth.txt` · fixtures do oráculo · `README` de 3 linhas |
| ⛔ nunca | código de produto; `git push`; `git add -A`; `git stash` |

### Cobertura da travessia (§3.E) — 2026-09-05, por shell (`cat`/`sed`/`grep`), fonte v5.2.0

| área | ficheiros (caminho relativo a `source/blender/` ou `scripts/`) | linhas | lido |
|---|---|---|---|
| **o pincel + o filtro de tecido** | `editors/sculpt_paint/mesh/sculpt_cloth.cc` · `.hh` | 2 590 + 167 | ⭐ **INTEIROS**, do 1.º ao último byte (4 chunks + o header) |
| o que o pincel invoca e decide comportamento | `editors/sculpt_paint/mesh/sculpt.cc` | 8 417 | as regiões: testes de passo/simetria (650–672) · alinhamento à normal (690–730) · esfera-vs-nó (2579–2600) · força do pincel por tipo (2300–2390) · normal de escultura (2722–2760) · plano do pincel (3048–3130) · máscara de nós (3270–3290) · alvo de deformação de outros pincéis (3460–3490, 3640–3670) · despacho (3585–3600) · delta de agarrar (4180–4320) · flips de simetria (3675–3725) · escala/gravidade/vista (5680–5750) · raio por passo (5860–5905) · restauro anti-âncora (5170–5205) · leitura da localização por passo (5835–5850) · raio em espaço-objecto · vizinhos · o pipeline partilhado de factores (7202–7600, 7737, 7950) |
| contratos do pipeline partilhado | `editors/sculpt_paint/mesh/mesh_brush_common.hh` | — | 60–80, 295–425 |
| traço: espaçamento e exec scriptado | `editors/sculpt_paint/paint_stroke.cc` | 1 777 | 160–215, 600–720, 975–1015, 1629–1700 |
| cursor | `editors/sculpt_paint/mesh/paint_cursor.cc` | 820 | 290–320, 738–760 |
| filtro: cache, orientação, eixos, props | `editors/sculpt_paint/mesh/sculpt_filter_mesh.cc` · `sculpt_filter.hh` | 2 737 | 80–200, 2600–2710 (props) |
| outros pincéis com alvo = simulação | `editors/sculpt_paint/mesh/sculpt_pose.cc` (190–212) · `sculpt_boundary.cc` (1150–1170, 1220–1235) | — | os braços do alvo |
| kernel do pincel: curvas de falloff, classes de pincel, gravidade, persistência | `blenkernel/intern/brush.cc` | 2 018 | 1478–1560, 1608–1660, 1780–1815, 1890–1900, 1994–2005 |
| defaults/faixas/enums | `makesdna/DNA_brush_types.h` (387–398) · `DNA_brush_enums.h` (170–207, 400–404, 464) · `makesrna/intern/rna_brush.cc` (2785–2800, 3411–3450, 3656–3670) | — | ✔ |
| painel e barra de ferramentas | `scripts/startup/bl_ui/properties_paint_common.py` (965–983) · `space_toolsystem_toolbar.py` (1914–1930) | — | ✔ |
| base persistente (operador) | `editors/sculpt_paint/mesh/sculpt_ops.cc` (85–155) | — | ✔ |
| ⛔ NÃO lido, de propósito | `blenkernel/intern/cloth*` (o modificador de pano do Blender — **outro** solver, não é o alvo) · `extern/quadriflow` · o resto de `sculpt.cc` | — | fora do alcance da espec |

**História (web, porque o checkout é grafted):** as **104** mensagens de commit que tocaram o ficheiro do
pincel desde 2020-02-28 (API Gitea, 3 caminhos históricos: `sculpt_cloth.c` → `.cc` → `mesh/…`), com o
corpo integral de **39** delas guardado em `~/Referencias/blender-cloth/notes/commits.txt` · os dois posts
do blog dos programadores (2020-02-25 e 2020-10-20, guardados em `notes/blog_*.txt`) · **80** issues do
tracker por «cloth brush»/«cloth filter» (`notes/issues_cloth_brush.txt`). ⚠️ As páginas de revisão
(D6715, D8424…) devolvem **403** a fetch automático — o conteúdo delas está nas mensagens de commit
correspondentes, que as citam.

**Brush assets:** os **13** pincéis de tecido da biblioteca *Essentials* do binário 5.2.1 foram **lidos por
`bpy`** (valores, nunca copiados como ficheiro) — são os defaults que o artista de facto vê, e diferem dos
defaults do código em `damping`, `strength`, `spacing`, área e plasticidade. ⛔ Os `.blend` são assets
(§8.3) e não entram no repo; só os NÚMEROS entram na espec, como facto observado.

---

## Oráculo (§5)

| campo | valor |
|---|---|
| binário | `/usr/bin/blender` 5.2.1 LTS (⚠️ patch acima do fonte 5.2.0 lido; nenhum commit de comportamento do pincel entre os dois — a lista do Gitea termina em 2026-08-04 com um rename) |
| harness | `~/Referencias/blender-cloth/oracle/harness.py` — corre **com janela** (o modo `-b` não tem contexto de vista 3D e o traço scriptado recusa; medido) sobre a tela real, ortográfica, e sai sozinho |
| entrada | ⭐ **NOSSA**: grelha plana 64×64 (lado 3,0) e esfera UV 96×64 (raio 1), geradas no próprio script; o pincel é o asset *Drag Cloth* **anexado só para ter um pincel de tecido activo** (a API não deixa criar+activar um pincel novo), com TODOS os parâmetros reescritos para os defaults do código |
| saída | `oracle/out/*.npz` (repouso, deformado, caminho, settings) — 30 corridas: 8 modos × {plano, esfera} + plano-falloff × 4 + 1-passo × 5 + massa/damping/pino/global/dinâmico/força/densidade de passos |
| o que vira fixture | ver `docs/3D/cleanroom/fixtures/cloth/README.md` (proveniência de ENTRADA nossa; saída = dado) |

## Corrente I

| janela | session-id | data | motivo | declaração |
|---|---|---|---|---|
| I-1 (janela-mãe) | `1246816c-63cf-414b-842d-663a8baa86ca` | 2026-09-05 | abriu a obra e despachou este E | ⏳ a janela declara pelo **inbox**: *"nenhum conteúdo do fonte do alvo entrou no CONTEXTO desta janela (incluindo reports de subagentes e compactação); exposição via pesos do modelo não é atestável por construção — mitigada §7.3"* |

---

### Achados de PAREDE para o R (registados pelo E em 2026-09-05)

1. ⚠️ **O nome do ficheiro do pincel do alvo já existe em CINCO sítios da árvore, antes desta obra** —
   três doc-comments em `crates/ph2d-sculpt3d/src/` (`brush_verb_defaults.rs:183`,
   `stroke_dab_core.rs:303`, `verb_layer_front_face_tests.rs:16`), um handoff de 2026-08-16 e o
   `docs/3D/cloth/01_pesquisa_o_estado_da_arte.md:167`. São citações **nominativas do nome do
   ficheiro** (proveniência de uma lista de pincéis), sem mecanismo — a mesma família que o
   [`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md) já mediu no
   repo inteiro. A vassoura desta obra inclui o nome (é identificador interno, §4.2), logo o sweep de
   ÁRVORE do R vai acusá-los: **veredito do R**, não do E (o E não edita código de produto nem docs de
   outras linhas).
2. O sweep dos artefactos entregues por este E (espec, fixtures, README, INBOX, report) correu
   **limpo** contra as 70 entradas.
3. O oráculo revelou que o traço scriptado do binário **ignora o tamanho travado do pincel quando o
   tamanho unificado está ligado** (omissão de fábrica) — a primeira corrida mediu um raio de `~0,2`
   em vez de `0,35`, e foi descartada; a matriz final força os dois. *Um harness que não confere o
   raio que pediu mede outro programa.*
4. E o segundo defeito do harness, do mesmo tipo: o centro da área *Local* é o ponto de HOVER do
   cursor (escrito só pelo desenho do cursor com normal amostrada), que um traço scriptado nunca
   actualiza — a 2.ª matriz mediu áreas centradas na ORIGEM do objecto (o Grab radial na esfera
   movia **zero** vértices). Cura: harness por temporizador, cursor do sistema movido ao pixel do
   pen-down e um redesenho antes de cada traço; validado com uma varredura de cinco posições de
   pen-down (todas a agarrar no sítio certo). *Um operador de sondagem que salta o laço de eventos
   mede um programa que o artista nunca corre.*

## Papel R

| papel | id | data |
|---|---|---|
| R-pré | ⏳ | — |
| R-pós | ⏳ | — |

---

## Espec

| versão | caminho | commit |
|---|---|---|
| v1 | `docs/3D/cleanroom/SPEC_cloth_brush.md` | `c7905f616` (2026-09-05, commit único pós-filtragem) |

---

## Incidentes

**INC-1 (2026-09-05, registado pelo subagente-E) — material do alvo no scratchpad partilhado (§3.E).**
- **Origem:** o scratchpad da sessão (`/tmp/claude-1000/.../1246816c-.../scratchpad/`) continha, ANTES
  desta obra, ~120 artefactos de pesquisa do alvo criados por uma janela anterior (a auditoria de
  `docs/3D/cloth/` do MESMO dia): páginas de manual/API/notas-de-versão/blog (factos públicos) **e**
  páginas de revisão/commit do repositório do alvo que **carregam diff de código** (as duas revisões
  públicas e seis páginas de commit, por hash). O scratchpad é alcançável pela janela-mãe ⇒ é o vector
  de contaminação que o §3.E proíbe (*«nada do alvo... em /tmp nem no scratchpad»*).
- **Régua de substancial:** as 8 páginas de revisão/commit são **substanciais** (contêm corpo de
  função/diff). As páginas de manual/API/blog são factos públicos (relance).
- **Acção deste E:** ⛔ **não li o conteúdo** de nenhuma delas (o registo DESCREVE, não reproduz). As 8
  páginas de código foram **relocadas por `mv` cego** para `~/Referencias/blender-cloth/prior_scratchpad_recovered/`
  (zona contaminada, fora do repo e fora do /tmp), sem entrarem no meu contexto. As restantes (factos
  públicos, mais fixtures de OUTRA obra — malhas `.obj`, backups `.bak` de crates NOSSAS, scripts `.py`)
  foram deixadas onde estavam, para não quebrar trabalho vivo da janela-mãe.
- **Veredito para o R e para o Enio:** ⚠️ isto é um achado sobre a **janela-mãe/auditoria anterior**, não
  sobre esta espec. A espec, o ledger, a vassoura e as fixtures desta obra nasceram todos em
  `~/Referencias/` ou no repo, e o sweep de árvore e de histórico corre **verde**. A recomendação é o
  operador **esvaziar o scratchpad da sessão** (ou o que sobra do alvo nele) antes da próxima janela — e
  o doc `04_espec_do_comportamento.md`, que afirma *«nenhum código-fonte do alvo foi aberto por este
  agente»*, ser reconciliado com a existência daquelas páginas de revisão/commit no scratchpad da sessão
  que o produziu.


---

## Fechamento R

⏳
