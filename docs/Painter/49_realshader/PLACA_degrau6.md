# A água na placa — degrau 6 (31/10): submissões curtas, guarda por carimbos, descida numa espera

Continuação de [LIGACAO_PLACA.md](LIGACAO_PLACA.md) (ronda 3). Motivo: a RTX 5060 Ti travou o PC do dono duas vezes
em 31/10 (NVRM Xid 62; Xid 109 CTX SWITCH TIMEOUT + «GPU is probably locked» → reboot). Suspeita: um passo inteiro
(até ~30 passadas de folha 4K) numa só submissão longa na placa que também desenha o ecrã. ⛔ **Nada corrido na NVIDIA**
neste degrau: só a Radeon onboard, forçada por
`VK_ICD_FILENAMES=VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json __EGL_VENDOR_LIBRARY_FILENAMES=…/50_mesa.json`
(o `vulkaninfo` com esse env só lista a RADV; cada teste imprime `placa: AMD … (RADV RAPHAEL_MENDOCINO)`).

## Submissões (encoder → `queue.submit`) do `PassoPlaca::passo`, antes

| caso do passo `n` | submissões | passadas |
|---|---|---|
| ímpar, sem secagem | 1 | plano (campo 7 ou suave 6 + sedimenta 0–2 + bordas 2 + projeção 0–5 + vmax 1) |
| ímpar, com secagem | 2 (secagem; plano) | 4 (separa, borda, seca, junta) + plano |
| par, sem secagem | 1 | rebuild 5 + porta 1 + plano (indirecto) |
| par, com secagem (`n % 6`) | 3 (rebuild + espera; secagem; plano) | 5 + 4 + plano |

A 4K cada passada fina/de folha despacha a folha inteira (`Rect::full`: 41 k workgroups de 256 fios); o retângulo é
filtrado no shader. Leituras: 1 espera por passo (2 no par com secagem); descida suja = 3 faixas × 11 `read_at` = 33 esperas.

## Depois

- **Fatias** (`passo.rs::Fatias`): cada passada passa por `passada(pipe, grupo, despacho, custo)`; corta-se a submissão
  antes da passada que passaria `ORCAMENTO_GRUPOS = 40 000` workgroups (a 4K ⇒ uma passada de folha por submissão; as
  pequenas juntam-se). A ordem dos comandos é a mesma ⇒ o mesmo bit. As escritas da fila (`zera_vmax`, `base` da porta)
  vão todas antes da 1.ª submissão (aplicam-se antes da PRÓXIMA submissão — como no submit único). A secagem entra nas
  mesmas fatias (antes tinha submit próprio).
- **Carimbos**: início/fim de cada passada (`TIMESTAMP_QUERY`, 96 por passo), resolvidos na última submissão e copiados para
  o MESMO `stage` da caixa (64 B + carimbos) ⇒ nenhuma espera a mais. `ms_por_submissao()` / `ms_por_passada()`.
- **Guarda**: `estourou()` = a maior submissão do passo > `TETO_MS = 25 ms`. O worker (`offthread/placa.rs`) desce as linhas
  sujas, liga `ESTOUROU` (estático) e a água fica na CPU até fechar o app, com `⚠️ [wet-gpu] uma submissão levou …` no
  terminal. Sem `TIMESTAMP_QUERY` não há guarda (nem carimbos).
- **Descida suja numa espera** (`desce_faixas`): as 10 cópias de cada faixa (st, rgb, sett, trgb, active, wet, vel×2,
  flow×2) para um só buffer, um codificador, um `map` — 33 esperas → 1.

## Medido na Radeon 610M (RADV), 4096×2560, aguada de 3 traços, `mede_as_submissoes_a_4k`

Variantes intercaladas no MESMO processo (`afina(u64::MAX)` = o submit único de antes × `afina(ORCAMENTO_GRUPOS)`),
mínimo de 3 rondas de 12 passos, loadavg 0,3. **Mesmo bit** em film, susp, sett e cores nas 4 rondas (`assert_eq!`).

| | um submit | fatiado |
|---|---|---|
| ms/passo (relógio) | 74,0 | 73,9 |
| submissões/passo (máx) | 2 | 22 |
| **maior submissão (carimbos)** | **93,3 ms** | **22,6 ms** |
| maior passada sozinha | 22,6 ms | 22,6 ms |

Passadas do passo 12 (par + secagem + campo + projeção), ms: rebuild `0,00 0,99 4,06 0,40 0,00` · secagem
`2,27 0,87 4,23 2,13` · campo `6,18 2,08 3,88 6,42 22,58 12,86 10,01` · bordas `0,38 0,48` · projeção
`1,88 2,20 2,18 2,37 0,48` · vmax `1,33`. A maior é o `outflow_pass` (pipe 1) sobre a folha inteira.

**Extrapolação RTX** (o passo inteiro: 74 ms aqui × 9,24 ms lá ⇒ 8,0×): maior submissão ~11,7 ms antes → **~2,8 ms**
agora (`outflow_pass`); as outras ≤ 1,6 ms. Dentro do alvo 2–4 ms. Não medido na RTX.

Paridade (só Radeon, por nome): `a_secagem_na_placa_casa_com_a_cpu` · paridade.rs (`a_placa_casa_com_a_cpu_conserva_e_e_simetrica`,
`o_quadro_de_suavizacao…`, `o_quadro_de_campo…`, `o_passo_sem_secagem…`) · passo.rs (`o_passo_placa_sobre_o_engine…` máx 0,4384,
`…com_paredes…` máx 0,4009, `a_subida_e_a_descida_so_das_linhas_sujas…` máx 0,4064 — os números da ronda 3, iguais) — verdes.
Clippy `-D warnings` da crate: limpo.

## Não feito / o que falta

1. **2.ª espera da secagem NÃO removida**: o passo par com secagem ainda espera a caixa (faixas `faixas_da_caixa` e os
   grupos de `SecaGpu::passadas` vêm da caixa na CPU). Cura: uma passada que escreva `faixa` e os grupos indirectos da secagem a
   partir de `caixa[10..14]` (como a `porta`) — não é trivial de provar ao bit no prazo; fica para o próximo degrau.
2. **Partir uma passada de folha em faixas de linhas**: o `outflow_pass` (22,6 ms na 610M) não se corta com submissões. Caminho:
   uniformes por faixa com offset dinâmico (o `Rect` já parametriza `x0..y1`) — ou despachar só o retângulo da caixa em vez de
   `Rect::full` (a 4K a caixa era 3340×900 ≈ 29 % da folha ⇒ ~3,5× menos grupos por passada). Precisa de conferir cada shader.
3. **O tecto 25 ms** é por dispositivo à força (um número só): na 610M a passada maior já está a 22,6 ms, e sob carga a guarda
   pode desligar a placa — o que nessa placa é o certo (CPU ~10× mais rápida).
4. **Para ligar por omissão**: medir na RTX **com o PC livre** (o dono fora): `mede_as_submissoes_a_4k` + os gates de cor,
   e vigiar `journalctl -k | grep -i xid` durante e depois. Só depois disso pensar em `PH2D_WET_GPU` por omissão.

```text
bash scripts/ph2d-run.sh env PH2D_GPU=1 VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json \
  VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json \
  cargo test --profile smoke -p ph2d-wet-gpu --test passo -- --ignored --nocapture mede_as_submissoes
```
