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

(por preencher: R-PRÉ e R-PÓS)

## Incidentes

(nenhum)
