import { openExternal, type LicenceStatus, type ScanSummary } from "../api";
import { date, n, plural } from "../format";
import Icon from "../Icon";
import { BUY_URL } from "../licence";
import { markStep } from "../steps";
import type { View } from "../App";

const LEARN_URL = "https://sompio.org/what-is-personal-information.php";

interface Props {
  licence: LicenceStatus | null;
  scan: ScanSummary | null;
  cleanReady: boolean;
  learned: boolean;
  secured: boolean;
  onGo: (v: View) => void;
}

export default function GettingStarted({ licence, scan, cleanReady, learned, secured, onGo }: Props) {
  const found = cleanReady && !!scan && scan.scanned > 0;
  const cleaned = found && scan!.open === 0;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1>Getting started</h1>
          <p className="muted">Four steps to find what you've left online, remove it, and keep your digital footprint safe. Everything happens on this computer.</p>
        </div>
      </div>

      <Notices licence={licence} onGo={onGo} />

      <ol className="plan">
        <li className={`plan-step${learned ? " done" : ""}`}>
          <span className="plan-num">{learned ? <Icon name="check" size={16} /> : "1"}</span>
          <div className="plan-body">
            <h2>Learn</h2>
            <p>What counts as personal information, where it ends up, how it gets misused, and when sharing it is fine. Five minutes, plain words.</p>
          </div>
          <div className="plan-act">
            <button onClick={() => { markStep("learned"); openExternal(LEARN_URL).catch(() => {}); }}>{learned ? "Read again" : "Read the basics"}<Icon name="external" size={13} /></button>
          </div>
        </li>
        <li className={`plan-step${found ? " done" : ""}`}>
          <span className="plan-num">{found ? <Icon name="check" size={16} /> : "2"}</span>
          <div className="plan-body">
            <h2>Find</h2>
            <p>
              {found
                ? <>Trace has looked at {n(scan!.scanned)} messages and files. Add more: your mailbox, another folder, or the data you asked companies for.</>
                : <>Scan your computer and mailbox in minutes. Ask Discord, Instagram and others for your data; Trace reminds you when it's ready.</>}
            </p>
          </div>
          <div className="plan-act">
            <button className={found ? "" : "primary"} onClick={() => onGo(found ? "sources" : "home")}>{found ? "Add a source" : "Start finding"}</button>
          </div>
        </li>
        <li className={`plan-step${cleaned ? " done" : ""}`}>
          <span className="plan-num">{cleaned ? <Icon name="check" size={16} /> : "3"}</span>
          <div className="plan-body">
            <h2>Clean</h2>
            <p>
              {!found
                ? <>Go through what was found, most risky first. Delete it, and send letters that make companies erase what they hold.</>
                : cleaned
                  ? <>Everything found so far is dealt with. Companies still hold copies: ask them to delete what they have.</>
                  : <><strong>{plural(scan!.open, "thing", "things")} to review</strong>, most risky first. Delete them, then ask companies to erase their copies.</>}
            </p>
          </div>
          <div className="plan-act">
            <button className={found && !cleaned ? "primary" : ""} disabled={!found} onClick={() => onGo(cleaned ? "requests" : "review")}>{cleaned ? "Ask companies" : "Start cleaning"}</button>
          </div>
        </li>
        <li className={`plan-step${secured ? " done" : ""}`}>
          <span className="plan-num">{secured ? <Icon name="check" size={16} /> : "4"}</span>
          <div className="plan-body">
            <h2>Secure</h2>
            <p>Stop leaking from now on: change passwords that were out there, turn on two-step login, and keep your passwords in Vault, encrypted on this computer.</p>
          </div>
          <div className="plan-act">
            <button onClick={() => onGo("vault")}>Open Vault</button>
          </div>
        </li>
      </ol>

      <section className="promise">
        <div><Icon name="lock" /><strong>Stays on this computer</strong><span>No account, no cloud. Nothing is scanned until you start it, and nothing is uploaded.</span></div>
        <div><Icon name="ticket" /><strong>Keys, not logins</strong><span>A key unlocks what you bought. It's checked here, offline.</span></div>
        <div><Icon name="check" /><strong>Never held hostage</strong><span>When a key runs out, everything stays readable and exportable.</span></div>
      </section>
    </div>
  );
}

/** The beta, trial and ended-access notices, shown at the top of Overview and Getting started */
export function Notices({ licence, onGo }: { licence: LicenceStatus | null; onGo: (v: View) => void }) {
  const trialOnly = licence?.apps.every((a) => a.trial_only) ?? false;
  const anyActive = licence?.apps.some((a) => a.active) ?? false;
  return (
    <>
        {licence?.beta && (
          <div className="notice">
            <Icon name="bug" />
            <div>
              <strong>This is the Sompio beta: everything is unlocked, free.</strong>
              <span>
                Your 30-day free trial starts when Sompio launches. It's a beta, so things can still go wrong: keep a copy of
                your passwords (Vault, then Auto-type, import and export, then Save as KeePass file) and your recovery sheet.
                Found a problem or have an idea? Tell us with Report a bug (also at the top right).
              </span>
            </div>
            <div className="notice-actions">
              <button onClick={() => onGo("report")}><Icon name="bug" size={15} />Report a bug</button>
            </div>
          </div>
        )}
        {licence && !licence.beta && trialOnly && anyActive && licence.trial_ends_at && (
          <div className="notice">
            <Icon name="sparkle" />
            <div>
              <strong>You're on the free trial until {date(licence.trial_ends_at)}.</strong>
              <span>Everything is unlocked. Activate a product before then to keep going; nothing you add here is lost either way.</span>
            </div>
            <div className="notice-actions">
              <button onClick={() => openExternal(BUY_URL).catch(() => {})}>Get a key</button>
              <button onClick={() => onGo("licences")}>Activate a product</button>
            </div>
          </div>
        )}
        {licence && !anyActive && (
          <div className="notice warn">
            <Icon name="lock" />
            <div>
              <strong>Your access has ended.</strong>
              <span>Everything you added is still here and readable. Activate a product to keep cleaning and syncing.</span>
            </div>
            <div className="notice-actions">
              <button className="primary" onClick={() => openExternal(BUY_URL).catch(() => {})}>Get a key</button>
              <button onClick={() => onGo("licences")}>Activate a product</button>
            </div>
          </div>
        )}
    </>
  );
}
