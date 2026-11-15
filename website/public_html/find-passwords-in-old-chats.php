<?php
// Guide: finding passwords you've sent or saved in chats, mail and notes.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('How to find passwords you\'ve shared in old chats · Sompio', 'The Wi-Fi password for a guest, the streaming login for family, a note to yourself. How to find the passwords sitting in your old chats, mail and notes, and what to do about each one.');
site_header('guide');
?>
  <main class="wrap docs guide">
    <aside class="docs-toc" aria-label="On this page">
      <p class="toc-title">On this page</p>
      <a href="#why">Why it matters</a>
      <a href="#where">1. Where they hide</a>
      <a href="#search">2. Search by hand</a>
      <a href="#trace">3. Search everything at once</a>
      <a href="#fix">4. Fix what you find</a>
      <a href="#next">Next time</a>
      <a href="guides.php" class="toc-back">← All guides</a>
    </aside>

    <article class="docs-body">
      <p class="eyebrow">Passwords · updated November 2026</p>
      <h1>How to find passwords you've shared in old chats</h1>
      <p class="lead">Almost everyone has done it: sent the Wi-Fi password to a guest, the streaming login to family, or saved a password in a chat with yourself "just for now". Years later, those messages are still there. Here's how to find them and make them harmless.</p>

      <section id="why">
        <h2>Why it matters</h2>
        <ul class="points">
          <li><strong>Chats last longer than you think.</strong> A message from 2019 is still in your history, the other person's history, and any backup either of you made.</li>
          <li><strong>Accounts get taken over.</strong> If someone gets into your email or chat app, the first thing they search for is "password". Whatever they find opens the next door.</li>
          <li><strong>Old passwords are often still in use.</strong> Especially if you reuse them, a password sent years ago may still open accounts today.</li>
        </ul>
      </section>

      <section id="where">
        <h2>1. Where passwords usually hide</h2>
        <ul class="points">
          <li><strong>Chats with yourself:</strong> Telegram's Saved Messages, WhatsApp's "Message yourself", a Discord server or DM you use as a notepad.</li>
          <li><strong>Family and partner chats:</strong> shared logins for streaming, shopping, banking apps or the router.</li>
          <li><strong>Work chats and mail:</strong> logins sent to a colleague "for a second", or setup emails with temporary passwords.</li>
          <li><strong>Your sent mail:</strong> anything you emailed to yourself or others.</li>
          <li><strong>Notes and files:</strong> text files called "passwords", Word documents, and screenshots of login screens or 2FA backup codes.</li>
          <li><strong>Recovery codes and phrases:</strong> 2FA backup codes and crypto wallet recovery phrases are even more dangerous than a password: they bypass it.</li>
        </ul>
      </section>

      <section id="search">
        <h2>2. Search by hand</h2>
        <p>Every chat app has a search. Try these words one by one, in each app you use, and in your mail:</p>
        <p><code>password</code> <code>pass</code> <code>pw</code> <code>login</code> <code>wifi</code> <code>pin</code> <code>code</code> <code>backup codes</code>, and the same words in your own language (for example <code>wachtwoord</code>, <code>Passwort</code>, <code>mot de passe</code>).</p>
        <p>This finds the obvious ones. It misses passwords sent without a label, like "it's <code>Summer2021!</code>", and it doesn't look inside files or screenshots.</p>
      </section>

      <section id="trace">
        <h2>3. Search everything at once</h2>
        <p><a href="docs.php#trace">Sompio Trace</a> does this search for you, on your own computer, across all your sources together:</p>
        <ol class="guide-steps">
          <li>Add your chats: a Telegram or WhatsApp export (ready in minutes), your mailbox, or a data package from Discord, Instagram and others.</li>
          <li>Add folders you choose on your computer. Tick <strong>Read text in pictures</strong> (Windows) to include screenshots.</li>
          <li>Trace lists what looks like passwords, 2FA backup codes and recovery phrases, together with addresses, bank details and ID numbers, sorted by risk.</li>
        </ol>
        <p>If you use Sompio Vault, its health check also flags any saved password that turns up in your own messages and files, so you know exactly which logins to change first. Nothing is uploaded, and nothing is scanned until you start it.</p>
      </section>

      <section id="fix">
        <h2>4. Fix what you find</h2>
        <p class="callout"><strong>Change the password first, then delete the message.</strong> Deleting doesn't undo the past: the other person, or a backup, may still have it.</p>
        <ol class="guide-steps">
          <li><strong>Change the password</strong> on that account, and anywhere else you used the same one.</li>
          <li><strong>Turn on two-step verification</strong> where you can. A leaked password alone then isn't enough to get in.</li>
          <li><strong>Replace backup codes</strong> you found: most sites let you create a new set, which makes the old ones useless.</li>
          <li><strong>Delete the message.</strong> Choose "delete for everyone" where the app offers it (some apps only allow this for a limited time), and ask the other person to delete their copy if they can't.</li>
          <li><strong>Delete the files and screenshots</strong>, and empty the Recycle Bin or Bin.</li>
        </ol>
      </section>

      <section id="next">
        <h2>Next time</h2>
        <ul class="points">
          <li>Keep passwords in a password manager, not in chats or notes. Any good one works; <a href="docs.php#vault">Sompio Vault</a> keeps them on your computer, in the open KeePass format.</li>
          <li>If you have to give someone a password, tell them in person or by phone, and change it afterwards if it was only for a short while.</li>
          <li>For guests, a separate guest Wi-Fi network means your main Wi-Fi password never has to leave the house.</li>
        </ul>
      </section>

<?php more_guides('find-passwords-in-old-chats.php'); ?>
    </article>
  </main>
<?php site_footer(); ?>
