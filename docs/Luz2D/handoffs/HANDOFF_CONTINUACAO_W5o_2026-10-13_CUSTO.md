# Handoff de continuação — Luz 2D, W5o: o custo da luz (2026-10-13)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre `main`
> `757afebd58` (tudo da W5n já integrado). Commits: `dc081fa8bc` (o custo: produto, portões, sondas) · `6be9d9348e`
> `3ce1f68f6e` (docs, clippy da sonda) · `fcab331472` + `5a566833c4` (a PRECISÃO, outra família: §7) · este (docs).
> Nada integrado, nada enviado. Método de medida: [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md) — as
> variantes são CONFIG EM EXECUÇÃO no mesmo processo, 7 rodadas intercaladas, vale o MÍNIMO (a mediana ao lado).
> Cache do driver: `__GL_SHADER_DISK_CACHE_SIZE` 16 GiB (o `GLCache` em 1,1 GB, acima do tecto antigo).

## 1. Onde estavam os «~4,4 ms do Low» — em lado nenhum

A adenda da W5n leu `Low 8,87 → 13,3 · High 22,3 → 30,1 ms` no portão `a_nitidez_da_luz` **dentro da corrida do
integrador** (o grupo `luz2d-placa`, 2 testes de placa por vez: os carimbos de um contam a fila do outro). O
mesmo portão SOZINHO (load 4–10, 2×): **Low 3,85 · High 30,15** (sem o miolo 2,76 · 25,49). A régua por passe
(rodada `medida_luz2d_w5o`) dá o Low da W5h **2,69** → W5n **3,89 ms**: o miolo da vista +0,66, o anel +0,23
(`c3_fina`) e o miolo do ENTORNO +0,33. O High não segue a W5n: segue a `c2` (§3).

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

Fases do miolo da vista (o instrumento, vazia · Low, ms / despachos):

| fase | W5n | W5o |
|---|---|---|
| classifica + puxa | 0,064 / 10 | 0,064 / 10 |
| cascata: mais grosso | 0,079 / 65 | 0,030 / **1** |
| cascata: desce | 0,340 / 196 | 0,261 / 130 |
| ciclo: defeito + restringe | 0,086 / 27 | 0,076 / 21 |
| ciclo: mais grosso | 0,235 / 195 | 0,056 / **3** |
| ciclo: sobe | 0,191 / 105 | 0,155 / 75 |
| ciclo: soma · final | 0,091 / 7 | 0,092 / 7 |
| **soma** | **1,087 / 605** | **0,733 / 247** |

O miolo é limitado pelo NÚMERO de despachos (~1,2 µs cada nos níveis minúsculos; um nível de 5×3 levava 64 por
ciclo), não pelo trabalho.

## 3. ⚠️ ABERTO, o maior: a `c2` do High muda 18,4–26,6 ms com o estado do ALOCADOR

A mesma configuração High, o mesmo shader, a cena vazia: o custo da cascata holográfica depende de ONDE o
`gpu-allocator` põe os buffers dela (todos < 128 MB, sub-alocados: `hrc_t0`/`hrc_acumulado` 56 MB, `hrc_t1..t12`,
`hrc_r0/r1` 31–46 MB). Um caso por PROCESSO (`medida_luz2d_w5o_lastro`, mínimo de 12 quadros, 2× cada):

| antes de criar o High | `c2` |
|---|---|
| nada (processo novo) | 24,6 ms |
| 1 GiB de texturas VIVAS (lastro) · 256 MB · 3 GiB | 24,6 |
| um Low criado e LARGADO · um High criado e largado · 1 GiB escrito e largado | **18,4** |
| um Low VIVO | 21,2 |
| `MemoryHints::MemoryUsage` (blocos 8→64 MB) | 20,8 |
| blocos fixos de 256 MB | 21,3 |
| blocos de 4 MB (toda textura grande dedicada) | 26,6 |

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

Provas (fluência da vista pelo caminho do PRODUTO, `sonda_grava_as_fluencias_w5o`: cena 3 com a câmera em x = 2 —
a luz de fora chega pelo entorno — e cena 5, Low e High, 80 quadros; sha256 da base `417916a7…84609d`, 2× igual):
- corte 1, corte 1 + 2, `PH2D_LIGHT2D_CORTES=0`: `cmp` = 0 bytes. A da α = 1 (`sonda_grava_a_fluencia_a_alfa_1`)
  também — mas ⚠️ ela NÃO vê o entorno (sem ele, 0 bytes mudam: na cena 7 tudo está na vista) nem prova a base.
- controlos: sem o entorno 28 642 988 bytes mudam; sem o miolo 1 968 043.
- a base corre: mutação na relaxação da cascata da base → 17 967 bytes; o `AMORTECE` da base não muda NADA nas
  cenas 3/5 (todo texel grosso sobre um corpo tem algum filho conhecido: o erro está preso a zero nos níveis da
  base) — por isso `sonda_grava_a_caixa_w5o`: a caixa do `o_interior_do_opaco` no zoom 2,5 (o corpo enche a
  vista), base desligada × ligada `cmp` = 0; a mesma mutação do `AMORTECE` → 695 414 bytes.

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
| faísca · fundo · faixa | 0,000012 · 0,0505 | 0,000012 · 0,0505 | 0,000012 · 0,0505 | 0,000020 · 0,0496 |
| parede · fundo · faixa | 0,000050 · 0,0062 | 0,000051 · 0,0062 | 0,000050 · 0,0062 | 0,000337 · 0,0062 |

Rodada W5g com a variante (`medida_luz2d_w5g`): cruzamentos ×0 em tudo, TV igual, o desvio do terminador desce
(polígonos, anel 0,97: 1,2 → 0,9 tx; imagem zoom 1: 0,4 → 0,1; zoom 1,5: 0,5 → 0,2). Custo da C3
(`medida_luz2d_w5g_custo`, mínimo intercalado, load 3,8): igual nas cenas 1–4 e 6, cena 5 **11,78 → 7,99 ms**.
Portão `o_interior_de_um_opaco_e_o_exterior_mais_proximo`: tetos da penumbra ao medido + 10 % (fundo 0,00158 →
0,00020, faixa 0,353 → 0,062), com o controlo da face afastada, que reprova. Verdes sozinhos: `c3_grade_fina`,
`junto_a_face`, `a_face_acesa_da_parede`, `as_formas_redondas` ×4, `o_interior_do_opaco` ×2,
`o_miolo_das_formas`, `o_traco_do_flip` ×3. Nas cenas do smoke a diferença é do tamanho do arredondamento (cena 5:
máx 0,125 numa luz de 122; cena 3: 0): o caso que muda é um corpo MEIO na sombra de outro, com lâmpada grande.

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu` — três campos PÚBLICOS novos (`Config::so_a_c0`, `Config::face_pelo_ponto`, `Miolo::base_no_grupo`;
  todo literal da workspace usa `..`, `grep 'Miolo {'` / `'Config {'`), o buffer `luz2d/buf/miolo` +4 fatias de
  256 B (declarado na `PassList`: o G5 conta-o). Nenhum contrato congelado (§6) tocado.
- A §5.1 já aponta este handoff (pedido do dono; troca do link só, 684 B).
- Instrumentos versionados: `crates/ph2d-app-light2d/tests/it/medida_luz2d_w5o.rs` (a rodada, a série e as
  instâncias, o lastro com `PH2D_W5O_LASTRO=<mb>[-largado]|low|low-vivo|high`, `PH2D_W5O_DICAS=memoria|256|4`,
  `PH2D_W5O_LISTA=1`; as duas sondas que gravam) e `Solver::medir_fases_do_miolo`.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_CORTES=0 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
   (o ANTES), e depois o mesmo sem `PH2D_LIGHT2D_CORTES=0` (o DEPOIS).
2. Em cada um: menu **Settings ▸ 2D Light**, escolha **High** e desligue **Automatic**; clique a **Lâmpada** na
   cena (ou na Hierarchy).
3. No Inspector, a linha «Quality: High (manual), **NN ms of GPU per frame**»: medido ~39 ms antes, ~33 depois
   (no Low ~14 → ~12). A imagem é a MESMA nos dois.
4. Errado: o número igual nos dois, ou a luz diferente entre eles.

## Gate e binário

Gate batched 1× (verificador, `dc081fa8bc`, load 9 → 130 — outras linhas a compilar): `nextest-impacted` (régua
= merge-base, `PH2D_GPU=1`) 10 040 corridos, 10 039 verdes; o vermelho `ph2d-physics-ecs`
`the_cost_of_a_player_is_linear_in_their_number` (12,46× contra ~7×) é de outra crate, fora do diff, e passa 3/3
sozinho (load 32–44) — flake de carga. Verdes dentro dele: `g5_bytes_declarados`, `o_custo_do_miolo` ×2,
`c3_grade_fina` ×6, `o_interior_do_opaco_nao_depende_da_posicao_na_grade`, `o_rebote_nao_depende_da_oitava`,
`as_formas_redondas` ×4, `a_nitidez_da_luz` ×6, `architecture_workspace_file_loc_cap`. `cargo check --workspace
--all-targets` com warnings = deny ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · `fmt --check`
✓ · clippy `--all-targets -D warnings`: 4 avisos na sonda `medida_luz2d_w5o.rs` (`type_complexity` ×2,
`unnecessary_sort_by` ×2), curados no commit dos docs e relidos limpos.

**2.º gate batched, sobre a precisão** (`fcab331472`): o único vermelho `o_translucido_da_placa_e_o_do_gabarito`
(real, curado em `5a566833c4`, §7); check `--workspace` deny ✓, clippy relido ✓, fmt ✓, standalone/workflow ✓.
**Gate final** (`5a566833c4`, `nextest-impacted`, régua = merge-base, `PH2D_GPU=1`, load 49 → 3): **10 040 / 10 040
verdes** (3 lentos), 18 819 saltados.

Perfil do loop (`scripts/agent-loop-profile.sh`, as 20 sessões mais recentes da máquina):

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                154   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                550 : 128   alvo: <= 1,0  razao 4.3x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  45%   alvo: >= 80%  (583 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         366 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

Binário do smoke: no fim (abaixo).
