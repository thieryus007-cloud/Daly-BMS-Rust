// ── ANIMATED FLOW EDGE (WAAPI, chemin smooth-step) ────────────────────────────
function AnimatedFlowEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, style, data }) {
  const color      = style?.stroke ?? '#2563eb';
  const pathResult = useMemo(
    () => getSmoothStepPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, borderRadius: 5 }),
    [sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition]
  );
  const edgePath = pathResult[0];
  const labelX   = pathResult[1];
  const labelY   = pathResult[2];
  const isH      = Math.abs(targetX - sourceX) >= Math.abs(targetY - sourceY);
  const ox = isH ? 0 : 3;
  const oy = isH ? 3 : 0;

  const flowValue  = data?.flowValue    ?? 0;
  const flowSpeed  = data?.flowSpeed    ?? 1;
  const flowColor  = data?.flowColor    ?? color;
  const reverse    = data?.reverse      ?? false;
  const pCount     = Math.max(1, Math.min(5, data?.particleCount ?? 2));
  const isActive   = flowValue > 0;

  const particleRefs = useRef([]);
  const animsRef     = useRef([]);

  useEffect(function () {
    animsRef.current.forEach(function (a) { try { if (a) a.cancel(); } catch (_) {} });
    animsRef.current = [];
    if (!isActive) return;
    const duration  = Math.max(500, Math.min(5000, 2000 / Math.max(0.4, flowSpeed)));
    const keyframes = reverse
      ? [{ offsetDistance: '100%' }, { offsetDistance: '0%'   }]
      : [{ offsetDistance: '0%'   }, { offsetDistance: '100%' }];
    const newAnims = [];
    for (let i = 0; i < pCount; i++) {
      const el = particleRefs.current[i];
      if (!el) continue;
      try {
        el.style.offsetPath = "path('" + edgePath + "')";
        newAnims.push(el.animate(keyframes, {
          duration, iterations: Infinity, easing: 'linear',
          delay: -(duration / pCount) * i,
        }));
      } catch (err) { console.warn('[AnimatedFlowEdge] WAAPI:', err); }
    }
    animsRef.current = newAnims;
    return function () { newAnims.forEach(function (a) { try { if (a) a.cancel(); } catch (_) {} }); };
  }, [edgePath, isActive, flowSpeed, reverse, pCount]);

  const lineOpacity = isActive ? 0.55 : 0.25;
  const lineStyle   = { fill: 'none', stroke: color, strokeWidth: 2, strokeOpacity: lineOpacity, strokeLinecap: 'round' };
  const children    = [
    h('path', { key: 'l1', d: edgePath, style: lineStyle, transform: 'translate(' + ox    + ',' + oy    + ')' }),
    h('path', { key: 'l2', d: edgePath, style: lineStyle, transform: 'translate(' + (-ox) + ',' + (-oy) + ')' }),
  ];
  if (isActive) {
    for (let i = 0; i < pCount; i++) {
      children.push(h('circle', {
        key: 'p' + i, r: 4, cx: 0, cy: 0, fill: flowColor,
        ref: (function (idx) { return function (el) { particleRefs.current[idx] = el; }; })(i),
        style: {
          offsetPath: "path('" + edgePath + "')", offsetDistance: reverse ? '100%' : '0%',
          offsetRotate: '0deg', willChange: 'offset-distance',
          filter: 'drop-shadow(0 0 3px ' + flowColor + ')',
        },
      }));
    }
  }
  return h(React.Fragment, null,
    h('g', null, ...children),
    data?.history ? h(EdgeHistoryButton, { labelX, labelY, edgeId: id, color }) : null,
  );
}

// ── CUSTOM BEZIER EDGE ────────────────────────────────────────────────────────
function CustomBezierEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, style, data }) {
  const color      = style?.stroke ?? '#2563eb';
  const pathResult = useMemo(
    () => getBezierPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition }),
    [sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition]
  );
  const edgePath = pathResult[0];
  const labelX   = pathResult[1];
  const labelY   = pathResult[2];
  const isH      = Math.abs(targetX - sourceX) >= Math.abs(targetY - sourceY);
  const ox = isH ? 0 : 3;
  const oy = isH ? 3 : 0;

  const flowValue  = data?.flowValue    ?? 0;
  const flowSpeed  = data?.flowSpeed    ?? 1;
  const flowColor  = data?.flowColor    ?? color;
  const reverse    = data?.reverse      ?? false;
  const pCount     = Math.max(1, Math.min(5, data?.particleCount ?? 2));
  const isActive   = flowValue > 0;

  const particleRefs = useRef([]);
  const animsRef     = useRef([]);

  useEffect(function () {
    animsRef.current.forEach(function (a) { try { if (a) a.cancel(); } catch (_) {} });
    animsRef.current = [];
    if (!isActive) return;
    const duration  = Math.max(500, Math.min(5000, 2000 / Math.max(0.4, flowSpeed)));
    const keyframes = reverse
      ? [{ offsetDistance: '100%' }, { offsetDistance: '0%' }]
      : [{ offsetDistance: '0%'   }, { offsetDistance: '100%' }];
    const newAnims = [];
    for (let i = 0; i < pCount; i++) {
      const el = particleRefs.current[i];
      if (!el) continue;
      try {
        el.style.offsetPath = "path('" + edgePath + "')";
        newAnims.push(el.animate(keyframes, {
          duration, iterations: Infinity, easing: 'linear',
          delay: -(duration / pCount) * i,
        }));
      } catch (err) { console.warn('[CustomBezierEdge] WAAPI:', err); }
    }
    animsRef.current = newAnims;
    return function () { newAnims.forEach(function (a) { try { if (a) a.cancel(); } catch (_) {} }); };
  }, [edgePath, isActive, flowSpeed, reverse, pCount]);

  const lineOpacity = isActive ? 0.55 : 0.25;
  const lineStyle   = { fill: 'none', stroke: color, strokeWidth: 2, strokeOpacity: lineOpacity, strokeLinecap: 'round' };
  const children    = [
    h('path', { key: 'l1', d: edgePath, style: lineStyle, transform: 'translate(' + ox    + ',' + oy    + ')' }),
    h('path', { key: 'l2', d: edgePath, style: lineStyle, transform: 'translate(' + (-ox) + ',' + (-oy) + ')' }),
  ];
  if (isActive) {
    for (let i = 0; i < pCount; i++) {
      children.push(h('circle', {
        key: 'p' + i, r: 4, cx: 0, cy: 0, fill: flowColor,
        ref: (function (idx) { return function (el) { particleRefs.current[idx] = el; }; })(i),
        style: {
          offsetPath: "path('" + edgePath + "')", offsetDistance: reverse ? '100%' : '0%',
          offsetRotate: '0deg', willChange: 'offset-distance',
          filter: 'drop-shadow(0 0 3px ' + flowColor + ')',
        },
      }));
    }
  }
  return h(React.Fragment, null,
    h('g', null, ...children),
    data?.history ? h(EdgeHistoryButton, { labelX, labelY, edgeId: id, color }) : null,
  );
}

// ── TRIPLE ATS EDGE ───────────────────────────────────────────────────────────
function TripleAtsEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, style, data }) {
  const color  = style?.stroke ?? '#10b981';
  const GAP    = 5;
  const useBez = data?.useBezier ?? false;

  const pathResult = useMemo(function () {
    return useBez
      ? getBezierPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition })
      : getSmoothStepPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, borderRadius: 5 });
  }, [sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, useBez]);
  const edgePath = pathResult[0];
  const labelX   = pathResult[1];
  const labelY   = pathResult[2];

  const isH = Math.abs(targetX - sourceX) >= Math.abs(targetY - sourceY);
  const ox  = isH ? 0 : GAP;
  const oy  = isH ? GAP : 0;

  const flowValue = data?.flowValue ?? 0;
  const flowSpeed = data?.flowSpeed ?? 1;
  const flowColor = data?.flowColor ?? color;
  const reverse   = data?.reverse   ?? false;
  const isActive  = flowValue > 0;

  const particleRef = useRef(null);
  const animRef     = useRef(null);

  useEffect(function () {
    try { if (animRef.current) animRef.current.cancel(); } catch (_) {}
    animRef.current = null;
    if (!isActive) return;
    const duration  = Math.max(500, Math.min(5000, 2000 / Math.max(0.4, flowSpeed)));
    const keyframes = reverse
      ? [{ offsetDistance: '100%' }, { offsetDistance: '0%' }]
      : [{ offsetDistance: '0%'   }, { offsetDistance: '100%' }];
    try {
      if (particleRef.current) {
        particleRef.current.style.offsetPath = "path('" + edgePath + "')";
        animRef.current = particleRef.current.animate(keyframes, { duration, iterations: Infinity, easing: 'linear' });
      }
    } catch (err) { console.warn('[TripleAtsEdge] WAAPI:', err); }
    return function () { try { if (animRef.current) animRef.current.cancel(); } catch (_) {} };
  }, [edgePath, isActive, flowSpeed, reverse]);

  const lineOpacity = isActive ? 0.7 : 0.45;
  const dashStyle   = { fill: 'none', stroke: color, strokeWidth: 2, strokeOpacity: lineOpacity, strokeDasharray: '6 6', strokeLinecap: 'round' };
  const children = [
    h('path', { key: 'l1', d: edgePath, style: dashStyle, transform: 'translate(' + ox    + ',' + oy    + ')' }),
    h('path', { key: 'l2', d: edgePath, style: dashStyle }),
    h('path', { key: 'l3', d: edgePath, style: dashStyle, transform: 'translate(' + (-ox) + ',' + (-oy) + ')' }),
  ];
  if (isActive) {
    children.push(h('circle', {
      key: 'dot', r: 4, cx: 0, cy: 0, fill: flowColor,
      ref: function (el) { particleRef.current = el; },
      style: {
        offsetPath: "path('" + edgePath + "')", offsetDistance: reverse ? '100%' : '0%',
        offsetRotate: '0deg', willChange: 'offset-distance',
        filter: 'drop-shadow(0 0 3px ' + flowColor + ')',
      },
    }));
  }
  return h(React.Fragment, null,
    h('g', null, ...children),
    data?.history ? h(EdgeHistoryButton, { labelX, labelY, edgeId: id, color }) : null,
  );
}
