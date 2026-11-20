// Prints the window number of the first on-screen window owned by the given
// process ID, for `screencapture -l`. Usage: swift window-id.swift <pid>
import CoreGraphics
import Foundation

let pid = Int32(CommandLine.arguments[1])!
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
for w in windows where (w[kCGWindowOwnerPID as String] as? Int32) == pid && (w[kCGWindowLayer as String] as? Int) == 0 {
    print(w[kCGWindowNumber as String] as! Int)
    exit(0)
}
exit(1)
