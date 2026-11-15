import { useEffect, useState } from "react";
import { api, pickScreenshot } from "../api";
import Icon from "../Icon";

const AREAS = ["Vault", "Trace", "Calendar", "Installing or starting", "The lock", "Something else"];
const KINDS = ["Something didn't work", "Something was confusing", "An idea", "Something I liked"];

/**
 * Report a bug from inside the app. Nothing is sent until the person presses Send, and the
 * window shows everything that goes along with their message.
 */
export default function ReportBug() {
  const [area, setArea] = useState(AREAS[0]);
  const [kind, setKind] = useState(KINDS[0]);
  const [message, setMessage] = useState("");
  const [contact, setContact] = useState("");
  const [withDetails, setWithDetails] = useState(true);
  const [showDetails, setShowDetails] = useState(false);
  const [screenshot, setScreenshot] = useState<string | null>(null);
  const [about, setAbout] = useState<{ version: string; os: string; recent: string[] } | null>(null);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<{ ok: boolean; text: string } | null>(null);

  useEffect(() => {
    api.appDetails().then(setAbout).catch(() => setAbout(null));
  }, []);

  const contactOk = contact.trim() === "" || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(contact.trim());
  const canSend = message.trim().length >= 3 && contactOk && !busy;

  const send = async () => {
    setBusy(true);
    setResult(null);
    try {
      const sent = await api.sendBugReport({ area, kind, message: message.trim(), contact: contact.trim(), details: withDetails, screenshot });
      setResult({ ok: true, text: sent.message + (contact.trim() ? " If we have questions, we'll email you." : "") });
      setMessage("");
      setScreenshot(null);
    } catch (e) {
      setResult({ ok: false, text: String(e) });
    } finally {
      setBusy(false);
    }
  };

  const fileName = screenshot?.split(/[\\/]/).pop();

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1>Report a bug</h1>
          <p className="muted">
            Bugs, confusing bits, ideas, or what you liked. It decides what gets fixed first. Nothing is sent until you press Send.
          </p>
        </div>
      </div>

      <section className="panel report-form">
        <div className="field-pair">
          <label className="field">
            <span>Which part?</span>
            <select value={area} onChange={(e) => setArea(e.target.value)}>
              {AREAS.map((a) => <option key={a}>{a}</option>)}
            </select>
          </label>
          <label className="field">
            <span>What kind?</span>
            <select value={kind} onChange={(e) => setKind(e.target.value)}>
              {KINDS.map((k) => <option key={k}>{k}</option>)}
            </select>
          </label>
        </div>

        <label className="field">
          <span>What did you do, and what happened?</span>
          <textarea rows={6} maxLength={8000} placeholder="I clicked …, and then …" value={message} onChange={(e) => setMessage(e.target.value)} />
          <small className="muted">Please don't include real passwords, bank details or other personal information.</small>
        </label>

        <label className="field">
          <span>Your email, if you'd like a reply <span className="muted">(optional)</span></span>
          <input type="email" maxLength={120} autoComplete="email" value={contact} onChange={(e) => setContact(e.target.value)} />
          {!contactOk && <small className="error-text">That email address doesn't look right.</small>}
        </label>

        <div className="field">
          <span>Screenshot <span className="muted">(optional)</span></span>
          <div className="toolbar">
            {screenshot ? (
              <>
                <span className="row-name small"><Icon name="save" size={14} />{fileName}</span>
                <button className="quiet small" onClick={() => setScreenshot(null)}>Remove</button>
              </>
            ) : (
              <button onClick={() => pickScreenshot().then((p) => p && setScreenshot(p)).catch((e) => setResult({ ok: false, text: String(e) }))}>
                Choose a picture
              </button>
            )}
          </div>
          <small className="muted">PNG or JPG up to 4 MB. Check it doesn't show real passwords or personal details.</small>
        </div>

        <label className="check">
          <input type="checkbox" checked={withDetails} onChange={(e) => setWithDetails(e.target.checked)} />
          <span>
            Add technical details
            <small className="muted">
              The last lines of Sompio's problems log: error messages only, never your passwords, messages or files, and with your user name taken out.{" "}
              <button className="link small" onClick={(e) => { e.preventDefault(); setShowDetails((s) => !s); }}>
                {showDetails ? "Hide them" : "Show them"}
              </button>
            </small>
          </span>
        </label>
        {showDetails && (
          <textarea readOnly rows={8} className="mono small" value={about ? about.recent.join("\n") || "(empty)" : "Couldn't read the problems log."} />
        )}

        <div className="toolbar">
          <button className="primary" disabled={!canSend} onClick={send}>{busy ? "Sending…" : "Send"}</button>
          <span className="small muted">
            Sent to sompio.org and emailed to us{about ? `, with "Sompio ${about.version} on ${about.os}"` : ""}. Not kept on the website.
          </span>
        </div>
        {result && <div className={result.ok ? "success" : "error"} role="status">{result.text}</div>}
      </section>

      <p className="small muted">You can also email <strong>hello@sompio.org</strong>.</p>
    </div>
  );
}
