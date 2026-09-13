# Fixtures — traços do pincel de POSE do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes arquivos são os vectores de teste da [`SPEC_pose_brush.md`](../../SPEC_pose_brush.md) §14**
— **69** traços scriptados, com as posições de repouso e as posições depois do traço, mais **11**
séries **por evento** (o estado depois de *cada* evento do mesmo traço, não só o final).

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas proceduralmente pelo próprio harness do E: uma **figura** plana recortada numa grelha de passo `0,025` (4 930 vértices, 4 476 faces — torso, pescoço, cabeça, braços, palmas, três dedos por mão, pernas); a mesma figura com o **pulso aberto** (uma fileira de quads removida, 4 460 faces — serve às fixtures de peças desligadas); e um **braço** tubular de 81 anéis × 16 lados, raio `0,12`, comprimento `2,0`, com tampas (1 298 vértices). ⛔ **Nenhum asset do alvo** |
| **Quem calculou** | o binário Blender 5.2.1 LTS, corrido pelo E **fora da árvore** (`~/Referencias/blender-pose/oracle/`, ⛔ negado ao I) numa sessão gráfica **virtual** (`kwin_wayland --virtual`, para não tomar o ecrã do dono), com o traço entregue por script. O pincel activo vem de um preset do binário **só para existir um pincel de pose activo** (a API não deixa criar+activar um de raiz), com **todos** os parâmetros reescritos para os valores do cabeçalho de cada fixture |
| **Estatuto legal** | ⭐ **dados** — «the output from the Program is covered only if its contents constitute a work based on the Program» (GPLv2 §0): posições de vértices de uma malha nossa não são |
| **Vocabulário** | ⭐ **do domínio, sempre** — as chaves dos cabeçalhos e os nomes dos ficheiros foram **renomeados** (SKILL §5): nenhum identificador interno do alvo aparece aqui, e o `cleanroom-sweep.sh` corre sobre esta pasta |
| **Regenerar** | ⛔ acto de **E**, nunca do I (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-13 |

## Os ficheiros

| padrão | o que é |
|---|---|
| `<superficie>.malha.txt.gz` | a malha de **entrada**: linhas `v x y z` (repouso) e `f i j k [l]`. Três: `figura`, `figura_pulso_aberto`, `braco` |
| `<fixture>.deformado.txt.gz` | cabeçalho com **todos** os controlos, depois `c x y z` (os pontos do caminho, em espaço de objecto) e `p x y z` (a posição de **cada** vértice depois do traço, na ordem da malha) |
| `<fixture>.porpasso.txt.gz` | para 11 fixtures: um bloco `evento k …` + `p x y z` por vértice, **depois de cada evento** |
| `analise.json` | índice derivado: por fixture, a superfície, o modo, quantos eventos, quantos vértices se moveram e o deslocamento máximo |

⚠️ **Leia sempre o cabeçalho da fixture, nunca uma lista de excepções** — cada `.deformado.txt.gz`
traz, sem excepção, as **19** grandezas que a definem: `superficie · modo · origem · raio ·
segmentos · desvio_da_origem · suavizacoes_do_peso · ancorado · trava_rotacao · so_conectado ·
distancia_max_entre_pecas · forca · curva · invertido · simetria_x · autosuavizacao ·
pressao_na_forca · eventos · pixels_por_unidade`, mais `mascara_*` e `pressao_por_evento` quando
existem. *Uma fixture cujo cabeçalho é completo não precisa de uma tabela de excepções ao lado — e
foi uma tabela dessas, no corpus do tecido, que teve de ser recontada três vezes.*

## O traço

Vista **ortográfica**, sem sobreposições, olhando `−Z`; o cursor anda em linha recta em espaço de
objecto, em `eventos` passos iguais. Salvo o que o cabeçalho disser: raio `0,25`–`0,30`, força
`1,0`, curva *Smooth*, `1` segmento, desvio `0`, `4` suavizações, **ancorado**, trava de rotação
desligada, «só conectado» **ligado**, sem máscara, sem auto-máscaras, sem simetria, alvo
`GEOMETRY`, sem pressão, sem auto-suavização.

⚠️ **`pixels_por_unidade` é load-bearing para o modo de torção, e só para ele** — é o único modo
cuja lei lê **pixels** (espec §5.2). Sem esse factor, uma fixture de torção não é reproduzível.

⚠️ **O harness move o cursor do sistema para o pixel do primeiro evento e deixa a janela redesenhar
antes do traço** — sem isso o ponto de aplicação nascia num sítio velho.

## Três fixtures que valem mais do que parecem

| fixture | porquê |
|---|---|
| `figura_girar_dedo_pressao03` | ⭐ **controlo negativo**: pressão `0,3` por evento, e a saída é **idêntica ao bit** à de `figura_girar_dedo`. É assim que se prova que a pressão **não** entra na força deste pincel (espec §1.3) |
| `figura_girar_dedo_origem_no_cursor` | ⭐ o pivô cai **em cima** do cursor ⇒ o alvo **não move nada** (`movidos = 0`) apesar de um arrasto de `0,6`. É a fixture que apanha uma implementação que ali produz `NaN` ou uma rotação arbitrária (espec §11.1) |
| `figura_girar_braco_ik3` + `_4ev` + `_36ev` | ⭐⭐ o **mesmo** arrasto em `12`, `4` e `36` eventos. Com 3 segmentos as três poses **diferem** (`5,3e-2`) — é a prova de que o solver carrega estado entre eventos (espec §5.1-bis). ⛔ Um gate que exija que as três concordem **reprova sobre produto correcto** |

## Como comparar

A barra **não é um epsilon de conforto** — ela sai da classe da fixture, e a tabela está na
[espec §12.3](../../SPEC_pose_brush.md). Resumo: `1e-6` absoluto no corpo normal; **nenhuma
paridade** sobre as fixtures que caem numa descontinuidade; barra **relativa** perto do polo da
escala; e **veredito** (deslocamento nulo) nas degeneradas.
