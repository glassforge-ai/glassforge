# Applying Liquid Glass to custom views

*Source: [Applying Liquid Glass to custom views](https://developer.apple.com/documentation/SwiftUI/Applying-Liquid-Glass-to-custom-views) — Apple Developer Documentation. Brought into this repo for reference.*

Configure, combine, and morph views using Liquid Glass effects.

## Overview

Interfaces across Apple platforms feature a new dynamic material called Liquid Glass, which combines the optical properties of glass with a sense of fluidity. Liquid Glass is a material that blurs content behind it, reflects color and light of surrounding content, and reacts to touch and pointer interactions in real time. Standard components in SwiftUI use Liquid Glass. Adopt Liquid Glass on custom components to move, combine, and morph them into one another with unique animations and transitions.

To learn about Liquid Glass and more, see [Landmarks: Building an app with Liquid Glass](https://developer.apple.com/documentation/SwiftUI/Landmarks-Building-an-app-with-Liquid-Glass).

## Apply and configure Liquid Glass effects

Use the [`glassEffect(_:in:)`](https://developer.apple.com/documentation/SwiftUI/View/glassEffect(_:in:)) modifier to add Liquid Glass effects to a view. By default, the modifier uses the [`regular`](https://developer.apple.com/documentation/SwiftUI/Glass/regular) variant of [`Glass`](https://developer.apple.com/documentation/SwiftUI/Glass) and applies the given effect within a [`Capsule`](https://developer.apple.com/documentation/SwiftUI/Capsule) shape behind the view's content.

Configure the effect to customize your components in a variety of ways:

- Add `Glass.interactive(_:)` to custom components to make them react to touch and pointer interactions. This applies the same responsive and fluid reactions that `PrimitiveButtonStyle.glass` provides to standard buttons.
- Assign a tint color to suggest prominence.
- Use different shapes to have a consistent look and feel across custom components in your app (e.g. rounded rectangle for larger components).

Example: apply Liquid Glass, use an alternate shape, and create a tinted interactive view:

```swift
Text("Hello, World!")
    .font(.title)
    .padding()
    .glassEffect()

Text("Hello, World!")
    .font(.title)
    .padding()
    .glassEffect(in: .rect(cornerRadius: 16.0))

Text("Hello, World!")
    .font(.title)
    .padding()
    .glassEffect(.regular.tint(.orange).interactive())
```

## Combine multiple views with Liquid Glass containers

Use [`GlassEffectContainer`](https://developer.apple.com/documentation/SwiftUI/GlassEffectContainer) when applying Liquid Glass effects on multiple views to achieve the best rendering performance. A container also allows views with Liquid Glass effects to blend their shapes together and to morph in and out of each other during transitions. Inside a container, each view with the `glassEffect(_:in:)` modifier renders with the effects behind it.

Customize the spacing on the container to control how the Liquid Glass effects behind views interact with one another. The larger the spacing value on the container, the sooner the Liquid Glass effects behind views blend together and merge the shapes during a transition. A spacing value on the container that's larger than the spacing of an interior `HStack`, `VStack`, or other layout container causes Liquid Glass effects to blend together at rest because the views are too close to each other. Animating views in or out causes the shapes to morph apart or together as the space in the container changes.

Apply the `glassEffect(_:in:)` modifier **after** other modifiers that affect the appearance of the view.

Example: two images in a container; effects blend as they move:

```swift
GlassEffectContainer(spacing: 40.0) {
    HStack(spacing: 40.0) {
        Image(systemName: "scribble.variable")
            .frame(width: 80.0, height: 80.0)
            .font(.system(size: 36))
            .glassEffect()

        Image(systemName: "eraser.fill")
            .frame(width: 80.0, height: 80.0)
            .font(.system(size: 36))
            .glassEffect()
            .offset(x: -40.0, y: 0.0)
    }
}
```

### Unified effect with `glassEffectUnion(id:namespace:)`

When you want the geometries of multiple views to contribute to a **single** Liquid Glass effect capsule even at rest, use the [`glassEffectUnion(id:namespace:)`](https://developer.apple.com/documentation/SwiftUI/View/glassEffectUnion(id:namespace:)) modifier. This combines all effects with the same ID and namespace into one shape. Useful for views created dynamically or that live outside a single layout container.

```swift
let symbolSet: [String] = ["cloud.bolt.rain.fill", "sun.rain.fill", "moon.stars.fill", "moon.fill"]

GlassEffectContainer(spacing: 20.0) {
    HStack(spacing: 20.0) {
        ForEach(symbolSet.indices, id: \.self) { item in
            Image(systemName: symbolSet[item])
                .frame(width: 80.0, height: 80.0)
                .font(.system(size: 36))
                .glassEffect()
                .glassEffectUnion(id: item < 2 ? "1" : "2", namespace: namespace)
        }
    }
}
```

## Morph Liquid Glass effects during transitions

Morphing occurs during transitions or animations between views with Liquid Glass effects. Use the [`glassEffectID(_:in:)`](https://developer.apple.com/documentation/SwiftUI/View/glassEffectID(_:in:)) modifier to coordinate transitions within a container. [`GlassEffectTransition`](https://developer.apple.com/documentation/SwiftUI/GlassEffectTransition) lets you specify the transition type when adding or removing effects. For effects within the container's spacing, the default is `matchedGeometry`. For effects farther apart, use the `materialize` transition with `withAnimation(_:_:)`. Associate each effect with a unique ID in a `Namespace` so SwiftUI animates shapes correctly when the view hierarchy changes.

Example: eraser morphs into pencil when `isExpanded` toggles:

```swift
@State private var isExpanded: Bool = false
@Namespace private var namespace

var body: some View {
    GlassEffectContainer(spacing: 40.0) {
        HStack(spacing: 40.0) {
            Image(systemName: "scribble.variable")
                .frame(width: 80.0, height: 80.0)
                .font(.system(size: 36))
                .glassEffect()
                .glassEffectID("pencil", in: namespace)

            if isExpanded {
                Image(systemName: "eraser.fill")
                    .frame(width: 80.0, height: 80.0)
                    .font(.system(size: 36))
                    .glassEffect()
                    .glassEffectID("eraser", in: namespace)
            }
        }
    }

    Button("Toggle") {
        withAnimation {
            isExpanded.toggle()
        }
    }
    .buttonStyle(.glass)
}
```

## Optimize performance when using Liquid Glass effects

Creating too many Liquid Glass effect containers and applying too many effects to views outside of containers can degrade performance. Limit the use of Liquid Glass effects onscreen at the same time. Use `GlassEffectContainer` to combine effects and morph shapes. See [Explore UI animation hitches and the render loop](https://developer.apple.com/videos/play/tech-talks/10855/) and [Optimize SwiftUI performance with Instruments](https://developer.apple.com/videos/play/wwdc2025/306/).

---

*Copyright © 2026 Apple Inc. All rights reserved. | [Terms of Use](https://www.apple.com/legal/internet-services/terms/site.html) | [Privacy Policy](https://www.apple.com/privacy/privacy-policy)*
