// ── SPIRAL RACE NODE (remplace les 3 SummaryNodes production/batteries/météo) ─
// 3 jauges concentriques qui s'ouvrent de 07:30 à l'heure actuelle :
//   anneau extérieur = Production journalière (kWh, max 18)
//   anneau milieu    = SOC batteries (%, max 100)
//   anneau intérieur = Irradiance instantanée (W/m², max 900)
const SpiralRaceNode = function SpiralRaceNode({ data }) {
  const chartRef = useRef(null);
  const instRef  = useRef(null);

  const prodKwh = data.prodKwh ?? 0;
  const soc     = data.soc     ?? 0;
  const irrWm2  = data.irrWm2  ?? 0;

  useEffect(function () {
    if (!chartRef.current || !window.echarts) return;
    if (!instRef.current) {
      instRef.current = window.echarts.init(chartRef.current);
    }
    const chart = instRef.current;

    // Progression de la journée : 07:30 → 19:30 (720 min)
    const now    = new Date();
    const nowMin = now.getHours() * 60 + now.getMinutes();
    const s0     = 7 * 60 + 30;   // 450
    const s1     = 19 * 60 + 30;  // 1170
    const prog   = Math.min(1, Math.max(0, (nowMin - s0) / (s1 - s0)));

    // Dans ECharts gauge : startAngle=90 = midi (haut), clockwise=true
    // endAngle = 90 − 360*prog  → l'arc s'ouvre au fil de la journée
    const gStart = 90;
    const gEnd   = 90 - 360 * prog;

    const socCol = soc > 60 ? '#22c55e' : soc > 30 ? '#f59e0b' : '#ef4444';

    const mkGauge = (radius, max, value, color) => ({
      type: 'gauge',
      radius,
      startAngle: gStart,
      endAngle:   gEnd,
      min: 0, max,
      clockwise: true,
      pointer:  { show: false },
      progress: {
        show: true, width: 11, roundCap: true, clip: false,
        itemStyle: { color }
      },
      axisLine: {
        show: true,
        lineStyle: { width: 11, color: [[1, 'rgba(148,163,184,0.10)']] }
      },
      axisTick: { show: false }, splitLine: { show: false },
      axisLabel: { show: false }, detail: { show: false }, title: { show: false },
      data: [{ value: Math.max(0, value) }]
    });

    chart.setOption({
      animation: true,
      animationDuration: 1400,
      animationEasing:   'cubicOut',
      backgroundColor:   'transparent',
      series: [
        mkGauge('88%',  18,  prodKwh, '#fbbf24'),  // Production (ext.)
        mkGauge('65%', 100,  soc,     socCol),      // SOC (milieu)
        mkGauge('42%', 900,  irrWm2,  '#38bdf8'),   // Irradiance (int.)
      ]
    });
  }, [prodKwh, soc, irrWm2]);

  useEffect(function () {
    return function () {
      if (instRef.current) {
        try { instRef.current.dispose(); } catch (_) {}
        instRef.current = null;
      }
    };
  }, []);

  const socCol = soc > 60 ? '#22c55e' : soc > 30 ? '#f59e0b' : '#ef4444';

  return h('div', { className: 'spiral-node' },
    mkHandle('target', Position.Top,    'tt'),
    mkHandle('source', Position.Bottom, 'sb'),
    mkHandle('source', Position.Right,  'sr'),
    mkHandle('target', Position.Left,   'tl'),
    mkHandle('source', Position.Left,   'sl', { top: '68%' }),
    h('div', { className: 'spiral-hdr' },
      h('span', { className: 'spiral-title' }, '☀ Production · SOC · Météo')
    ),
    h('div', { ref: chartRef, className: 'spiral-chart' }),
    h('div', { className: 'spiral-legend' },
      h('div', { className: 'spiral-leg-item' },
        h('div', { className: 'spiral-leg-dot', style: { background: '#fbbf24' } }),
        h('span', { className: 'spiral-leg-lbl' }, 'Production'),
        h('span', { className: 'spiral-leg-val' }, prodKwh != null ? prodKwh.toFixed(1) + ' kWh' : '—')
      ),
      h('div', { className: 'spiral-leg-item' },
        h('div', { className: 'spiral-leg-dot', style: { background: socCol } }),
        h('span', { className: 'spiral-leg-lbl' }, 'SOC Batteries'),
        h('span', { className: 'spiral-leg-val' }, soc != null ? soc.toFixed(0) + '%' : '—')
      ),
      h('div', { className: 'spiral-leg-item' },
        h('div', { className: 'spiral-leg-dot', style: { background: '#38bdf8' } }),
        h('span', { className: 'spiral-leg-lbl' }, 'Irradiance'),
        h('span', { className: 'spiral-leg-val' }, irrWm2 != null ? irrWm2.toFixed(0) + ' W/m²' : '—')
      )
    )
  );
};

// ── SANKEY NODE (flux énergétique sources → consommation) ─────────────────────
// Sources gauche : Micro-onduleurs, MPPT
// Puits droite   : Batteries (charge), Maison (Tongou), Export (si positif)
const SankeyNode = function SankeyNode({ data }) {
  const chartRef = useRef(null);
  const instRef  = useRef(null);

  const microW  = Math.max(0, data.microPwr  ?? 0);
  const mpptW   = Math.max(0, data.mpptPwr   ?? 0);
  const batW    = Math.max(0, data.batPwr    ?? 0);  // charge batterie
  const loadW   = Math.max(0, data.loadPwr   ?? 0);  // charges Tongou
  const totalIn = microW + mpptW;

  useEffect(function () {
    if (!chartRef.current || !window.echarts) return;
    if (!instRef.current) {
      instRef.current = window.echarts.init(chartRef.current);
    }
    const chart = instRef.current;

    if (totalIn < 10) {
      chart.clear();
      chart.setOption({
        backgroundColor: 'transparent',
        graphic: [{ type: 'text', left: 'center', top: 'middle',
          style: { text: 'Pas de production', fill: '#64748b', fontSize: 10 } }]
      });
      return;
    }

    // Répartition des sorties (capée à totalIn pour équilibrer le Sankey)
    const batOut  = Math.min(batW,  totalIn);
    const loadOut = Math.min(loadW, Math.max(0, totalIn - batOut));
    const expOut  = Math.max(0, totalIn - batOut - loadOut);

    // Construction des nœuds uniquement pour les flux significatifs (> 5 W)
    const srcNodes  = [];
    const sinkNodes = [];
    if (microW > 5) srcNodes.push({ name: 'Micro-ond', w: microW, color: '#3b82f6' });
    if (mpptW  > 5) srcNodes.push({ name: 'MPPT',      w: mpptW,  color: '#f59e0b' });
    if (batOut > 5) sinkNodes.push({ name: 'Batteries', w: batOut,  color: '#22c55e' });
    if (loadOut> 5) sinkNodes.push({ name: 'Maison',    w: loadOut, color: '#a855f7' });
    if (expOut > 5) sinkNodes.push({ name: 'Export',    w: expOut,  color: '#0ea5e9' });

    if (sinkNodes.length === 0) {
      sinkNodes.push({ name: 'Consomm.', w: totalIn, color: '#64748b' });
    }

    const sinkTotal = sinkNodes.reduce((s, n) => s + n.w, 0);

    const eNodes = [
      ...srcNodes.map(n  => ({ name: n.name,  itemStyle: { color: n.color  } })),
      ...sinkNodes.map(n => ({ name: n.name,  itemStyle: { color: n.color  } })),
    ];
    const eLinks = [];
    srcNodes.forEach(function (src) {
      sinkNodes.forEach(function (sink) {
        const v = Math.round(src.w * (sink.w / sinkTotal));
        if (v > 1) eLinks.push({ source: src.name, target: sink.name, value: v });
      });
    });

    chart.setOption({
      animation: true,
      animationDuration: 1000,
      backgroundColor: 'transparent',
      series: [{
        type: 'sankey',
        left: '2%', right: '2%', top: '8%', bottom: '8%',
        orient: 'horizontal',
        nodeAlign: 'justify',
        nodeWidth: 10,
        nodeGap: 12,
        emphasis: { focus: 'adjacency' },
        label: {
          show: true, fontSize: 9, color: '#94a3b8',
          formatter: function (p) {
            return p.name + '\n' + Math.round(p.value) + ' W';
          }
        },
        lineStyle: { color: 'gradient', opacity: 0.45, curveness: 0.5 },
        data: eNodes,
        links: eLinks
      }]
    });
  }, [microW, mpptW, batW, loadW, totalIn]);

  useEffect(function () {
    return function () {
      if (instRef.current) {
        try { instRef.current.dispose(); } catch (_) {}
        instRef.current = null;
      }
    };
  }, []);

  return h('div', { className: 'sankey-node' },
    mkHandle('target', Position.Top,    'tt'),
    mkHandle('source', Position.Bottom, 'sb'),
    mkHandle('target', Position.Left,   'tl'),
    mkHandle('source', Position.Right,  'sr'),
    h('div', { className: 'sankey-hdr' },
      h('span', { className: 'sankey-icon' }, '⚡'),
      h('span', { className: 'sankey-title' }, 'Flux Énergétique'),
      h('span', { className: 'sankey-total' }, totalIn > 0 ? Math.round(totalIn) + ' W' : '—')
    ),
    h('div', { ref: chartRef, className: 'sankey-chart' })
  );
};
