# Inspiration Notes

These notes summarize the GitHub topic pages used as inspiration for the skill.

Sources:

- `dribbble-clone` GitHub topic: https://github.com/topics/dribbble-clone?o=desc&s=updated
- `dribbble` GitHub topic filtered to Kotlin: https://github.com/topics/dribbble?l=kotlin&utf8=%E2%9C%93
- `dribble` topic link provided by user: https://github.com/topics/dribble?l=typescript&o=asc%3Fu%3Dhttp%3A%2F%2Fgithub.com%2Fsponsors%2Fmaful

## Observed Patterns From `dribbble-clone`

### `Developer-Zahid/Whispr-Dribbble-Clone`

- HTML-based clone work tagged with `gsap`, `gsap-scrolltrigger`, and timeline animation
- Useful cue: motion can be a first-class layout device, not just micro-interaction polish

### `Developer-Zahid/Micro-Space-Dribbble-Clone`

- Uses `gsap` and `gsap-flip`
- Useful cue: spatial transitions and FLIP-style movement reinforce depth and continuity

### `Aasuyadav1/Dribbble-Fullstack-Clone`

- Built with Next.js, Server Actions, MongoDB, and Tailwind CSS
- Useful cue: high-polish inspiration can still live inside a real product stack with auth, uploads, comments, and bookmarks

### `salimi-my/dribbble-clone`

- Next.js + Prisma + Clerk + shadcn/ui
- Useful cue: creative gallery layouts can coexist with structured product primitives and account systems

## Observed Patterns From Kotlin `dribbble`

### `ZacSweers/CatchUp`

- Large multi-source reader that includes Dribbble among content sources
- Useful cue: feed design, editorial density, and polished browsing surfaces matter as much as the hero

### `muramrr/Complex-Android-MotionLayout-Animation`

- MotionLayout animation inspired by a Dribbble shot
- Useful cue: choreographed transitions create perceived depth even without true 3D rendering

### `muramrr/LoadingView`

- Custom loading view based on a Dribbble shot
- Useful cue: even small states like loading can carry the visual identity of the product

### `jsericksk/Cleidesigns`

- XML and Jetpack Compose implementations inspired by Dribbble designs
- Useful cue: expressive visuals should still translate into real UI code, not just screenshots

## How To Translate These Repos Into Practical Design Rules

- Use motion to reveal structure
- Treat cards and panels like objects with mass
- Make hero sections feel staged, not simply centered
- Build interfaces with strong focal depth
- Keep the design ambitious, but make it shippable

## Motion Translation Notes

- Build one dominant reveal system instead of many unrelated effects
- Use stagger and shared layout continuity to make cards feel connected
- Let hover and tap states reinforce materiality with lift, compression, and sheen
- Treat scroll motion as scene direction, not constant decoration
- Preserve a readable fallback when motion is reduced or disabled

## What Not To Copy Blindly

- shallow clones of Dribbble layouts
- animation for its own sake
- crowded glow-heavy surfaces with weak text contrast
- desktop-only compositions that collapse on mobile
