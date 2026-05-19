/// Client-side tag filtering for nhentai.net.
///
/// This script only activates on nhentai.net pages (hostname check).
/// It provides:
/// - Gallery filtering — removes matching galleries from the DOM
/// - Click-to-blacklist on gallery pages (toggles between DEFAULT and AVOIDED)
///
/// Tags use a three-state system (matching NClientV3):
///   DEFAULT (0) — not filtered
///   AVOIDED (1) — excluded from results
///   ACCEPTED (2) — only show galleries with this tag
///
/// The full three-state management UI lives in the app's custom frontend.
/// On nhentai.net pages, clicking a tag toggles between DEFAULT and AVOIDED.
///
/// All data is stored in localStorage, no Rust backend or IPC involved.
pub fn build_tag_blacklist_script() -> String {
    r#"(function () {
    'use strict';

    var SITE_HOSTNAME = 'nhentai.net';
    if (window.location.hostname !== SITE_HOSTNAME &&
        !window.location.hostname.endsWith('.' + SITE_HOSTNAME)) {
        return;
    }

    var STORAGE_KEY = 'nhentai_tag_state_v4';

    function load() {
        try { return JSON.parse(localStorage.getItem(STORAGE_KEY)) || {}; } catch (_) { return {}; }
    }
    function save(state) { localStorage.setItem(STORAGE_KEY, JSON.stringify(state)); }

    var tagState = load();

    // Migrate old format (pre-v4, no status field) — assume AVOIDED
    var needsSave = false;
    Object.keys(tagState).forEach(function (id) {
        var t = tagState[id];
        if (t && t.status === undefined) {
            t.status = 1; // AVOIDED
            needsSave = true;
        }
    });
    if (needsSave) save(tagState);

    var TS = { DEFAULT: 0, AVOIDED: 1, ACCEPTED: 2 };

    function removeGallery(id) {
        var el = document.querySelector('a[href^="/g/' + id + '/"]');
        if (!el) return;
        var cover = el.closest('.gallery') || el.parentElement;
        if (cover && cover.parentElement) cover.remove();
    }

    function parseGalleries(data) {
        if (Array.isArray(data)) return data;
        if (data && Array.isArray(data.result)) return data.result;
        return [];
    }

    function filterGalleries() {
        var blocked = [];
        Object.keys(tagState).forEach(function (id) {
            var t = tagState[id];
            if (t && t.status === TS.AVOIDED) blocked.push(id);
        });
        if (blocked.length === 0) return;

        var scripts = document.querySelectorAll(
            'script[type="application/json"][data-sveltekit-fetched]'
        );
        scripts.forEach(function (s) {
            var data;
            try { data = JSON.parse(s.textContent); } catch (_) { return; }
            parseGalleries(data).forEach(function (g) {
                if (!g || !g.tag_ids || !g.id) return;
                for (var i = 0; i < g.tag_ids.length; i++) {
                    if (blocked.indexOf(String(g.tag_ids[i])) !== -1) {
                        removeGallery(g.id);
                        return;
                    }
                }
            });
        });
    }

    function tagTypeFromHref(href) {
        try {
            var p = new URL(href, window.location.href).pathname;
            var seg = p.split('/').filter(Boolean);
            if (seg.length >= 2) {
                switch (seg[0]) {
                    case 'tag':        return 'tag';
                    case 'artist':     return 'artist';
                    case 'character':  return 'character';
                    case 'parody':     return 'parody';
                    case 'group':      return 'group';
                    case 'category':   return 'category';
                    case 'language':   return 'language';
                }
            }
        } catch (_) {}
        return 'tag';
    }

    function tagIdFromHref(href) {
        var m = href.match(/\/(?:tag|artist|character|parody|group|category|language)\/(\d+)\//);
        return m ? m[1] : null;
    }

    function tagNameFromEl(el) {
        var span = el.querySelector('.name');
        if (span) return span.textContent.trim();
        return el.textContent.trim();
    }

    function toggleTagStatus(e) {
        var anchor = e.currentTarget;
        var tid = tagIdFromHref(anchor.getAttribute('href'));
        if (!tid) return;
        e.preventDefault();
        e.stopPropagation();

        var t = tagState[tid];
        if (t && t.status !== TS.DEFAULT) {
            // Remove (set to DEFAULT)
            delete tagState[tid];
            anchor.classList.remove('tag-avoided');
        } else {
            // Set to AVOIDED
            var name = tagNameFromEl(anchor);
            var type = tagTypeFromHref(anchor.getAttribute('href'));
            tagState[tid] = { status: TS.AVOIDED, name: name, type: type };
            anchor.classList.add('tag-avoided');
        }
        save(tagState);
        filterGalleries();
    }

    function attachTagHandlers() {
        document.querySelectorAll('a.tag').forEach(function (a) {
            if (a._bl_hooked) return;
            var tid = tagIdFromHref(a.getAttribute('href'));
            if (!tid) return;
            a._bl_hooked = true;
            var t = tagState[tid];
            if (t && t.status === TS.AVOIDED) a.classList.add('tag-avoided');
            a.addEventListener('click', toggleTagStatus);
        });
    }

    var style = document.createElement('style');
    style.textContent = 'a.tag-avoided{outline:2px solid #e74c3c!important;outline-offset:2px!important;border-radius:3px!important;cursor:pointer!important}';
    if (document.head) document.head.appendChild(style);

    function init() {
        filterGalleries();
        attachTagHandlers();
    }

    var _pushState = history.pushState;
    history.pushState = function () {
        _pushState.apply(this, arguments);
        setTimeout(init, 100);
    };
    var _replaceState = history.replaceState;
    history.replaceState = function () {
        _replaceState.apply(this, arguments);
        setTimeout(init, 100);
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
}());
"#.to_string()
}
