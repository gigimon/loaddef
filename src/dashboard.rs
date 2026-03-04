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
      <div class="card"><div class="label">Avg RPS (whole run)</div><div id="avgRps" class="value">0</div></div>
      <div class="card"><div class="label">Total Bytes</div><div id="totalBytes" class="value">0</div></div>
      <div class="card"><div class="label">Uptime (s)</div><div id="uptime" class="value">0</div></div>
    </div>

    <div class="panel">
      <div class="label">Requests / second</div>
      <canvas id="rpsCanvas" width="1024" height="300"></canvas>
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
      avgRps: document.getElementById('avgRps'),
      totalBytes: document.getElementById('totalBytes'),
      uptime: document.getElementById('uptime'),
      endpointBody: document.querySelector('#endpointTable tbody'),
      statusBody: document.querySelector('#statusTable tbody'),
      canvas: document.getElementById('rpsCanvas'),
      updateState: document.getElementById('updateState'),
    };

    function fmtNum(value) {
      return new Intl.NumberFormat().format(value);
    }

    function fmtFloat(value) {
      return Number(value || 0).toFixed(2);
    }

    function drawRpsChart(points) {
      const ctx = els.canvas.getContext('2d');
      const width = els.canvas.width;
      const height = els.canvas.height;
      ctx.clearRect(0, 0, width, height);

      const pad = 28;
      const values = points.map(p => p.requests);
      const maxValue = Math.max(1, ...values);

      ctx.strokeStyle = '#cbd5e1';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(pad, height - pad);
      ctx.lineTo(width - pad, height - pad);
      ctx.moveTo(pad, pad);
      ctx.lineTo(pad, height - pad);
      ctx.stroke();

      ctx.fillStyle = '#475569';
      ctx.font = '12px monospace';
      ctx.fillText('0', 6, height - pad + 4);
      ctx.fillText(String(maxValue), 6, pad + 4);

      if (!points.length) {
        return;
      }

      const stepX = points.length > 1 ? (width - pad * 2) / (points.length - 1) : 0;

      ctx.strokeStyle = '#0ea5e9';
      ctx.lineWidth = 2;
      ctx.beginPath();
      points.forEach((point, idx) => {
        const x = pad + idx * stepX;
        const ratio = point.requests / maxValue;
        const y = height - pad - ratio * (height - pad * 2);
        if (idx === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });
      ctx.stroke();
    }

    function renderSummary(summary) {
      els.totalRequests.textContent = fmtNum(summary.total_requests || 0);
      els.totalErrors.textContent = fmtNum(summary.total_errors || 0);
      els.avgRps.textContent = fmtFloat(summary.avg_rps);
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
        drawRpsChart(timeseries.points || []);
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

    refreshNow();
    setInterval(refreshNow, 1000);
  </script>
</body>
</html>
"#;
