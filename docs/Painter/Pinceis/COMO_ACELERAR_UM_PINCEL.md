# Como medir e acelerar um pincel (caso: Wet Wash Mix, Bug #84)

> Guia de referência para a próxima LLM. Fonte dos números: [Bug #84](../BUGS_painter.md) (curas 1–5) e
> [`exames/custo_no_app.md`](exames/custo_no_app.md) (o mesmo enigma no Filbert: a bancada lia lento, o app era 12 fps).
> Regra-mãe: **onde o custo aterra ≠ o que o provoca**. Meça no app, reproduza com a velocidade do dono, ache a peça por ablação.

## 1. Método em 4 passos (a ordem importa — medido)

| # | Passo | Comando / instrumento | No caso Wet Wash Mix |
|---|---|---|---|
| 1 | **Medir NO APP primeiro** | `PH2D_PAINT_PERF=1 ./target/smoke/ph2d-host-desktop 2>&1 \| tee ~/x.txt`; ler `EVENTO->FRAME` (`periodo real`, `eventos/frame`, `INPUT move p50/max`) e `[frame] stamps: media … (N entregas, X ms cada)` | 8,74 ms/entrega × 9,4 entregas/quadro, período 96,5 ms (6 fps); prévia, painéis, chrome e upload ~0 |
| 2 | **Reproduzir na sonda do tool** com a VELOCIDADE do dono | bloco abaixo | 65–67 ms/quadro a 360 px/quadro = o número do app |
| 3 | **Achar a PEÇA por ablação**, não por perfilador | `PH2D_SONDA_ABLA='brush.spray_count=1;brush.texture=null;brush.dab_jitter=null'` (variantes separadas por `;`, chaves por `,`; JSON com vírgula não cabe) | sem o véu (`spray_count 1`) 11 ms de 66: o véu era ~83 % |
| 4 | **Dividir a peça por fase** com relógios `Instant` temporários (`AtomicU64` acumulado, impresso a cada N carimbos); comparar série × paralelo (forçar `faixas_no_pool` a 1, em `crates/ph2d-painter-brush/src/dab/bands.rs`) | `Instant` + `AtomicU64` | por carimbo de véu (~10 k texels): kernel/pegada 107 µs (32 ns/visita em série, só 3× com faixas), película 76–86 µs (17 ns em série, só 2×) → **o tecto era o fork-join POR CARIMBO, não o trabalho** |

**Passo 1 — a lição.** A sonda antiga dava ~35 fps porque andava 6 px por evento; a caneta do dono entrega ~40 px por evento
(~4 000 px/s) num traço rápido. O custo de pincel de cápsula/véu é **por px de caminho**: reproduza a velocidade do dono
na sonda antes de qualquer cura.

**Passo 2 — a sonda** (`crates/ph2d-tool-painter/tests/it/sonda_custo_no_app.rs`; `--profile smoke`, nunca `--release`):

```bash
cd <worktree> && PH2D_SONDA_PRESETS="<pincel>" PH2D_SONDA_EVQ=9 PH2D_SONDA_VELS=10,40 \
  PH2D_SONDA_TAMS=30 PH2D_SONDA_TELA=1024 \
  bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it sonda_custo_no_app -- --ignored --nocapture
```

- O quadro do app fora do ponteiro (produtor, drenagem, upload, placa) mede-se à parte (`app_frame` + GPU): 1–2 ms ⇒ o custo era todo do carimbo.
- Tamanho da tela (1024/2048/4096) e papel + camada transparente × tela branca: **não mudam nada** (±1 ms) — não gaste rodadas nisso.

**Passo 3 — sem perfilador.** `perf`/`samply` estão **BLOQUEADOS** nesta máquina (`perf_event_paranoid` 2, mesmo `cpu-clock:u`).
A ablação (chave do motor trocada no MESMO processo, intercalada, o MÍNIMO — [MEDIR_VELOCIDADE](../../DevOps/MEDIR_VELOCIDADE.md))
mais relógios `Instant` temporários são o instrumento. Remova os relógios antes do commit.

## 2. As curas que ficaram (Bug #84)

| # | Cura | Mecanismo | Ganho medido | Aparência |
|---|---|---|---|---|
| 1 | Saltar a cópia do spray no mesmo centro (`ReservatorioOpaco::repete_o_centro`, `mistura_opaca.rs`) | `dist = 0` não move o reservatório e põe só o `k_passo` mínimo, mas pagava a pegada inteira | 30 px 2,50 → 1,29 s (1,9×) | **muda** (Δ médio 0,8–1,0, máx 8–10); aprovado |
| 2 | Película em faixas no pool + taxa por tabela de 256 + kernel no pool ([ADR-0200](../../architecture/decisions/0200-o-carimbo-de-mistura-e-o-kernel-em-faixas-no-pool-do-rayon.md)); plano do traço compõe 1× por quadro (`PlanoDoTraco::pendente`, `traco_plano.rs`) | tira a potência por texel do fio único; tira a composição por evento | 30 px 1,29 → 0,66 s (3,8× somado ao plano: 0,60 s) | ao bit |
| 3 | Memória do grão Tiled (`texture/grao_cache.rs`) | com Tiled o `sample` só depende do píxel; cada píxel recebe ~90 dabs → guarda a 1.ª amostra (blocos 64²) | grão 75 → 27 ns/visita (3× no kernel); 30 px 0,585 → 0,349 s | ao bit |
| 4 | Pegada lisa na MESMA visita (`dab/liso.rs`) | o kernel escreve o `liso` antes do grão/dente/stencil; some a 2.ª passagem | 30 px 0,360 → ~0,31 s | ao bit |
| 5 | Pegadas do lote em paralelo ENTRE os dabs (10/11, `6537d5309a`) | a pegada de um dab **não lê a tela**, só a película lê: `pegada_textured_masked_po_com_liso` → `PegadaLocal` (buffer da caixa do dab, `DabCtx::ox`; a película lê por `Pegada::Mascara::{org, largura}`); a cópia no centro da anterior (a que a cura 1 salta) não se calcula — calcula-se na hora se não saltar | 360 px/quadro **67 → 41 ms**; 90 px/quadro 17,8 → 12,7 ms | ao bit |
| 6 | Mistura a MEIA definição (10/11, `stamp_route/mistura/meia.rs`, `MisturaOpaca::meia_definicao`, só o Wet Wash Mix) | no plano do traço, pegada e película numa tela de metade do lado (dab ÷ 2, grão `size` × 2 — é frequência), amostra por médias 2×2 pré-multiplicadas, ampliação bilinear pré-multiplicada por lote | 360 px/quadro **43,5 → 24 ms**; 90 px/quadro 13,4 → 8,7 ms | **muda**: Δ médio 0,35–0,7, máx 14; o grão fino com metade da energia — **aprovada pelo dono 10/11** |

Prova da cura 5: gate `a_pegada_na_caixa_e_a_da_tela_ao_bit` (`dab/tests.rs`, vermelho por mutação da origem), `.ppm` da
`sonda_custo_do_veu` iguais em `cmp`, alarmes de pincel verdes. Smoke do dono 10/11: «muito melhor e aceitável».

## 3. ⛔ Recusas MEDIDAS (não reconstruir)

| Ideia | Resultado |
|---|---|
| Afinar o véu a ¼–½ passo (saltar o que anda menos que isso) | 3,3× e 5,6×, mas listras verticais, borda ondulada, Δ máx 48–73: o liso é a MÉDIA de muitos carimbos sorteados |
| Faixas mais finas no kernel (512/256 visitas por faixa) | 106 → ~63 µs e **satura**: o fork-join come |
| Pintar a película por ladrilho 64×16 | ~3× as tarefas, o mesmo tempo, ao bit |
| Pegadas do lote em paralelo, cada uma em SÉRIE por dentro | o lote tem só ~3 carimbos grandes por evento de 6 px: 0,29 → 0,77 s, **mais LENTO**; é preciso o aninhado (entre dabs E faixas dentro de cada um) |
| Calcular as pegadas que a cura 1 salta | metade do trabalho desperdiçado |

## 4. Regras gerais para outros pincéis

- Custo de um pincel com mistura ≈ (carimbos por px) × (área da caixa) × (ns por texel): **meça os três**. Um carimbo grande e frequente (véu, spray com escala) domina.
- Paralelismo dentro de UM carimbo de ~10 k texels rende só 2–3× com 32 núcleos (fork-join = dezenas de µs). Paralelize **ENTRE carimbos** onde não há dependência (a pegada); só o que lê a tela (a película) fica em fila.
- Piso em células que decide série × paralelo mente se o custo por célula varia (caso Filbert, [custo_no_app](exames/custo_no_app.md)): meça o pincel real.
- Toda cura «ao bit» prova-se com `cmp` das `.ppm` + os dois alarmes (`a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) + um gate com mutação.
- Cura que **muda a aparência** vai ao dono lado a lado (relatório `target/relatorio-pinceis/index.html`); só ele regrava.
- Um gate de custo precisa de pincel de mistura na população (o `o_custo_no_caminho_do_app` só media Filbert e Express Oil e ficou verde com o app a 6 fps).
- Smoke/sonda sempre em `--profile smoke`; load acima de ~5 invalida o relógio ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).

## 5. Aberto / próximo

A cura 6 foi aprovada pelo dono (10/11; lado a lado em `../handoffs/material_2026-11-10_veu_meia/`). **Aplicá-la a
outro pincel:** (1) medir TODOS os de fábrica à velocidade do dono (`sonda_custo_no_app`, 9 × 40 px) e listar os que passam
de 16,7 ms/quadro; (2) dos lentos, só os de marca LARGA e MACIA (véu, aguada, aerógrafo, esfuminho) — a meia definição
come o detalhe de 1–2 px (grão fino, dente do papel, borda dura, píxel); (3) ligar `meia_definicao` como variante em
execução (`PH2D_SONDA_ABLA='brush.mistura_opaca.meia_definicao=true'`) e mostrar o lado a lado; (4) só o dono aprova, pincel
a pincel. Hoje a porta só arma na rota da mistura dentro do plano do traço; um pincel lento fora dela pede estender a
porta (a tela directa, a rota sem mistura) — meça antes. O que resta a 360 px/quadro
(~24 ms) não foi dividido por fase — meça antes de cortar.
