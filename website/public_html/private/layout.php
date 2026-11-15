<?php
// The parts every page shares: <head>, the top menu and the footer.
declare(strict_types=1);

const SOURCE_URL = 'https://github.com/Sompio-Suite/sompio.org';
const DISCORD_URL = 'https://discord.gg/M2UKWaqpxP';
// Discord's mark in its own colour, linking to the Sompio server (announcements and community)
const DISCORD_ICON = '<svg class="dc-icon" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path fill="#5865F2" d="M20.317 4.37a19.79 19.79 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.865-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.74 19.74 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.1 13.1 0 0 1-1.872-.892.077.077 0 0 1-.008-.128c.126-.094.252-.192.372-.291a.074.074 0 0 1 .078-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.3 12.3 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.84 19.84 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>';
/** Prices in euros, including VAT, for the pricing and terms pages waiting in private/drafts (not shown yet). */
const PRICES = ['month' => 4, 'year' => 30, 'forever' => 50];
/** The guides, newest first. guides.php shows them as cards; each guide page links to the others. */
const GUIDES = [
    ['delete-discord-messages.php', 'How to delete your old Discord messages', 'Discord has no "delete all" button, and deleting your account leaves your messages behind. What actually works, step by step.', 'Discord', 6],
    ['discord-data-package.php', "What's in your Discord data package", 'Every message you ever sent, and more. How to request your copy, what each folder holds, and what to look for.', 'Discord', 4],
    ['delete-reddit-comments.php', 'How to delete your old Reddit posts and comments', 'Find the posts that give away too much, delete them properly, and know what deleting your account does and doesn\'t do.', 'Reddit', 5],
    ['make-companies-delete-your-data.php', 'How to make companies delete your data', 'The GDPR gives you the right to see and erase what companies hold about you. How to ask, what they must do, and what to do if they ignore you.', 'Your rights', 6],
];

const SUPPORT_URL = 'https://ko-fi.com/7ab9ne3021sn4b';
const HEART_ICON = '<svg class="heart-icon" viewBox="0 0 24 24" width="15" height="15" aria-hidden="true"><path fill="currentColor" d="M12 21s-7.5-4.6-9.6-9.3C.9 8.2 3 4.5 6.6 4.5c2.1 0 3.6 1.2 4.4 2.5.8-1.3 2.3-2.5 4.4-2.5 3.6 0 5.7 3.7 4.2 7.2C19.5 16.4 12 21 12 21z"/></svg>';
/** A plain onion outline: the menu's "Open in Tor" link */
const TOR_ICON = '<svg class="tor-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path fill="#7d4698" d="M8 2.2c.5 1 1.3 1.8 2.3 2.5C12.5 6.1 14 8 14 10.3 14 13.4 11.3 15.6 8 15.6S2 13.4 2 10.3C2 8 3.5 6.1 5.7 4.7 6.7 4 7.5 3.2 8 2.2z"/><path fill="none" stroke="#fff" stroke-width="1.1" stroke-linecap="round" d="M8 6.2c-1.9 1-3.1 2.4-3.1 4.1 0 1.8 1.4 3.1 3.1 3.1"/><path fill="none" stroke="#fff" stroke-width="1.1" stroke-linecap="round" d="M8 8.6c-.9.6-1.4 1.2-1.4 1.9 0 .7.6 1.2 1.4 1.2"/></svg>';
/** A shield with a tick: next to download fingerprints */
const SHIELD_ICON = '<svg viewBox="0 0 24 24" width="15" height="15" aria-hidden="true"><path d="M12 3l7 3v5c0 4.5-3 8.3-7 10-4-1.7-7-5.5-7-10V6z M9 12l2 2 4-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
const GITHUB_ICON = '<svg class="gh-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>';

/**
 * Opens the page. $public pages may be found by search engines; the rest are kept out of them.
 */
function page_head(string $title, string $description, bool $public = true): void
{
    // Tor Browser shows ".onion available" when the site has an onion address (set in config.php)
    $onion = onion_address();
    if ($onion !== null && !headers_sent()) {
        $here = strtok((string) ($_SERVER['REQUEST_URI'] ?? '/'), '#') ?: '/';
        header('Onion-Location: http://' . $onion . $here);
    }
    $css = 'style.css?v=' . filemtime(dirname(__DIR__) . '/style.css');
    // The address of this page on sompio.org, for link previews (front page without "index.php")
    $path = strtok((string) ($_SERVER['REQUEST_URI'] ?? '/'), '?#') ?: '/';
    $url = 'https://sompio.org' . (preg_match('#/index\.php$#', $path) ? substr($path, 0, -9) : $path);
    ?>
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <?php if (!$public): ?><meta name="robots" content="noindex, nofollow">
  <?php endif; ?><title><?= e($title) ?></title>
  <meta name="description" content="<?= e($description) ?>">
  <?php if ($public): ?>
  <meta property="og:type" content="website">
  <meta property="og:site_name" content="Sompio">
  <meta property="og:title" content="<?= e($title) ?>">
  <meta property="og:description" content="<?= e($description) ?>">
  <meta property="og:url" content="<?= e($url) ?>">
  <meta property="og:image" content="https://sompio.org/img/og.png">
  <meta property="og:image:width" content="1200">
  <meta property="og:image:height" content="630">
  <meta property="og:image:alt" content="Sompio: private tools for your computer. Nothing leaves it.">
  <meta name="twitter:card" content="summary_large_image">
  <link rel="canonical" href="<?= e($url) ?>">
  <?php endif; ?>
  <link rel="icon" type="image/svg+xml" href="logo.svg">
  <link rel="stylesheet" href="<?= e($css) ?>">
</head>
<body>
<?php
}

/** sompio.org's onion address for Tor Browser, once one is set up ('onion' in config.php). */
function onion_address(): ?string
{
    $o = strtolower(trim((string) (config()['onion'] ?? '')));
    return preg_match('/^[a-z2-7]{56}\.onion$/', $o) ? $o : null;
}

/** The top menu. $here is the page you're on: home, guide, docs, download or feedback. */
function site_header(string $here): void
{
    $link = function (string $href, string $label, string $key) use ($here): string {
        $cur = $key === $here ? ' aria-current="page"' : '';
        return '<a href="' . e($href) . '"' . $cur . '>' . e($label) . '</a>';
    };
    ?>
  <header class="top">
    <div class="wrap top-row">
      <a class="brand" href="./"><img src="logo.svg" alt="" width="28" height="28"> sompio <span class="tag">beta</span></a>
      <nav aria-label="Site">
        <?= $link('./#tools', 'Features', 'features') ?>
        <?= $link('guides.php', 'Guides', 'guide') ?>
        <?= $link('docs.php', 'Documentation', 'docs') ?>
        <a href="<?= e(SOURCE_URL) ?>" class="gh-link" rel="noopener noreferrer" aria-label="Source code on GitHub" title="Source code on GitHub"><?= GITHUB_ICON ?><span class="gh-label">Source code</span></a>
        <a href="tor.php" class="gh-link tor-link"<?= $here === 'tor' ? ' aria-current="page"' : '' ?> aria-label="Open in Tor" title="Open sompio.org in Tor Browser"><?= TOR_ICON ?><span class="gh-label">Open in Tor</span></a>
        <a class="nav-tester<?= $here === 'download' ? ' on' : '' ?>" href="download.php"<?= $here === 'download' ? ' aria-current="page"' : '' ?>>Download</a>
      </nav>
    </div>
  </header>
<?php
}

/** "More guides" at the end of a guide: every guide except the one you're on. */
function more_guides(string $current): void
{
    ?>
      <section class="more-guides">
        <h2>More guides</h2>
        <div class="guide-grid small">
          <?php foreach (GUIDES as [$href, $title, $summary, $tag, $min]): if ($href === $current) continue; ?>
            <a class="guide-card-link" href="<?= e($href) ?>">
              <span class="guide-tag"><?= e($tag) ?></span>
              <strong><?= e($title) ?></strong>
              <span class="guide-more">Read guide →</span>
            </a>
          <?php endforeach; ?>
        </div>
      </section>
<?php
}

function site_footer(): void
{
    ?>
  <footer class="site-foot">
    <div class="wrap foot-row">
      <div class="foot-brand"><img src="logo.svg" alt="" width="22" height="22"> <strong>sompio</strong> <span class="muted">Private tools for your computer.</span></div>
      <nav aria-label="More">
        <a href="docs.php">Documentation</a>
        <a href="guides.php">Guides</a>
        <a href="<?= e(SOURCE_URL) ?>" class="gh-link" rel="noopener noreferrer"><?= GITHUB_ICON ?>Source code</a>
        <a href="<?= e(DISCORD_URL) ?>" class="gh-link" rel="noopener noreferrer"><?= DISCORD_ICON ?>Discord</a>
        <a href="feedback.php">Report a bug</a>
        <a href="security.php">Security</a>
        <a href="privacy.php">Privacy</a>
        <a href="mailto:hello@sompio.org">hello@sompio.org</a>
        <a href="<?= e(SUPPORT_URL) ?>" class="support-link" rel="noopener noreferrer"><?= HEART_ICON ?>Support Sompio</a>
      </nav>
      <p class="small muted">This site has no trackers, no ads, no analytics and no tracking cookies.</p>
    </div>
  </footer>
</body>
</html>
<?php
}
