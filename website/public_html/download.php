<?php
// The open beta: anyone can download the newest setup file. The file sits in private/, which the
// web server never serves directly; download.php?get=1 hands it out, a few times per network per
// hour, so nobody can run up the site's traffic. The download section itself is on the front page.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';

$r = latest_release();

if (isset($_GET['get'])) {
    if ($r === null) {
        http_response_code(404);
        exit('No beta version is up yet. Please check back soon.');
    }
    if (over_net_limit('download', 10, 3600)) {
        http_response_code(429);
        header('Retry-After: 3600');
        exit('Too many downloads from your network in the last hour. Please try again later.');
    }
    @ini_set('zlib.output_compression', 'Off');
    while (ob_get_level()) {
        ob_end_clean();
    }
    header('Content-Type: application/octet-stream');
    header('Content-Disposition: attachment; filename="' . $r['name'] . '"');
    header('Content-Length: ' . $r['size']);
    header('X-Content-Type-Options: nosniff');
    header('Cache-Control: private, no-store');
    readfile($r['path']);
    exit;
}

header('Location: ./#download', true, 301);
