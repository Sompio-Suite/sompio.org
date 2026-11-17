<?php
// Documentation: what Sompio is, each tool, and how it keeps data private. Open to everyone; the
// app's Documentation button opens this page.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('How Sompio works · Documentation', 'How Sompio helps you find, clean up and secure your digital footprint with Trace and Vault, how the Sompio Lock keeps your data encrypted, and why nothing leaves your computer.');
site_header('docs');
?>

  <main class="wrap docs">
    <aside class="docs-toc" aria-label="On this page">
      <p class="toc-title">On this page</p>
      <a href="#what">What Sompio is</a>
      <a href="#private">Why it's private</a>
      <a href="#lock">The Sompio Lock</a>
      <a href="#score">Overview and your score</a>
      <a href="#trace">Trace</a>
      <a href="#vault">Vault</a>
      <a href="#extension">Browser extension</a>
      <a href="#limits">Good to know</a>
      <a href="./" class="toc-back">← Back to sompio.org</a>
    </aside>

    <article class="docs-body">
      <h1>How Sompio works</h1>

      <section id="what">
        <h2>What Sompio is</h2>
        <p>Sompio helps you take back your digital footprint, in four steps: <strong>learn</strong> what personal information is and why it matters, <strong>find</strong> what you've left behind in your chats, mail and files, <strong>clean</strong> it up, and <strong>secure</strong> yourself from now on. Everything works on your own computer. There is no account to make, no cloud, and no server of ours that holds your data.</p>
        <p>It has two tools: <strong>Trace</strong> finds and removes, and <strong>Vault</strong> keeps your passwords safe. Both sit behind one lock, the <strong>Sompio Lock</strong>: one password opens both, and they're encrypted again when you step away. New to all this? Start with <a href="what-is-personal-information.php">What is personal information?</a></p>
        <div class="tools">
          <a class="tool" href="#vault"><strong>Vault</strong><span>Keeps your passwords safe, fills them in for you (also in your browser, with the Sompio extension), and tells you which ones to change first.</span></a>
          <a class="tool" href="#trace"><strong>Trace</strong><span>Finds personal information you've shared online (bank numbers, passwords, addresses…), helps you remove it, and gets companies to delete what they hold.</span></a>
        </div>
      </section>

      <section id="private">
        <h2>Why it's private</h2>
        <p>Sompio is built so your information never has to leave your computer. There is no Sompio server holding your passwords or messages, so there is nothing of yours on our side that could be hacked, sold or leaked.</p>
        <ul class="points">
          <li><strong>Offline by design.</strong> Every tool works without internet. You can switch off the internet and use all of it.</li>
          <li><strong>Encrypted on your disk.</strong> Trace's findings and requests, and your passwords, are stored encrypted. Someone who copies the files gets nothing readable.</li>
          <li><strong>No account, no tracking.</strong> No sign-up, no email address, no analytics, no ads, no "phone home". Sompio doesn't know who you are.</li>
          <li><strong>Pop-ups out of screenshots.</strong> Pop-ups and the auto-type picker are left out of screenshots and screen sharing, so a stream or meeting doesn't show what pops up. The main window can be captured, for help and bug reports. Copied passwords stay out of Windows' clipboard history and cloud clipboard, and clear themselves after 20 seconds.</li>
          <li><strong>The browser extension stays local too.</strong> It talks only to Sompio on your computer, never to a server, and only after you allow your browser in Sompio. See <a href="#extension">Browser extension</a>.</li>
          <li><strong>Problem reports stay with you.</strong> If something goes wrong, Sompio writes a short note to a file on your computer. It's never sent anywhere; you decide whether to share it.</li>
        </ul>
        <h3>The only times Sompio goes online</h3>
        <p>Each one goes only to the place named, and only what's listed is sent.</p>
        <div class="table-wrap">
          <table>
            <thead><tr><th>When</th><th>Where it goes</th><th>What is sent</th></tr></thead>
            <tbody>
              <tr><td>Vault saves an entry with a website</td><td>That website only, to fetch its little icon</td><td>A normal page request, like opening the site. Nothing about you or your passwords. Can be switched off.</td></tr>
              <tr><td>Sompio checks for updates (every few hours, or when you click <em>Check now</em>; can be switched off in Security)</td><td>sompio.org</td><td>A request for one small file with the newest version number. Nothing about you or your data. Updates install only when you click <em>Update now</em>, and only if they're signed by us.</td></tr>
              <tr><td>You click <em>Check for breaches</em> in the Vault</td><td>Have I Been Pwned, the public breach database</td><td>The first 5 characters of a scrambled fingerprint of a password. Never the password itself.</td></tr>
              <tr><td>You connect Trace to your Telegram or email</td><td>Telegram or your email provider</td><td>Your login, to read the messages you sent and delete the ones you choose. Nothing is uploaded anywhere else.</td></tr>
              <tr><td>You press <em>Send</em> in <em>Report a bug</em></td><td>sompio.org, which emails it to us</td><td>What you wrote, which part of Sompio it's about, Sompio's version and your system. Only if you tick them: your email address, the last lines of the problems log (error messages, with your user name taken out) and a screenshot you chose. You see it all before sending.</td></tr>
              <tr><td>You click a search, a download page or a company's form in Trace</td><td>That website, in your own browser</td><td>Only what you click. The site sees what you search for, as if you typed it there yourself.</td></tr>
            </tbody>
          </table>
        </div>
        <p>If you never use these, Sompio never touches the internet. Letters to companies open in <em>your own</em> email app, ready for you to send; Sompio never sends mail itself.</p>
      </section>

      <section id="lock">
        <h2>The Sompio Lock</h2>
        <p>The lock protects Vault and Trace. You set it up the first time you open one of them, in one screen. It always needs <strong>two things</strong> to open: your <strong>password</strong> and a <strong>second key</strong>, a random secret that Sompio makes for you.</p>
        <ul class="points">
          <li><strong>Without a key picture</strong> (the quick way), the second key stays in your computer's protected credential store. Your password then opens Sompio on this computer only; someone who copies your files, or knows your password, still can't open them elsewhere.</li>
          <li><strong>With a key picture</strong> (recommended), the second key is also saved as a night-sky .png with the secret hidden in its pixels. Keep a copy off the computer (a USB stick, or your phone as a file): with it, you can open Sompio on another computer. You can also save the picture later, under Security.</li>
          <li><strong>Recovery code.</strong> You also get a code to print or write down. It's your way back in if you forget the password or lose the second key.</li>
          <li><strong>YubiKey (optional).</strong> A YubiKey can stand in for the key picture.</li>
          <li><strong>Auto-lock.</strong> Vault and Trace lock themselves after a while without use. Deadline reminders keep coming.</li>
        </ul>
        <p class="callout"><strong>Nobody can reset it, not even us.</strong> That's the point: we never have your keys. Lose the password or picture <em>and</em> the recovery code, and the data can't be opened by anyone. Don't edit, crop or send the key picture through chat apps that shrink photos, because that erases the secret.</p>
      </section>

      <section id="score">
        <h2>Overview and your exposure score</h2>
        <p>Click the Sompio logo at the top left to open <strong>Overview</strong>: how exposed you are, the next best thing to do, and what Trace, Vault and your requests to companies have found. New to Sompio? <strong>Getting started</strong>, next to it, walks you through the four steps. Once all four are done, it moves under the <strong>?</strong> menu.</p>
        <p>The <strong>exposure score</strong> runs from 0 to 100, and <strong>lower is better</strong>. It is worked out on your computer, only from what Sompio has found so far:</p>
        <ul>
          <li><strong>Trace, up to 60 points:</strong> findings you haven't dealt with yet. A "fix now" finding (a password, ID or bank number) counts three times as much as one "worth a look", and a low-risk one a quarter.</li>
          <li><strong>Vault, up to 25 points:</strong> passwords that also turned up in your chats or files count most, then weak passwords and passwords used for more than one account.</li>
          <li><strong>Requests, up to 15 points:</strong> 5 for each company that missed its one-month deadline to answer you.</li>
        </ul>
        <p>The first few things count most, so a long list of small things never outweighs a few serious ones. The smaller circle shows what your score will be once you've dealt with the risky findings, the passwords at risk and the late requests. The score can't see what you haven't scanned, so a low score means "nothing risky found so far", not "nothing out there".</p>
      </section>

      <section id="trace">
        <h2>Trace</h2>
        <p>Trace shows you what personal information you've left behind online over the years, and helps you get rid of it. Most people are surprised: an old bank number sent to a friend, a password pasted in a chat, a home address in a server. Trace used to be called Clean.</p>
        <p><strong>How it works.</strong> Start with what's ready in minutes: folders on your computer, your sent mail (connect your mailbox), Telegram, or a WhatsApp chat exported from your phone. At the same time, ask Discord, Instagram, Reddit and the rest for your data: every big platform must let you download it. Tick "I've asked", and Trace reminds you when each export should be ready. Trace reads everything on your computer, and never logs into your accounts to read them. Starting with Discord? The <a href="delete-discord-messages.php">Discord guide</a> walks you through it step by step.</p>
        <ul class="points">
          <li><strong>Sources:</strong> Discord, Reddit, Telegram, X, Facebook, Instagram, TikTok, Snapchat, LinkedIn, WhatsApp, Gmail and other mailboxes, plus any folder on your own computer.</li>
          <li><strong>What it finds:</strong> passwords and access tokens, 2FA backup codes and wallet recovery phrases, payment cards, bank accounts (IBAN), national ID numbers (like a BSN), passport and ID card numbers, street addresses, postcodes, dates of birth, phone numbers, email addresses, IP addresses, number plates, exact locations, crypto wallets, and sensitive files on your computer. Add your own <strong>watch terms</strong> too, like your real name, school or street, which no pattern could guess.</li>
          <li><strong>Pictures, only if you ask.</strong> When you scan folders, two extra checks are off until you tick them for that scan: reading the text in screenshots and scanned documents (with Windows' own text recognition, on your computer), and finding photos that hold the exact spot where they were taken. Nothing is ever scanned on its own, and Trace keeps only what it finds, never copies of your files.</li>
          <li><strong>One clear list.</strong> Everything found sits in a table, sorted by risk, with the message it was in and advice on what to do (for a leaked password: change it now; for a card: call your bank).</li>
          <li><strong>Clean it up, one by one.</strong> "Clean up" takes you through everything in turn: it opens each message where it was posted, you delete it there, and move on to the next. Connected Telegram and email messages are deleted straight from Sompio, and files on your computer go to the Recycle Bin.</li>
        </ul>
        <h3>Requests: get your data, and get it deleted</h3>
        <p>Under the GDPR, companies must give you your data, delete it, or stop using it when you ask, and answer within one month. Trace's <strong>Requests</strong> tab does the paperwork for you:</p>
        <ul class="points">
          <li><strong>Get your data.</strong> Where and how to ask each platform for your export, how long it usually takes, and a reminder for when it should be ready.</li>
          <li><strong>Find it now.</strong> Don't want to wait? Ready-made searches for your watch terms on Discord, Reddit and X, limited to your own posts.</li>
          <li><strong>Ask a company.</strong> Search about 3,000 companies, data brokers included, and pick what you want: delete my data, send me a copy, or stop using it. Trace writes the letter with the right legal articles, and opens it in your own email app or the company's form.</li>
          <li><strong>Track the deadline.</strong> Every request gets its one-month deadline and a reminder. No answer? Trace writes the reminder, and then a complaint to your country's data protection authority, which handles it for free.</li>
        </ul>
        <p class="privacy-line"><strong>Privacy:</strong> your exports, everything Trace finds and your requests stay encrypted on your computer. Unlike online "privacy scanners" and paid removal services, nobody else ever sees your messages or the list of companies you deal with. The company directory is built in, so searching it never goes online.</p>
      </section>

      <section id="vault">
        <h2>Vault</h2>
        <p>Vault is a password manager that keeps every login in one encrypted place on your computer, so you can use a different, strong password everywhere without remembering them. It saves in the open KeePass format, so your passwords also open in other apps and you're never locked in.</p>
        <ul class="points">
          <li><strong>Folders, search and a clear table</strong> of all your logins, with the selected entry shown underneath.</li>
          <li><strong>Fill in passwords for you.</strong> Auto-type types your username and password into the login window with one shortcut (Ctrl+Alt+A). Copying works too, with KeePassXC's shortcuts (Ctrl+B username, Ctrl+C password).</li>
          <li><strong>2FA codes</strong> for sites that use authenticator apps, right next to the password.</li>
          <li><strong>Password generator</strong> for strong passwords or easy-to-type passphrases.</li>
          <li><strong>Health check</strong> that spots weak, reused, old and expired passwords, saved passwords Trace found in your own chats and files (checked on your computer), and an optional breach check (see <a href="#private">Why it's private</a>).</li>
          <li><strong>Extras:</strong> earlier versions of each entry, attachments, a recycle bin, website icons, and import from your browser (Vault shows the steps), KeePass, KeePassXC, Bitwarden and 1Password. Export to KeePass any time.</li>
        </ul>
        <p class="privacy-line"><strong>Privacy:</strong> unlike cloud password managers, your vault is never uploaded or synced to a company's servers. It's one encrypted file on your computer, opened only by your lock.</p>
      </section>

      <section id="extension">
        <h2>Browser extension</h2>
        <p>The Sompio extension brings your Vault into Chrome, Edge, Brave and Firefox, the way password managers like Proton Pass or Bitwarden do: a small Sompio button in login fields, your logins one click away, and an offer to save new ones. The difference: there is no Sompio account or server. The extension talks only to Sompio on your own computer.</p>
        <ul class="points">
          <li><strong>Fill with one click.</strong> Click the Sompio button in a login field and pick a login. Only logins saved for that website are shown.</li>
          <li><strong>Save and update.</strong> After you sign in somewhere new, or change a password, Sompio offers to save it. Choose "Never for this site" to stop asking there.</li>
          <li><strong>Strong passwords when you sign up.</strong> In a "new password" field, "Use a strong password" makes one with Sompio's generator and fills both fields.</li>
          <li><strong>2FA codes</strong> for the website, filled into the code field.</li>
          <li><strong>The toolbar button</strong> shows the logins for the page you're on, searches your vault, copies usernames and passwords, and makes passwords.</li>
        </ul>
        <h3>How it stays private</h3>
        <ul class="points">
          <li><strong>No internet, no account.</strong> The browser starts a small part of Sompio on your computer, which passes messages to the Sompio app. Nothing goes over the network.</li>
          <li><strong>You allow each browser once.</strong> When you click "Connect to Sompio", Sompio asks you to allow it and shows a code that must match the one in your browser. You can remove a browser any time in Vault, Settings.</li>
          <li><strong>Encrypted between the browser and Sompio.</strong> Each allowed browser gets its own secret key, kept inside your encrypted vault. Every request is encrypted with it, and an old request can't be sent again.</li>
          <li><strong>Only for the right website.</strong> Sompio checks the website the browser itself reports, not what the page claims, and refuses to fill a login on a different or look-alike website (like <em>example.com.evil.net</em>).</li>
          <li><strong>Nothing while locked.</strong> When Sompio is locked or closed, the extension can't get anything, and offers to open Sompio so you can unlock it.</li>
          <li><strong>Off until you turn it on.</strong> The extension does nothing until you turn it on in Sompio (Vault, Settings).</li>
        </ul>
        <h3>Setting it up (test version)</h3>
        <p>While Sompio is in testing, the extension isn't in the browser stores yet, so you add it by hand. Sompio installs its files and walks you through it: open <strong>Vault</strong> and click <strong>Set it up</strong> in the "Fill passwords in your browser" bar (or <em>Set it up step by step</em> in Vault's settings). It turns the extension on, opens your browser's extensions page and copies the folder for you. The same steps by hand:</p>
        <ol class="points">
          <li>In Sompio, open <strong>Vault</strong>, then <strong>Settings</strong> (the "Auto-type, import and export" tab), and turn on <strong>Let the Sompio extension use my vault</strong>. Copy the folder it shows.</li>
          <li><strong>Chrome, Edge or Brave:</strong> open <code>chrome://extensions</code> (Edge: <code>edge://extensions</code>), turn on <em>Developer mode</em>, click <em>Load unpacked</em>, and choose that folder.<br><strong>Firefox:</strong> open <code>about:debugging</code>, choose <em>This Firefox</em>, then <em>Load Temporary Add-on</em>, and pick <code>manifest.json</code> in that folder. Firefox removes it when it closes, until the extension is published.</li>
          <li>Restart the browser. Click the Sompio button in the toolbar (pin it from the puzzle-piece menu), then <strong>Connect to Sompio</strong>, and click <strong>Allow</strong> in Sompio when the codes match.</li>
        </ol>
        <p class="privacy-line"><strong>Privacy:</strong> the extension can see the pages you visit, as every password extension must, to find login fields. It keeps nothing about them and sends nothing anywhere: it only asks Sompio, on your computer, for logins for the page you're on.</p>
      </section>

      <section id="limits">
        <h2>Good to know</h2>
        <p>Privacy tools should be honest about their limits, so here are Sompio's:</p>
        <ul class="points">
          <li><strong>Your computer is the vault.</strong> While Sompio is unlocked, it's only as safe as the computer it runs on. No app can fully protect you from malware or someone using your unlocked computer, so keep Windows updated and lock Sompio when you step away.</li>
          <li><strong>Trace finds, you decide.</strong> It spots patterns and your watch terms, but it can miss things and occasionally flag something harmless. Deleting a message doesn't undo that others may have seen it, which is why it tells you to change leaked passwords too. Companies sometimes answer late or partly; Trace helps you follow up, but it can't force them.</li>
          <li><strong>Backups are your job.</strong> With no cloud copy, keep your recovery code (and key picture, if you made one) safe, and export your vault now and then.</li>
        </ul>
        <h3>Questions</h3>
        <dl class="faq">
          <dt>Does Sompio work without internet?</dt><dd>Yes, everything. Only the optional actions listed under <a href="#private">Why it's private</a> go online.</dd>
          <dt>Can you see my passwords or messages?</dt><dd>No. They never leave your computer, and we don't have your keys.</dd>
          <dt>What if I forget my password?</dt><dd>Use your recovery code. Without it, nobody can open your data, including us.</dd>
          <dt>Does the browser extension upload my passwords?</dt><dd>No. It has no server to send them to. It asks Sompio on your computer for one login when you click it, encrypted, and only for the website you're on.</dd>
          <dt>Which systems?</dt><dd><?= e(systems_text()) ?> today<?= strpos(systems_text(), 'Windows,') === 0 ? ' (on Mac and Linux, auto-type and reading text in pictures come later)' : '. Mac and Linux are on the way' ?>. Phones are planned.</dd>
          <dt>Does Trace send emails for me?</dt><dd>No. It writes the letter and opens it in your own email app, so you send it from your own address and the answer comes back to you.</dd>
        </dl>
      </section>

      <div class="docs-end">
        <a class="button primary" href="guides.php">Guides: delete what you've shared</a>
        <a class="button" href="feedback.php">Report a bug</a>
      </div>
    </article>
  </main>

<?php site_footer(); ?>
