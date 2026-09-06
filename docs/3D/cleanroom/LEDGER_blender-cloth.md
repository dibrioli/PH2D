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

---

## Espec

| versão | caminho | commit |
|---|---|---|
| v1 | `docs/3D/cleanroom/SPEC_cloth_brush.md` | `c7905f616` (2026-09-05, commit único pós-filtragem) |
| v1-r | idem — atestada; curas do R-pré (1 expressão · 6 higienes · anexos) | `0c884a2b2` (2026-09-05, R-pré) |
| v1-e | idem — ERRATA do E (as 6 perguntas do I; §2.1 · §3.1 · §5.2 · §10; 8 fixtures de esfera regeradas como área Dinâmica) | `3d621e94b` + `d5844ad5c` (2026-09-06, E) |
| v1-er | idem — errata atestada; curas do R-pré (3 nomes internos · 1 higiene) | ⏳ registado no commit seguinte (2026-09-06, R-pré) |

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

## Fechamento R

⏳
