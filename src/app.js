/**
 * Claude 中文助手 — Frontend
 */
const $ = (s) => document.querySelector(s);

let isTauri = false;

// ── Detect Tauri ─────────────────────────────────────────
async function getTauriInvoke() {
  try {
    const m = await import("@tauri-apps/api/core");
    isTauri = true;
    return m.invoke;
  } catch {
    isTauri = false;
    return null;
  }
}

// ── Bootstrap ────────────────────────────────────────────
(async function boot() {
  const invoke = await getTauriInvoke();
  await refreshAll(invoke);
  bindEvents(invoke);
  // Auto-check for app updates on startup (silent, only notify if available)
  checkAppUpdateSilent(invoke);
})();

// ── App Auto-Update ──────────────────────────────────────
async function checkAppUpdate(invoke, showUpToDate = false) {
  if (!isTauri || !invoke) return;
  const handledByUpdater = await checkTauriUpdater(showUpToDate);
  if (handledByUpdater) return;

  await checkReleaseUpdateFallback(invoke, showUpToDate);
}

async function checkTauriUpdater(showUpToDate = false) {
  const info = $("#updateInfo");
  try {
    const updater = await import("@tauri-apps/plugin-updater");
    const update = await updater.check();

    info.classList.remove("hidden");
    info.classList.toggle("available", !!update);
    info.classList.toggle("uptodate", !update && showUpToDate);

    if (update) {
      const notes = update.body ? `<div class="update-detail">${escapeHtml(update.body)}</div>` : "";
      info.innerHTML = `
        <div class="update-alert">🔔 发现新版本 v${escapeHtml(update.version || "")}</div>
        ${notes}
        <div class="button-row" style="margin-top:8px;">
          <button class="btn btn-primary" id="btnDoUpdate">⬇️ 立即更新</button>
        </div>`;
      $("#btnDoUpdate")?.addEventListener("click", async () => {
        const btn = $("#btnDoUpdate");
        btn.disabled = true;
        btn.textContent = "⏳ 正在下载并安装…";
        try {
          await update.downloadAndInstall();
          info.classList.remove("available");
          info.classList.add("uptodate");
          info.innerHTML = `<div class="update-ok">✅ 更新已安装，请重启应用完成更新。</div>`;
        } catch (e) {
          btn.disabled = false;
          btn.textContent = "⬇️ 立即更新";
          info.insertAdjacentHTML(
            "beforeend",
            `<div class="update-detail">自动更新失败：${escapeHtml(e.message || String(e))}</div>`,
          );
        }
      });
    } else if (showUpToDate) {
      info.innerHTML = `<div class="update-ok">✅ 已是最新版本</div>`;
    } else {
      info.classList.add("hidden");
    }
    return true;
  } catch (e) {
    console.warn("tauri updater failed, falling back to release check:", e);
    return false;
  }
}

async function checkReleaseUpdateFallback(invoke, showUpToDate = false) {
  const info = $("#updateInfo");
  try {
    const update = await invoke("check_for_updates");
    info.classList.remove("hidden");
    info.classList.toggle("available", update.hasUpdate);
    info.classList.toggle("uptodate", !update.hasUpdate && showUpToDate);

    if (update.hasUpdate) {
      info.innerHTML = `
        <div class="update-alert">🔔 发现新版本 v${escapeHtml(update.latestVersion)}</div>
        <div class="update-detail">当前版本 v${escapeHtml(update.currentVersion)}</div>
        <div class="update-detail">自动更新清单暂不可用，已切换到手动下载。</div>
        <div class="button-row" style="margin-top:8px;">
          <button class="btn btn-primary" id="btnDoUpdate">⬇️ 打开下载页</button>
        </div>
        <div class="update-time">检查时间：${escapeHtml(update.checkTime || "")}</div>`;
      $("#btnDoUpdate")?.addEventListener("click", () => {
        invoke("open_url_in_browser", { url: update.releaseUrl });
      });
    } else if (showUpToDate) {
      info.innerHTML = `
        <div class="update-ok">✅ 已是最新版本</div>
        <div class="update-time">检查时间：${escapeHtml(update.checkTime || "")}</div>`;
    } else {
      info.classList.add("hidden");
    }
  } catch(e) {
    if (showUpToDate) {
      info.classList.remove("hidden");
      info.classList.remove("available", "uptodate");
      info.innerHTML = `⚠️ 检查更新失败: ${e.message || e}`;
    }
  }
}

async function checkAppUpdateSilent(invoke) {
  // Wait a moment for the UI to settle, then check silently
  setTimeout(() => checkAppUpdate(invoke, false), 3000);
}

// ── Refresh ──────────────────────────────────────────────
async function refreshAll(invoke) {
  // 1) System info
  try {
    const info = await invoke("get_system_info");
    renderSystemInfo(info);
    const zh = await invoke("check_zh_cn_status");
    renderZhCnStatus(zh);
  } catch (e) {
    console.error("system info failed:", e);
    $("#sysOs").textContent = "检测失败";
    $("#sysClaude").textContent = "检测失败";
    $("#sysZhCn").textContent = "检测失败";
  }
  enableButtons();

  // 2) Version
  try {
    const v = await invoke("get_versions");
    $("#verApp").textContent = `v${v.app}`;
  } catch (e) {
    console.error("version failed:", e);
    $("#verApp").textContent = "检测失败";
  }

  // 3) cc-switch
  try {
    const cc = await invoke("check_ccswitch_status");
    updateCcswitchUI(cc);
  } catch (e) {
    console.error("ccswitch check failed:", e);
    $("#sysCcswitch").textContent = "检测失败";
    $("#ccswitchStatus").textContent = "检测失败";
  }

  // 4) API Guide (load inline)
  try {
    const guide = await invoke("get_api_guide");
    $("#apiGuide").innerHTML = renderMarkdown(guide);
  } catch (e) {
    console.error("api guide failed:", e);
    $("#apiGuide").innerHTML = `<p style="padding:16px;">⚠️ 教程加载失败: ${e}</p>`;
  }
}

// ── Render ───────────────────────────────────────────────
function renderSystemInfo(info) {
  const osMap = { macos: "🍎 macOS", windows: "🪟 Windows" };
  $("#sysOs").textContent = osMap[info.os] || info.os;
  $("#sysClaude").textContent = info.claudeInstalled
    ? (info.claudeVersion ? `✅ 已安装 (v${info.claudeVersion})` : "✅ 已安装")
    : "❌ 未安装";
}

function renderZhCnStatus(installed) {
  $("#sysZhCn").textContent = installed ? "✅ 已汉化" : "⚪ 未汉化";
  $("#sysZhCn").style.color = installed ? "var(--green)" : "var(--text-secondary)";

  const dot = $("#statusDot"), text = $("#statusText");
  dot.className = installed ? "status-dot ok" : "status-dot warn";
  text.textContent = installed ? "已汉化" : "待汉化";
}

function enableButtons() {
  $("#btnInstall").disabled = false;
  $("#btnUninstall").disabled = false;
  $("#btnCheckUpdate").disabled = false;
}

function updateCcswitchUI(cc) {
  if (cc.installed) {
    $("#sysCcswitch").textContent = "✅ 已安装"; $("#sysCcswitch").style.color = "var(--green)";
    $("#ccswitchStatus").textContent = "✅ 已安装";
    if (cc.version) { $("#ccswitchVersion").textContent = `版本: ${cc.version}`; }
    $("#btnCcswitchInstall").textContent = "✅ 已安装"; $("#btnCcswitchInstall").disabled = true;
  } else {
    $("#sysCcswitch").textContent = "⚠️ 未安装"; $("#sysCcswitch").style.color = "var(--orange)";
    $("#ccswitchStatus").textContent = "⚠️ 未安装"; $("#ccswitchVersion").textContent = "";
    $("#btnCcswitchInstall").textContent = "⚡ 一键安装"; $("#btnCcswitchInstall").disabled = false;
  }
}

// ── Events ───────────────────────────────────────────────
function bindEvents(invoke) {

  // Install
  $("#btnInstall").addEventListener("click", async () => {
    const lang = $("#langSelect").value, mode = $("#modeSelect").value;
    lockButtons();
    showProgress(true); showLog(true, "");
    appendLog(`🔧 ${lang} / ${mode} 安装中…\n📋 请在弹出的授权窗口确认；Windows 会通过 PowerShell 执行\n`);

    const start = Date.now();
    let t = setInterval(() => appendLog(`⏱️ ${Math.floor((Date.now()-start)/1000)}s\n`), 10000);

    try {
      const r = await invoke("install_localization", { options: { langCode: lang, mode } });
      clearInterval(t);
      showProgress(false);
      appendLog(`\n✅ 完成 (${Math.floor((Date.now()-start)/1000)}s)\n${r.message}\n`);
    } catch(e) {
      clearInterval(t);
      showProgress(false);
      appendLog(`\n❌ ${e}\n`);
    }
    unlockButtons();
    await refreshAll(invoke);
  });

  // Uninstall
  $("#btnUninstall").addEventListener("click", async () => {
    lockButtons();
    showProgress(true); showLog(true, "🔧 正在恢复原始版本…\n📋 请在弹出的授权窗口确认；Windows 会通过 PowerShell 执行\n");
    const start = Date.now();
    let t = setInterval(() => appendLog(`⏱️ ${Math.floor((Date.now()-start)/1000)}s\n`), 10000);
    try {
      const r = await invoke("uninstall_localization");
      clearInterval(t); showProgress(false);
      appendLog(`\n✅ 完成 (${Math.floor((Date.now()-start)/1000)}s)\n${r.message}\n`);
    } catch(e) {
      clearInterval(t); showProgress(false);
      appendLog(`\n❌ ${e}\n`);
    }
    unlockButtons();
    await refreshAll(invoke);
  });

  // Updates (manual)
  $("#btnCheckUpdate").addEventListener("click", async () => {
    await checkAppUpdate(invoke, true);
  });

  // cc-switch install
  $("#btnCcswitchInstall").addEventListener("click", async () => {
    const btn = $("#btnCcswitchInstall"), res = $("#ccswitchResult");
    btn.disabled = true; btn.textContent = "⏳ 安装中…";
    res.classList.remove("hidden"); res.textContent = "⏳ 正在安装，可能需要几分钟；Windows 可能会弹出 UAC 授权…"; res.style.color = "var(--text-secondary)";
    let installed = false;
    try {
      const msg = await invoke("install_ccswitch");
      const cc = await refreshCcswitchStatus(invoke, 5);
      installed = !!cc?.installed;
      res.textContent = installed
        ? msg
        : `${msg}\n\n如果按钮未自动变为“已安装”，请从开始菜单启动 cc-switch 后再刷新检测。`;
      res.style.color = "var(--green)";
    } catch(e) {
      res.textContent = "❌ " + e; res.style.color = "var(--red)";
      try { await invoke("open_ccswitch_releases"); } catch {}
    } finally {
      if (!installed) {
        btn.disabled = false;
        btn.textContent = "⚡ 一键安装";
      }
    }
  });

  // cc-switch guide
  $("#btnCcswitchGuide").addEventListener("click", async () => {
    try { openModal("cc-switch 教程", renderMarkdown(await invoke("get_ccswitch_guide"))); }
    catch(e) { openModal("错误", `<p>${e}</p>`); }
  });

  // cc-switch site
  $("#btnCcswitchSite").addEventListener("click", () => invoke("open_ccswitch_site"));
  $("#btnCcswitchSite2").addEventListener("click", () => invoke("open_ccswitch_site"));

  // Config
  $("#btnOpenConfig").addEventListener("click", async () => {
    try { await invoke("open_config_file"); } catch(e) { alert("失败: " + e); }
  });

  // Modal
  $("#btnCloseModal").addEventListener("click", closeModal);
  $(".modal-overlay").addEventListener("click", closeModal);
  document.addEventListener("keydown", e => { if (e.key === "Escape") closeModal(); });
}

// ── UI ───────────────────────────────────────────────────
function lockButtons()   { ["btnInstall","btnUninstall","btnCheckUpdate"].forEach(id => $(`#${id}`).disabled = true); }
function unlockButtons() { ["btnInstall","btnUninstall","btnCheckUpdate"].forEach(id => $(`#${id}`).disabled = false); }

function showProgress(show) {
  const bar = $("#installProgress");
  bar.classList.toggle("hidden", !show);
  bar.classList.toggle("running", show);
}

function showLog(show, text = "") {
  const log = $("#installLog");
  log.classList.toggle("hidden", !show);
  if (text) log.textContent = text;
}

function appendLog(text) {
  const log = $("#installLog");
  log.textContent += text; log.scrollTop = log.scrollHeight;
}

function openModal(title, html) {
  $("#modalTitle").textContent = title;
  $("#modalBody").innerHTML = html;
  $("#guideModal").classList.remove("hidden");
}
function closeModal() { $("#guideModal").classList.add("hidden"); }
function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

async function refreshCcswitchStatus(invoke, attempts = 1) {
  let latest = null;
  for (let i = 0; i < attempts; i++) {
    try {
      latest = await invoke("check_ccswitch_status");
      updateCcswitchUI(latest);
      if (latest.installed) return latest;
    } catch (e) {
      console.error("ccswitch refresh failed:", e);
    }
    if (i < attempts - 1) await sleep(1000);
  }
  return latest;
}

// ── Markdown ─────────────────────────────────────────────
function renderMarkdown(md) {
  if (!md) return "";

  let lines = md.split("\n");
  let html = [];
  let inCode = false, codeBuf = [], codeLang = "";
  let inTable = false, tableRows = [];
  let inPara = false;

  function flushPara() {
    if (inPara) { html.push("</p>"); inPara = false; }
  }

  function flushTable() {
    if (!inTable) return;
    inTable = false;
    if (tableRows.length === 0) return;
    let t = "<table><thead>";
    // First row is header
    let hCells = parseTableRow(tableRows[0]);
    t += "<tr>" + hCells.map(c => `<th>${inlineMarkdown(c.trim())}</th>`).join("") + "</tr>";
    t += "</thead><tbody>";
    // Skip separator row if present
    let start = (tableRows.length > 1 && /^\|[\s\-:|]+\|/.test(tableRows[1])) ? 2 : 1;
    for (let i = start; i < tableRows.length; i++) {
      let cells = parseTableRow(tableRows[i]);
      t += "<tr>" + cells.map(c => `<td>${inlineMarkdown(c.trim())}</td>`).join("") + "</tr>";
    }
    t += "</tbody></table>";
    html.push(t);
    tableRows = [];
  }

  function parseTableRow(row) {
    return row.replace(/^\||\|$/g, "").split("|");
  }

  function inlineMarkdown(text) {
    return text
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
      .replace(/https?:\/\/[^\s<)]+/g, '<a href="$&">$&</a>');
  }

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];

    // Code block fence
    if (/^```/.test(line)) {
      flushTable();
      flushPara();
      if (!inCode) {
        inCode = true;
        codeLang = line.slice(3).trim();
        codeBuf = [];
      } else {
        html.push(`<pre><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`);
        inCode = false;
        codeLang = "";
      }
      continue;
    }

    if (inCode) {
      codeBuf.push(line);
      continue;
    }

    // Blank line -> close paragraph
    if (line.trim() === "") {
      flushTable();
      flushPara();
      // Don't start a new one yet
      continue;
    }

    // Table
    if (line.startsWith("|")) {
      flushPara();
      if (!inTable) inTable = true;
      tableRows.push(line);
      continue;
    }

    // Not a table line but we were in a table
    if (inTable && !line.startsWith("|")) {
      flushTable();
    }

    // Headers
    if (line.startsWith("# "))    { flushPara(); flushTable(); html.push(`<h1>${inlineMarkdown(line.slice(2))}</h1>`); continue; }
    if (line.startsWith("## "))   { flushPara(); flushTable(); html.push(`<h2>${inlineMarkdown(line.slice(3))}</h2>`); continue; }
    if (line.startsWith("### "))  { flushPara(); flushTable(); html.push(`<h3>${inlineMarkdown(line.slice(4))}</h3>`); continue; }
    if (line.startsWith("#### ")) { flushPara(); flushTable(); html.push(`<h4>${inlineMarkdown(line.slice(5))}</h4>`); continue; }

    // Horizontal rule
    if (/^---+$/.test(line.trim())) {
      flushPara(); flushTable();
      html.push("<hr>");
      continue;
    }

    // Blockquote
    if (line.startsWith("> ")) {
      flushPara(); flushTable();
      html.push(`<blockquote>${inlineMarkdown(line.slice(2))}</blockquote>`);
      continue;
    }

    // Regular paragraph text
    if (!inPara) { html.push("<p>"); inPara = true; }
    else { html.push(" "); }
    html.push(inlineMarkdown(line));
  }

  flushTable();
  flushPara();
  if (inCode) {
    html.push(`<pre><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`);
  }

  return html.join("\n");
}

function escapeHtml(s) { return s.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;"); }
