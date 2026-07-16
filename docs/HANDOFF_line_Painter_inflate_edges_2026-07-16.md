# HANDOFF — a borda do Inflate: a bola tem DUAS bordas e elas discordam (`line/Painter`, 2026-07-16)

> **Para o PRÓXIMO agente da linha.** O smoke do Enio aprovou o **Filter Layer para Inflate** (*"ficou muito
> bom!"*) e reprovou a **BORDA**: o relevo cresce com a borda **serrilhada/rasgada**, e — o mais grave —
> *"essa irregularidade externa é imune ao filtro global e ao pincel smooth, nada pode corrigi-la."*
>
> **O diagnóstico está FECHADO e confirmado no código** (não é teoria — os números de linha estão abaixo).
> Não re-derive nada: comece implementando. A linha está commitada e verde; só isto está aberto.

## 0. Protocolo (não pule)

Modo L (worktree `Worktrees/line-Painter`, base `main = 12ccaecd`). **Você NÃO integra, NÃO pusha, NÃO roda
ship** — fecha, escreve handoff, PARA. Fast mode: `git commit --no-verify -- <seus paths>`. Inner loop
`cargo check -p`. Mutações por caminho ABSOLUTO. Restaure mutação por replace reverso com `assert old in s`,
NUNCA `git checkout`. Mutação-RED só vale sobre gate visto VERDE.
**Leia [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) antes de cada passo.**

⚠️ **O gate de LOC mora na `ph2d-editor-core`, NÃO roda com `cargo test -p ph2d-painter-brush`.** Ele já
pegou 2 arquivos meus hoje. Rode `cargo test -p ph2d-editor-core` no fechamento. `sculpt.rs` está a
**698/700** — campo novo lá = split.

## 1. O MECANISMO (leia isto antes de qualquer código)

O Inflate é a **bola** (`render_inflate` em `sculpt_blur.rs`): uma dilatação parabólica separável
(Felzenszwalb) que devolve `hbuf` (a altura) + `sbuf` (o **argmax**: de que texel a matéria veio, como um
offset **INTEIRO** empacotado — `sculpt_offset::unpack_src`).

**O post-pass do orçamento** (`sculpt_blur.rs:470-480`) aplica um **taper**: a parábola só é esfera perto do
ápice, então do equador (`d² = R²/2`) até o alcance o lift é desvanecido a ZERO, **ao quadrado**, C¹ — é o
que matou a prateleira retangular do P0.

```rust
// sculpt_blur.rs:470
let t = if a_s <= 0.0 || d2 >= reach2_s { 0.0 } else { let lin = (2.0 - 2.0*d2/reach2_s).clamp(0.0,1.0); lin*lin };
// :487
let lifted = p0 + t * (hbuf[i].max(p0) - p0);   // a ALTURA desvanece suave
```

**Mas o `sbuf` só é zerado quando `t == 0`** (`:482-484`, `:491`). Dentro da zona de taper ele segue
apontando pro vencedor. E 120 linhas depois, a matéria é copiada **CHEIA**:

```rust
// sculpt_blur.rs:592-607  — a advecção da matéria
let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[ci]);   // offset INTEIRO
if dx == 0 && dy == 0 { continue; }
...
if pre_cover[si] <= cov[gi] { continue; }
cov[gi] = pre_cover[si];                                     // 255 CHEIO — o taper NÃO existe aqui
mat[gi] = pre_mats[si];
rgba[gi*4..gi*4+4].copy_from_slice(&pre_rgba[si*4..si*4+4]); // o alpha do pigmento idem
```

> **A altura desvanece suavemente até zero; a cobertura fica em 255 até o último texel e cai de uma vez.**

A luz **pesa por cobertura** (`impasto_light::paint_body(cover) = cover`), então a silhueta que o artista vê
tem uma borda **BINÁRIA**, cortada em `d² = reach2_s`. E `reach2_s = full_reach2 · a_s` é lido **no
vencedor**, através de um **argmax discreto** — o vencedor muda de texel pra texel num padrão tipo Voronoi.
**A borda É esse padrão, binarizado. É a escada das fotos.**

É a MESMA classe de todo bug que esta linha pagou: **duas coisas que precisam concordar sobre um fato,
discordando** (o filme vs o pigmento · a âncora do aro · seed vs sample · o produto da mordida). Aqui: **a
altura e a matéria discordam sobre onde a forma termina — e a luz acredita na matéria.**

## 2. Por que o Smooth é IMPOTENTE (a 2ª metade, e a mais funda)

Não é fraqueza do Smooth — é **estrutural**. O §5 do plano 18 diz *"o sculpt escreve `h` e SÓ `h`"*, e era
verdade até o **Inflate virar o verbo que MOVE MATÉRIA** (a 2ª rodada de 2026-07-14: *"inflate não
engorda"* — a bola passou a responder **duas** perguntas, que altura e **de onde veio a matéria**). Hoje:

- **O Inflate é o ÚNICO verbo que ESCREVE `covers`/`mats`/`canvas_rgba`.**
- **NENHUM verbo consegue EDITAR `covers`.** Smooth/Sharpen escrevem só `heights`.

A borda que o Inflate cria é **write-once, para sempre**. A 3ª foto do Enio é exatamente isso: o interior
(`h`) alisou, a borda (`cover`) não se moveu um pixel. O `SculptMode::moves_matter()` é a porta única que
diz quem é dessa família — mas não existe a porta simétrica: *quem pode CONSERTAR a matéria?*

## 3. A DIREÇÃO DE FIX

### 3.1 — A matéria segue o MESMO taper da altura (o conserto 1 do Enio)

*"Subdivisões mais finas + um smooth embutido"* — **o taper já é as duas coisas**: ele é a gradação
sub-texel e é o alisamento. Ele só não chega na matéria.

```rust
let v = f32::from(pre_cover[si]) * t;              // a cobertura desvanece ONDE a altura desvanece
if (v as u8) <= cov[gi] { continue; }
cov[gi] = v as u8;
```

⚠️ **O `t` mora no post-pass (`:470`) e a advecção roda depois, sem ele.** Duas cópias da fórmula
DIVERGEM (a lição desta linha inteira) — então **extraia `ball_taper(d2, reach2) -> f32` para
`sculpt_offset.rs`** (o dono do `unpack_src`/`blob_dilate`) e faça os DOIS sítios perguntarem a ela.
Recomputar na advecção é barato: `(dx,dy)` vem do `sbuf`, `d2 = dx²+dy²`, `a_s = amount[si]`,
`reach2_s = full_reach2 · a_s`.

⚠️ **O ALPHA DO PIGMENTO também** (`rgba[si*4+3]`): a silhueta vermelho-vs-branco das fotos é o alpha do
`canvas_rgba`, não a cobertura. Se só a cobertura desvanecer, a *sombra* suaviza e a *tinta* continua com
recorte duro. O que chega tem opacidade `t` e deve **compor `over`** o que está lá — é o mesmo operador
que o `commit_stroke_height` usa pro material, e pela mesma razão (cobertura é PRESENÇA → `max`; material
e cor são IDENTIDADE → `over`).

### 3.2 — A borda tem que ser EDITÁVEL (o conserto 2 — decisão de arquitetura, não um fix)

O Enio: *"vc precisa encontrar um modo de deixar as bordas sensíveis às edições dos filtros e dos
pincéis."* Isso **revoga o §5 no ponto exato onde o Inflate já o revogou**: se um verbo pode escrever
matéria, algum verbo tem de poder editá-la.

Candidato (não implementado — **projete antes**): o Smooth, quando o alvo tem matéria, borra também a
**borda da cobertura** (o alpha), não só `h`. Perguntas que a decisão tem de responder ANTES do código:
- Isso borra a ARTE (a cobertura é o alpha do pigmento) ou só a franja que o Inflate fabricou? Um Smooth
  que come a borda de uma pincelada que o artista pintou à mão é um bug pior que o serrilhado.
- Precisa de um `moves_matter()` simétrico (`edits_matter()`), com **porta única**?
- O 3.1 pode DISSOLVER o sintoma (a borda nasce macia e não há o que consertar). **Meça primeiro:** se
  depois do 3.1 a borda ficar boa, o 3.2 vira uma capacidade a nomear, não uma urgência — e o Enio decide.

## 4. Estado da linha (tudo commitado, verde; NÃO refaça)

| | |
|---|---|
| `fd77f9c5` | âncora do aro no CORPO (`rim_t0`/`rim_lift`) — **smoke OK** |
| `2e1806fb` | a mordida é função do CAMINHO (share sobre a SOBRA telescopa) — **smoke OK** (o Push ficou mais forte: Push=1 limpa o canal; knob ≈0,63 devolve o antigo) |
| `57d9881e` | **W5b** — o filtro de camada inteira (botão Filter Layer) |
| `ea0a5c02` | **W5b** — 2 escopos (Layer + **último traço**, mascarado por `relief.live_paint`) + **Layer cortado da lista** (knob morto: a luz lê `∇h`, e uma constante não tem gradiente) |
| `493665c2` | sondas 7/8/9a/9b do filtro |

Gates: tool **705** · brush **255** · seam_sculpt **15** · clippy **0** · `test --workspace` verde · LOC cap
verde. Mutações da jornada: **9/9** (W5b) + 3 (âncora) + 1 (mordida).

**Aprovado no smoke e NÃO deve se mover:** Conserve · Push · Filter Layer/Stroke para Smooth · **o Inflate
por-traço** (só o filtro de camada expôs a borda — o pincel Inflate usa o MESMO `render_inflate`, então
o 3.1 melhora os dois; **re-smoke do Inflate por-pincel declarado**).

## 5. O INSTRUMENTO

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_PUSH_LOOK_DIR=/tmp/look cargo test -p ph2d-host-desktop probe_push_render_and_look -- --ignored
```
Cena **8** (`8_filter_layer_inflate`) é o Inflate de camada inteira — **a cena da borda**. ⚠️ A laje da
sonda é uma forma retangular grande e a borda serrilhada aparece pouco nela; o repro do Enio é um **blob
com relevo alto e borda curva** (as fotos). **Acrescente uma cena com a forma dele** — a fixture TEM de
conter o fenômeno, e esta linha já foi mordida 3× por fixture que não continha (o falloff macio da âncora,
o Sphere da coria, o polígono da lasca do Vector).

Gate a escrever (**red-first**): *num blob de relevo alto, a borda da cobertura depois do Inflate é
**monótona e suave** — a diferença de `cover` entre texels vizinhos ao longo da borda não pode dar saltos
de 255* (hoje dá: 0→255 num texel). E o irmão: **a borda do `cover` termina onde a do `h` termina** (as
duas concordam sobre onde a forma acaba) — é a afirmação direta do bug, e nasce VERMELHA.

Smoke do Enio: `PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop` → tinta grossa → SCULP →
Inflate → **Filter Layer**. O certo: a forma engorda com a borda **macia e redonda**, e o Smooth (filtro ou
pincel) consegue mexer nela.

## 6. Aberto depois disto (a fila do impasto inteiro)

1. **Este handoff** (a borda do Inflate: 3.1 e depois a decisão do 3.2).
2. **Passe de luz na GPU** — a luz é CPU, e **enquanto for, relevo visível DESLIGA o compositor GPU
   inteiro** (`painter_gpu_preview::gpu_eligible` retorna `None` se `impasto_visible()`). Exige
   reconciliação bit-a-bit contra a CPU (doc 16 §6).
3. **Relevo do PAPEL** — acopla impasto↔aquarela: **exige ordem nova do Enio** (§2 do doc 16 é barreira).
4. Dar ao **BANCO** do Push a cura que a mordida ganhou (cada dab ainda normaliza o próprio aro = um
   produto sobre a lista de dabs; residual medido 0,0286 — invisível hoje).
5. Conserve p/ Flatten/Fill (decisão de design) · perf do Deform não é gateada · knob de `forward_share`?
