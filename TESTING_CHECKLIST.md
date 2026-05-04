# GlanceGuard — Manual Testing Checklist

## Prerequisites
- ✅ Models downloaded: `src-tauri/models/scrfd_10g_bnkps.onnx`, `src-tauri/models/arcface_w600k_r50.onnx`
- ✅ ONNX Runtime dylib: `src-tauri/target/debug/libonnxruntime.dylib`
- ✅ Camera permission granted (macOS will prompt on first camera access)
- ✅ Notification permission granted

## 1. Camera Selection
- [ ] Launch app
- [ ] Navigate to Monitoring screen
- [ ] Verify camera dropdown lists available cameras
- [ ] Select a camera
- [ ] Confirm camera selection persists after restart

## 2. Owner Enrollment (Upload Photo)
- [ ] Navigate to Owner setup screen
- [ ] Click "Upload photo"
- [ ] Select a clear face photo (JPEG/PNG)
- [ ] Confirm "Owner enrolled successfully" message
- [ ] Verify "Owner enrolled" badge appears

## 3. Owner Enrollment (Live Capture)
- [ ] Navigate to Owner setup screen
- [ ] Click "Capture from camera"
- [ ] Position face in frame
- [ ] Confirm enrollment succeeds
- [ ] Verify owner embedding is stored

## 4. Basic Monitoring
- [ ] Navigate to Monitoring screen
- [ ] Click "Start monitoring"
- [ ] Verify status changes to "Monitoring"
- [ ] Enable "Debug overlay" in Settings
- [ ] Confirm face box appears around owner face with "owner" label

## 5. Observer Detection
- [ ] Have a second person stand behind you
- [ ] Verify second face box appears with "observer" label
- [ ] Confirm observer score value appears
- [ ] Wait 2 seconds while observer is visible
- [ ] Verify red overlay appears with "Someone may be looking at your screen"
- [ ] Verify system notification fires
- [ ] Verify status changes to "Alert"

## 6. Cooldown Behavior
- [ ] After alert triggers, verify status changes to "Cooldown"
- [ ] Try triggering another alert during cooldown
- [ ] Confirm no new alerts fire during cooldown period
- [ ] Wait for cooldown to expire (15/30/60s based on settings)
- [ ] Verify monitoring resumes after cooldown

## 7. Sensitivity Settings
- [ ] Set sensitivity to "Low"
- [ ] Verify observer needs to be closer/more frontal to trigger
- [ ] Set sensitivity to "High"
- [ ] Verify observer triggers more easily (distant/angled)
- [ ] Set sensitivity to "Medium"

## 8. Cooldown Settings
- [ ] Set cooldown to 15 seconds
- [ ] Trigger an alert
- [ ] Verify cooldown lasts 15 seconds
- [ ] Repeat with 30 and 60 seconds

## 9. Debug Overlay
- [ ] Enable debug overlay in Settings
- [ ] Verify face boxes draw correctly
- [ ] Verify labels ("owner" vs "observer") are correct
- [ ] Verify similarity scores appear
- [ ] Verify observer scores appear
- [ ] Disable debug overlay and confirm canvas clears

## 10. Clear Owner
- [ ] Navigate to Owner setup
- [ ] Click "Clear owner"
- [ ] Verify owner status changes to "Not enrolled"
- [ ] Attempt to start monitoring
- [ ] Confirm error: "Enroll an owner before starting monitoring"

## 11. Edge Cases
- [ ] Test with no faces in frame (should show "idle")
- [ ] Test with only owner face (should show "monitoring", no alert)
- [ ] Test with multiple observers (should track highest score)
- [ ] Test face at very edge of frame
- [ ] Test face very close to camera
- [ ] Test face far from camera

## 12. Persistence
- [ ] Enroll owner, close app, reopen
- [ ] Verify owner remains enrolled
- [ ] Change settings, close app, reopen
- [ ] Verify settings persisted

## 13. Stop Monitoring
- [ ] Start monitoring
- [ ] Click "Stop"
- [ ] Verify status returns to "idle"
- [ ] Verify debug overlay clears

## 14. Performance
- [ ] Monitor CPU usage (Activity Monitor on macOS)
- [ ] Verify FPS is stable (target ~15 FPS)
- [ ] Check for memory leaks during extended monitoring

## Known Limitations
- Camera format fallback: If 720p MJPEG isn't available, the app will try highest FPS, then any format
- ONNX models are large (16MB + 166MB) and may take a moment to load on first enrollment/monitoring
- Keyring availability: If OS keychain is unavailable, app will error (no silent fallback)
