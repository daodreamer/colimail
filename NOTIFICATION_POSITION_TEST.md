# Notification Window Position Testing Guide

## What Was Fixed

### Problem
- On Windows: Notification window appeared correctly in bottom-right corner
- On macOS: Notification window appeared outside the visible screen area
- Root cause: Used hardcoded margins (130px right, 110px bottom) that worked on Windows but not on macOS with different screen resolutions and DPI scaling

### Solution
- Reduced margins to 20px on both sides for better cross-platform compatibility
- Used Tauri's monitor API to get the actual work area (excludes taskbar/dock)
- Added detailed logging to help diagnose position issues

## Testing Instructions

### Before Testing
1. Make sure you have the latest code (v0.1.4)
2. Rebuild the application:
   ```bash
   npm run tauri dev
   ```

### Test Scenarios

#### 1. Test on macOS (M4 chip)
1. Open Colimail
2. Add an email account if you haven't already
3. Wait for a new email to arrive, OR manually trigger a notification
4. **Expected Result**: 
   - Notification window appears in the bottom-right corner of your screen
   - Window is fully visible (not cut off)
   - Window is positioned 20px from the right edge
   - Window is positioned 20px from the bottom edge (above the Dock)

#### 2. Test on Different Screen Resolutions
1. Change your display resolution (System Settings > Displays)
2. Trigger a notification
3. **Expected Result**: 
   - Notification still appears correctly in bottom-right corner
   - Margins are consistent across resolutions

#### 3. Test with Multiple Monitors
1. If you have multiple monitors, try moving Colimail to different monitors
2. Trigger a notification
3. **Expected Result**: 
   - Notification appears on the same monitor as Colimail
   - Position is correct relative to that monitor's work area

#### 4. Test with Dock in Different Positions (macOS)
1. Move your Dock to different positions (bottom, left, right)
2. Trigger a notification
3. **Expected Result**: 
   - Notification always appears in a visible position
   - Never overlaps with the Dock

### Checking the Console Logs

When a notification is triggered, you should see logs like:
```
📍 Monitor work area - size: PhysicalSize { width: 2560, height: 1440 }, position: PhysicalPosition { x: 0, y: 0 }
📍 Notification window position: (2160, 1300)
📍 Margins - right: 20px, bottom: 20px
✅ Notification window created and shown
```

These logs help verify:
- The monitor's work area dimensions
- The calculated position for the notification window
- The margins being applied

### Manual Testing (If Needed)

If you want to manually trigger a notification for testing:

1. Open the browser console in the dev tools
2. Use Tauri's event system to simulate a new email (if implemented)
3. Or simply wait for a real email to arrive

## Known Limitations

- Notification window dimensions are fixed at 380x120 pixels
- Auto-closes after 5 seconds
- Only shows one notification at a time (queued if multiple arrive)

## Troubleshooting

### Notification still appears outside screen
1. Check the console logs for the calculated position
2. Verify the monitor work area dimensions
3. If using HiDPI/Retina display, ensure Tauri is handling scaling correctly

### Notification appears but in wrong position
1. Check if you have custom display scaling settings
2. Verify the margin values in the logs
3. Report the issue with your screen resolution and OS version

## Version History

- **v0.1.4 (2025-10-25)**: Fixed notification positioning on macOS with adaptive margins
- **v0.1.3**: Original implementation with hardcoded margins
