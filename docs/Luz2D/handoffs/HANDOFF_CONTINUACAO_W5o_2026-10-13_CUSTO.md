# Handoff de continuação — Luz 2D, W5o: o custo da luz (2026-10-13)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre `main`
> `757afebd58` (tudo da W5n já integrado). Commits: `dc081fa8bc` (o custo: produto, portões, sondas) · `6be9d9348e`
> `3ce1f68f6e` (docs, clippy da sonda) · `fcab331472` + `5a566833c4` (a PRECISÃO, outra família: §7) · `5049f0f0e9`
> (a ABERTURA: §8) · os de docs.
> Nada integrado, nada enviado. Método de medida: [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md) — as
> variantes são CONFIG EM EXECUÇÃO no mesmo processo, 7 rodadas intercaladas, vale o MÍNIMO (a mediana ao lado).
> Cache do driver: `__GL_SHADER_DISK_CACHE_SIZE` 16 GiB (o `GLCache` em 1,1 GB, acima do tecto antigo).

## 1. Onde estavam os «~4,4 ms do Low» — em lado nenhum

A adenda da W5n leu o `a_nitidez_da_luz` na corrida do integrador (2 testes de placa por vez). Sozinho: **Low 3,85
· High 30,15** (sem o miolo 2,76 · 25,49); por passe, o Low da W5h 2,69 → W5n 3,89 ms. O High segue a `c2` (§3).

## 2. A tabela por passe, antes × depois (RTX 5060 Ti, `smoke`, 1080p, vista de fábrica, sem o Automático)

`medida_luz2d_w5o` (load 7–8; o relógio é o da placa, mínimo ≈ mediana em todas as linhas). ms, vista + entorno:

| cena · nível | sem miolo | W5h | **W5n (antes)** | corte 1 | **W5o (1 + 2)** |
|---|---|---|---|---|---|
| vazia · Low | 2,05 | 2,69 | **3,89** | 3,36 | **2,97** |
| vazia · High | 21,89 | 22,96 | 30,93* | 30,07* | 29,76* |
| cena 5 · Low | 11,50 | 13,08 | **14,30** | 12,76 | **12,44** |
| cena 5 · High | 31,76 | 37,68 | **39,01** | 33,06 | **32,77** |

\* a `c2` destas instâncias caiu no regime lento do alocador (§3); a vista sem a `c2` desce como nas outras linhas.

Por passe (vazia · Low, a vista): `c2` 1,05 · `c3_fina` (+anel) 0,207 → 0,440 com o anel · `c3_preenche` 0,37 ·
`c3_miolo` W5n **1,067** → W5o **0,720**; entorno W5n 0,874 (miolo 0,544) → 0,306. Cena 5 · High: o entorno
**8,70 → 2,82 ms** (a grade fina dele, que ninguém lê, custava ~5,9 ms).

Fases do miolo da vista (o instrumento, vazia · Low, ms / despachos, W5n → W5o): cascata mais grosso 0,079/65 →
0,030/**1** · cascata desce 0,340/196 → 0,261/130 · ciclo mais grosso 0,235/195 → 0,056/**3** · ciclo sobe 0,191/105
→ 0,155/75 · o resto igual; **soma 1,087/605 → 0,733/247**. O miolo é limitado pelo NÚMERO de despachos (~1,2 µs cada nos níveis minúsculos; um nível de 5×3 levava 64 por
ciclo), não pelo trabalho.

## 3. ⚠️ ABERTO, o maior: a `c2` do High muda 18,4–26,6 ms com o estado do ALOCADOR

A mesma configuração High, o mesmo shader, a cena vazia: o custo da cascata holográfica depende de ONDE o
`gpu-allocator` põe os buffers dela (todos < 128 MB, sub-alocados: `hrc_t0`/`hrc_acumulado` 56 MB, `hrc_t1..t12`,
`hrc_r0/r1` 31–46 MB). Um caso por PROCESSO (`medida_luz2d_w5o_lastro`, mínimo de 12 quadros, 2× cada):

Antes de criar o High: nada (processo novo) **24,6 ms** · 256 MB–3 GiB de texturas VIVAS 24,6 · um Low ou um High
criado e LARGADO, ou 1 GiB escrito e largado **18,4** · um Low vivo 21,2 · `MemoryUsage` (blocos 8→64 MB) 20,8 ·
blocos fixos de 256 MB 21,3 · de 4 MB (tudo dedicado) 26,6.

Seis instâncias no mesmo processo, criadas em série: 18,46 · 18,47 · 18,52 · 19,81 · 20,16 · 24,55 ms — segue a
ORDEM de criação, não a variante. O portão `a_nitidez_da_luz` cria o Low antes do High e lê ~30 ms. **Não é o
miolo** (a W5n não toca na `c2`). ⛔ Acolchoar cada buffer da HRC: 4 KB · 64 KB · 1 MB não mudam nada, 2 MB tira
~3 ms — não é choque simples de endereços. Por ETAPA (experimento temporário, revertido: saltar uma etapa da HRC
e ler a queda; um caso por processo, ⚠️ com a placa partilhada — load 22–43, outra linha com o app aberto e um
agente a medir relógio —, só a ordem de grandeza vale): no regime lento a «sobe» +3,0 ms e a «desce» +3,3 (as
fusões entre níveis, que saltam pelos buffers `t`), a «traça» +1,0, soma e saída iguais. Próximo passo: os buffers da HRC numa alocação SÓ (offsets nossos) e
medir a mesma tabela; mudar as `MemoryHints` da casa é global (todo o app) e pede a sua própria rodada.

## 4. Os cortes (ambos AO BIT)

1. **`Config::so_a_c0`** (o entorno do produto, `quadro_entorno.rs`): da vista só se lê a c0 do entorno
   (`Solver::somar_entorno`); a `C3 rebote` dele lê c0/cobertura/`tau`/sementes e corre ANTES da fina — a
   fluência dele não tinha leitor. Com a flag a visibilidade, a grade fina, o anel e o miolo não correm, e o miolo
   não aloca (`niveis_do_miolo` = `None`).
2. **`c3_miolo_base.wgsl`** (`Miolo::base_no_grupo`, por omissão): os níveis `[b, topo]` cuja soma de texels cabe
   na memória de grupo do DISPOSITIVO (`max_compute_workgroup_storage_size` ÷ 52 B: 13 `f32` por texel) resolvem-se
   num grupo de 256 fios, uma barreira por varrida: `miolo_base` (a cascata) e `miolo_erro_base` (cada V-ciclo, a
   partir do defeito do nível `b`). A 1080p `b` = 7 (315 texels, CAP 315 com o limite de 16 KiB). Jacobi não soma
   ENTRE texels: a mesma conta, na mesma ordem, dá o mesmo bit.

Provas (`sonda_grava_as_fluencias_w5o`, a fluência da vista pelo caminho do PRODUTO: cena 3 com a câmera em x = 2 —
a luz de fora chega pelo entorno — e cena 5, Low e High, 80 quadros; a base 2× com o mesmo sha256): corte 1,
corte 1 + 2 e `PH2D_LIGHT2D_CORTES=0` dão 0 bytes de diferença; controlos: sem o entorno mudam 28,6 M bytes, sem o
miolo 2,0 M. ⚠️ A da α = 1 não vê o entorno (sem ele, 0 bytes mudam). A base corre: uma mutação na relaxação dela
muda 17 967 bytes; o `AMORTECE` só se vê com um corpo a encher a vista (nas cenas 3/5 o erro grosso está preso a
zero) — `sonda_grava_a_caixa_w5o` (a caixa no zoom 2,5): base desligada × ligada 0 bytes, a mutação 695 414.

## 5. Portões

- **`o_custo_do_miolo`** (crate `ph2d-light2d-gpu`, a grade do Low a 1080p 2560×1536, um disco, região parcial):
  `a_base_do_miolo_corta_os_despachos` — ≤ 260 (medido 247; controlo sem a base 605, reprova);
  `o_custo_do_miolo_desce_com_a_base` — a razão `c3_miolo` com ÷ sem a base ≤ 0,8 (medido 0,714 2×; controlo do
  custo antigo contra si 0,999, reprova) e o entorno com `so_a_c0` ≤ 0,1 da fina+vis+miolo sem a flag (medido
  0,000 contra 19–20 ms). Régua relativa, mesmo processo, intercalada.
- `a_nitidez_da_luz`: a tabela do comentário corrigida (os números da W5n eram contaminados); a régua `> 1,5×` fica.

## ⛔ Recusas MEDIDAS

| tentativa | medido |
|---|---|
| ler o custo do portão `a_nitidez_da_luz` na corrida da suíte (2 por vez na placa) | Low 13,3 contra 3,85 sozinho |
| acolchoar os buffers da HRC para fugir do regime lento da `c2` | 4 KB–1 MB: nada; 2 MB: −3 ms de 6 |
| `MemoryUsage` como cura da `c2` | 24,6 → 20,8, longe dos 18,4, e muda o app inteiro |

## 6. ABERTO, na ordem

1. A `c2` e o alocador (§3) — ±6 ms no High, o maior custo da luz que não é trabalho.
2. O anel na grade inteira: +0,23 ms na `c3_fina` (um teste de corpo por texel em ~80 % da grade) + ~0,2 nos níveis
   inteiros do miolo. Saltar ladrilhos sem corpo pede uma lista de ladrilhos e a limpeza do `miolo_fino` velho.
3. Os 247 despachos que ficam: as relaxações dos níveis 2–6 (32 por nível na cascata) — várias iterações por
   despacho com halo na memória do grupo (a mesma conta, ao bit).
4. ✅ A precisão da `vis_na_face` — §7.
5. Os da W5n §4 (4–5), W5h §3 (4–6) e W5m §4.

## 7. A precisão (commit `fcab331472`, outra família): a face vê a lâmpada do PRÓPRIO ponto

A `vis_na_face` (W5g) marchava a lâmpada a partir de `Q = P + 1,5 texel × normal`: a abertura e os raios saíam de
`Q`, e na penumbra a visibilidade varia com a posição (W5n §2: 0,001436 contra 0,000292 sem a face). **Cura**
(`Config::face_pelo_ponto` → `FACE_PELO_PONTO` do `c3_pe.wgsl`, por omissão; `PH2D_LIGHT2D_FACE_PONTO=0`
compara): a abertura vista de `P` (a metade atrás da tangente tapada como antes) e cada raio de `P` só se testa
a partir de onde está a `AFASTA_NA_FACE` do plano da face (`pelo_ponto_ate`: `s = h / (u·n)`). No translúcido a
frente é a mesma; só a metade de TRÁS (a que atravessa o corpo, ∝ τ) marcha de dentro como antes
(`atras_de_dentro`). ⛔ A 1.ª versão deixava o translúcido inteiro na marcha antiga: o gate batched apanhou
`o_translucido_da_placa_e_o_do_gabarito` (de α 1 a 0,999 o interior saltava 0,10113, teto 0,01; 3/3 sozinho) —
`5a566833c4`.

| `sonda_a_face_pelo_ponto` (`miolo: None`, mesmo processo) | face afastada | **pelo ponto** | só o pé | nenhum |
|---|---|---|---|---|
| penumbra · fundo | 0,001436 | **0,000182** | 0,000337 | 0,000292 |
| penumbra · faixa | 0,2009 | **0,0563** | 0,0581 | 0,0771 |
| faísca · parede (fundo) | 0,000012 · 0,000050 | 0,000012 · 0,000051 | 0,000012 · 0,000050 | 0,000020 · 0,000337 |

Rodada W5g com a variante (`medida_luz2d_w5g`): cruzamentos ×0 em tudo, TV igual, o desvio do terminador desce
(polígonos, anel 0,97: 1,2 → 0,9 tx; imagem zoom 1: 0,4 → 0,1; zoom 1,5: 0,5 → 0,2). Custo da C3
(`medida_luz2d_w5g_custo`, mínimo intercalado, load 3,8): igual nas cenas 1–4 e 6, cena 5 **11,78 → 7,99 ms**.
Portão `o_interior_de_um_opaco_e_o_exterior_mais_proximo`: tetos da penumbra ao medido + 10 % (fundo 0,00158 →
0,00020, faixa 0,353 → 0,062), com o controlo da face afastada, que reprova. Verdes sozinhos: `c3_grade_fina`,
`junto_a_face`, `a_face_acesa_da_parede`, `as_formas_redondas` ×4, `o_interior_do_opaco` ×2,
`o_miolo_das_formas`, `o_traco_do_flip` ×3. Nas cenas do smoke a diferença é do tamanho do arredondamento (cena 5:
máx 0,125 numa luz de 122; cena 3: 0): o caso que muda é um corpo MEIO na sombra de outro, com lâmpada grande.

## 8. A abertura (commit `5049f0f0e9`): «do abrir o app até simular a luz demora muitíssimo» (dono, 13/10)

`medida_luz2d_w5o_abertura` (a luz do produto com a estufa, cena 5, 1080p) e o app real num ecrã virtual
(`fotografa_cena.sh` com `PH2D_LIGHT2D_TEMPO=1`; ⚠️ ele isola o `HOME`, logo o cache do driver: aponte
`__GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia` para medir o cache real):

| | primeira luz | estufa inteira |
|---|---|---|
| cache do driver QUENTE (antes e depois) | 0,06 s | 0,12 s |
| frio, antes (uma configuração por vez, em série) | 14–20 s | 47–55 s |
| frio, depois (load 8–12) | **6,2–6,7 s** | ~13 s |
| app real, cache real: 1.ª abertura depois de shaders novos · 2.ª | 9,6 s · **0,06 s** | — |

O frio acontece só quando os SHADERS mudam (toda build que os toca): zoom e janela não mudam as constantes
(`medida_luz2d_w5o_constantes_do_zoom`: (fina, captura) = (4,8) · (2,4) · (1,2) por nível em 1280–2560 px e 64
zooms), e o cache é um só para todos os programas (`~/.cache/nvidia/GLCache`). Onde ia o tempo
(`PH2D_LIGHT2D_COMPILA=1`): 63 pipelines, 14 delas (a fina e o anel) ~45 s, 2–4,7 s cada.

Cortes: a fina e o anel (e as duas variantes dos distantes) compilam em paralelo (`Par::em_paralelo`,
`solver_c3_par.rs`); o entorno não cria o que não corre; a estufa monta até ¼ dos núcleos de configurações ao
mesmo tempo, e as PEDIDAS montam sozinhas (primeira luz 7,4–8,2 → 6,2 s, alternado); a fábrica espera por uma
pipeline que outro fio já compila (116–124 → 58 compilações). ⛔ **A régua estava errada**:
`pipelines_compiladas_ao_nascer` era a conta da FÁBRICA inteira na janela do nascimento — com a estufa a
compilar noutros fios, o solver do nível inicial herdava as compilações dela (`a_luz_nasce_na_estufa`: 1 em 3–4,
«1–2 pipelines» sem nenhuma compilação depois da estufa ociosa; bisseção com 6–8 corridas por variante: só o C3
em paralelo a mostrava — uma corrida por variante tinha acusado a fábrica). Cura: `Placa::conta`, cada montagem
conta o que ELA compilou; 10/10. ⛔ O que falta para «imediato» a frio é o custo de compilar a própria grade
fina (2–4,7 s por pipeline): o próximo passo é medir o que nela pesa no compilador (os laços com limites
constantes — `RAIOS_DIRETOS`, `BISSECOES` — desenrolados?) antes de mexer.

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu` — três campos PÚBLICOS novos (`Config::so_a_c0`, `Config::face_pelo_ponto`, `Miolo::base_no_grupo`;
  todo literal da workspace usa `..`, `grep 'Miolo {'` / `'Config {'`), o buffer `luz2d/buf/miolo` +4 fatias de
  256 B (declarado na `PassList`: o G5 conta-o). Nenhum contrato congelado (§6) tocado.
- A §5.1 já aponta este handoff (pedido do dono; troca do link só, 684 B).
- Instrumentos versionados: `tests/it/medida_luz2d_w5o.rs` do app (as rodadas e as duas sondas que gravam; as
  variáveis no cabeçalho de cada teste), `Solver::medir_fases_do_miolo`, `PH2D_LIGHT2D_TEMPO=1` e `_COMPILA=1`.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_CORTES=0 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
   (o ANTES), e depois o mesmo sem `PH2D_LIGHT2D_CORTES=0` (o DEPOIS).
2. Em cada um: menu **Settings ▸ 2D Light**, escolha **High** e desligue **Automatic**; clique a **Lâmpada** na
   cena (ou na Hierarchy).
3. No Inspector, a linha «Quality: High (manual), **NN ms of GPU per frame**»: medido ~39 ms antes, ~33 depois
   (no Low ~14 → ~12). A imagem é a MESMA nos dois.
4. Errado: o número igual nos dois, ou a luz diferente entre eles.

## Gate e binário

Gates batched (`nextest-impacted`, régua = merge-base, `PH2D_GPU=1`): sobre `5a566833c4` **10 040/10 040**; sobre
`5049f0f0e9` 10 038/10 040 com load 62–95 — os 2 vermelhos (`g4_o_custo_do_rc_nao_depende_do_numero_de_luzes`,
`g4_o_custo_da_holografica_…`: relógio das cascatas, que o diff não toca) passam **3/3 sozinhos** (load ~20).
Verdes também: check `--workspace --all-targets` deny, clippy `-D warnings`, fmt, standalone/workflow. Perfil:

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                200   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                666 : 158   alvo: <= 1,0  razao 4.2x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  43%   alvo: >= 80%  (702 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         375 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
