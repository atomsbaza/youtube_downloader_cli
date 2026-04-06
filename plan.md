# Project Plan — youtube_downloader_cli

Current branch: `feature_add-gui`

---

## สรุปสถานะโปรเจค

โปรเจคนี้เป็น YouTube downloader ที่เขียนด้วย Rust ครอบ `yt-dlp` ไว้ข้างใน ใช้ Clean Architecture (Domain → Application → Infrastructure) มี 2 interface คือ CLI (`ytcli`) และ GUI (`gui` ด้วย egui) branch ปัจจุบันกำลัง add GUI อยู่

---

## สิ่งที่ต้องทำ (เรียงตามความสำคัญ)

### 1. แก้ Bug / ปัญหาเร่งด่วน

- [x] **Version mismatch ใน dependencies** — ตรวจแล้ว ทั้ง `eframe` และ `egui` ตรงกันที่ `0.27`
- [x] **Stderr จาก yt-dlp ไม่ถูก capture** — แก้แล้ว อ่าน stderr บน background thread และนำไปรวมกับ error message
- [x] **ลบ video files ออกจาก git** — `git rm --cached` ไฟล์ `.mp4` 2 ไฟล์ และเพิ่ม `*.mp4`, `*.webm` ฯลฯ ใน `.gitignore`

### 2. Error Handling

- [x] **แทน `.unwrap()` ใน Mutex locks** — เปลี่ยนเป็น `.expect("lock poisoned")` ทุกจุดใน `gui.rs`
- [x] **แทน `.unwrap()` ใน `yt_dlp.rs`** — `child.stdout.take()` ใช้ `.ok_or_else()` return error แทน panic
- [x] **แสดง stderr ของ yt-dlp ใน GUI** — error message จาก yt-dlp แสดงใน status area แล้ว
- [x] **Status thread result bug** — ก่อนหน้านี้ผลจาก download thread ไม่เคย propagate กลับมา UI เลย แก้ด้วย `Arc<Mutex<Option<Result<()>>>>` ที่ poll ใน `update()`
- [x] **Quality ถูก ignore** — `download_with_progress` และ `download` ไม่เคยใช้ `video.quality` แก้ด้วย `build_format_string()` ที่รวม quality เข้าไปใน format selector

### 3. Test Coverage

- [x] **เพิ่ม unit test สำหรับ `parse_yt_dlp_progress`** — 6 tests ครอบ normal, 100%, leading spaces, no percent, empty, other line
- [x] **เพิ่ม unit test สำหรับ quality/format selection** — 7 tests ครอบ mp4/webm × best/worst/720p/480p + invalid type
- [x] **เพิ่ม unit test สำหรับ playlist parameter** — test `playlist_items` ผ่าน Video struct
- [ ] **เพิ่ม integration test** สำหรับ yt-dlp wrapper (ใช้ mock หรือ test URL สั้นๆ)
- [x] **Thread safety audit** — ตรวจแล้ว: ไม่มี deadlock เพราะแต่ละ Mutex lock แบบ brief (ไม่ lock ซ้อนกัน); stderr thread ใช้ pipe แยก; cancel ใช้ `AtomicBool` (lock-free)

### 4. GUI Improvements

- [x] **Playlist controls ใน GUI** — เพิ่ม "Playlist Items" text field (`1,3,5-7`) ส่งผ่าน `Video.playlist_items` โดยตรง
- [x] **Save directory + Browse button** — เพิ่ม field + ปุ่ม Browse ด้วย `rfd`, ส่งผ่าน `Video.save_dir` และใช้ `-P` flag ของ yt-dlp
- [x] **แสดง error detail จาก yt-dlp** — stderr ถูก capture และแสดงใน status area เมื่อ download fail
- [ ] **Persist save directory ข้าม session** — จำ directory ที่เลือกไว้ใน config file (ปัจจุบันจำแค่ within session)
- [x] **Cancel / Stop download** — `Arc<AtomicBool>` cancel token ส่งเข้า `download_with_progress`; ตรวจหลังทุก stdout line แล้ว `child.kill()`; ปุ่มสีแดงโชว์เฉพาะตอน downloading
- [x] **แสดง Speed / ETA ใน GUI** — `parse_yt_dlp_line()` parse percent + speed + eta บรรทัดเดียวกัน; แสดงใต้ progress bar แบบ real-time ผ่าน `Arc<Mutex<String>>`
- [x] **Audio-only mode** — มี checkbox "Audio Only" ใน GUI อยู่แล้ว (ส่งผ่าน `video.audio_only`)
- [x] **URL validation ก่อน download** — `validate_url()` ตรวจ empty, missing scheme, non-YouTube domain; แสดง error ใน status bar ก่อน spawn thread
- [ ] **Download queue / batch** — รองรับการ queue หลาย URL พร้อมกัน
- [ ] **Cross-platform styling** — ทดสอบและแก้ UI บน Windows/Linux (focus: rfd dialog behavior, font rendering)

### 5. Progress Parsing (Robustness)

- [ ] **ทำให้ `parse_yt_dlp_progress` แข็งแกร่งขึ้น** — ปัจจุบัน simple string matching อาจพังถ้า yt-dlp เปลี่ยน output format ควรใช้ regex หรือ parse อย่างระมัดระวังกว่านี้
- [ ] **Handle multi-line / unexpected yt-dlp output** — รวมถึง merge formats, post-processing progress
- [x] **ขยาย parsed fields** — `parse_yt_dlp_line()` return `DownloadProgress { percent, speed, eta }` แทน `Option<f32>` เดิม

### 6. Developer Experience

- [ ] **เพิ่ม doc comments** บน public types และ functions (`Video`, `Downloader`, `DownloadVideoUseCase`)
- [ ] **เพิ่ม CONTRIBUTING.md** — อธิบาย architecture และ dev setup สำหรับ contributor
- [ ] **Validate format combinations** — เช่น audio-only กับ video quality ไม่ควรถูก combine โดยไม่มี warning
- [x] **yt-dlp availability check ตอน startup** — `check_ytdlp_available()` ใน `main()`; ถ้าไม่พบแสดง native error dialog (via `rfd::MessageDialog`) และออก ไม่เปิด UI

### 7. CI/CD & Release

- [ ] **ตรวจสอบ GitHub Actions workflow** — ให้ build ทั้ง `ytcli` และ `gui` binary สำหรับ release
- [ ] **เพิ่ม CI step สำหรับ `cargo test`** — ถ้ายังไม่มี
- [ ] **Release distribution** — กำหนด strategy การ distribute binary เช่น GitHub Release assets พร้อม checksums (sha256) และ/หรือ Homebrew formula สำหรับ macOS

---

## สิ่งที่ทำได้ทีหลัง (Nice to have)

- Configuration file (`~/.config/ytcli/config.toml`) สำหรับ default settings
- Download history log
- Thumbnail preview ใน GUI ก่อน download
- ระบบ notification เมื่อ download เสร็จ (OS notification)
- รองรับ cookies file สำหรับ private videos

---

## ลำดับการทำงานที่แนะนำ

1. ~~แก้ version mismatch + ลบ video files จาก git~~ ✅
2. ~~Fix `.unwrap()` และ stderr handling~~ ✅
3. ~~เพิ่ม tests~~ ✅ (ยังขาด integration test)
4. ~~GUI improvements (playlist UI, save dir, error display)~~ ✅ (ยังขาด persist ข้าม session)
5. ~~แก้ UX ที่ขาดก่อน merge~~ ✅ (Cancel, startup check, URL validation, Speed/ETA)
6. Merge `feature_add-gui` → `main` เมื่อ: integration test ผ่าน, CI green
7. Release: build binary + checksums + distribution strategy
