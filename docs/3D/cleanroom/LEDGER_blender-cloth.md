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

### Passagens do papel E (uma linha por subagente, com o que cada um reabriu)

| passagem | data | id | o que leu do fonte, e o que produziu |
|---|---|---|---|
| E (travessia inicial) | 2026-09-05 | subagente-E da janela `1246816c-…` | a travessia integral abaixo; a espec, o ledger, a vassoura, as fixtures da 1.ª geração |
| E (Q8 · Q9 · Q10) | 2026-09-06 | idem | o laço das varreduras, a construção do conjunto *Local*, o instante do centro do gancho |
| E (Q11 · Q12 · Q14 · Q15 · Q16 · Q17) | 2026-09-06/07 | idem | o aperto, a normal e o factor de escala do empurrar, a rede de restrições, a ordem de criação, o censo do esticão, o instante das normais |
| **E (Q18)** | **2026-09-07** | **subagente-E da janela `1246816c-63cf-414b-842d-663a8baa86ca`** | **o par de factores por vértice (o das varreduras e o da integração) lado a lado; a lei incremental da normal por vértice, na árvore de aceleração do desenho; produziu a EMENDA Q18 (§4.2-quater · §4.6 linha 4 · §5.2 · §5.4 · §5.4-bis · §10.12–§10.14 · §11 · gates 43-47) e CINCO fixtures novas** |
| **E (Q22 — o FILTRO)** | **2026-09-07** | **subagente-E da janela `1246816c-63cf-414b-842d-663a8baa86ca`** | **ABERTA ANTES DA PRIMEIRA LEITURA (§6).** Objecto: a emenda do §7 (o filtro de tecido) e o **oráculo do filtro** — o §10 não tinha UM ÚNICO traço de filtro, logo toda barra que a janela escrevesse mediria os defeitos dela própria (`CLAUDE.md` §0.9). **Releu:** o comando modal do filtro de tecido e a abertura dele (as propriedades expostas, os valores de omissão e as faixas); a construção da área e da lista de restrições no ramo sem traço de pincel; a expressão de cada um dos cinco tipos; as duas matrizes de orientação e o troço que anula componentes por eixo; a soma da gravidade da cena e o mesmo troço no ramo do pincel, lado a lado; o sítio onde as normais da malha são recalculadas dentro da preparação da peça para edição; e o destino (nenhum) da fotografia de normais tirada ao criar a simulação. **Produziu:** a EMENDA Q22 (§7 reescrito nas 12 linhas · **§7.1 NOVA** · **§10.17 NOVA** · **§14-bis NOVA**, gates 55-60), **17 fixtures** novas em `fixtures/cloth/filtro/` (+ 4 ficheiros por passo), a secção do filtro no README das fixtures, o arnês novo do oráculo do filtro e **36 corridas novas** |
| **E (Q19 · Q20 · Q21)** | **2026-09-07** | **subagente-E da janela `1246816c-63cf-414b-842d-663a8baa86ca`** | **releu: a função que devolve a localização usada pela área simulada e as três áreas; onde a localização por passo é (ou não é) relida do evento do traço e onde ela é reescrita pela origem ancorada; a classificação de «ferramenta de agarrar» que decide o tamanho por pressão; a porta que inclina o delta para a normal e a capacidade que a declara (mais o Python do painel e a porta de propriedades que a expõe); a construção de restrições com as posições persistentes e as quatro leituras que elas substituem; os quatro construtores de restrição; a resolução de colisão e a ordem dela dentro do passo. Produziu a EMENDA Q19 (§2.1 · §4.3 · §5.6 · §6.4 · §8.1 · §8.4 · §10.13 errata · §10.15 · §10.16 · §11 · gate 46 reescrito + gates 48-54), OITO fixtures novas, a chave `dispersao_entre_realizacoes` em TREZE cabeçalhos, e 80 corridas novas do oráculo** |

| **E (Q23 — o filtro ATRAVESSA gestos)** | **2026-09-09** | **subagente-E da janela `1246816c-63cf-414b-842d-663a8baa86ca`** | **ABERTA ANTES DA PRIMEIRA LEITURA (§6)** — ver a secção «Q23» no fim deste ledger para o pedido transcrito, o que foi reaberto e o veredicto |

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

⛔⛔ **A I-1 está QUEIMADA como I para este módulo desde 2026-09-09 (INC-2, classificado SUBSTANCIAL
pelo R).** Ela escreve o BLOCO-RETOMADA e **PARA**; a **I-2** assume a **MESMA** linha e retoma da
espec — que está agora **atestada** nas duas emendas que faltavam (Q22 e Q23). ⚠️ **A queima é da
EXPOSIÇÃO, não do produto:** a quarentena das seis regiões saiu **limpa** e ⛔ **nada do que a I-1
escreveu se reescreve** (§6.3 e §6.4 são perguntas separadas, de propósito). A I-2 herda **uma**
dívida, e ela é de **expressão, não de lei**: os dois sítios nomeados na prescrição do INC-2.

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
| R-pré (emenda Q11) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu a emenda (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestado no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (emenda Q11)» |
| R-pré (emenda Q12) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu a emenda (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestado no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (emenda Q12)» |
| R-pré (emenda Q14) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu a emenda (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestado no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (emenda Q14)» |
| R-pré (emendas Q15 **e** Q16) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu as emendas (transcript = zona contaminada — leu o fonte por shell) | 2026-09-06 — ✅ **atestadas no cabeçalho da espec**; veredictos em «Auditoria R-pré — 2026-09-06 (emendas Q15 e Q16)». ⚠️ **A Q15 tinha shipado SEM atestação** e foi auditada aqui |
| R-pré (emenda Q19 · Q20 · Q21) | subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — contexto novo, independente do subagente que escreveu a emenda (transcript = zona contaminada — leu o fonte por shell) | 2026-09-07 — ✅ **ATESTADO no cabeçalho da espec**; veredictos em «R-pré da emenda Q19 … 2026-09-07». **ZERO achados de expressão · UMA higiene §4.2 · NOVE curas funcionais**, a maior delas a banda antiga a dividir os quocientes que a própria emenda tornou obsoletos. **Os três itens que o E deixou nomeados, conferidos:** (a) o censo do cabeçalho corre **silencioso** com o título renomeado, e o instrumento do I passa a ler `11`; (b) o `analise.json` tem mesmo `47` objectos com as chaves **do arnês** para `86` fixtures — ⛔ fica como está (é dado nosso, o sweep passa e o README já manda não acreditar nele), e o **índice derivado** é que é a fonte: `gera_indice.py` regenera-o byte-a-byte, 86 para 86; (c) o auto-teste do arnês vive **fora da árvore** e não é auditável daqui — o que o R-pré verificou é que a **regra registada** (`< 50` movidos ou mais de metade da malha) recusa exactamente os dois modos de falha que ela apanhou, porque a esfera tem `6 050` vértices e as corridas más deram `6 050` e `0` |
| **R de INCIDENTE (INC-2)** | subagente-R despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca` — **contexto novo**, ⛔ não é o subagente-E de nenhuma emenda nem o R-pré que achou o item, e **viu os dois lados** (transcript = zona contaminada) | 2026-09-09 — ⛔ **classificou o INC-2 como SUBSTANCIAL** ⇒ a **I-1 queima**; **quarentena LIMPA** nas 6 regiões (comparação de expressão sobre `850` linhas · sweep verde · busca dirigida a zero); **sweep de memória §6.4 verde, nada revertido**; prescrição §7.3.d escrita. Bloco em *Incidentes* → **INC-2** |
| **R de INCIDENTE (INC-3) + CENSO §4.2 da árvore** | subagente-R despachado pela janela-mãe `72723fbf-b300-4f42-9178-535bb37b3941` (elo **I-2**) — **contexto novo**, ⛔ não é o subagente-E de nenhuma emenda nem o R do INC-2, e **viu os dois lados** (fonte por shell; `Read` deny-listed para a sessão) | 2026-09-09 — ⛔ **INC-3 classificado RELANCE** (a I-2 **não** queima), com a leitura registada de que **o canal não muda o §6.2 e AGRAVA o §4.2**; **censo §4.2 da árvore rastreada: 64 ficheiros ⇒ 57 (a) · 7 (b) · 0 (c)**, com **três cegueiras medidas do detector** (a população real é ≥107); os **três** hits da vassoura são **REAIS** e são **uma nota copiada três vezes**; tarefa nº 1 do BLOCO-RETOMADA **confirmada** (2 de finalidade · 3 de facto) e a **TERCEIRA cópia da oração ACHADA fora do repo e apagada**; resíduo de atestado da Q23 **removido** e a **segunda metade do censo** (detector de CONTRADIÇÃO) escrita na espec, provada por mutação nos dois braços |
| R-pós | ⏳ | — |

### Auditoria R-pré — 2026-09-06 (emendas Q15 e Q16)

**Âmbito.** A emenda **Q16** (cabeçalho · **§5.7-bis NOVA** · §10 contagem · **§10.10 NOVA** · §11
uma linha · §14 gate 31 reescrito + gates **35-38**, mais as **oito** fixtures por passo, o
`indice.json` regenerado e o README das fixtures) — e, por ausência descoberta durante ela, a
emenda **Q15** (§2.1 errata · §3.1 · **§3.1-bis NOVA** · **§10.9 NOVA** · §14 gates **32-34** e as
quatro fixtures de topologia), que **não tinha atestação nenhuma** no quadro do cabeçalho nem secção
neste ledger, e cujos factos **já estavam implementados** (§3.R: *sem esse atestado, a janela não
implementa*). ⇒ ⛔ **o defeito de processo fica registado**: uma emenda pode chegar ao código sem
passar por aqui, e o único instrumento que a apanhou foi a leitura do quadro pelo R-pré seguinte.

**Conformidade §4.2 — ZERO achados de expressão nas duas.** Sem trecho, sem nome interno (funções,
variáveis, ficheiros, structs, macros), sem wording de manual ou de comentário. ⭐ **O risco
nomeado no briefing — um censo de ausências alinhar-se pela ORGANIZAÇÃO do fonte — foi conferido
linha a linha: a ordem das dez linhas do §5.7-bis é a das PERGUNTAS do INBOX Q16**, e nada nela
segue a disposição do alvo. Os nomes das oito fixtures novas, a chave nova do cabeçalho
(`passos_com_cursor_parado`) e o vocabulário das quatro alavancas são do domínio.

**CINCO higienes §4.2/§4.3 curadas no acto, por re-expressão e sem perder facto** — três na Q16:
um termo de código em inglês onde a palavra do domínio já dizia tudo; *«há uma lista e um laço»*,
que descrevia a FORMA do código (passa a *o conjunto é percorrido inteiro, `5` vezes, e nenhuma
passagem selecciona as esticadas*); e *«uma constante do ficheiro»*, que localizava a constante no
fonte (passa a *um valor fixo do programa*). E duas na Q15: *«um passo explícito do código»* e, a
mais séria das cinco, *«o próprio código a declara»* — **citar a prosa do alvo como autoridade**,
hoje escrita como o comportamento que ela descreve.

**FIDELIDADE — Q16.** As **dez** linhas do censo conferidas no fonte, uma a uma: o factor único da
projecção e o guarda de separação nula · ausência de tecto · **uma** travessia do conjunto, repetida
`5` vezes, sem passe selectivo sobre as esticadas · correcção linear na separação · comprimento de
repouso escrito **uma** vez na criação, com **um** sítio de escrita para o desvio do Expand · nenhum
termo que dependa da normal dentro da projecção · nenhuma restrição sobre ângulo · **quatro**
espécies, nem uma quinta · **um** passo de solver por passo de pincel, sem laço de sub-passos · o
passo de tempo como valor fixo, lido num sítio só. ⭐ Todos os números do §10.10 **reconstruídos do
zero** das fixtures, com script próprio fora do repo: as `2 × 11` do pen-down, as `2 × 11` a `1R`,
as `4 × 11` do máximo da malha, a amplitude e a calibração; mais a **prova do fatiamento
recalculada** (bloco `k = 12` contra o `.deformado`: `0,000000` nas oito), as oito contagens de
`movidos`, os oito `max_deslocamento`, o `indice.json` a regenerar-se **byte-a-byte** (73/73) e o
`verifica_traco.py` **verde sobre os 73**. As duas armadilhas reportadas pelo E conferem: a faixa da
massa declarada pela porta de propriedades é `0,01..2` (logo o `4` pedido foi coagido, e a §5.4 já
dizia `0,01..2`), e a régua das excepções do README dá **`23` de `73`** com a repartição
`6/6/4/3/2/1/1/1` e `16` fora da área *Local*.

**CINCO correcções de fidelidade/suficiência na Q16, aplicadas no acto:**
1. o manifesto da emenda dizia tocar a **§5.2** e o diff não lhe põe uma linha;
2. o máximo da malha do Push `_parado` vira no passo **`5`**, não no `4` (o `4` é o do Inflate
   `_parado`) — os três viram em passos diferentes;
3. ⛔ o gate 37 convertia resíduo em projecções por `x / 15,1` e **esquecia o vão da própria régua**
   (os `15,1 %` valem **`5`** projecções), contradizendo o *«uma projecção a mais»* da secção que o
   gera ⇒ `5 · x / 15,1`;
4. ⛔ o gate 36 punha a barra em `f32` sobre razões lidas de um ficheiro de **seis casas**, e o
   oráculo lê `0,2499924`/`0,0624943` — *a barra reprovaria a fixture que a define* ⇒ `±2·10⁻⁵`,
   derivada da resolução (e o «ao bit» do §10.10 caiu);
5. ⛔ o gate 38 mandava a sequência «descer» depois do máximo, e a cauda do `_origem` **volta a
   subir** nos dois últimos passos (`0,24762 → 0,25738 → 0,25937`), porque o vértice que realiza o
   máximo viaja com o cursor ⇒ a régua é o **argmax**.
⭐⭐ **Mais duas de suficiência:** a linha `1R` do §10.10 não nomeava a sonda e o vértice
**espelhado** dá outros números (`0,00477` contra `0,00421` no passo 3) — ficam as duas escritas por
coordenada, com o achado que vem de graça: *numa cena de simetria perfeita a única coisa que separa
os dois lados é a ORDEM (§3.1-bis parte o plano na fileira do pen-down), o que é uma medição a favor
da explicação (a) do §5.7-bis*; e o controlo do gate 37 sobe de uma célula para **a malha inteira**
(no passo 2 as duas áreas dão `máx` da diferença por vértice `= 0,000000` sobre os `4 225`).

**FIDELIDADE — Q15.** Reconstruídos do zero das quatro fixtures de topologia: plano `2` células
(`2 145` + `2 080`, `2 048` faces cada, **`1`** descenso, as duas **contíguas**, a de índice `1` a
ser `[2080..4224]`) · esfera `4` células (`1 569`/`1 520`/`1 504`/`1 457`, `1 536` faces cada,
**`3`** descensos) · somas exactas (`4 225` e `6 050`, sem repetidos) · anéis divergentes `128`
(`3,0 %`) e `5 959` (`98,5 %`) · o mais próximo dos `128` a `1,5` do pen-down, `0` dentro de
`1,2250` e `126` dentro de `2,1000` · os «dois `2 145` diferentes», intersecção `1 099`. No fonte:
tecto de `2 500` faces por folha e profundidade `99`; a lista de vértices da folha **ordenada** e
reclamada por índice crescente de folha (é isso que faz os próprios saírem crescentes); e o teste
`centro[eixo] ≥ limiar` a mandar o lado `≥` para o **primeiro** filho, no qual a recursão desce
primeiro. ⛔ **UMA correcção:** a regra do **empate de eixos** era dada como facto de fonte e a
função que a resolve **não está na parte do fonte que esta linha tem** (o checkout é esparso) — a
metade `X`↔`Y` fica **provada pela partição medida do plano** (a caixa empata `3,0`/`3,0` e a
partição sai por `y`, com a célula `1` do lado `≥` — o que também confirma as duas cláusulas
seguintes), e a metade `Y`↔`Z` fica nomeada como a única cláusula da secção sem prova deste lado.

**Sweep.** ✅ verde sobre a espec emendada + a pasta **inteira** das fixtures + INBOX + os dois
READMEs + `docs/3D/cloth/`. `--git-history` sobre `docs/3D/cleanroom/`: os **dois** hits são os
pré-existentes de 2026-09-05 (linhas deste ledger, já registadas em «Achados de PAREDE»), e nenhum
vem das emendas Q15/Q16. ⚠️ **Fora do âmbito, e para o R-pós:** `git grep` acusa **quatro** linhas do
produto e de um handoff de 2026-08-16 que citam proveniência por nome interno de ficheiro do alvo —
são **anteriores** a esta linha e já vivem no `ACHADO_proveniencia_por_nome_interno.md`.

⚠️ **Nota para o E, não para a espec:** existe no fonte um valor declarado com o sentido de «tecto de
restrições por vértice» e **nada o lê** — não há tecto nenhum. Fica aqui para que a pergunta não
reabra o censo do §5.7-bis.

**Veredicto: ATESTADAS as duas** — as emendas Q15 e Q16 podem ser lidas pela janela-mãe.

### Auditoria R-pré — 2026-09-06 (emenda Q14)

**Âmbito.** A emenda Q14: cabeçalho · §2.2 (errata da banda) · §3.2 (bloco novo) · §4.2 · §4.5 · §5.2
(dois blocos novos + o censo dos limites) · **§5.2-quater NOVA** · §10 contagem · **§10.8 NOVA** · §11
(três linhas) · §14 gates **25-31**, mais as **nove** fixtures `*_origem_1passo*`, o `indice.json`
regenerado e o README das fixtures.

**Conformidade §4.2 — TRÊS achados, todos da MESMA espécie, todos curados no acto.** A espécie é
*comentário/doc-comment do alvo re-dito quase-verbatim*: (1) a justificação de o 1.º passo de uma
passagem não simular, no §1 fase 0 — ⚠️ **pré-existente**, e a tradução mais colada do documento
inteiro; (2) a frase do §3.2 que descrevia como a restrição guarda as duas pontas; (3) a razão, no
§5.2 nº 5, de o peso por vértice ter de ser um vector da malha inteira. As três ficam re-expressas
como **requisito funcional** em vocabulário do domínio, sem perder um facto. ⛔ Nada mais: sem
trechos, sem nomes internos (funções, variáveis, ficheiros, structs, macros), sem wording de manual.
Os nomes das nove fixtures novas, as chaves dos cabeçalhos e o vocabulário que a emenda introduz
(**gaveta · memória de forma · desvio de repouso · factor por passo**) são do domínio; `Gauss–Seidel`
e `Jacobi` são literatura pública (§4.1.2); e *Local*/*Global*/*Smooth*/*Constant* são rótulos que o
artista vê, que o §4.1.13 permite e que este documento já usava.

**Sweep (§7.1).** `bash scripts/cleanroom-sweep.sh` com a vassoura de 70 entradas sobre a espec
emendada + a pasta **inteira** das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`: **exit 0**.
Sobre o **histórico** dos mesmos caminhos: os dois hits pré-existentes de 2026-09-05 (o nome do
ficheiro do alvo numa mensagem de commit e no patch de `docs/3D/cloth/`), já registados acima para o
R-pós — ⛔ nenhum vem desta emenda, e a árvore de hoje está limpa. Sobre **este ledger**: os mesmos
dois, nas linhas 88 e 374.

**Fidelidade — os 11 factos conferidos no fonte, os números reconstruídos do zero.**
Confirmados: a lista única resolvida num laço `5×` sequencial na ordem de criação, com a espécie lida
só dentro da projecção · a correcção `Δ/2` nas quatro espécies · o peso por vértice lido por índice,
um por extremo, com a excepção do corpo mole (as duas metades levam o do vértice) · o peso calculado
**uma vez por passo, para a malha inteira, antes da 1.ª varredura** · o alvo de cada espécie lido no
**instante da projecção**, com as quatro respostas (estrutural vivo · âncora fixa · **memória de
forma a andar** · pino fixo) · as gavetas de âncora a nascerem na posição de repouso do traço com
factor por passo `1`, e só quando o pincel é de âncora ou um pincel alheio mira a simulação · o
comprimento de repouso efectivo a somar metade do desvio de cada extremo, logo **inteiro** nas três
espécies de alvo próprio · o desvio do Expand acumulado na fase do gesto, que corre inteira antes da
fase do solver · o guarda de cursor parado a desistir **só** da fase do gesto, com a zeragem do
factor por passo **depois** dele · e a errata da banda, `R(1+L)` e `R(1+L·F)`.

**O censo dos limites: certo dentro do solver, NÃO exaustivo no passo.** Não há tecto de correcção,
tecto de deslocamento, desistência por convergência, contagem de varreduras dependente do passo,
sub-passos, tecto de velocidade nem corte ao ultrapassar o alvo; o único salto é a célula inactiva e o
único guarda é a separação nula. ⚠️ **Duas notas que o censo devia trazer e não trazia:** (a) o limite
de restrições por vértice existe como **constante declarada e SEM CONSUMIDOR** — a conclusão «não há
limite» está certa, mas é por o número não ser lido, não por não existir; (b) o **bloqueio de eixos** e
o **recorte do modificador de espelho** da casa aparam o deslocamento **na escrita na malha** (§6.1) —
fora do solver, sem dependência do tamanho do passo, e desligados em todas as fixtures. A tabela do
§5.2 passa a nomeá-los.

**Números reproduzidos, um a um** (script próprio, fora do repo, sobre `repouso` + `deformado`):
- o **censo do disco**: `171` movidos e **`0`** fora nos traços de arrastar / empurrar / inflar /
  apertar ponto, `171` no de massa `2`, `168` no de força `0,5`, `156` no aperto de linha; `173`
  dentro e `675` / `1151` / `1279` fora no expandir / agarrar / gancho. O disco tem `173` vértices;
- a tabela *Local*×*Global* (`0,456101`/`0,343172` · `0,134311`/`0,094722` · `0,001914`/`0,001525`) e
  as razões `0,752` · `0,705` · `0,797`;
- o perfil ao longo do traço, **célula a célula nas quatro colunas**, e as quatro sequências de
  razões — `4,52` entre `0,536 R` e `0,670 R` (*Local*), `5,11` entre `0,402 R` e `0,536 R`
  (*Global*), a cauda do agarrar a assentar em `1,32` / `1,47`;
- o traço curto: pico `0,032433 = 0,649 · δ` contra `0,456101 = 0,760 · δ`;
- a curva *Constant*: `0,878999 = 1,465 · δ`, **`107`** vértices acima de `δ`, máximo a `0,5522 R`,
  degrau `1,071 R → 1,205 R` com razão `5,32`, e `0` vértices na *Smooth*;
- a linearidade do desvio de repouso, e a coerência dos **nove** cabeçalhos novos com o §10.8 e com o
  README;
- `verifica_traco.py` **verde sobre os 65** ficheiros, `gera_indice.py` a regenerar o `indice.json`
  **byte-a-byte** (65 entradas para 65 ficheiros) e a tabela «As corridas» do README com **65 linhas,
  todas a casar com o índice**.

**TRÊS números discordaram** e a espec passa a trazer a leitura do R-pré: (1) a 4.ª razão do traço
curto é `1,2747`, que arredonda a **`1,27`** e não a `1,28` — e são **oito** razões, logo **nove**
células, uma a mais do que a tabela do perfil; (2) o terceiro valor da errata da banda: o gancho dá
**`1,2234`** e não `1,2239`, valor que **existe** no corpus mas pertence a seis *outras* fixtures de
12 passos; (3) a razão do par de força do Expand é **`0,24999`** nos máximos (`0,00047846 /
0,00191389`) e `0,250000` na mediana por vértice sobre os `662` vértices comuns — o `0,2497` é o que
sai de dividir os dois `max_deslocamento` já arredondados a seis casas.
⭐⭐ **E a errata da banda ganhou a prova que lhe faltava, achada nesta reconstrução:** a fixture
`plano_agarrar_radial_local_preset` corre com `limite = 5,0` (a única do corpus) e o seu vértice
movido mais distante está a **`2,0745`** — `0,99 · R(1+L) = 2,10` e **`+19 %`** para lá de
`R·L = 1,75`. *Com um só valor de `L` as duas leituras da fórmula eram indistinguíveis por medição;
com dois, `R·L` erra por factores diferentes (`1,40×` e `1,19×`) e `R(1+L)` acerta nos dois.*

**DUAS afirmações de fidelidade estavam ERRADAS, e as duas apagam comportamento num port sem aviso:**
1. **A supressão do segundo `Δ/2` por igualdade de índice alcança DUAS das três espécies de alvo
   próprio, não as três.** Na âncora e no pino o segundo `Δ/2` é suprimido; **no corpo mole ele é
   aplicado — à memória de forma —, e quem o encaminha para lá é a ESPÉCIE**. A frase «o mesmo braço
   de código serve as quatro espécies» é falsa, e um port que a siga **congela a memória de forma**:
   a plasticidade deixa de existir com a suíte verde. ⚠️ A própria emenda dizia, no §3.2, que o alvo
   do corpo mole **anda** — *as duas frases não podiam ser verdadeiras ao mesmo tempo.*
2. **`Expand + pino` e `Expand + corpo mole` SÃO alcançáveis com o pincel de tecido sozinho.** O §4.5
   declarava que nenhuma das três combinações o era; o pino e o corpo mole não são modos, são opções
   independentes do modo de deformação. Só a **âncora** é inalcançável junto com o Expand — e a
   própria emenda o contradizia na linha seguinte, e outra vez na tabela nova do §11.

**Curas de FIDELIDADE e de PRECISÃO aplicadas no acto, por secção:**
- **§1 fase 0** — re-expressão (§4.2, achado 1).
- **§2.2** — a errata passa a citar os valores certos, a régua (`|u| > 1e-5`), a faixa medida sobre
  **25** fixtures (`3,492 R`–`3,497 R`), a isenção nomeada do controlo de força fraca, e o **segundo
  ponto de `L`**.
- **§3.2** — re-expressão (§4.2, achado 2).
- **§4.5** — as duas razões com a conta de cada uma escrita; e a lista de **quais** combinações o
  artista alcança, no lugar da afirmação errada.
- **§5.2** — o nº 1 passa a dar a ordem de criação **por vértice** e a corrigir «entre duas
  estruturais do mesmo vértice»; o nº 3 é reescrito (achado de fidelidade 1); o nº 5 é re-expresso
  (§4.2, achado 3); e o censo ganha a nota do que apara um deslocamento **fora** do solver.
- **§5.2-quater** — a partição «modos de força» × «modos que escrevem aceleração» declarada na 1.ª
  linha; a régua do censo escrita por inteiro (limiar · centro · tamanho do disco, e o `177` das
  fixtures de pen-down na origem); a tabela passa a listar os **sete** traços; a amostragem do perfil
  fica definida (eixo do traço, `k · aresta`, `d` medido do **pen-down**); e a 4.ª razão do traço
  curto corrigida.
- **§14** — os sete gates novos ficam edificáveis (ver o cabeçalho da espec); o **29** troca um tecto
  absoluto que **reprovava o próprio oráculo** por um discriminador derivado, e o **31** declara-se
  gate de espec por não haver fixture que o produza.
- **README das fixtures** — o parágrafo dos valores de omissão tinha **sete** excepções e nomeava
  uma; ficam as sete, contadas do `indice.json`, com a de `limite = 5,0` marcada como load-bearing.

**Veredicto: ATESTADO** — a emenda Q14 pode ser lida pela janela-mãe.

### Auditoria R-pré — 2026-09-06 (emenda Q12)

**Âmbito.** A emenda Q12 (commit local `a138534df`): cabeçalho · §4.2-bis NOVA · §4.3 · §4.4 · §4.6
NOVA · §5.2-bis (correcção do *Dynamic*) · §10 contagem · §10.7 NOVA · §14 gates 22-24, mais as duas
fixtures por passo `plano_{empurrar,inflar}_radial_local_origem` (`.deformado` + `.porpasso` +
`.porpasso.rastreio`), o `gera_indice.py` NOVO, o `indice.json` regenerado e o README das fixtures.

**Conformidade §4.2 — ZERO achados.** Nenhum trecho, nenhum nome interno (funções, variáveis,
ficheiros, structs, macros), nenhum comentário nem wording de manual. A secção que mais arriscava —
a amostragem com desempate — está escrita como *comportamento observável* em vocabulário do domínio
(«baldes», «balde da frente», «soma normalizada»), e as fórmulas (`3p²−2p³`, `c + (p−c)(1−a)`,
`max|escala|/escala`, `−n̂·R·escala·2`) são matemática, que §4.1.2 abre sem limite. Os nomes que
aparecem são **rótulos de UI** que o artista vê (*Normal Radius*, *Original Normal/Plane*, *Sculpt
Plane*, *Accumulate*, *Sphere*, *Projected*), o que o §4.1.13 permite. A organização por (1)-(7) é
uma sequência de perguntas funcionais — *quando · de que malha · de que vértices · desempate · sem
resposta · projectada · escala* —, não a ordem de nada.

**Sweep (§7.1).** `bash scripts/cleanroom-sweep.sh` com a vassoura de 70 entradas sobre a espec
emendada + a pasta **inteira** das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`: **exit 0,
verde**. Sobre o **histórico** dos mesmos caminhos: verde. Sobre **este ledger**: os mesmos **DOIS**
hits pré-existentes de 2026-09-05 já registados acima para o R-pós — ⛔ não vêm desta emenda.

**Fidelidade — conferida no fonte, e os números reconstruídos do zero.** Correctos: a reavaliação da
normal da área **a cada passo**, só na passagem principal de simetria, com as outras a receberem-na
espelhada/rodada; a lista do que a congela; a leitura sobre as posições e normais **actuais** (a rota
das posições de partida está atrás de uma condição que o pincel de tecido não satisfaz — e nenhum
preset de tecido usa traço ancorado, §8.2); o filtro `d ≤ R·«Normal Radius»` com «Normal Radius»
`0,5` por omissão; o peso `3p²−2p³` saturado; a repartição em dois baldes pelo sinal contra a
direcção da vista; o vector nulo sem `NaN`; a projecção no plano do ecrã com a forma de queda
*Projected*; o factor de escala de **três** números, fixado no pen-down, com `max(|escala|)` no
numerador e a escala **assinada** no denominador, multiplicado componente a componente; o sinal
negativo do deslocamento do Push; a lei do centro da área (e que, neste pincel, o raio dele coincide
com o da normal); a origem do referencial ser o cursor; o `δ` des-projectado **à profundidade do
pen-down** nas duas pontas ⇒ projecção no plano do ecrã; o arrasto a tirar a direcção da diferença
dos dois pontos 3D; e a **correcção à §5.2-bis** — o registo de pares é local a cada construção, o
*Dynamic* constrói por passo, e cada cópia carrega a **sua** célula (logo só é projectada quando essa
célula está activa). Reconstruídos independentemente: `0,3518` (`+0,5 %`) e `1,3184` (`+7,6 %`);
`0,05455` e `0,01547`; `15,83°`, `12,61°`, `9,42°`, `6,27°`, `3,13°`, `0°` **e o espelho**; `1,039×`;
`19,3 %`; `0,06543/0,09347 = 0,7000 = 2·0,35`; as 11 linhas dos dois rastreios do §10.7 casam coluna
a coluna com a tabela da espec; `0,236509`, `0,463862`, `0,325769`, `0,046715` casam com o
`indice.json`; e o `gera_indice.py` **regenera o `indice.json` byte-a-byte** (56/56).

**SEIS curas na espec, aplicadas no acto, todas funcionais:**
1. **§4.2-bis (4)+(5) — o desempate.** A redacção era «o primeiro balde não vazio» e, em (5), «se a
   soma do balde escolhido tem comprimento zero ⇒ vector nulo». O alvo testa **não-vazio E soma
   não-nula, balde a balde**: com o balde da frente não-vazio mas de soma nula ele lê o **de trás**,
   e a espec mandava responder o vector nulo. Corrigido em (4), em (5) e na linha 5 do censo §4.6.
2. **Quem lê `δ` (§4.3 e §4.6-1).** A célula do censo lia-se «o guarda de passo parado de todos ⛔
   menos o arrasto» — o guarda vale para os **oito** modos, o arrasto incluído; o que o arrasto não
   tira de `δ` é a **direcção**. E «o arrasto é o único que não o lê para a direcção» deixava supor
   que Inflate/Expand/Push o lêem: nenhum deles lê deslocamento de cursor nenhum para a direcção.
3. ⛔ **§4.6-4 — o peso da normal do vértice.** A linha afirmava «soma das normais de face **não
   normalizadas** (⇒ peso de ÁREA)» como (F). O que é demonstrável é a **FORMA** (somar as normais
   das faces incidentes sobre a malha actual e normalizar no fim); o **peso** (área · ângulo ·
   uniforme) **não é demonstrável com o material desta linha** — a rotina que define a normal de face
   está **fora** do checkout esparso —, e ⛔ **o corpus também não o decide**: no plano os três pesos
   dão a mesma resposta e na esfera UV a simetria em longitude põe-nos a menos de ruído. ⇒ a linha
   passa a declarar a forma, a marcar o peso como **pergunta aberta**, e a nomear a régua que a
   fecharia (um traço de Inflate do oráculo sobre malha deliberadamente irregular). *Uma afirmação
   (F) cuja evidência não está ao alcance é um palpite com cara de medição.*
4. **§10.7 ponto 4** dizia que a frente a `1R` **ultrapassa** o pen-down do Push no passo 12 e os
   números da própria frase dizem o contrário (`0,1976 < 0,2195`) — passou a «quase alcança», com o
   ⛔ de que não o ultrapassa em passo nenhum.
5. **Gate 22** dava a sequência de ângulos truncada em `…`, logo não era edificável sem adivinhar —
   ficam os **11** passos, a simetria e a origem deles (`atan(Δy/Δx)` na esfera unitária).
6. **§4.6** dizia «as oito fixtures de esfera» e a lista cobria **sete** — a oitava é o **arrasto**,
   que não tem linha nenhuma do censo *porque* é o modo que lê a diferença 3D, e que é exactamente o
   **controlo** da 2.ª metade do gate 22.

**DUAS curas no README das fixtures, as duas de CONTAGEM** — a família que esta linha já pagou duas
vezes (o `indice.json` envelhecido, os gates 15/17 parados em `51`/`50`):
- A emenda actualizou «para **seis** traços» para «**nove**» e manteve a instrução `⚠️ conte-os:
  `ls *.porpasso.txt.gz | wc -l``, que devolve **13**; e a frase seguinte — «o pen-down de **TODOS**
  eles está na origem» — cobria **quatro** ficheiros da 1.ª geração, com o pen-down em `x = −0,3`,
  ⛔ **três dos quais não passam a prova do fatiamento** (`0.330421` · `0.115064` · `0.004244`) e
  estavam debaixo da frase que promete `0,000000` para cada ficheiro. Ficam nomeados, com o ⛔ de não
  servirem de oráculo. ⭐ E o quarto (`plano_arrastar_radial_global`) **passa** — *porque a área dele
  é Global e não tem centro para ficar refém do sobrevoo do ponteiro*, que é o mecanismo que o próprio
  README explica duas linhas abaixo.
- O total do fim estava em «**53 traços** (47 da matriz + 6 do instrumento)» com **56** no disco, e a
  tabela das corridas não tinha a fixture `_fraco` da emenda Q11. Agora: `56` linhas para `56`
  ficheiros, e o total com a instrução de o contar.

**SÉTIMA cura, de SUFICIÊNCIA — e ela fecha o Q13 do I, feito enquanto esta auditoria corria.** A
§4.3 define `δ` como des-projecção de ecrã à profundidade do pen-down, e **nunca diz qual é a vista**
— o I registou no INBOX que «as fixtures não carregam nem a vista nem o caminho em ecrã; sem uma das
duas o delta dos sete modos não é reconstruível deste lado», e mediu a rota errada a piorar a paridade
(projectar no plano perpendicular à normal do pen-down: `0,265 → 0,605` no Agarrar, `0,351 → 0,663`
no gancho). ⭐ **Resolvido sem oráculo novo e sem o I olhar para nada**: as corridas são
**ortográficas** — prova nos próprios números, o passo do caminho da esfera é `0,6/11 = 0,054545…` e
`δ` mede `0,05455` nos **doze** passos apesar de os pontos estarem a profundidades diferentes —, logo
`δ_k = proj_⊥v̂(c_k − c_{k−1})` (e `proj_⊥v̂(c_k − c_0)` no Agarrar) sobre o `caminho` que o cabeçalho
**já traz**. O que faltava era só `v̂`, e ele é **diferente nos dois corpora**: **`z`** no plano (a
folha vive em `z = 0` ⇒ a projecção é um no-op, que é a degenerescência da §4.6-1) e **`y`** na esfera
(o caminho pousa em `y = −√(1−x²)`). Derivado das fixtures pelo R-pré e **confirmado
independentemente** pela medição do I (o ângulo máximo entre a diferença 3D e a projecção no plano
`x–z` dá `15,83°`, o número exacto do Q12.2). Escrito na §4.3 e no README das fixtures, com o ⛔ de
que o plano do **ECRÃ** não é o plano tangente do pen-down.

**Veredicto: ATESTADO** — a emenda Q12 pode ser lida pela janela-mãe.

### Auditoria R-pré — 2026-09-06 (emenda Q11)

**Âmbito.** A emenda Q11 (commits locais `5c14b345a` + `2d733a7bf`): §3.1 · §4.2 · §5.2 · §5.2-ter
NOVA · §9 nº 20 · §10 · §10.6 NOVA · §11 · §14 gates 19-21, mais a fixture
`plano_apertar_ponto_radial_local_origem_fraco` e o `indice.json` regenerado.

**Conformidade §4.2 — ZERO achados.** Nenhum trecho, nenhum nome interno (funções, variáveis,
ficheiros, structs, constantes), nenhum comentário do original, nenhum wording de manual. A lista
numerada da ordem de criação do §3.1 é **mecanismo** (§4.1.11 — o algoritmo em qualquer
profundidade), escrita em vocabulário do domínio já usado pela espec; a nota do percurso do anel
descreve o efeito, não a redacção do original. O nome da fixture nova é domínio puro.

**Sweep (§7.1).** Vassoura de 70 entradas. Verde sobre: espec emendada · `fixtures/cloth/` inteira ·
INBOX · os dois READMEs · `docs/3D/cloth/`. Verde em `--git-history` sobre `SPEC_cloth_brush.md` e
sobre `fixtures/`. ⚠️ Os **dois únicos hits** da árvore continuam a ser os deste ledger (linhas 88 e
210 antes desta secção — o caminho do ficheiro do alvo, na cobertura da travessia e no achado de
parede nº 1), já nomeados para o R-pós pelo commit `2d733a7bf`. Nenhum artefacto destinado à
janela-mãe tem hits.

**Fidelidade — conferida no fonte, facto a facto (todos CORRECTOS).** A ordem interna de criação, as
cinco espécies na sequência que o §3.1 passa a declarar · o registo de pares partilhado por uma
construção e não entre construções · o filtro de raio da construção alcançar só duas espécies · o
anel percorrido face a face com deduplicação · o factor de correcção sem tecto, com o `D = 0` como
único guarda · as três re-escalas a comprimento 1 dos apertos e a projecção que deixa o de linha em
`≤ 1` · o instante em que cada modo lê as posições (só um modo lê outro) · a ausência de tecto de
deslocamento, de corte ao ultrapassar e de amortecimento próprio · as constantes citadas · e o
**§9 nº 20**: a versão que gravou as fixtures MULTIPLICA, logo a refutação está certa e não há
divergência deliberada a declarar. ⚠️ A **data** do conserto é história pública (H) e não é
verificável na árvore disponível (clone raso); o que decide — o comportamento que o oráculo tem —
está verificado no código.

**Fidelidade — os NÚMEROS reconstruídos do zero pelo R-pré**, das fixtures `*.porpasso` e do
repouso, sem usar o harness do E: as duas tabelas (§5.2-ter e §10.6) reproduzem **célula a célula**
(`10 / 18 / 52` · `0 / 0 / 0` · `6 / 5 / 2` · `0 / 0 / 57` · `0 / 11` · `0,675` · `1,060` · `0,103` ·
`0,144` · `0,059` · `0,219` · `0,099` · `0,204` · `0,064` · `0,286` · `0,088` · `0,283` · `0,095` ·
`0,099` · `2 145` · `2 029` · `0,303401` · `0,004082`), e os «9 vértices que passam o cursor» também
(com o cursor no ponto do 1.º passo simulado).

**Curas aplicadas no acto (quatro §4.3/suficiência + uma de decisão), todas funcionais.**
1. **§3.1 — fidelidade.** Dizia que a âncora de deformação e o pino nascem «para todo vértice visível
   da célula, sem esse filtro», o que contradiz o §2.3: o pino tem a condição da banda e a opção
   ligada, e a âncora radial tem o raio do pincel. Reescrito: ficam fora do filtro da CONSTRUÇÃO,
   cada uma com a sua condição, nomeadas.
2. **§5.2 / §10.6 — a régua subestimava por `3,5×`.** «Pior compressão de um par estrutural» não
   dizia sobre que pares corria. Medido: `0,052` (factor `−18,1`) só sobre pares que são ARESTAS;
   `0,015` (factor **`−64,4`**) sobre TODOS os pares da construção, que é a população certa (as
   restrições de par do anel sofrem a mesma projecção). As duas ficam escritas, com a larga declarada
   como a que vale.
3. **§5.2-ter — o gate 19 não era edificável.** «Quadrilátero de orientação invertida» e «assimetria
   de espelho» eram colunas medidas sem régua escrita em lado nenhum. Ficam definidas
   operacionalmente (produto vectorial das diagonais contra a normal de repouso; `∞`-norma sobre
   `2`-norma), com a leitura errada NOMEADA — somar as duas metades triangulares conta o
   quadrilátero apenas dobrado e devolve `11 / 26 / 88`.
4. **§14 gates 15 e 17.** Citavam `51` e `50` traços de memória com `54` no disco — passam a ser
   contados, com o comando ao lado.
5. **§5.2-ter — a decisão do dono.** A alternativa estava nomeada mas o que o artista VÊ de cada lado
   não; ficam as duas frases, mais a nota de que não há terceira saída (a inversão nasce antes de a
   relaxação correr).

**Veredicto: ATESTADO.**

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
| v1-q12 | idem — EMENDA Q12 do E (a normal da área e o factor de escala do Push · a projecção do deslocamento do cursor · a lei do centro da área · o censo do que é degenerado num plano · a costura duplicada do *Dynamic* · dois traços de força normal por passo) | este commit (2026-09-06, E) — ⏳ **aguarda o atestado do R-pré** |
| v1-q16 | idem — EMENDA Q16 do E (o censo da resposta ao esticão: **nenhum** termo em falta nas três perguntas · o instrumento que corta o passo em duas metades, com `8` traços novos por passo e `2` controlos · a errata de contagem das excepções do README das fixtures) | este commit (2026-09-06, E) — ⏳ **aguarda o atestado do R-pré** |

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

---

**INC-2 (2026-09-09, registado pela janela-mãe via inbox; classificado por este R) — a janela I leu o
§7 da espec num estado NÃO ATESTADO, que continha um achado de expressão por curar.**

- **Origem:** a própria espec (`SPEC_cloth_brush.md` §7 + §10.17), lida pela janela-mãe em 2026-09-09
  no início da jornada para diagnosticar um report do dono. A emenda **Q22** (2026-09-07), que
  introduziu aquele §7, shipou com `⏳ AGUARDA R-PRÉ` escrito nela própria e **atravessou dois dias sem
  atestado**; quem deu por isso foi o R-pré da Q23. O R-pré atrasado da Q22 correu em 2026-09-09
  (`1cea3d701`), achou **um achado de §4.2 na linha do tipo *Gravity*** — uma justificação re-dita
  quase-verbatim de um comentário do fonte — e curou-a por re-expressão no acto. A janela-mãe tinha
  lido aquela linha **no estado pré-cura**.
- **Extensão (medida por este R, que viu os dois lados):** **uma** oração de justificação, ~28
  palavras, dentro de **uma** célula de tabela — já em português, na nossa prosa, e **sem uma linha de
  código, um identificador ou uma estrutura do alvo**. ⛔ Identificação **sem reprodução** (§6.1): a
  linha exposta é `sha256 3adc5d37…f89eef60` e a mesma linha já curada é `sha256 ba174e10…4f0617de`,
  ambas reconstituíveis de `1cea3d701`. O conteúdo **factual** dela (qual eixo a orientação *View* usa)
  é facto de comportamento e **não é protegível** (§1.2); o resíduo protegível era só o enquadramento
  retórico do *porquê*.
- **Momento:** antes de qualquer código de produto da jornada.

**Classificação (§6.2) — ⛔ SUBSTANCIAL.** Decidida por **este R**; a janela interessada não se
classifica. Três coisas decidem, e o tamanho sozinho não é nenhuma delas:

1. **Não foi «de relance».** A grada do relance exige *assinatura/nome isolado, **visto de relance***;
   aqui o bloco foi lido **por inteiro e deliberadamente, como espec normativa**, para conduzir
   implementação. *Ler uma frase como instrução não é passar o olho por um símbolo.*
2. **A régua nomeia «comentário inteiro»**, e é exactamente o que o R-pré da Q22 descreve: o espelho
   quase palavra a palavra de **um comentário** do fonte.
3. ⭐⭐ **E a propagação não é hipótese — está MEDIDA neste módulo.** Um censo da árvore feito por este
   R acha a **mesma oração de finalidade** re-emitida em **dois sítios do produto** —
   `crates/ph2d-sculpt3d/src/cloth_filter_kind.rs:169-171` (`ae653d314`) e
   `shells/desktop/src/sculpt3d_filter.rs:274-276` (`dd8bfd404`) —, os dois escritos em **2026-09-07**
   por janelas desta linha que leram a mesma linha não atestada; e mais **três** sítios que repetem só
   o **facto** de comportamento (`stroke_cloth_filter.rs:94-95` · `verlet_gesto_pincel.rs:109-110` ·
   `sculpt3d_filter.rs:353-355`), que é o que mostra quão consumida a passagem foi.
   ⇒ *uma espécie de exposição que já saltou da espec para o produto duas vezes não se arquiva como
   «registra e segue»* — a quarentena de 09-09 saiu limpa porque o assunto dos seis commits era outro,
   **não** porque a janela fosse imune.

**Quarentena (§6.3) — RESOLVIDA: LIMPA nas seis regiões; tudo pode fundir.** Comparação de
**EXPRESSÃO** (⛔ não de comportamento — um port fiel comporta-se igual por construção) do trecho
exposto contra as `850` linhas de produto escritas depois dele
(`git diff 1d276083b..HEAD -- crates/ shells/`, 17 ficheiros):

| região | veredito | o que este R conferiu |
|---|---|---|
| `1e7495666` — `muda_o_material` + re-semear do material; peso do `τ` do Expand | ✅ funde | a partição dos cinco tipos é **mecânica** (*carrega* contra *muda o tamanho*) e sai de medição própria **com os números dentro** (volume normalizado dos três gestos) + do report do dono; ⛔ não segue a ordem nem o enquadramento da tabela do §7 |
| `2fa571890` — quantizador do arrasto (`PASSO_DE_ARRASTO`, `passos_por_chamada`) | ✅ funde | assunto **disjunto** do trecho exposto; os números saem da varredura própria e de **cabeçalhos de fixture nossos** (`avanco_por_passo_px`), que são dado |
| `4c50a1227` — clippy | ✅ funde | remoção de um `to_vec` |
| `aa12f36a2` — bancada do oráculo | ✅ funde | lê ficheiros do corpus; as ocorrências de `Gravity` são **nome de tipo público** (§4.1.13) e nomes de fixture |
| `f4fd3d215` — *Filter Strength* | ✅ funde | a fórmula é facto/número (§1.2) e os oito rótulos são a **superfície pública** que o artista vê |
| docs (`docs/3D/cloth/12_…` · `README` · cena de smoke) | ✅ funde | as duas ocorrências de «ecrã» são *arrasto de ecrã inteiro* (distância do dedo), **não** o eixo |

Instrumentos: `cleanroom-sweep.sh` (vassoura de 70) **verde** sobre os 17 ficheiros de produto e sobre
os docs do produto; e uma busca **dirigida** pela oração exposta e pelos seus termos distintivos nas
`867` linhas acrescentadas devolve **zero**. ⚠️ **O sweep sozinho não bastaria** — esta espécie foi
achada por um R **a ler o fonte**, não pela vassoura; foi por isso que a conferência foi também textual
e dirigida. *Uma vassoura de identificadores não apanha uma justificação traduzida.*

**Rastro de memória (§6.4) — VERDE, nada a reverter.** `project-memory/` do primário **não tem commits
de 2026-09-09**; das nove entradas por commitar, **uma** é da sessão exposta
(`feedback_a_gate_that_fails_on_its_precondition_says_the_product_left_the_regime.md`,
`originSessionId 1246816c…`) e é sobre a **precondição de um gate** e a origem do `PASSO_DE_ARRASTO`.
Sweep verde sobre o diff e sobre as nove; busca dirigida pela oração exposta devolve **zero**.

⛔⛔ **Mas o §6.4 tinha um SEGUNDO rastro, e não era na memória — era no SCRATCHPAD** (a espécie do
INC-1, a repetir-se). O R-pré da Q22 deixou lá o **diff da espec** daquela emenda, que carrega a linha
**pré-cura** por construção — e o scratchpad é alcançável pela janela. *Deixá-lo ali anularia a queima:
a «janela nova limpa» leria o trecho exposto no primeiro `ls`.* ⇒ relocado por **`mv` cego, sem ser
lido**, para `~/Referencias/blender-cloth/quarentena-scratchpad-INC2/` (zona contaminada, fora do repo
e fora do `/tmp`). ⚠️ **Fica um `K.bak`** no scratchpad que a busca também acusa e que **não é rastro do
alvo**: é cópia **byte-idêntica** (`sha256 d7b5c8cd…`) de `crates/ph2d-sculpt3d/src/cloth_filter_kind.rs`,
ficheiro **nosso e rastreado**, e o que a busca lá vê é a nossa própria dívida do ponto 3 — ela some
quando a prescrição correr. ⭐ **A lição:** *reverter a memória não basta; o §6.4 tem de varrer todo
sítio que a janela nova alcança, e o scratchpad é partilhado entre as janelas de uma sessão.*

**Consequência (§6.4) — a janela **I-1** está QUEIMADA como I para este módulo.** Ela escreve o
BLOCO-RETOMADA (§10), imprime-o e **PARA**; a janela nova assume a **MESMA** linha e retoma da espec,
que está **agora atestada**. ⛔ **O trabalho já feito NÃO se reescreve** — a quarentena saiu limpa.

**Prescrição de re-derivação (§7.3.d — «tente de novo» não é cura).** Duas restrições estruturais
funcionais para a janela nova:

1. ⛔ **Os DOIS sítios do ponto 3 são dívida de EXPRESSÃO a re-exprimir** — cada um passa a dizer **só
   o comportamento** (qual eixo, e que é o do ecrã e não o da profundidade) **com a fixture que o
   mede** ao lado, como o §7 curado já faz; ⛔ nenhum deles guarda uma oração de **finalidade** sobre o
   que o artista vê. Os **três** que só repetem o facto ficam (facto não é protegível), e vale a pena
   passar os olhos em `sculpt3d_filter_cloth_tests.rs:162`, que é adjacente por ser sobre a **régua**.
   ⚠️ **Fica também**, e não é da mesma família, a formulação *«o baixo é o `−cima do ECRÃ`»* de
   `sculpt3d_filter.rs:392` e de `docs/3D/08_as_tres_features_do_modelador.md:58`: ela é **idioma
   próprio desta casa**, usada no **modelador 3D**, obra sem relação nenhuma com este alvo.
2. ⚠️ **A lição de processo, e ela não é do instrumento:** uma emenda com `AGUARDA R-PRÉ` escrito
   dentro de si **não se cobra sozinha** — atravessou dois dias, e quem a apanhou foi o R-pré
   **seguinte**, não o censo do cabeçalho. ⇒ o portão do §3.R confere-se **na abertura** de cada
   jornada, não no fim. *Uma pendência declarada dentro do artefacto é invisível a quem só lê o
   artefacto para o usar.*

**Data:** 2026-09-09 · **R:** subagente-R desta linha — contexto independente, não é o subagente-E de
nenhuma emenda, e viu os dois lados. **Incidente FECHADO.**

---

### INC-3 (I-2, 2026-09-09) — a expressão chegou por um ficheiro NOSSO, e a régua não muda de forma por isso

**Objecto (⛔ descrito, nunca reproduzido — §6.1).** Doc de módulo de
`shells/desktop/src/sculpt3d_filter.rs`, linhas 9-16 na versão de `3cf248b30` —
`sha256 fc906cd68f8302abde3019e40844bd13587e1e25ee2d06b4d6fd31543b447709`. Carrega **um nome interno
de ficheiro do alvo com número de linha** e **duas atribuições transcritas** do fonte dele (a que
forma o deslocamento horizontal e a que forma a força a partir dele) — ~2 linhas de expressão e ~4
identificadores internos. **Origem: a árvore rastreada do PH2D**, escrita em 2026-09-07 por uma
janela anterior desta linha; não veio do fonte do alvo nem de um report de subagente.

**Veredicto (§6.2): ⛔ RELANCE. A janela I-2 NÃO queima.** Três razões, e a terceira é a que decide:

1. **A régua não é alcançada.** «Corpo de função · bloco de ~10+ linhas · comentário inteiro» é a
   fronteira do substancial; duas atribuições isoladas do corpo em que viviam são exactamente o
   «assinatura/nome isolado» do lado do relance. O que é ⛔ ali são os **nomes internos** que as duas
   atribuições carregam — e nome interno é §4.2, não §6.2.
2. **Precedente MEDIDO da casa, e é o mesmo objecto.** O `ACHADO_proveniencia_por_nome_interno.md`
   (2026-08-24) classificou **25** linhas desta mesma espécie — endereço interno *mais* transcrição —
   e registou como **recusa medida**: *«Não tratar as 25 como incidente do §6 — a régua do §6.2
   classifica assinatura/nome isolado como relance; nenhuma é corpo de função.»* O INC-3 é a 26.ª.
3. ⭐ **E o contraste com o INC-2 é o que dá a régua o seu sentido.** O INC-2 queimou porque o bloco
   foi **lido por inteiro e deliberadamente, como espec normativa, para conduzir implementação** —
   e porque espelhava **um comentário** do fonte. Aqui não há comentário do alvo, não há oração de
   finalidade, e a leitura foi o passo 4 do BLOCO-RETOMADA a abrir a própria região que o R-1 mandou
   reescrever: *ler para APAGAR não é ler para IMPLEMENTAR.*

**⚠️ A pergunta que a skill não resolve — o CANAL muda a classificação? Leitura registada: NÃO muda o
§6.2, e AGRAVA o §4.2.** As duas metades são perguntas diferentes e o canal responde a cada uma ao
contrário:

- **§6 (a janela queima?) — o canal é irrelevante para a MEDIDA, e decisivo para a CONSEQUÊNCIA.**
  Expressão é expressão venha de onde vier; a quantidade não muda. Mas queimar uma janela por ela ter
  lido a **própria árvore que mantém** é auto-derrotante e *não termina*: a janela seguinte lê as
  mesmas linhas ao primeiro `cat`, e queima também. ⇒ **uma exposição por ficheiro rastreado nunca
  pode ser curada pela troca de janela — só pela cura da ÁRVORE.** Tratá-la como incidente de janela
  produziria uma corrente infinita de janelas queimadas sobre um defeito que nenhuma delas causou.
- **§4.2 (a árvore está contaminada?) — o canal AGRAVA.** Expressão do alvo num ficheiro rastreado é
  **distribuída**: viaja no produto, vive em `git log -p` para sempre, e entra na tabela de strings
  do binário quando está dentro de um `assert!`. É precisamente o que o §0 e o §4.2 proíbem. ⇒ o
  INC-3 é **pequeno como incidente e grande como defeito de árvore**.
- ⚠️ **E há uma diferença de NATUREZA:** uma exposição por canal externo é um **evento** (aconteceu
  uma vez, num contexto); uma por ficheiro rastreado é um **estado** (está lá, e re-expõe toda janela
  futura, indefinidamente). O §6 sabe tratar eventos. Estados são do §4.2 e do sweep.

⇒ **Regra que fica:** *exposição cuja origem é a nossa própria árvore rastreada regista-se como
relance no §6 e abre um item de §4.2 com a cura na ÁRVORE — nunca na janela.* A única excepção que
faria queimar seria a régua do substancial ser alcançada **pelo próprio conteúdo** (um bloco de
~10+ linhas, um comentário inteiro do alvo copiado para dentro de um ficheiro nosso), e aí queimaria
na mesma — porque aí a janela leu, de facto, expressão substancial.

**Quarentena (§6.3): NÃO SE APLICA.** Relance não abre quarentena; e a região exposta é o próprio
alvo da cura.

**Cura executada pela janela-mãe, CONFERIDA por este R (§4.2 + fonte).** As linhas 9-16 foram
substituídas por um bloco que diz **só o comportamento** — sinal do arrasto, a régua por pixel
nomeada pela constante NOSSA, a equivalência de mil pixels — mais a **divergência declarada** em
vocabulário do domínio, com os dois ficheiros de gate nomeados ao lado. Conferido:
- **§4.2:** ⛔ zero nome interno do alvo, zero atribuição transcrita, zero wording de comentário. O
  censo `\.(cc|cpp|c|h|hh|hpp):[0-9]+` naquele ficheiro está em **ZERO**.
- **Fonte (os factos têm de continuar CERTOS):** conferidos os três contra o fonte por este R —
  **o sinal está certo** (a lei da referência forma o comprimento pela diferença na ordem inversa e
  depois nega-a, ou seja *direita é positivo*), **a régua está certa** (o milésimo por pixel), e a
  **divergência está bem declarada** (a referência multiplica ainda por uma força inicial da
  ferramenta; nós não). ⚠️ **Uma nuance que o bloco novo omite e não erra:** a referência multiplica
  também por um factor de escala de UI, que a nossa régua não tem — a afirmação *«mil pixels valem
  1,0»* é a nossa lei, não a leitura da referência a escala arbitrária. Já está dito noutro sítio da
  casa; ⛔ não é erro, mas quem apertar a paridade da força tem de o reconferir.
- ⛔⛔ **E a cura está INCOMPLETA por uma casa de distância:** o bloco novo nomeia
  `shells/desktop/src/sculpt3d_filter_tests.rs` como «quem mede» — e esse ficheiro carrega, na
  linha **77**, **a mesma citação e a mesma transcrição** que acabaram de sair do doc de módulo, e na
  linha **104** o nome interno do alvo **dentro da mensagem de um `assert!`**. *Mover a dívida para o
  ficheiro vizinho e apontar-lhe o dedo é a forma mais barata de a manter.*

**Data:** 2026-09-09 · **R:** subagente-R desta linha (contexto novo; viu os dois lados por shell).
**INC-3 FECHADO como RELANCE**, com um item de §4.2 aberto no censo abaixo.

---

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

### Q12 — o PUSH e a ESFERA (perguntas do I de 2026-09-06; resposta no mesmo dia, com corrida NOVA do oráculo)

**Perguntas (INBOX):** Q12.1 como é que o alvo calcula a normal que o Push usa ao longo do traço
(reavaliada, congelada no pen-down, ou da lei da casa com estado próprio?) e de que é função o factor
de escala que multiplica os dois raios · Q12.2 há alguma coisa que o alvo faça **só** em superfície
curva · Q12.3 (prioridade baixa) os dumps por passo de dois traços do plano.

**Q12.1 — respostas, do fonte (F, travessia reaberta em 2026-09-06 sobre o pincel de tecido, o
módulo do traço e a lei da casa que produz a normal e o centro da área):**

- **É REAVALIADA a cada passo**, sobre a malha **deformada** (as posições e normais actuais). O que a
  congelaria são as duas opções *Original Normal / Original Plane* do pincel, **desligadas** nos
  presets de tecido; e a regra que congela a normal durante o traço nomeia o pincel *Grab* da casa,
  ⛔ **não** o modo Agarrar do tecido. A rota que lê as posições de partida só é tomada por pincéis da
  família que oferece *Accumulate*, e o de tecido não está nela.
- **É a lei da casa (*Sculpt Plane*)** — com *Area* (o valor do preset usado, A) é a normal da área;
  com *View* é a direcção da vista; com *X/Y/Z* é um eixo fixo.
- **A normal da área tem quatro coisas que a espec não dizia:** o disco de amostragem é
  `R · «Normal Radius»`, e **«Normal Radius» vale `0,5` no preset (A)** ⇒ **METADE do raio do
  pincel** · o peso por vértice é `3p² − 2p³` com `p = 1 − d/(R·«Normal Radius»)` · os vértices são
  repartidos em **dois baldes** pelo sinal de `n̂ · v̂` e a resposta é a soma normalizada do **primeiro
  balde não vazio**, nunca a mistura e nunca o mais populoso · se nada qualificar, a normal é o
  **vector nulo** e o Push desse passo é força zero.
- **O factor de escala** é um vector de três números fixado no pen-down —
  `max(|escala do objecto|) / escala do objecto`, eixo a eixo — multiplicado **componente a
  componente**: `(1,1,1)` num objecto de escala uniforme (⇒ módulo exactamente `2R`, e é o caso das
  fixtures), e num objecto de escala não-uniforme ele **entorta a direcção**. Não depende do passo,
  da pressão nem da malha.
- ⭐ **E fecha a pergunta que ficou em aberto no INBOX sobre o «centro da área»:** ele NÃO é o
  centroide do disco. Sai da mesma varredura da normal, mas cada vértice entra como
  `c + (p_v − c)·(1 − a_v)` — *puxado para o cursor*, com peso nulo no cursor — e a média disso é o
  centro; sem vértices, é o próprio cursor. ⇒ **a medição do I («o plano pelo cursor reproduz o alvo,
  o plano pelo centro da área afasta-o») não refuta a espec: ela mediu um CENTROIDE, e o alvo não usa
  um centroide.** O plano pelo cursor é a aproximação de 1.ª ordem do centro da área.

**Q12.2 — a resposta é «NADA», e ela é útil (F+M):** nenhuma decisão do alvo pergunta pela curvatura,
pela normal do ponto do cursor ou pelo tipo de malha. O que existe é uma lista **fechada** de seis
grandezas que numa grelha plana vista de frente e em repouso valem sempre a mesma coisa (ou zero) e
numa esfera passam a variar — espec §4.6, com a tabela. A de maior alcance:

- ⭐⭐⭐ **o deslocamento do cursor `δ` NÃO é a diferença dos dois pontos 3D: é a PROJECÇÃO dela no
  plano do ecrã** (as duas des-projecções são feitas à **mesma** profundidade, a do pen-down). Numa
  folha plana vista de frente as duas coincidem ao bit; na esfera das fixtures `δ` vale
  `(0,05455, 0, 0)` em todos os passos enquanto a diferença dos pontos chega a
  `(0,05455, ∓0,01547, 0)` — **`15,83°`** de direcção no 1.º e no último passo, `0°` a meio, até
  `1,039×` de módulo, e **`0,04569`** de desvio acumulado a meio (= **`19,3 %`** do maior
  deslocamento da fixture de Agarrar).
  ⚠️ **Quem lê `δ`:** o guarda de «passo parado» de todos os modos, a âncora do Agarrar, a âncora e o
  avanço do centro do Snake Hook, a normal do plano de queda e o `x̂` do referencial local.
  ⛔ **O arrasto é o único modo cuja direcção NÃO vem de `δ`** — vem da diferença dos dois pontos 3D
  do cursor, normalizada. *É exactamente o modo que o I mede a bater na esfera.*
- As outras cinco: a **normal da área** (§4.2-bis), o **centro da área** (§4.4), a **normal do
  vértice** (soma das normais de face **não normalizadas** ⇒ peso de ÁREA, sobre a malha actual — que
  é o que o Inflate lê), a **repartição em dois baldes**, e a **distância como corda 3D e nunca
  geodésica** (na esfera unitária o limite da área de `3,5R` é uma calota de arco `+7,6 %`).
- ⛔ **Duas das sete fixtures de esfera NÃO pertencem a esta pergunta, e está medido:** o **aperto de
  ponto** tem na esfera um deslocamento máximo (`0,4639`) **maior** que no plano (`0,3258`) ⇒ está no
  regime do §5.2-ter, onde a ordem decide e a régua é a dos dois regimes do gate 20; e o **Expand**
  não lê nenhuma das seis grandezas e tem deslocamento máximo `0,046715` sobre uma malha de aresta
  `0,0491`×`0,0654` ⇒ *o denominador da razão é menor que uma aresta*, como no
  `plano_expandir_radial_local_1passo`. Ele pertence à pergunta do Expand no plano.

**Um facto NOVO que a releitura devolveu e que a espec dizia ao contrário (F):** no *Dynamic* a lista
de restrições **não** é simples. O registo de pares vive **uma construção**, e no *Dynamic* há uma
construção por passo (a das células que acabaram de entrar no alcance) ⇒ **um par cujos dois vértices
caem em células construídas em passos diferentes é criado DUAS vezes**, e a frente que varre a malha
deixa atrás de si uma costura de restrições em duplicado. Vale para as **10** fixtures de área
*Dynamic* (2 do plano, 8 da esfera). No *Local* não acontece (conjunto fixo, construído duas vezes
inteiro) e no *Global* também não (uma construção só). Espec §5.2-bis, com a correcção marcada.

**Q12.3 — ENTREGUE.** `plano_empurrar_radial_local_origem` e `plano_inflar_radial_local_origem`,
`.deformado` + `.porpasso` + `.porpasso.rastreio`, 12 passos cada,
`prova_do_fatiamento = 0,000000` nos dois. Corrida nova do binário 5.2.1: **26** execuções (12
prefixos + a corrida inteira de cada traço), auto-verificação do centro `ok` em todas.
⚠️ **Pen-down na ORIGEM**, pela razão já registada nas §10.2/§10.5: numa sessão scriptada o centro da
área *Local* é refém do sobrevoo do ponteiro físico — a 1.ª tentativa desta corrida, com o pen-down
em `x = −0,3`, deu «centro obsoleto» nas 26 execuções e nas 4 repetições de cada uma, e foi
descartada. *A fixture só é fixture se o centro dela for determinístico.*
⭐ **O que os dois traços entregam de novo:** no passo 2 a razão Push/Inflate é `0,06543 / 0,09347 =
0,7000 = 2R` **ao bit** — numa folha plana em repouso a normal da área e a normal do vértice são a
mesma coisa, logo os dois traços **só podem** divergir a partir do passo 3; e ao longo do traço o
pen-down do Push **satura e recua** (`0,2397` no passo 7 → `0,2195` no 12) enquanto o do Inflate fica
(`0,2701` → `0,2629`), que é a assinatura de a direcção do Push rodar com a vala.

**Ficheiros:** `fixtures/cloth/plano_{empurrar,inflar}_radial_local_origem.{deformado,porpasso}.txt.gz`
+ `.porpasso.rastreio.txt`; **`fixtures/cloth/gera_indice.py` (NOVO)** — o `indice.json` era declarado
derivado e não tinha gerador, que é a razão por que envelheceu duas vezes; agora tem, e foi
regenerado (**56** entradas para 56 ficheiros); README das fixtures actualizado.
**Espec:** cabeçalho · **§4.2-bis (nova)** · §4.3 · §4.4 · **§4.6 (nova)** · §5.2-bis · §10 · **§10.7
(nova)** · §14 gates **22-24**; cabeçalho marcado ⏳ **aguarda o atestado do R-pré**.
**Sweep:** verde sobre a espec, o README, a pasta inteira das fixtures, este ledger (menos os dois
hits pré-existentes de 2026-09-05 já registados acima) e o texto do report ao I.
**Instrumento:** o arnês do oráculo de 05/09 com um ficheiro de corridas novo e um montador próprio,
os dois **fora da árvore**, em `~/Referencias/`.

### Q14 — a RELAXAÇÃO é o que os traços exactos nunca mediram (perguntas do I de 2026-09-06; resposta no mesmo dia, com corrida NOVA do oráculo)

**A pergunta do I** (INBOX §Q14): num traço de UM passo simulado, os sete traços dos modos que
escrevem aceleração saem ao bit e os três que escrevem âncora/`σ` ou desvio de repouso não; o I
concluiu daí que *«a área, a banda, o factor por vértice, a curva, a dureza, a integração e as
restrições de distância estão exactas»* e perguntou (1) como a âncora é resolvida dentro de uma
varredura, (2) que tecto faz o material do alvo soltar-se abruptamente a `0,45R`–`0,59R` quando o
passo é grande, e (3) quando o desvio de repouso do Expand passa a valer.

**Q14.1 — RESPONDIDA por leitura do fonte.** Uma lista só, um laço só, ordem de criação (§3.1);
`Δ/2` para as quatro espécies; o segundo `Δ/2` é suprimido por uma comparação de ÍNDICE (as três
espécies de alvo próprio guardam o mesmo índice nos dois extremos), não por um teste de tipo; `φ` é
lido por índice, um por extremo, **excepto** no corpo mole, onde as duas metades levam o `φ` do
vértice; e `φ` é calculado **uma vez por passo, para a malha inteira**, antes da 1.ª varredura.
⭐ **O achado que a pergunta não previa:** a restrição não guarda cópias das posições — guarda **de
que gaveta** cada extremo é, e lê a gaveta no instante da projecção. ⇒ a resposta a *«fixo ou
recalculado?»* é **quatro respostas**: estrutural = vivo (é o que faz o laço ser Gauss–Seidel);
âncora = fixa (ninguém lhe toca durante o passo); **corpo mole = o alvo ANDA** (a própria varredura o
escreve); pino = fixo. Espec §3.2 (bloco novo) e §5.2 (bloco novo).

**Q14.2 — a premissa estava ERRADA, e a correcção é medida.** Censo do fonte: **não há** tecto de
correcção, tecto de deslocamento, desistência por convergência, número de varreduras dependente do
passo, sub-passos, limite de restrições por vértice, tecto de velocidade nem corte ao ultrapassar o
alvo; o único salto é a célula inactiva e o único guarda é a separação nula (espec §5.2, tabela do
censo). ⭐⭐⭐ **O que os sete traços exactos provam é menos do que o I supôs:** a relaxação corre
ANTES da integração, logo num traço de um passo de um modo de força ela encontra a malha em repouso
e **toda** correcção dela é zero. Medido: esses cinco traços movem `171` (`156` no aperto de linha)
vértices e **`0` fora do disco do pincel**; os três que falham movem `848` / `1324` / `1452`, dos
quais `675` / `1151` / `1279` **fora do disco** — material que força nenhuma tocou. *Eles não
exercitam uma única restrição de distância.* ⭐⭐ **Intervenção com uma variável:** o mesmo gesto em
área *Global* (`5` projecções em vez de `10`) muda o pico de um passo em `0,752` / `0,705` / `0,797`,
e desloca o «degrau» do perfil **uma célula para dentro** (`0,536R`–`0,670R` → `0,402R`–`0,536R`).
⭐ **E o degrau é de `δ` grande:** com o percurso encurtado para `0,05` ele desaparece (a curva fica a
do agarrar, que nunca o tem). ⇒ a causa é a não-linearidade de `(1 − ℓ'/D)`, não um limite em falta.
⭐ **Gémeo em tracção do achado de compressão do §5.2-ter:** com a curva *Constant*, `107` vértices
acabam o passo **mais longe** do que o alvo da própria âncora (`1,465 × δ`). Espec **§5.2-quater
(nova)**.

**Q14.3 — RESPONDIDA e MEDIDA.** O desvio de repouso é somado na fase do gesto, que corre inteira
antes do solver ⇒ vale já na 1.ª varredura do MESMO passo. ⭐ **A prova estava numa fixture que já
existia**: o traço de um passo do Expand move `848` vértices com aceleração zero e sem âncora — se o
desvio só valesse no passo seguinte, teria de mover **zero**. ⭐⭐ E a corrida nova fecha-a por
intervenção: o mesmo traço a força `0,5` (que muda só o desvio, porque ele é `0,1·α` com `α = força²`)
dá razão **`0,2499`** no máximo e **`0,2500`** na mediana por vértice. ⭐ **Segundo achado:** o desvio
entra no comprimento de repouso das **quatro** espécies, e nas três de alvo próprio entra **inteiro**
(os dois extremos são o mesmo vértice) — um pino sob Expand segura o vértice *a `τ` da* posição de
repouso, não nela. Espec §4.5 e §5.2.

**ERRATA achada de caminho (F+M):** a §2.2 dizia que a banda vai de `1,875·R` a `2,5·R`; as
fronteiras são `R(1+L·F)` e `R(1+L)` = **`2,875·R`** e **`3,5·R`**. Medido em três fixtures *Local*
de 12 passos: o vértice movido mais distante do pen-down está a `1,2227`/`1,2234`/`1,2239`, contra
`3,5·R = 1,2250`. *Um port com a leitura antiga simula um disco `40 %` mais pequeno.*

**Corrida NOVA do oráculo:** binário 5.2.1, **10** execuções (1 de validação + 9 gravadas), pen-down
na origem, auto-verificação do centro `ok` em todas. ⚠️ **Validação antes de gravar:** a corrida de
controlo (gancho, caminho `0 → 0,3 → 0,6`) devolveu `máx = 0,343869`, idêntico a seis casas ao da
fixture da sessão anterior — sem isso, um número novo e um número velho não são comparáveis.
**Ficheiros:** nove `fixtures/cloth/*_origem_1passo*.deformado.txt.gz`; `indice.json` regenerado por
`gera_indice.py` (**65** entradas para 65 ficheiros); `verifica_traco.py` verde sobre os 65; README
das fixtures actualizado (a tabela das corridas, o total e a subsecção nova das nove — com o aviso de
que duas delas quebram o parágrafo «o traço»: a de curva `constant` e a de percurso `0,05`).
**Espec:** cabeçalho · §2.2 (errata) · §3.2 · §4.2 · §4.5 · §5.2 · **§5.2-quater (nova)** · §10 ·
**§10.8 (nova)** · §11 · §14 gates **25-31**.
**Instrumento:** o arnês do oráculo de 05/09 com um ficheiro de corridas novo, um montador próprio e
os scripts de análise — todos **fora da árvore**, em `~/Referencias/`.
**Sweep:** verde sobre a espec, o README, a pasta inteira das fixtures, o INBOX, este ledger (menos
os dois hits pré-existentes de 2026-09-05 já registados acima) e o texto do report ao I.

### Q15 — a ORDEM DO ANEL e a PARTIÇÃO EM CÉLULAS (perguntas do I de 2026-09-06; resposta no mesmo dia, com corrida NOVA do harness)

**A pergunta do I:** a §3.1 dizia que a ordem do anel é «a ordem das faces à volta do vértice», e
essa lei é **inaplicável** do lado limpo na esfera — as fixtures trazem só posições de repouso, o
arnês reconstrói a malha **por posição**, logo tem os índices de vértice do alvo e a lista de faces
do gerador dele. Q15.1: dá para acrescentar a lista de faces? Q15.2: e a ordem em que a busca da
árvore espacial devolve as células — é determinística? Q15.3: no plano a ordem das células importa?

**Q15.1 — SIM, e a lei do anel fica fechada por inteiro (F).** As faces incidentes num vértice vêm
por **ordem crescente de índice de face**: a tabela vértice→faces é preenchida em paralelo e depois
**ordenada** num passo explícito, que o próprio código declara ser o que lhe tira a corrida. De cada
face saem dois cantos, na ordem **anterior → seguinte** segundo o sentido de percurso da face, e a
deduplicação guarda a **primeira** ocorrência. ⇒ a lista de faces basta para reconstruir o anel, e
ela é agora fixture (`plano.faces.txt.gz` · `esfera.faces.txt.gz`).

**Q15.2 — a ordem das CÉLULAS é determinística e não depende do cursor (F).** A busca **ordena** os
índices das células antes de os entregar; a varredura *Global* percorre o vector por índice
crescente. ⇒ o cursor decide QUAIS células entram, nunca a ordem relativa delas. ⛔ **Mas a pergunta
por trás dela é outra, e essa não estava respondida:** dentro de uma célula a construção percorre os
**vértices PRÓPRIOS** dela (⛔ não todos os que as faces dela usam), por índice crescente — e cada
vértice é próprio de **exactamente uma** célula. ⇒ a sequência global é uma **permutação** de
`0..N−1` agrupada por célula, ⛔ **não** a ordem crescente. Ela é agora fixture
(`plano.celulas.txt.gz` · `esfera.celulas.txt.gz`).

**Q15.3 — a presunção do I está REFUTADA.** O plano das fixtures tem `4 096` faces contra um tecto
de `2 500` por folha ⇒ **duas** células, e a ordem de visita é a identidade **rodada**
(`[2080..4224]` e depois `[0..2079]`), com o descenso exactamente no pen-down das fixtures `_origem`.
A esfera tem **quatro** células entrelaçadas (nenhuma é um intervalo de índices) ⇒ três descensos.

**⛔ E a segunda presunção — «no plano a lei do anel degenera» — também (M).** Ela degenera no
INTERIOR (`3 969` vértices de 4 vizinhos saem todos crescentes) e **não no bordo**: `128` dos `4 225`
divergem. O corpus não os vê porque estão **todos fora da banda** com `limite = 2,5` (o mais próximo
está a `1,5` do pen-down, a banda acaba a `1,2250`) ⇒ `φ = 0`. ⭐ **Uma fixture do plano
discrimina:** `plano_agarrar_radial_local_preset` corre com `limite = 5,0`, a banda vai a `2,1000` e
alcança `126` dos `128`. Na esfera divergem `5 959` de `6 050` (`98,5 %`).

**⭐⭐ A partição é MEDIDA, não derivada — e a medição foi o trabalho desta emenda.** Ela não é
observável pela API de scripting e o binário instalado não se recompila. O observável é um gesto da
própria aplicação que **reordena a malha** para o consumo desta mesma árvore, pela **mesma** lei de
partição, o **mesmo** tecto de folha, a **mesma** ordenação e o **mesmo** critério de posse: a
permutação que ele devolve lê-se de Python comparando posições antes/depois (nas duas malhas todas as
posições são distintas ⇒ bijecção exacta, sem tolerância). **Derivada == observada, elemento a
elemento, nas duas malhas**, e por célula os blocos batem: plano `2 145`+`2 080`, esfera
`1 569`+`1 520`+`1 504`+`1 457`, somas `4 225` e `6 050`. ⚠️ A malha das fixtures **não** está
reordenada (o gesto é um comando explícito do artista; o harness não o corre) — um port que assuma a
malha reordenada implementa o caso em que a ordem de visita **é** a crescente global, que é o caso
errado.

**ERRATA achada de caminho (M):** a §2.1 dizia «a grelha é ~2 células; a esfera ~3». São
**exactamente 2** e **exactamente 4**; e a esfera tem `6 144` faces, `5 952` quads e `192`
triângulos (os dois anéis dos pólos), não «~6 144 quads».

**⚠️ Armadilha registada:** no plano a célula `1` tem `2 145` vértices próprios **e** a banda de
`3,5 R` contém `2 145` vértices — **conjuntos distintos** (intersecção `1 099`), e é o segundo que é
a coluna `movidos` do índice. *A coincidência é do tamanho, não do conjunto.*

**Corrida NOVA do harness** (⛔ não do oráculo: nenhum traço novo foi gravado): três execuções do
binário 5.2.1 em `--background` sobre malhas geradas pelo mesmo gerador das fixtures — (1) derivação
da partição e da lista de faces; (2) validação contra a permutação observável, `FINISHED` e igualdade
exacta nas duas malhas; (3) conferência das posições contra `plano.repouso`/`esfera.repouso`,
diferença máxima `0,0` às seis casas. Scripts **fora da árvore**, em
`~/Referencias/blender-cloth/oracle/`.
**Ficheiros:** quatro fixtures novas em `fixtures/cloth/` (`{plano,esfera}.{faces,celulas}.txt.gz`);
README das fixtures com a subsecção de proveniência delas. ⚠️ `indice.json` **não** foi regenerado —
o gerador só vê `*.deformado.txt.gz` e nenhum traço mudou (`65`).
**Espec:** cabeçalho · §2.1 (errata) · §3.1 · **§3.1-bis (nova)** · **§10.9 (nova)** ·
§14 gates **32-34**.
**Sweep:** verde sobre a espec, o README, a pasta inteira das fixtures, o INBOX, este ledger (menos
os dois hits pré-existentes de 2026-09-05 já registados acima) e o texto do report ao I.

### Q16 — a RESPOSTA AO ESTICÃO: as três perguntas devolvem NÃO, e o que faltava era um INSTRUMENTO (perguntas do I de 2026-09-06; resposta no mesmo dia, com corrida NOVA do oráculo)

**As perguntas do I.** Com a ordem de visita da Q15 implementada, `50` dos `65` traços batem a barra
e `17` saem praticamente ao bit; o que sobra de determinista é **só** o Push e o Inflate, `5`–`9 %`
ABAIXO, com o passo 2 ao bit e o erro a nascer no passo 3 — o primeiro em que a relaxação tem
trabalho. Q16.1: há algo na projecção de uma restrição de distância que dependa de **quanto o par
está esticado**, além do factor `(1 − ℓ'/D)`? Q16.2: há termo, peso ou restrição que só entre com
componente ao longo da **normal** (dobra, rigidez angular)? Q16.3: há **sub-passos**?

**Q16.1 — NÃO (F).** Releitura integral do laço de relaxação a partir desta pergunta: o factor é
`(1 − ℓ'/D)` e mais nada; **sem tecto**, **sem segundo passe** sobre restrições muito esticadas,
**sem termo de ordem superior** (a correcção é linear na separação, sempre), e o comprimento de
repouso é gravado **uma vez, na criação**, com o único somando a ser o desvio do Expand (§4.5). O
único guarda do laço continua a ser a separação nula.

**Q16.2 — NÃO, e a §3.1 continua verdadeira (F).** Não há rigidez angular nem modelo de dobra
próprio: o papel de dobra é feito pela restrição de **distância ao segundo vizinho pelo anel**, que
é uma restrição de distância como as outras. As **quatro** espécies do §3.2 são a lista inteira, e a
espécie só é lida **dentro** da projecção. Nada no gesto nem no solver testa a direcção do
deslocamento contra a normal.

**Q16.3 — NÃO (F).** Um passo de pincel corre **um** passo de solver; o número de varreduras é uma
constante do ficheiro que não lê o `dt`, nem o tamanho do gesto, nem o estado da malha; o `dt` é
fixo. (Confirma e reforça o censo do §5.2/§5.7 pela pergunta oposta.)

**⇒ A 4.ª pergunta do I era a certa, e a resposta é o §10.10.** Se nenhum termo falta, uma
divergência que **nasce com o esticão** só pode vir da **ORDEM** das projecções (sequencial: a
influência da ordem cresce com o tamanho das correcções, logo é invisível num gesto quase-rígido),
da **POPULAÇÃO** de restrições (quantas, e quantas vezes cada uma é projectada), ou da **fase do
gesto** (que lê quase tudo na posição actual). ⛔ Nenhuma fixture do corpus antigo separava as três:
todas misturam as duas fases em cada passo.

**A corrida NOVA do oráculo: 107 execuções do binário 5.2.1, oito traços novos por passo.**
- ⭐⭐⭐ **`plano_{empurrar,inflar}_radial_local_origem_parado`** — o caminho avança **uma** vez e
  depois repete o mesmo ponto `10` vezes. Um passo sem deslocamento de cursor não aplica força
  (§4.2) ⇒ **dez passos de solver puro** a seguir a um impulso conhecido. **CONTROLO medido:** um
  caminho com **todos** os pontos iguais devolve `0` movidos e `máx |u| = 0,00000` em 12 passos, nos
  dois modos ⇒ a fase do gesto está mesmo calada, e não é conjectura.
- **`…_forca05` · `…_forca025` · `…_massa2`** (Push; `massa2` também no Inflate) — a mesma cena a
  `¼`, `1/16` e `½` do impulso. ⭐ O impulso do passo 2 escala com o **quadrado** da força
  (`0,2500` e `0,0625`, ao `f32`) e o fim do traço escala `0,6677` e `0,3388` ⇒ **a cena é
  não-linear em toda a faixa do corpus**, e comparar só o fim do traço lê um regime, não uma lei.
- **`…_amort1`** (`damping = 1`) — sem memória de velocidade. ⭐ Só ele é **estritamente crescente**
  no `máx |u|` por passo (`0,06990 → 0,19698`); os outros três viram (passo `6` no `_origem`, `4` no
  `_parado`). ⚠️ **No vértice do pen-down os quatro passam por um máximo** ⇒ a régua da monotonia
  tem de ser a da malha (gate 38), senão o discriminador some.
- **`plano_empurrar_radial_global_origem`** — `5` projecções por restrição em vez de `10`. ⭐⭐⭐ O
  passo 2 é **idêntico** (`0,06543`) e os planaltos ficam em `0,23968` contra `0,28222` ⇒ **dobrar
  as projecções custa `15,1 %` do planalto**, que é a régua com que um resíduo se lê (`4 %` ≈ um
  quarto desse degrau).

**⛔ Duas coisas que a corrida revelou e que um port lê ao contrário sem elas escritas:**
1. **`massa2` chama-se assim porque `2` é o TECTO.** A corrida foi pedida com `4` e a porta de
   propriedades do binário **coagiu para `2,0` em silêncio**; é o `2,0` que está no cabeçalho.
   *Um valor pedido não é um valor aplicado.*
2. **O gate 31 deixou de ser «gate de espec».** Ele declarava-se A/B sobre o nosso motor porque
   *«nenhuma fixture do §10 traz um ponto de caminho repetido»* — as duas `_parado` trazem `10`, e o
   gate passa a ter oráculo nas duas metades.

**⛔⛔ ERRATA de CONTAGEM no README das fixtures, achada ao acrescentar as oito (M).** A nota «as
excepções ao parágrafo de omissões são **SETE**» varria só `força · curva · percurso · limite` e
deixava de fora `amortecimento`, `massa`, `plasticidade` e `pino` — que o **mesmo** parágrafo
também fixa. Com a régua completa (as nove grandezas + o percurso pelo **vão em `x`**; a área fica
de fora porque o parágrafo já lhe põe «salvo indicação») são **23 de 73**, e a régua ficou escrita
ao lado da tabela. ⚠️ O percurso mede-se pelo vão em `x`: na esfera a poli-linha entre os pontos
mede `0,6093` e acusaria os oito traços de esfera, que não é o que o parágrafo diz.

**Ficheiros:** 8 fixtures novas × 3 ficheiros (`.deformado.txt.gz` · `.porpasso.txt.gz` ·
`.porpasso.rastreio.txt`) em `fixtures/cloth/`; `indice.json` **regenerado** (`73` entradas para
`73` ficheiros); `gera_indice.py` passa a conhecer a chave nova `passos_com_cursor_parado` como
inteiro; README das fixtures (tabela das corridas **derivada** do índice, secção de proveniência das
oito, régua das excepções, contagens do instrumento por passo `21`/`17`). ⚠️ **Nenhum cabeçalho de
fixture pré-existente foi reescrito** — a chave nova existe só nas oito.
**Espec:** cabeçalho · §5.7-bis (nova) · §10 (contagem) · **§10.10 (nova)** · §11 · §14 gate **31**
reescrito + gates **35-38**.
**Verificador:** `verifica_traco.py` **verde sobre os 73**.
**Sweep:** verde sobre a espec emendada, a pasta inteira das fixtures, o INBOX, os dois READMEs e
este ledger (menos os hits pré-existentes de 2026-09-05 já registados acima).

### Q17 — a FRENTE DE ATAQUE: as três perguntas devolvem NÃO, e o defeito estava na ESPEC, não no port (perguntas do I de 2026-09-07; resposta no mesmo dia, com corrida NOVA do oráculo)

**As perguntas do I.** `53` dos `73` traços batem a barra; o gate 35 (dez passos de relaxação pura
depois de um impulso conhecido) passa em toda a malha, o que pelo critério da própria §14 põe o
resíduo do Push e do Inflate na **fase do gesto**; e o perfil localiza-o na **borda de ataque** (no
núcleo `4`–`7 %`, em `x = 0,844` o alvo desloca `0,0830` contra `0,0204` nossos). Q17.1: há diferença
entre o corte `d ≥ R` do alvo e o nosso? Q17.2: o conjunto de células acresce ou é re-decidido, e a
unidade é a célula ou o vértice? Q17.3: o Inflate tem uma segunda causa? Q17.4 (se as três derem
nada): há algo que **acumule ao longo do traço**?

⭐⭐⭐ **O método foi outro, e é o que fez a diferença: em vez de um censo, um CONTROLO.** Construí,
fora da árvore, um arnês independente que implementa a espec tal como estava, alimentado pelas
fixtures. Ele reproduz `plano_arrastar_radial_local_origem` com **`err_max = 3,7·10⁻⁶`** sobre os
`4 225` vértices e nos 12 passos — e reproduz o produto do I em `plano_empurrar_radial_local_origem`
a **quatro algarismos** (`0,21019` · `0,24185` · `0,12501` · `0,06556` · `0,02048` · `0,00338`
contra os `0,2100` · `0,2417` · `0,1249` · `0,0654` · `0,0204` · `0,0033` que o I reportou).
⇒ *o port está fiel à espec; a espec é que estava errada* — e como o arrasto atravessa exactamente
o mesmo corte, a mesma banda, o mesmo conjunto de células, a mesma ordem de criação e a mesma lista
em duplicado, as três perguntas ficam respondidas **por resultado**, não por leitura.

**Q17.1 — NÃO (F + M).** O corte é `d ≥ cache.radius` sobre a distância do vértice **actual** ao
cursor (esférica, ou no plano da vista com queda projectada), exactamente como a §4.1 já dizia; o
Grab é a única excepção e mede no repouso. Nenhum outro raio, nenhum outro espaço.

**Q17.2 — NEM UMA NEM OUTRA, e a §2.1/§3.1-bis já o diziam (F).** O conjunto de células é
**re-decidido por inteiro a cada passo**, e na área *Local* os dois números que o decidem — o centro
inicial e o raio inicial — são constantes, logo o conjunto é o mesmo em todos os passos. As
**restrições**, essas, **acrescem**: uma célula só é construída enquanto nunca tiver sido activada, e
a activação é o que a fecha. A unidade da decisão do **conjunto** é a **célula**; a do filtro de raio
da construção é o **vértice** (posição de repouso contra o centro fixo); e os vértices visitados são
os **próprios** da célula.

**Q17.3 — SIM, e é a MESMA causa vista de outro lado (M).** O Inflate lê a normal **por vértice** e
nunca fica sem direcção — ele dispara nos onze passos. O que estava errado é **de que superfície** a
normal sai: das que o **traço encontrou**, não das actuais. Ajuste por mínimos quadrados sobre
`plano_inflar_radial_local_origem` e `…_massa2`: resíduo `7·10⁻⁶ … 7,4·10⁻⁴` contra a normal de
repouso, `0,32 … 0,77` contra a actual, com amplitude `1,00000` nos 22 passos.

**Q17.4 — SIM: o que acumula é a COVA, e ela CALA o Push (M).** A normal da área é amostrada num
disco de `R · «Normal Radius» = 0,175` medido contra as posições **de agora**; quando a folha afunda
mais do que isso sob o cursor, **nenhum** vértice qualifica, a normal é o vector nulo e o Push não
escreve força nenhuma (a §4.2-bis (5) já tinha a regra, como caso degenerado). Limiar medido sobre
`67` passos de sete traços: maior `min_v |v−c|` **com** gesto `0,17313`, menor **sem** gesto
`0,17586`, e `R·0,5 = 0,17500` cai no vão — zero passos do lado errado. E ele **volta** a disparar
quando o cursor avança para terreno raso, o que dá quatro padrões diferentes no mesmo caminho
(`2 3 4 5 10` · `2 3 4 5 6 10` · `2..9` · os onze).

**Verificação ponta-a-ponta:** com as duas correcções, os **dez** traços de empurrar/inflar por passo
reproduzem-se com `err_max ≤ 5·10⁻⁶` sobre a malha inteira e nos 12 passos.

**⛔ Uma afirmação da espec REVOGADA E INVERTIDA:** o gate 23 dizia *«a normal do Push é REAVALIADA
sobre a malha DEFORMADA»* e mandava reprovar quem a congelasse. A metade dele que estava certa era o
**controlo** (no 1.º passo simulado as duas leis coincidem por construção) — e é por isso que ele
nunca distinguiu nada. Substituído pelos gates 39-42.

**⛔ Uma leitura minha (Q16) que a medição derrubou:** *«o resíduo está na resposta ao ESTICÃO»*. O
défice crescia com o esticão porque o esticão é o que afunda a cova, e a cova é o que cala o gesto.
*Duas grandezas que crescem juntas, e a que se mediu não era a causa.*

**Corrida nova do oráculo (E, fora da árvore):** 6 execuções — 3 que viraram fixture
(`esfera_empurrar_radial_local_1passo` · `esfera_inflar_radial_local_1passo` ·
`plano_inflar_radial_local_1passo_2tracos`) e 3 de sonda (dois prefixos de esfera e um par de
preseed que foi refeito na convenção do corpus).
⭐ **A de esfera refuta uma nota do README das fixtures**: dizia que um traço scriptado não fixa o
centro da área *Local* numa esfera. Fixa, se o hover for semeado — mas nessas duas fixtures a
**banda** não é observável (`120` vértices movidos, todos onde `w = 1` sob qualquer dos dois centros
possíveis), e isso está escrito ao lado delas.

**Espec:** cabeçalho · §4.2 (linha do Inflate) · §4.2-bis (2) ERRATA + (5) + **(8) novo** ·
**§4.2-ter (nova)** · §5.7-bis (fecho) · §10 (contagem `73 → 76`) · **§10.11 (nova)** · §14 gate
**23 revogado e invertido** + gates **39-42**.
**Fixtures:** `+3` (`76`); `indice.json` regenerado (76/76); `gera_indice.py` passa a ler `tracos`
como inteiro; README das fixtures com a régua de excepções alargada (`24` de `76`; a coluna nova é o
**número de traços**) e a secção das três novas.
**Verificador:** `verifica_traco.py` **verde sobre os 76**.
**Sweep:** verde sobre a espec emendada, a pasta inteira das fixtures, o INBOX, os dois READMEs e
este ledger (menos os hits pré-existentes de 2026-09-05 já registados acima).
**Auditoria §4.2 (R-pré): ✅ CORRIDA E ATESTADA em 2026-09-07** — ver a secção abaixo.

### R-pré da emenda Q17 (2026-09-07) — ATESTADO

**Quem:** subagente R-pré despachado com contexto novo, independente do subagente-E que escreveu a
emenda. Leu os **dois** lados — o fonte por shell (`cat`/`grep`/`sed`), nunca pela ferramenta de
leitura, que o deny da linha nega. Não escreveu nem ditou código de produto.

**§4.2 (expressão): ZERO achados.** Sem trecho, sem nome interno do alvo, sem wording de comentário
ou de manual, sem pseudo-código espelhado, sem organização ficheiro-a-ficheiro. Conferências feitas
de propósito: (a) **nenhum comentário do fonte fala do instante das normais** — a §4.2-ter é
medição, não tradução de um comentário; (b) o bloco de fórmula da §10.11 é o modelo do **nosso**
arnês de ajuste, não uma transcrição; (c) a §5.7-bis FECHO mantém a ordem das **perguntas do INBOX**,
que é onde um censo de ausências escorrega para a organização do alvo; (d) considerada e **liberada**
uma proximidade de ideia (a espec chama à sua fotografia de normais «a mais barata», e o alvo tem um
comentário sobre custo noutro contexto): a espec não atribui razão nenhuma ao alvo, e a frase é
sobre a NOSSA escolha.

**Sweep:** verde sobre a espec emendada + a pasta inteira das fixtures + INBOX + os dois READMEs +
`docs/3D/cloth/`; e sobre o **histórico** desses caminhos os únicos hits são os **pré-existentes de
2026-09-05** já adjudicados acima. ⭐ Controlo extra: o **patch do commit da emenda**, varrido
sozinho, passa limpo.

**FIDELIDADE — as duas correcções são verdadeiras, e o mecanismo de cada uma foi isolado:**
1. **As normais que o gesto lê são as da superfície que o traço encontrou.** O mecanismo é que quem
   as refresca é o **passo de preparação do objecto para edição**, e um traço corre-o **uma vez**; a
   escrita de posições da escultura marca as normais da **árvore de desenho** e ⛔ **não** as da
   malha. ⇒ a normal da área do Push e a normal por vértice do Inflate saem da **mesma** fotografia.
2. **O disco de amostragem mede contra as posições de agora.** Existe um ramo que leria as posições
   e as normais **de partida**, e ele está atrás de uma condição que **este pincel não activa** —
   por isso as duas grandezas vêm de superfícies diferentes, que é o ponto da errata. Quando o disco
   fica vazio a resposta é o **vector nulo** e o deslocamento do Push é exactamente zero, sem
   direcção de reserva.

**Números reconstruídos do zero pelo R-pré, a partir das fixtures (script próprio, fora do repo):**
- **O limiar, célula a célula.** Sete traços de empurrar por passo ⇒ **`67`** passos com o cursor em
  movimento. Medindo `min_v |p_v(bloco k−1) − c_k|` sobre **todos** os vértices: **`53` abaixo** de
  `0,175` com máximo **`0,17313`**, **`14` acima** com mínimo **`0,17586`**; `R · 0,5 = 0,17500` cai
  no vão, **nem um passo do lado errado**. Os quatro padrões de disparo saem os quatro:
  `2 3 4 5 10` (*Local*) · `2 3 4 5 6 10` (*Global*) · `2..9` (massa `2`) · **os onze** (forças `0,5`
  e `0,25`, onde a folha nunca chega a `0,175`).
- **A esfera.** `esfera_empurrar_radial_local_1passo`: deslocamento colinear (fracção não-colinear
  `1,5·10⁻⁵`), direcção `(+0,2996, −0,9541, 0)` ⇒ **`0,03°`** da normal da superfície no cursor e
  **`17,43°`** do eixo da vista (⭐ o `17,43` da espec reproduz-se exactamente com a direcção do
  ajuste; um estimador ingénuo dá `17,44`, e a barra do gate 42 (`≈ 17,4°`) é robusta aos dois).
  Controlo `esfera_inflar_radial_local_1passo`: direcção por vértice = a normal de repouso do próprio
  vértice, mediana `7,0·10⁻⁵`.
- **Os dois traços.** No 2.º traço a direcção por vértice bate as normais da malha deixada pelo 1.º
  com mediana **`1,7·10⁻⁵`** e as planas com **`0,299`**; inclinação máxima das normais **`22,38°`**
  (a espec diz `22,4°` ✓) e cova do 1.º traço **`0,09917`**.
- **Saúde do corpus.** `verifica_traco.py` **verde sobre os 76**; `gera_indice.py` regenera o
  `indice.json` **byte-a-byte** (76 entradas para 76 ficheiros); a tabela «As corridas» tem **76**
  linhas e o conjunto de nomes é **igual ao disco**; o censo das excepções do README dá **`24`
  distintas de `76`** (as linhas somam `25` porque `plano_agarrar_radial_local_preset` é excepção em
  duas grandezas) e **`16`** fora da área *Local*.

**NOVE curas aplicadas no acto, todas funcionais (nenhum facto perdido):**
1. **§4.6 linha 2** ainda dizia que a normal da área é a média das normais **actuais** e que «roda
   com a vala»; passa a dizer que o que roda é o **conjunto amostrado** e que a grandeza **deixa de
   existir** quando o disco fica vazio.
2. **§4.6 linha 4** dizia «sobre a malha ACTUAL» e a leitura do censo dizia «da malha deformada» —
   as duas corrigidas, com a nota de que a fixture de esfera de **um** passo não separa os dois
   instantes (ali coincidem por construção).
3. **§4.2-ter** afirmava que *nada mais no pincel lê normais*: há um **terceiro leitor**, o recorte
   *Front Faces Only* do §4.1, que obedece à mesma lei e está **desligado** nos presets (logo o
   corpus não o observa). A colisão lê normais **do colisor**, que é outra geometria.
4. ⭐⭐ **O §7 estava certo e virou divergência não-nomeada: o FILTRO faz o CONTRÁRIO.** Cada passo
   do filtro repete a preparação do objecto para edição, logo ali as normais **são** as de agora.
   *A mesma palavra («Inflate») nomeia duas leis neste documento* — agora as duas dizem-no, e a §4.6
   linha 4 nomeia os dois consumidores.
5. **Gate 23:** «REVOGADO E **INVERTIDO**» é a palavra errada e é perigosa. O veredito não se
   inverte — o que se reavalia a cada passo é o **conjunto amostrado**, logo um port que **congele**
   o vector do gesto e empurre em todos os passos continua errado (é o gate 40 que o apanha). A razão
   real de o gate sair é ser **ambíguo**: num plano a mutação dele é byte-idêntica à lei correcta em
   todo passo de disco não-vazio.
6. **A régua do resíduo** não estava definida em lado nenhum e os gates 39/41/42 penduram-se nela —
   fica escrita (`‖r − a⊗u‖ / ‖r‖`, com `1,00` = nenhum vector explica nada), com o ⛔ de que a
   barra é o **vão de ordens de grandeza** e não o dígito (ela mede a direcção **e** o `a_v`), e com
   uma régua irmã que não depende da queda, calibrada com números que o R-pré mediu.
7. **Gate 40:** a régua não dizia sobre que vértices corria o mínimo, contra que posições, nem com
   que índice de cursor — fica escrita como foi reconstruída.
8. **A cova do 1.º traço** era `0,0999` na espec e no README; o ficheiro diz **`0,09917`**.
9. **Dois totais do README** ainda em `73` com `76` no disco (o do bloco das fixtures de topologia e
   o da deriva do `analise.json`).

**⭐⭐⭐ E um ACHADO que não é cura: a fixture nova RESPONDE em parte a uma pergunta que a §4.6
declarava indecidível.** O **peso** da soma por face (área · ângulo · uniforme) era «não decidível
pelo corpus» porque no plano os três dão `+ẑ` e na esfera UV a simetria os iguala. O 2.º traço da
fixture de dois traços corre sobre uma superfície **já não plana**, e ali os três separam-se:
mediana do desvio **uniforme `1,7·10⁻⁵` · ângulo `3,0·10⁻⁴` · área `6,6·10⁻⁴`**. É **indicação**, não
prova (os três passam qualquer barra, a cova é rasa e a fixture não foi desenhada para isto), e está
escrito como tal. ⚠️ *Quem acrescenta uma fixture tem de reconferir as notas que diziam «o corpus não
decide isto»* (CLAUDE.md §0.0).

**⚠️ E o instrumento que confere os atestados nasceu errado DUAS vezes.** A 1.ª (no INBOX) era
case-sensitive e lia `4` para `8`. A 2.ª — a redacção que a curava — **contava-se a si própria**: a
linha que ensinava a contar casava com o próprio padrão, e o awk devolvia um falso verde. ⇒ o censo
honesto não é uma contagem, é **por bloco** (nenhuma linha `EMENDA Q<n>` sem atestado antes da
seguinte), está escrito no cabeçalho da espec, e **tem controlo**: apagar a linha do atestado da Q17
faz o instrumento acusá-la.

**Veredicto: ATESTADO.** A emenda Q17 pode ser lida e implementada pela janela-mãe.

### Q18 — o par de `φ`, o PESO da normal, o traço LONGO e os dumps de ESFERA (perguntas do I de 2026-09-07; resposta no mesmo dia, com corridas NOVAS do oráculo)

**As quatro perguntas e o veredito, um a um.**

**Q18.1 — o par de `φ`. CONFIRMADO no fonte, lado a lado.** Os **três** sítios em que a banda podia
entrar num passo: o factor das cinco varreduras de relaxação **traz** a banda; o termo de aceleração
da integração **não**; o termo de velocidade **traz**, e é a única vez que ela entra ali. Confirmada
também a **ordem**, que é o que faz a diferença: o factor chega à integração sem banda, o termo de
aceleração consome-o assim, depois ele é escalado pelo amortecimento, **depois** disso pela banda, e
só então escala a velocidade. E nos dois sítios a banda é avaliada na posição de **repouso**, e só
quando há traço activo. ⇒ espec §5.2 (o factor passa a chamar-se `φ_relax`), §5.4 (`φ_int`) e
**§5.4-bis nova**, que põe o par numa tabela só. ⭐ A medição do I (`3,9·10⁻³ → < 5·10⁻⁶` no
controlo) é a leitura certa da lei, e o censo da invisibilidade dela confere: *Global* dá `1² = 1`,
o suporte da força e o da banda **não se tocam**, e o anel onde ela morde tem deslocamento de ordem
`10⁻³`.

**Q18.2 — o peso da soma por face. CONFIRMADO uniforme, e o mecanismo é maior que a pergunta.**
⭐⭐⭐ O programa tem **DUAS** leis de normal por vértice. A que corre no caminho da escultura — o
refrescamento incremental, sempre que a estrutura de normais já está preenchida e só alguns pedaços
mudaram — é a soma **sem peso** de normais de face **unitárias**, normalizada no fim, com um eixo
fixo do objecto como resposta quando a soma tem comprimento zero: isto está **confirmado no fonte**.
A outra, a reconstrução total, corre na primeira avaliação de uma malha que ainda não foi
esculpida, e é **ponderada pelo ângulo do canto**: ⛔ o código dela **não vive na parte do fonte que
esta linha tem**, e por isso ela está estabelecida por **medição** (o mesmo estatuto do empate de
eixos da Q15).
⚠️ **A medição do I não estava errada — estava a medir a malha certa pela via indirecta.** Medidas
as três candidatas contra o vector de normais que o próprio programa guarda **para aquela mesma
malha**, elas concordam a `0,000°` de mediana e `0,31°` de máximo ⇒ *a fixture de dois traços não
separa nada; o que a fez responder foi a lei estar certa.* A régua que decide é perguntar as
normais ao programa sobre malhas em que os três pesos discordam por **graus** (11 corridas, §10.14):
esculpida e refrescada dá **uniforme** a `mediana 0,000°` e `máx ≤ 0,027°` contra `0,022`–`0,073` de
mediana das outras duas; malha nunca esculpida com vértices deslocados ao acaso dá **ângulo** a
`máx ≤ 0,027°` contra `17°`–`116°` das outras.
⛔ **Consequência que se lê ao contrário com facilidade:** as fixtures de **esfera** são primeiros
traços sobre malha nunca esculpida ⇒ elas trazem a lei do **ângulo**; o 2.º traço da fixture de dois
traços traz a lei **sem peso**. *O corpus contém as duas, e não é o pincel que escolhe.* O preço de
usar a lei sem peso em toda a parte está medido e é `0,021°` de mediana na esfera em repouso
(`1,3·10⁻⁴` sobre um deslocamento de `0,25`), duas ordens abaixo da barra. ⇒ espec **§4.2-quater
nova**, §4.6 linha 4 **fechada**, §11 (duas linhas novas), gate **44**.

**Q18.3 — o traço LONGO. GRAVADO, e a premissa REFUTADA.** Duas fixtures novas de `36` passos sobre
o mesmo caminho de `0,6` das `_origem`, com `.deformado` **e** por passo. ⛔ **Em área *Local* um
traço mais longo não é mais fundo**: `0,94 R` (12 passos) · `0,71` (24) · `0,76` (36) · `0,75` (48),
e mudar o comprimento do caminho não o move (`0,66`–`0,76 R`). Mecanismo: a área *Local* é uma bola
fixa de `3,5 R` no pen-down e a banda leva o factor a zero na borda ⇒ a folha satura. ⭐⭐ **O regime
de `4,8 R` que o gate de artefacto do produto corre existe e é um facto da ÁREA**: o mesmo traço em
*Global* chega a **`5,41 R`** e não assenta. ⇒ isso é também um **diagnóstico para o lado limpo**:
um gate que corra um traço *Local* de ~35 eventos e chegue a `4,8 R` está a medir uma cena que o
alvo não produz. ⇒ espec **§10.12 nova**, §11, gate **45**.

**Q18.4 — os três dumps de ESFERA por passo. GRAVADOS, com uma advertência nova que muda como se lê
o corpus.** ⛔⛔⛔ **Na esfera, duas corridas da mesma configuração não dão a mesma saída.** Quatro
realizações de cada uma: plano `0,000000` (12 **e** 36 passos, nas duas áreas); esfera `0,027`
(agarrar) · `0,037` (gancho) · `0,028` (expandir) — `11 %`, `22 %` e `60 %` do sinal. A divergência
nasce no **1.º passo simulado** (`4,3·10⁻⁴`), amplifica `~65×`, e é **difusa** (`481` de `631`
vértices movidos já diferem ali). ⛔ Quatro explicações medidas e **refutadas**: concorrência (uma só
linha de execução dá `0,036`), semeadura do sobrevoo (`0,041`), pré-traço (`0,038`), desenho forçado
da vista (`0,033`). ⇒ os três ficheiros levam a **banda de realização por passo** no `.rastreio` e
três colunas de honestidade no cabeçalho; e a prova de que o instrumento continua válido é que
`prova_do_fatiamento ≈ banda da corrida inteira ≈ desvio ao `.deformado` do repo` nos três
(`0,0177`/`0,0200`/`0,0189` · `0,0365`/`0,0362`/`0,0340` · `0,0219`/`0,0218`/`0,0212`) — *se o
prefixo não fosse o passo `k`, a 1.ª coluna seria maior que a 2.ª*. ⚠️ **A lotaria NÃO explica os
erros abertos** (`0,182`/`0,255`/`0,581` são `5×` a `26×` a banda), mas põe um **chão** para
qualquer barra de esfera. ⇒ espec **§10.13 nova**, §10.11 (a nota do que faltava gravar), §11,
gates **46** e **47**.

**A corrida NOVA do oráculo: 304 execuções do binário 5.2.1**, mais 11 de leitura de normais —
113 que viraram fixture (74 dos dois traços longos por passo + 39 da 1.ª tentativa de esfera),
162 do censo de realização e da banda por passo, 29 de sonda (as varreduras de contagem de passos e
de modo, os controlos de uma linha de execução / semeadura / pré-traço / desenho forçado). Malhas
**nossas**, geradas pelo mesmo harness e pela mesma lei das outras fixtures; ⛔ nenhum activo do alvo.

**Entregue:** espec §4.2-quater (nova) · §4.6 linha 4 (fechada) · §5.2 · §5.4 · §5.4-bis (nova) ·
§10 (contagem `76 → 78`) · §10.11 (nota) · §10.12 · §10.13 · §10.14 (novas) · §11 (quatro linhas) ·
§14 gates **43-47**.
**Fixtures:** `+2` `.deformado` (`78`) e `+5` por passo (`26`); `indice.json` regenerado (78/78);
README das fixtures com a secção nova das cinco, a errata da célula do peso, e as contagens
actualizadas (`24` de `78`, `17` não-*Local*, `26`/`22` por passo).
**Verificador:** `verifica_traco.py` **verde sobre os 78** (`78 OK`, `0 BAD`, exit `0`).
**Sweep:** verde sobre a espec emendada, a pasta inteira das fixtures, o INBOX, os dois READMEs e
este ledger (menos os hits pré-existentes de 2026-09-05 já registados acima).
**Auditoria §4.2 (R-pré): ✅ CORRIDA em 2026-09-07 — ATESTADO no cabeçalho da espec.**
Subagente R-pré de contexto NOVO, independente do subagente-E que escreveu a emenda; leu os dois
lados (o fonte por shell, que é o que o deny da linha permite).

**Veredicto de EXPRESSÃO (§4.2): ZERO achados.** Sem trecho, sem nome interno do alvo, sem wording
de comentário ou de manual, sem tabela verbatim, sem organização ficheiro-a-ficheiro. Os termos
novos (`φ_relax`/`φ_int`, *banda de realização*, *prova do fatiamento*, *lei incremental*,
*reconstrução total*) são vocabulário do domínio, e `Local`/`Global`/`Dynamic`/`Plane`/`Inflate`/
`damping` são nomes que o artista vê (§4.1.13 — a propriedade de amortecimento é rotulada na porta
de propriedades, conferido).

**Sweep (§7.1):** verde sobre a espec emendada + a pasta INTEIRA das fixtures + o INBOX + os
READMEs + `docs/3D/cloth/`, e sobre o **histórico** de todos esses caminhos e o texto do report
final. Os únicos ✗ são pré-existentes e já adjudicados neste ledger: (a) as duas linhas de cobertura
daqui (2026-09-05), e (b) no `--git-history` de `docs/3D/cloth/`, os commits de 2026-09-05 em que o
próprio R-pré **curou** o doc `01` — o patch de uma cura contém necessariamente os dois lados, e o
texto vivo passa limpo. Nenhum vem da Q18 (conferido: o patch de `098e45be4`, sozinho, passa limpo).

**FIDELIDADE — os quatro factos conferidos no fonte, os números reconstruídos do zero.**
`Q18.1` **confere lado a lado**: o factor das cinco varreduras é pré-calculado uma vez por passo
para a malha inteira e traz a banda avaliada na posição de repouso; o da integração não a traz, e
ela entra uma só vez, no termo de velocidade; nos dois só há banda quando existe traço activo (o que
fecha a nota do filtro). Os suportes disjuntos conferem (`2,875 R` contra o corte da força em `R`).
`Q18.2` **confere**: a lei incremental (soma sem peso de normais de face, normalização no fim, eixo
fixo `+z` na soma nula) está no fonte; a da reconstrução total **não vive** na parte do fonte que
esta linha tem, tal como a emenda declara.
`Q18.3` e `Q18.4` reproduzem-se célula a célula das fixtures — `0.267205` (`0,76 R`) · `1.893192`
(`5,41 R`) · o pen-down `0,28635 → 0,14411` **monotonamente** em *Local* e `0,0993 → 1,0550` sem uma
descida em *Global* · `prova = 0.000000` nos dois traços longos (ali o bloco `k = N` **é** o
`.deformado`, ao bit) · as **seis** linhas da tabela por passo da esfera · `24` de `78` excepções e
`17` não-*Local* no README · `verifica_traco.py` **78 OK / 0 BAD / exit 0** · `indice.json`
regenerado **byte-a-byte** (78/78).

**NOVE curas aplicadas no acto, todas funcionais** (detalhe no cabeçalho da espec): três frases que
contradiziam números da própria emenda (as «três ordens» do censo da §5.4-bis, o «não depende do
comprimento do caminho» da §10.12, a faixa `10⁻⁴`–`10⁻³` do gate 46); a narração da ORDEM de
escritas de um acumulador na §5.4-bis, trocada pelos dois PRODUTOS que a medição de facto separa
(§4.3); a proveniência da normal de face **unitária**, que estava sob «confirmada no fonte» e é
`(M)`; o eixo do caso de soma nula, nomeado; a desambiguação global do `φ` sem índice; os operandos
das três colunas de honestidade dos ficheiros de esfera, com o quarto sorteio medido
(`0,007919`/`0,035424`/`0,014350`); e — a de maior alcance — a atribuição **não isolada** de
«a esfera não é reproduzível».

⛔⛔ **O achado que o R-pré devolve como pedido de trabalho ao E:** todas as corridas de esfera da
§10.13 são de área *Dynamic* e todos os controlos de plano são de *Local*/*Global* ⇒ **superfície e
área variaram juntas**, e a §10.13 concluía «é da esfera e não do plano» enquanto o §11 já o
escrevia como lei. O que está demonstrado é o par «esfera + *Dynamic*» contra «plano +
*Local*/*Global*». ⭐ **O controlo que separa os dois já existe como configuração no corpus**
(`plano_arrastar_radial_dinamica`, e o `_preset`): quatro realizações dele, medidas como as da
§10.13, decidem numa corrida. ⏳ **Pedido, e não bloqueante** — o veredito operacional da secção (há
lei em falta, a caça continua, a lotaria é só um chão) não depende dele.
⭐⭐ **E o R-pré acrescentou duas confirmações que o repo já continha**, reconstruídas das fixtures
sem correr o oráculo, e que valem mais que duas das quatro hipóteses refutadas por não dependerem de
a experiência ter «pegado»: o bloco de repouso é **bit-idêntico nos três ficheiros de esfera** (não é
o arnês a devolver malha diferente), e o **plano** exercita vértices partilhados entre células no
ponto de **maior contenção possível** — a divisória cai na fileira do pen-down, `53` dos `65`
partilhados movem-se e um deles carrega o máximo da malha — e ainda assim dá `0,000000`.

**Veredicto: ATESTADO.** A janela-mãe pode ler a emenda Q18.

### R-pré da emenda Q19 (que leva dentro as Q20 e Q21) — 2026-09-07 — ATESTADO

**Quem:** subagente R-pré despachado com contexto novo, independente do subagente-E que escreveu a
emenda. Leu os **dois** lados — o fonte por shell (`sed`/`rg`/`grep`), nunca pela ferramenta de
leitura, que o deny da linha nega. Não escreveu nem ditou código de produto.

**§4.2 (expressão): ZERO achados de trecho, tabela verbatim, comentário, wording de manual,
pseudo-código espelhado ou organização ficheiro-a-ficheiro. UMA higiene curada no acto:** a §4.3
citava, entre aspas, a **tradução de um identificador interno** do alvo para nomear a declaração de
capacidade que gateia a linha do painel — é a espécie *«mesmos nomes traduzidos»* do §7.2-3. Passa a
descrever o comportamento (*a declaração de capacidade de que o painel depende para desenhar este
controlo*). ⭐ Liberados de propósito: `dispersao_entre_realizacoes` e *banda de realização*
(vocabulário nosso), `peso_normal`/`persistente` (rótulos que o artista vê, §4.1.13), e as cinco
cláusulas da colisão + as quatro leituras da base, que descrevem **ordem observável** e não a forma
do código.

**Sweep:** verde sobre a espec emendada + a pasta inteira das fixtures + INBOX + os dois READMEs +
`docs/3D/cloth/` + `docs/3D/README.md`, e sobre o **histórico** destes caminhos. Os únicos ✗ são os
**pré-existentes já adjudicados**: as duas linhas de cobertura deste ledger (2026-09-05) e, no
histórico de `docs/3D/cloth/`, os commits do dia em que o R-pré curou o doc `01` — o texto vivo passa
limpo. ⭐ Controlo extra: `--git-history` restrito à espec + fixtures + INBOX passa **limpo**.

**Fidelidade — os quatro factos, conferidos no fonte:**

- ⭐⭐⭐ **(Q21.1) A AFIRMAÇÃO NEGATIVA é verdadeira, e as três provas são independentes de facto.**
  (a) A porta que devolve o delta já inclinado tem **três** chamadores e o pincel de tecido não é
  nenhum deles — ele consome o delta cru, num sítio só; (b) a declaração de capacidade que gateia a
  linha do painel nomeia **três** pincéis, o de tecido fora; (c) o painel desenha a linha **só** sob
  essa condição. ⭐⭐ **E o R-pré alargou a pergunta ao ESCOPO DECLARADO da espec** — três fixtures
  byte-idênticas provam que o botão não move *aquela cena*, nunca que o controlo não existe, e o
  cabeçalho desta espec cobre também os pincéis alheios que miram a simulação: o selector de alvo de
  deformação é exposto **exactamente** aos dois que o cabeçalho nomeia, e nenhum deles está na
  capacidade ⇒ *a ausência vale para tudo o que este documento descreve.* ⚠️ **E o que saiu da espec
  era só sobre ele:** a cláusula do achatamento *Projected* ficou intacta na §4.3, saiu **uma** linha
  da tabela do §8.1 e **um** termo da ordem do painel do §8.4.
- ⭐⭐ **(Q20) O mecanismo é mesmo DUPLO e as duas metades são independentes.** Os modos de âncora — e
  os pincéis alheios que miram a simulação — deixam de reler a localização do evento a partir do 1.º
  passo da passagem de simetria; e, entre os pincéis cujo delta acumula desde o ponto de partida, a
  localização é **reescrita com o pen-down todo passo**. O Agarrar de tecido está nas duas listas. A
  segunda metade é outra coisa: o tamanho por pressão é a **negação** de «é ferramenta de agarrar», e
  ali o Agarrar de tecido está e ⭐ **o Gancho de tecido NÃO** — a parentese da emenda sobre o raio do
  gancho variar com a pressão confere —, e a mesma classe deixa de reler a pressão por passo. A área
  *Dynamic* lê exactamente a localização e o raio que essas duas metades congelam. ⭐ **E o «diferem
  em três coisas só» confere:** o raio da criação de restrições é `R₀(1+L)` nas **duas** áreas, e o
  que sobra é o centro, o filtro de raio e a lista duplicada.
- ⭐ **(Q21.2)** A base substitui as posições de repouso em **exactamente quatro** leituras, todas na
  construção, e **as exclusões também estão certas**: os alvos das três espécies de alvo próprio e a
  banda por passo lêem o repouso do traço.
- ⭐ **(Q21.3)** As **cinco** cláusulas conferem, a quinta incluída (cada colisor lê a posição já
  corrigida pelo anterior mas **a mesma** origem de raio), e a origem do raio nasce nas posições de
  repouso do traço.

**Números reconstruídos do zero a partir das fixtures** (script próprio, fora do repo): as três de
*peso normal* dão o **mesmo bloco de vértices** e diferem só na linha do botão ·
`plano_agarrar_radial_dinamica` recalcula `max = 0,162721` e o maior `x` movido é **`+0,890625`**
(`1,190625 < 1,225`; a coluna seguinte fica a `1,2375`), e ⭐ `1,225` é o corte da **banda** no fonte
(`R(1+L)`), não o do filtro de restrições, que a *Dynamic* nem aplica · **treze** ficheiros trazem a
chave nova e os treze valores batem a tabela do §10.15 · toda a coluna derivada do §10.15 recomputa ·
o censo do README dá **`30` de `86`**, **`33`** linhas, `18` não-*Local* e os **três** de dupla
excepção · `gera_indice.py` regenera o `indice.json` **byte-a-byte** (86 para 86) · o
`verifica_traco.py` fica **verde sobre as 86** · a esfera tem `6 050` vértices ⇒ **a regra do
auto-teste (`< 50` ou mais de metade) recusa exactamente os dois modos de falha que ele apanhou**
(`6 050` movidos e `0`).

⛔ **O que NÃO é reconstruível deste lado, e fica dito:** as bandas de `p95`, de mediana e dos
vértices nomeados exigem **quatro** realizações e a fixture carrega **uma**. O R-pré verificou a
**coerência interna** delas — todas as razões publicadas recomputam a partir das colunas ao lado —,
não os valores.

**Nove curas aplicadas no acto, todas funcionais. As cinco primeiras são a mesma espécie, e é a
espécie desta emenda: ela RE-MEDIU a banda, escreveu o resultado no cabeçalho, e deixou os
quocientes divididos pelo sorteio antigo.**

1. ⛔⛔⛔ **A tabela do §10.13 chamava «banda (cabeçalho)» à banda que o cabeçalho já não tem** — e a
   correcção derruba a conclusão da emenda num dos três traços: com a banda publicada por ela
   própria, os quocientes são `1,67×` · **`0,99×`** · `1,24×` ⇒ **no gancho o erro aberto está dentro
   da lotaria e ali não há prova de lei em falta.** A 1.ª redacção mandava o lado limpo caçar uma lei
   onde só há ruído.
2. A 2.ª tabela do §10.13 muda com ela: o agarrar decide com `1,20×` e não `1,54×`; os dois
   indecidíveis passam a `1,97×` e `3,62×` abaixo.
3. O **gate 46** repetia os três quocientes antigos e dizia a banda medida para **doze** traços — são
   **treze**.
4. O **gate 47** listava `0,0200` · `0,0362` · `0,0218` como a banda dos três; com a de hoje o gancho
   deixa de ser o caso «igual à banda».
5. A §2.1 dava a banda do agarrar como `0,0200` (é `0,025712`; o resultado do lado limpo fica **mais**
   dentro dela, `0,30×`). ⚠️ **E a §10.13 tem uma SEXTA leitura da mesma grandeza:** a célula de
   esfera + *Dynamic* do desenho `superfície × área` publica `0,58264` / `0,059974` onde a fixture
   traz `0,582806` / `0,041931` — outro conjunto de quatro corridas, e sem esta nota quem compare as
   duas tabelas conclui que uma está errada.
6. ⛔ **A recusa da barra «`k ×` a banda» declarava-se «por MEDIÇÃO» em três sítios e não traz número
   nenhum** — o argumento é o §0.0, que é um princípio. Passa a recusa **por princípio**, com a
   medição que de facto existe ao lado (*o `p95` já tem chão abaixo da barra, logo ela não faz
   falta*). ⚠️ *Uma recusa de princípio arquivada como recusa medida polui a única lista que impede
   refazer trabalho já pago.*
7. ⭐ **O gate 48 não era edificável do lado limpo:** «o conjunto que se moveu» não tinha limiar nem
   dizia que campos se comparam, e o `p95` não tinha convenção — ficam escritos. Mais três
   correcções no mesmo gate: o tecto do chão é `25,7×` e não `19,9×`; a barra da régua alternativa é
   `0,13 × |u|` **daquele vértice**; e o `± 1` dos movidos é **em torno do valor do cabeçalho**, não
   uma amplitude (uma das realizações varre `2161`–`2163`). ⏳ **Dívida nomeada:** a §10.15 abre com
   *«a régua que dá o chão tem de estar ao lado do dado»* e só o chão do **máximo** viaja no
   cabeçalho — o do `p95`, que é o que decide, vive numa tabela.
8. ⭐ **O gate 50 mandava comparar ficheiros «byte-idênticos» que não podem sê-lo:** cada cabeçalho
   regista o próprio valor do botão. A igualdade é do **bloco de vértices**, e é assim que está
   verificada (espec e README).
9. ⭐ **A §5.6 anunciava uma errata e deixava-a por corrigir duas linhas abaixo** — a cláusula 4 diz
   que a origem do raio é escrita depois da colisão e a lista seguinte continuava a dizer
   «pós-integração do passo anterior», que é onde um port a vai ler (a espécie que a Q17 já pagou
   três vezes). Mais **duas** de contagem no §10.15: as duas linhas de plano davam `0,612818` /
   `0,329616` e os ficheiros (e a recomputação) dão `0,612821` / `0,329617`.

**Veredicto: ATESTADO.** A janela-mãe pode ler a emenda Q19 (com as Q20 e Q21 dentro).

### Q22 — O FILTRO DE TECIDO ganha oráculo, e o §7 tinha quatro afirmações a corrigir (emenda do I de 2026-09-07; resposta no mesmo dia, com 36 corridas NOVAS do oráculo)

**O pedido.** O §7 da espec descrevia o filtro **por leitura** e o §10 não tinha **um único** vector
dele: as `86` fixtures eram todas do pincel. Sem lado aprovado, qualquer barra que o lado limpo
escrevesse sobre o filtro mediria os defeitos dele próprio.

**O que foi reaberto no fonte** (por shell, só o que a pergunta exigia): o comando modal do filtro e
a abertura dele · as propriedades expostas, valores de omissão e faixas · a construção da área e da
lista de restrições no ramo **sem** traço de pincel · a expressão de cada um dos cinco tipos · as
duas matrizes de orientação e o troço que anula componentes por eixo · a soma da gravidade da cena,
comparada **lado a lado** com a do ramo do pincel · o sítio, dentro da preparação da peça para
edição, onde as normais da malha são recalculadas · e o destino da fotografia de normais tirada ao
criar a simulação (**não tem consumidor**).

**O arnês novo, e por que ele não é o do pincel.** O filtro é um comando **modal**: ele abre onde o
cursor está e lê **um passo por movimento do rato**. O arnês do pincel entrega uma lista de pontos a
um operador de traço e nunca precisou de eventos; para o filtro não há operador equivalente.
⚠️ **Duas tentativas falharam antes da que funcionou**, e vale registá-las: mover o ponteiro do
sistema **não entrega evento nenhum** neste ambiente gráfico (foi confirmado com um observador de
eventos: zero eventos nossos, e só os movimentos reais do utilizador a chegar) — o que também
explica por que o arnês do pincel tem um auto-teste com repetições para o sobrevoo. A que funciona é
a **simulação de eventos** que o próprio programa oferece por opção de linha de comando: ela entrega
o movimento **e** o premir do botão (que fixa a origem do arrasto) **e** o largar (que fecha o
comando), tudo determinista e sem tocar no ponteiro real. ⇒ várias configurações por processo.

**As 36 corridas** (17 configurações; duas realizações no plano, quatro na esfera) e as fixtures
estão em `docs/3D/cleanroom/fixtures/cloth/filtro/`, com a secção própria no README de lá.
⛔ **Elas ficam num SUBDIRETÓRIO por causa de um gate:** o censo do arnês de paridade do pincel
exige que as listas dele sejam o corpus **inteiro do diretório** e varre a raiz sem recursão — pôr um
traço de filtro na raiz deixaria esse gate vermelho, e ele não é corrível por aquele arnês. O
verificador do corpus foi estendido para varrer também um nível abaixo (herdando as malhas de
repouso da raiz) e o gerador de índice ganhou as chaves novas; **103 ficheiros, exit 0**.

**Os quatro veredictos da medição**, todos já escritos na espec:

1. ⭐⭐⭐ **A normal do *Inflate* é relida a cada passo — e agora é (M), não (F).** O instrumento é um
   **par-espelho**: com a normal congelada, a força do *Inflate* seria o simétrico exacto da do
   tipo *Gravity* na mesma cena, e a cena é simétrica em `z` ⇒ a diferença entre um traço e a
   reflexão do outro é **zero** se a lei for a do repouso. Sem máscara: `0,000000000` nos oito
   passos (a peça translada rígida e a normal nunca vira). Com a peça presa fora de um disco:
   `0` no passo 1 e `0,0055` no passo 8 (`5,06 %` do maior deslocamento), com a normal a virar até
   `46,74°`. ⚠️ **A fixture que o pedido chamava «a que decide» — o plano sem máscara — decide
   ZERO**; a régua nasceu do mecanismo, não da lista.
2. ⛔ **A gravidade da cena tinha TRÊS afirmações erradas** e nenhuma era observável no corpus do
   pincel (ali ela está a `0`): o sentido com um objecto de gravidade é o **`+Z`** dele e não o
   `−Z` (o ramo do pincel nega, este não); o vector é usado **cru**, sem normalizar e sem ser levado
   ao referencial da peça; e ele é multiplicado pela força do arrasto, logo **inverte com o
   arrasto**. Mais uma quarta, medida: ela **não** passa pelo factor por vértice, então um vértice
   fora do conjunto de faces activo **cai na mesma** (`2080` de `2080` medidos).
3. ⭐ **As bandeiras de eixo eram «o código lido só as aplica ao Scale» e passam a facto medido**: o
   mesmo traço de gravidade com um eixo só sai **byte a byte igual** ao das três.
4. ⚠️ **A regra do pincel de que o 1.º passo não simula não existe no filtro** (um evento já
   deforma), e o programa emite um movimento extra logo após o premir do botão, no mesmo píxel, com
   força `0` — no-op exacto sobre a malha em repouso.

**Reprodutibilidade.** No plano as dezasseis corridas são bit-reprodutíveis (`0,000000` entre duas
realizações); a esfera sorteia `0,001015` sobre quatro — a mesma família já registada para o pincel,
com a **superfície** como variável.

**Estado:** ✅ **ATESTADA — R-pré de 2026-09-09** (revisão ATRASADA, dois dias depois da emenda,
pedida pelo achado de processo que o R-pré da Q23 registou no fim deste ledger).

**Papel R (R-PRÉ da Q22) — 2026-09-09, subagente R-pré despachado pela janela-mãe
`1246816c-63cf-414b-842d-663a8baa86ca`; contexto novo, independente do subagente-E que a escreveu;
leu os dois lados (o fonte por shell, `cat`/`grep`, sob o deny de `Read` da linha).**

**Cobertura.** O diff de `3cd972280` inteiro (§7 nas 12 linhas · §7.1 NOVA · §10.17 NOVA · §14-bis
NOVA com os gates 55-60 · a secção do filtro no README das fixtures · o gerador do índice · o
verificador · as 17 fixtures + 4 ficheiros por passo) mais a errata `3095d397b` (a contagem de
corridas, 26 → 36).

**PAREDE.** Sweep **verde** (vassoura de 70 entradas) sobre: a espec emendada · o README das
fixtures · `gera_indice.py` · `verifica_traco.py` · `filtro/indice.json` · ⭐ o conteúdo
**DESCOMPRIMIDO** das 43 fixtures do directório do filtro (o `strings` de um `.gz` não vê o texto de
dentro) · e o **histórico** da espec e das fixtures (`--git-history`), todos exit `0`.
⛔ **UM achado de §4.2**, curado no acto: a linha do tipo *Gravity* no §7 justificava o eixo da
*View* com uma frase que espelhava, quase palavra a palavra, um comentário do fonte (a mesma espécie
que os R-prés da Q14 e da Q17 curaram três vezes) — passa a dizer **o comportamento** (qual eixo, e
que é o do ECRÃ e não o da profundidade), com a fixture que o mede.
⚠️ Conferido de propósito: os nomes das 17 fixtures, as chaves novas do cabeçalho e o vocabulário do
§7.1 (*par-espelho*, *controlo*, *giro da normal*) são do domínio; `Local`/`World`/`View`/`Gravity`/
`Inflate`/`Expand`/`Pinch`/`Scale`/*Force Axis*/*Strength* são a superfície pública que o artista vê
(§4.1.13).

**FIDELIDADE — os factos conferidos no fonte e TODOS os números reconstruídos do zero das fixtures**,
com script próprio fora do repo. Confirmados no fonte, um a um: a lei da força escalar em píxeis (e
o sinal do arrasto para a direita); a ordem do passo (preparar a peça para edição → guardar estado →
forças → activar → simular) e, com ela, que as normais do *Inflate* do filtro são as **de agora**;
a soma da gravidade da cena **antes e sem** o factor por vértice, com as TRÊS correcções da emenda
(o `+Z` do objecto de gravidade contra o `−Z` do pincel · o vector cru, sem normalizar e sem mudar
de referencial · a multiplicação pela força, logo segue o arrasto); as bandeiras de eixo a tocarem
**só** a âncora do *Scale*; `τ += 0,01 · f` no *Expand*; a âncora do *Scale* relida da pose de
abertura; o ponto de aperto fixado na abertura; a plasticidade a entrar na criação com `0` e as
restrições construídas uma vez com raio infinito; e os valores de omissão e faixas (massa `1`,
`0..2` · amortecimento `0`, `0..1` · força `1`, `−10..10` · colisões desligadas).
⭐ Reproduzem-se **célula a célula**: os 17 cabeçalhos da tabela do §10.17 (movidos · máx `|u|` ·
dispersão · realizações) · a translação rígida `(0,0,−0,108000)` com desvio `0,000000000` · a
identidade byte a byte do traço de eixo único (`0,0`) · a metade exacta da massa `2` (`0,054000`) ·
os `0` movidos da força `0` · o giro da normal nos oito passos (`0,61` · `2,46` · `6,12` · `12,05` ·
`20,14` · `29,16` · `38,51` · **`46,74`**) · o `+0,016882` radial no flanco · os `96,8 %` de arestas
esticadas a `+5,0 %` · os `100 %` de radial negativo do aperto (`4224/4224`) · os `2080` de `2080`
excluídos a mover-se · `36 = 16 × 2 + 4` corridas · o `indice.json` dos dois corpora a regenerar-se
**byte-a-byte** (`86` e `27`) e o verificador **verde sobre os 113**.

**SETE curas aplicadas no acto, todas funcionais (nenhum facto perdido)** — detalhe no cabeçalho da
espec: (1) o achado de §4.2 acima; (2) o `1,7·10⁻⁶` do XY do *Scale* com gravidade de cena é leitura
do oráculo e **reprovaria a fixture que o define** (o corpus dá `2,0·10⁻⁶` por componente), com a
régua «vértice a vértice contra o traço de escala» a ficar escrita; (3) os `−0,2156`/`−0,1095` dos
conjuntos de faces vinham de **duas réguas diferentes, nenhuma escrita** — passam aos **patamares**
`−0,216 = 2 × 0,108` e `−0,108`, com a média crua (`−0,2146`/`−0,1095`) nomeada como o que o anel de
fronteira faz; (4) o **gate 60** exigia uma leitura do 1.º passo de uma fixture que saiu **sem**
`.porpasso` — passa ao estado final, que o corpus publica; (5) os nove dígitos da tabela do §7.1 são
do oráculo e o corpus só os devolve a menos da resolução do ficheiro ⇒ o gate 56 passa a medir a
FORMA com tolerância `2·10⁻⁶`, mantendo o `0` exacto do controlo; (6) a régua das percentagens de
alargamento (o **vão da caixa** por eixo) não estava escrita em lado nenhum e três leituras
plausíveis discordam — fica escrita no §10.17 e citada no gate 58; (7) o §10 mandava contar as
fixtures com um `ls` que **não recursa** e, desde esta emenda, há **dois** corpora — a instrução
passa a nomear os dois.
⚠️ **Mais duas fora da espec:** o README das fixtures prometia que o verificador procura as malhas de
repouso «no diretório-pai», o que ele **não faz** (apontá-lo a `filtro/` estoura) — passa a dizer a
invocação real; e a errata da contagem de corridas (`3095d397b`) tinha deixado o **título** desta
secção em `26` com o corpo em `36`.

**Veredicto: ATESTADO** — a emenda Q22 pode ser lida e implementada pela janela-mãe.

---

### Q23 — O FILTRO ATRAVESSA GESTOS (emenda do I de 2026-09-09; ABERTURA registada ANTES da primeira leitura)

**Abertura (§6).** Passagem E de 2026-09-09, subagente-E despachado pela janela-mãe
`1246816c-63cf-414b-842d-663a8baa86ca`. Esta linha foi escrita **antes** de qualquer leitura do
fonte do alvo nesta passagem. Patente (§8.1): ⭐ **nada mudou** — o checkpoint incondicional já foi
feito em 2026-09-05 (quatro achados, dois vivos e nenhum lendo sobre o método, com as duas cercas
nomeadas); o objecto desta emenda é **composição entre invocações do filtro**, que não alarga o
método para nenhuma das quatro reivindicações auditadas ⇒ **a busca NÃO se repete** (§8.1 exige
o checkpoint por alvo, e ele está no topo deste ficheiro).

**O pedido, transcrito do `INBOX_blender-cloth.md` §«Q23» (2026-09-09, sessão 1246816c).**

> Report do dono, verbatim: *«quando uso inflate e faco mais de uma simulacao o objeto desinfla a
> cada inicio de simulacao»*.
>
> **Medido do lado limpo** (três gestos de Inflate seguidos sobre uma esfera UV 32×64, volume
> normalizado ao repouso): gesto 1 sobe monotonamente de `1,000` a `1,168`; os gestos 2 e 3 **começam
> em `1,103`/`1,101`, AFUNDAM até `0,887`/`0,886` no `k5`** e só depois voltam a `1,166`.
> ⇒ duas coisas: (a) a inflação **não acumula** (os três gestos acabam no mesmo sítio); (b) o início
> de cada gesto novo **afunda a peça abaixo do repouso** antes de voltar a subir — e é a (b) que ele vê.
>
> **A causa é NOSSA e está identificada:** a wave de 08/09 fez o material atravessar os gestos (cura
> do report *«o pano continua esticando»*), e no lado limpo isso é **incondicional**. A espec §6.3 diz
> que no alvo a deformação **acumula** entre traços, e a §6.4 diz que a base persistente é uma
> **opção do artista** cujo efeito medido é **saturação**. ⇒ o lado limpo tomou como lei o que no alvo
> é um interruptor, e os cinco tipos do filtro não querem a mesma coisa.
>
> **Q23.1** — o **FILTRO** lê a base persistente? (gravar a base no repouso com o pincel activo,
> depois correr o filtro *Gravity* duas vezes; contra o mesmo par sem base gravada). A §7 diz que o
> filtro nasce «sem pino e sem memória de forma» e **não diz nada sobre a base** — se ela não for
> lida, o alvo não tem resposta nenhuma para o report do dono e a divergência do lado limpo é
> deliberada e tem de ser declarada.
> **Q23.2** — *Gravity* corrido **três vezes seguidas** sobre o plano mascarado: a deformação acumula
> como no pincel (`+81 %`, `+145 %` do §10.16)? Posições depois de cada corrida.
> **Q23.3** — *Inflate* corrido **três vezes seguidas** sobre a esfera: a peça cresce monotonamente?
> Há algum passo em que ela ENCOLHE a seguir a uma corrida anterior?
> **Q23.4** — o mesmo par de três corridas para **Expand** e **Scale** (o censo da família: os dois
> mexem no repouso por construção, e a pergunta é se o alvo os deixa compor).
> **Q23.5** — **uma** corrida longa de *Inflate* na esfera (36 passos): a peça cresce sem tecto? Qual
> o esticão máximo por aresta contra o repouso no fim?
> **Q23.6** — censo do PAINEL do filtro: há algum controlo que limite esticão, conserve volume ou
> enrijeça dobra? (a §5.7 diz que o solver não os tem; a pergunta é sobre a superfície que o artista
> vê, para a nossa declaração de divergência nomear o que acrescentámos).

**O que foi reaberto no fonte** (por shell, só o que as seis perguntas exigiam): o ponto onde a
simulação do filtro é criada e o que ela recebe (em particular se as posições persistentes chegam
lá, e com que valor de plasticidade); as quatro leituras que as posições persistentes substituiriam
na construção de restrições; o troço onde a lista de posições persistentes é criada, e por quem;
o que acontece a essa lista quando um novo uso do filtro abre; a expressão dos tipos *Expand* e
*Scale* na parte que toca o repouso; e a lista de propriedades expostas pelo comando modal do
filtro (o censo do painel da Q23.6), com o Python que as desenha.

**As corridas novas.** `docs/3D/cleanroom/fixtures/cloth/filtro/`, no mesmo formato e cabeçalho das
17 anteriores, com a chave `invocacoes` nova (⇒ coluna nova na régua de excepções do README) e as
posições **depois de cada invocação**. Reprodutibilidade medida nas duas superfícies (duas
realizações no plano, quatro na esfera), como manda a família §10.13/§10.15/§10.17.

**Os veredictos** estão na espec: **§7** (a linha nova da tabela + a §7.2) e **§10.18**.

**Sweep (§7.1), corrido em 2026-09-09 pelo E:** ✅ **verde** sobre a espec, o README das fixtures, o
gerador de índice, o diretório `fixtures/cloth/filtro/` inteiro e o texto do report final (varrido no
`draft/` antes da entrega). ⚠️ **E verde sobre as 67 linhas NOVAS deste ledger, corridas à parte.**
⛔ O ficheiro inteiro do ledger continua a acusar **dois** achados **pré-existentes** de 2026-09-05
(as linhas da tabela de cobertura da travessia e o achado de parede nº 1) — eles são o que a
**SKILL §6 exige** que o ledger contenha (*«áreas/arquivos do fonte percorridos»*), já foram
auditados pelo R-pré em 2026-09-05, e o ledger é deny-listed para a janela-mãe. Esta emenda **não
lhes tocou**.

**Estado:** ✅ **ATESTADA pelo R-pré em 2026-09-09.**

### Papel R — R-PRÉ da Q23 (2026-09-09)

**Quem.** Subagente R-pré despachado pela janela-mãe `1246816c-63cf-414b-842d-663a8baa86ca`, com
contexto novo e independente do subagente-E que escreveu a Q23. Leu os dois lados (o fonte do alvo
por shell, nunca por `Read`). ⛔ Não escreveu nem ditou código de produto.

**Cobertura.** O diff de `aea203f43` inteiro: a §7 (3 linhas novas), a §7.2 NOVA, a §10.18 NOVA, o
§14-bis.2 NOVO (gates 61-65), as 10 fixtures novas de `fixtures/cloth/filtro/` (22 ficheiros
`.gz` + `indice.json`), as chaves novas do README das fixtures, o `gera_indice.py` e esta secção do
ledger. Mais o **cabeçalho inteiro** da espec, onde o atestado ficou.

**Parede (§7.1) — corrida pelo próprio R-pré, ⛔ não pelo relatório do E.** `cleanroom-sweep.sh`
com a vassoura de 70 entradas: ✅ verde sobre a espec emendada · o README das fixtures · o
`gera_indice.py` · o directório `fixtures/cloth/filtro/` inteiro · ⭐ **o conteúdo DESCOMPRIMIDO das
43 fixtures do directório** (o `strings` de um `.gz` não alcança o texto de dentro — este passo não
constava do relatório do E) · e `--git-history` sobre a espec e sobre as fixtures. ⛔ Os **dois**
hits do ficheiro do ledger são os pré-existentes de 2026-09-05, já registados; a Q23 não lhes tocou,
e o ledger é deny-listed para a janela-mãe (§3.I).

**§4.2 — ZERO achados de expressão.** Sem trecho, sem nome interno, sem wording de comentário ou de
manual. Os rótulos de painel citados são a **superfície pública** que o artista vê (§4.1.13, com o
precedente já atestado desta espec); as cinco chaves novas de cabeçalho e os dez nomes de fixture são
vocabulário do domínio; as fixtures declaram `entrada NOSSA` no cabeçalho e a proveniência do README
cobre a base persistente (gravada pela porta do programa **sobre a nossa malha em repouso**).

**Fidelidade — todos os números reconstruídos do zero das fixtures**, com script próprio fora do
repo: os máximos por invocação das 7 corridas de plano e as 8 razões; a monotonia estrita dos 24
passos sem base e a quebra dela com base; `0.170358` (`−19,8 %`); os esticões `1,3371 → 1,3516` e
`1,3371 → 1,1427`; o volume da esfera e os 24 incrementos positivos; a corrida de 36 passos (volume,
aresta, raio, pico no passo 33, `−1,01 %`); as duas identidades byte a byte (controlo e *Repeat* 5);
as duas dispersões; o `indice.json` a regenerar-se **byte-a-byte** (27 para 27); e a régua de
excepções do README, **derivada dos cabeçalhos** — as 13 linhas batem uma a uma.

**SETE curas aplicadas no acto, todas funcionais** (detalhe no cabeçalho da espec): um número
**refutado pela própria fixture** (o passo mais lento dos 24 é o primeiro de todos, não o da 2.ª
invocação); os `34` blocos e duas leituras do *Inflate* que são **do oráculo** e não têm contraparte
no corpus (aquelas fixtures saíram sem `.porpasso`), com o gate 63 reduzido ao par que a tem; um
cross-reference para um `§8.4-bis` **inexistente**; uma frase que descrevia a **forma do código**;
o **gate 51**, obsoleto por omissão desde esta emenda (afirmava como universal uma lei que a Q23
mostra ser da população do pincel); e as barras dos gates **61** e **64**, que não nomeavam de onde
saíam.

⛔⛔⛔ **ACHADO DE PROCESSO — a Q22 nunca foi atestada.** Toda outra emenda desta linha tem um commit
`R-PRE … ATESTADO`; a Q22 (`3cd972280`, 2026-09-07) não tem nenhum, a entrada dela no quadro da
espec continua **⏳** e a secção dela neste ledger continua a dizer *«aguarda R-pré»*. Ela é quem
escreveu o §7 inteiro, a §7.1, a §10.17 e os gates 55-60 — o substrato da Q23. **O atestado da Q23
NÃO a cobre**, e a §3.R diz que sem atestado a janela não implementa (esta linha já pagou o erro uma
vez, no incidente de processo da Q15). ⭐ A **parede** da Q22 está verificada de facto (o sweep do
R-pré da Q23 correu sobre a espec inteira, sobre o directório das fixtures descomprimido e sobre o
histórico, tudo verde); o que lhe falta é a auditoria de **§4.2/expressão** e a **fidelidade** dos
números. ⇒ despachar um R-pré para a Q22 antes de implementar a parte do FILTRO.
✅ **FECHADO em 2026-09-09, no mesmo dia:** a janela-mãe despachou o R-pré atrasado, que auditou o
diff de `3cd972280` + `3095d397b` e **ATESTOU** a Q22 — 1 achado de §4.2 e 7 curas funcionais, todas
aplicadas no acto (secção «Q22», acima). ⚠️ **A lição fica nomeada:** o portão que falhou não foi o
sweep nem a fidelidade — foi o **despacho**; uma emenda que shipa com `⏳ Aguarda R-pré` escrito nela
própria só é apanhada se alguém correr o censo por bloco do cabeçalho da espec (§ do instrumento
`awk`), e desta vez foi um R-pré posterior a dar por ela, não o censo.

---

## Censo §4.2 da ÁRVORE DO PRODUTO — 2026-09-09 (subagente-R do INC-3)

⭐⭐ **É o `ACHADO_proveniencia_por_nome_interno.md` (2026-08-24) VIVO no código rastreado, medido de
novo.** Detector: `grep -rlnE '\.(cc|cpp|c|h|hh|hpp):[0-9]+' --include='*.rs' crates shells`.

### O veredito, por espécie

- **(a) nome interno de ficheiro de alvo RESTRITO (GPL-2.0-or-later) — 57 ficheiros ⇒ ⛔ §4.2, tem
  de sair.** Todos da mesma família de alvo (escultura/pintura/animação 2D do mesmo programa).
- **(b) alvo PERMISSIVO — 7 ficheiros ⇒ atribuição legítima, FICA.** Cinco citam o tema de um editor
  **MIT**, uma cita a biblioteca de quantização **MIT** do quad remesh. ⭐ Uma delas (`a_list_is_not_a_form.rs`)
  **nomeia a licença ao lado da citação**, que é a forma certa e devia ser a convenção das outras seis.
- **(c) falso positivo — ZERO.** ⚠️ Isto é ele próprio um achado: o detector **não erra para cima**
  nesta árvore, então a contagem não pode ser descontada como ruído.

### ⛔ A CONTAGEM DE 64 É UM PISO, NÃO A POPULAÇÃO — três cegueiras medidas do detector

1. **Ele exige `:<linha>`.** Há **156 linhas** que citam um ficheiro interno **sem** número
   (`` `nome.cc` ``) e que ele não vê — e **dois dos três achados da vassoura** (ponto 3) são
   exactamente dessa forma. Alargando para *«ficheiro interno entre crases OU com linha»*, a
   população passa de **64 para 107 ficheiros** (+43).
2. **Ele só conhece extensões de C/C++.** Há **8 citações com linha em ficheiros `.py`** do mesmo
   alvo, em 8 ficheiros — **4 deles fora dos 64**.
3. **Ele é cego ao SÍTIO.** Três das citações vivem **dentro da mensagem de um `assert!`**
   (`ref_mode_tests.rs:251` · `sculpt3d_filter_tests.rs:104` · `the_sculpt_gesture_is_wired.rs:372`):
   elas entram na **tabela de strings do binário** e são impressas no log do CI quando o gate falha.
   ⚠️ **O `cleanroom-sweep.sh` varre `strings` de binário de propósito** — é a única sub-espécie deste
   censo que o instrumento da parede apanharia *no artefacto compilado*, e a única que sai do repo
   sem passar por `git`. ⇒ **cure estas três primeiro**, independentemente da ordem do resto.

### A tabela

| # | ficheiro (NOSSO) | linhas | espécie | expressão? |
|---|---|---|---|---|
| 1 | `crates/ph2d-editor-core/src/ids/chrome/sculpt3d.rs` | 301,302 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 2 | `crates/ph2d-editor-core/src/paint.rs` | 247 | (b) permissivo — FICA | só endereço |
| 3 | `crates/ph2d-editor-core/src/widget/list_rows/selection.rs` | 11 | (b) permissivo — FICA | só endereço |
| 4 | `crates/ph2d-editor-core/tests/it/a_list_is_not_a_form.rs` | 29 | (b) permissivo — FICA | só endereço |
| 5 | `crates/ph2d-flip-render/src/pipeline.rs` | 428 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 428 |
| 6 | `crates/ph2d-flip-reshape/src/brushes.rs` | 20,243 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 7 | `crates/ph2d-flip-reshape/src/lib.rs` | 154,192 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 154 |
| 8 | `crates/ph2d-flip/src/autokey.rs` | 1 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 9 | `crates/ph2d-flip/src/layer.rs` | 198,257 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 198 |
| 10 | `crates/ph2d-flip/src/onion.rs` | 4,7 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 7 |
| 11 | `crates/ph2d-flip/src/stroke.rs` | 15 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 12 | `crates/ph2d-flip/src/tween_match.rs` | 3 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 13 | `crates/ph2d-painter-brush/src/spec.rs` | 27,53,63,86,176,179,184,188,197,199 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 184,188,199 |
| 14 | `crates/ph2d-panel-painter-layers/src/brush_fallback.rs` | 154 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 15 | `crates/ph2d-panel-sculpt3d/src/paint/brush.rs` | 289 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 16 | `crates/ph2d-panel-sculpt3d/src/rows.rs` | 152,182,365 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 152 |
| 17 | `crates/ph2d-quantize/src/refine.rs` | 35 | (b) permissivo — FICA | só endereço |
| 18 | `crates/ph2d-sculpt3d/src/auto_smooth.rs` | 10,49,78 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 10,78 |
| 19 | `crates/ph2d-sculpt3d/src/auto_smooth_tests.rs` | 4 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 20 | `crates/ph2d-sculpt3d/src/brush.rs` | 203,231,264,290,483,513,521 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 203,231,264,483,513,521 |
| 21 | `crates/ph2d-sculpt3d/src/brush_magnitudes.rs` | 39,147,159,171,202,237,251,262,272 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 39,159,171,202,237,251,272 |
| 22 | `crates/ph2d-sculpt3d/src/brush_scale.rs` | 26,63,86,147 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 63,86,147 |
| 23 | `crates/ph2d-sculpt3d/src/brush_tests.rs` | 194,459 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 24 | `crates/ph2d-sculpt3d/src/brush_verb.rs` | 147,177,214 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 147,177,214 |
| 25 | `crates/ph2d-sculpt3d/src/brush_verb_defaults.rs` | 183 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 183 |
| 26 | `crates/ph2d-sculpt3d/src/brush_verb_filter.rs` | 35,111,266 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 111,266 |
| 27 | `crates/ph2d-sculpt3d/src/brush_verb_predicados.rs` | 107 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 28 | `crates/ph2d-sculpt3d/src/falloff.rs` | 21 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 29 | `crates/ph2d-sculpt3d/src/falloff_tests.rs` | 17 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 30 | `crates/ph2d-sculpt3d/src/footprint.rs` | 68 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 31 | `crates/ph2d-sculpt3d/src/ref_mode.rs` | 93,129,130,138,157,170,207,529 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 138,170,207 |
| 32 | `crates/ph2d-sculpt3d/src/ref_mode_tests.rs` | 251,309 | **(a)** ⛔ alvo restrito (GPL) | só endereço · ⚠️ endereço dentro de `assert!` @ 251 |
| 33 | `crates/ph2d-sculpt3d/src/ref_profiles.rs` | 270,277,301,307,308,309,310,311,312 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 270,277,307,308,309,310,311,312 |
| 34 | `crates/ph2d-sculpt3d/src/stroke.rs` | 362 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 35 | `crates/ph2d-sculpt3d/src/stroke_dab_core.rs` | 21,273,302 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 21,273,302 |
| 36 | `crates/ph2d-sculpt3d/src/stroke_filter.rs` | 59,272 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 59 |
| 37 | `crates/ph2d-sculpt3d/src/stroke_filter_laws_tests.rs` | 400 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 38 | `crates/ph2d-sculpt3d/src/stroke_filter_sharpen.rs` | 12,44,62,92,102 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 44,62,92 |
| 39 | `crates/ph2d-sculpt3d/src/stroke_hc.rs` | 43,75 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 43,75 |
| 40 | `crates/ph2d-sculpt3d/src/stroke_law_tests.rs` | 507,648 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 41 | `crates/ph2d-sculpt3d/src/stroke_plane.rs` | 169 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 42 | `crates/ph2d-sculpt3d/src/stroke_ring.rs` | 76 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 43 | `crates/ph2d-sculpt3d/src/stroke_symmetry.rs` | 98,157 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 44 | `crates/ph2d-sculpt3d/src/stroke_target.rs` | 440,469,502,554,559,570 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 469,502,554,570 |
| 45 | `crates/ph2d-sculpt3d/src/stroke_target_ring.rs` | 68,104 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 68,104 |
| 46 | `crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs` | 16 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 16 |
| 47 | `crates/ph2d-sculpt3d/src/verb_layer_tests.rs` | 537 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 537 |
| 48 | `crates/ph2d-sculpt3d/src/verb_mode_tests.rs` | 262,337,370,418,468 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 49 | `crates/ph2d-sculpt3d/src/verb_strip_law_tests.rs` | 296 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 50 | `crates/ph2d-sculpt3d/src/verb_thumb_tests.rs` | 242 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 51 | `crates/ph2d-sculpt3d/tests/it/measure_layer_front_face.rs` | 13,31 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 13,31 |
| 52 | `crates/ph2d-sculpt3d/tests/it/measure_layer_law.rs` | 284 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 53 | `crates/ph2d-sculpt3d/tests/it/measure_layer_zoom_and_flank.rs` | 22,23 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 22,23 |
| 54 | `crates/ph2d-sculpt3d/tests/it/measure_raycast_feedback.rs` | 14,16,17 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 14 |
| 55 | `crates/ph2d-tokens/src/slider_style.rs` | 108 | (b) permissivo — FICA | só endereço |
| 56 | `crates/ph2d-tokens/src/spacing.rs` | 164,178,210,229,271 | (b) permissivo — FICA | só endereço |
| 57 | `crates/ph2d-tokens/src/visuals.rs` | 128 | (b) permissivo — FICA | só endereço |
| 58 | `crates/ph2d-app-flip/src/smooth.rs` | 3,82 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 59 | `shells/desktop/src/sculpt3d.rs` | 455 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 60 | `shells/desktop/src/sculpt3d_filter_tests.rs` | 77,104 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 77 · ⚠️ endereço dentro de `assert!` @ 104 |
| 61 | `shells/desktop/src/sculpt3d_input.rs` | 271,311 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 62 | `shells/desktop/src/sculpt3d_rulers.rs` | 96 | **(a)** ⛔ alvo restrito (GPL) | só endereço |
| 63 | `shells/desktop/src/sculpt3d_space_tests.rs` | 129,130,140 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 129,130,140 |
| 64 | `shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs` | 353,372,958 | **(a)** ⛔ alvo restrito (GPL) | ⛔ **COM EXPRESSÃO** @ 958 · ⚠️ endereço dentro de `assert!` @ 372 |

**Totais: 64 ficheiros · (a) 57 · (b) 7 · (c) falso positivo **0** · com transcrição: 29.**

⚠️ **`crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs:16` é o PIOR da árvore** e não é da
mesma ordem dos outros: além de **cinco** nomes internos de ficheiro e de um sexto num `.py`, ele
traz um **bloco de código do alvo entre cercas** — condição, chamada e os nomes dos argumentos dela.
Pela régua do §6.2 continua abaixo do «substancial» (não chega a ~10 linhas), mas pelo §4.2 é o item
**1** da lista: *«texto de código, trechos, diffs — nem uma linha»*. ⇒ **cura funcional:** apagar a
cerca inteira e deixar o doc dizer **o facto** (que a lei da referência é condicionada por uma opção
do pincel, que ela nasce desligada e que ninguém no programa a liga) com o gate que o mede ao lado;
o mesmo bloco está re-emitido em `brush_verb_defaults.rs:183` e em `measure_layer_front_face.rs:31`,
e **os três curam-se juntos ou o facto passa a ter três redacções**.

### A cura, em duas classes (⇒ é a taxonomia do ACHADO de 2026-08-24, revalidada)

- **Classe A — só endereço (28 ficheiros).** O **facto** é lícito e o §4.1.2/§4.1.3 manda guardá-lo;
  o que é ⛔ é o **endereço interno como forma de o citar**. ⇒ trocar por uma referência de
  **domínio** (*«observado na referência, no verbo de camada, no passo de frente-de-face»*) **mais o
  gate/fixture que mede o facto**, que é o que devolve a rastreabilidade que o endereço dava.
  ⚠️ **O custo de rastreabilidade é real e já foi nomeado como recusa medida em 2026-08-24** — não
  faça a troca em massa às cegas; faça-a por módulo, à medida que cada um é tocado.
- **Classe B — com transcrição (29 ficheiros).** É a violação de **expressão** e não tem contrapartida
  nenhuma: a atribuição, a condição, a assinatura e o nome do argumento **não acrescentam facto
  nenhum** ao número ou à ordem de operações que a linha já diz. ⇒ sai, e o facto fica.
  ⛔ **Sub-classe que não é higiene nenhuma:** as linhas que citam **um comentário do alvo entre
  aspas** (`ref_mode.rs:170` · `brush_verb.rs:147` · `stroke_filter_sharpen.rs:62`) — o §4.2 chama-lhes
  *«a expressão mais protegida do arquivo»*, e é a claim que a SAS **ganhou** (§1.2). **Estas três não
  têm cura por reescrita: apagam-se.**

⚠️⚠️ **E o que este censo NÃO diz:** nenhum destes 64 ficheiros contém fonte do alvo. A propriedade
que segura a parede — *nenhum arquivo de fonte do alvo está na árvore* — continua verdadeira
(`git ls-files` dá zero). O que vazou, e continua a vazar, é a **FORMA da nota de proveniência**.

---

## Os três hits da VASSOURA de 2026-09-09 — todos REAIS

`bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-cloth.txt <path>` acusa três
ficheiros, e os três casam **a mesma entrada**: o nome interno do ficheiro do alvo que implementa o
sub-sistema desta linha. ⛔ **Nenhum é acidente de vassoura genérica demais** — a entrada é um
identificador idiossincrático, com sublinhado, do alvo, e não uma palavra do domínio.

| ficheiro (NOSSO) | linha | veredito | o que tem de sair, FUNCIONALMENTE |
|---|---|---|---|
| `crates/ph2d-sculpt3d/src/brush_verb_defaults.rs` | 183 | ⛔ **REAL** — Classe B | a nota lista **cinco** ficheiros internos do alvo e transcreve a condição que os abre, mais o nome de uma bandeira interna, mais um sexto ficheiro `.py`. ⇒ fica **o facto**: *a lei de frente-de-face é condicionada por uma opção do pincel, ela nasce desligada, e nenhum caminho do programa de referência a liga* — com o gate que o mede nomeado ao lado. ⛔ zero endereços, zero condição transcrita |
| `crates/ph2d-sculpt3d/src/stroke_dab_core.rs` | 302-303 | ⛔ **REAL** — Classe B | **a mesma nota, re-emitida** no comentário de bloco do caminho quente. ⇒ **não** a reescreva duas vezes: uma das duas passa a **apontar para a outra** (a que vive junto do campo que guarda a opção), senão o facto ganha duas redacções que envelhecem em separado |
| `crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs` | 16-24 | ⛔ **REAL, e o mais grave da árvore** | **a terceira cópia da mesma nota**, e a única com um **bloco de código do alvo entre cercas**. ⇒ apagar a cerca; o doc do módulo de gates diz **por que a fixture é outra** (a grelha plana torna a lei inobservável — isso é medição NOSSA e fica inteira) e **qual facto** os gates cobram |

⭐ **A leitura das três juntas:** não são três dívidas, é **uma nota copiada três vezes**. *Uma lei
escrita em três sítios ainda não é uma lei — só uma PORTA é* (lei já paga por esta casa noutro
módulo). ⇒ a cura certa é **uma** redacção, no dono do campo, e duas referências a ela.

⚠️ **E o sweep verde não absolve a árvore:** esta vassoura tem **70** entradas e cobre **uma** família
de sub-sistema. Os 57 ficheiros do censo acima são da **mesma casa de origem** e o sweep não os vê.
*Um sweep verde vale exactamente o que a vassoura contém.*

---

## Tarefa nº 1 do BLOCO-RETOMADA — as duas regiões CONFIRMADAS, e a TERCEIRA cópia achada

**Os DOIS de FINALIDADE** (a mesma oração traduzida, re-emitida; ⇒ re-exprimir):

| # | ficheiro | linhas | o que fica, o que sai |
|---|---|---|---|
| 1 | `crates/ph2d-sculpt3d/src/cloth_filter_kind.rs` | **169-171** (doc do variante do referencial de vista) | **FICA** o facto: nessa orientação a direcção de queda é o eixo vertical do ecrã e **não** a profundidade, e este é o único caso especial do referencial, resolvido por quem tem a matriz. **SAI** a oração de finalidade sobre o que o artista vê |
| 2 | `shells/desktop/src/sculpt3d_filter.rs` | **278-281** (doc do passo que lê a câmera) | idem — **FICA** o facto e o *porquê ARQUITECTURAL* (este é o único sítio com matriz de câmera, e por isso o caso especial vive aqui); **SAI** a oração de finalidade |

⇒ **Instrução funcional para os dois:** cada um passa a dizer **só o comportamento** — *qual eixo*,
e que é o do ecrã e não o da profundidade — **com a fixture/gate que o mede nomeado ao lado**, na
forma que o §7 curado já usa. ⛔ Nenhum dos dois guarda uma oração de **finalidade** sobre o que o
artista vê. ⚠️ **Não é o facto que é dívida** (facto de comportamento não é protegível, §1.2): é o
**enquadramento retórico do porquê**, que é o que foi traduzido.

**Os TRÊS de FACTO — CONFIRMADOS e FICAM** (repetem o facto, sem a oração de finalidade):

| ficheiro | linhas | por que fica |
|---|---|---|
| `crates/ph2d-sculpt3d/src/stroke_cloth_filter.rs` | **94-95** | diz o eixo e cita a espec; nenhuma finalidade |
| `crates/ph2d-cloth/src/verlet_gesto_pincel.rs` | **106-110** | diz que esta crate não sabe o que é uma vista e que o caso especial é resolvido antes de chegar; é **desenho NOSSO** |
| `shells/desktop/src/sculpt3d_filter.rs` | **356-360** (era 353-355 antes da cura do INC-3, que deslocou o ficheiro) | diz **por que aqui** — argumento de arquitectura desta casa |

⚠️ **Adjacente e também FICA:** `shells/desktop/src/sculpt3d_filter_cloth_tests.rs:162` (é sobre a
**régua**, não sobre o eixo) e a formulação *«o baixo é o −cima do ECRÃ»* de
`shells/desktop/src/sculpt3d_filter.rs:392` + `docs/3D/08_as_tres_features_do_modelador.md:58` —
**idioma próprio desta casa**, usado no modelador 3D, obra sem relação com este alvo.

### ⭐ A TERCEIRA ocorrência: ACHADA, e não estava na árvore

Censo exacto da oração pelo tronco distintivo, sobre `crates/`, `shells/`, `docs/` e a árvore inteira:
**duas** ocorrências rastreadas (as da tabela acima) e **uma terceira FORA do repo** —
`…/scratchpad/K.bak` da sessão **`1246816c…`, a janela QUEIMADA (I-1)**: cópia **byte-idêntica**
(`sha256 d7b5c8cd…`) de `crates/ph2d-sculpt3d/src/cloth_filter_kind.rs`.

⛔⛔ **E a nota do INC-2 sobre ela estava ERRADA no ponto que decidia a acção.** Ela diz que a cópia
*«some quando a prescrição correr»* — **não some**: um `.bak` é congelado no instante em que foi
feito, e curar o ficheiro rastreado não lhe toca. Ela sobreviveria à cura, e é alcançável por
qualquer janela que faça um `grep` em `/tmp` (a espécie do INC-1, a repetir-se pela terceira vez).
⇒ **APAGADA por este R** (é cópia de ficheiro **nosso**, logo higiene e não incidente; o `sha256`
fica registado aqui, que é a evidência). Re-censo depois: **zero** cópias fora da árvore.

⭐ **Lição:** *o §6.4 não termina na memória nem numa relocação — uma cópia congelada de um ficheiro
NOSSO que carrega a dívida a curar é um terceiro portador que nenhuma cura da árvore alcança.* E
⚠️ **o sweep do scratchpad daquela janela sai VERDE** (70 entradas): a cópia não carrega identificador
do alvo nenhum — carrega a **tradução**, que é precisamente o que uma vassoura de identificadores
nunca apanha.

---

## O resíduo de atestado da Q23, e a segunda metade do censo do cabeçalho da espec

**(a) O marcador obsoleto foi APAGADO.** No bloco da `EMENDA Q23 de 2026-09-09` da
`SPEC_cloth_brush.md`, o parágrafo de objecto fechava com um marcador de pendência de R-pré escrito
dentro de si, **com o atestado do R-pré da mesma data logo abaixo** (`e0275bba1`). A emenda **está**
atestada; o marcador era resíduo. Removido por este R (a espec é artefacto de E/R; a janela-mãe não
lhe toca), preservando a forma do título que o censo procura (`EMENDA Q<dígitos> de`).

**(b) A segunda metade do censo — um detector de CONTRADIÇÃO — está escrita na espec, ao lado da
primeira.** O censo que lá estava pergunta *«veio um atestado depois desta linha?»* e é **mudo** quando
a resposta é *sim* **e** o marcador ficou lá: ⇒ *uma emenda genuinamente pendente e uma já atestada
com marcador obsoleto lêem-se **iguais** para quem abre a espec* — foi assim que a Q22 pagou dois dias.
O detector novo percorre **os mesmos blocos, no mesmo `awk`**, e acusa o bloco que tem **as duas
coisas ao mesmo tempo**. ⛔ **A lei de que não pode virar contagem foi mantida:** a saída é o **nome do
bloco**, e silêncio continua a ser o verde.

Três coisas que só a construção impôs, e que ficam escritas ao lado do detector:

1. ⚠️ **A `gsub` das crases é LOAD-BEARING, e é uma LEI DE REDACÇÃO, não uma esperteza do script.**
   A espec fala **sobre** o marcador em dois sítios (a lição da Q22 e a do fecho); sem apagar os vãos
   entre crases, o detector acusaria justamente os blocos que **documentam** a doença. ⇒ **marcador de
   pendência escreve-se NU; menção em prosa escreve-se entre crases.**
2. ⛔ **O padrão casava-se a si próprio** — a 1.ª redacção do detector acusou o bloco onde ele
   **vive**, porque a linha que diz *como* procurar contém o que se procura. É a **terceira** vez que
   este instrumento morde pela redacção (a contagem *case-sensitive*, a redacção que se contava, e
   agora esta). Curado com uma classe de carácter que quebra o literal sem mudar o que ele casa.
3. ⭐ **Os dois braços foram provados por MUTAÇÃO:** o braço novo acusa a Q23 na versão anterior à cura
   e cala-se depois dela; apagar o atestado da Q23 devolve `SEM ATESTADO` pelo braço antigo.
   ⚠️ **E o 1.º controlo desta prova falhou por culpa MINHA, não do detector** — mutei o número de
   linha errado e li «sobreviveu». *Uma mutação que «sobrevive» num ficheiro que se está a editar é,
   primeiro, uma suspeita sobre o endereço da mutação.*

⚠️ **O que continua sem instrumento:** este censo é um comando escrito na espec, e **nada no
`ship.sh` o corre**. Enquanto for assim, ele é uma nota que envelhece (`CLAUDE.md` §2) — a decisão de
o pôr num portão é do dono, e o custo é uma linha.

---

## Fechamento R

⏳
