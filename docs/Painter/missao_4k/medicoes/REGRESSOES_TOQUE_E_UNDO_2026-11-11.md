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
