<?php
// Feedback now goes from inside the app ("Report a bug" in the menu, sent by report.php). This page
// stays so links in older versions and old posts still land somewhere useful.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';

$c = config();
page_head('Report a bug · Sompio', 'How to tell us what broke, what confused you, or what you\'d like in Sompio.');
site_header('feedback');
?>
  <main class="wrap narrow">
    <p class="eyebrow">Beta</p>
    <h1>Report a bug</h1>
    <p class="lead">Bugs, confusing bits, ideas, or what you liked: everything helps, and it decides what gets fixed first.</p>

    <section class="card">
      <h2>From inside Sompio</h2>
      <p>Click <strong>Report a bug</strong> in the menu on the left. Write what happened, and Sompio can add its version,
        your system and the last lines of its problems log for you. You see everything before it's sent, and nothing
        is sent until you press Send.</p>
      <p class="small muted">Using an older version without that button? Update Sompio first, or email us.</p>
    </section>

    <section class="card">
      <h2>By email</h2>
      <p>Write to <a href="mailto:hello@sompio.org">hello@sompio.org</a>. A screenshot helps a lot, but please don't show real passwords or personal details.
        <?php if (!empty($c['discord'])): ?>You can also post in our <a href="<?= e($c['discord']) ?>" rel="noopener noreferrer">Discord</a>.<?php endif; ?></p>
    </section>

    <p class="small muted">How we handle reports: see the <a href="privacy.php#what">privacy statement</a>.</p>
  </main>
<?php site_footer(); ?>
