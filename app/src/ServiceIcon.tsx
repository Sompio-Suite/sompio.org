import { BRANDS } from "./brands";
import Icon from "./Icon";

/** Email providers by their id in Sompio, with the logo to show for each. */
const MAIL_BRAND: Record<string, string> = { gmail: "gmail", proton: "proton", icloud: "icloud", gmx: "gmx", tuta: "tuta" };
const MAIL_NAME: Record<string, string> = {
  gmail: "Gmail", proton: "Proton Mail", yahoo: "Yahoo Mail", icloud: "iCloud Mail", gmx: "GMX", fastmail: "Fastmail",
  mailboxorg: "mailbox.org", posteo: "Posteo", runbox: "Runbox", startmail: "StartMail", tuta: "Tuta", other: "Email",
};

/** The email provider behind a finding: Gmail if its link is a Gmail link, else the connected account's. */
export function mailProvider(link: string | null | undefined, connected: string | null): string {
  if (link?.includes("mail.google.com")) return "gmail";
  return connected ?? "other";
}
export const mailName = (provider: string) => MAIL_NAME[provider] ?? "Email";

/**
 * A service's logo, small and in its own colour, or a plain letter when there's no logo to show.
 * `id` is a platform ("discord"), a brand key ("gmail"), or "mail:<provider>" for an email account.
 */
export default function ServiceIcon({ id, size = 16, name }: { id: string; size?: number; name?: string }) {
  if (id === "pc") return <Icon name="folder" size={size} />;
  const key = id.startsWith("mail:") ? MAIL_BRAND[id.slice(5)] ?? null : id === "email" ? null : id;
  const brand = key ? BRANDS[key] : undefined;
  const label = name ?? brand?.title ?? (id.startsWith("mail:") ? mailName(id.slice(5)) : id);
  if (!brand) {
    if (id === "email" || id === "mail:other") return <Icon name="mail" size={size} />;
    return (
      <span className="svc-letter" style={{ width: size, height: size, fontSize: size * 0.62 }} role="img" aria-label={label} title={label}>
        {label.charAt(0).toUpperCase()}
      </span>
    );
  }
  // Near-black logos (X, TikTok) follow the text colour, so they stay visible in dark mode.
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(brand.hex.slice(i, i + 2), 16));
  const dark = 0.299 * r + 0.587 * g + 0.114 * b < 50;
  return (
    <svg className="svc-icon" width={size} height={size} viewBox="0 0 24 24" role="img" aria-label={label} style={{ color: dark ? "var(--ink)" : brand.hex }}>
      <title>{label}</title>
      <path d={brand.path} fill="currentColor" />
    </svg>
  );
}
