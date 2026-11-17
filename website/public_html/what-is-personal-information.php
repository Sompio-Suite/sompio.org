<?php
// Guide: the beginner's starting point. What personal information is, where it ends up, how it gets
// misused, and when sharing it is fine.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('What is personal information, and when is sharing it risky? · Sompio', 'A plain-language starting point: what counts as personal information, where it ends up, how it gets misused, what is risky to share and what is fine, and how to clean it up.');
site_header('guide');
?>
  <main class="wrap docs guide">
    <aside class="docs-toc" aria-label="On this page">
      <p class="toc-title">On this page</p>
      <a href="#what">What counts</a>
      <a href="#combined">Small pieces add up</a>
      <a href="#where">Where it ends up</a>
      <a href="#misuse">How it gets misused</a>
      <a href="#risk">Risky or fine?</a>
      <a href="#next">What you can do</a>
      <a href="guides.php" class="toc-back">← All guides</a>
    </aside>

    <article class="docs-body">
      <p class="eyebrow">Start here · updated November 2026</p>
      <h1>What is personal information, and when is sharing it risky?</h1>
      <p class="lead">Everything you've typed online since your first account is probably still stored somewhere. Most of it is harmless. Some of it isn't. This page helps you tell the difference, before you start cleaning anything up.</p>

      <section id="what">
        <h2>What counts as personal information</h2>
        <p>Personal information (also called personal data, or PII: personally identifiable information) is anything that says something about you, or can be used to find out who you are. In the EU, the GDPR uses a broad definition: if it can be linked to you, it counts.</p>
        <ul class="points">
          <li><strong>Things that point straight at you:</strong> your full name, home address, phone number, email address, photos of your face, and ID numbers like a passport, driving licence or citizen service number.</li>
          <li><strong>Things that open doors:</strong> passwords, two-step login backup codes, crypto recovery phrases, and bank and card numbers.</li>
          <li><strong>Things about your life:</strong> date of birth, where you work or study, your daily routine, where you are right now, and the people close to you.</li>
          <li><strong>Extra-sensitive information:</strong> health, religion, sexual orientation, political views, ethnicity and fingerprints or face scans. The law protects these more strictly, because misuse can hurt more.</li>
        </ul>
      </section>

      <section id="combined">
        <h2>Small pieces add up</h2>
        <p>One detail on its own rarely identifies you. Several together often do. A well-known study from the US found that most people can be identified by just their postcode, date of birth and gender.</p>
        <p>That's why "it's only my birthday" or "it's only my street" isn't the whole story. Someone who finds your first name in one chat, your street in another and your birthday in a third can put a surprisingly complete picture together.</p>
      </section>

      <section id="where">
        <h2>Where it ends up</h2>
        <ul class="points">
          <li><strong>On company servers, for years.</strong> Chats, posts, comments and emails are stored by the service, usually long after you've forgotten them, and often even after you've closed the account.</li>
          <li><strong>With the other person.</strong> A message you sent lives in their chat history and their backups too, and anyone who gets into their account can read it.</li>
          <li><strong>In data breaches.</strong> When a company is hacked, what it stored about you can end up in leaked databases that are traded and searched.</li>
          <li><strong>With data brokers.</strong> Companies collect public and bought information and combine it into profiles that they sell.</li>
          <li><strong>On your own devices.</strong> Screenshots, scans, downloads and photos (which often include the exact spot where they were taken).</li>
        </ul>
      </section>

      <section id="misuse">
        <h2>How it gets misused</h2>
        <ul class="points">
          <li><strong>Account takeover:</strong> a password or backup code found in an old chat opens your email, and from there everything else.</li>
          <li><strong>Identity fraud:</strong> ID numbers, date of birth and address are enough to open accounts or take out loans in your name.</li>
          <li><strong>Convincing scams:</strong> the more a scammer knows about you, the more believable their message or phone call.</li>
          <li><strong>Doxxing and harassment:</strong> someone collects your details and makes them public, or turns up where you live.</li>
          <li><strong>Embarrassment:</strong> old posts and messages resurfacing at a job application, in a dispute or after a breakup.</li>
        </ul>
      </section>

      <section id="risk">
        <h2>Risky or fine? A rough guide</h2>
        <p>Context matters: what's fine in a private chat with your partner is different from a public post. As a rule of thumb, this is how Sompio's Trace sorts what it finds:</p>
        <ul class="points">
          <li><strong>Fix now:</strong> passwords, two-step login backup codes, recovery phrases, bank and card numbers, ID and passport numbers, and your full home address. Anyone holding these can act as you or find you.</li>
          <li><strong>Worth a look:</strong> phone numbers, email addresses, date of birth, IP addresses and photos with a precise location. Usually fine on their own, risky in combination or in public.</li>
          <li><strong>Usually fine:</strong> your first name, the city you live in, your hobbies, things you like. Normal parts of talking to people online.</li>
        </ul>
        <p class="callout"><strong>You don't have to delete everything.</strong> A clean footprint isn't an empty one. Focus on what could be used against you, and keep the rest.</p>
      </section>

      <section id="next">
        <h2>What you can do</h2>
        <ol class="guide-steps">
          <li><strong>Find it.</strong> Look through your own chats, mail and files. <a href="find-passwords-in-old-chats.php">How to find passwords you've shared in old chats</a> shows how; <a href="docs.php#trace">Sompio Trace</a> does it for you, on your own computer.</li>
          <li><strong>Remove it.</strong> Delete what you can yourself: <a href="delete-discord-messages.php">Discord messages</a>, <a href="delete-reddit-comments.php">Reddit comments</a>, <a href="remove-location-from-photos.php">photo locations</a>. For what companies hold, <a href="make-companies-delete-your-data.php">ask them to delete it</a>; the law requires it.</li>
          <li><strong>Secure what's left.</strong> Change passwords that were out there, turn on two-step login, and keep passwords in a password manager instead of chats and notes. <a href="move-passwords-out-of-browser.php">Moving your passwords out of the browser</a> is a good first step.</li>
        </ol>
      </section>

<?php more_guides('what-is-personal-information.php'); ?>
    </article>
  </main>
<?php site_footer(); ?>
