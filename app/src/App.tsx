import { useCallback, useEffect, useState, type ReactNode } from "react";
import { api, listen, openExternal, type BrowserPairAsk, type KindKey, type LicenceStatus, type LockStatus, type ScanSummary } from "./api";
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
import Licences from "./views/Licences";
import LockScreen from "./views/LockScreen";
import Security from "./views/Security";
import Vault from "./views/Vault";
import { DOCS_URL } from "./views/VaultTools";
import Samples from "./views/Samples";
import ReportBug from "./views/ReportBug";
import { BUY_URL, SUITE, accessText, passEnding } from "./licence";
import { date } from "./format";
import { autoCheckEnabled, checkForUpdate, dismissUpdate, dueForCheck, updateDismissed, type UpdateInfo } from "./updates";

/** Suite pages, then Clean's pages. */
export type View =
  | "overview" | "vault" | "licences" | "security" | "samples" | "report"
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
  const [menu, setMenu] = useState<"help" | "settings" | null>(null);
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
  useEffect(() => {
    const run = () => {
      if (autoCheckEnabled() && dueForCheck())
        checkForUpdate()
          .then((u) => {
            setUpdate(u);
            if (u && !updateDismissed(u.version)) setUpdatePopup(true);
          })
          .catch(() => {});
    };
    run();
    const t = window.setInterval(run, 3_600_000);
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

  const lockNow = useCallback(async () => {
    await api.lockLock().catch(() => {});
    setLock(await api.lockStatus());
    setReady(null);
    setScan(null);
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
        <div className="brand">
          <Logo size={24} />
          <span className="brand-name">sompio</span>
        </div>
        <nav className="tabs" aria-label="Sections">
          {navButton("overview", "Overview", "grid")}
          {navButton("vault", "Vault", "key", vaultLocked || !unlocked ? <Icon name="lock" size={13} /> : undefined)}
          {navButton("home", "Trace", "review", cleanLocked || !unlocked ? <Icon name="lock" size={13} /> : toReview > 0 && !inClean ? <span className="count" aria-label={`${toReview} to review`}>{n(toReview)}</span> : undefined)}
          {SUITE.filter((a) => !a.ready).map((a) => navButton(a.key as View, a.name, a.icon, <span className="soon-dot" title={a.when} />))}
        </nav>
        <div className="bar-tools">
          {update && (
            <button className="pill accent" onClick={() => setUpdatePopup(true)} title="Your data stays as it is">
              <Icon name="refresh" size={14} />
              {typeof installing === "number" ? `Updating${installing ? ` ${installing}%` : "…"}` : `Update to ${update.version}`}
            </button>
          )}
          {lock.set_up ? (
            <button
              className="pill"
              onClick={() => (unlocked ? lockNow() : setView("vault"))}
              title={`${unlocked ? "Vault and Trace are open." : "Vault and Trace are locked."} ${connected.length ? `${connected.join(" and ")} connected to Trace.` : "Sompio works offline: your data never leaves this computer."}`}
            >
              <Icon name="lock" size={14} />
              <span className="muted">{unlocked ? "Open" : "Locked"} ·</span> <strong>{unlocked ? "Lock" : "Unlock"}</strong>
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
          <div className="menu-wrap">
            <button className="icon-btn" aria-label="Help" aria-haspopup="menu" aria-expanded={menu === "help"} title="Help" onClick={() => setMenu(menu === "help" ? null : "help")}>
              <span className="q">?</span>
            </button>
            {menu === "help" && (
              <div className="dropdown" role="menu">
                <button role="menuitem" onClick={() => { setMenu(null); openExternal(DOCS_URL).catch(() => {}); }}><Icon name="book" size={16} />Documentation<Icon name="external" size={12} /></button>
                <button role="menuitem" onClick={() => { setMenu(null); setView("samples"); }}><Icon name="sparkle" size={16} />Samples</button>
              </div>
            )}
          </div>
          <div className="menu-wrap">
            <button className={`icon-btn${view === "security" || view === "licences" ? " on" : ""}`} aria-label="Settings" aria-haspopup="menu" aria-expanded={menu === "settings"} title="Settings" onClick={() => setMenu(menu === "settings" ? null : "settings")}>
              <Icon name="settings" size={18} />
            </button>
            {menu === "settings" && (
              <div className="dropdown" role="menu">
                <button role="menuitem" onClick={() => { setMenu(null); setView("security"); }}><Icon name="lock" size={16} />Security and updates</button>
                <button role="menuitem" onClick={() => { setMenu(null); setView("licences"); }}><Icon name="ticket" size={16} />Activate a product</button>
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
        {view === "overview" && <Overview licence={licence} scan={scan} cleanReady={!!ready} onGo={setView} />}
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
        <div className="modal-backdrop" role="presentation">
          <div className="modal update-ask" role="dialog" aria-modal="true" aria-labelledby="update-title">
            <h2 id="update-title">Sompio {update.version} is ready</h2>
            {update.notes.trim() && (
              <ul className="update-notes">
                {update.notes.split("\n").map((l) => l.trim()).filter(Boolean).map((l, i) => <li key={i}>{l.replace(/^[-•]\s*/, "")}</li>)}
              </ul>
            )}
            {installing === null && <p className="small muted">It takes about a minute. Your data stays as it is, and Sompio restarts by itself.</p>}
            {typeof installing === "number" && <p className="small">Downloading{installing ? ` ${installing}%` : "…"}. Sompio restarts by itself.</p>}
            {installing === "error" && <p className="error" role="alert">The update didn't install. Try again later, or from Security.</p>}
            <div className="guide-actions">
              <button className="primary" disabled={typeof installing === "number"} onClick={installUpdate}>
                {typeof installing === "number" ? "Updating…" : "Update now"}
              </button>
              <button disabled={typeof installing === "number"} onClick={() => { dismissUpdate(update.version); setUpdatePopup(false); }}>Later</button>
            </div>
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
