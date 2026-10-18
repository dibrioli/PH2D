# Handoff de continuação — Luz 2D, W6c: o renascimento sem o quadro sem luz (2026-10-17)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6b](HANDOFF_CONTINUACAO_W6b_2026-10-17_A_VISTA_RENASCE.md) (HEAD `3d89ab67ae`, não
> integrada; smoke do dono OK 17/10). Item 1 do §4 da W6b. Placa RTX 5060 Ti, `wgpu` 29.0.4.

## 1. O mecanismo e o kill-criterion (escrito ANTES de medir)

**Quem segura a fluência velha.** Os três receptores — o passe de sprite (`SpriteRenderer::receber_luz`), a colagem
(`BandBlit::receber_luz`) e o blit do Flip (`FlipCompose::receber_luz`) — guardam um `GrupoDeLuz::aceso`
(`ph2d-render/src/luz_recebida.rs`): um bind group com a vista da fluência do `ligar` ANTERIOR. Na C1b (hoje) a vista
larga-se no fim do `gravar_com` do quadro N e o `ligar` de N vê a luz apagada: o bind group velho cai aí, a textura
morre com os envios de N, e o quadro N+1 monta nos mesmos blocos (12 → 12) — mas N sai sem luz. Montar no início de
N+1 (`67c2e1bdac`, revertido) mantém N aceso, mas a fluência velha ainda está presa pelos bind groups de `ligar(N)`
quando a nova se aloca: 12 → 13 blocos.

**As duas curas** (o MESMO binário, `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=adota|solta|0`; sem a variável, a C1b):

- **(a) `adota`** — no início de N+1 a vista velha larga-se MENOS a textura da fluência; a placa espera
  (`poll(wait)`) destruir o resto e a nova monta-se com a MESMA textura (`Solver::novo_adotando`), zerada por um
  passe de computação (uma textura nova do wgpu nasce zerada: os bits da nova). Os bind groups velhos apontam para
  ela: nada fica preso. Não toca na shell.
- **(b) `solta`** — no início de N+1, ANTES do `gravar_com`, a shell solta os três receptores
  (`LuzDoQuadro::soltar_se_renasce`, uma linha em `present_luz.rs`); depois larga-se, espera-se e monta-se como na
  C1b, mas no mesmo quadro que desenha.

**Régua** (a da W6b §2–3): 2560×1080 High, cena 5, 6 000 quadros, `CUSTO=1 SERIE=1`, sem `RENASCE`; (a), (b) e OFF,
6 limpos de cada, intercalados em blocos rodados; c2 = mediana do último terço; os blocos do alocador antes → depois do
renascimento. Limpo = a tabela INTEIRA do `nvidia-smi` (C e G) só com kwin, Xwayland, plasmashell, portal, VSCode e o
app, durante todo o processo, e 60 s de placa livre antes; o Chrome do dono fechado; quem tiver alheio é descartado.

**Passa** (cada cura, sozinha): 6/6 ≤ 11,65 · blocos 12 → 12 em 6/6 · OFF com o evento (≥ 1 processo ≥ 12,2; senão a
rodada não vale e repete) · ≤ 2 renascimentos por processo · os bits iguais **e nenhum quadro sem luz** (o portão
`a_vista_renascida_da_os_mesmos_bits_que_a_nova`: `recebida()` presente depois de TODO `gravar`, o do renascimento
incluído; controlo: a C1b reprova essa asserção).
**Morre**: algum ≥ 12,2 com a cura · renasce > 2× · a imagem muda · um quadro sem luz.
**Nem passa nem morre** (ex.: 12 → 13 blocos, ou 11,65 < c2 < 12,2): não substitui a C1b — o piscar fica, e a medida
entra nas recusas.
**As duas passam**: fica a de menor mediana dos 6; diferença < 0,03 ms ⇒ fica a (a) (sem linha na shell nem
obrigação para quem chama). A perdedora sai do código.
