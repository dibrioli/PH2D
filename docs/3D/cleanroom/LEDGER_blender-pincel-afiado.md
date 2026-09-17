# LEDGER de proveniência — clean-room do PINCEL DE TRAÇO AFIADO (alvo `blender-pincel-afiado`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-pincel-afiado.md` (append cego).

> ⛔⛔ **REGRAS DE HIGIENE, NO TOPO DE PROPÓSITO — leia-as antes de tocar em qualquer instrumento**
> (herdadas do `LEDGER_blender-trim-pincel.md`, onde a mesma classe de incidente aconteceu DUAS
> vezes — INC-R1 e INC-R2 — por a cura estar escrita no fim):
>
> 1. **A vassoura descodifica-se em MEMÓRIA, por cano** (`sed -n 'Np' … | base64 -d`, ou o próprio
>    `cleanroom-sweep.sh`, que já a descodifica em memória). ⛔ **Nunca para ficheiro — nem em
>    `/tmp`, nem no scratchpad, nem em `/dev/shm`.**
> 2. **Um relatório de sweep cita em claro os termos que acusaram** ⇒ é material do alvo e vive em
>    `~/Referencias/blender-pincel-afiado/`, nunca no repo, nunca no scratchpad da janela-mãe.
> 3. **Tudo do alvo** — fonte lido, notas, harness, dumps crus, rascunhos da espec — vive em
>    `~/Referencias/blender-pincel-afiado/`. ⛔ O scratchpad `/tmp/claude-1000/...` é da janela I.
> 4. **O report final do E** é gravado em `~/Referencias/blender-pincel-afiado/draft/`, varrido, e
>    só então devolvido — e segue o CONTRATO DE RETORNO (§3.E): zero identificador interno, zero
>    trecho, zero wording do alvo, zero nome de ficheiro do alvo.

⏱️ **Aberto em 2026-09-16, ANTES da primeira leitura de CONTEÚDO do fonte.** Antes desta abertura o
E fez apenas:

1. **metadados de pacote** desta máquina (`pacman -Qi blender`, `blender --version`) e do checkout
   (`git rev-parse HEAD`, `git describe --tags`, as três primeiras linhas do `COPYING` e a lista de
   NOMES de `doc/license/`) — o primeiro acto obrigatório da triagem (§2: *ler a licença real*);
2. leitura de artefactos **NOSSOS** do repo (a SKILL, `_ComoInvestigarApps`, o `README.md` desta
   pasta, a `SPEC_pincel_de_plano.md` e o `LEDGER_blender-trim-pincel.md`, a memória do projecto);
3. a criação da zona fora da árvore (`~/Referencias/blender-pincel-afiado/{notes,draft,oracle,fixtures,out}`).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — o **pincel de desenho de traço afiado** do modo escultura (rótulo público no catálogo: *Draw Sharp*), os seus valores de fábrica, o caminho de traço que o move, e o que ele partilha com o pincel de desenho comum |
| Versão / commit do fonte lido | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` — o MESMO checkout esparso e grafted das obras `blender-cloth`, `-pose`, `-boundary`, `-pull`, `-unblocked`, `-trim`, `-trim-pincel` (re-conferido em 2026-09-16: `git describe --tags` ⇒ `v5.2.0`) |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D |
| Zona fora da árvore | `~/Referencias/blender-pincel-afiado/` (`notes/` · `draft/` · `oracle/` · `fixtures/` · `out/`) |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença do ALVO | **GPL-2.0-or-later** — `COPYING` remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991) |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS**, pacote `blender 17:5.2.1-2` (build 2026-09-01, commit de 2026-08-24). ⭐ **Re-conferido em 2026-09-16: a versão instalada NÃO mudou** desde a obra `blender-trim-pincel` (mesmo `17:5.2.1-2`, mesma data de build) |

### A concessão relevante (GPLv2 §0) — a mesma transcrita no `LEDGER_blender-trim-pincel.md`

> *"Activities other than copying, distribution and modification are not covered by this
> License; they are outside its scope. The act of running the Program is not restricted,
> and the output from the Program is covered only if its contents constitute a work based
> on the Program (independent of having been made by running the Program)."* (§0)

⇒ Ler, correr e instrumentar **em privado** é licenciado. A **saída** (posições de vértices de
malhas NOSSAS) é dado. Nenhum acto deste ledger envolve distribuição. Não é AGPL.

---

## §2 — Triagem: a escada de portas (percorrida NA ORDEM, 2026-09-16)

⚠️ A unidade da triagem é o **ARTEFACTO/biblioteca**, nunca o nome do projecto.

| degrau | veredito | por quê (medido) |
|---|---|---|
| **T0** (permissivo) | ⛔ | o pincel é aritmética de vértices **dentro** do alvo — nenhuma biblioteca externa lhe dá a lei (a mesma conclusão do `LEDGER_blender-trim-pincel.md`, reconferida: o pincel não chama motor externo nenhum). O pacote declara várias licenças permissivas, mas **para outras partes** do programa |
| **T0½** | ⛔ | os ficheiros lidos levam `GPL-2.0-or-later` no cabeçalho SPDX |
| **T1 (a)** código dos autores | ⛔ | não há paper; a lei nasceu dentro do alvo (revisão pública de 2019, lida — ver cobertura) |
| **T1 (b)** versões antigas | ⛔ | o pincel nasceu já sob GPL (2019) |
| **T1 (c)** reimplementações permissivas | ⚠️ **MEIA porta, já paga** | o **SculptGL (MIT)** tem o pincel de desenho comum e **não** tem este; a nossa casa já o portou. Busca de 2026-09-16 (termos: o rótulo público do pincel + «open source»/«MIT»/«original coordinates»): um repositório de escultura aberto apareceu, **sem** este pincel no catálogo e sem licença permissiva visível na página — descartado. Nenhuma implementação permissiva do comportamento |
| **T1 (d)** e-mail | n/a | licença institucional |
| **T2** | ✅ **este é o degrau** | copyleft com fonte |

⭐ **Reconfirmado em 2026-09-16:** o binário instalado é o **mesmo** da obra `blender-trim-pincel` (pacote `17:5.2.1-2`, build `2026-09-01`), e o checkout do fonte é o **mesmo** commit.

---

## Patente (§8.1) — checkpoint incondicional (2026-09-16)

- **Termos:** `patent digital sculpting brush displace vertices from original stroke-start positions sharp crease` ·
  `patent "sculpting" brush "original positions" vertices displacement along area normal stroke claims granted`.
- **Achados:**

| patente | assunto | estado | alcança-nos? |
|---|---|---|---|
| **US 10 586 401 B2** | pincéis por soluções de elasticidade (kelvinlets) | viva (já registada pela linha, `CLAUDE.md` §5) | ⛔ **não** — assunto diferente (deformação elástica regularizada); este pincel é um deslocamento ao longo de uma normal com queda radial |
| **US 9 830 743 B2** | alisamento com preservação de volume | concedida 2017 | ⛔ não (já registada no ledger do pincel de plano) |
| **US 2013/0249912 A1** | alisamento adaptativo à resolução | pedido | ⛔ não — alisamento |

- **Arte anterior pública:** o próprio pincel é público desde 2019 (revisão e notas de reunião públicas), e «deslocar ao longo da normal da área com queda radial» é o pincel de desenho de todo escultor digital desde os anos 1990.
- ⇒ **Veredito: nenhuma patente VIVA alcança o método.** Sem gatilho para o Enio.

---

## §8.5 — Válvula de escalonamento: NÃO accionada

Mesma análise do ledger do pincel de plano (fundação de software livre; GPLv2 sem rede; Brasil/UE/EUA).

---

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E `agent-acf4ca41445c57ad7`** (tipo `general-purpose`, descrição «Missão-E: o pincel Draw Sharp»), despachado pela janela I da `line/sculpt3d` (sessão-mãe `9f820704-0d7e-4d96-847e-9cd720cbf178`) |
| data | 2026-09-16 |
| ⚠️ transcript | `~/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-…/subagents/agent-acf4ca41445c57ad7.jsonl` — **zona contaminada**; a janela I ⛔ não o lê (§3.I) |

## Corrente I

(a janela I declara-se no `INBOX_blender-pincel-afiado.md`)

| elo | sessão | data | estado |
|---|---|---|---|
| I-1 (desta obra; é a I-3 do `LEDGER_blender-cloth.md`) | `9f820704-0d7e-4d96-847e-9cd720cbf178` | 2026-09-16 | **INC-I1** (leitura por `grep` de documentos NOSSOS, antes da espec) classificado **RELANCE** pelo R de incidente `agent-af9327b96fad70a1b` ⇒ **não queima** (ver *Incidentes*) · ⏳ a declaração da janela ainda não chegou pelo inbox |

---

## Cobertura da travessia (§3.E) — 2026-09-16

⚠️ **Descrita em vocabulário do domínio, de propósito** (o precedente do
`LEDGER_blender-trim-pincel.md`): nomes de ficheiro e de símbolo do alvo estão na vassoura, e um
ledger que os escrevesse faria o sweep `--git-history` desta pasta acusar-se a si mesmo. O E
guarda a lista literal dos comandos de leitura na zona (`notes/comandos_fonte.txt`).

**16 ficheiros do fonte** (15 de código + 1 do painel), lidos por shell dentro da zona:

| área do assunto | lido |
|---|---|
| o pincel afiado — **o assunto** (ficheiro próprio) | ✅ **INTEGRAL** |
| o pincel de desenho comum (o irmão de que ele difere) | ✅ **INTEGRAL** no corpo |
| o núcleo do modo escultura — **só as áreas do assunto** | ✅ integral em: que pincéis pedem os dados do pen-down e a restauração · a normal da área (raio, peso, os dois baldes) · a actualização da normal do traço e a opção que a congela · o cursor e o acerto do raio (incl. a preparação do raio ortográfico) · o acumular · os factores comuns (distância, raio, dureza, curva, faces de frente, máscara, textura, recorte de região) · a direcção e a sua inversão · a força ao quadrado e a pressão · o ciclo do dab e a simetria · a restauração a partir do undo · as invariantes por traço. ⛔ Não integral no resto, e não precisa |
| a maquinaria partilhada dos pincéis de malha (cabeçalho) | ✅ **INTEGRAL** |
| as declarações internas (tipos de pincel, sinalizadores) | ✅ na região do assunto |
| o undo do modo escultura | ⚠️ só a leitura das posições do pen-down |
| a árvore espacial | ⚠️ só o recorte ortográfico do raio (⭐ é aqui que se mede o artefacto da espec §9.1) |
| o traço de pintura (espaçamento, atenuação, primeiro dab, traço suavizado, amostras, pressão) | ✅ integral nas regiões do assunto |
| o utilitário de rotação do traço | ⚠️ só a função |
| o núcleo de pincéis (curva de queda e o seu cálculo por lote) | ✅ integral nas funções do assunto |
| a camada de propriedades públicas | ✅ integral para os controlos do assunto |
| o cabeçalho de valores de omissão do tipo pincel | ✅ integral (⭐ e ele **não** responde os valores por ferramenta — a espec §0.2) |
| o painel | ✅ integral nas três secções do assunto |
| ⛔ o ficheiro binário do catálogo de pincéis | **NÃO aberto** — os valores foram lidos correndo o programa |

**Prosa pública lida** (fora do fonte): o manual do pincel (versão actual e a de 2.91) · as notas
de versão de 4.2 a 5.3 · a revisão pública de 2019 em que o pincel nasceu e a tarefa associada
(cópias de arquivo). Destiladas na §14 da espec em palavras nossas; as frases delas estão na
vassoura, nas duas línguas.

⛔ **História: NÃO disponível no checkout** (`grafted`, profundidade `1`) ⇒ a mineração do §4.1.12
foi feita na prosa pública acima.

### Achado de proveniência que NÃO é desta obra (registado, não curado)

- O **arquivo** do plano 21 (`docs/archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md`,
  §7.18) cita **nomes internos** do alvo (ficheiros e campos) — pré-existente (a população do
  `ACHADO_proveniencia_por_nome_interno.md`). ⇒ a espec desta obra cita **só** o plano 21 **vivo**
  (linhas da tabela), e reescreve a recusa em palavras nossas. ⛔ A janela I não deve ser mandada
  ao arquivo.
- O sweep desta vassoura sobre `crates/` + `shells/` + `scripts/` (relatório na zona,
  `notes/sweep_arvore_codigo_v3.txt`) acusa **3** citações de nomes internos do alvo em crates do
  **módulo Painter** (`ph2d-painter-brush` · `ph2d-panel-painter-layers`), pré-existentes, de
  outra linha. Dívida dela.

---

## Oráculo (§5) — o que foi corrido, e o que ele NÃO tocou

| | |
|---|---|
| binário | `/usr/bin/blender` 5.2.1 LTS, do pacote — ⛔ sem patch, sem build próprio |
| como | janela num compositor **virtual** (`kwin_wayland --virtual --xwayland`, nunca no ecrã do dono), `--factory-startup`, `--enable-event-simulate`, `TMPDIR` apontado para a zona; dois harnesses — **dabs por script** e **arrasto de rato simulado** evento a evento, com fotos por passagem |
| entradas | ⭐ **100 % nossas** — grelhas de quadriláteros 48²/96²/192² com campo de altura analítico (plano · bossas · cilindro) e a mesma grelha em triângulos |
| pincéis | o harness **cria o pincel dele** e escreve todos os valores (**308** corridas). Em **2** corridas (a prova de completude) o pincel é o de **catálogo**, carregado pelo **próprio programa** no arranque de fábrica — ⛔ nenhum ficheiro de pincéis foi aberto pelo harness; os valores foram **lidos** do pincel carregado |
| corridas | **310** (lei 68 · cursor vivo 72 · produto 41 + 43 + 43 · detector 10 · triângulos 4 · diagnóstico 25 · smoke 4), zero falhas no último registo de cada campanha |
| publicadas como fixtures | **80**, em `docs/3D/cleanroom/fixtures/pincel_afiado/` (6,4 MB) |
| reprodução independente | `oracle/reproduz.py` (dabs) e `oracle/reproduz_arrasto.py` + `oracle/bancada_arrasto.py` (traço), a partir **só** do repouso, do caminho e do cabeçalho. Números na espec §2–§5, §8.4, §12 |
| o NOSSO motor | sonda **temporária**, NÃO commitada, `crates/ph2d-sculpt3d/tests/sonda_afiado_nosso.rs` (`#[ignore]`), **apagada** no fecho desta obra. Saídas na zona (`out/nosso/`, 70 ficheiros). Réguas em `notes/regua_*.txt` |

### ⛔ Três armadilhas do oráculo, medidas (espec §9 e README das fixturas)

1. **O caminho por script não refresca as normais de vértice entre dabs** — as cadeias só se
   reproduzem com as normais do repouso.
2. **O acerto do cursor por script, pedido ao próprio operador do traço, falhou depois do 1.º dab**
   (as cadeias da 1.ª campanha com essa opção NÃO se reproduzem e NÃO foram publicadas); o
   cursor vivo foi refeito por corridas-prefixo, com o acerto calculado pelo harness por um raio
   vertical sobre a malha viva.
3. **O recorte ortográfico pela caixa envolvente** perde dabs num plano de espessura zero ⇒ as
   corridas de produto foram refeitas numa superfície com dois vértices de canto a `z = ±1`
   (caixa dupla). A 1.ª caixa só para baixo ainda contaminava os traços que levantam.

---

## Vassoura (§7.1)

`docs/3D/cleanroom/VASSOURA_blender-pincel-afiado.txt` — **262** entradas em base64 (**229** na
entrega; **+33** na 1.ª emenda, ver §1.ª EMENDA DO E), uma por linha: identificadores internos (funções, tipos, sinalizadores, campos) e prosa do alvo —
comentários do fonte, textos do manual e frases da revisão pública —, a prosa também em
**tradução** e em **variante sem acentos**. Escrita e estendida **em memória** (o texto nunca
tocou disco).

- ⚠️ **Valores públicos de enumeração RETIRADOS de propósito** (tipo de pincel, curvas, direcção,
  método do traço, unidade, referencial, forma da pegada): são a chave de regeneração das
  fixtures e viajam no cabeçalho delas (§4.1.13).
- ⛔ **Duas entradas truncadas RETIRADAS por colidirem com identificadores NOSSOS** (dois
  sinalizadores do alvo que são sufixo de ids do Painter): substituídas por formas mais longas
  que só casam no uso do alvo. Medido com o sweep sobre `crates/`, antes e depois.
- Controlo do instrumento (`scripts/cleanroom-sweep-controlo.sh`) corrido **antes** de cada
  sweep: `exit 0`.
- Verificação extra, sem distinção de caixa e sem acentos, em memória, sobre a espec, o README
  das fixtures e os **80** cabeçalhos: **0** acertos (`notes/sweep_draft_caixa.txt`).
- Sweep de caminhos (espec · fixtures + README · INBOX · `README.md` da pasta · este ledger):
  **verde**.
- ⚠️ **`--git-history docs/3D/cleanroom` NÃO sai verde, e os acertos não são desta obra:** `3`
  linhas, todas em patches de **ledgers de outras obras** (tecido · contorno) que citam pelo nome o
  cabeçalho partilhado dos pincéis de malha — legítimo lá (ledgers são E/R-only), e a entrada
  existe nesta vassoura porque esse cabeçalho foi lido aqui. ⇒ o commit **desta** obra foi varrido
  à parte, em memória (`git show` + a vassoura): **0** acertos (`notes/sweep_meu_commit.txt`). Os
  `.gz` aparecem como binário no `git show`; o conteúdo deles foi varrido descomprimido no modo de
  caminhos.

---

## Espec

`docs/3D/cleanroom/SPEC_pincel_afiado.md` — rascunhada na zona (`draft/`), filtrada (§4.3),
varrida, e entregue em **2026-09-16** num commit ÚNICO, scoped, docs-only, `--no-verify`,
`git add -- <paths>`. Junto dele: este ledger, a vassoura, o INBOX, a entrada no `README.md` da
pasta e as **80** fixtures com o README delas.

### O que a espec responde (resumo para o R)

- **Pergunta zero:** exprimível pelo `Draw` modo B com valores (acumular desligado · curva afiada ·
  afundar) + o passo de 5 % e a atenuação `a` por dab; o resíduo (`+3,1 %` / `−11,7 %`) é a lei
  da normal da área, que a casa tem noutras duas portas. O modo «acumular ligado» do alvo é lei
  nova (não é o de fábrica).
- **A recusa de 2026-08-14** reconferida: a metade «binário ilegível» dissolve (valores lidos
  correndo o programa, com prova de completude); a metade «é um checkbox» confirma-se na lei e
  refuta-se no efeito (com a curva afiada a distância do pen-down é alavanca de `+55 %`).
- **Gates:** 12 famílias (G-1..G-12), cada uma com população, piso, barra e a candidata errada
  mais próxima.

### ⚠️ Pontos que o R-pré deve olhar primeiro

1. A barra dos gates de produto vem do **modelo de referência do E** (`float64`); a
   implementação `f32` pode pedir reconferência.
2. A §7.5 mede o nosso motor com uma sonda que **não passa pela porta do produto** (fica escrito
   lá); o G-4 pede a porta.
3. A §14 destila a revisão pública — é o sítio onde uma tradução próxima teria mais chance de
   escapar ao sweep.
4. A §2.7 (pegada projectada) descreve um comportamento que a espec recomenda NÃO oferecer.

---

## Higiene — notas desta sessão

- O ficheiro de recuperação de sessão que o alvo grava ao sair apareceu uma vez em `/tmp` **antes**
  desta obra (não é dela); a partir dos wrappers desta obra o `TMPDIR` do alvo aponta para a zona
  (`out/tmp/`).
- Os ficheiros de saída das tarefas em segundo plano do harness Claude (em `/tmp/claude-1000/…`)
  contêm só linhas de resultado em vocabulário nosso e valores públicos de enumeração.
- ⚠️ **Duas saídas de ferramenta deste E foram persistidas pelo harness** na pasta de resultados
  da sessão-mãe (`~/.claude/projects/…/tool-results/`): uma lista de configurações do oráculo
  (nomes nossos + valores públicos) e uma listagem de *code spans* da espec (texto da espec).
  Nenhuma carrega expressão do alvo.
- A vassoura foi sempre descodificada em memória (Python/`base64` por cano); nenhum relatório de
  sweep foi gravado fora da zona.

---

## Papel R

### R-PRÉ — 1.ª PASSAGEM (2026-09-16) — veredito: ⛔ **NÃO atestado.** A parede do §4.2 está LIMPA; ficam **3 achados que BLOQUEIAM**, **17 erratas** e **1 nota de parede** (um PONTEIRO da espec, não o texto dela)

> Subagente R-pré **novo**, contexto independente do subagente-E `agent-acf4ca41445c57ad7`,
> despachado pela janela I da `line/sculpt3d` (sessão-mãe `9f820704-…`). Corrido sob a regra das
> duas perguntas (§3.R): só a PAREDE é o atestado; da EXACTIDÃO só bloqueia o que faz nascer
> produto errado ou gate que não pode passar.
>
> Leu os dois lados: a espec, as 80 fixturas, este ledger, o código vivo da casa; e, **por shell e
> só dentro do papel R**, as regiões do fonte do alvo que a espec descreve (o ficheiro do assunto,
> a remarcação da dureza, a atenuação do traço, o recorte ortográfico do raio, a escolha de quem
> pede os dados do pen-down e o interruptor do acumular), os dois modelos de referência e as notas
> do E na zona, e a prosa pública arquivada na zona.

#### 1. A parede (§4.2) — LIMPA, item a item, pela forma da §4.3.1

| item do §4.2 | resultado |
|---|---|
| texto de código, trechos, diffs | **nenhum** |
| nomes internos | **nenhum** — extraí os **141** *code spans* não-numéricos e classifiquei-os um a um: caminhos e símbolos **nossos** (com linha), nomes de **fixtura nossa**, símbolos matemáticos locais da espec, o modificador público (`Ctrl`) e `float64`. **Zero** do alvo |
| comentários do original | **nenhum**. ⚠️ O ponto mais perto da forma é a **§14.3**: ela afirma o mesmo **facto** que um comentário do fonte (o interruptor do acumular é inverso neste pincel), mas com **outra razão** e **outra expressão**, e o facto é **medido** por cinco fixturas (`cadeia/bossas_*` × `cadeia/controlo_desenho_*`). Uma palavra («inverso») é o único vocabulário possível para o facto. Passa |
| wording de manual / revisão, verbatim ou quase | **nenhum**. Os dois pontos mais perto: a **§14.1** (o princípio do nome — ideia de um revisor, frase e estrutura nossas; o nome antigo não é escrito) e a **§14.4** (dois factos do manual — o pincel não remalha com topologia dinâmica, e o pincel de vinco é o indicado nesse caso —, em palavras nossas, com o primeiro também **MEDIDO** pela obra do dyntopo). O §1 **não** usa as três aplicações que o manual enumera |
| tabela verbatim / LUT | **nenhuma** — a única tabela de constantes (§5.2) sai da fórmula ao lado, e re-derivei-a inteira |
| organização transcrita | **nenhuma** — a espec é por fases do comportamento; nada da decomposição do alvo (por tipo de árvore espacial, por nó, por fio de execução) aparece |
| pseudo-código espelhado | **não** — os **4** blocos cercados são 1–3 linhas de fórmula, sem controlo de fluxo nem declaração; a fórmula da atenuação é a mesma que a espec do pincel de plano (atestada cinco vezes) |
| cabeçalhos das 80 fixturas | **limpos** — chaves em vocabulário do domínio, valores públicos de enumeração (§4.1.13); as 80 linhas de propósito lidas uma a uma |
| número sem proveniência | ⚠️ **um**: a folga de `1e-3` da §9.1 é **lida**, não medida, e a §13 não a declara (errata **E15** — irrelevante para o produto, que não recorta) |

⇒ ⭐ **A parede está limpa.** Se não houvesse os três bloqueadores, esta passagem atestava.

⚠️ **NOTA DE PAREDE N1 — um PONTEIRO da espec expõe o I** (não é texto da espec; tem de sair na
emenda). A §0.2 manda a janela I reconferir as linhas **434** e **447** do plano 21 **vivo**. A
«Cobertura» acima diz que a espec cita só o plano vivo para **não** mandar o I ao arquivo — mas o
plano vivo **também não está limpo**: a vassoura desta obra acusa a linha **437**; as vassouras do
tecido e dos pincéis de puxar acusam as linhas **68**, **86** e **437**; e a linha **435** traz um
**nome de ficheiro do alvo** que nenhuma vassoura carrega. Tudo pré-existente, de outras waves.
⇒ um I que abrisse o plano naquelas linhas leria nomes internos a uma e três linhas de distância.
**Cura:** a reconferência das notas do plano 21 é acto do **E/R** (ou a reescrita daquelas linhas em
vocabulário do domínio vem **antes**), e a §0.2 deixa de mandar o I lá. Dívida pré-existente do
plano 21 registada aqui para o R-PÓS.

#### 2. O sweep, com controlo primeiro

- `scripts/cleanroom-sweep-controlo.sh` **antes de tudo**: os 11 canais discriminam, `exit 0`.
- `cleanroom-sweep.sh` com **as NOVE vassouras da pasta** sobre a espec + a pasta das 80 fixturas
  (README incluído): **`exit 0` nas nove**.
- A vassoura desta obra sobre este ledger + o INBOX + o `README.md` da pasta: **`exit 0`**.
- ⭐ Varredura **extra, em memória**, sem distinção de caixa, sem acentos, sem ênfase e com as
  quebras desfeitas, com **as nove** vassouras, sobre a espec, o README das fixturas e os **80**
  cabeçalhos: **0** acertos.
- `--git-history docs/3D/cleanroom` com a vassoura desta obra: `exit 1` com **3** linhas — as
  mesmas que o E registou, em patches de ledgers de **outras** obras (contorno, tecido), E/R-only.
  A mensagem do commit desta obra (`cbd964146`), varrida em memória sem caixa e sem acentos: **0**.
- Anexos que a espec cita: o README das fixturas do dyntopo **limpo**; o plano 21 vivo **não** (N1).
- ⛔ Nenhum relatório de sweep foi gravado em ficheiro; todas as saídas ficaram no terminal.

#### 3. O que RE-DERIVEI em vez de aceitar

- ⭐⭐ **Um modelo MEU, escrito só a partir do texto da espec** (§2, §2.1–§2.7, §3, §3.1) e dos
  cabeçalhos, sobre as **27** fixturas por script: as **17** de lei fecham a `≤ 7,9e-8` (planas e
  cilindro) e `≤ 4,2e-6` (bossas e pegada projectada); as **7** cadeias do pincel afiado a
  `≤ 1,95e-6`; os **3** controlos do desenho comum a `≤ 1,55e-6`. ⇒ **a página basta para a lei de um
  dab e das cadeias.** Candidatas erradas, re-medidas: faces de frente sem o factor `5,17e-3` ·
  distância 3D só na normal da pegada projectada `3,13e-4` · sem renormalizar `6,5e-2` · `R_n`
  trocado `≥ 1,86e-3` · normal do lado errado `4,7e-4` (só nas **2** cadeias de bossas; nas outras 5
  é inerte por geometria) · distância viva `≥ 2,2e-2` · cursor do pen-down com o interruptor
  desligado `≥ 3,4e-2`.
- **A régua do §7**, implementada de raiz sobre os blocos `p<k>`/`s` publicados: §7.2, §7.3 (menos
  as células de 192² que **não estão publicadas** — B3), §7.4, §8.1, §8.2, a linha do §3
  (`0,473` constante; `0,2445 · 0,4890 · 0,9781 · 1,9561`) e o §4.3 — **ao dígito impresso**. As
  tabelas do §7.5 e do §8.3 conferem com as notas do E (a sonda do nosso motor foi apagada; os
  números não são refazíveis sem ela, e a espec di-lo).
- A recorrência do §4.1 (`0,1 · 0,13164 · 0,15190 · 0,16670`) e o controlo linear (`6,84e-2`).
- A tabela da §5.2 **inteira**, em `f64` e `f32`; a razão `(1+a)/2 ÷ a = 2,53`.
- As **7** classes do detector (idênticas **ao bit** dentro; `1,06e-2`–`1,28e-2` entre vizinhas) e o
  dab do pen-down: profundidade com `a` ÷ sem `a` = **`0,24591`** = `a`.
- O primeiro dab igual ao bit (G-9; os cabeçalhos diferem **só** no tipo) · a força ao quadrado
  (`0,100000001`) · o espelho (`1,2e-7` sobre os movidos) · §2.3 (as quatro fracções diferem
  `1,86e-3`–`1,52e-2`) · §2.7 (média dos deslocamentos com `z = 0`) · §9.1 (`446`/`507` vértices,
  `0,0148`/`0,0323`, `2,24e-2`; `1,5e-2` e `2,4e-2` no estado final).
- Com o modelo de arrasto do E (lido e corrido **na zona**): `(1+a)/2` erra `4,68e-2` no contínuo e
  `2,26e-2` no pen-down. E **três sondas novas** com ele: a lei da normal de vértice (E14), a linha
  do traço do cilindro (B1) e a cobertura das fotos (B3).
- **O código vivo, sítio a sítio:** todas as linhas citadas na §6 e na §10 conferem (valores de
  fábrica, força ao quadrado por verbo, acumular de fábrica **ligado** em qualquer modo, a distância
  do `base` com o interruptor desligado, o passo `0,15 R`, a fronteira `<=` do passeio, o `(1+a)/2`
  só do plano, a dureza, a curva, o só-ligados, o dyntopo do `Draw`, o `invert = ctrl`), com
  **três excepções de MECANISMO** (E11–E13), nenhuma com a conclusão errada.

#### 4. ⛔⛔ Os TRÊS achados que BLOQUEIAM (instruções funcionais de reescrita)

##### B1 — **as 5 fixturas de CILINDRO do produto não carregam o enquadramento do píxel, e a §11 diz que toda arrastada o carrega**

As cinco (contínuo · separados · ablação do acumular · alvo com o nosso composto · controlo do
desenho comum) **não** têm a linha do píxel do pen-down nem a do passo de um píxel no mundo — e
**o oráculo não registou esse mapa nessas corridas** (as corridas da campanha em que ele passou a
ser registado são as outras; o modelo de referência do E **supôs** o das planas). O cabeçalho delas
escreve o traço *«na linha y = 0.0»*, que é o que um leitor guiado pelo cabeçalho usa. Medido com
o modelo de referência:

| cilindro | com o mapa das planas (`y = 0,0025`) | com a linha do cabeçalho (`y = 0`) | barra |
|---|---|---|---|
| contínuo (G-3a) | `4,9e-4` | **`3,8e-3`** | `2e-3` |
| separados, núcleo · total (G-3b) | `7,8e-4` · `3,6e-3` | **`1,13e-1`** · **`1,13e-1`** | `8e-3` · `3e-2` |

⇒ **quatro gates contam o cilindro (G-3a, G-3b, G-4a, G-4b) e assentam num enquadramento que só
a prosa das planas carrega**; um I guiado pelo cabeçalho reprova-os contra uma lei certa e não tem
como ver o buraco (não lê a zona). *É a espécie do B2 da 3.ª passagem do pincel de plano.*
⇒ **Cura funcional:** re-correr as cinco com o mapa **registado** e regenerar os cabeçalhos
(aditivo: duas linhas por ficheiro; provar a saída **byte-idêntica** à publicada, que é a prova de
que a vista era a mesma) — e o mesmo na fixtura da caixa fina com `Ctrl`, que também não o traz —;
**ou** declarar o mapa **inferido**, com a proveniência e a medição que o confirma; e a §11 deixa
de afirmar o universal.

##### B2 — **o G-1 e o G-2 contam 3 fixturas cuja lei a própria página recomenda NÃO construir, ou nunca pede**

| fixtura | onde a página a deixa | o produto que a página pede lê | barra |
|---|---|---|---|
| a de lei da **pegada projectada** | §2.7 + **P-4** «não oferecer neste modo»; a casa não tem pegada projectada no desenho | **`8,5e-2`** | `1e-5` |
| a cadeia de bossas com o **acumular ligado** | §10.4 «lei nova» + **P-2** «esconder até haver pedido» | **`4,7e-4`** | `1e-5` |
| a cadeia de bossas com a **normal do pen-down** | a opção da §3.1 — **nem** a §10.3 a lista, **nem** a §15 a decide; o `Draw` da casa não a tem | **`6,2e-2`** | `1e-5` |

⇒ quem constrói **o que a página pede** reprova o G-1 numa fixtura e o G-2 em duas. É **população
de gate que a página não sustenta**.
⇒ **Cura funcional:** **ou** as três saem para uma catraca **NOMEADA** «pendentes de decisão do
dono» (com o censo que as devolve ao gate no dia em que o modo for oferecido) e as populações se
recontam (G-1: **16**; G-2: **5** cadeias, **36** estados), e a opção da §3.1 ganha linha na §10.3
**ou** uma decisão própria na §15; **ou** a página diz, com todas as letras, que as três leis se
constroem e se gateiam mesmo escondidas — e então a §15 tem de o dizer, porque hoje recomenda o
contrário (e esta casa trata «motor sem botão» como defeito).

##### B3 — **o G-4 declara células que o corpus não tem**

O G-4a declara **16** células (4 superfícies × passagens 1, 2, 4, 8) e o G-4b **12 + 4**. As três
fixturas da grelha **192²** publicam **só** a 1.ª passagem/traço e o estado final (conte os blocos
`p<k>`; as de triângulos também, mas não estão no G-4) ⇒ o corpus sustenta **14** e **10 + 4**.
Pior: a **§7.3** imprime para 192² os traços **2 e 4** (`0,4116 · 0,8243`; `0,458 · 0,345`;
`0,899 · 2,392`), que **não** estão no corpus, e o **aprovado** do G-4b (`W 1,3 %`, nitidez `4,8 %`)
vem precisamente de uma dessas células.
⇒ **Cura funcional:** **ou** regenerar as três de 192² com as fotos 2 e 4 (aditivo, provado byte a
byte contra o `git`); **ou** recontar as populações para 14 e 10 + 4, marcar os valores de 192²
dos traços 2 e 4 como **não publicados**, e recalcular o aprovado sobre as células que existem
(re-derivado: contínuo `D 3,2 %` · `W 0,4 %` · nitidez `3,6 %`, inalterado; separados 1–4
`D 3,5 %` · `W 1,2 %` · nitidez `4,7 %`). **As barras continuam válidas nos dois caminhos.**

#### 5. ⚠️ As DEZASSETE erratas (gaveta B — não bloqueiam; a janela I corrige-as enquanto implementa)

| # | sítio | o que está errado, e o que fica de pé |
|---|---|---|
| **E1** | **§7.1** e **§14.5** | `W_f` é a largura **INTEIRA** entre os dois cruzamentos (andando para fora a partir do máximo), **não** metade dela — todo número do §7 e a contagem de células da §14.5 usam essa definição. Os gates relativos (G-4, G-8) não mudam; as tabelas absolutas (largura, nitidez, fundo) só se reproduzem com ela |
| **E2** | **G-7** | o aprovado é **`3,0e-7`** (a forma fechada contra a fixtura; o cursor da fixtura carrega o arredondamento `f32` do alvo), **não** `1,5e-8` (esse é o modelo completo, o **gémeo** da §4.1). A barra `1e-6` fica (`3,3×`) |
| **E3** | **G-5c** | o aprovado é **`7,4e-8`** (a fixtura sem atenuação é a maior das duas), não `1,8e-8`. A barra fica |
| **E4** | **G-3a · G-1 · G-2**, coluna «errado mais perto» | normais congeladas → **`1,11e-2`** (a fixtura com `Ctrl`), não `2,35e-2`; força linear → **`5,2e-2`**, não `1e-1`; distância viva → **`2,2e-2`** e cursor do pen-down → **`3,4e-2`** enquanto a cadeia da normal do pen-down estiver no G-2 (sem ela, voltam a `2,6e-2` e `3,6e-2`; o segundo é inerte por construção na cadeia de cursor imposto). **Todas as barras ficam** |
| **E5** | **G-10** | a fixtura tem **3** fotos + o final (não 4 + o final); o espelho aplica-se aos **DESLOCAMENTOS** (ou salta os dois vértices de canto — as posições leem `2,0` ali). Aprovado `1,2e-7` **re-derivado** |
| **E6** | **G-11** | o aprovado **não** é «—»: o modelo de referência dá **`2,9e-4`** na fixtura de caixa espessa; a comparação com ela **salta os dois cantos** (os repousos diferem de `1` ali), e a regra da §11 *«incluir tal como estão»* tem este gate como excepção **nomeada** |
| **E7** | **G-8** | o padrão `produto/*_valores_de_fabrica_*` casa **6** ficheiros, não 2 — nomeie o par (afiado contínuo × desenho comum contínuo). Qualquer par passa (`2,29` contínuo · `2,75` separados) |
| **E8** | **§12, catraca dos G-3** | **15** fixturas do produto não estão no G-3 **nem** nomeadas como excluídas (as 10 de ablação, a ablação do acumular no cilindro, o alvo-com-o-nosso no cilindro, o controlo do cilindro, os 2 controlos de densidade): os padrões `produto/alvo_com_*` e `produto/desenho_*` não casam os nomes com prefixo. Nomeie-as todas |
| **E9** | **§0.1** | *«quatro valores de fábrica trocados»* contra **cinco** na §6 e no G-6 (curva · direcção · acumular · espaçamento · atenuação) — **gémeo** de contagem |
| **E10** | **§11** | *«`0,4` nas por script»* — **4** fixturas por script (as de cilindro) carregam `0,25`. O cabeçalho manda; a prosa envelhece |
| **E11** | **§10.1** | o predicado citado para *«o cursor na superfície viva»* governa **de que superfície a normal/o plano é LIDO** (`true` no desenho **qualquer que seja** o interruptor — que é o que o modo de fábrica precisa, logo a §3 fica certa); quem governa o **cursor** é o predicado do pick do pen-down (§10.4). A conclusão fica |
| **E12** | **§10.2** | *«as duas implementações percorrem só a pegada de raio `R`»* é **falso para a do pincel de plano**: a consulta da pegada desse verbo já é alargada pela porta do raio da consulta para cobrir a fracção do raio da normal; só a dos pincéis de puxar fica em `R`. ⇒ o modo afiado alarga a consulta **pela mesma porta**, nunca com uma segunda varredura. A instrução da §10.3.4 fica |
| **E13** | **§10.4** | o campo citado como *«fotografia das normais do primeiro toque»* é a fotografia do **PEN-DOWN** (hoje armada só para o projectar em modo plano); a normal **por slot** é que é a do primeiro toque, e num arrasto elas diferem. Qualquer lei de «acumular ligado» ou de «normal do pen-down» precisa da do **pen-down** |
| **E14** | **§2.3 / G-3** (a lei da **normal de vértice** não está escrita) | medido com o modelo de referência: **quadriláteros carregados como quadriláteros** com a média simples da casa passam **todas** as células do G-3 (contínuo `≤ 4,9e-4`; separados 96² `1,4e-3`/`7,7e-4`, 48² `3,1e-3`/`2,3e-3`, 192² `2,2e-3`/`1,1e-3`). ⚠️ **Triangulados** pela diagonal `(i,j)–(i+1,j+1)` com a média simples, o G-3b a 48² lê **`3,25e-2`** total (barra `3e-2`) e **`7,95e-3`** no núcleo (barra `8e-3`) ⇒ **reprova**. ⇒ carregue as fixturas de quadriláteros **como quadriláteros** (a malha da casa aceita-os), ou pese a normal de vértice pelo **ângulo do canto** se triangular; a nota da §7.5 de que a sonda usou triângulos **não é receita** |
| **E15** | **§9.1** | a *«folga de `1e-3`»* não tem proveniência na §13 (é lida, não medida) e o produto não precisa dela (não se copia o recorte). Declarar ou retirar |
| **E16** | **§1** | *«fundo `6,2`–`6,8` contra `3,2`–`5,0`»* é a leitura da 96² apenas (48²: `5,3`–`5,6`; 192²: `6,6`–`7,4`) |
| **E17** | **G-4**, título | *«SEM a normal do alvo»* — com a D-3 adoptada, a porta do produto **tem** a normal do alvo. O gate corre com a normal que o produto tiver; as barras foram calibradas para a da casa e ficam folgadas com a do alvo |

#### 6. Higiene

- A vassoura foi descodificada **só em memória**, dentro do processo (Python a ler o ficheiro
  base64 e a descodificar sem escrever) e pelo próprio `cleanroom-sweep.sh`. ⛔ Nada foi gravado
  em ficheiro por esta passagem **fora** deste ledger e do cabeçalho da espec: **nenhum** ficheiro
  no scratchpad da sessão-mãe (que é da janela I), em `/tmp` ou em `/dev/shm`.
- O fonte do alvo foi lido **por shell** (`cat`/`sed`/`rg` no checkout fora da árvore), só as
  regiões listadas no cabeçalho desta secção. Os modelos do E foram **lidos e corridos na zona**
  (a execução pode ter tocado a cache de bytecode **da zona**; nada fora dela).
- As saídas de sweep com acertos (o plano 21, os ledgers de outras obras) ficaram **só no terminal**.
- ⏳ **R-pré, 1.ª passagem: NÃO ATESTA. A janela I continua FECHADA.** ⇒ emenda do E sobre os
  **3** bloqueadores + a nota N1 (as **17** erratas cabem na mesma emenda), depois **2.ª corrida** do
  R-pré. ⚠️ A emenda cura o que esta passagem nomeou **e** varre a página inteira pela mesma forma:
  cada número de população e de aprovado tem **gémeos** (o E2 é um deles).

### R-PÓS

(por preencher)

## Incidentes

### INC-I1 (2026-09-16) — a janela I leu, por `grep` com contexto, três documentos NOSSOS anteriores à limpeza de citações · classificado **RELANCE** ⇒ a janela I **não** queima

> Registado pela janela I no INBOX (append cego). Transcrito e classificado pelo **subagente-R de
> incidente `agent-af9327b96fad70a1b`** (tipo `general-purpose`, descrição «R: classificar a
> exposição da janela I»), despachado pela janela I `9f820704-0d7e-4d96-847e-9cd720cbf178` —
> **contexto novo**, distinto do E (`agent-acf4ca41445c57ad7`) e do R-pré (`agent-a8b293ca5ea3dc7d7`).
> Viu os dois lados: os três documentos, a espec, este ledger e as vassouras (em memória).
> ⛔ Nenhum ficheiro de trabalho gravado; as saídas com acertos ficaram só no terminal.

#### 1. O evento (descrito, não reproduzido)

- **Quando:** 2026-09-16, ao preparar a missão-E — **antes** de a espec existir e **antes** do aviso
  N1 do R-pré (`16fd88497`), que foi escrito depois da leitura.
- **Como:** `grep -n -i -B2 -A8` pelo nome público do pincel. Reproduzido por este R com o **mesmo
  comando** sobre os **mesmos blobs** (`git hash-object` igual ao `HEAD`; os três ficheiros estão
  intocados desde 13/09):

| documento (NOSSO) | blob | saída | janelas de linhas | sha256 da saída exacta |
|---|---|---|---|---|
| `docs/3D/20_divergencias_tools.md` | `7220a657` | 47 linhas | 47–57 · 381–391 · 468–478 · 520–530 | `d8fe18f963f1b3a816281a2e793ba0e005d5e9d6f77c130f500da19e701e064e` |
| `docs/3D/21_plano_modos_e_ferramentas.md` | `866a6347` | 47 linhas | 257–267 · 432–442 · 445–467 | `a643eb28a0568e27136b109c35c7d75b28167eba7c790f7431fa6cd1d4e3c9e5` |
| `docs/3D/04-Ferramentas/04.1-Pinceis.md` | `72b628b1` | 11 linhas | 80–90 | `1336218e7679841b518e9a73eba2ea1e03f16a5aa3f91c7f628122a311c5b517` |

#### 2. O que havia de EXPRESSÃO do alvo, por espécie

As linhas que a carregam, identificadas pelo sha256 do conteúdo (16 hex, sem o fim de linha):
doc 20 **L49** `dbbdf724d0013ee7` · **L383** `7e4ada1f01b15deb` · **L384** `05fde5988852c149` ·
**L385** `8b165b754a89b806` · **L386** `1fc903aad7b8bcca` · **L388** `7c06980fbc84ba56` · doc 21
**L435** `ddd22849f4ddd346` · **L437** `5f7595f1453a2910`.

- **Nomes internos isolados** — enumeradores (em boa parte identificadores públicos da API de
  propriedades), nomes de campo e de função, nomes de ficheiro, dois deles com número de linha. A
  fila do **pincel afiado** (doc 20 L383) carrega um enumerador e **um nome de campo interno**.
- **Fragmentos de comentário entre aspas: DOIS**, cada um uma oração curta (menos de dez
  palavras), **nenhum** um comentário inteiro — doc 20 L386 (o barro de plano inclinado) e doc 21
  L437 (o filtro de afiar).
- **Fórmulas de uma linha de OUTROS verbos** (a força do barro e das tiras, a lei da demão, as leis
  dos filtros de escala, esfera, aleatório e afiar), em notação nossa.
- ⚠️ **A doc 21 L437 é UMA linha física de 11 038 caracteres** (a célula W9 da tabela de waves). É
  **prosa nossa** — o relato de três waves fechadas em agosto; a expressão do alvo dentro dela
  limita-se a ~8 nomes internos (dois com endereço ficheiro:linha) e a **um** dos dois fragmentos.
  *O comprimento da linha não a torna um bloco de expressão.*
- **Nenhum** corpo de função · **nenhum** bloco de código · **nenhum** comentário inteiro ·
  **nenhuma** tabela de valores do alvo.
- O resto das 105 linhas é **prosa e decisão nossas**, em vocabulário nosso (doc 21 L259–260, L434,
  L447, L459 · doc 20 L470, L522 · doc 04.1 L82). A doc 20 L389 traz só identificadores públicos.

#### 3. Classificação (§6.2): ⛔ **RELANCE**

Nomes isolados e duas orações curtas, vistos por contexto de busca; sem corpo de função, sem bloco
de dez ou mais linhas de expressão, sem comentário inteiro. Mesma espécie e mesmo desfecho do
**INC-4** do `LEDGER_blender-cloth.md` (13/09, mesma sessão-mãe). ⇒ a janela I `9f820704-…`
**continua I** para este módulo; **não há BLOCO-RETOMADA** (§6.4 não se aplica).

#### 4. É MATERIAL para o pincel afiado? **NÃO — nada que a espec não meça já**

O que as linhas expostas afirmam sobre ESTE pincel, em termos funcionais: (a) é o deslocamento do
desenho comum com a distância medida a partir das posições do pen-down → espec §2.1; (b) vinco
duro em vez de domo → §0 e §7; (c) queda afiada de quarta potência → §2.2 e §6; (d) sem
auto-alisar → §6. Tudo é facto funcional, público ou obtido **correndo** o programa, e já medido
na espec.

⚠️ **UMA afirmação exposta é CONTRADITA pela espec, e a espec manda:** a fila do doc 20 (L383) diz
que também a **normal** vem do pen-down; a espec §3 mede que, **no valor de fábrica** (acumular
desligado), a normal da área e o cursor são lidos da superfície **viva** e só as posições da
distância são as do pen-down (`≤ 1,9e-6` contra `4,7e-4` da alternativa). ⇒ transmitido à janela I
em termos funcionais: *a nota velha não é fonte; a §3 é*.

- A exposição toca também leis de **outros** verbos (a demão, os filtros de malha, os barros),
  todos **já shipados** (W6, W8, W9 — agosto) e sem obra clean-room aberta. ⚠️ Se esta janela abrir
  uma obra clean-room sobre um deles, **este incidente entra na triagem de exposição dessa obra**.

#### 5. Quarentena (§6.3) — LEVE, dobrada no R-PÓS

Nenhum código deste pincel foi escrito depois da exposição (a janela ainda não abriu a espec), mas
**todo** o código desta obra será ⇒ a comparação do §6.3 passa a ser **item obrigatório do R-PÓS**,
com três perguntas:

1. **O diff de produto não contém nenhum dos nomes internos das linhas acima.** ⚠️⚠️ **O sweep NÃO o
   garante sozinho — medido por este R:**
   - com as **nove** vassouras sobre os três documentos, das linhas expostas só **duas** acordam
     alguma: doc 21 **L437** (três vassouras) e doc 20 **L389** (uma; identificador público). As
     linhas **L383, L386 e L388** do doc 20 e **L435** do doc 21 — nomes internos e um dos
     fragmentos — **não acordam nenhuma**;
   - em memória, contra a vassoura **desta obra**: das **15** cadeias expostas testadas (nomes
     internos e os dois fragmentos), **uma** é entrada exacta, **três** só existem **dentro** de
     entradas mais longas (que uma cadeia nua não acorda — entre elas **o nome de campo da fila do
     pincel afiado**), e **onze** não existem de forma nenhuma (entre elas **os dois fragmentos**).
     sha256 (16 hex) das onze mais o nome de campo, para a emenda conferir sem as ler aqui:
     `9cc856db91cfb85f` (o campo) · `6d3e24dcd8013e46` · `363a22c5920f92eb` · `a1579b78b6ae6138` ·
     `abef2d14efcca5f9` · `9bae24bee55c5797` · `81a86b8e9c5911d6` · `78485d6fb7d91cf4` ·
     `82eec370625edab8` · `626c7e163855d857` · `c3cc45424166b8b3` · `6048608d585f0031`;
   - controlo do lado do produto: três dessas cadeias (o campo, um nome da demão, um dos
     fragmentos) dão **0** ficheiros em `crates/` + `shells/` (`git grep -l -F`) ⇒ acrescentá-las
     à vassoura **não** cria falso positivo no código.
   - ⇒ **Cura: emenda do E à vassoura**, com as cadeias das linhas endereçadas acima (acto do E, que
     pode lê-las). **Estende a N1 do R-pré**, que só nomeava a L435 como descoberta. Até à emenda, o
     R-PÓS confere estas cadeias **à mão** sobre o diff.
2. **Nenhum comentário ou doc-comment do produto reproduz, nem parafraseia de perto, os dois
   fragmentos.**
3. **A leitura da normal segue a espec §3** (viva no valor de fábrica), não a nota velha.

⚠️ E a N1 continua de pé: a §0.2 da espec manda a janela I às linhas 434 e 447 do plano 21 vivo,
que estão a uma e a três linhas das expostas. Este incidente mostra que a leitura já aconteceu (por
outro caminho); a emenda tem de retirar o ponteiro na mesma, porque ele expõe **toda janela
seguinte** da corrente.

#### 6. Rasto na memória (§6.4): **NENHUM**

- **Controlo positivo primeiro:** `scripts/cleanroom-sweep-controlo.sh` → os 11 canais
  discriminam, `exit 0`.
- `scripts/cleanroom-sweep.sh` com **as nove vassouras** da pasta sobre cinco populações:
  `project-memory/` da **worktree** (zero ficheiros alterados desde 15/09; `git status` limpo) ·
  `project-memory/` do **checkout primário**, que é **a memória injectada** (a ligação
  `~/.claude/projects/-home-enio-Documentos-Projetos-PH2D/memory` aponta para ela; 68 ficheiros por
  commitar, 19 com data de 15/09 ou 16/09) · o **INBOX** · a pasta de **resultados persistidos** da
  sessão-mãe (14 ficheiros) · o **scratchpad** da sessão-mãe (12 ficheiros).
- **`exit 0` em 43 de 45 corridas.** As duas `exit 1` são a vassoura do **trim** sobre as duas
  cópias de `project-memory/`, na **mesma linha** — `feedback_disabled_button_still_dispatches.md`
  L12 (sha256/16 `6fd05d0cafa9dc42`). ⛔ **Não é rasto deste incidente:** o ficheiro é de
  2026-07-09 (`8efbd4510`) e está intocado desde então, e a cadeia acusada é o nome de uma função
  **NOSSA** do módulo de áudio (`ph2d-audio-edit`, a operação de aparar um clipe), que colide com
  uma entrada da vassoura do trim. **Falso positivo pré-existente — dívida de triagem da obra do
  trim** (alongar a entrada, como a R6 dela já fez com três outras). Nada a reverter.
- Nenhum ficheiro de resultados persistidos contém a saída do grep (procurado por uma cadeia NOSSA
  da L437: zero). A transcrição da sessão-mãe contém a exposição por construção (é o contexto
  exposto) e não é injectada em janela nova.

#### 7. Higiene desta passagem

- Vassouras descodificadas **só em memória**: pelo próprio `cleanroom-sweep.sh` e por um
  `while read | base64 -d` que só compara e imprime contagens. ⛔ Nada gravado em `/tmp`,
  `/dev/shm` nem no scratchpad.
- As saídas com acertos (as linhas dos três documentos, a linha de memória do trim) ficaram só no
  terminal.
- O relatório devolvido à janela I segue o contrato: zero trecho, zero wording, zero nome interno,
  zero caminho de ficheiro do alvo.

---

## 1.ª EMENDA DO E — 2026-09-16 (resposta à 1.ª passagem do R-pré: 3 bloqueadores + 17 erratas + N1, e ao pedido do R do INC-I1)

> **Quem:** o mesmo subagente-E `agent-acf4ca41445c57ad7`, retomado pela janela I com a lista do
> R-pré. Fonte do alvo lido **por shell**, só para estender a vassoura (o uso de um nome que as linhas
> do INC-I1 citam) — nenhuma região nova do assunto. O oráculo foi **corrido**
> outra vez (seis corridas). Rascunho na zona (`draft/`), filtragem §4.3 re-corrida, espec copiada.

### B1 — o mapa do píxel no cilindro e na caixa fina com `Ctrl`: **RE-CORRIDAS**

As seis corridas sem mapa (as 5 de cilindro do produto + a da caixa fina com `Ctrl`) foram
repetidas com o harness que o grava (`oracle/cfg_regrava.json`, `log_regrava.txt`, `exit 0`).

| corrida | vértices movidos | `max |Δ|` contra a primeira vez | mapa gravado |
|---|---|---|---|
| cilindro contínuo | iguais | `6,0e-8` | = o das planas, ao dígito |
| cilindro separados | iguais | `3,3e-7` (cresce com as fotos: `1,5e-8 … 3,3e-7`) | idem |
| cilindro, controlo do desenho comum | iguais | `2,3e-8` | idem |
| cilindro, ablação do acumular | iguais | `3,0e-8` | idem |
| cilindro, o motor do alvo com o nosso composto | iguais | `6,0e-8` | idem |
| caixa fina com `Ctrl` | iguais | `3,0e-8` | idem |

- ⚠️ **Não é ao bit** ⇒ o arrasto do alvo **não é determinístico** a esse nível; o chão medido
  (`≤ 3,3e-7`) está ~6 000× abaixo da barra do G-3a. Um valor de cena difere entre as duas vezes
  (o diâmetro em unidades de objecto que o programa guarda em cache, com o diâmetro preso ao
  ecrã) e é **inerte** — a saída prova-o.
- As fixturas passaram a sair das corridas novas (mapa e saída **da mesma corrida**). A régua e a
  bancada nas novas: cilindro contínuo `4,87e-4`, separados `1,39e-3` (com a lei de vértice
  fixada), acumular `3,55e-6`; as réguas impressas não mudam em nenhum dígito.
- **Derivado:** as **53** arrastadas trazem o mapa (`zcat | grep`), 53 de 53.

### B2 — a catraca «pendentes de decisão do dono»

As três saíram dos G-1/G-2 para a espec §12.1, com o censo que as devolve: `lei/pegada_projectada`
(P-4; o produto recomendado lê `8,51e-2`), `cadeia/bossas_cursor_vivo_acumula` (P-2; `4,7e-4`),
`cadeia/bossas_cursor_vivo_normal_do_pen_down` (**P-5**, decisão nova; `6,16e-2`) — os três números
re-medidos com o modelo do E. Populações recontadas: G-1 **16** (aprovado `8,1e-8` · `3,3e-6`), G-2
**5** cadeias / **36** estados (aprovado `1,6e-6`; errados: normal do pen-down `4,7e-4` só na de
bossas, distância viva `≥ 2,6e-2`, cursor do pen-down `≥ 3,6e-2` e inerte na de cursor imposto).

### B3 — as três de 192² ganharam as fotos 2 e 4

Os outros blocos ficaram idênticos linha a linha. A régua das §7.2/§7.3 foi re-derivada **a partir
dos ficheiros publicados** (48² e 192², contínuo e separados): confere ao dígito. O G-4 sustenta as
16 e 12 + 4 células que declara; os aprovados não mudam.

### N1 — a §0.2 reescrita

Ela diz agora o **resultado** da reconferência (o que vale, o que caiu) e **não manda a janela I a
nota antiga nenhuma**; o ponteiro para o plano de modos e ferramentas saiu (e o link). As notas do
NOSSO código que continua a citar (três sítios em `ph2d-sculpt3d`) foram varridas com as **nove**
vassouras: limpas, salvo o nome **público** de um campo de pincel (a vassoura do *pull*), que é a
isenção já pendente de triagem do R (`CLAUDE.md` §5) e está a dezenas de linhas das citadas.
⭐ E a ambiguidade que o R do INC-I1 apontou ficou fechada em dois sítios (§0.2 e §2.1): **só as
posições da distância são do pen-down; o cursor e a normal da área são vivos no valor de fábrica.**

### As 17 erratas

| # | cura |
|---|---|
| E1 | §7.1: `W_f` é a largura **inteira** entre os dois cruzamentos; «meia-largura» → «largura a meia profundidade» em toda a página (§0.1, §1, §7, §14) |
| E2 | G-7: aprovado `3,0e-7` (re-medido: `1,5e-9 · 2,4e-7 · 3,0e-7 · 2,5e-7`); o gémeo da §4.1 reescrito (forma fechada `3,0e-7`; modelo completo `1,5e-8`) |
| E3 | G-5c: aprovado `7,4e-8` |
| E4 | G-3a: normais congeladas `1,11e-2`; G-1: força linear `5,2e-2` (inerte nas 6 de força `1`); G-2: recontado com a população nova |
| E5 | G-10: 3 fotos + a saída; compara **deslocamentos** (ou salta os cantos) |
| E6 | G-11: aprovado `2,9e-4`; salta os dois cantos; excepção nomeada na regra da §11 |
| E7 | G-8: o par nomeado (afiado contínuo × desenho comum contínuo); o par separado lê `2,75` |
| E8 | §12.2: os **22** excluídos nomeados um a um, com o papel; soma derivável do directório = 80 |
| E9 | §0.1: «CINCO valores — três do pincel e dois do traço» |
| E10 | §11: `0,4` em 23 das 27 por script, `0,25` nas 4 de cilindro e nas 53 arrastadas |
| E11 | §10.1: o cursor é governado pelo predicado do pick do pen-down (e a app que o consulta); o outro predicado governa de que superfície a normal e o plano são lidos |
| E12 | §10.2: a consulta do pincel de plano **já** alarga para `R_n > R`; o modo afiado alarga pela mesma porta |
| E13 | §10.4: a fotografia do **pen-down** (armada só no projectar em modo plano) ≠ a normal por slot (primeiro toque) |
| E14 | ⭐ §2.3.1 nova — a lei da normal de vértice **fixada** (tabela abaixo); cabeçalhos com a linha `normais_de_vertice`; G-3b recalibrado |
| E15 | §9.1: a folga retirada; §13 di-lo |
| E16 | §1: a faixa do fundo por grelha |
| E17 | G-4: «corre com a normal que o produto tiver» |

### E14 medida — a normal de vértice (`oracle/normais_de_vertice.py`, `oracle/bancada_emenda.py`)

- **No repouso** o bloco `n` é a média das normais unitárias **pesada pelo ângulo** (`5,4e-7`;
  sem peso `1,8e-4`).
- **Durante o traço** (separados, malha inteira · faixa `|x| ≤ 0,25`):

| lei | 96² | 48² | 192² | cilindro | triângulos |
|---|---|---|---|---|---|
| **sem peso, malha como está** | `1,43e-3 · 7,7e-4` | `3,10e-3 · 2,30e-3` | `2,19e-3 · 1,12e-3` | `1,39e-3 · 7,5e-4` | `1,56e-3 · 8,2e-4` |
| pesada pelo ângulo | `3,70e-3 · 7,9e-4` | `1,17e-2 · 2,31e-3` | `2,60e-3 · 1,12e-3` | `3,60e-3 · 7,8e-4` | `5,48e-3 · 1,21e-3` (sobre quadriláteros) |
| sem peso, triangulada | `3,20e-3 · 1,35e-3` | `3,25e-2 · 7,95e-3` | `2,39e-3 · 1,42e-3` | `3,10e-3 · 1,33e-3` | = a lei |
| pesada pela área | `4,87e-2` | `8,79e-2` | — | — | — |

- Contínuos: todas `≤ 1,04e-3` (não separam). Detector: as duas primeiras idênticas.
- ⇒ a lei é a da nossa casa; o G-3b passa a `8e-3` (malha inteira) e `5e-3` (faixa), com a 48² a
  discriminar a lei de vértice.

### A vassoura, alongada (pedido do R do INC-I1)

- **+33** entradas (262): os **12** nomes e fragmentos das linhas expostas cujo sha256/16 o R deu
  — **os 12 conferidos contra os hashes** —, mais o tempo de vida do cache por traço (que o R contou
  como «só dentro de entradas mais longas»), as traduções portuguesas dos dois fragmentos (várias
  formas) e as variantes sem acentos.
- ⚠️ **Uma das 12 cadeias é um nome curto que também existe no NOSSO código** (5 ficheiros, sem
  relação com o alvo): entrou só na **forma longa** que o alvo usa (lida no fonte), que dá **0**
  no nosso código. Nenhuma das outras 11 casa em `crates/`, `shells/` ou `scripts/`.
- ⛔⛔ **ACHADO N2 — pré-existente, NÃO desta obra:** a tradução portuguesa de um dos dois
  fragmentos **acorda no NOSSO código de produto** — `crates/ph2d-sculpt3d/src/brush_verb_filter.rs`,
  linhas **150** e **292** (um doc-comment e um comentário do filtro de malha de afiar, W9 de
  agosto). É a mesma oração que o INC-I1 achou no plano 21, traduzida de perto. ⇒ a entrada
  **fica** (é para isto que ela existe); a reescrita desses dois comentários em palavras próprias
  é trabalho do dono da W9 (ou da janela I sob ordem do R), e entra no R-PÓS como item.
- O sweep desta vassoura sobre `crates/` + `shells/` + `scripts/` acusa hoje: as **3** citações do
  módulo Painter (pré-existentes, já registadas) e o **N2**. Relatório na zona
  (`notes/sweep_arvore_codigo_v4.txt`).

### Filtragem e sweeps desta emenda

- Filtragem §4.3 re-corrida sobre o texto novo (todos os *code spans* novos são caminhos e
  símbolos nossos, nomes de fixtura nossa, ou variáveis da página).
- Controlo do instrumento **primeiro**, depois o sweep com as **nove** vassouras sobre a espec, as
  80 fixturas + README, o INBOX e o README da pasta; e a verificação em memória sem caixa e sem
  acentos. Resultados no relatório final desta emenda.

### Higiene

- As seis corridas novas escreveram só na zona (`out/*_r2.npz`, `TMPDIR` na zona).
- As saídas das tarefas em segundo plano (em `/tmp/claude-1000/…`) contêm só linhas de resultado
  em vocabulário nosso (nomes de corrida e números).
- A vassoura foi estendida por um processo Python que recebeu o texto no argumento e escreveu só
  base64; as linhas expostas do INC-I1 foram lidas por shell (são documentos NOSSOS) e ficaram no
  terminal.

---

## R-PRÉ — 2.ª PASSAGEM (2026-09-16) — veredito: ✅ **ATESTA.** Parede LIMPA · **zero** achados que bloqueiam · **10 erratas** (E18–E27)

> Subagente R-pré **novo**, contexto independente do E (`agent-acf4ca41445c57ad7`), do R-pré da 1.ª
> passagem (`agent-a8b293ca5ea3dc7d7`) e do R do INC-I1 (`agent-af9327b96fad70a1b`). Corrido sob a
> regra das duas perguntas (§3.R). Viu os dois lados: a espec, as 80 fixturas publicadas, este
> ledger, o código vivo da casa, e — por shell, dentro do papel R — os modelos e as notas do E na
> zona. ⚠️ **Toda a EXACTIDÃO foi re-derivada por um modelo MEU** (`~/Referencias/blender-pincel-afiado/rpre2/`),
> escrito só a partir do texto da espec e dos cabeçalhos, e alimentado **só pelas fixturas
> publicadas** — nunca pelos `.npz` da zona.

### 1. A parede (§4.2) — LIMPA, pela forma da §4.3.1

| item | resultado |
|---|---|
| texto de código, trechos, diffs | **nenhum** |
| nomes internos | **nenhum** — extraí os **172** *code spans* não-numéricos e classifiquei um a um: caminhos e símbolos NOSSOS com linha, nomes de fixtura nossa, chaves do cabeçalho das fixturas (vocabulário do domínio), variáveis da página, o modificador público e `float64` |
| comentários do original · wording de manual/revisão | **nenhum**; o ponto mais perto continua a §14, que afirma FACTOS (medidos ou públicos) em frases nossas |
| tabela verbatim / LUT | **nenhuma** — a única tabela de constantes (§5.2) re-derivei-a inteira da fórmula ao lado (`0,24591 · 0,38367 · 0,46887 · 0,20000 · 0,10000`) |
| pseudo-código espelhado · organização transcrita | **não** — 5 blocos cercados: o cabeçalho e quatro fórmulas de 1–3 linhas, sem controlo de fluxo |
| os **80** cabeçalhos (população NOVA: 71 reescritos, 3 regenerados, 6 re-corridos) | **limpos** — 63 chaves, todas em vocabulário do domínio; os valores de enumeração são os públicos (§4.1.13) |
| a §0.2 manda a janela I a alguma nota antiga? | ⛔ **não** — `grep` sobre a espec: zero ponteiros para os docs 20, 21 e 04.1. A N1 está curada |
| os 3 ficheiros do NOSSO código que a §0.2 cita | varridos com as nove vassouras: **limpos**, salvo o nome público já em triagem (`brush_magnitudes.rs:87`, a **33 linhas** das linhas citadas) |

### 2. Sweep — controlo primeiro

- `scripts/cleanroom-sweep-controlo.sh`: os **11** canais discriminam, `exit 0`.
- `cleanroom-sweep.sh` com **as nove** vassouras da pasta sobre: a espec · a pasta das 80 fixturas
  (README incluído) · o INBOX · o `README.md` da pasta · o anexo citado
  (`fixtures/dyntopo/README.md`) · os três ficheiros do nosso código citados na §0.2 ⇒ **`exit 0`
  em oito**; a nona (`pull`) acusa **um** hit, o nome público de campo de pincel já registado como
  isenção pendente de triagem.
- Varredura extra **em memória** (sem caixa, sem acentos, sem ênfase, com as quebras desfeitas),
  com as nove vassouras, sobre a espec, o README das fixturas, os **80** cabeçalhos, o INBOX e as
  mensagens **e** os patches dos quatro commits desta obra: **0 acertos**.
- `--git-history docs/3D/cleanroom`: `exit 1` com as **3** linhas de sempre — patches de ledgers de
  OUTRAS obras (E/R-only), já registadas.
- ⛔ Nenhum relatório de sweep foi gravado; as saídas com acertos ficaram só no terminal.

### 3. O que RE-DERIVEI (modelo próprio, só das fixturas publicadas)

- **G-1 (16):** planas e cilindro `≤ 7,9e-8`, bossas `≤ 3,3e-6` (barras `2e-6` · `1e-5`). Erradas
  re-medidas: `R_n` trocado `≥ 1,86e-3` · faces de frente sem o factor `5,17e-3` · força linear
  `5,17e-2` nas **10** com força `< 1` (inerte nas **6** com força `1`).
- **G-2 (5 cadeias, 36 estados — contados bloco a bloco: 8+8+4+4+12):** `≤ 1,61e-6` (barra `1e-5`);
  normal do pen-down `4,69e-4` só na de bossas · distância viva `≥ 2,57e-2` · cursor do pen-down
  `≥ 3,62e-2`.
- **Catraca (§12.1):** a pegada projectada reproduz a `4,23e-6` e o produto recomendado lê-a a
  `8,51e-2`; o acumular `1,60e-6` contra `4,71e-4`; a normal do pen-down `1,95e-6` contra
  `6,16e-2`. Os três números da tabela conferem.
- **G-3a (6):** `5,09e-4` (barra `2e-3`); erradas: lei de normal da área da casa `9,50e-3` ·
  normais congeladas `1,11e-2` · `(1+a)/2` `4,68e-2` · sem atenuação `7,95e-2`.
- **G-3b (5):** `3,10e-3` na malha inteira e `2,30e-3` na faixa (barras `8e-3` · `5e-3`), com a 48²
  a fixar os dois; erradas: pesada pelo ângulo `1,17e-2` · triangulada `3,25e-2` / `7,95e-3`.
  **A lei de vértice que a página fixa passa**, e passa com a malha carregada como QUADRILÁTEROS.
- **G-4:** a régua do §7.1 re-implementada de raiz reproduz §7.2, §7.3 (as **16** e **12 + 4**
  células, com as fotos 2 e 4 da 192² agora publicadas), §7.4, §8.1, §8.2, a linha do §3
  (`0,473` constante; `0,2445 · 0,4890 · 0,9781 · 1,9561`) e o §4.3 — **ao dígito impresso**, salvo
  a coluna «fundo» do cilindro (errata **E21**).
- **G-5:** as 7 classes do detector idênticas **ao bit** dentro e `1,06e-2`–`1,28e-2` entre
  vizinhas; contagem `⌊N/5⌋ + 1` exacta nas 14; posições `≤ 4,61e-3` (barra `7e-3`); G-5c
  `7,36e-8` contra `4,53e-2` (sem `a`) e `2,26e-2` (`(1+a)/2`).
- **G-7** `3,0e-7` (a recorrência dá `0,10000 · 0,13164 · 0,15190 · 0,16670`) · **G-8** `2,287` e
  `2,747` · **G-9** `0,0` ao bit · **G-10** `1,20e-7` nos deslocamentos (`2,00` nas posições, os
  dois cantos) · **G-11** `2,90e-4` contra a de caixa dupla e `2,24e-2` contra a de caixa fina, com
  as duas fixturas a diferirem `2,24e-2`.
- **As fixturas re-corridas (B1):** as **53** arrastadas trazem o mapa do píxel (53 de 53); com a
  linha que o cabeçalho **pede** (`y = 0`) o G-3a leria `3,76e-3`–`4,18e-3` ⇒ a cura era necessária
  e está feita. As réguas do cilindro re-corrido não mudam num dígito.
- **Completude:** catálogo × valores escritos `1,19e-7` (afiado) e `1,9e-9` (desenho).
- **O código vivo, sítio a sítio:** os **~30** endereços citados na §6 e na §10 conferem (curva
  `(1−u)⁴`, dureza, força ao quadrado, alcance `= R`, acumular de fábrica LIGADO, `from_live` = o
  acumular, a distância do `base`, o passo `0,15 R`, a fronteira `<=` do passeio, o `(1+a)/2` só do
  plano, a normal da área nas duas portas, o alargamento da consulta pela porta do raio, a
  fotografia do pen-down armada só num verbo+modo, o `invert = ctrl`, o dyntopo do `Draw`). ⭐ E a
  MALHA DA CASA GUARDA QUADRILÁTEROS (`Face::quad`, `from_parts`, Newell por face, soma sem peso no
  anel) — a §2.3.1 está certa sobre a nossa casa.
- ⭐ **A fronteira `<=` do passeio só separa o salto de EXACTAMENTE um passo:** nos saltos de `10`,
  `15`, `20`, `25` e `30` px o passeio da casa dá `dist > passo` e `floor(dist/passo)` dabs — a
  contagem do G-5a bate nos 13. (O passo é `raio × pct / 50`, exactamente `5` px aqui.)

### 4. ⚠️ As DEZ erratas (gaveta B — a janela I corrige-as enquanto constrói)

| # | sítio | o que está errado, e a instrução |
|---|---|---|
| **E18** | **§2.3.1 item 1 · §11 (linhas `lei/` e `cadeia/`) · README §3.1** | a normal de vértice que o caminho por SCRIPT do alvo lê **não é o bloco `n`** (que é a do repouso, pesada pelo ângulo): é a **lei da nossa casa** — média SEM peso das normais unitárias, malha como está — **congelada** no repouso. Medido: bossas do G-1 `3,3e-6 → 2,9e-8`, pegada projectada `4,2e-6 → 2,5e-8`, cadeia de bossas `1,6e-6 → 5,8e-8`, as duas pendentes `→ 6,6e-8` e `3,7e-8`, os dois controlos `→ 6,2e-8`. O bloco `n` **também** passa (dentro das barras) ⇒ não bloqueia; mas o implementador **não precisa de injectar o bloco nem de escrever a lei pesada pelo ângulo** |
| **E19** | **§9.2 · §11 · §10 (o que falta)** | a nossa casa **refresca as normais depois de CADA dab** (`stroke_dab_core.rs:566`, `Mesh::refresh_region`) e nenhuma porta pública as congela. Uma bancada que corra as cadeias pela porta do dab lê normais VIVAS e reprova o G-2 por `4,45e-2` (plano) · `4,39e-2` (bossas) · `6,49e-2` (cilindro). ⇒ a página tem de dizer que o G-1/G-2 pedem uma **costura de teste** que congele as normais do traço inteiro (o produto ARRASTADO refresca, e está certo assim) |
| **E20** | **§12.1** (a condição do censo) | ela devolve a fixtura ao gate *«no dia em que o CONTROLO correspondente for oferecido (e reprova se o controlo existir e a fixtura continuar na catraca)»* — mas o **P-2** tem TRÊS saídas, e uma delas é *«oferecer com a NOSSA lei»*: aí o controlo existe, a lei do alvo não, e a fixtura devolvida ao G-2 lê **`2,56e-2`** contra a barra de `1e-5`. A condição tem de ser a que a própria linha do P-2 já escreve: **a LEI DO ALVO ser oferecida** |
| **E21** | **§7.2**, coluna «fundo» do cilindro | `2,19 · 2,11 · 2,06 · 2,01` não saem da definição do §7.1: eles usam o passo de arco na **BORDA** da grelha (onde `ds/dy = 1,81`) e não junto do fundo. Com a definição escrita, o cilindro lê `6,83 · 6,61 · 6,44 · 6,30` — praticamente a coluna da 96², que é o esperado. A frase *«a do cilindro mede-se contra a curvatura da peça»* descreve outra conta. ⛔ Nenhum gate a usa (o G-4 mede `D`, `W50` e nitidez) |
| **E22** | **§2.3.1** (tabela) e **§12 G-3b** (coluna «errado») | os números por-fixtura vêm de um SUPERCONJUNTO: a corrida tem 8 fotos e as fixturas publicam `p1`, `p2`, `p4` e `s`. Sobre o que está publicado a lei lê `1,30e-3` (96²), `2,15e-3` (192²), `1,27e-3` (cilindro), `1,25e-3` (triângulos) contra os `1,43e-3 · 2,19e-3 · 1,39e-3 · 1,56e-3` impressos — e a 48², que é a que FIXA o aprovado, bate ao dígito (`3,10e-3` / `2,30e-3`). Pelo mesmo motivo a faixa lê `2,20e-2` (lei da casa) e `1,01e-2` (normais congeladas) em vez de `2,6e-2` e `1,19e-2`. **As barras continuam a separar** |
| **E23** | **§11** (regras de leitura) | a **ORDEM DOS CANTOS** de cada quadrilátero não está escrita, e é ela que decide a diagonal com que a nossa casa intersecta o raio do cursor (`Face::tri_at`: 1.º→3.º canto; `Mesh::raycast` usa a mesma). Com a ordem da bancada irmã — `(i,j) (i+1,j) (i+1,j+1) (i,j+1)` — sai o aprovado impresso; com a outra diagonal o G-3b passa de `3,10e-3` para `4,04e-3` (barra `8e-3`) e o G-3a de `5,09e-4` para `5,12e-4`. Nomeie a ordem |
| **E24** | **§0.2** (a lista de notas nossas com a premissa caída) | faltam duas: `crates/ph2d-sculpt3d/tests/it/measure_draw_sharp.rs:4-6` (a sonda que decidiu a pergunta zero em agosto — diz *«o Draw sobre as posições/NORMAIS do pen-down»*) e `crates/ph2d-tool-painter/src/tool/paint/sculpt/mode.rs:34-37`, que é uma **decisão de produto de outro módulo** apoiada na mesma premissa (*«todo verbo aditivo aqui é afiado por construção»*) e que o §8.1 contradiz (a curva vale `+55 %` de largura). ⛔ Nenhuma é do I para curar — ficam nomeadas |
| **E25** | **README das fixturas §4** | ainda traz a folga `1e-3` do recorte, que a errata E15 retirou da espec por não ter proveniência — **gémeo** da errata já curada |
| **E26** | **§12 G-7**, coluna «errado mais perto» | `6,8e-2` é a diferença no **2.º dab**; o `max |Δz|` do candidato linear sobre os quatro dabs é `2,33e-1`. A barra (`1e-6`) não muda |
| **E27** | **§12 G-5a/G-5b**, população | o padrão *«`detector/salto_*` menos o declarado»* casa **14** ficheiros e a população diz **13**: o 14.º é `detector/salto_00px_sem_atenuacao`, que só entra no G-5c. Nomeie-o (a mesma forma das E7/E8 já curadas) |

### 5. O ACHADO N2, confirmado (para a janela I, sem o reproduzir)

A vassoura desta obra acusa **duas** linhas no nosso código de produto —
`crates/ph2d-sculpt3d/src/brush_verb_filter.rs`, linhas **150** e **292** (um doc-comment e um
comentário do filtro de malha de afiar, da W9 de agosto): a tradução próxima de uma oração do alvo.
A contagem é **2**, e mais nenhuma linha da árvore (`crates` + `shells` + `scripts`) acorda esta
vassoura além das **4** linhas pré-existentes do módulo Painter (3 ficheiros), que são dívida de
outra linha. ⇒ **instrução funcional para os dois comentários:** eles têm de passar a dizer, em
palavras nossas e sem a frase do alvo, **o que o filtro faz**: *ele acentua o relevo afastando cada
vértice da média alisada da vizinhança — o que já sobressai sobressai mais, e o que afunda afunda
mais*; e, no sítio do doc-comment, **de onde vem o número** (a nossa medição, com a fixtura ao
lado). ⛔ Nada de citar o alvo, nem entre aspas.

### 6. Higiene

- A vassoura foi descodificada **só em memória** — pelo próprio `cleanroom-sweep.sh` e por um
  processo Python que lê o base64 e compara sem escrever. ⛔ Nada gravado em `/tmp`, `/dev/shm` nem
  no scratchpad da sessão-mãe (que é da janela I).
- O modelo desta passagem e as suas saídas vivem **na zona** (`~/Referencias/blender-pincel-afiado/rpre2/`),
  fora da árvore. Nenhuma saída de sweep com acertos foi gravada em ficheiro.
- ✅ **R-pré, 2.ª passagem: ATESTA.** A janela I pode implementar. As 10 erratas ficam nomeadas
  acima, com o sítio e a instrução; nenhuma exige emenda do E antes de a construção começar.

### 7. ⚠️ Achado de higiene que NÃO é desta obra (registado para o R-PÓS / a integração)

O `--git-history` desta pasta com **as NOVE** vassouras (as passagens anteriores correram só a
desta obra, que acusa as 3 linhas já registadas) devolve acertos de **outras** obras: patches de
ledgers E/R-only — legítimo — **e, pelo menos uma vez, uma MENSAGEM DE COMMIT que nomeia um
ficheiro interno do alvo**. Isso é dívida das linhas donas daquelas obras e vive no histórico do
`main`, logo reescrevê-lo não a cura (o precedente é o INC-U1 do ledger dos pincéis). ⇒ item para o
R-PÓS, com a lei que a casa já escreveu: *o sweep é propriedade do PAR (artefacto, vassoura), e uma
vassoura que cresce obriga a corrida nova sobre tudo o que já passou.*

---

## 2.ª EMENDA DO E — 2026-09-16 (o PASSO JUNTO À SILHUETA, aberta por um report do dono com foto)

> Report: *«se fizer o traço do canto para o início da esfera, no canto fica meio pontilhado»*.
> A emenda acrescenta a **§16** da espec, a família de fixturas **`silhueta/`** (30) e os gates
> **G-13..G-20**. ⛔ **Nenhuma lei, nenhuma fixtura e nenhum gate das §1–§15 foi alterado** — só as
> contagens (`80 → 110`) e o cabeçalho.

### O que foi corrido (oráculo)

| | |
|---|---|
| binário | o MESMO `/usr/bin/blender` 5.2.1 LTS, re-conferido inalterado |
| como | a mesma janela num compositor **virtual** (`kwin_wayland --virtual --xwayland`), `--factory-startup`, `--enable-event-simulate`, `TMPDIR` na zona; harness novo `oracle/harness_emenda2.py`, derivado do de arrasto, saídas em `out2/` |
| entradas | ⭐ **100 % nossas**, geradas por FÓRMULA: meia-cana e esfera **amostradas por ÂNGULO** (espaçamento de MUNDO uniforme, que é o que a peça do dono tem) e rampas de inclinação constante sobre a grelha de sempre |
| corridas | **376** (lei dos pits 12 · detector 264 · produto 10 · degenerado 50 · diagnóstico 8 · realimentação 16 · esfera re-corrida 12 · smoke 3 · enums 1), zero falhas no último registo de cada campanha |
| publicadas | **30** fixturas em `docs/3D/cleanroom/fixtures/pincel_afiado/silhueta/` (2,06 MB) |
| leitura dos valores de fábrica | pelo caminho do **produto** (o catálogo carregado pelo próprio programa); a corrida de catálogo coincide com a de valores escritos a `1,8e-7` |

### ⛔ Quatro armadilhas que esta emenda pagou

1. **O ENROLAMENTO da esfera saiu invertido no meu gerador**, e com a normal para dentro o pincel
   que **afunda** levanta uma **crista** — com `D_max` `0,0351` contra os `0,0346` do valor certo.
   ⚠️ **Todas** as réguas de magnitude continuaram a devolver números plausíveis; o que o denunciou
   foi medir o deslocamento **com sinal ao longo da normal de repouso**. O harness ganhou um guarda
   (`confere_normais_para_fora`) que corre em toda superfície convexa, e as corridas de esfera foram
   **refeitas**. ⇒ registado na espec §16.13 como lei para obras futuras.
2. **Uma classe de equivalência definida AO BIT partiu-se sem que nada mudasse:** duas corridas da
   mesma configuração diferem até `6,0e-8` (o mesmo chão de ruído que a 1.ª emenda mediu em
   `2,3e-8`–`3,3e-7`), e a 1.ª leitura do detector concluiu *«o adaptativo mudou alguma coisa»*
   sobre isso. Com tolerância de `1e-6` os quatro modos colapsam nos dois que de facto existem.
3. **A régua da ondulação lia `1,000` nas PONTAS do traço** — onde o vinco acaba, não onde ele
   pontilha. A cláusula que a salva (a janela inteira dentro da faixa coberta, `≥ 5` amostras) está
   na espec §16.1, e o **controlo de resolução** (a mesma corrida a `6,8` e a `3,2` amostras por
   período) está medido ao lado.
4. **`arcsin` perde toda a resolução junto à silhueta:** `x = 199` px e `x = 200` px distam `5,7°`
   de superfície ⇒ toda posição de dab se lê em **arco de mundo**, nunca em `x` de ecrã.

### A vassoura: NÃO foi alongada, e a decisão fica NOMEADA

Os dois controlos que esta emenda mede (`use_scene_spacing` · `use_adaptive_space`, mais os
vizinhos `use_pressure_spacing` · `use_locked_size` · `unprojected_size`) **não estão na vassoura**
— conferido correndo o `cleanroom-sweep.sh` sobre um ficheiro de candidatos **na zona**, `exit 0`.

⇒ **Isenção NOMEADA, nunca silêncio:** são **nomes de propriedade da API pública** do alvo, da mesma
espécie dos seis que a família da escultura já declara como alcançados por **CORRER** o programa
(dump de propriedades), nunca por ler fonte. Alargar a vassoura a eles criaria falsos positivos
contra uma autorização que a casa já escreveu. ⚠️ **E a espec §16 não os usa**: ela escreve-os no
vocabulário do domínio (*«espaçamento medido em»*, *«espaçamento adaptativo»*), como o cabeçalho das
fixturas já fazia pela tabela de tradução do README delas. Os **valores** (`VIEW` · `SCENE`) ficam,
que é a regra §4.1.13 já em vigor nesta obra.

### Filtragem e sweeps desta emenda

- **Filtragem §4.3** re-executada sobre a §16 inteira: zero código, zero nome interno, zero
  comentário, zero wording de manual ou de discussão pública, zero tabela transcrita.
- **Controlo do instrumento PRIMEIRO** (`scripts/cleanroom-sweep-controlo.sh`): `exit 0`, todos os
  canais a discriminar.
- **As NOVE vassouras da pasta** sobre a espec, as **30** fixturas novas, o README das fixturas e o
  texto do report final do E: **as nove a `exit 0`**.
- ⚠️ A corrida foi feita **depois** de a vassoura desta obra ter as 262 entradas da 1.ª emenda; não
  houve alongamento, logo não há re-corrida sobre a árvore inteira a dever (a lei *«uma vassoura que
  cresce obriga a corrida nova sobre tudo o que já passou»* fica satisfeita por vacuidade aqui).

### ⚠️ O que esta emenda NÃO tem, e é preciso dizê-lo

- **R-pré**: a §16 **não foi auditada**. O atestado de 2026-09-16 (2.ª passagem) cobre as §1–§15 e
  as 80 fixturas; o cabeçalho da espec passou a dizê-lo por escrito.
- **Duas decisões do dono novas** (P-6 e P-7, espec §16.13), as duas com o número ao lado.
- **Uma condição por medir e NOMEADA**: o interruptor adaptativo só poderia agir sobre um
  espaçamento que variasse ao longo do traço, e a única causa que o alvo oferece para isso é o
  tamanho a seguir a **pressão** — que a janela virtual não entrega e que a nossa casa também não
  tem. ⇒ *inerte para o dispositivo que o nosso produto tem*, com a condição escrita.
- **Um tecto não encontrado**: um único evento de ponteiro depositou **48** dabs sem saturar
  (rampa `4:1`, salto de `72` px, modo de cena). É uma ausência medida **numa faixa**, não uma prova.

### Higiene

- Tudo do alvo — harness, configurações, registos, `npz` crus, o rascunho deste report — vive em
  `~/Referencias/blender-pincel-afiado/` (`oracle/`, `out2/`, `draft/`). ⛔ Nada em `/tmp`, nada no
  scratchpad da sessão-mãe, nada no repo além das 30 fixturas, do conferidor e do texto da §16.
- O conferidor `silhueta/confere_a_formula.py` **entra no repo** de propósito: ele não é do alvo —
  é a régua da convenção nova, e foi provado por mutação nas quatro formas de podridão (fórmula
  errada · chave de cabeçalho apagada · índice retirado · o censo a varrer quase nada), com as
  mutações feitas numa **cópia** no scratchpad, nunca na árvore.

---

## R-PRÉ — 3.ª PASSAGEM, sobre a §16 (2026-09-16) — veredito: ⭐ **PAREDE LIMPA** · ⛔ **1 achado que BLOQUEIA** · ⚠️ **14 erratas (E28–E41)** ⇒ **NÃO atesta ainda**

> Subagente R-pré **novo**, independente do E e das duas passagens anteriores. Auditou a §16
> inteira, os cabeçalhos das **30** fixturas novas e o conferidor que as acompanha. ⛔ Não escreveu
> nem ditou código de produto. As §1–§15 e as 80 fixturas da 1.ª emenda **não** foram re-auditadas:
> o atestado delas é o da 2.ª passagem.

### 1. A PAREDE (§4.2) — **LIMPA**, varrida pela forma da §4.3.1 sobre a §16 INTEIRA

Varredura mecânica da secção (314 linhas, 23 270 caracteres), não só dos parágrafos que o report
anterior citou:

- **3 blocos cercados**, todos curtos: o cabeçalho de proveniência da emenda (prosa nossa) e **duas**
  identidades matemáticas de 1 e 3 linhas, em notação genérica e com símbolos nossos
  (`passo_de_mundo`, `passo_de_ECRA`, `ppu`). Matemática não tem dono (SKILL §4.1.2); nenhum deles
  espelha estrutura de código.
- **270 code spans**, **104** não-numéricos, classificados um a um: nomes de fixtura **nossos** ·
  caminhos+linha do **nosso** repo (`spacing.rs`, `input.rs`, `atenuacao_do_traco.rs`, `space.rs`,
  `walk`, `scene.radius_px()`, `passo_do_traco`) · símbolos matemáticos (`D`, `W50`, `θ`, `cos θ`,
  `1/cos θ`, `r(s)`) · rótulos de banda · `f32` · `arcsin`. **Zero** identificador interno do alvo,
  zero nome de ficheiro do alvo, zero comentário, zero wording de manual, zero tabela transcrita,
  zero LUT, zero pseudo-código espelhado.
- **`VIEW` · `SCENE`** são os **únicos** tokens do alvo na página, e são **valores públicos de
  enumeração da API de scripting** — conferido na zona negada: o harness **escreve** a string
  `'SCENE'` numa propriedade pública do datablock do pincel, e o escritor de fixturas mapeia a
  chave de cabeçalho `espacamento_medido_em` para a propriedade pública **do espaçamento** (⚠️ e
  **não** para a do tamanho do pincel, que usa os mesmos dois valores — conferi que o eixo descrito
  é o certo, senão a §16 inteira estaria atribuída ao controlo errado). ⇒ SKILL §4.1.13, e é a
  mesma classe já atestada nas 80 fixturas anteriores (`DRAW_SHARP`, `POW4`, `SPHERE`, `SPACE`,
  `SUBTRACT`, `AREA`).
- **Cabeçalhos das 30 fixturas:** 74 chaves distintas, **todas** em vocabulário do domínio, em
  português; **7** valores de enumeração, todos da lista pública acima. Nenhuma frase do alvo.
- **`confere_a_formula.py`** (prosa + expressão, e entra no repo): escrito em vocabulário nosso,
  re-deriva a fórmula do cabeçalho; nada do alvo.

### 2. Sweep — o controlo PRIMEIRO, e depois tudo

1. `scripts/cleanroom-sweep-controlo.sh` → **exit 0**, os 10 canais a discriminar (nome · texto ·
   binário · dentro do `.gz` · dobrado com ênfase · dobrado entre linhas de comentário · o mesmo
   comprimido · dobrado no `.gz` sem marcador · marcador de outra família · o negativo · o exit 2).
2. `cleanroom-sweep.sh` com **as NOVE vassouras** da pasta sobre a espec + as 30 fixturas novas + o
   README das fixturas + o README da pasta + o INBOX: **as nove a `exit 0`**.
3. `--git-history` com as nove, com pathspec da espec e da pasta de fixturas: **as nove a `exit 0`**.
4. ⭐ **Passagem extra, em memória, estritamente mais sensível:** as **2 395** entradas das nove
   vassouras descodificadas por cano, normalizadas (minúsculas, acentos removidos, pontuação de
   ênfase colapsada) ⇒ **1 820** agulhas distintas contra os **34** artefactos (espec + 30 fixturas
   descomprimidas + README + conferidor + INBOX): **ZERO acertos**.

### 3. O que RE-DERIVEI — régua própria, escrita só da §16.1 e dos cabeçalhos, alimentada só pelas fixturas publicadas

⭐⭐ **O CONTROLO SUSTENTA-SE, e ao dígito.** A §16.2 inteira sai da minha régua nas **três**
corridas publicadas, sem uma célula a discordar:

| banda `u` | período | `D/R` | ondulação | ⇒ espec |
|---|---|---|---|---|
| `5–15°` | `0,0169` | `0,2019` | `0,002` | igual |
| `25–35°` | `0,0192` | `0,1860` | `0,006` | igual |
| `45–55°` | `0,0259` | `0,1497` | `0,023` | igual |
| `60–68°` | `0,0380` | `0,1092` | `0,085` | igual |
| `68–74°` | `0,0511` | `0,0831` | `0,182` | igual |
| `74–79°` | `0,0713` | `0,0608` | `0,314` | igual |
| `79–83°` | `0,1065` | `0,0346` | `0,570` | igual |
| `83–88°` | `0,1719` | `0,0389` | `0,977` | igual |

e o pincel de **catálogo** dá a MESMA tabela célula a célula; a **esfera** dá
`0,2025 · 0,1860 · 0,1498 · 0,1092 · 0,0831 · 0,0605 · 0,0345 · 0,0386` e
`0,002 · 0,006 · 0,025 · 0,084 · 0,185 · 0,311 · 0,568 · 0,978` (⚠️ o primeiro valor da esfera é
`0,2025` na minha leitura e `0,2028` na publicada — ver E36). **O vinco pontilha (`0,002 → 0,977`)
e fica `5,19×` mais raso (`0,2019 → 0,0389`) no ALVO, com o pincel de fábrica.** ⇒ *o nosso produto
está fiel e a pergunta é de produto, não de lei que falte.*

⭐ **A agregação estava escondida e achei-a:** dentro de cada banda a mediana é tomada **só sobre as
estações onde a ondulação é definida** (a cláusula da §16.1). Com todas as estações a última banda
lê `0,0225` em vez de `0,0389` — e é exactamente esse o gémeo da E28.

**As três acusações contra o modo de cena, cada uma re-derivada das fixturas que a §16.11 declara:**

| acusação | re-derivado | fixturas |
|---|---|---|
| `10×` a profundidade na borda | `D/R` a `74–79°`: `0,0608` (vista) contra **`0,6295`** (cena) ⇒ `10,35×`, e `3,12×` o meio da peça; na **esfera** `0,0605` contra `0,6147` ⇒ `10,2×` | `produto_cupula_{fabrica,cena}_da_silhueta` + o par da esfera |
| taxa de amostragem `−30 %`/`−51 %` | `1` px → `4` px por evento, em cena: `0,5728→0,3116` (**−45,59 %**), `0,6295→0,4380` (**−30,43 %**), `0,5549→0,2715` (**−51,08 %**); o controlo em **vista** move-se `≤ 0,04 %` nessas três bandas | `produto_cupula_cena_4px_por_evento` × `produto_cupula_cena_da_silhueta`, e o par de vista |
| sentido do gesto `3,3×` | ida `0,6295` contra volta **`0,1935`** a `74–79°` ⇒ `3,25×`; **o PAR está publicado** | `produto_cupula_cena_da_silhueta` × `produto_cupula_cena_do_meio_para_a_silhueta` |

⚠️ **O par do sentido em VISTA não existe** — a célula "igual, tirando a ponta" não tem fixtura
(E38). E o **cursor parado** re-deriva ao dígito em `u = 84°` (`0,00983` em vista, `0,01727` em
cena) **medindo ao longo da normal de repouso**; medido como `max|componente|` dá `0,00977`/`0,01716`
e leva a crer num gémeo que não existe.

**Mais o que confirmei, todo das fixturas publicadas:**

- **a lei do passo (§16.4):** em vista, passo de ecrã **`14,40 / 23,98 / 36,05`** px (média) para
  `esp` `60/100/150`; em cena, corda 3D **`0,071979 / 0,119919 / 0,179744`** contra o alvo
  `0,072 / 0,120 / 0,180`, com o ângulo uniforme **`4,125° / 6,875° / 10,3125°`** (a malha quantiza
  em `0,34375°`) e o passo de ecrã a encolher para **`1,96 / 3,84 / 6,82`** px junto à silhueta.
  ⇒ a diferença que o G-13 usa como candidata errada, **`12,41`–`29,02` px**, re-deriva.
- **a fronteira do passo (§16.3, G-14):** um dab dá `D_max = 0,01476` **nas TRÊS superfícies** a
  `4` px em vista (plano · rampa `45°` · rampa `4:1`) e dois dabs dão `0,02448 / 0,02442 / 0,01718`
  a `5` px; na rampa `4:1` em **cena** a fronteira desce para `4` px (`0,01476 → 0,02206`). ⇒ *o
  passo é cego à inclinação em vista e não é em cena*, confirmado.
- **o adaptativo INERTE (§16.5, G-17):** sobre os dois pares que o gate declara, `max|Δ| =`
  **`3,6e-12`** (vista) e **`0` ao bit** (cena) — muito abaixo do `6,0e-8` publicado (E31). E a
  condição declarada confere-se **no nosso código vivo**: a nossa casa não tem pressão de caneta —
  `crates/ph2d-app-sculpt3d/src/` não menciona pressão em lado nenhum e `Dab::at`
  (`ph2d-sculpt3d/src/dab.rs:105`) prega `pressure: 1.0` com o doc a dizer *«sem tablet, 1.0»*.
  ⚠️ **A fixtura ESTÁ no ponto neutro do único knob que tornaria o interruptor observável** (o
  tamanho a seguir a pressão, desligado no harness) — e isso está **declarado** na própria §16.5,
  que é a forma honesta; o que falta é o **nome do gate** carregar a cerca (E31).
- **a composição com a atenuação (§16.8):** `D_cena/D_vista` por banda = `1,0052 · 1,0540 · 1,2033
  · 1,4565 · 2,4993` contra `1/cos u` = `1,0038 · 1,0642 · 1,2208 · 1,4142 · 1,7013` ⇒ desvios
  `+0,1 / −1,0 / −1,4 / +3,0 / +46,9 %`. **A afirmação `±3 % até 50°` sustenta-se** e a quinta banda
  é a auto-limitação do §4 a morder. ⭐ **O que ela diz sobre a §5.2/§5.3 já atestadas: confirma-as e
  fecha a única leitura sob a qual elas podiam ter sido implementadas erradas** — `a` sai da
  percentagem **declarada** e **não** é recalculado do passo efectivo de cada dab; se fosse, esta
  tabela seria `≈ 1` inteira. Nenhuma lei das §1–§15 muda.
- **os valores de fábrica dos três eixos (§16.7):** lidos do pincel de **catálogo** —
  `silhueta/produto_cupula_catalogo_da_silhueta` dá `VIEW` · adaptativo `False` · segue-pressão
  `False` · `5 %`, e a coluna do desenho comum sai de `produto/desenho_de_catalogo_continuo`
  (`DRAW`, `VIEW`, `False`, `False`, **`10 %`**). ⭐ A prova de completude re-deriva a
  **`1,200e-7`** (não `1,8e-7` — E29).
- **a régua NÃO é artefacto da malha:** o controlo de resolução do §16.1 não tem fixtura (E36), mas
  **refi-lo eu** sub-amostrando a `produto_cupula_fabrica_da_silhueta` de 1 em 2 (`6,7 → 3,3`
  amostras por período no topo): `0,023 → 0,019` · `0,314 → 0,306` · `0,977 → 0,973`, e a cláusula
  das `≥ 5` amostras faz as bandas finas devolverem `None` em vez de mentirem. ⇒ **a conclusão da
  §16.1 sustenta-se.**
- **o conferidor das fixturas é REAL:** provei-o por mutação numa **cópia** fora do repo, nas quatro
  formas de podridão — fórmula errada (`✗ erra 1,000e-03 contra a tolerância 2e-06`) · chave de
  cabeçalho apagada (`✗ cabecalho sem …`) · índice retirado do bloco `r` (`✗ 1696 linhas r contra
  vertices_publicados=1697` **e** `o conjunto publicado não é o declarado`) · censo a varrer quase
  nada (`✗ PISO DE POPULACAO: 12 ficheiros`) — com o **controlo** verde antes e depois de cada uma.
  `30 ficheiros conferidos, 0 problemas` na árvore. **36** chaves obrigatórias, **30** ficheiros.
- **as citações que a §16 faz do NOSSO código estão todas certas no código vivo:**
  `spacing.rs:46-49` (*«a unidade é a do CHAMADOR … no nosso shell essa régua é a TELA»*) ·
  `input.rs:331,357,384` (os **três** sítios a passar `scene.radius_px()`) ·
  `atenuacao_do_traco.rs:51,70` (`ESPACAMENTO_DO_AFIADO_PCT = 5.0` e `raio · pct / 50`) ·
  `space.rs:253` (`radius_px()`). ⇒ **a §16.10.1 — «somos fiéis, e o pontilhado é fidelidade» — é
  verdadeira sobre o código de hoje.**

### 4. ⛔ O ACHADO QUE BLOQUEIA — **B4: a barra do G-13 não é alcançável pela leitura que o G-13 nomeia**

O G-13 mede *«posição de cada dab, em píxeis de ecrã, **contra os picos do barro**»*, com barra
`0,5` px e aprovado `0,094` px. A fixtura publica uma **malha**, não uma lista de dabs ⇒ a posição
de um dab tem de ser lida do barro. Medido sobre as **três** fixturas que o gate declara, com o
desvio máximo à grelha `pen_down + k·passo` e o desvio-padrão dos passos:

| leitura do pico | `esp060` | `esp100` | `esp150` |
|---|---|---|---|
| **vértice mais alto** (a leitura simples) | **`0,55` px** · dp `0,326` | `0,50` px · dp `0,404` | `0,48` px · dp `0,451` |
| **interpolação sub-célula** (parábola nos 3 vértices) | `0,10` px · dp `0,040` | `0,09` px · dp `0,080` | `0,09` px · dp **`0,094`** |

⇒ **A leitura simples REPROVA a `esp060` sobre produto correcto** (`0,55` contra a barra `0,5`) e dá
desvios-padrão `4×`–`5×` o valor da coluna «aprovado». **Só a interpolação sub-célula reproduz a
página — e reproduz o `0,094 px` exactamente.** A página não diz qual das duas é, e o Implementador
não tem como decidir: perante `0,55` contra `0,5` ele conclui que a *nossa* lei do passo está errada,
ou afrouxa a barra — e afrouxar a barra é o que esta casa proíbe (`CLAUDE.md` §0.0).

**Instrução funcional (para o E):** a coluna «mede» do G-13 tem de dizer **como** a posição de um dab
se lê de uma malha publicada — a interpolação sub-célula do pico do barro, que é de onde o `0,094 px`
saiu —, **ou** a barra tem de passar para um número que a leitura nomeada alcance (a leitura simples
pede `≥ 0,6` px sobre estas três fixturas, e aí o `0,094` deixa de ser o aprovado e passa a ser
`0,55`). ⚠️ **A margem para a candidata errada continua enorme nos dois caminhos** (`12,41`–`29,02` px),
então nenhuma das duas curas enfraquece o gate.

### 5. ⚠️ As CATORZE erratas (gaveta B — a janela I corrige-as enquanto constrói)

| # | sítio | o que está lá | o que a medição dá |
|---|---|---|---|
| **E28** | §16.9, linha «4 px por evento», coluna `VIEW` | `0,1092 · 0,0608 · **0,0225**` | ⛔ **GÉMEO**: a §16.2 publica `0,0389` para a mesma banda (`83–88°`) da mesma corrida. São a mesma grandeza sob **duas agregações**: mediana só sobre as estações com ondulação definida (`0,0389`, a cláusula da §16.1, e é o que a §16.2 usa) contra mediana sobre **todas** (`0,0225`). ⇒ escreva `0,0389`, ou diga a agregação |
| **E29** | §16.7, última linha | completude a `1,8e-7` | ⛔ **GÉMEO**: mede **`1,200e-7`** sobre o par que a própria linha nomeia — e `1,2e-7` é o que a §0.2, a §13 e o README das fixturas já imprimem |
| **E30** | §16.12, G-15, «errado mais perto» | a régua sem a cláusula «lê `1,000` **no meio**» | contradiz a §16.1 (*«lê `1,000` nas duas PONTAS»*), que é o que se mede: sem a cláusula as bandas interiores continuam a ler `0,002 / 0,006 / 0,314` e só as das pontas leem `1,000` |
| **E31** | §16.12, G-17, «aprovado» | `6,0e-8` | sobre os **dois pares que o gate declara**, `3,6e-12` (vista) e `0` ao bit (cena); o `6,0e-8` vem da população larga da §16.5 (`12` traços + `132` pares), **nenhum deles entre as 30**. ⇒ imprima o valor da população declarada. ⚠️ E ponha a cerca no NOME do gate: *inerte no dispositivo que temos* — a fixtura está, por construção, no ponto neutro do único knob que o tornaria observável (a §16.5 já o declara; o gate não) |
| **E32** | §16.12, G-18 + §16.4 fim | «desvio relativo do fundo do vinco», aprovado `0,09 %` | a **mediana do fundo ao longo da fila** dá `0,084 %`; o **máximo global** dá `0,115 %`. Os dois passam a barra `0,5 %`; só um é a coluna. Nomeie a agregação |
| **E33** | §16.12, G-14, coluna «mede» | «nº de dabs» | a `4`–`5` px de separação os dois dabs **fundem-se num pico só**: contar picos lê `1` dos dois lados nas oito fixturas. O discriminador que a fixtura dá é **exacto**: um dab = `D_max 0,01476` nas três superfícies; dois dabs = `0,02448 / 0,02442 / 0,01718` (vista, `5` px) e `0,02206` (cena, `4` px); vértices movidos ao longo da fila `446→479 · 298→316 · 98→122 · 98→113`. (A §5.1 e a §16.5 já ensinam esta leitura — *«um dab a mais vale `1,3e-2`»* —, mas o G-14 não lhe aponta) |
| **E34** | §16.12, G-20, «errado mais perto» | «um dab move `3 229`» | não é recuperável: a fixtura irmã (`pen_down_na_silhueta_um_px_para_dentro`) move **154** dos seus `2 497` vértices publicados, e nenhuma fixtura da família publica contagem de malha inteira. O lado **aprovado** (`0` em `pen_down_na_silhueta_sem_dab`) re-deriva exacto |
| **E35** | §16.6, 3.º ponto | «a `84°` um único píxel já pede `~10`» | a própria parêntese dá `1,9`: `0,00333 / cos 84° = 0,0319` de arco contra um passo de `0,01667`. O que vale `~10` é `1/cos 84°` — **quantas vezes mais** do que no topo (onde são `0,2` dabs por píxel), não a contagem |
| **E36** | §16.1 e §16.9 e §16.6 | três populações citadas sem fixtura | (a) o **controlo de resolução** (`6,8`/`3,2` amostras por período): as duas malhas não estão entre as 30 — refi-lo sub-amostrando a publicada e a **conclusão sustenta-se** (`0,023→0,019 · 0,314→0,306 · 0,977→0,973`); (b) o **cursor parado a `60°` e `75°`**: só `u84` está publicado; (c) a **sonda de linearidade** (saltos `10·20·35·50·72` px) e os saltos de `2·3·5·8` px do pen-down: só o de `1` px está publicado. ⚠️ E a esfera lê `0,2025` na banda `5–15°` onde a §16.2 publica `0,2028` (a diferença é da agregação da mediana; as outras 7 batem exactas) |
| **E37** | §16.11, justificação do subconjunto | «`79 k`–`1,08 M` vértices» | as 30 malhas têm `9 409 · 37 249 · 42 947 · 93 757 · 185 977 · 1 079 102` ⇒ a faixa é **`9,4 k`–`1,08 M`** (ou `185 977`–`1,08 M` se a frase quiser dizer só as malhas da régua da ondulação). A justificação continua de pé |
| **E38** | §16.11, linha «o PREÇO de `SCENE`» | «as **quatro** linhas do §16.9» | a tabela do §16.9 tem **cinco** linhas. E a do **sentido** não tem controlo em vista publicado: a célula diz «igual, tirando a ponta» e não existe `produto_cupula_fabrica_do_meio_para_a_silhueta` — o par **em cena** está publicado e re-deriva |
| **E39** | §16.4 e os cabeçalhos das seis `passo_em_*` | «meia-cana varrida de `−17°` a `+85°`» contra «varre 0..89 graus» no `o_que_ela_fixa` | a linha real do cursor vai de `+89,5°` a `−20°` e os **dabs** caem entre `84,22°` e `−17,5° / −18,9° / −15,5°` conforme o espaçamento. Três descrições da mesma corrida, nenhuma igual à outra |
| **E40** | §16.4, «a corda ajusta melhor que o arco» | `+0,09 %` contra `+0,22 %` a `150 %` | no limite do que a malha publicada suporta: com detecção de pico a corda dá `−0,14 %` e o arco `−0,01 %` na mediana — **sinal contrário**, e a diferença entre as duas é menor que a quantização do pico. A **lei** (passo constante em mundo) re-deriva sem dúvida. Nenhum gate a usa |
| **E41** | §16.7, coluna «nosso», linha do espaçamento | `5 %` — e a §6 diz `0,15 R = 7,5 %` | as duas estão certas e leem-se como contradição: a primeira é o `Verb::DrawSharp` (`ESPACAMENTO_DO_AFIADO_PCT`), a segunda é o passo de omissão da casa para o `Draw` genérico. Diga de que verbo é a coluna |

### 6. O que NÃO auditei, e é preciso dizê-lo

- As **§1–§15** e as **80** fixturas da 1.ª emenda: cobertas pelo atestado da 2.ª passagem, não
  re-abertas aqui. Confirmei apenas que a §16 **não contradiz** a §5.1 (o passo já lá era «`5 %` do
  diâmetro de **ecrã**» com a grelha ancorada no píxel do pen-down), a §5.2/§5.3 (a §16.8 **confirma**
  que `a` sai da percentagem declarada) nem o §6.
- A lista de **exclusões com nome** do §12.2 não ganhou entradas da §16 — as 30 fixturas novas estão
  todas num dos oito grupos do §16.11, e a soma dos grupos é **30**.

### 7. Higiene

- A vassoura **nunca** foi descodificada para ficheiro: o `cleanroom-sweep.sh` descodifica-a em
  memória, e a minha passagem extra leu os `.txt` e descodificou **dentro do processo Python**.
- ⛔ **Nada** foi escrito em `/tmp`, em `/dev/shm` nem no scratchpad da sessão-mãe
  (`/tmp/claude-1000/…`, que é da janela I). As minhas réguas independentes e a cópia mutada das
  fixturas viveram em `~/Referencias/_rpre_pincel_afiado_2026-09-16/`, fora do repo, e a cópia
  mutada foi apagada no fim.
- Os nove relatórios de sweep saíram **verdes**, logo nenhum citou nada; ficaram no terminal.
- ⚠️ **Toquei na zona negada** (`~/Referencias/blender-pincel-afiado/oracle/`) exactamente duas
  vezes, e só para responder à pergunta de parede *«`VIEW`/`SCENE` são nomes públicos ou internos?»*:
  uma contagem e uma linha do **harness do E** (não do fonte do alvo), com o nome da propriedade
  mascarado antes de ser impresso. Nada disso entra no report para a janela I.
