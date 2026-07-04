---
name: project_wash_pigment_color_mixbox_residual
description: "Cor de pigmento (wash K–M) = Mixbox residual; cor sozinha fiel, só mistura é espectral (ADR-0091)"
metadata: 
  node_type: memory
  type: project
  originSessionId: da867ef3-9b65-4b2c-b452-604f23cca0f9
---

O modo **Pigment** (K–M) do wash colapsava cores distintas (vermelho/laranja/amarelo→o mesmo laranja,
2 azuis→1 azul; test strip do Enio 2026-06-14). Causa: a "K–M ingênua" do ADR-0089 normalizava TODA cor
para uma magnitude de referência fixa (`K_REF`) e tirava a luminosidade só da cobertura → **descartava
o VALOR** da cor escolhida.

**Fix = estado da arte (não escolha pessoal):** o **Mixbox** (Sochorová & Jamriška, *Practical Pigment
Mixing for Digital Painting*, SIGGRAPH Asia 2021 — o modelo do **Rebelle**) representa cada cor como um
latente **pigmentos + residual**: `c = unmix(rgb)` (NNLS, sem magnitude fixa) + `r = rgb − mix(c)`. O
composite decodifica `mix(c̄) + r̄`. Como `mix(c)+(rgb−mix(c)) = rgb`, uma **cor sozinha reproduz EXATA**
(identidade por construção); só a MISTURA wet-on-wet de cores diferentes mostra o pigmento espectral
(azul+amarelo→verde). Requisito que o paper crava: *"handle all RGB colors without clipping or
distortion"* — nunca distorcer uma cor sozinha. Implementamos a **técnica** (residual), não a lib
Mixbox (licença não-comercial).

**No nosso código (ADR-0091, commit 6030156b):** `km.rs` ganhou `unmix`/`pigment_residual`/
`compose_km_mixbox`. O solver ganhou um canal `res` (signed premul-RGB) transportado igual ao `dye`;
o `FieldSnap` do undo inclui `res` (regra "restaure todo estado dinâmico", [[project_wash_undo_event_driven_rebuild]]).
Gotcha: o step bateu no **limite de 8 storage-buffers/stage** com res×2 — liberei removendo o binding
`paper` (inerte no gate desde o B5; granulação v1.1 re-adiciona). Cap unificado por massa p/ `c̄`/`r̄`
consistentes. Validado no Metal: vermelho [0.7,0.1,0.1]→sRGB(218,89,89); green-excess 53 vs −6 Linear.

**Picker:** a fidelidade é da ENGINE (cor pintada = cor escolhida), então o seletor já é WYSIWYG sem
transformação extra — o decode de uma cor sozinha é identidade. Postmortem: `docs/Painter_projeto/wash_solucao_de_erros.md` B9 + §0.8.
