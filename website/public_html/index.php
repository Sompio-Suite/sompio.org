<?php
// sompio.org: the public front page. The beta download is on download.php.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('Sompio · Find what you\'ve left online, and remove it yourself', 'Sompio helps you understand what\'s out there about you, find it in your chats, mail and files, clean it up and protect yourself from now on. On your own computer: no account, no cloud, no data broker in between.');
site_header('home');
?>
  <main>
    <section class="home-hero">
      <div class="wrap home-hero-row">
        <div class="home-hero-text">
          <p class="status-pill"><span class="dot" aria-hidden="true"></span> Open beta · free</p>
          <h1>Find what you've left online. <span class="hl">Remove it yourself.</span></h1>
          <p class="lead">Most of what you've typed online is still sitting on someone's server: an address in an old chat, a password sent to a friend, a passport scan in an email. Sompio helps you understand what's out there, find it, clean it up, and keep your digital footprint safe from now on.</p>
          <div class="cta-row">
            <a class="button primary" href="download.php">Download the beta</a>
            <a class="button" href="what-is-personal-information.php">Start with the basics</a>
          </div>
          <p class="small muted">Free for <?= e(systems_text()) ?>. It all happens on your computer: no account, no cloud.</p>
        </div>
        <figure class="shot shot-hero">
          <img src="<?= asset('img/trace.webp') ?>" width="1280" height="780" alt="Sompio's Trace tool listing ID numbers, passport numbers, backup codes and bank numbers found in sample chats, sorted by risk.">
        </figure>
      </div>
    </section>

    <section id="how" class="wrap journey-section">
      <p class="eyebrow">How it works</p>
      <h2>Four steps to a smaller digital footprint</h2>
      <p class="muted section-lead">You don't need to be a privacy expert. Sompio walks you through it, at your own pace.</p>
      <ol class="journey">
        <li>
          <span class="journey-num">1</span>
          <h3>Learn</h3>
          <p>What counts as personal information, where it ends up, how it gets misused, and when sharing it is fine.</p>
          <a href="what-is-personal-information.php">What is personal information?</a>
        </li>
        <li>
          <span class="journey-num">2</span>
          <h3>Find</h3>
          <p>Scan your computer and mailbox in minutes. Ask Discord, Instagram and others for your data; Sompio reminds you when it's ready.</p>
          <a href="#find">How Sompio finds it</a>
        </li>
        <li>
          <span class="journey-num">3</span>
          <h3>Clean</h3>
          <p>Go through what was found, most risky first. Delete it, and send ready-made letters that make companies erase what they hold.</p>
          <a href="make-companies-delete-your-data.php">Make companies delete your data</a>
        </li>
        <li>
          <span class="journey-num">4</span>
          <h3>Secure</h3>
          <p>Stop leaking from now on: change the passwords that were out there, turn on two-step login, and keep passwords in a vault.</p>
          <a href="#secure">Keep it safe</a>
        </li>
      </ol>
    </section>

    <section class="wrap home-tools">
      <article class="feature" id="find">
        <div class="feature-text">
          <p class="eyebrow">Find and clean · Trace</p>
          <h3>See what you've shared over the years, then clean it up</h3>
          <p>An old bank number sent to a friend, a password pasted in a chat, a scan of your passport in Downloads. Trace starts with what's quick: folders on your computer, your sent mail, a WhatsApp or Telegram chat. Meanwhile it helps you ask Discord, Instagram, Reddit and the rest for your data, and reminds you when it's ready.</p>
          <ul class="ticks">
            <li>Finds passwords, bank and card numbers, ID and passport numbers, backup codes, addresses, dates of birth and more, sorted by risk</li>
            <li>Explains each finding in plain words: why it matters, and what to do about it</li>
            <li>"Clean up" takes you to each message in turn, and writes the letters that make companies delete what they hold about you</li>
          </ul>
        </div>
        <figure class="shot"><img src="<?= asset('img/trace.webp') ?>" width="1280" height="780" alt="Trace findings: a table of risky items with advice on what to do for each." loading="lazy"></figure>
      </article>

      <article class="feature flip" id="secure">
        <div class="feature-text">
          <p class="eyebrow">Secure · Vault</p>
          <h3>Stop leaking passwords, for good</h3>
          <p>Most leaks start with a password shared or reused. Vault keeps your passwords encrypted on your own computer, fills them in for you, and tells you which ones to change first: weak, reused, found in a data breach, or found in your own chats by Trace.</p>
          <ul class="ticks">
            <li>Never uploaded: encrypted on your device, opened with your password and a second key</li>
            <li>Fills logins in Chrome, Edge, Brave and Firefox, with two-factor codes built in</li>
            <li>Never locked in: saved in the open KeePass format, so your passwords open in other apps too</li>
          </ul>
        </div>
        <figure class="shot"><img src="<?= asset('img/vault.webp') ?>" width="1280" height="780" alt="Sompio Vault showing a list of sample logins in folders." loading="lazy"></figure>
      </article>
    </section>

    <section class="wrap why-diy">
      <div class="card why-card">
        <div>
          <p class="eyebrow">Why do it yourself?</p>
          <h2>No removal service in between</h2>
          <p>Data-removal services charge every month, and need your name, address and more to find you. With Sompio you do it yourself, guided step by step, and your details never leave your computer. The law is on your side: in the EU and UK, companies must delete your data when you ask.</p>
        </div>
        <a class="button" href="make-companies-delete-your-data.php">How deletion requests work</a>
      </div>
    </section>

    <section class="band">
      <div class="wrap">
        <h2>Private by design, not by promise</h2>
        <div class="promises">
          <div><h3>No account</h3><p>Nothing to sign up for. Nothing about you sits on a server of ours, so there's nothing to leak.</p></div>
          <div><h3>Works offline</h3><p>Sompio does all its work on your computer. The only thing it fetches on its own is the newest version number, and you can switch that off. <a href="docs.php#private">See every time it goes online</a>.</p></div>
          <div><h3>Encrypted with your lock</h3><p>Your password plus a second key that only your computer, or your key picture, holds. Nobody can open it without both, not even us.</p></div>
          <div><h3>Code you can check</h3><p>The full source is <a href="<?= e(SOURCE_URL) ?>" rel="noopener noreferrer">on GitHub</a>, so anyone can see what Sompio does with your data.</p></div>
          <div><h3>Scans only when you say so</h3><p>Nothing is read until you start a scan, and only the folders and sources you pick. Sompio keeps what it finds, never copies of your files.</p></div>
          <?php $sys = systems_text(); ?><div><h3>Runs on <?= e(strpos($sys, 'Windows,') === 0 ? 'your computer' : 'Windows') ?></h3><p><?= e($sys) ?>. One small program, no extra installs.</p></div>
        </div>
      </div>
    </section>

    <section class="wrap guide-promo">
      <div class="guide-card">
        <div>
          <p class="eyebrow">Free guides</p>
          <h2>Learn before you clean</h2>
          <p>What counts as personal information, how to find old passwords in your chats, remove the location from photos, delete Discord messages and Reddit comments, and make companies erase your data. Step by step, free.</p>
        </div>
        <a class="button primary" href="guides.php">Read the guides</a>
      </div>
    </section>

    <section id="join" class="wrap join">
      <div class="join-intro">
        <p class="eyebrow">Beta</p>
        <h2>Try it now, free</h2>
        <p>The beta is open to everyone, with everything unlocked. Your feedback decides what Sompio becomes.</p>
        <ul class="ticks">
          <li><a href="download.php">Download the beta</a> for <?= e(systems_text()) ?></li>
          <li>Found a problem or have an idea? Click <a href="feedback.php">Report a bug</a> at the top of Sompio: it decides what gets fixed first</li>
          <li>New versions come to you inside Sompio</li>
        </ul>
      </div>
      <div class="card join-card">
        <?php if (isset($_GET['joined'])): ?>
          <div class="join-done" role="status">
            <h3>You're on the list</h3>
            <p>Thanks! You'll hear from <strong>hello@sompio.org</strong> when a big new version is out, and once at launch. Nothing else.</p>
          </div>
        <?php else: ?>
          <h3>Get updates by email</h3>
          <p class="small muted">Want to hear about big new versions and the launch? Leave your email.</p>
          <?php $errs = ['email' => 'That email address doesn\'t look right.', 'stale' => 'The form was open too long, or sent too fast. Please try again.', 'limit' => 'Too many sign-ups from your network. Try again in an hour.', 'server' => 'Something went wrong on our side. Please try again later.']; ?>
          <?php if (isset($_GET['join_error'], $errs[$_GET['join_error']])): ?><p class="error" role="alert"><?= e($errs[$_GET['join_error']]) ?></p><?php endif; ?>
          <form method="post" action="signup.php" class="join-form">
            <input type="hidden" name="stamp" value="<?= e(signup_stamp()) ?>">
            <div class="hp" aria-hidden="true"><label>Leave this empty <input name="website" tabindex="-1" autocomplete="off"></label></div>
            <label>Email<input name="email" type="email" maxlength="120" required autocomplete="email"></label>
            <button class="primary" type="submit">Get updates</button>
            <p class="small muted">Your email is only used for these updates: a few emails at most, and one at launch. It's never shared, and there's no tracking. Want it removed? Write to <a href="mailto:hello@sompio.org">hello@sompio.org</a>. Details in our <a href="privacy.php">privacy statement</a>.</p>
          </form>
        <?php endif; ?>
      </div>
    </section>
  </main>
<?php site_footer(); ?>
