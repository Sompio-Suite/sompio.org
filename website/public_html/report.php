<?php
// Bug reports sent from inside the Sompio app (menu, "Report a bug"). There is no form on the
// website any more: the app posts JSON here only when the person presses Send. The report is
// emailed to the address in config.php and not kept on this website (it waits in private/outbox
// only while email can't go out). No IP address or browser details are kept: the per-network
// limit uses a scrambled code made from the address, for one hour.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';

const AREAS = ['Trace', 'Vault', 'Calendar', 'Installing or starting', 'The lock', 'Something else'];
const KINDS = ['Something didn\'t work', 'Something was confusing', 'An idea', 'Something I liked'];
const MAX_SHOT = 4 * 1024 * 1024;

function reply(int $code, bool $ok, string $text, string $id = ''): never
{
    http_response_code($code);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode(['ok' => $ok, 'message' => $text, 'id' => $id]);
    exit;
}

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    header('Location: feedback.php', true, 303);
    exit;
}
// Only the app sends here; it says so in its user agent
if (!preg_match('/^Sompio\/\d+\.\d+\.\d+/', (string) ($_SERVER['HTTP_USER_AGENT'] ?? ''))) {
    reply(403, false, 'Bug reports are sent from inside the Sompio app.');
}
$raw = (string) file_get_contents('php://input', false, null, 0, 7 * 1024 * 1024);
$d = json_decode($raw, true);
if (!is_array($d)) {
    reply(400, false, 'The report didn\'t come through in one piece. Please try again.');
}

$clip = fn(string $k, int $max) => mb_substr(trim(str_replace("\r", '', is_string($d[$k] ?? null) ? $d[$k] : '')), 0, $max);
$one = fn(string $s) => preg_replace('/\s+/', ' ', $s);
$version = $one($clip('version', 20));
$os = $one($clip('os', 120));
$contact = $one($clip('contact', 120));
$area = in_array($a = $one($clip('area', 40)), AREAS, true) ? $a : 'Something else';
$kind = in_array($k = $one($clip('kind', 40)), KINDS, true) ? $k : 'An idea';
$message = $clip('message', 8000);
$details = $clip('details', 8000);
if (mb_strlen($message) < 3) {
    reply(422, false, 'Please write a few words about what happened.');
}
if ($contact !== '' && !filter_var($contact, FILTER_VALIDATE_EMAIL)) {
    reply(422, false, 'That email address doesn\'t look right. Leave it empty if you don\'t need a reply.');
}

$shot = null;
if (is_array($d['screenshot'] ?? null) && is_string($d['screenshot']['data'] ?? null)) {
    $data = base64_decode($d['screenshot']['data'], true);
    $info = $data !== false && strlen($data) <= MAX_SHOT ? @getimagesizefromstring($data) : false;
    $types = [IMAGETYPE_PNG => ['png', 'image/png'], IMAGETYPE_JPEG => ['jpg', 'image/jpeg'], IMAGETYPE_WEBP => ['webp', 'image/webp']];
    if (!$info || !isset($types[$info[2]])) {
        reply(422, false, 'The screenshot needs to be a PNG or JPG picture under 4 MB.');
    }
    $shot = ['data' => $data, 'ext' => $types[$info[2]][0], 'type' => $types[$info[2]][1]];
}

// Spam gets the same answer as everyone else, so it can't learn what was caught
if (looks_like_spam($contact, "$message\n$details")) {
    reply(200, true, 'Thank you! Your report was sent.');
}
if (over_net_limit('feedback', 10, 3600)) {
    reply(429, false, 'That\'s a lot of reports from your network in one hour. Please wait a bit before sending more.');
}

$id = date('Y-m-d_His') . '-' . bin2hex(random_bytes(3));
$body = "Sompio bug report ($id)\n"
    . "App: Sompio $version on $os\n"
    . 'Reply to: ' . ($contact !== '' ? $contact : '(no email given)') . "\n"
    . "Part: $area\nKind: $kind\n\n"
    . "What happened:\n$message\n"
    . ($details !== '' ? "\nTechnical details:\n$details\n" : '')
    . ($shot ? "\nScreenshot attached ($id.{$shot['ext']}).\n" : '');
$subject = "Sompio report: $kind ($area)";
if (!deliver($subject, $body, $shot ? ['name' => "$id.{$shot['ext']}", 'type' => $shot['type'], 'data' => $shot['data']] : null)) {
    reply(503, false, 'The report couldn\'t be sent just now. Please try again later, or write to hello@sompio.org.');
}
reply(200, true, 'Thank you! Your report was sent.', $id);
