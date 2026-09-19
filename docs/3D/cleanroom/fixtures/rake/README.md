# Corpus do PENTE DE TOPOLOGIA — proveniência e a RÉGUA

> ⚠️ **O CABEÇALHO DE CADA FICHEIRO É A FONTE; esta prosa nunca é.** Uma grandeza nova que não
> entre no cabeçalho esconde-se debaixo da frase que descreve o corpus, e **nenhuma varredura a
> acusa**.

## §1 — Proveniência

| | |
|---|---|
| **Entradas** | **NOSSAS**, geradas pelo harness: um plano `28×28` células de lado `2,4`, triangulado com diagonais **alternadas** e com abanão determinístico de `0,55` da célula nos vértices interiores (a entrada canónica, `entrada_9fb3d9dea0d0.txt.gz`); mais um plano de **quads**, um plano com **furo** (bordo aberto interior) e uma esfera UV. ⛔ Nenhum asset do alvo entra como entrada. |
| **Saídas** | Lista de posições e de faces devolvida pelo alvo, corrido **sem interface** por script sobre as entradas acima. Saída de programa é **dado** (GPLv2 §0). |
| **Alvo corrido** | binário **5.2.2 LTS**, pacote `17:5.2.2-1`, build **2026-09-15**. ⚠️ **As obras anteriores desta linha correram `5.2.1-2`** — este corpus e os delas **não vêm do mesmo binário**, e é por isso que a versão está no cabeçalho de cada célula. |
| **Data da colheita** | 2026-09-17 |
| **Quem colheu** | o subagente **E**; o harness vive **fora da árvore** e não viaja com ela. |

Cada célula é `<família>/<nome>.txt.gz`, com o cabeçalho `# chave=valor`, depois o percurso do cursor
(uma linha `# ponto x y z` por ponto), depois `V n` + `n` posições e `F m` + `m` faces.
A **entrada** de cada célula é nomeada no cabeçalho (`malha_de_entrada=`) e vive em `entrada/`.

⚠️ Os valores como `DRAW`, `SMOOTH`, `CONSTANT`, `SUBDIVIDE_COLLAPSE` são **identificadores públicos
de enumeração** da interface do alvo, presentes só porque são a **chave de regeneração** da célula —
é por eles que uma corrida futura reproduz o mesmo enquadramento.

## §2 — A RÉGUA, e porque ela é esta

A grandeza desta obra é a **direcção do fluxo de arestas**, não a posição. Para cada aresta cujo
ponto médio caia dentro da pegada, mede-se `α ∈ [0°, 90°]`, o ângulo entre a aresta e a direcção do
traço, ambos projectados no plano do ecrã (uma aresta não tem sentido, logo o ângulo dobra-se). A
régua é o **parâmetro de ordem de quatro dobras**:

```
Q = média( cos 4α )           Q = +1  toda aresta a 0° OU a 90° do traço  (uma GRADE alinhada)
                              Q =  0  isotrópico
                              Q = −1  toda aresta a 45° do traço
```

⛔⛔ **A régua ÓBVIA — a de duas dobras, `S = média(cos 2α)` — é CEGA a este fenómeno, por
construção**, e foi a primeira que se escreveu. Ela vale `+1` a 0° e `−1` a 90°, logo uma grade
alinhada, que tem as duas famílias, dá-lhe `≈ 0`. Medido nas quatro rotações do traço:

| rotação do traço | `S` desligado | `S` ligado | `Q` desligado | `Q` ligado |
|---|---|---|---|---|
| 0° | +0,0412 | +0,0595 | −0,0456 | **+0,1243** |
| 22,5° | +0,0327 | **+0,0215** | −0,0154 | **+0,0901** |
| 45° | +0,0157 | +0,0368 | +0,0273 | **+0,1491** |
| 67,5° | +0,0535 | +0,0547 | −0,0123 | **+0,0940** |

*A régua de duas dobras **troca de sinal** entre rotações do mesmo traço (a `22,5°` ela DESCE) e o
seu `Δ` é `5×` a `90×` mais pequeno; a de quatro separa nas quatro rotações, sempre no mesmo sentido
e nunca por menos de `+0,10`.* ⇒ **a régua escolheu-se MEDINDO**, e o que ela mede é uma **grade**, não uma
direcção só.

⚠️ **A PEGADA É UM DISCO DE ECRÃ, e medi-la em 3D dá `None`.** A primeira redacção media a distância
do ponto médio ao percurso em três dimensões. Com duas passagens o relevo sobe a `0,24` e a região
de raio `0,175` fica **vazia** — a régua devolvia *«não há arestas»* sobre uma célula perfeita. As
fixturas planas são vistas de topo em projecção ortográfica, logo **a projecção XY é o ecrã**.

⚠️ **A região usada nos números abaixo é `raio/2`** e não o raio inteiro. Medido: a `raio/2` a
separação é `2`–`3×` maior (0° ligado: `+0,0353` no raio inteiro contra `+0,1243` a `raio/2`; desligado, `−0,0632` contra `−0,0456`) —
**o efeito concentra-se na faixa central do traço**, e o raio inteiro dilui-o com a orla.

⛔ **A régua é PLANAR.** Numa esfera ela lê o mesmo valor com o pente ligado e desligado
(`+0,0453` nos dois) enquanto a malha mudou mesmo — `2 032` faces e `3 048` arestas dos dois lados,
com **`1 260`** de diferença simétrica no conjunto de arestas: ali as arestas junto da
silhueta projectam curtas e o ângulo projectado não é o ângulo na superfície. ⇒ **dívida NOMEADA:
uma régua para malha curva projecta a aresta e a direcção do traço no plano tangente local.** Os
números deste corpus valem para as fixturas **planas**.

## §3 — O VALE, e de onde sai a barra

⛔ **A população tem de ser UMA COISA.** A primeira tentativa varreu «tudo o que tem detalhe
constante» e devolveu um vale **invertido**, porque lá dentro estavam células de outra malha de
entrada, células sem passe de refino e a esfera. A população é:

> `malha_de_entrada = entrada_9fb3d9dea0d0.txt.gz` · `topologia_dinamica = armada` ·
> `modo_de_detalhe = CONSTANT` · `passe_de_refino = SUBDIVIDE_COLLAPSE` ·
> `resolucao_do_detalhe = 18` · `PASSAGENS = 1` · **sem** `SEQUENCIA_DE_TRACOS` ·
> `pontos_usados_truncagem = 10` (percurso inteiro) ·
> e o **controlo**: o verbo subdividiu (`v_saida > 1,5 × v_entrada`).

⚠️ **Todos os nove critérios são CHAVES DO CABEÇALHO**, de propósito: a 1.ª redacção descrevia-os em
prosa e um terceiro a aplicá-los obteve outro `n` que o declarado.

| | n | o pior da população | célula |
|---|---|---|---|
| **pente desligado** | **32** | `Q = +0,02982270` (o MAIOR) | `escada/k_a0450_p0000` |
| **pente no máximo** | **32** | `Q = +0,06323128` (o MENOR) | `verbos/v_blob_p100` |

⛔⛔ **Este `n` é a TERCEIRA redacção do mesmo número, e as duas anteriores (`37`, `34`) estavam
erradas** — a segunda **depois** da cura desenhada para o tornar reprodutível. O `32` sai de aplicar
as nove chaves **à letra**, como uma comparação de strings do cabeçalho; largar qualquer uma de
`malha_de_entrada`, `passe_de_refino` ou `resolucao_do_detalhe` devolve `34`.
⭐ **E a barra é ROBUSTA a isso, medido:** com `32`, `34` e `35` as duas células extremas são as
MESMAS e as reprovações são **zero dos dois lados**. ⇒ o `n` é escrituração, não segurança — mas é
escrituração que já falhou três vezes, e é por isso que ele vai agora com a receita ao lado.

**VALE = `[+0,0298 , +0,0632]`**, largura `0,0334`.
⭐ **Os dois lados são saída do PRÓPRIO alvo** — o lado «aprovado» é o alvo com o pente ligado —,
que é a condição que duas barras do corpus do tecido desta casa foram retiradas por não cumprir.

**BARRA = `+0,046527`, arredondada a `+0,0465` — o MEIO do vale, e ela é UMA SÓ:** `Q ≥ +0,0465`
para *«o pente está no máximo»* e `Q ≤ +0,0465` para *«o pente está desligado»*, com margem
**`±0,0167`** para as duas células extremas.
⛔⛔ **A 1.ª redacção dava `Q ≤ +0,0298` para o lado desligado — o próprio extremo, arredondado a
quatro casas PARA BAIXO** — e a célula que o gerou mede `+0,02982270`: margem `−0,0000227`, ou seja
**o gate nascia vermelho sobre a saída do próprio alvo**. *Uma barra tirada de um extremo tem margem
zero por construção e o arredondamento decide o sinal dela.*

⚠️ **A barra separa DESLIGADO de MÁXIMO, não desligado de qualquer posição.** A meio curso os
valores atravessam o vale (ver a escada no `SPEC`), e é isso que a torna honesta.

## §4 — O ORÁCULO NÃO REPETE, e a banda está medida

Seis corridas da **mesma** célula (`banda/b_con*`):

| | v de saída | `Q` | amplitude |
|---|---|---|---|
| **pente desligado** | `2525` nas seis | `−0,04555619` .. `−0,04555623` | `4e-8` |
| **pente no máximo** | `2444` .. `2448` | `+0,11428` .. `+0,12542` | **`0,0111`** |

⇒ **vale / banda = `3,0`.**

⛔⛔ **E a primeira leitura desta não-repetição estava ERRADA, de uma forma que vale para toda
bancada futura desta casa.** Comparando as duas corridas **por índice**, `522` de `~4 200` arestas
«diferiam» com o pente **desligado** — e a contagem de vértices, a contagem de arestas da região e o
`Q` eram idênticos a sete casas. A causa: **o passe de refino emite a mesma geometria com os
vértices por OUTRA ORDEM**. Ordenadas as posições, `10` de `2 525` diferem (`0,4 %`).
⇒ *uma comparação por índice mede a ORDEM, não a malha* — e é por isso que **toda régua deste corpus
é invariante à ordem**. Com o pente ligado a contagem de vértices ela própria muda (`2444`–`2448`),
e aí a variação é real.

⭐ **No regime SEM passe de refino o alvo é determinístico nos dois lados**: o conjunto de arestas é
idêntico entre corridas e `Q` repete à 4.ª casa. É esse o regime em que as leis de mecanismo da
`composicao/` e da `mecanismo/` foram fixadas, e é por isso que elas se podem cobrar **exactamente**.

## §5 — As famílias

⚠️ **Os números desta tabela são CONTADOS da pasta** (`find <pasta> -name '*.gz' | wc -l`) — a 1.ª
redacção escreveu-os de memória e **três** células estavam erradas.

| pasta | ficheiros | o que ela responde |
|---|---|---|
| `entrada/` | **14** | as malhas de entrada, todas nossas |
| `porta/` | **6** | a **pré-condição**: com a topologia dinâmica desarmada a saída é **byte-idêntica** com o pente a `0` e a `1` |
| `escada/` | **29** | a **escada do botão** (7 posições × 3 rotações, `k_*`) **mais** as 8 de passagens repetidas (`n_x1..n_x8`) |
| `rotacao/` | **10** | o **discriminador da direcção**: 4 rotações do traço × 2 lados, mais 2 pares de passagens |
| `verbos/` | **62** | **38** do censo (`v_*`: 19 verbos × 2 lados) **mais 20** de composição (`t_*` o modo de detalhe · `q_res*` a resolução · `s_sim` a simetria) |
| `composicao/` | **26** | os dois verbos **refeitos** (`m_rot`, `m_mask`), o refino (`m_subdiv`/`m_collapse`/`m_manual`), os degenerados de direcção e os de malha |
| `mecanismo/` | **30** | o regime **determinístico** (sem passe de refino): é aqui que se prova o que o pente faz |
| `banda/` | **18** | a repetibilidade, nos dois regimes |
| `lei_unica/` | **26** | ⭐ **a medição que decide se é UMA lei ou duas** (`SPEC` §14): o alvo a fazer as duas metades juntas contra fazê-las em série, com o entrelaçamento a `1×`, `3×`, `4×` e `9×` |

## §6 — Duas coisas que o cabeçalho declara e que a 1.ª redacção não declarava

⭐ **O `sha256` do §2.1 da `SPEC`** é sobre o **corpo** do ficheiro: as linhas que **não** começam
por `#`, juntas por `\n`. (Sem esta receita um revisor teve de a descobrir por varredura de seis
definições.)

⭐ **A contagem de «vértices que o pente moveu»** usa `eps = 1e-7` sobre a maior componente de `Δ`;
e «maior deslocamento» é a maior **norma** de `Δ`. *Metade de uma tabela reproduzível e a outra
metade não, sem nada no texto a separá-las, é pior que uma tabela ausente.*

⛔ **As células `verbos/v_rotate_*` e `verbos/v_mask_*` estão SUPERADAS e trazem-no no cabeçalho**
(`SUPERADA_POR=` + `SUPERADA_PORQUE=`): a primeira mediu um verbo **inerte** (um percurso recto dá
ângulo zero a um verbo de torção ancorado) e a segunda não trazia o controlo de **canal**. As
refeitas são `composicao/m_rot_*` e `composicao/m_mask_*`. *Elas ficam no corpus porque a medição
que as invalidou é ela própria um facto — mas ficam MARCADAS.*

---

## §7 — O censo

⚠️ **Cada célula do censo de verbos tem o CONTROLO dentro**: o número de vértices que o próprio
verbo moveu contra o repouso. Sem ele, um verbo **inerte** lê-se exactamente como um verbo que
ignora o pente — e duas células deste corpus (a torção com um percurso recto, e a esfera sem passe
de refino) foram apanhadas assim, **medidas como inertes e refeitas** com a fixtura que contém o
fenómeno (uma varredura angular; o passe armado).

⚠️ **Família `acumula` (18 células), colhida 2026-09-18** — a metade da tabela-verdade
que faltava: o interruptor de acumulação nos **dois** estados, com o cursor parado e
`N ∈ {1,2,4,8,14,27,40}` carimbos, mais quatro espelhos das células do `mecanismo`.
⚠️ **Elas NÃO entram na população do §3** (são `modo_de_detalhe=MANUAL` e não mudam a
contagem de vértices), logo o vale e a barra ficam intactos. ⭐ A âncora que as liga ao
corpus antigo é uma IDENTIDADE: `acumula/escada_n14_off` é byte-idêntica, no corpo, a
`mecanismo/y_parado_p000`.

⛔ **Risco de leitura nomeado:** em `escada/` o prefixo `n_x*` quer dizer **passagens**;
em `acumula/` o `escada_n*` quer dizer **carimbos**.
