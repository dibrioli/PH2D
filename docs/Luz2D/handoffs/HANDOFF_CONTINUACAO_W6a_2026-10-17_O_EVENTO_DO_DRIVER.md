# Handoff de continuação — Luz 2D, W6a: o resto das 1.ªs montagens é um EVENTO do driver (2026-10-17)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5z ([handoff](HANDOFF_CONTINUACAO_W5z_2026-10-16_OS_NIVEIS_EM_TEXTURA.md)), W5y e W5x.
> Item 1 do §4 da W5z. Commit da W6a: `688e210b28` (instrumento; nada de produto mudou). **Medição feita, cura
> PROPOSTA e não implementada** (o dono mandou integrar antes, 17/10). Placa RTX 5060 Ti, `wgpu` 29.0.4.

## 1. O instrumento (só com `PH2D_LIGHT2D_CUSTO=1`; sem as variáveis o app é o mesmo)

- `SERIE=1`: e `t <ms>` (relógio de parede desde a 1.ª leitura) — uma paragem vê-se no intervalo (normal ~80 ms).
- `MEMORIA=lista|1`: a lista em TODA janela (antes só na 1.ª; o degrau vem a meio da geração), `memória g<g> w<w>`;
  e as texturas ≥ 16 MiB da lista AOS SALTOS (`SondaDeMemoria::ms_textura_aos_saltos`, 16 texels ao acaso por
  fio: a tradução de endereços, cega à ordem dos ladrilhos). Calibração e holográfica só na 1.ª janela.
- `FASES=1`: e cada etapa por NÍVEL (`fases g w · hrc_sobe · n3 … n4 …`).

## 2. O que se mediu (2560 High, `RENASCE=2`, 4 000 quadros; placa só com kwin antes de cada processo)

Rodada 1: 9 processos (texturas · texturas+FASES · buffers `HRC_TEXTURAS=0`), 1 com alheio (motion-value).
Rodada 2: 8 processos (texturas · texturas+FASES por nível); 3 com alheios (testes de placa da Macros e da
motion-value) e load 233–453 em 4 deles — decide só pelos limpos. Gerações lentas = c2 mín da w1 ≥ piso + 0,4.

- **O degrau (+0,85 a +1,0 ms, os «12,3–12,7») é UM evento por processo**, 0,3–5 s depois de um nascimento e só
  nos 3 primeiros: uma paragem de ~1 s em que a c2 de uma leitura vai a ~30 e depois ~54 ms (assinatura igual
  em todos: 29,7–30,3 / 54,1–54,4). Uso da placa 90–100 %, tráfego de memória 38 → 13–27 %, **memória total
  parada** (3367–3368 MiB) e o alocador do app fixo (2230 MiB, 12 blocos): trabalho INTERNO do driver.
  Fica até a vista renascer; depois dele todo nascimento é rápido (11,47–11,50).
- **Nenhuma textura cai para a banda da DRAM.** Impressão digital fixa em 9/9 eventos: t10 e t11 −9 a −12 %,
  r0 −6 a −8 %, às vezes t1 −5 a −7 % na leitura SEQUENCIAL; tudo o resto igual (±5 %).
- **Saltos iguais (±5 %) em todos os T/R** ⇒ não é a tradução de endereços.
- **FASES:** o excesso está na `hrc_sobe` (2,44 → 3,31–3,36) e espalha-se por TODOS os níveis (n5–n11, +0,10–0,13
  cada) — não nos níveis do grupo. A perda de compressão do grupo ACOMPANHA o evento; o mecanismo está dentro do
  driver e não é visível daqui.
- Os «+0,5» de uma janela num só nível (desce n3/n4/n5, sobe n8/n11) são outra coisa: aparecem sob carga alta,
  numa janela; nos T limpos sem evento (r2 04_T, 05_T) não há geração lenta nenhuma.
- **Caminho em buffers (`HRC_TEXTURAS=0`):** o berço falhou em 3/3 processos desde a g1 (c2 19,8–26) — a
  fragilidade da W5y §3; o caminho só serve de comparação.

Roteiros (fora do repo; reconstruir é trivial): laço sobre `mede_custo_no_app.sh` com casos intercalados, esperar
a placa só com o kwin e registar os alheios; `nvidia-smi … -lms 250` (memória, uso, tráfego) ao lado; os
analisadores alinham a série com o relógio de parede pelo `mtime` do `app.log`.

## 3. ABERTO, por ordem

1. **A cura C1 (proposta, kill-criterion escrito ANTES):** a vista renasce UMA vez quando a placa a degrada —
   assinatura: uma leitura com c2 ≥ 2× o piso e, nas 30 seguintes, o mínimo ≥ piso + max(0,3 ms, 3 %); no
   máximo 2× por processo; bits iguais (o nascimento é determinístico). Régua: PRODUTO (sem `RENASCE`), 2560 High,
   6 000 quadros, cura ON × OFF por variável no MESMO binário, 6 processos de cada, intercalados; c2 = mediana do
   último terço. **Passa** se com a cura 6/6 ≤ 11,65 e sem ela ≥ 1 processo com o evento (≥ 12,2; senão repete) e
   nenhum renascimento sem paragem. **Morre** se algum com cura ≥ 12,2, renasce > 2× ou a imagem muda.
   C1 contorna o driver; evitar o evento pediria saber o que o dispara (não visível daqui).
2. Itens 2–3 do §4 da W5z.

## 4. ⛔ Recusas MEDIDAS (somam às da W5z §5, W5y §5 e W5x §8)

| Recusa | Medida |
|---|---|
| Uma textura da lista (ou os ladrilhos) na banda da DRAM como causa do resto | Nenhuma cai (§2); o grupo perde 6–12 %, não ~45 %. |
| A tradução de endereços (páginas) dos T/R | Saltos iguais (±5 %) nas lentas. |
| Um nível só da holográfica | O Δ da `hrc_sobe` espalha-se por todos os níveis. |
| Alocação do app (ou de outro processo) no degrau | Alocador e memória total parados no evento. |

## 5. Para o integrador

Sem contrato congelado, sem schema. API nova `doc(hidden)`: `SondaDeMemoria::ms_textura_aos_saltos`. LOC:
`quadro_leituras.rs` 380 · `sonda_memoria.rs` 428 (teto 700). CLAUDE.md §5.1 Luz 2D: «Último:» para este handoff.
**O smoke da W5x/W5z ainda não voltou do dono** — integrar é ordem dele (17/10), não aprovação.
