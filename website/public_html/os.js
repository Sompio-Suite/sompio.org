// Download page: shows the download and install steps for the visitor's system, with tabs for the
// others. Without this script, every system's section simply shows.
(function () {
  var ua = navigator.userAgent;
  var guess = /Mac OS X|Macintosh/.test(ua) && !/iPhone|iPad/.test(ua) ? "mac" : /Linux/.test(ua) && !/Android/.test(ua) ? "linux" : "windows";
  var tabs = document.querySelectorAll(".os-tab");
  if (!tabs.length) return;
  var offered = Array.prototype.map.call(tabs, function (t) { return t.getAttribute("data-os"); });
  var fromLink = (location.hash.match(/^#for-(windows|mac|linux)$/) || [])[1];
  function show(os) {
    if (offered.indexOf(os) < 0) os = "windows";
    document.querySelectorAll(".os-panel").forEach(function (p) { p.hidden = p.getAttribute("data-os") !== os; });
    tabs.forEach(function (t) {
      var on = t.getAttribute("data-os") === os;
      t.classList.toggle("on", on);
      t.setAttribute("aria-selected", on ? "true" : "false");
    });
  }
  tabs.forEach(function (t) {
    t.addEventListener("click", function (e) {
      e.preventDefault();
      show(t.getAttribute("data-os"));
      history.replaceState(null, "", "#for-" + t.getAttribute("data-os"));
    });
  });
  show(fromLink || guess);
})();

// After a download starts (the front page's big button, or a button in the download section): say
// so, and show the install steps for that system, since the computer is about to ask questions.
(function () {
  var note = document.querySelector(".dl-started");
  function started(os) {
    var tab = os && document.querySelector('.os-tab[data-os="' + os + '"]');
    if (tab) tab.click();
    if (note) note.hidden = false;
    var steps = document.getElementById("install");
    if (steps) setTimeout(function () { steps.scrollIntoView({ behavior: "smooth", block: "start" }); }, 400);
  }
  document.querySelectorAll(".dl-go, .os-panel .button.primary.big").forEach(function (b) {
    b.addEventListener("click", function () {
      var panel = b.closest(".os-panel");
      started(b.getAttribute("data-os") || (panel && panel.getAttribute("data-os")));
    });
  });
})();

// Front page on a phone: send the download link to yourself
(function () {
  var b = document.querySelector("[data-share]");
  if (!b) return;
  b.addEventListener("click", function () {
    var url = b.getAttribute("data-share");
    var done = document.querySelector("[data-share-done]");
    if (navigator.share) {
      navigator.share({ title: "Sompio", text: "Download Sompio on my computer", url: url }).catch(function () {});
    } else if (navigator.clipboard) {
      navigator.clipboard.writeText(url).then(function () { if (done) done.hidden = false; });
    } else {
      location.href = "mailto:?subject=Sompio&body=" + encodeURIComponent(url);
    }
  });
})();
