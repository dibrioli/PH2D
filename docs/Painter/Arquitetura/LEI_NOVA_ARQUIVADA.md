# A lei nova do Wet Paint está ARQUIVADA (dono, 05/11/2026)

O dono testou a lei `Pigmento` ([ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md)) em todos os pincéis e decidiu: voltar à lei antiga e arquivar a nova, sem a jogar fora. Não disse quais problemas viu.

- **O produto pinta com `LeiDaTinta::Velha`.** O código da nova continua na árvore (`grid/tinta`, `ph2d-paint-tinta`, `seca_tinta.wgsl`) e só os testes a ligam (`set_wet_lei_da_tinta`).
- **Para ver como ela era:** tag `lei-nova-arquivada` (`410076692a`), onde ela é a de todos os pincéis, com `PH2D_LEI_ANTIGA=1` para comparar. A tag é local até um envio.
- **Medido ao arquivar:** cada evento do traço custa 2 a 3× a antiga (Wet Liner 1,5 → 4,2 ms, Marker 2,7 → 8); 55 miniaturas e 54 larguras mudam.
- ⛔ Não apagar o código nem voltar a propor a troca sem ordem do dono. E12/E13 ficam paradas.
