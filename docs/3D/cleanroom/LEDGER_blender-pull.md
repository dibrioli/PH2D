# LEDGER de proveniência — clean-room dos pincéis de PUXAR (alvo `blender-pull`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-pull.md` (append cego).

**Aberto em 2026-09-13, ANTES da primeira leitura do fonte e da licença**, pelo subagente-E
`agent-acc2adbfcf5cf748e` da janela `9f820704-0d7e-4d96-847e-9cd720cbf178`.
Único acto sobre o checkout anterior a esta abertura: um `ls` do directório das ferramentas de
pintura/escultura e um `git log -1`/`git describe` para confirmar a tag (nomes de ficheiro e o
número de versão — nenhum conteúdo).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — pincéis **Thumb** e **Nudge** do Sculpt Mode (+ o partilhado que eles invocam), e as opções públicas `use_grab_active_vertex`, `use_grab_silhouette`, `slide_deform_type`, `snake_hook_deform_type` e `deform_target = CLOTH_SIM` nos pincéis desta família |
| Versão do fonte | (preenchido na triagem) |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` (o checkout que a obra `blender-cloth` já usou) — fora de qualquer árvore do PH2D. Zona de notas/rascunhos/oráculo desta obra: `~/Referencias/blender-pull/` |
| Oráculo (binário) | `/usr/bin/blender` |
| Licença | (preenchido na triagem, lida no ficheiro) |

---

## INBOX transcrito (append cego da janela I)

| data | linha (transcrita) |
|---|---|
| 2026-09-13 | `I session: 9f820704-0d7e-4d96-847e-9cd720cbf178 2026-09-13 — janela I da line/sculpt3d (retomada pos-compactacao, a mesma do INC-4 classificado RELANCE); obra: os pinceis que faltam, familia pull (Thumb, Nudge)` |

---

## §2 — Triagem (2026-09-13)

**Licença lida no ficheiro:** o `COPYING` da raiz remete a `doc/license/GPL-license.txt` (GNU GPL
**versão 2**, junho de 1991) e declara que o programa não está disponível sob outra licença; os
ficheiros dos pincéis desta obra levam cabeçalho SPDX `GPL-2.0-or-later` (conferido nos cinco
ficheiros de pincel lidos). Não é AGPL. Concessão relevante (GPLv2 §0/§2) já transcrita no
[ledger irmão](LEDGER_blender-cloth.md#a-concessão-relevante-gplv2-§0-e-§2-transcrita-do-ficheiro-do-checkout)
— o mesmo checkout, a mesma licença.

| degrau | veredito | o que foi buscado |
|---|---|---|
| T0 | ⛔ | os pincéis Thumb/Nudge e as opções desta obra só existem neste alvo |
| T0½ | ⛔ | nenhum ficheiro por-arquivo MPL/LGPL implementa estes pincéis |
| T1 (a) autores | ⛔ | os recursos foram escritos directamente no alvo (commits de 2019–2020, revisão interna); não há paper nem código de referência dos autores |
| T1 (b) versões antigas | ⛔ | o alvo é GPL desde sempre |
| T1 (c) reimplementações | ⛔ **dois candidatos, os dois recusados**: `github.com/khanhha/digital_sculpting` — tem pincéis com os mesmos nomes, **não tem ficheiro de licença** (logo, todos os direitos reservados) **e** a sua camada de malha traz o cabeçalho GPL do alvo (clonado para `~/Referencias/blender-pull/notes/t1/`, conferido por `grep`) ⇒ descende do alvo, é lavagem alheia · `github.com/marmelab/sculpt-3D` — MIT, mas **não tem** nenhum destes pincéis (só Add/Subtract/Push + transformações) · o SculptGL (MIT, já portado na casa) não tem Thumb, Nudge, active vertex, silhouette nem slide |
| T1 (d) e-mail | não aplicável — a fundação declara no `COPYING` que não licencia sob outra licença |
| **T2** | ✅ **degrau desta obra** | copyleft com fonte |

## Patente (§8.1) — checkpoint incondicional (2026-09-13)

- **Termos:** `sculpting brush thumb nudge tangent plane stroke direction patent` ·
  `grab brush silhouette mesh sculpt patent` · `snake hook / drag pull geometry along stroke patent` ·
  `slide vertices along surface tangent relax topology patent` ·
  `grab tool active vertex / topologically connected region patent` · o número do Kelvinlet;
  cruzados com Pixar, Pixologic/Maxon, Autodesk, Disney, Adobe.
- **Resultado:**

| patente | dono | estado | lê sobre esta obra? |
|---|---|---|---|
| **US 10 586 401 B2** — *Sculpting brushes based on solutions of elasticity* | Pixar | ⛔ **VIVA** — prioridade 2017-05-02, concedida 2020-03-10, expira **2038-05-02** (Google Patents, lido 2026-09-13) | ⛔ **SIM, sobre UMA opção:** a reivindicação 1 é *seleccionar um pincel e um tamanho, receber movimento, e determinar a deformação por soluções regularizadas da elasticidade linear com um coeficiente de Poisson especificado*. O modo **ELASTIC** do `snake_hook_deform_type` faz exactamente isso (Kelvinlet regularizado com coeficiente de Poisson fixo — lido no fonte). ⇒ **o modo ELASTIC fica FORA desta espec** e o achado vai ao Enio. ⚠️ **E ela lê também sobre o que a casa JÁ TEM:** o nosso repo implementa um modo por Kelvinlet regularizado para os verbos de agarrar (conferido no nosso próprio código, que cita o paper). Reportado como facto, sem mais análise — a decisão é do dono, com parecer humano (§8.5). ⚠️ Não lê sobre Thumb, Nudge, active vertex, silhouette, slide nem o modo FALLOFF (nenhum resolve elasticidade) |
| US 9 792 723 B2 — *progressively sculpting 3D geometry* | Disney | viva até 2035-10-30 | ⛔ não — reivindica guardar esculpidos como *fixes* com offsets por instante e interpolá-los numa animação; nada sobre a lei de um pincel |
| US 7 446 778 / 7 652 675 / 7 728 843 — pincel com referencial no plano tangente para pintura directa em superfícies | — | não aprofundadas: pintura de atributos em superfícies parametrizadas, sem deformação | não lê sobre deformação de vértices |
| US 8 907 976 — *resolution-adaptive mesh smoothing brush* | — | não aprofundada | fora do âmbito (alisamento, não puxar) |

⇒ **Veredito: PATENTE VIVA US 10 586 401 B2 sobre o modo ELASTIC do Snake Hook** — a opção
sai da espec; o resto da obra prossegue (nenhuma outra patente lê).

## Papel E

| campo | valor |
|---|---|
| quem | subagente-E `agent-acc2adbfcf5cf748e` (descrição do despacho: «E: Thumb, Nudge e opções do puxar») |
| janela-mãe (I) | `9f820704-0d7e-4d96-847e-9cd720cbf178` |
| transcript (⛔ zona contaminada — I nunca lê) | `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178/subagents/agent-acc2adbfcf5cf748e.jsonl` |
| paralelos na mesma worktree | três subagentes-E irmãos (`blender-pose`, `blender-boundary`, `blender-unblocked`) — cada um toca só os seus caminhos |
| ⛔ nunca | código de produto; `git push`; `git add -A`; `git stash`; escrever em `project-memory/`; editar o harness partilhado `docs/3D/ferramentas/blender_sculpt_oracle.py` |

### Segundo despacho de E (a corrida que entregou)

| campo | valor |
|---|---|
| quem | subagente-E `agent-a5ce83c4776f4dff1`, 2026-09-13 |
| porquê um segundo | a 1.ª corrida (`agent-acc2adbfcf5cf748e`) morreu por limite de uso da conta **depois** da triagem, da patente, do oráculo e da mineração da história, e **antes** da travessia e da espec. Nada dela tinha sido comitado |
| o que herdou e **conferiu** | a triagem (T2) · o checkpoint de patente · o rascunho da vassoura · o harness e **154 dumps** · as notas de história |
| o que refez | a travessia integral do fonte (abaixo), a validação numérica (⚠️ **três resíduos herdados eram defeito da RÉGUA, não do alvo** — ver *Correcções*), a vassoura (**298** entradas) e a espec |
| transcript (⛔ zona contaminada — I nunca lê) | `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178/subagents/agent-a5ce83c4776f4dff1.jsonl` |

---

## Versão — e o desvio entre fonte e binário (registado como RISCO)

| | |
|---|---|
| Fonte lido | etiqueta `v5.2.0`, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` (2026-07-13), em `/home/enio/Documentos/Recursos/BlenderSculpt/` |
| Binário-oráculo | `5.2.1 LTS`, hash de build `9e2066aef7ef` (commit de 2026-08-24) |

⚠️ **O checkout NÃO tem a etiqueta `5.2.1`**, logo o desvio não pôde ser diferenciado. Nenhuma
divergência de comportamento foi observada nesta família — e **nenhuma foi procurada**. Registado
na espec §12.5 como risco, não como facto.

---

## Cobertura da travessia (§3.E) — 2026-09-13

Percorrida a família inteira do gesto de escultura sobre malha, arquivo a arquivo, mais a
história. Em vocabulário do domínio (⛔ nomes internos só na vassoura):

| área | o que foi lido |
|---|---|
| **os dois gestos que faltam** | a unidade de compilação do gesto ancorado (**integral**, 201 linhas) e a do gesto que viaja (**integral**, 231 linhas — ela hospeda três gestos, e o que falta é um deles) |
| **os gestos que já temos, pelas opções** | a unidade do agarrar (integral, incluindo a etapa da silhueta) · a do deslize (integral: as três direcções, a influência da vizinhança, o cálculo por nó) · a do gancho (o caminho do modo de alcance e a fronteira do modo excluído) |
| **o condutor partilhado** | o ficheiro-motor de 8 417 linhas, por regiões: a cadeia de pesos (as 9 etapas e as 6 variantes por tipo de malha) · o cálculo da força por gesto (o `switch` inteiro) · a conversão gesto→vector e as duas famílias de delta · a amostragem da normal da área (raio, pesos, os dois baldes) · a plumagem e as passagens de simetria · o recorte e a trava de eixos · a escrita de posições · a reposição a partir do passo de undo |
| **o cabeçalho partilhado** | o vocabulário de operações sobre pesos e translações (535 linhas, integral) e o cabeçalho interno do módulo, nas partes da escrita de posições e das constantes de *rake* |
| **o traço** | a colocação de carimbos, o espaçamento, o factor de sobreposição, quais gestos exigem ponto de superfície, quais recebem um carimbo por evento, e ⭐ **o caminho por lista de eventos** (o que o oráculo usa) |
| **a curva do alcance** | a avaliação das formas de curva, no módulo de pincéis do núcleo |
| **a auto-máscara** | só a fronteira: quais gestos têm a auto-máscara limitada pelo raio |
| **o alvo «simulação»** | a criação e o passo do solver, os tipos de restrição, as constantes, e **quem escreve os alvos de deformação** |
| **a superfície pública** | as definições de propriedade e as tabelas de itens das quatro opções do âmbito (identificadores, valores, rótulos) e **onde a interface as oferece** |
| **história** | 10 conjuntos de mensagens de commit dos ficheiros da família (recentes e antigos), 7 buscas de relatórios por assunto, 13 relatórios individuais com os respectivos comentários, e as notas de versão de 2.81 a 2.92 (a janela em que esta família nasceu) |

⛔ **Fora da travessia, de propósito:** tudo o que é cor, tudo o que depende de conjuntos de faces
(§9.3 da espec), e o interior do modo excluído por patente — lido **só** o suficiente para saber
onde ele começa e termina, e nada da matemática dele.

---

## Correcções ao que a 1.ª corrida tinha registado

⚠️ **Três resíduos herdados acusavam o alvo e o defeito era da NOSSA régua.** Refeitos a partir da
lei lida no fonte:

| o que a 1.ª corrida registou | o que a medição de 2026-09-13 deu | o defeito |
|---|---|---|
| gesto ancorado com espelho: resíduo `1,5e-01` | **`1,6e-07`** | a régua somava as duas passagens com o raio do alcance escrito num literal em vez do valor da corrida |
| gesto ancorado em superfície curva, com a fracção do raio da normal a `0,3`: `4,5e-02` | **`1,8e-07`** | a régua amostrava a normal com a fracção `0,5` fixa, ignorando a da corrida |
| gesto ancorado em superfície curva, truncagens: `3,1e-02` a `8,9e-02` | **`≤ 2,4e-07`** | o mesmo literal |

⇒ **os quatorze vectores do gesto ancorado fecham no ruído de `f32`.** *Uma discordância com o
oráculo é uma hipótese sobre o alvo **ou** sobre a nossa régua — e a régua é a mais barata de
conferir primeiro.* A régua nova vive em `~/Referencias/blender-pull/oracle/valida_fonte.py`.

⚠️ E uma quarta: os resíduos do gesto que VIAJA em superfície curva (`~10 %`) **não** são da régua —
quatro hipóteses sobre a normal foram medidas e as quatro dão o mesmo número. A causa com endereço
está na espec §6.3.

---

## Achados de MEDIÇÃO que mudam o que se constrói

1. ⭐⭐ **O slider de força entra ao QUADRADO** nos dois gestos que faltam (e no deslize), e
   **linearmente** no agarrar e no gancho. Medido: força `0,5` dá pico `0,150000` contra `0,600000`
   — razão **`0,25`** ao bit. ⇒ um implementador que generalize de um gesto para o outro erra `2×`.
2. ⭐⭐ **Os dois gestos que faltam partilham a fórmula do deslocamento**; separa-os **de que pose a
   pegada é medida**. Medido pelo par de fixtures de 12 e 24 eventos: o ancorado dá `0,600000` nos
   dois, o que viaja dá `0,599054` contra `0,595226`.
3. ⛔⛔ **O alvo de deformação «simulação» move ZERO vértices** nos **cinco** gestos testados,
   enquanto os pares de controlo movem normalmente; e a interface do próprio alvo **não o oferece**
   em nenhum gesto desta família. ⇒ **não se constrói** (espec §9.2).
4. ⛔ **A silhueta nunca foi exibida:** as cinco corridas com ela ligada dão zero. Duas causas
   possíveis e o mesmo zero — a armadilha do sinal nulo (real, lida no fonte) e o facto de o valor
   que a lei usa ser amostrado no **hover**, que um traço scriptado não produz (a ferramenta de
   sistema que o harness tentou usar **não está instalada** nesta máquina). ⇒ lei **lida, não
   medida**; instrumento de fecho na espec §12.1.
5. ⛔⛔ **O factor de sobreposição vale `1` no oráculo e não vale `1` no produto interactivo** — o
   caminho por lista de eventos nunca o calcula. ⇒ as magnitudes medidas do gesto que viaja e do
   deslize são as de sobreposição `1`; as do ancorado não são afectadas (ele não a lê).

---

## Espec

| versão | caminho | data | commit | sweep |
|---|---|---|---|---|
| 1 | [`SPEC_pull_brushes.md`](SPEC_pull_brushes.md) | 2026-09-13 | o commit que **criou** este ficheiro na `line/sculpt3d` (`git log --diff-filter=A -- docs/3D/cleanroom/SPEC_pull_brushes.md`) | ✅ verde (vassoura de **298** entradas) |

⚠️ **O hash não se escreve aqui à mão, e a razão é estrutural:** a espec, o ledger e as fixtures
entram no **mesmo** commit (a missão pede um só), logo qualquer hash escrito neste ficheiro é o do
commit *anterior* ao que o contém — ele nasce errado e um `--amend` volta a invalidá-lo. *Um número
que não pode estar certo no ficheiro que o carrega deriva-se do git, nunca se copia.*

⭐ **O sweep foi verificado com CONTROLO nas duas classes** antes de valer como aprovação: um
identificador e uma frase da vassoura, cada um num ficheiro de teste fora da árvore, levaram o
script a `exit 1`; os artefactos reais deram `exit 0`. *Um instrumento que nunca foi visto
reprovar não aprova nada.*

## Papel R — modo PRÉ (auditoria §4.2 da espec), 2026-09-13

| campo | valor |
|---|---|
| quem | subagente-R `agent-ae2f81f455d29b69f` (descrição do despacho: «R-pré: auditoria da espec do puxar») |
| janela-mãe (I) | `9f820704-0d7e-4d96-847e-9cd720cbf178` |
| independência (§3.R) | ✅ contexto **novo**, distinto dos dois subagentes-E (`agent-acc2adbfcf5cf748e`, `agent-a5ce83c4776f4dff1`) — autofiltragem não se audita |
| transcript (⛔ zona contaminada — I nunca lê) | `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/9f820704-0d7e-4d96-847e-9cd720cbf178/subagents/agent-ae2f81f455d29b69f.jsonl` |
| **veredito §4.2** | ✅ **VERDE** — atestado no cabeçalho da espec |
| sweep (§7.1) | `exit 0` sobre `SPEC_pull_brushes.md` + `fixtures/pull/` (68 ficheiros), vassoura de **298** entradas |

### O que foi auditado, item a item do §4.2

| cláusula do §4.2 | veredito |
|---|---|
| texto de código, trechos, diffs | ✅ nenhum — os blocos são pseudo-código de nível de paper, em português e em vocabulário do domínio |
| nomes internos (funções, variáveis, ficheiros, structs) | ✅ nenhum. Os identificadores que aparecem (os dois nomes de pincel e as cinco propriedades/itens de enum das opções do §7) são **API pública** do alvo, alcançáveis pela linguagem de script dele ⇒ §4.1.13 (*«interno se renomeia, interface pública se documenta»*); a espec ainda lhes dá nomes nossos e descreve cada opção funcionalmente |
| comentários do original | ✅ nenhum verbatim. Há **uma** frase (§3) que re-diz, em palavras nossas, a intenção de desenho registada pelos autores — licenciada pelo §4.1.12, com proveniência **funcional** (*«onde o quadrado é feito»*), sem nome nem endereço de ficheiro |
| wording de manual/doc/paper verbatim ou quase | ✅ nenhum. A única paráfrase entre aspas é de um documento **de patente** (USPTO), que não é obra do alvo |
| organização arquivo-a-arquivo / função-a-função | ✅ a espec organiza-se por **fases funcionais** (referencial · peso · força · as duas famílias de deslocamento · cada gesto · o partilhado); a decomposição em unidades de código fica para o I |
| pseudo-código espelhado linha a linha | ✅ não — cada bloco tem 5–12 linhas, é a matemática do método (§4.1.2/§4.1.11) e vem acompanhado do **porquê** funcional |
| LUTs grandes copiadas verbatim | ✅ nenhuma — as três curvas de alcance são **fórmulas**, não tabelas |

### Mandato específico desta auditoria — o modo excluído por patente

1. ✅ **Está declarado FORA DE ÂMBITO e a espec não carrega lei nem constante dele.** Varrida a
   espec por toda a matemática daquele modo: o modo é nomeado em **três** sítios (§7.4, §9.1, §13) e
   **sempre** para dizer que sai. Zero fórmula, zero constante de material, zero regularização.
2. ⚠️ **A leitura sobre o que a casa já shipa SUSTENTA-SE**, e o R-pré corrigiu o §9.1 com três
   factos de patente que faltavam (texto concedido, lido em 2026-09-13): são **três**
   reivindicações independentes (1 método · 13 meio legível · 20 sistema) e não uma; o argumento
   *«usamos um meio, logo estamos fora»* é **derrubado pela própria dependente 12**; e a família é
   **só dos EUA**, o que é a alavanca de território que o documento não tinha.
   ⛔ Nenhuma dessas correcções toca material do alvo — patente é documento público.

## Fixtures

`docs/3D/cleanroom/fixtures/pull/` — 62 traços + 5 malhas de repouso, chaves e etiquetas em
vocabulário do domínio, proveniência da ENTRADA e da SAÍDA no README de lá. ⚠️ **Dez medem ZERO de
propósito** (as cinco do alvo «simulação» e as cinco da silhueta): elas são a prova de que aqueles
comportamentos **não foram exibidos**.
