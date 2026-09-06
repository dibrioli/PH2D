# Fixtures — traços do pincel de tecido do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes arquivos são os vectores de teste da [`SPEC_cloth_brush.md`](../../SPEC_cloth_brush.md) §10**
— um traço scriptado por modo de deformação (e por variante de solver), com as posições de
repouso e as posições depois do traço.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: uma grelha plana `64×64` de lado `3,0` (4 225 vértices) e uma esfera UV `96×64` de raio `1` (6 082 vértices). ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário Blender 5.2.1 LTS, corrido pelo E **fora da árvore** (`~/Referencias/blender-cloth/oracle/`, ⛔ negado ao I) com um traço scriptado; o pincel usado é um preset do binário **só para existir um pincel de tecido activo** (a API não deixa criar+activar um de raiz), com TODOS os parâmetros reescritos para os valores do cabeçalho de cada fixture |
| **Estatuto legal** | ⭐ **dados** — «the output from the Program is covered only if its contents constitute a work based on the Program» (GPLv2 §0): posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do I (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-05 |

## O traço

Vista ortográfica; o cursor anda em linha recta ao longo de `+X`, comprimento `0,6`, em `passos`
passos iguais (o 1.º passo nunca simula — espec §1); no plano, sobre a face de cima (`z = 0`);
na esfera, sobre o equador visível (`y < 0`). Raio do pincel em espaço de objecto `0,35`
(≈ 7,5 arestas da grelha); força `1,0` (salvo `_forca05`), pressão `1`, curva *Smooth*, dureza `0`,
área *Local* (salvo indicação), limite `2,5`, banda `0,75`, massa `1`, amortecimento `0,01`,
plasticidade `0`, pino desligado, sem colisões, sem gravidade — i.e., **as omissões do código**
(espec §8.1), não as dos presets (§8.2).

⚠️ **Duas coisas do harness que mudam a leitura de uma fixture:**
- o centro da área *Local* é o ponto da superfície sob o cursor **no hover antes do pen-down** — o
  harness move o cursor do sistema para o pixel do pen-down e deixa a janela redesenhar antes do
  traço, e é por isso que o centro coincide com o 1.º ponto do caminho (sem isso, ficava num ponto velho
  e a simulação nascia noutro sítio — foi medido e a matriz refeita);
- nas variantes `_1passo` o caminho tem **dois** pontos, logo nos modos de âncora (Grab, Snake Hook)
  o passo simulado carrega **o percurso inteiro de `0,6`** de uma vez; nos modos de força o percurso
  só dá a DIRECÇÃO, e a magnitude é a da espec §4.1.

## O formato (texto, `gzip`, vocabulário do domínio)

`<superficie>.repouso.txt.gz` — uma vez por superfície:
```
vertices <N>
v <x> <y> <z>        # N linhas, índice = ordem
```
`<superficie>_<modo>_<falloff>_<area>[_<variante>].deformado.txt.gz` — por corrida:
```
superficie plano|esfera · modo · falloff_da_forca radial|plano · area local|global|dinamica
raio · limite · banda · massa · amortecimento · plasticidade · pino · forca · curva · passos
movidos <n>  max_deslocamento <d>          # recontados pelo verificador
caminho <k>  +  k linhas  c <x> <y> <z>    # os pontos do cursor, em espaço de objecto
vertices <N> +  N linhas  d <x> <y> <z>    # as posições DEPOIS do traço, mesma ordem do repouso
```

## O verificador

`python3 verifica_traco.py` (neste diretório) relê tudo, reconta `movidos` e `max_deslocamento` e
compara com o cabeçalho — **exit 0 = coerente**. Ele não carrega algoritmo nenhum: é a prova de que
o ficheiro diz o que contém.

## As corridas

| fixture | modo | passos | movidos | máx |u| |
|---|---|---|---|---|
| `plano_arrastar_plano_local` | arrastar | 12 | 2146 | `0.8996` |
| `plano_arrastar_radial_dinamica` | arrastar | 12 | 2508 | `0.612821` |
| `plano_arrastar_radial_dinamica_preset` | arrastar | 12 | 2455 | `0.329617` |
| `plano_arrastar_radial_global` | arrastar | 12 | 4225 | `0.644607` |
| `plano_arrastar_radial_local` | arrastar | 12 | 2144 | `0.331637` |
| `plano_arrastar_radial_local_1passo` | arrastar | 2 | 171 | `0.09917` |
| `plano_arrastar_radial_local_2steps` | arrastar | 3 | 1438 | `0.135888` |
| `plano_arrastar_radial_local_amort1` | arrastar | 12 | 2141 | `0.219903` |
| `plano_arrastar_radial_local_amort05` | arrastar | 12 | 2142 | `0.254386` |
| `plano_arrastar_radial_local_massa2` | arrastar | 12 | 2143 | `0.154596` |
| `plano_arrastar_radial_local_mass2_1step` | arrastar | 2 | 171 | `0.049585` |
| `plano_arrastar_radial_local_pino` | arrastar | 12 | 2144 | `0.323528` |
| `plano_arrastar_radial_local_plast05` | arrastar | 12 | 2141 | `0.234305` |
| `plano_arrastar_radial_local_forca05` | arrastar | 12 | 2139 | `0.073252` |
| `plano_arrastar_radial_local_str05_1step` | arrastar | 2 | 168 | `0.024792` |
| `plano_expandir_radial_local` | expandir | 12 | 2134 | `0.011523` |
| `plano_expandir_radial_local_1passo` | expandir | 2 | 848 | `0.001902` |
| `plano_agarrar_plano_local` | agarrar | 12 | 2146 | `0.307644` |
| `plano_agarrar_radial_local` | agarrar | 12 | 2139 | `0.16991` |
| `plano_agarrar_radial_local_1passo` | agarrar | 2 | 1324 | `0.134099` |
| `plano_agarrar_radial_local_24passos` | agarrar | 24 | 2142 | `0.158543` |
| `plano_agarrar_radial_local_2steps` | agarrar | 3 | 1872 | `0.146115` |
| `plano_agarrar_radial_local_amort06` | agarrar | 12 | 2131 | `0.131488` |
| `plano_agarrar_radial_local_preset` | agarrar | 12 | 4123 | `0.132623` |
| `plano_inflar_radial_local` | inflar | 12 | 2146 | `0.317159` |
| `plano_inflar_radial_local_1passo` | inflar | 2 | 171 | `0.09917` |
| `plano_apertar_linha_radial_local` | apertar_linha | 12 | 2135 | `0.100451` |
| `plano_apertar_linha_radial_local_1passo` | apertar_linha | 2 | 156 | `0.087609` |
| `plano_apertar_ponto_plano_local` | apertar_ponto | 12 | 2146 | `0.623884` |
| `plano_apertar_ponto_radial_local` | apertar_ponto | 12 | 2146 | `0.325769` |
| `plano_apertar_ponto_radial_local_1passo` | apertar_ponto | 2 | 171 | `0.09917` |
| `plano_empurrar_plano_local` | empurrar | 12 | 2146 | `0.520138` |
| `plano_empurrar_radial_local` | empurrar | 12 | 2145 | `0.258986` |
| `plano_empurrar_radial_local_1passo` | empurrar | 2 | 171 | `0.069419` |
| `plano_gancho_radial_local` | gancho | 12 | 2140 | `0.09155` |
| `plano_gancho_radial_local_1passo` | gancho | 2 | 1452 | `0.489383` |
| `plano_gancho_radial_local_24passos` | gancho | 24 | 2142 | `0.02932` |
| `plano_gancho_radial_local_2steps` | gancho | 3 | 1950 | `0.364813` |
| `plano_gancho_radial_local_amort06` | gancho | 12 | 2135 | `0.063396` |
| `esfera_arrastar_radial_local` | arrastar | 12 | 6050 | `0.531513` |
| `esfera_expandir_radial_local` | expandir | 12 | 6050 | `0.05374` |
| `esfera_agarrar_radial_dinamica` | agarrar | 12 | 1862 | `0.236625` |
| `esfera_inflar_radial_local` | inflar | 12 | 6050 | `0.167462` |
| `esfera_apertar_ponto_radial_local` | apertar_ponto | 12 | 6050 | `0.330488` |
| `esfera_empurrar_radial_local` | empurrar | 12 | 6050 | `0.435752` |
| `esfera_gancho_radial_local` | gancho | 12 | 6050 | `0.262211` |

**46 traços.** ⚠️ Grab Local e Pinch Perpendicular Local **na esfera** não foram gravados (o hover do harness não fixou o centro da área Local nessas duas — os dois modos estão medidos no plano; o Grab também na esfera Dynamic).
