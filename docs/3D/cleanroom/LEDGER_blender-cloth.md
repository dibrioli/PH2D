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
| I-1 (janela-mãe) | `1246816c-63cf-414b-842d-663a8baa86ca` | 2026-09-05 | abriu a obra e despachou este E | ⏳ a janela declara pelo **inbox**: *"nenhum conteúdo do fonte do alvo entrou no CONTEXTO desta janela (incluindo reports de subagentes e compactação); exposição via pesos do modelo não é atestável por construção — mitigada §7.3"* · **INC-1 (2026-09-05, via briefing do R-pré):** *«a janela I não abriu nenhum dos ficheiros quarentenados; leu apenas a listagem de nomes»* — e o R mediu que nenhum deles continha código (ver *Incidentes*) |

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
| R-pré | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` (transcript = zona contaminada — leu o fonte por shell) | 2026-09-05 — ✅ **atestado no cabeçalho da espec**; veredictos abaixo |
| R-pré (errata) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu a errata (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestado no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (errata)» |
| R-pré (emendas Q8/Q9/Q10) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu as emendas (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestado no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (emendas Q8, Q9 e Q10)» |
| R-pós | ⏳ | — |

### Auditoria R-pré — 2026-09-05

**Método.** Espec inteira + anexos lidos; o fonte lido por shell nas regiões que a espec descreve com
pseudo-código (o passo principal, a relaxação, a integração por vértice, o kernel de forças, a banda, a
lista de funções e a de comentários do ficheiro do pincel, e o header) — comparada a EXPRESSÃO, nunca o
comportamento. Sweep (70 entradas) sobre espec · `fixtures/cloth/` · INBOX · os dois READMEs ·
`docs/3D/cloth/02–04`: **verde**. `--git-history` sobre `docs/3D/cleanroom` + `docs/3D/cloth`: só o nome
do ficheiro do pincel (a mensagem de commit do `01` e o patch DESTE ledger, que o carrega de propósito).
Verificador das fixtures: 46/46 OK.

**Veredictos §4.2, por espécie.**
- *Pseudo-código.* §5.2 é a projecção de distância do PBD (Müller 2007 §3.3) com massas iguais e
  rigidez `0,6`; os intermediários estão em ordem de dependência de dados, não arbitrária ⇒ **fórmula, não
  tradução**. §5.4 é o passo de Verlet por posições de Jakobsen 2001 mais os três factores do alvo; a única
  ordem não forçada (aceleração vs. velocidade retida) é comutativa ⇒ idem. §2.2 é um smoothstep ⇒ idem.
- *Organização.* Por fases funcionais com dependências de dados; **não** segue a ordem de funções do
  ficheiro (utilitários de grelha → banda → restrições → forças → colisão → solver → gesto → passo →
  cursor → filtro). ✔
- *Nomes internos.* Nenhum (os únicos `snake_case` são nomes NOSSOS de fixture e símbolos). *Tabelas.*
  §8.1 = defaults/faixas (facto §4.1.3) · §8.2 = valores lidos por `bpy` (facto observado) · §10 =
  medição nossa. ✔ *Wording.* O §9 re-diz 23 mensagens de commit/blog públicas com citações curtas
  entre «» e fonte (§4.1.12). ✔
- ⛔ **UM achado de expressão:** no §7 (linha *Gravity* do filtro) uma frase de COMENTÁRIO do fonte estava
  citada entre «» e marcada (F) — §4.2 proíbe comentários mesmo curtos. **Curado pelo R-pré**: re-expressa
  como comportamento (o eixo da gravidade na orientação de vista), sem as palavras do comentário.

**Higienes §4.3 (como-o-autor-escreveu ⇒ isco de convergência §7.3), curadas pelo R-pré:** (1) §3.3 «há
uma constante nomeada para isso no fonte, não usada» → só o facto; (2) §5.5 as linhas «reserva inicial
100 000 (só um reserve)» e «teto por vértice 1024, declarado e não usado» removidas — nenhuma é observável;
(3) §3.1 e (4) §13 «construção mono-thread de propósito» → «ordem determinística» (o comportamento);
(5) §4.3 «(o array é limpo antes de cada passo)» removido; (6) §4.1 «nesta ordem» → «produto dos
factores», com a única ordem que importa nomeada (dureza antes da curva).

**Anexos (coerência — o I lê-os), curadas pelo R-pré:** o §14 citava **6 fixtures inexistentes** (nomes
do harness) → renomeadas às reais; gate 15 «30 dumps» → 46; README das fixtures com **5 linhas** da
tabela em nome de harness (`_2steps` · `_mass2_1step` · `_str05_1step`) → nomes reais; `analise.json`
tinha um TERCEIRO vocabulário (inglês do harness) e não estava documentado → renomeado por **junção
verificada** com `indice.json` (46/46 iguais em movidos/máximo/passos, `assert`) para o vocabulário das
fixtures e das colunas do §10, com o nome de harness guardado em `corrida_oraculo` (para o E regenerar);
os dois JSON agora documentados no README. ⚠️ A linha «30 corridas» da tabela *Oráculo* acima envelheceu:
são **46** fixtures (o E acrescentou variantes depois de a escrever).

**Achado de parede nº 1 do E — veredito do R:** `sculpt_cloth.cc` existe em **6** sítios da árvore
rastreada (3 doc-comments em `crates/ph2d-sculpt3d/src/`, `docs/3D/cloth/01`, o handoff LAYER de
2026-08-16 e este ledger — de propósito) e nos `.rlib/.rmeta` do scratchpad da sessão (os MESMOS
doc-comments, compilados pelo rustdoc — não é exposição nova). É **Classe A** do
[`ACHADO_proveniencia`](ACHADO_proveniencia_por_nome_interno.md) (citação nominativa de endereço, sem
transcrição): **higiene, não violação, não incidente.** Curei o `01` (doc vivo que o I lê). Para o R-pós:
os 3 doc-comments + o handoff são 4 edições de uma expressão («o ficheiro do pincel de tecido do
Blender»), a fazer pelo I ou pelo R-pós — o I já tem o nome na própria árvore, não é exposição; e o patch
deste ledger no histórico é a excepção declarada do §6 — nomeá-la no fechamento em vez de a contar.

**Declaração da janela I sobre o INC-1 (transmitida pelo briefing do R-pré, 2026-09-05):** *«a janela I
não abriu nenhum dos ficheiros quarentenados; leu apenas a listagem de nomes»* — verificada pelo R na
secção *Incidentes*. ⚠️ O INBOX está **vazio**: a declaração geral do §6 da Corrente I ainda não foi
apendada pela janela; fica ⏳ até o I a escrever no canal.

### Auditoria R-pré — 2026-09-06 (a ERRATA `3d621e94b` + `d5844ad5c`)

**Âmbito:** o diff `622df9c52..d5844ad5c` sobre a espec (§2.1 · §3.1 · §5.2 · §10 · gate 15), o README e o
`indice.json` das fixtures, e o INBOX (a declaração da janela I e as seis medições dela). O R-pré da
errata é um contexto novo, independente do subagente que a escreveu, e leu os dois lados (o fonte por
shell).

**Sweep (vassoura de 70 entradas) — VERDE** sobre: a espec · o README · o `indice.json` · o INBOX · a
pasta inteira `fixtures/cloth/` (os 55 ficheiros rastreados **e** os `*.porpasso.*` ainda não rastreados
do I — conteúdo, `strings` e NOMES) · e `--git-history` sobre os cinco caminhos. ⚠️ Os únicos hits do
histórico e do ledger são os **dois nomes de ficheiro do fonte na tabela de cobertura da travessia**
(pré-existentes de 05/09; o §6 EXIGE essa lista, e o I nunca abre o ledger) — não são achado.

**Achados §4.2 — TRÊS nomes internos do alvo, confirmados no fonte por `grep` (1 · 2 · 9 ficheiros),
curados no acto pelo R-pré:**
1. §5.2, bloco «Confirmação de fonte»: o nome da variável local que guarda a metade do vector de
   correcção, entre crases e com a expressão dela ⇒ re-dito como o `h = Δ/2` que o laço da própria
   secção já define («o que cada extremo recebe»).
2. §5.2, mesmo bloco: o nome do campo por-vértice do factor de deformação, com o default dele ⇒ re-dito
   como «vale `1` em toda restrição que não seja âncora de deformação, e só nessas é `(σ_A+σ_B)/2`».
3. §10, errata das esferas: o nome do campo que guarda a localização inicial do traço ⇒ «a localização
   inicial guardada — a célula «centro» do Local na tabela do §2.1».
**Higiene §4.3 — UMA:** o §10 trazia uma linha órfã de saída de arnês (uma nota «em falta» com um nome
inglês que não é de nenhuma fixture, a contradizer a errata que a segue) ⇒ apagada.
**Higiene do ledger:** a entrada Q5 das erratas citava o nome interno do achado 1 ⇒ re-dita.

**Wording de manual/comentário · pseudo-código espelhado · tabela verbatim · organização transcrita:
nenhum.** As duas frases de comentário que a vassoura guarda (o filtro por raio só no Local; as restrições
repetidas) estão RE-DITAS em português funcional, sem citação. O bloco «Local contra Dynamic» do §2.1
enumera cinco diferenças de COMPORTAMENTO (centro · raio · filtro · momento da criação · banda) e não a
decomposição do código. As tabelas do §10 / README / índice são saída do oráculo (§4.1.6), e as três
concordam entre si (47 traços; as 8 esferas com área `dinamica` no índice).

**Os seis factos da errata, conferidos no fonte (R vê os dois lados) — TODOS correctos:**
- **Q1** — a vizinhança sai das **faces poligonais** (o colector recebe as faces e os vértices de canto,
  e percorre por face os dois cantos adjacentes; não recebe triangulação) ⇒ 4 vizinhos num quad interior.
- **Q2** — o factor por vértice é multiplicado pela banda ANTES das varreduras, e nas quatro rotas que
  chegam ao solver.
- **Q3** — o raio que filtra a criação de restrições só é finito na área Local (ilimitado nas outras).
- **Q4** — o tecto de **2 500 faces** por célula-folha é uma constante do construtor da árvore espacial.
- **Q5** — a metade da correcção vale para toda espécie; o 2.º extremo só se move se for um vértice
  DISTINTO do 1.º (numa âncora os dois índices coincidem); o factor por passo é `1` salvo nas âncoras de
  deformação; o corpo mole não o leva (leva a plasticidade); o Grab radial pesa `0,1` pela **curva do
  pincel**, e o de plano leva `0,1` seco (a força vem depois, na aplicação).
- **Q6** — o `indice.json`, o README e a tabela do §10 registam as 8 esferas como área Dinâmica com os
  mesmos `movidos`/`máx` (2 096..2 234 movidos, alcance `3,44..3,52 R`).


### Auditoria R-pré — 2026-09-06 (as EMENDAS Q8, Q9 e Q10)

**Âmbito:** os quatro commits `52e6f75a0` (Q8) · `bdc378b5f` (Q9) · `82ecde1b6` + `9a79c1721` (Q10)
— espec (§1 fases 0/1 · §2.1 · §3.1 · §3.3 · §4.3 · §5.2-bis NOVA · §10.2 · §10.3 NOVA · §10.4 NOVA ·
§10.5 NOVA · §13 · §14 gates 8/12/16/17/18), o README das fixtures e as 6 fixtures novas por passo.
Contexto novo, independente do subagente-E que as escreveu; leu os dois lados (o fonte por shell).

**Sweep (vassoura de 70 entradas) — VERDE** sobre: a espec emendada · a pasta inteira `fixtures/cloth/`
(conteúdo, `strings` e NOMES) · o INBOX · `docs/3D/cloth/` · o README do `cleanroom/`. `--git-history`
sobre esses caminhos: os únicos hits são os **pré-existentes de 2026-09-05** já adjudicados (o nome do
ficheiro do fonte na mensagem de commit do `01` e no patch que o R-pré daquele dia já curou no doc vivo,
mais a tabela de cobertura DESTE ledger, que o §6 exige) — **nada de novo, e a árvore viva está limpa**.

**Achado §4.2 — UM, curado no acto pelo R-pré:** no §10.4 (Q9) a forma da queda do pincel estava nomeada
por um **identificador em forma de código** entre crases. Re-dito em vocabulário do domínio, com o nome
que o artista vê (a queda esférica por omissão, ou a *Projected*, medida no plano da vista) — que é a
mesma palavra que o §4.3 já usava.

**Insuficiência — UMA, curada no acto pelo R-pré:** o gate 16 (Q8) fixava a contagem em `2×` sem dizer de
onde vem o `2`. A regra derivável estava só no corpo (§5.2-bis: `n` passagens de simetria ⇒ `n+1` cópias,
e as fixtures têm `n = 1`); o gate passa a nomeá-la, senão um teste com espelho reprova sobre produto
correcto.

**Wording de manual/comentário · pseudo-código espelhado · tabela verbatim · organização transcrita ·
outros nomes internos: nenhum.** As emendas descrevem o mecanismo em fases funcionais e em vocabulário
nosso (célula · construção · activação · âncora · varredura); os únicos `snake_case` novos são nomes de
fixture NOSSOS. As tabelas do §10.3/§10.4/§10.5 são medição nossa (M) e saída do oráculo (§4.1.6).
⚠️ O bloco que descreve o tempo de vida do registo de pares (§3.3) foi pesado e **fica**: ele é a LEI que
produz o multiconjunto de restrições — comportamento observável por contagem e pela régua de vértices
movidos — e está escrito sem nomear função, ficheiro ou variável do alvo (§4.1.11: o limite é a FORMA,
nunca a profundidade).

**Os factos das três emendas, conferidos no fonte (R vê os dois lados) — TODOS correctos:**
- **Q8.1** — o número de varreduras de relaxação por passo é uma constante única, igual nos três tipos de
  área; não há multiplicador por área.
- **Q8.2** — o registo de pares já criados é **local a uma construção** e a construção só corre para
  células ainda **não activadas**; a activação é um passo SEPARADO, que o primeiro passo do traço nunca
  alcança no ramo *Local* (ele constrói e retorna). ⇒ a mesma célula é construída outra vez no passo
  seguinte e **cada restrição fica em duplicado**; em *Global*/*Dynamic* o primeiro passo **não constrói**
  e a lista nasce simples. A generalização `n+1` para `n` passagens de simetria também confere.
- **Q8 (colateral)** — duplicam-se **todas** as espécies: a de distância porque o registo é novo, e as de
  corpo mole / âncora / pino porque não têm registo nenhum. ✔
- **Q8 (ordem)** — a relaxação corre **antes** da integração dentro do passo, e as âncoras são escritas
  **antes** da relaxação ⇒ a invisibilidade no 1.º passo simulado dos modos de força, e a visibilidade
  imediata nos de âncora, estão certas. ✔
- **Q9.1** — a localização do pincel deixa de ser lida do evento a partir do 2.º passo nos dois modos de
  âncora; no gancho ela é somada ao delta **antes** de o delta deste passo ser recalculado ⇒ o centro é
  `pen-down + Σ_{i<k} δ_i`, um passo atrasado, e no 1.º passo simulado é exactamente o pen-down (o delta
  do primeiro passo é zerado). ✔ E esse centro é de facto o que a queda por-vértice usa como referência.
- **Q9.2** — há uma escolha explícita de posições no kernel de forças: **repouso para o Grab, actuais para
  todos os outros**, e ela alimenta as três coisas que a emenda nomeia (distância, recorte de região,
  textura). ✔
- **Q9.3** — não há eixo, plano nem limite de profundidade próprios do gancho; a des-projecção à
  profundidade original e o achatamento no plano da vista pertencem ao **delta** e valem para os oito
  modos. ✔
- **Q9 (colateral)** — a força por passo das âncoras é zerada em **todo o objecto** nos DOIS modos de
  âncora antes de ser reescrita, e os valores de reescrita (`1` radial / recorte no plano, para o Grab;
  a queda, para o gancho) conferem. A redacção anterior do gate 12 estava **errada** e a correcção é a
  certa. ✔
- **Q10** — é medição nossa sobre o oráculo (fixtures novas com a prova de fatiamento a `0,000000`); a
  leitura do rastreio bate com a lei da força que aponta ao cursor, e a do modo de linha com a lei que
  aperta contra a linha do traço. ✔

**Veredito: ATESTADO.** As três emendas descrevem comportamento, não expressão, e são fiéis.

---

## Espec

| versão | caminho | commit |
|---|---|---|
| v1 | `docs/3D/cleanroom/SPEC_cloth_brush.md` | `c7905f616` (2026-09-05, commit único pós-filtragem) |
| v1-r | idem — atestada; curas do R-pré (1 expressão · 6 higienes · anexos) | `0c884a2b2` (2026-09-05, R-pré) |
| v1-e | idem — ERRATA do E (as 6 perguntas do I; §2.1 · §3.1 · §5.2 · §10; 8 fixtures de esfera regeradas como área Dinâmica) | `3d621e94b` + `d5844ad5c` (2026-09-06, E) |
| v1-er | idem — errata atestada; curas do R-pré (3 nomes internos · 1 higiene) | `4cfc1745a` (2026-09-06, R-pré) |
| v1-q | idem — EMENDAS Q8/Q9/Q10 do E (a lista duplicada do *Local* · o centro atrasado do gancho e o zeramento das âncoras · os dois traços de aperto por passo) | `52e6f75a0` · `bdc378b5f` · `82ecde1b6` · `9a79c1721` (2026-09-06, E) |
| v1-qr | idem — emendas atestadas; curas do R-pré (1 nome interno no §10.4 · 1 insuficiência no gate 16) | este commit (2026-09-06, R-pré) |

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

**Veredito do R-pré sobre o INC-1 (2026-09-05) — medido, não lido.** Os 8 ficheiros em
`~/Referencias/blender-cloth/prior_scratchpad_recovered/` foram medidos SEM abrir o conteúdo (tamanho,
`<title>`, marcadores de diff, células de código): **6 são páginas de desafio do Cloudflare** («Just a
moment…», 61 KB idênticos, 0 células de código) e **2 são a landing page do arquivo do Phabricator**
(45 KB, sem a revisão). ⇒ **nenhum contém expressão do alvo**; pela régua §6.2 não há sequer «relance»
possível, e a classificação «substancial» do E — feita às cegas, como manda o §6.1 — é **refutada pela
medição**. Os restantes 148 (manual, API, notas de versão, blog, papers) estão em
`quarentena-scratchpad-2337/` — factos públicos. O scratchpad da sessão hoje só tem material NOSSO
(sondas, `.bak` de crates nossas, `.obj` de esculturas, scripts); o sweep sobre ele acusa apenas os
`.rlib/.rmeta` da nossa crate (o nome do ficheiro do pincel num doc-comment nosso — achado nº 1). O `04`
não afirma nada que só o fonte daria: cada linha estrutural cita manual/API/notas/blog, e as duas «via
resumo de busca» apontam para páginas que **nem tinham conteúdo**. ⇒ **nenhuma janela queimada; a
janela-mãe continua I.** Declaração dela registada na Corrente I.

## Erratas / seguimento do I (2026-09-06)

⭐ **Seis perguntas fechadas do I, nascidas do arnês de paridade contra as fixtures. Respostas por
leitura do fonte + medição; as que mexiam na espec foram emendadas (§3.1, §2.1, §5.2, §10).**

- **Q1 — o anel-1 das restrições:** é sobre as **arestas das FACES POLIGONAIS** (o vizinho de `v` são,
  por face, os dois cantos adjacentes a `v` naquela face, deduplicados), **não** sobre uma
  triangulação. ⇒ **quad interior = 4 vizinhos**, sem diagonal como vizinho; a diagonal só entra como
  restrição de PAR. O gate 8 («4 + 2 + 4») **está correcto** e o §3.1 passou a dizê-lo explicitamente.
  *A leitura do arnês de que «o Local casa com a grelha triangulada (6 vizinhos)» é do porte/harness,
  não do alvo — o alvo usa 4.*
- **Q2 — o `φ` das restrições inclui a banda `w(p⁰)`? SIM** (o factor por vértice é pré-calculado
  dobrando a banda, e a retenção de velocidade re-aplica-a). Há DOIS portões: a **célula inactiva**
  (grosso) e `w = 0` além do limite (fino). No plano o Local pára exactamente no disco de `3,5 R` pelo
  portão fino.
- **Q3 — Local vs Dynamic (fonte):** diferem em (a) o CENTRO de tudo (fixo no pen-down vs. o cursor a
  cada passo), (b) `R₀` vs `R`, (c) a criação de restrições ser filtrada por raio no Local e sem
  filtro no Dynamic, (d) construção de uma vez (Local) vs incremental (Dynamic), (e) a banda/força/
  retenção centradas no ponto fixo (Local) ⇒ o fim de um traço longo recebe menos força. A alavanca
  dominante do «Local < Dynamic» é o centro fixo vs. móvel; a razão exacta é emergente. §2.1 emendado.
- **Q4 — célula-folha ≤ 2 500 FACES (fonte):** a grelha de 4 225 v é ~2 células; a esfera de 6 050 ~3.
  A activação é grossa, mas a «parede» é a banda em `φ`, não a granularidade. §2.1 emendado.
- **Q5 — âncoras (fonte):** a correcção é **`Δ/2`** (a metade do vector de correcção) para toda espécie; numa
  âncora **B não se move, só A leva `Δ/2`** ⇒ fecha metade por varredura. O **`σ` por passo multiplica
  SÓ as âncoras de deformação** (pino e corpo mole não o levam). A força `s` do Grab radial é
  `0,1 · curva_do_pincel(d⁰)`. *Um port que fique abaixo do oráculo com `Δ/2` tem o défice noutro
  factor, não em trocar `Δ/2` por `Δ`.* §5.2 reforçado.
- **Q6 / ERRATA das fixtures de ESFERA:** a 1.ª entrega gravou-as como área **Local**, e estava
  errada — um traço scriptado não dispara o hover que fixa o centro da área Local, que ficou na
  **ORIGEM do objecto**; numa esfera unitária a origem põe toda a malha dentro da banda (todo vértice
  a `1,0` < início `1,006`) ⇒ `6 050/6 050` movidos, a esfera a mover-se como um corpo — **artefacto
  do arnês**. O `R₀` estava certo (`0,35`); o defeito era o CENTRO. **Corrigido:** as 8 fixtures de
  esfera foram **regeradas como área Dinâmica** (centro no cursor, que o traço fornece) — param no
  bordo da banda (`≈ 3,5 R`, zero além). A área **Local** fica medida só no PLANO (onde a origem cai
  na superfície). Commit da errata: `3d621e94b`.
- **Q7 — instrumento por passo (pedido de 2026-09-06):** quatro traços regravados como corridas-prefixo
  (`k` elementos do mesmo caminho, malha fresca) com prova `k = N` ≡ corrida inteira. ⚠️ O pen-down foi
  posto **na ORIGEM** porque a semeadura do hover é refém do ponteiro físico (medido: uma sessão inteira
  certa, a seguinte com o centro na origem em 30/30 corridas e zero em 2; a auto-verificação por
  bbox só discrimina em corridas com o disco inteiro excitado). Os fixtures *Local* anteriores foram
  auditados: corridas completas todas centradas em `x = −0,305` (o pen-down) — ver tabela abaixo.
  `f`/`φ` não são observáveis sem recompilar (checkout esparso); entregou-se o rastreio de sete vértices. Commit: `2807337c6`.
  | traço por passo | N | prova `k=N` ≡ inteira | movidos | máx |
  |---|---|---|---|---|
  | `plano_arrastar_radial_local_origem` | 12 | `0.000000` | 2145 | `0.329649` |
  | `plano_arrastar_radial_global_origem` | 12 | `0.000000` | 4225 | `0.645708` |
  | `plano_gancho_radial_local_2passos_origem` | 3 | `0.000000` | 1950 | `0.343869` |
  | `plano_agarrar_radial_local_2passos_origem` | 3 | `0.000000` | 1869 | `0.14572` |
  Auditoria dos fixtures *Local* já entregues (corridas completas, disco de movidos): TODOS centrados em `x = −0,305` = o pen-down (os de 1–2 passos não excitam o disco inteiro e não são auditáveis por bbox; nasceram na mesma sessão que os auditados).


### Q8 — a AMPLITUDE do *Local* (pergunta do I de 2026-09-06; resposta por leitura do fonte, mesmo dia)

**Pergunta (INBOX §Q8):** o oráculo entrega, no ramo *Local*, `0,34–0,57×` a amplitude do *Global* no
INTERIOR da área, e a nossa lei reproduz o oráculo *Global* a `5` varreduras e o *Local* a `10`.
Q8.1 — quantas passagens de relaxação faz cada ramo? Q8.2 — a lista do *Local* é deduplicada?
Q8.3 — o *Local* corre o solver mais vezes / com `dt` menor?

**Resposta — nenhuma das três hipóteses como formuladas; é uma QUARTA, e é a Q8.2 «pela outra ponta».**

- **Q8.1 — o número de passagens é o MESMO nos dois ramos:** um passo de pincel corre a relaxação
  **uma** vez, e ela varre a lista de restrições **5** vezes, em qualquer área (F). Nenhum
  multiplicador aqui.
- **Q8.2 — a lista É deduplicada, mas o registo de duplicados vive UMA CONSTRUÇÃO** (a fase 1 de um
  passo), não a simulação (F). ⇒ duas construções sobre a mesma célula deixam **duas cópias** de cada
  restrição. ⚠️ E a área *Local* faz exactamente duas construções: a fase 0 (1.º passo do traço)
  constrói e **não activa** — a marca de «construída» é a ACTIVAÇÃO, e o 1.º passo termina antes
  dela —, logo a fase 1 do 2.º passo reconstrói tudo. **Cada restrição do *Local* existe DUAS vezes;
  as de *Global*/*Dynamic*, uma** (esses ramos não têm fase 0 de construção). ⇒ **`10` projecções por
  restrição e por passo no *Local*, `5` nos outros** — a assimetria de `~2×` medida pelo I.
- **Q8.3 — não se aplica:** um solver por passo, `dt` igual nos dois ramos, sem sub-passos (F).

**Porque casa com a restrição que o I impôs à resposta:** o mecanismo é **constante desde o início**
(a lista já está dobrada quando a primeira relaxação com efeito corre) e **invisível no passo 2 dos
modos de FORÇA**, porque a relaxação corre ANTES da integração e nesse passo a malha ainda está em
repouso — percorrer duas vezes uma lista de correcções nulas dá o mesmo. Nos modos de ÂNCORA já não é
invisível no passo 2 (a âncora é escrita antes da relaxação e não está satisfeita), e é isso que o I
mediu como `agarrar_1passo` `869 → 1307` movidos contra `1324` do oráculo.

**Origem histórica (H, §9 item 15):** a fase 0 de construção do *Local* existe para que TODAS as
passagens de simetria acrescentem as suas restrições antes de qualquer activação — sem ela a 2.ª
passagem encontrava a célula já activada e não acrescentava as dela. A cópia extra no 2.º passo é o
efeito colateral dessa cura, e é **comportamento observável** do alvo: o corpus do I mede-a em 38
traços. ⚠️ **Consequência com simetria:** uma célula tocada por `n` passagens fica com `n + 1` cópias.
Nas fixtures o factor é exactamente `2`.

**Correcções que a Q8 impõe a respostas anteriores deste ledger:**
- **Q3 fica emendada:** *«a alavanca dominante do "Local < Dynamic" é o centro fixo vs. móvel; a razão
  exacta é emergente»* está **REFUTADO** — o centro fixo e o aro preso explicam o **bordo**; o
  **interior** é a lista duplicada, e a razão **não** é emergente. (§2.1 e §10.2 da espec emendados.)
- **Q5 fica CONFIRMADA e o défice NOMEADO:** a correcção de âncora é mesmo `Δ/2`; *«o défice de um
  port está noutro factor»* — o factor é o número de PASSAGENS, porque todas as fixtures de âncora
  são de área *Local*. (§5.2-bis da espec.)
- A leitura do §10.2 *«até ao passo 2 são iguais ⇒ o mecanismo é o aro»* está **refutada como
  inferência**: os dois mecanismos previam essa igualdade. O facto medido fica; a conclusão mudou.

**Emenda à espec (commit desta entrada):** §1 fases 0 e 1 · §2.1 · §3.1 · §3.3 · **§5.2-bis (nova)** ·
§10.2 · **§10.3 (nova, o lado medido pelo I)** · §13 · §14 gates 8, 16 e 17.
**Cobertura desta leitura:** o ficheiro do pincel/filtro de tecido (2 590 linhas) relido nas fases de
construção de restrições, relaxação, passo de simulação, activação de células e entrada do pincel,
mais o predicado de «primeiro passo» e o produtor do conjunto de células, no ficheiro central do modo
de escultura. Fonte v5.2.0, por shell, 2026-09-06.
**Sweep:** verde sobre a espec emendada e sobre esta secção do ledger, isoladamente (vassoura de 70
entradas), 2026-09-06.
⚠️ **FACTO PARA O R-PÓS, não causado por esta emenda:** o sweep sobre o LEDGER INTEIRO sai **✗ com
duas ocorrências PRÉ-EXISTENTES** (linhas 88 e 209 na versão de `fa785e173`, as duas na tabela de
cobertura e no achado de parede nº 1, onde o caminho de um ficheiro do alvo é nomeado por exigência
do §6 «cobertura da travessia»). ⇒ há uma tensão real entre o §6 (que MANDA registar os ficheiros
percorridos) e a barra do §7.2 (zero hits na árvore inteira), e os atestados de 05 e 06/09 dizem
«sweep verde ... + ledger». **Decisão é do R** — o E não silencia nem apaga o registo de cobertura
por conta própria.
⏳ **Falta o atestado do R-pré sobre esta emenda** (a janela-mãe despacha-o antes de implementar).

### Q9 — o SNAKE HOOK deforma no sítio errado (pergunta do I de 2026-09-06; resposta no mesmo dia)

**Pergunta (INBOX §Q9):** o pico da deformação do gancho fica, no port, sob o cursor (`0,05R`) e no
oráculo onde o pincel **estava** (`0,86R`); no arrasto os dois coincidem. Q9.1 — o centro da queda é
a posição do fim ou do início do passo? Q9.2 — a distância mede-se contra que posições? Q9.3 — falta
uma restrição de FORMA?

- **Q9.1 — o INÍCIO do passo.** Nos dois modos de âncora a localização do pincel **deixa de ser lida
  do evento a partir do 2.º passo** (F). No Grab ela fica pregada no pen-down todo o traço; no Snake
  Hook ela é avançada pelo delta — **mas o avanço corre ANTES de o delta deste passo ser calculado**,
  logo usa o delta do passo ANTERIOR. ⇒ `c_k = pen-down + Σ_{i<k} δ_i`: o centro está **um passo
  atrasado**, e no 1.º passo simulado (delta anterior `= 0`) é **exactamente o pen-down** — que é o
  vértice mais deslocado que o I mediu no oráculo. A hipótese que o I construiu e reverteu está
  **CORRECTA**; a espec dizia `c ← c + δ` sem dizer **qual** `δ`, e é essa a emenda (§4.3).
- **Q9.2 — as posições ACTUAIS** (o estado deformado com que o passo começa). O **Grab é o único**
  modo que mede distância, recorte e textura sobre as posições de repouso; os outros sete, o Snake
  Hook incluído, medem sobre as actuais (F). ⇒ o material já puxado viaja **com** o centro atrasado.
- **Q9.3 — NÃO existe eixo, plano nem limite de profundidade próprios do Snake Hook** (leitura
  integral da fase de gesto). O «plano de profundidade» é do **delta** e vale para os oito modos; a
  queda é a distância comum ao centro com a forma de queda do pincel. ⇒ a forma que falta a um port
  é o PAR (a) centro atrasado + (b) distâncias sobre as posições actuais — não uma lei nova.

**Achado colateral, e é uma CORRECÇÃO à espec:** a força por passo das âncoras é **zerada em todo o
objecto** antes de ser reescrita **nos DOIS** modos de âncora — a espec e o gate 12 diziam «o Grab
não», e estava errado. O que distingue os dois é o valor com que reescrevem (`1` radial / `clamp(f)`
plano no Grab; `f` no Snake Hook) e o conjunto afectado do Grab ser fixo. Gate 12 reescrito, gate 18
novo (a posição do pico).

**Emenda à espec (commit desta entrada):** §4.3 (o `δ` do avanço; o Grab como único que mede no
repouso; o zeramento nos dois) · **§10.4 (nova, o lado medido pelo I)** · §14 gates 12 e 18.
**Cobertura desta leitura:** as fases de gesto e de aplicação de forças do ficheiro do pincel de
tecido, mais a actualização do estado do traço (localização, delta, predicados de delta ancorado e de
orientação de ponta) e o cálculo de distâncias do pincel, no ficheiro central do modo de escultura.
Fonte v5.2.0, por shell, 2026-09-06.
**Sweep:** verde sobre a espec emendada, 2026-09-06.
⏳ **Falta o atestado do R-pré** (junto com o da Q8).

### Q10 — os dumps POR PASSO dos dois modos de APERTO (pedido do I de 2026-09-06; ENTREGUE no mesmo dia)

**Pedido:** os modos de aperto são exactos no traço de um passo e erram `1,07` (ponto) e `2,02`
(linha) no fim do traço inteiro, sempre a sobrepassar; as varreduras não os explicam (a `10` o de
linha melhora e o de ponto piora, ao contrário do resto do ramo *Local*). ⇒ dumps por passo para
localizar em que passo nasce.

**Entregue:** `plano_apertar_ponto_radial_local_origem` e `plano_apertar_linha_radial_local_origem`,
**12 passos cada** (o pedido dizia 2; entregaram-se os 12 porque a divergência nasce «entre o passo 1
e o fim» e um traço de 2 passos não a alcança). Método idêntico ao da Q7: corridas-prefixo de `k`
elementos do MESMO caminho sobre malha fresca, `k = 1..12`, mais uma corrida inteira da mesma sessão
como referência. **`prova_do_fatiamento = 0,000000` nos dois.** Pen-down **na origem** (determinismo
do centro da área *Local*). Auto-verificação do centro: `ok` em todas as corridas retidas.

| traço | passos | prova | movidos | máx `|u|` |
|---|---|---|---|---|
| `plano_apertar_ponto_radial_local_origem` | 12 | `0.000000` | 2145 | `0.303401` |
| `plano_apertar_linha_radial_local_origem` | 12 | `0.000000` | 2137 | `0.100744` |

⭐ **E o rastreio já responde metade da pergunta antes de o I correr a sonda:** sob o pen-down o
aperto de PONTO **não é monótono** (`0,093 · 0,184 · 0,118 · 0,106 · 0,197 · 0,208 · 0,201 · 0,187 ·
0,160 · 0,149 · 0,154` nos passos 2..12) — a força aponta para o **cursor**, que se afasta, logo o
vértice é puxado e largado a cada passo. Uma lei que integre monotonamente ultrapassa, que é
exactamente o sinal que o I mede no fim do traço. O de LINHA quase não move o pen-down (`≤ 0,006`,
contra `0,10` no vizinho a `1R`): ele aperta contra a **linha** do traço, e o que está sobre ela já
lá está.

**Ficheiros:** `fixtures/cloth/plano_apertar_{ponto,linha}_radial_local_origem.{deformado,porpasso}.txt.gz`
+ `.porpasso.rastreio.txt`. README das fixtures actualizado (6 traços por passo, 53 no total).
**Espec:** §10.5 (nova). **Sweep:** verde sobre a espec, o README e a pasta inteira das fixtures.
**Instrumento:** o mesmo arnês do oráculo de 05/09, com um ficheiro de corridas novo e o montador
por passo estendido aos dois modos — os dois fora da árvore, em `~/Referencias/`.

### Q11 — o APERTO DE PONTO (perguntas do I de 2026-09-06; resposta no mesmo dia, com corrida NOVA do oráculo)

**Perguntas (INBOX):** Q11.1 o vértice sobre o cursor recebe força? a direcção nula tem tratamento
próprio? · Q11.2 o factor e a direcção do aperto são avaliados no mesmo instante que os do arrasto? ·
Q11.3 há no aperto tecto de deslocamento, corte ao ultrapassar o cursor, ou amortecimento próprio? ·
Q11.4 (acrescentada pelo I) o que a relaxação faz de diferente num passo de aperto e num de arrasto,
partindo do mesmo estado? · e a hipótese do coordenador: a versão do oráculo carrega a regressão do
§9 nº 20 (multiplicação trocada por subtracção)?

**Respostas, do fonte (F, travessia reaberta em 2026-09-06 sobre o pincel de tecido inteiro + o
módulo que actualiza o cache do traço a cada evento do rato):**

- **Q11.1** — não há tratamento especial do vértice sobre o cursor **além** da direcção nula: a
  re-escala a comprimento `1` da casa devolve o vector NULO para separação nula ⇒ **força zero**, sem
  `NaN`, sem direcção de reserva, sem saltar o vértice; a um epsilon dali a força é **inteira**. E a
  variante de plano tem a mesma propriedade (distância assinada zero ⇒ vector nulo). ⇒ **o ponto onde
  o aperto é mais forte é o ponto onde a direcção dele está pior determinada.** Espec §4.2 · §11.
- **Q11.2** — **mesmo instante**. Todos os modos de força lêem as posições com que o passo começa (a
  malha escrita pelo passo anterior), antes da relaxação deste passo; o único modo que lê outro
  instante é o Grab (posições de repouso). Confirmado também que a localização do cursor é
  **re-lida do evento a cada passo** em todos os modos excepto os dois de âncora e o traço *anchored*.
- **Q11.3** — **não**. Zero tecto de deslocamento, zero corte ao ultrapassar o alvo, zero
  amortecimento próprio: tudo o que o aperto tem, o arrasto também tem.
- **Q11.4** — a relaxação **não faz nada de diferente**: ela não sabe qual é o modo de deformação, e
  nos modos de força não existem sequer âncoras. O que o aperto faz é **ANTES** dela.
- **Hipótese da regressão: REFUTADA.** A entrada #127836 foi relatada **e fechada no mesmo dia**,
  2024-09-19 — **dois anos** antes da versão que gravou as fixtures — e o conserto foi voltar à
  multiplicação. A versão do oráculo lê o aperto tal como a §4.2 o descreve; **não há divergência
  deliberada a declarar**. A linha do §9 nº 20 passa a dizer a data do conserto (uma regressão
  fechada, escrita numa tabela de história sem a data do fim, lê-se como dívida viva).

**O achado, e ele é MEDIDO (M, 2026-09-06):** a magnitude dos dois apertos **não decresce com a
proximidade** — `u` é a separação re-escalada a `1` e o único factor que sabe da distância é a curva
de falloff, que ali está no máximo. Na malha de referência o impulso máximo é `2,1×` a aresta ⇒ **no
1.º passo simulado, a partir do repouso, o oráculo põe `9` vértices para lá do cursor e devolve `10`
quadriláteros de orientação invertida** (o arrasto: `0`). A partir daí a relaxação recebe pares
comprimidos, `(1 − ℓ/D)` inverte o sinal e cresce sem tecto (medido `D/ℓ = 0,052` ⇒ factor `−18,1`
no aperto; `0,49` ⇒ `−1,1` no arrasto, o pior dos 12 passos), e **o resultado por vértice passa a ser
decidido pela ORDEM de resolução**.

**A prova vive dentro do oráculo e não precisa do nosso lado — a simetria de espelho.** Malha de
repouso, caminho (em `y = 0`), lei da força e **conjunto** de restrições são simétricos em relação ao
traço; a **ordem** da lista não é. `max|u(v) − espelho(u(espelho(v)))| ÷ |u|max`, por passo:

| traço | quadriláteros invertidos `k=2/3/12` | assimetria ÷ `|u|max`, `k=2/3/12` |
|---|---|---|
| aperto de PONTO, força `1` | `10` / `18` / `52` | `0,000` / **`0,675`** / `1,060` |
| aperto de PONTO, força `0,2` (controlo NOVO) | `0` / `0` / `0` | `0,000` / `0,103` / `0,144` |
| aperto de LINHA | `6` / `5` / `2` | `0,000` / `0,099` / `0,204` |
| arrastar *Local* | `0` / `0` / `0` | `0,000` / `0,059` / `0,219` |
| arrastar *Global* | `0` / `0` / `57` | `0,000` / `0,064` / `0,286` |
| Snake Hook (`_2passos_origem`) | `0` / `11` / — | `0,088` / `0,283` / — |
| Grab (`_2passos_origem`) | `0` / `0` / — | `0,095` / `0,099` / — |

⭐ Em **todos** os modos de FORÇA a assimetria é `0,000000` no 1.º passo simulado — o passo em que a
relaxação corre sobre a malha em repouso e não tem o que corrigir — e nasce no seguinte; nos **dois**
modos de ÂNCORA ela já lá está no 1.º, que é o passo em que a âncora dá trabalho à relaxação. *A
régua concorda com o mecanismo nos dois sentidos, e mostra que a assimetria é fabricada pela ordem.*

**Intervenção, não correlação — a fixture nova.** `plano_apertar_ponto_radial_local_origem_fraco`:
o mesmo traço, a mesma malha, o mesmo caminho, com **uma** coisa mudada (força `1 → 0,2`, impulso
`0,085×` a aresta em vez de `2,1×`). Zero faces invertidas nos doze passos e a assimetria cai para
`0,103`, o piso do arrasto. Corrida nova do binário 5.2.1, 13 execuções (12 prefixos + a inteira),
`prova_do_fatiamento = 0,000000`, auto-verificação do centro `ok` em todas.

**Duas coisas do fonte que a espec não dizia e que só passaram a importar com este achado:** (a) a
**ordem interna** de criação por vértice — corpo mole · `(v,n)` · `(a,b)` · âncora de deformação ·
pino — e que o registo de pares partilhado pela construção faz a PRIMEIRA ocorrência fixar a posição
na lista; (b) o **filtro de raio da construção só vale para as estruturais e o corpo mole** — a
âncora de deformação e o pino nascem para todo vértice visível da célula, e a âncora radial do Grab
tem um filtro próprio e diferente (o raio do PINCEL).

**Consequência declarada (decisão do dono, não do port):** num retalho invertido o resultado por
vértice do aperto **não é reproduzível** por uma árvore espacial diferente da do alvo — não é lei em
falta, é uma resposta que a ordem define; e reproduzir o oráculo aqui é reproduzir **um defeito
conhecido e aberto do alvo** (§9 nº 23: artefactos dos pincéis de tecido · o aperto do filtro numa
superfície plana). A saída alternativa — limitar o impulso do aperto à distância que falta até ao
alvo — muda o produto e diverge do oráculo de propósito.

**Ficheiros:** `fixtures/cloth/plano_apertar_ponto_radial_local_origem_fraco.{deformado,porpasso}.txt.gz`
+ `.porpasso.rastreio.txt`; `indice.json` **regenerado** (tinha 48 entradas para 54 ficheiros —
faltavam-lhe os seis traços `_origem`); README das fixtures actualizado, incluindo a correcção de uma
descrição do `analise.json` que não era a do ficheiro. **Espec:** §3.1 · §4.2 · §5.2 · **§5.2-ter
(nova)** · §9 nº 20 · §10 · **§10.6 (nova)** · §11 · §14 gates 19-21; cabeçalho marcado
⏳ **aguarda o atestado do R-pré**. **Sweep:** verde sobre a espec, o README e a pasta inteira das
fixtures. **Instrumento:** o arnês do oráculo de 05/09 com um ficheiro de corridas novo e um montador
próprio, os dois **fora da árvore**, em `~/Referencias/`.

⚠️ **Para o R-PÓS, registado aqui para não se perder:** o sweep sobre **este ledger** (e sobre o
histórico dos caminhos de `cleanroom/`) devolve **DOIS** hits, os dois **pré-existentes de
2026-09-05** e os dois na mesma coisa — o caminho/nome do ficheiro do alvo, escrito nas linhas da
cobertura da travessia e do achado de parede nº 1. Não vêm desta emenda (conferido no `git diff`),
e o ledger é, por desenho do §6, o sítio onde a proveniência vive e que a janela I nunca lê (o
`settings.local.json` da worktree nega-lhe o `Read`). ⇒ **é decisão do R-PÓS**: ou reescrever as duas
linhas em vocabulário do domínio (e então o histórico continua a tê-las, o que só o `--git-history`
vê), ou declarar a excepção do ledger por escrito no fechamento. ⛔ O que não pode ficar é
implícito — a barra do §7.2 é «zero hits sobre a árvore inteira».

## Fechamento R

⏳
