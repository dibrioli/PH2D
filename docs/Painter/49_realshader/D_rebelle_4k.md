# D — Rebelle 4K: criação da tela × 20 pinceladas de faca (30/10)

Oráculo corrido, caixa-preta, ecrã real `:0` (aval do dono 30/10), Rebelle 8 por wine, Motion IO websocket 8265.

**Comando:** `R=3 bash ~/Apps/rebelle/oraculo/custo_facas_4k.sh 4k_so_pinceladas` (cópia de `custo_facas.sh`, original
intacto) · saída `~/Apps/rebelle/oraculo/sonda/facas_4k_so_pinceladas/` (`medidas.csv`, `cpu_*`, `gpu_*`, `perf_*`, foto).
**Tela:** `NEW_ARTWORK 3840×2160` px, 72 dpi, EX00 White Simple; RealShader ligado, impasto 8, gloss 10, paper_texture 0;
`settings.ini.dono`. As facas antigas (`facas_real/np0/ss0`) usavam **1200×800**. Os traços de `t_facas.jsonl` estão em
coordenadas 1200×800 → caem no canto superior esquerdo da tela 4K (foto conferida).
**Intercalado R=3:** por ronda `criacao` (NEW_ARTWORK + seed + params, 3 eventos) e logo `pinceladas` (10 facas ×2 =
3 060 eventos, 2 ms entre eventos ⇒ 6,36 s só de envio).
**loadavg:** antes `0.92 4.72 6.84` (o 1-min manda: máquina calma) · depois `0.43 1.41 4.14`. Placa ociosa 4 %.

⚠️ **O SAVE não serve de régua** («SAVE and LOAD events aren't supported via Websockets», resposta do servidor, 26/10);
a régua do script é CPU ocupado → ocioso (< 0,1 núcleo durante 1 s). **Em 4K o Rebelle parado gasta 0,08–0,11
núcleo** — encostado ao limiar — e as 6 medidas bateram no tecto de 90 s (`ms_fila` 90 707–97 098 = tecto, NÃO
duração). O tempo real sai do traço por segundo da CPU (`amostra_cpu`, 1 Hz): span = até ao último segundo acima de
ocioso + 0,15 núcleo; CPU = soma acima do ocioso.

| ronda | variante | span (s) | CPU acima do ocioso (s) | núcleos pico | placa % mediana / máx | CPU s bruto (90 s) |
|---|---|---|---|---|---|---|
| 1 | criação 4K | 2 | 3,2 | 2,95 | 4 / 94 | 14,15 |
| 1 | pinceladas | 8 | 9,1 | 1,85 | 4 / 22 | 19,37 |
| 2 | criação 4K | 1 | 3,3 | 3,44 | 4 / 47 | 13,48 |
| 2 | pinceladas | 8 | 9,2 | 1,72 | 4 / 23 | 19,10 |
| 3 | criação 4K | 1 | 3,2 | 3,33 | 4 / 33 | 13,41 |
| 3 | pinceladas | 8* | 11,4 | 2,46 | 4 / 25 | 20,31 |

\* ronda 3: um pico tardio estica o span calculado para 83 s; o corpo do trabalho é o mesmo (8 s).
Referência 1200×800 (`facas_real`, base = criação + pinceladas no mesmo relógio): CPU bruto 7,85–15,46 s, pico ~2,0 núcleos.

**Leitura**
1. **As pinceladas pesam ~3× mais que a criação da tela 4K:** 9,1–11,4 s de CPU contra 3,2–3,3 s. A criação é um
   estouro curto (≤ 2 s, pico de 3,4 núcleos e 33–94 % da placa num instante); as 20 facas são ~1,3–1,9 núcleos
   durante 8 s (das quais 6,4 s são o ritmo de envio — o Rebelle acompanha o fluxo).
2. A placa mal trabalha (mediana 4 %) nas duas; o custo do Rebelle aqui é CPU, ~2 núcleos, nunca a máquina inteira.
3. Os `ms_fila` de 97 s das corridas antigas (np0/ss0) são o mesmo tecto, não lentidão: a régua de calma falha
   quando o ocioso do Rebelle fica perto de 0,1 núcleo — corrija o limiar (ocioso medido + margem) antes de reusar.
