# Fixtures — traços do pincel de CONTORNO do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes arquivos são os vectores de teste da [`SPEC_boundary_brush.md`](../../SPEC_boundary_brush.md) §19**
— um traço scriptado por célula do corpus, com a malha de repouso e as posições depois do traço.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness (grelhas planas em quatro variantes, tira de uma fileira, tubo aberto, cúpula aberta, casca fechada). ⛔ **Nenhum asset do alvo** |
| **Quem calculou** | o binário Blender 5.2.1 LTS, corrido pelo E **fora da árvore** (`~/Referencias/blender-boundary/oracle/`, ⛔ negado ao I) com um traço scriptado, numa **tela virtual** (nunca no ecrã do dono). O pincel vem de um preset do binário **só para existir um pincel de contorno activo** (a API não deixa criar+activar um de raiz), com **TODOS** os parâmetros reescritos para os valores do cabeçalho de cada fixture |
| **Estatuto legal** | ⭐ **dados** — *«the output from the Program is covered only if its contents constitute a work based on the Program»* (GPLv2 §0): posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do I (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-13 |

## O que há aqui

| ficheiro | quantos | o quê |
|---|---|---|
| `<peça>.repouso.txt.gz` | `9` | a malha de repouso: `v` = posições (`float32` tal como o oráculo as guardou), `f` = faces. ⭐ O montador **aborta** se a malha que ele gera em Python não bater ao bit com a que o oráculo devolveu |
| `<traço>.deformado.txt.gz` | `61` | cabeçalho + `c` = o caminho do cursor + `d` = as posições depois do traço |
| `<traço>.porpasso.txt.gz` | `7` | o mesmo traço **truncado** em `1..8` eventos, um bloco `passo k` por truncagem |
| `confere_cabecalhos.py` | 1 | **a régua** deste README (ver abaixo) |

## O traço (o que vale quando o cabeçalho não disser outra coisa)

Vista ortográfica (**topo** nas grelhas e na tira; **frente** no tubo, na cúpula e na esfera). O
cursor começa sobre a superfície, a uma célula da borda de baixo, e arrasta **`0,1` para BAIXO na
vista** em **`8`** eventos iguais, método *dots*, espaçamento `10`.
Raio do pincel em espaço de objecto **`0,25`** (`0,3` nas peças curvas); Força **`1,0`**, pressão
`1`, curva *Smooth*, dureza `0`, deslocamento da origem **`0`**, sem máscara, sem automáscara, sem
simetria, sem esbatimento, alvo **geometria**, sem inversão — i.e. **as omissões do código**, não as
de um preset.

⚠️ **O eixo do corpus são três grandezas e elas NÃO são excepção**: `superficie`, `modo` e
`queda_no_contorno` — é para as variar que o corpus existe. Tudo o resto que fuja ao parágrafo acima
é excepção, e a lista está **derivada**, nunca escrita à mão:

```
python3 docs/3D/cleanroom/fixtures/boundary/confere_cabecalhos.py            # imprime a tabela
python3 docs/3D/cleanroom/fixtures/boundary/confere_cabecalhos.py --check 59 # falha se mudou
```

⚠️⚠️ **A régua está no repo porque uma lista de excepções sem ela não é auditável** — quem
acrescentar um traço herda a régua que não vê, e a lista envelhece em silêncio. Hoje ela varre
`68` fixtures e conta **`59`** excepções em **`11`** grandezas. ⭐ **Uma grandeza NOVA no cabeçalho é
uma coluna nova na régua, no MESMO commit** — e a régua **aborta** (não «passa vazia») se encontrar
uma fixture cujo cabeçalho não traga uma das grandezas que ela mede, ou se não encontrar fixture
nenhuma. *Uma varredura partida que devolve zero lê-se exactamente como aprovada.*

⚠️ **O `arrasto` é comparado contra a vista**, não contra um literal: as `11` peças de vista frontal
arrastam em `−z` e as de topo em `−y`. Compará-las com um só literal acusaria as frontais **por
construção da régua** — que é o oposto de medir.

## Três coisas que mudam a leitura de uma fixture

1. ⭐⭐ **As cinco fixtures de `suavizar` têm arrasto ZERO** (`0 0 0`) e mesmo assim deformam. Não é
   erro de configuração: é a prova de que o modo `SMOOTH` **não é conduzido pelo arrasto** — ele
   actua com o cursor parado, e acumula com o número de eventos (espec §10.6, §14.3). Os outros
   cinco modos com arrasto zero não mexeriam um vértice.
2. ⚠️ **`repeticoes_por_nan`** no cabeçalho diz quantas vezes aquela corrida foi **refeita em cena
   nova** por ter devolvido `NaN`. A causa está diagnosticada e é do **harness**, não do produto: o
   ponto de superfície sob o cursor só existe depois de um quadro de sobrevoo, e um traço scriptado
   que chegue antes lê lixo (espec §17). Todas as `186` corridas do corpus final estão **livres de
   `NaN`** — o montador não escreve fixture com `NaN`.
3. ⭐ **`dispersao_entre_realizacoes`** é `0,00000000` em **todas** as fixtures que têm duas
   realizações: o pincel é **determinístico ao bit**. ⇒ a barra de paridade não precisa de tolerância
   para ruído do alvo (espec §19.3).
   ⚠️ As seis fixtures `sinal_*` não têm o campo — foram corridas uma vez só, de propósito, para
   fechar o sinal do avanço nos dois sentidos (espec §9.1).

## A prova do fatiamento

As `7` fixtures `*.porpasso` trazem `prova_do_fatiamento`, que é o desvio máximo entre o **último**
bloco truncado e a corrida cheia correspondente. Ele é **`0,00000000` nas sete**. ⇒ o resultado é
função do arrasto **TOTAL** e não do caminho nem do número de eventos — nos cinco modos conduzidos
pelo arrasto (espec §14.2). ⛔ O `SMOOTH` é a excepção e a série dele mostra-o: os blocos **crescem**
com o número de passos.

## Formato

Texto, uma grandeza por linha no cabeçalho, depois `caminho N` + `N` linhas `c x y z`, depois
`vertices N` + `N` linhas `d x y z` (ou, no `.porpasso`, um `passo k` + bloco `d` por truncagem).
Tudo com `%.8f`. As chaves são **vocabulário do domínio** (SKILL §3.E) — ⛔ nenhum nome interno do
alvo entra num cabeçalho, num nome de ficheiro ou numa pasta.
