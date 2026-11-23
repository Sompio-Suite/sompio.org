import { useState } from "react";
import type { HealthReport, LicenceStatus, ScanSummary, TraceData, VaultInTrace } from "../api";
import { date, n, plural } from "../format";
import Icon from "../Icon";
import { EXPORTS, addDays, addMonth, daysUntil } from "../trace";
import { Notices } from "./GettingStarted";
import type { View } from "../App";

interface Props {
  licence: LicenceStatus | null;
  scan: ScanSummary | null;
  /** Trace has data (a scan or the samples) */
  traced: boolean;
  health: HealthReport | null;
  inChats: VaultInTrace[];
  trace: TraceData | null;
  onGo: (v: View) => void;
  onReview: () => void;
}

/** What goes into the exposure score. Kept simple on purpose, so "Why" can explain every point. */
interface Parts {
  fix: number; // open "fix now" findings
  look: number; // open "worth a look" findings
  low: number; // open low-risk findings
  inChats: number; // Vault passwords that also turned up in chats or files
  weak: number; // Vault passwords that are weak or used more than once
  late: number; // requests to companies past the one-month deadline
}

// Each part fills up towards its ceiling: the first findings count most, so a long tail of
// small things never drowns out a few serious ones. Trace up to 60, Vault 25, requests 15.
const fill = (weight: number, half: number, max: number) => max * (1 - Math.exp(-weight / half));
function points(p: Parts) {
  const trace = fill(3 * p.fix + p.look + 0.25 * p.low, 40, 60);
  const vault = fill(6 * p.inChats + 2 * p.weak, 12, 25);
  const requests = Math.min(15, 5 * p.late);
  return { trace, vault, requests, total: Math.round(trace + vault + requests) };
}

const tone = (score: number) => (score >= 60 ? "high" : score >= 30 ? "mid" : "low");

function Ring({ value, size, label, color }: { value: number; size: number; label: string; color: string }) {
  const stroke = size > 120 ? 14 : 10;
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  return (
    <div className="ring" style={{ width: size, height: size }}>
      <svg width={size} height={size} aria-hidden="true">
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--sunken)" strokeWidth={stroke} />
        <circle
          cx={size / 2} cy={size / 2} r={r} fill="none" stroke={color} strokeWidth={stroke} strokeLinecap="round"
          strokeDasharray={c} strokeDashoffset={c * (1 - Math.max(value, 1) / 100)}
        />
      </svg>
      <div className="ring-v"><b>{value}</b><span>{label}</span></div>
    </div>
  );
}

export default function Overview({ licence, scan, traced, health, inChats, trace, onGo, onReview }: Props) {
  const [how, setHow] = useState(false);

  const open = (sev: number) => (traced && scan ? scan.by_kind.filter((k) => k.severity === sev).reduce((a, k) => a + k.mine_open, 0) : 0);
  const weakOrReused = health?.items.filter((i) => i.issues.some((x) => x.kind === "weak" || x.kind === "reused")).length ?? 0;
  const oldPw = health?.items.filter((i) => i.issues.some((x) => x.kind === "old")).length ?? 0;
  const requests = trace?.requests ?? [];
  const waiting = requests.filter((r) => r.status === "waiting");
  const late = waiting.filter((r) => r.sent_on && daysUntil(addMonth(r.sent_on)) < 0);
  const answered = requests.filter((r) => r.status !== "waiting").length;
  const exports = EXPORTS.filter((e) => trace?.exports[e.key]?.requested_on);
  const exportsReady = exports.filter((e) => daysUntil(addDays(trace!.exports[e.key].requested_on!, e.days)) <= 0);

  const parts: Parts = { fix: open(3), look: open(2), low: open(1), inChats: inChats.length, weak: weakOrReused, late: late.length };
  const now = points(parts);
  const after = points({ ...parts, fix: 0, look: 0, inChats: 0, weak: 0, late: 0 });
  const hasVault = (health?.checked ?? 0) > 0;
  const hasAnything = traced || hasVault || requests.length > 0;

  const dealt = scan ? scan.done : 0;
  const allFindings = scan ? scan.open + scan.later + scan.done : 0;
  const goodPw = health ? health.checked - health.items.filter((i) => i.severity >= 2).length : 0;

  // The most useful thing to do now, most serious first
  const next: { title: string; text: string; go: () => void; button: string } = !traced
    ? { title: "Find what you've left online", text: "Scan your computer and mailbox with Trace. It takes a few minutes and nothing leaves this computer.", go: () => onGo("home"), button: "Start finding" }
    : inChats.length > 0
      ? { title: `Change ${plural(inChats.length, "password", "passwords")} that turned up in your chats`, text: "Trace found them in messages or files, and they're still the same in Vault. Change them on the website first, then in Vault.", go: () => onGo("vault"), button: "Show me" }
      : parts.fix > 0
        ? { title: `Deal with ${plural(parts.fix, '"fix now" finding', '"fix now" findings')}`, text: "Passwords, ID and bank numbers you've shared. Delete them where you can, most risky first.", go: onReview, button: "Start cleaning" }
        : late.length > 0
          ? { title: `Remind ${plural(late.length, "company", "companies")} that missed the deadline`, text: "They had one month to answer. Trace writes the reminder for you.", go: () => onGo("requests"), button: "Open requests" }
          : weakOrReused > 0
            ? { title: `Fix ${plural(weakOrReused, "weak or reused password", "weak or reused passwords")}`, text: "Give each account its own strong password. Vault can make one for you.", go: () => onGo("vault"), button: "Open Vault" }
            : parts.look > 0
              ? { title: `Go through ${plural(parts.look, "finding", "findings")} worth a look`, text: "Addresses, phone numbers and other details. Delete what you don't want out there.", go: onReview, button: "Keep cleaning" }
              : requests.length === 0
                ? { title: "Ask companies to delete what they hold", text: "Deleting your messages doesn't delete the company's copy. Trace writes the letter and tracks the one-month deadline.", go: () => onGo("requests"), button: "Ask a company" }
                : { title: "You're on top of it", text: "Scan again now and then, especially after importing new data.", go: () => onGo("sources"), button: "Add a source" };

  // "Why" names the parts that count, biggest first
  const why = [
    { pts: now.trace, text: parts.fix ? `${n(parts.fix)} "fix now" ${parts.fix === 1 ? "finding" : "findings"}` : parts.look ? `${n(parts.look)} ${parts.look === 1 ? "finding" : "findings"} worth a look` : parts.low ? `${n(parts.low)} low-risk ${parts.low === 1 ? "finding" : "findings"}` : "" },
    { pts: now.vault, text: parts.inChats + parts.weak ? `${plural(parts.inChats + parts.weak, "password", "passwords")} at risk in Vault` : "" },
    { pts: now.requests, text: parts.late ? `${plural(parts.late, "request", "requests")} past the deadline` : "" },
  ].filter((w) => w.text && w.pts >= 0.5).sort((a, b) => b.pts - a.pts);

  return (
    <div className="page overview">
      <div className="page-head">
        <div>
          <h1>Overview</h1>
          <p className="muted">
            How exposed you are, based on what Sompio has found so far.
            {traced && scan?.scanned_at_ms ? ` Last scan: ${date(scan.scanned_at_ms)}.` : ""}
          </p>
        </div>
      </div>

      <Notices licence={licence} onGo={onGo} />

      <div className="ov-top">
        <section className="panel ov-score">
          <h2>Your exposure</h2>
          {hasAnything ? (
            <>
              <div className="ov-rings">
                <Ring value={now.total} size={150} label="out of 100" color={tone(now.total) === "low" ? "var(--ok)" : `var(--risk-${tone(now.total)})`} />
                {after.total < now.total && (
                  <>
                    <Icon name="arrow" size={22} />
                    <Ring value={after.total} size={104} label="after your plan" color="var(--ok)" />
                  </>
                )}
              </div>
              <div className="ov-why">
                {why.length ? (
                  <><strong>Why {now.total}?</strong> {why.length > 1 ? `Mostly ${why.map((w) => w.text).join(", then ")}.` : `It comes from ${why[0].text}.`} Lower is better.</>
                ) : (
                  <><strong>Nothing open.</strong> Everything Sompio found is dealt with. Lower is better, and you're at the bottom.</>
                )}
                {!traced && <> Trace hasn't scanned anything yet, so this only counts Vault.</>}{" "}
                <button className="link" onClick={() => setHow((h) => !h)}>{how ? "Hide how it works" : "How the score works"}</button>
                {how && (
                  <ul className="ov-how">
                    <li><strong>Trace, up to 60:</strong> open findings. "Fix now" counts three times, "worth a look" once, low risk a quarter.</li>
                    <li><strong>Vault, up to 25:</strong> passwords that turned up in your chats count six times, weak or reused ones twice.</li>
                    <li><strong>Requests, up to 15:</strong> 5 for each company past its one-month deadline.</li>
                    <li>The first few things count most, so a long list of small things never outweighs a few serious ones. "After your plan" is what's left once the risky findings, passwords and late requests are dealt with. It only knows what you've scanned.</li>
                  </ul>
                )}
              </div>
            </>
          ) : (
            <div className="ov-empty">
              <p><strong>No score yet.</strong> Sompio works out how exposed you are once Trace has scanned something or Vault has passwords in it.</p>
              <button className="primary" onClick={() => onGo("start")}>Getting started</button>
            </div>
          )}
        </section>

        <div className="ov-side">
          <section className="ov-next">
            <div>
              <strong>Next best step: {next.title.charAt(0).toLowerCase() + next.title.slice(1)}</strong>
              <span>{next.text}</span>
            </div>
            <button className="primary" onClick={next.go}>{next.button}</button>
          </section>
          <section className="panel">
            <h2>Progress</h2>
            <div className="meter" aria-hidden="true"><i style={{ width: `${allFindings ? Math.round((100 * dealt) / allFindings) : 0}%` }} /></div>
            <div className="ov-row"><span>Findings dealt with</span><strong>{traced ? `${n(dealt)} of ${n(allFindings)}` : "–"}</strong></div>
            <div className="ov-row"><span>Passwords in good shape</span><strong>{hasVault ? `${n(goodPw)} of ${n(health!.checked)}` : "–"}</strong></div>
            <div className="ov-row"><span>Companies that answered</span><strong>{requests.length ? `${n(answered)} of ${n(requests.length)}` : "–"}</strong></div>
          </section>
        </div>
      </div>

      <div className="ov-cards">
        <section className="panel">
          <h2><Icon name="review" size={17} />Trace</h2>
          {traced ? (
            <>
              <div className="ov-row"><span><i className="dot risk-bg-3" />Fix now</span><strong>{n(parts.fix)}</strong></div>
              <div className="ov-row"><span><i className="dot risk-bg-2" />Worth a look</span><strong>{n(parts.look)}</strong></div>
              <div className="ov-row"><span><i className="dot risk-bg-1" />Low risk</span><strong>{n(parts.low)}</strong></div>
              <div className="ov-foot"><span className="muted small">{plural(scan?.scanned ?? 0, "message or file", "messages and files")} scanned</span><button className="link" onClick={onReview}>Clean up</button></div>
            </>
          ) : (
            <div className="ov-foot"><span className="muted small">Nothing scanned yet.</span><button className="link" onClick={() => onGo("home")}>Start finding</button></div>
          )}
        </section>
        <section className="panel">
          <h2><Icon name="key" size={17} />Vault</h2>
          {hasVault ? (
            <>
              <div className="ov-row"><span>Found in your chats</span><strong className={inChats.length ? "risk-3" : ""}>{n(inChats.length)}</strong></div>
              <div className="ov-row"><span>Weak or reused</span><strong>{n(weakOrReused)}</strong></div>
              <div className="ov-row"><span>Not changed in a year</span><strong>{n(oldPw)}</strong></div>
              <div className="ov-foot"><span className="muted small">{plural(health!.checked, "password", "passwords")}</span><button className="link" onClick={() => onGo("vault")}>Open Vault</button></div>
            </>
          ) : (
            <div className="ov-foot"><span className="muted small">No passwords in Vault yet.</span><button className="link" onClick={() => onGo("vault")}>Open Vault</button></div>
          )}
        </section>
        <section className="panel">
          <h2><Icon name="mail" size={17} />Requests to companies</h2>
          <div className="ov-row"><span>Waiting for an answer</span><strong>{n(waiting.length - late.length)}</strong></div>
          <div className="ov-row"><span>Past the deadline</span><strong className={late.length ? "risk-3" : ""}>{n(late.length)}</strong></div>
          <div className="ov-row"><span>Data exports ready to download</span><strong>{n(exportsReady.length)}</strong></div>
          <div className="ov-foot">
            <span className="muted small">{exports.length > exportsReady.length ? `${exports.length - exportsReady.length} more on the way` : requests.length ? "" : "No requests yet."}</span>
            <button className="link" onClick={() => onGo("requests")}>Requests</button>
          </div>
        </section>
      </div>
    </div>
  );
}
