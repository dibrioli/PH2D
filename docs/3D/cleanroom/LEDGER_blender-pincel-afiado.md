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

`docs/3D/cleanroom/VASSOURA_blender-pincel-afiado.txt` — **229** entradas em base64, uma por
linha: identificadores internos (funções, tipos, sinalizadores, campos) e prosa do alvo —
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

(nenhum)
