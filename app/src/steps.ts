// The four Getting started steps. Find and Clean are read from Trace; Learn and Secure are
// remembered on this computer once they happen (reading the basics, a password in Vault).
type Flag = "learned" | "secured" | "started";

const read = (k: Flag) => {
  try {
    return localStorage.getItem(`sompio.${k}`) === "1";
  } catch {
    return false;
  }
};

export function markStep(k: Flag) {
  if (read(k)) return;
  try {
    localStorage.setItem(`sompio.${k}`, "1");
  } catch {
    /* only shown as not done */
  }
  window.dispatchEvent(new Event("sompio-steps"));
}

/** All four done once: Getting started moves under ? for good */
export const allDoneOnce = () => read("started");

export function stepsDone(found: boolean, cleaned: boolean) {
  return { learn: read("learned"), find: found, clean: cleaned, secure: read("secured") };
}
