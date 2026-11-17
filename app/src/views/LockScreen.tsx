import { useEffect, useState } from "react";
import { api, listen, onFileDrop, passwordProblem, pickPicture, pickPictureSave, type LockStatus } from "../api";
import Icon from "../Icon";
import Logo from "../Logo";

interface Props {
  status: LockStatus;
  onUnlocked: () => void;
  /** What's behind the lock here, like "Vault". Without it: Vault and Trace. */
  reason?: string;
}

type Mode = "setup" | "unlock" | "recover";
type SetupStep = "form" | "sheet" | "phone";

function Mark({ size = 56 }: { size?: number }) {
  return <Logo size={size} className="lock-mark" />;
}

function PasswordFields(p: { pw: string; setPw: (s: string) => void; again: string; setAgain: (s: string) => void; label?: string }) {
  const problem = p.pw ? passwordProblem(p.pw) : null;
  const mismatch = p.again.length > 0 && p.again !== p.pw;
  return (
    <>
      <label className="field">
        <span>{p.label ?? "Password"}</span>
        <input type="password" autoComplete="new-password" value={p.pw} onChange={(e) => p.setPw(e.target.value)} autoFocus />
        <small className={problem ? "warn-text" : "muted"}>{problem ?? "At least 10 characters. A short sentence or a few unrelated words is easiest to remember."}</small>
      </label>
      <label className="field">
        <span>Type it again</span>
        <input type="password" autoComplete="new-password" value={p.again} onChange={(e) => p.setAgain(e.target.value)} />
        {mismatch && <small className="warn-text">These don't match yet.</small>}
      </label>
    </>
  );
}

function RecoverySheet({ code, picture, onDone }: { code: string; picture: boolean; onDone: () => void }) {
  const [kept, setKept] = useState(false);
  return (
    <div className="lock-step">
      <p className="step-count">Last step</p>
      <h1>Your recovery code</h1>
      <p className="muted">
        {picture
          ? "If you ever forget your password or lose the key picture, this code is the only way back in."
          : "If you ever forget your password, or need Sompio on a new computer, this code is the only way back in."}{" "}
        Nobody can reset it for you, not even us. Print it or write it down, and keep it somewhere safe at home, away from this computer.
      </p>
      <div className="recovery-sheet" id="recovery-sheet">
        <div className="sheet-head"><Mark size={28} /><strong>Sompio recovery sheet</strong></div>
        <div className="recovery-code selectable">{code}</div>
        <p className="small">Use it in Sompio under "Use your recovery code". Letters and numbers only; dashes and capitals don't matter.</p>
        <p className="small sheet-date">Made {new Date().toLocaleDateString()}</p>
      </div>
      <div className="toolbar">
        <button onClick={() => window.print()}><Icon name="copy" size={16} />Print recovery sheet</button>
      </div>
      <label className="check">
        <input type="checkbox" checked={kept} onChange={(e) => setKept(e.target.checked)} />
        I've printed it or written it down, and I'll keep it safe.
      </label>
      <button className="primary big" disabled={!kept} onClick={onDone}>Continue</button>
    </div>
  );
}

function PhoneStep({ path, onDone }: { path: string; onDone: () => void }) {
  return (
    <div className="lock-step">
      <h1>Put the key picture on your phone</h1>
      <p className="muted">Your key picture is saved at:</p>
      <code className="path selectable">{path}</code>
      <ul className="plan-list">
        <li><Icon name="check" size={16} /><span>Move it to your phone <strong>as a file</strong>: by cable, AirDrop, Nearby Share, or "Save to Files".</span></li>
        <li><Icon name="check" size={16} />Don't send it through WhatsApp, Messenger or similar. They shrink pictures, which erases the key inside.</li>
        <li><Icon name="check" size={16} />It looks like an ordinary night-sky wallpaper. Keep it with your photos; nobody needs to know what it is.</li>
        <li><Icon name="check" size={16} />Once it's on your phone, you can delete it from this computer.</li>
      </ul>
      <button className="primary big" onClick={onDone}>Continue</button>
    </div>
  );
}

export default function LockScreen({ status, onUnlocked }: Props) {
  const [mode, setMode] = useState<Mode>(status.set_up ? "unlock" : "setup");
  const [step, setStep] = useState<SetupStep>("form");
  const [withPicture, setWithPicture] = useState(false);
  // Recovery: a new picture file, or this computer keeps the new key (the usual case without one)
  const [newPicture, setNewPicture] = useState(!status.remembers_picture);
  const [pw, setPw] = useState("");
  const [again, setAgain] = useState("");
  const [code, setCode] = useState("");
  const [picture, setPicture] = useState<string | null>(null);
  const [remember, setRemember] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newCode, setNewCode] = useState<string | null>(null);
  const [savedTo, setSavedTo] = useState<string | null>(null);
  const [over, setOver] = useState(false);
  const hasKeys = status.hardware_keys.length > 0;
  // With a YubiKey added and no remembered picture, the YubiKey is the usual way in
  const [useKey, setUseKey] = useState(hasKeys && !status.remembers_picture);
  const [touch, setTouch] = useState(false);
  useEffect(() => {
    const un = listen("yubikey-touch", () => setTouch(true));
    return () => void un.then((f) => f());
  }, []);

  useEffect(() => {
    const un = onFileDrop((p) => p.toLowerCase().endsWith(".png") && setPicture(p), setOver);
    return () => void un.then((f) => f());
  }, []);

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const ready = !passwordProblem(pw) && pw === again;

  const createLock = () =>
    run(async () => {
      let c: string;
      if (withPicture) {
        const path = await pickPictureSave();
        if (!path) return;
        await api.lockNewPicture(path);
        c = await api.lockSetupWithPicture(pw, true);
        setSavedTo(path);
      } else {
        c = await api.lockSetupPlain(pw);
        setSavedTo(null);
      }
      setPw("");
      setAgain("");
      setNewCode(c);
      setStep("sheet");
    });

  const unlock = () =>
    run(async () => {
      try {
        if (useKey) await api.lockUnlockYubikey(pw);
        else await api.lockUnlock(pw, picture, remember);
      } finally {
        setTouch(false);
      }
      setPw("");
      onUnlocked();
    });

  const recover = () =>
    run(async () => {
      let path: string | null = null;
      if (newPicture) {
        path = await pickPictureSave();
        if (!path) return;
      }
      const c = await api.lockRecover(code, pw, path, remember);
      setPw("");
      setAgain("");
      setCode("");
      setSavedTo(path);
      setNewCode(c);
      setStep("sheet");
    });

  // After setup or recovery: the sheet, then (with a key picture) where to keep it
  if (newCode && step === "sheet") {
    return (
      <div className="lock-screen">
        <RecoverySheet code={newCode} picture={!!savedTo} onDone={() => (savedTo ? setStep("phone") : (setNewCode(null), onUnlocked()))} />
      </div>
    );
  }
  if (newCode && step === "phone" && savedTo) {
    return <div className="lock-screen"><PhoneStep path={savedTo} onDone={() => { setNewCode(null); onUnlocked(); }} /></div>;
  }

  const rememberBox = (
    <label className="check">
      <input type="checkbox" checked={remember} onChange={(e) => setRemember(e.target.checked)} />
      <span>
        Remember the key picture on this computer
        <small className="muted"> Then you only type your password here. Leave it off on a shared or work computer.</small>
      </span>
    </label>
  );

  return (
    <div className={`lock-screen${over ? " over" : ""}`}>
      {mode === "setup" && (
        <form className="lock-step" onSubmit={(e) => { e.preventDefault(); if (ready) createLock(); }}>
          <Mark />
          <h1>Set up your lock</h1>
          <p className="muted">
            Vault and Trace hold things only you should see, so they open with a lock. One password opens both.
          </p>
          <ul className="lock-rules">
            <li><Icon name="lock" size={15} />Only you can open it. Nobody can reset it, not even us.</li>
            <li><Icon name="key" size={15} />Forget the password, and the recovery code you get next is the only way back in.</li>
          </ul>
          <PasswordFields pw={pw} setPw={setPw} again={again} setAgain={setAgain} label="Choose a password" />
          <label className="check">
            <input type="checkbox" checked={withPicture} onChange={(e) => setWithPicture(e.target.checked)} />
            <span>
              Also use a key picture <span className="muted">(recommended)</span>
              <small className="muted">
                A picture file that works as a second key: your password alone can't open Sompio on another computer. You save it, and keep
                a copy off this computer. Don't edit it or send it through chat apps; that breaks the key.
              </small>
            </span>
          </label>
          {error && <div className="error">{error}</div>}
          <div className="toolbar">
            <button className="primary big" disabled={!ready || busy}>{busy ? "Creating the lock…" : withPicture ? "Save the key picture and create the lock…" : "Create the lock"}</button>
          </div>
        </form>
      )}

      {mode === "unlock" && (
        <form className="lock-step narrow" onSubmit={(e) => { e.preventDefault(); if (pw) unlock(); }}>
          <Mark />
          <h1>Vault and Trace are locked</h1>
          <label className="field">
            <span>Password</span>
            <input type="password" autoComplete="current-password" value={pw} onChange={(e) => setPw(e.target.value)} autoFocus />
          </label>
          {hasKeys && (
            <div className="segmented" role="radiogroup" aria-label="Second key">
              <button type="button" role="radio" aria-checked={useKey} aria-selected={useKey} onClick={() => setUseKey(true)}>YubiKey</button>
              <button type="button" role="radio" aria-checked={!useKey} aria-selected={!useKey} onClick={() => setUseKey(false)}>Key picture</button>
            </div>
          )}
          {useKey && (
            <div className={`picture-pick${touch ? " done" : ""}`}>
              <Icon name="key" />
              <div>
                <strong>{touch ? "Touch your YubiKey now" : "Plug in your YubiKey"}</strong>
                <span className="small muted">{touch ? "It's blinking: touch the gold disc." : status.hardware_keys.map((k) => k.label).join(", ")}</span>
              </div>
            </div>
          )}
          {!useKey && !status.remembers_picture && (
            <div className={`picture-pick${picture ? " done" : ""}`}>
              <Icon name={picture ? "check" : "eye"} />
              <div>
                <strong>{picture ? "Key picture ready" : "Your key picture"}</strong>
                <span className="small muted">{picture ? picture.split(/[\\/]/).pop() : "Choose it, or drop it here. Use the original file from your phone."}</span>
              </div>
              <button type="button" onClick={async () => setPicture(await pickPicture())}>{picture ? "Change" : "Choose…"}</button>
            </div>
          )}
          {!useKey && !status.remembers_picture && picture && rememberBox}
          {error && <div className="error">{error}</div>}
          <button className="primary big" disabled={busy || !pw || (!useKey && !status.remembers_picture && !picture)}>{busy ? (useKey ? "Asking the YubiKey…" : "Unlocking…") : "Unlock"}</button>
          <button type="button" className="link" onClick={() => { setMode("recover"); setError(null); setPw(""); }}>
            {hasKeys ? "Forgot your password, or lost the picture and YubiKey?" : status.remembers_picture ? "Forgot your password?" : "Forgot your password or lost the picture?"}
          </button>
        </form>
      )}

      {mode === "recover" && (
        <form className="lock-step" onSubmit={(e) => { e.preventDefault(); if (ready && code) recover(); }}>
          <h1>Use your recovery code</h1>
          <p className="muted">Your data stays as it is. You choose a new password and get a new recovery code; the old password, key picture and code stop working.</p>
          <label className="field">
            <span>Recovery code</span>
            <input type="text" className="mono" autoComplete="off" spellCheck={false} placeholder="XXXX-XXXX-XXXX-XXXX-XXXX-XXXX" value={code} onChange={(e) => setCode(e.target.value)} autoFocus />
          </label>
          <PasswordFields pw={pw} setPw={setPw} again={again} setAgain={setAgain} label="New password" />
          <label className="check">
            <input type="checkbox" checked={newPicture} onChange={(e) => setNewPicture(e.target.checked)} />
            <span>
              Make a new key picture too
              <small className="muted">Leave it off to open Sompio with the password alone on this computer.</small>
            </span>
          </label>
          {newPicture && rememberBox}
          {error && <div className="error">{error}</div>}
          <div className="toolbar">
            <button className="primary big" disabled={!ready || !code || busy}>{busy ? "Checking…" : newPicture ? "Save my new key picture…" : "Set my new password"}</button>
            <button type="button" className="quiet" onClick={() => { setMode("unlock"); setError(null); }}>Back</button>
          </div>
        </form>
      )}
    </div>
  );
}
