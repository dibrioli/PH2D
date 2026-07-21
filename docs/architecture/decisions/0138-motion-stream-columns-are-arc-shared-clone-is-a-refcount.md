# ADR-0138 — as colunas do `Stream` vivem atrás de `Arc`: clonar é refcount, escrever substitui

- **Status:** aceito (implementado nesta linha, `line/gpu-nodes`)
- **Data:** 2026-07-21
- **Contexto:** C2 da fila §E da auditoria (§B1: o campo era deep-clonado
  ~8-10×/tick no laço de sim). A cura já estava nomeada DUAS vezes no repo — o
  follow-up do próprio ring CPU (*"an Arc/COW column would make the deep clone
  cheap"*) e o padrão `SampleData` do ADR-0120 no áudio.

## Decisão

`Stream.attrs` vira `BTreeMap<String, Arc<Column>>`. A cirurgia é API-estável:
`get()`/`columns()` seguem devolvendo `&Column` (deref do Arc), `set()` embrulha
a coluna fresca — **zero mudança em qualquer consumidor** (o workspace inteiro
compilou intocado). Todo `Stream::clone` do laço — `Cook::checkpoint` (o ring
denso grava por tick!), o prev do `advance_tick`, o hand-off de boundary do
pump, o `state.clone()` dos nós — vira um mapa de refcounts.

**Sólido porque nada muta `Column` in place**: a API não tem `get_mut`, e todo
escritor constrói coluna nova e a `set`a — a mesma imutabilidade em que os
buffers do lado GPU já se apoiam. O gate
(`cloning_a_stream_shares_columns_and_writing_replaces_them`, mutação-testado
com um `Clone` deep-copiante) pina as duas metades: clone compartilha a MESMA
alocação; `set` no clone des-compartilha sem tocar o original.

## Medição (honesta, com o ruído no registro)

Sonda `the_zone_demo_scale_cook_cost` (262k, release, RTX ociosa): CPU
**22,31 → ~18,4 ms/tick** estável quente (−17%; corridas frias/measures sob
carga variaram 27–49 ms — a sonda é single-shot e sensível a load). O grosso do
custo CPU restante é a CONSTRUÇÃO fresca de colunas por tick (`par_build` — 
trabalho por-elemento irredutível deste desenho), não cópia; e o ganho do
checkpoint denso do ring (um deep-clone por tick que virou refcount) nem
aparece nesta sonda, que mede o cook sem o pump. Um `share_from` para
pass-through coluna-a-coluna foi ESCRITO E REMOVIDO na mesma sessão: nenhum
sítio de hoje o consumiria (quem reconstrói coluna-a-coluna filtra ou concatena
— cópia honesta), e API pública sem consumidor é a classe "código morto mente".
