use crate::scheduler::WeeklyScheduler;
use crate::storage::PersistentState;
use crate::types::LimitedString;
use crate::wifi::Auth;
use maud::{DOCTYPE, PreEscaped, html};

fn str_from_limited<const N: usize>(s: &LimitedString<N>) -> &str {
    s.as_utf8_str().unwrap_or("")
}

fn js_str(s: &str) -> alloc::string::String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn auth_options(selected: Auth) -> maud::Markup {
    let options = [
        (Auth::Open, "Open"),
        (Auth::Wpa, "WPA"),
        (Auth::Wpa2, "WPA2"),
        (Auth::Wpa3, "WPA3"),
        (Auth::Wpa2Wpa3, "WPA2/WPA3"),
    ];
    html! {
        @for (auth, label) in &options {
            option selected[*auth == selected] value=(alloc::format!("{:?}", auth)) { (label) }
        }
    }
}

fn scheduler_table_js(scheduler: &WeeklyScheduler) -> alloc::string::String {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    // Build a correct JS array: DATA[day][hour] = [tasks, delay]
    let mut data = alloc::string::String::from("[");
    for (d, daily) in scheduler.daily.iter().enumerate() {
        if d > 0 {
            data.push(',');
        }
        data.push('[');
        for (h, hourly) in daily.hourly.iter().enumerate() {
            if h > 0 {
                data.push(',');
            }
            data.push_str(&alloc::format!(
                "[{},{}]",
                hourly.tasks.0,
                hourly.minutes_delay
            ));
        }
        data.push(']');
    }
    data.push(']');

    let days_js = days
        .iter()
        .map(|d| alloc::format!("'{d}'"))
        .collect::<alloc::vec::Vec<_>>()
        .join(",");

    alloc::format!(
        r#"(function(){{
            const DAYS=[{days_js}];
            const DATA={data};
            window.__schedulerData = DATA;

            const table = document.getElementById('sched-table');

            let html = '<thead><tr><th>Hour</th>';
            DAYS.forEach(d => {{ html += '<th>' + d + '</th>'; }});
            html += '<th></th>';
            html += '</tr></thead><tbody>';

            for (let h = 0; h < 24; h++) {{
                html += '<tr><td style="text-align:center;font-weight:600">'
                      + String(h).padStart(2,'0') + ':00</td>';
                for (let d = 0; d < 7; d++) {{
                    const delay = DATA[d][h][1];
                    html += '<td style="padding:2px">'
                          + '<input type="number" min="0" max="59" style="width:3.5rem" title="Delay (min)" value="' + delay + '" '
                          + 'id="sc_' + d + '_' + h + '" '
                          + 'oninput="window.__schedulerData[' + d + '][' + h + '][1]=+this.value;syncScheduler()">'
                          + '</td>';
                }}
                html += '<td style="padding:2px">'
                      + '<button type="button" style="padding:1px 6px;font-size:.75rem;cursor:pointer" '
                      + 'onclick="copyMonRow(' + h + ')">Copy Monday to all</button>'
                      + '</td>';
                html += '</tr>';
            }}

            html += '</tbody>';
            table.innerHTML = html;
        }})();"#,
        days_js = days_js,
        data = data,
    )
}

pub fn render_page(state: &PersistentState) -> maud::Markup {
    let join = &state.wifi_join_options;
    let join_net = &state.wifi_join_network_config;
    let ap = &state.wifi_access_point_options;
    let ap_net = &state.wifi_access_point_network_config;
    let ntp = &state.ntp_config;
    let weather = &state.weather_options;

    let x_init = alloc::format!(
        "$nextTick(() => {{\
         state.version={version};\
         state.connect_to_wifi={connect_to_wifi};\
         state.wifi_join_options.ssid='{ssid}';\
         state.wifi_join_options.auth='{auth}';\
         state.wifi_join_options.cipher_tkip={cipher_tkip};\
         state.wifi_join_options.cipher_aes={cipher_aes};\
         state.wifi_join_options.passphrase='{passphrase}';\
         state.wifi_join_options.passphrase_is_prehashed={prehashed};\
         state.wifi_join_network_config.ipv4_address='{join_ip}';\
         state.wifi_join_network_config.dhcp={join_dhcp};\
         state.wifi_access_point_options.ssid='{ap_ssid}';\
         state.wifi_access_point_options.channel={ap_channel};\
         state.wifi_access_point_network_config.ipv4_address='{ap_ip}';\
         state.wifi_access_point_network_config.dhcp=false;\
         state.ntp_config.ntp_server='{ntp_server}';\
         state.weather_options.latitude={latitude};\
         state.weather_options.longitude={longitude};\
         }})",
        version = state.version,
        connect_to_wifi = state.connect_to_wifi.as_bool(),
        ssid = js_str(str_from_limited(&join.ssid)),
        auth = js_str(&alloc::format!("{:?}", join.auth)),
        cipher_tkip = join.cipher_tkip.as_bool(),
        cipher_aes = join.cipher_aes.as_bool(),
        passphrase = js_str(str_from_limited(&join.passphrase)),
        prehashed = join.passphrase_is_prehashed.as_bool(),
        join_ip = js_str(&alloc::format!("{:?}", join_net.ipv4_address)),
        join_dhcp = join_net.dhcp.as_bool(),
        ap_ssid = js_str(str_from_limited(&ap.ssid)),
        ap_channel = ap.channel,
        ap_ip = js_str(&alloc::format!("{:?}", ap_net.ipv4_address)),
        ntp_server = js_str(str_from_limited(&ntp.ntp_server)),
        latitude = weather.latitude,
        longitude = weather.longitude,
    );

    let sched_js = scheduler_table_js(&state.scheduler);

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "EPD Frame – Configuration" }
                script defer src="/alpine.js" {}
                style {
                    (PreEscaped(r#"
                        body{font-family:system-ui,sans-serif;max-width:900px;
                             margin:2rem auto;padding:0 1rem}
                        fieldset{border:1px solid #ccc;border-radius:6px;
                                 padding:1rem;margin-bottom:1rem}
                        legend{font-weight:600;padding:0 .4rem}
                        label{display:block;margin-bottom:.5rem}
                        input[type=text],input[type=number],input[type=password],select{
                            width:100%;box-sizing:border-box;padding:.4rem;
                            margin-top:.2rem;border:1px solid #aaa;border-radius:4px}
                        button{padding:.5rem 1.2rem;border:none;border-radius:4px;cursor:pointer}
                        .primary{background:#2980b9;color:#fff}
                        .danger{background:#c0392b;color:#fff}
                        .banner{padding:.6rem 1rem;border-radius:4px;margin-bottom:1rem}
                        .banner-ok{background:#27ae60;color:#fff}
                        .banner-err{background:#c0392b;color:#fff}
                        #sched-table{border-collapse:collapse;width:100%;font-size:.8rem}
                        #sched-table th,#sched-table td{
                            border:1px solid #ddd;padding:3px 4px;text-align:center}
                        #sched-table th{background:#f4f4f4;font-weight:600}
                        .sched-legend{font-size:.75rem;color:#555;margin-top:.4rem}
                    "#))
                }
                script {
                    (PreEscaped(r#"
                    function syncScheduler() {
                        // Called whenever a scheduler cell changes; writes back into Alpine state.
                        if (!window.__alpineState) return;
                        const data = window.__schedulerData;
                        const sched = [];
                        for (let d = 0; d < 7; d++) {
                            const hourly = [];
                            for (let h = 0; h < 24; h++) {
                                hourly.push({ tasks: data[d][h][0], minutes_delay: data[d][h][1] });
                            }
                            sched.push({ hourly });
                        }
                        window.__alpineState.state.scheduler = { daily: sched };
                    }

                    function copyMonRow(h) {
                        const data = window.__schedulerData;
                        const monDelay = data[0][h][1];
                        const monTasks = data[0][h][0];
                        for (let d = 1; d < 7; d++) {
                            data[d][h][0] = monTasks;
                            data[d][h][1] = monDelay;
                            const input = document.getElementById('sc_' + d + '_' + h);
                            if (input) input.value = monDelay;
                        }
                        syncScheduler();
                    }

                    function appState() {
                        return {
                            banner: '', bannerOk: true,
                            state: {
                                version: 0,
                                connect_to_wifi: false,
                                wifi_join_options: {
                                    ssid: '', auth: 'Open',
                                    cipher_tkip: false, cipher_aes: false,
                                    passphrase: '', passphrase_is_prehashed: false
                                },
                                wifi_join_network_config:  { ipv4_address: '0.0.0.0/0', dhcp: false },
                                wifi_access_point_options: { ssid: '', channel: 1 },
                                wifi_access_point_network_config: { ipv4_address: '0.0.0.0/0', dhcp: false },
                                ntp_config: { ntp_server: '' },
                                weather_options: { latitude: 0.0, longitude: 0.0 },
                                scheduler: { daily: [] }
                            },
                            bannerClass() {
                                return this.bannerOk ? 'banner banner-ok' : 'banner banner-err';
                            },
                            async post(url) {
                                // Flush scheduler before serialising.
                                syncScheduler();
                                try {
                                    const r = await fetch(url, {
                                        method: 'POST',
                                        headers: {'Content-Type': 'application/json'},
                                        body: JSON.stringify(this.state)
                                    });
                                    this.bannerOk = r.ok;
                                    this.banner = r.ok ? '✓ Done.' : '✗ Error: ' + r.status;
                                } catch(e) {
                                    this.bannerOk = false;
                                    this.banner = '✗ ' + e;
                                }
                                setTimeout(() => this.banner = '', 4000);
                            },
                            init() { window.__alpineState = this; }
                        };
                    }
                    "#))
                }
            }
            body x-data="appState()" x-init=(x_init) {
                h1 { "EPD Frame – Configuration" }

                (PreEscaped(r#"<div x-show="banner" x-text="banner" :class="bannerClass()"></div>"#))

                // ── Wi-Fi client ──────────────────────────────────────────────
                fieldset {
                    legend { "Wi-Fi Client" }
                    label {
                        input type="checkbox" x-model="state.connect_to_wifi";
                        " Connect to Wi-Fi"
                    }
                    label {
                        "SSID"
                        input type="text" maxlength="32" x-model="state.wifi_join_options.ssid";
                    }
                    label {
                        "Passphrase"
                        input type="password" maxlength="64"
                            x-model="state.wifi_join_options.passphrase";
                    }
                    label {
                        "Authentication"
                        select x-model="state.wifi_join_options.auth" {
                            (auth_options(join.auth))
                        }
                    }
                    label {
                        input type="checkbox" x-model="state.wifi_join_options.cipher_tkip";
                        " TKIP cipher"
                    }
                    label {
                        input type="checkbox" x-model="state.wifi_join_options.cipher_aes";
                        " AES cipher"
                    }
                    fieldset {
                        legend { "Network" }
                        label {
                            input type="checkbox" x-model="state.wifi_join_network_config.dhcp";
                            " Use DHCP"
                        }
                        label {
                            "Static IP / CIDR (e.g. 192.168.1.50/24)"
                            input type="text" x-model="state.wifi_join_network_config.ipv4_address";
                        }
                    }
                }

                // ── Access point ──────────────────────────────────────────────
                fieldset {
                    legend { "Access Point" }
                    label {
                        "SSID"
                        input type="text" maxlength="32"
                            x-model="state.wifi_access_point_options.ssid";
                    }
                    label {
                        "Channel (1–13)"
                        input type="number" min="1" max="13"
                            x-model="state.wifi_access_point_options.channel";
                    }
                    fieldset {
                        legend { "Network" }
                        // DHCP is intentionally hidden for the AP — it must always be
                        // disabled to keep the static IP required for client connectivity.
                        label {
                            "Static IP / CIDR"
                            input type="text"
                                x-model="state.wifi_access_point_network_config.ipv4_address";
                        }
                    }
                }

                // ── NTP ───────────────────────────────────────────────────────
                fieldset {
                    legend { "NTP" }
                    label {
                        "Server"
                        input type="text" x-model="state.ntp_config.ntp_server";
                    }
                }

                // ── Weather ───────────────────────────────────────────────────
                fieldset {
                    legend { "Weather" }
                    label {
                        "Latitude"
                        (PreEscaped(r#"<input type="number" step="0.001" min="-90" max="90" x-model.number="state.weather_options.latitude">"#))
                    }
                    label {
                        "Longitude"
                        (PreEscaped(r#"<input type="number" step="0.001" min="-180" max="180" x-model.number="state.weather_options.longitude">"#))
                    }
                }

                // ── Scheduler ─────────────────────────────────────────────────
                fieldset {
                    legend { "Scheduler" }
                    p.sched-legend {
                        "Each cell: "
                        strong { "delay" }
                        " (minutes from now, 0–59). "
                        "Columns = weekdays (Mon–Sun), rows = hours (00–23)."
                    }
                    (PreEscaped(r#"<div style="overflow-x:auto"><table id="sched-table"></table></div>"#))
                    // Populate the table after the DOM is ready.
                    script { (PreEscaped(sched_js)) }
                }

                (PreEscaped(r#"
                <div style="display:flex;gap:.5rem;margin-top:1rem">
                    <button type="button" class="danger" @click="post('/save_restart')">Save &amp; Restart</button>
                </div>
                "#))
            }
        }
    }
}
