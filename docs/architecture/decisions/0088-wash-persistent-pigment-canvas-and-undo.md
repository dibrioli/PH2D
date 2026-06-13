# ADR-0088 — Wash: canvas de pigmento persistente + undo por snapshot de campo

- **Status:** ACEITO (Enio 2026-06-13), em implementação.
- **Contexto:** estende [ADR-0086](0086-watercolor-minimal-core-wash.md) §8.1 + [ADR-0087](0087-wash-integration-parallel-watercolor-mode.md).
- **Supersede parcial:** o modelo "bake-por-traço → pixel chato" do wash (o campo deixava de existir
  após assentar). Agora o campo de pigmento **persiste** e é re-composto ao vivo.

## 1. Problema

O Enio quer **dois sistemas de cor selecionáveis** (RGB/Linear vs Kubelka–Munk espectral) e que
**trocar o modo transforme o que já está pintado AO VIVO** (cinza↔verde sem repintar). Isso exige que
a tela seja **pigmento vivo** (concentrações), não pixels chatos. Mas o undo do painter é
**snapshot de `canvas_rgba`** (pixels) por traço — incompatível com um campo de pigmento que
re-compõe a tela todo frame (o undo restaura pixels, o campo os sobrescreve).

## 2. Decisão

1. **Campo sempre em CONCENTRAÇÕES** dos 4 pigmentos-base (encoding único). Os dois modos leem o
   MESMO campo: `linear_compose` (média aditiva das masstones = metamérico, azul+amarelo→cinza),
   `km_compose` (espectral = vibrante, azul+amarelo→verde). Trocar modo = `set_color_model` +
   re-compor → **transformação ao vivo** (sem limpar, sem re-encodar). ([já landado](../../HANDOFF_wash.md))
2. **Canvas de pigmento PERSISTENTE:** o `WashSolver` mantém o campo acumulado entre traços; base
   backdrop capturado 1×; bridge re-compõe só em dabs/troca/assentamento (ocioso = override em cache).
3. **Undo por SNAPSHOT DE CAMPO, sincronizado ao undo global:**
   - O bridge guarda `committed[i]` = readback do campo (concentrações) **após o traço i+1 assentar**.
   - O tool conta `wash_active_strokes` (traços wash commitados e não-desfeitos): `end_stroke` (wash,
     não-vazio) incrementa; `undo`/`redo` ajustam via flags paralelas que marcam quais entradas do
     stack de undo são wash.
   - O bridge **polla** `wash_active_strokes`. Mudou → restaura `committed[want-1]` (ou campo zero)
     no solver + re-compõe full + re-bake. Mantém os snapshots além do ponto desfeito (pro redo);
     trunca só ao pintar um traço novo (invalida o branch de redo). Lag pós-`end_stroke` (antes do
     assentar) é distinguido de redo por `committed.len() < want`.

## 3. Consequências

**Funciona (workflow wash puro):** undo/redo de traços wash + transformação Linear↔K–M ao vivo de
TUDO. Custo ocioso ~zero (override em cache).

**Limitações (adiadas — precisam de integração de LAYER real):**
- **Memória:** cada snapshot = `cw·ch·16 B` (1408×768 ≈ 17 MB) × profundidade. OK em res baixa
  (demo 64²); em 4K precisa de snapshot comprimido OU replay-de-dabs. Cap a profundidade do wash.
- **Readback por traço** (GPU→CPU) no assentar — stall pequeno em res alta.
- **Ferramentas não-wash** no meio de uma sessão wash: o composite usa a base fixa → edições alheias
  são sobrescritas. (Wash ainda não é um layer de verdade.)
- **Save em disco:** só o `canvas_rgba` assado é salvo; o campo de pigmento não persiste entre
  sessões de app. Reabrir = pixels chatos (não mais transformável).
- **Undo interleaved wash/não-wash:** correto pra contagem (flags marcam wash), mas o composite
  sobre base-fixa ainda limita o caso misto.

**Saída futura (não nesta ADR):** o wash vira um **layer de pigmento** de 1ª classe (concentrações
no formato de layer, compositado pelo sistema de layers, salvo, com undo nativo). Aí todas as
limitações acima caem.

## 4. Gates

Testes GPU em `wash_invariants` cobrem o composite/transporte; o fluxo de undo é validado
manualmente (Enio) — não há harness e2e de pen-input headless.
