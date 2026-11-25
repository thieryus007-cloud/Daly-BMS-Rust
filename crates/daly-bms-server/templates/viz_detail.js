// ── FICHE DÉTAIL ──────────────────────────────────────────────────────────────
// Clic sur une box (ou sur le 📈 d'une liaison) : fiche centrée dans la fenêtre, hors de la
// carte, donc de taille indépendante du zoom. À gauche la box elle-même, en direct et
// agrandie (ses boutons restent actifs) ; à droite ses historiques, lus dans le
// metrics-store par /api/v1/query_range (PromQL, comme Grafana).
// Fermeture : ✕, Échap, clic / molette / glisser sur le fond.
// Les graphiques de chaque box et liaison sont décrits dans visualization.html (NODE_DETAIL).

const DetailCtx = React.createContext(null);

const DETAIL_PERIODS = [{ min: 60, label: '1 h' }, { min: 360, label: '6 h' }, { min: 1440, label: '24 h' }];
const detailStepS = min => (min <= 60 ? 20 : min <= 360 ? 60 : 180);   // 180 à 480 points par courbe

// Une courbe : couleur de la box (AC / DC / batterie). Plusieurs : ordre catégoriel fixe,
// jamais recyclé, avec sa variante pour le thème sombre.
const VD_SERIES_LIGHT = ['#2a78d6', '#eb6834', '#1baf7a', '#eda100', '#e87ba4', '#008300', '#6250d6', '#e34948'];
const VD_SERIES_DARK  = ['#3987e5', '#d95926', '#199e70', '#c98500', '#d55181', '#0ca30c', '#9085e9', '#e66767'];

const VD_DEC = { W: 0, A: 1, V: 1, '%': 1, '°C': 1, 'W/m²': 0 };
const vdFmt = (v, unit, dec) => (v == null || !isFinite(v)) ? '—'
  : v.toLocaleString('fr-FR', { minimumFractionDigits: dec ?? VD_DEC[unit] ?? 1, maximumFractionDigits: dec ?? VD_DEC[unit] ?? 1 });
// Énergie sur la période : intégrale de la courbe (W → kWh, A → Ah).
const VD_ENERGY = { W: { unit: 'kWh', k: 1 / 3600000, dec: 2 }, A: { unit: 'Ah', k: 1 / 3600, dec: 1 } };

async function vdFetchRange(query, minutes) {
  const end = Math.floor(Date.now() / 1000), start = end - minutes * 60;
  const url = '/api/v1/query_range?query=' + encodeURIComponent(query)
    + '&start=' + start + '&end=' + end + '&step=' + detailStepS(minutes);
  const r = await fetch(url, { cache: 'no-cache' });
  if (!r.ok) throw new Error('HTTP ' + r.status);
  const j = await r.json();
  if (j.status !== 'success') throw new Error(j.error || 'query_range');
  // Chaque requête vise une seule série (étiquettes précises) : la première suffit.
  const values = j.data?.result?.[0]?.values ?? [];
  return values.map(e => [e[0] * 1000, parseFloat(e[1])]).filter(p => isFinite(p[1]));
}

// Min, moyenne, max, dernier point ; énergie par la méthode des trapèzes, séparée par
// signe. Un écart de plus de 3 pas entre deux points (trou d'historique) n'est pas compté.
function vdStats(points, stepS) {
  if (!points || !points.length) return null;
  let min = Infinity, max = -Infinity, sum = 0, pos = 0, neg = 0;
  points.forEach(p => { if (p[1] < min) min = p[1]; if (p[1] > max) max = p[1]; sum += p[1]; });
  for (let i = 1; i < points.length; i++) {
    const dt = (points[i][0] - points[i - 1][0]) / 1000;
    if (dt <= 0 || dt > stepS * 3) continue;
    const a = points[i - 1][1], b = points[i][1];
    pos += (Math.max(a, 0) + Math.max(b, 0)) / 2 * dt;
    neg += (Math.min(a, 0) + Math.min(b, 0)) / 2 * dt;
  }
  return { min, max, mean: sum / points.length, last: points[points.length - 1], pos, neg };
}

function vdEnergyText(st, unit, sign) {
  const en = VD_ENERGY[unit];
  if (!st || !en) return '';
  const p = st.pos * en.k, n = -st.neg * en.k;
  const f = v => vdFmt(v, null, en.dec) + ' ' + en.unit;
  if (n > 0 && p > 0) return '+' + f(p) + (sign ? ' ' + sign.pos : '') + ' · −' + f(n) + (sign ? ' ' + sign.neg : '');
  if (n > 0) return '−' + f(n) + (sign ? ' ' + sign.neg : '');
  return f(p) + (sign && p > 0 ? ' ' + sign.pos : '');
}

// ── Un graphique d'historique (une mesure, une ou plusieurs courbes) ───────────
function HistChart({ chart, minutes, tick, color }) {
  const divRef = useRef(null);
  const instRef = useRef(null);
  const [res, setRes] = useState({ loading: true, data: null, failed: false });
  const [hidden, setHidden] = useState({});        // courbes masquées par un clic sur la légende
  const key = JSON.stringify(chart.series.map(s => s.q));
  const unit = chart.unit;
  const stepS = detailStepS(minutes);

  useEffect(function () {
    let cancelled = false;
    setRes(r => ({ ...r, loading: true }));
    Promise.all(chart.series.map(s => vdFetchRange(s.q, minutes).catch(() => null)))
      .then(arr => { if (!cancelled) setRes({ loading: false, data: arr, failed: arr.every(a => a === null) }); });
    return function () { cancelled = true; };
  }, [key, minutes, tick]);

  const stats = useMemo(() => (res.data || []).map(pts => vdStats(pts, stepS)), [res.data, stepS]);
  const empty = !res.loading && (!res.data || res.data.every(a => !a || a.length === 0));

  useEffect(function () {
    if (!divRef.current || !window.echarts || !res.data) return;
    if (!instRef.current) instRef.current = window.echarts.init(divRef.current);
    const dark  = document.documentElement.classList.contains('dark');
    const ink2  = dark ? '#8b949e' : '#64748b';
    const gridC = dark ? '#30363d' : '#e2e8f0';
    const axisC = dark ? '#484f58' : '#cbd5e1';
    const multi = chart.series.length > 1;
    const pal   = dark ? VD_SERIES_DARK : VD_SERIES_LIGHT;
    const end = Date.now(), start = end - minutes * 60000;
    const zeroLine = { silent: true, symbol: 'none', label: { show: false },
      lineStyle: { color: axisC, type: 'dashed', width: 1 }, data: [{ yAxis: 0 }] };
    instRef.current.setOption({
      animation: false,
      grid: { top: 12, right: 18, bottom: 28, left: 60 },
      tooltip: {
        trigger: 'axis', axisPointer: { type: 'line', lineStyle: { color: ink2 } },
        formatter: function (ps) {
          if (!ps || !ps.length) return '';
          const t = new Date(ps[0].value[0]).toLocaleString('fr-FR',
            { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
          return t + ps.map(p => '<br/>' + p.marker + chart.series[p.seriesIndex].name + '  <b>'
            + vdFmt(p.value[1], unit) + ' ' + unit + '</b>').join('');
        },
      },
      xAxis: {
        type: 'time', min: start, max: end,
        axisLabel: { color: ink2, fontSize: 12, hideOverlap: true,
          formatter: { day: '{dd}/{MM}', hour: '{HH}:{mm}', minute: '{HH}:{mm}', second: '{HH}:{mm}:{ss}' } },
        axisLine: { lineStyle: { color: axisC } }, splitLine: { show: false },
      },
      yAxis: {
        type: 'value', scale: !VD_ENERGY[unit],
        axisLabel: { color: ink2, fontSize: 12, formatter: v => vdFmt(v, unit, Number.isInteger(v) ? 0 : 1) },
        splitLine: { lineStyle: { color: gridC } },
      },
      series: chart.series.map((s, i) => {
        const c = multi ? pal[i % pal.length] : (color || pal[0]);
        return {
          name: s.name, type: 'line', showSymbol: false, symbolSize: 8, sampling: 'lttb',
          data: hidden[i] ? [] : (res.data[i] || []), lineStyle: { width: 2, color: c }, itemStyle: { color: c },
          areaStyle: multi ? undefined : { color: c, opacity: 0.12 },
          markLine: (!multi && chart.sign) ? zeroLine : undefined,
        };
      }),
    }, true);
  }, [res.data, minutes, color, hidden]);

  useEffect(function () {
    const el = divRef.current;
    const ro = window.ResizeObserver ? new ResizeObserver(() => { if (instRef.current) instRef.current.resize(); }) : null;
    if (ro && el) ro.observe(el);
    return function () {
      if (ro) ro.disconnect();
      if (instRef.current) { try { instRef.current.dispose(); } catch (_) {} instRef.current = null; }
    };
  }, []);

  // En-tête : titre ; pour une seule courbe, dernier relevé, min · moy · max et énergie.
  // Plusieurs courbes : légende HTML (passe à la ligne d'elle-même), avec l'énergie de la
  // période ; un clic masque / réaffiche la courbe.
  const multi = chart.series.length > 1;
  const st = multi ? null : stats[0];
  const en = st ? vdEnergyText(st, unit, chart.sign) : '';
  const pal = document.documentElement.classList.contains('dark') ? VD_SERIES_DARK : VD_SERIES_LIGHT;
  const legend = multi ? h('div', { className: 'vd-legend' }, chart.series.map((s, i) => {
    const e = vdEnergyText(stats[i], unit, chart.sign);
    return h('button', {
      key: i, type: 'button', className: 'vd-leg' + (hidden[i] ? ' off' : ''), 'aria-pressed': !hidden[i],
      title: hidden[i] ? 'Afficher la courbe' : 'Masquer la courbe',
      onClick: () => setHidden(x => ({ ...x, [i]: !x[i] })),
    }, h('span', { className: 'vd-leg-sw', style: { background: pal[i % pal.length] } }), s.name,
       e ? h('span', { className: 'vd-leg-en' }, ' · ' + e) : null);
  })) : null;
  return h('div', { className: 'vd-chart' },
    h('div', { className: 'vd-chart-head' },
      h('span', { className: 'vd-chart-title' }, chart.title, h('span', { className: 'vd-chart-unit' }, ' (' + unit + ')')),
      st ? h('span', { className: 'vd-chart-stats' },
        h('span', null, 'relevé de ' + new Date(st.last[0]).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' }) + ' : ',
          h('b', null, vdFmt(st.last[1], unit) + ' ' + unit)),
        h('span', null, 'min ' + vdFmt(st.min, unit) + ' · moy ' + vdFmt(st.mean, unit) + ' · max ' + vdFmt(st.max, unit)),
        en ? h('span', null, en) : null
      ) : null
    ),
    legend,
    h('div', { className: 'vd-chart-plot' },
      h('div', { ref: divRef, className: 'vd-chart-canvas' }),
      res.loading && !res.data ? h('div', { className: 'vd-chart-msg' }, 'Chargement…') : null,
      res.failed ? h('div', { className: 'vd-chart-msg' }, 'Historique indisponible') : null,
      (!res.failed && empty) ? h('div', { className: 'vd-chart-msg' }, 'Pas de données sur la période') : null
    )
  );
}

// ── La box, en direct et agrandie : un second React Flow avec ce seul nœud ──────
// (les composants des box ont besoin du contexte React Flow pour leurs poignées).
function DetailBox({ node }) {
  const instRef = useRef(null);
  const wrapRef = useRef(null);
  const fit = useCallback(function () {
    requestAnimationFrame(() => { if (instRef.current) instRef.current.fitView({ padding: 0.04, maxZoom: 4 }); });
  }, []);
  useEffect(function () {
    const ro = window.ResizeObserver ? new ResizeObserver(fit) : null;
    if (ro && wrapRef.current) ro.observe(wrapRef.current);
    return function () { if (ro) ro.disconnect(); };
  }, [fit]);
  const nodes = [{ id: node.id, type: node.type, data: node.data, position: { x: 0, y: 0 } }];
  return h('div', { ref: wrapRef, className: 'vd-flow' },
    h(ReactFlow, {
      id: 'vd-flow', nodes, edges: [], nodeTypes,
      fitView: true, fitViewOptions: { padding: 0.04, maxZoom: 4 }, minZoom: 0.2, maxZoom: 4,
      onInit: inst => { instRef.current = inst; fit(); },
      onNodesChange: changes => { if (changes.some(c => c.type === 'dimensions')) fit(); },
      panOnDrag: false, panOnScroll: false, zoomOnScroll: false, zoomOnPinch: false, zoomOnDoubleClick: false,
      nodesDraggable: false, nodesConnectable: false, elementsSelectable: false, preventScrolling: false,
      proOptions: { hideAttribution: true },
    })
  );
}

// ── La fiche ──────────────────────────────────────────────────────────────────
function DetailSheet({ spec, minutes, setMinutes, onClose }) {
  const [tick, setTick] = useState(0);
  useEffect(function () {
    const t = setInterval(() => setTick(x => x + 1), 60000);   // historiques relus chaque minute
    return function () { clearInterval(t); };
  }, []);
  useEffect(function () {
    const onKey = e => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return function () { window.removeEventListener('keydown', onKey); };
  }, [onClose]);
  // Fond : un clic, la molette ou un glisser (= se déplacer dans la page) ferme la fiche.
  const onBackdrop = e => { if (e.target === e.currentTarget) onClose(); };

  const hasBox = !!spec.node;
  return ReactDOM.createPortal(
    h('div', { className: 'vd-backdrop', onPointerDown: onBackdrop, onWheel: onBackdrop },
      h('div', { className: 'vd-sheet' + (hasBox ? '' : ' vd-nobox'), role: 'dialog', 'aria-modal': 'true', 'aria-label': spec.title },
        h('div', { className: 'vd-head' },
          h('span', { className: 'vd-title' }, spec.title),
          h('div', { className: 'vd-periods', role: 'group', 'aria-label': 'Période des historiques' },
            DETAIL_PERIODS.map(p => h('button', {
              key: p.min, type: 'button', className: 'vd-period' + (p.min === minutes ? ' on' : ''),
              'aria-pressed': p.min === minutes, onClick: () => setMinutes(p.min),
            }, p.label))
          ),
          h('button', { type: 'button', className: 'vd-close', onClick: onClose, 'aria-label': 'Fermer', title: 'Fermer (Échap)' }, '✕')
        ),
        h('div', { className: 'vd-body' },
          hasBox ? h('div', { className: 'vd-box' }, h(DetailBox, { node: spec.node })) : null,
          spec.charts.length
            ? h('div', { className: 'vd-charts' },
                spec.charts.map((c, i) => h(HistChart, { key: c.title + '|' + i, chart: c, minutes, tick, color: spec.color })))
            : null
        )
      )
    ),
    document.body
  );
}

// ── Bouton 📈 au milieu d'une liaison : ouvre la fiche de la liaison ───────────
function EdgeHistoryButton({ labelX, labelY, edgeId, color }) {
  const open = React.useContext(DetailCtx);
  if (!EdgeLabelRenderer || !open) return null;
  return h(EdgeLabelRenderer, null,
    h('button', {
      type: 'button', className: 'vd-edge-btn nodrag nopan',
      style: { transform: `translate(-50%,-50%) translate(${labelX}px,${labelY}px)`, '--vd-c': color },
      title: 'Historique de la liaison', 'aria-label': 'Historique de la liaison',
      onClick: e => { e.stopPropagation(); open('edge', edgeId); },
    }, '📈')
  );
}
