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

  function boot() {
    initMobileNav();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
