# As duas regressões a 4K da segunda rodada — bissecção, perfil e curas (11/11, noite)

Continuação de [SEGUNDA_RODADA_AxB.md](SEGUNDA_RODADA_AxB.md). Radeon 610M (RADV) sob Xvfb, papel Textured com o relevo
do dono, `mede_pen_up_4k.sh matriz` (`CURTO=1`), binários `--profile smoke`. Roteiros em [`regressoes_11_11/`](regressoes_11_11/)
(`build.sh` constrói pontos numa worktree única com target próprio; `mede.sh`/`mede_tela.sh` intercalam; `junta.sh`
agrega por tela). **Resumo = o MÍNIMO; carga anotada por corrida.**

## ⛔ Duas armadilhas da régua (lidas antes de qualquer número)

1. **O ambiente de build é parte da variante.** O mesmo código, compilado no target da linha × no target da bissecção,
   deu Filbert 4K toque **79 × 64 ms** e Ctrl+Z **63 × 49** (r6–r8: trocar a ÁRVORE do roteiro não mudou nada; trocar o
   BINÁRIO sim; limpar o `incremental` não curou). Os hashes dos artefactos são iguais nos dois targets (as flags
   também), logo é disposição de código (caminho da pasta embebido, CGU de 16). ⇒ **A×B só entre binários do MESMO
   target**, construídos um a seguir ao outro. As linhas `curaB`/`cura12` abaixo saíram do target da bissecção, como A e B.
2. **O Filbert 4K é BIMODAL no tempo**: em rodadas inteiras todos os binários caem num modo «rápido» (A 48,9 · B 48,7
   no Ctrl+Z, r5) e noutras num modo «lento» (A 54,8 · B 62,7, r10). A regressão do Ctrl+Z só aparece no modo lento;
   a bissecção a n=1 do Ctrl+Z é inválida (o par `00f3ca1751`×`47ec5ad159` deu 48,9×45,2 em r4). Decide a matriz
   intercalada n≥4 da mesma rodada.

## Bissecção

| Regressão | Culpado | Prova |
|---|---|---|
| Toque Impasto 4K +8 ms | **`0ba4950a51`** (B29.1: a banda do pincel liso leva o tecto largo) | degrau único, 13,2 → 20,9; os 5 pontos seguintes 21–23 |
| Ctrl+Z Filbert 4K +7 ms (modo lento) · toque Filbert/Wet +5–6 | **`47ec5ad159`** (B34 + B31.1: o plano do papel na luz) | 00f3 56,6 → 47ec 70,0 (n=1); toque 47ec × 00f3 +4,8 (n=3) e +6,3 (n=6); perfil abaixo |

## Perfil do Ctrl+Z Filbert 4K (00f3 × 47ec, 4 pares intercalados, `perf -k mono` sem `-g`)

| fase (fio da UI salvo nota) | 00f3 | 47ec | Δ |
|---|---|---|---|
| `memcpy` do `queue.write_texture` dos planos (107 → 140 MB) | 9,3 | 14,3 | **+5,0** |
| `Vec<f32>::resize` dos planos a crescer (caixa → tela) | 1,3 | 3,8 | **+2,5** |
| espera sem CPU no fim do quadro (placa) | 15,2 | 17,6 | **+2,4** |
| dobra nos operários | ~62 ms CPU | ~62 ms CPU | ~0 |

O wgpu-core 30.0.1 copia para o staging com `pending_writes.lock()` preso: `write_texture` em paralelo não ganha.

## Curas e o «depois» (r11, mesmo target, n=5, mínimo / mediana, carga 0,8–21,6, 34 de 50 corridas < 5)

| 4K | A `e81e56bb1f` | B `57b4ddbac7` | B + cura do toque (`curaB`) | + `planos_de_tamanho` (`cura12`) |
|---|---|---|---|---|
| Impasto toque | — | 20,8 / 21,9 | — | **12,0 / 12,8** |
| Impasto Ctrl+Z | — | 14,0 / 14,8 | — | 14,1 / 15,0 |
| Filbert toque | 70,9 / 71,9 | 76,9 / 80,0 | 76,6 / 78,8 | 75,9 / 78,2 |
| Filbert Ctrl+Z | 55,5 / 57,0 | 61,7 / 63,3 | 62,8 / 63,1 | 62,2 / 64,6 |
| Filbert soltar | 48,1 | 33,0 | 34,0 | 35,2 |
| Wet toque | 55,7 / 59,0 | 61,7 / 65,0 | 63,1 / 64,1 | 62,9 / 66,4 |
| Wet Ctrl+Z | 41,2 | 42,0 | 46,1 | 45,0 |

(r10, mais calma, n=4: Impasto toque A 12,0 · B 19,9 · cura 12,0; Filbert Ctrl+Z 54,8 · 62,7 · 62,2.)

- **Toque do Impasto: curado ao A** (`a0148c712e`): `TexelLargo: bytemuck::Zeroable` + `bytemuck::zeroed_vec` — o
  `vec![T::default(); n]` só usa `alloc_zeroed` para primitivos; num struct escrevia ~50 MB a 4K no pen-down.
- **`planos_de_tamanho` (crescer por páginas zeradas): sem ganho medido** (62,2 × 61,7) — o custo muda para as faltas
  de página dos operários. Não comitado como cura isolada.
- **Ctrl+Z não confinado** (`a95be6612d`): só as camadas cujo `Arc` mudou re-sobem à placa (era 33 MB por camada).
  A cena desta matriz tem uma camada pintada: o ganho não aparece aqui; aparece com N camadas (o B42 abre 64).
- **Aberto** (a seguir neste ficheiro): o envio sem cópia dos planos da luz e o toque/Ctrl+Z do Wet.

## r12 (12/11, 00:42–01:12) — envio sem cópia e as curas do Wet

Mesmo target (`.ph2d-bis-4k-target`), três binários construídos em série: base `4381ee6d4a` (HEAD da linha, com o laço
a pedido) · `887f234eec` = base + envio sem cópia · `4c375d37f0` = + `dd4d9207b6` `ca6bf04bf1` `04ca713600` (cherry-pick).
n=5 blocos intercalados (pares invertidos), 60 corridas, carga mediana 1,9 (blocos 3–4 com 9–46: outra sessão).
Mínimo / mediana:

| cena | base | +envio | +envio+Wet |
|---|---|---|---|
| Filbert 4K toque | 67,0 / 78,7 | **62,6** / 66,4 | 63,4 / 71,5 |
| Filbert 4K Ctrl+Z | 63,6 / 66,2 | **50,1** / 52,0 | 50,1 / 55,0 |
| Wet 4K toque | 61,8 / 62,9 | **50,6** / 52,3 | 50,8 / 51,4 |
| Wet 4K Ctrl+Z | 47,0 / 48,0 | 48,1 / 48,9 | 46,0 / 49,9 |
| Impasto 4K toque / Ctrl+Z | 12,0 / 13,9 | 11,9 / 14,1 | 12,1 / 13,9 |
| Filbert 1080p toque / Ctrl+Z | 28,2 / 34,9 | 26,1 / 37,0 | 25,7 / 36,0 |

- **Envio sem cópia: ganho** (Filbert Ctrl+Z −13,5, Wet toque −11,2, Filbert toque −4,4) ⇒ comitado `d9bd722622`.
- **Curas do Wet: ambíguas** — Ctrl+Z −2,1 no mínimo e +1,0 na mediana; toque igual. O preparo mede-se no MESMO
  binário com `PH2D_TOQUE_PREPARADO=0` (r13, abaixo).

## r13 (12/11, 01:20–01:26) — o preparo do toque no MESMO binário

`4c375d37f0`, Wet 4K, `PH2D_TOQUE_PREPARADO=1` × `=0`, 6 pares intercalados (carga 2,5–11, outra sessão a compilar).
Toque: ligado **51,3** / 52,0 × desligado **66,7** / 67,8 — o preparo vale **15 ms** no 1.º toque. Na base o tique
já o corria antes do toque (o roteiro mexe o rato: há quadros), por isso r12 não o vê; `ca6bf04bf1` garante-o
quando NÃO há quadro antes do gesto (o laço a pedido, `899f2d2e71`: sem evento não há quadro nem `on_tick`), p. ex.
abrir o documento e tocar sem passar o rato. Controlo: sem as três chamadas de `arma_o_preparo_do_toque` o teste
`o_preparo_fica_pronto_sem_tique_antes_do_primeiro_toque` fica vermelho. Ctrl+Z igual (48,0 × 49,0). As três
curas do Wet entram na linha: `b5f4a90128` · `62bb7c2efa` · `3848f195aa`.

## Perfil p1 (12/11, 01:26–01:29, `4c375d37f0`, modo `perf`, quadros > 60 ms)

| cena | quadros lentos | leitura |
|---|---|---|
| W5 Flat Bristle `]`×10 | **1 055 ms no mouse-up** · 102 no Ctrl+Z | o pior caso que resta: o depósito da lâmina larga (B14.3) |
| W9 Bristle 16·4000 | 62 no toque · 63 no Ctrl+Z | o soltar de 1 340 ms de 10/11 já não existe |
| D1 Round Mixing `]`×10 | nenhum | o Color mixing largo não passa de 60 ms |
| Wet · Filbert · W6 | 62–93 (toque, Ctrl+Z) | os da onda 1 |
| W6 parado depois de `]`×10 | CPU **492 %** de um núcleo durante 5 s | achado: trabalho de fundo sem entrada (por perfilar) |

## rf (12/11, 03:35–04:11) — a matriz FINAL da janela: base `4381ee6d4a` × HEAD `a5fcd15668`

Mesmo target (`.ph2d-bis-4k-target`), os dois binários construídos em série; 9 cenas × 2 × 4 blocos intercalados (pares
invertidos) = 72 corridas; carga 1,7–65 (mediana 16,7: outra sessão a integrar). Mínimo / mediana, ms (CPU em % de um núcleo):

| cena | toque | traço p95 | soltar | Ctrl+Z | outro |
|---|---|---|---|---|---|
| Wet 4K | 62,9/66,2 → **50,7/51,6** | 17,1/18,4 → 13,8/15,1 | 22,1 → 21,7 | 49,9/50,0 → **34,0/36,1** | |
| Filbert 4K | 71,8/82,0 → 67,3/74,3 | 23,2/36,4 → 28,9/30,3 | 36,0 → 38,0 | 61,9/66,1 → **48,1/52,0** | |
| Impasto 4K | 14,1 → 12,1 | 14,0 → 12,3 | 19,8 → 19,0 | 14,9 → 15,0 | |
| Filbert 1080p | 28,7 → 25,2 | 22,9 → 21,1 | 29,8/39,5 → **20,0/23,5** | 37,0/40,5 → **25,1/28,6** | |
| Wet + Óleo Real 4K | 68,6/70,5 → **55,6/57,6** | 15,1 → 15,0 | 33,2 → 27,9 | 64,7/67,2 → **46,8/50,8** | água 40 Hz nos dois |
| W5 Flat Bristle `]`×10 | 44,3/45,4 → **31,0/32,2** | 25,1/564 → 14,8/434 | (régua sem soltar fiável) | 88,4/95,7 → **47,9/51,0** | água 16,1 → 39,7 Hz (mín.) |
| W6 Wet `]`×10 | 64,1/68,1 → **51,7/52,0** | 13,9 → 13,9 | 21,9 → 18,9 | 48,1/50,0 → **32,9/36,5** | CPU parado 283/468 → **147/160** |
| W9 Bristle 16·4000 | 55,9/58,0 → **30,7/46,2** | 23,0 → 23,2 | 41,8 → 39,8 | 62,7/66,0 → **49,2/52,6** | |
| D1 Round Mixing `]`×10 | 23,0 → 22,0 | 12,9 → 14,2 | 12,0 → 11,0 | 15,0 → 15,0 | |

Leitura: o toque do Wet/Óleo/W5/W6/W9 cai 12–25 ms (envio sem cópia + preparo + mistura e cerdas mais baratas + a
miniatura fora do pool da tela); o Ctrl+Z do Wet chega ao alvo de 33 ms (34,0 · W6 32,9) e o do Filbert/Óleo cai 13–18 ms
(troca de tiles, restauro no pool); o W5 continua a ter quadros de ~0,4–0,6 s no traço (o depósito do Flat 240 é ~0,4 s por
dab: aberto, ver o handoff); D1 e Impasto iguais (já não eram problema). Matriz: `$SP/matriz2.sh` com `@KNOBS@TECLAS@nome`.
