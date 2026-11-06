# SFX Lab — W2: a rodada de medição (05/11)

Instrumento: [`ferramentas/sonda_w2/corre.sh`](ferramentas/sonda_w2/corre.sh), projeto à parte (compila em qualquer alvo sem arrastar o app). Uma rodada: variantes
no mesmo processo, intercaladas, resumo = o mínimo. Perfil `smoke`, Ryzen 9950X, `load` 5–25 (outras linhas a compilar). Decisão do motor: [ADR-0198](../architecture/decisions/0198-o-motor-do-sfx-lab-e-kernels-proprios-sobre-det-math.md).

## `aliasing`

Régua: [`tests/it/regua.rs`](../../crates/ph2d-sfx-dsp/tests/it/regua.rs). Um oscilador não tem nada abaixo da fundamental: o que aparece ali é harmônico que dobrou em Nyquist. dB sobre o sinal.

| Serra, varredura 2 → 20 kHz em 2 s, pior janela | dB |
|---|---|
| Ingênua | −7,8 |
| polyBLEP de dois pontos (o kernel da sonda de W0) | −33,6 |
| Tabela, raio 8, corte 0,8 · 0,9 · 0,95 | −97,9 · −93,4 · −102,1 |
| **Tabela, raio 16, corte 0,9 (o produto)** | **−102,1** |
| Tabela, raio 32, corte 0,9 | −102,1 |

Raio 16 é onde a tabela pára de melhorar; a −102 dB a régua lê o próprio ruído de janela. O corte de 0,9 × Nyquist é a transição do filtro, e por isso `MAX_HZ` = 21 600 Hz.

| O produto | Pior janela, dB |
|---|---|
| Seno, triângulo, serra, quadrada, varredura 2 → 20 kHz | −99,7 · −99,8 · −101,6 · −100,4 |
| FM de índice 50 a 7 kHz, modulador igual à portadora | −87,3 |

**Limite do gate: −80 dB**, entre o pior medido (−87,3) e o polyBLEP (−33,6), para qualquer regressão real aparecer.

⛔ **Recusa medida — interpolar por cima do salto:** a primeira tabela guardava o degrau menos o salto ideal e interpolava em linha reta através do salto. Mediu **−21 dB, pior que polyBLEP**; o erro é raro e enorme (uma amostra em 256 por descontinuidade). Cura: guardar o degrau suave e subtrair o salto por conta exata. A rampa do triângulo tem o mesmo cuidado, e a soma pela esquerda mediu −55 dB; trapézio, −99,8.

⛔ **Recusa medida — seno de fixture em `f32`:** a fase `i as f32 × …` tem ruído próprio a −44 dB; a régua lia a fixture em vez do kernel. Fixtura em `f64`.

### O oráculo (`sox`, só executado; triagem de licença em [02_W0_medicao.md](02_W0_medicao.md))

Passa-banda de 20 Hz a 0,8 × fundamental e RMS do que sobra, sobre o RMS do arquivo, sem nenhuma conta nossa. Fixture com cabeçalho: [`fixtures/aliasing_sox.tsv`](../../crates/ph2d-sfx-dsp/tests/it/fixtures/aliasing_sox.tsv); o gate `a_regua_bate_com_o_sox` compara linha a linha (tolerância 3 dB, abaixo de −110 os dois leem ruído numérico).

| | 5 kHz | 9 kHz | 15 kHz |
|---|---|---|---|
| Serra | −137,6 | −125,7 | −121,0 |
| Quadrada | −138,3 | −131,5 | −125,9 |
| Triângulo | −134,4 | −122,0 | −114,6 |
| **Serra ingênua do próprio `sox` (controlo)** | | | **−8,6**; a nossa régua lê −8,6 sobre a mesma forma de onda |

⚠️ Duas armadilhas do instrumento, ambas achadas pelo controlo: o `stat` escreve 6 casas (o filtrado vira `0.000000`), e subir 60 dB para ler satura o sinal forte (a ingênua leu −49,8). O script só sobe o que está abaixo de 0,001.

## Custo de `render` (ms, mínimo de 7 rodadas)

Mesma estrutura de W0 (2 corpos, 2 ruídos, 2 tons, 2 FM; pior = 8 corpos de 128 modos), **não os mesmos kernels**: W0 era a sonda.

| Receita de 2 s | W0 | W2 sem cache | Edita 1 camada | Só o ganho |
|---|---|---|---|---|
| Típica | 9,61 | **7,04** | 0,47 (ruído) · 3,16 (corpo) | 0,08 |
| Pior | 38,79 | **23,86** | **3,97** | 0,08 |

**O kill-criterion do plano (30 ms, proposto) está cumprido sem o cache**, e com ele a edição cai 6×. O número certo continua a ser o do ouvido do dono, em W3. Cache: [`RenderCache`](../../crates/ph2d-sfx-dsp/src/lib.rs); o gate `o_cache_da_os_mesmos_bytes_e_refaz_so_a_camada_editada` compara com o render sem cache a cada passo.

Por que melhorou sem SIMD: o ressonador pára quando cala (2⁻²⁴, um passo de quantização perto de 1,0), o que também impede subnormais, lentos e dependentes do modo da thread.

| Uma camada de 2 s | ms |
|---|---|
| Corpo 32 modos, t60 4 s | 3,29 |
| Corpo 128 modos, t60 4 s | 4,17 (os modos acima da banda não existem) |
| Corpo 16 modos, t60 0,2 s | 0,26 (cala cedo) |
| Ruído, corte fixo · varrendo | 0,44 · 0,60 |
| Serra de 15 kHz | 2,25 |

### Corpo: de onde vem o limite de modos

Sem teto próprio: o que limita é a banda. Base mínima `MIN_BODY_HZ` = 20 Hz, o começo do audível, e `MAX_HZ` em cima: um corpo harmônico tem no máximo 1 080 modos, com `modes` em `u32::MAX`.

| O pior corpo: 1 080 modos, sem amortecer | 2 s, t60 4 s | 120 s, t60 120 s |
|---|---|---|
| ms | 158 | 9 370 |

⚠️ Passa de 30 ms de longe, e é o caso extremo. W3 precisa do render fora da thread da tela.

### Gravação: de onde vem `MAX_SEMITONES = 48`

2 s de saída, material de 10 s a 44,1 kHz. Cada oitava acima dobra os toques do filtro.

| Semitons | −48 | −24 | 0 | +12 | +24 | +48 |
|---|---|---|---|---|---|---|
| ms | 9,07 | 9,06 | 9,04 | 16,35 | 32,23 | 39,96 |

⚠️ **+24 em diante uma camada de gravação sozinha passa dos 30 ms propostos**, ao ser editada. O cache protege as outras edições. Fica para o ouvido de W3 decidir; não há curto-circuito hoje. Mesma taxa e 0 semitons: cópia, sem filtro nenhum.

### Duração: de onde vem `MAX_SECONDS = 120`

O recurso é a RAM do cache: 4 B × 48 000 × s × camadas.

| A pior receita, sem cache | 2 s | 30 s | 120 s |
|---|---|---|---|
| ms | 25 | 51 | 98 |
| MB no cache (8 camadas) | 3 | 46 | 184 |

Acima disso, `MAX_BYTES` = 1 GiB recusa o render antes de alocar (`RenderError::TooLarge`): o recurso é a RAM da menor máquina do projeto, 8 GiB. Sem ele, mil camadas de 120 s levaram 48 s e pediram 23 GB. O tempo não é o que limita, e o teto não é do motor: é um som de jogo, e 184 MB por receita em memória é o que um artista notaria. Se W5 pedir som longo, reabra com esta tabela.

## Bytes entre alvos

Um hash do conjunto (as 8 receitas de fábrica e a de gravação), igual em x86-64 de base, x86-64-v3 (AVX2 e FMA no compilador) e wasm32 no `node`: `86ea4a05df14620b`. Binário dos hashes: nenhuma função de matemática da libm do sistema importada (`nm -D`).

⛔ **NÃO PROVADO: Intel e ARM.** Esta máquina é AMD. Para fechar, no Mac, depois da integração: `cd docs/SfxLab/ferramentas/sonda_w2 && cargo run --profile smoke --bin hashes` tem de terminar em `conjunto 86ea4a05df14620b`.

## Sonda da cena do dono

`shells/game-scenes/tests/it/receitas.rs::sonda_cada_receita_sai_do_mixer`: cada objeto da cena pelo mesmo `play_target` do botão Preview, sem dispositivo, contra o buffer de `render`.

| Som | Duração | Render | Pico (dBFS) | Resíduo contra o render |
|---|---|---|---|---|
| Metal Hit | 2,00 s | 7,4 ms | −6,6 | −152 dB |
| Wood Knock | 0,50 s | 0,4 ms | −7,7 | −152 dB |
| Glass Ping | 1,20 s | 1,4 ms | −5,9 | −152 dB |
| Drum Thump | 0,80 s | 3,2 ms | −6,1 | −152 dB |
| Impact | 1,20 s | 6,0 ms | −3,6 | −152 dB |
| Whoosh | 0,80 s | 0,9 ms | −7,2 | −152 dB |
| Laser | 0,45 s | 0,4 ms | −7,7 | −152 dB |
| FM Bell | 2,00 s | 2,2 ms | −6,0 | −152 dB |
| Recording Down | 0,24 s | 3,0 ms | −5,3 | −152 dB |

**Nada disto diz se soa bem.** O corpo vibrante só tem o kill-criterion do ouvido.
