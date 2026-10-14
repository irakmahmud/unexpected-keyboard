# 📋 Audit Report: Keyboard Build Location & Output Artifact Verification

- **Date & Time:** 2026-10-12T03:38:00Z
- **User Query:** "Ekhon build kothay hoise aar output kothay?" (Where did the build happen now, and where is the output?)
- **Workspace:** `/home/opc/keyboard` (Unexpected Flutter Keyboard)
- **Monorepo Repository:** https://github.com/IroScript/Oracle-VM-Projects
- **Build Mode Mandate:** ZERO LOCAL BUILD on Oracle VM

---

## 1. বিল্ড কোথায় হয়েছে? (Where Did the Build Happen?)
- **লোকাল ওরাকল ভিএমে কোনো বিল্ড হয়নি:** ওরাকল ভিএমে লোকাল ফ্লাটার/গ্রেডল কম্পাইলেশন কঠোরভাবে নিষিদ্ধ।
- **রিমোট ক্লাউড ইনফ্রাস্ট্রাকচারে বিল্ড সম্পন্ন হয়েছে:**
  - **ক্লাউড প্ল্যাটফর্ম:** CircleCI Cloud Infrastructure (Docker Executor: `ghcr.io/cirruslabs/flutter:stable`)
  - **পাইপলাইন ও ওয়ার্কফ্লো:** `remote-flutter-compilation`
  - **এক্সিকিউশন জব:** `flutter-build-apks`
  - **ক্লাউড টুলচেন:** Java 17, Android SDK 34/36, Gradle 8.7, AGP 8.6.0, Flutter Stable 3.24.x

---

## 2. আউটপুট কোথায় আছে? (Where Is the Output Artifact?)
- **১. লোকাল ওরাকল ভিএম পাথ:**
  - সম্পূর্ণ পাথ: `/home/opc/keyboard/dist/apks/unexpected-keyboard-release.apk`
  - সাইজ: ৪৯ মেগাবাইট (Zip archive / Android APK)
  - SHA-256 চেকসাম: `7a5b44b01d637975de9b2b4030e39721c4acd358b707cdb4838cd55d369efb7f`
- **২. সার্কেল সিআই ক্লাউড আর্টিফ্যাক্ট স্টোরেজ:**
  - রিমোট আর্টিফ্যাক্ট ডিরেক্টরি: `keyboard-apks/`
  - রিলিজ APK: `unexpected-keyboard-release.apk` (৪৯ MB)
  - ডিবাগ APK: `unexpected-keyboard-debug.apk` (১৪১ MB)
  - ইন্টিগ্রিটি চেকসাম ফাইল: `SHA256SUMS.txt`
