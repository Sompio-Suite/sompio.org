<?php
// The download section on the front page: the newest version for each system, its fingerprint,
// and what to click when the computer warns about a new program. Needs $r = latest_release().
// Show the visitor's system straight away, so the page doesn't jump while os.js loads
$ua = (string) ($_SERVER['HTTP_USER_AGENT'] ?? '');
$osGuess = preg_match('/Mac OS X|Macintosh/', $ua) && !preg_match('/iPhone|iPad/', $ua) && isset($r['platforms']['mac']) ? 'mac'
    : (preg_match('/Linux/', $ua) && !preg_match('/Android/', $ua) && (isset($r['platforms']['linux_appimage']) || isset($r['platforms']['linux_deb'])) ? 'linux' : 'windows');
$hide = fn(string $os) => $os === $osGuess ? '' : ' hidden';
$tabOn = fn(string $os) => $os === $osGuess ? ' on" aria-selected="true' : '" aria-selected="false';

?>
    <div class="hero">
      <div class="hero-intro">
        <h2 class="dl-title">Download Sompio</h2>
        <p class="lead">Everything is unlocked during the beta, free, no key or account needed. When Sompio launches, you get the 30-day free trial on top.</p>
        <ul class="ticks">
          <li>Trace and Vault in one program</li>
          <li>No account and no cloud: your data stays on your computer, encrypted</li>
          <li>New versions come to you inside Sompio</li>
        </ul>
        <div class="beta-note">
          <h2>It's a beta</h2>
          <p>Sompio works, but things can still change. Keep the recovery sheet Sompio gives you when you set up the lock, and now and then save a copy of your passwords from Vault. Found a problem? Click <strong>Report a bug</strong> at the top of Sompio.</p>
        </div>
      </div>
      <div class="card download" id="top-download">
        <?php if ($r): $pl = $r['platforms']; $hasMac = isset($pl['mac']); $hasLinux = isset($pl['linux_appimage']) || isset($pl['linux_deb']); ?>
          <h2>Sompio <?= e($r['version']) ?></h2>
          <?php if ($hasMac || $hasLinux): ?>
            <div class="os-tabs" role="tablist" aria-label="Your system">
              <a href="#for-windows" class="os-tab<?= $tabOn('windows') ?>" data-os="windows" role="tab">Windows</a>
              <?php if ($hasMac): ?><a href="#for-mac" class="os-tab<?= $tabOn('mac') ?>" data-os="mac" role="tab">Mac</a><?php endif; ?>
              <?php if ($hasLinux): ?><a href="#for-linux" class="os-tab<?= $tabOn('linux') ?>" data-os="linux" role="tab">Linux</a><?php endif; ?>
            </div>
          <?php endif; ?>

          <div class="os-panel" id="for-windows" data-os="windows"<?= $hide('windows') ?>>
            <p class="small muted">Windows 10 and 11, 64-bit · <?= e(human_size($r['size'])) ?> · <?= e(date('j M Y', $r['date'])) ?></p>
            <a class="button primary big" href="download.php?get=1" rel="nofollow">Download for Windows</a>
            <div class="fingerprint">
              <p class="fp-label"><?= SHIELD_ICON ?>File fingerprint (SHA-256)</p>
              <code class="hash"><?= e(strtoupper($r['sha256'])) ?></code>
              <p class="small muted">To check: run <code class="nowrap">Get-FileHash .\<?= e($r['name']) ?></code> in PowerShell. It must match this and <a href="<?= e(SOURCE_URL) ?>/releases/tag/v<?= e($r['version']) ?>" rel="noopener noreferrer">GitHub</a>.</p>
            </div>
          </div>

          <?php if ($hasMac): $m = $pl['mac']; ?>
          <div class="os-panel" id="for-mac" data-os="mac"<?= $hide('mac') ?>>
            <p class="small muted">macOS 11 or newer, Apple silicon and Intel · <?= e(human_size($m['size'])) ?></p>
            <a class="button primary big" href="<?= e($m['url']) ?>" rel="nofollow noopener noreferrer">Download for Mac</a>
            <div class="fingerprint">
              <p class="fp-label"><?= SHIELD_ICON ?>File fingerprint (SHA-256)</p>
              <code class="hash"><?= e(strtoupper($m['sha256'])) ?></code>
              <p class="small muted">To check: run <code class="nowrap">shasum -a 256</code> on the file in Terminal. It must match this and <a href="<?= e(SOURCE_URL) ?>/releases/tag/v<?= e($r['version']) ?>" rel="noopener noreferrer">GitHub</a>.</p>
            </div>
          </div>
          <?php endif; ?>

          <?php if ($hasLinux): ?>
          <div class="os-panel" id="for-linux" data-os="linux"<?= $hide('linux') ?>>
            <p class="small muted">64-bit Linux, like Ubuntu, Debian or Fedora<?= isset($pl['linux_appimage']) ? ' · ' . e(human_size($pl['linux_appimage']['size'])) : '' ?></p>
            <?php if (isset($pl['linux_appimage'])): $f = $pl['linux_appimage']; ?>
              <a class="button primary big" href="<?= e($f['url']) ?>" rel="nofollow noopener noreferrer">Download for Linux (AppImage)</a>
            <?php elseif (isset($pl['linux_deb'])): $f = $pl['linux_deb']; ?>
              <a class="button primary big" href="<?= e($f['url']) ?>" rel="nofollow noopener noreferrer">Download for Ubuntu and Debian (.deb)</a>
            <?php endif; ?>
            <div class="fingerprint">
              <p class="fp-label"><?= SHIELD_ICON ?>File fingerprint (SHA-256)</p>
              <?php if (isset($pl['linux_appimage'])): ?><code class="hash"><?= e(strtoupper($pl['linux_appimage']['sha256'])) ?></code><?php endif; ?>
              <p class="small muted">To check: run <code>sha256sum</code> on the file. It must match this and <a href="<?= e(SOURCE_URL) ?>/releases/tag/v<?= e($r['version']) ?>" rel="noopener noreferrer">GitHub</a>.<?php if (isset($pl['linux_deb'])): ?> Prefer a package? Get the <a href="<?= e($pl['linux_deb']['url']) ?>" rel="nofollow noopener noreferrer">.deb for Ubuntu and Debian</a> (<?= e(human_size($pl['linux_deb']['size'])) ?>); its fingerprint is on GitHub.<?php endif; ?></p>
            </div>
          </div>
          <?php endif; ?>
          <p class="phone-note small">On a phone? Open this page on your computer to install Sompio.</p>
          <p class="card-foot small"><a href="<?= e(SOURCE_URL) ?>/releases/tag/v<?= e($r['version']) ?>" rel="noopener noreferrer">What's new</a> · <a href="tor.php">Download through Tor</a></p>
        <?php else: ?>
          <h2>Coming soon</h2>
          <p class="muted">The beta isn't up yet. Check back shortly.</p>
        <?php endif; ?>
      </div>
    </div>

    <section id="install" class="hero-install">
      <div class="dl-started" role="status" hidden>
        <strong>Your download has started.</strong>
        <span>Here's what to click when your computer asks. Didn't start? <a href="#top-download">Use the download button</a>.</span>
      </div>
      <div class="os-panel" data-os="windows"<?= $hide('windows') ?>>
        <h2>Installing on Windows: it will warn you, and that's expected</h2>
        <p>The beta isn't signed with a Microsoft certificate yet (the finished version will be). Until then, Windows and your browser don't know Sompio and show warnings. Here's what you'll see and what to click.</p>
        <ol class="steps">
          <li>
            <h3>1. Your browser may hold the download</h3>
            <p><strong>Edge:</strong> "Sompio_… isn't commonly downloaded". Click the <strong>…</strong> next to it, then <strong>Keep</strong>, then <strong>Show more</strong>, then <strong>Keep anyway</strong>.</p>
            <p><strong>Chrome:</strong> click <strong>Keep</strong> (or open the downloads list and choose <strong>Download unverified file</strong>).</p>
            <div class="mock"><span class="mock-icon">!</span><span class="mock-text"><b>Sompio_<?= e($r['version'] ?? '1.0.0') ?>_x64-setup.exe</b> isn't commonly downloaded. Make sure you trust it before you open it.</span><span class="mock-btn">Keep</span><span class="mock-btn">Delete</span></div>
          </li>
          <li>
            <h3>2. "Windows protected your PC"</h3>
            <p>A blue box appears when you open the setup file. Click <strong>More info</strong>, then <strong>Run anyway</strong>.</p>
            <div class="mock blue"><div><b>Windows protected your PC</b><span>Microsoft Defender SmartScreen prevented an unrecognized app from starting. Running this app might put your computer at risk.</span><u>More info</u></div><span class="mock-btn">Run anyway</span><span class="mock-btn">Don't run</span></div>
          </li>
          <li>
            <h3>3. If Microsoft Defender removes the file</h3>
            <p>Rarely, Defender quarantines a new unsigned program by mistake. If that happens, don't turn off your antivirus. Email <a href="mailto:hello@sompio.org">hello@sompio.org</a>, so we can send it to Microsoft as a false alarm.</p>
          </li>
          <li>
            <h3>4. Start using it</h3>
            <p>Sompio opens straight away. The first time you open Vault or Trace, you choose a password and get a recovery code to write down. Keep that code safe: nobody can reset the lock for you, not even us.</p>
          </li>
        </ol>
      </div>

      <div class="os-panel" data-os="mac"<?= $hide('mac') ?>>
        <h2>Installing on a Mac: macOS will stop it once, and that's expected</h2>
        <p>The beta isn't notarized by Apple yet (the finished version will be), so macOS doesn't know Sompio and blocks it the first time. You allow it once.</p>
        <ol class="steps">
          <li>
            <h3>1. Move it to Applications</h3>
            <p>Open the downloaded <strong>.dmg</strong> file and drag <strong>Sompio</strong> onto the <strong>Applications</strong> folder.</p>
          </li>
          <li>
            <h3>2. Open it, and let macOS block it</h3>
            <p>Open Sompio from Applications. macOS says it couldn't verify Sompio. Click <strong>Done</strong> (not Move to Bin).</p>
          </li>
          <li>
            <h3>3. Allow it in System Settings</h3>
            <p>Open <strong>System Settings</strong>, then <strong>Privacy &amp; Security</strong>. Scroll down to "Sompio was blocked" and click <strong>Open Anyway</strong>, then confirm with your password or Touch ID. From then on, Sompio opens normally.</p>
            <p class="small muted">On macOS 14 or older: right-click Sompio in Applications, choose <strong>Open</strong>, then <strong>Open</strong> again.</p>
          </li>
          <li>
            <h3>4. Set up the lock</h3>
            <p>Sompio walks you through it. Read the first screen: nobody can reset the lock for you, not even us.</p>
          </li>
        </ol>
        <p class="small muted">Not on the Mac yet: auto-type into other apps, and reading text in pictures in Trace. Both are coming.</p>
      </div>

      <div class="os-panel" data-os="linux"<?= $hide('linux') ?>>
        <h2>Installing on Linux</h2>
        <ol class="steps">
          <li>
            <h3>AppImage (most systems)</h3>
            <p>Make the file runnable: right-click it, <strong>Properties</strong>, <strong>Allow executing file as program</strong>, or run <code>chmod +x Sompio_*.AppImage</code>. Then double-click it. On Ubuntu 22.04 and newer you may first need <code>sudo apt install libfuse2</code> (on 24.04: <code>libfuse2t64</code>). New versions come to you inside Sompio.</p>
          </li>
          <li>
            <h3>.deb (Ubuntu, Debian)</h3>
            <p>Run <code>sudo apt install ./Sompio_*_linux_amd64.deb</code> in the folder you downloaded it to. Sompio tells you about new versions; install those the same way.</p>
          </li>
          <li>
            <h3>Tails</h3>
            <p>Being tested. Use the AppImage, kept in your Persistent Storage folder. Sompio's data lives in your home folder, which Tails wipes at shutdown unless the <strong>Dotfiles</strong> feature of Persistent Storage is on. Tried it? <a href="feedback.php">Tell us</a> how it went.</p>
          </li>
        </ol>
        <p class="small muted">Not on Linux yet: auto-type into other apps, and reading text in pictures in Trace.</p>
      </div>
      <p class="small muted">New versions come to you inside Sompio: a pop-up offers them, and your data stays as it is.</p>
    </section>
