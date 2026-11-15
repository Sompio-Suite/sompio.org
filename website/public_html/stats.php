<?php
// Private numbers page: downloads per system and update checks, per day. Open it with
// stats.php?key=... where the key is 'stats_key' in private/config.php. Without the right key,
// the page doesn't exist.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';

$key = (string) (config()['stats_key'] ?? '');
if (strlen($key) < 16 || !hash_equals($key, (string) ($_GET['key'] ?? ''))) {
    http_response_code(404);
    exit;
}
header('Cache-Control: private, no-store');
header('X-Robots-Tag: noindex, nofollow');

$all = read_stats();
$days = [];
for ($i = 0; $i < 30; $i++) {
    $d = gmdate('Y-m-d', time() - $i * 86400);
    $days[$d] = $all[$d] ?? [];
}
$sum = fn(array $rows, string $k) => array_sum(array_map(fn($r) => (int) ($r[$k] ?? 0), $rows));
$week = array_slice($days, 1, 7, true); // the last 7 full days
$checks7 = $sum($week, 'update_check');

page_head('Numbers · Sompio', 'Private numbers.');
site_header('');
?>
  <main class="wrap narrow">
    <p class="eyebrow">Private</p>
    <h1>Numbers</h1>
    <p class="lead">Downloads and update checks, per day (UTC). Only totals are kept.</p>

    <section class="card">
      <h2>At a glance</h2>
      <ul class="points">
        <li><strong>All time:</strong>
          <?php foreach (STAT_NAMES as $k => $label): ?>
            <?= e(str_replace('Downloads: ', '', $label)) ?> <?= $sum($all, $k) ?><?= $k === array_key_last(STAT_NAMES) ? '' : ' ·' ?>
          <?php endforeach; ?>
        </li>
        <li><strong>Installed copies in use, roughly:</strong> <?= (int) ceil($checks7 / 7 / 4) ?> to <?= (int) ceil($checks7 / 7) ?> a day
          <span class="small muted">(each running copy checks for updates at most every 6 hours, so 1 to 4 checks a day; based on the last 7 full days)</span></li>
      </ul>
    </section>

    <section class="card">
      <h2>Last 30 days</h2>
      <div class="table-wrap">
        <table>
          <thead><tr><th>Day</th><th>Windows</th><th>Mac</th><th>AppImage</th><th>.deb</th><th>Update checks</th></tr></thead>
          <tbody>
          <?php foreach ($days as $d => $r): ?>
            <tr><td><?= e($d) ?></td>
              <?php foreach (array_keys(STAT_NAMES) as $k): ?><td><?= (int) ($r[$k] ?? 0) ?></td><?php endforeach; ?>
            </tr>
          <?php endforeach; ?>
          </tbody>
        </table>
      </div>
    </section>
    <p class="small muted">Visitors to the website itself: see the Cloudflare dashboard, under Analytics.</p>
  </main>
<?php site_footer(); ?>
