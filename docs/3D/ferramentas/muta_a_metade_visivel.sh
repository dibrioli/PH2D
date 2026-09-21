#!/usr/bin/env bash
# Prova de mutacao da METADE VISIVEL da tinta fina (§16 do doc 27): a porta que
# e' dona do plano, a voz do pen-down, a fileira do painel e a cena.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO em QUATRO pontos, porque cada um deles ja'
# mentiu nesta casa:
#   (a) a ancora tem de casar EXACTAMENTE UMA vez — casar zero le-se como
#       «sobreviveu» e casar duas aplica a mutacao no sitio errado;
#   (b) a mutacao tem de COMPILAR — um erro de compilacao le-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, e a populacao e' `passed + failed`
#       — nunca so' `passed` (o `running N tests` CONTA os `#[ignore]`).
#   (d) a corrida LIMPA tem de estar VERDE — com a arvore ja' vermelha, TODA
#       mutacao le-se como SANGRA e o placar sai perfeito e fabricado.
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

# ⭐⭐⭐⭐ **O 4.o CONTROLO DO ARNES: a corrida limpa tem de estar VERDE.**
# ⛔⛔ Ate' 2026-09-21 so' se contava que ela CORREU testes (`> 0`), e o `|`
# deitava fora o codigo de saida. Com a arvore vermelha ANTES de mutar, TODA
# mutacao le-se como SANGRA e o placar sai perfeito e fabricado — e o erro e'
# para o lado que nao se nota, porque um placar cheio nao faz ninguem olhar.
limpa=$(corrida); rc_limpo=$?
verde=$(printf '%s' "$limpa" | populacao)
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
if [ "$rc_limpo" -ne 0 ]; then
  echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
  printf '%s' "$limpa" | grep -E '^ *(FAIL|test result:|Summary)' | tail -8 | sed 's/^/      | /'
  exit 2
fi

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
  '            && crate::tinta_da_peca::o_gesto_muda_a_topologia(verbo, self.tinta_fina_armada)' \
  '            && true' \
  'M10 recusa: a lente passa a ser o interruptor e nao a PORTA'

muta "$APP/recusa.rs" \
  '            tinta_fina_armada: self.tinta_fina_armada(),' \
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

muta "$PAN/state_modes.rs" \
  '            Self::Dezasseis => Some(4),' \
  '            Self::Dezasseis => Some(3),' \
  'M14b DetalheDaTinta: o degrau NOVO colapsa no anterior'

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


# ── ⭐⭐⭐⭐ A CURA DE 2026-09-21: o plano sobrevive a um traco de COR ──────
muta "$APP/tinta_da_peca.rs" \
  '    if verbo.paints_color() && tinta_fina_armada {
        return false;
    }' \
  '' \
  'M17 porta: um pincel de COR volta a refinar com o plano armado'

muta "$APP/tinta_da_peca.rs" \
  '    verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo()
}' \
  '    false
}' \
  'M18 porta: NINGUEM mexe na topologia (o CONTROLO do verbo de forma)'

muta "$APP/tinta_da_peca.rs" \
  '        self.stroke.tinta_fina.is_some()
            || self' \
  '        false
            || self' \
  'M19 porta: a tinta EMPRESTADA deixa de contar — e e a que conta no gesto'

muta "$APP/dyntopo.rs" \
  '        if !self.o_gesto_em_maos_muda_a_topologia(verbo) {' \
  '        if !verbo.refina_no_dyntopo() && !verbo.colapsa_no_dyntopo() {' \
  'M20 refine_for_dab: volta a perguntar so ao verbo'

muta "$APP/dyntopo.rs" \
  '        if self.tinta_fina_armada() {
            return (true, 0);
        }' \
  '' \
  'M21 toggle: ligar o interruptor volta a triangular com o plano armado'

muta "$APP/history_dyntopo.rs" \
  '        if !self.o_gesto_em_maos_muda_a_topologia(self.brush.verb) {
            self.dyn_before = None;
            return;
        }' \
  '' \
  'M22 pen-down: a triangulacao volta a correr para quem nao mexe na topologia'

muta "$APP/scenes_tinta_fina.rs" \
  'pub(crate) const DEGRAU_DA_LICAO: ph2d_panel_sculpt3d::state::DetalheDaTinta =
    ph2d_panel_sculpt3d::state::DetalheDaTinta::Oito;' \
  'pub(crate) const DEGRAU_DA_LICAO: ph2d_panel_sculpt3d::state::DetalheDaTinta =
    ph2d_panel_sculpt3d::state::DetalheDaTinta::Quatro;' \
  'M23 cena: a const da licao separa-se do roteiro'

muta "$APP/tinta_da_peca.rs" \
  'pub(crate) const NIVEL_MAX: u8 = 4;' \
  'pub(crate) const NIVEL_MAX: u8 = 3;' \
  'M24 tecto: o motor deixa de alocar o degrau que o painel oferece'

# ── ⭐⭐⭐⭐ O GESTO QUE ERRA A PECA (o que sobrou do report de 21/09) ─────
# ⚠️ A prova de comportamento dela e' `#[ignore]` + placa, logo o ELO tem de
# estar no censo de texto — senao esta mutacao SOBREVIVE, como as M19-M22.
muta "$APP/input_down.rs" \
  '                scene.close_stroke();
                scene.drag = Some(Drag::Orbit);' \
  '                scene.drag = Some(Drag::Orbit);' \
  'M25 o gesto que erra a peca volta a morrer com o plano dentro'

# ---- ⭐ A ORDEM DO DONO DE 21/09: «permita pintar mesmo se [nao] tocar um vertex»
# ⚠️ Mesmo caso da M25: a prova de comportamento
# (`um_traco_de_cor_que_comeca_fora_da_peca_pinta`) e' `#[ignore]` + placa, logo
# quem a mata aqui e' o ELO no censo de texto. Sem ele esta mutacao SOBREVIVE.
muta "$APP/input_down.rs" \
  '            if took || scene.brush.verb.paints_color() {' \
  '            if took {' \
  'M26 um traco de cor que comeca fora da peca volta a virar orbita'

echo
echo "MUTACAO: $sangram de $total sangram"
