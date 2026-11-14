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

**Passo 3 — sem perfilador.** `samply` e `perf record -g` estão **BLOQUEADOS** nesta máquina (`perf_event_paranoid` 2); o `perf` simples (função folha) abre o app — §6.
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

## 6. Medir o PEN-UP (soltar o mouse) no app real — Bug #85

O custo do pincel (§1) é do traço; o do **mouse-up** é outro: tela inteira refeita depois do Up. **Lição:** a sonda sem o
papel/relevo do dono não tinha o fenómeno (`sonda_pen_up_4k` e `measure_o_quadro_do_pen_up_em_4k` leram 1,6–7 ms; o app, 1,7 s).
Meça no app, com `PH2D_COMPOSITE_PAPEL=1`.

1. **Compilar com nomes:** o perfil `smoke` herda `strip=symbols` do release; o `perf` sem nomes é inútil. Só religar:
   `CARGO_PROFILE_SMOKE_STRIP=none bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`.
2. **Correr o roteiro** (Bash com o sandbox desligado, por causa do `perf`):
   `bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh target/smoke/ph2d-host-desktop <rotulo> 3840x2160 <digital|water|impasto|wet>`.
   Xvfb `:94` + rato real por XTEST; 6 traços de 41 eventos, 2,5 s parado depois de cada Up. O `.py` irmão resume.
3. **O `perf` ABRE o app** (`perf record … -- <app>`): o `ptrace_scope` do Yama recusa `perf record -p` num não-filho.
   `-k CLOCK_MONOTONIC -F 997` para casar as amostras com os `[quadro-lento]`; **sem `-g`** (com `paranoid` 2 a cadeia não abre:
   amostra só a função folha — some o «quem chamou», some a atribuição por camada).
4. **`PH2D_PAINT_PERF=1 PH2D_QUADRO_LENTO_MS=<n>`** imprime `[quadro-lento] parede=<ns> intervalo=<ms> quadro=<ms>` por quadro
   lento. O agregado do paint-perf (janela de 90/120 quadros) esconde um quadro só, e um quadro à ESPERA (fila, GPU) não
   aparece no perfil de CPU — o intervalo fim-a-fim sim.
5. **Leia o instante:** o quadro lento a ~0,85 s depois do Up são os `QUADROS_PARADOS = 30` do `follow_live` (a folha de animação
   relê `imagem_aplicada`). Se o atraso mudar, o suspeito mudou — não assuma.
6. **Cura de passe sobre a tela inteira:** faixas no pool que leem o campo UMA vez por píxel (moldura de 1 px), o passe antigo
   guardado como oráculo `#[cfg(test)]` e um gate ao bit (inteira e sub-região). Foi 1 776,5 → 153,9 ms (`sonda_luz_4k`).
Aberto: a leitura incremental ([prompt 10f](../handoffs/PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md)).

## 7. Missão 4K (10–11/11): técnicas que funcionaram

Medidas a 4K na Radeon integrada (nunca a RTX), A×B no mesmo binário; o detalhe por item no [BACKLOG](../missao_4k/BACKLOG.md).

| Técnica | Onde | Antes → depois |
|---|---|---|
| **Custar pela CAIXA, não pela tela**: cada passe recebe a região mudada (dabs do passo, união dos sujos) e o antigo fica como oráculo `#[cfg(test)]` | balde (B48), aguada pen-up (B16.2), enterro do Wet (B02.A3), premul/mips da prévia (B20a), planos Gaussiano/Nitidez (B15.5) | 11,3→3,2 ms · 48,5→1,95 ms · visitas 77 473→12 100 · planos 434→3 ms |
| **Campo esparso por pedaços** (`CampoEsparso<T>`, 4096 células; tiles mudados no undo) | `Grid::aparencia` (B12b), `StoredPlane::changed_tiles` (B17b), RewetCache 64² (B16b) | Wet dab r30 897→26 MB · undo Impasto diagonal 265→32 MB · Rewet 201→3,1 MB |
| **Pool do rayon com a mesma partição** (faixas de linhas, sem redução: o byte não depende do nº de fios) | compositor e ajustes (ADR-0202), Feather em janela deslizante (B40.A2), cópia do par_clone em blocos de 256 KiB (B17a) | 1 044 fios do SO → 0 · Feather r64 11 121→16,8 ms · cópia 7,72→1,67 ms |
| **Pó/dente por ladrilho 256²** em vez da tela | `po_do_papel` (B49.1) | pen-down Pastel 4K 6,65→2,62 ms |
| **Memória própria do dente** no `GraoCache` (sem reamostrar) | B26.A2 | 0,149→0,045 ms/dab |
| **Silhueta base da ponta memorizada** | Flat Bristle 240 (B14c) | 7,11 M→1,89 M avaliações/dab (mas só −4,9 % no traço: a silhueta já não é a alavanca) |
| **Uma cópia de tela a menos no Ctrl+Z** (journal com dono único) | B17b-cont/B17.4 | Digital undo 11,6→7,8 ms; Impasto 20,0→12,6 |
| **Saltar a camada Normal com α≤0** quando o bit é o mesmo | `blend_window` (B19.11) | 4K vazias 38,1→19,2 ms |
| **Recompor só o que o arrasto muda** (quadro reciclado no `CompositorCache`; a seleção só reescreve a máscara) | B19b, B16.7 | arrasto de ajuste 4K 1 149→134 ms; 8 recomposições→0 |
| **Prévia inteira sob interruptor** (`PH2D_PREVIA_INTEIRA=1` volta ao antigo) | B20a-cont | quadro 4K 38,8→10,2 ms; toque/traço/Ctrl+Z 32–38→15–17 ms |
| **A CPU compõe o papel desde o armar do Watercolor** (a troca GPU→CPU sai do quadro do toque) | B16c | toque 4K 186→32 ms |
| **Quadro a pedido** (`quadro::Laco`) | B23 | janela oculta 97–100 %→44–46 % de CPU |

## 8. ⛔ Recusas MEDIDAS da missão 4K (não reconstruir)

| Proposta | Medição | Porquê |
|---|---|---|
| Faixa de 64 linhas na luz do Impasto (B15.1) | 2× mais lenta que 32 | a faixa de 32 fica |
| Cache de afim/ECS por quadro na fila da caneta (B27.3) | mesh_uv 4,2 µs/evento (1 sprite), carimbo 2,7–4,4 ms | ≤ 1 % do evento |
| Registo do dab no worker Wet (B12.1) | 2 rodadas A·B: fora do topo do perfil; `marca_dab` no pouso já existe | sem alvo |
| `Publicado<T>` para o caso (e) do quadro a pedido (B23.4) | ≤ 0,3 ms < 1 ms | não paga a peça nova |
| Ladrilho 512 do papel no Wet (B33.2) | o Wet do app usa o papel comum (`sample_tiled_rot_wrapped`) | o defeito não existe no produto |
| Limiar de «célula mudou» na óptica do Wet (B14-cont2) | 85–88 % das células mudam ao bit | a margem não é a causa |
| Tabela de `hash2` por composite (B16d) | ganho < 0,1 ms | não paga |
| Rewet esparso SEM caminho híbrido (B16b) | memória 201→3,1 MB, mas traço 9,1–10,0→12,6–12,8 ms | só com caminho híbrido (aberto) |
| Relógio de parede a load > 5 como gate (B10.4, B27.1) | sem forma comum entre rodadas | contar ciclos/instruções ou medir a load ≤ 5 |
