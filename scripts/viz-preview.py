#!/usr/bin/env python3
"""Aperçu local de la page Visualisation, sans compiler ni déployer daly-bms-server.

Principe : la page servie par le Pi5 (base.html + blocs askama déjà rendus) est récupérée,
puis sa feuille de style et son script principal sont remplacés par ceux des templates
locaux (`visualization.html` avec ses `{% include %}` assemblés). Un petit script redirige
les lectures `/api/...` vers le Pi5 : l'aperçu affiche les données réelles.

Sécurité : seules les requêtes GET partent vers le Pi5. Toute écriture (POST : boutons
ATS, prises Tasmota, Shelly) est bloquée dans l'aperçu.

Usage :
    python scripts/viz-preview.py build [--pi5 http://192.168.1.141:8080] [--out DOSSIER]
        → DOSSIER/preview.html          (aperçu)
          DOSSIER/preview-measure.html  (aperçu + mesure des cartes, pour --dump-dom)
    python scripts/viz-preview.py build --open bms1 [--period 1440]
        → en plus, DOSSIER/preview-open.html : la fiche détail de la box bms1 s'ouvre seule
          (cible = id d'une box de NODES0, ou edge:N pour le N-ième bouton 📈, compté à partir
          de 0 ; --period = 60, 360 ou 1440 minutes)
    python scripts/viz-preview.py sizes DOSSIER/dom.html
        → tailles des cartes en px du canevas (largeur, hauteur), au format JSON

Captures et mesures (PowerShell, Edge headless) : docs/TRANSITION-refonte-visualisation.md §1.7.
"""
import argparse
import os
import re
import sys
import tempfile
import urllib.request

TEMPLATES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "daly-bms-server", "templates")

# Repères présents dans visualization.html : le <style> qui suit la feuille React Flow et le
# <script> qui suit ECharts sont ceux de la page.
STYLE_ANCHOR = 'reactflow@11/dist/style.css">'
SCRIPT_ANCHOR = 'echarts.min.js"></script>'

SHIM = (
    "<script>(function(){var f=window.fetch,P=%r;window.fetch=function(u,o){"
    "var m=((o&&o.method)||'GET').toUpperCase();"
    "if(typeof u==='string'&&u.charAt(0)==='/'){"
    "if(m!=='GET'){console.warn('[aperçu] écriture bloquée : '+m+' '+u);"
    "return Promise.reject(new Error('aperçu : écriture bloquée'));}u=P+u;}"
    "return f(u,o);};})();</script>"
)

MEASURE = """<script>setTimeout(function(){
  var vp = document.querySelector('.react-flow__viewport');
  var m = /scale\\(([\\d.]+)\\)/.exec(vp ? vp.style.transform : ''); var z = m ? parseFloat(m[1]) : 1;
  var out = { zoom: z };
  document.querySelectorAll('.react-flow__node').forEach(function(n){ var r = n.getBoundingClientRect();
    out[n.getAttribute('data-id')] = [Math.round(r.width / z), Math.round(r.height / z)]; });
  var pre = document.createElement('pre'); pre.id = 'sizes'; pre.textContent = JSON.stringify(out);
  document.body.appendChild(pre);
}, 9000);</script>"""

# Ouvre la fiche détail d'une box (clic sur son nœud) ou d'une liaison (clic sur son 📈).
OPEN = """<script>setTimeout(function(){
  var el = %s;
  if (!el) { document.title = 'CIBLE INTROUVABLE'; return; }
  el.click();
  var p = %s;
  if (p) setTimeout(function(){ document.querySelectorAll('.vd-period').forEach(function(b){
    if (b.textContent.replace(/\\s/g, '') === p) b.click(); }); }, 1500);
}, 7000);</script>"""


def assemble_template():
    """visualization.html avec ses {% include "..." %} remplacés par le contenu des fichiers."""
    src = open(os.path.join(TEMPLATES, "visualization.html"), encoding="utf-8").read()

    def inc(m):
        return open(os.path.join(TEMPLATES, m.group(1)), encoding="utf-8").read()

    return re.sub(r'\{% include "([^"]+)" %\}', inc, src)


def region(text, anchor, open_tag, close_tag):
    """(début, fin) du contenu du premier open_tag…close_tag qui suit anchor."""
    a = text.index(anchor)
    start = text.index(open_tag, a) + len(open_tag)
    return start, text.index(close_tag, start)


def splice(live, new):
    for anchor, o, c in ((STYLE_ANCHOR, "<style>", "</style>"), (SCRIPT_ANCHOR, "<script>", "</script>")):
        ns, ne = region(new, anchor, o, c)
        ls, le = region(live, anchor, o, c)
        live = live[:ls] + new[ns:ne] + live[le:]
    return live


def build(args):
    os.makedirs(args.out, exist_ok=True)
    with urllib.request.urlopen(args.pi5.rstrip("/") + "/visualization", timeout=15) as r:
        live = r.read().decode("utf-8")
    page = splice(live, assemble_template())
    page = page.replace("<head>", "<head>" + SHIM % args.pi5.rstrip("/"), 1)
    with open(os.path.join(args.out, "preview.html"), "w", encoding="utf-8") as f:
        f.write(page)
    with open(os.path.join(args.out, "preview-measure.html"), "w", encoding="utf-8") as f:
        f.write(page.replace("</body>", MEASURE + "</body>", 1))
    print("Aperçu :", os.path.join(args.out, "preview.html"))
    print("Mesure :", os.path.join(args.out, "preview-measure.html"))
    if args.open:
        if args.open.startswith("edge:"):
            sel = "document.querySelectorAll('.vd-edge-btn')[%d]" % int(args.open[5:])
        else:
            sel = "document.querySelector('.react-flow__node[data-id=\"%s\"]')" % args.open
        period = "'%dh'" % (args.period // 60) if args.period else "null"
        with open(os.path.join(args.out, "preview-open.html"), "w", encoding="utf-8") as f:
            f.write(page.replace("</body>", OPEN % (sel, period) + "</body>", 1))
        print("Fiche  :", os.path.join(args.out, "preview-open.html"))


def sizes(args):
    dom = open(args.dom, encoding="utf-8", errors="replace").read()
    m = re.search(r'<pre id="sizes">([^<]*)</pre>', dom)
    if not m:
        sys.exit("Tailles introuvables : la page n'a pas fini de s'afficher (augmenter --virtual-time-budget).")
    print(m.group(1))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build", help="construit preview.html et preview-measure.html")
    b.add_argument("--pi5", default="http://192.168.1.141:8080")
    b.add_argument("--out", default=os.path.join(tempfile.gettempdir(), "viz-preview"))
    b.add_argument("--open", help="id d'une box (ex. bms1) ou edge:N : fiche détail ouverte d'office")
    b.add_argument("--period", type=int, choices=(60, 360, 1440), help="période de la fiche, en minutes")
    s = sub.add_parser("sizes", help="extrait les tailles des cartes d'un DOM sauvegardé")
    s.add_argument("dom")
    args = p.parse_args()
    build(args) if args.cmd == "build" else sizes(args)


if __name__ == "__main__":
    main()
