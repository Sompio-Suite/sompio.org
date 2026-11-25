import { useEffect, useState } from "react";
import { api } from "./api";

/** "Start Sompio when my computer starts": it then waits locked in the tray, ready for the browser extension. */
export default function StartWithComputer({ hint }: { hint?: string }) {
  const [on, setOn] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    api.calAutostart().then(setOn).catch(() => setOn(false));
  }, []);
  if (on === null) return null;
  return (
    <div className="start-with">
      <label className="check">
        <input type="checkbox" checked={on} onChange={async (e) => {
          const next = e.target.checked;
          try {
            await api.calSetAutostart(next);
            setOn(next);
            setError(null);
          } catch (err) {
            setError(String(err));
          }
        }} />
        Start Sompio when my computer starts
      </label>
      <p className="small muted">
        {hint ?? "It starts locked and waits in the tray (next to the clock), without opening a window."} Closing the window keeps it
        there too; choose Quit Sompio in the tray to close it completely.
      </p>
      {error && <p className="small error">{error}</p>}
    </div>
  );
}
