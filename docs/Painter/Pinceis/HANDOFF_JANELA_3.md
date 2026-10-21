# HANDOFF — JANELA 3 → JANELA 4 (2026-10-21)

> Leitor: a JANELA 4 (pintura, textura, efeitos) e a 5. Leia antes: `ESTADO.md` (decisões 8–14 novas),
> `BRIEFING_FAMILIA.md` (o contrato de um agente de família — reutilize-o), `04_BANCADA.md` §9 (a fábrica por
> família). Árvore `Worktrees/line-PainterWatercolor`, `line/PainterWatercolor`, HEAD `c9087dc183`, limpa.

## Como a J3 trabalhou (copie)
Coordenador + um agente `general-purpose` por FAMÍLIA, em paralelo (até 5), cada um com `BRIEFING_FAMILIA.md` +
a lista dos pincéis e os oráculos (`fixtures/INVENTARIO_J3.md`). Vezes: `ferramentas/vez_pesada.sh` (2 corridas
pesadas), `ferramentas/vez_do_app.sh` (Rebelle/ArtRage/Krita). Família reprovada por MOTOR ⇒ um agente «motor da
família» faz o recurso e refaz o exame. No fim, um agente de FECHO (status, regressões, portões por nome, a fábrica
= só aprovados). ⚠️ O scratchpad é partilhado: cada agente na sua subpasta.

## Infra entregue
`938271318c` a fábrica por família nos 4 meios (`tests/it/familias/<família>.rs`: `WET`, `MEIOS`, `MIDIAS`,
`EM_ESTUDO`; as 7 da J4 já existem) · `size_max_px` o tecto de tamanho por preset (decisão 8, `31f4ae9b13`…) · a
bancada aceita a caneta deitada (`c8e414c1d3`, `fixtures/krita_deitado/`).

## Os pincéis (detalhe: `exames/<família>.md`, pranchas `pranchas/<família>/`)
| família | aprovados (entram) | ficam curados, NÃO aprovados | reprovados (em estudo, sem ficheiro) | fundidos |
|---|---|---|---|---|
| basicos | Hard Round · Soft Round · Pixel 1px · Glaze Round · Flat Block-in | — | — | Round Pressure → Hard Round |
| canetas | Mapping Pen | — | Studio Pen · Technical Pen · Ballpoint · Dry Ink | Brush Pen → Studio |
| grafite | — | HB · 2H · 2B · 8B | Erodible (motor grande L17) | Mechanical 0.5 → Hard Round |
| lapis | — | Colored 4B · Scratch | Tilted Shade · Rough Sketch · Craft Flat | — |
| tintas | — | Dip Pen · Ink Bullet · Ink Brush | Reed · Wood · Gloop · Gesinski · Ink Bleed | Fountain → Ink Bullet |
| caligrafia_marcadores | — | Marker Bullet · Marker Liner (→ Digital) | Chisel · Dry Marker · Broad Nib · Script · Shale | Monoline → Pixel 1px |
| aerografo | — | Airbrush | Spray Can · Stipple · Splatter · Spray Drips | Grainy Shader → Airbrush |

**Fábrica: 34** (Wet 27, Digital 7). Seis novos aprovados. Os pré-existentes «não aprovados» ficam pela decisão 3
com a melhor cura medida; o que lhes falta está no relatório da família.

## Motor feito pela J3 (aditivo, desligado por omissão, portão red-first + mutação)
Digital: ponta recta (`dab_quina`) · tecto por dab (opacidade do Krita) · sub-passo fino · borda de AA em px
(`borda_px`) · curva de pressão em S · velocidade desde o 1.º dab · cauda L5 sem bolha · pressão leve espalha ·
papel não é pigmento (K–M). Wet: `TipReservoir` (o borrão da tinta: sobre o seco ✅, sobre o molhado ✗). Todos: o
tecto de tamanho · o dual à frente pelo arco (o rápido = o lento) · a 2.ª ponta deita-se com a caneta · ponta romba
do taper só esmaece.

## A fila de MOTOR que a J3 deixa (o que destrava cada reprovado; medido nos relatórios)
- **Wet**: troca de pigmento entre DUAS tintas molhadas (o borrão na faixa molhada: 5 tintas) · a entrada clara =
  carga na opacidade visível (a cápsula satura ≥ ~400 de pigmento) · capilaridade pelas fibras (Ink Bleed) · o grão a
  modular o tecto do traço (Gesinski) · a borda serrilhada do Rebelle · o Wet a 5 px pontilhado · velocidade no Wet
  (Broad Nib) · dabs soltos (Craft Flat) · a gota do pó por preset (Airbrush) · escorridos por limiar (Drips).
- **Digital**: o glaze do marcador (a mesma tinta escurece a meio do Multiply; Liner/Chisel) · ponta romba que só
  esmaece · ponta afilada CONVEXA (Script/Shale) · estabilizador que alisa sem cortar a curva (0,97 px de tremor) ·
  ponta em rampa dentro do tecto (Tilted) · N imagens na receita de fábrica (Scratch) · mistura espectral (Ballpoint)
  · dureza pelo tamanho (Mapping a 5 px).
- **Spray**: partícula em PÍXEIS · duas populações (véu + gotas) · acumular parado (Krita 0,24 → 1,00) · lei de
  potência no tamanho (Splatter) · sorteio de cor enviesado (Stipple).
- **Grafites/lápis**: caroços maiores na borda · o grão parte o traço de 5 px · pontas em tampa redonda · semear a
  orla por preset (os revisores emparelham pela mesma semente).
- ⚠️ Vários destes servem a J4 (óleo, guache, aguarela usam o Wet): faça-os quando um pincel da J4 os pedir.

## Lições (para os agentes da J4)
- A matriz funde muito: um pincel que só difere em tamanho/curva de um existente NÃO entra (decisão 13).
- Diferença «visível» = confiança média/alta ou repetida por dois revisores (decisão 12).
- O gerador parte do ficheiro que existe: tirar uma chave da receita NÃO a tira do preset.
- Receita curada contra a origem pode partir uma lei do DONO (a 3.ª camada não clareia, as bordas não
  esbranquiçam): corra `a_lei_da_tinta` e `leis_sistemicas` por nome antes de comitar uma tinta.
- Texturas procedurais: os 2 primeiros `params` são o tom; `[0, 0, …]` apaga o dab.
- Custo: todas as medições da J3 com load 8–90 (outras linhas): o relógio é indicativo — a J5 re-mede calmo.
