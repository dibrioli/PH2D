# 06 — O PLANO do que falta no pincel de tecido

> ⚠️ **Leia isto ANTES de pegar qualquer item.** Cada linha traz o número que a define, o que já foi
> tentado e medido, e a pergunta exacta que a destrava — porque **onze** explicações já foram
> construídas, medidas e refutadas nesta linha, e reconstruir uma delas é trabalho pago duas vezes.
>
> A vitória e as réguas: [`05_a_vitoria_medida.md`](05_a_vitoria_medida.md).
> A espec clean-room (atestada): [`SPEC_cloth_brush.md`](../cleanroom/SPEC_cloth_brush.md).
> ⛔ **Quem implementa NUNCA abre o fonte do alvo** — as perguntas vão por subagente-E, pelo INBOX.

## §1 — O estado, em números

`29` dos `56` traços do oráculo estão dentro da barra de paridade (`0,13`) e `7` saem ao bit. Os
`27` abertos **NÃO são uma família** — as duas réguas construídas para os agrupar foram refutadas
(§3). Do pior para o melhor:

| # | traço | erro | família |
|---|---|---|---|
| 1 | `plano_apertar_ponto_radial_local` | `1,380` | **aperto** (⚠️ regime caótico do alvo — §4) |
| 2 | `plano_apertar_ponto_radial_local_origem` | `1,079` | idem |
| 3 | `plano_apertar_linha_radial_local` | `1,024` | idem |
| 4 | `plano_empurrar_plano_local` | **`0,944`** | **Push** ← *o maior número que é lei a sério* |
| 5 | `plano_apertar_ponto_plano_local` | `0,613` | aperto |
| 6 | `esfera_apertar_linha_radial_dinamica` | `0,576` | aperto + esfera |
| 7 | `plano_expandir_radial_local_1passo` | `0,560` | **Expand** (⚠️ razão com denominador minúsculo) |
| 8 | `esfera_expandir_radial_dinamica` | `0,557` | Expand |
| 9 | `esfera_apertar_ponto_radial_dinamica` | `0,542` | aperto + esfera |
| 10-12 | `plano_gancho_radial_local_*` | `0,388`–`0,420` | **Snake Hook** a partir do passo 3 |
| 13-19 | os restantes de esfera e plano | `0,18`–`0,38` | Push · Inflate · esfera |

## §2 — A FILA, por valor

### ⭐ 1.º — o PUSH (`0,944` no plano, `0,329` radial, `0,303` na esfera)

**O que se sabe.** O traço de UM passo sai **ao bit**; o oráculo não inverte faces nesses traços e
não comprime pares (`0,89`–`0,95`) ⇒ ⛔ **não** é o regime caótico do §4. O Push é o único modo cuja
direcção vem da **normal da área**, que é a única grandeza dele que muda entre passos. No dump por
passo (`plano_empurrar_radial_local_origem`) o erro nasce no passo 3 e o **PICO fica noutro sítio**:
`0,13R`–`0,24R` do cursor em nós contra `0,40R`–`0,58R` no oráculo — *a mesma assinatura que o Snake
Hook tinha antes do Q9, e ali a causa foi o CENTRO estar noutro ponto.*

**A resposta do especificador (Q12.1) está na espec §4.2-bis e ainda NÃO foi implementada:** a
direcção é reavaliada a cada passo sobre a malha deformada, amostrada num disco de **metade** do
raio, com um peso próprio e um desempate por dois conjuntos de amostras (ganha o primeiro que seja
não-vazio **e** de soma não-nula, avaliado um a um, nunca a mistura); se nada qualificar, é o vector
nulo e a força é zero. O factor que a multiplica é um vector de três números fixado no pen-down.
⇒ **Este é o item mais barato e de maior valor da fila: a lei já está escrita e atestada.**

⚠️ **O nosso `normal_area` é a soma não ponderada das normais no raio INTEIRO** — três diferenças de
uma vez (raio, peso, desempate).

### ⭐ 2.º — a ESFERA, o que sobra depois do Q12 (`0,18`–`0,58`)

**O que se sabe.** Nada no alvo pergunta pela curvatura; o que separa a esfera do plano são seis
grandezas que **degeneram** numa folha vista de frente, e a espec lista-as na §4.6. A de maior
alcance já está implementada (a projecção do `δ` no plano do ecrã, Q12) e melhorou três traços.
⏳ **As outras cinco continuam por conferir uma a uma** — cada uma é uma medição contra a fixture de
esfera correspondente, e a §4.6 diz qual é qual.

### ⭐ 3.º — o SNAKE HOOK a partir do passo 3 (`0,388`–`0,420`)

**O que se sabe.** O Q9 curou o passo 2: o pico passa a cair no vértice do oráculo (`0,86R` contra
`0,86R`) e `c0` lê `0,2043` contra `0,1971`. No passo 3 o nosso `max` é `0,4460` contra `0,3439`.
⇒ a lei do centro está certa e sobra **acumulação**. ⏳ A pergunta ainda não foi feita.

### 4.º — o INFLATE (`0,253` no plano, `0,378` na esfera)

Traço de um passo **ao bit**, sem inversão e sem compressão. Dump por passo entregue
(`plano_inflar_radial_local_origem`): o erro nasce no passo 3 e o nosso `c0` fica **abaixo** do
oráculo (`0,2413` contra `0,2629`) com o pico no sítio certo. ⇒ é magnitude, não lugar.

### 5.º — o EXPAND (`0,192` · `0,560` no de um passo · `0,557` na esfera)

⚠️ **Antes de pegar, olhe a UNIDADE:** os deslocamentos deste modo são `0,0019`, e `0,560` é uma
razão com denominador minúsculo — o erro **absoluto** é `0,0011`, que são `2,3 %` de uma aresta.
*Pode não haver defeito nenhum aqui, e a primeira coisa a fazer é uma régua que não divida por um
número pequeno.*

### 6.º — o aperto FORA do regime caótico

Os traços de aperto com força alta são o §4 (decisão do dono). Mas o
`plano_apertar_ponto_plano_local` (`0,613`) e o `esfera_apertar_linha_radial_dinamica` (`0,576`)
merecem a régua das faces invertidas antes de serem tratados como lei em falta.

## §3 — ⛔ RECUSAS MEDIDAS — não as reconstrua

| o que foi tentado | o número que o matou |
|---|---|
| o anel-1 pela **triangulação** | acerta o Arrastar *Local* e derruba o *Global* de `0,6457` para `0,2699` |
| **`PH2D_VARREDURAS=10`** como lei do *Local* | bit-idêntico à construção dupla nos modos de força e **diverge** nos de âncora |
| o filtro de raio na criação de restrições da área *Dynamic* | `0,181 → 0,182`, e as outras nove inalteradas |
| o peso da **normal** por vértice (área contra uniforme) | 19 traços, 18 inalterados |
| a **direcção do aperto medida no repouso** | melhora o plano (`1,380 → 1,012`), piora a esfera (`0,542 → 0,939`) |
| a **trava** que impede o vértice de ultrapassar o alvo | **não é inerte**: parte os traços de um passo que hoje saem ao bit |
| o **plano de queda pelo centro da área** (leitura literal da §4.4) | `empurrar 0,944 → 1,250`, `arrastar 0,233 → 0,716` |
| a projecção do `δ` no plano **tangente do pen-down** | `agarrar 0,265 → 0,605`, `gancho 0,351 → 0,663` |
| **faces invertidas** como régua de classificação | não discrimina: o arrasto tem `41`–`57` e bate a `0,071` |
| a **compressão** do par mais apertado como régua | explica a família do aperto e nada mais |
| a regressão do §9 nº 20 da espec como causa do aperto | foi **fechada em 2024**, dois anos antes da versão que gravou as fixtures |

## §4 — ⛔⛔ A DECISÃO que é do DONO (e não há terceira saída)

Ao apertar com força alta, a inversão nasce no **primeiro** passo, antes de a relaxação correr, logo
nenhuma afinação do solver a evita. As duas saídas, nas frases que a espec §5.2-ter fixa:

- **(a) reproduzir** — o retalho debaixo do cursor vira do avesso, as faces atravessam-se e a
  superfície fica com um nó que nada desfaz depois; é o que o alvo faz hoje, e apertar com força
  baixa continua limpo. **É o que shipa agora.**
- **(b) limitar** — o aperto nunca ultrapassa o ponto para onde puxa, o nó não aparece em força
  nenhuma, e a nossa saída deixa de casar com a do alvo exactamente nos traços fortes.

⚠️ **O alvo sabe que (a) é defeito dele** — são duas entradas ABERTAS do tracker dele.

## §5 — Os instrumentos que já existem (⛔ não construa outro sem olhar)

```
cargo test --release -p ph2d-cloth --test oraculo_do_pincel <sonda> -- --ignored --nocapture
```

| sonda | o que devolve |
|---|---|
| `sonda_da_paridade_com_o_oraculo` | o corpus inteiro: movidos, máximos, erro e a razão, **nos dois lados** |
| `sonda_passo_a_passo` (`PH2D_TRACO=<nome>`) | passo a passo: o anel imediato, o **erro do passo**, onde está o **pico**, o vector do vértice do pen-down |
| `sonda_dos_artefatos_do_oraculo` | espinho · rasgo · estica · **faces invertidas** · **compressão**, nos dois lados |
| `sonda_do_perfil` · `sonda_da_cadeia_com_parede` | o perfil radial e a cadeia 1D |

Experiências por env, para bissecar: `PH2D_VARREDURAS` · `PH2D_ORDEM` (`inversa`, `celula:<n>`) ·
`PH2D_PARES=0` · `PH2D_TRI` · `PH2D_ESC_PHI` · `PH2D_ESC_RET`. No produto: `PH2D_CLOTH_LAW=vbd`.

## §6 — O PROTOCOLO, em quatro passos

1. **Meça** com a sonda por passo até localizar o defeito num vértice, num passo, ou numa grandeza.
2. **Refute o que puder sozinho** — metade das perguntas morre aqui, e cada refutação vai para o
   INBOX com o número.
3. **Escreva a pergunta no INBOX** (`docs/3D/cleanroom/INBOX_blender-cloth.md`) com a tabela dentro,
   e despache um **subagente-E**. ⛔ Contrato de retorno: uma frase, cinco linhas funcionais, zero
   identificador do alvo.
4. **Despache o R-pré** antes de LER a emenda, implemente, meça outra vez, e escreva o gate com a
   prova de mutação.
