// Creates a HiDPI virtual display and makes it the main one, so windows open
// on it at 2x. GitHub's macOS runners only offer 1x modes on their own screen.
// Uses CoreGraphics' private CGVirtualDisplay classes, declared below. The
// display lives as long as this process does.
//
//   clang -fobjc-arc -framework Foundation -framework CoreGraphics \
//     virtual-display.m -o virtual-display && ./virtual-display &

#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>

@interface CGVirtualDisplayDescriptor : NSObject
@property(retain) dispatch_queue_t queue;
@property(retain) NSString *name;
@property unsigned int maxPixelsWide;
@property unsigned int maxPixelsHigh;
@property CGSize sizeInMillimeters;
@property unsigned int serialNum;
@property unsigned int productID;
@property unsigned int vendorID;
@property(copy) void (^terminationHandler)(id, id);
@end

@interface CGVirtualDisplayMode : NSObject
- (instancetype)initWithWidth:(NSUInteger)width height:(NSUInteger)height refreshRate:(double)rate;
@end

@interface CGVirtualDisplaySettings : NSObject
@property(retain) NSArray *modes;
@property unsigned int hiDPI;
@end

@interface CGVirtualDisplay : NSObject
- (instancetype)initWithDescriptor:(CGVirtualDisplayDescriptor *)descriptor;
- (BOOL)applySettings:(CGVirtualDisplaySettings *)settings;
@property(readonly) CGDirectDisplayID displayID;
@end

int main(void) {
    @autoreleasepool {
        const NSUInteger width = 1600, height = 1000;  // points

        CGVirtualDisplayDescriptor *descriptor = [CGVirtualDisplayDescriptor new];
        descriptor.queue = dispatch_get_main_queue();
        descriptor.name = @"Screenshot";
        descriptor.maxPixelsWide = (unsigned int)(width * 2);
        descriptor.maxPixelsHigh = (unsigned int)(height * 2);
        descriptor.sizeInMillimeters = CGSizeMake(345, 216);
        descriptor.serialNum = 1;
        descriptor.productID = 0x1234;
        descriptor.vendorID = 0x3456;

        CGVirtualDisplay *display = [[CGVirtualDisplay alloc] initWithDescriptor:descriptor];
        if (!display || display.displayID == 0) {
            fprintf(stderr, "could not create the virtual display\n");
            return 1;
        }

        CGVirtualDisplaySettings *settings = [CGVirtualDisplaySettings new];
        settings.hiDPI = 1;
        settings.modes = @[ [[CGVirtualDisplayMode alloc] initWithWidth:width height:height refreshRate:60] ];
        if (![display applySettings:settings]) {
            fprintf(stderr, "could not apply the HiDPI mode\n");
            return 1;
        }

        // The display at (0, 0) is the main one: put the virtual display there
        // and the runner's own screen to its right. Wait for the window server
        // to register the new display first.
        CGDirectDisplayID previous = CGMainDisplayID();
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
            CGDisplayConfigRef config;
            CGBeginDisplayConfiguration(&config);
            CGConfigureDisplayOrigin(config, display.displayID, 0, 0);
            CGConfigureDisplayOrigin(config, previous, (int32_t)width, 0);
            CGError error = CGCompleteDisplayConfiguration(config, kCGConfigureForSession);
            CGDisplayModeRef mode = CGDisplayCopyDisplayMode(display.displayID);
            printf("virtual %u, main %u, error %d, %zux%zu points, %zux%zu pixels\n", display.displayID,
                   CGMainDisplayID(), error, CGDisplayModeGetWidth(mode), CGDisplayModeGetHeight(mode),
                   CGDisplayModeGetPixelWidth(mode), CGDisplayModeGetPixelHeight(mode));
            CGDisplayModeRelease(mode);
            fflush(stdout);
        });
        [[NSRunLoop mainRunLoop] run];
    }
    return 0;
}
