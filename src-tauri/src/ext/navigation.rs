/// Build the navigation init script, including the app origin for the
/// "Back to App" button on nhentai.net pages and auto-redirect after
/// login.
pub fn build_navigation_init(app_origin: &str) -> String {
    format!(
        r#"(function () {{
    'use strict';

    var APP_ORIGIN = '{}';
    var SITE_HOSTNAME = 'nhentai.net';
    var LS_LOGIN_FLAG = 'nhentai_was_on_login';

    function tauriInvoke(command, payload) {{
        try {{
            if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {{
                return window.__TAURI_INTERNALS__.invoke(command, payload || {{}});
            }}
            if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {{
                return window.__TAURI__.invoke(command, payload || {{}});
            }}
            if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {{
                return window.__TAURI__.tauri.invoke(command, payload || {{}});
            }}
            if (typeof window.__TAURI_INVOKE__ === 'function') {{
                return window.__TAURI_INVOKE__(command, payload || {{}});
            }}
        }} catch (err) {{
            console.warn('Tauri invoke failed:', command, err);
        }}
        console.warn('Tauri invoke unavailable:', command);
        return Promise.reject(new Error('not in Tauri'));
    }}

    function goToApp() {{
        tauriInvoke('open_in_main_window_cmd', {{
            url: APP_ORIGIN + '/index.html'
        }});
    }}

    /* ------------------------------------------------------------------ */
    /*  Login redirect — auto-return to app after successful login.       */
    /*  Uses sessionStorage so the flag survives SPA navigations but not  */
    /*  new tabs.                                                         */
    /* ------------------------------------------------------------------ */

    var currentPath = window.location.pathname;
    var isLoginPage = currentPath === '/login' || currentPath.startsWith('/login/');
    var isSite = window.location.hostname === SITE_HOSTNAME ||
                 window.location.hostname.endsWith('.' + SITE_HOSTNAME);

    /* ------------------------------------------------------------------ */
    /*  Cookie forwarding — send WebView cookies to Rust reqwest client   */
    /*  Polls for 5s after page load since Cloudflare may take time.      */
    /* ------------------------------------------------------------------ */

    function forwardCookies() {{
        try {{
            var c = document.cookie;
            if (!c) return;
            var pairs = c.split(';');
            var count = 0;
            for (var i = 0; i < pairs.length; i++) {{
                var p = pairs[i].trim();
                if (!p) continue;
                // Skip cookies already set via the Name check
                tauriInvoke('api_set_cookie', {{ cookieStr: p }}).catch(function () {{}});
                count++;
            }}
        }} catch (_) {{}}
    }}

    if (isSite) {{
        // Forward cookies immediately and then every second for 5 seconds
        // to catch Cloudflare cookies that arrive after page load.
        var _cfTries = 0;
        function tryCookies() {{
            forwardCookies();
            _cfTries++;
            if (_cfTries < 5) setTimeout(tryCookies, 1000);
        }}
        if (document.readyState === 'complete') {{
            setTimeout(tryCookies, 200);
        }} else {{
            window.addEventListener('load', function () {{
                setTimeout(tryCookies, 200);
            }});
        }}

        if (isLoginPage) {{
            // Mark that the user is on the login page
            try {{ sessionStorage.setItem(LS_LOGIN_FLAG, '1'); }} catch (_) {{}}
        }} else {{
            // Check if we just came from the login page
            var wasOnLogin = false;
            try {{ wasOnLogin = sessionStorage.getItem(LS_LOGIN_FLAG) === '1'; }} catch (_) {{}}
            if (wasOnLogin) {{
                try {{ sessionStorage.removeItem(LS_LOGIN_FLAG); }} catch (_) {{}}
                // Show a brief notification before redirecting
                var note = document.createElement('div');
                note.textContent = 'Login successful \u2014 returning to app...';
                Object.assign(note.style, {{
                    position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%, -50%)',
                    zIndex: '9999999',
                    padding: '16px 28px', background: 'rgba(39,174,96,0.95)', color: '#fff',
                    borderRadius: '6px', fontFamily: 'sans-serif', fontSize: '15px',
                    boxShadow: '0 4px 20px rgba(0,0,0,0.5)',
                }});
                document.body.appendChild(note);
                setTimeout(goToApp, 1500);
            }}
        }}
    }}

    if (isSite) {{
        /* ------------------------------------------------------------------ */
        /*  Back to App button (shown on nhentai.net pages)                   */
        /* ------------------------------------------------------------------ */
        function injectBackButton() {{
            if (document.getElementById('nhentai-back-btn')) return;

            var btn = document.createElement('div');
            btn.id = 'nhentai-back-btn';
            btn.textContent = '\u2190 App';
            Object.assign(btn.style, {{
                position: 'fixed', top: '12px', left: '12px', zIndex: '999999',
                padding: '6px 14px', background: '#2f2f2f', color: '#d5d7de',
                border: '1px solid #444', borderRadius: '4px',
                cursor: 'pointer', fontFamily: 'sans-serif', fontSize: '13px',
                boxShadow: '0 2px 8px rgba(0,0,0,0.4)', userSelect: 'none',
            }});
            btn.addEventListener('mouseenter', function () {{
                btn.style.background = '#3f3f3f';
            }});
            btn.addEventListener('mouseleave', function () {{
                btn.style.background = '#2f2f2f';
            }});
            btn.addEventListener('click', goToApp);
            document.body.appendChild(btn);

            /* ------------------------------------------------------------------ */
            /*  Link interception                                                 */
            /* ------------------------------------------------------------------ */

            function log(who, url) {{
                try {{ console.log('[nhentai-nav]', who, url); }} catch (_) {{}}
            }}

            function isInternal(url) {{
                try {{
                    var parsed = new URL(url, window.location.href);
                    return (
                        parsed.hostname === SITE_HOSTNAME ||
                        parsed.hostname.endsWith('.' + SITE_HOSTNAME)
                    );
                }} catch (_) {{
                    return false;
                }}
            }}

            function openExternal(url) {{
                log('openExternal', url);
                tauriInvoke('plugin:opener|open_url', {{ url: url }});
            }}

            function openInMain(url) {{
                log('openInMain', url);
                tauriInvoke('open_in_main_window_cmd', {{ url: url }});
            }}

            function isLoginUrl(url) {{
                try {{
                    var p = new URL(url).pathname;
                    var match = p === '/login' || p.startsWith('/login/') ||
                                p === '/register' || p.startsWith('/register/') ||
                                p === '/auth' || p.startsWith('/auth/');
                    log('isLoginUrl(' + p + ') = ' + match);
                    return match;
                }} catch (_) {{
                    return false;
                }}
            }}

            var _open = window.open.bind(window);
            window.open = function (url, target, features) {{
                log('window.open', url);
                if (!url) return _open(url, target, features);

                var resolved;
                try {{ resolved = new URL(url, window.location.href).href; }}
                catch (_) {{ return _open(url, target, features); }}

                if (isInternal(resolved)) {{
                    openInMain(resolved);
                    return null;
                }}

                openExternal(resolved);
                return null;
            }};

            window.addEventListener('click', function (e) {{
                if (e.defaultPrevented || e.button !== 0) return;

                var anchor = e.composedPath().find(function (el) {{
                    return el instanceof Node && el.nodeName &&
                           el.nodeName.toUpperCase() === 'A';
                }});
                if (!anchor || !anchor.href) return;

                if (anchor.hasAttribute('download')) return;

                var opensNew = anchor.target === '_blank' || e.ctrlKey || e.metaKey || e.shiftKey;
                if (!opensNew) return;

                var proto;
                try {{ proto = new URL(anchor.href).protocol; }} catch (_) {{ return; }}
                if (!['http:', 'https:', 'mailto:', 'tel:'].includes(proto)) return;

                e.preventDefault();

                if (isInternal(anchor.href)) {{
                    openInMain(anchor.href);
                }} else {{
                    openExternal(anchor.href);
                }}
            }}, true);

            injectBackButton();
        }}
    }}


    function isInternal(url) {{
        try {{
            var parsed = new URL(url, window.location.href);
            return (
                parsed.hostname === SITE_HOSTNAME ||
                parsed.hostname.endsWith('.' + SITE_HOSTNAME)
            );
        }} catch (_) {{
            return false;
        }}
    }}

    function openExternal(url) {{
        log('openExternal', url);
        tauriInvoke('plugin:opener|open_url', {{ url: url }});
    }}

    function openInMain(url) {{
        log('openInMain', url);
        tauriInvoke('open_in_main_window_cmd', {{ url: url }});
    }}

    function isLoginUrl(url) {{
        try {{
            var p = new URL(url).pathname;
            var match = p === '/login' || p.startsWith('/login/') ||
                        p === '/register' || p.startsWith('/register/') ||
                        p === '/auth' || p.startsWith('/auth/');
            log('isLoginUrl(' + p + ') = ' + match);
            return match;
        }} catch (_) {{
            return false;
        }}
    }}

    var _open = window.open.bind(window);
    window.open = function (url, target, features) {{
        log('window.open', url);
        if (!url) return _open(url, target, features);

        var resolved;
        try {{ resolved = new URL(url, window.location.href).href; }}
        catch (_) {{ return _open(url, target, features); }}

        if (isInternal(resolved)) {{
            openInMain(resolved);
            return null;
        }}

        openExternal(resolved);
        return null;
    }};

    window.addEventListener('click', function (e) {{
        if (e.defaultPrevented || e.button !== 0) return;

        var anchor = e.composedPath().find(function (el) {{
            return el instanceof Node && el.nodeName &&
                   el.nodeName.toUpperCase() === 'A';
        }});
        if (!anchor || !anchor.href) return;

        if (anchor.hasAttribute('download')) return;

        var opensNew = anchor.target === '_blank' || e.ctrlKey || e.metaKey || e.shiftKey;
        if (!opensNew) return;

        var proto;
        try {{ proto = new URL(anchor.href).protocol; }} catch (_) {{ return; }}
        if (!['http:', 'https:', 'mailto:', 'tel:'].includes(proto)) return;

        e.preventDefault();

        if (isInternal(anchor.href)) {{
            openInMain(anchor.href);
        }} else {{
            openExternal(anchor.href);
        }}
    }}, true);

    injectBackButton();
}}());
"#, app_origin)
}
