---
name: ui-immersive-3d-dribbble
description: Design and implement premium, immersive, 3D-leaning interfaces with strong motion, layered depth, bold typography, and Dribbble-inspired composition while staying production-realistic and responsive.
---

# UI Immersive 3D Dribbble

Use this skill when the user wants a UI that feels:

- premium
- immersive
- 3D or depth-rich
- motion-led
- inspired by Dribbble shots or design showcases
- less template-like and more art-directed

This skill is especially useful for:

- landing pages
- dashboards
- portfolio-style product marketing
- app home screens
- hero sections
- bold redesign requests

Pair this skill with `$framer-motion-ui-animations` when the visual direction needs to be implemented as a real React motion system.

## Core Goal

Ship interfaces that feel intentional, tactile, and spatial without turning into unusable concept art.

The result should feel:

- cinematic, not noisy
- dimensional, not gimmicky
- expressive, not generic
- buildable, not fantasy-only

## Inspiration Direction

Base the direction on the repo inspirations captured in [references/inspiration-notes.md](references/inspiration-notes.md).

Translate those references into these recurring traits:

- layered card stacks with foreground and background separation
- rich lighting, glows, reflective surfaces, and depth cues
- motion that reveals hierarchy instead of adding random hover effects
- bold, editorial typography paired with compact utility text
- sculpted layouts with overlap, diagonals, and asymmetric balance
- product UIs that still feel shippable on desktop and mobile

## Motion Pairing

This skill chooses the art direction.

`$framer-motion-ui-animations` should handle:

- variants and stagger choreography
- layout and shared element transitions
- scroll-linked transforms
- gesture polish in React

If the task is mostly a visual concept or static mockup, this skill can stand alone.
If the task is production React UI with premium motion, pair the two skills.

## Workflow

### 1. Read the Room

Before designing:

- inspect the existing app or component
- determine whether this is a greenfield redesign or an evolution of an existing system
- preserve product structure unless the user clearly wants a full layout rethink

If the repo already has a design system, adapt this skill to the current system rather than fighting it.

### 2. Pick a Visual Thesis

Choose one clear art direction instead of blending everything together. Good modes include:

- glass + glow control room
- soft matte luxury
- cosmic product gallery
- metallic dark cockpit
- editorial gradient showroom
- ambient mobile-card stack

Name the direction in your own head and keep the rest of the decisions consistent with it.

### 3. Build Depth Deliberately

Use depth through layout and lighting first:

- layered backgrounds with gradients, radial light pools, and subtle noise
- overlapping panels
- cards on different z-levels
- perspective wrappers and gentle tilt
- shadows that separate planes instead of simply darkening everything

Prefer CSS transforms and composited motion before jumping to WebGL.

### 4. Typography Rules

- use expressive display typography
- avoid default system-looking stacks when a custom font choice is appropriate
- keep body copy readable and calmer than the hero
- create scale contrast between headline, labels, and dense UI metadata

Do not default to Inter unless the product already does.

### 5. Motion Rules

Motion should explain the interface:

- stagger reveals on first load
- layered parallax or depth-shift on pointer move
- hover states with slight tilt, sheen, or lift
- transitions between states that preserve orientation
- meaningful loading or focus states

For React implementations, prefer:

- parent/child reveal systems instead of isolated per-element animation
- one strong hero choreography pattern plus one supporting stagger pattern
- shared layout transitions for cards that expand or reorganize
- reduced-motion fallbacks that keep the hierarchy readable

Avoid:

- random bouncing
- excessive blur-on-everything
- long cinematic delays that slow the app down

### 6. Production Constraints

Always keep these grounded:

- responsive on desktop and mobile
- accessible contrast for core text and controls
- reduced-motion fallback
- acceptable performance on mid-tier hardware
- touch interactions that do not depend on hover

The interface can feel luxurious without becoming fragile.

## Output Contract

When using this skill, aim to produce:

1. A one-paragraph visual direction
2. A compact palette and typography plan
3. A depth and motion system
4. Concrete component or page implementation
5. A quick pass for responsiveness and accessibility

If the task is a redesign, explicitly define:

- background treatment
- hero composition
- card language
- button language
- motion language

## Design Heuristics

Push toward:

- asymmetry with balance
- strong negative space
- clear focal planes
- one or two memorable visual gestures
- components that feel collected into a world

Avoid:

- flat white SaaS sameness
- purple-on-white defaults
- generic Tailwind dashboard blocks
- overusing glassmorphism without hierarchy
- fake 3D that breaks readability

## Implementation Tips

For web UI:

- use CSS variables for color, depth, and glow tokens
- create reusable surface classes for raised, inset, and floating layers
- use transform-style, perspective, and translateZ sparingly
- prefer gradients, masks, and pseudo-elements for lighting
- use animation timing that feels weighted, not snappy by default

When implementing in React:

- let Motion handle choreography, layout continuity, and gestures
- keep long-lived ambient loops subtle and low-frequency
- avoid moving every surface at once
- prefer one or two memorable motion ideas over animation saturation

For React:

- keep component structure understandable
- use motion where it supports hierarchy
- prefer a few strong animated elements over everything moving at once

## If the User Says “Inspired By Dribbble”

Interpret that as:

- visually ambitious
- high-fidelity
- emotionally designed
- polished in composition and motion

Do not interpret it as:

- impractical
- inaccessible
- image-first with broken UX

## Success Test

The design passes if it feels like:

- a real product with a point of view
- something a founder would proudly show
- something a frontend engineer can actually maintain

It fails if it feels like:

- a generic starter template with extra blur
- a concept shot that ignores usability
- a pile of trendy effects with no hierarchy
