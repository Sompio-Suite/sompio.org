<?php
// Guide: removing the location hidden in photos before sharing them.
declare(strict_types=1);
require __DIR__ . '/private/lib.php';
page_head('How to remove the location from your photos · Sompio', 'Phones save the exact spot where each photo was taken. How to check a photo, remove the location on Windows, Mac, Linux, iPhone and Android, and stop it being saved at all.');
site_header('guide');
?>
  <main class="wrap docs guide">
    <aside class="docs-toc" aria-label="On this page">
      <p class="toc-title">On this page</p>
      <a href="#why">What's hidden in a photo</a>
      <a href="#check">1. Check a photo</a>
      <a href="#computer">2. Remove it on a computer</a>
      <a href="#phone">3. Remove it on a phone</a>
      <a href="#stop">4. Stop saving it</a>
      <a href="#old">Photos you already shared</a>
      <a href="guides.php" class="toc-back">← All guides</a>
    </aside>

    <article class="docs-body">
      <p class="eyebrow">Photos · updated November 2026</p>
      <h1>How to remove the location from your photos</h1>
      <p class="lead">Most phones save the exact spot where each photo was taken, often to within a few metres, inside the photo file itself. Share the wrong file and you may be sharing your home address. Here's how to check, remove it, and stop it happening.</p>

      <section id="why">
        <h2>What's hidden in a photo</h2>
        <p>Besides the picture, a photo file holds extra details called metadata (or EXIF): the date and time, the phone model, and, if location was on, the GPS position. You can't see it in the picture, but anyone with the file can read it in seconds.</p>
        <ul class="points">
          <li><strong>Many social apps remove it when they compress your photo,</strong> but not all of them, and not always.</li>
          <li><strong>It usually stays</strong> when you send a photo as a file or document, attach it to an email, or share a link from cloud storage.</li>
          <li><strong>Photos taken at home are the risk.</strong> A picture of your cat or a new purchase can show exactly where you live.</li>
        </ul>
      </section>

      <section id="check">
        <h2>1. Check a photo</h2>
        <ul class="points">
          <li><strong>Windows:</strong> right-click the photo, choose <strong>Properties</strong>, then the <strong>Details</strong> tab. Look for <strong>GPS</strong>, Latitude and Longitude.</li>
          <li><strong>Mac:</strong> open it in Preview, choose <strong>Tools</strong>, <strong>Show Inspector</strong>, and look for a <strong>GPS</strong> tab.</li>
          <li><strong>iPhone:</strong> open the photo in Photos and swipe up. A map means the location is saved.</li>
          <li><strong>Android:</strong> open the photo in your gallery or Google Photos and swipe up or tap the info button.</li>
        </ul>
        <p>Checking a whole folder this way takes ages. <a href="docs.php#trace">Sompio Trace</a> can check every photo in folders you choose at once: tick <strong>Check where photos were taken</strong> and it lists the photos that give away a precise location, on your own computer.</p>
      </section>

      <section id="computer">
        <h2>2. Remove it on a computer</h2>
        <h3>Windows</h3>
        <ol class="guide-steps">
          <li>Select one or more photos, right-click, and choose <strong>Properties</strong>, then <strong>Details</strong>.</li>
          <li>Click <strong>Remove Properties and Personal Information</strong> at the bottom.</li>
          <li>Choose <strong>Create a copy with all possible properties removed</strong>, or tick only the GPS lines to remove from the original. Click <strong>OK</strong>.</li>
        </ol>
        <h3>Mac</h3>
        <ol class="guide-steps">
          <li>Open the photo in <strong>Preview</strong>, choose <strong>Tools</strong>, then <strong>Show Inspector</strong>.</li>
          <li>Open the <strong>GPS</strong> tab and click <strong>Remove Location Info</strong>.</li>
        </ol>
        <p>In the Photos app, you can also select photos and choose <strong>Image</strong>, <strong>Location</strong>, <strong>Hide Location</strong>.</p>
        <h3>Linux</h3>
        <p>With <code>exiftool</code> (in most distributions' software centre), this removes the location from every photo in the current folder and keeps a backup of each original:</p>
        <p><code>exiftool -gps:all= *.jpg</code></p>
      </section>

      <section id="phone">
        <h2>3. Remove it on a phone</h2>
        <ul class="points">
          <li><strong>iPhone:</strong> when you share from Photos, tap <strong>Options</strong> at the top of the share screen and turn off <strong>Location</strong>. It applies to that share only.</li>
          <li><strong>Android:</strong> in the photo's info, many galleries have an option to remove or edit the location. In Google Photos, look for a setting that removes the location from photos you share by link. Names differ per phone.</li>
        </ul>
      </section>

      <section id="stop">
        <h2>4. Stop saving it</h2>
        <ul class="points">
          <li><strong>iPhone:</strong> Settings, <strong>Privacy &amp; Security</strong>, <strong>Location Services</strong>, <strong>Camera</strong>, then <strong>Never</strong>.</li>
          <li><strong>Android:</strong> open the Camera app's settings and turn off the option called something like <strong>Location tags</strong> or <strong>Save location</strong>.</li>
        </ul>
        <p>You lose the map view of your photos, but nothing you share can give away where it was taken. A middle way: leave it on, and remove the location before sharing photos taken at home.</p>
      </section>

      <section id="old">
        <h2>Photos you already shared</h2>
        <p>A photo sent as a file stays as it was in the other person's chat or mailbox. If one showed somewhere sensitive, like your home, delete it where you can and ask the other person to delete their copy. For photos on websites and platforms you can't reach anymore, see <a href="make-companies-delete-your-data.php">How to make companies delete your data</a>.</p>
      </section>

<?php more_guides('remove-location-from-photos.php'); ?>
    </article>
  </main>
<?php site_footer(); ?>
