# E6b — o que o pincel LEVA e as ferramentas movem somas (31/10)

Plano: `F4_2_plano_wet_paint.md` §4 (pick-up, re-molhar, arrasto/smear/blend), R18, §7 E6b. Só com
`Sim::tinta == Pigmento`; a lei `Velha` não passa por nenhum código novo (cada porta é `if p.tinta == Pigmento`).

## O que mudou

| sítio | o quê |
|---|---|
| `wp/trail.rs` (campo `ponta_tinta`, Default, `start_stroke`) | a `Soma` que cada texel da ponta levou da tela; vazia sem `Pigmento` (+4 linhas, 699) |
| `wp/trail/bico.rs:20` | `tip_rows` recebe `&mut Grid` (o único chamador, `transfer.rs:49`, já passava `&mut`); no fim, a porta `if p.tinta == Pigmento { ponta_pigmento }` |
| `wp/trail/bico_tinta.rs` (novo) | `Trail::ponta_pigmento`: a troca ponta ↔ tela |
| `wp/tools/tinta.rs` (novo) | `Foto::tira`/`segue`: a soma segue a massa nas ferramentas que relaxam para uma média |
| `wp/tools/smear.rs` (`smear_com`) | `Foto` sobre o recorte `r` do Smear (e do arrasto do re-molhar) |
| `wp/trail/transfer.rs:~480,~682` (`transfer_blend`) | `Foto` sobre a janela inteira; `segue` depois do laço (inclui o lift do blend) |

### A ponta (pick-up)
Por texel `l` sobre a célula `i`, com `k = retido(fe, …)` (a fracção da cor que o bico velho guarda):
- recolhe `f = 1 − k`: `pick_up(filme → ponta, f, o=1, 1)` e `pick_up(aberto → ponta, f, o, assentado)`
  (`assentado` = `pickup_assentado`, ou 1 com o reservatório). `susp`/`sett` caem pela MESMA fracção (invariante exacto).
- onde a janela pousa (`pig[l] > 0`) solta `solta = Pickup` (ou `TipReservoir`) do que levava: a massa solta soma-se a
  `pig[l]` (`· M_UM`) e a cor da ponta (`tip_r/g/b`) passa a `srgb(devolvido ⊕ Soma::de(base, pig/M_UM))` — o
  `deposito_tinta::pousa` (E6a) pousa-a como já fazia. `limpo[l] = pig/(pig+solto)` na cor por célula.
- `TipClean` tira a fracção `clean` da ponta (sai do sistema, como a limpeza da cor).
Tela + ponta conservam-se; a tinta nova do pincel e a limpeza são as únicas fontes/sumidouros.

### Ferramentas que relaxam (Smear, Blend)
A massa relaxa para uma média (lei velha intacta). Depois, por célula da pegada: fatia que PERDEU massa perde a
mesma fracção da soma (`transfere`, a cor dela); fatia que GANHOU recebe `Δm/M_UM` na cor média pelo volume das somas
da pegada ANTES da ferramenta. Invariante exacto; Σ soma segue Σ massa.

### Re-molhar
Já feito pela E6a dentro de `Pigmento::pousa` (`transfere(aberto, filme, 1 − sett_depois/sett_antes)`); `o`
ainda não entra (é 1 até à E8). `drying.rs:41 lift_settled` é da E8 — não tocado.

## Testes (por nome)

| teste | resultado |
|---|---|
| `o_pick_up_leva_o_aberto_pela_abertura` | ver relatório (conserva rel. < 1e-5; perda do aberto com `o = 0,5` = metade da de `o = 1`) |
| `a_ponta_suja_pousa_a_mistura_pela_soma` | ponta azul + pincel amarelo sobre papel: `L` do pousado entre os dois |
| `o_smear_conserva_somas` | relaxação que conserva a massa + `Foto::segue`: invariante por célula, Σ v·M_UM = Σ susp |

## Em aberto
- `wp/tools/{arrasto_agua,secador,bordas}.rs`, `wp/tools.rs` (10 escritas de massa), `wp/trail/transfer/perto.rs`,
  `wp/lamina.rs` (faca, R18): NÃO ligados — a mesma `Foto` serve-lhes (tira antes, segue depois); falta o recorte de
  cada uma.
- O Blend com `BlendReach > 0` (médias LOCAIS de `perto`) dá à massa a média local e à soma a média da janela.
- O lift do blend (sett → susp) chega ao filme com a cor média do filme, não a do aberto que saiu.
- `mistura_o_reservatorio` não promedia as somas da ponta (só a cor velha, que a porta sobrescreve onde pousa).
- A transparência da tinta solta perde-se (a pousada leva a do material do pincel).
- `ponta_tinta` zera quando a janela muda de tamanho (como `mat_ponta`).
- Com `Pigmento` a MASSA deixa de igualar a da lei velha (o pick-up tira-a da tela):
  `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha` (E3, `grid/tinta_tests.rs`) compara a massa — ver relatório.
