/** SAQR Control Terminal — مرآة حية فقط (محرك + Binance). لا Math.random ولا صفقات وهمية. */
let alertActive = true;
let calcEnabled = false;
const API_POLL_MS = 3000;

let openPositions = [];
let liquidityPairs = [];
let tradeHistory = [];
let signals = [];
let engineLive = null;
let mirrorLive = null;
const READ_ONLY = true;

function showToast(msg) {
  const toast = document.getElementById('toast');
  const toastMsg = document.getElementById('toastMsg');
  if (!toast || !toastMsg) return;
  toastMsg.innerText = msg;
  toast.classList.remove('hidden');
  setTimeout(() => toast.classList.add('hidden'), 2800);
}

function toggleAccordion(id) {
  const sec = document.getElementById(id);
  const icon = document.getElementById(`${id}-icon`);
  if (!sec) return;
  sec.classList.toggle('open');
  if (icon) icon.style.transform = sec.classList.contains('open') ? 'rotate(180deg)' : '';
}

function readOnlyAction(label) {
  showToast(`اللوحة للعرض فقط — ${label} من Binance Testnet أو المحرك`);
}

function closeWinningTrades() {
  readOnlyAction('إغلاق الصفقات');
}
function closeLosingTrades() {
  readOnlyAction('إغلاق الصفقات');
}
function partialClosePosition() {
  readOnlyAction('تعديل الصفقات');
}
function closePosition() {
  readOnlyAction('إغلاق الصفقات');
}

function selectReport() {
  showToast('التقارير: من سجل المحرك لاحقاً — لا أرقام تجريبية');
}
function setReportPeriod() {
  selectReport();
}
function toggleCalculatorPower() {
  readOnlyAction('حاسبة الكelly');
}
function calculateLotSize() {
  readOnlyAction('حاسبة العقد');
}
function toggleBotState() {
  showToast('إيقاف/تشغيل البوت من نافذة SAQR_RUN (Ctrl+C) — اللوحة للعرض');
}
function emergencyCloseAll() {
  readOnlyAction('الإغلاق الطارئ');
}

function renderSignals() {
  const container = document.getElementById('signalsList');
  if (!container) return;
  container.innerHTML = '';
  if (!signals.length) {
    container.innerHTML =
      '<div class="text-center text-xs text-gray-400 font-medium py-3">لا إشارات مسجّلة من المحرك — انتظر Pass كامل أو راجع Verdict</div>';
    return;
  }
  signals.forEach((sig) => {
    const el = document.createElement('div');
    el.className = 'bg-[#121724]/90 p-2.5 rounded-xl border border-gray-800/60';
    el.innerHTML = `
      <div class="flex justify-between items-center">
        <span class="font-bold text-xs text-gray-200">${sig.pair}</span>
        <span class="text-[10px] font-num text-gold-400/90">${sig.direction} · ${sig.strengthScore}/100</span>
      </div>
      <div class="text-[10px] text-gray-400 mt-1">Sovereign: ${sig.rsi} | ${sig.executed ? 'نُفّذ' : 'لم يُنفّذ'}</div>`;
    container.appendChild(el);
  });
}

function renderPositions() {
  const container = document.getElementById('positionsList');
  if (!container) return;
  container.innerHTML = '';
  let totalWinning = 0;
  let totalLosing = 0;
  if (openPositions.length === 0) {
    container.innerHTML =
      '<div class="text-center text-xs text-gray-400 font-medium py-3">لا صفقات — المحرك لم ينفّذ (أو SAQR_ENABLE_TRADING=false)</div>';
    document.getElementById('winningPnlVal').innerText = '+0.00 $';
    document.getElementById('losingPnlVal').innerText = '-0.00 $';
    document.getElementById('activePositionsCount').innerText = '0';
    return;
  }
  openPositions.forEach((pos) => {
    const isProfitable = pos.pnl >= 0;
    if (isProfitable) totalWinning += pos.pnl;
    else totalLosing += Math.abs(pos.pnl);
    const item = document.createElement('div');
    item.className =
      'bg-[#121724]/90 p-2.5 rounded-xl border border-gray-800/60 flex items-center justify-between';
    item.innerHTML = `
      <div>
        <div class="flex items-center gap-1.5">
          <span class="font-bold text-xs text-gray-200">${pos.pair}</span>
          <span class="text-[10px] font-num font-bold ${isProfitable ? 'text-emerald-400/90' : 'text-rose-400/90'}">${isProfitable ? '+' : ''}${pos.pnl.toFixed(2)} $</span>
        </div>
        <div class="text-[10px] text-gray-400 mt-1">دخول: ${pos.entry} | حالي: ${pos.current}</div>
      </div>
      <span class="text-[10px] text-gray-500">مرآة</span>`;
    container.appendChild(item);
  });
  document.getElementById('winningPnlVal').innerText = `+${totalWinning.toFixed(2)} $`;
  document.getElementById('losingPnlVal').innerText = `-${totalLosing.toFixed(2)} $`;
  document.getElementById('activePositionsCount').innerText = String(openPositions.length);
}

function renderLiquidity() {
  const container = document.getElementById('liquidityList');
  if (!container) return;
  container.innerHTML = '';
  if (!liquidityPairs.length) {
    container.innerHTML = '<div class="text-center text-xs text-gray-400 py-2">لا سعر حي — WS غير متصل</div>';
    return;
  }
  liquidityPairs.forEach((item, index) => {
    const el = document.createElement('div');
    el.className =
      'bg-[#121724]/90 p-2.5 rounded-xl border border-gray-800/60 flex items-center justify-between';
    el.innerHTML = `<span class="font-bold text-gray-300 font-num">${index + 1}. ${item.pair}</span><span class="font-num text-amber-400/90 font-bold">${item.volume}</span>`;
    container.appendChild(el);
  });
}

function renderHistory() {
  const container = document.getElementById('historyList');
  if (!container) return;
  container.innerHTML =
    '<div class="text-center text-xs text-gray-400 py-2">سجل الصفقات: من المحرك/Binance — فارغ حالياً</div>';
}

function updateSignalTimerUI() {
  const timerEl = document.getElementById('signalTimer');
  if (!timerEl) return;
  timerEl.innerText = 'API';
}

function refreshSignals(isManual = false) {
  refreshFromEngineApis(isManual);
}

async function refreshFromEngineApis(isManual = false) {
  const icon = document.getElementById('refreshIcon');
  if (icon) icon.classList.add('fa-spin');
  try {
    const [sigRes, posRes, engRes, statusRes, mirrorRes] = await Promise.all([
      fetch('/api/signals'),
      fetch('/api/positions'),
      fetch('/api/engine'),
      fetch('/api/status'),
      fetch('/api/binance_mirror'),
    ]);

    if (sigRes.ok) {
      const sigJson = await sigRes.json();
      if (Array.isArray(sigJson)) {
        signals = sigJson.map((s) => ({
          id: s.id,
          pair: s.pair,
          direction: s.direction,
          strengthScore: s.strength_score ?? s.strengthScore ?? 0,
          timeframe: s.timeframe || 'live',
          rsi: String(Math.round(s.sovereign_score ?? 50)),
          executed: !!s.executed,
        }));
        renderSignals();
      }
    }

    if (posRes.ok) {
      const posJson = await posRes.json();
      if (Array.isArray(posJson)) {
        openPositions = posJson.map((p) => ({
          id: p.id,
          pair: p.pair,
          type: p.side || p.type || 'BUY',
          lot: p.lot,
          entry: p.entry,
          current: p.current,
          pnl: p.pnl,
        }));
        renderPositions();
      }
    }

    if (mirrorRes.ok) {
      mirrorLive = await mirrorRes.json();
      const banner = document.getElementById('mirrorBanner');
      if (banner) {
        const err = mirrorLive.binance_error;
        banner.innerText = err
          ? `مرآة: محرك حي — Binance: ${err}`
          : `مرآة حية | WS: ${mirrorLive.ws_connected ? 'نعم' : 'لا'} | أوامr Binance: ${mirrorLive.open_orders_on_binance ?? 0} | إشارات: ${mirrorLive.engine_signal_count ?? 0}`;
      }
    }

    if (statusRes.ok) {
      const st = await statusRes.json();
      const balEl = document.getElementById('totalBalance');
      const pctEl = document.getElementById('dailyProfitPct');
      if (balEl) {
        if (mirrorLive && mirrorLive.usdt_free != null) {
          balEl.innerText = `${Number(mirrorLive.usdt_free).toLocaleString('en-US', { minimumFractionDigits: 2 })} USDT (Binance)`;
        } else if (st.initial_balance_usd != null) {
          balEl.innerText = `${Number(st.initial_balance_usd).toFixed(2)} $ (إعداد)`;
        }
      }
      if (pctEl && st.binance_live_price > 0) {
        pctEl.innerText = `${Number(st.binance_live_price).toFixed(2)} · ${st.mode || 'demo'}`;
      }
      if (st.binance_live_price > 0 && st.symbol) {
        const sym = st.symbol.endsWith('USDT') ? st.symbol.replace('USDT', ' / USDT') : st.symbol;
        liquidityPairs = [{ pair: sym, volume: `${Number(st.binance_live_price).toFixed(2)} USDT` }];
        renderLiquidity();
      }
    }

    if (engRes.ok) {
      engineLive = await engRes.json();
      const statusText = document.getElementById('statusText');
      const statusDot = document.getElementById('statusDot');
      const statusPing = document.getElementById('statusPing');
      if (statusText && engineLive) {
        const ws = engineLive.ws_connected ? 'WS متصل' : 'WS غير متصل';
        statusText.innerText = `مرآة: ${ws} | ${engineLive.last_verdict || '—'}`;
        if (engineLive.emergency_lock || engineLive.stream_frozen) {
          statusText.className = 'text-[11px] font-bold text-red-400/90 tracking-wide';
          if (statusDot) statusDot.className = 'relative inline-flex rounded-full h-2.5 w-2.5 bg-red-500';
          if (statusPing) statusPing.classList.add('hidden');
        }
      }
    }

    if (isManual) showToast('تم تحديث المرآة من API');
  } catch (_) {
    showToast('المحرك غير متصل — شغّل SAQR_RUN.bat');
  } finally {
    if (icon) icon.classList.remove('fa-spin');
  }
}

function toggleAlerts() {
  alertActive = !alertActive;
  showToast(alertActive ? 'تنبيهات الواجهة مفعّلة' : 'متوقفة');
}

function updateSyncClock() {
  const el = document.getElementById('syncTime');
  if (el) el.innerText = new Date().toLocaleTimeString('ar-IQ', { hour12: false });
}

document.addEventListener('DOMContentLoaded', () => {
  renderSignals();
  renderPositions();
  renderLiquidity();
  renderHistory();
  updateSignalTimerUI();
  updateSyncClock();
  setInterval(updateSyncClock, 1000);
  refreshFromEngineApis();
  setInterval(refreshFromEngineApis, API_POLL_MS);
  fetch('/api/health').catch(() => {});
});
