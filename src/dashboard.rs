pub const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Bench Server Dashboard</title>
  <style>
    :root {
      color-scheme: light;
      --bg: #f8fafc;
      --card: #ffffff;
      --text: #0f172a;
      --muted: #475569;
      --line: #e2e8f0;
      --accent: #0ea5e9;
      --accent-2: #22c55e;
      --error: #ef4444;
    }
    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
      background: radial-gradient(circle at 20% 0%, #e0f2fe 0%, var(--bg) 42%);
      color: var(--text);
    }
    .wrap {
      max-width: 1100px;
      margin: 0 auto;
      padding: 20px;
    }
    h1 {
      margin: 0 0 12px;
      font-size: 28px;
      letter-spacing: -0.02em;
    }
    .muted {
      color: var(--muted);
      font-size: 14px;
    }
    .grid {
      margin-top: 18px;
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 10px;
    }
    .card {
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
      box-shadow: 0 8px 20px rgba(2, 6, 23, 0.06);
    }
    .label {
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .value {
      margin-top: 4px;
      font-size: 24px;
      font-weight: 700;
    }
    .panel {
      margin-top: 14px;
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
      box-shadow: 0 8px 20px rgba(2, 6, 23, 0.06);
    }
    canvas {
      width: 100%;
      height: 260px;
      border-radius: 8px;
      border: 1px solid var(--line);
      background: linear-gradient(180deg, rgba(14, 165, 233, 0.05) 0%, rgba(255, 255, 255, 1) 60%);
    }
    .chart-wrap {
      position: relative;
      margin-top: 8px;
    }
    .chart-tooltip {
      position: absolute;
      left: 0;
      top: 0;
      transform: translate(-50%, calc(-100% - 10px));
      pointer-events: none;
      opacity: 0;
      transition: opacity 0.12s ease;
      background: #0f172a;
      color: #f8fafc;
      font-size: 12px;
      font-family: "IBM Plex Mono", "Fira Mono", monospace;
      padding: 6px 8px;
      border-radius: 8px;
      box-shadow: 0 8px 20px rgba(2, 6, 23, 0.35);
      white-space: nowrap;
      z-index: 2;
    }
    .chart-tooltip.visible {
      opacity: 1;
    }
    .chart-tooltip::after {
      content: "";
      position: absolute;
      left: 50%;
      top: 100%;
      margin-left: -5px;
      border-width: 5px;
      border-style: solid;
      border-color: #0f172a transparent transparent transparent;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin-top: 8px;
      font-size: 14px;
    }
    th, td {
      border-bottom: 1px solid var(--line);
      text-align: left;
      padding: 8px 6px;
    }
    th {
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .actions {
      margin-top: 8px;
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
    button {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: white;
      color: var(--text);
      padding: 8px 12px;
      cursor: pointer;
      font-weight: 600;
    }
    button.primary {
      background: var(--accent);
      color: white;
      border-color: var(--accent);
    }
    button.warn {
      background: var(--error);
      color: white;
      border-color: var(--error);
    }
    .status-ok { color: var(--accent-2); }
    .status-err { color: var(--error); }
  </style>
</head>
<body>
  <div class="wrap">
    <h1>Bench Server Dashboard</h1>
    <div class="muted">In-memory stats for current process lifetime (polling each 1s)</div>

    <div class="grid">
      <div class="card"><div class="label">Total Requests</div><div id="totalRequests" class="value">0</div></div>
      <div class="card"><div class="label">Total Errors</div><div id="totalErrors" class="value">0</div></div>
      <div class="card"><div class="label">RPS (last 1s)</div><div id="rpsLastSecond" class="value">0</div></div>
      <div class="card"><div class="label">Total Bytes</div><div id="totalBytes" class="value">0</div></div>
      <div class="card"><div class="label">Uptime (s)</div><div id="uptime" class="value">0</div></div>
    </div>

    <div class="panel">
      <div class="label">Requests / second</div>
      <div class="chart-wrap">
        <canvas id="rpsCanvas" width="1024" height="300"></canvas>
        <div id="chartTooltip" class="chart-tooltip"></div>
      </div>
    </div>

    <div class="panel">
      <div class="label">Requests by Endpoint</div>
      <table id="endpointTable">
        <thead>
          <tr>
            <th>Endpoint</th>
            <th>Requests</th>
            <th>Errors</th>
            <th>Bytes</th>
            <th>P50 ms</th>
            <th>P95 ms</th>
            <th>P99 ms</th>
          </tr>
        </thead>
        <tbody></tbody>
      </table>
    </div>

    <div class="panel">
      <div class="label">Status Codes</div>
      <table id="statusTable">
        <thead>
          <tr>
            <th>Status</th>
            <th>Count</th>
          </tr>
        </thead>
        <tbody></tbody>
      </table>
      <div class="actions">
        <button class="primary" onclick="refreshNow()">Refresh</button>
        <button class="warn" onclick="resetStats()">Reset Stats</button>
        <span id="updateState" class="muted">Loading...</span>
      </div>
    </div>
  </div>

  <script>
    const els = {
      totalRequests: document.getElementById('totalRequests'),
      totalErrors: document.getElementById('totalErrors'),
      rpsLastSecond: document.getElementById('rpsLastSecond'),
      totalBytes: document.getElementById('totalBytes'),
      uptime: document.getElementById('uptime'),
      endpointBody: document.querySelector('#endpointTable tbody'),
      statusBody: document.querySelector('#statusTable tbody'),
      canvas: document.getElementById('rpsCanvas'),
      chartTooltip: document.getElementById('chartTooltip'),
      updateState: document.getElementById('updateState'),
    };

    const chartState = {
      rawPoints: [],
      plotPoints: [],
      width: 0,
      height: 0,
      pad: 36,
      activeIndex: null,
    };

    function fmtNum(value) {
      return new Intl.NumberFormat().format(value);
    }

    function fmtFloat(value) {
      return Number(value || 0).toFixed(2);
    }

    function hideChartTooltip() {
      els.chartTooltip.classList.remove('visible');
    }

    function showChartTooltip(point) {
      if (!point || chartState.width === 0 || chartState.height === 0) {
        hideChartTooltip();
        return;
      }

      const xScale = els.canvas.clientWidth / chartState.width;
      const yScale = els.canvas.clientHeight / chartState.height;
      const left = point.x * xScale;
      const top = point.y * yScale;

      els.chartTooltip.textContent = `t+${point.secondOffset}s: ${fmtNum(point.requests)} RPS`;
      els.chartTooltip.style.left = `${left}px`;
      els.chartTooltip.style.top = `${top}px`;
      els.chartTooltip.classList.add('visible');
    }

    function normalizeChartPoints(points) {
      return (points || []).map((point, idx) => ({
        requests: Number(point.requests || 0),
        secondOffset: Number(point.second_offset ?? point.secondOffset ?? idx),
      }));
    }

    function drawRpsChart(points, activeIndex = null) {
      const normalizedPoints = normalizeChartPoints(points);
      chartState.rawPoints = normalizedPoints;
      chartState.activeIndex = activeIndex;

      const ctx = els.canvas.getContext('2d');
      const width = els.canvas.width;
      const height = els.canvas.height;
      ctx.clearRect(0, 0, width, height);

      const pad = chartState.pad;
      const values = normalizedPoints.map(point => point.requests);
      const maxValue = Math.max(1, ...values);
      const innerWidth = width - pad * 2;
      const innerHeight = height - pad * 2;
      chartState.width = width;
      chartState.height = height;

      ctx.strokeStyle = '#cbd5e1';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(pad, height - pad);
      ctx.lineTo(width - pad, height - pad);
      ctx.moveTo(pad, pad);
      ctx.lineTo(pad, height - pad);
      ctx.stroke();

      const yTicks = 4;
      ctx.strokeStyle = '#e2e8f0';
      ctx.fillStyle = '#64748b';
      ctx.font = '11px "IBM Plex Mono", "Fira Mono", monospace';
      for (let i = 0; i <= yTicks; i += 1) {
        const ratio = i / yTicks;
        const y = height - pad - ratio * innerHeight;
        const value = Math.round(maxValue * ratio);
        ctx.beginPath();
        ctx.moveTo(pad, y);
        ctx.lineTo(width - pad, y);
        ctx.stroke();
        ctx.fillText(String(value), 8, y + 4);
      }

      if (!normalizedPoints.length) {
        chartState.plotPoints = [];
        hideChartTooltip();
        return;
      }

      const stepX = normalizedPoints.length > 1 ? innerWidth / (normalizedPoints.length - 1) : 0;
      chartState.plotPoints = normalizedPoints.map((point, idx) => {
        const x = pad + idx * stepX;
        const y = height - pad - (point.requests / maxValue) * innerHeight;
        return {
          x,
          y,
          requests: point.requests,
          secondOffset: point.secondOffset,
        };
      });

      ctx.fillStyle = 'rgba(14, 165, 233, 0.14)';
      ctx.beginPath();
      chartState.plotPoints.forEach((point, idx) => {
        if (idx === 0) {
          ctx.moveTo(point.x, point.y);
        } else {
          ctx.lineTo(point.x, point.y);
        }
      });
      ctx.lineTo(width - pad, height - pad);
      ctx.lineTo(pad, height - pad);
      ctx.closePath();
      ctx.fill();

      ctx.strokeStyle = '#0ea5e9';
      ctx.lineWidth = 2;
      ctx.beginPath();
      chartState.plotPoints.forEach((point, idx) => {
        if (idx === 0) {
          ctx.moveTo(point.x, point.y);
        } else {
          ctx.lineTo(point.x, point.y);
        }
      });
      ctx.stroke();

      ctx.fillStyle = '#0369a1';
      chartState.plotPoints.forEach((point, idx) => {
        const radius = idx === activeIndex ? 4.5 : 2.5;
        ctx.beginPath();
        ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
        ctx.fill();
      });

      if (activeIndex !== null && chartState.plotPoints[activeIndex]) {
        const point = chartState.plotPoints[activeIndex];
        ctx.strokeStyle = 'rgba(15, 23, 42, 0.35)';
        ctx.setLineDash([5, 4]);
        ctx.beginPath();
        ctx.moveTo(point.x, pad);
        ctx.lineTo(point.x, height - pad);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      const first = chartState.plotPoints[0];
      const last = chartState.plotPoints[chartState.plotPoints.length - 1];
      ctx.fillStyle = '#64748b';
      ctx.font = '11px "IBM Plex Mono", "Fira Mono", monospace';
      ctx.fillText(`t+${first.secondOffset}s`, pad, height - 8);
      ctx.textAlign = 'right';
      ctx.fillText(`t+${last.secondOffset}s`, width - pad, height - 8);
      ctx.textAlign = 'left';
    }

    function findNearestPointIndex(canvasX) {
      if (!chartState.plotPoints.length) {
        return null;
      }

      let bestIndex = 0;
      let bestDistance = Number.POSITIVE_INFINITY;

      chartState.plotPoints.forEach((point, idx) => {
        const distance = Math.abs(point.x - canvasX);
        if (distance < bestDistance) {
          bestDistance = distance;
          bestIndex = idx;
        }
      });

      const spacing = chartState.plotPoints.length > 1
        ? (chartState.width - chartState.pad * 2) / (chartState.plotPoints.length - 1)
        : 24;
      const threshold = Math.max(10, spacing * 0.6);
      return bestDistance <= threshold ? bestIndex : null;
    }

    function handleChartMouseMove(event) {
      if (!chartState.rawPoints.length) {
        hideChartTooltip();
        return;
      }

      const rect = els.canvas.getBoundingClientRect();
      const scaleX = chartState.width / rect.width;
      const canvasX = (event.clientX - rect.left) * scaleX;

      const nearestIndex = findNearestPointIndex(canvasX);
      if (nearestIndex === null) {
        if (chartState.activeIndex !== null) {
          drawRpsChart(chartState.rawPoints, null);
        }
        hideChartTooltip();
        return;
      }

      if (chartState.activeIndex !== nearestIndex) {
        drawRpsChart(chartState.rawPoints, nearestIndex);
      }
      showChartTooltip(chartState.plotPoints[nearestIndex]);
    }

    function handleChartMouseLeave() {
      if (chartState.activeIndex !== null) {
        drawRpsChart(chartState.rawPoints, null);
      }
      hideChartTooltip();
    }

    function renderSummary(summary) {
      els.totalRequests.textContent = fmtNum(summary.total_requests || 0);
      els.totalErrors.textContent = fmtNum(summary.total_errors || 0);
      els.rpsLastSecond.textContent = fmtNum(summary.rps_last_second || 0);
      els.totalBytes.textContent = fmtNum(summary.total_bytes || 0);
      els.uptime.textContent = fmtNum(summary.uptime_seconds || 0);

      els.endpointBody.innerHTML = '';
      for (const row of (summary.by_endpoint || [])) {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td>${row.endpoint}</td>
          <td>${fmtNum(row.requests)}</td>
          <td class="${row.errors > 0 ? 'status-err' : 'status-ok'}">${fmtNum(row.errors)}</td>
          <td>${fmtNum(row.bytes)}</td>
          <td>${fmtFloat(row.p50_ms)}</td>
          <td>${fmtFloat(row.p95_ms)}</td>
          <td>${fmtFloat(row.p99_ms)}</td>
        `;
        els.endpointBody.appendChild(tr);
      }

      els.statusBody.innerHTML = '';
      for (const row of (summary.by_status || [])) {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td class="${row.status >= 400 ? 'status-err' : 'status-ok'}">${row.status}</td>
          <td>${fmtNum(row.count)}</td>
        `;
        els.statusBody.appendChild(tr);
      }
    }

    async function refreshNow() {
      try {
        els.updateState.textContent = 'Updating...';
        const [summaryResp, timeseriesResp] = await Promise.all([
          fetch('/api/stats/summary'),
          fetch('/api/stats/timeseries')
        ]);

        if (!summaryResp.ok || !timeseriesResp.ok) {
          throw new Error(`HTTP ${summaryResp.status}/${timeseriesResp.status}`);
        }

        const summary = await summaryResp.json();
        const timeseries = await timeseriesResp.json();

        renderSummary(summary);
        drawRpsChart(timeseries.points || [], chartState.activeIndex);
        if (
          chartState.activeIndex !== null &&
          chartState.plotPoints[chartState.activeIndex]
        ) {
          showChartTooltip(chartState.plotPoints[chartState.activeIndex]);
        } else {
          hideChartTooltip();
        }
        els.updateState.textContent = `Updated ${new Date().toLocaleTimeString()}`;
      } catch (error) {
        els.updateState.textContent = `Update failed: ${error.message}`;
      }
    }

    async function resetStats() {
      try {
        els.updateState.textContent = 'Resetting...';
        const response = await fetch('/api/stats/reset', { method: 'POST' });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        await refreshNow();
      } catch (error) {
        els.updateState.textContent = `Reset failed: ${error.message}`;
      }
    }

    els.canvas.addEventListener('mousemove', handleChartMouseMove);
    els.canvas.addEventListener('mouseleave', handleChartMouseLeave);

    refreshNow();
    setInterval(refreshNow, 1000);
  </script>
</body>
</html>
"#;
