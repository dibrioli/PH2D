# Handoff de continuação — Luz 2D, W2 → W3 (2026-10-08)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (main de 07/10, sem rebase pendente).
- **Commits da W2:** `05d18ead5` (tabela do G12 como porta partilhada) · `f454e0066` (a crate `ph2d-light2d-gpu` + gates) · este doc.
- **Estado:** W2 FECHADA (onda, não a linha). Nada para o Enio ver ainda: o solver existe e mede-se, mas nada na tela o usa (isso é a W3).

## O que existe

`crates/ph2d-light2d-gpu` (depende de `ph2d-gpu`, `ph2d-light2d-ecs`; `ph2d-light2d-ref` e `half` só em dev):

| Porta | O quê |
|---|---|
| `Solver::novo(&GpuContext, Config)` · `quadro` · `gravar(enc)` | C0 → C1 → C2 num quadro; `gravar` grava no encoder de quem chama (a W3 usa esta) |
| `Solver::fluencia()` | a FLUÊNCIA rgb por sonda da c0 (`Rgba16Float`, um texel por texel da grade efetiva) |
| `Config::padrao(grade)` · `padrao_para(device, grade)` | as escolhas da rodada (abaixo); `padrao_para` liga a aritmética f16 onde há `SHADER_F16` |
| `Grade { origem_m, texel_m, largura, altura }` | ancorada no mundo; `config().grade` é a EFETIVA (estendida a múltiplos de `2^(n−1)`) |
| `PassList::de(cfg, com_tempos)` · `bytes_alocados` · `bytes_movidos` · `bytes_por_quadro(cfg)` | a lista que CRIA texturas e buffers fixos e responde os bytes |
| `RelatorioC0 { fora, bytes_da_geometria }` · `o_que_fica_fora(cena, grade)` | o que a C0 não captura, DITO: `PontoSemArea` (C3), `MenorQueOTexel`, `ForaDaGrade` |
| `Solver::tempos(gpu) -> TemposMs` | carimbos por passe (C0, C1, cada cascata) do último quadro — **bloqueia** (sonda/teste) |
| `QUEDA_WGSL` · `fonte_do_rc` | a gêmea WGSL da `queda`, concatenada; texto único (censo no G12) |

Instrumentos versionados (`tests/it/`): `medida_luz2d.rs` (a rodada), `diagnostico.rs` (o MODELO de força bruta
pela grade capturada — separa erro de captura de erro de cascata — e o erro por faixa de distância).

## A representação da C0 (fixada na W2 — a W3 captura sprites NELA)

Três alvos, uma chamada de desenho: `emissao` (`Rgba16Float`, max: radiância do DONO + `1/alcance`),
`cobertura` (`R16Float`, soma: 8×8 subamostras por texel) e `tau_dentro` (`R16Float`, ×: `∏(1−α)` dos donos).
Dono = cobertura própria ≥ ½. A marcha diz **dentro ⟺ cobertura bilinear ≥ ½** (um `textureGather`) e lê
o material do vizinho mais coberto. ⇒ **o alfa por texel de um sprite É a cobertura**: a captura de sprite
da W3 escreve nos mesmos três alvos.

Por quê (medido com o modelo de força bruta, 4 096 direções, penumbra, erro contra o gabarito):

| representação | Alto | Médio | Baixo |
|---|---|---|---|
| texel mais próximo, binário pelo centro | 0,097 (passo ¼: 0,158) | 0,433 | 0,160 |
| texel mais próximo, com cobertura | idem — o raio rente recolhe o MÁXIMO da cobertura do caminho | | |
| **limiar da cobertura interpolada** | **0,018** | **0,052** | **0,134** |

Com o texel mais próximo, um passo mais fino PIORA (toca mais bordas): é a assinatura da dilatação da silhueta.

## Escolhas (`Config::padrao`) e a rodada que as fixou

RTX 5060 Ti · Vulkan · driver 615.71.09 · perfil smoke · 20 rodadas intercaladas, mínimo do relógio da
placa · `loadavg` 25–43 (a placa em exclusão, `PH2D_GPU=1`). Cena: réplica da HRC Fig. 8 (512², luz de 10 px,
oclusor de 14 px). RMSE relativo = `√média(gpu−ref)² / média(ref)`, texel a texel.

| variante (Alto) | C2 ms | erro penumbra | erro céu e vidro | decisão |
|---|---|---|---|---|
| **padrão: bilinear · intervalo 1 · passo 1 · JFA 2 · f16 nas cascatas** | **1,42** | **0,090** | **0,013** | ✔ |
| fusão clássica | 0,45 | 0,209 | 0,032 | anel (região livre 0,126 × 0,040) |
| vizinha intercalada (nossa reconstrução) | 0,44 | 0,449 | — | K6: 5× pior ⇒ aceitar o custo |
| intervalo ½ (7 cascatas) | 1,69 | 0,095 | 0,021 | pior e mais caro |
| intervalo 2 | 1,33 | 0,116 | 0,010 | troca: perde na penumbra |
| passo ½ | 1,75 | 0,107 | 0,012 | só ajuda no Baixo (0,233 → 0,184) |
| JFA 1 · JFA 4 | 1,49 · 1,45 | 0,090 · 0,090 | — | JFA 1: C1 2,3×; JFA 4 perde no enxame denso |
| cascatas f32 | 1,44 | 0,090 | — | +53 % de memória por nada |
| aritmética f16 | 1,42 | 0,090 | — | igual no desktop (o ganho é do celular) |

**Por passe (padrão, 512²):** C0 0,027 ms · C1 0,032 ms · C2 1,42 ms (por cascata 0,093 · 0,125 · 0,198 ·
0,426 · 0,499 · 0,070) · **solver 1,45 ms**. Médio (256²): 0,61 ms · Baixo (128²): 0,32 ms.
**Bytes:** 7,50 MB alocados · **49,8 MB/quadro** movidos (piso de cache perfeita, da `PassList`) a 512²;
1,88 MB · 12,3 MB/q a 256²; 0,47 MB · 2,6 MB/q a 128². Geometria: 64 B por forma + 8 B por vértice, à parte.

## Gates (todos verdes; `PH2D_GPU=1`)

| gate | medido | tolerância |
|---|---|---|
| G3 penumbra (Alto · Médio · Baixo) | 0,0902 · 0,1393 · 0,2333 | medido + 10 % |
| G3 céu e vidro (vidros sobrepostos, céu, alcance finito) | 0,0126 · 0,0198 · 0,0374 | medido + 10 % |
| G3 faísca (luz de 2 px) | Alto 0,2821; Médio/Baixo: **fora** (`MenorQueOTexel` ⇒ C3) | 0,310 |
| G3 anel (região livre) | bilinear 0,0395 · clássica 0,1262 | 0,0435 (a clássica TEM de passar dele) |
| G3 precisão | f32/f32 0,0900 · f16/f16 0,0901 | a do Alto |
| G4 / **K3** (1 → 10 000 luzes, mesma geometria) | **0,42–1,37 %** | 5 % — faixa isolada do nextest + `FLAKES_DE_CARGA.md` |
| G4 interior (disco opaco 100 px ÷ 25 px) | **1,34** (sem a saída antecipada: 3,41) | 2 — mesma faixa |
| **K6** (bilinear ÷ clássica, C2) | **3,1–3,4×** > 2× | vizinha intercalada medida: 5× mais erro ⇒ aceitar o custo |
| G5 | toda alocação nova é declarada, uma vez; enchimento do driver CONSTANTE nas 3 grades: uniforme 16 → 64 B, `emissao` +736 B, resto 0 | igual nas 3 grades e < 1 KiB |
| G10 | dois quadros e dois solvers bit-idênticos | — |
| G12 | 1 000 pontos, máx. 2,4e-7; texto único (censo) | 4e-6 |

Prova de mutação: ver a secção no fim. ⚠️ As tolerâncias do G3 assumem `Config::padrao`; mudar um
padrão é refazer a rodada e a tabela do `g3_rc_contra_gabarito.rs`.

## ⚠️ ABERTO — o custo depende da GEOMETRIA (eixo S3), e a W5 tem de o enfrentar

O K3 isola o NÚMERO de luzes (mesma geometria) e passa. Com a área emissora fixa e a geometria a mudar
com N (`cenas::enxame`: um disco de 100 px → 10 000 de 1 px), o solver vai de **2,04 a 7,50 ms (+267 %)**:
a marcha por campo de distância anda texel a texel perto de bordas, e o enxame de N = 1 000 (raio 3 px,
espaço 12 px) é o pior. Já curado: um intervalo que COMEÇA dentro de algo opaco devolve `(0, 0)` (exato) —
o disco de 100 px custava 6,4 ms marchando o interior. O que resta é a variante «DDA sem SDF / mip» do
relatório (S3) e um campo de distância INTERIOR (saltar dentro de vidros: «céu e vidro» custa 3,4 ms, 2,4×
a penumbra, por andar texel a texel dentro dos vidros).

## ⛔ Recusas MEDIDAS (não reconstruir)

| tentativa | número | por quê |
|---|---|---|
| ler o texel mais próximo (binário ou com cobertura) | +4–6 % de viés na região livre; 0,097–0,433 de erro de captura | dilata a silhueta (máximo do caminho) |
| vizinha intercalada | 0,449 × 0,090 | pontilhado; o K6 manda aceitar o bilinear |
| intervalo ½ como padrão | 0,095 · 0,021 × 0,090 · 0,013, +19 % | o «ganho» inicial era o defeito de dupla contagem (abaixo) |
| recuo de UM PASSO inteiro na fronteira do intervalo | vidro 0,5 contado como 0,25 | com o intervalo de baixo mais curto que o passo, a amostra anterior caía atrás da sonda — o recuo é `min(passo, comprimento do de baixo)` |
| cascatas f32 | 0,0900 × 0,0902, +53 % mem | — |
| contadores internos do wgpu (feature `counters`) para o G5 | — | mudaria o build do wgpu do workspace inteiro; o `generate_allocator_report` do Vulkan nomeia cada alocação sem feature |

## Premissas do briefing que a medição derrubou

1. «depende de `ph2d-render`» — a W2 não usa nada dele (o `machete` reprovaria): entra na W3, com o shader de sprite.
2. «G4: mesma área» — a fixture de área emissora fixa muda a GEOMETRIA com N e mede o eixo S3, não o K3. O G4
   usa a fixture que ISOLA a contagem (`enxame_de_luzes`: as mesmas 10 000 formas, N delas emitem); a de área
   fixa fica na sonda, com o número acima.
3. «C0 rasteriza pelo centro do texel» (plano §2.4 / handoff W1) — correto como decisão de cobertura, errado
   como amostragem da marcha (tabela da representação).
4. A faísca não cabe na C0 nos degraus grossos — vem dita como `MenorQueOTexel`; é o caminho C3 (W5).

## Decisão do gap da W1 (captura de sprite por texel; vetor/Flip/Motion por tile) → **W3**

A W3 mexe no passe de sprite (o shader ganha o `@group` da luz) — capturar o sprite na C0 é o mesmo passe,
o mesmo atlas e a mesma `Sprite`; e o alfa do texel entra direto como `cobertura`. Na W2 isso seria tocar
`ph2d-render` só para a captura e de novo na W3 para receber. Vetor/Flip/Motion: o tile de DPI fixo do
`motion_glow_layer`, também na W3 (é o mesmo compositor que recebe o vetor).

## Para a integração (somam entre linhas)

Foundational: `crates/ph2d-gpu/src/context.rs` (+`SHADER_F16` na máscara pedida, quando o adaptador o tem —
a mesma regra das outras features; nada que funcionava muda). Partilhados: `.config/nextest.toml` (+2 nomes na
faixa isolada) e `docs/DevOps/FLAKES_DE_CARGA.md` (+2 linhas) — as DUAS escritas do gate
`architecture_toda_flake_de_carga_corre_na_faixa_isolada`. `Cargo.lock` (+`ph2d-light2d-gpu`). Shell: 0 linhas.
`PROJECT_SCHEMA` intocado (187). Nenhum contrato congelado tocado.

## O que a W3 precisa saber (receber a luz)

1. **Gravar no quadro:** `Solver::gravar(gpu, &mut enc, &cena)` antes do passe de sprite; amostrar
   `fluencia()` com a transformação mundo → grade (`(p − origem)/texel − ½` em coordenadas de texel, bilinear).
   `fluencia` guarda a FLUÊNCIA (2π × média); a irradiância de uma superfície de frente é outra conta — a do
   plano §2.4 (`luz_em()` em `receber.wgsl`, concatenado no sprite E no compositor do vetor; gate G7).
2. **Ancoragem (G11):** a `Grade` já é do mundo; quem a monta tem de encaixar a origem num múltiplo do
   espaçamento da cascata de TOPO (`2^(n−1)` texels) para o pan não tremer — ainda ninguém o faz.
3. **Ponto neutro (G6):** sem `Luz` nem emissor, não chamar o solver (o quadro de hoje, byte a byte).
4. `tempos()` BLOQUEIA (mapa síncrono): serve à sonda; no produto, leitura assíncrona ou nenhuma.
5. A geometria só cresce (potências de dois); o `RelatorioC0` tem de chegar à UI na W4 (recusa que só o
   terminal vê é botão mudo).
6. Comandos: gates `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-light2d-gpu --test it` (~6 s);
   rodada `… --profile smoke --test it medida_luz2d -- --ignored --nocapture`; modelo `… modelo -- --ignored`.

## Prova de mutação (agente `mutacao` + M8/M9 e o gate do interior à mão)

Pré-voo verde em todos os filtros (`g3_` 5 · `g5_` 1 · `g12_` 2 testes), âncoras contadas (= 1).
**14 de 15 mutações sangraram; os 3 controlos (equivalentes) sobreviveram.**

| # | mutação | sangrou em |
|---|---|---|
| M1 | padrão sem bilinear fix | G3 penumbra 0,209 · céu 0,032 · faísca 0,430 |
| M2 | só opaco atenua (vidro transparente) | G3 céu e vidro 0,106 |
| M3 | sem céu | G3 céu e vidro 0,993 |
| M4 | sem queda (alcance finito ignorado) | G3 céu e vidro 0,069 |
| M5 | limiar da cobertura ½ → ¼ | os 5 do G3 (penumbra 0,331 · anel 0,108) |
| M6 | opacidade por τ absoluto, não por ENTRADA | G3 céu e vidro 0,0147 > 0,014 (⚠️ margem de 5 %: os vidros sobrepostos são a única fixture que separa as duas leis) |
| M7 | sem a saída antecipada dentro de opaco | **sobreviveu ao G3** (exatidão: 0,0902 → 0,0882) ⇒ gate de CUSTO novo `g4_o_interior_de_um_opaco_nao_custa`: 1,34 → 3,41 > 2, sangra |
| M8 | textura com rótulo fora da `PassList` | G5 «`luz2d/tex/emissao` alocado 2 vezes» |
| M9 | textura SEM rótulo | G5 «`Unlabeled texture` (16 384 B) foi alocado e a PassList não o declara» |
| M10 | fórmula de bytes da `Textura` errada | G5 declarado 1 048 576, alocado 524 288 |
| M11 | buffer alocado maior que o declarado | G5 enchimento muda com a grade [1536, 1536, 1280] |
| M12 | `x⁴ → x³` na gêmea WGSL | G12 1,49e-1 |
| M13 | clamp inferior −1 na gêmea WGSL | G12 1,0 |
| M14 | `x⁴ → x³` no lado RUST | G12 da GPU **e** os dois G12 do gabarito |
| M15 | segunda cópia de `fn queda(` noutro `.wgsl` | G12 censo (aponta `c1_jfa.wgsl`) |

A M7 também mostra o preço exato da saída antecipada: ~2 % de exatidão na penumbra (as cascatas de cima
que começam dentro do oclusor por paralaxe deixam de «ver através»). Fica, porque o custo sem ela segue a
área de cada objeto opaco.
