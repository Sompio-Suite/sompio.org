import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { api, listen, openExternal, type BrowserPairAsk, type HealthReport, type KindKey, type LicenceStatus, type LockStatus, type ScanSummary, type TraceData, type VaultInTrace } from "./api";
import { n } from "./format";
import Icon, { type IconName } from "./Icon";
import Logo from "./Logo";
import Welcome from "./views/Welcome";
import Home from "./views/Home";
import Review from "./views/Review";
import Requests from "./views/Requests";
import Messages from "./views/Messages";
import Settings from "./views/Settings";
import Sources from "./views/Sources";
import Overview from "./views/Overview";
import GettingStarted from "./views/GettingStarted";
import { allDoneOnce, markStep, stepsDone } from "./steps";
import Licences from "./views/Licences";
import LockScreen from "./views/LockScreen";
import Security from "./views/Security";
import Vault from "./views/Vault";
import { DOCS_URL } from "./views/VaultTools";

const SOURCE_URL = "https://github.com/sompiohq/sompio-app";
import Samples from "./views/Samples";
import ReportBug from "./views/ReportBug";
import { BUY_URL, accessText, passEnding } from "./licence";
import { date } from "./format";
import { setThemeChoice, themeChoice, type ThemeChoice } from "./theme";
import { autoCheckEnabled, checkForUpdate, dueForCheck, type UpdateInfo } from "./updates";

/** Suite pages, then Clean's pages. */
export type View =
  | "overview" | "start" | "vault" | "licences" | "security" | "samples" | "report"
  | "home" | "review" | "requests" | "messages" | "sources" | "settings";
const CLEAN_VIEWS: View[] = ["home", "review", "requests", "messages", "sources", "settings"];
export interface Jump { channelId: string; messageId?: string }

export default function App() {
  const [ready, setReady] = useState<boolean | null>(null);
  const [demo, setDemo] = useState(false);
  // The day the pass reminder was last closed (it comes back the next day)
  const [snoozedOn, setSnoozedOn] = useState<string | null>(() => {
    try {
      return localStorage.getItem("sompio.passReminder");
    } catch {
      return null;
    }
  });
  const [view, setView] = useState<View>("overview");
  const [licence, setLicence] = useState<LicenceStatus | null>(null);
  const [lock, setLock] = useState<LockStatus | null>(null);
  const [idleMinutes, setIdleMinutes] = useState(() => {
    try {
      const v = localStorage.getItem("sompio.idleMinutes");
      return v == null ? 15 : Number(v);
    } catch {
      return 15;
    }
  });
  const [jump, setJump] = useState<Jump | null>(null);
  const [reviewKind, setReviewKind] = useState<KindKey | null>(null);
  const [scan, setScan] = useState<ScanSummary | null>(null);
  const [version, setVersion] = useState(0); // bumps after data changes so views reload
  const [connected, setConnected] = useState<string[]>([]);
  const [pairAsk, setPairAsk] = useState<BrowserPairAsk | null>(null);
  useEffect(() => {
    const a = listen<BrowserPairAsk>("browser-pair", setPairAsk);
    const b = listen<string>("browser-pair-done", (id) => setPairAsk((p) => (p?.id === id ? null : p)));
    return () => {
      a.then((f) => f());
      b.then((f) => f());
    };
  }, []);
  const checkConnections = useCallback(async () => {
    const [tg, mail] = await Promise.all([api.tgStatus().catch(() => null), api.emailStatus().catch(() => null)]);
    setConnected([...(tg?.connected ? ["Telegram"] : []), ...(mail?.connected ? ["email"] : [])]);
  }, []);

  const refresh = useCallback(async () => {
    const has = await api.hasData();
    setReady(has);
    if (has) {
      setScan(await api.scanSummary());
      setDemo(await api.isDemo());
    }
  }, []);

  useEffect(() => {
    api.lockStatus().then(setLock).catch(() => setLock({ set_up: true, unlocked: false, remembers_picture: false, hardware_keys: [], calendar_open: false }));
    api.licenceStatus().then(setLicence).catch(() => {});
  }, []);
  const unlocked = lock?.unlocked ?? false;

  // Updates: checked quietly at start and every few hours, if the person hasn't turned it off.
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [installing, setInstalling] = useState<number | null | "error">(null);
  // A new version found by the automatic check pops up once; "Later" keeps it in the sidebar only
  const [updatePopup, setUpdatePopup] = useState(false);
  // The version the person said "Later" to: not shown again until the next start or a newer version.
  const laterFor = useRef<string | null>(null);
  const [theme, setTheme] = useState<ThemeChoice>(themeChoice);
  const [menu, setMenu] = useState<"settings" | null>(null);
  // Close the small menus on a click elsewhere or Escape
  useEffect(() => {
    if (!menu) return;
    const close = (e: Event) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !(e.target as Element).closest(".menu-wrap")) setMenu(null);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => { document.removeEventListener("mousedown", close); document.removeEventListener("keydown", close); };
  }, [menu]);
  // Look for an update every time Sompio opens, then every two hours while it stays open. A check
  // that failed (no internet yet) is tried again every 10 minutes. "Later" hides the notice until
  // the next start; it comes back then.
  useEffect(() => {
    const run = (always: boolean) => {
      if (autoCheckEnabled() && (always || dueForCheck()))
        checkForUpdate()
          .then((u) => {
            setUpdate(u);
            if (u && u.version !== laterFor.current) setUpdatePopup(true);
          })
          .catch(() => {});
    };
    run(true);
    const t = window.setInterval(() => run(false), 600_000);
    return () => window.clearInterval(t);
  }, []);
  const installUpdate = async () => {
    if (!update) return;
    setInstalling(0);
    try {
      await update.install((p) => setInstalling(p));
    } catch {
      setInstalling("error");
    }
  };
  useEffect(() => {
    if (!unlocked) return;
    refresh();
    checkConnections();
  }, [unlocked, refresh, checkConnections]);

  // What Overview and the tabs count: Vault's health, its passwords that turned up in Trace, and
  // the requests to companies. Read again whenever Overview or Getting started is opened.
  const [health, setHealth] = useState<HealthReport | null>(null);
  const [inChats, setInChats] = useState<VaultInTrace[]>([]);
  const [traceData, setTraceData] = useState<TraceData | null>(null);
  const [, setStepTick] = useState(0);
  useEffect(() => {
    const on = () => setStepTick((t) => t + 1);
    window.addEventListener("sompio-steps", on);
    return () => window.removeEventListener("sompio-steps", on);
  }, []);
  const atOverview = view === "overview" || view === "start";
  useEffect(() => {
    if (!unlocked) return;
    api.vaultHealth().then((h) => { setHealth(h); if (h.checked > 0) markStep("secured"); }).catch(() => setHealth(null));
    api.vaultInTrace().then(setInChats).catch(() => setInChats([]));
    api.traceData().then(setTraceData).catch(() => setTraceData(null));
  }, [unlocked, atOverview, version]);

  const lockNow = useCallback(async () => {
    await api.lockLock().catch(() => {});
    setLock(await api.lockStatus());
    setReady(null);
    setScan(null);
    setHealth(null);
    setInChats([]);
    setTraceData(null);
    setVersion((v) => v + 1);
  }, []);
  // "Lock Sompio" in the tray menu
  useEffect(() => {
    const un = listen("locked", () => lockNow());
    return () => { un.then((f) => f()); };
  }, [lockNow]);

  // Lock after a while without use
  useEffect(() => {
    try {
      localStorage.setItem("sompio.idleMinutes", String(idleMinutes));
    } catch {
      /* not remembered, fine */
    }
    if (!unlocked || idleMinutes <= 0) return;
    let last = Date.now();
    const seen = () => { last = Date.now(); };
    const events = ["mousemove", "keydown", "mousedown", "wheel", "touchstart"];
    events.forEach((e) => window.addEventListener(e, seen, { passive: true }));
    const timer = window.setInterval(() => {
      if (Date.now() - last > idleMinutes * 60_000) lockNow();
    }, 20_000);
    return () => {
      events.forEach((e) => window.removeEventListener(e, seen));
      window.clearInterval(timer);
    };
  }, [unlocked, idleMinutes, lockNow]);

  const openChannel = (j: Jump) => {
    setJump(j);
    setView("messages");
  };
  const openReview = (kind: KindKey | null) => {
    setReviewKind(kind);
    setView("review");
  };

  const leave = async () => {
    await api.wipeData();
    setReady(false);
    setDemo(false);
    setScan(null);
    setVersion((v) => v + 1);
  };

  // Getting started counts its four steps and moves under ? once all are done
  const found = !!ready && !!scan && scan.scanned > 0;
  const steps = stepsDone(found, found && scan!.open === 0);
  const stepCount = Object.values(steps).filter(Boolean).length;
  useEffect(() => { if (stepCount === 4) markStep("started"); }, [stepCount]);
  const started = stepCount === 4 || allDoneOnce();
  const vaultRisk = inChats.length + (health?.items.filter((i) => i.issues.some((x) => x.kind === "weak" || x.kind === "reused")).length ?? 0);

  if (!lock) return null;
  // Vault and Trace sit behind the lock, set up the first time one of them is opened. Overview,
  // Activate a product, Security and the rest open without it.
  const gate = (reason?: string) => (
    <LockScreen key={`${lock.set_up}-${reason}`} status={lock} reason={reason} onUnlocked={async () => setLock(await api.lockStatus())} />
  );

  const toReview = scan ? scan.open : 0;
  // A paid pass running out soon: a reminder at the top, at most once a day once closed
  const ending = passEnding(licence);
  const today = new Date().toDateString();
  const remindShown = snoozedOn !== today;
  const snoozePass = () => {
    setSnoozedOn(today);
    try {
      localStorage.setItem("sompio.passReminder", today);
    } catch {
      /* still hidden for now */
    }
  };
  const cleanAccess = licence?.apps.find((a) => a.app === "clean");
  const cleanLocked = licence != null && !cleanAccess?.active;
  const inClean = CLEAN_VIEWS.includes(view);
  const vaultLocked = licence != null && !licence.apps.find((a) => a.app === "vault")?.active;

  const navButton = (id: View, label: string, icon: string, extra?: ReactNode, sub = false) => (
    <button
      key={id}
      className={`nav${sub ? " sub" : ""}`}
      aria-current={view === id || (id === "home" && inClean) ? "page" : undefined}
      aria-label={label}
      onClick={() => (id === "review" || (id === "home" && ready && scan?.total_findings) ? openReview(null) : setView(id))}
    >
      <Icon name={icon as IconName} />
      <span className="label">{label}</span>
      {extra}
    </button>
  );

  const cleanPage = () => {
    if (!ready) {
      return (
        <Welcome
          onReady={async (summary) => {
            setScan(summary);
            setDemo(await api.isDemo());
            setReady(true);
            setVersion((v) => v + 1);
            setView("review");
          }}
        />
      );
    }
    switch (view) {
      case "home":
        return <Home key={version} scan={scan} onReview={openReview} onGo={setView} onOpen={openChannel} />;
      case "review":
        return <Review key={`${version}-${reviewKind}`} scan={scan} demo={demo} initialKind={reviewKind} onScan={setScan} onOpen={openChannel} />;
      case "requests":
        return <Requests key={version} readOnly={cleanLocked} onGo={setView} />;
      case "messages":
        return <Messages key={version} jump={jump} onJumpHandled={() => setJump(null)} />;
      case "sources":
        return (
          <Sources
            demo={demo}
            onScan={setScan}
            onChanged={async () => {
              setVersion((v) => v + 1);
              checkConnections();
              const has = await api.hasData();
              if (!has) return leave();
              setDemo(await api.isDemo());
            }}
          />
        );
      default:
        return <Settings scan={scan} demo={demo} onScan={setScan} onLeave={leave} />;
    }
  };

  return (
    <div className="shell">
      <header className="topbar">
        <button className={`brand${view === "overview" ? " on" : ""}`} aria-label="Overview" title="Overview" aria-current={view === "overview" ? "page" : undefined} onClick={() => setView("overview")}>
          <Logo size={24} />
          <span className="brand-name">sompio</span>
        </button>
        <nav className="tabs" aria-label="Sections">
          {!started && navButton("start", "Getting started", "check", !unlocked && lock.set_up ? undefined : <span className="steps-badge" aria-label={`${stepCount} of 4 steps done`}>{stepCount}/4</span>)}
          {navButton("home", "Trace", "review", cleanLocked || !unlocked ? <Icon name="lock" size={13} /> : toReview > 0 && !inClean ? <span className="count" aria-label={`${toReview} to review`}>{n(toReview)}</span> : undefined)}
          {navButton("vault", "Vault", "key", vaultLocked || !unlocked ? <Icon name="lock" size={13} /> : vaultRisk > 0 && view !== "vault" ? <span className="count" aria-label={`${vaultRisk} passwords at risk`}>{n(vaultRisk)}</span> : undefined)}
          <span className="tab-sep" aria-hidden="true" />
          <button className="nav ext" title="See Sompio's code on GitHub (opens in your browser)" aria-label="GitHub" onClick={() => openExternal(SOURCE_URL).catch(() => {})}>
            <Icon name="code" /><span className="label">GitHub</span>
          </button>
          <button className="nav ext" title="Documentation: how Sompio works, in plain words (opens in your browser)" aria-label="Documentation" onClick={() => openExternal(DOCS_URL).catch(() => {})}>
            <Icon name="book" /><span className="label">Docs</span>
          </button>
        </nav>
        <div className="bar-tools">
          {update && (
            <button className="pill accent" onClick={() => setUpdatePopup(true)} title={`Update to Sompio ${update.version}. Your data stays as it is.`}>
              <Icon name="refresh" size={14} />
              {typeof installing === "number" ? `Updating${installing ? ` ${installing}%` : "…"}` : "Update"}
            </button>
          )}
          {lock.set_up ? (
            <button
              className="pill"
              onClick={() => (unlocked ? lockNow() : setView("vault"))}
              title={`${unlocked ? "Vault and Trace are open." : "Vault and Trace are locked."} ${connected.length ? `${connected.join(" and ")} connected to Trace.` : "Sompio works offline: your data never leaves this computer."}`}
            >
              <Icon name="lock" size={14} />
              <strong>{unlocked ? "Lock" : "Unlock"}</strong>
            </button>
          ) : (
            <button className="pill" onClick={() => setView("vault")} title="Vault and Trace ask for a lock the first time you open them">
              <Icon name="lock" size={14} /><strong>Set up the lock</strong>
            </button>
          )}
          {/* Its own button during the beta, so testers always find it */}
          <button className={`pill${view === "report" ? " on" : ""}`} onClick={() => setView("report")}>
            <Icon name="bug" size={14} />Report a bug
          </button>
          {/* Once Getting started is done, it lives behind ? */}
          {started && (
            <button className={`icon-btn${view === "start" ? " on" : ""}`} aria-label="Getting started" title="Getting started" onClick={() => setView("start")}>
              <span className="q">?</span>
            </button>
          )}
          <div className="menu-wrap">
            <button className={`icon-btn${view === "security" || view === "licences" || view === "samples" ? " on" : ""}`} aria-label="Settings" aria-haspopup="menu" aria-expanded={menu === "settings"} title="Settings" onClick={() => setMenu(menu === "settings" ? null : "settings")}>
              <Icon name="settings" size={18} />
            </button>
            {menu === "settings" && (
              <div className="dropdown" role="menu">
                <button role="menuitem" onClick={() => { setMenu(null); setView("security"); }}><Icon name="lock" size={16} />Security and updates</button>
                <button role="menuitem" onClick={() => { setMenu(null); setView("licences"); }}><Icon name="ticket" size={16} />Activate a product</button>
                <button role="menuitem" onClick={() => { setMenu(null); setView("samples"); }}><Icon name="folder" size={16} />Sample data for testing</button>
                <div className="menu-theme">
                  <span className="small muted" id="theme-label">Appearance</span>
                  <div className="segmented" role="radiogroup" aria-labelledby="theme-label">
                    {([["light", "Light"], ["dark", "Dark"], ["system", "System"]] as [ThemeChoice, string][]).map(([id, label]) => (
                      <button key={id} role="radio" aria-checked={theme === id} aria-selected={theme === id}
                        title={id === "system" ? "Match your computer's setting" : undefined}
                        onClick={() => { setTheme(id); setThemeChoice(id); }}>{label}</button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </header>
      <main>
        {ending && remindShown && (
          <div className="demo-bar pass-bar" role="status">
            <span className="pass-bar-text">
              <Icon name="clock" size={16} />
              <span>
                Your Sompio pass ends {ending.daysLeft === 1 ? "tomorrow" : `in ${ending.daysLeft} days`}, on {date(ending.endsAt)}.
                After that, everything stays readable, but you can't add or change things.
              </span>
            </span>
            <span className="pass-bar-actions">
              <button className="primary" onClick={() => openExternal(BUY_URL).catch(() => {})}>Get a new key</button>
              <button onClick={() => setView("licences")}>Add a key</button>
              <button className="icon-btn" aria-label="Remind me tomorrow" title="Remind me tomorrow" onClick={snoozePass}><Icon name="x" size={14} /></button>
            </span>
          </div>
        )}
        {inClean && unlocked && cleanLocked && (
          <div className="demo-bar locked-bar" role="status">
            <span>
              <Icon name="lock" size={16} /> Trace is read-only: {accessText(cleanAccess, licence?.now ?? Date.now()).text.toLowerCase()}. You can look at
              everything, but not import, scan or delete.
            </span>
            <span className="pass-bar-actions">
              <button onClick={() => openExternal(BUY_URL).catch(() => {})}>Get a key</button>
              <button onClick={() => setView("licences")}>Add a key</button>
            </span>
          </div>
        )}
        {inClean && unlocked && demo && (
          <div className="demo-bar" role="status">
            <span>You're looking at sample data. Nothing here is yours.</span>
            <button onClick={leave}>Use my own data</button>
          </div>
        )}
        {inClean && unlocked && ready !== null && (
          <div className="app-head">
            <div className="app-title">
              <h1>Trace</h1>
              <span className="muted">Find and remove what you've shared over the years.</span>
            </div>
            {ready && (
              <div className="segmented" role="tablist" aria-label="Trace">
                {([["review", "Findings"], ["requests", "Requests"], ["home", "Summary"], ["messages", "Messages"], ["sources", "Sources"], ["settings", "Settings"]] as [View, string][]).map(([id, label]) => (
                  <button key={id} role="tab" aria-selected={view === id} onClick={() => (id === "review" ? openReview(null) : setView(id))}>
                    {label}
                    {id === "review" && toReview > 0 && <span className="count">{n(toReview)}</span>}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
        {view === "overview" && (unlocked || !lock.set_up
          ? <Overview licence={licence} scan={scan} traced={found} health={health} inChats={inChats} trace={traceData} onGo={setView} onReview={() => openReview(null)} />
          : gate("Overview"))}
        {view === "start" && <GettingStarted licence={licence} scan={scan} cleanReady={!!ready} learned={steps.learn} secured={steps.secure} onGo={setView} />}
        {view === "licences" && <Licences licence={licence} onChanged={setLicence} />}
        {view === "report" && <ReportBug />}
        {view === "security" && (
          <Security
            status={lock}
            idleMinutes={idleMinutes}
            onIdleMinutes={setIdleMinutes}
            onLockNow={lockNow}
            onChanged={async () => setLock(await api.lockStatus())}
            onUpdateFound={setUpdate}
            onSetUp={() => setView("vault")}
          />
        )}
        {view === "vault" && (unlocked ? <Vault readOnly={vaultLocked} onAddKey={() => setView("licences")} /> : gate("Vault"))}
        {view === "samples" && !unlocked && gate()}
        {view === "samples" && unlocked && (
          <Samples
            cleanHasData={!!ready && !demo}
            onCleanDemo={async () => {
              await api.loadDemo();
              // Check the sample straight away, so what it holds shows up in Trace
              await api.runScan(await api.watchTerms());
              await refresh();
              setVersion((v) => v + 1);
              setView("review");
            }}
            onGo={setView}
          />
        )}
        {inClean && (!unlocked ? gate("Trace") : ready === null ? null : cleanPage())}
      </main>
      {update && updatePopup && !pairAsk && (
        <div className="update-card" role="status" aria-labelledby="update-title">
          <div className="update-card-head">
            <Icon name="refresh" size={16} />
            <strong id="update-title">Sompio {update.version} is available</strong>
            <button className="quiet x" aria-label="Later" disabled={typeof installing === "number"} onClick={() => { laterFor.current = update.version; setUpdatePopup(false); }}>×</button>
          </div>
          {update.notes.trim() && (
            <details className="update-notes-fold">
              <summary className="small">What's new</summary>
              <ul className="update-notes small">
                {update.notes.split("\n").map((l) => l.trim()).filter(Boolean).map((l, i) => <li key={i}>{l.replace(/^[-•]\s*/, "")}</li>)}
              </ul>
            </details>
          )}
          {typeof installing === "number" && <p className="small">Downloading{installing ? ` ${installing}%` : "…"}. Sompio restarts by itself.</p>}
          {installing === "error" && <p className="error small" role="alert">The update didn't install. Try again later, or from Security.</p>}
          <div className="toolbar">
            <button className="primary" disabled={typeof installing === "number"} onClick={installUpdate}>
              {typeof installing === "number" ? "Updating…" : "Update now"}
            </button>
            <button className="quiet" disabled={typeof installing === "number"} onClick={() => { laterFor.current = update.version; setUpdatePopup(false); }}>Later</button>
          </div>
        </div>
      )}
      {pairAsk && (
        <div className="modal-backdrop" role="presentation">
          <div className="modal pair-ask" role="dialog" aria-modal="true" aria-labelledby="pair-title">
            <h2 id="pair-title">Allow {pairAsk.browser} to use your Vault?</h2>
            <p>The Sompio extension in {pairAsk.browser} wants to fill and save logins. Check that it shows the same code:</p>
            <div className="pair-code" aria-label="Pairing code">{pairAsk.code}</div>
            <p className="small muted">If you didn't just click "Connect to Sompio" in your browser, don't allow it. You can remove a browser any time in Vault, Settings.</p>
            <div className="guide-actions">
              <button className="primary" onClick={() => { api.browserAnswer(pairAsk.id, true); setPairAsk(null); }}>Allow</button>
              <button onClick={() => { api.browserAnswer(pairAsk.id, false); setPairAsk(null); }}>Don't allow</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
