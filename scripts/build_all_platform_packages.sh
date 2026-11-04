#!/usr/bin/env bash
# ==============================================================================
# Script: build_all_platform_packages.sh
# Purpose: Builds Native Rust Desktop Installers (Windows .exe/.zip, macOS .app,
#          Linux .tar.gz) and Signed Android Mobile APK (.apk) on CircleCI.
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${WORKSPACE_DIR}/dist"
DOWNLOADS_DIR="${DIST_DIR}/downloads"
BUILD_TMP="/tmp/attendance_platform_build"

rm -rf "${BUILD_TMP}"
mkdir -p "${DIST_DIR}" "${DOWNLOADS_DIR}" "${BUILD_TMP}"

echo "===================================================================="
echo "🚀 1. Building Linux x86_64 Native Rust Desktop & Server Package"
echo "===================================================================="
LINUX_STAGE="${BUILD_TMP}/SmartAttendance-Desktop-Linux-x86_64"
mkdir -p "${LINUX_STAGE}"

cp "${WORKSPACE_DIR}/target/release/attendance-desktop" "${LINUX_STAGE}/attendance-desktop"
cp "${WORKSPACE_DIR}/target/release/attendance-api" "${LINUX_STAGE}/attendance-api"
cp "${WORKSPACE_DIR}/target/release/attendance-admin" "${LINUX_STAGE}/attendance-cli"
cp "${WORKSPACE_DIR}/target/release/attendance-device-agent" "${LINUX_STAGE}/attendance-device-agent"
chmod +x "${LINUX_STAGE}/attendance-"*

cat > "${LINUX_STAGE}/install-linux-desktop.sh" << 'EOF'
#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/share/SmartAttendanceDesktop"
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"

mkdir -p "$INSTALL_DIR" "$BIN_DIR" "$APP_DIR"
cp "$DIR/attendance-desktop" "$INSTALL_DIR/attendance-desktop"
chmod +x "$INSTALL_DIR/attendance-desktop"
ln -sf "$INSTALL_DIR/attendance-desktop" "$BIN_DIR/attendance-desktop"

cat > "$APP_DIR/smart-attendance-desktop.desktop" << DESKTOP
[Desktop Entry]
Version=1.0
Type=Application
Name=Smart Attendance Desktop
Comment=Pure Rust Biometric & Millions-Scale Attendance Desktop App
Exec=$INSTALL_DIR/attendance-desktop
Terminal=false
Categories=Office;Utility;
DESKTOP

chmod +x "$APP_DIR/smart-attendance-desktop.desktop"
if [ -d "$HOME/Desktop" ]; then
    cp "$APP_DIR/smart-attendance-desktop.desktop" "$HOME/Desktop/SmartAttendanceDesktop.desktop"
    chmod +x "$HOME/Desktop/SmartAttendanceDesktop.desktop"
fi

echo "✅ Smart Attendance Desktop installed successfully on Linux!"
echo "Launching Smart Attendance Desktop..."
nohup "$INSTALL_DIR/attendance-desktop" >/dev/null 2>&1 &
EOF
chmod +x "${LINUX_STAGE}/install-linux-desktop.sh"

cat > "${LINUX_STAGE}/README-Linux.txt" << 'EOF'
Smart Attendance System — Native Rust Desktop for Linux (x86_64)
================================================================
1. Run ./install-linux-desktop.sh to install desktop shortcuts and launch.
2. Or run ./attendance-desktop directly.
3. Local Desktop Bridge runs on http://127.0.0.1:18080 and connects to the live cloud server or local :8080 server.
EOF

tar -czf "${DOWNLOADS_DIR}/SmartAttendance-Desktop-Linux-x86_64.tar.gz" -C "${BUILD_TMP}" "SmartAttendance-Desktop-Linux-x86_64"
echo "✔ Created ${DOWNLOADS_DIR}/SmartAttendance-Desktop-Linux-x86_64.tar.gz"

echo "===================================================================="
echo "🪟 2. Cross-Compiling Windows 64-bit Native Rust Desktop (.exe)"
echo "===================================================================="
rustup target add x86_64-pc-windows-gnu
cd "${WORKSPACE_DIR}"
cargo build --release -p attendance-desktop --target x86_64-pc-windows-gnu

WIN_EXE="${WORKSPACE_DIR}/target/x86_64-pc-windows-gnu/release/attendance-desktop.exe"
file "${WIN_EXE}"

WIN_STAGE="${BUILD_TMP}/SmartAttendance-Windows-x64"
mkdir -p "${WIN_STAGE}"
cp "${WIN_EXE}" "${WIN_STAGE}/attendance-desktop.exe"

cat > "${WIN_STAGE}/Launch-SmartAttendance.bat" << 'EOF'
@echo off
title Smart Attendance System — Native Rust Desktop (Windows x64)
echo ======================================================================
echo   Starting Smart Attendance Rust Desktop Application...
echo ======================================================================
start "" "%~dp0attendance-desktop.exe"
EOF

cat > "${WIN_STAGE}/Install-Windows-Desktop.bat" << 'EOF'
@echo off
setlocal
set "TARGET_DIR=%LOCALAPPDATA%\SmartAttendanceDesktop"
echo ======================================================================
echo   Installing Smart Attendance Desktop (Rust Native Windows x64)...
echo ======================================================================
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
copy /Y "%~dp0attendance-desktop.exe" "%TARGET_DIR%\attendance-desktop.exe" >nul
copy /Y "%~dp0Launch-SmartAttendance.bat" "%TARGET_DIR%\Launch-SmartAttendance.bat" >nul

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ws = New-Object -ComObject WScript.Shell; ^
   $s = $ws.CreateShortcut([Environment]::GetFolderPath('Desktop') + '\Smart Attendance Desktop.lnk'); ^
   $s.TargetPath = '%TARGET_DIR%\attendance-desktop.exe'; ^
   $s.WorkingDirectory = '%TARGET_DIR%'; ^
   $s.Description = 'Smart Attendance Rust Desktop App'; ^
   $s.Save();"

echo.
echo [SUCCESS] Installed to %TARGET_DIR% and created Desktop shortcut!
echo Launching Smart Attendance Desktop...
start "" "%TARGET_DIR%\attendance-desktop.exe"
EOF

cat > "${WIN_STAGE}/README-Windows.txt" << 'EOF'
Smart Attendance System — Native Rust Desktop for Windows 10/11 (64-bit)
========================================================================
Option A: Double-click Install-Windows-Desktop.bat to install to your PC and create a Desktop Shortcut.
Option B: Double-click attendance-desktop.exe to run immediately in Portable Mode.
EOF

(cd "${WIN_STAGE}" && zip -q -r "${DOWNLOADS_DIR}/SmartAttendance-Desktop-Windows-x64-Portable.zip" .)
echo "✔ Created ${DOWNLOADS_DIR}/SmartAttendance-Desktop-Windows-x64-Portable.zip"

cat > "${BUILD_TMP}/installer.nsi" << EOF
!include "MUI2.nsh"
Name "Smart Attendance Desktop (Rust v8.0)"
OutFile "${DOWNLOADS_DIR}/SmartAttendance-Desktop-Windows-x64-Setup.exe"
InstallDir "\$LOCALAPPDATA\SmartAttendanceDesktop"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "\$INSTDIR\attendance-desktop.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Smart Attendance Desktop now"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Section "Smart Attendance Desktop" SecMain
    SetOutPath "\$INSTDIR"
    File "${WIN_STAGE}/attendance-desktop.exe"
    File "${WIN_STAGE}/Launch-SmartAttendance.bat"
    File "${WIN_STAGE}/Install-Windows-Desktop.bat"
    File "${WIN_STAGE}/README-Windows.txt"

    CreateShortcut "\$DESKTOP\Smart Attendance Desktop.lnk" "\$INSTDIR\attendance-desktop.exe"
    CreateDirectory "\$SMPROGRAMS\Smart Attendance Desktop"
    CreateShortcut "\$SMPROGRAMS\Smart Attendance Desktop\Smart Attendance Desktop.lnk" "\$INSTDIR\attendance-desktop.exe"
    WriteUninstaller "\$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "\$DESKTOP\Smart Attendance Desktop.lnk"
    Delete "\$SMPROGRAMS\Smart Attendance Desktop\Smart Attendance Desktop.lnk"
    RMDir "\$SMPROGRAMS\Smart Attendance Desktop"
    Delete "\$INSTDIR\attendance-desktop.exe"
    Delete "\$INSTDIR\Launch-SmartAttendance.bat"
    Delete "\$INSTDIR\Install-Windows-Desktop.bat"
    Delete "\$INSTDIR\README-Windows.txt"
    Delete "\$INSTDIR\Uninstall.exe"
    RMDir "\$INSTDIR"
SectionEnd
EOF

makensis "${BUILD_TMP}/installer.nsi"
file "${DOWNLOADS_DIR}/SmartAttendance-Desktop-Windows-x64-Setup.exe"
echo "✔ Created ${DOWNLOADS_DIR}/SmartAttendance-Desktop-Windows-x64-Setup.exe"

echo "===================================================================="
echo "🍎 3. Building macOS Universal Desktop Bundle (.app + Installer)"
echo "===================================================================="
MAC_STAGE="${BUILD_TMP}/SmartAttendance-macOS"
APP_BUNDLE="${MAC_STAGE}/SmartAttendanceDesktop.app/Contents"
mkdir -p "${APP_BUNDLE}/MacOS" "${APP_BUNDLE}/Resources/rust-src"

cp -r "${WORKSPACE_DIR}/crates/attendance-desktop/"* "${APP_BUNDLE}/Resources/rust-src/"

cat > "${APP_BUNDLE}/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>SmartAttendanceDesktop</string>
    <key>CFBundleIdentifier</key>
    <string>com.smartattendance.desktop</string>
    <key>CFBundleName</key>
    <string>Smart Attendance Desktop</string>
    <key>CFBundleDisplayName</key>
    <string>Smart Attendance Desktop</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>8.0.0</string>
    <key>CFBundleVersion</key>
    <string>8.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

printf "APPL????" > "${APP_BUNDLE}/PkgInfo"

cat > "${APP_BUNDLE}/MacOS/SmartAttendanceDesktop" << 'EOF'
#!/usr/bin/env bash
DIR="$(cd "$(dirname "$0")" && pwd)"
RES_DIR="$(cd "$DIR/../Resources" && pwd)"
NATIVE_BIN="$HOME/.smart_attendance_desktop/attendance-desktop-macos"
CLOUD_URL="https://confident-officials-relatively-julie.trycloudflare.com"

mkdir -p "$HOME/.smart_attendance_desktop"

# If Rust cargo is installed on this Mac, compile native binary once automatically
if [ ! -x "$NATIVE_BIN" ] && command -v cargo >/dev/null 2>&1; then
    (cd "$RES_DIR/rust-src" && cargo build --release && cp target/release/attendance-desktop "$NATIVE_BIN" && chmod +x "$NATIVE_BIN") >/dev/null 2>&1 || true
fi

if [ -x "$NATIVE_BIN" ]; then
    exec "$NATIVE_BIN"
fi

# Universal macOS Desktop Launcher (Apple Silicon & Intel Ready without Xcode)
if [ -d "/Applications/Google Chrome.app" ]; then
    open -na "Google Chrome" --args --app="$CLOUD_URL" --window-size=1320,860
elif [ -d "/Applications/Microsoft Edge.app" ]; then
    open -na "Microsoft Edge" --args --app="$CLOUD_URL" --window-size=1320,860
else
    open "$CLOUD_URL"
fi
EOF
chmod +x "${APP_BUNDLE}/MacOS/SmartAttendanceDesktop"

cat > "${MAC_STAGE}/Install-macOS.command" << 'EOF'
#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"
echo "======================================================================"
echo "🍎 Installing Smart Attendance Desktop to /Applications..."
echo "======================================================================"
xattr -cr "$DIR/SmartAttendanceDesktop.app" 2>/dev/null || true
if [ -w "/Applications" ]; then
    cp -R "$DIR/SmartAttendanceDesktop.app" "/Applications/"
    xattr -cr "/Applications/SmartAttendanceDesktop.app" 2>/dev/null || true
    echo "✅ Installed to /Applications/SmartAttendanceDesktop.app"
    open "/Applications/SmartAttendanceDesktop.app"
else
    mkdir -p "$HOME/Applications"
    cp -R "$DIR/SmartAttendanceDesktop.app" "$HOME/Applications/"
    echo "✅ Installed to $HOME/Applications/SmartAttendanceDesktop.app"
    open "$HOME/Applications/SmartAttendanceDesktop.app"
fi
EOF
chmod +x "${MAC_STAGE}/Install-macOS.command"

(cd "${MAC_STAGE}" && zip -q -r "${DOWNLOADS_DIR}/SmartAttendance-Desktop-macOS-Universal.zip" .)
echo "✔ Created ${DOWNLOADS_DIR}/SmartAttendance-Desktop-macOS-Universal.zip"

echo "===================================================================="
echo "🤖 4. Building Signed Android Mobile APK (Mandatory Location + Wi-Fi)"
echo "===================================================================="
ANDROID_SDK_DIR="/tmp/android_sdk_minimal"
mkdir -p "${ANDROID_SDK_DIR}"

if [ ! -f "${ANDROID_SDK_DIR}/android.jar" ]; then
    curl -fsSL "https://dl.google.com/android/repository/build-tools_r34-linux.zip" -o "${ANDROID_SDK_DIR}/bt.zip"
    curl -fsSL "https://dl.google.com/android/repository/platform-34-ext7_r02.zip" -o "${ANDROID_SDK_DIR}/pf.zip"
    unzip -q -o "${ANDROID_SDK_DIR}/bt.zip" -d "${ANDROID_SDK_DIR}"
    unzip -q -o "${ANDROID_SDK_DIR}/pf.zip" -d "${ANDROID_SDK_DIR}"
    BT_SUBDIR="$(find "${ANDROID_SDK_DIR}" -maxdepth 1 -type d -name "android-*" | head -n 1)"
    PF_JAR="$(find "${ANDROID_SDK_DIR}" -name "android.jar" | head -n 1)"
    cp "${PF_JAR}" "${ANDROID_SDK_DIR}/android.jar"
    ln -sf "${BT_SUBDIR}/aapt" "${ANDROID_SDK_DIR}/aapt"
    ln -sf "${BT_SUBDIR}/d8" "${ANDROID_SDK_DIR}/d8"
    ln -sf "${BT_SUBDIR}/zipalign" "${ANDROID_SDK_DIR}/zipalign"
    ln -sf "${BT_SUBDIR}/apksigner" "${ANDROID_SDK_DIR}/apksigner"
fi

APK_WORK="${BUILD_TMP}/android_apk_src"
mkdir -p "${APK_WORK}/src/com/smartattendance/enterprise" \
         "${APK_WORK}/res/values" \
         "${APK_WORK}/assets" \
         "${APK_WORK}/obj" \
         "${APK_WORK}/bin"

cat > "${APK_WORK}/AndroidManifest.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.smartattendance.enterprise"
    android:versionCode="80"
    android:versionName="8.0.0-PureRust">

    <uses-sdk android:minSdkVersion="23" android:targetSdkVersion="33" />

    <!-- MANDATORY GPS LOCATION & WI-FI PERMISSIONS -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
    <uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
    <uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.CAMERA" />

    <application
        android:label="Smart Attendance Mobile"
        android:usesCleartextTraffic="true"
        android:hardwareAccelerated="true"
        android:theme="@android:style/Theme.DeviceDefault.NoActionBar">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:configChanges="orientation|screenSize|keyboardHidden">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
EOF

cat > "${APK_WORK}/res/values/strings.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">Smart Attendance Mobile</string>
</resources>
EOF

cat > "${APK_WORK}/src/com/smartattendance/enterprise/MainActivity.java" << 'EOF'
package com.smartattendance.enterprise;

import android.app.Activity;
import android.content.Context;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.net.wifi.WifiInfo;
import android.net.wifi.WifiManager;
import android.os.Build;
import android.os.Bundle;
import android.webkit.GeolocationPermissions;
import android.webkit.JavascriptInterface;
import android.webkit.PermissionRequest;
import android.webkit.WebChromeClient;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

public class MainActivity extends Activity {
    private WebView webView;
    private LocationManager locationManager;
    private volatile double lastLat = 0.0;
    private volatile double lastLon = 0.0;
    private volatile float lastAcc = 0.0f;
    private volatile boolean hasFix = false;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        webView = new WebView(this);
        setContentView(webView);

        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setGeolocationEnabled(true);
        settings.setMediaPlaybackRequiresUserGesture(false);
        settings.setAllowFileAccess(true);
        settings.setAllowContentAccess(true);

        webView.addJavascriptInterface(new AndroidBridge(), "AndroidNativeBridge");
        webView.setWebViewClient(new WebViewClient());
        webView.setWebChromeClient(new WebChromeClient() {
            @Override
            public void onGeolocationPermissionsShowPrompt(String origin, GeolocationPermissions.Callback callback) {
                callback.invoke(origin, true, false);
            }
            @Override
            public void onPermissionRequest(final PermissionRequest request) {
                request.grant(request.getResources());
            }
        });

        requestMandatoryPermissions();
        startHardwareGpsTracking();
        webView.loadUrl("file:///android_asset/index.html");
    }

    private void requestMandatoryPermissions() {
        if (Build.VERSION.SDK_INT >= 23) {
            String[] perms = new String[] {
                "android.permission.ACCESS_FINE_LOCATION",
                "android.permission.ACCESS_COARSE_LOCATION",
                "android.permission.ACCESS_WIFI_STATE",
                "android.permission.CAMERA"
            };
            requestPermissions(perms, 1001);
        }
    }

    private void startHardwareGpsTracking() {
        try {
            locationManager = (LocationManager) getSystemService(Context.LOCATION_SERVICE);
            if (locationManager == null) return;
            LocationListener listener = new LocationListener() {
                @Override
                public void onLocationChanged(Location loc) {
                    if (loc != null) {
                        lastLat = loc.getLatitude();
                        lastLon = loc.getLongitude();
                        lastAcc = loc.getAccuracy();
                        hasFix = true;
                    }
                }
                @Override public void onStatusChanged(String p, int s, Bundle e) {}
                @Override public void onProviderEnabled(String p) {}
                @Override public void onProviderDisabled(String p) {}
            };
            if (locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
                locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, 2000L, 1.0f, listener);
                Location last = locationManager.getLastKnownLocation(LocationManager.GPS_PROVIDER);
                if (last != null) {
                    lastLat = last.getLatitude();
                    lastLon = last.getLongitude();
                    lastAcc = last.getAccuracy();
                    hasFix = true;
                }
            }
            if (locationManager.isProviderEnabled(LocationManager.NETWORK_PROVIDER)) {
                locationManager.requestLocationUpdates(LocationManager.NETWORK_PROVIDER, 2000L, 1.0f, listener);
                if (!hasFix) {
                    Location lastNet = locationManager.getLastKnownLocation(LocationManager.NETWORK_PROVIDER);
                    if (lastNet != null) {
                        lastLat = lastNet.getLatitude();
                        lastLon = lastNet.getLongitude();
                        lastAcc = lastNet.getAccuracy();
                        hasFix = true;
                    }
                }
            }
        } catch (SecurityException ignored) {
        } catch (Exception ignored) {}
    }

    public class AndroidBridge {
        @JavascriptInterface
        public String getHardwareLocationJson() {
            startHardwareGpsTracking();
            boolean gpsEnabled = false;
            try {
                if (locationManager != null) {
                    gpsEnabled = locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)
                              || locationManager.isProviderEnabled(LocationManager.NETWORK_PROVIDER);
                }
            } catch (Exception ignored) {}
            return "{\"has_fix\":" + hasFix +
                   ",\"gps_enabled\":" + gpsEnabled +
                   ",\"latitude\":" + lastLat +
                   ",\"longitude\":" + lastLon +
                   ",\"accuracy\":" + lastAcc + "}";
        }

        @JavascriptInterface
        public String getWifiInfoJson() {
            try {
                WifiManager wm = (WifiManager) getApplicationContext().getSystemService(Context.WIFI_SERVICE);
                if (wm != null && wm.isWifiEnabled()) {
                    WifiInfo info = wm.getConnectionInfo();
                    if (info != null) {
                        String ssid = info.getSSID();
                        if (ssid != null) {
                            ssid = ssid.replace("\"", "");
                        } else {
                            ssid = "";
                        }
                        String bssid = info.getBSSID() != null ? info.getBSSID() : "";
                        return "{\"wifi_enabled\":true,\"ssid\":\"" + ssid + "\",\"bssid\":\"" + bssid + "\"}";
                    }
                }
            } catch (Exception ignored) {}
            return "{\"wifi_enabled\":false,\"ssid\":\"\",\"bssid\":\"\"}";
        }

        @JavascriptInterface
        public String getServerUrl() {
            SharedPreferences prefs = getSharedPreferences("smart_att_prefs", Context.MODE_PRIVATE);
            return prefs.getString("server_url", "https://confident-officials-relatively-julie.trycloudflare.com");
        }

        @JavascriptInterface
        public void setServerUrl(String url) {
            SharedPreferences prefs = getSharedPreferences("smart_att_prefs", Context.MODE_PRIVATE);
            prefs.edit().putString("server_url", url).apply();
        }
    }
}
EOF

cat > "${APK_WORK}/assets/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="bn">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>Smart Attendance Android App</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: system-ui, sans-serif; }
        body { background: #090d16; color: #f1f5f9; min-height: 100vh; display: flex; flex-direction: column; }
        .top-bar { background: #0f172a; border-bottom: 1px solid rgba(0,242,254,0.3); padding: 10px 14px; display: flex; justify-content: space-between; align-items: center; }
        .title { font-weight: 800; color: #00f2fe; font-size: 0.92rem; }
        .gate { flex: 1; padding: 20px; display: flex; flex-direction: column; justify-content: center; align-items: center; text-align: center; }
        .card { background: #131c31; border: 2px solid #ff5252; border-radius: 16px; padding: 20px; width: 100%; max-width: 420px; }
        .btn { width: 100%; padding: 12px; border-radius: 10px; border: none; font-weight: 800; font-size: 0.9rem; margin-top: 10px; cursor: pointer; }
        .btn-cyan { background: linear-gradient(135deg, #00f2fe, #4facfe); color: #041421; }
        .input { width: 100%; padding: 10px; border-radius: 8px; background: #090d16; border: 1px solid #334155; color: #fff; margin-top: 8px; }
    </style>
</head>
<body>
    <div class="top-bar">
        <span class="title">⚡ SMART ATTENDANCE ANDROID v8.0</span>
        <span id="gps-pill" style="font-size:0.75rem; color:#ff5252; font-weight:700;">📍 GPS Checking...</span>
    </div>
    <div class="gate" id="lock-screen">
        <div class="card" id="status-card">
            <div style="font-size: 2.8rem; margin-bottom: 8px;">📍</div>
            <h2 style="color: #ff5252; font-size: 1.15rem; margin-bottom: 8px;">লোকেশন ছাড়া এই মোবাইল অ্যাপ কাজ করবে না!</h2>
            <p style="font-size: 0.84rem; color: #cbd5e1; margin-bottom: 12px;" id="status-msg">
                আপনার ফোনের হার্ডওয়্যার GPS Location এবং নির্ধারিত Wi-Fi অপশন যাচাই করা হচ্ছে...
            </p>
            <div style="text-align:left; font-size:0.78rem; background:#090d16; padding:10px; border-radius:10px; margin-bottom:10px;">
                <div>📍 GPS Status: <b id="lbl-gps" style="color:#ffab00;">Checking...</b></div>
                <div>📶 Wi-Fi Status: <b id="lbl-wifi" style="color:#00f2fe;">Checking...</b></div>
                <div style="margin-top:6px;">🌐 Server URL:</div>
                <input type="text" id="srv-url" class="input" value="https://confident-officials-relatively-julie.trycloudflare.com">
            </div>
            <button class="btn btn-cyan" onclick="verifyAndLaunchMobilePortal()">
                📍 GPS লোকেশন যাচাই করে অ্যাপে প্রবেশ করুন
            </button>
        </div>
    </div>

    <script>
        function verifyAndLaunchMobilePortal() {
            const srvInput = document.getElementById('srv-url').value.trim().replace(/\/+$/, '');
            if (window.AndroidNativeBridge) {
                window.AndroidNativeBridge.setServerUrl(srvInput);
                const loc = JSON.parse(window.AndroidNativeBridge.getHardwareLocationJson());
                const wifi = JSON.parse(window.AndroidNativeBridge.getWifiInfoJson());
                document.getElementById('lbl-wifi').textContent = wifi.ssid || (wifi.wifi_enabled ? 'Connected' : 'Optional / Mobile Data');
                if (!loc.gps_enabled) {
                    document.getElementById('lbl-gps').textContent = 'GPS বন্ধ আছে (OFF)!';
                    document.getElementById('status-msg').textContent = 'লোকেশন ছাড়া অ্যাপ কাজ করবে না! অনুগ্রহ করে ফোনের Location / GPS চালু করুন।';
                    return;
                }
                if (loc.has_fix) {
                    document.getElementById('lbl-gps').textContent = `${loc.latitude.toFixed(5)}, ${loc.longitude.toFixed(5)}`;
                    document.getElementById('gps-pill').style.color = '#00e676';
                    document.getElementById('gps-pill').textContent = '📍 GPS Active';
                    window.location.href = srvInput + '/mobile';
                    return;
                }
            }
            if (navigator.geolocation) {
                navigator.geolocation.getCurrentPosition(
                    (pos) => {
                        document.getElementById('lbl-gps').textContent = `${pos.coords.latitude.toFixed(5)}, ${pos.coords.longitude.toFixed(5)}`;
                        window.location.href = srvInput + '/mobile';
                    },
                    (err) => {
                        document.getElementById('lbl-gps').textContent = 'GPS Permission Denied!';
                        document.getElementById('status-msg').textContent = 'লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না! আগে ফোনের GPS Location চালু করুন।';
                    },
                    { enableHighAccuracy: true, timeout: 8000 }
                );
            }
        }
        window.addEventListener('DOMContentLoaded', () => {
            if (window.AndroidNativeBridge) {
                document.getElementById('srv-url').value = window.AndroidNativeBridge.getServerUrl();
            }
            setTimeout(verifyAndLaunchMobilePortal, 600);
        });
    </script>
</body>
</html>
EOF

# 1. Generate R.java
"${ANDROID_SDK_DIR}/aapt" package -f -m \
    -J "${APK_WORK}/src" \
    -M "${APK_WORK}/AndroidManifest.xml" \
    -S "${APK_WORK}/res" \
    -I "${ANDROID_SDK_DIR}/android.jar"

# 2. Compile Java sources to JVM 8 bytecode
javac -source 1.8 -target 1.8 \
    -bootclasspath "${ANDROID_SDK_DIR}/android.jar" \
    -classpath "${ANDROID_SDK_DIR}/android.jar" \
    -d "${APK_WORK}/obj" \
    "${APK_WORK}/src/com/smartattendance/enterprise/R.java" \
    "${APK_WORK}/src/com/smartattendance/enterprise/MainActivity.java"

# 3. Convert .class files to classes.dex using d8
CLASS_FILES=$(find "${APK_WORK}/obj" -name "*.class")
"${ANDROID_SDK_DIR}/d8" --release \
    --lib "${ANDROID_SDK_DIR}/android.jar" \
    --output "${APK_WORK}/bin" \
    ${CLASS_FILES}

# 4. Package resources, AndroidManifest.xml, and assets into unaligned APK
"${ANDROID_SDK_DIR}/aapt" package -f \
    -M "${APK_WORK}/AndroidManifest.xml" \
    -S "${APK_WORK}/res" \
    -A "${APK_WORK}/assets" \
    -I "${ANDROID_SDK_DIR}/android.jar" \
    -F "${APK_WORK}/bin/unaligned.apk"

# Add classes.dex to root of APK
(cd "${APK_WORK}/bin" && "${ANDROID_SDK_DIR}/aapt" add unaligned.apk classes.dex >/dev/null)

# 5. Zipalign on 4-byte boundaries
"${ANDROID_SDK_DIR}/zipalign" -f 4 \
    "${APK_WORK}/bin/unaligned.apk" \
    "${APK_WORK}/bin/aligned.apk"

# 6. Generate release signing keystore & sign APK with V1 + V2 + V3 signatures
KEYSTORE="${APK_WORK}/release.keystore"
keytool -genkeypair -v \
    -keystore "${KEYSTORE}" \
    -alias smartattendance \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -storepass android123 \
    -keypass android123 \
    -dname "CN=Smart Attendance Enterprise, OU=Mobile Engineering, O=SmartAttendance, L=Dhaka, S=Dhaka, C=BD" 2>/dev/null

"${ANDROID_SDK_DIR}/apksigner" sign \
    --ks "${KEYSTORE}" \
    --ks-key-alias smartattendance \
    --ks-pass pass:android123 \
    --key-pass pass:android123 \
    --v1-signing-enabled true \
    --v2-signing-enabled true \
    --out "${DOWNLOADS_DIR}/SmartAttendance-Mobile-Android.apk" \
    "${APK_WORK}/bin/aligned.apk"

"${ANDROID_SDK_DIR}/apksigner" verify --verbose "${DOWNLOADS_DIR}/SmartAttendance-Mobile-Android.apk" | head -n 10
echo "✔ Created & Verified ${DOWNLOADS_DIR}/SmartAttendance-Mobile-Android.apk"

echo "===================================================================="
echo "🔒 5. Generating Checksums for All Multi-Platform Artifacts"
echo "===================================================================="
cd "${DOWNLOADS_DIR}"
sha256sum * > SHA256SUMS_PLATFORMS.txt
cat SHA256SUMS_PLATFORMS.txt
ls -lh "${DOWNLOADS_DIR}"
