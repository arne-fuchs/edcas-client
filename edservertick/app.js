"use strict";

// Same-origin: this site's nginx reverse-proxies /api/ to the edcas-server container.
const API = {
  tick: "/api/v1/server-tick",
  ticks: "/api/v1/server-ticks",
};

// ─── Countdown ────────────────────────────────────────────────
const el = {
  cd: document.getElementById("countdown"),
  h: document.getElementById("cd-h"),
  m: document.getElementById("cd-m"),
  s: document.getElementById("cd-s"),
  next: document.getElementById("m-next"),
  last: document.getElementById("m-last"),
  count: document.getElementById("m-count"),
};

let nextTickMs = null; // epoch ms of predicted next tick

const pad = (n) => String(n).padStart(2, "0");

function fmtUtc(iso) {
  if (!iso) return "—";
  const d = new Date(iso);
  if (isNaN(d)) return "—";
  return (
    `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ` +
    `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())} UTC`
  );
}

async function fetchTick() {
  try {
    const r = await fetch(API.tick, { cache: "no-store" });
    if (!r.ok) throw new Error(r.status);
    const d = await r.json();
    nextTickMs = d.next_predicted_tick ? new Date(d.next_predicted_tick).getTime() : null;
    el.next.textContent = fmtUtc(d.next_predicted_tick);
    el.last.textContent = fmtUtc(d.last_tick);
    el.count.textContent =
      d.system_count != null ? d.system_count.toLocaleString() : "—";
  } catch (e) {
    // leave previous values; render() handles a null target
    console.warn("fetchTick failed:", e);
  }
}

function render() {
  if (nextTickMs == null) {
    el.cd.dataset.state = "loading";
    el.h.textContent = el.m.textContent = el.s.textContent = "--";
    return;
  }
  const diff = nextTickMs - Date.now();
  if (diff <= 0) {
    // Tick window reached — show dramatic state and poll for the next prediction.
    el.cd.dataset.state = "imminent";
    return;
  }
  el.cd.dataset.state = "ready";
  const total = Math.floor(diff / 1000);
  el.h.textContent = pad(Math.floor(total / 3600));
  el.m.textContent = pad(Math.floor((total % 3600) / 60));
  el.s.textContent = pad(total % 60);
}

// ─── History chart ────────────────────────────────────────────
async function fetchHistory() {
  const wrap = document.getElementById("chart-empty");
  let ticks = [];
  try {
    const r = await fetch(API.ticks, { cache: "no-store" });
    if (r.ok) ticks = await r.json();
  } catch (e) {
    console.warn("fetchHistory failed:", e);
  }

  if (!Array.isArray(ticks) || ticks.length === 0) {
    wrap.hidden = false;
    return;
  }
  wrap.hidden = true;

  // Oldest → newest for left-to-right time progression.
  ticks.sort((a, b) => new Date(a.tick_time) - new Date(b.tick_time));

  const labels = [];
  const data = [];
  for (const t of ticks) {
    const d = new Date(t.tick_time);
    labels.push(
      `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`
    );
    data.push(d.getUTCHours() + d.getUTCMinutes() / 60);
  }

  const orange = "#ff7100";
  const orangeHi = "#ffb000";
  const dim = "#8a5a30";

  new Chart(document.getElementById("tickChart"), {
    type: "line",
    data: {
      labels,
      datasets: [
        {
          label: "Tick time (UTC)",
          data,
          borderColor: orange,
          backgroundColor: orangeHi,
          pointBackgroundColor: orangeHi,
          pointBorderColor: orange,
          pointRadius: 4,
          pointHoverRadius: 7,
          borderWidth: 1.5,
          tension: 0.25,
          spanGaps: true,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
        tooltip: {
          backgroundColor: "rgba(10,5,0,0.95)",
          borderColor: orange,
          borderWidth: 1,
          titleColor: orangeHi,
          bodyColor: "#ffce9e",
          callbacks: {
            label: (ctx) => {
              const t = ticks[ctx.dataIndex];
              return [
                `Tick: ${fmtUtc(t.tick_time)}`,
                `Systems: ${(t.system_count ?? 0).toLocaleString()}`,
              ];
            },
          },
        },
      },
      scales: {
        x: {
          grid: { color: "rgba(255,113,0,0.08)" },
          ticks: { color: dim, maxRotation: 60, minRotation: 45, autoSkipPadding: 16 },
        },
        y: {
          min: 0,
          max: 24,
          grid: { color: "rgba(255,113,0,0.08)" },
          ticks: {
            color: dim,
            stepSize: 3,
            callback: (v) => `${pad(v)}:00`,
          },
          title: { display: true, text: "UTC time of day", color: dim },
        },
      },
    },
  });
}

// ─── Boot ─────────────────────────────────────────────────────
fetchTick();
fetchHistory();
setInterval(render, 250);            // smooth second rollover
setInterval(fetchTick, 60_000);     // refresh prediction every minute
// When in the imminent window, poll faster so we pick up the new prediction quickly.
setInterval(() => {
  if (nextTickMs != null && nextTickMs - Date.now() <= 0) fetchTick();
}, 15_000);
