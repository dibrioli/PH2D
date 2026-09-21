#!/usr/bin/env bash
# Prova de mutacao da METADE VISIVEL da tinta fina (§16 do doc 27): a porta que
# e' dona do plano, a voz do pen-down, a fileira do painel e a cena.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO em tres pontos, porque cada um deles ja'
# mentiu nesta casa:
#   (a) a ancora tem de casar EXACTAMENTE UMA vez — casar zero le-se como
#       «sobreviveu» e casar duas aplica a mutacao no sitio errado;
#   (b) a mutacao tem de COMPILAR — um erro de compilacao le-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, e a populacao e' `passed + failed`
#       — nunca so' `passed` (o `running N tests` CONTA os `#[ignore]`).
set -u
APP=crates/ph2d-app-sculpt3d/src
PAN=crates/ph2d-panel-sculpt3d/src
BK=$(mktemp -d)
cp -r "$APP" "$BK/app"; cp -r "$PAN" "$BK/pan"
restore() {
  rm -rf "$APP" "$PAN"
  cp -r "$BK/app" "$APP"; cp -r "$BK/pan" "$PAN"
  find "$APP" "$PAN" -name '*.rs' -exec touch {} +
}
trap restore EXIT

# ⛔⛔⛔ **UM PROCESSO POR TESTE, e a razão está MEDIDA (2026-09-20).**
# A 1.ª corrida deste arnes deu CINCO abortos «zero testes correram», e o ramo
# que passou a imprimir a prova mostrou o que eles eram: `SIGSEGV` no binario
# de `--lib` da `ph2d-app-sculpt3d` — o defeito NOMEADO no CLAUDE.md §5, que
# mata a corrida INTEIRA e leva o `test result:` com ele. O `nextest` corre um
# processo por teste e le `229/229` em 3 de 3 na mesma arvore.
# ⚠️ *Um arnes que morre a meio nao reporta uma mutacao: ele reporta o proprio
# acidente, e o numero final le-se igual nas duas leituras.*
corrida() { cargo "nextest" run -p ph2d-app-sculpt3d -p ph2d-panel-sculpt3d --lib 2>&1; }

# A populacao honesta e a que o `nextest` diz ter CORRIDO — nunca a que ele
# listou (os `#[ignore]` entram na listagem e nao correm).
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

verde=$(corrida | populacao)
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      # ⛔⛔ **Um aborto MUDO lê-se exactamente como uma mutação que não entrou**,
      #    e foi isso que este arnês fez em 2026-09-20: CINCO abortos, um deles
      #    sobre o próprio CONTROLO (uma linha em branco, que não pode abortar).
      #    Corridas à mão provaram que as cinco de facto SANGRAVAM. ⇒ o aborto
      #    passa a IMPRIMIR a prova, porque *um instrumento que se declara
      #    inconclusivo sem dizer porquê não é mais honesto que um que mente*.
      echo "  ABORTO [$nome]: zero testes correram — as ultimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A PORTA que e' dona do plano ─────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  'Some(c) => Tinta::semeada(c, faces(), k),' \
  'Some(_) => Tinta::nova(mesh.vert_count(), faces(), k),' \
  'M1 garante: o plano novo nasce BRANCO em vez de semeado'

muta "$APP/tinta_da_peca.rs" \
  '        && t.nivel() == k
        && concorda_com(t, mesh)' \
  '        && concorda_com(t, mesh)' \
  'M2 garante: trocar de nivel deixa de reconstruir'

muta "$APP/tinta_da_peca.rs" \
  't.topologia().verts() == mesh.vert_count() && t.topologia().faces() == mesh.faces().len()' \
  't.topologia().verts() == mesh.vert_count()' \
  'M3 concorda_com: a metade das FACES desaparece'

muta "$APP/tinta_da_peca.rs" \
  't.topologia().verts() == mesh.vert_count() && t.topologia().faces() == mesh.faces().len()' \
  't.topologia().faces() == mesh.faces().len()' \
  'M4 concorda_com: a metade dos VERTICES desaparece'

muta "$APP/tinta_da_peca.rs" \
  '        let por_vertice = t.plano_por_vertice().to_vec();
        mesh.colors_mut().copy_from_slice(&por_vertice);' \
  '        let _ = &mesh;' \
  'M5 devolve: o canal por vertice deixa de ser reescrito'

muta "$APP/tinta_da_peca.rs" \
  '    if concorda_com(&t, mesh) {' \
  '    if true {' \
  'M6 devolve: um plano desactualizado escreve na mesma'

muta "$APP/tinta_da_peca.rs" \
  '    let k = k.min(NIVEL_MAX);' \
  '    let k = k;' \
  'M7 garante: o tecto deixa de cortar'

# ── A ROTA: quem segura o plano ──────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  '    if e_a_activa && o_traco_segura {' \
  '    if o_traco_segura {' \
  'M8 rota: um traco NOUTRA peca empresta o plano a esta'

muta "$APP/tinta_da_peca.rs" \
  '        pedir: if e_a_activa || a_peca_tem {' \
  '        pedir: if true {' \
  'M9 rota: TODA peca da cena ganha um plano novo'

# ── A VOZ do pen-down ────────────────────────────────────────────────────
muta "$APP/recusa.rs" \
  '        if self.tinta_fina_armada
            && self.dyntopo_armado
            && (verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo())' \
  '        if self.tinta_fina_armada && self.dyntopo_armado' \
  'M10 recusa: a lente passa a ser o interruptor e nao o verbo'

muta "$APP/recusa.rs" \
  '            tinta_fina_armada: o.tinta.is_some(),' \
  '            tinta_fina_armada: false,' \
  'M11 recusa: a voz nunca arma no produto'

# ── A CENA ───────────────────────────────────────────────────────────────
muta "$APP/scenes_tinta_fina.rs" \
  '        cena.wireframe = true;' \
  '        cena.wireframe = false;' \
  'M12 cena: o arame nasce desligado e o CONTROLO some'

muta "$APP/scenes_tinta_fina.rs" \
  'pub(crate) const LATITUDES: usize = 24;' \
  'pub(crate) const LATITUDES: usize = 96;' \
  'M13 cena: a peca deixa de ser grossa e o degrau some'

# ── A FILEIRA do painel ──────────────────────────────────────────────────
muta "$PAN/state_modes.rs" \
  '            Self::Quatro => Some(2),' \
  '            Self::Quatro => Some(1),' \
  'M14 DetalheDaTinta: dois chips colapsam no mesmo nivel'

muta "$PAN/paint/brush.rs" \
  '    let y = brush_cor::paint_detalhe_da_tinta(ctx, snap, x, w, y);' \
  '    let y = brush_cor::paint_detalhe_da_tinta(ctx, snap, x, w, y);
' \
  'M15 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

# ── O PESO da peça ───────────────────────────────────────────────────────
muta "$APP/objects.rs" \
  '            + self
                .tinta
                .as_ref()
                .map_or(0, ph2d_mesh_colors::Tinta::footprint_bytes)' \
  '' \
  'M16 footprint: o plano deixa de contar no peso da peca'

echo
echo "MUTACAO: $sangram de $total sangram"
