import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  api, copyText, listen, openLabel, openLink as openUrl, pickSavePath, platformName, severityLabel,
  type Conversation, type FindingView, type GroupInfo, type KindKey, type Platform, type Previous, type ScanSummary, type SourceView, type Status, type StatusTab, type TraceData,
} from "../api";
import type { Jump, View } from "../App";
import ServiceIcon, { mailName, mailProvider } from "../ServiceIcon";
import { dateTime, n, plural } from "../format";
import Icon from "../Icon";
import { usePaged } from "../usePaged";

interface Props {
  scan: ScanSummary | null;
  demo: boolean;
  initialKind: KindKey | null;
  onScan: (s: ScanSummary) => void;
  onOpen: (j: Jump) => void;
  /** The other Trace pages: adding a source, requests, settings, messages */
  onGo: (v: View) => void;
}

interface Toast { text: string; undo?: () => void }

/** One message to delete on its platform. */
interface Target { id: number; message_id: string; platform: Platform }

/** The guided clean-up: one finding at a time, with the right way to remove it. */
interface Guide { items: FindingView[]; i: number; deleted: number; ok: number; skipped: number; busy?: boolean; error?: string }

/** A run of deletions on a connected platform, one message at a time. */
interface Batch { label: string; done: number; total: number; failed: number }

/** Where you were: the type you had open and the row you were on. */
const WHERE_KEY = "sompio.review.where";
const loadWhere = (): { kind: KindKey | null; focus: number } => {
  try {
    const w = JSON.parse(localStorage.getItem(WHERE_KEY) ?? "null");
    return { kind: w?.kind ?? null, focus: Number.isInteger(w?.focus) ? w.focus : 0 };
  } catch {
    return { kind: null, focus: 0 };
  }
};

const loadReveal = () => {
  try {
    return localStorage.getItem("sompio.review.reveal") !== "0";
  } catch {
    return true;
  }
};

const STATUS_DONE_TEXT: Record<Status, string> = {
  deleted: "Marked as deleted",
  ok: "Marked as not a problem",
  later: "Saved for later",
};

const NOUN: Record<KindKey, string> = {
  credential: "password or token",
  card: "card number",
  iban: "account number",
  watch_term: "term",
  address: "address",
  phone: "phone number",
  email: "email address",
  ip_address: "IP address",
  postcode: "postcode",
  crypto_wallet: "wallet",
  sensitive_file: "file",
  id_number: "ID number",
  passport: "document number",
  backup_code: "backup code",
  birth_date: "date of birth",
  number_plate: "number plate",
  location: "location",
};

export default function Review({ scan, demo, initialKind, onScan, onOpen, onGo }: Props) {
  const [tab, setTab] = useState<StatusTab>("open");
  const [where] = useState(loadWhere);
  const [kind, setKind] = useState<KindKey | null>(initialKind ?? where.kind);
  const [platform, setPlatform] = useState<Platform | null>(null);
  const [platforms, setPlatforms] = useState<Platform[]>([]);
  const [revealAll, setRevealAll] = useState(loadReveal);
  const [showTerms, setShowTerms] = useState(false);
  const [revealed, setRevealed] = useState<Map<number, FindingView>>(new Map());
  const [focus, setFocus] = useState(initialKind ? 0 : where.focus);
  const [listVersion, setListVersion] = useState(0);
  const [toast, setToast] = useState<Toast | null>(null);
  const [scanning, setScanning] = useState<[number, number] | null>(null);
  const [terms, setTerms] = useState<string[]>([]);
  const [scannedTerms, setScannedTerms] = useState<string[]>([]);
  const [draft, setDraft] = useState("");
  const [confirmRecycle, setConfirmRecycle] = useState(false);
  const [tgConnected, setTgConnected] = useState(false);
  const [emailConnected, setEmailConnected] = useState(false);
  // Each value once, with the conversations it's in: the same number in 6 replies is one row.
  const grouped = true;
  const [sources, setSources] = useState<SourceView[]>([]);
  const [mailAccount, setMailAccount] = useState<string | null>(null);
  const [trace, setTrace] = useState<TraceData | null>(null);
  const [allConvs, setAllConvs] = useState(false);
  const [sameAs, setSameAs] = useState<FindingView | null>(null);
  const [selected, setSelected] = useState<Map<number, FindingView>>(new Map());
  const [anchor, setAnchor] = useState<number | null>(null);
  const [group, setGroup] = useState<GroupInfo | null>(null);
  const [batch, setBatch] = useState<Batch | null>(null);
  const stopBatch = useRef(false);
  const [confirmGroupDelete, setConfirmGroupDelete] = useState<Platform | null>(null);
  const [guide, setGuide] = useState<Guide | null>(null);

  useEffect(() => {
    api.watchTerms().then((t) => {
      setTerms(t);
      setScannedTerms(t);
    });
    api.traceData().then(setTrace).catch(() => {});
    api.sources().then((s) => {
      setSources(s);
      setPlatforms(s.map((x) => x.platform));
      // Only ask about the Telegram connection when there is Telegram data to act on.
      if (s.some((x) => x.platform === "telegram")) api.tgStatus().then((t) => setTgConnected(t.connected)).catch(() => {});
      if (s.some((x) => x.platform === "email"))
        api.emailStatus().then((t) => {
          setEmailConnected(t.connected);
          setMailAccount(t.provider ?? null);
        }).catch(() => {});
    });
    const un = listen<[number, number]>("scan-progress", setScanning);
    // Data that was never checked: check it now, so the findings are always there
    if (scan && scan.scanned_at_ms == null) api.watchTerms().then((t) => runScanRef.current(t));
    return () => void un.then((f) => f());
  }, []);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 6000);
    return () => clearTimeout(t);
  }, [toast]);

  const scanned = scan?.scanned_at_ms != null;
  const groupList = grouped && !sameAs;
  const listKey = scanned ? `${tab}|${kind}|${platform}|${revealAll}|${groupList}|${sameAs?.id}|${listVersion}` : null;
  const { total, items, ensure, theirs } = usePaged<FindingView>(listKey, (o, l) =>
    api.findings(kind, platform, tab, o, l, revealAll, groupList, sameAs?.id ?? null),
  );
  // Someone else's findings come last, under their own heading, folded away until opened.
  const [showTheirs, setShowTheirs] = useState(false);
  const mine = (total ?? 0) - theirs;
  const theirsOpen = showTheirs || mine === 0;
  const hasHead = theirs > 0;
  const reachable = theirsOpen ? (total ?? 0) : mine;
  const toRow = (i: number) => (hasHead && i >= mine ? i + 1 : i);
  const toItem = (r: number) => (hasHead && r >= mine ? (r === mine ? -1 : r - 1) : r);
  useEffect(() => {
    setSelected(new Map());
    setAnchor(null);
  }, [tab, kind, platform, grouped, sameAs, listVersion]);
  useEffect(() => {
    try {
      localStorage.setItem("sompio.review.reveal", revealAll ? "1" : "0");
    } catch {
      /* not available: the setting just isn't remembered */
    }
  }, [revealAll]);
  // Remember where you were, so Trace opens there next time.
  useEffect(() => {
    if (tab !== "open" || sameAs) return;
    try {
      localStorage.setItem(WHERE_KEY, JSON.stringify({ kind, focus }));
    } catch {
      /* fine */
    }
  }, [kind, focus, tab, sameAs]);

  // If the selected type has nothing left (say, after "never flag this again"), go back to everything.
  useEffect(() => {
    if (kind && scan && !scan.by_kind.some((k) => k.kind === kind && k.count > 0)) setKind(null);
  }, [scan, kind]);

  useEffect(() => {
    if (total != null) setFocus((f) => Math.max(0, Math.min(f, reachable - 1)));
  }, [total, reachable]);
  useEffect(() => {
    setConfirmRecycle(false);
    setConfirmGroupDelete(null);
  }, [focus, listVersion]);

  const parentRef = useRef<HTMLDivElement>(null);
  const v = useVirtualizer({
    count: reachable + (hasHead ? 1 : 0),
    getScrollElement: () => parentRef.current,
    estimateSize: () => 30,
    overscan: 8,
  });
  const virtualItems = v.getVirtualItems();
  useEffect(() => {
    virtualItems.forEach((r) => {
      const i = toItem(r.index);
      if (i >= 0 && !items[i]) ensure(i);
    });
  }, [virtualItems, items, ensure]);

  const reload = () => setListVersion((x) => x + 1);

  // Where else the focused item's value appears.
  const focusedId = items[focus]?.id;
  const focusedRepeats = items[focus]?.repeats ?? 0;
  useEffect(() => {
    setGroup(null);
    if (focusedId == null || focusedRepeats < 2) return;
    let live = true;
    api.groupInfo(focusedId, tab).then((g) => live && setGroup(g)).catch(() => {});
    return () => { live = false; };
  }, [focusedId, focusedRepeats, tab, listVersion]);
  useEffect(() => setAllConvs(false), [focusedId]);

  /** One decision for many findings, with Undo. */
  const applyMany = useCallback(
    async (ids: number[], status: Status, text: string) => {
      const [previous, summary] = await api.setStatusMany(ids, status);
      onScan(summary);
      reload();
      setToast({
        text,
        undo: async () => {
          onScan(await api.restoreStatuses(previous as Previous));
          reload();
          setToast(null);
        },
      });
    },
    [onScan],
  );

  const decide = useCallback(
    async (f: FindingView, status: Status | null) => {
      const previous = f.status;
      onScan(await api.setStatus(f.id, status));
      reload();
      if (status) {
        setToast({
          text: STATUS_DONE_TEXT[status],
          undo: async () => {
            onScan(await api.setStatus(f.id, previous));
            reload();
            setToast(null);
          },
        });
      } else {
        setToast({ text: "Moved back to To review" });
      }
    },
    [onScan],
  );

  /** In the grouped list, a decision covers every place the value appears. */
  const groupIds = (f: FindingView) => (groupList && group && f.repeats > 1 && group.ids.includes(f.id) ? group.ids : null);
  const decideHere = useCallback(
    (f: FindingView, status: Status) => {
      const ids = groupIds(f);
      if (ids) return applyMany(ids, status, `${STATUS_DONE_TEXT[status]}: all ${n(ids.length)} places`);
      return decide(f, status);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [groupList, group, applyMany, decide],
  );

  const ignoreEverywhere = async (f: FindingView) => {
    const [count, summary] = await api.ignoreEverywhere(f.id);
    onScan(summary);
    reload();
    setToast({ text: `Won't flag this again. Cleared ${plural(count, "place", "places")}. You can undo this in Settings.` });
  };

  const toggleReveal = useCallback(async (f: FindingView) => {
    if (revealed.has(f.id)) {
      const next = new Map(revealed);
      next.delete(f.id);
      setRevealed(next);
      return;
    }
    const full = await api.revealFinding(f.id);
    if (full) setRevealed(new Map(revealed).set(f.id, full));
  }, [revealed]);

  /** Takes you to the item: its web page, or the file in File Explorer. */
  const goTo = useCallback(async (f: FindingView) => {
    try {
      if (f.platform === "pc") {
        await api.showInFolder(f.id);
        setToast({ text: "Opened the folder with the file selected." });
      } else if (f.link) {
        // Pages and emails don't show what Sompio found, so copy it: Ctrl+F and paste finds it there.
        const full = await api.revealFinding(f.id).catch(() => null);
        const value = (full ?? f).matched.trim();
        if (value) await copyText(value).catch(() => {});
        await openUrl(f.link);
        const find = navigator.platform.toLowerCase().includes("mac") ? "⌘F" : "Ctrl+F";
        setToast({
          text: value
            ? `Opened in your browser, and "${value.length > 40 ? value.slice(0, 40) + "…" : value}" is copied: press ${find} there and paste to find it. Delete it there, then mark it as deleted here.`
            : "Opened in your browser. Delete it there, then mark it as deleted here.",
        });
      } else {
        setToast({ text: `There's no web link for this one. Open ${platformName[f.platform]} and find it by its date.` });
      }
    } catch (e) {
      setToast({ text: String(e) });
    }
  }, []);
  const copyLink = useCallback(async (f: FindingView) => {
    const value = f.link ?? f.file_path;
    if (!value) return;
    await copyText(value);
    setToast({ text: f.link ? "Link copied." : "File path copied." });
  }, []);
  /** Deleting on the platform itself, for connected accounts. */
  const liveDelete = (f: FindingView) =>
    f.status
      ? null
      : f.platform === "telegram" && tgConnected
        ? { ask: "Delete from Telegram…", confirm: "Yes, delete it for everyone", run: api.tgDelete, done: "Deleted from Telegram for everyone in that chat." }
        : f.platform === "email" && emailConnected
          ? { ask: "Move to Trash…", confirm: "Yes, move it to Trash", run: api.emailDelete, done: "Moved to your mailbox's Trash. You can restore it from there." }
          : null;
  const liveFor = (p: Platform) =>
    p === "telegram" && tgConnected
      ? { run: api.tgDelete, noun: "Telegram message", verb: "Deleting from Telegram", ask: (k: number) => `Delete ${plural(k, "message", "messages")} from Telegram…` }
      : p === "email" && emailConnected
        ? { run: api.emailDelete, noun: "email", verb: "Moving emails to Trash", ask: (k: number) => `Move ${plural(k, "email", "emails")} to Trash…` }
        : null;

  /** Deletes messages on their platform one at a time, with a pause so the platform doesn't block the account. */
  const runLive = async (targets: Target[]) => {
    const seen = new Set<string>();
    const todo = targets.filter((t) => liveFor(t.platform) && !seen.has(t.message_id) && seen.add(t.message_id));
    if (todo.length === 0) return;
    stopBatch.current = false;
    let failed = 0;
    let lastError = "";
    setBatch({ label: liveFor(todo[0].platform)!.verb, done: 0, total: todo.length, failed: 0 });
    for (let i = 0; i < todo.length; i++) {
      if (stopBatch.current) break;
      try {
        onScan(await liveFor(todo[i].platform)!.run(todo[i].id));
      } catch (e) {
        failed++;
        lastError = String(e);
      }
      setBatch((b) => (b ? { ...b, done: i + 1, failed } : b));
      if (i < todo.length - 1) await new Promise((r) => setTimeout(r, 400));
    }
    const done = todo.length - failed;
    setBatch(null);
    reload();
    setToast({
      text: stopBatch.current
        ? "Stopped. What was already deleted stays deleted."
        : failed > 0
          ? `Deleted ${n(done)} of ${n(todo.length)}. ${failed} didn't work: ${lastError}`
          : `Deleted ${plural(done, "message", "messages")}. Emails went to Trash, where you can restore them.`,
    });
  };

  /** The selected items, plus every place each one's value appears when the list is grouped. */
  const expandSelection = async (): Promise<Target[]> => {
    const out: Target[] = [];
    for (const f of selected.values()) {
      if (groupList && f.repeats > 1) {
        const g = await api.groupInfo(f.id, tab);
        g.ids.forEach((id, i) => out.push({ id, message_id: g.message_ids[i], platform: g.member_platforms[i] }));
      } else {
        out.push({ id: f.id, message_id: f.message_id, platform: f.platform });
      }
    }
    return out;
  };
  const decideSelected = async (status: Status) => {
    const targets = await expandSelection();
    await applyMany(targets.map((t) => t.id), status, `${STATUS_DONE_TEXT[status]}: ${plural(targets.length, "item", "items")}`);
  };
  const toggleSelect = (f: FindingView, index: number, range: boolean) => {
    const next = new Map(selected);
    if (range && anchor != null) {
      const [a, b] = [Math.min(anchor, index), Math.max(anchor, index)];
      for (let i = a; i <= b; i++) if (items[i]) next.set(items[i]!.id, items[i]!);
    } else if (next.has(f.id)) next.delete(f.id);
    else next.set(f.id, f);
    setSelected(next);
    setAnchor(index);
  };

  const deleteLive = async (f: FindingView) => {
    const how = liveDelete(f);
    if (!how) return;
    try {
      onScan(await how.run(f.id));
      reload();
      setToast({ text: how.done });
    } catch (e) {
      setToast({ text: String(e) });
    }
  };
  const recycle = async (f: FindingView) => {
    try {
      const [path, summary] = await api.recycleFile(f.id);
      onScan(summary);
      reload();
      setToast({ text: `Moved ${path.split(/[\\/]/).pop()} to the Recycle Bin. You can restore it from there.` });
    } catch (e) {
      setToast({ text: String(e) });
    }
  };

  /** Starts the guided clean-up with the selected items, or everything still to review in this view. */
  const startGuide = async (fromSelection: boolean) => {
    let list: FindingView[];
    if (fromSelection) list = [...selected.values()].filter((f) => !f.status);
    else list = (await api.findings(kind, platform, "open", 0, 500, true, false, null)).items;
    if (list.length === 0) return setToast({ text: "Nothing left to clean up here." });
    setSelected(new Map());
    setGuide({ items: list, i: 0, deleted: 0, ok: 0, skipped: 0 });
  };
  /** Records what happened to the current item and moves on. */
  const guideNext = async (outcome: "deleted" | "ok" | "skipped", markIt = true) => {
    if (!guide) return;
    const f = guide.items[guide.i];
    if (markIt && outcome !== "skipped") {
      try {
        onScan(await api.setStatus(f.id, outcome));
      } catch (e) {
        return setGuide({ ...guide, error: String(e) });
      }
    }
    setGuide({ ...guide, i: guide.i + 1, busy: false, error: undefined, [outcome]: guide[outcome] + 1 });
  };
  /** Removes it for you where Sompio can: connected Telegram or email, or a file on this computer. */
  const guideRemove = async () => {
    if (!guide) return;
    const f = guide.items[guide.i];
    setGuide({ ...guide, busy: true, error: undefined });
    try {
      if (f.platform === "pc") onScan((await api.recycleFile(f.id))[1]);
      else onScan(await liveDelete(f)!.run(f.id));
      await guideNext("deleted", false);
    } catch (e) {
      setGuide({ ...guide, busy: false, error: String(e) });
    }
  };
  const closeGuide = () => {
    setGuide(null);
    reload();
  };

  // Keyboard: works anywhere on the page except while typing in a field.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (guide) return;
      const el = e.target as HTMLElement;
      if (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.tagName === "SELECT" || e.metaKey || e.ctrlKey || e.altKey) return;
      if (!total) return;
      const f = items[focus];
      const move = (d: number) => {
        e.preventDefault();
        const next = Math.max(0, Math.min(reachable - 1, focus + d));
        setFocus(next);
        v.scrollToIndex(toRow(next), { align: "auto" });
      };
      switch (e.key) {
        case "j": case "ArrowDown": return move(1);
        case "k": case "ArrowUp": return move(-1);
      }
      if (!f) return;
      const act = (fn: () => void) => { e.preventDefault(); fn(); };
      switch (e.key.toLowerCase()) {
        case " ": return act(() => toggleReveal(f));
        case "x": return act(() => toggleSelect(f, focus, e.shiftKey));
        case "escape": return selected.size ? act(() => setSelected(new Map())) : undefined;
        case "d": return f.status ? undefined : act(() => decideHere(f, "deleted"));
        case "n": return f.status ? undefined : act(() => decideHere(f, "ok"));
        case "l": return f.status ? undefined : act(() => decideHere(f, "later"));
        case "u": return f.status ? act(() => decide(f, null)) : undefined;
        case "o": return act(() => goTo(f));
        case "c": return act(() => copyLink(f));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [total, items, focus, decide, decideHere, toggleReveal, goTo, copyLink, v, selected, anchor]);

  const runScan = async (list = terms) => {
    setScanning([0, 1]);
    try {
      onScan(await api.runScan(list));
      setScannedTerms(list);
      setRevealed(new Map());
      reload();
    } catch (e) {
      setToast({ text: `The scan stopped: ${e}` });
    } finally {
      setScanning(null);
    }
  };

  const runScanRef = useRef(runScan);
  runScanRef.current = runScan;

  const addTerm = () => {
    const t = draft.trim();
    if (t.length < 2 || terms.some((x) => x.toLowerCase() === t.toLowerCase())) return;
    setTerms([...terms, t]);
    setDraft("");
  };

  const exportList = async () => {
    const path = await pickSavePath("sompio-flagged-items.csv");
    if (!path) return;
    const count = await api.exportFlagged(path);
    setToast({ text: `Saved ${plural(count, "item", "items")} with links. The file has no message text in it.` });
  };

  const termsChanged = scanned && JSON.stringify(terms) !== JSON.stringify(scannedTerms);
  const pct = scanning && scanning[1] > 0 ? (scanning[0] / scanning[1]) * 100 : 0;
  const all = scan ? scan.total_findings : 0;
  const base = items[focus];
  const current = base && (revealed.get(base.id) ? { ...base, ...pickText(revealed.get(base.id)!) } : base);

  const kinds = scan ? scan.by_kind.filter((k) => k.count > 0) : [];
  const kindCount = (k: (typeof kinds)[number]) => (tab === "open" ? k.values_open ?? k.open : null);
  const bySeverity: [number, string][] = [[3, "Fix now"], [2, "Worth a look"], [1, "Low risk"]];
  const tabCount = scan
    ? tab === "open" ? scan.values_open ?? scan.open : tab === "later" ? scan.values_later ?? scan.later : scan.values_done ?? scan.done
    : 0;
  const whereName = (f: FindingView) => (f.platform === "email" ? mailName(mailProvider(f.link, mailAccount)) : platformName[f.platform]);
  const whereIcon = (f: FindingView) => (f.platform === "email" ? `mail:${mailProvider(f.link, mailAccount)}` : f.platform);
  const placeNoun = (f: FindingView, k: number) => plural(k, f.platform === "email" ? "email" : f.platform === "pc" ? "file" : "message", f.platform === "email" ? "emails" : f.platform === "pc" ? "files" : "messages");

  return (
    <div className="xc-page review-page">
      <div className="xc-toolbar" role="toolbar" aria-label="Findings">
        <button className="search-field" onClick={() => onGo("messages")} title="Search everything you've sent">
          <Icon name="search" size={15} /><span>Search your messages</span>
        </button>
        <button onClick={() => runScan()} disabled={!!scanning} title="Check every message and file again"><Icon name="refresh" size={16} />Check again</button>
        <button aria-pressed={showTerms} onClick={() => setShowTerms(!showTerms)} title="Your name, street, school: things only you know are sensitive">
          <Icon name="filter" size={16} />Watch terms{terms.length ? ` (${terms.length})` : ""}
        </button>
        <button onClick={() => startGuide(false)} disabled={!scanned || !scan?.open} title="Go through everything still to review, one at a time, and remove it">
          <Icon name="trash" size={16} />Clean up
        </button>
        <button onClick={exportList} disabled={!scanned || all === 0} title="A list of everything found, with links. No message text."><Icon name="save" size={16} />Save list</button>
        <span className="grow" />
        <label className="switch" title="Show what was found. Turn off when someone can see your screen.">
          <input type="checkbox" checked={revealAll} onChange={(e) => { setRevealAll(e.target.checked); setRevealed(new Map()); }} />
          Show values
        </label>
        <button className="icon-only" aria-label="Trace settings" title="Trace settings: names, things never to flag, your data" onClick={() => onGo("settings")}>
          <Icon name="settings" size={16} />
        </button>
      </div>
      {tab !== "open" && (
        <div className="banner-row">
          <span>{tab === "later" ? "Saved for later." : "What you've dealt with. Kept so you can undo a slip."}</span>
          <button className="quiet" onClick={() => { setTab("open"); setFocus(0); }}>Back to what's left</button>
        </div>
      )}

      {(showTerms || !scanned) && (
        <WatchTerms
          terms={terms}
          draft={draft}
          demo={demo}
          scanned={scanned}
          changed={termsChanged}
          scanning={!!scanning}
          onDraft={setDraft}
          onAdd={addTerm}
          onRemove={(t) => setTerms(terms.filter((x) => x !== t))}
          onScan={() => runScan()}
          onClose={() => setShowTerms(false)}
        />
      )}
      {scanning && (
        <div className="progress xc-progress" role="status" aria-live="polite">
          <span>Checking {n(Math.round(scanning[0]))} of {n(scanning[1])} items for personal information</span>
          <div className="bar"><span style={{ width: `${pct}%` }} /></div>
        </div>
      )}
      {sameAs && (
        <div className="banner-row">
          <span>Every place this {NOUN[sameAs.kind]} appears{total != null ? ` (${n(total)})` : ""}. Decide on them one by one.</span>
          <button className="quiet" onClick={() => { setSameAs(null); setFocus(0); }}>Back to the full list</button>
        </div>
      )}
      {selected.size > 0 && (
        <BulkBar
          count={selected.size}
          grouped={groupList}
          tab={tab}
          live={(() => {
            const ps = new Set([...selected.values()].map((f) => f.platform));
            if (ps.size !== 1) return null;
            const how = liveFor([...ps][0]);
            return how && tab !== "done" ? how.ask(selected.size) : null;
          })()}
          onDecide={decideSelected}
          onGuide={() => startGuide(true)}
          onLive={async () => runLive(await expandSelection())}
          onClear={() => setSelected(new Map())}
        />
      )}

      <div className="xc-body">
        <aside className="xc-tree" aria-label="What was found">
          <button className="xc-node root" aria-current={kind == null} onClick={() => { setKind(null); setFocus(0); }}>
            <Icon name="review" size={16} /><span>{tab === "open" ? "Everything left" : "Everything"}</span><span className="num">{n(tabCount)}</span>
          </button>
          {bySeverity.map(([sev, title]) => {
            const ks = kinds.filter((k) => k.severity === sev);
            if (!ks.length) return null;
            return (
              <div key={sev} className="xc-branch">
                <div className="xc-branch-label"><span className={`sev sev-${sev}`} />{title}</div>
                {ks.map((k) => (
                  <button key={k.kind} className={`xc-node${tab === "open" && kindCount(k) === 0 ? " all-done" : ""}`} aria-current={kind === k.kind} onClick={() => { setKind(k.kind); setFocus(0); }}>
                    <span>{k.label}</span>
                    <span className="num">{kindCount(k) == null ? "" : kindCount(k) === 0 ? <Icon name="check" size={13} /> : n(kindCount(k)!)}</span>
                  </button>
                ))}
              </div>
            );
          })}
          {platforms.length > 1 && (
            <div className="xc-branch">
              <div className="xc-branch-label">Where</div>
              <button className="xc-node" aria-current={platform == null} onClick={() => { setPlatform(null); setFocus(0); }}><span>All sources</span></button>
              {platforms.map((p) => (
                <button key={p} className="xc-node" aria-current={platform === p} onClick={() => { setPlatform(p); setFocus(0); }}>
                  <span className="with-icon"><ServiceIcon id={p === "email" ? `mail:${mailAccount ?? "other"}` : p} size={14} />{p === "email" && mailAccount ? mailName(mailAccount) : platformName[p]}</span>
                </button>
              ))}
            </div>
          )}
          {scanned && (scan?.values_later ?? 0) > 0 && (
            <button className="xc-node" aria-current={tab === "later"} onClick={() => { setTab(tab === "later" ? "open" : "later"); setFocus(0); }}>
              <Icon name="clock" size={15} /><span>Saved for later</span><span className="num">{n(scan!.values_later!)}</span>
            </button>
          )}
          <span className="grow" />
          <SourcesBox sources={sources} trace={trace} mailAccount={mailAccount} onAdd={() => onGo("sources")} onRequests={() => onGo("requests")} />
          {scanned && all > 0 && (
            <button className="xc-tree-foot progress-link" onClick={() => { setTab(tab === "done" ? "open" : "done"); setFocus(0); }}
              title="See what you've dealt with">
              <div className="bar" aria-hidden><span style={{ width: `${((scan!.values_done ?? scan!.done) / Math.max(1, scan!.values_total ?? all)) * 100}%` }} /></div>
              <span className="small muted">{n(scan!.values_done ?? scan!.done)} of {n(scan!.values_total ?? all)} dealt with</span>
            </button>
          )}
        </aside>

        <div className="xc-main">
          <div className="xc-table">
            <div className="xc-thead" role="row">
              <span />
              <span>Type</span>
              <span>Found</span>
              <span>How often</span>
              <span>Where</span>
              <span>Last seen</span>
            </div>
            <div className="xc-rows" ref={parentRef} role="listbox" aria-label="Findings" aria-activedescendant={base ? `f-${base.id}` : undefined} tabIndex={0}>
              {scanned && total === 0 && (
                <div className="empty">
                  {all === 0 ? "Nothing personal found. Add more sources, or watch terms like your name." : tab === "open" ? "Nothing left here. Well done." : tab === "later" ? "Nothing saved for later." : "Nothing dealt with yet."}
                </div>
              )}
              {!scanned && !scanning && <div className="empty">Not checked yet. Click Check again.</div>}
              <div style={{ height: v.getTotalSize(), position: "relative" }}>
                {virtualItems.map((row) => {
                  const i = toItem(row.index);
                  if (i < 0) {
                    return (
                      <div key={row.key} style={{ position: "absolute", top: 0, left: 0, right: 0, height: row.size, transform: `translateY(${row.start}px)` }}>
                        <button className="xc-group-head" aria-expanded={theirsOpen} disabled={mine === 0} onClick={() => setShowTheirs(!theirsOpen)} title={THEIRS_TITLE}>
                          <span className="chev" aria-hidden>{theirsOpen ? "▾" : "▸"}</span>
                          <strong>In emails from others</strong>
                          <span className="grp-count">{n(theirs)}</span>
                          <span className="small muted">Replies, forwards and the people you wrote to. Usually less urgent.</span>
                        </button>
                      </div>
                    );
                  }
                  const f = items[i];
                  const shown = f && revealed.get(f.id);
                  const t = shown ? { ...f!, ...pickText(shown) } : f;
                  return (
                    <div key={row.key} style={{ position: "absolute", top: 0, left: 0, right: 0, height: row.size, transform: `translateY(${row.start}px)` }}>
                      {t ? (
                        <div
                          id={`f-${t.id}`}
                          role="option"
                          aria-selected={i === focus}
                          className={`xc-row${i === focus ? " active" : ""}${selected.has(t.id) ? " picked" : ""}${t.status ? " decided" : ""}`}
                          onClick={(e) => {
                            if (e.shiftKey || e.ctrlKey || e.metaKey) toggleSelect(t, i, e.shiftKey);
                            setFocus(i);
                          }}
                          onDoubleClick={() => goTo(t)}
                        >
                          <span>
                            <input type="checkbox" aria-label="Select" checked={selected.has(t.id)} onClick={(e) => e.stopPropagation()}
                              onChange={(e) => toggleSelect(t, i, (e.nativeEvent as MouseEvent).shiftKey)} />
                          </span>
                          <span className="c-type" title={t.theirs ? THEIRS_TITLE : undefined}>
                            <span className={`sev sev-${t.severity}`} aria-label={t.theirs ? `${severityLabel(t.severity)}, someone else's` : severityLabel(t.severity)} />{t.label}
                          </span>
                          <span className="c-found">
                            {t.masked ? <span className="redact">{t.matched}</span> : <strong>{t.matched}</strong>}
                          </span>
                          <span className="c-msg">
                            {t.repeats > 1 && groupList
                              ? <><strong>{placeNoun(t, t.repeats)}</strong>{(t.conversations ?? 1) > 1 ? ` in ${plural(t.conversations!, "conversation", "conversations")}` : t.platform === "email" ? " in 1 conversation" : ""}</>
                              : <span className="muted">{t.before.slice(-30)}<mark>{t.masked ? "•••" : t.matched}</mark>{t.after.slice(0, 40)}</span>}
                          </span>
                          <span className="c-where with-icon" title={`${whereName(t)}, ${t.channel_name}`}><ServiceIcon id={whereIcon(t)} size={15} />{whereName(t)}</span>
                          <span className="c-date">{dateTime(t.ts)}</span>
                        </div>
                      ) : (
                        <div className="xc-row"><span /><div className="placeholder" /></div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          <div className="xc-preview detail">
            {current ? (
              <>
                <div className="xc-preview-head">
                  <span className={`risk risk-${current.severity}`}>{severityLabel(current.severity)}</span>
                  <h2>{current.label}</h2>
                  {current.theirs && <TheirsChip />}
                  <span className="tag with-icon"><ServiceIcon id={whereIcon(current)} size={13} />{whereName(current)}</span>
                  <span className="small muted">{current.channel_name}, {dateTime(current.ts)}</span>
                  {current.status && (
                    <span className="status-chip"><Icon name="check" size={14} />{current.status === "deleted" ? "Deleted" : current.status === "ok" ? "Not a problem" : "Saved for later"}</span>
                  )}
                </div>
                <div className="xc-preview-body">
                  <div className="xc-preview-text">
                    <p className="quote">
                      {current.before}
                      <button
                        className={`redact${current.masked ? "" : " shown"}`}
                        onClick={() => toggleReveal(base!)}
                        aria-label={current.masked ? `Show hidden ${current.label.toLowerCase()}` : "Hide again"}
                      >
                        {current.matched}
                      </button>
                      {current.after}
                    </p>
                    {current.repeats > 1 && !sameAs && (
                      <div className="group-box">
                        <div>
                          <strong>This {NOUN[current.kind]} is in {placeNoun(current, current.repeats)}{group?.conversations ? `, ${plural(group.conversations.length, "conversation", "conversations")}` : ""}</strong>
                          <span className="small muted">
                            {group ? yearSpan(group.first_ts, group.last_ts) : "Counting…"}{current.platform === "email" ? " Replies quote earlier emails, so one thing you wrote can be in a whole conversation." : ""}
                          </span>
                        </div>
                        {group?.conversations && (
                          <div className="conv-list">
                            {(allConvs ? group.conversations : group.conversations.slice(0, 4)).map((c) => (
                              <ConversationRow key={c.first_id} c={c} icon={c.platform === "email" ? `mail:${mailProvider(c.link, mailAccount)}` : c.platform}
                                onOpen={() => goTo({ ...current, id: c.first_id, platform: c.platform, link: c.link, message_id: c.first_message_id, channel_id: c.first_channel_id })}
                                onShow={() => onOpen({ channelId: c.first_channel_id, messageId: c.first_message_id })} />
                            ))}
                            {!allConvs && group.conversations.length > 4 && (
                              <button className="link small" onClick={() => setAllConvs(true)}>Show all {n(group.conversations.length)} conversations</button>
                            )}
                          </div>
                        )}
                        <div className="toolbar">
                          {group && tab !== "done" && group.platforms
                            .map((p) => ({ p: p.platform, how: liveFor(p.platform), count: p.count }))
                            .filter((x) => x.how)
                            .map((x) =>
                              confirmGroupDelete === x.p ? (
                                <span key={x.p} className="toolbar">
                                  <button
                                    className="danger"
                                    onClick={() => {
                                      setConfirmGroupDelete(null);
                                      runLive(group.ids.map((id, i) => ({ id, message_id: group.message_ids[i], platform: group.member_platforms[i] })).filter((t) => t.platform === x.p));
                                    }}
                                  >
                                    <Icon name="trash" size={16} />Yes, delete them
                                  </button>
                                  <button className="quiet" onClick={() => setConfirmGroupDelete(null)}>Cancel</button>
                                </span>
                              ) : (
                                <button key={x.p} onClick={() => setConfirmGroupDelete(x.p)}><Icon name="trash" size={16} />{x.how!.ask(x.count)}</button>
                              ),
                            )}
                          <button className="quiet" onClick={() => { setSameAs(current); setFocus(0); }}>Decide on each place separately</button>
                        </div>
                      </div>
                    )}
                    {current.theirs && <p className="small theirs-note">{THEIRS_NOTE}</p>}
                    <p className="small muted advice-line"><strong>What to do: </strong>{current.advice}</p>
                    {current.link && current.platform !== "pc" && (
                      <p className="small muted find-hint">
                        When you open it, Sompio copies what it found. Press {navigator.platform.toLowerCase().includes("mac") ? "⌘F" : "Ctrl+F"} on the page and paste to jump to it.
                      </p>
                    )}
                  </div>

                  <div className="xc-preview-actions">
                    <div className="go-row">
                      {current.platform === "pc" ? (
                        <>
                          <button className="primary" onClick={() => goTo(current)}><Icon name="folder" size={16} />Show in folder</button>
                          {!current.status &&
                            (confirmRecycle ? (
                              <>
                                <button className="danger" onClick={() => recycle(current)}><Icon name="trash" size={16} />Yes, to the Recycle Bin</button>
                                <button className="quiet" onClick={() => setConfirmRecycle(false)}>Cancel</button>
                              </>
                            ) : (
                              <button onClick={() => setConfirmRecycle(true)}><Icon name="trash" size={16} />Move to Recycle Bin…</button>
                            ))}
                          <button className="quiet" onClick={() => copyLink(current)}><Icon name="copy" size={16} />Copy path</button>
                        </>
                      ) : (
                        <>
                          {liveDelete(current) &&
                            (confirmRecycle ? (
                              <>
                                <button className="danger" onClick={() => deleteLive(current)}><Icon name="trash" size={16} />{liveDelete(current)!.confirm}</button>
                                <button className="quiet" onClick={() => setConfirmRecycle(false)}>Cancel</button>
                              </>
                            ) : (
                              <button className="primary" onClick={() => setConfirmRecycle(true)}><Icon name="trash" size={16} />{liveDelete(current)!.ask}</button>
                            ))}
                          {(current.link || !liveDelete(current)) && (
                            <button className={liveDelete(current) ? undefined : "primary"} onClick={() => goTo(current)}><Icon name="external" size={16} />{openLabel(current.platform)}</button>
                          )}
                          {current.link && <button onClick={() => copyLink(current)}><Icon name="copy" size={16} />Copy link</button>}
                          <button className="quiet" onClick={() => onOpen({ channelId: current.channel_id, messageId: current.message_id })}>
                            <Icon name="messages" size={16} />Show conversation
                          </button>
                        </>
                      )}
                    </div>
                    <div className="decide">
                      {current.status ? (
                        <button onClick={() => decide(current, null)}>Move back to review <span className="key">U</span></button>
                      ) : (
                        <div className="decide-row">
                          <button onClick={() => decideHere(current, "deleted")}><Icon name="check" size={16} />{groupIds(current) ? "I deleted them all" : "I deleted it"} <span className="key">D</span></button>
                          <button onClick={() => decideHere(current, "ok")}>Not a problem{groupIds(current) ? ` (all ${n(groupIds(current)!.length)})` : ""} <span className="key">N</span></button>
                          <button onClick={() => decideHere(current, "later")}><Icon name="clock" size={16} />Later <span className="key">L</span></button>
                        </div>
                      )}
                      {!current.status && (
                        <button className="link small" onClick={() => ignoreEverywhere(current)}>Never flag this {NOUN[current.kind]} again</button>
                      )}
                    </div>
                  </div>
                </div>
              </>
            ) : (
              <div className="detail-empty small muted">
                {scanned ? "Pick something in the list to see it here. Double-click opens it where it was posted." : ""}
              </div>
            )}
          </div>
        </div>
      </div>

      {guide && (
        <GuidePanel
          guide={guide}
          revealAll={revealAll}
          removeLabel={(f) => (f.platform === "pc" ? "Move to Recycle Bin" : liveDelete(f) ? (f.platform === "telegram" ? "Delete from Telegram" : "Move to Trash") : null)}
          onOpen={goTo}
          onRemove={guideRemove}
          onNext={guideNext}
          onClose={closeGuide}
        />
      )}
      {batch && (
        <div className="toast" role="status" aria-live="polite">
          <span>{batch.label}: {n(batch.done)} of {n(batch.total)}{batch.failed ? `, ${batch.failed} failed` : ""}</span>
          <button className="link" onClick={() => { stopBatch.current = true; }}>Stop</button>
        </div>
      )}
      {toast && !batch && (
        <div className="toast" role="status" aria-live="polite">
          <span>{toast.text}</span>
          {toast.undo && <button className="link" onClick={toast.undo}>Undo</button>}
          <button className="toast-close" aria-label="Dismiss" onClick={() => setToast(null)}>×</button>
        </div>
      )}
    </div>
  );
}

/** One conversation a value appears in, with a way to open where it started. */
function ConversationRow({ c, icon, onOpen, onShow }: { c: Conversation; icon: string; onOpen: () => void; onShow: () => void }) {
  const noun = c.platform === "email" ? ["email", "emails"] : ["message", "messages"];
  return (
    <div className={`conv-row${c.open ? "" : " done"}`}>
      <ServiceIcon id={icon} size={16} />
      <div className="conv-text">
        <strong title={c.title}>{c.title}</strong>
        <span className="small muted">
          {plural(c.count, noun[0], noun[1])} · {c.count > 1 ? `first on ${dateTime(c.first_ts)}` : dateTime(c.first_ts)}
        </span>
      </div>
      {!c.open && <span className="small done-mark"><Icon name="check" size={13} />Dealt with</span>}
      {c.link ? (
        <button onClick={onOpen} title="Opens the first email of this conversation that has it">{c.count > 1 ? "Open the first one" : "Open"}</button>
      ) : (
        <button className="quiet" onClick={onShow}>Show</button>
      )}
    </div>
  );
}

/** Your sources and requests, at the bottom of the list of types. */
function SourcesBox({ sources, trace, mailAccount, onAdd, onRequests }: {
  sources: SourceView[]; trace: TraceData | null; mailAccount: string | null; onAdd: () => void; onRequests: () => void;
}) {
  const have = new Set(sources.map((s) => s.platform));
  const asked = Object.entries(trace?.exports ?? {}).filter(([key]) => !have.has((key === "gmail" ? "email" : key) as Platform));
  const waiting = (trace?.requests ?? []).filter((r) => r.status === "waiting").length;
  const ago = (day: string | null) => {
    if (!day) return "";
    const d = Math.max(0, Math.round((Date.now() - new Date(`${day}T00:00:00`).getTime()) / 86_400_000));
    return d === 0 ? "today" : d === 1 ? "yesterday" : `${d} days ago`;
  };
  return (
    <div className="xc-sources">
      <div className="xc-branch-label">Sources</div>
      {sources.map((s) => (
        <div key={s.platform} className="xc-source">
          <ServiceIcon id={s.platform === "email" ? `mail:${mailAccount ?? "other"}` : s.platform} size={14} />
          <span>{s.platform === "email" && mailAccount ? mailName(mailAccount) : platformName[s.platform]}</span>
          <span className="faint small">{dateTime(s.imported_at).split(",")[0]}</span>
        </div>
      ))}
      {asked.map(([key, v]) => (
        <div key={key} className="xc-source waiting" title="You asked for your data. Add the file when it arrives.">
          <ServiceIcon id={key} size={14} />
          <span>{platformName[key as Platform] ?? key}</span>
          <span className="faint small">asked {ago(v.requested_on)}</span>
        </div>
      ))}
      <button className="add-source" onClick={onAdd}><Icon name="plus" size={14} />Add a source</button>
      <button className="req-card" onClick={onRequests}>
        <strong>Ask companies to delete your data →</strong>
        <span className="small muted">{waiting > 0 ? `${waiting} waiting for an answer` : "Letters and deadlines"}</span>
      </button>
    </div>
  );
}

/** Marks a finding in someone else's words: an email you quoted or forwarded, or a recipient's address. */
const THEIRS_TITLE = "In an email you replied to or forwarded, or a recipient's address: written by or about someone else";
const TheirsChip = () => (
  <span className="theirs-chip" title={THEIRS_TITLE}>
    Someone else's
  </span>
);
const THEIRS_NOTE =
  "This is in the email you replied to or forwarded, or it's who you sent it to. Someone else wrote it, or it's about them, so it matters less for you. Deleting your sent email removes your copy; they still have theirs.";

const pickText = (f: FindingView) => ({ before: f.before, matched: f.matched, after: f.after, masked: false });

const yearSpan = (a: number, b: number) => {
  const [x, y] = [new Date(a).getFullYear(), new Date(b).getFullYear()];
  return !a ? "" : x === y ? `All from ${x}.` : `From ${x} to ${y}.`;
};

function BulkBar(p: {
  count: number; grouped: boolean; tab: StatusTab; live: string | null;
  onDecide: (s: Status) => void; onGuide: () => void; onLive: () => void; onClear: () => void;
}) {
  const [confirm, setConfirm] = useState(false);
  return (
    <div className="bulk-bar" role="region" aria-label="Selected items">
      <strong>{plural(p.count, "selected", "selected")}</strong>
      {p.grouped && <span className="small muted">Includes every place each value appears.</span>}
      <span className="grow" />
      {p.live &&
        (confirm ? (
          <>
            <button className="danger" onClick={() => { setConfirm(false); p.onLive(); }}><Icon name="trash" size={16} />Yes, delete them</button>
            <button className="quiet" onClick={() => setConfirm(false)}>Cancel</button>
          </>
        ) : (
          <button onClick={() => setConfirm(true)}><Icon name="trash" size={16} />{p.live}</button>
        ))}
      {p.tab !== "done" && (
        <>
          <button onClick={p.onGuide} title="Open each one where it was posted, then mark it"><Icon name="arrow" size={16} />One by one</button>
          <button onClick={() => p.onDecide("deleted")}><Icon name="check" size={16} />I deleted them</button>
          <button onClick={() => p.onDecide("ok")}>Not a problem</button>
          {p.tab !== "later" && <button onClick={() => p.onDecide("later")}><Icon name="clock" size={16} />Later</button>}
        </>
      )}
      <button className="quiet" onClick={p.onClear}>Clear</button>
    </div>
  );
}

function WatchTerms(p: {
  terms: string[]; draft: string; demo: boolean; scanned: boolean; changed: boolean; scanning: boolean;
  onDraft: (s: string) => void; onAdd: () => void; onRemove: (t: string) => void; onScan: () => void; onClose: () => void;
}) {
  const empty = p.terms.length === 0;
  return (
    <div className="watch slim">
      <div className="watch-title">
        <h3>Watch terms</h3>
        <span className="muted small">
          Your real name, street, school or employer. Patterns can't guess these. They never leave this computer.
          {p.demo && " In the sample, try Sam Rivers."}
        </span>
      </div>
      {!empty && (
        <div className="chips">
          {p.terms.map((t) => (
            <span className="chip" key={t}>
              {t}
              <button aria-label={`Remove ${t}`} onClick={() => p.onRemove(t)}>×</button>
            </span>
          ))}
        </div>
      )}
      <div className="watch-row">
        <input
          type="text"
          value={p.draft}
          placeholder="Your full name, street, school…"
          onChange={(e) => p.onDraft(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && p.onAdd()}
          aria-label="New watch term"
        />
        <button onClick={p.onAdd} disabled={p.draft.trim().length < 2}><Icon name="plus" size={16} />Add</button>
        <button className={p.changed || !p.scanned ? "primary" : undefined} onClick={p.onScan} disabled={p.scanning}>
          {p.scanned ? "Check again" : "Start checking"}
        </button>
        {p.scanned && <button className="quiet" onClick={p.onClose}>Close</button>}
      </div>
      {p.changed && !p.scanning && <p className="small note">Your watch terms changed. Check again to include them. Your decisions are kept.</p>}
    </div>
  );
}

function GuidePanel(p: {
  guide: Guide; revealAll: boolean;
  removeLabel: (f: FindingView) => string | null;
  onOpen: (f: FindingView) => void; onRemove: () => void;
  onNext: (o: "deleted" | "ok" | "skipped") => void; onClose: () => void;
}) {
  const g = p.guide;
  const f = g.items[g.i];
  const [opened, setOpened] = useState(false);
  useEffect(() => setOpened(false), [g.i]);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).tagName === "TEXTAREA") return;
      if (e.key === "Escape") return p.onClose();
      if (!f || g.busy) return;
      if (e.key === "d") p.onNext("deleted");
      else if (e.key === "s") p.onNext("skipped");
      else if (e.key === "o") { p.onOpen(f); setOpened(true); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
  const remove = f ? p.removeLabel(f) : null;
  return (
    <div className="modal-backdrop" role="presentation">
      <div className="modal guide" role="dialog" aria-modal="true" aria-label="Clean up one by one">
        {f ? (
          <>
            <div className="guide-head">
              <strong>Clean up one by one</strong>
              <span className="muted small">{n(g.i + 1)} of {n(g.items.length)}</span>
              <span className="grow" />
              <button className="quiet" onClick={p.onClose}>Stop</button>
            </div>
            <div className="bar"><span style={{ width: `${(g.i / g.items.length) * 100}%` }} /></div>
            <div className="guide-item">
              <div className="guide-meta">
                <span className={`risk risk-${f.severity}`}>{severityLabel(f.severity)}</span>
                <strong>{f.label}</strong>
                {f.theirs && <TheirsChip />}
                <span className="tag">{platformName[f.platform]}</span>
                <span className="small muted">{f.channel_name}, {dateTime(f.ts)}</span>
              </div>
              <p className="quote">{f.before}<mark>{f.masked && !p.revealAll ? "•••" : f.matched}</mark>{f.after}</p>
              {f.theirs && <p className="small theirs-note">{THEIRS_NOTE}</p>}
              {f.advice && <p className="small"><strong>What to do:</strong> {f.advice}</p>}
            </div>
            {g.error && <p className="error small" role="alert">{g.error}</p>}
            <div className="guide-actions">
              {remove ? (
                <button className="primary" disabled={g.busy} onClick={p.onRemove}><Icon name="trash" size={16} />{g.busy ? "Removing…" : remove}</button>
              ) : (
                <button className={opened ? undefined : "primary"} onClick={() => { p.onOpen(f); setOpened(true); }}>
                  <Icon name="external" size={16} />{openLabel(f.platform)} <span className="key">O</span>
                </button>
              )}
              <button className={!remove && opened ? "primary" : undefined} disabled={g.busy} onClick={() => p.onNext("deleted")}>
                <Icon name="check" size={16} />I deleted it, next <span className="key">D</span>
              </button>
              <button disabled={g.busy} onClick={() => p.onNext("ok")}>Not a problem</button>
              <button className="quiet" disabled={g.busy} onClick={() => p.onNext("skipped")}>Skip <span className="key">S</span></button>
            </div>
            {!remove && f.platform !== "pc" && (
              <p className="small muted">Sompio opens the message in your browser or app. Delete it there, come back, and click "I deleted it".</p>
            )}
          </>
        ) : (
          <>
            <div className="guide-head"><strong>All done</strong></div>
            <p>
              {plural(g.deleted, "item removed", "items removed")}, {n(g.ok)} marked as not a problem, {n(g.skipped)} skipped.
              {g.skipped > 0 && " Skipped items are still under To review."}
            </p>
            <div className="guide-actions"><button className="primary" onClick={p.onClose}>Close</button></div>
          </>
        )}
      </div>
    </div>
  );
}
