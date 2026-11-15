<?php
// Guide: moving saved passwords out of Chrome, Edge, Brave or Firefox, safely.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('How to move your passwords out of Chrome, Edge or Firefox · Sompio', 'Export the passwords your browser saved, bring them into a password manager, and clean up afterwards, without leaving an unprotected copy behind.');
site_header('guide');
?>
  <main class="wrap docs guide">
    <aside class="docs-toc" aria-label="On this page">
      <p class="toc-title">On this page</p>
      <a href="#why">Why move them</a>
      <a href="#export">1. Export them</a>
      <a href="#import">2. Bring them in</a>
      <a href="#delete">3. Delete the file</a>
      <a href="#browser">4. Clean up the browser</a>
      <a href="#after">Afterwards</a>
      <a href="guides.php" class="toc-back">← All guides</a>
    </aside>

    <article class="docs-body">
      <p class="eyebrow">Passwords · updated November 2026</p>
      <h1>How to move your passwords out of Chrome, Edge or Firefox</h1>
      <p class="lead">Letting the browser remember passwords is easy, so years of logins end up there. Moving them to a password manager takes about ten minutes. The one thing to get right: the export file isn't protected, so it shouldn't stay on your computer.</p>

      <section id="why">
        <h2>Why move them</h2>
        <ul class="points">
          <li><strong>One place for everything.</strong> A password manager also keeps passwords for apps, Wi-Fi and cards, works in every browser, and tells you which passwords are weak or reused.</li>
          <li><strong>Browsers are a common target.</strong> Password-stealing malware often goes straight for the passwords saved in browsers.</li>
          <li><strong>You choose where they live.</strong> Synced browser passwords are tied to your Google, Microsoft or Mozilla account. A local password manager keeps them on your own computer.</li>
        </ul>
      </section>

      <section id="export">
        <h2>1. Export them from your browser</h2>
        <ul class="points">
          <li><strong>Chrome:</strong> open <code>chrome://password-manager/settings</code>, click <strong>Export passwords</strong>, then <strong>Download file</strong>. Chrome asks for your computer's password first.</li>
          <li><strong>Edge:</strong> open <code>edge://wallet/passwords</code>, open the menu or settings for saved passwords, and choose <strong>Export passwords</strong>.</li>
          <li><strong>Brave:</strong> open <code>brave://password-manager/settings</code> and click <strong>Export passwords</strong>.</li>
          <li><strong>Firefox:</strong> open <code>about:logins</code>, click <strong>⋯</strong> at the top right, then <strong>Export Passwords</strong>.</li>
        </ul>
        <p>Each browser saves a <strong>.csv</strong> file, usually in your Downloads folder. Menus move around between versions; if a step doesn't match, search the browser's help for "export passwords".</p>
        <p class="callout"><strong>This file isn't encrypted.</strong> Anyone who opens it sees every password in plain text. Don't email it, upload it or keep it in a synced folder like OneDrive, Dropbox or iCloud.</p>
      </section>

      <section id="import">
        <h2>2. Bring them into a password manager</h2>
        <p>Almost every password manager can import this file. In <a href="docs.php#vault">Sompio Vault</a>:</p>
        <ol class="guide-steps">
          <li>Open Vault and go to <strong>Auto-type, import and export</strong>.</li>
          <li>Under <strong>From your browser</strong>, click <strong>Choose the exported file</strong> and pick the .csv file.</li>
          <li>Sompio brings everything in and says how many passwords it found. Check a few to be sure.</li>
        </ol>
        <p>Vault saves them in the open KeePass format, so you can move them again later to any KeePass-compatible app.</p>
      </section>

      <section id="delete">
        <h2>3. Delete the export file</h2>
        <ol class="guide-steps">
          <li>Delete the .csv file from your Downloads folder.</li>
          <li>Empty the Recycle Bin (Windows) or Bin (Mac).</li>
          <li>If the file ended up in a synced folder, delete it there too, including the cloud service's own "deleted files" or "trash".</li>
        </ol>
      </section>

      <section id="browser">
        <h2>4. Clean up the browser</h2>
        <p>Once you've checked the passwords arrived safely:</p>
        <ul class="points">
          <li><strong>Stop saving new ones.</strong> Chrome and Brave: in the password manager settings, turn off <strong>Offer to save passwords</strong>. Edge: the same option is in its password settings. Firefox: Settings, <strong>Privacy &amp; Security</strong>, untick <strong>Ask to save passwords</strong>.</li>
          <li><strong>Remove the saved ones.</strong> Chrome, Edge and Brave: delete browsing data, choose <strong>All time</strong> and only <strong>Passwords and other sign-in data</strong>. Firefox: in <code>about:logins</code>, click <strong>⋯</strong>, then <strong>Remove All Passwords</strong>.</li>
          <li><strong>If you used sync,</strong> removing them in the browser removes them from your account too. For Chrome, you can check at <code>passwords.google.com</code>.</li>
        </ul>
      </section>

      <section id="after">
        <h2>Afterwards</h2>
        <ul class="points">
          <li>Install your password manager's browser extension so it fills in logins for you. Sompio's works in Chrome, Edge, Brave and Firefox and talks only to Sompio on your computer.</li>
          <li>Run the health check and start with the passwords it marks as reused or breached.</li>
          <li>Keep a backup of your password file somewhere safe, off your computer.</li>
        </ul>
      </section>

<?php more_guides('move-passwords-out-of-browser.php'); ?>
    </article>
  </main>
<?php site_footer(); ?>
