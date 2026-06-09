# HANDOFF — Smoke do Enio: paleta de pigmentos + franja ramificada + 4K full-res

> Fila de 3 features implementada do início ao fim em loop autônomo implementação→auditoria
> (pedido: *"loop automático de implementação - auditoria do início ao fim, até concluir as três
> features da fila. Smoke só no final de tudo. Ao final de toda fila e antes do smoke, Ship/CI"*).
> Este doc é o roteiro de **smoke visual** depois que a CI fechar verde.

## O que entrou (7 commits, `822f27c6..421a433a`)

| # | Feature | ADR | Commits |
|---|---------|-----|---------|
| F1 | **Paleta de pigmentos reais** (18 pigmentos) + canal **staining** por-célula + **lift** opt-in | [ADR-0081](architecture/decisions/0081-watercolor-real-pigment-palette.md) | 822f27c6 / f9b323f7 (GPU) / 52fdf2d7 (UI) |
| F2 | **Franja capilar ramificada** (fiber-channeled), opt-in | [ADR-0082](architecture/decisions/0082-watercolor-branched-capillary-fringe.md) | e738d45d |
| F3 | **4K full-res GPU-residency** do campo de 32 canais | [ADR-0083](architecture/decisions/0083-4k-fullres-watercolor-field-residency.md) | 54eb671b |
| — | auditoria: teste de paridade combinado + fmt | — | e1e40dc8 / 421a433a |

## Garantia não-destrutiva (a instrução do Enio)

> *"cuidado para não destruir o que já temos; coloque novos parâmetros para introduzir o efeito
> quando o usuário assim desejar, e não sobreescreva o que já temos."*

- Os dois parâmetros novos são **opt-in, default 0**: `Lift` (slider idx 17) e `Branching` (idx 18).
- **`Branching = 0` ⇒ capilaridade BIT-IDÊNTICA** à de hoje (`fiber_factor = 1`, sem supressão).
- **`Lift = 0` ⇒ a passada `cs_lift` nem é despachada** (early-out) → camada depositada intocada.
- Trocar de pigmento só mexe na **granulação** (verificado por teste); não toca lift/branching/cor.
- Brushes salvos antigos abrem sem quebrar (`#[serde(default)]` → lift/branching = 0).
- Auditoria 2-lentes (correção/paridade + não-destrutivo/integração) fechada: ver `e1e40dc8`.

## Smoke visual — passo a passo

Abra o **Painter** num sprite (a demo é 64×64; lembre [canvas-res-64](../) — borrão = canvas pequeno,
não o sim). Painel **Brush Studio**:

1. **Paleta de pigmentos** — cicle o botão de pigmento (cycler). Cada um seta a granulação própria.
   Pinte traços sobrepostos de pigmentos diferentes (ex.: amarelo + azul) **molhado-sobre-molhado** →
   a mistura deve dar **verde subtrativo** real (K–M de 24 bandas), **não cinza/lama**.
2. **Lift** (slider novo, default 0) — pinte um traço, deixe secar parcialmente, suba o **Lift** e
   passe água/pincel por cima de uma área molhada → o pigmento depositado **re-mobiliza** (volta a
   fluir). Pigmentos **staining** (ex.: ftalo) resistem mais ao lift que os **granulando/lift-fáceis**.
3. **Branching** (slider novo, default 0) — com Branching em 0, a franja molhada é o **anel liso** de
   hoje. Suba o **Branching** → a frente capilar fica **lobada/ramificada** seguindo o tooth do papel
   (wicka menos nos vales, ~full nas cristas). Em 0 nada muda (confirme a preservação).
4. **4K full-res** — o campo de 32 canais agora aloca em full-res 4K onde há VRAM (seu Apple Silicon).
   Produção usa grid low-res (canvas/4) por default; isto destrava detalhe fino quando o usuário quer.

## Provas técnicas (já verde, sem smoke)

- **368 unit tests** do `ph2d-painter-brush` verdes (lift/branching/staining/pigmento incl.).
- **Paridade GPU↔CPU em Metal**: lift, branching e staining individuais **+ os três combinados**
  (`gpu_cpu_parity_lift_branching_staining_combined`) → pior |Δ| **~1e-6** em todos os canais
  (ks 24 bandas / err / mass / PIG_STAIN / água), bem abaixo do gate 2e-2.
- **Conservação**: `lift_pigment` é transferência depositado→fluindo (sem criar/perder massa);
  `fiber_factor ≤ 1` preserva a prova de média-convexa/CFL da capilaridade (estável + conservativa).
- **perf_resident** roda os 4 tamanhos incl. **3840×2160 (8.3M células)** sem skip (antes pulava ≥2048
  sob o cap de 128 MiB). Custo/frame é region-scoped (O(frente molhada)), não O(grid).

## Estado / próximos

- Fila de 3 features **concluída**. Smoke é o gate final do Enio.
- Follow-ups de maior fidelidade registrados nos ADRs (não in-scope agora): LBM/MoXi completo
  (dendritos finos fibra-a-fibra, [ADR-0082](architecture/decisions/0082-watercolor-branched-capillary-fringe.md) §2.3);
  tiling esparso 4K (alocar só tiles molhados, [ADR-0083](architecture/decisions/0083-4k-fullres-watercolor-field-residency.md) §4).
