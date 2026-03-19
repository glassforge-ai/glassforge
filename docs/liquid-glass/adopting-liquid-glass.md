# Adopting Liquid Glass

*Source: [Adopting Liquid Glass](https://developer.apple.com/documentation/TechnologyOverviews/adopting-liquid-glass) — Apple Developer Documentation. Brought into this repo for reference.*

Find out how to bring the new material to your app.

## Overview

If you have an existing app, adopting Liquid Glass doesn't mean reinventing your app from the ground up. Start by building your app in the latest version of Xcode to see the changes. As you review your app, use the following sections to understand the scope of changes and learn how you can adopt these best practices in your interface.

---

#### See your app with Liquid Glass

If your app uses standard components from SwiftUI, UIKit, or AppKit, your interface picks up the latest look and feel on the latest platform releases for iOS, iPadOS, macOS, tvOS, and watchOS. In Xcode, build your app with the latest SDKs, and run it on the latest platform releases to see the changes in your interface.

---

## Visual refresh

Interfaces across Apple platforms feature a new dynamic [material](https://developer.apple.com/design/human-interface-guidelines/materials) called Liquid Glass, which combines the optical properties of glass with a sense of fluidity. This material forms a distinct functional layer for controls and navigation elements. It affects how the interface looks, feels, and moves, adapting in response to a variety of factors to help bring focus to the underlying content.

Leverage system frameworks to adopt Liquid Glass automatically. In system frameworks, standard components like bars, sheets, popovers, and controls automatically adopt this material. System frameworks also dynamically adapt these components in response to factors like element overlap and focus state. Take advantage of this material with minimal code by using standard components from SwiftUI, UIKit, and AppKit.

Reduce your use of custom backgrounds in controls and navigation elements. Any custom backgrounds and appearances you use in these elements might overlay or interfere with Liquid Glass or other effects that the system provides, such as the scroll edge effect. Make sure to check any custom backgrounds in elements like split views, tab bars, and toolbars. Prefer to remove custom effects and let the system determine the background appearance.

Test your interface with a variety of display and accessibility settings. Translucency and fluid morphing animations contribute to the look and feel of Liquid Glass, but can adapt to people's needs. For example, people can choose a preferred look for Liquid Glass in their device's settings, or turn on accessibility settings that reduce transparency or motion in the interface. These settings can remove or modify certain effects. If you use standard components from system frameworks, this experience adapts automatically. Ensure you test your app's custom elements, colors, and animations with different configurations of these settings.

Avoid overusing Liquid Glass effects. If you apply Liquid Glass effects to a custom control, do so sparingly. Liquid Glass seeks to bring attention to the underlying content, and overusing this material in multiple custom controls can provide a subpar user experience by distracting from that content. Limit these effects to the most important functional elements in your app.

---

## App icons

[App icons](https://developer.apple.com/design/human-interface-guidelines/app-icons) take on a design that's dynamic and expressive. Updates to the icon grid result in a standardized iconography that's visually consistent across devices and concentric with hardware and other elements across the system. App icons now contain layers, which dynamically respond to lighting and other visual effects the system provides. iOS, iPadOS, and macOS all now offer default (light), dark, clear, and tinted appearance variants, empowering people to personalize the look and feel of their Home Screen.

Reimagine your app icon for Liquid Glass. Apply key design principles to help your app icon shine:

- Let the system handle applying masking, blurring, and other visual effects, rather than factoring them into your design.
- Consider a simplified design comprised of solid, filled, overlapping semi-transparent shapes.
- Provide a visually consistent, optically balanced design across the platforms your app supports.

Design using layers. The system automatically applies effects like reflection, refraction, shadow, blur, and highlights to your icon layers. Determine which elements of your design make sense as foreground, middle, and background elements, then define separate layers for them. You can perform this task in the design app of your choice.

Compose and preview in Icon Composer. Drag and drop app icon layers that you export from your design app directly into the Icon Composer app. Icon Composer lets you add a background, create layer groupings, adjust layer attributes like opacity, and preview your design with system effects and appearances. Icon Composer is available in the latest version of Xcode and for download from [Apple Design Resources](https://developer.apple.com/design/resources/).

Preview against the updated grids. The system applies masking to produce your final icon shape — rounded rectangle for iOS, iPadOS, and macOS, and circular for watchOS. Keep elements centered to avoid clipping. Irregularly shaped icons receive a system-provided background. See how your app icon looks with the updated grids to determine whether you need to make adjustments. Download these grids from [Apple Design Resources](https://developer.apple.com/design/resources/).

---

## Controls

Controls have a refreshed look across platforms, and come to life when a person interacts with them. For controls like sliders and toggles, the knob transforms into Liquid Glass during interaction, and [buttons](https://developer.apple.com/design/human-interface-guidelines/buttons) fluidly morph into menus and popovers. The shape of the hardware informs the curvature of controls, so many controls adopt rounder forms to elegantly nestle into the corners of windows and displays. Controls also feature an option for an extra-large size, allowing more space for labels and accents.

Review updates to control appearance and dimensions. If you use standard controls from system frameworks and don't hard-code their layout metrics, your app adopts changes to shapes and sizes automatically when you rebuild your app with the latest version of Xcode.

Review your use of color in controls. Be judicious with your use of [color](https://developer.apple.com/design/human-interface-guidelines/color) in controls and navigation so they stay legible. If you do apply color to these elements, leverage system colors, or define a custom color with light and dark variants, and an increased contrast option for each variant.

Check for crowding or overlapping of controls. Prefer to use standard spacing metrics instead of overriding them, and avoid overcrowding or layering Liquid Glass elements on top of each other.

Optimize for legibility when content scrolls beneath controls. Scroll views offer a [scroll edge effect](https://developer.apple.com/documentation/SwiftUI/View/scrollEdgeEffectStyle(_:for:)) that helps maintain sufficient legibility and contrast for controls by obscuring content that scrolls beneath them. System bars like toolbars adopt this behavior by default. If you use a custom bar with elements like controls, text, or icons that have content scrolling beneath them, register those views to use a scroll edge effect via the platform APIs.

Consider aligning the shape of controls with other rounded elements throughout the interface. Across Apple platforms, the shape of the hardware informs the curvature, size, and shape of nested interface elements, including controls, sheets, popovers, windows, and more.

Leverage new button styles. Instead of creating buttons with custom Liquid Glass effects, adopt the look and feel of the material with minimal code by using the standard button style APIs (e.g. SwiftUI `.buttonStyle(.glass)`).

---

## Navigation

Liquid Glass applies to the topmost layer of the interface, where you define your navigation. Key navigation elements like [tab bars](https://developer.apple.com/design/human-interface-guidelines/tab-bars) and [sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars) float in this Liquid Glass layer to help people focus on the underlying content.

Establish a clear navigation hierarchy. Ensure you clearly separate your content from navigation elements, like tab bars and sidebars, to establish a distinct functional layer above the content layer.

Consider adapting your tab bar into a sidebar automatically using the standard APIs. Consider using split views to build sidebar layouts with an inspector panel. [Split views](https://developer.apple.com/design/human-interface-guidelines/split-views) are optimized for sidebar and inspector layouts across platforms.

Check content safe areas for sidebars and inspectors. Audit the safe area compatibility of content next to the sidebar and inspector so underlying content peeks through appropriately.

Extend content beneath sidebars and inspectors. A background extension effect creates a sense of extending a background under a sidebar or inspector, without actually scrolling or placing content under it. Use the standard APIs for this effect.

Choose whether to automatically minimize your tab bar in iOS. Tab bars can recede when a person scrolls; opt in via APIs, e.g.:

```swift
TabView {
    // ...
}
.tabBarMinimizeBehavior(.onScrollDown)
```

---

## Menus and toolbars

[Menus](https://developer.apple.com/design/human-interface-guidelines/menus) have a refreshed look across platforms. They adopt Liquid Glass, and menu items for common actions use icons to help people quickly scan and identify those actions. New to iPadOS, apps also have a [menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar) for faster access to common commands.

Adopt standard icons in menu items. For standard actions like Cut, Copy, and Paste, use standard selectors so the system can apply the correct icon.

Match top menu actions to swipe actions. Surface the same actions at the top of contextual menus as you provide for swipe actions.

[Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars) take on a Liquid Glass appearance and provide a grouping mechanism for toolbar items. Group items that perform similar actions or affect the same part of the interface. Use fixed spacers via standard APIs to separate groups. Use [standard icons](https://developer.apple.com/design/human-interface-guidelines/icons) for common actions. Provide an accessibility label for every icon. Audit toolbar customizations and ensure you hide the entire toolbar item (not just the view inside it) when hiding items.

---

## Windows and modals

[Windows](https://developer.apple.com/design/human-interface-guidelines/windows) adopt rounder corners. In iPadOS, apps show window controls and support continuous window resizing. Support arbitrary window sizes and use split views for fluid resizing. Use layout guides and safe areas so the system can adjust window controls and title bar in relation to your content.

Modal views like [sheets](https://developer.apple.com/design/human-interface-guidelines/sheets) and [action sheets](https://developer.apple.com/design/human-interface-guidelines/action-sheets) adopt Liquid Glass. Sheets feature an increased corner radius; half sheets are inset from the edge. Check content around the edges of sheets and audit popover/sheet backgrounds; remove custom visual effect views for consistency. Action sheets originate from the initiating element; set the source view or item for the action sheet so it appears inline.

---

## Organization and layout

[List-based layouts](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables) have larger row height and padding; sections have an increased corner radius. Use title-style capitalization for [section headers](https://developer.apple.com/documentation/SwiftUI/Section/init(content:header:)). Use SwiftUI forms with the [grouped form style](https://developer.apple.com/documentation/SwiftUI/FormStyle/grouped) to get updated layout metrics.

---

## Search

Review [search](https://developer.apple.com/design/human-interface-guidelines/search-fields) design conventions. Check keyboard layout when activating search. Use semantic search tabs (e.g. SwiftUI `Tab(role: .search)` / UIKit `UISearchTab`) so the search tab is placed at the trailing end.

---

## Platform considerations

Test your app across devices. In watchOS, adopt standard button styles and toolbar APIs. In tvOS, adopt standard focus APIs so custom controls get Liquid Glass when focused; Apple TV 4K (2nd generation) and newer support Liquid Glass. Combine custom Liquid Glass effects using a single container (e.g. `GlassEffectContainer`) to improve performance. Performance test your app across platforms.

---

To update and ship your app with the latest SDKs while keeping the previous look when built against older SDKs, you can add the appropriate key to your project's Info pane.

---

*Copyright © 2026 Apple Inc. All rights reserved. | [Terms of Use](https://www.apple.com/legal/internet-services/terms/site.html) | [Privacy Policy](https://www.apple.com/privacy/privacy-policy)*
