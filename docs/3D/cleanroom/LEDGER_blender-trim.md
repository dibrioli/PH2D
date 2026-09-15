# LEDGER de proveniência — clean-room do gesto de CORTE (alvo `blender-trim`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-trim.md` (append cego).

⏱️ **Aberto em 2026-09-15, ANTES da primeira leitura de CONTEÚDO do fonte do gesto.** Antes desta
abertura o E fez apenas: (a) metadados de **pacote e ligação dinâmica** desta máquina (`pacman -Qi`,
`ldd`, `/usr/share/licenses`) — que é o primeiro acto obrigatório da triagem (§2: *ler a licença
real*); (b) uma **listagem de NOMES** de ficheiros do directório do modo escultura no checkout
(para localizar a obra — nenhum conteúdo aberto); (c) a leitura de artefactos **NOSSOS** do repo
(a SKILL, o `cleanroom-sweep.sh`, o precedente `LEDGER_blender-boundary.md` e a sua espec).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — a família de gestos de **corte** do Sculpt Mode (as variantes de caixa, laço, linha e polilinha) e o que elas invocam que decide comportamento (construção do volume varrido, motor de booleana, orientação, profundidade, máscara/simetria/multirresolução, recusas) |
| Versão / commit do fonte lido | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` — checkout **esparso e grafted**, o MESMO que as obras `blender-cloth`, `-pose`, `-boundary`, `-pull`, `-unblocked` leram |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D |
| Zona fora da árvore (notas, rascunhos, oráculo) | `~/Referencias/blender-trim/` (`notes/` · `draft/` · `oracle/` · `fixtures/`) |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença do ALVO | **GPL-2.0-or-later** — `COPYING` remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991) |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS**, pacote `blender 17:5.2.1-2` (build 2026-08-31) |

### A concessão relevante (GPLv2 §0), transcrita do ficheiro do checkout

> *"Activities other than copying, distribution and modification are not covered by this
> License; they are outside its scope. The act of running the Program is not restricted,
> and the output from the Program is covered only if its contents constitute a work based
> on the Program (independent of having been made by running the Program)."* (§0)

⇒ Ler, correr e instrumentar **em privado** é licenciado. A **saída** (posições de vértices de
malhas NOSSAS) é dado. Nenhum acto deste ledger envolve distribuição. Não é AGPL.

---

## §2 — Triagem: a escada de portas — **DUAS METADES, e elas dão degraus DIFERENTES**

⚠️ A unidade da triagem é o **ARTEFACTO/biblioteca**, nunca o nome do projecto (lição do `mypaint`:
app GPL, motor ISC). O pacote instalado declara uma **mistura**: `Apache-2.0 BSD-2-Clause
BSD-3-Clause GPL-2.0-or-later GPL-3.0-or-later LGPL-2.1-or-later MIT MPL-2.0 Zlib`.

### Metade (a) — o MOTOR DE BOOLEANA DE MALHA: ⭐⭐⭐ **T0 — PORTA ABERTA**

**Medido em 2026-09-15 nesta máquina, não suposto:**

| medição | comando | resultado |
|---|---|---|
| o alvo **liga-se** a uma biblioteca externa de geometria | `ldd /usr/bin/blender` | `libmanifold.so.3 => /usr/lib/libmanifold.so.3` ✅ |
| a licença **dessa biblioteca** | `pacman -Qi manifold` | **`Apache-2.0`** · versão `3.4.1-1.1` · `Necessário para: blender` |
| o ficheiro do alvo que a usa | adaptador malha↔biblioteca, `1 919` linhas | usa a estrutura de interoperação **pública** dela |
| ⭐ qual solucionador o gesto de corte usa **por omissão** | o valor de omissão do selector, lido no fonte | **a biblioteca externa** (não o solucionador próprio do alvo) |

⇒ **O algoritmo que de facto faz o corte é Apache-2.0 e não pertence ao alvo.** O que é GPL é só
(i) o adaptador entre a malha do alvo e a biblioteca — a parte que teríamos de escrever de qualquer
modo — e (ii) a lei do gesto, que é a metade (b).

**Caça T1 (irmão permissivo em Rust) — feita, com PROVENIÊNCIA validada:**

| candidato (nome público) | licença | proveniência | veredito |
|---|---|---|---|
| `manifold-csg` | **MIT OR Apache-2.0** | ligações seguras para a biblioteca C++ Apache-2.0 | ✅ elegível |
| `manifold3d` | idem | fachada que re-exporta a anterior | ✅ elegível |
| ⛔ *(uma 4.ª rota, só-git)* | declarava Apache-2.0 **na página do repositório** | porte puro-Rust da biblioteca Apache-2.0 | ⛔⛔ **RETIRADA. Não existe no registo de pacotes** (busca por nome devolve zero) e **eu nunca inspeccionei o artefacto** — a licença que eu citei foi lida na *página*, não num ficheiro que eu tenha aberto. ⚠️ A linha original chamava-lhe *«o melhor encaixe»*: julguei o código pela descrição e não olhei a **porta de distribuição**. ⇒ ela **não é candidata** e o nome sai deste ledger, porque nomear algo que o próximo auditor não consegue buscar é pior que não a listar |
| `manifold-rust` | **Apache-2.0** | porte puro-Rust **declarado** da mesma biblioteca (com plano de porte e registo de divergências no pacote; o porte é discutido no rastreador da própria origem) | ⭐ **✅ a recomendação da emenda §1.5.9** — e o risco dela é a IDADE, medido |

⚠️ **A validação de proveniência era o passo que não se podia saltar** («um porte permissivo que
descende do alvo é lavagem alheia»): os quatro descendem da **biblioteca Apache-2.0**, ⛔ nenhum do
alvo GPL. A biblioteca é anterior e independente do alvo — o alvo é que passou a ligá-la.

### Metade (b) — a LEI DO GESTO: **T2** (é o degrau desta obra)

| degrau | veredito | por quê |
|---|---|---|
| T0 | ⛔ | a lei do gesto só existe neste alvo, sob GPL |
| T0½ | ⛔ | os ficheiros do gesto levam cabeçalho `GPL-2.0-or-later`; nenhum é MPL/LGPL |
| T1 (a) autores | ⛔ | não há paper nem código de referência paralelo do gesto |
| T1 (b) versões antigas | ⛔ | o gesto nasceu já sob GPL |
| T1 (c) reimplementações | ⛔ | busca de 2026-09-15: o que existe de permissivo é **motor de booleana** (metade a), ⛔ não a lei do gesto (como o desenho de ecrã vira volume, a profundidade, os modos, as recusas) |
| **T2** | ✅ | copyleft com fonte — o pipeline da SKILL |

⇒ ⭐ **A obra parte-se em duas e só metade paga clean-room.** O motor **porta-se** (T0); a lei do
gesto **especifica-se** (T2, `SPEC_trim_gesture.md`).

---

## Patente (§8.1) — checkpoint incondicional (2026-09-15)

- **Termos:** `patent mesh boolean CSG sculpting trim tool swept volume from screen lasso 3D
  modeling` · `"11238649" patent hybrid modeling geometric facets claims boolean`.
- **Achados e veredito, um a um:**

| patente | assunto | estado | alcança-nos? |
|---|---|---|---|
| **US 6 724 393** | esculpir modelos digitais com volume varrido ao longo de uma curva | prioridade ~2001 ⇒ **EXPIRADA** | ⛔ não (e expirada = espec grátis) |
| **US 7 031 790** · **US 7 034 818** | esculpir sólidos com corpos-folha · converter dados de alcance em modelos 3D | mesma era, **expiradas** | ⛔ não |
| **US 11 238 649** | booleanas/aparos sobre «facetas de renderização» num sistema de modelação híbrida | concedida 2022, **`Expired — Fee Related`** (anuidades não pagas) | ⛔ **não** — a reivindicação independente exige **contexto específico** (facetas de renderização, um «atributo de tipo de reserva», um método nomeado de subdivisão de triângulo); ⛔ ela **não** cobre booleana de malha em geral |

- **Prior art esmagadora e pública** para o que a espec descreve: booleana sobre poliedros é
  literatura dos anos 1970–80, e a própria biblioteca da metade (a) distribui-se sob Apache-2.0.
- ⇒ **Veredito: nenhuma patente VIVA alcança o método.** Não há gatilho de §8.1 para o Enio.

---

## Cobertura da travessia (§3.E) — 2026-09-15

⚠️ **Honesto sobre o que é INTEGRAL e o que é PARCIAL.**

| ficheiro (área do assunto) | linhas | lido |
|---|---|---|
| o gesto de corte — o assunto | 980 | ✅ **INTEGRAL** |
| a maquinaria partilhada de gestos (cabeçalho) | 160 | ✅ **INTEGRAL** |
| a maquinaria partilhada de gestos (corpo) | 474 | ✅ **INTEGRAL** |
| a interface pública do módulo de booleana | ~110 | ✅ **INTEGRAL** |
| o gesto IRMÃO de projecção (o que o manual recomenda para alta resolução) | 273 | ⚠️ **~70 %** — lido o bastante para o contraste da espec §12 |
| o adaptador para a biblioteca externa | 1 919 | ⚠️ **PARCIAL** — includes, uso da interface dela e caminho de erro; ⛔ não integral, e **não precisa de ser**: esta parte é **T0** e porta-se, não se especifica |
| os conjuntos de faces — as 3 funções que o gesto chama | — | ✅ integral **nas três** |
| a definição das 4 ferramentas na barra | — | ✅ integral |

**História (§4.1.12):** ⚠️ **o checkout é `--depth 1` e NÃO tem história** (`git log` devolve 1
commit). ⇒ a sabedoria dos autores entrou por **fonte pública de fora do código**: o **manual
público** (colhido e guardado em `~/Referencias/blender-trim/notes/`), que deu a nota dos 100 k
vértices, a recomendação do gesto alternativo, o fluxo «cortar depois remalhar» e as descrições dos
três solucionadores. ⏳ **Dívida nomeada:** o rastreador público e as mensagens de commit **não**
foram percorridos.

---

## Higiene — um deslize REGISTADO (§6 manda descrever, nunca esconder)

⚠️ O manual público foi descarregado para `/tmp/trim_manual.html` antes de eu reparar que a SKILL
proíbe material do alvo em `/tmp`. **Movido para `~/Referencias/blender-trim/notes/manual_publico.html`
no mesmo turno.** ⛔ **Não é incidente §6**: é prosa **pública** do alvo, nunca entrou no contexto
da janela-mãe, e nada dela está no repo. Fica registado porque a regra é registar.

---

## O ORÁCULO (§5)

| | |
|---|---|
| binário | `/usr/bin/blender` 5.2.1 LTS, **com GUI** — ⚠️ em modo `--background` a região 3D não tem dados e o gesto não corre; o arnês trabalha dentro de um temporizador, que só dispara depois do primeiro desenho |
| arnês | `~/Referencias/blender-trim/oracle/trim_oracle.py` — **NOSSO**, dirige só a API pública |
| driver | `run_each.sh` — **um processo por caso, com tecto de tempo** |
| malhas de ENTRADA | ⭐ **NOSSAS** — `ph2d_mesh::shapes` e `::shapes_open`, exportadas em OBJ pelo gerador do repo. ⛔ **nenhum asset do alvo**, nem como entrada |
| corridas | **56**, com nome único e verificadas por `fixtures/trim/verifica_corridas.py`: 26 (matriz principal) + 15 (malha aberta, profundidade, perspectiva) + 11 (knob morto, laço côncavo, custo, raio do cursor) + 4 (a orientação coagida, §3) — ⚠️ **menos 2 corridas fantasma apagadas** (ver abaixo) |

⚠️⚠️ **Uma armadilha do arnês que custou 13 minutos e quase virou «achado»:** uma edição minha
partiu a indentação do script; o binário abriu a janela, o script morreu no arranque e o processo
**ficou parado para sempre** — que de fora se lê exactamente como *«o solucionador pendurou numa
malha degenerada»*. ⇒ o driver passou a **compilar o arnês antes de cada lote** e a correr **um
processo por caso**. *Um oráculo sem tecto de tempo por caso não distingue «pendurou» de «nunca
arrancou».*

---

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E (alvo `blender-trim`)** despachado pela janela I da `line/sculpt3d` |
| data | 2026-09-15 |
| ⚠️ transcript | **zona contaminada** — a janela I ⛔ não o lê (§3.I) |

## Incidentes

(vazio)

---

## ⛔⛔ ACHADO DE INSTRUMENTO — o sweep era CEGO a toda frase que o parágrafo QUEBRA (2026-09-15)

**Medido com controlo dos dois lados, e curado no mesmo turno.**

O `scripts/cleanroom-sweep.sh` varre com `grep -F`, que casa **dentro de uma linha e nada mais**.
Toda espec deste repo é markdown quebrado a ~100 colunas. ⇒ **uma frase de prosa da vassoura com
mais de ~100 caracteres é inalcançável sempre que cai no sítio errado do parágrafo.**

⛔ **O §6 manda DESCREVER, nunca REPRODUZIR** — a frase de prova não se transcreve aqui. Ela é uma
entrada de prosa da vassoura desta obra, com **30 caracteres**, `sha256` (16 primeiros)
**`127d584938a42ec0`**. O ensaio foi:

| ficheiro de prova | como a frase estava | veredito ANTES da cura |
|---|---|---|
| uma linha só | inteira, numa linha | ✗ **ACUSADO** |
| duas linhas | **partida** por uma quebra de linha no meio | ✅ **«limpo»** |

*Mesma vassoura, mesma frase, mesmo ficheiro — só a quebra de linha muda.*

⚠️ **A primeira redacção desta secção transcrevia a frase**, e foi o **próprio sweep curado** que a
apanhou no ledger. *O instrumento novo cobrou a regra a quem acabara de o consertar.*

⚠️⚠️ **E foi exactamente assim que uma citação traduzida do manual sobreviveu nesta espec:** o
sweep dizia verde, e a frase estava lá, partida entre as linhas 583 e 584. **A frase foi
removida** da espec (o facto ficou, o wording saiu) e o instrumento foi curado.

**A cura:** o sweep passa agora o ficheiro **uma segunda vez com as quebras de linha desfeitas** e
o espaço em branco normalizado. É estritamente **mais sensível** — só pode transformar um verde em
vermelho, que é a direcção certa para um instrumento de parede.

⭐ **Corrido sobre as CINCO especs que já estavam no repo, cada uma contra a sua vassoura: as cinco
continuam VERDES.** ⇒ *a cegueira era real e não tinha sido explorada* — mas era uma questão de
tempo, porque a probabilidade de uma frase traduzida cair sobre uma quebra cresce com o
comprimento dela, e as frases de prosa são as mais compridas da vassoura.

⚠️ **A lição que fica, e que é irmã da de 2026-09-14** (`.gz` opaco ao `strings`): *um sweep verde
prova o que o INSTRUMENTO alcança, nunca o que o texto contém* — e as duas cegueiras achadas até
hoje foram **do alcance**, não dos padrões.

---

## Fechamento

- ✅ **R-PRÉ: ATESTADA em 2026-09-15**, à **2.ª passagem** (1.ª: 9 achados, 2 bloqueantes; 2.ª:
  verde). O atestado está no cabeçalho da espec e o registo na secção «R-PRÉ, 2.ª passagem», no
  fim deste ficheiro. **A janela pode implementar.**
- ⏳ **R-PÓS:** só depois de haver paridade. ⚠️ Herda **duas** notas não-bloqueantes do R-pré (a
  cegueira do `_` na 3.ª passagem do sweep; o cabeçalho de coluna da §11.1) e a dívida de
  **histórico** medida acima, que continua a ser de R-pós e não desta obra.


---

## Vassoura — duas entradas RETIRADAS, e a razão (2026-09-15)

A vassoura fechou com **565** entradas. Duas foram retiradas depois do primeiro sweep completo,
porque eram **subcadeias de nomes PÚBLICOS** que a espec e as fixturas usam legitimamente
(SKILL §4.1.13): o nome interno curto do campo de extrusão é subcadeia da propriedade pública
homónima, e o mesmo para o campo do filtro de faces frontais. `grep -F` casa subcadeias ⇒ as duas
faziam o instrumento **acusar uso lícito**.

⚠️ **Uma entrada de vassoura que dispara sobre uso lícito é pior que uma entrada em falta:** ela
treina quem corre o sweep a ignorar achados. ⇒ a regra que fica: **ao pôr um nome interno na
vassoura, verifique se ele é subcadeia de algum nome público que a obra vai usar** — e, se for,
ponha o nome interno **inteiro e distinguível**, ou deixe-o fora com esta nota ao lado.

---

## ⏳ Dívida PRÉ-EXISTENTE, agora com NÚMERO — o sweep de HISTÓRICO está VERMELHO (2026-09-15)

⛔ **Nada disto é desta obra:** o commit desta linha foi varrido isoladamente contra **as 7
vassouras** — mensagem e patch — e fecha **✅ VERDE**. O que segue é o estado do repo à volta.

O `--git-history` reprova **com todas as 7 vassouras**. Medido, contando só entradas com forma de
**identificador** (≥ 6 caracteres, sem espaços — a prosa fica de fora):

| grandeza | valor |
|---|---|
| identificadores de vassoura distintos que aparecem em **mensagens de commit** | **64** |
| **commits** implicados | **75** |
| ficheiros **rastreados** (árvore viva) com ao menos um | **115** |
| ⚠️ destes, em **código de produto** (`crates/` · `shells/`) | **20** entradas |

⚠️⚠️ **E o número está INFLACIONADO por construção — não o cite sem esta linha ao lado.** Uma parte
é **colisão com o nosso próprio vocabulário**: um nome interno curto do alvo é muitas vezes o
composto inglês óbvio que nós escolheríamos sozinhos, e `grep -F` casa **subcadeias**. Esta obra já
retirou **duas** entradas da própria vassoura exactamente por isso. ⇒ **separar fuga real de
colisão exige revisão entrada a entrada**, e isso é trabalho de **R-pós**, não desta corrida.

⭐ **A dívida já estava NOMEADA no roteador da casa** (`CLAUDE.md` §5, módulo 3D/Sculpt): *«os nomes
de SÍMBOLO internos são §4.2 pela mesma linha da SKILL e o gate não os mede; os `docs/**` ficam
fora do censo por construção»*. O gate que existe (`architecture_no_restricted_source_citations`)
tem a catraca **vazia** e mede **citações de ficheiro**, não nomes de símbolo. ⇒ *esta é a primeira
vez que a dívida conhecida recebe uma MEDIÇÃO* — e ela é maior do lado das **mensagens de commit**,
que são **permanentes** e que nenhum gate deste repo varre.

⏳ **O que fica para decisão do dono** (⛔ não é acto de um subagente): as mensagens de commit só se
limpam reescrevendo história, o que nesta casa é operação de risco e ordem explícita. A alternativa
barata e imediata é **parar a hemorragia**: um gate que varra a mensagem do commit **novo** contra
as vassouras, no `ship.sh` ou num hook — o que impede o 76.º commit sem tocar nos 75.


---

## EMENDA pedida pela janela-I (2026-09-15) — os factos da DEPENDÊNCIA

A janela-I não pode abrir este ledger (o `settings.local.json` da linha nega-lho por construção) e
a §1.4 mandava-a cá para os nomes. ⇒ os factos foram **medidos e escritos na própria espec, §1.5**,
que é o documento que ela lê. Nada disso é expressão do alvo: são bibliotecas de **terceiros**.

**O que a emenda mediu** (o detalhe está na §1.5): construção a frio das duas rotas (`9,51 s` em
Rust puro contra `1 min 39 s` **mais um `git clone` e uma build de CMake**); licenças lidas nos
artefactos; maturidade em descargas, versões e datas; a superfície mínima de que precisamos (cinco
chamadas); e a robustez corrida **nos três motores** sobre as fixturas desta obra.

⚠️⚠️ **DUAS afirmações minhas foram REFUTADAS pela própria emenda:**
1. o candidato que eu chamara *«o melhor encaixe»* **não está publicado em crates.io** — eu julguei
   o código e não a porta de distribuição;
2. a **primeira** medição de robustez da emenda mediu **um** dos dois motores, porque a chamada de
   omissão usa o exacto. Refeita nos três, ela mostra o motor robusto a ganhar exactamente no caso
   para que existe — e a custar `12×`.

⭐ **E o achado que mais vale para quem implementa** está na §1.5.6: com malha **aberta** a operação
devolve malha **vazia** e o estado do RESULTADO diz «sem erro». *Quem verificar só o resultado
entrega uma escultura apagada.* A verificação é do estado da **ENTRADA**.

---

## R-PRÉ, 1.ª passagem (2026-09-15) — NÃO ATESTADA: 9 achados, 2 bloqueantes, todos curados

Veredito do auditor independente: *a parte medida é forte, as fixturas cobrem o que a espec afirma,
e não há fuga de nome interno nem de organização.* O que a impediu de ser atestada:

| # | achado | cura |
|---|---|---|
| **1** ⛔ | a cura do sweep estava **incompleta**: apanha a quebra de linha, **não** a frase com **ênfase markdown no meio** — e esta espec usava o buraco, na **§6.1** | a passagem plana passa a remover **ênfase e marcas de citação**; a oração citada **saiu** e o regime é agora **derivação nossa** (§6.1) |
| **1b** | a largura nova junta **através de parágrafo e de título** ⇒ pode acusar uso lícito | o cabeçalho do script declara que **um acerto da passagem plana exige leitura humana** antes de contar |
| **2** ⛔ | o fecho da §6.2 dizia **quatro** leis de ponto médio; a coacção da §3 corre **antes** da profundidade ⇒ uma célula é **inalcançável** | reescrito com os **três** estados alcançáveis, e a dizer que um despacho `2×2` shipa **um braço morto** |
| **3a** | a frase do propósito do factor de alcance era tradução próxima da comentaria | virou **requisito sobre a SAÍDA** (os pontos caem fora da silhueta projectada) |
| **3b** | idem para a validade do raio, e nomeava o valor **pela estrutura que o guarda** | virou **condição de fronteira**; a atribuição de histórico de defeito **saiu** (o rastreador nunca foi percorrido ⇒ não há origem citável) |
| **3c** | a divergência da §5 acompanhava a estrutura da origem | **re-derivada do observável** (uma normal não se transforma como um ponto ⇒ o eixo do varrimento desvia) |
| **4** | três afirmações **sem selo** (§5, §9, §15) | **seladas `L`**; na §9 ficou a minha frase de mecanismo, que é expansão minha |
| **5** | o cabeçalho mandava o auditor à secção errada | ponteiro corrigido e **ampliado para os sete** itens `L` |
| **6** | dois **nomes de caso repetidos** no corpus, com registos contraditórios, e um mandava ler **o ledger** — que o I está barrado de abrir | as **2 corridas fantasma apagadas** (vieram de um defeito do arnês, não do oráculo) + **`verifica_corridas.py`** com unicidade, piso de população e proibição de motivo que aponte para fora do corpus |
| **7** | a §11.1 dizia **1** face na fixtura degenerada (são **2**); e a célula do README contradizia o parágrafo abaixo dela | ambos corrigidos |
| **8** | o ledger elegia uma rota que **não existe no registo** e cuja licença eu lera **na página**, não num artefacto | a rota **saiu** dos dois documentos, com a razão escrita |
| **9** | o T0 é **por solucionador**, e as tabelas abrangem três | tabela nova na §10.1: só o de **omissão** é a biblioteca externa; **escolher solucionador é escolher degrau** |

⭐⭐ **A lição do #1 é a terceira da mesma espécie nesta obra, e a espécie é sempre ALCANCE, nunca
padrões:** `.gz` opaco ao `strings` (2026-09-14) → frase partida por quebra de linha → frase partida
por **ênfase**. ⇒ *cada vez que o sweep fica verde, a pergunta certa é «o instrumento CHEGA lá?», e
a resposta só vale com controlo positivo do lado que devia acusar.*

⭐ **E o #2 é a lição do «dreno de um braço só» aplicada a uma ESPEC, não a código:** eu enumerei o
produto cartesiano de dois knobs sem verificar a **ORDEM** em que eles são lidos. Uma espec que
descreve estados inalcançáveis manda construir braços que gesto nenhum atinge — e um braço que o
produto nunca alcança não tem como ser testado.

⚠️ **O #6 tem a forma que esta casa já paga por escrito:** o corpus estava **contraditório consigo
mesmo** e nenhum instrumento o media — a prova disso é que o `verifica_corridas.py` **não existia**,
e ao nascer acusou o defeito na primeira corrida. As três mutações (nome repetido · motivo a apontar
para fora · corpus esvaziado) **matam-no** as três.

---

## ✅ R-PRÉ, 2.ª passagem (2026-09-15) — **ATESTADA**

Auditor **independente** (≠ o subagente-E que escreveu a espec), com os dois lados à vista.
Os **9 achados** da 1.ª passagem foram conferidos **um a um** e os **9** estão curados.
O atestado está no cabeçalho da espec; **a janela pode implementar a partir desta versão.**

### O que o auditor re-mediu pelos PRÓPRIOS instrumentos

⚠️ *O E emendou o instrumento que o audita pela terceira vez nesta obra ⇒ nada aqui foi aceite
pela declaração dele.* Todos os controlos abaixo usam **frase do auditor**, nunca do alvo, logo
nada do alvo tocou o disco.

| o que | como | resultado |
|---|---|---|
| a 3.ª passagem **apanha** o que diz apanhar | 5 ficheiros de controlo (negrito · itálico · código · citação · ênfase+quebra) | **acusados os 5** — e os **5** eram limpos na versão anterior |
| ela **não ficou larga** sobre prosa legítima | **9** especs da casa × **7** vassouras, 3 passagens | **2** acertos, os **2 PRÉ-EXISTENTES** (confirmados contra o script anterior à 1.ª cura): termo de arte numa obra, nome público noutra ⇒ colisão, não fuga |
| a espec sob normalização **mais forte que a shipada** | marcadores removidos dos **DOIS** lados + minúsculas + espaço normalizado, 5 artefactos × 7 vassouras | **1** acerto, e é um nome de **propriedade pública** no ficheiro de corridas, que o §0 da espec declara conservado (§4.1.13) ⇒ **uso lícito** |
| o `verifica_corridas.py` | **5** mutações do auditor (nome repetido · piso · motivo a apontar para fora · concluída sem metade · sem veredito) + **controlo verde** | **5 de 5 mortas**, cada uma com a mensagem certa; piso não trivial (`56` contra `50`, dispara a `49`) |
| as **2 corridas apagadas** eram medição? | diff do JSON contra `dc62c83b4`, campo a campo | ⭐ **não**: sem `antes`, sem `depois`, raio por fixar ⇒ **zero** medição; e **`0`** registos alterados ou acrescentados — nada foi destruído |
| as contagens | `58` / `56` / `52` | **reconciliam**: `58` = `56` reais + `2` fantasma · `56` = nomes únicos **antes e agora** · o `52` era o ledger sem a leva de **4** da §3, hoje escrita |
| o commit `dc62c83b4` isolado | mensagem + patch × 7 vassouras × com e sem quebras desfeitas | **limpo nas 4 combinações** ⇒ a dívida de histórico **não é desta obra**, e a mensagem **não precisa de varredura** |

### Os itens de selo `L`, atacados primeiro

- **§6.1 (o enchimento)** e **§10 (as três suposições)**: conferem **exactamente** contra o fonte.
- **§6.2 (o ponto médio)**: era o bloqueante nº 2 — a espec descrevia **quatro** estados e a ordem
  de leitura torna **um inalcançável**; hoje está escrita pelos **três** alcançáveis, com o braço
  morto nomeado.

### Proveniência (a metade T0), validada na máquina e no registo

| rota | licença **lida** | veredito |
|---|---|---|
| a biblioteca externa do solucionador de omissão | `Apache-2.0` (`pacman -Qi`), ligada dinamicamente (`ldd`), projecto de montante **independente** do alvo | ✅ o degrau **T0 mantém-se** |
| as ligações seguras, o `-sys` e a fachada | `Apache-2.0 OR MIT`, mesmo repositório, sobre a API C dela | ✅ |
| o porte **puro-Rust** | `Apache-2.0`, declara-se porte da mesma biblioteca, e o tree dele **não menciona o alvo** | ✅ |
| a 4.ª rota (a que o ledger elegia) | **não existe no registo** | ✅ **retirada** pelo E no achado #8 — confirmado nesta passagem |

⇒ **nenhuma descende do alvo**, e o achado #9 fecha o resto: o T0 é **por solucionador**, e os
outros dois da tabela da §10.1 são código do próprio alvo, **não portáveis**.

### ⏳ Duas notas que NÃO bloqueiam (nenhuma é §4.2)

1. **A 3.ª passagem apaga também `_`**, o que a cega para toda entrada de vassoura que o contenha.
   Medido: **`0`** usos de `_` como ênfase nestes documentos contra `15`/`16` identificadores; a
   passagem 1 cobre-as (um identificador não tem espaços, logo o parágrafo não o quebra), e
   normalizar os **dois** lados **fabrica** a colisão com o nome público. ⇒ correcção de **um
   caractere**, no script, e a espec foi conferida **com e sem** ela.
2. **O cabeçalho de coluna da §11.1** ainda diz «1 face» onde a prosa acima e a nota abaixo dizem
   **duas** — correcção de **uma palavra**. *Três afirmações da mesma grandeza em sete linhas, uma
   delas errada, é a forma que o achado #7 curou do outro lado.*

### ⭐ A lei que esta passagem deixa

*Um sweep verde prova o que o INSTRUMENTO alcança, e a pergunta «ele chega lá?» só tem resposta
com controlo positivo do lado que devia acusar — feito por quem não escreveu nem o texto nem o
instrumento.* Nas três cegueiras desta obra (`.gz` → quebra de linha → ênfase) a espécie foi
sempre **alcance**, nunca padrões; e a terceira foi encontrada **por o auditor correr a régua
corrigida por terceiro**, que é exactamente a razão de o R-pré não poder ser o E.
