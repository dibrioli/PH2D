# R2-E — Oil Grunge (K-9, Rebelle `Oil/Grunge/Grunge`) — 31/10, NÃO fechou em 2 rondas

Receita `OIL_GRUNGE` (`familias/oleo_acrilica.rs`, fim do ficheiro), **fora da fábrica** (não está no `WET`): as cerdas
do «Grunge C» J6 (ExpressOil, 600 × 3 px, pigmento 120, papel 0,3, riscas cortadas, ponta esfiapada) + a Shape
`MANCHAS_DO_GRUNGE` (Noise em Flow, como o Wax Scrape). **Sem Real Oil**: a foto do Rebelle não tem relevo (zoom do
recto de 200: plano, só riscas e falhas) ⇒ o `sem_entalhes` do R2-B não se aplica.
Sonda `sonda_r2e_grunge` (ignorada, ~2 min smoke, `PH2D_SONDA_DIR`): as três cenas A com `Oleo::Desligado`.
Prancha `pranchas/r2_oil_grunge.png` (nosso r2 C | Rebelle).

## Medido (recto de 200 px, cena `tamanhos` A; furos = px < 0,35 × mediana)
| variante (Shape: contraste, viés, Size) | mediana | furos | corrida do furo |
|---|---|---|---|
| Rebelle | 0,570 | 0,054 | 5,1 px |
| r1 0,8 · 0,6 · 1,5×10 | 0,601 | 0,185 | 58 |
| r1 0,9 · 0,75 · 1×6 | 0,601 | 0,188 | 95 |
| r1 0,7 · 0,45 · 2,5×14 | 0,596 | 0,315 | 44 |
| r1 sem Shape (= Grunge C J6) | 0,595 | 0,019 | 6,5 |
| r2 0,8 · 0,8 · 3×25 | 0,601 | 0,077 | 24 |
| r2 0,6 · 0,85 · 3×25 | 0,601 | 0,000 | — |
| **r2 0,8 · 0,85 · 2×18 (adoptada)** | 0,601 | **0,057** | 30 |
| r2 1,0 · 0,8 · 4×30 | 0,600 | 0,188 | 22 |

Riscas (desvio entre linhas da escuridão média, recto de 200): **Rebelle 0,117 · nosso 0,056**.

## Porque não está pronto (olhe a prancha)
1. **O corpo é OPACO e liso**: o Rebelle é um feixe de riscas SEMI-transparentes que se sobrepõem (o vermelho vê-se
   através do azul no `sobre`; o claro/escuro varia risca a risca: 0,117 contra 0,056). O pigmento 120 satura cada
   célula e apaga o contraste das cerdas; a Shape só abre furos num corpo cheio.
2. **Os furos são fendas de borda dura** (o limiar da Shape é binário); os do Rebelle são falhas macias e
   pontilhadas, curtas através, misturadas com riscas claras.
3. **Traços pretos** (dashes quase negros dentro do traço): já estavam no Grunge C J6, são da pilha de pigmento das
   riscas cortadas (`bristleStreak*`). O Rebelle só tem linhas finas um tom mais escuro.
4. A borda: a nossa morde em lascas; a do Rebelle desfaz-se em riscas soltas de comprimentos vários.

## Mecanismo para a próxima janela (não construído — regra das 2 rondas)
Simular primeiro: (a) pigmento 40–60 com `opacity`/`pressureOpacity` mais baixos para o feixe ficar translúcido e as
riscas aparecerem (o Express Oil a 75 já dá faixas com papel entre elas); (b) a Shape com contraste baixo (0,3–0,5)
para as falhas serem MEIO-tom e não fendas; (c) tirar `bristleStreakCut` (os pretos). Se (a)+(b) não derem as falhas
macias, a peça de motor é a Shape a modular a CARGA (não o alfa) — perguntar ao coordenador antes.
Entrar na fábrica: pôr `OIL_GRUNGE` no `WET` e, pela fila pesada,
`PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Oil Grunge"` + `gera_os_presets_de_fabrica` (a contagem
`the_factory_presets_are_counted` sobe sozinha). Fora da fila: ainda não serve.
