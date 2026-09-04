(function () {
  if (window.__uniquityThemeBound) return;
  window.__uniquityThemeBound = true;

  function initMobileNav() {
    document.querySelectorAll('[data-gjs-type="p_website.navbar"], .gjs-navbar').forEach(function (nav) {
      var toggleBtn = nav.querySelector(".mobile-nav-toggle");
      var navMenu = nav.querySelector(".gjs-navbar-links");
      if (!toggleBtn || !navMenu || toggleBtn.dataset.bound) return;
      toggleBtn.dataset.bound = "true";
      toggleBtn.addEventListener("click", function (e) {
        e.stopPropagation();
        var open = !navMenu.classList.contains("open");
        navMenu.classList.toggle("open", open);
        toggleBtn.classList.toggle("open", open);
        toggleBtn.setAttribute("aria-expanded", String(open));
      });
      document.addEventListener("click", function (e) {
        if (!navMenu.contains(e.target) && !toggleBtn.contains(e.target)) {
          navMenu.classList.remove("open");
          toggleBtn.classList.remove("open");
          toggleBtn.setAttribute("aria-expanded", "false");
        }
      });
      navMenu.querySelectorAll("a").forEach(function (link) {
        link.addEventListener("click", function () {
          navMenu.classList.remove("open");
          toggleBtn.classList.remove("open");
          toggleBtn.setAttribute("aria-expanded", "false");
        });
      });
    });
  }

  function initHeroVideos() {
    document.querySelectorAll('[data-gjs-type="p_website.video"]').forEach(function (el) {
      if (!el || el.tagName !== "VIDEO" || el.dataset.larivVideoBound) return;
      el.dataset.larivVideoBound = "true";
      function landscapeSrc() {
        return (el.getAttribute("data-src-landscape") || el.getAttribute("src") || "").trim();
      }
      function portraitSrc() {
        return (el.getAttribute("data-src-portrait") || "").trim();
      }
      function pickSrc() {
        var portrait = portraitSrc();
        if (portrait && window.matchMedia("(orientation: portrait)").matches) return portrait;
        return landscapeSrc();
      }
      function apply() {
        if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
          el.pause();
          el.removeAttribute("autoplay");
          return;
        }
        var next = pickSrc();
        if (!next) return;
        if ((el.getAttribute("src") || "") === next) {
          var same = el.play();
          if (same && same.catch) same.catch(function () {});
          return;
        }
        el.setAttribute("src", next);
        el.load();
        var play = el.play();
        if (play && play.catch) play.catch(function () {});
      }
      apply();
      if (window.matchMedia) {
        var mq = window.matchMedia("(orientation: portrait)");
        if (mq.addEventListener) mq.addEventListener("change", apply);
        else if (mq.addListener) mq.addListener(apply);
      }
    });
  }

  function boot() {
    initMobileNav();
    initHeroVideos();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
