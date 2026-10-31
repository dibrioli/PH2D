# A água na placa — degrau 4 (31/10): o passo SEM SECAGEM inteiro

Continuação de [PLACA_degrau3.md](PLACA_degrau3.md). Crate `crates/ph2d-wet-gpu`.

## Feito

O passo de `sim_step_stage` sem o estágio 1 (secagem), na ordem do produto, é `plano_passo(n, sed)`:
rebuild (`n % 2`) → campo (`n % 4`) ou suave → advecção → `sedimenta` (se `p.sedimenta > 0`) →
bordas → `project` + bordas de velocidade (`n % 3`). Passadas novas (índices `8..=19` em `pipes`):

- **bordas** = `apply_boundaries`: `bordas_massa` (folha fina, film/susp 0, wet 1) e `bordas_vel`
  (grade de fluxo; tangencial a 0, normal ±0,1 na ordem da CPU — o último escrito vence nos cantos).
- **project**: `proj_zera` (retângulo de fluxo ±1) → `proj_div` (em `flow_x`) → `proj_pressão`
  (em `flow_y`) → `proj_corr` (em `vel`, com o guarda do retro-traço). Knob `Projection` no `K`.
- **sedimenta**: gather sobre `dst`/`drgb` com o film inalterado, depois o `commit`. Só a média
  linear (`km_mixing = false`, o padrão). `Knobs::com_sedimenta` = a geometria de `sedimenta_rows`.
- **rebuild_active_region**: `re_zera` → `re_limpa` → `re_passe1` (em GATHER: a célula é ativa se
  algum trio horizontal centrado em x−1/x/x+1 molha; film ≥ 0 ⇒ a soma f32 > 0 é o mesmo predicado da
  soma f64) → `re_saia` (UMA thread por COLUNA, laço em y de cima para baixo: a saia só lê e escreve
  `i−s, i, i+s`, por isso as colunas são independentes e a ordem da CPU fica intacta) → `re_caixa`.
  A caixa nova fica no buffer `caixa` (64 B, `readback_caixa`) — é o que o despachante leria.
- `K` passou a 16 × 4 B (`projection`, `sed_ax/ay`, `sed_dx/dy`). `cena` mudou para `src/cena.rs`
  (tecto de 700 linhas do `lib.rs`).

## Gate `o_passo_sem_secagem_na_placa_casa_com_a_cpu`

Cena `poca_papel` 320×200, rf 1 e 4, `sedimenta` 0 (produto) e 20 (`a = 0,25`), quadros 4.., gravidade
(0,004; 0,012). Confere antes que a cena exercita os três passes (máscara muda, susp muda com a
sedimenta, a projeção muda a velocidade). Afirma: caixa e máscara `active` **idênticas** em todo n;
1 passo ≤ 1e-6 por campo; 8 e 64 passos: cor pesada pela massa suspensa ≤ 3e-6 e film médio ≤ 3e-6;
massa placa × CPU ≤ 1e-7·n.

| placa | passos | film máx (médio) | susp máx | cor pesada (pior) | active ≠ | Σ rel. placa × CPU |
|---|---|---|---|---|---|---|
| RTX | 1 | 9,5e-7 (2,2e-8) | 8,9e-8 | 6,0e-8 | 0 | 2,5e-9 |
| RTX | 8 | 1,9e-6 (6,1e-8) | 3,0e-7 | 1,1e-7 | 0 | 3,2e-8 |
| RTX | 64 | 8,8e-5 (2,6e-7) | 2,3e-5 | 3,5e-7 | 0 | 3,3e-7 |
| Radeon | 64 | 2,9e-4 (3,0e-7) | — | 5,7e-7 | 0 | 3,3e-7 |

O máximo de film a 64 passos é o `trunc` do freio (degrau 3); o médio e a cor pesada são o número.

## ms de placa (mínimo, `mede_advect`, loadavg ~36–49 — a placa não depende dela)

| medida | RTX 5060 Ti | Radeon RADV |
|---|---|---|
| quadro de campo 4096×2560 | 6,57 | 126,9 |
| **PASSO sem secagem 4096×2560 (média do ciclo de 12)** | **7,74** | 132,8 |
| PASSO sem secagem região 1024² | 0,88 | 14,8 |
| PASSO sem secagem 1024² + readback só da região (parede) | 4,91 | 20,1 |
| CPU `advect_jacobi` sozinho 4096×2560 (rayon, sob carga) | 54,6 | — |

Rebelle: 37,3 ms/passo água 4K na CPU dele. O passo inteiro custa +18 % sobre o quadro de campo; o
readback da região continua a dominar na RTX (~4 ms de 4,9).

## Próximo passo exacto

1. **Despacho por caixa**: `re_limpa`, `re_passe1`, `bordas_massa` despacham a folha INTEIRA (early-out);
   na região 1024² isso é custo à toa. Despachar `re_*` sobre a caixa ±3 e as bordas sobre o anel.
   E o produto tem de ler `caixa` (16 B) para o retângulo do passo seguinte — hoje o gate usa a folha
   inteira (a cena mantém a caixa = folha, e o gate afirma a caixa idêntica à da CPU).
2. Faixas vivas por linha (`spans_enabled`) e `has_fluid = false` (abortar o passo) — por portar.
3. Extensões do campo (degrau 3 §4): ladeira do óleo, paredes, gota, fingering, `backrun_rows`;
   `sedimenta` com K–M.
4. A secagem (estágio 1, `drying_pass_jacobi`).
