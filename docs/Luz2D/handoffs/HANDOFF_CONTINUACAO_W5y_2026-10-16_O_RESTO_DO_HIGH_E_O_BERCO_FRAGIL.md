# Handoff de continuação — Luz 2D, W5y: o resto do High e o berço frágil (2026-10-16)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5x ([handoff](HANDOFF_CONTINUACAO_W5x_2026-10-15_O_BERCO_DA_HOLOGRAFICA.md)) e das
> anteriores, todas não integradas. **A W5x ainda espera o smoke do dono; não integrar.** Commits da W5y: `26b2e5af90`
> (instrumento) e o deste handoff (o relatório do alocador no berço e a calibração da sonda de texturas). Nada de
> produto mudou: sem as variáveis abaixo o app é o da W5x. **Sem gate de fecho ainda** (o `a_luz_recua_quando_a_placa_recusa_a_memoria`
> pelo nextest fica para o fecho). Placa RTX 5060 Ti, `max_texture_dimension_2d` do dispositivo = **8192**.

## 1. O instrumento (todos só com `PH2D_LIGHT2D_CUSTO=1`)

| Variável | O que imprime |
|---|---|
| `PH2D_LIGHT2D_MEMORIA=1` | (W5x) Σler dos buffers da holográfica; **W5y:** e aos SALTOS (`Medida::Saltar`), as texturas da lista e os ladrilhos (vista e entorno, emprestados por `Solver::recursos_da_lista`), a calibração (texturas novas de zeros) e o **alocador no berço** (`berço vivo/largado/buffers · reservado · alocado · blocos`). |
| `PH2D_LIGHT2D_MEMORIA=lista` | O mesmo sem os clones da holográfica (a alocação do produto). |
| `PH2D_LIGHT2D_SERIE=1` | Cada leitura: c2, total e o alocador (alocações, MiB, blocos). |
| `PH2D_LIGHT2D_FASES=1` | As etapas da holográfica da vista por janela (cada despacho no seu passe; a c2 dos `TemposMs` fica 0). |
| `PH2D_LIGHT2D_SOZINHA=1\|2` | A luz com a placa ociosa (espera antes/depois do envio; 2: o entorno num envio próprio). |

Roteiros das rodadas (fora do repo, na scratchpad da sessão; reconstruir é trivial): um laço sobre
`mede_custo_no_app.sh` com casos intercalados e rodados, `PH2D_GPU=1`, e **esperar a placa só com o kwin antes de
cada processo** (`nvidia-smi --query-compute-apps`), registando quem apareceu DEPOIS. Neste dia outras linhas
(PainterWatercolor, UIUX, CodeEditor) correram apps e testes de placa FORA da fila: duas rodadas tiveram de ser
descartadas.

## 2. O item 1 do §5 da W5x (o resto nas primeiras montagens) — medido, NÃO explicado

Seis rodadas (rodada 1: 1920/2560 × berço 1/0 × memória 0/lista/1 × 3 processos, 4000 quadros, `RENASCE=2`).

- **A c2 é constante dentro de cada geração** (quintis iguais ao centésimo); só a 1.ª leitura depois de renascer
  sai ~0,7 mais baixa (um quadro especial).
- **O resto vem em DEGRAUS a meio da geração**, com a MESMA alocação: +0,6 a +1,0 ms, muitas vezes depois de uma
  paragem de 40–60 ms; dura até a vista renascer (às vezes mais). Típico nas duas 1.ªs gerações (2560 com berço:
  g0/g1 12,6–13,3, depois 11,4–11,8), mas houve processo com as 1.ªs rápidas e o degrau na g3.
- **Refutado como causa** (cada um MEDIDO, por geração):
  - os buffers da holográfica: com berço, Σler no piso (0,43 / 0,54) também nas gerações lentas; **aos saltos**
    também plano (±0,5 %) ⇒ nem compressão nem TLB/tipo de página;
  - o relógio da placa (2782–2835 MHz, memória 14001, em todas);
  - o alocador do app (total e blocos fixos quando a c2 dá o degrau; só as alocações transitórias 213↔257 oscilam);
  - o entorno (`PH2D_LIGHT2D_ENTORNO=0`: as duas 1.ªs gerações continuam lentas);
  - o trabalho do próprio app ao lado (`SOZINHA`: a g0 mantém o excesso: 9,3–10 / 12–13);
  - a sonda não perturba o produto (m0 × lista × 1 iguais dentro da variação entre processos).
- **NÃO refutado (a régua é cega):** as TEXTURAS. A calibração mostrou que a sonda de texturas lê uma textura
  NOVA DE ZEROS a 541–620 GB/s, o mesmo que todas as outras: o `textureLoad` em ordem de linha satura antes de
  distinguir compressão. O «texturas planas» desta onda NÃO vale como refutação.
- O que sobra: algo fora do solver (driver, outra carga da placa) — inferência, não medida.

## 3. ⭐ Achado: o berço é FRÁGIL (medido, com o mecanismo)

Com a placa ociosa à volta da luz (`SOZINHA=1|2`), a partir da g1 a c2 do High **duplica** (2560: 22,8–26; 1920:
16–20) e fica, geração após geração; a sonda confirma (t3…r1 a 424 GB/s, Σler 1,52 contra 0,54).

**Mecanismo** (`berço vivo/largado/buffers`, 2560, 2 processos × `SOZINHA` 0/1): o `gpu-allocator` destrói o bloco
geral que fica VAZIO (salvo o último; `gpu-allocator 0.28 src/vulkan/mod.rs:671`). Com `SOZINHA=1`: berço vivo 2645
MiB / 12 blocos → largado 1877 / 9 (**3 blocos destruídos**) → buffers 2645 / 12 (**3 blocos NOVOS**, nunca com
textura) ⇒ sem compressão. No uso normal (a vista anterior ainda viva): vivo 3925 / 17 → largado 3157 / 14 →
buffers 3157 / 14 — também destrói blocos, mas os buffers cabem em buracos de blocos sobreviventes. **O berço só
funciona por sorte de ocupação.** Nas ~300 gerações do uso normal medidas não falhou.

## 4. ABERTO, por ordem

1. **Blindar o berço** (escolha do dono, 16/10). Dois caminhos:
   - **(A) T e R em TEXTURAS** (`Rg32Uint`, `texture_2d<u32>` para ler, `texture_storage_2d<rg32uint, write>` para
     escrever; índice 1D → (i & 8191, i >> 13) com o lado do dispositivo). O acesso no `c2_hrc.wgsl` passa por
     poucos pontos (4 leituras: linhas ~238–259 e 549; ~10 escritas; o acumulado fica buffer — escrito inteiro a
     cada quadro, lê sempre à DRAM). Prelúdio de acessores escolhido no `rc!` de `shaders.rs` (como o `saida_*`);
     marcadores vazios em textura (leitura × escrita separados); `solver_hrc.rs` está em 660/700 LOC ⇒ módulo irmão.
     ⚠️ **PREMISSA POR MEDIR:** que uma textura de armazém nova com o «vácuo» constante comprime. A sonda atual
     não serve (§2): antes de reescrever, meça com um núcleo que leia a textura na ordem dos ladrilhos (ou
     compare a c2 de um protótipo mínimo). Kill-criterion: se a c2 com texturas não ficar ≤ a do berço que
     funciona (11,5 no 2560, 8,8 no 1920), o caminho morre.
   - **(B) segurar os blocos do berço** para não serem destruídos — cura a falha medida, mas um buffer ainda pode
     cair num buraco que nunca teve textura (first-fit entre blocos).
2. O resto das 1.ªs montagens (§2): só depois do item 1, e com uma régua de texturas que veja a compressão.
3. Itens 2–3 do §5 da W5x.

## 5. ⛔ Recusas MEDIDAS (somam às da W5x §8)

| Recusa | Medida |
|---|---|
| Os buffers da holográfica (compressão, TLB/página) como causa do resto com berço | Σler e saltos planos nas gerações lentas e rápidas (§2). |
| Relógio da placa, alocações do app, entorno, trabalho do app ao lado como causa do resto | §2. |
| `SOZINHA` (placa ociosa) como cura | Piora: tira a ocupação de que o berço depende (§3). |
