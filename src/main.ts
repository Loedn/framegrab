import { addPluginListener, invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import "./style.css";

type Status = "queued" | "starting" | "downloading" | "processing" | "completed" | "failed" | "cancelled";
type Job = {
  id: string;
  url: string;
  site: string;
  status: Status;
  percent: number | null;
  message: string;
};
type DownloadEvent = { id: string; status: Status; percent: number | null; message: string };
type InstallStatus = { available: boolean; installed: boolean };

const sources = [
  { label: "YouTube", domain: "youtube.com", className: "youtube" },
  { label: "Instagram", domain: "instagram.com", className: "instagram" },
  { label: "TikTok", domain: "tiktok.com", className: "tiktok" },
  { label: "X / Twitter", domain: "x.com", className: "x" },
  { label: "Reddit", domain: "reddit.com", className: "reddit" },
  { label: "Facebook", domain: "facebook.com", className: "facebook" },
];
const supportedHosts = [
  "x.com", "twitter.com", "instagram.com", "facebook.com", "fb.watch",
  "reddit.com", "redd.it", "tiktok.com", "youtube.com", "youtu.be",
];
const android = isTauri() && /Android/i.test(navigator.userAgent);
if (android) document.body.classList.add("android");

const icons = {
  arrow: '<path d="M5 12h14m-6-6 6 6-6 6"/>',
  clipboard: '<rect x="8" y="4" width="12" height="16" rx="2"/><path d="M16 4.5V3a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v1.5M4 7v12a2 2 0 0 0 2 2h10"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  close: '<path d="M18 6 6 18M6 6l12 12"/>',
  download: '<path d="M12 3v12m-4-4 4 4 4-4M4 17v3h16v-3"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10H3z"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7.1-7.1l-1.7 1.7M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7.1 7.1l1.7-1.7"/>',
  more: '<path d="M5 12h.01M12 12h.01M19 12h.01" stroke-width="4" stroke-linecap="round"/>',
  spark: '<path d="m12 3 1.9 6.1L20 11l-6.1 1.9L12 19l-1.9-6.1L4 11l6.1-1.9L12 3Zm7 14 .7 1.3L21 19l-1.3.7L19 21l-.7-1.3L17 19l1.3-.7L19 17Z"/>',
};
function icon(name: keyof typeof icons, size = 20): string {
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${icons[name]}</svg>`;
}

const app = document.querySelector<HTMLDivElement>("#app")!;
app.innerHTML = `
  <div class="shell">
    <aside class="sidebar">
      <div class="brand"><div class="brand-mark">${icon("download", 19)}</div><span>framegrab<span class="brand-dot">.</span></span></div>
      <div class="side-label">WORKSPACE</div>
      <div class="side-item active">${icon("download", 18)}<span>Downloads</span><span class="side-count" id="side-count">0</span></div>
      <div class="side-spacer"></div>
      <div class="install-panel" id="install-panel" hidden>
        <button id="install-action" class="install-action" type="button"></button>
        <div id="install-feedback" class="install-feedback" role="status"></div>
      </div>
      <div class="side-foot"><div class="ready-dot"></div><span>Ready to download</span></div>
      <div class="version">FRAMEGRAB · ${android ? "ANDROID PREVIEW" : "DESKTOP"}</div>
    </aside>
    <main class="main">
      <header class="topbar"><div class="crumb">Workspace <span>/</span> <strong>Downloads</strong></div><div class="platform-tag"><span class="tag-dot"></span> ${android ? "ANDROID PREVIEW" : "DESKTOP APP"}</div></header>
      <div class="content">
        <div class="page-heading"><div><div class="eyebrow">YOUR MEDIA, YOUR WAY</div><h1>Download videos<span class="heading-period">.</span></h1><p>Drop in a link. Keep what matters, right on your device.</p></div><div class="heading-decoration">${icon("spark", 25)}</div></div>
        <section class="composer" aria-label="New download">
          <div class="composer-label"><span class="label-icon">${icon("link", 16)}</span> VIDEO LINK <span class="label-hint">Paste a public video URL to get started</span></div>
          <div class="url-row"><input id="url" type="url" spellcheck="false" autocomplete="off" placeholder="https://youtube.com/watch?v=..." aria-label="Video URL" /><button id="paste" class="paste-btn" type="button">${icon("clipboard", 17)} Paste</button></div>
          <div class="composer-bottom"><div class="destination"><div class="destination-icon">${icon("folder", 17)}</div><div class="destination-info"><small>SAVE TO</small><span id="folder-label">${android ? "Movies / Framegrab" : "Choose a destination folder"}</span></div>${android ? "" : `<button id="choose-folder" class="choose-folder" type="button">Change folder ${icon("arrow", 15)}</button>`}</div><button id="add" class="primary-btn" type="button">Add to queue ${icon("arrow", 18)}</button></div>
          <div id="form-error" role="alert" class="form-error" hidden></div>
        </section>
        <section class="sources" aria-label="Supported platforms"><div class="section-kicker">${android ? "COMPATIBLE LINKS VARY" : "WORKS WITH YOUR FAVORITES"} <span class="section-line"></span></div><div class="source-list">${sources.map((source) => `<div class="source-chip"><span class="source-icon ${source.className}">${source.label[0]}</span>${source.label}</div>`).join("")}</div></section>
        <section class="queue-section" aria-label="Download queue"><div class="queue-heading"><div><div class="section-kicker">ACTIVITY</div><h2>Download queue <span id="queue-count" class="queue-count">0</span></h2></div><span id="queue-summary" class="queue-summary">Nothing in the queue yet</span></div><div id="queue" class="queue"></div></section>
        <footer>${android ? "Android preview: only progressive MP4 videos with audio are supported. Some links and high-quality formats need FFmpeg or a JavaScript runtime and will not work yet. " : ""}Only download videos you have permission to save. Availability depends on each platform.</footer>
      </div>
    </main>
  </div>`;

const urlInput = document.querySelector<HTMLInputElement>("#url")!;
const folderLabel = document.querySelector<HTMLSpanElement>("#folder-label")!;
const errorElement = document.querySelector<HTMLDivElement>("#form-error")!;
const queueElement = document.querySelector<HTMLDivElement>("#queue")!;
const jobs: Job[] = [];
let directory = android ? "Movies / Framegrab" : localStorage.getItem("framegrab.directory") ?? "";
let activeId: string | null = null;
let launching = false;
let installStatus: InstallStatus = { available: false, installed: false };

function showError(message: string): void {
  errorElement.textContent = message;
  errorElement.hidden = !message;
}

function siteFor(url: string): string {
  const host = new URL(url).hostname.toLowerCase();
  if (host.includes("youtu")) return "YouTube";
  if (host.includes("instagram")) return "Instagram";
  if (host.includes("tiktok")) return "TikTok";
  if (host.includes("reddit") || host === "redd.it") return "Reddit";
  if (host.includes("facebook") || host === "fb.watch") return "Facebook";
  return "X / Twitter";
}

function validateUrl(input: string): string | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.username || url.password || url.port) {
      return "Use an HTTPS video link without credentials or a custom port.";
    }
    const host = url.hostname.toLowerCase();
    if (!supportedHosts.some((domain) => host === domain || host.endsWith(`.${domain}`))) {
      return "That site isn't supported yet. Try one of the platforms below.";
    }
    return null;
  } catch {
    return "Enter a valid video link to continue.";
  }
}

function updateFolder(): void {
  folderLabel.textContent = directory || "Choose a destination folder";
  folderLabel.title = directory;
}

function renderQueue(): void {
  document.querySelector("#queue-count")!.textContent = String(jobs.length);
  document.querySelector("#side-count")!.textContent = String(jobs.length);
  const pending = jobs.filter((job) => ["queued", "starting", "downloading", "processing"].includes(job.status)).length;
  document.querySelector("#queue-summary")!.textContent = pending ? `${pending} in progress or waiting` : jobs.length ? "All caught up" : "Nothing in the queue yet";
  if (!jobs.length) {
    queueElement.innerHTML = `<div class="empty-state"><div class="empty-icon">${icon("download", 25)}</div><strong>Your queue is clear</strong><p>Paste a video link above and your downloads will show up here.</p></div>`;
    return;
  }
  queueElement.replaceChildren();
  for (const job of jobs) {
    const row = document.createElement("div");
    row.className = `job job-${job.status}`;
    const badge = document.createElement("span");
    badge.className = `job-badge ${job.site.toLowerCase().replace(/[^a-z]/g, "")}`;
    badge.textContent = job.site === "X / Twitter" ? "X" : job.site[0];
    const body = document.createElement("div");
    body.className = "job-body";
    const title = document.createElement("div");
    title.className = "job-title";
    title.textContent = job.site + " video";
    const link = document.createElement("div");
    link.className = "job-link";
    link.textContent = job.url;
    link.title = job.url;
    body.append(title, link);
    const details = document.createElement("div");
    details.className = "job-details";
    const status = document.createElement("span");
    status.className = "job-status";
    const statusText: Record<Status, string> = {
      queued: "Waiting", starting: "Starting", downloading: "Downloading", processing: "Processing",
      completed: "Done", failed: "Failed", cancelled: "Cancelled",
    };
    status.textContent = statusText[job.status] + (job.status === "downloading" && job.percent !== null ? ` · ${Math.round(job.percent)}%` : "");
    const message = document.createElement("span");
    message.className = "job-message";
    message.textContent = job.status === "completed" ? `Saved to ${job.message}` : job.message;
    message.title = message.textContent;
    details.append(status, message);
    if (job.status === "downloading" && job.percent !== null) {
      const progress = document.createElement("div");
      progress.className = "progress-track";
      const bar = document.createElement("div");
      bar.className = "progress-bar";
      bar.style.width = `${Math.min(100, Math.max(0, job.percent))}%`;
      progress.append(bar);
      body.append(progress);
    }
    body.append(details);
    row.append(badge, body);
    if (["queued", "starting", "downloading", "processing"].includes(job.status)) {
      const cancel = document.createElement("button");
      cancel.type = "button";
      cancel.className = "cancel-btn";
      cancel.title = "Cancel download";
      cancel.setAttribute("aria-label", `Cancel ${job.site} download`);
      cancel.innerHTML = icon("close", 17);
      cancel.addEventListener("click", () => cancelJob(job));
      row.append(cancel);
    } else {
      const endIcon = document.createElement("span");
      endIcon.className = "job-end";
      endIcon.innerHTML = icon(job.status === "completed" ? "check" : "more", 19);
      row.append(endIcon);
    }
    queueElement.append(row);
  }
}

async function launchNext(): Promise<void> {
  if (launching || activeId || !directory) return;
  const job = jobs.find((entry) => entry.status === "queued");
  if (!job) return;
  launching = true;
  activeId = job.id;
  job.status = "starting";
  job.message = "Getting ready…";
  renderQueue();
  try {
    await invoke("start_download", { id: job.id, url: job.url, directory });
  } catch (error) {
    job.status = "failed";
    job.message = String(error);
    activeId = null;
    renderQueue();
  } finally {
    launching = false;
    if (!activeId) void launchNext();
  }
}

async function cancelJob(job: Job): Promise<void> {
  if (job.status === "queued") {
    job.status = "cancelled";
    job.message = "Removed from queue.";
    renderQueue();
    return;
  }
  if (job.id === activeId) {
    job.message = "Cancelling…";
    renderQueue();
    try { await invoke("cancel_download", { id: job.id }); }
    catch (error) { showError(String(error)); }
  }
}

document.querySelector<HTMLButtonElement>("#choose-folder")?.addEventListener("click", async () => {
  try {
    const result = await open({ directory: true, multiple: false, title: "Choose download folder" });
    if (typeof result === "string") {
      directory = result;
      localStorage.setItem("framegrab.directory", directory);
      updateFolder();
      showError("");
      void launchNext();
    }
  } catch (error) { showError(`Could not open the folder picker: ${String(error)}`); }
});

document.querySelector<HTMLButtonElement>("#paste")!.addEventListener("click", async () => {
  try {
    urlInput.value = await navigator.clipboard.readText();
    urlInput.focus();
    showError("");
  } catch {
    showError("Clipboard access was denied. Paste into the field with Ctrl+V instead.");
  }
});

function addJob(): void {
  const url = urlInput.value.trim();
  const error = validateUrl(url);
  if (error) return showError(error);
  if (!directory) return showError("Choose a download folder first.");
  if (!isTauri()) return showError("Open the desktop app to download videos.");
  showError("");
  jobs.unshift({ id: crypto.randomUUID(), url, site: siteFor(url), status: "queued", percent: null, message: "Waiting in queue" });
  urlInput.value = "";
  renderQueue();
  void launchNext();
}
document.querySelector<HTMLButtonElement>("#add")!.addEventListener("click", addJob);
urlInput.addEventListener("keydown", (event) => { if (event.key === "Enter") addJob(); });

const installPanel = document.querySelector<HTMLDivElement>("#install-panel")!;
const installAction = document.querySelector<HTMLButtonElement>("#install-action")!;
const installFeedback = document.querySelector<HTMLDivElement>("#install-feedback")!;
function renderInstallAction(): void {
  installPanel.hidden = !installStatus.available;
  installAction.innerHTML = installStatus.installed
    ? `${icon("check", 16)}<span>Remove from apps</span>`
    : `${icon("download", 16)}<span>Add to applications</span>`;
  installAction.title = installStatus.installed
    ? "Remove the menu entry and installed copy; your downloads stay untouched."
    : "Copy the AppImage to ~/.local/opt/framegrab and add a menu launcher.";
}
installAction.addEventListener("click", async () => {
  if (installStatus.installed && !window.confirm("Remove Framegrab from your applications menu? Your downloaded videos will not be deleted.")) return;
  installAction.disabled = true;
  installFeedback.textContent = installStatus.installed ? "Removing…" : "Adding to applications…";
  try {
    await invoke(installStatus.installed ? "desktop_uninstall_app" : "desktop_install_app");
    installStatus = await invoke<InstallStatus>("desktop_install_status");
    installFeedback.textContent = installStatus.installed
      ? "Installed. Find Framegrab in your app menu."
      : "Removed. This portable copy still works.";
    renderInstallAction();
  } catch (error) {
    installFeedback.textContent = String(error);
  } finally {
    installAction.disabled = false;
  }
});

if (isTauri()) {
  void invoke<InstallStatus>("desktop_install_status").then((status) => {
    installStatus = status;
    renderInstallAction();
  }).catch(() => {});
  const onDownloadEvent = (payload: DownloadEvent) => {
    const job = jobs.find((entry) => entry.id === payload.id);
    if (!job) return;
    job.status = payload.status;
    job.percent = payload.percent;
    job.message = payload.message;
    renderQueue();
    if (["completed", "failed", "cancelled"].includes(payload.status)) {
      activeId = null;
      void launchNext();
    }
  };
  if (android) {
    void addPluginListener<DownloadEvent>("media", "download-event", onDownloadEvent)
      .catch((error) => showError(`Could not monitor downloads: ${String(error)}`));
  } else {
    void listen<DownloadEvent>("download-event", ({ payload }) => onDownloadEvent(payload))
      .catch((error) => showError(`Could not monitor downloads: ${String(error)}`));
  }
}
updateFolder();
renderQueue();
